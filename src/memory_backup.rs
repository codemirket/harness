//! Finite managed SQLite snapshots, separate from the canonical memory store.
//! The caller selects one private backup root; its binding lives with forgetting
//! authority and cannot be silently changed. Byte admission includes catalog,
//! rollback-journal headroom and reserved in-progress snapshots. Native private
//! filesystem assumptions apply; this is not a hostile-path sandbox.

use crate::file_lock::FileLock;
use crate::{
    maintenance::MaintenanceGuard,
    memory::{MemoryError, MemoryStore, Scope},
    memory_lifecycle::{self, ExclusiveScope},
};
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, MemoryError>;
const APP_ID: i64 = 0x4d424b50;
const META_HEADROOM: u64 = 128 * 1024;
const MAX_CATALOG: u64 = 64 * 1024 * 1024;
const MAX_SNAPSHOTS: i64 = 4096;
const MAX_RESTORE_RECEIPTS: i64 = 4096;
const RESTORE_RECEIPT_SECONDS: u64 = 7 * 86400;
pub const DEFAULT_BACKUP_BYTES: u64 = 5_368_709_120;
#[derive(Clone, Copy, Debug)]
pub struct BackupPolicy {
    pub max_bytes: u64,
}
impl Default for BackupPolicy {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_BACKUP_BYTES,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BackupKind {
    Daily,
    Weekly,
    PreMigration,
    PreRestore,
}
impl BackupKind {
    fn text(self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::PreMigration => "pre_migration",
            Self::PreRestore => "pre_restore",
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct BackupInfo {
    pub snapshot_id: String,
    pub scope: Scope,
    pub byte_len: u64,
    pub created_at: u64,
    pub eligible: bool,
    pub protected: bool,
    pub phase: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct RestoreStatus {
    pub phase: String,
    pub operation_id: String,
    pub scope: Scope,
    pub snapshot_id: String,
    pub preserved_snapshot_id: String,
    pub complete: bool,
}

struct Catalog {
    // Close SQLite before releasing the catalog exclusion.
    connection: Connection,
    root: PathBuf,
    _lock: FileLock,
}
#[derive(Clone)]
struct Snapshot {
    id: String,
    key: String,
    scope: String,
    epoch: i64,
    phase: String,
    bytes: i64,
    reserved: i64,
    hash: String,
    stamp: String,
    created: i64,
    protected: bool,
    input_hash: String,
    input_bytes: i64,
}
impl Snapshot {
    fn path(&self, root: &Path) -> PathBuf {
        root.join(format!("{}.sqlite3", self.id))
    }
    fn info(&self, epoch: u64) -> Result<BackupInfo> {
        Ok(BackupInfo {
            snapshot_id: self.id.clone(),
            scope: serde_json::from_str(&self.scope)?,
            byte_len: u64::try_from(self.bytes).map_err(|_| MemoryError::UnsupportedSchema)?,
            created_at: u64::try_from(self.created).map_err(|_| MemoryError::UnsupportedSchema)?,
            eligible: self.phase == "eligible"
                && self.epoch
                    == i64::try_from(epoch).map_err(|_| MemoryError::UnsupportedSchema)?,
            protected: self.protected,
            phase: self.phase.clone(),
        })
    }
}
fn row_snapshot(row: &rusqlite::Row<'_>) -> rusqlite::Result<Snapshot> {
    Ok(Snapshot {
        id: row.get(0)?,
        key: row.get(1)?,
        scope: row.get(2)?,
        epoch: row.get(3)?,
        phase: row.get(4)?,
        bytes: row.get(5)?,
        reserved: row.get(6)?,
        hash: row.get(7)?,
        stamp: row.get(8)?,
        created: row.get(9)?,
        protected: row.get(10)?,
        input_hash: row.get(11)?,
        input_bytes: row.get(12)?,
    })
}
const COLUMNS: &str = "id,scope_key,scope_json,epoch,phase,byte_len,reserved_bytes,sha256,file_stamp,created_at,protected,input_sha256,input_bytes";
fn validate_snapshot(snapshot: Snapshot) -> Result<Snapshot> {
    let id = snapshot
        .id
        .strip_prefix("snapshot-")
        .ok_or(MemoryError::UnsupportedSchema)?;
    if id.len() != 64
        || !id.bytes().all(|b| b.is_ascii_hexdigit())
        || snapshot.key.len() != 64
        || !snapshot.key.bytes().all(|b| b.is_ascii_hexdigit())
        || !matches!(snapshot.phase.as_str(), "copying" | "eligible" | "deleting")
        || snapshot.epoch < 0
        || snapshot.bytes < 0
        || snapshot.reserved < 0
        || snapshot.created < 0
        || snapshot.input_bytes < 0
        || (snapshot.input_hash.is_empty() != (snapshot.input_bytes == 0))
        || (!snapshot.input_hash.is_empty()
            && (snapshot.input_hash.len() != 64
                || !snapshot
                    .input_hash
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || snapshot.input_bytes as u64 > crate::memory::transfer::MAX_STREAM_BYTES))
    {
        return Err(MemoryError::UnsupportedSchema);
    }
    let scope: Scope = serde_json::from_str(&snapshot.scope)?;
    if memory_lifecycle::scope_key(&scope)? != snapshot.key {
        return Err(MemoryError::ScopeMismatch);
    }
    Ok(snapshot)
}
fn snapshots(connection: &Connection) -> Result<Vec<Snapshot>> {
    let mut q = connection.prepare(&format!(
        "SELECT {COLUMNS} FROM snapshots ORDER BY created_at,rowid LIMIT {}",
        MAX_SNAPSHOTS + 1
    ))?;
    let saved: Vec<Snapshot> = q
        .query_map([], row_snapshot)?
        .map(|row| validate_snapshot(row?))
        .collect::<Result<_>>()?;
    if saved.len() > MAX_SNAPSHOTS as usize {
        return Err(MemoryError::InvalidInput(
            "managed snapshot inventory exceeds its bound",
        ));
    }
    Ok(saved)
}
fn snapshot(connection: &Connection, id: &str) -> Result<Snapshot> {
    validate_snapshot(
        connection
            .query_row(
                &format!("SELECT {COLUMNS} FROM snapshots WHERE id=?1"),
                [id],
                row_snapshot,
            )
            .optional()?
            .ok_or(MemoryError::NotFound)?,
    )
}
fn now() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MemoryError::Clock)?
        .as_secs())
}
fn snapshot_id(operation: &str) -> Result<String> {
    if !operation.starts_with("op-")
        || operation.len() > 100
        || !operation
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(MemoryError::InvalidInput(
            "backup requires a server-issued operation ID",
        ));
    }
    Ok(format!(
        "snapshot-{}",
        Sha256::digest(operation.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}

pub fn backup(
    state_root: &Path,
    backup_root: &Path,
    scope: Scope,
    operation_id: &str,
    kind: BackupKind,
    policy: BackupPolicy,
) -> Result<BackupInfo> {
    let access = ExclusiveScope::acquire(state_root, &scope)?;
    check_restore_pending(state_root, &scope)?;
    let mut live = MemoryStore::open_for_lifecycle(state_root, scope.clone(), &access)?;
    memory_lifecycle::check_current_exclusions(state_root, &scope, live.lifecycle_connection())?;
    backup_locked(
        state_root,
        backup_root,
        &scope,
        operation_id,
        Capture { kind, at: now()? },
        policy,
        live.lifecycle_connection(),
    )
}

struct Capture {
    kind: BackupKind,
    at: u64,
}
/// Expected bytes are bound in the catalog before constructing a recovery point.
/// The caller has already validated the selected path and retains this descriptor;
/// no archive-supplied path or SQLite schema is ever opened here.
pub(crate) struct ImportInput<'a> {
    pub operation_id: &'a str,
    /// Lowercase SHA-256 hex, without a prefix.
    pub sha256: &'a str,
    pub byte_len: u64,
}
fn validate_import(input: &ImportInput<'_>) -> Result<()> {
    snapshot_id(input.operation_id)?;
    if input.byte_len == 0
        || input.byte_len > crate::memory::transfer::MAX_STREAM_BYTES
        || input.sha256.len() != 64
        || !input
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(MemoryError::InvalidInput(
            "invalid bounded import byte identity",
        ));
    }
    Ok(())
}
fn import_binding(saved: &Snapshot, scope: &Scope, input: &ImportInput<'_>) -> Result<()> {
    if saved.key != memory_lifecycle::scope_key(scope)?
        || saved.input_hash != input.sha256
        || saved.input_bytes != input.byte_len as i64
    {
        return Err(MemoryError::OperationConflict);
    }
    Ok(())
}

/// Read only: a completed import can be inspected without reopening its archive.
/// A missing row never permits replay of the caller's previously issued intent.
pub(crate) fn inspect_import(
    state_root: &Path,
    backup_root: &Path,
    scope: Scope,
    input: ImportInput<'_>,
) -> Result<Option<BackupInfo>> {
    validate_import(&input)?;
    let _access = memory_lifecycle::ScopeAccess::shared(state_root, &scope, false)?;
    let catalog = match Catalog::read(state_root, backup_root) {
        Err(MemoryError::NotFound) => return Ok(None),
        Err(MemoryError::Io(error)) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        other => other?,
    };
    let saved = match snapshot(&catalog.connection, &snapshot_id(input.operation_id)?) {
        Err(MemoryError::NotFound) => return Ok(None),
        other => other?,
    };
    import_binding(&saved, &scope, &input)?;
    let (epoch, _) = memory_lifecycle::epoch_and_exclusions(state_root, &scope)?;
    if saved.phase == "eligible" {
        let source = verify_snapshot(&saved, &catalog.root, &scope)?;
        memory_lifecycle::check_current_exclusions(state_root, &scope, &source)?;
    }
    Ok(Some(saved.info(epoch)?))
}

/// Export a consistent existing scope while maintenance and scope exclusion hold.
/// Publication, destination ownership and the export operation receipt belong to
/// the caller. This path neither creates a backup nor imports confirmations.
pub(crate) fn export_scope(
    maintenance: &MaintenanceGuard,
    scope: Scope,
    writer: impl Write,
) -> Result<crate::memory::transfer::TransferStats> {
    let root = maintenance.state_root();
    let access = ExclusiveScope::acquire(root, &scope)?;
    check_restore_pending(root, &scope)?;
    let mut live = MemoryStore::open_for_lifecycle(root, scope.clone(), &access)?;
    memory_lifecycle::check_current_exclusions(root, &scope, live.lifecycle_connection())?;
    crate::memory::transfer::export(live.lifecycle_connection(), &scope, writer)
}

/// Construct only a managed recovery point. Activation remains a separate issued
/// restore operation. Failed construction stays accounted as `copying`; explicit
/// reconcile_backup may remove that exact owned stage, consuming the old intent.
pub(crate) fn import_snapshot(
    maintenance: &MaintenanceGuard,
    backup_root: &Path,
    scope: Scope,
    source: &mut File,
    input: ImportInput<'_>,
    policy: BackupPolicy,
) -> Result<BackupInfo> {
    validate_import(&input)?;
    if policy.max_bytes > i64::MAX as u64 || policy.max_bytes < META_HEADROOM {
        return Err(MemoryError::InvalidInput(
            "invalid import backup byte budget",
        ));
    }
    // A hard SQLite page ceiling is enforced even if this deliberately generous
    // archive-to-database allowance is insufficient. Never silently exceed the
    // reservation to accommodate pathological row/index expansion. Three times
    // the DB ceiling covers the database, rollback pages/headers and headroom.
    let db_limit = input
        .byte_len
        .checked_mul(4)
        .and_then(|n| n.checked_add(1024 * 1024 + 4095))
        .map(|n| n / 4096 * 4096)
        .ok_or(MemoryError::InvalidInput("import reservation overflow"))?;
    let reservation = db_limit
        .checked_mul(3)
        .and_then(|n| n.checked_add(META_HEADROOM))
        .ok_or(MemoryError::InvalidInput("import reservation overflow"))?;
    // Refuse an impossible single reservation before any eligible eviction.
    if reservation > policy.max_bytes {
        return Err(MemoryError::InvalidInput(
            "import construction exceeds backup capacity",
        ));
    }
    ensure_native_backup()?;
    let started = Instant::now();
    let root = maintenance.state_root();
    let access = ExclusiveScope::acquire(root, &scope)?;
    check_restore_pending(root, &scope)?;
    // Require an existing selected canonical scope; importing cannot register or
    // create live memory. Its contents remain untouched throughout construction.
    let mut live = MemoryStore::open_for_lifecycle(root, scope.clone(), &access)?;
    memory_lifecycle::check_current_exclusions(root, &scope, live.lifecycle_connection())?;
    drop(live);
    let before = input_stamp(source)?;
    verify_input_bytes(source, &input, started)?;
    if input_stamp(source)? != before {
        return Err(MemoryError::InvalidInput(
            "import source changed during verification",
        ));
    }
    let id = snapshot_id(input.operation_id)?;
    let key = memory_lifecycle::scope_key(&scope)?;
    let (epoch, _) = memory_lifecycle::epoch_and_exclusions(root, &scope)?;
    let mut catalog = Catalog::open(root, backup_root, true)?;
    match snapshot(&catalog.connection, &id) {
        Ok(saved) => {
            import_binding(&saved, &scope, &input)?;
            if saved.phase == "eligible" && saved.epoch == epoch as i64 {
                let canonical = verify_snapshot(&saved, &catalog.root, &scope)?;
                memory_lifecycle::check_current_exclusions(root, &scope, &canonical)?;
                return saved.info(epoch);
            }
            return Err(MemoryError::InvalidInput(
                "import is incomplete or invalidated; inspect and reconcile its issued ID, never replay construction",
            ));
        }
        Err(MemoryError::NotFound) => (),
        Err(error) => return Err(error),
    }
    let count: i64 = catalog
        .connection
        .query_row("SELECT count(*) FROM snapshots", [], |r| r.get(0))?;
    if count >= MAX_SNAPSHOTS {
        return Err(MemoryError::InvalidInput(
            "managed snapshot inventory is full",
        ));
    }
    catalog.make_room(reservation, policy.max_bytes)?;
    catalog.connection.execute("INSERT INTO snapshots(id,scope_key,scope_json,epoch,phase,byte_len,reserved_bytes,sha256,file_stamp,created_at,protected,input_sha256,input_bytes) VALUES(?1,?2,?3,?4,'copying',0,?5,'','',?6,0,?7,?8)",params![id,key,serde_json::to_string(&scope)?,epoch as i64,reservation as i64,i64::try_from(now()?).map_err(|_|MemoryError::Clock)?,input.sha256,input.byte_len as i64])?;
    let path = catalog.root.join(format!("{id}.sqlite3"));
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path)?;
    let stamp = file_stamp(&file.metadata()?)?;
    drop(private_file(&path, false)?);
    file.sync_all()?;
    sync_directory(&catalog.root)?;
    catalog.connection.execute(
        "UPDATE snapshots SET file_stamp=?2 WHERE id=?1",
        params![id, stamp],
    )?;
    drop(catalog);
    let mut destination = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let constructed = (|| -> Result<()> {
        destination.execute_batch("PRAGMA page_size=4096; PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA temp_store=MEMORY; PRAGMA foreign_keys=ON;")?;
        destination.pragma_update(
            None,
            "max_page_count",
            i64::try_from(db_limit / 4096).map_err(|_| MemoryError::UnsupportedSchema)?,
        )?;
        let ceiling: u64 = destination.query_row("PRAGMA max_page_count", [], |r| {
            r.get::<_, i64>(0).and_then(|n| {
                u64::try_from(n).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(0, n))
            })
        })?;
        if ceiling != db_limit / 4096 {
            return Err(MemoryError::InvalidInput(
                "SQLite refused import page ceiling",
            ));
        }
        source.seek(SeekFrom::Start(0))?;
        let mut reader = ImportReader {
            source,
            count: 0,
            hash: Sha256::new(),
            limit: input.byte_len,
            started,
        };
        crate::memory::transfer::import(&mut destination, &scope, &mut reader)?;
        if reader.count != input.byte_len
            || reader
                .hash
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
                != input.sha256
        {
            return Err(MemoryError::InvalidInput(
                "import source bytes changed during construction",
            ));
        }
        if input_stamp(source)? != before {
            return Err(MemoryError::InvalidInput(
                "import source identity changed during construction",
            ));
        }
        verify_input_bytes(source, &input, started)?;
        if input_stamp(source)? != before {
            return Err(MemoryError::InvalidInput(
                "import source changed after construction",
            ));
        }
        validate_database(&mut destination, &scope)?;
        memory_lifecycle::check_current_exclusions(root, &scope, &destination)?;
        let confirmations: i64 =
            destination.query_row("SELECT count(*) FROM mutations", [], |r| r.get(0))?;
        let restore_marker: bool = destination.query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='last_restore_operation')",
            [],
            |r| r.get(0),
        )?;
        if confirmations != 0 || restore_marker {
            return Err(MemoryError::InvalidInput(
                "import cannot supply operation confirmations",
            ));
        }
        Ok(())
    })();
    drop(destination);
    if let Err(error) = constructed {
        // A definite rejection must not introduce managed copies of previously
        // forgotten plaintext. Unknown ownership still prevents removal and
        // leaves the copying reservation inspectable for explicit recovery.
        let mut catalog = Catalog::open(root, backup_root, false)?;
        let saved = snapshot(&catalog.connection, &id)?;
        import_binding(&saved, &scope, &input)?;
        if saved.phase != "copying" || saved.stamp != stamp {
            return Err(MemoryError::InvalidInput(
                "rejected import cleanup ownership is unknown",
            ));
        }
        drop(file);
        catalog.remove(&saved)?;
        return Err(error);
    }
    if file_stamp(&file.metadata()?)? != stamp {
        return Err(MemoryError::InvalidInput("import stage ownership changed"));
    }
    file.sync_all()?;
    let (bytes, hash) = hash_file(&path, db_limit)?;
    let catalog = Catalog::open(root, backup_root, false)?;
    let saved = snapshot(&catalog.connection, &id)?;
    import_binding(&saved, &scope, &input)?;
    if saved.phase != "copying"
        || saved.stamp != stamp
        || file_stamp(&fs::symlink_metadata(&path)?)? != stamp
    {
        return Err(MemoryError::InvalidInput(
            "import stage ownership changed before eligibility",
        ));
    }
    let (current_epoch, _) = memory_lifecycle::epoch_and_exclusions(root, &scope)?;
    if current_epoch != epoch {
        return Err(MemoryError::InvalidInput(
            "forgetting epoch changed before import eligibility",
        ));
    }
    catalog.connection.execute(
        "UPDATE snapshots SET phase='eligible',byte_len=?2,reserved_bytes=0,sha256=?3 WHERE id=?1",
        params![id, bytes as i64, hash],
    )?;
    sync_directory(&catalog.root)?;
    snapshot(&catalog.connection, &id)?.info(epoch)
}

