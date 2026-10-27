//! Transactional registration of Mirket's MCP server in Codex TOML config.
//!
//! The caller supplies both the Codex config path and the installation-owned
//! receipt path. This module never discovers a host home, copies credentials,
//! edits client policy, or falls back to another config format.

use crate::{file_lock::FileLock, release::ReleaseId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use toml_edit::{Array, DocumentMut, Item, value};

const SERVER_NAME: &str = "mirket";
const MAX_CONFIG_BYTES: u64 = 4 * 1024 * 1024;
const MAX_LAUNCHER_BYTES: u64 = 256 * 1024 * 1024;
const MAX_RECEIPT_BYTES: u64 = 32 * 1024;
const RECEIPT_SCHEMA: u8 = 1;
const STARTUP_TIMEOUT_SEC: i64 = 60;

#[derive(Clone, Debug)]
pub struct CodexConfig {
    config_path: PathBuf,
    receipt_path: PathBuf,
    lock_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct CodexRegistrationRequest {
    pub release_id: String,
    pub home: PathBuf,
    pub launcher: PathBuf,
}

#[derive(Clone, Debug, Serialize)]
pub struct CodexConfigReport {
    pub config_path: PathBuf,
    pub server_name: &'static str,
    pub status: &'static str,
    pub managed_release_id: Option<String>,
    pub entry_digest: Option<String>,
    pub config_digest: Option<String>,
    pub pending_transaction: bool,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CodexRegistrationResult {
    pub status: &'static str,
    pub config_path: PathBuf,
    pub release_id: String,
    pub entry_digest: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CodexRemovalResult {
    pub status: &'static str,
    pub config_path: PathBuf,
    pub release_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CodexReconciliationResult {
    pub status: &'static str,
    pub config_path: PathBuf,
    pub release_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u8,
    action: ReceiptAction,
    config_path: String,
    release_id: String,
    launcher_path: String,
    launcher_digest: String,
    entry_digest: String,
    previous_present: bool,
    previous_config_digest: String,
    desired_config_digest: String,
    state: ReceiptState,
    prior: Option<Box<Receipt>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReceiptAction {
    Register,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReceiptState {
    Pending,
    Committed,
}

struct Snapshot {
    present: bool,
    digest: String,
    document: DocumentMut,
}

impl CodexConfig {
    /// Open an explicitly selected Codex config and installation receipt.
    pub fn open(config_path: &Path, receipt_path: &Path) -> Result<Self, String> {
        let config_path = select_path(config_path, "Codex config")?;
        let receipt_path = select_path(receipt_path, "Codex receipt")?;
        let parent = config_path.parent().ok_or("Codex config has no parent")?;
        verify_config_parent(parent)?;
        let receipt_parent = receipt_path.parent().ok_or("Codex receipt has no parent")?;
        verify_parent(receipt_parent, "Codex receipt parent")?;
        let lock_path = receipt_path.with_extension("lock");
        Ok(Self {
            config_path,
            receipt_path,
            lock_path,
        })
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Inspect without taking a runtime/session lock or writing the config.
    pub fn inspect(&self) -> Result<CodexConfigReport, String> {
        let receipt = read_receipt(&self.receipt_path)?;
        let snapshot = read_snapshot(&self.config_path)?;
        let mut report = CodexConfigReport {
            config_path: self.config_path.clone(),
            server_name: SERVER_NAME,
            status: "missing",
            managed_release_id: receipt.as_ref().map(|r| r.release_id.clone()),
            entry_digest: None,
            config_digest: snapshot.present.then_some(snapshot.digest.clone()),
            pending_transaction: receipt
                .as_ref()
                .is_some_and(|r| matches!(r.state, ReceiptState::Pending)),
            errors: Vec::new(),
        };
        let Some(item) = server_item(&snapshot.document)? else {
            if receipt.is_some() {
                report.status = "drifted";
                report.errors.push("managed Codex entry is missing".into());
            }
            return Ok(report);
        };
        let digest = match entry_digest(item) {
            Ok(digest) => digest,
            Err(error) => {
                report.status = "drifted";
                report.errors.push(error);
                return Ok(report);
            }
        };
        report.entry_digest = Some(digest.clone());
        match receipt {
            Some(receipt) if receipt.entry_digest == digest => {
                report.status = if matches!(receipt.state, ReceiptState::Pending) {
                    "pending"
                } else {
                    "managed"
                };
            }
            Some(_) => {
                report.status = "drifted";
                report
                    .errors
                    .push("managed Codex entry changed externally".into());
            }
            None => {
                report.status = "conflict";
                report
                    .errors
                    .push("Codex entry exists without a Mirket receipt".into());
            }
        }
        Ok(report)
    }

    /// A lifecycle may reconcile only its own target and rollback receipt.
    /// Inspection stays read-only and uses the native owner's receipt parser.
    pub(crate) fn verify_receipt_release(&self, expected: &str) -> Result<(), String> {
        if let Some(receipt) = read_receipt(&self.receipt_path)? {
            validate_receipt(&receipt, &self.config_path)?;
            if receipt.release_id != expected
                || receipt
                    .prior
                    .as_ref()
                    .is_some_and(|prior| prior.release_id != expected)
            {
                return Err(
                    "native receipt transition belongs to another lifecycle release; preserved"
                        .into(),
                );
            }
        }
        Ok(())
    }

    pub fn register(
        &self,
        request: &CodexRegistrationRequest,
    ) -> Result<CodexRegistrationResult, String> {
        let _lock = self.acquire_lock()?;
        let release = ReleaseId::parse(&request.release_id).map_err(|e| e.to_string())?;
        validate_home(&request.home)?;
        validate_launcher_location(&request.home, &request.launcher)?;
        let launcher_digest = validate_launcher(&request.launcher)?;
        let receipt = read_receipt(&self.receipt_path)?;
        if let Some(receipt) = &receipt {
            validate_receipt(receipt, &self.config_path)?;
            if matches!(receipt.state, ReceiptState::Pending) {
                return Err(
                    "Codex registration has a pending transaction; reconcile it first".into(),
                );
            }
        }
        let snapshot = read_snapshot(&self.config_path)?;
        let current = server_item(&snapshot.document)?;
        if let Some(item) = current {
            let digest = entry_digest(item)?;
            match &receipt {
                Some(receipt) if receipt.entry_digest == digest => (),
                Some(_) => return Err("managed Codex entry changed externally".into()),
                None => return Err("Codex entry exists without a Mirket receipt".into()),
            }
        } else if receipt.is_some() {
            return Err("managed Codex entry is missing; refusing implicit recreation".into());
        }
        let mut document = snapshot.document;
        let mcp = document
            .entry("mcp_servers")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .ok_or("Codex mcp_servers must be a TOML table")?;
        mcp.insert(SERVER_NAME, desired_item(&request.launcher, &request.home)?);
        let desired_item = mcp
            .get(SERVER_NAME)
            .ok_or("failed to create Mirket Codex entry")?;
        let entry_digest = entry_digest(desired_item)?;
        let desired_bytes = document.to_string().into_bytes();
        let desired_digest = digest_bytes(&desired_bytes);
        let receipt = Receipt {
            schema_version: RECEIPT_SCHEMA,
            action: ReceiptAction::Register,
            config_path: self.config_path.to_string_lossy().into_owned(),
            release_id: release.as_str().to_owned(),
            launcher_path: request.launcher.to_string_lossy().into_owned(),
            launcher_digest,
            entry_digest: entry_digest.clone(),
            previous_present: snapshot.present,
            previous_config_digest: snapshot.digest.clone(),
            desired_config_digest: desired_digest,
            state: ReceiptState::Pending,
            prior: receipt.map(Box::new),
        };
        write_receipt(&self.receipt_path, &receipt)?;
        if let Err(error) = replace_config(
            &self.config_path,
            snapshot.present,
            &receipt.previous_config_digest,
            &desired_bytes,
        ) {
            clear_owned_pending(
                &self.receipt_path,
                &receipt,
                snapshot.present,
                &snapshot.digest,
            )?;
            return Err(error);
        }
        let mut committed = receipt;
        committed.state = ReceiptState::Committed;
        committed.prior = None;
        write_receipt(&self.receipt_path, &committed)?;
        Ok(CodexRegistrationResult {
            status: "registered",
            config_path: self.config_path.clone(),
            release_id: committed.release_id,
            entry_digest,
        })
    }

    pub fn remove(&self) -> Result<CodexRemovalResult, String> {
        let _lock = self.acquire_lock()?;
        let receipt = read_receipt(&self.receipt_path)?.ok_or("Mirket Codex receipt is missing")?;
        validate_receipt(&receipt, &self.config_path)?;
        if !matches!(receipt.action, ReceiptAction::Register)
            || !matches!(receipt.state, ReceiptState::Committed)
        {
            return Err("Codex registration has a pending transaction; reconcile it first".into());
        }
        let snapshot = read_snapshot(&self.config_path)?;
        let item = server_item(&snapshot.document)?.ok_or("managed Codex entry is missing")?;
        if entry_digest(item)? != receipt.entry_digest {
            return Err("managed Codex entry changed externally".into());
        }
        let mut document = snapshot.document;
        let mcp = document
            .get_mut("mcp_servers")
            .and_then(Item::as_table_mut)
            .ok_or("Codex mcp_servers must be a TOML table")?;
        mcp.remove(SERVER_NAME);
        let desired_bytes = document.to_string().into_bytes();
        let desired_digest = digest_bytes(&desired_bytes);
        let mut pending = receipt.clone();
        pending.action = ReceiptAction::Remove;
        pending.previous_present = snapshot.present;
        pending.previous_config_digest = snapshot.digest.clone();
        pending.desired_config_digest = desired_digest;
        pending.state = ReceiptState::Pending;
        pending.prior = Some(Box::new(receipt.clone()));
        write_receipt(&self.receipt_path, &pending)?;
        if let Err(error) = replace_config(
            &self.config_path,
            snapshot.present,
            &pending.previous_config_digest,
            &desired_bytes,
        ) {
            clear_owned_pending(
                &self.receipt_path,
                &pending,
                snapshot.present,
                &snapshot.digest,
            )?;
            return Err(error);
        }
        fs::remove_file(&self.receipt_path).map_err(io_message)?;
        Ok(CodexRemovalResult {
            status: "removed",
            config_path: self.config_path.clone(),
            release_id: receipt.release_id,
        })
    }

    pub fn reconcile(&self) -> Result<CodexReconciliationResult, String> {
        let _lock = self.acquire_lock()?;
        let Some(receipt) = read_receipt(&self.receipt_path)? else {
            return Ok(CodexReconciliationResult {
                status: "none",
                config_path: self.config_path.clone(),
                release_id: None,
            });
        };
        validate_receipt(&receipt, &self.config_path)?;
        let snapshot = read_snapshot(&self.config_path)?;
        if snapshot.present && snapshot.digest == receipt.desired_config_digest {
            if matches!(receipt.action, ReceiptAction::Remove) {
                fs::remove_file(&self.receipt_path).map_err(io_message)?;
                return Ok(CodexReconciliationResult {
                    status: "removed",
                    config_path: self.config_path.clone(),
                    release_id: Some(receipt.release_id),
                });
            }
            let mut committed = receipt.clone();
            committed.state = ReceiptState::Committed;
            committed.prior = None;
            write_receipt(&self.receipt_path, &committed)?;
            return Ok(CodexReconciliationResult {
                status: "committed",
                config_path: self.config_path.clone(),
                release_id: Some(receipt.release_id),
            });
        }
        if snapshot.present == receipt.previous_present
            && snapshot.digest == receipt.previous_config_digest
        {
            restore_prior_receipt(&self.receipt_path, &receipt)?;
            return Ok(CodexReconciliationResult {
                status: "rolled_back",
                config_path: self.config_path.clone(),
                release_id: Some(receipt.release_id),
            });
        }
        Err(
            "pending Codex transaction has an unknown config state; inspect and resolve explicitly"
                .into(),
        )
    }

    fn acquire_lock(&self) -> Result<FileLock, String> {
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = match options.open(&self.lock_path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let metadata = fs::symlink_metadata(&self.lock_path).map_err(io_message)?;
                verify_lock(&metadata)?;
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&self.lock_path)
                    .map_err(io_message)?
            }
            Err(error) => return Err(io_message(error)),
        };
        file.try_lock()
            .map_err(|_| "another Mirket Codex config operation owns the lock")?;
        let owned = FileLock::acquired(file);
        let opened = owned.as_file().metadata().map_err(io_message)?;
        let current = fs::symlink_metadata(&self.lock_path).map_err(io_message)?;
        verify_lock(&opened)?;
        verify_lock(&current)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (opened.dev(), opened.ino()) != (current.dev(), current.ino()) {
                return Err("Codex config lock changed while acquiring ownership".into());
            }
        }
        Ok(owned)
    }
}

fn verify_lock(metadata: &fs::Metadata) -> Result<(), String> {
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() != 0 {
        return Err("Codex config lock must be an empty regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.permissions().mode() & 0o777 != 0o600 || metadata.nlink() != 1 {
            return Err("Codex config lock must be private and unaliased".into());
        }
    }
    Ok(())
}

fn select_path(path: &Path, name: &str) -> Result<PathBuf, String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!("{name} path must be absolute and traversal-free"));
    }
    if let Ok(metadata) = fs::symlink_metadata(path)
        && metadata.file_type().is_symlink()
    {
        return Err(format!("{name} path must not be a symlink"));
    }
    Ok(path.to_owned())
}

fn verify_parent(path: &Path, name: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(io_message)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{name} must be a real directory"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(format!("{name} must be private"));
        }
    }
    Ok(())
}

