//! Bounded operation records, separate from canonical memory and effect authority.
//! Records are server-issued: a missing ID is never a new operation. The caller
//! serializes the minimal receipt, never document bodies or credentials. All
//! mutations use the same native lock; publication remains a separate effect.
use crate::file_lock::FileLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub const MAX_RECEIPT_BYTES: usize = 64 * 1024;
pub const MAX_STORE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_RECORDS: usize = 4096;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
const RETENTION_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
enum Retention {
    Protected,
    Retirable { completed_at: u64 },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: u8,
    retention: Retention,
    issued_payload_sha256: String,
    payload: String,
}
impl Envelope {
    fn decode(bytes: &[u8]) -> std::io::Result<Self> {
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| error("invalid operation envelope; preserve for inspection"))?;
        if value.schema != 2
            || value.issued_payload_sha256.len() != 64
            || !value
                .issued_payload_sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(error(
                "unsupported operation envelope; preserve for inspection",
            ));
        }
        Ok(value)
    }
    fn expired(&self, now: u64) -> bool {
        matches!(self.retention, Retention::Retirable { completed_at }
            if now.checked_sub(completed_at).is_some_and(|age| age >= RETENTION_SECONDS))
    }
    fn readable(self, now: u64) -> std::io::Result<Vec<u8>> {
        if self.expired(now) {
            return Err(error(
                "operation receipt expired; missing evidence never permits replay",
            ));
        }
        Ok(self.payload.into_bytes())
    }
}
fn payload_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn now_seconds() -> std::io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_secs())
        .map_err(|_| error("system clock predates epoch"))
}
struct InventoryEntry {
    id: String,
    bytes: u64,
    completed_at: Option<u64>,
}

#[derive(Clone, Copy)]
struct StoreLimits {
    receipt_bytes: usize,
    store_bytes: u64,
    records: usize,
}
const STORE_LIMITS: StoreLimits = StoreLimits {
    receipt_bytes: MAX_RECEIPT_BYTES,
    store_bytes: MAX_STORE_BYTES,
    records: MAX_RECORDS,
};

#[derive(Clone)]
pub struct OperationStore {
    root: PathBuf,
    limits: StoreLimits,
}
pub struct LockedStore {
    root: PathBuf,
    limits: StoreLimits,
    _lock: FileLock,
}