fn input_stamp(file: &File) -> Result<String> {
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(MemoryError::InvalidInput(
            "import requires a selected regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!(
            "{}:{}:{}:{}:{}:{}:{}",
            file_stamp(&meta)?,
            meta.len(),
            meta.mtime(),
            meta.mtime_nsec(),
            meta.ctime(),
            meta.ctime_nsec(),
            meta.nlink()
        ))
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        Err(MemoryError::InvalidInput(
            "native import identity is not qualified",
        ))
    }
}
struct ImportReader<'a> {
    source: &'a mut File,
    count: u64,
    hash: Sha256,
    limit: u64,
    started: Instant,
}
impl Read for ImportReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.started.elapsed() > Duration::from_secs(120) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "import input deadline exceeded",
            ));
        }
        let allowance =
            usize::try_from((self.limit + 1).saturating_sub(self.count)).unwrap_or(usize::MAX);
        let size = buffer.len().min(allowance);
        let read = self.source.read(&mut buffer[..size])?;
        self.count += read as u64;
        if self.count > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "import input exceeds bound length",
            ));
        }
        self.hash.update(&buffer[..read]);
        Ok(read)
    }
}
fn verify_input_bytes(file: &mut File, input: &ImportInput<'_>, started: Instant) -> Result<()> {
    if file.metadata()?.len() != input.byte_len {
        return Err(MemoryError::InvalidInput("import input length changed"));
    }
    file.seek(SeekFrom::Start(0))?;
    let mut reader = ImportReader {
        source: file,
        count: 0,
        hash: Sha256::new(),
        limit: input.byte_len,
        started,
    };
    io::copy(&mut reader, &mut io::sink())?;
    if reader.count != input.byte_len
        || reader
            .hash
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
            != input.sha256
    {
        return Err(MemoryError::InvalidInput("import input checksum changed"));
    }
    Ok(())
}

