//! The one reviewed design stack in Mirket's existing private Podman engine.
use crate::{package_mcp::LaunchSpec, paths::Paths, podman::Engine, util};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    net::TcpListener,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const REGISTRY: &str = include_str!("../../registry/design-stack.json");
const OWNER_LABEL: &str = "org.mirket.design-owner";
const BUILD_LABEL: &str = "org.mirket.design-build";
const ALIASES: [&str; 12] = [
    "penpot-postgres",
    "penpot-valkey",
    "penpot-backend",
    "penpot-exporter",
    "penpot-mcp",
    "penpot-frontend",
    "browserless",
    "kroki-excalidraw",
    "kroki",
    "imgproxy",
    "comfyui",
    "design-tools",
];
const VOLUMES: [&str; 3] = ["db", "cache", "assets"];
const PUBLISHED: [(&str, &str, u16); 7] = [
    ("browserless", "browserless", 3000),
    ("penpot_frontend", "penpot-frontend", 8080),
    ("penpot_mcp", "penpot-mcp", 4401),
    ("penpot_plugin", "penpot-mcp", 4400),
    ("kroki", "kroki", 8000),
    ("imgproxy", "imgproxy", 8080),
    ("comfyui", "comfyui", 8188),
];
const REMOTE: [&str; 10] = [
    "penpot-postgres",
    "penpot-valkey",
    "penpot-backend",
    "penpot-exporter",
    "penpot-frontend",
    "browserless",
    "kroki-excalidraw",
    "kroki",
    "imgproxy",
    "node-build",
];
const BUILD_TIMEOUT: Duration = Duration::from_secs(1800);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolBudgets {
    tools: BTreeMap<String, u64>,
}

pub(super) fn tool_budgets() -> Result<(Duration, BTreeMap<String, Duration>)> {
    let budgets: ToolBudgets =
        serde_json::from_str(include_str!("../../runtime/design/tools/tool-budgets.json"))?;
    let policy: BTreeMap<String, Value> =
        serde_json::from_str(include_str!("../../runtime/design/tools/tool-policy.json"))?;
    ensure!(
        budgets.tools.keys().eq(policy.keys()),
        "design tool budgets do not cover the owned policy exactly"
    );
    let mut tools = BTreeMap::new();
    let mut maximum = Duration::ZERO;
    for (name, millis) in budgets.tools {
        ensure!(
            millis > 0 && millis <= 2 * 60 * 60 * 1000,
            "invalid design tool budget: {name}"
        );
        let duration = Duration::from_millis(millis);
        maximum = maximum.max(duration);
        tools.insert(name, duration);
    }
    Ok((maximum, tools))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    owner: String,
    registry_sha256: String,
    images: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema_version: u32,
    owner: String,
    engine_owner: String,
    registry_sha256: String,
    images: BTreeMap<String, String>,
    browserless_token: String,
    published_ports: BTreeMap<String, u16>,
    database_password: String,
    service_secret: String,
    penpot_email: String,
    penpot_password: String,
    imgproxy_key: String,
    imgproxy_salt: String,
}