fn error(message: &str) -> std::io::Error {
    std::io::Error::other(message)
}
fn lock_error(error: TryLockError) -> std::io::Error {
    match error {
        TryLockError::WouldBlock => std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "operation store is busy; retry inspection without replaying effects",
        ),
        TryLockError::Error(error) => error,
    }
}
fn private_file(path: &Path, new: bool) -> std::io::Result<File> {
    open_private_file(path, new, true)
}
fn open_private_file(path: &Path, new: bool, writable: bool) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(writable);
    if new {
        options.create_new(true);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(error("receipt entry must be a regular private file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.mode() & 0o077 != 0 || meta.nlink() != 1 {
            return Err(error("receipt entry must be private and unaliased"));
        }
    }
    Ok(file)
}
fn sync_directory(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    // Windows directory flush and power-loss behavior require native qualification.
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 100
        && id.starts_with("op-")
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

impl OperationStore {
    pub fn open(state_root: &Path) -> std::io::Result<Self> {
        Self::open_impl(state_root, true)
    }
    /// Inspect an existing receipt namespace without creating a directory.
    pub fn open_existing(state_root: &Path) -> std::io::Result<Self> {
        Self::open_impl(state_root, false)
    }
    fn open_impl(state_root: &Path, create: bool) -> std::io::Result<Self> {
        if !state_root.is_absolute() {
            return Err(error("state directory must be absolute"));
        }
        let meta = fs::symlink_metadata(state_root)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(error("state directory must be a real private directory"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if meta.mode() & 0o077 != 0 {
                return Err(error("state directory must deny group and other access"));
            }
        }
        let root = state_root.canonicalize()?.join("operations");
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        if create {
            match builder.create(&root) {
                Ok(()) => sync_directory(state_root)?,
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e),
            }
        }
        let meta = fs::symlink_metadata(&root)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(error("operations directory must not be substituted"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if meta.mode() & 0o077 != 0 {
                return Err(error("operations directory must be private"));
            }
        }
        Ok(Self {
            root,
            limits: STORE_LIMITS,
        })
    }
    fn open_lock_file(&self) -> std::io::Result<File> {
        let path = self.root.join("store.lock");
        match private_file(&path, true) {
            Ok(file) => Ok(file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => private_file(&path, false),
            Err(e) => Err(e),
        }
    }
    pub fn lock(&self) -> std::io::Result<LockedStore> {
        let file = self.open_lock_file()?;
        file.try_lock().map_err(lock_error)?;
        let file = FileLock::acquired(file);
        Ok(LockedStore {
            root: self.root.clone(),
            limits: self.limits,
            _lock: file,
        })
    }
    /// Retirement may wait briefly for another owner's ordinary receipt write.
    /// Retry only native contention, on this one validated descriptor, for at
    /// most one second. No receipt operation or filesystem failure is retried.
    pub(crate) fn lock_for_shutdown(&self) -> std::io::Result<LockedStore> {
        let file = self.open_lock_file()?;
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            match file.try_lock() {
                Ok(()) => break,
                Err(TryLockError::WouldBlock) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return Err(lock_error(TryLockError::WouldBlock));
                    }
                    std::thread::sleep(remaining.min(Duration::from_millis(5)));
                    if Instant::now() >= deadline {
                        return Err(lock_error(TryLockError::WouldBlock));
                    }
                }
                Err(TryLockError::Error(error)) => return Err(error),
            }
        }
        let file = FileLock::acquired(file);
        Ok(LockedStore {
            root: self.root.clone(),
            limits: self.limits,
            _lock: file,
        })
    }
    /// Read under the existing shared lock without creating or rewriting state.
    /// A missing lock or receipt is missing evidence, never a new operation.
    pub fn read_existing(&self, id: &str) -> std::io::Result<Vec<u8>> {
        if !valid_id(id) {
            return Err(error("invalid operation ID"));
        }
        let lock = open_private_file(&self.root.join("store.lock"), false, false)?;
        if lock.metadata()?.len() != 0 {
            return Err(error("operation-store lock must be empty"));
        }
        lock.try_lock_shared().map_err(lock_error)?;
        let _lock = FileLock::acquired(lock);
        let file = open_private_file(&self.root.join(format!("{id}.json")), false, false)?;
        Envelope::decode(&read_bounded(file, self.limits.receipt_bytes)?)?.readable(now_seconds()?)
    }
}
fn read_bounded(file: File, limit: usize) -> std::io::Result<Vec<u8>> {
    if file.metadata()?.len() > limit as u64 {
        return Err(error("operation receipt exceeds its byte limit"));
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(error("operation receipt grew beyond its byte limit"));
    }
    Ok(bytes)
}
impl LockedStore {
    /// Inventory transfer payloads for this exact scope, including receipts past
    /// their read-retention age. The transfer owner must decode the full typed
    /// receipt and prove cleanup authority; selection never authorizes replay.
    pub(crate) fn memory_transfer_receipts(
        &self,
        scope: Option<&crate::memory::Scope>,
    ) -> std::io::Result<Vec<(String, Vec<u8>)>> {
        let scope = scope
            .map(serde_json::to_value)
            .transpose()
            .map_err(std::io::Error::other)?;
        let mut selected = Vec::new();
        let mut selected_bytes = 0u64;
        for (index, entry) in fs::read_dir(&self.root)?.enumerate() {
            if index >= self.limits.records + 64 {
                return Err(error("receipt inventory exceeds its inspection bound"));
            }
            let entry = entry?;
            let name = entry.file_name();
            let Some(id) = name
                .to_str()
                .and_then(|name| name.strip_suffix(".json"))
                .filter(|id| valid_id(id))
            else {
                continue;
            };
            // Do not call readable(): expired completed receipts can still bind
            // a staged export whose cleanup has not been proven.
            let bytes = self.read_envelope(id)?.payload.into_bytes();
            let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
                continue;
            };
            if value["kind"] == "memory_transfer"
                && scope.as_ref().is_none_or(|scope| value["scope"] == *scope)
            {
                selected_bytes = selected_bytes
                    .checked_add(bytes.len() as u64)
                    .ok_or_else(|| error("transfer receipt inventory byte overflow"))?;
                if selected_bytes > self.limits.store_bytes {
                    return Err(error("transfer receipt inventory exceeds its byte bound"));
                }
                selected.push((id.to_owned(), bytes));
            }
        }
        Ok(selected)
    }

    /// Select only typed memory-save receipts bound to this exact scope/target.
    /// Caller holds the exclusive scope guard, preventing new matching intents
    /// from being issued between this inventory and forgetting cleanup.
    pub(crate) fn memory_target_receipts(
        &self,
        scope: &crate::memory::Scope,
        record_id: &str,
    ) -> std::io::Result<Vec<String>> {
        let scope = serde_json::to_value(scope).map_err(std::io::Error::other)?;
        let mut ids = Vec::new();
        for (index, entry) in fs::read_dir(&self.root)?.enumerate() {
            if index > MAX_RECORDS + 64 {
                return Err(error("receipt inventory exceeds its inspection bound"));
            }
            let entry = entry?;
            let name = entry.file_name();
            let Some(id) = name
                .to_str()
                .and_then(|s| s.strip_suffix(".json"))
                .filter(|id| valid_id(id))
            else {
                continue;
            };
            let bytes = self.read_envelope(id)?.payload.into_bytes();
            let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
                continue;
            };
            if value["schema"] == 1
                && value["kind"] == "memory_mutation"
                && value["scope"] == scope
                && value["target"]["record_id"] == record_id
                && matches!(
                    value["status"].as_str(),
                    Some("prepared" | "unknown" | "cancelled" | "expired_not_started")
                )
            {
                ids.push(id.to_owned());
            }
        }
        Ok(ids)
    }
    /// Forgetting only: the caller has already durably recorded that this exact
    /// issued ID is consumed. Remove its receipt and recognized replacement
    /// stages, preserving every unrelated entry. Missing evidence is never fresh.
    pub(crate) fn remove_consumed(&self, id: &str) -> std::io::Result<()> {
        let final_path = self.path(id)?;
        let prefix = format!("{id}-");
        let mut selected = Vec::new();
        for (index, entry) in fs::read_dir(&self.root)?.enumerate() {
            if index > MAX_RECORDS + 64 {
                return Err(error("receipt inventory exceeds its inspection bound"));
            }
            let entry = entry?;
            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| error("invalid receipt filename"))?;
            let is_stage = name
                .strip_prefix(&prefix)
                .and_then(|s| s.strip_suffix(".tmp"))
                .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()));
            if entry.path() == final_path || is_stage {
                let file = private_file(&entry.path(), false)?;
                if file.metadata()?.len() > self.limits.receipt_bytes as u64 {
                    return Err(error(
                        "consumed receipt exceeds its byte limit; inspect before cleanup",
                    ));
                }
                selected.push(entry.path());
            }
        }
        for path in selected {
            fs::remove_file(path)?;
        }
        sync_directory(&self.root)
    }
    fn path(&self, id: &str) -> std::io::Result<PathBuf> {
        if !valid_id(id) {
            return Err(error("invalid operation ID"));
        }
        Ok(self.root.join(format!("{id}.json")))
    }
    fn inventory(&self) -> std::io::Result<Vec<InventoryEntry>> {
        let mut records = Vec::new();
        for (index, entry) in fs::read_dir(&self.root)?.enumerate() {
            if index > MAX_RECORDS + 64 {
                return Err(error("receipt inventory exceeds its inspection bound"));
            }
            let entry = entry?;
            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or_else(|| error("unexpected operation-store entry"))?;
            let file = private_file(&entry.path(), false)?;
            let size = file.metadata()?.len();
            if name == "store.lock" {
                if size != 0 {
                    return Err(error("operation-store lock must be empty"));
                }
                continue;
            }
            let id = name
                .strip_suffix(".json")
                .filter(|id| valid_id(id))
                .ok_or_else(|| {
                    error("unresolved operation-store entry; inspect before mutation")
                })?;
            let envelope = Envelope::decode(&read_bounded(file, self.limits.receipt_bytes)?)?;
            records.push(InventoryEntry {
                id: id.to_owned(),
                bytes: size,
                completed_at: match envelope.retention {
                    Retention::Protected => None,
                    Retention::Retirable { completed_at } => Some(completed_at),
                },
            });
        }
        Ok(records)
    }
    fn fits(&self, total: u64, count: usize, added: usize, new_record: bool) -> bool {
        let Some(count) = count.checked_add(usize::from(new_record)) else {
            return false;
        };
        let reserved = (count as u64)
            .checked_add(1)
            .and_then(|slots| slots.checked_mul(self.limits.receipt_bytes as u64));
        count <= self.limits.records
            && reserved.is_some_and(|bytes| bytes <= self.limits.store_bytes)
            && total
                .checked_add(added as u64)
                .is_some_and(|bytes| bytes <= self.limits.store_bytes)
    }
    fn reserve_bytes(&self, added: usize, new_record: bool) -> std::io::Result<()> {
        let inventory = self.inventory()?; // Validate everything before any eviction.
        let mut total = inventory
            .iter()
            .try_fold(0u64, |n, r| n.checked_add(r.bytes))
            .ok_or_else(|| error("operation-store size overflow"))?;
        let mut count = inventory.len();
        let mut removed = Vec::new();
        if new_record {
            let now = now_seconds()?;
            let mut eligible: Vec<_> = inventory
                .iter()
                .filter(|r| r.completed_at.is_some())
                .collect();
            eligible.sort_by_key(|r| (r.completed_at.unwrap(), &r.id));
            for record in eligible {
                let expired = now
                    .checked_sub(record.completed_at.unwrap())
                    .is_some_and(|age| age >= RETENTION_SECONDS);
                if !expired && self.fits(total, count, added, true) {
                    break;
                }
                removed.push(&record.id);
                total -= record.bytes;
                count -= 1;
            }
        }
        if !self.fits(total, count, added, new_record) {
            return Err(error(
                "operation receipt capacity exhausted; inspection remains available",
            ));
        }
        for id in &removed {
            fs::remove_file(self.path(id)?)?;
        }
        if !removed.is_empty() {
            sync_directory(&self.root)?;
        }
        Ok(())
    }
    fn encode(&self, envelope: &Envelope) -> std::io::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(envelope).map_err(std::io::Error::other)?;
        if bytes.len() > self.limits.receipt_bytes {
            return Err(error("operation envelope exceeds its byte limit"));
        }
        // Issued protected records reserve enough space for future completion
        // metadata even at the maximum timestamp width.
        if matches!(envelope.retention, Retention::Protected) {
            let mut completed = envelope.clone();
            completed.retention = Retention::Retirable {
                completed_at: u64::MAX,
            };
            if serde_json::to_vec(&completed)
                .map_err(std::io::Error::other)?
                .len()
                > self.limits.receipt_bytes
            {
                return Err(error(
                    "operation payload leaves no room for completion metadata",
                ));
            }
        }
        Ok(bytes)
    }
    fn read_envelope(&self, id: &str) -> std::io::Result<Envelope> {
        let file = private_file(&self.path(id)?, false)?;
        Envelope::decode(&read_bounded(file, self.limits.receipt_bytes)?)
    }
    /// Issue one new identity. No API accepts a caller-supplied ID for creation.
    pub fn issue(&self, receipt: &[u8]) -> std::io::Result<String> {
        let envelope = Envelope {
            schema: 2,
            retention: Retention::Protected,
            issued_payload_sha256: payload_digest(receipt),
            payload: std::str::from_utf8(receipt)
                .map_err(|_| error("operation payload must be UTF-8"))?
                .into(),
        };
        let encoded = self.encode(&envelope)?;
        self.reserve_bytes(encoded.len(), true)?;
        for _ in 0..64 {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| error("system clock predates epoch"))?
                .as_nanos();
            let id = format!(
                "op-{nanos:x}-{:x}-{:x}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            );
            let path = self.path(&id)?;
            match private_file(&path, true) {
                Ok(mut file) => {
                    file.write_all(&encoded)?;
                    file.sync_all()?;
                    sync_directory(&self.root)?;
                    return Ok(id);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(error("could not allocate a new operation identity"))
    }
    /// Missing, evicted, expired or corrupt receipts cannot authorize start.
    pub fn read(&self, id: &str) -> std::io::Result<Vec<u8>> {
        self.read_envelope(id)?.readable(now_seconds()?)
    }
    /// Bind a typed owner's reconstructed preparation to the originally issued
    /// bytes. Updates/finalization never replace this fingerprint. This proves
    /// issuance consistency, not publisher authenticity or effect completion.
    pub(crate) fn matches_issued_payload(&self, id: &str, receipt: &[u8]) -> std::io::Result<bool> {
        let envelope = self.read_envelope(id)?;
        if envelope.expired(now_seconds()?) {
            return Err(error(
                "operation receipt expired; do not reconstruct missing intent",
            ));
        }
        Ok(envelope.issued_payload_sha256 == payload_digest(receipt))
    }
    /// Update only protected work. Identical completed payloads are idempotent;
    /// a completed record cannot be changed or reopened by an ordinary update.
    pub fn update(&self, id: &str, receipt: &[u8]) -> std::io::Result<()> {
        self.replace(id, receipt, false)
    }
    /// The typed owner has confirmed its effect AND cleanup. Completion metadata
    /// is store-owned; repeated identical finalization never renews retention.
    pub fn finalize(&self, id: &str, receipt: &[u8]) -> std::io::Result<()> {
        self.replace(id, receipt, true)
    }
    fn replace(&self, id: &str, receipt: &[u8], finalize: bool) -> std::io::Result<()> {
        let mut envelope = self.read_envelope(id)?;
        let now = now_seconds()?;
        if envelope.expired(now) {
            return Err(error(
                "operation receipt expired; missing evidence never permits replay",
            ));
        }
        let payload =
            std::str::from_utf8(receipt).map_err(|_| error("operation payload must be UTF-8"))?;
        if matches!(envelope.retention, Retention::Retirable { .. }) {
            return if envelope.payload.as_bytes() == receipt {
                Ok(())
            } else {
                Err(error(
                    "completed operation receipt cannot be changed or reopened",
                ))
            };
        }
        envelope.payload = payload.into();
        if finalize {
            envelope.retention = Retention::Retirable { completed_at: now };
        }
        let encoded = self.encode(&envelope)?;
        self.reserve_bytes(encoded.len(), false)?;
        let final_path = self.path(id)?;
        let temp = self.root.join(format!(
            "{id}-{}.tmp",
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = private_file(&temp, true)?;
        file.write_all(&encoded)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, final_path)?;
        sync_directory(&self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn issued_payload_fingerprint_survives_updates_finalization_and_rejects_old_schema() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let locked = store.lock().unwrap();
        let id = locked.issue(b"exact prepared identity").unwrap();
        assert!(
            locked
                .matches_issued_payload(&id, b"exact prepared identity")
                .unwrap()
        );
        locked.update(&id, b"changed current payload").unwrap();
        assert!(
            locked
                .matches_issued_payload(&id, b"exact prepared identity")
                .unwrap()
        );
        assert!(
            !locked
                .matches_issued_payload(&id, b"changed current payload")
                .unwrap()
        );
        locked.finalize(&id, b"terminal").unwrap();
        let before = fs::read(locked.path(&id).unwrap()).unwrap();
        locked.finalize(&id, b"terminal").unwrap();
        assert_eq!(fs::read(locked.path(&id).unwrap()).unwrap(), before);
        assert!(
            locked
                .matches_issued_payload(&id, b"exact prepared identity")
                .unwrap()
        );
        let mut old: serde_json::Value = serde_json::from_slice(&before).unwrap();
        old["schema"] = serde_json::json!(1);
        assert!(Envelope::decode(&serde_json::to_vec(&old).unwrap()).is_err());
        old["schema"] = serde_json::json!(2);
        old.as_object_mut().unwrap().remove("issued_payload_sha256");
        assert!(Envelope::decode(&serde_json::to_vec(&old).unwrap()).is_err());
        drop(locked);
        fs::remove_dir_all(root).unwrap();
    }
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "mirket-store-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&root).unwrap();
        root.canonicalize().unwrap()
    }
    fn limited_store(root: &Path, store_bytes: u64, records: usize) -> OperationStore {
        let mut store = OperationStore::open(root).unwrap();
        store.limits = StoreLimits {
            receipt_bytes: 256,
            store_bytes,
            records,
        };
        store
    }
    #[test]
    fn inspection_never_creates_entries_and_obeys_writer_lock() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        assert!(store.read_existing("op-missing").is_err());
        assert_eq!(fs::read_dir(root.join("operations")).unwrap().count(), 0);
        let writer = store.lock().unwrap();
        let id = writer.issue(b"issued receipt").unwrap();
        assert_eq!(
            store.read_existing(&id).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        drop(writer);
        let path = root.join("operations").join(format!("{id}.json"));
        let before = fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(store.read_existing(&id).unwrap(), b"issued receipt");
        assert_eq!(fs::metadata(path).unwrap().modified().unwrap(), before);
        assert!(store.read_existing("op-missing").is_err());
        assert_eq!(fs::read_dir(root.join("operations")).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn forgetting_scrubs_only_exact_consumed_receipt_and_stages() {
        let root = fixture();
        assert!(OperationStore::open_existing(&root).is_err());
        assert!(!root.join("operations").exists());
        let store = OperationStore::open(&root).unwrap();
        let locked = store.lock().unwrap();
        let id = locked.issue(b"content-derived fingerprint").unwrap();
        let other = locked.issue(b"unrelated evidence").unwrap();
        let stage = root.join("operations").join(format!("{id}-123.tmp"));
        private_file(&stage, true)
            .unwrap()
            .write_all(b"old fingerprint")
            .unwrap();
        let similar = root.join("operations").join(format!("{id}-suffix-123.tmp"));
        private_file(&similar, true)
            .unwrap()
            .write_all(b"unrelated stage")
            .unwrap();
        locked.remove_consumed(&id).unwrap();
        assert!(!stage.exists());
        assert!(locked.read(&id).is_err());
        assert_eq!(locked.read(&other).unwrap(), b"unrelated evidence");
        assert_eq!(fs::read(&similar).unwrap(), b"unrelated stage");
        locked.remove_consumed(&id).unwrap();
        assert!(locked.update(&id, b"not fresh").is_err());
        drop(locked);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn byte_reservations_leave_room_for_full_confirmation_and_its_stage() {
        let root = fixture();
        // Three maximum-sized records plus one replacement stage, not four
        // tiny initial records whose later confirmation could exhaust storage.
        let store = limited_store(&root, 4 * 256, 100);
        let held = store.lock().unwrap();
        let ids: Vec<_> = (0..3).map(|_| held.issue(b"prepared").unwrap()).collect();
        assert!(held.issue(b"prepared").is_err());
        let size = (0..256)
            .rev()
            .find(|n| {
                held.encode(&Envelope {
                    schema: 2,
                    retention: Retention::Protected,
                    issued_payload_sha256: payload_digest(b"prepared"),
                    payload: "c".repeat(*n),
                })
                .is_ok()
            })
            .unwrap();
        for id in &ids {
            held.update(id, &vec![b'c'; size]).unwrap();
            assert_eq!(held.read(id).unwrap(), vec![b'c'; size]);
        }
        // All payloads are at their future-completion bound; replacement fits.
        held.update(&ids[0], &vec![b'r'; size]).unwrap();
        assert!(held.issue(b"prepared").is_err());
        assert_eq!(held.read(&ids[0]).unwrap(), vec![b'r'; size]);
        held.finalize(&ids[0], &vec![b'r'; size]).unwrap();
        drop(held);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn record_limit_refuses_new_issue_without_blocking_existing_updates() {
        let root = fixture();
        let store = limited_store(&root, 10 * 256, 2);
        let held = store.lock().unwrap();
        let first = held.issue(b"prepared").unwrap();
        let second = held.issue(b"prepared").unwrap();
        assert!(held.issue(b"one-too-many").is_err());
        held.update(&first, b"confirmed").unwrap();
        held.update(&second, b"unknown").unwrap();
        assert_eq!(held.read(&first).unwrap(), b"confirmed");
        assert_eq!(held.read(&second).unwrap(), b"unknown");
        drop(held);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unresolved_stages_and_unknown_entries_block_mutation_but_not_inspection() {
        for name in ["op-abandoned-1.tmp", "unexpected.txt"] {
            let root = fixture();
            let store = limited_store(&root, 10 * 256, 5);
            let held = store.lock().unwrap();
            let id = held.issue(b"unknown").unwrap();
            let path = held.root.join(name);
            private_file(&path, true)
                .unwrap()
                .write_all(b"retained recovery bytes")
                .unwrap();
            assert!(held.issue(b"prepared").is_err());
            assert!(held.update(&id, b"confirmed").is_err());
            assert_eq!(held.read(&id).unwrap(), b"unknown");
            assert_eq!(fs::read(path).unwrap(), b"retained recovery bytes");
            drop(held);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn oversized_existing_record_cannot_hide_outside_the_reservation() {
        let root = fixture();
        let store = limited_store(&root, 10 * 256, 5);
        let held = store.lock().unwrap();
        let id = held.issue(b"prepared").unwrap();
        fs::write(held.path(&id).unwrap(), [b'x'; 257]).unwrap();
        assert!(held.issue(b"prepared").is_err());
        assert!(held.update(&id, b"confirmed").is_err());
        assert!(held.read(&id).is_err());
        drop(held);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn identities_are_issued_and_missing_never_creates() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let lock = store.lock().unwrap();
        let id = lock.issue(b"prepared").unwrap();
        assert_eq!(lock.read(&id).unwrap(), b"prepared");
        lock.update(&id, b"unknown").unwrap();
        assert_eq!(lock.read(&id).unwrap(), b"unknown");
        assert!(lock.update("op-missing", b"new").is_err());
        assert!(!root.join("operations/op-missing.json").exists());
        assert!(lock.read("../escape").is_err());
        drop(lock);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn two_owners_cannot_mutate_receipts_together() {
        let root = fixture();
        let a = OperationStore::open(&root).unwrap();
        let b = OperationStore::open(&root).unwrap();
        let held = a.lock().unwrap();
        assert_eq!(
            b.lock().err().unwrap().kind(),
            std::io::ErrorKind::WouldBlock
        );
        drop(held);
        assert!(b.lock().is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn shutdown_lock_waits_for_transient_owner_without_replaying_receipts() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let held = store.lock().unwrap();
        let id = held.issue(b"unchanged evidence").unwrap();
        assert_eq!(
            store.lock().err().unwrap().kind(),
            std::io::ErrorKind::WouldBlock
        );
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            drop(held);
        });
        let acquired = store.lock_for_shutdown().unwrap();
        release.join().unwrap();
        assert_eq!(acquired.read(&id).unwrap(), b"unchanged evidence");
        assert_eq!(fs::read_dir(&store.root).unwrap().count(), 2);
        drop(acquired);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn shutdown_lock_refuses_after_bounded_contention() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let held = store.lock().unwrap();
        let start = Instant::now();
        let failure = store
            .lock_for_shutdown()
            .err()
            .expect("owner still holds lock");
        let elapsed = start.elapsed();
        assert_eq!(failure.kind(), std::io::ErrorKind::WouldBlock);
        assert!(elapsed >= Duration::from_millis(950));
        // The retry budget is one second. Allow scheduler delay in this test;
        // an unbounded/blocking acquisition cannot complete while held lives.
        assert!(elapsed < Duration::from_secs(2));
        assert_eq!(fs::read_dir(&store.root).unwrap().count(), 1);
        drop(held);
        drop(store.lock_for_shutdown().unwrap());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn shutdown_lock_preserves_noncontention_errors_without_retry() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        fs::remove_dir(&store.root).unwrap();
        let start = Instant::now();
        let failure = store.lock_for_shutdown().err().expect("missing namespace");
        assert_eq!(failure.kind(), std::io::ErrorKind::NotFound);
        assert!(start.elapsed() < Duration::from_millis(800));
        assert!(!store.root.exists());
        fs::remove_dir_all(root).unwrap();

        // Real native failures retain their kind, raw code and message instead
        // of becoming contention (including an OS error with WouldBlock kind).
        let original = std::io::Error::from_raw_os_error(13);
        let message = original.to_string();
        let kind = original.kind();
        let mapped = lock_error(TryLockError::Error(original));
        assert_eq!(mapped.raw_os_error(), Some(13));
        assert_eq!(mapped.kind(), kind);
        assert_eq!(mapped.to_string(), message);
        let mapped = lock_error(TryLockError::Error(std::io::Error::new(
            std::io::ErrorKind::WouldBlock,
            "native error, not the contention variant",
        )));
        assert_eq!(
            mapped.to_string(),
            "native error, not the contention variant"
        );
    }
    #[test]
    fn byte_limit_and_eviction_do_not_recreate_an_intent() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let held = store.lock().unwrap();
        assert!(held.issue(&vec![0; MAX_RECEIPT_BYTES + 1]).is_err());
        let id = held.issue(b"prepared").unwrap();
        fs::remove_file(held.path(&id).unwrap()).unwrap();
        assert!(held.read(&id).is_err());
        assert!(held.update(&id, b"running").is_err());
        drop(held);
        fs::remove_dir_all(root).unwrap();
    }
    fn completed_at(locked: &LockedStore, id: &str, at: u64) {
        let mut envelope = locked.read_envelope(id).unwrap();
        envelope.retention = Retention::Retirable { completed_at: at };
        fs::write(locked.path(id).unwrap(), locked.encode(&envelope).unwrap()).unwrap();
    }
    #[test]
    fn finalization_is_explicit_immutable_and_does_not_refresh_time() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let locked = store.lock().unwrap();
        let payload = b"{\"status\":\"committed\",\"cleanup\":\"pending\"}";
        let id = locked.issue(payload).unwrap();
        assert!(matches!(
            locked.read_envelope(&id).unwrap().retention,
            Retention::Protected
        ));
        locked.update(&id, b"cleanup confirmed").unwrap();
        locked.finalize(&id, b"cleanup confirmed").unwrap();
        completed_at(&locked, &id, now_seconds().unwrap() - 10);
        let before = fs::read(locked.path(&id).unwrap()).unwrap();
        locked.finalize(&id, b"cleanup confirmed").unwrap();
        locked.update(&id, b"cleanup confirmed").unwrap();
        assert_eq!(fs::read(locked.path(&id).unwrap()).unwrap(), before);
        assert!(locked.update(&id, b"running again").is_err());
        assert!(locked.finalize(&id, b"different effect").is_err());
        assert_eq!(locked.read(&id).unwrap(), b"cleanup confirmed");
        drop(locked);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn capacity_evicts_oldest_completed_before_seven_days_but_preserves_unknown() {
        let root = fixture();
        let store = limited_store(&root, 4 * 256, 3);
        let locked = store.lock().unwrap();
        let unknown = locked.issue(b"unknown effect with scratch").unwrap();
        let older = locked.issue(b"finished first").unwrap();
        let newer = locked.issue(b"finished later").unwrap();
        completed_at(&locked, &older, now_seconds().unwrap() - 20);
        completed_at(&locked, &newer, now_seconds().unwrap() - 10);
        let fresh = locked.issue(b"new intent").unwrap();
        assert!(locked.read(&older).is_err());
        assert!(locked.update(&older, b"replay").is_err());
        assert!(locked.finalize(&older, b"replay").is_err());
        assert_eq!(
            locked.read(&unknown).unwrap(),
            b"unknown effect with scratch"
        );
        assert_eq!(locked.read(&newer).unwrap(), b"finished later");
        assert_eq!(locked.read(&fresh).unwrap(), b"new intent");
        drop(locked);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn expired_completion_is_readonly_unavailable_then_pruned_on_issue() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let locked = store.lock().unwrap();
        let id = locked.issue(b"finished").unwrap();
        completed_at(&locked, &id, now_seconds().unwrap() - RETENTION_SECONDS);
        let path = locked.path(&id).unwrap();
        let before = fs::read(&path).unwrap();
        assert!(
            locked
                .read(&id)
                .unwrap_err()
                .to_string()
                .contains("expired")
        );
        assert!(locked.update(&id, b"finished").is_err());
        assert!(locked.finalize(&id, b"finished").is_err());
        drop(locked);
        assert!(
            store
                .read_existing(&id)
                .unwrap_err()
                .to_string()
                .contains("expired")
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        let locked = store.lock().unwrap();
        let fresh = locked.issue(b"new").unwrap();
        assert!(!path.exists());
        assert_eq!(locked.read(&fresh).unwrap(), b"new");
        drop(locked);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn full_inventory_validation_precedes_any_retention_deletion() {
        for name in ["op-crashed-1.tmp", "foreign", "op-corrupt.json"] {
            let root = fixture();
            let store = OperationStore::open(&root).unwrap();
            let locked = store.lock().unwrap();
            let old = locked.issue(b"finished").unwrap();
            completed_at(&locked, &old, now_seconds().unwrap() - RETENTION_SECONDS);
            let old_path = locked.path(&old).unwrap();
            let before = fs::read(&old_path).unwrap();
            let path = locked.root.join(name);
            private_file(&path, true)
                .unwrap()
                .write_all(b"unknown authority")
                .unwrap();
            assert!(locked.issue(b"new").is_err());
            assert_eq!(fs::read(old_path).unwrap(), before);
            assert_eq!(fs::read(path).unwrap(), b"unknown authority");
            drop(locked);
            fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn escaped_payload_budget_includes_envelope_and_future_completion() {
        let root = fixture();
        let store = limited_store(&root, 4 * 256, 3);
        let locked = store.lock().unwrap();
        let original = "quotes \" and newline\n and Turkish İı";
        let id = locked.issue(original.as_bytes()).unwrap();
        assert_eq!(locked.read(&id).unwrap(), original.as_bytes());
        assert!(locked.issue(&[0; 40]).is_err()); // JSON escaping, not raw40bytes.
        assert!(locked.issue(&[255]).is_err()); // Not UTF-8.
        let size = (0..256)
            .rev()
            .find(|n| {
                locked
                    .encode(&Envelope {
                        schema: 2,
                        retention: Retention::Protected,
                        issued_payload_sha256: payload_digest(b"prepared"),
                        payload: "a".repeat(*n),
                    })
                    .is_ok()
            })
            .unwrap();
        let full = locked.issue("a".repeat(size).as_bytes()).unwrap();
        assert!(locked.issue("a".repeat(size + 1).as_bytes()).is_err());
        locked.finalize(&full, "a".repeat(size).as_bytes()).unwrap();
        assert!(fs::metadata(locked.path(&full).unwrap()).unwrap().len() <= 256);
        drop(locked);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn future_completion_time_is_protected_from_age_expiry() {
        let root = fixture();
        let store = OperationStore::open(&root).unwrap();
        let locked = store.lock().unwrap();
        let id = locked.issue(b"clock moved back").unwrap();
        completed_at(&locked, &id, now_seconds().unwrap() + RETENTION_SECONDS);
        let fresh = locked.issue(b"new").unwrap();
        assert_eq!(locked.read(&id).unwrap(), b"clock moved back");
        assert_eq!(locked.read(&fresh).unwrap(), b"new");
        drop(locked);
        fs::remove_dir_all(root).unwrap();
    }
}