fn backup_locked(
    state_root: &Path,
    backup_root: &Path,
    scope: &Scope,
    operation_id: &str,
    capture: Capture,
    policy: BackupPolicy,
    source: &Connection,
) -> Result<BackupInfo> {
    let Capture { kind, at } = capture;
    if policy.max_bytes > i64::MAX as u64 || policy.max_bytes < META_HEADROOM {
        return Err(MemoryError::InvalidInput(
            "backup byte budget cannot admit catalog and journal headroom",
        ));
    }
    let id = snapshot_id(operation_id)?;
    let key = memory_lifecycle::scope_key(scope)?;
    let (epoch, _) = memory_lifecycle::epoch_and_exclusions(state_root, scope)?;
    let mut catalog = Catalog::open(state_root, backup_root, true)?;
    let existing = match snapshot(&catalog.connection, &id) {
        Ok(saved) => Some(saved),
        Err(MemoryError::NotFound) => None,
        Err(error) => return Err(error),
    };
    if let Some(existing) = existing {
        if existing.key != key || !existing.input_hash.is_empty() {
            return Err(MemoryError::OperationConflict);
        }
        if existing.phase == "eligible" && existing.epoch == epoch as i64 {
            verify_snapshot(&existing, &catalog.root, scope)?;
            return existing.info(epoch);
        }
        return Err(MemoryError::InvalidInput(
            "backup operation has incomplete or invalidated state; inspect and reconcile it before issuing another backup",
        ));
    }
    if matches!(kind, BackupKind::Daily | BackupKind::Weekly) {
        let bucket = if kind == BackupKind::Daily {
            at / 86400
        } else {
            at / (86400 * 7)
        };
        let due:Option<String>=catalog.connection.query_row("SELECT snapshot_id FROM snapshot_refs WHERE scope_key=?1 AND kind=?2 AND bucket=?3",params![key,kind.text(),bucket as i64],|r|r.get(0)).optional()?;
        if let Some(due) = due {
            let saved = snapshot(&catalog.connection, &due)?;
            if saved.epoch == epoch as i64 && saved.phase == "eligible" {
                verify_snapshot(&saved, &catalog.root, scope)?;
                return saved.info(epoch);
            }
        }
    }
    let pages: i64 = source.query_row("PRAGMA page_count", [], |r| r.get(0))?;
    let page_size: i64 = source.query_row("PRAGMA page_size", [], |r| r.get(0))?;
    let source_bytes = u64::try_from(pages)
        .ok()
        .and_then(|p| u64::try_from(page_size).ok().and_then(|s| p.checked_mul(s)))
        .ok_or(MemoryError::UnsupportedSchema)?;
    let reservation = source_bytes
        .checked_mul(2)
        .and_then(|n| n.checked_add(META_HEADROOM))
        .ok_or(MemoryError::InvalidInput("backup size overflow"))?;
    catalog.make_room(reservation, policy.max_bytes)?;
    let count: i64 = catalog
        .connection
        .query_row("SELECT count(*) FROM snapshots", [], |r| r.get(0))?;
    if count >= MAX_SNAPSHOTS {
        return Err(MemoryError::InvalidInput(
            "managed snapshot inventory is full",
        ));
    }
    catalog.connection.execute("INSERT INTO snapshots(id,scope_key,scope_json,epoch,phase,byte_len,reserved_bytes,sha256,file_stamp,created_at,protected) VALUES(?1,?2,?3,?4,'copying',0,?5,'','',?6,?7)",params![id,key,serde_json::to_string(scope)?,epoch as i64,reservation as i64,i64::try_from(at).map_err(|_|MemoryError::Clock)?,kind==BackupKind::PreMigration])?;
    let path = catalog.root.join(format!("{id}.sqlite3"));
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&path)?;
    drop(private_file(&path, false)?);
    file.sync_all()?;
    sync_directory(&catalog.root)?;
    let stamp = file_stamp(&file.metadata()?)?;
    drop(file);
    catalog.connection.execute(
        "UPDATE snapshots SET file_stamp=?2 WHERE id=?1",
        params![id, stamp],
    )?;
    drop(catalog); // No installation-wide catalog lock is held while SQLite copies.
    let mut destination = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    copy_database(source, &mut destination)?;
    validate_database(&mut destination, scope)?;
    drop(destination);
    File::open(&path)?.sync_all()?;
    let (bytes, hash) = hash_file(&path, reservation)?;
    let mut catalog = Catalog::open(state_root, backup_root, false)?;
    let saved = snapshot(&catalog.connection, &id)?;
    if saved.phase != "copying" || saved.stamp != file_stamp(&fs::symlink_metadata(&path)?)? {
        return Err(MemoryError::InvalidInput(
            "snapshot ownership changed before completion",
        ));
    }
    let (current_epoch, _) = memory_lifecycle::epoch_and_exclusions(state_root, scope)?;
    if current_epoch != epoch {
        return Err(MemoryError::InvalidInput(
            "forgetting epoch changed before snapshot completion",
        ));
    }
    let tx = catalog
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "UPDATE snapshots SET phase='eligible',byte_len=?2,reserved_bytes=0,sha256=?3 WHERE id=?1",
        params![id, bytes as i64, hash],
    )?;
    if matches!(kind, BackupKind::Daily | BackupKind::Weekly) {
        // A capture can serve the due daily and weekly points without storing twice.
        for (name, bucket) in [("daily", at / 86400), ("weekly", at / (86400 * 7))] {
            tx.execute("INSERT OR IGNORE INTO snapshot_refs(scope_key,kind,bucket,snapshot_id) VALUES(?1,?2,?3,?4)",params![key,name,bucket as i64,id])?;
        }
        for (name, keep) in [("daily", 7), ("weekly", 4)] {
            tx.execute("DELETE FROM snapshot_refs WHERE scope_key=?1 AND kind=?2 AND bucket NOT IN(SELECT bucket FROM snapshot_refs WHERE scope_key=?1 AND kind=?2 ORDER BY bucket DESC LIMIT ?3)",params![key,name,keep])?;
        }
    }
    tx.commit()?;
    catalog.prune_unreferenced()?;
    snapshot(&catalog.connection, &id)?.info(epoch)
}

pub fn inspect_backups(
    state_root: &Path,
    backup_root: &Path,
    scope: Scope,
) -> Result<Vec<BackupInfo>> {
    let _access = memory_lifecycle::ScopeAccess::shared(state_root, &scope, false)?;
    let catalog = Catalog::read(state_root, backup_root)?;
    let key = memory_lifecycle::scope_key(&scope)?;
    let (epoch, _) = memory_lifecycle::epoch_and_exclusions(state_root, &scope)?;
    snapshots(&catalog.connection)?
        .into_iter()
        .filter(|s| s.key == key)
        .map(|s| {
            if s.phase == "eligible" && s.epoch == epoch as i64 {
                verify_snapshot(&s, &catalog.root, &scope)?;
            }
            s.info(epoch)
        })
        .collect()
}

fn copy_database(source: &Connection, target: &mut Connection) -> Result<()> {
    target.execute_batch("PRAGMA synchronous=FULL; PRAGMA journal_mode=DELETE;")?;
    let started = Instant::now();
    let backup = rusqlite::backup::Backup::new(source, target)?;
    loop {
        match backup.step(128)? {
            rusqlite::backup::StepResult::Done => return Ok(()),
            rusqlite::backup::StepResult::More => (),
            rusqlite::backup::StepResult::Busy | rusqlite::backup::StepResult::Locked => {
                std::thread::sleep(Duration::from_millis(5))
            }
            _ => {
                return Err(MemoryError::InvalidInput(
                    "unsupported SQLite backup progress",
                ));
            }
        }
        if started.elapsed() > Duration::from_secs(120) {
            return Err(MemoryError::InvalidInput(
                "SQLite backup deadline exceeded; staged snapshot remains ineligible",
            ));
        }
    }
}
fn validate_database(connection: &mut Connection, scope: &Scope) -> Result<()> {
    crate::memory::validate_lifecycle_snapshot(connection, scope)?;
    let integrity: String = connection.query_row("PRAGMA integrity_check(1)", [], |r| r.get(0))?;
    let foreign: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)",
        [],
        |r| r.get(0),
    )?;
    if integrity != "ok" || foreign {
        return Err(MemoryError::InvalidInput(
            "memory recovery snapshot integrity failed",
        ));
    }
    Ok(())
}
fn verify_snapshot(snapshot: &Snapshot, root: &Path, scope: &Scope) -> Result<Connection> {
    let path = snapshot.path(root);
    let file = private_file(&path, false)?;
    if file_stamp(&file.metadata()?)? != snapshot.stamp {
        return Err(MemoryError::InvalidInput("snapshot file identity changed"));
    }
    drop(file);
    let (bytes, hash) = hash_file(
        &path,
        u64::try_from(snapshot.bytes).map_err(|_| MemoryError::UnsupportedSchema)?,
    )?;
    if bytes != snapshot.bytes as u64 || hash != snapshot.hash {
        return Err(MemoryError::InvalidInput("managed snapshot bytes changed"));
    }
    let mut connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    validate_database(&mut connection, scope)?;
    Ok(connection)
}
fn hash_file(path: &Path, max: u64) -> Result<(u64, String)> {
    let mut file = private_file(path, false)?;
    let mut sha = Sha256::new();
    let mut size = 0u64;
    let mut chunk = [0; 65536];
    loop {
        let n = file.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        size = size
            .checked_add(n as u64)
            .ok_or(MemoryError::InvalidInput("snapshot size overflow"))?;
        if size > max {
            return Err(MemoryError::InvalidInput(
                "snapshot exceeded reserved bytes",
            ));
        }
        sha.update(&chunk[..n]);
    }
    Ok((
        size,
        sha.finalize().iter().map(|b| format!("{b:02x}")).collect(),
    ))
}

