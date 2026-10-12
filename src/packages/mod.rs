//! Reviewed MCP packages. Setup owns activation; this module owns pinned content.
mod archive;
pub mod container;
mod dmg;
mod embedded;
mod services;
mod store;

use crate::{package_mcp::LaunchSpec, paths::Paths, util};
use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

static NOTICES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/docs/licenses/packages");
const OPTIONAL: &str = include_str!("../../registry/packages-optional.json");
const EDITORS: &str = include_str!("../../registry/packages-editors.json");
const ENGINES: &str = include_str!("../../registry/packages-engines.json");
const AXE: &str = include_str!("../../registry/packages-axe-core.json");
const HYPERFRAMES: &str = include_str!("../../registry/packages-hyperframes.json");
const MOTION_CANVAS: &str = include_str!("../../registry/packages-motion-canvas.json");
const PENPOT: &str = include_str!("../../registry/packages-penpot.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub kind: String,
    #[serde(default)]
    pub adapter: Option<crate::editor_mcp::Editor>,
    pub status: String,
    #[serde(default)]
    pub reason: Option<String>,
    pub summary: String,
    pub license: String,
    pub source: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub notices: Vec<String>,
    #[serde(default)]
    pub common: Vec<Artifact>,
    /// Reviewed first-party files compiled from runtime/packages into this CLI.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embedded: Vec<Member>,
    /// Reviewed, digest-pinned local service closure. Explicit CLI lifecycle only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<services::Service>,
    /// Verified Linux execution closure. The native CLI remains the controller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<container::Profile>,
    #[serde(default)]
    pub platforms: BTreeMap<String, Platform>,
    #[serde(default)]
    pub launch: Launch,
    #[serde(default)]
    pub tools: ToolPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    pub executable: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Launch {
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPolicy {
    #[serde(default)]
    pub allowed: Option<BTreeSet<String>>,
    #[serde(default)]
    pub denied: BTreeSet<String>,
    #[serde(default = "no_policy")]
    pub input_policy: String,
}
fn no_policy() -> String {
    "none".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub url: String,
    pub sha256: String,
    pub format: String,
    pub max_download_bytes: u64,
    pub max_unpacked_bytes: u64,
    #[serde(default)]
    pub strip_prefix: Option<String>,
    #[serde(default)]
    pub destination: String,
    #[serde(default)]
    pub files: Vec<Member>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub path: String,
    pub destination: String,
    #[serde(default)]
    pub executable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema_version: u32,
    packages: Vec<Package>,
}

#[derive(Debug)]
pub struct Registry {
    packages: BTreeMap<String, Package>,
}

impl Registry {
    pub fn embedded() -> Result<Self> {
        Self::parse(&[
            OPTIONAL,
            EDITORS,
            ENGINES,
            AXE,
            HYPERFRAMES,
            MOTION_CANVAS,
            PENPOT,
        ])
    }
    fn parse(documents: &[&str]) -> Result<Self> {
        let mut packages = BTreeMap::new();
        for document in documents {
            let doc: Document =
                serde_json::from_str(document).context("invalid package catalog")?;
            if doc.schema_version != 1 {
                bail!("unsupported package catalog schema");
            }
            for package in doc.packages {
                validate_package(&package)?;
                if packages.insert(package.id.clone(), package).is_some() {
                    bail!("duplicate package ID");
                }
            }
        }
        let registry = Self { packages };
        for id in registry.packages.keys() {
            registry.order(std::slice::from_ref(id))?;
        }
        Ok(registry)
    }
    pub fn get(&self, id: &str) -> Result<&Package> {
        self.packages
            .get(id)
            .with_context(|| format!("unknown package: {id}; use mirket package list"))
    }
    pub fn list(&self) -> Value {
        json!({"packages":self.packages.values().filter(|p| p.kind!="runtime").map(|p|json!({"id":p.id,"kind":p.kind,"adapter":p.adapter,"name":p.name,"version":p.version,"status":p.status,"reason":p.reason,"summary":p.summary,"license":p.license,"dependencies":p.dependencies,"platforms":p.container.as_ref().map(|c|c.verified_hosts.clone()).unwrap_or_else(||p.platforms.keys().cloned().collect()),"payload_platforms":p.platforms.keys().collect::<Vec<_>>(),"execution":if p.container.is_some(){"offline-podman"}else if p.service.is_some(){"podman-services-native-browser"}else{"native"}})).collect::<Vec<_>>(),
            "runtime_ownership":"The CLI and client launchers are native. Reviewed OCI profiles enforce workload boundaries; native package paths use host permissions. Payload pins alone do not establish platform readiness.",
            "downloads":"Only explicit setup downloads pinned artifacts; serving never installs software."})
    }
    pub fn validate_selection(&self, ids: &[String]) -> Result<()> {
        if ids.len() > 32 {
            bail!("at most 32 packages may be selected");
        }
        let mut seen = BTreeSet::new();
        let target = crate::update::target_triple()?;
        for id in ids {
            let package = self.get(id)?;
            if let Some(profile) = &package.container
                && !profile.verified_hosts.iter().any(|host| host == target)
            {
                bail!(
                    "package {id} OCI controller is held on {target}; Linux payload pins do not establish host readiness"
                );
            }
            if !seen.insert(id) {
                bail!("package selection repeats {id}");
            }
            if !matches!(package.kind.as_str(), "mcp" | "editor" | "engine") {
                bail!(
                    "select an MCP, editor or engine package; runtimes are resolved as dependencies"
                );
            }
        }
        let mut checked = BTreeSet::new();
        for selected in ids {
            let execution_target = if self.get(selected)?.container.is_some() {
                container::target()
            } else {
                target
            };
            for id in self.order(std::slice::from_ref(selected))? {
                if !checked.insert((id.clone(), execution_target)) {
                    continue;
                }
                let package = self.get(&id)?;
                if package.status != "ready" {
                    bail!(
                        "package {id} is held: {}",
                        package
                            .reason
                            .as_deref()
                            .unwrap_or("packaging verification is incomplete")
                    );
                }
                if !package.platforms.contains_key(execution_target) {
                    bail!("package {id} has no reviewed package for {execution_target}");
                }
            }
        }
        Ok(())
    }
    fn order(&self, selected: &[String]) -> Result<Vec<String>> {
        fn visit(
            reg: &Registry,
            id: &str,
            active: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
            out: &mut Vec<String>,
        ) -> Result<()> {
            if done.contains(id) {
                return Ok(());
            }
            if !active.insert(id.into()) {
                bail!("package dependency cycle at {id}");
            }
            for dep in &reg.get(id)?.dependencies {
                visit(reg, dep, active, done, out)?;
            }
            active.remove(id);
            done.insert(id.into());
            out.push(id.into());
            Ok(())
        }
        let mut out = Vec::new();
        let mut active = BTreeSet::new();
        let mut done = BTreeSet::new();
        for id in selected {
            visit(self, id, &mut active, &mut done, &mut out)?;
        }
        Ok(out)
    }
    fn identity(&self, id: &str, target: &str) -> Result<String> {
        let package = self.get(id)?;
        let platform = package
            .platforms
            .get(target)
            .context("package target is unavailable")?;
        let mut bytes = serde_json::to_vec(&(package, platform, target))?;
        for name in &package.notices {
            bytes.extend_from_slice(notice(name)?);
        }
        embedded::append_identity(&mut bytes, &package.embedded)?;
        if package.container.is_some() {
            container::append_identity(&mut bytes);
        }
        for dep in &package.dependencies {
            bytes.extend_from_slice(self.identity(dep, target)?.as_bytes());
        }
        Ok(util::sha256(&bytes))
    }
    #[cfg(test)]
    fn root(&self, paths: &Paths, id: &str) -> Result<PathBuf> {
        self.root_for(paths, id, crate::update::target_triple()?)
    }
    fn root_for(&self, paths: &Paths, id: &str, target: &str) -> Result<PathBuf> {
        Ok(paths
            .root
            .join("packages")
            .join("store")
            .join(self.identity(id, target)?))
    }
}

fn notice(path: &str) -> Result<&'static [u8]> {
    archive::normal_path(path)?;
    NOTICES
        .get_file(path)
        .map(|f| f.contents())
        .with_context(|| format!("missing embedded package notice {path}"))
}
pub(crate) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn validate_package(package: &Package) -> Result<()> {
    if !valid_id(&package.id)
        || !matches!(
            package.kind.as_str(),
            "mcp" | "editor" | "engine" | "runtime"
        )
        || !matches!(package.status.as_str(), "ready" | "held")
    {
        bail!("invalid package identity, kind or status");
    }
    if matches!(package.kind.as_str(), "editor" | "engine") != package.adapter.is_some() {
        bail!(
            "editor and engine packages require an explicit Mirket-owned adapter; MCP servers and runtimes cannot declare one"
        );
    }
    if package.version.is_empty()
        || package.license.is_empty()
        || !package.source.starts_with("https://")
    {
        bail!("package version, source and license are required");
    }
    if package.status == "held" && package.reason.as_deref().is_none_or(str::is_empty) {
        bail!("held package requires a reason");
    }
    for path in &package.notices {
        notice(path)?;
    }
    for (target, platform) in &package.platforms {
        if ![
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "aarch64-unknown-linux-gnu",
            "x86_64-unknown-linux-gnu",
            "aarch64-pc-windows-msvc",
            "x86_64-pc-windows-msvc",
        ]
        .contains(&target.as_str())
        {
            bail!("invalid package platform {target}");
        }
        if platform.executable != "{node}" {
            archive::normal_path(&platform.executable)?;
        }
        for artifact in &platform.artifacts {
            archive::validate_artifact(artifact)?;
        }
    }
    for artifact in &package.common {
        archive::validate_artifact(artifact)?;
    }
    embedded::validate(&package.embedded)?;
    if let Some(service) = &package.service {
        if package.id != "penpot"
            || package
                .platforms
                .keys()
                .any(|t| t != "aarch64-apple-darwin")
        {
            bail!("local service profile is only verified for Penpot on macOS ARM64");
        }
        services::validate(service)?;
    }
    if let Some(profile) = &package.container {
        container::validate(profile, &package.id)?;
    }
    if package.dependencies.iter().any(|s| !valid_id(s)) {
        bail!("invalid dependency ID");
    }
    if package.launch.args.len() > 128 || package.launch.env.len() > 64 {
        bail!("package launch is too large");
    }
    if !matches!(package.tools.input_policy.as_str(), "none" | "browser") {
        bail!("unknown package input policy");
    }
    Ok(())
}

/// Store content is prepared before the existing setup transaction activates it.
pub fn prepare(paths: &Paths, selected: &[String]) -> Result<()> {
    let registry = Registry::embedded()?;
    registry.validate_selection(selected)?;
    for id in selected {
        let package = registry.get(id)?;
        let target = if package.container.is_some() {
            container::target()
        } else {
            crate::update::target_triple()?
        };
        for dep in registry.order(std::slice::from_ref(id))? {
            services::prepare(paths, registry.get(&dep)?)?;
            store::install_target(paths, &registry, &dep, target)?;
        }
    }
    Ok(())
}

const MACHINE_SHA: &str = "633971788fcfe7df04f49da53f06c98b28933c5df2f684d57c2f9831411cac77";
pub fn verify_machine_image(path: &Path) -> Result<bool> {
    Ok(store::file_identity(path, 1024 * 1024 * 1024)?.sha256 == MACHINE_SHA)
}
pub fn machine_image(paths: &Paths) -> Result<PathBuf> {
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .timeout(std::time::Duration::from_secs(240))
        .build()?;
    store::download(
        paths,
        &Artifact {
            url: format!("https://quay.io/v2/podman/machine-os/blobs/sha256:{MACHINE_SHA}"),
            sha256: MACHINE_SHA.into(),
            format: "file".into(),
            max_download_bytes: 1024 * 1024 * 1024,
            max_unpacked_bytes: 0,
            strip_prefix: None,
            destination: String::new(),
            files: Vec::new(),
        },
        &client,
    )
}

pub fn doctor(paths: &Paths, selected: &[String]) -> Result<Value> {
    let registry = Registry::embedded()?;
    registry.validate_selection(selected)?;
    let mut generations = BTreeSet::new();
    for selected in selected {
        let target = if registry.get(selected)?.container.is_some() {
            container::target()
        } else {
            crate::update::target_triple()?
        };
        for id in registry.order(std::slice::from_ref(selected))? {
            generations.insert((id, target));
        }
    }
    let checks=generations.into_iter().map(|(id,target)| {
        let checked = (|| {
            let package = registry.get(&id)?;
            services::verify(paths, package)?;
            let receipt = store::verify_target(paths, &registry, &id,target)?;
            let health = package.service.as_ref().map(|_|services::status(paths,package)).transpose()?;
            Ok::<_, anyhow::Error>((receipt, health))
        })();
        match checked {
            Ok((receipt,health))=>json!({"id":id,"ok":true,"files":receipt.files.len(),"version":registry.get(&id).map(|p|&p.version).ok(),"status":"local_receipt_verified","runtime_status":"not_invoked","service":health}),
            Err(error)=>json!({"id":id,"ok":false,"error":format!("{error:#}")}),
        }
    }).collect::<Vec<_>>();
    Ok(
        json!({"ok":checks.iter().all(|c|c["ok"]==true),"checks":checks,"activation":"Installed files match a locally trusted receipt produced from digest-verified archives. Receipts are user-owned, not attestations. Use mirket package probe and a real task for invocation evidence."}),
    )
}

fn substitute(value: &str, variables: &BTreeMap<&str, String>) -> Result<String> {
    let mut remaining = value;
    let mut out = String::new();
    while let Some(start) = remaining.find('{') {
        let (literal, tail) = remaining.split_at(start);
        if literal.contains('}') {
            bail!("invalid package launch template");
        }
        out.push_str(literal);
        let end = tail
            .find('}')
            .context("unclosed package launch placeholder")?;
        let name = &tail[1..end];
        out.push_str(
            variables
                .get(name)
                .context("unresolved package launch placeholder")?,
        );
        remaining = &tail[end + 1..];
    }
    if remaining.contains('}') {
        bail!("invalid package launch template");
    }
    out.push_str(remaining);
    Ok(out)
}

/// Resolve the installed executable. Editor applications and engines use the
/// Mirket-owned adapter, never connected as though they were upstream MCP servers.
pub fn launch(paths: &Paths, id: &str, project: &Path) -> Result<LaunchSpec> {
    let saved = crate::install::saved_options(paths)?.context("run mirket setup first")?;
    if !saved.packages.iter().any(|selected| selected == id) {
        bail!("package {id} is not selected; use mirket package install {id}");
    }
    let registry = Registry::embedded()?;
    registry.validate_selection(&saved.packages)?;
    let selected = vec![id.into()];
    let uses_container = registry.get(id)?.container.is_some();
    let target = if uses_container {
        container::target()
    } else {
        crate::update::target_triple()?
    };
    let mut selected_receipt = None;
    for dep in registry.order(&selected)? {
        let receipt = store::verify_target(paths, &registry, &dep, target)?;
        if dep == id {
            selected_receipt = Some(receipt);
        }
    }
    let container_runtime = if uses_container {
        Some(container::runtime(
            paths,
            &registry,
            id,
            selected_receipt.context("selected generation")?,
        )?)
    } else {
        None
    };
    let project = crate::paths::canonical_project(project)?;
    let package = registry.get(id)?;
    let root = registry.root_for(paths, id, target)?;
    let data = paths.root.join("packages").join("data").join(id);
    util::ensure_directory(&data)?;
    let home = data.join("home");
    util::ensure_directory(&home)?;
    let mut variables = BTreeMap::from([
        ("root", root.to_string_lossy().into_owned()),
        ("data", data.to_string_lossy().into_owned()),
        ("project", project.to_string_lossy().into_owned()),
    ]);
    let mut runtime_paths = Vec::new();
    for (dep, placeholder) in [
        ("nodejs", "node"),
        ("chromium", "chromium"),
        ("ffmpeg", "ffmpeg"),
    ] {
        if registry.order(&selected)?.iter().any(|id| id == dep) {
            let p = registry.get(dep)?;
            let binary = registry
                .root_for(paths, dep, target)?
                .join(&p.platforms[target].executable);
            runtime_paths.push(
                binary
                    .parent()
                    .context("runtime executable parent")?
                    .to_path_buf(),
            );
            variables.insert(placeholder, binary.to_string_lossy().into_owned());
        }
    }
    if variables.contains_key("ffmpeg") {
        let probe = registry
            .root_for(paths, "ffmpeg", target)?
            .join("bin/ffprobe");
        if probe.is_file() {
            variables.insert("ffprobe", probe.to_string_lossy().into_owned());
        }
    }
    let executable = &package.platforms[target].executable;
    let command = if executable.starts_with('{') {
        PathBuf::from(substitute(executable, &variables)?)
    } else {
        root.join(executable)
    };
    util::reject_symlink_components(&command)?;
    let mut env = BTreeMap::new();
    // Credentials, NODE_OPTIONS, user profiles and proxies
    // are intentionally absent. This controls environment, not OS permissions.
    for name in [
        "PATH",
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
        "TMPDIR",
        "TEMP",
        "TMP",
        "LANG",
        "LC_ALL",
        "DISPLAY",
        "WAYLAND_DISPLAY",
    ] {
        if let Ok(value) = std::env::var(name) {
            env.insert(name.into(), value);
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        runtime_paths.extend(std::env::split_paths(&path));
    }
    if !runtime_paths.is_empty() {
        env.insert(
            "PATH".into(),
            std::env::join_paths(runtime_paths)?
                .to_string_lossy()
                .into_owned(),
        );
    }
    env.insert("HOME".into(), home.to_string_lossy().into_owned());
    env.insert("USERPROFILE".into(), home.to_string_lossy().into_owned());
    env.insert(
        "XDG_CACHE_HOME".into(),
        data.join("cache").to_string_lossy().into_owned(),
    );
    env.insert("DO_NOT_TRACK".into(), "1".into());
    for (key, value) in &package.launch.env {
        env.insert(key.clone(), substitute(value, &variables)?);
    }
    if package.service.is_some() {
        env.extend(services::launch_env(paths, package)?);
    }
    Ok(LaunchSpec {
        container: container_runtime,
        startup_timeout: std::time::Duration::from_secs(30),
        command,
        args: package
            .launch
            .args
            .iter()
            .map(|v| substitute(v, &variables))
            .collect::<Result<_>>()?,
        env,
        cwd: project,
        allowed_tools: package.tools.allowed.clone(),
        denied_tools: package.tools.denied.clone(),
        input_policy: package.tools.input_policy.clone(),
    })
}

/// CLI probe/call use a real stdio client for upstream servers and owned
/// adapters. Persistent host sessions enter `package serve` directly.
pub fn client_launch(paths: &Paths, id: &str, project: &Path) -> Result<LaunchSpec> {
    let registry = Registry::embedded()?;
    let package = registry.get(id)?;
    if package.adapter.is_none() {
        return launch(paths, id, project);
    }
    let saved = crate::install::saved_options(paths)?.context("run mirket setup first")?;
    if !saved.packages.iter().any(|selected| selected == id) {
        bail!("package {id} is not selected; use mirket package install {id}");
    }
    registry.validate_selection(&saved.packages)?;
    let cwd = crate::paths::canonical_project(project)?;
    let mut env = BTreeMap::from([
        ("PATH".into(), "/usr/bin:/bin:/usr/sbin:/sbin".into()),
        ("LANG".into(), "en_US.UTF-8".into()),
    ]);
    if package
        .adapter
        .is_some_and(|editor| editor != crate::editor_mcp::Editor::Inkscape)
    {
        // CLI probe/call and directly hosted serve use the same host-selected
        // temporary storage without inheriting unrelated host environment.
        env.insert(
            "TMPDIR".into(),
            std::env::temp_dir()
                .canonicalize()
                .context("resolve the host temporary directory for the local design runtime")?
                .to_str()
                .context("design runtime temporary directory must be UTF-8")?
                .to_owned(),
        );
    }
    // The executing `serve` child verifies every native package byte before it
    // starts the editor. Repeating that full inventory in the CLI client doubles
    // startup cost without adding a trust boundary. Reuse this CLI's executable.
    Ok(LaunchSpec {
        command: std::env::current_exe()?.canonicalize()?,
        container: None,
        startup_timeout: std::time::Duration::from_secs(90),
        args: vec![
            "--home".into(),
            paths.home.to_string_lossy().into_owned(),
            "package".into(),
            "serve".into(),
            id.into(),
            "--project".into(),
            cwd.to_string_lossy().into_owned(),
        ],
        env,
        cwd,
        allowed_tools: package.tools.allowed.clone(),
        denied_tools: package.tools.denied.clone(),
        input_policy: "none".into(),
    })
}

/// Receipt migration removes only explicitly retired selections and settings.
/// Mutable package data and user-selected external files are never deleted here.
pub(crate) fn is_retired_id(id: &str) -> bool {
    matches!(
        id,
        "semiotic"
            | "agent-device"
            | "safe-docx"
            | "markitdown"
            | "dbhub"
            | "doc-scraper"
            | "jscpd"
            | "glaxnimate"
            | "helios"
            | "openpencil"
    )
}

pub(crate) fn normalize_saved_selection(options: &mut crate::install::SetupOptions) -> Vec<String> {
    let removed = options
        .packages
        .iter()
        .filter(|id| is_retired_id(id))
        .cloned()
        .collect();
    options.packages.retain(|id| !is_retired_id(id));
    options.package_settings.retain(|id, _| !is_retired_id(id));
    removed
}

pub fn service_start(paths: &Paths, id: &str, port: Option<u16>) -> Result<Value> {
    let selected = crate::install::saved_options(paths)?.context("run mirket setup first")?;
    if !selected.packages.iter().any(|p| p == id) {
        bail!("package is not selected; use mirket package install {id}");
    }
    services::start(paths, &Registry::embedded()?, id, port)
}
pub fn service_stop(paths: &Paths, id: &str, purge_data: bool) -> Result<Value> {
    services::stop(paths, id, purge_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_has_reviewed_profiles_and_no_unpinned_downloads() {
        let registry = Registry::embedded().unwrap();
        for id in [
            "playwright",
            "inkscape",
            "axe-core",
            "hyperframes",
            "motion-canvas",
            "penpot",
        ] {
            assert!(registry.get(id).is_ok(), "{id}");
        }
        assert_eq!(
            registry.get("playwright").unwrap().tools.input_policy,
            "browser"
        );
    }
    #[test]
    fn invalid_package_and_unknown_placeholder_fail_closed() {
        assert!(!valid_id("../node"));
        assert!(!valid_id("ABC"));
        assert!(substitute("{secret}", &BTreeMap::new()).is_err());
        assert_eq!(
            substitute(
                "{root}/tool",
                &BTreeMap::from([("root", "/folder/{literal}".into())])
            )
            .unwrap(),
            "/folder/{literal}/tool"
        );
    }
    #[test]
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    fn mixed_selection_resolves_each_execution_target_without_native_render_fallback() {
        let registry = Registry::embedded().unwrap();
        registry
            .validate_selection(&[
                "motion-canvas".into(),
                "hyperframes".into(),
                "axe-core".into(),
            ])
            .unwrap();
        let mut missing = Registry {
            packages: registry.packages.clone(),
        };
        missing
            .packages
            .get_mut("nodejs")
            .unwrap()
            .platforms
            .remove(container::target());
        assert!(
            missing
                .validate_selection(&["motion-canvas".into(), "axe-core".into()])
                .is_err()
        );
        let mut missing = Registry {
            packages: registry.packages.clone(),
        };
        missing
            .packages
            .get_mut("nodejs")
            .unwrap()
            .platforms
            .remove("aarch64-apple-darwin");
        missing
            .validate_selection(&["motion-canvas".into()])
            .unwrap();
        assert!(
            missing
                .validate_selection(&["motion-canvas".into(), "axe-core".into()])
                .is_err()
        );
    }
    #[test]
    fn held_profile_fails_before_resolution_or_installation() {
        let mut package = Registry::embedded()
            .unwrap()
            .get("playwright")
            .unwrap()
            .clone();
        package.dependencies.clear();
        package.status = "held".into();
        package.reason = Some("runtime verification is incomplete".into());
        let document = json!({"schema_version":1,"packages":[package]}).to_string();
        let registry = Registry::parse(&[&document]).unwrap();
        assert!(
            registry
                .validate_selection(&["playwright".into()])
                .unwrap_err()
                .to_string()
                .contains("is held")
        );
    }
    #[test]
    fn owned_engine_selection_requires_an_adapter_and_preserves_runtime_boundary() {
        let mut package = Registry::embedded()
            .unwrap()
            .get("playwright")
            .unwrap()
            .clone();
        package.dependencies.clear();
        package.kind = "engine".into();
        assert!(validate_package(&package).is_err());
        package.adapter = Some(crate::editor_mcp::Editor::Inkscape);
        validate_package(&package).unwrap();
        let document = json!({"schema_version":1,"packages":[package.clone()]}).to_string();
        let registry = Registry::parse(&[&document]).unwrap();
        registry.validate_selection(&[package.id.clone()]).unwrap();
        for kind in ["runtime", "mcp"] {
            package.kind = kind.into();
            assert!(validate_package(&package).is_err());
        }
        package.kind = "editor".into();
        validate_package(&package).unwrap();
        package.adapter = None;
        assert!(validate_package(&package).is_err());
    }
    #[test]
    fn missing_embedded_source_is_refused_during_registry_validation() {
        let mut package = Registry::embedded()
            .unwrap()
            .get("playwright")
            .unwrap()
            .clone();
        package.dependencies.clear();
        package.embedded = vec![Member {
            path: "absent/runner.mjs".into(),
            destination: "mirket/runner.mjs".into(),
            executable: false,
        }];
        let document = json!({"schema_version":1,"packages":[package]}).to_string();
        assert!(
            Registry::parse(&[&document])
                .unwrap_err()
                .to_string()
                .contains("missing embedded package source")
        );
    }
    #[test]
    fn retired_saved_choices_are_removed_without_admitting_unknown_ids_or_touching_data() {
        let home = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let preserved = home.path().join("documents/customer.docx");
        std::fs::create_dir_all(preserved.parent().unwrap()).unwrap();
        std::fs::write(&preserved, b"private user document").unwrap();
        let mut options = crate::install::SetupOptions {
            packages: vec![
                "playwright".into(),
                "safe-docx".into(),
                "dbhub".into(),
                "unknown-user-package".into(),
            ],
            package_settings: BTreeMap::from([
                ("safe-docx".into(), json!({"document": preserved})),
                (
                    "dbhub".into(),
                    json!({"database":"/external/private.sqlite"}),
                ),
                ("unknown-user-package".into(), json!({"keep": true})),
            ]),
            ..Default::default()
        };
        assert_eq!(
            normalize_saved_selection(&mut options),
            ["safe-docx", "dbhub"]
        );
        assert_eq!(options.packages, ["playwright", "unknown-user-package"]);
        assert_eq!(options.package_settings.len(), 1);
        assert_eq!(std::fs::read(preserved).unwrap(), b"private user document");
        assert!(
            Registry::embedded()
                .unwrap()
                .validate_selection(&options.packages)
                .is_err()
        );
        for id in [
            "semiotic",
            "agent-device",
            "safe-docx",
            "markitdown",
            "dbhub",
            "doc-scraper",
            "jscpd",
            "glaxnimate",
            "helios",
            "openpencil",
            "python",
        ] {
            assert!(
                Registry::embedded().unwrap().get(id).is_err(),
                "{id} is exposed"
            );
        }
    }
}
