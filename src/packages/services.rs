//! The one reviewed design stack in Mirket's existing private Podman engine.
use super::oci::{image_id, inspect};
use super::oci::{inspected_image_id, normalize_image_id};
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
pub(crate) mod backend_check;

const REGISTRY: &str = include_str!("../../registry/design-stack.json");
const OWNER_LABEL: &str = "org.mirket.design-owner";
const BUILD_LABEL: &str = "org.mirket.design-build";
const ALIASES: [&str; 10] = [
    "penpot-postgres",
    "penpot-valkey",
    "penpot-backend",
    "penpot-exporter",
    "penpot-mcp",
    "penpot-frontend",
    "kroki-excalidraw",
    "kroki",
    "design-tools",
    "penpot-session",
];
const VOLUMES: [&str; 3] = ["db", "cache", "assets"];
const PUBLISHED: [(&str, &str, u16); 4] = [
    ("penpot_frontend", "penpot-frontend", 8080),
    ("penpot_mcp", "penpot-mcp", 4401),
    ("penpot_plugin", "penpot-mcp", 4400),
    ("kroki", "kroki", 8000),
];
const REMOTE: [&str; 9] = [
    "penpot-postgres",
    "penpot-valkey",
    "penpot-backend-payload",
    "debian-backend",
    "penpot-exporter",
    "penpot-frontend",
    "kroki-excalidraw",
    "kroki",
    "node-build",
];
const BUILD_TIMEOUT: Duration = Duration::from_secs(1800);

fn selected_groups(groups: &[String]) -> Result<BTreeSet<String>> {
    let mut selected = BTreeSet::from(["core".to_owned()]);
    for group in groups {
        if group == "all" {
            selected.extend(["diagrams".into(), "penpot".into()]);
        } else {
            selected.insert(group.clone());
        }
    }
    validate_groups(&selected)?;
    Ok(selected)
}
fn validate_groups(groups: &BTreeSet<String>) -> Result<()> {
    ensure!(
        groups.contains("core")
            && groups
                .iter()
                .all(|v| ["core", "diagrams", "penpot"].contains(&v.as_str())),
        "invalid design service groups"
    );
    Ok(())
}
fn enabled(state: &State, alias: &str) -> bool {
    group_enabled(&state.groups, alias)
}
fn group_enabled(groups: &BTreeSet<String>, alias: &str) -> bool {
    alias == "design-tools"
        || groups.contains(if alias.starts_with("penpot-") {
            "penpot"
        } else {
            "diagrams"
        })
}