impl Catalog {
    fn open(state_root: &Path, backup_root: &Path, create: bool) -> Result<Self> {
        Self::open_mode(state_root, backup_root, create, true)
    }
    fn read(state_root: &Path, backup_root: &Path) -> Result<Self> {
        Self::open_mode(state_root, backup_root, false, false)
    }
    fn open_mode(
        state_root: &Path,
        backup_root: &Path,
        create: bool,
        writable: bool,
    ) -> Result<Self> {
        ensure_native_backup()?;
        let root = memory_lifecycle::bind_backup_root(state_root, backup_root, create)?;
        let lock = private_file(&root.join("catalog.lock"), create)?;
        if lock.metadata()?.len() != 0 {
            return Err(MemoryError::InvalidInput(
                "backup catalog lock must be empty",
            ));
        }
        lock.try_lock()
            .map_err(|_| MemoryError::InvalidInput("backup catalog is busy; retry admission"))?;
        let lock = FileLock::acquired(lock);
        let path = root.join("catalog.sqlite3");
        let catalog_file = private_file(&path, create)?;
        if catalog_file.metadata()?.len() > MAX_CATALOG {
            return Err(MemoryError::InvalidInput(
                "backup catalog exceeds its bound",
            ));
        }
        drop(catalog_file);
        let mut connection = Connection::open_with_flags(
            path,
            (if writable {
                OpenFlags::SQLITE_OPEN_READ_WRITE
            } else {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            }) | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.busy_timeout(Duration::from_millis(250))?;
        connection.set_db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
        connection.set_db_config(
            rusqlite::config::DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA,
            false,
        )?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
        let journal_mode: String = connection.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
        if journal_mode != "delete" {
            return Err(MemoryError::UnsupportedSchema);
        }
        let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if version == 0 && create {
            let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
                [],
                |r| r.get(0),
            )?;
            if count != 0 {
                return Err(MemoryError::UnsupportedSchema);
            }
            tx.execute_batch("CREATE TABLE snapshots(id TEXT PRIMARY KEY,scope_key TEXT NOT NULL,scope_json TEXT NOT NULL,epoch INTEGER NOT NULL,phase TEXT NOT NULL,byte_len INTEGER NOT NULL,reserved_bytes INTEGER NOT NULL,sha256 TEXT NOT NULL,file_stamp TEXT NOT NULL,created_at INTEGER NOT NULL,protected INTEGER NOT NULL,input_sha256 TEXT NOT NULL DEFAULT '',input_bytes INTEGER NOT NULL DEFAULT 0) STRICT; CREATE TABLE snapshot_refs(scope_key TEXT NOT NULL,kind TEXT NOT NULL,bucket INTEGER NOT NULL,snapshot_id TEXT NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,PRIMARY KEY(scope_key,kind,bucket)) STRICT; CREATE TABLE restores(operation_id TEXT PRIMARY KEY,scope_key TEXT NOT NULL,snapshot_id TEXT NOT NULL,preserved_snapshot_id TEXT NOT NULL,phase TEXT NOT NULL,created_at INTEGER NOT NULL DEFAULT 0) STRICT;")?;
            tx.pragma_update(None, "application_id", APP_ID)?;
            tx.pragma_update(None, "user_version", 1)?;
            tx.commit()?;
        }
        let(app,version):(i64,i64)=connection.query_row("SELECT (SELECT application_id FROM pragma_application_id),(SELECT user_version FROM pragma_user_version)",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if app != APP_ID || version != 1 {
            return Err(MemoryError::UnsupportedSchema);
        }
        let catalog = Self {
            _lock: lock,
            connection,
            root,
        };
        catalog.usage()?;
        Ok(catalog)
    }
    fn usage(&self) -> Result<u64> {
        let saved = snapshots(&self.connection)?;
        let mut total = 0u64;
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name
                .to_str()
                .ok_or(MemoryError::InvalidInput("unknown backup file name"))?;
            let file = private_file(&entry.path(), false)?;
            let len = file.metadata()?.len();
            if name == "catalog.lock" {
                if len != 0 {
                    return Err(MemoryError::InvalidInput("backup lock drift"));
                }
            } else if name == "catalog.sqlite3" || name == "catalog.sqlite3-journal" {
                if len > MAX_CATALOG {
                    return Err(MemoryError::InvalidInput(
                        "backup catalog exceeds its bound",
                    ));
                }
                total = total
                    .checked_add(len)
                    .ok_or(MemoryError::InvalidInput("backup size overflow"))?;
            } else if !saved.iter().any(|s| {
                name == format!("{}.sqlite3", s.id) || name == format!("{}.sqlite3-journal", s.id)
            }) {
                return Err(MemoryError::InvalidInput(
                    "unknown backup entry is preserved; admission requires reconciliation",
                ));
            }
        }
        // Catalog rollback may duplicate its existing pages; reserve bounded row
        // growth as well. Snapshot reservations include their own journal peak.
        let catalog_bytes = fs::metadata(self.root.join("catalog.sqlite3"))?.len();
        total = total
            .checked_add(catalog_bytes)
            .and_then(|v| v.checked_add(META_HEADROOM))
            .ok_or(MemoryError::InvalidInput("backup metadata overflow"))?;
        for snapshot in saved {
            let mut actual = 0u64;
            for suffix in [".sqlite3", ".sqlite3-journal"] {
                match fs::symlink_metadata(self.root.join(format!("{}{suffix}", snapshot.id))) {
                    Ok(meta) => {
                        actual = actual
                            .checked_add(meta.len())
                            .ok_or(MemoryError::InvalidInput("backup size overflow"))?
                    }
                    Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                    Err(e) => return Err(e.into()),
                }
            }
            let charge = if snapshot.phase == "copying" {
                snapshot.reserved
            } else {
                snapshot.bytes
            };
            if actual > u64::try_from(charge).map_err(|_| MemoryError::UnsupportedSchema)? {
                return Err(MemoryError::InvalidInput(
                    "snapshot bytes exceed catalog accounting; preserve and reconcile drift",
                ));
            }
            total = total
                .checked_add(u64::try_from(charge).map_err(|_| MemoryError::UnsupportedSchema)?)
                .ok_or(MemoryError::InvalidInput("backup usage overflow"))?;
        }
        Ok(total)
    }
    fn protected(&self, snapshot: &Snapshot) -> Result<bool> {
        let latest:Option<String>=self.connection.query_row("SELECT id FROM snapshots WHERE scope_key=?1 AND phase='eligible' ORDER BY created_at DESC,rowid DESC LIMIT 1",[&snapshot.key],|r|r.get(0)).optional()?;
        let restore:bool=self.connection.query_row("SELECT EXISTS(SELECT 1 FROM restores WHERE phase IN('preserving','prepared') AND (snapshot_id=?1 OR preserved_snapshot_id=?1))",[&snapshot.id],|r|r.get(0))?;
        Ok(snapshot.protected || latest.as_deref() == Some(&snapshot.id) || restore)
    }
    fn make_room(&mut self, reservation: u64, cap: u64) -> Result<()> {
        if self
            .usage()?
            .checked_add(reservation)
            .is_none_or(|bytes| bytes > cap)
        {
            // Never sacrifice an older good recovery point to a latest entry
            // whose stored bytes no longer verify.
            for saved in snapshots(&self.connection)? {
                if saved.phase == "eligible" && self.protected(&saved)? {
                    let scope: Scope = serde_json::from_str(&saved.scope)?;
                    verify_snapshot(&saved, &self.root, &scope)?;
                }
            }
        }
        while self
            .usage()?
            .checked_add(reservation)
            .is_none_or(|v| v > cap)
        {
            let mut chosen = None;
            for saved in snapshots(&self.connection)? {
                if saved.phase == "eligible" && !self.protected(&saved)? {
                    chosen = Some(saved);
                    break;
                }
            }
            let Some(saved) = chosen else {
                return Err(MemoryError::InvalidInput(
                    "protected recovery points leave insufficient backup capacity; live memory is unchanged",
                ));
            };
            self.remove(&saved)?;
        }
        Ok(())
    }
    fn remove(&mut self, snapshot: &Snapshot) -> Result<()> {
        let path = snapshot.path(&self.root);
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                let file = private_file(&path, false)?;
                if snapshot.stamp.is_empty() || file_stamp(&file.metadata()?)? != snapshot.stamp {
                    return Err(MemoryError::InvalidInput(
                        "unknown snapshot ownership; cleanup remains pending",
                    ));
                }
                drop(file);
                if !snapshot.hash.is_empty() {
                    let (bytes, hash) = hash_file(&path, snapshot.bytes as u64)?;
                    if bytes != snapshot.bytes as u64 || hash != snapshot.hash {
                        return Err(MemoryError::InvalidInput("snapshot drift prevents cleanup"));
                    }
                }
                self.connection.execute(
                    "UPDATE snapshots SET phase='deleting' WHERE id=?1",
                    [&snapshot.id],
                )?;
                fs::remove_file(path)?;
                sync_directory(&self.root)?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
        let journal = self.root.join(format!("{}.sqlite3-journal", snapshot.id));
        match fs::symlink_metadata(&journal) {
            Ok(_) => {
                drop(private_file(&journal, false)?);
                fs::remove_file(journal)?;
                sync_directory(&self.root)?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
        self.connection
            .execute("DELETE FROM snapshots WHERE id=?1", [&snapshot.id])?;
        Ok(())
    }
    fn prune_unreferenced(&mut self) -> Result<()> {
        for saved in snapshots(&self.connection)? {
            let referenced: bool = self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM snapshot_refs WHERE snapshot_id=?1)",
                [&saved.id],
                |r| r.get(0),
            )?;
            if saved.phase == "eligible" && !referenced && !self.protected(&saved)? {
                self.remove(&saved)?;
            }
        }
        Ok(())
    }
}

fn ensure_native_backup() -> Result<()> {
    #[cfg(unix)]
    {
        Ok(())
    }
    #[cfg(not(unix))]
    {
        Err(MemoryError::InvalidInput(
            "native backup publication is not qualified on this platform",
        ))
    }
}
fn private_file(path: &Path, create: bool) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    if create {
        options.create_new(true);
    }
    let file = match options.open(path) {
        Ok(f) => f,
        Err(e) if create && e.kind() == io::ErrorKind::AlreadyExists => {
            OpenOptions::new().read(true).write(true).open(path)?
        }
        Err(e) => return Err(e.into()),
    };
    let path_meta = fs::symlink_metadata(path)?;
    let opened = file.metadata()?;
    if !path_meta.is_file() || path_meta.file_type().is_symlink() {
        return Err(MemoryError::InvalidInput(
            "backup files must be private regular files",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_meta.mode() & 0o077 != 0
            || path_meta.nlink() != 1
            || (path_meta.dev(), path_meta.ino()) != (opened.dev(), opened.ino())
        {
            return Err(MemoryError::InvalidInput(
                "backup file identity or permissions changed",
            ));
        }
    }
    Ok(file)
}
fn file_stamp(meta: &fs::Metadata) -> Result<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let created = meta
            .created()?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MemoryError::Clock)?;
        Ok(format!(
            "{}:{}:{}",
            meta.dev(),
            meta.ino(),
            created.as_nanos()
        ))
    }
    #[cfg(not(unix))]
    {
        let _ = meta;
        Err(MemoryError::InvalidInput(
            "native backup file identity is not qualified on this platform",
        ))
    }
}
fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/// Called with the selected scope's exclusive lifecycle lock already held.
/// Missing or drifted bound roots fail cleanup rather than hiding backed content.
pub(crate) fn reconcile_forgetting(state_root: &Path, scope: &Scope) -> Result<()> {
    reconcile_snapshots(state_root, scope, true)
}
fn reconcile_snapshots(state_root: &Path, scope: &Scope, invalidate_restore: bool) -> Result<()> {
    let Some(root) = memory_lifecycle::configured_backup_root(state_root)? else {
        return Ok(());
    };
    let (epoch, excluded) = memory_lifecycle::epoch_and_exclusions(state_root, scope)?;
    let key = memory_lifecycle::scope_key(scope)?;
    let mut catalog = Catalog::open(state_root, &root, false)?;
    for saved in snapshots(&catalog.connection)?
        .into_iter()
        .filter(|s| s.key == key && s.epoch != epoch as i64)
    {
        if saved.phase == "copying" || saved.phase == "deleting" {
            catalog.remove(&saved)?;
            continue;
        }
        let canonical = verify_snapshot(&saved, &catalog.root, scope)?;
        let mut affected = false;
        for record in &excluded {
            let present:bool=canonical.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE record_id=?1 UNION ALL SELECT 1 FROM revisions WHERE record_id=?1 UNION ALL SELECT 1 FROM mutations WHERE record_id=?1 AND intent_hash<>'')",[record],|r|r.get(0))?;
            if present {
                affected = true;
                break;
            }
        }
        drop(canonical);
        if affected {
            catalog.remove(&saved)?;
        } else {
            catalog.connection.execute(
                "UPDATE snapshots SET epoch=?2 WHERE id=?1",
                params![saved.id, epoch as i64],
            )?;
        }
    }
    if invalidate_restore {
        catalog.connection.execute("UPDATE restores SET phase='invalidated' WHERE scope_key=?1 AND phase IN('preserving','prepared')",[key])?;
        if let Some(operation) = memory_lifecycle::pending_restore(state_root, scope)? {
            memory_lifecycle::clear_pending_restore(state_root, scope, &operation)?;
        }
    }
    Ok(())
}

/// Reconcile only this issued snapshot identity; it cannot adopt arbitrary files.
/// A known incomplete snapshot can be removed, releasing its reserved bytes,
/// before an explicit retry captures fresh current state. Unknown ownership stays.
pub fn reconcile_backup(
    state_root: &Path,
    backup_root: &Path,
    scope: Scope,
    operation_id: &str,
) -> Result<Option<BackupInfo>> {
    let _access = ExclusiveScope::acquire(state_root, &scope)?;
    reconcile_snapshots(state_root, &scope, false)?;
    let id = snapshot_id(operation_id)?;
    let key = memory_lifecycle::scope_key(&scope)?;
    let mut catalog = Catalog::open(state_root, backup_root, false)?;
    let saved = match snapshot(&catalog.connection, &id) {
        Err(MemoryError::NotFound) => return Ok(None),
        other => other?,
    };
    if saved.key != key {
        return Err(MemoryError::ScopeMismatch);
    }
    if saved.phase == "copying" || saved.phase == "deleting" {
        catalog.remove(&saved)?;
        return Ok(None);
    }
    let (epoch, _) = memory_lifecycle::epoch_and_exclusions(state_root, &scope)?;
    verify_snapshot(&saved, &catalog.root, &scope)?;
    Ok(Some(saved.info(epoch)?))
}