fn verify_config_parent(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(io_message)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("Codex config parent must be a real directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let uid = unsafe { libc::geteuid() };
        if metadata.uid() != uid || metadata.permissions().mode() & 0o022 != 0 {
            return Err(
                "Codex config parent must be owned by the current user and not group/other writable"
                    .into(),
            );
        }
    }
    Ok(())
}

fn validate_home(home: &Path) -> Result<(), String> {
    if !home.is_absolute()
        || home
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("Mirket home must be absolute and traversal-free".into());
    }
    verify_parent(home, "Mirket home")
}

fn validate_launcher(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(io_message)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("Mirket launcher must be a regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err("Mirket launcher must be executable".into());
        }
    }
    Ok(digest_bytes(&read_bounded(path, MAX_LAUNCHER_BYTES)?))
}

fn validate_launcher_location(home: &Path, launcher: &Path) -> Result<(), String> {
    let expected_parent = home.join("bin");
    if launcher.parent() != Some(expected_parent.as_path()) {
        return Err("Mirket launcher must be the stable home/bin launcher".into());
    }
    let expected_name = if cfg!(windows) {
        "mirket.exe"
    } else {
        "mirket"
    };
    if launcher.file_name().and_then(|name| name.to_str()) != Some(expected_name) {
        return Err("Mirket launcher has an unexpected stable filename".into());
    }
    verify_parent(&expected_parent, "Mirket launcher directory")
}

