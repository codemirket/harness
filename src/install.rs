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
    fs, io,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, Item, Table};

const MAX_FILE: u64 = 256 * 1024 * 1024;
const MAX_SETTINGS: u64 = 16 * 1024 * 1024;
const ABSENT_SETUP_TOKEN: &str = "absent";

pub use crate::target::Target;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SetupOptions {
    pub targets: Vec<Target>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub package_settings: BTreeMap<String, Value>,
}
impl<'de> Deserialize<'de> for SetupOptions {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Accept only the retired receipt key, never a current setup option.
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ReceiptOptions {
            targets: Vec<Target>,
            #[serde(default, rename = "microsoft_learn")]
            _removed_documentation_selection: bool,
            #[serde(default)]
            packages: Vec<String>,
            #[serde(default)]
            package_settings: BTreeMap<String, Value>,
        }
        let options = ReceiptOptions::deserialize(deserializer)?;
        Ok(Self {
            targets: options.targets,
            packages: options.packages,
            package_settings: options.package_settings,
        })
    }
}
impl Default for SetupOptions {
    fn default() -> Self {
        Self {
            targets: vec![Target::Codex, Target::Claude],
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

struct ReceiptSnapshot {
    state: InstallState,
    bytes: Vec<u8>,
    mode: Option<u32>,
    token: String,
}

pub(crate) struct RemovedReplayChoices {
    pub microsoft_learn: bool,
    pub targets: Vec<Target>,
}

/// An updater may supply retired arguments, but only the exact saved generation
/// can authorize their removal. No live configuration or receipt is changed here.
pub(crate) fn removed_replay_choices(
    paths: &Paths,
    expected: &str,
) -> Result<RemovedReplayChoices> {
    expected_receipt(expected)?.context("update replay requires an existing receipt")?;
    let (state, identity) =
        read_state_snapshot(paths)?.context("update replay receipt is absent")?;
    if identity != expected {
        bail!("update replay receipt changed; run mirket setup with current saved choices");
    }
    let bytes = read_bounded(&paths.settings(), MAX_SETTINGS)?;
    if sha256(&bytes) != identity {
        bail!("update replay receipt changed while reading retired choices");
    }
    let receipt: Value = serde_json::from_slice(&bytes)?;
    Ok(RemovedReplayChoices {
        microsoft_learn: receipt["options"]["microsoft_learn"].as_bool() == Some(true),
        targets: state.options.targets,
    })
}

pub(crate) fn read_state_snapshot(paths: &Paths) -> Result<Option<(InstallState, String)>> {
    Ok(read_receipt_snapshot(paths)?.map(|snapshot| (snapshot.state, snapshot.token)))
}

fn read_receipt_snapshot(paths: &Paths) -> Result<Option<ReceiptSnapshot>> {
    let receipt_path = paths.settings();
    let Some(bytes) = read_optional(&receipt_path, MAX_SETTINGS)? else {
        return Ok(None);
    };
    let mode = file_mode(&receipt_path)?;
    let state = parse_receipt(&bytes)?;
    Ok(Some(ReceiptSnapshot {
        state,
        token: sha256(&bytes),
        bytes,
        mode,
    }))
}

fn parse_receipt(bytes: &[u8]) -> Result<InstallState> {
    let state: InstallState =
        serde_json::from_slice(bytes).context("invalid mirket setup receipt")?;
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
    Ok(state)
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
    let registry = crate::packages::Registry::embedded()?;
    let (changes, report) = prepare(paths, catalog, options, &binary, &registry)?;
    crate::packages::prepare_with_registry(paths, &options.packages, &registry)?;
    check_setup_generation(paths, expected_state, "package preparation")?;
    apply_changes(paths, &changes)?;
    Ok(report)
}

pub fn plan(paths: &Paths, catalog: &Catalog, options: &SetupOptions) -> Result<SetupReport> {
    let binary = read_bounded(&std::env::current_exe()?, MAX_FILE)?;
    let registry = crate::packages::Registry::embedded()?;
    let (_, report) = prepare(paths, catalog, options, &binary, &registry)?;
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
    before: Option<Vec<u8>>,
    before_mode: Option<u32>,
    after: Option<Vec<u8>>,
    executable: Option<bool>,
) {
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
    registry: &crate::packages::Registry,
) -> Result<(Vec<Change>, SetupReport)> {
    validate_options(options)?;
    registry.validate_selection(&options.packages)?;
    reject_symlink_components(&paths.root)?;
    let receipt_snapshot = read_receipt_snapshot(paths)?;
    let prior = receipt_snapshot.as_ref().map(|snapshot| &snapshot.state);
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
            Some((bytes, exec)) => add_change(
                &mut changes,
                path,
                actual,
                mode,
                Some(bytes.clone()),
                Some(*exec),
            ),
            None if actual.is_some() => add_change(&mut changes, path, actual, mode, None, None),
            None => {}
        }
    }
    let mut new_servers = BTreeMap::new();
    for target in [Target::Codex, Target::Claude] {
        let wanted = if options.targets.contains(&target) {
            desired_servers(registry, paths, target, options)?
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
        let current_mode = file_mode(&path)?;
        let bytes = merge_config(target, current.as_deref(), &old, &wanted)?;
        add_change(&mut changes, path, current, current_mode, Some(bytes), None);
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
    let receipt_path = paths.settings();
    let (receipt_before, receipt_mode) = receipt_snapshot
        .map(|snapshot| (Some(snapshot.bytes), snapshot.mode))
        .unwrap_or((None, None));
    add_change(
        &mut changes,
        receipt_path,
        receipt_before,
        receipt_mode,
        Some(receipt_bytes),
        None,
    );
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
    registry: &crate::packages::Registry,
    paths: &Paths,
    target: Target,
    options: &SetupOptions,
) -> Result<BTreeMap<String, Value>> {
    let args = json!(["--home", paths.home, "mcp", "serve"]);
    let mut servers = BTreeMap::new();
    let mirket = match target {
        Target::Codex => json!({"command": paths.binary(), "args": args, "tool_timeout_sec": 7200}),
        Target::Claude => {
            json!({"type": "stdio", "command": paths.binary(), "args": args, "timeout": 7_200_000})
        }
    };
    servers.insert("mirket".into(), mirket);
    for id in &options.packages {
        if registry.get(id)?["host_connection"] == false {
            continue;
        }
        let args = json!(["--home", paths.home, "package", "serve", id]);
        let mut value = match target {
            Target::Codex => json!({"command":paths.binary(),"args":args}),
            Target::Claude => json!({"type":"stdio","command":paths.binary(),"args":args}),
        };
        let request_timeout = registry.request_timeout(id)?;
        {
            match target {
                Target::Codex => {
                    value["startup_timeout_sec"] = json!(90);
                    value["tool_timeout_sec"] = json!(request_timeout.as_secs());
                }
                Target::Claude => {
                    value["timeout"] = json!(request_timeout.as_millis() as u64);
                }
            }
        }
        servers.insert(format!("mirket-package-{id}"), value);
    }
    let value = match target {
        Target::Codex => json!({"url":"https://developers.openai.com/mcp"}),
        Target::Claude => json!({"type":"http", "url":"https://developers.openai.com/mcp"}),
    };
    servers.insert("openai-docs".into(), value);
    Ok(servers)
}
fn preserve_removed_server(
    target: Target,
    name: &str,
    current: Option<&Value>,
    previous: Option<&Value>,
    wanted: Option<&Value>,
) -> bool {
    if wanted.is_some() {
        return false;
    }
    if name == "microsoft-learn" {
        let installed = match target {
            Target::Codex => json!({"url":"https://learn.microsoft.com/api/mcp"}),
            Target::Claude => json!({"type":"http","url":"https://learn.microsoft.com/api/mcp"}),
        };
        // Receipt equality and the exact owned contract are both necessary.
        // A customized entry is left intact and omitted from the new receipt.
        return previous != Some(&installed) || current != previous;
    }
    false
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
                if preserve_removed_server(target, &name, current, old.get(&name), desired) {
                    continue;
                }
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
                if preserve_removed_server(target, &name, current.as_ref(), old.get(&name), desired)
                {
                    continue;
                }
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
        crate::util::atomic_write_with_mode(path, bytes, mode)?;
        #[cfg(windows)]
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
const SETUP_COMMITTED: &[u8] = b"mirket setup committed\n";
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

fn validate_journal(bytes: &[u8]) -> Result<Journal> {
    let journal: Journal =
        serde_json::from_slice(bytes).context("invalid setup recovery journal")?;
    if journal.format != 1 || journal.files.len() > 20_000 {
        bail!("invalid setup recovery journal format or size");
    }
    let mut seen = BTreeSet::new();
    for file in &journal.files {
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
    }
    Ok(journal)
}

// Validate the transaction directory before cleanup or rollback. Once setup
// has committed, before-images may have been removed by a partially completed
// cleanup, but any retained image still has to match its journal entry.
fn validate_transaction_contents(
    directory: &Path,
    journal: &Journal,
    committed: bool,
) -> Result<BTreeSet<String>> {
    let mut allowed = BTreeSet::from(["journal.json".to_owned()]);
    if committed {
        allowed.insert("committed".to_owned());
    }
    for (index, file) in journal.files.iter().enumerate() {
        if file.before.is_some() {
            let name = format!("before-{index}");
            allowed.insert(name.clone());
            let image = directory.join(&name);
            let bytes = read_optional(&image, MAX_FILE)?;
            if let Some(bytes) = bytes {
                if sha256(&bytes) != file.before.as_deref().unwrap() {
                    bail!("setup recovery image was modified");
                }
            } else if !committed {
                bail!("setup recovery image is missing");
            }
        }
    }
    let mut present = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !allowed.contains(&name) {
            bail!("setup recovery contains an unexpected file; preserve it before setup");
        }
        if !entry.file_type()?.is_file() {
            bail!("setup recovery contains an unexpected non-file entry; preserve it");
        }
        // Read every retained file before any cleanup. This also rejects a
        // symlink or an oversized transaction artifact rather than deleting it.
        read_bounded(
            &entry.path(),
            if name == "journal.json" {
                MAX_SETTINGS
            } else {
                MAX_FILE
            },
        )?;
        present.insert(name);
    }
    Ok(present)
}

fn validate_preparation_contents(directory: &Path) -> Result<BTreeSet<String>> {
    let mut present = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let before_image = name.strip_prefix("before-").is_some_and(|index| {
            !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())
        });
        let atomic_temporary = name.strip_prefix(".tmp").is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
        });
        if !entry.file_type()?.is_file() || !(before_image || atomic_temporary) {
            bail!("incomplete setup journal contains an unexpected file; preserve it before setup");
        }
        read_bounded(&entry.path(), MAX_FILE)?;
        present.insert(name);
    }
    Ok(present)
}

