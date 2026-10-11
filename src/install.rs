use crate::{
    catalog::Catalog,
    paths::Paths,
    util::{atomic_write, read_bounded, reject_symlink_components, safe_relative, sha256},
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Table};

const MAX_FILE: u64 = 256 * 1024 * 1024;
const MAX_SETTINGS: u64 = 16 * 1024 * 1024;
const ABSENT_SETUP_TOKEN: &str = "absent";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Codex,
    Claude,
}
impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        })
    }
}
impl Target {
    fn instructions(self) -> &'static str {
        match self {
            Self::Codex => ".codex/AGENTS.md",
            Self::Claude => ".claude/CLAUDE.md",
        }
    }
    fn skills(self) -> &'static str {
        match self {
            Self::Codex => ".agents/skills",
            Self::Claude => ".claude/skills",
        }
    }
    fn config(self) -> &'static str {
        match self {
            Self::Codex => ".codex/config.toml",
            Self::Claude => ".claude.json",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SetupOptions {
    pub targets: Vec<Target>,
    pub microsoft_learn: bool,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub package_settings: BTreeMap<String, Value>,
}
impl Default for SetupOptions {
    fn default() -> Self {
        Self {
            targets: vec![Target::Codex, Target::Claude],
            microsoft_learn: false,
            packages: Vec::new(),
            package_settings: BTreeMap::new(),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct SetupReport {
    pub version: String,
    pub targets: Vec<Target>,
    pub files: usize,
    pub changed: usize,
    pub binary: PathBuf,
    pub packages: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    pub message: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    pub installed: bool,
    pub checks: Vec<DoctorCheck>,
    #[serde(skip)]
    pub packages: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManagedFile {
    sha256: String,
    executable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallState {
    format: u32,
    pub(crate) version: String,
    pub(crate) options: SetupOptions,
    files: BTreeMap<String, ManagedFile>,
    servers: BTreeMap<Target, BTreeMap<String, Value>>,
}

pub(crate) struct OperationLock {
    _lock: crate::util::FileLock,
}
impl OperationLock {
    pub(crate) fn acquire(paths: &Paths, name: &str) -> Result<Self> {
        paths.ensure()?;
        let lock = crate::util::FileLock::acquire(&paths.root.join(format!("{name}.lock")))
            .with_context(|| format!("another mirket {name} operation is active"))?;
        Ok(Self { _lock: lock })
    }
}

pub fn saved_options(paths: &Paths) -> Result<Option<SetupOptions>> {
    Ok(read_state(paths)?.map(|state| state.options))
}

/// Saved choices and the exact receipt generation they came from, read together.
/// Pass the token to `setup_checked` when deriving a change from these choices.
pub struct SetupSnapshot {
    pub options: Option<SetupOptions>,
    pub token: String,
}

pub fn setup_snapshot(paths: &Paths) -> Result<SetupSnapshot> {
    let (options, token) = match read_state_snapshot(paths)? {
        Some((state, token)) => (Some(state.options), token),
        None => (None, ABSENT_SETUP_TOKEN.to_owned()),
    };
    Ok(SetupSnapshot { options, token })
}

fn expected_receipt(token: &str) -> Result<Option<&str>> {
    if token == ABSENT_SETUP_TOKEN {
        Ok(None)
    } else if token.len() == 64
        && token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        Ok(Some(token))
    } else {
        bail!("invalid setup generation token");
    }
}

fn check_setup_generation(paths: &Paths, token: Option<&str>, phase: &str) -> Result<()> {
    if let Some(token) = token {
        let expected = expected_receipt(token)?;
        let actual = read_optional(&paths.settings(), MAX_SETTINGS)?.map(|bytes| sha256(&bytes));
        if actual.as_deref() != expected {
            bail!("setup selection changed during {phase}; retry with the current saved choices");
        }
    }
    Ok(())
}

pub(crate) fn read_state(paths: &Paths) -> Result<Option<InstallState>> {
    Ok(read_state_snapshot(paths)?.map(|(state, _)| state))
}
pub(crate) fn read_state_snapshot(paths: &Paths) -> Result<Option<(InstallState, String)>> {
    let Some(bytes) = read_optional(&paths.settings(), MAX_SETTINGS)? else {
        return Ok(None);
    };
    let state: InstallState =
        serde_json::from_slice(&bytes).context("invalid mirket setup receipt")?;
    if state.format != 1 {
        bail!("unsupported mirket setup receipt format");
    }
    validate_options(&state.options)?;
    crate::update::parse_version(&state.version)?;
    if state.files.len() > 20_000 {
        bail!("setup receipt contains too many files");
    }
    for (path, file) in &state.files {
        validate_owned_path(path)?;
        if file.sha256.len() != 64 || !file.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
            bail!("invalid digest in setup receipt");
        }
    }
    for servers in state.servers.values() {
        if servers.keys().any(|name| {
            !matches!(name.as_str(), "mirket" | "openai-docs" | "microsoft-learn")
                && !name
                    .strip_prefix("mirket-package-")
                    .is_some_and(crate::packages::valid_id)
        }) {
            bail!("invalid server ownership in setup receipt");
        }
    }
    Ok(Some((state, sha256(&bytes))))
}

/// Resolve an installed newer runtime for the small executable bootstrap. The
/// receipt verifies managed bytes; it is not a release signature.
pub fn newer_runtime(paths: &Paths) -> Result<Option<PathBuf>> {
    let Some(state) = read_state(paths)? else {
        return Ok(None);
    };
    if crate::update::parse_version(&state.version)?
        <= crate::update::parse_version(env!("CARGO_PKG_VERSION"))?
    {
        return Ok(None);
    }
    let binary = paths.binary();
    if std::env::current_exe()?.canonicalize()? == binary.canonicalize()? {
        return Ok(None);
    }
    let key = binary
        .strip_prefix(&paths.home)?
        .to_string_lossy()
        .replace('\\', "/");
    let expected = state
        .files
        .get(&key)
        .context("setup receipt does not own the managed executable")?;
    let bytes = read_bounded(&binary, MAX_FILE)?;
    if !expected.executable
        || sha256(&bytes) != expected.sha256
        || !mode_matches(file_mode(&binary)?, true)
    {
        bail!("managed runtime differs from its setup receipt; run mirket doctor");
    }
    Ok(Some(binary))
}

pub fn setup(paths: &Paths, catalog: &Catalog, options: &SetupOptions) -> Result<SetupReport> {
    setup_checked(paths, catalog, options, None)
}
pub fn setup_checked(
    paths: &Paths,
    catalog: &Catalog,
    options: &SetupOptions,
    expected_state: Option<&str>,
) -> Result<SetupReport> {
    setup_from_binary_checked(
        paths,
        catalog,
        options,
        &std::env::current_exe()?,
        expected_state,
    )
}
pub fn setup_from_binary(
    paths: &Paths,
    catalog: &Catalog,
    options: &SetupOptions,
    binary: &Path,
) -> Result<SetupReport> {
    setup_from_binary_checked(paths, catalog, options, binary, None)
}
fn setup_from_binary_checked(
    paths: &Paths,
    catalog: &Catalog,
    options: &SetupOptions,
    binary: &Path,
    expected_state: Option<&str>,
) -> Result<SetupReport> {
    if let Some(token) = expected_state {
        expected_receipt(token)?;
    }
    let _lock = OperationLock::acquire(paths, "setup")?;
    // Recovery can replace the on-disk path of the process that is executing us.
    // Preserve the chosen runtime bytes before restoring any destination.
    let binary = read_bounded(binary, MAX_FILE).context("read mirket executable")?;
    recover_setup(paths)?;
    check_setup_generation(paths, expected_state, "update or configuration")?;
    cleanup_retired_runtime(paths)?;
    let (changes, report) = prepare(paths, catalog, options, &binary)?;
    crate::packages::prepare(paths, &options.packages)?;
    check_setup_generation(paths, expected_state, "package preparation")?;
    apply_changes(paths, &changes)?;
    Ok(report)
}

pub fn plan(paths: &Paths, catalog: &Catalog, options: &SetupOptions) -> Result<SetupReport> {
    let binary = read_bounded(&std::env::current_exe()?, MAX_FILE)?;
    let (_, report) = prepare(paths, catalog, options, &binary)?;
    Ok(report)
}

fn validate_options(options: &SetupOptions) -> Result<()> {
    if options.targets.is_empty() {
        bail!("choose at least one target");
    }
    let unique: BTreeSet<_> = options.targets.iter().collect();
    if unique.len() != options.targets.len() {
        bail!("setup targets must not be repeated");
    }
    if options.packages.len() > 32
        || options
            .packages
            .iter()
            .any(|id| !crate::packages::valid_id(id))
        || options.packages.iter().collect::<BTreeSet<_>>().len() != options.packages.len()
        || options.package_settings.len() > 32
        || options
            .package_settings
            .keys()
            .any(|id| !crate::packages::valid_id(id))
        || serde_json::to_vec(&options.package_settings)?.len() > 64 * 1024
    {
        bail!("invalid package selection or settings");
    }
    Ok(())
}
fn validate_owned_path(path: &str) -> Result<()> {
    safe_relative(Path::new(path))?;
    let binary = if cfg!(windows) {
        ".mirket/bin/mirket.exe"
    } else {
        ".mirket/bin/mirket"
    };
    let valid = path == binary
        || [Target::Codex, Target::Claude].iter().any(|target| {
            path == target.instructions()
                || path
                    .strip_prefix(&format!("{}/", target.skills()))
                    .is_some_and(|part| part.contains('/'))
        });
    if !valid {
        bail!("unexpected managed path in setup receipt: {path}");
    }
    Ok(())
}

#[derive(Debug)]
struct Change {
    path: PathBuf,
    before: Option<Vec<u8>>,
    after: Option<Vec<u8>>,
    before_mode: Option<u32>,
    after_mode: Option<u32>,
}
fn file_mode(path: &Path) -> Result<Option<u32>> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if !meta.is_file() {
                bail!("expected a regular file: {}", path.display());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                Ok(Some(meta.permissions().mode() & 0o777))
            }
            #[cfg(not(unix))]
            {
                Ok(None)
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}
fn executable_mode(mode: Option<u32>) -> bool {
    cfg!(unix) && mode.is_some_and(|mode| mode & 0o111 != 0)
}
fn mode_matches(actual: Option<u32>, executable: bool) -> bool {
    !cfg!(unix) || executable_mode(actual) == executable
}
fn read_optional(path: &Path, max: u64) -> Result<Option<Vec<u8>>> {
    reject_symlink_components(path)?;
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(Some(read_bounded(path, max)?)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
}
fn add_change(
    changes: &mut Vec<Change>,
    path: PathBuf,
    after: Option<Vec<u8>>,
    executable: Option<bool>,
) -> Result<()> {
    let before = read_optional(&path, MAX_FILE)?;
    let before_mode = file_mode(&path)?;
    let after_mode = if cfg!(unix) {
        Some(match (executable, before_mode) {
            (None, mode) => mode.unwrap_or(0o600),
            (Some(exec), Some(mode)) if mode_matches(Some(mode), exec) => mode,
            (Some(exec), Some(mode)) => (mode & 0o666) | if exec { 0o111 } else { 0 },
            (Some(true), None) => 0o755,
            (Some(false), None) => 0o644,
        })
    } else {
        None
    };
    if before != after || (after.is_some() && after_mode != before_mode) {
        changes.push(Change {
            path,
            before,
            after,
            before_mode,
            after_mode,
        });
    }
    Ok(())
}
fn assert_owned(
    path: &Path,
    actual: Option<&[u8]>,
    mode: Option<u32>,
    old: Option<&ManagedFile>,
    desired: Option<&ManagedFile>,
) -> Result<()> {
    if let Some(bytes) = actual {
        let digest = sha256(bytes);
        let matches =
            |entry: &ManagedFile| entry.sha256 == digest && mode_matches(mode, entry.executable);
        match old {
            Some(entry) if !matches(entry) => bail!(
                "managed file was modified: {}; restore or relocate the change before setup",
                path.display()
            ),
            None if !desired.is_some_and(matches) => {
                bail!("unmanaged file conflicts with setup: {}", path.display())
            }
            _ => {}
        }
    }
    Ok(())
}

fn prepare(
    paths: &Paths,
    catalog: &Catalog,
    options: &SetupOptions,
    binary: &[u8],
) -> Result<(Vec<Change>, SetupReport)> {
    validate_options(options)?;
    crate::packages::Registry::embedded()?.validate_selection(&options.packages)?;
    reject_symlink_components(&paths.root)?;
    let prior = read_state(paths)?;
    if let Some(state) = &prior
        && crate::update::parse_version(&state.version)?
            > crate::update::parse_version(env!("CARGO_PKG_VERSION"))?
    {
        bail!(
            "installed mirket {} is newer than this runtime {}; run the managed executable instead of downgrading setup",
            state.version,
            env!("CARGO_PKG_VERSION")
        );
    }
    let mut desired = BTreeMap::<String, (Vec<u8>, bool)>::new();
    let binary_key = paths
        .binary()
        .strip_prefix(&paths.home)?
        .to_string_lossy()
        .replace('\\', "/");
    desired.insert(binary_key, (binary.to_vec(), true));
    for target in &options.targets {
        desired.insert(
            target.instructions().into(),
            (catalog.instructions().to_vec(), false),
        );
        for id in catalog.global_skills() {
            let payload = catalog.payload(id)?;
            let root = format!("{}/{id}", target.skills());
            safe_relative(Path::new(&root))?;
            for (relative, bytes) in payload.files {
                safe_relative(Path::new(&relative))?;
                let executable = payload.executables.contains(&relative);
                desired.insert(format!("{root}/{relative}"), (bytes, executable));
            }
        }
    }
    let new_files: BTreeMap<_, _> = desired
        .iter()
        .map(|(path, (bytes, executable))| {
            (
                path.clone(),
                ManagedFile {
                    sha256: sha256(bytes),
                    executable: *executable,
                },
            )
        })
        .collect();
    let empty = BTreeMap::new();
    let old_files = prior.as_ref().map(|state| &state.files).unwrap_or(&empty);
    let all_paths: BTreeSet<_> = old_files.keys().chain(desired.keys()).cloned().collect();
    let mut skill_roots = BTreeSet::new();
    for relative in &all_paths {
        validate_owned_path(relative)?;
        for target in [Target::Codex, Target::Claude] {
            if let Some(tail) = relative.strip_prefix(&format!("{}/", target.skills())) {
                let id = tail.split('/').next().context("invalid skill path")?;
                skill_roots.insert(format!("{}/{id}", target.skills()));
            }
        }
    }
    for root in skill_roots {
        for relative in existing_files(&paths.home.join(&root))? {
            let key = format!("{root}/{}", relative.to_string_lossy().replace('\\', "/"));
            if !all_paths.contains(&key) {
                bail!(
                    "unmanaged file inside managed skill: {}",
                    paths.home.join(key).display()
                );
            }
        }
    }
    let mut changes = Vec::new();
    for relative in all_paths {
        let path = paths.home.join(&relative);
        let actual = read_optional(&path, MAX_FILE)?;
        let mode = file_mode(&path)?;
        assert_owned(
            &path,
            actual.as_deref(),
            mode,
            old_files.get(&relative),
            new_files.get(&relative),
        )?;
        match desired.get(&relative) {
            Some((bytes, exec)) => {
                add_change(&mut changes, path, Some(bytes.clone()), Some(*exec))?
            }
            None if actual.is_some() => add_change(&mut changes, path, None, None)?,
            None => {}
        }
    }
    let mut new_servers = BTreeMap::new();
    for target in [Target::Codex, Target::Claude] {
        let wanted = if options.targets.contains(&target) {
            desired_servers(paths, target, options)?
        } else {
            BTreeMap::new()
        };
        let old = prior
            .as_ref()
            .and_then(|state| state.servers.get(&target))
            .cloned()
            .unwrap_or_default();
        if wanted.is_empty() && old.is_empty() {
            continue;
        }
        let path = paths.home.join(target.config());
        let current = read_optional(&path, MAX_SETTINGS)?;
        let bytes = merge_config(target, current.as_deref(), &old, &wanted)?;
        add_change(&mut changes, path, Some(bytes), None)?;
        if !wanted.is_empty() {
            new_servers.insert(target, wanted);
        }
    }
    let receipt = InstallState {
        format: 1,
        version: env!("CARGO_PKG_VERSION").into(),
        options: options.clone(),
        files: new_files,
        servers: new_servers,
    };
    let mut receipt_bytes = serde_json::to_vec_pretty(&receipt)?;
    receipt_bytes.push(b'\n');
    add_change(&mut changes, paths.settings(), Some(receipt_bytes), None)?;
    let report = SetupReport {
        version: receipt.version,
        targets: options.targets.clone(),
        files: receipt.files.len(),
        changed: changes.len(),
        binary: paths.binary(),
        packages: options.packages.clone(),
    };
    Ok((changes, report))
}

fn existing_files(root: &Path) -> Result<Vec<PathBuf>> {
    reject_symlink_components(root)?;
    if !root.exists() {
        return Ok(Vec::new());
    }
    if !root.is_dir() {
        bail!("expected a skill directory: {}", root.display());
    }
    fn walk(base: &Path, current: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        if current.strip_prefix(base)?.components().count() > 32 {
            bail!("managed skill directory exceeds 32 levels");
        }
        for item in fs::read_dir(current)? {
            let item = item?;
            if out.len() >= 20_000 {
                bail!("managed skill directory exceeds 20000 files");
            }
            let kind = item.file_type()?;
            if kind.is_symlink() {
                bail!("refusing symlink: {}", item.path().display());
            }
            if kind.is_dir() {
                walk(base, &item.path(), out)?;
            } else if kind.is_file() {
                out.push(item.path().strip_prefix(base)?.into());
            } else {
                bail!(
                    "unsupported file in managed skill: {}",
                    item.path().display()
                );
            }
        }
        Ok(())
    }
    let mut found = Vec::new();
    walk(root, root, &mut found)?;
    Ok(found)
}

fn desired_servers(
    paths: &Paths,
    target: Target,
    options: &SetupOptions,
) -> Result<BTreeMap<String, Value>> {
    let packages = crate::packages::Registry::embedded()?;
    let args = json!(["--home", paths.home, "mcp", "serve"]);
    let mut servers = BTreeMap::new();
    let mirket = match target {
        Target::Codex => json!({"command": paths.binary(), "args": args}),
        Target::Claude => json!({"type": "stdio", "command": paths.binary(), "args": args}),
    };
    servers.insert("mirket".into(), mirket);
    for id in &options.packages {
        let args = json!(["--home", paths.home, "package", "serve", id]);
        let mut value = match target {
            Target::Codex => json!({"command":paths.binary(),"args":args}),
            Target::Claude => json!({"type":"stdio","command":paths.binary(),"args":args}),
        };
        if packages.get(id)?.adapter.is_some() {
            match target {
                Target::Codex => {
                    value["startup_timeout_sec"] = json!(90);
                    value["tool_timeout_sec"] = json!(120);
                }
                Target::Claude => {
                    value["timeout"] = json!(120_000);
                }
            }
        }
        servers.insert(format!("mirket-package-{id}"), value);
    }
    for (name, url) in [
        ("openai-docs", "https://developers.openai.com/mcp"),
        ("microsoft-learn", "https://learn.microsoft.com/api/mcp"),
    ] {
        if name == "microsoft-learn" && !options.microsoft_learn {
            continue;
        }
        let value = match target {
            Target::Codex => json!({"url":url}),
            Target::Claude => json!({"type":"http", "url":url}),
        };
        servers.insert(name.into(), value);
    }
    Ok(servers)
}
fn validate_server_change(
    name: &str,
    current: Option<&Value>,
    previous: Option<&Value>,
    wanted: Option<&Value>,
) -> Result<()> {
    if let Some(current) = current {
        if let Some(previous) = previous {
            if current != previous {
                bail!(
                    "managed MCP server {name} was modified; preserve or reconcile it before setup"
                );
            }
        } else if wanted != Some(current) {
            bail!("unmanaged MCP server {name} conflicts with setup");
        }
    }
    Ok(())
}
fn merge_config(
    target: Target,
    bytes: Option<&[u8]>,
    old: &BTreeMap<String, Value>,
    wanted: &BTreeMap<String, Value>,
) -> Result<Vec<u8>> {
    let names: BTreeSet<_> = old.keys().chain(wanted.keys()).cloned().collect();
    match target {
        Target::Claude => {
            let mut document = match bytes {
                Some(bytes) => serde_json::from_slice::<Value>(bytes)
                    .context("invalid Claude JSON settings")?,
                None => json!({}),
            };
            let root = document
                .as_object_mut()
                .context("Claude settings must be a JSON object")?;
            let servers = root
                .entry("mcpServers")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .context("Claude mcpServers must be an object")?;
            let mut changed = false;
            for name in names {
                let current = servers.get(&name);
                let desired = wanted.get(&name);
                validate_server_change(&name, current, old.get(&name), desired)?;
                if current != desired {
                    changed = true;
                    if let Some(value) = desired {
                        servers.insert(name, value.clone());
                    } else {
                        servers.remove(&name);
                    }
                }
            }
            if !changed && let Some(bytes) = bytes {
                return Ok(bytes.to_vec());
            }
            let mut bytes = serde_json::to_vec_pretty(&document)?;
            bytes.push(b'\n');
            Ok(bytes)
        }
        Target::Codex => {
            let text = std::str::from_utf8(bytes.unwrap_or(b""))
                .context("Codex settings must be UTF-8")?;
            let mut document: DocumentMut = text.parse().context("invalid Codex TOML settings")?;
            if document.get("mcp_servers").is_none() {
                document["mcp_servers"] = Item::Table(Table::new());
            }
            let servers = document["mcp_servers"]
                .as_table_like_mut()
                .context("Codex mcp_servers must be a table")?;
            let mut changed = false;
            for name in names {
                let current = servers.get(&name).map(toml_to_json).transpose()?;
                let desired = wanted.get(&name);
                validate_server_change(&name, current.as_ref(), old.get(&name), desired)?;
                if current.as_ref() != desired {
                    changed = true;
                    if let Some(value) = desired {
                        servers.insert(&name, json_to_toml(value)?);
                    } else {
                        servers.remove(&name);
                    }
                }
            }
            if !changed && let Some(bytes) = bytes {
                return Ok(bytes.to_vec());
            }
            Ok(document.to_string().into_bytes())
        }
    }
}
fn toml_to_json(item: &Item) -> Result<Value> {
    if let Some(table) = item.as_table_like() {
        let mut result = serde_json::Map::new();
        for (key, value) in table.iter() {
            result.insert(key.into(), toml_to_json(value)?);
        }
        return Ok(Value::Object(result));
    }
    let value = item
        .as_value()
        .context("MCP configuration must contain scalar values or tables")?;
    fn convert(value: &toml_edit::Value) -> Result<Value> {
        Ok(match value {
            toml_edit::Value::String(value) => json!(value.value()),
            toml_edit::Value::Integer(value) => json!(value.value()),
            toml_edit::Value::Float(value) => json!(value.value()),
            toml_edit::Value::Boolean(value) => json!(value.value()),
            toml_edit::Value::Datetime(value) => json!(value.value().to_string()),
            toml_edit::Value::Array(values) => {
                Value::Array(values.iter().map(convert).collect::<Result<_>>()?)
            }
            toml_edit::Value::InlineTable(values) => {
                let mut object = serde_json::Map::new();
                for (key, value) in values.iter() {
                    object.insert(key.into(), convert(value)?);
                }
                Value::Object(object)
            }
        })
    }
    convert(value)
}
fn json_to_toml(value: &Value) -> Result<Item> {
    match value {
        Value::Object(values) => {
            let mut table = Table::new();
            for (key, value) in values {
                table.insert(key, json_to_toml(value)?);
            }
            Ok(Item::Table(table))
        }
        Value::String(value) => Ok(toml_edit::value(value)),
        Value::Bool(value) => Ok(toml_edit::value(*value)),
        Value::Number(value) => Ok(toml_edit::value(
            value
                .as_i64()
                .context("MCP numeric setting must be a signed integer")?,
        )),
        Value::Array(values) => {
            let mut array = toml_edit::Array::new();
            for value in values {
                array.push(value.as_str().context("MCP argument must be a string")?);
            }
            Ok(toml_edit::value(array))
        }
        _ => bail!("unsupported MCP settings value"),
    }
}

fn set_mode(path: &Path, mode: Option<u32>) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(mode) = mode {
            fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    Ok(())
}
fn write_snapshot(path: &Path, bytes: Option<&[u8]>, mode: Option<u32>) -> Result<()> {
    if let Some(bytes) = bytes {
        #[cfg(windows)]
        if path.file_name().is_some_and(|name| name == "mirket.exe")
            && path
                .parent()
                .is_some_and(|parent| parent.ends_with(".mirket/bin"))
        {
            replace_running_windows_binary(path, bytes)?;
        } else {
            atomic_write(path, bytes)?;
        }
        #[cfg(not(windows))]
        atomic_write(path, bytes)?;
        set_mode(path, mode)?;
    } else if path.exists() {
        reject_symlink_components(path)?;
        fs::remove_file(path)?;
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    format: u32,
    files: Vec<JournalFile>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalFile {
    path: String,
    before: Option<String>,
    after: Option<String>,
    before_mode: Option<u32>,
    after_mode: Option<u32>,
}
fn transaction_path(paths: &Paths) -> PathBuf {
    paths.root.join(".setup-transaction")
}
fn prepare_journal(paths: &Paths, changes: &[Change]) -> Result<()> {
    let directory = transaction_path(paths);
    reject_symlink_components(&directory)?;
    fs::create_dir(&directory).context("create setup recovery journal")?;
    set_mode(&directory, Some(0o700))?;
    let result = (|| -> Result<()> {
        let mut files = Vec::new();
        for (index, change) in changes.iter().enumerate() {
            if let Some(before) = &change.before {
                atomic_write(&directory.join(format!("before-{index}")), before)?;
            }
            files.push(JournalFile {
                path: change
                    .path
                    .strip_prefix(&paths.home)?
                    .to_string_lossy()
                    .replace('\\', "/"),
                before: change.before.as_ref().map(|bytes| sha256(bytes)),
                after: change.after.as_ref().map(|bytes| sha256(bytes)),
                before_mode: change.before_mode,
                after_mode: change.after_mode,
            });
        }
        atomic_write(
            &directory.join("journal.json"),
            &serde_json::to_vec(&Journal { format: 1, files })?,
        )
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&directory);
    }
    result
}
fn recover_setup(paths: &Paths) -> Result<()> {
    let directory = transaction_path(paths);
    reject_symlink_components(&directory)?;
    if !directory.exists() {
        return Ok(());
    }
    if let Some(marker) = read_optional(&directory.join("committed"), 32)? {
        if marker != b"mirket setup committed\n" {
            bail!("invalid setup transaction commit marker");
        }
        fs::remove_dir_all(&directory)?;
        return Ok(());
    }
    let Some(bytes) = read_optional(&directory.join("journal.json"), MAX_SETTINGS)? else {
        // No destination can have been written before the complete journal exists.
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let before_image = name.strip_prefix("before-").is_some_and(|index| {
                !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())
            });
            let atomic_temporary = name.strip_prefix(".tmp").is_some_and(|suffix| {
                !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
            });
            if !entry.file_type()?.is_file() || !(before_image || atomic_temporary) {
                bail!(
                    "incomplete setup journal contains an unexpected file; preserve it before setup"
                );
            }
        }
        fs::remove_dir_all(&directory)?;
        return Ok(());
    };
    let journal: Journal =
        serde_json::from_slice(&bytes).context("invalid setup recovery journal")?;
    if journal.format != 1 || journal.files.len() > 20_000 {
        bail!("invalid setup recovery journal format or size");
    }
    let mut snapshots = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, file) in journal.files.iter().enumerate() {
        safe_relative(Path::new(&file.path))?;
        if !seen.insert(&file.path) {
            bail!("setup recovery journal repeats a destination");
        }
        let allowed = validate_owned_path(&file.path).is_ok()
            || file.path == ".mirket/setup.json"
            || [Target::Codex, Target::Claude]
                .iter()
                .any(|target| file.path == target.config());
        if !allowed {
            bail!("setup recovery journal claims an unmanaged path");
        }
        for digest in [file.before.as_ref(), file.after.as_ref()]
            .into_iter()
            .flatten()
        {
            if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                bail!("invalid setup recovery digest");
            }
        }
        if file.before_mode.is_some_and(|mode| mode > 0o777)
            || file.after_mode.is_some_and(|mode| mode > 0o777)
        {
            bail!("invalid setup recovery file mode");
        }
        let before = read_optional(&directory.join(format!("before-{index}")), MAX_FILE)?;
        if before.as_ref().map(|bytes| sha256(bytes)) != file.before {
            bail!("setup recovery image was modified");
        }
        let path = paths.home.join(&file.path);
        let current = read_optional(&path, MAX_FILE)?;
        let digest = current.as_ref().map(|bytes| sha256(bytes));
        if digest != file.before && digest != file.after {
            bail!(
                "interrupted setup conflicts with a user change at {}; change preserved",
                path.display()
            );
        }
        let changed = digest == file.after;
        snapshots.push((path, before, file.before_mode, changed, digest));
    }
    for (path, before, mode, changed, expected) in snapshots.into_iter().rev() {
        if changed {
            if read_optional(&path, MAX_FILE)?
                .as_ref()
                .map(|bytes| sha256(bytes))
                != expected
            {
                bail!("file changed during setup recovery: {}", path.display());
            }
            write_snapshot(&path, before.as_deref(), mode)?;
        }
    }
    fs::remove_dir_all(directory)?;
    Ok(())
}
fn apply_changes(paths: &Paths, changes: &[Change]) -> Result<()> {
    if changes.is_empty() {
        return Ok(());
    }
    // Downloads can take time. Revalidate every destination before the first
    // mutation, then retain per-write checks for edits during activation.
    for change in changes {
        if read_optional(&change.path, MAX_FILE)? != change.before
            || file_mode(&change.path)? != change.before_mode
        {
            bail!(
                "file changed before setup activation: {}",
                change.path.display()
            );
        }
    }
    prepare_journal(paths, changes)?;
    for change in changes {
        let operation = (|| -> Result<()> {
            if read_optional(&change.path, MAX_FILE)? != change.before
                || file_mode(&change.path)? != change.before_mode
            {
                bail!("file changed during setup: {}", change.path.display());
            }
            write_snapshot(&change.path, change.after.as_deref(), change.after_mode)
        })();
        if let Err(error) = operation {
            return match recover_setup(paths) {
                Ok(()) => Err(error.context("setup failed; completed writes were restored")),
                Err(recovery) => Err(error.context(format!(
                    "setup failed and recovery needs attention: {recovery}"
                ))),
            };
        }
    }
    if let Err(error) = atomic_write(
        &transaction_path(paths).join("committed"),
        b"mirket setup committed\n",
    ) {
        recover_setup(paths).context("restore setup after commit marker failure")?;
        return Err(error.context("setup commit failed; completed writes were restored"));
    }
    fs::remove_dir_all(transaction_path(paths))
        .context("setup completed but recovery journal cleanup failed")?;
    Ok(())
}

pub fn doctor(paths: &Paths, catalog: &Catalog) -> Result<DoctorReport> {
    let mut checks = Vec::new();
    if transaction_path(paths).exists() {
        checks.push(check("interrupted setup", false, "A setup recovery journal is pending. Run mirket setup to restore the interrupted transaction before configuring this home."));
    }
    let state = match read_state(paths) {
        Ok(Some(state)) => state,
        Ok(None) => {
            checks.push(check(
                "installation",
                false,
                "Run mirket setup to configure this home.",
            ));
            return Ok(DoctorReport {
                ok: false,
                installed: false,
                checks,
                packages: json!({"ok":false,"status":"not_run","reason":"installation is absent"}),
            });
        }
        Err(error) => {
            checks.push(check("receipt", false, &error.to_string()));
            return Ok(DoctorReport {
                ok: false,
                installed: true,
                checks,
                packages: json!({"ok":false,"status":"not_run","reason":"installation receipt is invalid"}),
            });
        }
    };
    checks.push(check(
        "runtime version",
        state.version == env!("CARGO_PKG_VERSION"),
        &format!(
            "Installed runtime: {}; executing runtime: {}.",
            state.version,
            env!("CARGO_PKG_VERSION")
        ),
    ));
    checks.push(check(
        "receipt",
        true,
        &format!(
            "mirket {} configured for {}",
            state.version,
            state
                .options
                .targets
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ));
    let mut file_errors = Vec::new();
    for (relative, expected) in &state.files {
        let path = paths.home.join(relative);
        let valid = read_optional(&path, MAX_FILE).and_then(|bytes| {
            Ok(bytes.is_some_and(|bytes| sha256(&bytes) == expected.sha256)
                && mode_matches(file_mode(&path)?, expected.executable))
        });
        match valid {
            Ok(true) => {}
            Ok(false) => file_errors.push(format!("missing or changed: {relative}")),
            Err(error) => file_errors.push(format!("{relative}: {error}")),
        }
    }
    let file_detail = if file_errors.is_empty() {
        "All installed file digests and executable modes match.".to_string()
    } else {
        file_errors.join("; ")
    };
    checks.push(check("managed files", file_errors.is_empty(), &file_detail));
    let current = read_bounded(&paths.binary(), MAX_FILE)
        .and_then(|binary| prepare(paths, catalog, &state.options, &binary));
    match current {
        Ok((changes, _)) => {
            let payload_changed = !changes.is_empty();
            checks.push(check("configuration", !payload_changed, if payload_changed { "Installed payload or MCP configuration differs from this runtime; run mirket setup." } else { "Managed payload and MCP definitions match this runtime." }));
        }
        Err(error) => checks.push(check("configuration", false, &error.to_string())),
    }
    let packages = match crate::packages::doctor(paths, &state.options.packages) {
        Ok(report) => report,
        Err(error) => json!({"ok":false,"error":format!("{error:#}")}),
    };
    checks.push(check(
        "packages",
        packages["ok"] == true,
        &serde_json::to_string(&packages)?,
    ));
    for target in &state.options.targets {
        let executable = find_client(*target, paths);
        checks.push(DoctorCheck { name: format!("{target} client"), status: if executable.is_some() { "pass" } else { "warning" }.into(), message: executable.map_or_else(|| format!("The {target} executable was not found. Install the client separately; setup does not install or authenticate it."), |path| format!("Executable found at {}. Authentication and live skill/MCP activation require a client task.", path.display())) });
    }
    checks.push(DoctorCheck { name: "activation".into(), status: "warning".into(), message: "Static checks establish installation readiness. Open a fresh client session and invoke a mirket tool to verify activation.".into() });
    let ok = checks.iter().all(|check| check.status != "fail");
    Ok(DoctorReport {
        ok,
        installed: true,
        checks,
        packages,
    })
}
fn check(name: &str, valid: bool, message: &str) -> DoctorCheck {
    DoctorCheck {
        name: name.into(),
        status: if valid { "pass" } else { "fail" }.into(),
        message: message.into(),
    }
}
fn find_client(target: Target, paths: &Paths) -> Option<PathBuf> {
    let name = match target {
        Target::Codex => "codex",
        Target::Claude => "claude",
    };
    let mut candidates = Vec::new();
    if let Some(search) = std::env::var_os("PATH") {
        for parent in std::env::split_paths(&search) {
            candidates.push(parent.join(if cfg!(windows) {
                format!("{name}.exe")
            } else {
                name.into()
            }));
        }
    }
    candidates.push(paths.home.join(".local/bin").join(name));
    if cfg!(target_os = "macos") && target == Target::Codex {
        candidates.push(PathBuf::from("/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex"));
    }
    candidates
        .into_iter()
        .filter_map(|path| path.canonicalize().ok())
        .find(|path| path.is_file() && file_mode(path).is_ok_and(|mode| mode_matches(mode, true)))
}

// Windows permits moving an executing image but keeps it open until its clients
// exit. Retain one verified retirement record for the next setup to clean.
#[cfg(windows)]
fn replace_running_windows_binary(path: &Path, bytes: &[u8]) -> Result<()> {
    match atomic_write(path, bytes) {
        Ok(()) => return Ok(()),
        Err(error) if !path.exists() => return Err(error),
        Err(_) => {}
    }
    let retired = path.with_file_name(".mirket-running.exe");
    let record = path.with_file_name(".mirket-running.sha256");
    reject_symlink_components(&retired)?;
    reject_symlink_components(&record)?;
    if retired.exists() || record.exists() {
        bail!("close clients using mirket and run setup again to release the running executable");
    }
    let original = read_bounded(path, MAX_FILE)?;
    atomic_write(&record, sha256(&original).as_bytes())?;
    if let Err(error) = fs::rename(path, &retired) {
        let _ = fs::remove_file(&record);
        return Err(error.into());
    }
    if let Err(error) = atomic_write(path, bytes) {
        fs::rename(&retired, path).context("restore executable after replacement failure")?;
        fs::remove_file(&record)?;
        return Err(error);
    }
    Ok(())
}
fn cleanup_retired_runtime(paths: &Paths) -> Result<()> {
    #[cfg(windows)]
    {
        let retired = paths.binary().with_file_name(".mirket-running.exe");
        let record = paths.binary().with_file_name(".mirket-running.sha256");
        let Some(expected) = read_optional(&record, 64)? else {
            return Ok(());
        };
        let bytes = read_optional(&retired, MAX_FILE)?
            .context("retired runtime receipt has no executable")?;
        if expected != sha256(&bytes).as_bytes() {
            bail!("retired executable was modified; preserve it before setup");
        }
        match fs::remove_file(&retired) {
            Ok(()) => fs::remove_file(&record)?,
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {}
            Err(error) => return Err(error.into()),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = paths;
    }
    Ok(())
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn editor_timeout_settings_round_trip_and_preserve_other_servers() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        let options = SetupOptions {
            packages: vec!["inkscape".into(), "glaxnimate".into()],
            ..SetupOptions::default()
        };
        for (target, original) in [
            (Target::Codex, b"# personal setting\nmodel = 'mine'\n[mcp_servers.personal]\ncommand = 'own-server'\n".as_slice()),
            (Target::Claude, br#"{"theme":"dark","mcpServers":{"personal":{"command":"own-server"}}}"#.as_slice()),
        ] {
            let wanted = desired_servers(&paths, target, &options).unwrap();
            let merged = merge_config(target, Some(original), &BTreeMap::new(), &wanted).unwrap();
            assert_eq!(merge_config(target, Some(&merged), &wanted, &wanted).unwrap(), merged);
            for id in ["inkscape", "glaxnimate"] {
                let name = format!("mirket-package-{id}");
                match target {
                    Target::Codex => {
                        let document: DocumentMut = std::str::from_utf8(&merged).unwrap().parse().unwrap();
                        assert_eq!(document["model"].as_str(), Some("mine"));
                        assert_eq!(document["mcp_servers"]["personal"]["command"].as_str(), Some("own-server"));
                        assert_eq!(toml_to_json(&document["mcp_servers"][&name]).unwrap(), wanted[&name]);
                        assert_eq!(document["mcp_servers"][&name]["startup_timeout_sec"].as_integer(), Some(90));
                        assert_eq!(document["mcp_servers"][&name]["tool_timeout_sec"].as_integer(), Some(120));
                    }
                    Target::Claude => {
                        let document: Value = serde_json::from_slice(&merged).unwrap();
                        assert_eq!(document["theme"], "dark");
                        assert_eq!(document["mcpServers"]["personal"]["command"], "own-server");
                        assert_eq!(document["mcpServers"][&name], wanted[&name]);
                        assert_eq!(document["mcpServers"][&name]["timeout"], 120_000);
                    }
                }
            }
        }
    }
    #[test]
    fn late_destination_edit_prevents_all_setup_writes() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        paths.ensure().unwrap();
        let first = paths.home.join(".codex/AGENTS.md");
        let later = paths.home.join(".claude/CLAUDE.md");
        atomic_write(&first, b"first before").unwrap();
        atomic_write(&later, b"later before").unwrap();
        let mut changes = Vec::new();
        add_change(
            &mut changes,
            first.clone(),
            Some(b"first after".to_vec()),
            Some(false),
        )
        .unwrap();
        add_change(
            &mut changes,
            later.clone(),
            Some(b"later after".to_vec()),
            Some(false),
        )
        .unwrap();
        atomic_write(&later, b"concurrent user edit").unwrap();
        assert!(
            apply_changes(&paths, &changes)
                .unwrap_err()
                .to_string()
                .contains("before setup activation")
        );
        assert_eq!(fs::read(first).unwrap(), b"first before");
        assert_eq!(fs::read(later).unwrap(), b"concurrent user edit");
        assert!(!transaction_path(&paths).exists());
    }
    #[test]
    fn interrupted_setup_restores_before_images_and_is_repeatable() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        paths.ensure().unwrap();
        let instruction = paths.home.join(".codex/AGENTS.md");
        write_snapshot(&instruction, Some(b"user-owned-before-image"), Some(0o600)).unwrap();
        let mut changes = Vec::new();
        add_change(
            &mut changes,
            instruction.clone(),
            Some(b"new-installed-output".to_vec()),
            Some(false),
        )
        .unwrap();
        add_change(
            &mut changes,
            paths.home.join(".claude/CLAUDE.md"),
            Some(b"second-output".to_vec()),
            Some(false),
        )
        .unwrap();
        prepare_journal(&paths, &changes).unwrap();
        // Simulate process termination after one committed file replacement.
        write_snapshot(
            &instruction,
            changes[0].after.as_deref(),
            changes[0].after_mode,
        )
        .unwrap();
        recover_setup(&paths).unwrap();
        assert_eq!(fs::read(instruction).unwrap(), b"user-owned-before-image");
        assert!(!paths.home.join(".claude/CLAUDE.md").exists());
        assert!(!transaction_path(&paths).exists());
        recover_setup(&paths).unwrap();
    }
    #[test]
    fn interrupted_setup_never_overwrites_divergent_user_changes() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        paths.ensure().unwrap();
        let instruction = paths.home.join(".codex/AGENTS.md");
        write_snapshot(&instruction, Some(b"before"), Some(0o600)).unwrap();
        let mut changes = Vec::new();
        add_change(
            &mut changes,
            instruction.clone(),
            Some(b"after".to_vec()),
            Some(false),
        )
        .unwrap();
        prepare_journal(&paths, &changes).unwrap();
        write_snapshot(&instruction, Some(b"a subsequent user edit"), Some(0o600)).unwrap();
        assert!(
            recover_setup(&paths)
                .unwrap_err()
                .to_string()
                .contains("user change")
        );
        assert_eq!(fs::read(instruction).unwrap(), b"a subsequent user edit");
        assert!(transaction_path(&paths).exists());
    }
    #[test]
    fn recovery_keeps_the_chosen_executable_bytes_for_setup() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        let catalog = Catalog::embedded().unwrap();
        let source = paths.home.join("downloaded-runtime");
        fs::write(&source, b"installed-runtime").unwrap();
        let options = SetupOptions {
            targets: vec![Target::Codex],
            microsoft_learn: false,
            ..SetupOptions::default()
        };
        setup_from_binary(&paths, &catalog, &options, &source).unwrap();
        let mut changes = Vec::new();
        add_change(
            &mut changes,
            paths.binary(),
            Some(b"chosen-running-runtime".to_vec()),
            Some(true),
        )
        .unwrap();
        prepare_journal(&paths, &changes).unwrap();
        write_snapshot(
            &paths.binary(),
            changes[0].after.as_deref(),
            changes[0].after_mode,
        )
        .unwrap();
        setup_from_binary(&paths, &catalog, &options, &paths.binary()).unwrap();
        assert_eq!(fs::read(paths.binary()).unwrap(), b"chosen-running-runtime");
        assert!(doctor(&paths, &catalog).unwrap().ok);
    }
    #[test]
    fn committed_setup_survives_interrupted_journal_cleanup() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        paths.ensure().unwrap();
        let instruction = paths.home.join(".codex/AGENTS.md");
        write_snapshot(&instruction, Some(b"before"), Some(0o600)).unwrap();
        let mut changes = Vec::new();
        add_change(
            &mut changes,
            instruction.clone(),
            Some(b"committed".to_vec()),
            Some(false),
        )
        .unwrap();
        prepare_journal(&paths, &changes).unwrap();
        write_snapshot(&instruction, Some(b"committed"), Some(0o600)).unwrap();
        atomic_write(
            &transaction_path(&paths).join("committed"),
            b"mirket setup committed\n",
        )
        .unwrap();
        fs::remove_file(transaction_path(&paths).join("before-0")).unwrap();
        recover_setup(&paths).unwrap();
        assert_eq!(fs::read(instruction).unwrap(), b"committed");
        assert!(!transaction_path(&paths).exists());
    }
}