fn server_item(document: &DocumentMut) -> Result<Option<&Item>, String> {
    let Some(item) = document.get("mcp_servers") else {
        return Ok(None);
    };
    let table = item
        .as_table()
        .ok_or("Codex mcp_servers must be a TOML table")?;
    Ok(table.get(SERVER_NAME))
}

fn desired_item(launcher: &Path, home: &Path) -> Result<Item, String> {
    let launcher = launcher
        .to_str()
        .ok_or("Mirket launcher path must be valid UTF-8")?;
    let home = home
        .to_str()
        .ok_or("Mirket home path must be valid UTF-8")?;
    let mut item = toml_edit::table();
    let table = item
        .as_table_mut()
        .ok_or("failed to create Codex server table")?;
    table.insert("command", value(launcher));
    let mut args = Array::new();
    args.push("serve");
    args.push("--home");
    args.push(home);
    table.insert("args", value(args));
    table.insert("startup_timeout_sec", value(STARTUP_TIMEOUT_SEC));
    table.insert("required", value(true));
    Ok(item)
}

fn entry_digest(item: &Item) -> Result<String, String> {
    let table = item
        .as_table()
        .ok_or("Codex Mirket entry must be a TOML table")?;
    if table.len() != 4
        || !table.contains_key("command")
        || !table.contains_key("args")
        || table
            .get("required")
            .and_then(Item::as_value)
            .and_then(|value| value.as_bool())
            != Some(true)
        || table
            .get("startup_timeout_sec")
            .and_then(Item::as_value)
            .and_then(|value| value.as_integer())
            != Some(STARTUP_TIMEOUT_SEC)
    {
        return Err("Codex Mirket entry contains unsupported or missing keys".into());
    }
    let command = table
        .get("command")
        .and_then(Item::as_value)
        .and_then(|value| value.as_str())
        .ok_or("Codex Mirket command must be a string")?;
    let args = table
        .get("args")
        .and_then(Item::as_value)
        .and_then(|value| value.as_array())
        .ok_or("Codex Mirket args must be an array")?;
    let args = args
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or("Codex Mirket args must contain only strings")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let canonical =
        serde_json::to_vec(&(command, args, STARTUP_TIMEOUT_SEC, true)).map_err(io_message)?;
    Ok(digest_bytes(&canonical))
}

