//! Reviewed MCP packages. Setup owns activation; this module owns pinned content.
mod archive;
mod dmg;
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
const NATIVE: &str = include_str!("../../registry/packages-native.json");
const OPTIONAL: &str = include_str!("../../registry/packages-optional.json");
const PYTHON: &str = include_str!("../../registry/packages-python.json");
const EDITORS: &str = include_str!("../../registry/packages-editors.json");

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
        Self::parse(&[NATIVE, OPTIONAL, PYTHON, EDITORS])
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
        json!({"packages":self.packages.values().filter(|p| p.kind!="runtime").map(|p|json!({"id":p.id,"kind":p.kind,"adapter":p.adapter,"name":p.name,"version":p.version,"status":p.status,"reason":p.reason,"summary":p.summary,"license":p.license,"dependencies":p.dependencies,"platforms":p.platforms.keys().collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "runtime_ownership":"Packages execute under host permissions. A license label is not a dependency audit or sandbox.",
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
            if !seen.insert(id) {
                bail!("package selection repeats {id}");
            }
            if !matches!(package.kind.as_str(), "mcp" | "editor") {
                bail!("select an MCP or editor package; runtimes are resolved as dependencies");
            }
        }
        for id in self.order(ids)? {
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
            if !package.platforms.contains_key(target) {
                bail!("package {id} has no reviewed package for {target}");
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
        for dep in &package.dependencies {
            bytes.extend_from_slice(self.identity(dep, target)?.as_bytes());
        }
        Ok(util::sha256(&bytes))
    }
    fn root(&self, paths: &Paths, id: &str) -> Result<PathBuf> {
        Ok(paths
            .root
            .join("packages")
            .join("store")
            .join(self.identity(id, crate::update::target_triple()?)?))
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
        || !matches!(package.kind.as_str(), "mcp" | "editor" | "runtime")
        || !matches!(package.status.as_str(), "ready" | "held")
    {
        bail!("invalid package identity, kind or status");
    }
    if (package.kind == "editor") != package.adapter.is_some() {
        bail!(
            "editor packages require an explicit Mirket-owned adapter; MCP servers and runtimes cannot declare one"
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
        if !["{node}", "{python}"].contains(&platform.executable.as_str()) {
            archive::normal_path(&platform.executable)?;
        }
        for artifact in &platform.artifacts {
            archive::validate_artifact(artifact)?;
        }
    }
    for artifact in &package.common {
        archive::validate_artifact(artifact)?;
    }
    if package.dependencies.iter().any(|s| !valid_id(s)) {
        bail!("invalid dependency ID");
    }
    if package.launch.args.len() > 128 || package.launch.env.len() > 64 {
        bail!("package launch is too large");
    }
    if !matches!(
        package.tools.input_policy.as_str(),
        "none"
            | "browser"
            | "jscpd-local"
            | "markitdown-local"
            | "safe-docx-local"
            | "semiotic-svg"
            | "dbhub-readonly"
    ) {
        bail!("unknown package input policy");
    }
    Ok(())
}

/// Store content is prepared before the existing setup transaction activates it.
pub fn prepare(paths: &Paths, selected: &[String]) -> Result<()> {
    let registry = Registry::embedded()?;
    registry.validate_selection(selected)?;
    for id in registry.order(selected)? {
        store::install(paths, &registry, &id)?;
    }
    Ok(())
}

pub fn doctor(paths: &Paths, selected: &[String]) -> Result<Value> {
    let registry = Registry::embedded()?;
    registry.validate_selection(selected)?;
    let settings = crate::install::saved_options(paths)?
        .map(|o| o.package_settings)
        .unwrap_or_default();
    let checks=registry.order(selected)?.into_iter().map(|id| {
        let readiness = if id=="dbhub" {
            settings.get(&id).and_then(|s|s.get("database")).and_then(Value::as_str)
                .context("DBHub needs mirket package configure dbhub --database /absolute/database.sqlite")
                .and_then(|p|sqlite_database(Path::new(p)).map(|_|()))
        } else { Ok(()) };
        match readiness.and_then(|()|store::verify(paths,&registry,&id)) {
            Ok(receipt)=>json!({"id":id,"ok":true,"files":receipt.files.len(),"version":registry.get(&id).map(|p|&p.version).ok(),"status":"local_receipt_verified","runtime_status":"not_invoked"}),
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

/// Resolve the installed executable. Editor applications are consumed by the
/// Mirket-owned adapter, never connected as though they were upstream MCP servers.
pub fn launch(paths: &Paths, id: &str, project: &Path) -> Result<LaunchSpec> {
    let saved = crate::install::saved_options(paths)?.context("run mirket setup first")?;
    if !saved.packages.iter().any(|selected| selected == id) {
        bail!("package {id} is not selected; use mirket package install {id}");
    }
    let registry = Registry::embedded()?;
    registry.validate_selection(&saved.packages)?;
    let selected = vec![id.into()];
    for dep in registry.order(&selected)? {
        store::verify(paths, &registry, &dep)?;
    }
    let project = crate::paths::canonical_project(project)?;
    let package = registry.get(id)?;
    let target = crate::update::target_triple()?;
    let root = registry.root(paths, id)?;
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
        ("python", "python"),
        ("chromium", "chromium"),
    ] {
        if registry.order(&selected)?.iter().any(|id| id == dep) {
            let p = registry.get(dep)?;
            let binary = registry
                .root(paths, dep)?
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
    let config = configuration(paths, id, &data, &saved.package_settings)?;
    if let Some(config) = config {
        variables.insert("config", config.to_string_lossy().into_owned());
    }
    let executable = &package.platforms[target].executable;
    let command = if executable.starts_with('{') {
        PathBuf::from(substitute(executable, &variables)?)
    } else {
        root.join(executable)
    };
    util::reject_symlink_components(&command)?;
    let mut env = BTreeMap::new();
    // Credentials, NODE_OPTIONS, Python startup hooks, user profiles and proxies
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
    Ok(LaunchSpec {
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
        read_only_sqlite: if id == "dbhub" {
            Some(sqlite_database(Path::new(
                saved
                    .package_settings
                    .get(id)
                    .and_then(|v| v.get("database"))
                    .and_then(Value::as_str)
                    .context("DBHub database is not configured")?,
            ))?)
        } else {
            None
        },
    })
}

/// CLI probe/call use a real stdio client for both upstream servers and owned
/// editor adapters. Persistent host sessions enter `package serve` directly.
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
    // The executing `serve` child verifies every native package byte before it
    // starts the editor. Repeating that full inventory in the CLI client doubles
    // startup cost without adding a trust boundary. Reuse this CLI's executable.
    Ok(LaunchSpec {
        command: std::env::current_exe()?.canonicalize()?,
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
        env: BTreeMap::from([
            ("PATH".into(), "/usr/bin:/bin:/usr/sbin:/sbin".into()),
            ("LANG".into(), "en_US.UTF-8".into()),
        ]),
        cwd,
        allowed_tools: package.tools.allowed.clone(),
        denied_tools: package.tools.denied.clone(),
        input_policy: "none".into(),
        read_only_sqlite: None,
    })
}

fn configuration(
    _paths: &Paths,
    id: &str,
    data: &Path,
    settings: &BTreeMap<String, Value>,
) -> Result<Option<PathBuf>> {
    match id {
        "dbhub" => {
            let database=settings.get(id).and_then(|s|s.get("database")).and_then(Value::as_str).context("DBHub needs a selected local database: mirket package configure dbhub --database /absolute/database.sqlite")?;
            let database = sqlite_database(Path::new(database))?;
            let dsn = sqlite_dsn(&database)?;
            let quoted = toml_edit::Value::from(dsn).to_string();
            let content = format!(
                "[[sources]]\nid = \"local\"\ndsn = {quoted}\n\n[[tools]]\nname = \"execute_sql\"\nsource = \"local\"\nreadonly = true\nmax_rows = 200\n\n[[tools]]\nname = \"search_objects\"\nsource = \"local\"\n"
            );
            Ok(Some(write_config(data, "toml", content.as_bytes())?))
        }
        "jscpd" => Ok(Some(write_config(data, "json", b"{\"semantic\":false}\n")?)),
        "doc-scraper" => {
            let site = settings
                .get(id)
                .and_then(|s| s.get("site"))
                .and_then(Value::as_str)
                .unwrap_or("https://modelcontextprotocol.io");
            let url = documentation_site(site)?;
            let key = if site == "https://modelcontextprotocol.io" {
                "mcp".into()
            } else {
                format!("site-{}", &util::sha256(site.as_bytes())[..12])
            };
            // JSON is valid YAML and avoids shell/YAML quoting for native paths.
            let content = json!({"output_base_dir":data.join("pages"),"state_dir":data.join("state"),"num_workers":2,"num_image_workers":1,"max_requests":4,"max_requests_per_host":2,"default_delay_per_host":"500ms","global_crawl_timeout":"2m","per_page_timeout":"30s","max_page_size_bytes":4194304,"skip_images":true,"enable_jsonl_output":true,"http_client_settings":{"timeout":"30s","max_idle_conns_per_host":2,"allow_private_networks":false},"sites":{key:{"start_urls":[site],"allowed_domain":url.host_str(),"allowed_path_prefix":"/","content_selector":"auto","max_depth":1,"respect_nofollow":true}}});
            Ok(Some(write_config(
                data,
                "yaml",
                &serde_json::to_vec_pretty(&content)?,
            )?))
        }
        _ => Ok(None),
    }
}

fn write_config(data: &Path, extension: &str, content: &[u8]) -> Result<PathBuf> {
    let path = data
        .join("configurations")
        .join(format!("{}.{}", util::sha256(content), extension));
    util::reject_symlink_components(&path)?;
    if path.exists() {
        if util::read_bounded(&path, 128 * 1024)? != content {
            bail!("generated package configuration has drifted");
        }
    } else {
        util::atomic_write(&path, content)?;
    }
    Ok(path)
}

fn sqlite_dsn(path: &Path) -> Result<String> {
    let text = path.to_str().context("DBHub database path must be UTF-8")?;
    // DBHub 1.5 uses URL.pathname without percent decoding. Reject paths it
    // would silently reinterpret rather than opening another database.
    if text
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, '%' | '?' | '#') || !c.is_ascii())
    {
        bail!("DBHub requires a database path without spaces, non-ASCII or URI escapes");
    }
    #[cfg(windows)]
    {
        let _ = text;
        bail!("DBHub SQLite path handling has not been verified on Windows");
    }
    #[cfg(not(windows))]
    {
        Ok(format!("sqlite://{text}"))
    }
}

fn documentation_site(site: &str) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(site)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        bail!("documentation site must be a credential-free HTTPS URL without a fragment");
    }
    Ok(url)
}

fn sqlite_database(path: &Path) -> Result<PathBuf> {
    use std::io::Read;
    if !path.is_absolute() {
        bail!("database path must be absolute");
    }
    util::reject_symlink_components(path)?;
    if !std::fs::symlink_metadata(path)?.is_file() {
        bail!("database must be a regular existing file");
    }
    let path = path.canonicalize()?;
    sqlite_dsn(&path)?;
    let mut header = [0u8; 16];
    std::fs::File::open(&path)?
        .read_exact(&mut header)
        .context("read SQLite header")?;
    if &header != b"SQLite format 3\0" {
        bail!("database does not have a SQLite file header");
    }
    Ok(path)
}

pub fn configure_options(
    options: &mut crate::install::SetupOptions,
    id: &str,
    database: Option<&Path>,
    site: Option<&str>,
) -> Result<()> {
    Registry::embedded()?.get(id)?;
    match id {
        "dbhub" => {
            let path = database.context("supply --database /absolute/database.sqlite")?;
            let path = sqlite_database(path)?;
            if site.is_some() {
                bail!("--site applies only to doc-scraper");
            }
            options
                .package_settings
                .insert(id.into(), json!({"database":path}));
        }
        "doc-scraper" => {
            if database.is_some() {
                bail!("--database applies only to dbhub");
            }
            let url =
                documentation_site(site.context("supply --site https://documentation-site")?)?;
            options
                .package_settings
                .insert(id.into(), json!({"site":url.as_str()}));
        }
        _ => bail!("package {id} has no user configuration"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn database_aliases_use_one_canonical_path_for_configuration_and_validation() {
        let root = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = root.path().join("inventory.sqlite");
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE inventory(count INTEGER);")
            .unwrap();
        drop(db);
        let alias = PathBuf::from(format!("/{}", path.display()));
        assert_eq!(sqlite_database(&alias).unwrap(), path);
        let mut options = crate::install::SetupOptions::default();
        configure_options(&mut options, "dbhub", Some(&alias), None).unwrap();
        assert_eq!(
            options.package_settings["dbhub"]["database"],
            path.to_str().unwrap()
        );
        let config = configuration(
            &Paths::new(root.path().into()).unwrap(),
            "dbhub",
            &root.path().join("data"),
            &options.package_settings,
        )
        .unwrap()
        .unwrap();
        let parsed = std::fs::read_to_string(config)
            .unwrap()
            .parse::<toml_edit::DocumentMut>()
            .unwrap();
        assert_eq!(
            parsed["sources"]
                .as_array_of_tables()
                .unwrap()
                .get(0)
                .unwrap()["dsn"]
                .as_str()
                .unwrap(),
            format!("sqlite://{}", path.display())
        );
    }
    #[test]
    fn catalog_has_reviewed_profiles_and_no_unpinned_downloads() {
        let registry = Registry::embedded().unwrap();
        for id in [
            "doc-scraper",
            "jscpd",
            "playwright",
            "dbhub",
            "markitdown",
            "safe-docx",
            "semiotic",
            "agent-device",
        ] {
            assert!(registry.get(id).is_ok(), "{id}");
        }
        assert_eq!(
            registry.get("jscpd").unwrap().tools.input_policy,
            "jscpd-local"
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
    fn held_profile_fails_before_resolution_or_installation() {
        let mut package = Registry::embedded().unwrap().get("jscpd").unwrap().clone();
        package.status = "held".into();
        package.reason = Some("runtime verification is incomplete".into());
        let document = json!({"schema_version":1,"packages":[package]}).to_string();
        let registry = Registry::parse(&[&document]).unwrap();
        assert!(
            registry
                .validate_selection(&["jscpd".into()])
                .unwrap_err()
                .to_string()
                .contains("is held")
        );
    }
}