fn cleanup_order(files: &BTreeSet<String>, committed: bool) -> Vec<&String> {
    let mut ordered = Vec::new();
    if !committed {
        // Rollback has already restored destinations before cleanup begins.
        // Remove the journal first so a crash cannot leave a rollback journal
        // with missing before-images after destinations have been restored.
        ordered.extend(files.iter().filter(|name| *name == "journal.json"));
    }
    ordered.extend(files.iter().filter(|name| name.starts_with("before-")));
    if committed {
        ordered.extend(files.iter().filter(|name| *name == "journal.json"));
    }
    ordered.extend(files.iter().filter(|name| *name == "committed"));
    ordered.extend(files.iter().filter(|name| {
        !name.starts_with("before-") && *name != "journal.json" && *name != "committed"
    }));
    ordered
}

fn cleanup_transaction(directory: &Path, files: &BTreeSet<String>, committed: bool) -> Result<()> {
    // Remove only entries validated before cleanup. A late or unknown entry
    // makes remove_dir fail and remains available for inspection.
    // Keep the commit marker until every before-image and the journal are gone:
    // a restart must never mistake a committed transaction for an interrupted
    // rollback merely because cleanup stopped between two unlink operations.
    for name in cleanup_order(files, committed) {
        let path = directory.join(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_file() => fs::remove_file(path)?,
            Ok(_) => bail!("setup recovery entry changed during cleanup; preserve it"),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    fs::remove_dir(directory)?;
    Ok(())
}

fn recover_setup(paths: &Paths) -> Result<()> {
    let directory = transaction_path(paths);
    reject_symlink_components(&directory)?;
    if !directory.exists() {
        return Ok(());
    }
    if let Some(marker) = read_optional(&directory.join("committed"), 32)? {
        if marker != SETUP_COMMITTED {
            bail!("invalid setup transaction commit marker");
        }
        if let Some(bytes) = read_optional(&directory.join("journal.json"), MAX_SETTINGS)? {
            let journal = validate_journal(&bytes)?;
            let files = validate_transaction_contents(&directory, &journal, true)?;
            cleanup_transaction(&directory, &files, true)?;
        } else {
            // Cleanup may have removed the journal first. A marker-only
            // directory is the sole unambiguous remainder in that state.
            let entries: Vec<_> = fs::read_dir(&directory)?.collect::<io::Result<Vec<_>>>()?;
            if entries.len() != 1 || entries[0].file_name() != "committed" {
                bail!("committed setup cleanup contains unexpected files; preserve them");
            }
            cleanup_transaction(&directory, &BTreeSet::from(["committed".to_owned()]), true)?;
        }
        return Ok(());
    }
    let Some(bytes) = read_optional(&directory.join("journal.json"), MAX_SETTINGS)? else {
        // No destination can have been written before the complete journal exists.
        let files = validate_preparation_contents(&directory)?;
        cleanup_transaction(&directory, &files, false)?;
        return Ok(());
    };
    let journal = validate_journal(&bytes)?;
    let files = validate_transaction_contents(&directory, &journal, false)?;
    let mut snapshots = Vec::new();
    for (index, file) in journal.files.iter().enumerate() {
        let before = read_optional(&directory.join(format!("before-{index}")), MAX_FILE)?;
        if before.as_ref().map(|bytes| sha256(bytes)) != file.before {
            bail!("setup recovery image was modified");
        }
        let path = paths.home.join(&file.path);
        let current = read_optional(&path, MAX_FILE)?;
        let digest = current.as_ref().map(|bytes| sha256(bytes));
        let mode = file_mode(&path)?;
        let is_before = digest == file.before && mode == file.before_mode;
        let is_after = digest == file.after && mode == file.after_mode;
        if !is_before && !is_after {
            bail!(
                "interrupted setup conflicts with a user change or ambiguous legacy partial chmod at {}; bytes and permissions preserved",
                path.display()
            );
        }
        let changed = !is_before;
        snapshots.push((path, before, file.before_mode, changed, digest, mode));
    }
    for (path, before, mode, changed, expected, expected_mode) in snapshots.into_iter().rev() {
        if changed {
            if read_optional(&path, MAX_FILE)?
                .as_ref()
                .map(|bytes| sha256(bytes))
                != expected
                || file_mode(&path)? != expected_mode
            {
                bail!("file changed during setup recovery: {}", path.display());
            }
            write_snapshot(&path, before.as_deref(), mode)?;
        }
    }
    cleanup_transaction(&directory, &files, false)?;
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
    recover_setup(paths).context("setup completed but recovery journal cleanup failed")?;
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
    let current = read_bounded(&paths.binary(), MAX_FILE).and_then(|binary| {
        let registry = crate::packages::Registry::embedded()?;
        prepare(paths, catalog, &state.options, &binary, &registry)
    });
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
    let name = target.as_str();
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

    fn add_change(
        changes: &mut Vec<Change>,
        path: PathBuf,
        after: Option<Vec<u8>>,
        executable: Option<bool>,
    ) -> Result<()> {
        let before = read_optional(&path, MAX_FILE).unwrap();
        let before_mode = file_mode(&path).unwrap();
        super::add_change(changes, path, before, before_mode, after, executable);
        Ok(())
    }

    fn assert_edit_rejected(paths: &Paths, change: Change, path: &Path, edited: &[u8]) {
        let error = apply_changes(paths, std::slice::from_ref(&change))
            .unwrap_err()
            .to_string();
        assert!(error.contains("before setup activation"), "{error}");
        assert_eq!(fs::read(path).unwrap(), edited);
        assert!(!transaction_path(paths).exists());
    }

    #[test]
    fn prepare_keeps_the_validated_receipt_snapshot_for_activation() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        paths.ensure().unwrap();
        let receipt = InstallState {
            format: 1,
            version: env!("CARGO_PKG_VERSION").into(),
            options: SetupOptions::default(),
            files: BTreeMap::new(),
            servers: BTreeMap::new(),
        };
        let mut receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        receipt_bytes.push(b'\n');
        atomic_write(&paths.settings(), &receipt_bytes).unwrap();
        let receipt_mode = file_mode(&paths.settings()).unwrap();
        let catalog = Catalog::embedded().unwrap();
        let registry = crate::packages::Registry::embedded().unwrap();

        let (changes, _) = prepare(
            &paths,
            &catalog,
            &SetupOptions::default(),
            b"runtime",
            &registry,
        )
        .unwrap();
        let change = changes
            .iter()
            .find(|change| change.path == paths.settings())
            .expect("setup receipt must be part of the activation changes");
        assert_eq!(change.before.as_deref(), Some(receipt_bytes.as_slice()));
        assert_eq!(change.before_mode, receipt_mode);
    }

    #[test]
    fn design_timeout_settings_round_trip_and_preserve_other_servers() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        let options = SetupOptions {
            packages: vec!["design".into()],
            ..SetupOptions::default()
        };
        for (target, original) in [
            (Target::Codex, b"# personal setting\nmodel = 'mine'\n[mcp_servers.personal]\ncommand = 'own-server'\n".as_slice()),
            (Target::Claude, br#"{"theme":"dark","mcpServers":{"personal":{"command":"own-server"}}}"#.as_slice()),
        ] {
            let registry = crate::packages::Registry::embedded().unwrap();
            let wanted = desired_servers(&registry, &paths, target, &options).unwrap();
            let merged = merge_config(target, Some(original), &BTreeMap::new(), &wanted).unwrap();
            assert_eq!(merge_config(target, Some(&merged), &wanted, &wanted).unwrap(), merged);
            {
                let id = "design";
                let name = format!("mirket-package-{id}");
                match target {
                    Target::Codex => {
                        let document: DocumentMut = std::str::from_utf8(&merged).unwrap().parse().unwrap();
                        assert_eq!(document["model"].as_str(), Some("mine"));
                        assert_eq!(document["mcp_servers"]["personal"]["command"].as_str(), Some("own-server"));
                        assert_eq!(toml_to_json(&document["mcp_servers"][&name]).unwrap(), wanted[&name]);
                        assert_eq!(document["mcp_servers"][&name]["startup_timeout_sec"].as_integer(), Some(90));
                        assert_eq!(document["mcp_servers"][&name]["tool_timeout_sec"].as_integer(), Some(registry.request_timeout(id).unwrap().as_secs() as i64));
                    }
                    Target::Claude => {
                        let document: Value = serde_json::from_slice(&merged).unwrap();
                        assert_eq!(document["theme"], "dark");
                        assert_eq!(document["mcpServers"]["personal"]["command"], "own-server");
                        assert_eq!(document["mcpServers"][&name], wanted[&name]);
                        assert_eq!(document["mcpServers"][&name]["timeout"], registry.request_timeout(id).unwrap().as_millis() as u64);
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
    fn edit_after_inspection_blocks_skill_and_client_config_activation() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        paths.ensure().unwrap();
        let skill = paths.home.join(".codex/skills/review/SKILL.md");
        let codex = paths.home.join(".codex/config.toml");
        let claude = paths.home.join(".claude/settings.json");
        fs::create_dir_all(skill.parent().unwrap()).unwrap();
        atomic_write(&skill, b"skill before").unwrap();
        atomic_write(&codex, b"model = 'mine'\n").unwrap();
        atomic_write(&claude, br#"{"theme":"dark"}"#).unwrap();
        let mut changes = Vec::new();

        let skill_before = read_optional(&skill, MAX_FILE).unwrap();
        let skill_mode = file_mode(&skill).unwrap();
        let owned_skill = ManagedFile {
            sha256: sha256(skill_before.as_ref().unwrap()),
            executable: false,
        };
        assert_owned(
            &skill,
            skill_before.as_deref(),
            skill_mode,
            Some(&owned_skill),
            Some(&owned_skill),
        )
        .unwrap();
        atomic_write(&skill, b"user edit after inspection").unwrap();

        super::add_change(
            &mut changes,
            skill.clone(),
            skill_before,
            skill_mode,
            Some(b"installed skill".to_vec()),
            Some(false),
        );
        let change = changes.pop().unwrap();
        assert_edit_rejected(&paths, change, &skill, b"user edit after inspection");

        let registry = crate::packages::Registry::embedded().unwrap();
        let options = SetupOptions {
            targets: vec![Target::Codex, Target::Claude],
            ..SetupOptions::default()
        };
        let wanted_codex = desired_servers(&registry, &paths, Target::Codex, &options).unwrap();
        let wanted_claude = desired_servers(&registry, &paths, Target::Claude, &options).unwrap();
        let codex_before = read_optional(&codex, MAX_SETTINGS).unwrap();
        let codex_mode = file_mode(&codex).unwrap();
        let codex_after = merge_config(
            Target::Codex,
            codex_before.as_deref(),
            &BTreeMap::new(),
            &wanted_codex,
        )
        .unwrap();
        atomic_write(&codex, b"user codex edit after inspection").unwrap();
        super::add_change(
            &mut changes,
            codex.clone(),
            codex_before,
            codex_mode,
            Some(codex_after),
            None,
        );
        let change = changes.pop().unwrap();
        assert_edit_rejected(&paths, change, &codex, b"user codex edit after inspection");

        let claude_before = read_optional(&claude, MAX_SETTINGS).unwrap();
        let claude_mode = file_mode(&claude).unwrap();
        let claude_after = merge_config(
            Target::Claude,
            claude_before.as_deref(),
            &BTreeMap::new(),
            &wanted_claude,
        )
        .unwrap();
        atomic_write(&claude, br#"{"theme":"user edit after inspection"}"#).unwrap();
        super::add_change(
            &mut changes,
            claude.clone(),
            claude_before,
            claude_mode,
            Some(claude_after),
            None,
        );
        let change = changes.pop().unwrap();
        assert_edit_rejected(
            &paths,
            change,
            &claude,
            br#"{"theme":"user edit after inspection"}"#,
        );
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
    #[cfg(unix)]
    #[test]
    fn recovery_preserves_permission_only_changes_and_ambiguous_legacy_chmod() {
        for (before_mode, after_mode, user_mode, bytes) in [
            (0o644, 0o644, 0o600, b"after".as_slice()),
            (0o600, 0o600, 0o640, b"after".as_slice()),
            (0o644, 0o644, 0o600, b"before".as_slice()),
            // Legacy atomic_write preserved the old mode before chmod. This
            // state cannot be distinguished from a user restoring that mode.
            (0o600, 0o644, 0o600, b"after".as_slice()),
            (0o644, 0o755, 0o644, b"after".as_slice()),
        ] {
            let temporary = tempfile::tempdir().unwrap();
            let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
            paths.ensure().unwrap();
            let path = paths.home.join(".codex/AGENTS.md");
            write_snapshot(&path, Some(b"before"), Some(before_mode)).unwrap();
            let changes = vec![Change {
                path: path.clone(),
                before: Some(b"before".to_vec()),
                after: Some(b"after".to_vec()),
                before_mode: Some(before_mode),
                after_mode: Some(after_mode),
            }];
            prepare_journal(&paths, &changes).unwrap();
            write_snapshot(&path, Some(bytes), Some(user_mode)).unwrap();
            let error = recover_setup(&paths).unwrap_err().to_string();
            assert!(error.contains("permissions preserved"), "{error}");
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert_eq!(file_mode(&path).unwrap(), Some(user_mode));
            assert!(transaction_path(&paths).exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn recovery_accepts_atomic_mode_transition_and_restores_deleted_files() {
        for after in [Some(b"after".to_vec()), None] {
            let temporary = tempfile::tempdir().unwrap();
            let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
            paths.ensure().unwrap();
            let path = paths.home.join(".codex/AGENTS.md");
            write_snapshot(&path, Some(b"before"), Some(0o644)).unwrap();
            let mode = after.as_ref().map(|_| 0o600);
            let changes = vec![Change {
                path: path.clone(),
                before: Some(b"before".to_vec()),
                after: after.clone(),
                before_mode: Some(0o644),
                after_mode: mode,
            }];
            prepare_journal(&paths, &changes).unwrap();
            write_snapshot(&path, after.as_deref(), mode).unwrap();
            assert_eq!(file_mode(&path).unwrap(), mode);
            recover_setup(&paths).unwrap();
            assert_eq!(fs::read(&path).unwrap(), b"before");
            assert_eq!(file_mode(&path).unwrap(), Some(0o644));
            assert!(!transaction_path(&paths).exists());
        }
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

    #[test]
    fn committed_setup_preserves_user_edits_and_rejects_unknown_transaction_files() {
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
        write_snapshot(&instruction, Some(b"after"), Some(0o600)).unwrap();
        atomic_write(&transaction_path(&paths).join("committed"), SETUP_COMMITTED).unwrap();

        atomic_write(&transaction_path(&paths).join("user-note"), b"preserve me").unwrap();
        let error = recover_setup(&paths).unwrap_err().to_string();
        assert!(error.contains("unexpected file"), "{error}");
        assert_eq!(
            fs::read(transaction_path(&paths).join("user-note")).unwrap(),
            b"preserve me"
        );

        fs::remove_file(transaction_path(&paths).join("user-note")).unwrap();
        write_snapshot(&instruction, Some(b"user edit after commit"), Some(0o600)).unwrap();
        recover_setup(&paths).unwrap();
        assert_eq!(fs::read(instruction).unwrap(), b"user edit after commit");
        assert!(!transaction_path(&paths).exists());
    }

    #[test]
    fn committed_cleanup_keeps_marker_until_journal_is_removed() {
        let names = BTreeSet::from([
            "before-0".to_owned(),
            "journal.json".to_owned(),
            "committed".to_owned(),
        ]);
        let order = cleanup_order(&names, true)
            .into_iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(order, ["before-0", "journal.json", "committed"]);
    }

    #[test]
    fn rollback_cleanup_removes_journal_before_before_images() {
        let names = BTreeSet::from(["before-0".to_owned(), "journal.json".to_owned()]);
        let order = cleanup_order(&names, false)
            .into_iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(order, ["journal.json", "before-0"]);
    }

    #[test]
    fn rollback_cleanup_preserves_late_unknown_entries() {
        let temporary = tempfile::tempdir().unwrap();
        let directory = temporary
            .path()
            .canonicalize()
            .unwrap()
            .join(".setup-transaction");
        fs::create_dir(&directory).unwrap();
        atomic_write(&directory.join("journal.json"), b"journal").unwrap();
        atomic_write(&directory.join("before-0"), b"before").unwrap();

        // The late entry was not in the validated set. Cleanup removes only
        // the entries that were validated before it appeared, then leaves the
        // directory (and the late evidence) available for inspection.
        let files = BTreeSet::from(["journal.json".to_owned(), "before-0".to_owned()]);
        atomic_write(&directory.join("late-note"), b"preserve me").unwrap();
        assert!(cleanup_transaction(&directory, &files, false).is_err());
        assert_eq!(
            fs::read(directory.join("late-note")).unwrap(),
            b"preserve me"
        );
        assert!(directory.exists());
    }

    #[test]
    fn recovery_accepts_rollback_cleanup_after_journal_was_removed() {
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

        // Simulate the state after rollback restored the destination and the
        // journal unlink completed, but before before-image cleanup did.
        fs::remove_file(transaction_path(&paths).join("journal.json")).unwrap();
        recover_setup(&paths).unwrap();
        assert_eq!(fs::read(&instruction).unwrap(), b"before");
        assert!(!transaction_path(&paths).exists());
    }

    #[test]
    fn committed_recovery_accepts_marker_only_partial_cleanup_without_rollback() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        paths.ensure().unwrap();
        let instruction = paths.home.join(".codex/AGENTS.md");
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
        atomic_write(&transaction_path(&paths).join("committed"), SETUP_COMMITTED).unwrap();
        fs::remove_file(transaction_path(&paths).join("journal.json")).unwrap();

        recover_setup(&paths).unwrap();
        assert_eq!(fs::read(instruction).unwrap(), b"committed");
        assert!(!transaction_path(&paths).exists());
    }

    #[test]
    fn rollback_rejects_unknown_transaction_files_before_restoring_anything() {
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
        write_snapshot(&instruction, Some(b"after"), Some(0o600)).unwrap();
        atomic_write(&transaction_path(&paths).join("user-note"), b"preserve me").unwrap();

        let error = recover_setup(&paths).unwrap_err().to_string();
        assert!(error.contains("unexpected file"), "{error}");
        assert_eq!(fs::read(&instruction).unwrap(), b"after");
        assert_eq!(
            fs::read(transaction_path(&paths).join("user-note")).unwrap(),
            b"preserve me"
        );
        assert!(transaction_path(&paths).exists());
    }
}