fn read_snapshot(path: &Path) -> Result<Snapshot, String> {
    let (present, bytes) = match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("Codex config must be a regular file".into());
            }
            ensure_private_file(&metadata, "Codex config")?;
            if metadata.len() > MAX_CONFIG_BYTES {
                return Err("Codex config exceeds its size limit".into());
            }
            (true, read_bounded(path, MAX_CONFIG_BYTES)?)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (false, Vec::new()),
        Err(error) => return Err(io_message(error)),
    };
    let text = std::str::from_utf8(&bytes).map_err(|_| "Codex config is not UTF-8")?;
    let document = text
        .parse::<DocumentMut>()
        .map_err(|error| format!("Codex config TOML is invalid: {error}"))?;
    Ok(Snapshot {
        present,
        digest: digest_bytes(&bytes),
        document,
    })
}

fn read_receipt(path: &Path) -> Result<Option<Receipt>, String> {
    let bytes = match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("Codex receipt must be a regular file".into());
            }
            ensure_private_file(&metadata, "Codex receipt")?;
            read_bounded(path, MAX_RECEIPT_BYTES)?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_message(error)),
    };
    let receipt: Receipt = serde_json::from_slice(&bytes).map_err(io_message)?;
    if receipt.schema_version != RECEIPT_SCHEMA {
        return Err("unsupported Codex receipt schema".into());
    }
    Ok(Some(receipt))
}