fn preservation_operation(operation_id: &str) -> String {
    format!(
        "op-preserve-{}",
        Sha256::digest(operation_id.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

/// The existing maintenance guard proves ordinary runtime sessions are absent.
/// Per-scope exclusion remains held through preservation, restore and completion.
/// A pending restore blocks normal scope opens; retrying its fixed saved input
/// cannot overwrite intervening conforming writes because none are admitted.
/// The caller validates issued-ID expiry independently; absent/expired receipts
/// do not establish permission to execute an old restore again.
pub fn restore(
    maintenance: &MaintenanceGuard,
    backup_root: &Path,
    scope: Scope,
    snapshot_id: &str,
    operation_id: &str,
    policy: BackupPolicy,
) -> Result<RestoreStatus> {
    let root = maintenance.state_root();
    let _ = self::snapshot_id(operation_id)?;
    let access = ExclusiveScope::acquire(root, &scope)?;
    let key = memory_lifecycle::scope_key(&scope)?;
    let (epoch, _) = memory_lifecycle::epoch_and_exclusions(root, &scope)?;
    let preserve_operation = preservation_operation(operation_id);
    let preserved = self::snapshot_id(&preserve_operation)?;
    let catalog = Catalog::open(root, backup_root, false)?;
    let prior:Option<(String,String,String,String)>=catalog.connection.query_row("SELECT scope_key,snapshot_id,preserved_snapshot_id,phase FROM restores WHERE operation_id=?1",[operation_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    if let Some((bound_key, bound_snapshot, bound_preserved, phase)) = &prior {
        if bound_key != &key || bound_snapshot != snapshot_id || bound_preserved != &preserved {
            return Err(MemoryError::OperationConflict);
        }
        if phase == "complete" {
            memory_lifecycle::clear_pending_restore(root, &scope, operation_id)?;
            return Ok(RestoreStatus {
                phase: "complete".into(),
                operation_id: operation_id.into(),
                scope,
                snapshot_id: snapshot_id.into(),
                preserved_snapshot_id: preserved,
                complete: true,
            });
        }
        if phase == "aborted" || phase == "invalidated" {
            return Err(MemoryError::InvalidInput(
                "restore operation is aborted or invalidated; inspect before preparing a new restore",
            ));
        }
    }
    if prior
        .as_ref()
        .is_some_and(|p| !matches!(p.3.as_str(), "preserving" | "prepared"))
    {
        return Err(MemoryError::UnsupportedSchema);
    }
    let selected = snapshot(&catalog.connection, snapshot_id)?;
    if selected.key != key {
        return Err(MemoryError::ScopeMismatch);
    }
    if selected.phase != "eligible" || selected.epoch != epoch as i64 {
        return Err(MemoryError::InvalidInput(
            "selected recovery point is not currently eligible",
        ));
    }
    let source = verify_snapshot(&selected, &catalog.root, &scope)?;
    memory_lifecycle::check_current_exclusions(root, &scope, &source)?;
    if prior.is_none() {
        let cutoff = i64::try_from(now()?.saturating_sub(RESTORE_RECEIPT_SECONDS))
            .map_err(|_| MemoryError::Clock)?;
        let expired: Vec<String> = catalog.connection.prepare("SELECT operation_id FROM restores WHERE phase IN('complete','aborted','invalidated') AND created_at < ?1")?
            .query_map([cutoff], |r|r.get(0))?.collect::<std::result::Result<_,_>>()?;
        for expired_id in expired {
            // Completion may have committed before clearing external authority.
            // Preserve that exact receipt until its pending owner reconciles.
            if !memory_lifecycle::restore_operation_pending(root, &expired_id)? {
                catalog
                    .connection
                    .execute("DELETE FROM restores WHERE operation_id=?1", [&expired_id])?;
            }
        }
        let count: i64 =
            catalog
                .connection
                .query_row("SELECT count(*) FROM restores", [], |r| r.get(0))?;
        if count >= MAX_RESTORE_RECEIPTS {
            return Err(MemoryError::InvalidInput(
                "restore receipt capacity exhausted; reconcile pending operations or wait for completed receipt expiry",
            ));
        }
    }
    memory_lifecycle::set_pending_restore(root, &scope, operation_id, snapshot_id)?;
    if prior.is_none() {
        catalog.connection.execute("INSERT INTO restores(operation_id,scope_key,snapshot_id,preserved_snapshot_id,phase,created_at) VALUES(?1,?2,?3,?4,'preserving',?5)",params![operation_id,key,snapshot_id,preserved,i64::try_from(now()?).map_err(|_| MemoryError::Clock)?])?;
    }
    drop(catalog);
    let mut live = MemoryStore::open_for_lifecycle(root, scope.clone(), &access)?;
    let need_preserve = prior.as_ref().is_none_or(|p| p.3 == "preserving");
    if need_preserve {
        match backup_locked(
            root,
            backup_root,
            &scope,
            &preserve_operation,
            Capture {
                kind: BackupKind::PreRestore,
                at: now()?,
            },
            policy,
            live.lifecycle_connection(),
        ) {
            Ok(_) => (),
            Err(error) => {
                let catalog = Catalog::open(root, backup_root, false)?;
                catalog.connection.execute(
                    "UPDATE restores SET phase='aborted' WHERE operation_id=?1",
                    [operation_id],
                )?;
                memory_lifecycle::clear_pending_restore(root, &scope, operation_id)?;
                return Err(error);
            }
        }
        let catalog = Catalog::open(root, backup_root, false)?;
        catalog.connection.execute(
            "UPDATE restores SET phase='prepared' WHERE operation_id=?1",
            [operation_id],
        )?;
    }
    // A prepared receipt is not evidence that preserved bytes still exist.
    // Verify the prerequisite on every resumed write, before replacing live state.
    {
        let catalog = Catalog::read(root, backup_root)?;
        let saved = snapshot(&catalog.connection, &preserved)?;
        if saved.key != key || saved.phase != "eligible" || saved.epoch != epoch as i64 {
            return Err(MemoryError::InvalidInput(
                "preserved current recovery point is unavailable",
            ));
        }
        let preserved_source = verify_snapshot(&saved, &catalog.root, &scope)?;
        memory_lifecycle::check_current_exclusions(root, &scope, &preserved_source)?;
    }
    let marker: Option<String> = live
        .lifecycle_connection()
        .query_row(
            "SELECT value FROM metadata WHERE key='last_restore_operation'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if marker.as_deref() != Some(operation_id) {
        copy_database(&source, live.lifecycle_connection())?;
        validate_database(live.lifecycle_connection(), &scope)?;
        memory_lifecycle::check_current_exclusions(root, &scope, live.lifecycle_connection())?;
        let tx = live
            .lifecycle_connection()
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM semantic_revisions", [])?;
        tx.execute("INSERT INTO record_fts(record_fts) VALUES('rebuild')", [])?;
        tx.execute("INSERT INTO metadata(key,value) VALUES('last_restore_operation',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[operation_id])?;
        tx.commit()?;
    }
    let catalog = Catalog::open(root, backup_root, false)?;
    catalog.connection.execute(
        "UPDATE restores SET phase='complete' WHERE operation_id=?1",
        [operation_id],
    )?;
    memory_lifecycle::clear_pending_restore(root, &scope, operation_id)?;
    Ok(RestoreStatus {
        phase: "complete".into(),
        operation_id: operation_id.into(),
        scope,
        snapshot_id: snapshot_id.into(),
        preserved_snapshot_id: preserved,
        complete: true,
    })
}

/// The maintenance owner calls this only after the migration no longer needs
/// its recovery point. It releases rotation protection; it does not delete bytes.
pub fn release_migration_protection(
    maintenance: &MaintenanceGuard,
    backup_root: &Path,
    scope: Scope,
    snapshot_id: &str,
) -> Result<BackupInfo> {
    let root = maintenance.state_root();
    let _access = ExclusiveScope::acquire(root, &scope)?;
    check_restore_pending(root, &scope)?;
    let catalog = Catalog::open(root, backup_root, false)?;
    let saved = snapshot(&catalog.connection, snapshot_id)?;
    let (epoch, _) = memory_lifecycle::epoch_and_exclusions(root, &scope)?;
    if saved.key != memory_lifecycle::scope_key(&scope)? {
        return Err(MemoryError::ScopeMismatch);
    }
    if saved.phase != "eligible" || saved.epoch != epoch as i64 {
        return Err(MemoryError::InvalidInput(
            "migration recovery point is not eligible",
        ));
    }
    let source = verify_snapshot(&saved, &catalog.root, &scope)?;
    memory_lifecycle::check_current_exclusions(root, &scope, &source)?;
    catalog.connection.execute(
        "UPDATE snapshots SET protected=0 WHERE id=?1",
        [snapshot_id],
    )?;
    snapshot(&catalog.connection, snapshot_id)?.info(epoch)
}

/// Inspect an issued restore without creating a catalog, updating its phase,
/// or registering a missing scope. Absence is unknown, never fresh permission.
pub fn inspect_restore(
    state_root: &Path,
    backup_root: &Path,
    scope: Scope,
    operation_id: &str,
) -> Result<Option<RestoreStatus>> {
    let _ = snapshot_id(operation_id)?;
    let _access = memory_lifecycle::ScopeAccess::shared(state_root, &scope, false)?;
    let catalog = Catalog::read(state_root, backup_root)?;
    let row: Option<(String, String, String, String)> = catalog.connection.query_row(
        "SELECT scope_key,snapshot_id,preserved_snapshot_id,phase FROM restores WHERE operation_id=?1",
        [operation_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)),
    ).optional()?;
    let Some((key, selected, preserved, phase)) = row else {
        // Authority commits before the catalog row. Keep that crash window
        // inspectable and resumable without granting fresh restore permission.
        return match memory_lifecycle::pending_restore_binding(state_root, &scope)? {
            Some((pending_id, selected)) if pending_id == operation_id => Ok(Some(RestoreStatus {
                phase: "preparing".into(),
                complete: false,
                operation_id: operation_id.into(),
                scope,
                snapshot_id: selected,
                preserved_snapshot_id: snapshot_id(&preservation_operation(operation_id))?,
            })),
            _ => Ok(None),
        };
    };
    if key != memory_lifecycle::scope_key(&scope)? {
        return Err(MemoryError::ScopeMismatch);
    }
    if !matches!(
        phase.as_str(),
        "preserving" | "prepared" | "complete" | "aborted" | "invalidated"
    ) {
        return Err(MemoryError::UnsupportedSchema);
    }
    Ok(Some(RestoreStatus {
        complete: phase == "complete",
        phase,
        operation_id: operation_id.into(),
        scope,
        snapshot_id: selected,
        preserved_snapshot_id: preserved,
    }))
}

/// Resume only an already recorded restore binding. This does not issue an
/// intent, accept a replacement target, or reinterpret an absent ID as fresh.
pub fn reconcile_restore(
    maintenance: &MaintenanceGuard,
    backup_root: &Path,
    scope: Scope,
    operation_id: &str,
    policy: BackupPolicy,
) -> Result<RestoreStatus> {
    let existing = inspect_restore(
        maintenance.state_root(),
        backup_root,
        scope.clone(),
        operation_id,
    )?
    .ok_or(MemoryError::NotFound)?;
    restore(
        maintenance,
        backup_root,
        scope,
        &existing.snapshot_id,
        operation_id,
        policy,
    )
}

pub(crate) fn check_restore_pending(state_root: &Path, scope: &Scope) -> Result<()> {
    if memory_lifecycle::pending_restore(state_root, scope)?.is_some() {
        return Err(MemoryError::InvalidInput(
            "scope has a pending restore; reconcile the issued restore operation before ordinary memory access",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Applicability, ClaimBasis, RecordInput, RecordKind, RecordStatus};
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
        backup: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mirket-backup-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            }
            let root = root.canonicalize().unwrap();
            Self {
                backup: root.join("backups"),
                root,
            }
        }
        fn scope(&self) -> Scope {
            Scope::Project("backup-project".into())
        }
        fn store(&self) -> MemoryStore {
            MemoryStore::open(&self.root, self.scope()).unwrap()
        }
        fn capture_at(
            &self,
            scope: Scope,
            at: u64,
            kind: BackupKind,
            policy: BackupPolicy,
        ) -> BackupInfo {
            let access = ExclusiveScope::acquire(&self.root, &scope).unwrap();
            let mut store =
                MemoryStore::open_for_lifecycle(&self.root, scope.clone(), &access).unwrap();
            backup_locked(
                &self.root,
                &self.backup,
                &scope,
                &op(),
                Capture { kind, at },
                policy,
                store.lifecycle_connection(),
            )
            .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    fn op() -> String {
        format!("op-backup-{}", NEXT.fetch_add(1, Ordering::Relaxed))
    }
    fn record(text: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::UserStated,
            content: text.into(),
            sources: vec![crate::memory::Source {
                source_id: "user-assertion".into(),
                revision: "1".into(),
                locator: "explicit test statement".into(),
                captured_at: 1,
            }],
            status: RecordStatus::Current,
            critical: false,
            applicability: Applicability::default(),
            valid_from: None,
            valid_until: None,
        }
    }
    fn archive_file(f: &Fixture, bytes: &[u8]) -> (File, String) {
        let path = f.root.join(format!(
            "archive-{}.jsonl",
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, bytes).unwrap();
        let hash = Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        (File::open(path).unwrap(), hash)
    }
    #[test]
    fn transfer_snapshot_preserves_history_without_live_replacement_or_confirmations() {
        let f = Fixture::new();
        let mut store = f.store();
        let saved_operation = op();
        let saved = store
            .create(&saved_operation, record("archived original"), None)
            .unwrap();
        drop(store);
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let mut archive = Vec::new();
        let stats = export_scope(&maintenance, f.scope(), &mut archive).unwrap();
        assert_eq!(stats.records, 1);
        assert!(
            !f.backup.exists(),
            "export must not create managed snapshot copies"
        );
        drop(maintenance);
        let mut store = f.store();
        store
            .revise(&op(), &saved.record_id, 1, record("current correction"))
            .unwrap();
        drop(store);
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let (mut file, hash) = archive_file(&f, &archive);
        let operation = op();
        let input = || ImportInput {
            operation_id: &operation,
            sha256: &hash,
            byte_len: archive.len() as u64,
        };
        let imported = import_snapshot(
            &maintenance,
            &f.backup,
            f.scope(),
            &mut file,
            input(),
            BackupPolicy::default(),
        )
        .unwrap();
        assert!(imported.eligible);
        let inspected = inspect_import(&f.root, &f.backup, f.scope(), input())
            .unwrap()
            .unwrap();
        assert_eq!(inspected.snapshot_id, imported.snapshot_id);
        let repeated = import_snapshot(
            &maintenance,
            &f.backup,
            f.scope(),
            &mut file,
            input(),
            BackupPolicy::default(),
        )
        .unwrap();
        assert_eq!(repeated.snapshot_id, imported.snapshot_id);
        {
            let catalog = Catalog::read(&f.root, &f.backup).unwrap();
            let count: i64 = catalog
                .connection
                .query_row("SELECT count(*) FROM snapshot_refs", [], |r| r.get(0))
                .unwrap();
            assert_eq!(
                count, 0,
                "manual import cannot fabricate daily/weekly points"
            );
            let saved = snapshot(&catalog.connection, &imported.snapshot_id).unwrap();
            let canonical = verify_snapshot(&saved, &f.backup, &f.scope()).unwrap();
            assert_eq!(
                canonical
                    .query_row("SELECT count(*) FROM mutations", [], |r| r.get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
        assert_eq!(
            f.store().read(&saved.record_id, None).unwrap().revision,
            2,
            "import must not activate its snapshot"
        );
        let restored = restore(
            &maintenance,
            &f.backup,
            f.scope(),
            &imported.snapshot_id,
            &op(),
            BackupPolicy::default(),
        )
        .unwrap();
        assert!(restored.complete);
        let mut live = f.store();
        assert_eq!(
            live.read(&saved.record_id, None).unwrap().data.content,
            "archived original"
        );
        assert!(
            live.read_mutation(&saved_operation).unwrap().is_none(),
            "external canonical data cannot manufacture operation proof"
        );
    }
    #[test]
    fn import_source_drift_and_impossible_reservation_leave_no_catalog() {
        let f = Fixture::new();
        drop(f.store());
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let mut bytes = Vec::new();
        export_scope(&maintenance, f.scope(), &mut bytes).unwrap();
        let (mut file, hash) = archive_file(&f, &bytes);
        let operation = op();
        let wrong = "0".repeat(64);
        assert!(
            import_snapshot(
                &maintenance,
                &f.backup,
                f.scope(),
                &mut file,
                ImportInput {
                    operation_id: &operation,
                    sha256: &wrong,
                    byte_len: bytes.len() as u64
                },
                BackupPolicy::default()
            )
            .is_err()
        );
        assert!(!f.backup.exists());
        assert!(
            import_snapshot(
                &maintenance,
                &f.backup,
                f.scope(),
                &mut file,
                ImportInput {
                    operation_id: &operation,
                    sha256: &hash,
                    byte_len: bytes.len() as u64
                },
                BackupPolicy {
                    max_bytes: META_HEADROOM
                }
            )
            .is_err()
        );
        assert!(!f.backup.exists());
        let path = f.root.join("drifting.jsonl");
        fs::write(&path, &bytes).unwrap();
        let mut changed = File::open(&path).unwrap();
        bytes[0] = b' ';
        fs::write(&path, &bytes).unwrap();
        assert!(
            import_snapshot(
                &maintenance,
                &f.backup,
                f.scope(),
                &mut changed,
                ImportInput {
                    operation_id: &operation,
                    sha256: &hash,
                    byte_len: bytes.len() as u64
                },
                BackupPolicy::default()
            )
            .is_err()
        );
        assert!(!f.backup.exists());
    }
    #[test]
    fn incomplete_import_is_bound_accounted_and_only_exact_cleanup_releases_it() {
        let f = Fixture::new();
        drop(f.store());
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let malformed = b"{\"broken\":true}\n";
        let (mut file, hash) = archive_file(&f, malformed);
        let operation = op();
        let input = || ImportInput {
            operation_id: &operation,
            sha256: &hash,
            byte_len: malformed.len() as u64,
        };
        // The durable checkpoint after stage creation, before codec work. An
        // abrupt process exit here cannot run definite-failure cleanup.
        let id = snapshot_id(&operation).unwrap();
        let key = memory_lifecycle::scope_key(&f.scope()).unwrap();
        let (epoch, _) = memory_lifecycle::epoch_and_exclusions(&f.root, &f.scope()).unwrap();
        {
            let catalog = Catalog::open(&f.root, &f.backup, true).unwrap();
            let path = f.backup.join(format!("{id}.sqlite3"));
            let staged = private_file(&path, true).unwrap();
            staged.sync_all().unwrap();
            let stamp = file_stamp(&staged.metadata().unwrap()).unwrap();
            catalog.connection.execute("INSERT INTO snapshots(id,scope_key,scope_json,epoch,phase,byte_len,reserved_bytes,sha256,file_stamp,created_at,protected,input_sha256,input_bytes) VALUES(?1,?2,?3,?4,'copying',0,4000000,'',?5,1,0,?6,?7)", params![id,key,serde_json::to_string(&f.scope()).unwrap(),epoch as i64,stamp,hash,malformed.len() as i64]).unwrap();
            sync_directory(&f.backup).unwrap();
        }
        let pending = inspect_import(&f.root, &f.backup, f.scope(), input())
            .unwrap()
            .unwrap();
        assert!(!pending.eligible);
        assert_eq!(pending.phase, "copying");
        let before = fs::read(f.backup.join(format!("{}.sqlite3", pending.snapshot_id))).unwrap();
        assert!(
            import_snapshot(
                &maintenance,
                &f.backup,
                f.scope(),
                &mut file,
                input(),
                BackupPolicy::default()
            )
            .is_err()
        );
        assert_eq!(
            fs::read(f.backup.join(format!("{}.sqlite3", pending.snapshot_id))).unwrap(),
            before
        );
        let mut valid = Vec::new();
        export_scope(&maintenance, f.scope(), &mut valid).unwrap();
        let (mut other, other_hash) = archive_file(&f, &valid);
        assert!(matches!(
            import_snapshot(
                &maintenance,
                &f.backup,
                f.scope(),
                &mut other,
                ImportInput {
                    operation_id: &operation,
                    sha256: &other_hash,
                    byte_len: valid.len() as u64
                },
                BackupPolicy::default()
            ),
            Err(MemoryError::OperationConflict)
        ));
        {
            let catalog = Catalog::read(&f.root, &f.backup).unwrap();
            let saved = snapshot(&catalog.connection, &pending.snapshot_id).unwrap();
            assert!(saved.reserved > 1024 * 1024);
            assert!(catalog.usage().unwrap() >= saved.reserved as u64);
        }
        assert!(
            reconcile_backup(&f.root, &f.backup, f.scope(), &operation)
                .unwrap()
                .is_none()
        );
        assert!(
            inspect_import(&f.root, &f.backup, f.scope(), input())
                .unwrap()
                .is_none()
        );
        assert!(
            !f.backup
                .join(format!("{}.sqlite3", pending.snapshot_id))
                .exists()
        );
    }
    #[test]
    fn forgotten_archive_cannot_become_an_eligible_import() {
        let f = Fixture::new();
        let mut store = f.store();
        let forgotten = store
            .create(&op(), record("forget this exported lineage"), None)
            .unwrap();
        let survivor = store
            .create(&op(), record("preserve unrelated live content"), None)
            .unwrap();
        drop(store);
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let mut bytes = Vec::new();
        export_scope(&maintenance, f.scope(), &mut bytes).unwrap();
        drop(maintenance);
        memory_lifecycle::forget(&f.root, f.scope(), &forgotten.record_id, &op(), 1).unwrap();
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let (mut file, hash) = archive_file(&f, &bytes);
        let operation = op();
        let input = || ImportInput {
            operation_id: &operation,
            sha256: &hash,
            byte_len: bytes.len() as u64,
        };
        assert!(
            import_snapshot(
                &maintenance,
                &f.backup,
                f.scope(),
                &mut file,
                input(),
                BackupPolicy::default()
            )
            .is_err()
        );
        assert!(
            inspect_import(&f.root, &f.backup, f.scope(), input())
                .unwrap()
                .is_none()
        );
        assert!(
            !f.backup
                .join(format!("{}.sqlite3", snapshot_id(&operation).unwrap()))
                .exists()
        );
        for entry in fs::read_dir(&f.backup).unwrap() {
            let bytes = fs::read(entry.unwrap().path()).unwrap();
            assert!(
                !bytes
                    .windows(b"forget this exported lineage".len())
                    .any(|part| part == b"forget this exported lineage"),
                "rejected import must not retain managed forgotten plaintext"
            );
        }
        assert!(f.store().read(&forgotten.record_id, None).is_err());
        assert_eq!(
            f.store()
                .read(&survivor.record_id, None)
                .unwrap()
                .data
                .content,
            "preserve unrelated live content"
        );
        reconcile_backup(&f.root, &f.backup, f.scope(), &operation).unwrap();
    }
    #[test]
    fn imported_source_with_wrong_scope_remains_ineligible_and_preserves_live_scope() {
        let f = Fixture::new();
        let mut live = f.store();
        live.create(&op(), record("selected project remains intact"), None)
            .unwrap();
        drop(live);
        let other = Scope::Project("other-project".into());
        let mut other_store = MemoryStore::open(&f.root, other.clone()).unwrap();
        other_store
            .create(&op(), record("other project's archive"), None)
            .unwrap();
        drop(other_store);
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let mut bytes = Vec::new();
        export_scope(&maintenance, other, &mut bytes).unwrap();
        let (mut file, hash) = archive_file(&f, &bytes);
        let operation = op();
        let failure = import_snapshot(
            &maintenance,
            &f.backup,
            f.scope(),
            &mut file,
            ImportInput {
                operation_id: &operation,
                sha256: &hash,
                byte_len: bytes.len() as u64,
            },
            BackupPolicy::default(),
        )
        .unwrap_err();
        assert!(failure.to_string().contains("scope"), "{failure}");
        assert!(
            inspect_backups(&f.root, &f.backup, f.scope())
                .unwrap()
                .iter()
                .all(|saved| !saved.eligible)
        );
    }

    #[test]
    fn daily_and_weekly_share_bytes_and_retention_is_bounded() {
        let f = Fixture::new();
        let mut store = f.store();
        store
            .create(&op(), record("retained recovery content"), None)
            .unwrap();
        drop(store);
        let daily = f.capture_at(f.scope(), 86400, BackupKind::Daily, BackupPolicy::default());
        let weekly = f.capture_at(
            f.scope(),
            86400,
            BackupKind::Weekly,
            BackupPolicy::default(),
        );
        assert_eq!(daily.snapshot_id, weekly.snapshot_id);
        for day in 2..=40 {
            f.capture_at(
                f.scope(),
                day * 86400,
                BackupKind::Daily,
                BackupPolicy::default(),
            );
        }
        let catalog = Catalog::open(&f.root, &f.backup, false).unwrap();
        let daily: i64 = catalog
            .connection
            .query_row(
                "SELECT count(*) FROM snapshot_refs WHERE kind='daily'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let weekly: i64 = catalog
            .connection
            .query_row(
                "SELECT count(*) FROM snapshot_refs WHERE kind='weekly'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(daily, 7);
        assert_eq!(weekly, 4);
        assert!(snapshots(&catalog.connection).unwrap().len() <= 11);
        assert!(catalog.usage().unwrap() <= DEFAULT_BACKUP_BYTES);
    }
    #[test]
    fn forgetting_removes_affected_snapshots_and_preserves_other_scope() {
        let f = Fixture::new();
        let mut store = f.store();
        let target = store
            .create(&op(), record("forget from managed snapshots"), None)
            .unwrap();
        drop(store);
        let saved = f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        let other = Scope::Project("other-backup-project".into());
        let mut store = MemoryStore::open(&f.root, other.clone()).unwrap();
        store
            .create(&op(), record("other scope retained"), None)
            .unwrap();
        drop(store);
        let unrelated = f.capture_at(other.clone(), 1, BackupKind::Daily, BackupPolicy::default());
        let unrelated_path = f.backup.join(format!("{}.sqlite3", unrelated.snapshot_id));
        let unchanged = fs::read(&unrelated_path).unwrap();
        assert!(
            memory_lifecycle::forget(&f.root, f.scope(), &target.record_id, &op(), 1)
                .unwrap()
                .cleanup_complete
        );
        assert!(
            !f.backup
                .join(format!("{}.sqlite3", saved.snapshot_id))
                .exists()
        );
        assert_eq!(fs::read(unrelated_path).unwrap(), unchanged);
        assert!(inspect_backups(&f.root, &f.backup, other).unwrap()[0].eligible);
    }
    #[test]
    fn missing_bound_backup_root_keeps_forgetting_cleanup_pending() {
        let f = Fixture::new();
        let mut store = f.store();
        let target = store
            .create(&op(), record("must not hide old backups"), None)
            .unwrap();
        drop(store);
        f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        let moved = f.root.join("moved-backups");
        fs::rename(&f.backup, &moved).unwrap();
        assert!(memory_lifecycle::forget(&f.root, f.scope(), &target.record_id, &op(), 1).is_err());
        assert!(
            !memory_lifecycle::inspect(&f.root, f.scope(), &target.record_id)
                .unwrap()
                .cleanup_complete
        );
        fs::rename(moved, &f.backup).unwrap();
        assert!(
            memory_lifecycle::reconcile(&f.root, f.scope(), &target.record_id)
                .unwrap()
                .cleanup_complete
        );
    }
    #[test]
    fn restore_preserves_current_first_and_does_not_change_other_scope() {
        let f = Fixture::new();
        let mut store = f.store();
        let old = store
            .create(&op(), record("old captured state"), None)
            .unwrap();
        store.lifecycle_connection().execute("INSERT INTO semantic_revisions(revision_row_id,model_identity,source_sha256,input_bytes,content_tokens,chunk_count) SELECT row_id,'test-model',?1,length(content),3,1 FROM revisions WHERE record_id=?2", params!["0".repeat(64),old.record_id]).unwrap();
        store.lifecycle_connection().execute("INSERT INTO semantic_chunks(revision_row_id,ordinal,start_byte,end_byte,input_tokens,vector) SELECT row_id,0,0,length(content),3,zeroblob(1536) FROM revisions WHERE record_id=?1", [&old.record_id]).unwrap();
        drop(store);
        let saved = f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        let mut store = f.store();
        store
            .revise(&op(), &old.record_id, 1, record("current before restore"))
            .unwrap();
        drop(store);
        let other = Scope::Project("other-restore-project".into());
        let mut store = MemoryStore::open(&f.root, other.clone()).unwrap();
        let untouched = store
            .create(&op(), record("unchanged other scope"), None)
            .unwrap();
        drop(store);
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let operation = op();
        let result = restore(
            &maintenance,
            &f.backup,
            f.scope(),
            &saved.snapshot_id,
            &operation,
            BackupPolicy::default(),
        )
        .unwrap();
        assert!(result.complete);
        drop(maintenance);
        let mut live = f.store();
        let derived: i64 = live.lifecycle_connection().query_row("SELECT (SELECT count(*) FROM semantic_revisions)+(SELECT count(*) FROM semantic_chunks)", [], |r|r.get(0)).unwrap();
        assert_eq!(derived, 0);
        drop(live);
        assert_eq!(
            f.store().read(&old.record_id, None).unwrap().data.content,
            "old captured state"
        );
        assert_eq!(
            MemoryStore::open_existing(&f.root, other)
                .unwrap()
                .read(&untouched.record_id, None)
                .unwrap()
                .data
                .content,
            "unchanged other scope"
        );
        let catalog = Catalog::open(&f.root, &f.backup, false).unwrap();
        let preserved = snapshot(&catalog.connection, &result.preserved_snapshot_id).unwrap();
        let conn = verify_snapshot(&preserved, &catalog.root, &f.scope()).unwrap();
        let text: String = conn
            .query_row(
                "SELECT content FROM revisions WHERE record_id=?1 AND is_current=1",
                [old.record_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(text, "current before restore");
    }
    #[test]
    fn inadequate_preservation_budget_and_corrupt_input_leave_live_state_unchanged() {
        let f = Fixture::new();
        let mut store = f.store();
        let old = store.create(&op(), record("old state"), None).unwrap();
        drop(store);
        let saved = f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        let mut store = f.store();
        store
            .revise(&op(), &old.record_id, 1, record("must survive refusal"))
            .unwrap();
        let live_path = PathBuf::from(store.lifecycle_connection().path().unwrap());
        drop(store);
        let bytes = fs::read(&live_path).unwrap();
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        assert!(
            restore(
                &maintenance,
                &f.backup,
                f.scope(),
                &saved.snapshot_id,
                &op(),
                BackupPolicy {
                    max_bytes: META_HEADROOM
                }
            )
            .is_err()
        );
        assert_eq!(fs::read(&live_path).unwrap(), bytes);
        let path = f.backup.join(format!("{}.sqlite3", saved.snapshot_id));
        let mut content = fs::read(&path).unwrap();
        content[100] ^= 1;
        fs::write(&path, content).unwrap();
        assert!(
            restore(
                &maintenance,
                &f.backup,
                f.scope(),
                &saved.snapshot_id,
                &op(),
                BackupPolicy::default()
            )
            .is_err()
        );
        drop(maintenance);
        assert_eq!(
            f.store().read(&old.record_id, None).unwrap().data.content,
            "must survive refusal"
        );
    }
    #[test]
    fn pending_restore_before_or_after_copy_blocks_writes_and_reconciles_same_id() {
        for after_copy in [false, true] {
            let f = Fixture::new();
            let mut store = f.store();
            let target = store
                .create(&op(), record("original snapshot value"), None)
                .unwrap();
            drop(store);
            let saved = f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
            let mut store = f.store();
            store
                .revise(
                    &op(),
                    &target.record_id,
                    1,
                    record("recoverable current value"),
                )
                .unwrap();
            drop(store);
            let operation = op();
            let preserve_operation = format!(
                "op-preserve-{}",
                Sha256::digest(operation.as_bytes())
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            let preserved = snapshot_id(&preserve_operation).unwrap();
            {
                let access = ExclusiveScope::acquire(&f.root, &f.scope()).unwrap();
                let mut live =
                    MemoryStore::open_for_lifecycle(&f.root, f.scope(), &access).unwrap();
                memory_lifecycle::set_pending_restore(
                    &f.root,
                    &f.scope(),
                    &operation,
                    &saved.snapshot_id,
                )
                .unwrap();
                let catalog = Catalog::open(&f.root, &f.backup, false).unwrap();
                catalog.connection.execute("INSERT INTO restores(operation_id,scope_key,snapshot_id,preserved_snapshot_id,phase) VALUES(?1,?2,?3,?4,'preserving')",params![operation,memory_lifecycle::scope_key(&f.scope()).unwrap(),saved.snapshot_id,preserved]).unwrap();
                drop(catalog);
                backup_locked(
                    &f.root,
                    &f.backup,
                    &f.scope(),
                    &preserve_operation,
                    Capture {
                        kind: BackupKind::PreRestore,
                        at: 2,
                    },
                    BackupPolicy::default(),
                    live.lifecycle_connection(),
                )
                .unwrap();
                let catalog = Catalog::open(&f.root, &f.backup, false).unwrap();
                catalog
                    .connection
                    .execute(
                        "UPDATE restores SET phase='prepared' WHERE operation_id=?1",
                        [&operation],
                    )
                    .unwrap();
                if after_copy {
                    let source = verify_snapshot(
                        &snapshot(&catalog.connection, &saved.snapshot_id).unwrap(),
                        &catalog.root,
                        &f.scope(),
                    )
                    .unwrap();
                    copy_database(&source, live.lifecycle_connection()).unwrap();
                }
            }
            assert!(MemoryStore::open(&f.root, f.scope()).is_err());
            assert!(
                backup(
                    &f.root,
                    &f.backup,
                    f.scope(),
                    &op(),
                    BackupKind::Daily,
                    BackupPolicy::default()
                )
                .is_err()
            );
            let catalog_before = fs::read(f.backup.join("catalog.sqlite3")).unwrap();
            let status = inspect_restore(&f.root, &f.backup, f.scope(), &operation)
                .unwrap()
                .unwrap();
            assert_eq!(status.phase, "prepared");
            assert!(!status.complete);
            assert_eq!(
                fs::read(f.backup.join("catalog.sqlite3")).unwrap(),
                catalog_before
            );
            let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
            if !after_copy {
                let preserved_path = f.backup.join(format!("{preserved}.sqlite3"));
                let valid = fs::read(&preserved_path).unwrap();
                let mut corrupt = valid.clone();
                corrupt[100] ^= 1;
                fs::write(&preserved_path, corrupt).unwrap();
                let live_path = f.root.join("memory").join(format!(
                    "scope-{}.sqlite3",
                    memory_lifecycle::scope_key(&f.scope()).unwrap()
                ));
                let live_before = fs::read(&live_path).unwrap();
                assert!(
                    restore(
                        &maintenance,
                        &f.backup,
                        f.scope(),
                        &saved.snapshot_id,
                        &operation,
                        BackupPolicy::default()
                    )
                    .is_err()
                );
                assert_eq!(fs::read(live_path).unwrap(), live_before);
                fs::write(preserved_path, valid).unwrap();
            }
            let result = restore(
                &maintenance,
                &f.backup,
                f.scope(),
                &saved.snapshot_id,
                &operation,
                BackupPolicy::default(),
            )
            .unwrap();
            assert!(result.complete);
            drop(maintenance);
            let mut live = f.store();
            assert_eq!(
                live.read(&target.record_id, None).unwrap().data.content,
                "original snapshot value"
            );
            live.revise(&op(), &target.record_id, 1, record("new post-restore edit"))
                .unwrap();
            drop(live);
            let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
            restore(
                &maintenance,
                &f.backup,
                f.scope(),
                &saved.snapshot_id,
                &operation,
                BackupPolicy::default(),
            )
            .unwrap();
            drop(maintenance);
            assert_eq!(
                f.store()
                    .read(&target.record_id, None)
                    .unwrap()
                    .data
                    .content,
                "new post-restore edit"
            );
        }
    }

    #[test]
    fn capacity_preserves_latest_and_unknown_entries_or_second_roots_are_refused() {
        let f = Fixture::new();
        let mut store = f.store();
        store
            .create(&op(), record("latest protected evidence"), None)
            .unwrap();
        drop(store);
        let saved = f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        let path = f.backup.join(format!("{}.sqlite3", saved.snapshot_id));
        let protected = fs::read(&path).unwrap();
        assert!(
            backup(
                &f.root,
                &f.backup,
                f.scope(),
                &op(),
                BackupKind::PreMigration,
                BackupPolicy {
                    max_bytes: META_HEADROOM
                }
            )
            .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), protected);
        let second = f.root.join("second-backups");
        assert!(
            backup(
                &f.root,
                &second,
                f.scope(),
                &op(),
                BackupKind::Daily,
                BackupPolicy::default()
            )
            .is_err()
        );
        assert!(!second.exists());
        let unknown = f.backup.join("unowned.txt");
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        use std::io::Write;
        options
            .open(&unknown)
            .unwrap()
            .write_all(b"unrelated bytes")
            .unwrap();
        assert!(
            backup(
                &f.root,
                &f.backup,
                f.scope(),
                &op(),
                BackupKind::Daily,
                BackupPolicy::default()
            )
            .is_err()
        );
        assert_eq!(fs::read(unknown).unwrap(), b"unrelated bytes");
        assert_eq!(fs::read(path).unwrap(), protected);
    }

    #[test]
    fn forgetting_revalidates_unaffected_older_snapshot_in_same_scope() {
        let f = Fixture::new();
        let mut live = f.store();
        let kept = live
            .create(&op(), record("unrelated retained fact"), None)
            .unwrap();
        drop(live);
        let before = f.capture_at(f.scope(), 86400, BackupKind::Daily, BackupPolicy::default());
        let mut live = f.store();
        let forgotten = live
            .create(&op(), record("later forgotten content"), None)
            .unwrap();
        drop(live);
        let affected = f.capture_at(
            f.scope(),
            2 * 86400,
            BackupKind::Daily,
            BackupPolicy::default(),
        );
        let result =
            memory_lifecycle::forget(&f.root, f.scope(), &forgotten.record_id, &op(), 1).unwrap();
        assert!(result.cleanup_complete);
        let remaining = inspect_backups(&f.root, &f.backup, f.scope()).unwrap();
        assert!(
            remaining
                .iter()
                .any(|s| s.snapshot_id == before.snapshot_id && s.eligible)
        );
        assert!(
            !remaining
                .iter()
                .any(|s| s.snapshot_id == affected.snapshot_id)
        );
        assert!(
            !f.backup
                .join(format!("{}.sqlite3", affected.snapshot_id))
                .exists()
        );
        assert_eq!(
            f.store().read(&kept.record_id, None).unwrap().data.content,
            "unrelated retained fact"
        );
    }

    #[test]
    fn corrupt_latest_cannot_evict_older_verified_recovery() {
        let f = Fixture::new();
        f.store()
            .create(&op(), record("good recovery"), None)
            .unwrap();
        let older = f.capture_at(f.scope(), 86400, BackupKind::Daily, BackupPolicy::default());
        let latest = f.capture_at(
            f.scope(),
            2 * 86400,
            BackupKind::Daily,
            BackupPolicy::default(),
        );
        let latest_path = f.backup.join(format!("{}.sqlite3", latest.snapshot_id));
        let mut bytes = fs::read(&latest_path).unwrap();
        bytes[100] ^= 1;
        fs::write(latest_path, bytes).unwrap();
        let older_path = f.backup.join(format!("{}.sqlite3", older.snapshot_id));
        let original = fs::read(&older_path).unwrap();
        assert!(
            backup(
                &f.root,
                &f.backup,
                f.scope(),
                &op(),
                BackupKind::PreMigration,
                BackupPolicy {
                    max_bytes: META_HEADROOM
                }
            )
            .is_err()
        );
        assert_eq!(fs::read(older_path).unwrap(), original);
    }

    #[test]
    fn incomplete_reservations_count_and_reconcile_only_owned_stage() {
        let f = Fixture::new();
        f.store()
            .create(&op(), record("canonical preserved"), None)
            .unwrap();
        f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        let operation = op();
        let id = snapshot_id(&operation).unwrap();
        let mut catalog = Catalog::open(&f.root, &f.backup, false).unwrap();
        let baseline = catalog.usage().unwrap();
        catalog.connection.execute("INSERT INTO snapshots(id,scope_key,scope_json,epoch,phase,byte_len,reserved_bytes,sha256,file_stamp,created_at,protected) VALUES(?1,?2,?3,0,'copying',0,?4,'','',2,0)", params![id,memory_lifecycle::scope_key(&f.scope()).unwrap(),serde_json::to_string(&f.scope()).unwrap(), 1024 * 1024]).unwrap();
        let path = f.backup.join(format!("{id}.sqlite3"));
        let file = private_file(&path, true).unwrap();
        file.sync_all().unwrap();
        let stamp = file_stamp(&file.metadata().unwrap()).unwrap();
        drop(file);
        assert!(catalog.usage().unwrap() >= baseline + 1024 * 1024);
        assert!(
            catalog
                .make_room(1024 * 1024, baseline + 1024 * 1024)
                .is_err()
        );
        drop(catalog);
        // The create-before-identity crash gap must preserve uncertain ownership.
        assert!(reconcile_backup(&f.root, &f.backup, f.scope(), &operation).is_err());
        assert!(path.exists());
        let catalog = Catalog::open(&f.root, &f.backup, false).unwrap();
        catalog
            .connection
            .execute(
                "UPDATE snapshots SET file_stamp=?2 WHERE id=?1",
                params![id, stamp],
            )
            .unwrap();
        drop(catalog);
        assert!(
            reconcile_backup(&f.root, &f.backup, f.scope(), &operation)
                .unwrap()
                .is_none()
        );
        assert!(!path.exists());
        let catalog = Catalog::read(&f.root, &f.backup).unwrap();
        assert!(catalog.usage().unwrap() < baseline + 1024 * 1024);
    }

    #[test]
    fn replaced_bound_root_does_not_hide_backed_content() {
        let f = Fixture::new();
        let target = f
            .store()
            .create(&op(), record("must remain accounted"), None)
            .unwrap();
        f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        let moved = f.root.join("old-backups");
        fs::rename(&f.backup, &moved).unwrap();
        fs::create_dir(&f.backup).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&f.backup, fs::Permissions::from_mode(0o700)).unwrap();
        }
        // Even a copied valid catalog cannot substitute for the bound directory.
        for entry in fs::read_dir(&moved).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), f.backup.join(entry.file_name())).unwrap();
        }
        assert!(memory_lifecycle::forget(&f.root, f.scope(), &target.record_id, &op(), 1).is_err());
        assert!(
            !memory_lifecycle::inspect(&f.root, f.scope(), &target.record_id)
                .unwrap()
                .cleanup_complete
        );
        fs::remove_dir_all(&f.backup).unwrap();
        fs::rename(moved, &f.backup).unwrap();
        assert!(
            memory_lifecycle::reconcile(&f.root, f.scope(), &target.record_id)
                .unwrap()
                .cleanup_complete
        );
    }

    #[test]
    fn active_migration_point_is_retained_until_explicit_maintenance_release() {
        let f = Fixture::new();
        f.store()
            .create(&op(), record("migration recovery evidence"), None)
            .unwrap();
        let migration = f.capture_at(
            f.scope(),
            1,
            BackupKind::PreMigration,
            BackupPolicy::default(),
        );
        f.capture_at(f.scope(), 86400, BackupKind::Daily, BackupPolicy::default());
        let path = f.backup.join(format!("{}.sqlite3", migration.snapshot_id));
        assert!(path.exists());
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        let released = release_migration_protection(
            &maintenance,
            &f.backup,
            f.scope(),
            &migration.snapshot_id,
        )
        .unwrap();
        assert!(!released.protected);
        assert!(path.exists());
        drop(maintenance);
        f.capture_at(
            f.scope(),
            2 * 86400,
            BackupKind::Daily,
            BackupPolicy::default(),
        );
        assert!(!path.exists());
    }

    #[test]
    fn authority_before_catalog_restore_is_inspectable_and_reconciles_exact_id() {
        let f = Fixture::new();
        let target = f
            .store()
            .create(&op(), record("selected historical value"), None)
            .unwrap();
        let saved = f.capture_at(f.scope(), 1, BackupKind::Daily, BackupPolicy::default());
        f.store()
            .revise(
                &op(),
                &target.record_id,
                1,
                record("current value to preserve"),
            )
            .unwrap();
        let operation = op();
        memory_lifecycle::set_pending_restore(&f.root, &f.scope(), &operation, &saved.snapshot_id)
            .unwrap();
        let authority_path = f.root.join("memory-lifecycle/authority.sqlite3");
        let catalog_path = f.backup.join("catalog.sqlite3");
        let authority_bytes = fs::read(&authority_path).unwrap();
        let catalog_bytes = fs::read(&catalog_path).unwrap();
        let inspected = inspect_restore(&f.root, &f.backup, f.scope(), &operation)
            .unwrap()
            .unwrap();
        assert_eq!(inspected.phase, "preparing");
        assert_eq!(inspected.snapshot_id, saved.snapshot_id);
        assert!(!inspected.complete);
        assert!(
            inspect_restore(&f.root, &f.backup, f.scope(), &op())
                .unwrap()
                .is_none()
        );
        assert_eq!(fs::read(&authority_path).unwrap(), authority_bytes);
        assert_eq!(fs::read(&catalog_path).unwrap(), catalog_bytes);
        assert!(MemoryStore::open_existing(&f.root, f.scope()).is_err());
        let maintenance = MaintenanceGuard::try_acquire(&f.root).unwrap();
        assert!(matches!(
            reconcile_restore(
                &maintenance,
                &f.backup,
                f.scope(),
                &op(),
                BackupPolicy::default()
            ),
            Err(MemoryError::NotFound)
        ));
        let result = reconcile_restore(
            &maintenance,
            &f.backup,
            f.scope(),
            &operation,
            BackupPolicy::default(),
        )
        .unwrap();
        assert!(result.complete);
        drop(maintenance);
        assert_eq!(
            f.store()
                .read(&target.record_id, None)
                .unwrap()
                .data
                .content,
            "selected historical value"
        );
        let catalog = Catalog::read(&f.root, &f.backup).unwrap();
        let preserved = snapshot(&catalog.connection, &result.preserved_snapshot_id).unwrap();
        let source = verify_snapshot(&preserved, &catalog.root, &f.scope()).unwrap();
        let text: String = source
            .query_row(
                "SELECT content FROM revisions WHERE record_id=?1 AND is_current=1",
                [&target.record_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(text, "current value to preserve");
    }
}
