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
    net::TcpListener,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const REGISTRY: &str = include_str!("../../registry/design-stack.json");
const OWNER_LABEL: &str = "org.mirket.design-owner";
const BUILD_LABEL: &str = "org.mirket.design-build";
const ALIASES: [&str; 1] = ["design-tools"];
const VOLUMES: [&str; 0] = [];
const PUBLISHED: [(&str, &str, u16); 0] = [];
const REMOTE: [&str; 2] = ["go-build", "node-build"];
const BUILD_TIMEOUT: Duration = Duration::from_secs(1800);

fn selected_groups(groups: &[String]) -> Result<BTreeSet<String>> {
    let mut selected = BTreeSet::from(["core".to_owned()]);
    selected.extend(groups.iter().cloned());
    validate_groups(&selected)?;
    Ok(selected)
}
fn validate_groups(groups: &BTreeSet<String>) -> Result<()> {
    ensure!(
        groups == &BTreeSet::from(["core".to_owned()]),
        "invalid design service groups"
    );
    Ok(())
}
fn enabled(state: &State, alias: &str) -> bool {
    group_enabled(&state.groups, alias)
}
fn group_enabled(_groups: &BTreeSet<String>, alias: &str) -> bool {
    alias == "design-tools"
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

#[derive(Debug)]
enum StartEffect {
    CreatedNetwork(String),
    CreatedContainer { alias: String, name: String },
    StartedStoppedContainer { alias: String, name: String },
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
    for kind in ALIASES {
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
        "openpencil",
        "typst",
        "svgo",
        "odiff",
        "axe-core",
        "d2",
    ]
    .into_iter()
    .collect();
    ensure!(
        actual == expected && value["tools"].as_array().unwrap().len() == expected.len(),
        "design registry requires the supported design domains"
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
        _ => None,
    }
}
const BROWSER_SECCOMP: &[u8] = include_bytes!("../../runtime/design/tools/browser-seccomp.json");
fn browser_seccomp_path(work: &Path) -> PathBuf {
    work.parent().expect("owned workspace parent").join(format!(
        "browser-seccomp-{}.json",
        util::sha256(BROWSER_SECCOMP)
    ))
}
fn ensure_browser_seccomp(work: &Path) -> Result<()> {
    let path = browser_seccomp_path(work);
    util::reject_symlink_components(&path)?;
    if path.exists() {
        ensure!(
            util::read_bounded(&path, 65536)? == BROWSER_SECCOMP,
            "browser seccomp policy drift"
        );
    } else {
        util::atomic_write(&path, BROWSER_SECCOMP)?;
    }
    Ok(())
}
fn security_options(alias: &str, work: &Path) -> Vec<String> {
    let mut options = vec!["no-new-privileges".into()];
    if alias == "design-tools" {
        // The confined SELinux domain permits nested proc mounts; the kernel
        // requires removal of outer locked proc submounts for private PID proc.
        // Seccomp permits chroot for Chromium's own user namespace, not SYS_ADMIN.
        options.extend([
            "label=type:container_engine_t".into(),
            "unmask=/proc/*".into(),
            format!("seccomp={}", browser_seccomp_path(work).display()),
        ]);
    }
    options
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
            .is_some_and(|id| state.images.get(alias) == Some(&id)),
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
            && host["SecurityOpt"] == json!(security_options(alias, work)),
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
        mounts.len() == expected_volumes.len() + usize::from(alias == "design-tools"),
        "container mount count drift: {alias}"
    );
    if alias == "design-tools" {
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
    }
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
fn saved_state_any(paths: &Paths, engine: &Engine) -> Result<State> {
    let state: State = serde_json::from_slice(&util::read_bounded(&state_path(paths), 32768)?)?;
    validate_state(paths, engine, &state)?;
    Ok(state)
}
fn validate_state(paths: &Paths, engine: &Engine, state: &State) -> Result<()> {
    ensure!(
        state.schema_version == 4
            && state.owner == owner(paths)
            && state.engine_owner == engine.owner(),
        "design state belongs to another environment"
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
    let mut bytes = recipe_id(&build_files(kind)).into_bytes();
    bytes.extend_from_slice(serde_json::to_string(&registry["images"])?.as_bytes());
    Ok(util::sha256(&bytes))
}
fn build_files(kind: &str) -> Vec<(&'static str, &'static [u8])> {
    match kind {
        "design-tools" => vec![
            (
                "tools/d2.go.mod",
                include_bytes!("../../runtime/design/tools/d2.go.mod"),
            ),
            (
                "tools/d2.go.sum",
                include_bytes!("../../runtime/design/tools/d2.go.sum"),
            ),
            ("tools/browser-seccomp.json", BROWSER_SECCOMP),
            (
                "tools/inspection.mjs",
                include_bytes!("../../runtime/design/tools/inspection.mjs"),
            ),
            (
                "tools/openpencil.mjs",
                include_bytes!("../../runtime/design/tools/openpencil.mjs"),
            ),
            (
                "tools/openpencil-cli.mjs",
                include_bytes!("../../runtime/design/tools/openpencil-cli.mjs"),
            ),
            (
                "tools/openpencil-tools.json",
                include_bytes!("../../runtime/design/tools/openpencil-tools.json"),
            ),
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
                "tools/browser.json",
                include_bytes!("../../runtime/design/tools/browser.json"),
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
                "tools/browser-worker.mjs",
                include_bytes!("../../runtime/design/tools/browser-worker.mjs"),
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
                "tools/workspace-server.mjs",
                include_bytes!("../../runtime/design/tools/workspace-server.mjs"),
            ),
            (
                "tools/NOTICE.md",
                include_bytes!("../../runtime/design/tools/NOTICE.md"),
            ),
        ],
        _ => unreachable!(),
    }
}
/// Development test run in an isolated container; never changes service selection,
/// receipts or running services. The normal reviewed recipe supplies dependencies.
pub(crate) fn test_tools(paths: &Paths, project: &Path) -> Result<Value> {
    let engine = Engine::resolve(paths)?;
    engine.candidate_ready()?;
    let registry = definition()?;
    let image = prepare_build(&engine, &registry, "design-tools")?;
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
    ensure_browser_seccomp(stage.path())?;
    let mut args = vec![
        "run".into(),
        "--rm".into(),
        "--userns=keep-id:uid=1000,gid=1000".into(),
        "--user=1000:1000".into(),
        "--network=none".into(),
        "--read-only".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--security-opt=label=type:container_engine_t".into(),
        "--security-opt=unmask=/proc/*".into(),
        format!(
            "--security-opt=seccomp={}",
            browser_seccomp_path(stage.path()).display()
        ),
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
    ensure!(ALIASES.contains(&kind), "unknown reviewed design build");
    let engine = Engine::resolve(paths)?;
    engine.candidate_ready()?;
    prepare_build(&engine, &definition()?, kind)
}

fn prepare_build(engine: &Engine, registry: &Value, kind: &str) -> Result<String> {
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
    for base in ["node-build", "go-build"] {
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
    let mut build_args = vec![
        "build".into(),
        "--jobs=1".into(),
        "--force-rm".into(),
        "--memory=2g".into(),
        "--memory-swap=2g".into(),
        "--ulimit=nproc=512:512".into(),
        "--pull=never".into(),
        "--platform=linux/arm64".into(),
        "--label".into(),
        format!("{BUILD_LABEL}={recipe}"),
        "--tag".into(),
        tag.clone(),
    ];
    build_args.push(stage.to_string_lossy().into_owned());
    run(engine, build_args, BUILD_TIMEOUT)?;
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

fn validate_receipt(paths: &Paths, value: &Receipt) -> Result<()> {
    ensure!(
        value.schema_version == 1
            && value.owner == owner(paths)
            && value.registry_sha256 == registry_hash(),
        "design install receipt belongs to another generation"
    );
    let keys: BTreeSet<_> = value.images.keys().map(String::as_str).collect();
    let core = BTreeSet::from(["node-build", "design-tools"]);
    ensure!(keys == core, "incomplete or unknown design image receipt");
    for id in value.images.values() {
        normalize_image_id(id)?;
        ensure!(id.starts_with("sha256:"), "invalid saved design image ID");
    }
    Ok(())
}
fn read_receipt(paths: &Paths, path: &Path) -> Result<Receipt> {
    let value = serde_json::from_slice(&util::read_bounded(path, 32768)?)?;
    validate_receipt(paths, &value)?;
    Ok(value)
}
fn publish_receipt(paths: &Paths, path: &Path, value: &Receipt) -> Result<()> {
    validate_receipt(paths, value)?;
    let bytes = serde_json::to_vec_pretty(value)?;
    if path.exists() {
        ensure!(
            util::read_bounded(path, 32768)? == bytes,
            "immutable design generation receipt drift"
        );
    } else {
        save_private(path, &bytes)?;
    }
    Ok(())
}
fn preparation_groups(
    paths: &Paths,
    engine: &Engine,
    groups: &[String],
) -> Result<BTreeSet<String>> {
    let mut selected = selected_groups(groups)?;
    // Setup/update preserves the current saved group selection.
    if state_path(paths).exists() {
        let state: State = serde_json::from_slice(&util::read_bounded(&state_path(paths), 32768)?)?;
        ensure!(
            state.owner == owner(paths) && state.engine_owner == engine.owner(),
            "design state belongs to another environment"
        );
        validate_state(paths, engine, &state)?;
        selected.extend(state.groups);
    }
    Ok(selected)
}
fn prepare_remote(engine: &Engine, registry: &Value, alias: &str) -> Result<String> {
    let reference = registry["images"][alias]
        .as_str()
        .context("missing image")?;
    match image_id(engine, reference) {
        Ok(id) => Ok(id),
        Err(_) => {
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
            image_id(engine, reference)
        }
    }
}
pub(super) fn prepare(paths: &Paths, groups: &[String]) -> Result<()> {
    selected_groups(groups)?; // Reject invalid intent before resolving the engine.
    let registry = definition()?;
    let engine = engine(paths)?;
    prepare_on_engine(paths, groups, &engine, &registry)
}
fn prepare_on_engine(
    paths: &Paths,
    groups: &[String],
    engine: &Engine,
    registry: &Value,
) -> Result<()> {
    if receipt_path(paths).exists() {
        read_receipt(paths, &receipt_path(paths))?;
    }
    ensure_workspace(paths)?;
    let dir = root(paths);
    util::ensure_directory(&dir)?;
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    preparation_groups(paths, engine, groups)?;
    preflight_installed_generation(paths, engine)?;
    let base = receipt_path(paths);
    if base.exists() {
        // Validate first: never repair a drifted receipt by overwriting it.
        let saved = local_receipt(paths)?;
        for (alias, expected) in &saved.images {
            let actual = if ALIASES.contains(&alias.as_str()) {
                prepare_build(engine, registry, alias)?
            } else {
                prepare_remote(engine, registry, alias)?
            };
            ensure!(actual == *expected, "immutable design image drift: {alias}");
        }
        receipt(paths, engine)?;
    } else {
        let images = BTreeMap::from([
            (
                "node-build".into(),
                prepare_remote(engine, registry, "node-build")?,
            ),
            (
                "design-tools".into(),
                prepare_build(engine, registry, "design-tools")?,
            ),
        ]);
        publish_receipt(
            paths,
            &base,
            &Receipt {
                schema_version: 1,
                owner: owner(paths),
                registry_sha256: registry_hash(),
                images,
            },
        )?;
    }
    receipt(paths, engine)?;
    Ok(())
}
fn local_receipt(paths: &Paths) -> Result<Receipt> {
    read_receipt(paths, &receipt_path(paths))
        .context("design stack is not installed or has drifted; inspect before mirket package install design")
}
fn receipt(paths: &Paths, engine: &Engine) -> Result<Receipt> {
    let value = local_receipt(paths)?;
    let registry = definition()?;
    let aliases: Vec<_> = value.images.keys().collect();
    let references: Vec<_> = aliases
        .iter()
        .map(|alias| {
            if ALIASES.contains(&alias.as_str()) {
                value.images[*alias].clone()
            } else {
                registry["images"][*alias].as_str().unwrap().to_owned()
            }
        })
        .collect();
    let inspected = inspect_many(engine, "image", &references)?;
    for ((alias, image), inspection) in aliases.iter().zip(&references).zip(&inspected) {
        ensure!(
            inspected_image_id(inspection, image)? == value.images[*alias],
            "design image drift: {alias}"
        );
        if ALIASES.contains(&alias.as_str()) {
            ensure!(
                inspection["Labels"][BUILD_LABEL] == build_recipe(&registry, alias)?,
                "design build recipe drift: {alias}"
            );
        }
    }
    Ok(value)
}
fn require_prepared(groups: &BTreeSet<String>, installed: &Receipt) -> Result<()> {
    for alias in ALIASES
        .into_iter()
        .filter(|alias| group_enabled(groups, alias))
    {
        ensure!(
            installed.images.contains_key(alias),
            "design is not prepared; run mirket package prepare design before starting it"
        );
    }
    Ok(())
}
fn state_matches_receipt(state: &State, installed: &Receipt) -> bool {
    state
        .images
        .iter()
        .all(|(alias, id)| installed.images.get(alias) == Some(id))
        && require_prepared(&state.groups, installed).is_ok()
        && ALIASES
            .into_iter()
            .filter(|alias| enabled(state, alias))
            .all(|alias| state.images.contains_key(alias))
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
            // Image or generation changes require the prior service to stop.
            if prior.registry_sha256 != registry_hash() || !state_matches_receipt(&prior, installed)
            {
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
            }
            prior.registry_sha256 = registry_hash();
            prior.images = installed.images.clone();
            save_private(&state_path(paths), &serde_json::to_vec_pretty(&prior)?)?;
        }
        return Ok(prior);
    }
    let state = fresh_state(paths, engine, installed)?;
    save_private(&state_path(paths), &serde_json::to_vec_pretty(&state)?)?;
    Ok(state)
}
fn fresh_state(paths: &Paths, engine: &Engine, installed: &Receipt) -> Result<State> {
    Ok(State {
        schema_version: 4,
        owner: owner(paths),
        engine_owner: engine.owner(),
        registry_sha256: registry_hash(),
        images: installed.images.clone(),
        published_ports: allocate_published_ports()?,
        groups: BTreeSet::from(["core".into()]),
    })
}
fn uid(_alias: &str) -> u32 {
    1000
}
fn limits(_alias: &str) -> (&'static str, &'static str, &'static str) {
    ("2048m", "2", "512")
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
fn volumes(_state: &State, _alias: &str) -> Vec<(String, &'static str, bool)> {
    vec![]
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
    ];
    if alias == "design-tools" {
        ensure_browser_seccomp(work)?;
    }
    for option in security_options(alias, work).into_iter().skip(1) {
        args.push(format!("--security-opt={option}"));
    }
    if alias == "design-tools" {
        args.extend([
            "--init".into(),
            "--volume".into(),
            format!("{}:/workspace:rw", work.display()),
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
    args.push(image.clone());
    if let Some(command) = service_command(alias) {
        args.extend(command.into_iter().map(str::to_owned));
    }
    effects.push(StartEffect::CreatedContainer {
        alias: alias.to_owned(),
        name: container,
    });
    run(engine, args, Duration::from_secs(120)).map(|_| ())
}
fn running(value: &Value) -> bool {
    value["State"]["Running"] == true
}
fn probe_container(engine: &Engine, state: &State, alias: &str) -> Result<()> {
    let n = name(&state.owner, alias);
    let cmd = match alias {
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
    Ok(())
}
fn startup_diagnostic(_state: &State, value: &str) -> String {
    value.chars().take(4096).collect()
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
pub(super) fn start(paths: &Paths, groups: &[String]) -> Result<Value> {
    let engine = engine(paths)?;
    let installed = receipt(paths, &engine)?;
    let work = ensure_workspace(paths)?;
    let dir = root(paths);
    util::ensure_directory(&dir)?;
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    let requested = if groups.is_empty() {
        preparation_groups(paths, &engine, groups)?
    } else {
        selected_groups(groups)?
    };
    require_prepared(&requested, &installed)?;
    let mut state = new_state(paths, &engine, &installed)?;
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
        state_matches_receipt(&state, &installed),
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
            if let Err(error) = create_container(&engine, &state, alias, &work, &mut effects) {
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
                create_container(&engine, &state, alias, &work, &mut effects)?;
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
        state_matches_receipt(&state, &installed),
        "design state image receipt drift"
    );
    status_with(&engine, &state, &workspace(paths)?)
}
/// Observe the installed generation before an upgrade, under the same ownership
/// and container-drift checks used by stop. Candidate readiness is a later fact.
pub(super) fn activation_status(paths: &Paths) -> Result<Value> {
    let engine = engine(paths)?;
    let state = saved_state_any(paths, &engine)?;
    let work = paths.root.join("execution/podman/staging/design/workspace");
    util::reject_symlink_components(&work)?;
    status_with(&engine, &state, &work)
}
pub(super) fn stop(paths: &Paths, purge_data: bool) -> Result<Value> {
    let engine = engine(paths)?;
    let state = saved_state_any(paths, &engine)?;
    let dir = root(paths);
    let _lock = util::FileLock::acquire(&dir.join("operation.lock"))?;
    let work = paths.root.join("execution/podman/staging/design/workspace");
    util::reject_symlink_components(&work)?;
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
        state_matches_receipt(&state, &installed),
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

    #[cfg(unix)]
    #[test]
    fn core_preparation_preserves_immutable_receipt_and_rejects_drift() {
        use std::os::unix::fs::PermissionsExt;
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let binary = paths.home.join("podman-fixture");
        let registry = definition().unwrap();
        let recipe = build_recipe(&registry, "design-tools").unwrap();
        let fixture = format!(
            r#"#!/usr/bin/python3
import json,sys,pathlib
root=pathlib.Path(__file__).parent
args=sys.argv[1:]
if args[:1]==['--connection']:args=args[2:]
with (root/'calls').open('a') as f:f.write(json.dumps(args)+'\n')
if args[:1]==['build']:
 (root/'built').touch();print('built');sys.exit(0)
if args[:1]==['pull']:print('pulled');sys.exit(0)
assert args[:2]==['image','inspect'],args
refs=args[2:args.index('--format')] if '--format' in args else args[2:]
result=[]
for ref in refs:
 owned=ref.startswith('localhost/mirket-design-design-tools:') or ref=='sha256:'+'a'*64
 if owned and not (root/'built').exists():sys.exit(1)
 result.append({{'Id':'sha256:'+('a' if owned else 'd')*64,'Architecture':'arm64','Os':'linux','RepoDigests':{images},'Labels':{{'org.mirket.design-build':{recipe} if owned else ''}}}})
print(json.dumps(result[0] if '--format' in args else result))
"#,
            images = serde_json::to_string(
                &registry["images"]
                    .as_object()
                    .unwrap()
                    .values()
                    .collect::<Vec<_>>()
            )
            .unwrap(),
            recipe = serde_json::to_string(&recipe).unwrap()
        );
        fs::write(&binary, fixture).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        crate::tools::register(&paths, "podman", &binary).unwrap();
        let engine = Engine::resolve(&paths).unwrap();
        prepare_on_engine(&paths, &[], &engine, &registry).unwrap();
        let before = fs::read(receipt_path(&paths)).unwrap();
        assert_eq!(local_receipt(&paths).unwrap().images.len(), 2);
        assert!(!state_path(&paths).exists());
        prepare_on_engine(&paths, &[], &engine, &registry).unwrap();
        assert_eq!(fs::read(receipt_path(&paths)).unwrap(), before);
        fs::write(paths.home.join("calls"), "").unwrap();
        fs::write(receipt_path(&paths), "corrupt").unwrap();
        assert!(prepare_on_engine(&paths, &[], &engine, &registry).is_err());
        assert_eq!(fs::read_to_string(paths.home.join("calls")).unwrap(), "");
    }

    #[test]
    fn receipt_accepts_only_the_reviewed_design_image_set() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_owned()).unwrap();
        let id = format!("sha256:{}", "a".repeat(64));
        let mut receipt = Receipt {
            schema_version: 1,
            owner: owner(&paths),
            registry_sha256: registry_hash(),
            images: ["node-build", "design-tools"]
                .into_iter()
                .map(|alias| (alias.into(), id.clone()))
                .collect(),
        };
        publish_receipt(&paths, &receipt_path(&paths), &receipt).unwrap();
        let before = fs::read(receipt_path(&paths)).unwrap();
        assert!(require_prepared(&selected_groups(&[]).unwrap(), &receipt).is_ok());
        receipt.images.insert("unexpected".into(), id);
        assert!(validate_receipt(&paths, &receipt).is_err());
        assert!(publish_receipt(&paths, &receipt_path(&paths), &receipt).is_err());
        assert_eq!(fs::read(receipt_path(&paths)).unwrap(), before);
    }

    #[test]
    fn only_core_group_is_supported() {
        assert_eq!(
            selected_groups(&[]).unwrap(),
            BTreeSet::from(["core".into()])
        );
        assert_eq!(
            selected_groups(&["core".into()]).unwrap(),
            BTreeSet::from(["core".into()])
        );
        for group in ["all", "optional", "invented"] {
            assert!(selected_groups(&[group.into()]).is_err());
        }
        assert!(validate_group_memory(&selected_groups(&[]).unwrap(), 2559 * 1024 * 1024).is_err());
        assert!(validate_group_memory(&selected_groups(&[]).unwrap(), 2560 * 1024 * 1024).is_ok());
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
        assert!(!tools.contains_key("hyperframes_render"));
        assert_eq!(tools["typst_compile"], Duration::from_secs(25 * 60));
    }

    #[test]
    fn occupied_host_port_is_reported_before_startup_effects() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mapping = format!("{}:3000", listener.local_addr().unwrap());
        let error = preflight_host_port("design-tools", &mapping).unwrap_err();
        assert!(format!("{error:#}").contains("design-tools requires free host address"));
        drop(listener);
        preflight_host_port("design-tools", &mapping).unwrap();
    }

    #[test]
    fn renderers_publish_no_host_ports() {
        assert!(allocate_published_ports().unwrap().is_empty());
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
    fn tools_build_closure_covers_every_runtime_module() {
        let authored: BTreeSet<String> =
            fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/design/tools"))
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .filter(|name| name.ends_with(".mjs"))
                .map(|name| format!("tools/{name}"))
                .collect();
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
            "tools/browser.json",
            "tools/NOTICE.md",
        ] {
            assert!(staged.contains(required), "missing build input: {required}");
        }
    }
}