#[derive(Debug)]
enum StartEffect {
    CreatedNetwork(String),
    CreatedContainer { alias: String, name: String },
    StartedStoppedContainer { alias: String, name: String },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SessionIdentity {
    schema_version: u32,
    file_id: String,
    team_id: String,
}

fn root(paths: &Paths) -> PathBuf {
    paths.root.join("packages/data/design")
}
fn receipt_path(paths: &Paths) -> PathBuf {
    root(paths)
        .join("receipts")
        .join(format!("{}.json", registry_hash()))
}
fn state_path(paths: &Paths) -> PathBuf {
    root(paths).join("state.json")
}
fn owner(paths: &Paths) -> String {
    format!(
        "mirket-design-{}",
        &util::sha256(paths.root.to_string_lossy().as_bytes())[..16]
    )
}
fn registry_hash() -> String {
    let mut bytes = REGISTRY.as_bytes().to_vec();
    for kind in ["penpot-mcp", "comfyui", "design-tools"] {
        bytes.extend_from_slice(recipe_id(&build_files(kind)).as_bytes());
    }
    util::sha256(&bytes)
}
fn name(owner: &str, alias: &str) -> String {
    format!("{owner}-{alias}")
}

pub(super) fn definition() -> Result<Value> {
    let value: Value = serde_json::from_str(REGISTRY)?;
    ensure!(
        value["schema_version"] == 1
            && value["id"] == "design"
            && value["workspace"] == "/workspace",
        "invalid design registry identity"
    );
    let actual: BTreeSet<_> = value["tools"]
        .as_array()
        .context("design tools missing")?
        .iter()
        .map(|v| v["id"].as_str().unwrap_or(""))
        .collect();
    let expected: BTreeSet<_> = [
        "style-dictionary",
        "satori",
        "browserless",
        "penpot",
        "kroki",
        "hyperframes",
        "typst",
        "imgproxy",
        "svgo",
        "comfyui",
        "odiff",
    ]
    .into_iter()
    .collect();
    ensure!(
        actual == expected && value["tools"].as_array().unwrap().len() == 11,
        "design registry requires exactly eleven domains"
    );
    for alias in REMOTE.iter().chain(["python-build"].iter()) {
        let image = value["images"][alias]
            .as_str()
            .context("missing reviewed image")?;
        let (repo, digest) = image
            .split_once("@sha256:")
            .context("image requires immutable SHA-256 manifest")?;
        ensure!(
            !repo.is_empty()
                && digest.len() == 64
                && digest
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "invalid reviewed image digest"
        );
    }
    Ok(value)
}

pub(super) fn workspace(paths: &Paths) -> Result<PathBuf> {
    let path = paths.root.join("execution/podman/staging/design/workspace");
    util::reject_symlink_components(&path)?;
    ensure!(
        path.is_dir(),
        "design workspace is absent; prepare the design stack"
    );
    Ok(path.canonicalize()?)
}
fn ensure_workspace(paths: &Paths) -> Result<PathBuf> {
    let path = paths.root.join("execution/podman/staging/design/workspace");
    util::reject_symlink_components(&path)?;
    util::ensure_directory(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(path.canonicalize()?)
}

fn engine(paths: &Paths) -> Result<Engine> {
    let engine = Engine::resolve(paths)?;
    let info = engine.ready()?;
    ensure!(
        info["host"]["arch"] == "arm64",
        "design stack requires the verified native Linux ARM64 Podman environment"
    );
    Ok(engine)
}
fn run(engine: &Engine, args: Vec<String>, timeout: Duration) -> Result<String> {
    engine.run(&args, timeout)
}
fn inspect(engine: &Engine, kind: &str, name: &str) -> Result<Value> {
    serde_json::from_str(&run(
        engine,
        vec![
            kind.into(),
            "inspect".into(),
            name.into(),
            "--format".into(),
            "{{json .}}".into(),
        ],
        Duration::from_secs(30),
    )?)
    .context("invalid Podman inspection")
}
fn normalize_image_id(value: &str) -> Result<String> {
    let digest = value.strip_prefix("sha256:").unwrap_or(value);
    ensure!(
        digest.len() == 64
            && digest
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "invalid OCI image ID"
    );
    Ok(format!("sha256:{digest}"))
}
fn image_id(engine: &Engine, image: &str) -> Result<String> {
    let v = inspect(engine, "image", image)?;
    ensure!(
        v["Architecture"] == "arm64" && v["Os"] == "linux",
        "image is not native Linux ARM64: {image}"
    );
    let id = normalize_image_id(v["Id"].as_str().context("image ID missing")?)?;
    if let Some((_, digest)) = image.split_once("@sha256:") {
        let pinned = format!("sha256:{digest}");
        let listed = v["RepoDigests"]
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(image)));
        ensure!(
            listed || v["Digest"] == pinned,
            "image manifest digest drift: {image}"
        );
    }
    Ok(id)
}
fn existing(engine: &Engine, kind: &str, name: &str, own: &str) -> Result<Option<Value>> {
    let listing = match kind {
        "container" => vec!["container", "ls", "--all", "--format", "{{.Names}}"],
        "network" => vec!["network", "ls", "--format", "{{.Name}}"],
        "volume" => vec!["volume", "ls", "--format", "{{.Name}}"],
        _ => bail!("invalid resource kind"),
    }
    .into_iter()
    .map(str::to_owned)
    .collect();
    if !run(engine, listing, Duration::from_secs(30))?
        .lines()
        .any(|line| line == name)
    {
        return Ok(None);
    }
    let value = inspect(engine, kind, name)?;
    let labels = match kind {
        "container" => &value["Config"]["Labels"],
        "network" => &value["labels"],
        _ => &value["Labels"],
    };
    ensure!(
        labels[OWNER_LABEL] == own,
        "refusing unrelated Podman {kind}: {name}"
    );
    Ok(Some(value))
}
fn validate_resource(kind: &str, value: &Value) -> Result<()> {
    if kind == "network" {
        ensure!(
            value["internal"] == true && value["driver"] == "bridge",
            "owned design network configuration drift"
        );
    } else {
        ensure!(
            value["Driver"] == "local"
                && value["Options"]
                    .as_object()
                    .is_none_or(|options| options.is_empty()),
            "owned design volume configuration drift"
        );
    }
    Ok(())
}
fn rootless_keep_id_map(value: &Value, user: u64) -> bool {
    let Some(entries) = value.as_array() else {
        return false;
    };
    if entries.len() != 3 {
        return false;
    }
    let parse = |entry: &Value| -> Option<(u64, u64, u64)> {
        let mut fields = entry.as_str()?.split(':');
        let id = fields.next()?.parse().ok()?;
        let host = fields.next()?.parse().ok()?;
        let count = fields.next()?.parse().ok()?;
        if fields.next().is_some() {
            return None;
        }
        Some((id, host, count))
    };
    let (Some(head), Some(kept), Some(tail)) =
        (parse(&entries[0]), parse(&entries[1]), parse(&entries[2]))
    else {
        return false;
    };
    head == (0, 1, user)
        && kept == (user, 0, 1)
        && tail.0 == user + 1
        && tail.1 == user + 1
        && tail.2 >= 65_536
        && tail.0.checked_add(tail.2).is_some()
}
fn rootless_keep_id_policy(host: &Value, user: u64) -> bool {
    host["UsernsMode"] == "private"
        && host["IDMappings"]["UidMap"] == host["IDMappings"]["GidMap"]
        && rootless_keep_id_map(&host["IDMappings"]["UidMap"], user)
}
fn validate_container(value: &Value, state: &State, alias: &str, work: &Path) -> Result<()> {
    let config = &value["Config"];
    let host = &value["HostConfig"];
    let n = name(&state.owner, alias);
    ensure!(
        config["Labels"][OWNER_LABEL] == state.owner
            && config["Labels"]["org.mirket.design-alias"] == alias,
        "container identity drift: {alias}"
    );
    ensure!(
        value["Image"]
            .as_str()
            .and_then(|id| normalize_image_id(id).ok())
            .is_some_and(|id| id == state.images[alias]),
        "container image drift: {alias}"
    );
    let user_id = uid(alias);
    let user = user_id.to_string();
    ensure!(
        config["User"] == format!("{user}:{user}") && rootless_keep_id_policy(host, user_id.into()),
        "container user mapping drift: {alias}"
    );
    ensure!(
        host["ReadonlyRootfs"] == true
            && host["Privileged"] == false
            && host["SecurityOpt"] == json!(["no-new-privileges"]),
        "container root/security policy drift: {alias}"
    );
    let dropped: BTreeSet<_> = host["CapDrop"]
        .as_array()
        .context("container capability metadata missing")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    ensure!(
        dropped.contains("CAP_CHOWN") && dropped.contains("CAP_SYS_CHROOT") && dropped.len() >= 10,
        "container capabilities drift: {alias}"
    );
    let (memory, cpus, pids) = limits(alias);
    let memory_bytes = memory.trim_end_matches('m').parse::<u64>()? * 1024 * 1024;
    let nano_cpus = (cpus.parse::<f64>()? * 1_000_000_000.0) as u64;
    ensure!(
        host["Memory"] == memory_bytes
            && host["MemorySwap"] == memory_bytes
            && host["NanoCpus"] == nano_cpus
            && host["PidsLimit"] == pids.parse::<u64>()?,
        "container resource limits drift: {alias}"
    );
    ensure!(
        host["LogConfig"]["Type"] == "k8s-file" && host["LogConfig"]["Size"] == "4MB",
        "container log bounds drift: {alias}"
    );
    ensure!(
        host["RestartPolicy"]["Name"] == "no" && host["PublishAllPorts"] == false,
        "container lifecycle/port policy drift: {alias}"
    );
    ensure!(
        if alias == "design-tools" {
            host["Init"] == true
        } else {
            host["Init"].is_null() || host["Init"] == false
        },
        "container process reaping policy drift: {alias}"
    );
    let mut expected = serde_json::Map::new();
    for mapping in published_mappings(state, alias) {
        let fields: Vec<_> = mapping.split(':').collect();
        expected.insert(
            format!("{}/tcp", fields[2]),
            json!([{"HostIp":fields[0],"HostPort":fields[1]}]),
        );
    }
    let actual_ports = &host["PortBindings"];
    ensure!(
        (expected.is_empty()
            && (actual_ports.is_null() || actual_ports.as_object().is_some_and(|v| v.is_empty())))
            || *actual_ports == Value::Object(expected),
        "container published ports drift: {alias}"
    );
    let networks = value["NetworkSettings"]["Networks"]
        .as_object()
        .context("container network metadata missing")?;
    let network = name(&state.owner, "network");
    ensure!(
        networks.len() == 1
            && networks.get(&network).is_some_and(|v| v["Aliases"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x.as_str() == Some(alias)))),
        "container network alias drift: {alias}"
    );
    let mounts = value["Mounts"]
        .as_array()
        .context("container mount metadata missing")?;
    let expected_volumes = volumes(state, alias);
    ensure!(
        mounts.len() == expected_volumes.len() + 1,
        "container mount count drift: {alias}"
    );
    let workspace_mount = mounts
        .iter()
        .find(|v| v["Destination"] == "/workspace")
        .context("shared design workspace is not mounted")?;
    ensure!(
        workspace_mount["Source"] == work.to_string_lossy().as_ref()
            && workspace_mount["RW"] == true
            && workspace_mount["Type"] == "bind",
        "container workspace mount drift: {alias}"
    );
    for (volume, target, writable) in expected_volumes {
        ensure!(
            mounts.iter().any(|v| v["Type"] == "volume"
                && v["Name"] == volume
                && v["Destination"] == target
                && v["RW"] == writable),
            "container volume drift: {alias}"
        );
    }
    ensure!(
        value["Name"]
            .as_str()
            .is_some_and(|actual| actual.trim_start_matches('/') == n),
        "container name drift: {alias}"
    );
    Ok(())
}
fn secret(bytes: usize) -> Result<String> {
    let mut data = vec![0u8; bytes];
    fs::File::open("/dev/urandom")?.read_exact(&mut data)?;
    Ok(data.iter().map(|b| format!("{b:02x}")).collect())
}
fn valid_secret(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn saved_state_any(paths: &Paths, engine: &Engine) -> Result<State> {
    let state: State = serde_json::from_slice(&util::read_bounded(&state_path(paths), 32768)?)?;
    ensure!(
        state.schema_version == 1
            && state.owner == owner(paths)
            && state.engine_owner == engine.owner(),
        "design state belongs to another environment"
    );
    for value in [
        &state.browserless_token,
        &state.database_password,
        &state.service_secret,
        &state.imgproxy_key,
        &state.imgproxy_salt,
    ] {
        ensure!(valid_secret(value), "invalid private design secret");
    }
    ensure!(
        state
            .penpot_password
            .strip_suffix("Aa1!")
            .is_some_and(valid_secret),
        "invalid private Penpot password"
    );
    validate_published_ports(&state.published_ports)?;
    Ok(state)
}
fn saved_state(paths: &Paths, engine: &Engine) -> Result<State> {
    let state = saved_state_any(paths, engine)?;
    ensure!(
        state.registry_sha256 == registry_hash(),
        "design state belongs to another registry generation; stop the owned prior generation before starting the new one"
    );
    Ok(state)
}
fn save_private(path: &Path, value: &[u8]) -> Result<()> {
    util::reject_symlink_components(path)?;
    util::ensure_directory(path.parent().context("private parent")?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path.parent().unwrap(), fs::Permissions::from_mode(0o700))?;
    }
    util::atomic_write_with_mode(path, value, Some(0o600))
}
fn recipe_id(files: &[(&str, &[u8])]) -> String {
    let mut bytes = Vec::new();
    for (name, contents) in files {
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(contents);
    }
    util::sha256(&bytes)
}
fn build_recipe(registry: &Value, kind: &str) -> Result<String> {
    let mut identity = recipe_id(&build_files(kind));
    if kind != "design-tools" {
        identity.push_str(
            registry["builds"][kind]["sha256"]
                .as_str()
                .context("source hash")?,
        );
    }
    Ok(util::sha256(identity.as_bytes()))
}
fn build_files(kind: &str) -> Vec<(&'static str, &'static [u8])> {
    match kind {
        "penpot-mcp" => vec![(
            "Dockerfile",
            include_bytes!("../../runtime/design/penpot-mcp/Dockerfile"),
        )],
        "comfyui" => vec![
            (
                "Dockerfile",
                include_bytes!("../../runtime/design/comfyui/Dockerfile"),
            ),
            (
                "requirements.lock",
                include_bytes!("../../runtime/design/comfyui/requirements.lock"),
            ),
            (
                "exclude-3d.py",
                include_bytes!("../../runtime/design/comfyui/exclude-3d.py"),
            ),
        ],
        "design-tools" => vec![
            (
                "Dockerfile",
                include_bytes!("../../runtime/design/tools/Dockerfile"),
            ),
            (
                "tools/package.json",
                include_bytes!("../../runtime/design/tools/package.json"),
            ),
            (
                "tools/package-lock.json",
                include_bytes!("../../runtime/design/tools/package-lock.json"),
            ),
            (
                "tools/server.mjs",
                include_bytes!("../../runtime/design/tools/server.mjs"),
            ),
            (
                "tools/tool-policy.json",
                include_bytes!("../../runtime/design/tools/tool-policy.json"),
            ),
            (
                "tools/tool-budgets.json",
                include_bytes!("../../runtime/design/tools/tool-budgets.json"),
            ),
            (
                "tools/runtime-guard.c",
                include_bytes!("../../runtime/design/tools/runtime-guard.c"),
            ),
            (
                "tools/paths.mjs",
                include_bytes!("../../runtime/design/tools/paths.mjs"),
            ),
            (
                "tools/tokens.mjs",
                include_bytes!("../../runtime/design/tools/tokens.mjs"),
            ),
            (
                "tools/operations.mjs",
                include_bytes!("../../runtime/design/tools/operations.mjs"),
            ),
            (
                "tools/request.mjs",
                include_bytes!("../../runtime/design/tools/request.mjs"),
            ),
            (
                "tools/penpot-export.mjs",
                include_bytes!("../../runtime/design/tools/penpot-export.mjs"),
            ),
            (
                "tools/penpot-tokens.mjs",
                include_bytes!("../../runtime/design/tools/penpot-tokens.mjs"),
            ),
            (
                "tools/workspace-server.mjs",
                include_bytes!("../../runtime/design/tools/workspace-server.mjs"),
            ),
            (
                "tools/NOTICE.md",
                include_bytes!("../../runtime/design/tools/NOTICE.md"),
            ),
            (
                "penpot-session.mjs",
                include_bytes!("../../runtime/design/penpot-session.mjs"),
            ),
        ],
        _ => unreachable!(),
    }
}
fn archive(paths: &Paths, registry: &Value, kind: &str, stage: &Path) -> Result<()> {
    let info = &registry["builds"][kind];
    let url = info["source_archive"]
        .as_str()
        .context("source archive URL")?;
    let hash = info["sha256"].as_str().context("source archive hash")?;
    let path = stage.join(if kind == "comfyui" {
        "comfyui-source.tar.gz"
    } else {
        "penpot-source.tar.gz"
    });
    if path.exists() && util::sha256(&util::read_bounded(&path, 256 * 1024 * 1024)?) == hash {
        return Ok(());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;
    let response = client.get(url).send()?.error_for_status()?;
    let mut bytes = Vec::new();
    response
        .take(256 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 256 * 1024 * 1024 && util::sha256(&bytes) == hash,
        "{kind} source archive digest mismatch"
    );
    let _ = paths;
    util::atomic_write(&path, &bytes)
}
fn prepare_build(paths: &Paths, engine: &Engine, registry: &Value, kind: &str) -> Result<String> {
    let files = build_files(kind);
    let recipe = build_recipe(registry, kind)?;
    let tag = format!("localhost/mirket-design-{kind}:{}", &recipe[..16]);
    if let Ok(id) = image_id(engine, &tag) {
        let inspected = inspect(engine, "image", &tag)?;
        ensure!(
            inspected["Labels"][BUILD_LABEL] == recipe,
            "reviewed design build tag has untrusted recipe label: {kind}"
        );
        return Ok(id);
    }
    let stage_parent = engine.staging.join("design/build");
    util::reject_symlink_components(&stage_parent)?;
    util::ensure_directory(&stage_parent)?;
    let stage_dir = tempfile::Builder::new()
        .prefix(&format!("{kind}-"))
        .tempdir_in(&stage_parent)?;
    let stage = stage_dir.path();
    for (relative, contents) in files {
        let destination = stage.join(relative);
        util::ensure_directory(destination.parent().context("build file parent")?)?;
        util::atomic_write(&destination, contents)?;
    }
    if kind != "design-tools" {
        archive(paths, registry, kind, stage)?;
    }
    run(
        engine,
        vec![
            "build".into(),
            "--jobs=1".into(),
            "--memory=2g".into(),
            "--memory-swap=2g".into(),
            "--ulimit=nproc=512:512".into(),
            "--pull=never".into(),
            "--platform=linux/arm64".into(),
            "--label".into(),
            format!("{BUILD_LABEL}={recipe}"),
            "--tag".into(),
            tag.clone(),
            stage.to_string_lossy().into_owned(),
        ],
        BUILD_TIMEOUT,
    )?;
    let id = image_id(engine, &tag)?;
    ensure!(
        inspect(engine, "image", &tag)?["Labels"][BUILD_LABEL] == recipe,
        "reviewed design build recipe label missing: {kind}"
    );
    Ok(id)
}

pub(super) fn prepare(paths: &Paths) -> Result<()> {
    let registry = definition()?;
    let engine = engine(paths)?;
    ensure_workspace(paths)?;
    let dir = root(paths);
    util::ensure_directory(&dir)?;
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    let mut images = BTreeMap::new();
    for alias in REMOTE.iter().chain(["python-build"].iter()) {
        let reference = registry["images"][alias]
            .as_str()
            .context("missing image")?;
        let id = match image_id(&engine, reference) {
            Ok(id) => id,
            Err(_) => {
                run(
                    &engine,
                    vec![
                        "pull".into(),
                        "--quiet".into(),
                        "--platform=linux/arm64".into(),
                        reference.into(),
                    ],
                    Duration::from_secs(900),
                )?;
                image_id(&engine, reference)?
            }
        };
        images.insert((*alias).to_owned(), id);
    }
    for alias in ["penpot-mcp", "comfyui", "design-tools"] {
        images.insert(
            alias.into(),
            prepare_build(paths, &engine, &registry, alias)?,
        );
    }
    let receipt = Receipt {
        schema_version: 1,
        owner: owner(paths),
        registry_sha256: registry_hash(),
        images,
    };
    let path = receipt_path(paths);
    if path.exists() {
        let existing = util::read_bounded(&path, 32768)?;
        ensure!(
            existing == serde_json::to_vec_pretty(&receipt)?,
            "immutable design generation receipt drift"
        );
    } else {
        save_private(&path, &serde_json::to_vec_pretty(&receipt)?)?;
    }
    Ok(())
}

fn local_receipt(paths: &Paths) -> Result<Receipt> {
    let value: Receipt = serde_json::from_slice(
        &util::read_bounded(&receipt_path(paths), 32768)
            .context("design stack is not installed; run mirket package install design")?,
    )?;
    ensure!(
        value.schema_version == 1
            && value.owner == owner(paths)
            && value.registry_sha256 == registry_hash(),
        "design install receipt belongs to another generation"
    );
    for alias in ALIASES {
        let expected = value
            .images
            .get(alias)
            .context("incomplete design image receipt")?;
        ensure!(
            expected.starts_with("sha256:")
                && expected.len() == 71
                && expected[7..]
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "invalid saved design image ID"
        );
    }
    Ok(value)
}
fn receipt(paths: &Paths, engine: &Engine) -> Result<Receipt> {
    let value = local_receipt(paths)?;
    let registry = definition()?;
    for alias in ALIASES {
        let expected = value
            .images
            .get(alias)
            .context("incomplete design image receipt")?;
        let image = if ["penpot-mcp", "comfyui", "design-tools"].contains(&alias) {
            expected.as_str()
        } else {
            registry["images"][alias]
                .as_str()
                .context("image reference")?
        };
        ensure!(
            image_id(engine, image)? == *expected,
            "design image drift: {alias}"
        );
        if ["penpot-mcp", "comfyui", "design-tools"].contains(&alias) {
            ensure!(
                inspect(engine, "image", image)?["Labels"][BUILD_LABEL]
                    == build_recipe(&registry, alias)?,
                "design build recipe drift: {alias}"
            );
        }
    }
    Ok(value)
}
pub(super) fn verify(paths: &Paths) -> Result<Value> {
    let receipt = local_receipt(paths)?;
    Ok(
        json!({"installed":true,"package":"design","registry_sha256":receipt.registry_sha256,"images":receipt.images,"workspace":workspace(paths)?}),
    )
}

fn new_state(paths: &Paths, engine: &Engine, installed: &Receipt) -> Result<State> {
    if state_path(paths).exists() {
        let mut prior = saved_state_any(paths, engine)?;
        if prior.registry_sha256 != registry_hash() || prior.images != installed.images {
            for alias in ALIASES {
                ensure!(
                    existing(
                        engine,
                        "container",
                        &name(&prior.owner, alias),
                        &prior.owner
                    )?
                    .is_none(),
                    "stop the owned prior design generation before upgrading"
                );
            }
            prior.registry_sha256 = registry_hash();
            prior.images = installed.images.clone();
            save_private(&state_path(paths), &serde_json::to_vec_pretty(&prior)?)?;
        }
        return Ok(prior);
    }
    let state = State {
        schema_version: 1,
        owner: owner(paths),
        engine_owner: engine.owner(),
        registry_sha256: registry_hash(),
        images: installed.images.clone(),
        browserless_token: secret(32)?,
        published_ports: allocate_published_ports()?,
        database_password: secret(32)?,
        service_secret: secret(32)?,
        penpot_email: format!("design-{}@mirket.invalid", &owner(paths)[14..]),
        // Penpot requires lowercase, uppercase, a digit and a special character.
        penpot_password: format!("{}Aa1!", secret(32)?),
        imgproxy_key: secret(32)?,
        imgproxy_salt: secret(32)?,
    };
    save_private(&state_path(paths), &serde_json::to_vec_pretty(&state)?)?;
    Ok(state)
}
fn env_for(state: &State, alias: &str) -> BTreeMap<String, String> {
    let flags = "disable-email-verification disable-secure-session-cookies enable-registration enable-login-with-password enable-mcp disable-telemetry enable-air-gapped-conf disable-google-fonts-provider disable-dashboard-templates-section";
    let mut env = BTreeMap::new();
    let mut put = |key: &str, value: String| {
        env.insert(key.to_owned(), value);
    };
    match alias {
        "penpot-postgres" => {
            put("POSTGRES_INITDB_ARGS", "--data-checksums".into());
            put("POSTGRES_DB", "penpot".into());
            put("POSTGRES_USER", "penpot".into());
            put("POSTGRES_PASSWORD", state.database_password.clone());
        }
        "penpot-backend" => {
            put("PENPOT_FLAGS", flags.into());
            put("PENPOT_PUBLIC_URI", "http://penpot-frontend:8080".into());
            put("PENPOT_SECRET_KEY", state.service_secret.clone());
            put(
                "PENPOT_DATABASE_URI",
                "postgresql://penpot-postgres/penpot".into(),
            );
            put("PENPOT_DATABASE_USERNAME", "penpot".into());
            put("PENPOT_DATABASE_PASSWORD", state.database_password.clone());
            put("PENPOT_DATABASE_MIN_POOL_SIZE", "4".into());
            put("PENPOT_DATABASE_MAX_POOL_SIZE", "16".into());
            put("PENPOT_REDIS_URI", "redis://penpot-valkey/0".into());
            put("PENPOT_TELEMETRY_ENABLED", "false".into());
            put("PENPOT_OBJECTS_STORAGE_BACKEND", "fs".into());
            put(
                "PENPOT_OBJECTS_STORAGE_FS_DIRECTORY",
                "/opt/data/assets".into(),
            );
            put(
                "JAVA_OPTS",
                "-Xms128m -Xmx768m -XX:TieredStopAtLevel=1".into(),
            );
        }
        "penpot-exporter" => {
            put("PENPOT_FLAGS", flags.into());
            put("PENPOT_PUBLIC_URI", "http://penpot-frontend:8080".into());
            put("PENPOT_INTERNAL_URI", "http://penpot-frontend:8080".into());
            put("PENPOT_SECRET_KEY", state.service_secret.clone());
            put("PENPOT_REDIS_URI", "redis://penpot-valkey/0".into());
            put("PENPOT_BROWSER_POOL_MIN", "0".into());
            put("PENPOT_BROWSER_POOL_MAX", "1".into());
        }
        "penpot-frontend" => {
            put("PENPOT_FLAGS", flags.into());
            put("PENPOT_PUBLIC_URI", "http://penpot-frontend:8080".into());
            put("PENPOT_BACKEND_URI", "http://penpot-backend:6060".into());
            put("PENPOT_EXPORTER_URI", "http://penpot-exporter:6061".into());
            put("PENPOT_MCP_URI", "http://penpot-mcp:4401".into());
            put("PENPOT_MCP_URI_WS", "http://penpot-mcp:4402".into());
        }
        "browserless" => {
            put("TOKEN", state.browserless_token.clone());
            // The pinned Browserless queue treats -1 as an immediate timer.
            // Zero disables that timer; callers own and close their sessions.
            put("TIMEOUT", "0".into());
            put("DEBUG", "browserless*:error".into());
            put("CONCURRENT", "2".into());
            put("QUEUED", "10".into());
            put("HOME", "/tmp".into());
            put("XDG_CACHE_HOME", "/tmp/.cache".into());
        }
        "kroki" => {
            put("KROKI_EXCALIDRAW_HOST", "kroki-excalidraw".into());
            put("KROKI_EXCALIDRAW_PORT", "8004".into());
        }
        "imgproxy" => {
            put("IMGPROXY_BIND", "0.0.0.0:8080".into());
            put("IMGPROXY_LOCAL_FILESYSTEM_ROOT", "/workspace".into());
            put("IMGPROXY_ALLOWED_SOURCES", "local://".into());
            put("IMGPROXY_KEY", state.imgproxy_key.clone());
            put("IMGPROXY_SALT", state.imgproxy_salt.clone());
        }
        "design-tools" => {
            put("MIRKET_BROWSERLESS_TOKEN", state.browserless_token.clone());
            put("MIRKET_PENPOT_EMAIL", state.penpot_email.clone());
            put("MIRKET_PENPOT_PASSWORD", state.penpot_password.clone());
            put("MIRKET_IMGPROXY_KEY", state.imgproxy_key.clone());
            put("MIRKET_IMGPROXY_SALT", state.imgproxy_salt.clone());
        }
        _ => {}
    }
    env
}
fn uid(alias: &str) -> u32 {
    match alias {
        "penpot-postgres" | "penpot-valkey" | "browserless" | "imgproxy" => 999,
        "penpot-backend" | "penpot-exporter" | "penpot-frontend" | "kroki" | "kroki-excalidraw" => {
            1001
        }
        _ => 1000,
    }
}
fn limits(alias: &str) -> (&'static str, &'static str, &'static str) {
    match alias {
        "penpot-backend" | "penpot-exporter" => ("1400m", "1.5", "256"),
        "browserless" => ("2000m", "1.5", "512"),
        // Multiple desktop MCP connections share this renderer container.
        // Leave headroom for a real render alongside their retained sessions.
        "design-tools" => ("3072m", "2", "512"),
        "comfyui" => ("3200m", "2", "256"),
        "kroki" => ("600m", "1", "128"),
        _ => ("350m", "1", "128"),
    }
}
fn validate_published_ports(ports: &BTreeMap<String, u16>) -> Result<()> {
    ensure!(
        ports.len() == PUBLISHED.len()
            && PUBLISHED.iter().all(|(key, _, _)| ports.contains_key(*key))
            && ports.values().all(|port| *port >= 1024)
            && ports.values().copied().collect::<BTreeSet<_>>().len() == ports.len(),
        "invalid design host port assignments"
    );
    Ok(())
}
fn reserve_available_port(excluded: &BTreeSet<u16>) -> Result<TcpListener> {
    for _ in 0..32 {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        if port >= 1024 && !excluded.contains(&port) {
            return Ok(listener);
        }
    }
    bail!("could not reserve a distinct available loopback port")
}
fn allocate_published_ports() -> Result<BTreeMap<String, u16>> {
    let mut ports = BTreeMap::new();
    let mut reservations = Vec::new();
    for (key, _, _) in PUBLISHED {
        let listener = reserve_available_port(&ports.values().copied().collect())?;
        ports.insert(key.to_owned(), listener.local_addr()?.port());
        reservations.push(listener);
    }
    validate_published_ports(&ports)?;
    Ok(ports)
}
fn published_mappings(state: &State, alias: &str) -> Vec<String> {
    PUBLISHED
        .iter()
        .filter(|(_, service, _)| *service == alias)
        .map(|(key, _, internal)| format!("127.0.0.1:{}:{internal}", state.published_ports[*key]))
        .collect()
}
fn volumes(state: &State, alias: &str) -> Vec<(String, &'static str, bool)> {
    let n = |part: &str| name(&state.owner, part);
    match alias {
        "penpot-postgres" => vec![(n("db"), "/var/lib/postgresql/data", true)],
        "penpot-valkey" => vec![(n("cache"), "/data", true)],
        "penpot-backend" => vec![(n("assets"), "/opt/data/assets", true)],
        "penpot-frontend" => vec![(n("assets"), "/opt/data/assets", false)],
        _ => Vec::new(),
    }
}
fn env_file(paths: &Paths, state: &State, alias: &str) -> Result<PathBuf> {
    let path = root(paths).join(format!("{alias}.env"));
    let contents = env_for(state, alias)
        .into_iter()
        .map(|(key, value)| format!("{key}={value}\n"))
        .collect::<String>();
    save_private(&path, contents.as_bytes())?;
    Ok(path)
}
fn ensure_resources(engine: &Engine, state: &State, effects: &mut Vec<StartEffect>) -> Result<()> {
    let network = name(&state.owner, "network");
    if let Some(value) = existing(engine, "network", &network, &state.owner)? {
        validate_resource("network", &value)?;
    } else {
        // Record the prior absence before the remote mutation. A lost client
        // reply can still mean Podman created the owned network.
        effects.push(StartEffect::CreatedNetwork(network.clone()));
        run(
            engine,
            vec![
                "network".into(),
                "create".into(),
                "--internal".into(),
                "--label".into(),
                format!("{OWNER_LABEL}={}", state.owner),
                network.clone(),
            ],
            Duration::from_secs(30),
        )?;
    }
    for volume in VOLUMES {
        let n = name(&state.owner, volume);
        if let Some(value) = existing(engine, "volume", &n, &state.owner)? {
            validate_resource("volume", &value)?;
        } else {
            run(
                engine,
                vec![
                    "volume".into(),
                    "create".into(),
                    "--label".into(),
                    format!("{OWNER_LABEL}={}", state.owner),
                    n.clone(),
                ],
                Duration::from_secs(30),
            )?;
            // Named volumes are durable design data even if later startup
            // fails. Only explicit purge may remove them.
        }
    }
    Ok(())
}
fn preflight_host_port(alias: &str, mapping: &str) -> Result<TcpListener> {
    let (address, _) = mapping
        .rsplit_once(':')
        .context("invalid design port mapping")?;
    TcpListener::bind(address).with_context(|| {
        format!("{alias} requires free host address {address}; inspect its current listener before starting design")
    })
}

fn reserve_start_ports(
    ports: &mut BTreeMap<String, u16>,
    present: &BTreeSet<&str>,
    running: &BTreeSet<&str>,
) -> Result<(BTreeMap<String, TcpListener>, bool)> {
    let mut reservations = BTreeMap::new();
    let mut changed = false;
    for (key, alias, internal) in PUBLISHED {
        if running.contains(alias) {
            continue;
        }
        let mapping = format!("127.0.0.1:{}:{internal}", ports[key]);
        let listener = match preflight_host_port(alias, &mapping) {
            Ok(listener) => listener,
            Err(error) if present.contains(alias) => {
                return Err(error.context("an owned stopped container retains this binding; run mirket package stop design, then start design to assign an available port"));
            }
            Err(_) => {
                let listener = reserve_available_port(&ports.values().copied().collect())?;
                ports.insert(key.to_owned(), listener.local_addr()?.port());
                changed = true;
                listener
            }
        };
        reservations.insert(key.to_owned(), listener);
    }
    Ok((reservations, changed))
}

fn release_service_ports(reservations: &mut BTreeMap<String, TcpListener>, alias: &str) {
    for (key, service, _) in PUBLISHED {
        if service == alias {
            reservations.remove(key);
        }
    }
}

fn create_container(
    paths: &Paths,
    engine: &Engine,
    state: &State,
    alias: &str,
    work: &Path,
    effects: &mut Vec<StartEffect>,
) -> Result<()> {
    let container = name(&state.owner, alias);
    if let Some(value) = existing(engine, "container", &container, &state.owner)? {
        validate_container(&value, state, alias, work)?;
        if !running(&value) {
            // The container was stopped before this attempt. Record that fact
            // before the remote start so an ambiguous reply can be reconciled.
            effects.push(StartEffect::StartedStoppedContainer {
                alias: alias.to_owned(),
                name: container.clone(),
            });
            run(
                engine,
                vec!["container".into(), "start".into(), container],
                Duration::from_secs(60),
            )?;
        }
        return Ok(());
    }
    let (memory, cpus, pids) = limits(alias);
    let user = uid(alias).to_string();
    let image = state
        .images
        .get(alias)
        .context("missing installed image ID")?;
    let mut args = vec![
        "run".into(),
        "--detach".into(),
        "--pull=never".into(),
        "--name".into(),
        container.clone(),
        "--hostname".into(),
        alias.into(),
        "--network".into(),
        name(&state.owner, "network"),
        "--network-alias".into(),
        alias.into(),
        "--label".into(),
        format!("{OWNER_LABEL}={}", state.owner),
        "--label".into(),
        format!("org.mirket.design-alias={alias}"),
        "--userns".into(),
        format!("keep-id:uid={user},gid={user}"),
        "--user".into(),
        format!("{user}:{user}"),
        "--read-only".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--memory".into(),
        memory.into(),
        "--memory-swap".into(),
        memory.into(),
        "--cpus".into(),
        cpus.into(),
        "--pids-limit".into(),
        pids.into(),
        "--restart=no".into(),
        "--http-proxy=false".into(),
        "--log-driver=k8s-file".into(),
        "--log-opt=max-size=4m".into(),
        "--tmpfs".into(),
        "/tmp:rw,nosuid,nodev,size=256m,mode=1777".into(),
        "--tmpfs".into(),
        "/run:rw,nosuid,nodev,size=64m,mode=1777".into(),
        "--volume".into(),
        format!("{}:/workspace:rw", work.display()),
    ];
    if alias == "design-tools" {
        args.push("--init".into());
    }
    if alias == "penpot-frontend" {
        args.extend([
            "--tmpfs".into(),
            "/var/cache/nginx:rw,nosuid,nodev,size=64m,mode=1777".into(),
            "--tmpfs".into(),
            "/etc/nginx:rw,nosuid,nodev,size=32m,mode=1777,tmpcopyup".into(),
            "--tmpfs".into(),
            "/var/www/app/js:rw,nosuid,nodev,size=128m,mode=1777,tmpcopyup".into(),
        ]);
    }
    for (volume, target, writable) in volumes(state, alias) {
        args.extend([
            "--volume".into(),
            format!("{volume}:{target}:{}", if writable { "rw,U" } else { "ro" }),
        ]);
    }
    for mapping in published_mappings(state, alias) {
        args.extend(["--publish".into(), mapping]);
    }
    let env_path = env_file(paths, state, alias)?;
    args.extend(["--env-file".into(), env_path.to_string_lossy().into_owned()]);
    args.push(image.clone());
    if alias == "penpot-valkey" {
        args.extend([
            "valkey-server".into(),
            "--save".into(),
            "60".into(),
            "1".into(),
            "--appendonly".into(),
            "yes".into(),
            "--dir".into(),
            "/data".into(),
        ]);
    }
    effects.push(StartEffect::CreatedContainer {
        alias: alias.to_owned(),
        name: container,
    });
    let outcome = run(engine, args, Duration::from_secs(120));
    let _ = fs::remove_file(env_path);
    outcome.map(|_| ())
}
fn running(value: &Value) -> bool {
    value["State"]["Running"] == true
}
fn readiness_json(reader: impl Read, alias: &str) -> Result<Value> {
    let mut bytes = Vec::new();
    reader.take(65537).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 65536,
        "{alias} readiness response is oversized"
    );
    serde_json::from_slice(&bytes).context("invalid readiness JSON")
}
fn http_ready(alias: &str, state: &State) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let (url, require_json) = match alias {
        "browserless" => (
            format!(
                "http://127.0.0.1:{}/pressure?token={}",
                state.published_ports["browserless"], state.browserless_token
            ),
            true,
        ),
        "penpot-mcp" => (
            format!(
                "http://127.0.0.1:{}/manifest.json",
                state.published_ports["penpot_plugin"]
            ),
            true,
        ),
        "penpot-frontend" => (
            format!(
                "http://127.0.0.1:{}/api/main/methods/get-profile",
                state.published_ports["penpot_frontend"]
            ),
            true,
        ),
        "kroki" => (
            format!("http://127.0.0.1:{}/health", state.published_ports["kroki"]),
            false,
        ),
        "imgproxy" => (
            format!(
                "http://127.0.0.1:{}/health",
                state.published_ports["imgproxy"]
            ),
            false,
        ),
        "comfyui" => (
            format!(
                "http://127.0.0.1:{}/system_stats",
                state.published_ports["comfyui"]
            ),
            true,
        ),
        _ => return Ok(()),
    };
    let response = client
        .get(url)
        .header("Accept", "application/json")
        .send()?;
    ensure!(
        response.status().is_success(),
        "{alias} HTTP readiness failed ({})",
        response.status()
    );
    if alias == "penpot-mcp" {
        let endpoint = client
            .get(format!(
                "http://127.0.0.1:{}/mcp",
                state.published_ports["penpot_mcp"]
            ))
            .send()?;
        ensure!(
            endpoint.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED,
            "Penpot Streamable HTTP endpoint did not serve its native GET response ({})",
            endpoint.status()
        );
    }
    if require_json {
        let value = readiness_json(response, alias)?;
        if alias == "browserless" {
            ensure!(
                value["pressure"]["isAvailable"].is_boolean()
                    && value["pressure"]["maxConcurrent"]
                        .as_u64()
                        .is_some_and(|count| count > 0),
                "Browserless pressure response shape drift"
            );
        }
        if alias == "penpot-frontend" {
            ensure!(
                value["id"] == "00000000-0000-0000-0000-000000000000"
                    && value["fullname"] == "Anonymous User",
                "Penpot frontend/backend did not serve anonymous profile"
            );
        }
        if alias == "penpot-mcp" {
            ensure!(
                value["name"] == "Penpot MCP Plugin",
                "official Penpot plugin manifest identity drift"
            );
        }
        if alias == "comfyui" {
            ensure!(
                value["system"].is_object() || value["devices"].is_array(),
                "ComfyUI system_stats shape drift"
            );
        }
    }
    Ok(())
}
fn probe_container(engine: &Engine, state: &State, alias: &str) -> Result<()> {
    let n = name(&state.owner, alias);
    let cmd = match alias {
        "penpot-postgres" => Some(vec!["pg_isready", "-U", "penpot", "-d", "penpot"]),
        "penpot-valkey" => Some(vec!["valkey-cli", "ping"]),
        "design-tools" => Some(vec![
            "node",
            "-e",
            "fetch('http://127.0.0.1:9002/health').then(r=>process.exit(r.ok?0:1)).catch(()=>process.exit(1))",
        ]),
        _ => None,
    };
    if let Some(cmd) = cmd {
        let mut args = vec!["exec".to_owned(), n];
        args.extend(cmd.into_iter().map(str::to_owned));
        run(engine, args, Duration::from_secs(10))?;
    } else {
        let value = inspect(engine, "container", &n)?;
        ensure!(running(&value), "{alias} has exited");
    }
    http_ready(alias, state)?;
    Ok(())
}
fn startup_diagnostic(state: &State, value: &str) -> String {
    let mut redacted = value.to_owned();
    for secret in [
        &state.browserless_token,
        &state.database_password,
        &state.service_secret,
        &state.penpot_email,
        &state.penpot_password,
        &state.imgproxy_key,
        &state.imgproxy_salt,
    ] {
        if !secret.is_empty() {
            redacted = redacted.replace(secret, "[redacted]");
        }
    }
    redacted.chars().take(4096).collect()
}
fn startup_logs(engine: &Engine, state: &State, name: &str) -> String {
    let result = engine.container_logs(name);
    startup_diagnostic(
        state,
        &match result {
            Ok(logs) => logs,
            Err(error) => format!("log capture failed: {error:#}"),
        },
    )
}
fn await_ready(engine: &Engine, state: &State, alias: &str) -> Result<()> {
    let deadline =
        Instant::now() + Duration::from_secs(if alias == "design-tools" { 180 } else { 90 });
    let name = name(&state.owner, alias);
    loop {
        let problem = match probe_container(engine, state, alias) {
            Ok(()) => return Ok(()),
            Err(error) => startup_diagnostic(state, &format!("{error:#}")),
        };
        if let Ok(value) = inspect(engine, "container", &name) {
            let status = value["State"]["Status"].as_str().unwrap_or("");
            if ["exited", "dead", "stopped"].contains(&status) {
                bail!(
                    "{alias} exited before readiness (status {status}, exit code {}): {problem}; recent logs: {}",
                    value["State"]["ExitCode"],
                    startup_logs(engine, state, &name)
                );
            }
        }
        if Instant::now() >= deadline {
            bail!(
                "{alias} did not become ready: {problem}; recent logs: {}",
                startup_logs(engine, state, &name)
            );
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}
fn rollback(engine: &Engine, effects: &[StartEffect], state: &State, work: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    for effect in effects.iter().rev() {
        let (label, cleanup) = match effect {
            StartEffect::CreatedContainer { alias, name } => {
                let cleanup = (|| -> Result<()> {
                    if let Some(value) = existing(engine, "container", name, &state.owner)? {
                        validate_container(&value, state, alias, work)?;
                        run(
                            engine,
                            vec![
                                "container".into(),
                                "rm".into(),
                                "--force".into(),
                                name.clone(),
                            ],
                            Duration::from_secs(30),
                        )?;
                    }
                    Ok(())
                })();
                (name.as_str(), cleanup)
            }
            StartEffect::StartedStoppedContainer { alias, name } => {
                let cleanup = (|| -> Result<()> {
                    if let Some(value) = existing(engine, "container", name, &state.owner)? {
                        validate_container(&value, state, alias, work)?;
                        if running(&value) {
                            run(
                                engine,
                                vec![
                                    "container".into(),
                                    "stop".into(),
                                    "--time".into(),
                                    "3".into(),
                                    name.clone(),
                                ],
                                Duration::from_secs(15),
                            )?;
                        }
                    }
                    Ok(())
                })();
                (name.as_str(), cleanup)
            }
            StartEffect::CreatedNetwork(n) => {
                let cleanup = (|| -> Result<()> {
                    if let Some(value) = existing(engine, "network", n, &state.owner)? {
                        validate_resource("network", &value)?;
                        run(
                            engine,
                            vec!["network".into(), "rm".into(), n.clone()],
                            Duration::from_secs(30),
                        )?;
                    }
                    Ok(())
                })();
                (n.as_str(), cleanup)
            }
        };
        if let Err(error) = cleanup {
            failures.push(format!("{label}: {error:#}"));
        }
    }
    failures
}
pub(super) fn start(paths: &Paths) -> Result<Value> {
    let engine = engine(paths)?;
    let installed = receipt(paths, &engine)?;
    let work = ensure_workspace(paths)?;
    let dir = root(paths);
    util::ensure_directory(&dir)?;
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    let mut state = new_state(paths, &engine, &installed)?;
    ensure!(
        state.images == installed.images,
        "design state image receipt drift"
    );
    // Validate the complete prior topology before creating or restarting any
    // resource; a same-label resource with changed policy is never adopted.
    if let Some(value) = existing(
        &engine,
        "network",
        &name(&state.owner, "network"),
        &state.owner,
    )? {
        validate_resource("network", &value)?;
    }
    for volume in VOLUMES {
        if let Some(value) = existing(&engine, "volume", &name(&state.owner, volume), &state.owner)?
        {
            validate_resource("volume", &value)?;
        }
    }
    let mut running_owned = BTreeSet::new();
    let mut present_owned = BTreeSet::new();
    for alias in ALIASES {
        if let Some(value) = existing(
            &engine,
            "container",
            &name(&state.owner, alias),
            &state.owner,
        )? {
            validate_container(&value, &state, alias, &work)?;
            present_owned.insert(alias);
            if running(&value) {
                running_owned.insert(alias);
            }
        }
    }
    let (mut reservations, ports_changed) =
        reserve_start_ports(&mut state.published_ports, &present_owned, &running_owned)?;
    if ports_changed {
        save_private(&state_path(paths), &serde_json::to_vec_pretty(&state)?)?;
    }
    let mut effects = Vec::new();
    let result = (|| -> Result<()> {
        ensure_resources(&engine, &state, &mut effects)?;
        for alias in ALIASES {
            release_service_ports(&mut reservations, alias);
            if let Err(error) = create_container(paths, &engine, &state, alias, &work, &mut effects)
            {
                if present_owned.contains(alias)
                    || !format!("{error:#}").contains("address already in use")
                {
                    return Err(error);
                }
                // Reconcile only this newly created owned container if a host
                // process wins the short bind-to-create race, then retry once.
                let container = name(&state.owner, alias);
                if let Some(value) = existing(&engine, "container", &container, &state.owner)? {
                    validate_container(&value, &state, alias, &work)?;
                    run(
                        &engine,
                        vec!["container".into(), "rm".into(), "--force".into(), container],
                        Duration::from_secs(30),
                    )?;
                }
                let mut retry_reservations = Vec::new();
                for (key, service, _) in PUBLISHED {
                    if service == alias {
                        let listener = reserve_available_port(
                            &state.published_ports.values().copied().collect(),
                        )?;
                        state
                            .published_ports
                            .insert(key.to_owned(), listener.local_addr()?.port());
                        retry_reservations.push(listener);
                    }
                }
                save_private(&state_path(paths), &serde_json::to_vec_pretty(&state)?)?;
                drop(retry_reservations);
                create_container(paths, &engine, &state, alias, &work, &mut effects)?;
            }
            await_ready(&engine, &state, alias)?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        let cleanup_failures = rollback(&engine, &effects, &state, &work);
        if !cleanup_failures.is_empty() {
            let report = startup_diagnostic(&state, &cleanup_failures.join("; "));
            return Err(error.context(format!("rollback incomplete: {report}")));
        }
        return Err(error);
    }
    status_with(&engine, &state, &work)
}
fn status_with(engine: &Engine, state: &State, work: &Path) -> Result<Value> {
    if let Some(value) = existing(
        engine,
        "network",
        &name(&state.owner, "network"),
        &state.owner,
    )? {
        validate_resource("network", &value)?;
    }
    for volume in VOLUMES {
        if let Some(value) = existing(engine, "volume", &name(&state.owner, volume), &state.owner)?
        {
            validate_resource("volume", &value)?;
        }
    }
    let mut entries = Vec::new();
    for alias in ALIASES {
        let n = name(&state.owner, alias);
        let value = existing(engine, "container", &n, &state.owner)?;
        let (present, live) = if let Some(value) = value {
            validate_container(&value, state, alias, work)?;
            (true, running(&value))
        } else {
            (false, false)
        };
        entries.push(json!({"service":alias,"container":n,"present":present,"running":live,"published_ports":published_mappings(state, alias)}));
    }
    let ready = entries.iter().all(|v| v["running"] == true)
        && [
            "browserless",
            "penpot-mcp",
            "penpot-frontend",
            "kroki",
            "imgproxy",
            "comfyui",
            "design-tools",
        ]
        .iter()
        .all(|alias| probe_container(engine, state, alias).is_ok());
    Ok(
        json!({"package":"design","ready":ready,"workspace":work,"services":entries,"network":name(&state.owner,"network"),"published_ports":state.published_ports}),
    )
}
pub(super) fn status(paths: &Paths) -> Result<Value> {
    let engine = engine(paths)?;
    let installed = receipt(paths, &engine)?;
    let state = saved_state(paths, &engine)?;
    ensure!(
        state.images == installed.images,
        "design state image receipt drift"
    );
    status_with(&engine, &state, &workspace(paths)?)
}
fn session_identity_for_purge(work: &Path) -> Result<Option<PathBuf>> {
    let path = work.join(".mirket/penpot-session.json");
    util::reject_symlink_components(&path)?;
    let metadata = match fs::symlink_metadata(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    ensure!(
        metadata.is_file(),
        "owned Penpot session identity is not a regular file"
    );
    let identity: SessionIdentity = serde_json::from_slice(&util::read_bounded(&path, 4096)?)?;
    let valid_uuid = |value: &str| {
        let bytes = value.as_bytes();
        bytes.len() == 36
            && bytes.iter().enumerate().all(|(index, byte)| {
                if [8, 13, 18, 23].contains(&index) {
                    *byte == b'-'
                } else {
                    byte.is_ascii_hexdigit()
                }
            })
    };
    ensure!(
        identity.schema_version == 1
            && valid_uuid(&identity.file_id)
            && valid_uuid(&identity.team_id),
        "invalid owned Penpot session identity"
    );
    Ok(Some(path))
}
pub(super) fn stop(paths: &Paths, purge_data: bool) -> Result<Value> {
    let engine = engine(paths)?;
    let state = saved_state_any(paths, &engine)?;
    let dir = root(paths);
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    let work = paths.root.join("execution/podman/staging/design/workspace");
    util::reject_symlink_components(&work)?;
    let session_identity = if purge_data {
        session_identity_for_purge(&work)?
    } else {
        None
    };
    for alias in ALIASES {
        if let Some(value) = existing(
            &engine,
            "container",
            &name(&state.owner, alias),
            &state.owner,
        )? {
            validate_container(&value, &state, alias, &work)?;
        }
    }
    if let Some(value) = existing(
        &engine,
        "network",
        &name(&state.owner, "network"),
        &state.owner,
    )? {
        validate_resource("network", &value)?;
    }
    for volume in VOLUMES {
        if let Some(value) = existing(&engine, "volume", &name(&state.owner, volume), &state.owner)?
        {
            validate_resource("volume", &value)?;
        }
    }
    for alias in ALIASES.iter().rev() {
        let n = name(&state.owner, alias);
        if existing(&engine, "container", &n, &state.owner)?.is_some() {
            run(
                &engine,
                vec!["container".into(), "rm".into(), "--force".into(), n],
                Duration::from_secs(60),
            )?;
        }
    }
    if purge_data {
        for volume in VOLUMES {
            let n = name(&state.owner, volume);
            if existing(&engine, "volume", &n, &state.owner)?.is_some() {
                run(
                    &engine,
                    vec!["volume".into(), "rm".into(), n],
                    Duration::from_secs(60),
                )?;
            }
        }
        let n = name(&state.owner, "network");
        if existing(&engine, "network", &n, &state.owner)?.is_some() {
            run(
                &engine,
                vec!["network".into(), "rm".into(), n],
                Duration::from_secs(60),
            )?;
        }
        if let Some(path) = session_identity {
            fs::remove_file(path)?;
        }
        fs::remove_file(state_path(paths))?;
    }
    Ok(
        json!({"package":"design","stopped":true,"purged_data":purge_data,"workspace":workspace(paths)?}),
    )
}
pub(super) fn launch(paths: &Paths) -> Result<LaunchSpec> {
    let engine = engine(paths)?;
    let installed = receipt(paths, &engine)?;
    let state = saved_state(paths, &engine)?;
    ensure!(
        state.images == installed.images,
        "design state image receipt drift"
    );
    let work = workspace(paths)?;
    let status = status_with(&engine, &state, &work)?;
    ensure!(
        status["ready"] == true,
        "design stack is not ready; run mirket package start design"
    );
    let mut args = Vec::new();
    if cfg!(target_os = "macos") || cfg!(windows) {
        args.extend(["--connection".into(), engine.connection.clone()]);
    }
    args.extend([
        "exec".into(),
        "-i".into(),
        "--workdir".into(),
        "/workspace".into(),
        name(&state.owner, "design-tools"),
        "node".into(),
        "/opt/design/server.mjs".into(),
    ]);
    let (request_timeout, tool_timeouts) = tool_budgets()?;
    Ok(LaunchSpec {
        command: engine.binary,
        args,
        env: engine.env,
        cwd: work,
        allowed_tools: None,
        denied_tools: BTreeSet::new(),
        startup_timeout: Duration::from_secs(120),
        request_timeout,
        tool_timeouts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_tool_budgets_cover_policy_and_outlast_long_jobs() {
        let (host, tools) = tool_budgets().unwrap();
        assert_eq!(host, tools.values().copied().max().unwrap());
        assert!(tools["hyperframes_render"] > Duration::from_secs(120));
        assert!(tools["comfyui_submit"] > Duration::from_secs(180));
        assert!(tools["typst_compile"] > tools["hyperframes_render"]);
    }

    #[test]
    fn readiness_rejects_an_unbounded_body_without_buffering_it() {
        let error = readiness_json(std::io::repeat(b'a'), "probe").unwrap_err();
        assert!(
            error
                .to_string()
                .contains("readiness response is oversized")
        );
        assert_eq!(
            readiness_json(std::io::Cursor::new(b"{\"ok\":true}"), "probe").unwrap(),
            json!({"ok": true})
        );
    }

    #[test]
    fn occupied_host_port_is_reported_before_startup_effects() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mapping = format!("{}:3000", listener.local_addr().unwrap());
        let error = preflight_host_port("browserless", &mapping).unwrap_err();
        assert!(format!("{error:#}").contains("browserless requires free host address"));
        drop(listener);
        preflight_host_port("browserless", &mapping).unwrap();
    }

    #[test]
    fn missing_service_gets_a_free_port_without_reassigning_existing_service() {
        let occupied = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = occupied.local_addr().unwrap().port();
        let mut ports = allocate_published_ports().unwrap();
        ports.insert("browserless".into(), port);
        validate_published_ports(&ports).unwrap();
        let before = ports.clone();
        assert!(
            reserve_start_ports(
                &mut ports,
                &BTreeSet::from(["browserless"]),
                &BTreeSet::new()
            )
            .is_err()
        );
        assert_eq!(ports, before);
        let (mut reservations, changed) =
            reserve_start_ports(&mut ports, &BTreeSet::new(), &BTreeSet::new()).unwrap();
        assert!(changed);
        assert_ne!(ports["browserless"], port);
        for (key, value) in before {
            if key != "browserless" {
                assert_eq!(ports[&key], value);
            }
        }
        assert_eq!(reservations.len(), PUBLISHED.len());
        validate_published_ports(&ports).unwrap();
        assert!(TcpListener::bind(("127.0.0.1", ports["browserless"])).is_err());
        release_service_ports(&mut reservations, "browserless");
        let claimed = TcpListener::bind(("127.0.0.1", ports["browserless"])).unwrap();
        assert!(TcpListener::bind(("127.0.0.1", ports["penpot_plugin"])).is_err());
        drop(claimed);
    }

    #[test]
    fn actual_private_keep_id_mapping_accepts_only_owned_user_translation() {
        let mut host = json!({
            "UsernsMode":"private",
            "IDMappings":{
                "UidMap":["0:1:999","999:0:1","1000:1000:999001"],
                "GidMap":["0:1:999","999:0:1","1000:1000:999001"]
            }
        });
        assert!(rootless_keep_id_policy(&host, 999));
        host["IDMappings"]["UidMap"][1] = json!("998:0:1");
        assert!(!rootless_keep_id_policy(&host, 999));
        host["IDMappings"]["UidMap"][1] = json!("999:0:1");
        host["UsernsMode"] = json!("host");
        assert!(!rootless_keep_id_policy(&host, 999));
    }

    #[test]
    fn purge_accepts_only_a_valid_owned_penpot_session_file() {
        let workspace = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let private = workspace.path().join(".mirket");
        fs::create_dir(&private).unwrap();
        let path = private.join("penpot-session.json");
        assert!(
            session_identity_for_purge(workspace.path())
                .unwrap()
                .is_none()
        );
        fs::write(
            &path,
            r#"{"schemaVersion":1,"fileId":"12345678-1234-1234-1234-123456789abc","teamId":"abcdef12-3456-7890-abcd-ef1234567890"}"#,
        )
        .unwrap();
        assert_eq!(
            session_identity_for_purge(workspace.path()).unwrap(),
            Some(path.clone())
        );
        fs::write(
            &path,
            r#"{"schemaVersion":1,"fileId":"not-a-uuid","teamId":"also-not-a-uuid"}"#,
        )
        .unwrap();
        assert!(session_identity_for_purge(workspace.path()).is_err());
    }

    #[test]
    fn tools_build_closure_covers_every_runtime_module() {
        let mut authored: BTreeSet<String> =
            fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/design/tools"))
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .filter(|name| name.ends_with(".mjs"))
                .map(|name| format!("tools/{name}"))
                .collect();
        authored.insert("penpot-session.mjs".into());
        let staged: BTreeSet<String> = build_files("design-tools")
            .iter()
            .map(|(name, _)| (*name).to_owned())
            .collect();
        assert_eq!(
            authored,
            staged
                .iter()
                .filter(|name| name.ends_with(".mjs"))
                .cloned()
                .collect()
        );
        for required in [
            "Dockerfile",
            "tools/package.json",
            "tools/package-lock.json",
            "tools/tool-policy.json",
            "tools/tool-budgets.json",
            "tools/NOTICE.md",
            "penpot-session.mjs",
        ] {
            assert!(staged.contains(required), "missing build input: {required}");
        }
    }

    #[test]
    fn comfy_build_closure_covers_reviewed_source_patch() {
        let authored: BTreeSet<String> = fs::read_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/runtime/design/comfyui"
        ))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
        let staged: BTreeSet<String> = build_files("comfyui")
            .iter()
            .map(|(name, _)| (*name).to_owned())
            .collect();
        assert_eq!(authored, staged);
        assert!(staged.contains("exclude-3d.py"));
    }
}