// One-way data migration, only after the old release has removed its containers.
// Preserve account identity and named volumes; never adapt an old live process.
fn migrate_stopped_state(paths: &Paths, engine: &Engine, installed: &Receipt) -> Result<()> {
    if !state_path(paths).exists() {
        return Ok(());
    }
    let mut value: Value = serde_json::from_slice(&util::read_bounded(&state_path(paths), 32768)?)?;
    if value["schema_version"] != 1 {
        return Ok(());
    }
    ensure!(
        value["owner"] == owner(paths) && value["engine_owner"] == engine.owner(),
        "design state belongs to another environment"
    );
    let names = run(
        engine,
        vec![
            "container".into(),
            "ls".into(),
            "--all".into(),
            "--filter".into(),
            format!("label={OWNER_LABEL}={}", owner(paths)),
            "--format".into(),
            "{{.Names}}".into(),
        ],
        Duration::from_secs(30),
    )?;
    ensure!(
        names.trim().is_empty(),
        "stop the prior design generation with its installed release before migration"
    );
    let object = value.as_object_mut().context("invalid design state")?;
    for key in ["browserless_token", "imgproxy_key", "imgproxy_salt"] {
        object.remove(key);
    }
    object.insert("schema_version".into(), json!(2));
    object.insert("groups".into(), json!(["core"]));
    let ports = object
        .get_mut("published_ports")
        .and_then(Value::as_object_mut)
        .context("missing design ports")?;
    ports.retain(|key, _| PUBLISHED.iter().any(|(expected, _, _)| key == expected));
    let state: State = serde_json::from_value(value.clone())?;
    validate_state(paths, engine, &state)?;
    // This topology cutover does not migrate database formats or application data.
    // Refuse an unrelated image upgrade before any retained volume is opened.
    for alias in ["penpot-postgres", "penpot-backend"] {
        ensure!(
            state.images.get(alias) == installed.images.get(alias),
            "prior {alias} image differs; a separately verified data upgrade is required"
        );
    }
    save_private(&state_path(paths), &serde_json::to_vec_pretty(&value)?)
}

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
    published_ports: BTreeMap<String, u16>,
    groups: BTreeSet<String>,
    database_password: String,
    service_secret: String,
    penpot_email: String,
    penpot_password: String,
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
    for kind in ["penpot-mcp", "design-tools", "penpot-backend"] {
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
        "playwright",
        "sharp",
        "penpot",
        "kroki",
        "hyperframes",
        "typst",
        "svgo",
        "odiff",
    ]
    .into_iter()
    .collect();
    ensure!(
        actual == expected && value["tools"].as_array().unwrap().len() == 10,
        "design registry requires the ten supported design domains"
    );
    for alias in REMOTE.iter() {
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
fn inspect_many(engine: &Engine, kind: &str, names: &[String]) -> Result<Vec<Value>> {
    if names.is_empty() {
        return Ok(Vec::new());
    }
    let mut args = vec![kind.into(), "inspect".into()];
    args.extend_from_slice(names);
    let values: Vec<Value> = serde_json::from_str(&run(engine, args, Duration::from_secs(30))?)
        .context("invalid Podman batch inspection")?;
    ensure!(
        values.len() == names.len(),
        "incomplete Podman batch inspection"
    );
    Ok(values)
}
fn existing(engine: &Engine, kind: &str, name: &str, own: &str) -> Result<Option<Value>> {
    Ok(existing_many(engine, kind, &[name.to_owned()], own)?.remove(name))
}
fn owned_inspections(
    kind: &str,
    names: &[String],
    values: Vec<Value>,
    own: &str,
) -> Result<BTreeMap<String, Value>> {
    let mut result = BTreeMap::new();
    for value in values {
        let name = value[if kind == "network" { "name" } else { "Name" }]
            .as_str()
            .context("Podman resource name missing")?
            .trim_start_matches('/')
            .to_owned();
        ensure!(names.contains(&name), "unexpected Podman resource: {name}");
        let labels = match kind {
            "container" => &value["Config"]["Labels"],
            "network" => &value["labels"],
            _ => &value["Labels"],
        };
        ensure!(
            labels[OWNER_LABEL] == own,
            "refusing unrelated Podman {kind}: {name}"
        );
        ensure!(
            result.insert(name, value).is_none(),
            "duplicate Podman resource"
        );
    }
    ensure!(
        result.len() == names.len(),
        "incomplete Podman resource inspection"
    );
    Ok(result)
}
fn existing_many(
    engine: &Engine,
    kind: &str,
    names: &[String],
    own: &str,
) -> Result<BTreeMap<String, Value>> {
    let listing = match kind {
        "container" => vec!["container", "ls", "--all", "--format", "{{.Names}}"],
        "network" => vec!["network", "ls", "--format", "{{.Name}}"],
        "volume" => vec!["volume", "ls", "--format", "{{.Name}}"],
        _ => bail!("invalid resource kind"),
    }
    .into_iter()
    .map(str::to_owned)
    .collect();
    let listing = run(engine, listing, Duration::from_secs(30))?;
    let present: Vec<_> = names
        .iter()
        .filter(|name| listing.lines().any(|line| line == name.as_str()))
        .cloned()
        .collect();
    owned_inspections(kind, &present, inspect_many(engine, kind, &present)?, own)
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
fn service_command(alias: &str) -> Option<Vec<&'static str>> {
    match alias {
        "design-tools" => Some(vec!["node", "/opt/design/workspace-server.mjs"]),
        "penpot-mcp" => Some(vec!["node", "/opt/serve.mjs"]),
        "penpot-session" => Some(vec!["node", "/opt/design/penpot-session.mjs"]),
        // The upstream entrypoint materializes nginx.conf before invoking this command.
        // Its defaults size workers/connections for a public server, not a local canvas.
        "penpot-frontend" => Some(vec![
            "/bin/bash",
            "-ec",
            "sed -i 's/^worker_processes auto;/worker_processes 1;/; s/worker_connections 65535;/worker_connections 1024;/' /etc/nginx/nginx.conf; grep -q '^worker_processes 1;' /etc/nginx/nginx.conf; grep -q 'worker_connections 1024;' /etc/nginx/nginx.conf; exec nginx -g 'daemon off;'",
        ]),
        "penpot-valkey" => Some(vec![
            "valkey-server",
            "--save",
            "60",
            "1",
            "--appendonly",
            "yes",
            "--dir",
            "/data",
        ]),
        _ => None,
    }
}
fn validate_container(value: &Value, state: &State, alias: &str, work: &Path) -> Result<()> {
    let config = &value["Config"];
    let host = &value["HostConfig"];
    if let Some(command) = service_command(alias) {
        ensure!(
            config["Cmd"] == json!(command),
            "container command drift: {alias}"
        );
    }
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
        if ["design-tools", "penpot-session"].contains(&alias) {
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
    validate_state(paths, engine, &state)?;
    Ok(state)
}
fn validate_state(paths: &Paths, engine: &Engine, state: &State) -> Result<()> {
    ensure!(
        state.schema_version == 2
            && state.owner == owner(paths)
            && state.engine_owner == engine.owner(),
        "design state belongs to another environment"
    );
    for value in [&state.database_password, &state.service_secret] {
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
    validate_groups(&state.groups)?;
    Ok(())
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
    if kind == "penpot-mcp" {
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
        "penpot-backend" => vec![
            (
                "Dockerfile",
                include_bytes!("../../runtime/design/penpot-backend/Dockerfile"),
            ),
            (
                "NOTICE.md",
                include_bytes!("../../runtime/design/penpot-backend/NOTICE.md"),
            ),
        ],
        "penpot-mcp" => vec![
            (
                "production.mjs",
                include_bytes!("../../runtime/design/penpot-mcp/production.mjs"),
            ),
            (
                "Dockerfile",
                include_bytes!("../../runtime/design/penpot-mcp/Dockerfile"),
            ),
            (
                "serve.mjs",
                include_bytes!("../../runtime/design/penpot-mcp/serve.mjs"),
            ),
        ],
        "design-tools" => vec![
            (
                "tools/vector-worker.mjs",
                include_bytes!("../../runtime/design/tools/vector-worker.mjs"),
            ),
            (
                "tools/raster-worker.mjs",
                include_bytes!("../../runtime/design/tools/raster-worker.mjs"),
            ),
            (
                "tools/chromium-launcher",
                include_bytes!("../../runtime/design/tools/chromium-launcher"),
            ),
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
                "tools/media-evidence.mjs",
                include_bytes!("../../runtime/design/tools/media-evidence.mjs"),
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
    let path = stage.join("penpot-source.tar.gz");
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
/// Development test run in an isolated container; never changes service selection,
/// receipts or running services. The normal reviewed recipe supplies dependencies.
pub(crate) fn test_tools(paths: &Paths, project: &Path) -> Result<Value> {
    let engine = Engine::resolve(paths)?;
    engine.ready()?;
    let registry = definition()?;
    let image = prepare_build(paths, &engine, &registry, "design-tools")?;
    let tests = project.join("runtime/design/tools/test").canonicalize()?;
    let payload = crate::catalog::read_payload_directory(&tests, &[])?;
    let stage = tempfile::Builder::new()
        .prefix("design-test-")
        .tempdir_in(&engine.staging)?;
    let mut names = Vec::new();
    for (name, bytes) in payload {
        let destination = stage.path().join(&name);
        util::ensure_directory(destination.parent().context("test fixture parent")?)?;
        util::atomic_write(&destination, &bytes)?;
        if name.ends_with(".test.mjs") {
            names.push(format!("/opt/design/test/{name}"));
        }
    }
    ensure!(!names.is_empty(), "no design tests found");
    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "--network=none".into(),
        "--read-only".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--pids-limit=512".into(),
        "--memory=2g".into(),
        "--memory-swap=2g".into(),
        "--tmpfs=/tmp:rw,size=768m,mode=1777".into(),
        "--env=MIRKET_LIVE_BROWSER_TEST=1".into(),
        "--env=MIRKET_ISOLATED_TEST=1".into(),
        "--tmpfs=/workspace:rw,size=512m,mode=1777".into(),
        "--add-host=design-tools:127.0.0.1".into(),
        format!("--volume={}:/opt/design/test:ro", stage.path().display()),
        "--entrypoint=node".into(),
        image.clone(),
        "--test".into(),
        "--test-reporter=spec".into(),
    ];
    args.extend(names);
    let output = engine.run(&args, Duration::from_secs(300))?;
    Ok(
        json!({"ok":true,"image":image,"output":output,"scope":"Isolated test container; no user workspace mount or running service change"}),
    )
}

pub(crate) fn candidate_image(paths: &Paths, kind: &str) -> Result<String> {
    ensure!(
        ["design-tools", "penpot-mcp", "penpot-backend"].contains(&kind),
        "unknown reviewed design build"
    );
    let engine = Engine::resolve(paths)?;
    engine.ready()?;
    prepare_build(paths, &engine, &definition()?, kind)
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
    let bases: &[&str] = if kind == "penpot-backend" {
        &["penpot-backend-payload", "debian-backend"]
    } else {
        &["node-build"]
    };
    for base in bases {
        let reference = registry["images"][base].as_str().context("build base")?;
        if image_id(engine, reference).is_err() {
            run(
                engine,
                vec![
                    "pull".into(),
                    "--quiet".into(),
                    "--platform=linux/arm64".into(),
                    reference.into(),
                ],
                Duration::from_secs(900),
            )?;
        }
        image_id(engine, reference)?;
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
    if kind == "penpot-mcp" {
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

// Refuse an active generation change before setup replaces the executable that
// can stop it. Same-generation reinstallation remains idempotent.
fn preflight_installed_generation(paths: &Paths, engine: &Engine) -> Result<()> {
    if !state_path(paths).exists() {
        return Ok(());
    }
    let names = run(
        engine,
        vec![
            "container".into(),
            "ls".into(),
            "--all".into(),
            "--filter".into(),
            format!("label={OWNER_LABEL}={}", owner(paths)),
            "--format".into(),
            "{{.Names}}".into(),
        ],
        Duration::from_secs(30),
    )?;
    if names.trim().is_empty() {
        return Ok(());
    }
    let instruction = "stop design with the currently installed release before changing its runtime generation; installation was not changed";
    let state = saved_state(paths, engine).context(instruction)?;
    let work = workspace(paths)?;
    for container in names.lines().filter(|line| !line.is_empty()) {
        let alias = ALIASES
            .into_iter()
            .find(|alias| name(&state.owner, alias) == container)
            .context(instruction)?;
        let value = existing(engine, "container", container, &state.owner)?.context(instruction)?;
        validate_container(&value, &state, alias, &work).context(instruction)?;
    }
    Ok(())
}

pub(super) fn prepare(paths: &Paths) -> Result<()> {
    let registry = definition()?;
    let engine = engine(paths)?;
    ensure_workspace(paths)?;
    let dir = root(paths);
    util::ensure_directory(&dir)?;
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    preflight_installed_generation(paths, &engine)?;
    let mut images = BTreeMap::new();
    for alias in REMOTE.iter() {
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
    for alias in ["penpot-mcp", "design-tools", "penpot-backend"] {
        images.insert(
            alias.into(),
            prepare_build(paths, &engine, &registry, alias)?,
        );
    }
    images.insert("penpot-session".into(), images["design-tools"].clone());
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
    let references = ALIASES
        .iter()
        .map(|alias| {
            if [
                "penpot-mcp",
                "design-tools",
                "penpot-session",
                "penpot-backend",
            ]
            .contains(alias)
            {
                value
                    .images
                    .get(*alias)
                    .map(String::as_str)
                    .context("incomplete design image receipt")
            } else {
                registry["images"][*alias]
                    .as_str()
                    .context("image reference")
            }
            .map(str::to_owned)
        })
        .collect::<Result<Vec<_>>>()?;
    let inspected = inspect_many(engine, "image", &references)?;
    for ((alias, image), inspection) in ALIASES.iter().zip(&references).zip(&inspected) {
        let expected = value
            .images
            .get(*alias)
            .context("incomplete design image receipt")?;
        ensure!(
            inspected_image_id(inspection, image)? == *expected,
            "design image drift: {alias}"
        );
        if [
            "penpot-mcp",
            "design-tools",
            "penpot-session",
            "penpot-backend",
        ]
        .contains(alias)
        {
            ensure!(
                inspection["Labels"][BUILD_LABEL]
                    == build_recipe(
                        &registry,
                        if *alias == "penpot-session" {
                            "design-tools"
                        } else {
                            alias
                        }
                    )?,
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
        schema_version: 2,
        owner: owner(paths),
        engine_owner: engine.owner(),
        registry_sha256: registry_hash(),
        images: installed.images.clone(),
        published_ports: allocate_published_ports()?,
        groups: BTreeSet::from(["core".into()]),
        database_password: secret(32)?,
        service_secret: secret(32)?,
        penpot_email: format!("design-{}@mirket.invalid", &owner(paths)[14..]),
        // Penpot requires lowercase, uppercase, a digit and a special character.
        penpot_password: format!("{}Aa1!", secret(32)?),
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
        "kroki" => {
            put("KROKI_EXCALIDRAW_HOST", "kroki-excalidraw".into());
            put("KROKI_EXCALIDRAW_PORT", "8004".into());
        }
        "design-tools" => {
            put(
                "MIRKET_DESIGN_GROUPS",
                state.groups.iter().cloned().collect::<Vec<_>>().join(","),
            );
        }
        "penpot-session" => {
            put("MIRKET_PENPOT_EMAIL", state.penpot_email.clone());
            put("MIRKET_PENPOT_PASSWORD", state.penpot_password.clone());
        }
        _ => {}
    }
    env
}
fn uid(alias: &str) -> u32 {
    match alias {
        "penpot-postgres" | "penpot-valkey" => 999,
        "penpot-backend" | "penpot-exporter" | "penpot-frontend" | "kroki" | "kroki-excalidraw" => {
            1001
        }
        _ => 1000,
    }
}
fn limits(alias: &str) -> (&'static str, &'static str, &'static str) {
    match alias {
        "penpot-backend" => ("1280m", "1.5", "256"),
        "penpot-exporter" => ("768m", "1.5", "256"),
        "penpot-session" => ("1400m", "1.5", "256"),
        "penpot-postgres" => ("256m", "1", "128"),
        "penpot-valkey" => ("64m", "1", "128"),
        // Multiple desktop MCP connections share this renderer container.
        // Leave headroom for a real render alongside their retained sessions.
        "design-tools" => ("2048m", "2", "512"),
        "kroki" => ("512m", "1", "128"),
        _ => ("350m", "1", "128"),
    }
}
fn validate_group_memory(groups: &BTreeSet<String>, available_bytes: u64) -> Result<()> {
    let mut required_mib = 512_u64; // VM kernel and service-manager headroom.
    for alias in ALIASES
        .into_iter()
        .filter(|alias| group_enabled(groups, alias))
    {
        required_mib += limits(alias).0.trim_end_matches('m').parse::<u64>()?;
    }
    ensure!(
        available_bytes >= required_mib * 1024 * 1024,
        "selected design groups require at least {required_mib} MiB of usable VM RAM including 512 MiB OS headroom; select fewer groups or resize the stopped environment with mirket environment resize"
    );
    Ok(())
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
    active: &BTreeSet<&str>,
) -> Result<(BTreeMap<String, TcpListener>, bool)> {
    let mut reservations = BTreeMap::new();
    let mut changed = false;
    for (key, alias, internal) in PUBLISHED {
        if !active.contains(alias) || running.contains(alias) {
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
    if ["design-tools", "penpot-session"].contains(&alias) {
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
    if let Some(command) = service_command(alias) {
        args.extend(command.into_iter().map(str::to_owned));
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
fn http_ready(alias: &str, state: &State, timeout: Duration) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let (url, require_json) = match alias {
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
    }
    Ok(())
}
fn probe_container(engine: &Engine, state: &State, alias: &str) -> Result<()> {
    let n = name(&state.owner, alias);
    let cmd = match alias {
        "penpot-postgres" => Some(vec!["pg_isready", "-U", "penpot", "-d", "penpot"]),
        "penpot-valkey" => Some(vec!["valkey-cli", "ping"]),
        "design-tools" | "penpot-session" => Some(vec![
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
    http_ready(
        alias,
        state,
        engine.remaining_budget(Duration::from_secs(3))?,
    )?;
    Ok(())
}
fn startup_diagnostic(state: &State, value: &str) -> String {
    let mut redacted = value.to_owned();
    for secret in [
        &state.database_password,
        &state.service_secret,
        &state.penpot_email,
        &state.penpot_password,
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
    let deadline = Instant::now()
        + Duration::from_secs(if ["design-tools", "penpot-session"].contains(&alias) {
            180
        } else {
            90
        });
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
pub(super) fn start(paths: &Paths, groups: &[String]) -> Result<Value> {
    let engine = engine(paths)?;
    let installed = receipt(paths, &engine)?;
    let work = ensure_workspace(paths)?;
    let dir = root(paths);
    util::ensure_directory(&dir)?;
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    migrate_stopped_state(paths, &engine, &installed)?;
    let mut state = new_state(paths, &engine, &installed)?;
    let requested = if groups.is_empty() {
        state.groups.clone()
    } else {
        selected_groups(groups)?
    };
    let usable_memory = engine.ready()?["host"]["memTotal"]
        .as_u64()
        .context("Podman did not report usable VM memory")?;
    validate_group_memory(&requested, usable_memory)?;
    if requested != state.groups {
        for alias in ALIASES {
            ensure!(
                existing(
                    &engine,
                    "container",
                    &name(&state.owner, alias),
                    &state.owner
                )?
                .is_none(),
                "stop the design stack before changing enabled groups"
            );
        }
        state.groups = requested;
        save_private(&state_path(paths), &serde_json::to_vec_pretty(&state)?)?;
    }
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
    let active = ALIASES
        .into_iter()
        .filter(|alias| enabled(&state, alias))
        .collect();
    let (mut reservations, ports_changed) = reserve_start_ports(
        &mut state.published_ports,
        &present_owned,
        &running_owned,
        &active,
    )?;
    if ports_changed {
        save_private(&state_path(paths), &serde_json::to_vec_pretty(&state)?)?;
    }
    let mut effects = Vec::new();
    let result = (|| -> Result<()> {
        ensure_resources(&engine, &state, &mut effects)?;
        for alias in ALIASES
            .into_iter()
            .filter(|alias| enabled(&state, alias))
            .collect::<Vec<_>>()
        {
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
    let volumes = VOLUMES
        .iter()
        .map(|alias| name(&state.owner, alias))
        .collect::<Vec<_>>();
    for value in existing_many(engine, "volume", &volumes, &state.owner)?.values() {
        validate_resource("volume", value)?;
    }
    let containers = ALIASES
        .iter()
        .map(|alias| name(&state.owner, alias))
        .collect::<Vec<_>>();
    let inspected = existing_many(engine, "container", &containers, &state.owner)?;
    let mut entries = Vec::new();
    for alias in ALIASES {
        let n = name(&state.owner, alias);
        let value = inspected.get(&n);
        let (present, live) = if let Some(value) = value {
            validate_container(value, state, alias, work)?;
            (true, running(value))
        } else {
            (false, false)
        };
        entries.push(json!({"service":alias,"container":n,"present":present,"running":live,"enabled":enabled(state, alias),"published_ports":published_mappings(state, alias)}));
    }
    let core_ready = entries
        .iter()
        .any(|v| v["service"] == "design-tools" && v["running"] == true)
        && probe_container(engine, state, "design-tools").is_ok();
    let ready = entries
        .iter()
        .filter(|v| v["enabled"] == true)
        .all(|v| v["running"] == true)
        && ALIASES
            .iter()
            .filter(|alias| enabled(state, alias))
            .all(|alias| probe_container(engine, state, alias).is_ok());
    Ok(
        json!({"package":"design","ready":ready,"core_ready":core_ready,"groups":state.groups,"workspace":work,"services":entries,"network":name(&state.owner,"network"),"published_ports":state.published_ports}),
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
    Ok(json!({"package":"design","stopped":true,"purged_data":purge_data,"workspace":work}))
}
pub(super) fn launch(paths: &Paths, deadline: Instant) -> Result<LaunchSpec> {
    let engine = Engine::resolve(paths)?.with_deadline(deadline);
    let info = engine.ready()?;
    ensure!(
        info["host"]["arch"] == "arm64",
        "design stack requires the verified native Linux ARM64 Podman environment"
    );
    let installed = receipt(paths, &engine)?;
    let state = saved_state(paths, &engine)?;
    ensure!(
        state.images == installed.images,
        "design state image receipt drift"
    );
    let work = workspace(paths)?;
    let core = existing(
        &engine,
        "container",
        &name(&state.owner, "design-tools"),
        &state.owner,
    )?
    .context("design core is absent; run mirket package start design")?;
    validate_container(&core, &state, "design-tools", &work)?;
    ensure!(
        running(&core),
        "design core is stopped; run mirket package start design"
    );
    probe_container(&engine, &state, "design-tools")?;
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
    let startup_timeout = engine.remaining_budget(Duration::from_secs(120))?;
    Ok(LaunchSpec {
        command: engine.binary,
        args,
        env: engine.env,
        cwd: work,
        allowed_tools: None,
        denied_tools: BTreeSet::new(),
        startup_timeout,
        request_timeout,
        tool_timeouts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_require_explicit_heavy_capabilities() {
        assert_eq!(
            selected_groups(&[]).unwrap(),
            BTreeSet::from(["core".into()])
        );
        assert_eq!(
            selected_groups(&["diagrams".into()]).unwrap(),
            BTreeSet::from(["core".into(), "diagrams".into()])
        );
        assert_eq!(selected_groups(&["all".into()]).unwrap().len(), 3);
        assert!(selected_groups(&["invented".into()]).is_err());
        assert!(validate_groups(&BTreeSet::from(["penpot".into()])).is_err());
    }

    #[test]
    fn selected_service_caps_leave_os_headroom_before_starting() {
        let core = selected_groups(&[]).unwrap();
        let all = selected_groups(&["all".into()]).unwrap();
        assert!(validate_group_memory(&core, 6144 * 1024 * 1024).is_ok());
        assert!(validate_group_memory(&all, 7900 * 1024 * 1024).is_ok());
        assert!(validate_group_memory(&all, 7680 * 1024 * 1024).is_err());
        assert!(validate_group_memory(&core, 2559 * 1024 * 1024).is_err());
    }

    #[test]
    fn batch_inspection_checks_every_identity_and_owner() {
        let names = vec!["one".into(), "two".into()];
        let item =
            |name: &str, owner: &str| json!({"Name":name,"Config":{"Labels":{OWNER_LABEL:owner}}});
        assert_eq!(
            owned_inspections(
                "container",
                &names,
                vec![item("/two", "ours"), item("one", "ours")],
                "ours"
            )
            .unwrap()
            .len(),
            2
        );
        for values in [
            vec![item("one", "ours")],
            vec![item("one", "ours"), item("one", "ours")],
            vec![item("one", "ours"), item("other", "ours")],
            vec![item("one", "ours"), item("two", "foreign")],
        ] {
            assert!(owned_inspections("container", &names, values, "ours").is_err());
        }
        assert!(
            owned_inspections(
                "volume",
                &["vol".into()],
                vec![json!({"Name":"vol","Labels":{OWNER_LABEL:"ours"}})],
                "ours"
            )
            .is_ok()
        );
        assert!(
            owned_inspections(
                "network",
                &["net".into()],
                vec![json!({"name":"net","labels":{OWNER_LABEL:"foreign"}})],
                "ours"
            )
            .is_err()
        );
    }

    #[test]
    fn batch_image_validation_keeps_architecture_and_manifest_checks() {
        let digest = "a".repeat(64);
        let reference = format!("registry/image@sha256:{digest}");
        let mut image = json!({"Id":"b".repeat(64),"Architecture":"arm64","Os":"linux","RepoDigests":[reference]});
        assert_eq!(
            inspected_image_id(&image, &reference).unwrap(),
            format!("sha256:{}", "b".repeat(64))
        );
        image["RepoDigests"] = json!([]);
        assert!(inspected_image_id(&image, &reference).is_err());
        image["Digest"] = json!(format!("sha256:{digest}"));
        assert!(inspected_image_id(&image, &reference).is_ok());
        image["Architecture"] = json!("amd64");
        assert!(inspected_image_id(&image, &reference).is_err());
    }

    #[test]
    fn design_tool_budgets_cover_policy_and_outlast_long_jobs() {
        let (host, tools) = tool_budgets().unwrap();
        assert_eq!(host, tools.values().copied().max().unwrap());
        assert!(tools["hyperframes_render"] > Duration::from_secs(120));
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
        let error = preflight_host_port("kroki", &mapping).unwrap_err();
        assert!(format!("{error:#}").contains("kroki requires free host address"));
        drop(listener);
        preflight_host_port("kroki", &mapping).unwrap();
    }

    #[test]
    fn missing_service_gets_a_free_port_without_reassigning_existing_service() {
        let occupied = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = occupied.local_addr().unwrap().port();
        let mut ports = allocate_published_ports().unwrap();
        ports.insert("kroki".into(), port);
        validate_published_ports(&ports).unwrap();
        let before = ports.clone();
        let (inactive, changed) = reserve_start_ports(
            &mut ports,
            &BTreeSet::from(["kroki"]),
            &BTreeSet::new(),
            &BTreeSet::from(["design-tools"]),
        )
        .unwrap();
        assert!(inactive.is_empty());
        assert!(!changed);
        assert_eq!(ports, before);
        assert!(
            reserve_start_ports(
                &mut ports,
                &BTreeSet::from(["kroki"]),
                &BTreeSet::new(),
                &ALIASES.into_iter().collect()
            )
            .is_err()
        );
        assert_eq!(ports, before);
        let (mut reservations, changed) = reserve_start_ports(
            &mut ports,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &ALIASES.into_iter().collect(),
        )
        .unwrap();
        assert!(changed);
        assert_ne!(ports["kroki"], port);
        for (key, value) in before {
            if key != "kroki" {
                assert_eq!(ports[&key], value);
            }
        }
        assert_eq!(reservations.len(), PUBLISHED.len());
        validate_published_ports(&ports).unwrap();
        assert!(TcpListener::bind(("127.0.0.1", ports["kroki"])).is_err());
        release_service_ports(&mut reservations, "kroki");
        let claimed = TcpListener::bind(("127.0.0.1", ports["kroki"])).unwrap();
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
}