fn ensure_private_file(metadata: &fs::Metadata, name: &str) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o777 != 0o600 {
            return Err(format!("{name} must have private 0600 permissions"));
        }
    }
    let _ = (metadata, name);
    Ok(())
}

fn validate_receipt(receipt: &Receipt, config_path: &Path) -> Result<(), String> {
    if receipt.config_path != config_path.to_string_lossy() {
        return Err("Codex receipt belongs to a different config path".into());
    }
    ReleaseId::parse(&receipt.release_id).map_err(|error| error.to_string())?;
    match (&receipt.state, &receipt.prior) {
        (ReceiptState::Committed, None) | (ReceiptState::Pending, None) => {}
        (ReceiptState::Pending, Some(prior))
            if matches!(prior.state, ReceiptState::Committed)
                && prior.prior.is_none()
                && prior.config_path == receipt.config_path => {}
        _ => return Err("Codex receipt has an invalid prior state".into()),
    }
    Ok(())
}

fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(io_message)?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(io_message)?;
    if bytes.len() as u64 > max {
        return Err("managed file grew beyond its limit".into());
    }
    Ok(bytes)
}

fn write_receipt(path: &Path, receipt: &Receipt) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(receipt).map_err(io_message)?;
    write_atomic(path, &bytes)
}

fn replace_config(
    path: &Path,
    expected_present: bool,
    expected_digest: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let snapshot = read_snapshot(path)?;
    if snapshot.present != expected_present || snapshot.digest != expected_digest {
        return Err("Codex config changed during registration".into());
    }
    write_atomic(path, bytes)
}

/// Clear a pending receipt only when the failed replacement left the exact
/// pre-image in place and the receipt still matches the transaction we wrote.
/// If the config changed, retain the pending receipt so reconcile can report
/// the ambiguous state instead of erasing evidence of a possible race.
fn clear_owned_pending(
    receipt_path: &Path,
    expected: &Receipt,
    previous_present: bool,
    previous_digest: &str,
) -> Result<(), String> {
    let Some(current) = read_receipt(receipt_path)? else {
        return Ok(());
    };
    if &current != expected || !matches!(current.state, ReceiptState::Pending) {
        return Ok(());
    }
    let snapshot = match read_snapshot(&PathBuf::from(&current.config_path)) {
        Ok(snapshot) => snapshot,
        Err(_) => return Ok(()),
    };
    if snapshot.present == previous_present && snapshot.digest == previous_digest {
        restore_prior_receipt(receipt_path, &current)?;
    }
    Ok(())
}

fn restore_prior_receipt(path: &Path, pending: &Receipt) -> Result<(), String> {
    if let Some(prior) = &pending.prior {
        write_receipt(path, prior)
    } else {
        fs::remove_file(path).map_err(io_message)
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_file_name(format!(
        ".{}.mirket-next-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config"),
        std::process::id()
    ));
    if fs::symlink_metadata(&temporary).is_ok() {
        return Err("managed temporary config already exists; inspect it before retrying".into());
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary).map_err(io_message)?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(&temporary);
        return Err(io_message(error));
    }
    if let Err(error) = replace_file(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    sync_parent(path.parent().ok_or("managed file has no parent")?)
}

fn replace_file(from: &Path, to: &Path) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        fs::rename(from, to).map_err(io_message)
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let from: Vec<u16> = from.as_os_str().encode_wide().chain([0]).collect();
        let to: Vec<u16> = to.as_os_str().encode_wide().chain([0]).collect();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            return Err(io_message(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

fn sync_parent(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(io_message)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

fn io_message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "mirket-codex-config-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("codex")).unwrap();
            fs::create_dir_all(root.join("mirket/state")).unwrap();
            fs::create_dir_all(root.join("mirket/bin")).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                for path in [
                    root.clone(),
                    root.join("codex"),
                    root.join("mirket"),
                    root.join("mirket/state"),
                    root.join("mirket/bin"),
                ] {
                    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
                }
            }
            Self(root)
        }
        fn config(&self) -> PathBuf {
            self.0.join("codex/config.toml")
        }
        fn receipt(&self) -> PathBuf {
            self.0.join("mirket/state/codex.json")
        }
        fn write_config(&self, bytes: &[u8]) {
            fs::write(self.config(), bytes).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(self.config(), fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
        fn request(&self) -> CodexRegistrationRequest {
            let launcher = self.0.join("mirket/bin/mirket");
            fs::write(&launcher, b"launcher").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
            }
            CodexRegistrationRequest {
                release_id:
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
                home: self.0.join("mirket"),
                launcher,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn registration_preserves_unrelated_toml_and_round_trips() {
        let fixture = Fixture::new();
        fixture.write_config(
            "# keep this\nmodel = \"gpt\"\n\n[mcp_servers.other]\ncommand = \"other\"\n".as_bytes(),
        );
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        let request = fixture.request();
        let result = manager.register(&request).unwrap();
        assert_eq!(result.status, "registered");
        let text = fs::read_to_string(fixture.config()).unwrap();
        assert!(text.contains("# keep this"));
        assert!(text.contains("[mcp_servers.other]"));
        assert!(text.contains("[mcp_servers.mirket]"));
        assert!(text.contains("startup_timeout_sec = 60"));
        assert!(text.contains("required = true"));
        assert_eq!(manager.inspect().unwrap().status, "managed");
        fixture.write_config(
            text.replace("required = true", "required = false")
                .as_bytes(),
        );
        assert_eq!(manager.inspect().unwrap().status, "drifted");
        fixture.write_config(text.as_bytes());
        fixture.write_config(
            text.replace("startup_timeout_sec = 60", "startup_timeout_sec = 10")
                .as_bytes(),
        );
        assert_eq!(manager.inspect().unwrap().status, "drifted");
        assert!(manager.remove().is_err());
        fixture.write_config(text.as_bytes());
        manager.remove().unwrap();
        assert_eq!(manager.inspect().unwrap().status, "missing");
        assert!(
            fs::read_to_string(fixture.config())
                .unwrap()
                .contains("[mcp_servers.other]")
        );
    }

    #[test]
    fn existing_entry_without_receipt_and_drift_are_refused() {
        let fixture = Fixture::new();
        fixture.write_config(
            "[mcp_servers.mirket]\ncommand = \"/user/owned\"\nargs = [\"serve\"]\n".as_bytes(),
        );
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        assert!(manager.register(&fixture.request()).is_err());
        let mut request = fixture.request();
        manager.register(&request).err();
        fixture.write_config(
            "[mcp_servers.mirket]\ncommand = \"/user/owned\"\nargs = [\"changed\"]\n".as_bytes(),
        );
        request.release_id =
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
        assert!(manager.register(&request).is_err());
    }

    #[test]
    fn missing_config_is_created_and_pending_reconciliation_is_safe() {
        let fixture = Fixture::new();
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        manager.register(&fixture.request()).unwrap();
        assert!(fixture.config().is_file());
        assert!(fixture.receipt().is_file());
        manager.remove().unwrap();
        assert!(!fixture.receipt().exists());
    }

    #[cfg(unix)]
    #[test]
    fn accepts_user_owned_non_writable_codex_parent() {
        let fixture = Fixture::new();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(fixture.0.join("codex"), fs::Permissions::from_mode(0o755)).unwrap();
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        manager.register(&fixture.request()).unwrap();
        assert_eq!(manager.inspect().unwrap().status, "managed");
    }

    #[test]
    fn failed_replace_clears_only_owned_pending_receipt_when_preimage_remains() {
        let fixture = Fixture::new();
        let original = b"model = \"gpt\"\n";
        fixture.write_config(original);
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        let temporary = fixture.config().with_file_name(format!(
            ".{}.mirket-next-{}",
            fixture.config().file_name().unwrap().to_string_lossy(),
            std::process::id()
        ));
        fs::write(&temporary, b"occupied").unwrap();
        assert!(manager.register(&fixture.request()).is_err());
        assert!(!fixture.receipt().exists());
        assert_eq!(fs::read(fixture.config()).unwrap(), original);
        fs::remove_file(temporary).unwrap();
    }

    #[test]
    fn failed_repeat_registration_and_removal_restore_existing_ownership() {
        let fixture = Fixture::new();
        fixture.write_config(b"model = \"gpt\"\n");
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        let first = fixture.request();
        manager.register(&first).unwrap();
        let owned_config = fs::read(&manager.config_path).unwrap();
        let owned_receipt = fs::read(&manager.receipt_path).unwrap();
        let temporary = fixture.config().with_file_name(format!(
            ".{}.mirket-next-{}",
            fixture.config().file_name().unwrap().to_string_lossy(),
            std::process::id()
        ));
        fs::write(&temporary, b"occupied").unwrap();

        let mut second = first.clone();
        second.release_id =
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
        assert!(manager.register(&second).is_err());
        assert_eq!(fs::read(&manager.config_path).unwrap(), owned_config);
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), owned_receipt);
        assert_eq!(manager.inspect().unwrap().status, "managed");

        assert!(manager.remove().is_err());
        assert_eq!(fs::read(&manager.config_path).unwrap(), owned_config);
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), owned_receipt);
        assert_eq!(manager.inspect().unwrap().status, "managed");
        fs::remove_file(temporary).unwrap();
        manager.remove().unwrap();
    }

    #[test]
    fn pending_repeat_edit_reconcile_restores_prior_receipt_on_preimage() {
        let fixture = Fixture::new();
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        let first = fixture.request();
        manager.register(&first).unwrap();
        let prior = read_receipt(&manager.receipt_path).unwrap().unwrap();
        let mut pending = prior.clone();
        pending.state = ReceiptState::Pending;
        pending.release_id =
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
        pending.desired_config_digest = digest_bytes(b"a different desired config");
        pending.previous_present = true;
        pending.previous_config_digest = read_snapshot(&manager.config_path).unwrap().digest;
        pending.prior = Some(Box::new(prior.clone()));
        write_receipt(&manager.receipt_path, &pending).unwrap();
        assert_eq!(manager.reconcile().unwrap().status, "rolled_back");
        assert_eq!(read_receipt(&manager.receipt_path).unwrap(), Some(prior));
        assert_eq!(manager.inspect().unwrap().status, "managed");
    }

    #[test]
    fn nonempty_config_lock_refuses_before_any_config_or_receipt_change() {
        let fixture = Fixture::new();
        let original = b"model = \"gpt\"\n";
        fixture.write_config(original);
        let manager = CodexConfig::open(&fixture.config(), &fixture.receipt()).unwrap();
        let lock = fixture.receipt().with_extension("lock");
        fs::write(&lock, b"foreign lock data").unwrap();
        assert!(manager.register(&fixture.request()).is_err());
        assert_eq!(fs::read(&lock).unwrap(), b"foreign lock data");
        assert_eq!(fs::read(fixture.config()).unwrap(), original);
        assert!(!fixture.receipt().exists());
    }
}
