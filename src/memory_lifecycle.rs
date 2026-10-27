//! Content-free forgetting authority outside replaceable per-scope databases.
//!
//! Lock order is the caller's runtime maintenance/session guard, scoped access,
//! then short SQLite/operation-store locks. Scoped lock files are permanent.
//! A committed exclusion survives interrupted cleanup and an older scope restore.
//! Managed snapshot cleanup cooperates with `memory_backup`; exclusions and
//! restore-in-progress authority remain outside each replaceable scope.
//! External exports and an installation missing this authority cannot be erased
//! or recognized retrospectively. File deletion is not forensic media erasure.

use crate::file_lock::FileLock;
use crate::memory::{MemoryError, MemoryStore, Scope};
use crate::operation_store::OperationStore;
use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, Metadata, OpenOptions, TryLockError},
    io,
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const APPLICATION_ID: i64 = 0x4d4c4946;
type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ForgetStatus {
    pub scope: Scope,
    pub record_id: String,
    pub logically_forgotten: bool,
    pub cleanup_complete: bool,
    pub pending_operation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ForgetOperationStatus {
    pub operation_id: String,
    pub expected_revision: u64,
    pub status: ForgetStatus,
}

pub(crate) struct ScopeAccess {
    _file: FileLock,
    root: PathBuf,
    scope_key: String,
}
pub(crate) struct ExclusiveScope(ScopeAccess);

pub(crate) fn scope_key(scope: &Scope) -> Result<String> {
    Ok(Sha256::digest(serde_json::to_vec(scope)?)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
impl ScopeAccess {
    pub(crate) fn shared(root: &Path, scope: &Scope, create: bool) -> Result<Self> {
        match access(root, scope, create, false) {
            Err(MemoryError::Io(e)) if !create && e.kind() == io::ErrorKind::NotFound => {
                let canonical = root
                    .join("memory")
                    .join(format!("scope-{}.sqlite3", scope_key(scope)?));
                match fs::symlink_metadata(canonical) {
                    Ok(_) => Err(MemoryError::InvalidInput(
                        "canonical memory exists but lifecycle authority or its permanent scope lock is missing; explicit recovery is required",
                    )),
                    Err(missing) if missing.kind() == io::ErrorKind::NotFound => {
                        Err(MemoryError::NotFound)
                    }
                    Err(other) => Err(other.into()),
                }
            }
            other => other,
        }
    }
    /// Every normal open checks current exclusions while the shared lock prevents
    /// new exclusions. Old imported canonical rows fail closed until reconciled.
    pub(crate) fn check_canonical(&self, canonical: &Connection) -> Result<()> {
        let authority = open_authority(&self.root, false, false)?;
        let mut statement =
            authority.prepare("SELECT record_id FROM exclusions WHERE scope_key=?1")?;
        for record in statement.query_map([&self.scope_key], |r| r.get::<_, String>(0))? {
            let record = record?;
            let present:bool=canonical.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE record_id=?1 UNION ALL SELECT 1 FROM revisions WHERE record_id=?1 UNION ALL SELECT 1 FROM mutations WHERE record_id=?1 AND intent_hash<>'')",[record],|r|r.get(0))?;
            if present {
                return Err(MemoryError::InvalidInput(
                    "forgotten lineage exists in canonical memory; explicit forgetting reconciliation is required",
                ));
            }
        }
        Ok(())
    }
}
impl ExclusiveScope {
    pub(crate) fn acquire(root: &Path, scope: &Scope) -> Result<Self> {
        Ok(Self(access(root, scope, false, true)?))
    }
    pub(crate) fn validates(&self, root: &Path, scope: &Scope) -> Result<bool> {
        Ok(root.canonicalize()? == self.0.root && scope_key(scope)? == self.0.scope_key)
    }
}
fn access(root: &Path, scope: &Scope, create: bool, exclusive: bool) -> Result<ScopeAccess> {
    crate::memory::validate_scope(scope)?;
    let root = validate_root(root)?;
    let directory = root.join("memory-lifecycle");
    let key = scope_key(scope)?;
    if create
        && root
            .join("memory")
            .join(format!("scope-{key}.sqlite3"))
            .try_exists()?
        && !directory.join("authority.sqlite3").try_exists()?
    {
        return Err(MemoryError::InvalidInput(
            "existing canonical memory has no forgetting authority; explicit recovery is required",
        ));
    }
    private_directory(&directory, create)?;
    let path = directory.join(format!("scope-{key}.lock"));
    let file = open_private(&path, create)?;
    if file.metadata()?.len() != 0 {
        return Err(MemoryError::InvalidInput("scope lock must be empty"));
    }
    let result = if exclusive {
        file.try_lock()
    } else {
        file.try_lock_shared()
    };
    match result {
        Ok(()) => (),
        Err(TryLockError::WouldBlock) => {
            return Err(MemoryError::InvalidInput(
                "memory scope is busy; retry after its current readers or lifecycle operation finish",
            ));
        }
        Err(TryLockError::Error(e)) => return Err(e.into()),
    }
    let file = FileLock::acquired(file);
    same_open_file(&path, file.as_file())?;
    // Initialize only through a writable selection. Read-only open creates nothing.
    open_authority(&root, create, create)?;
    Ok(ScopeAccess {
        _file: file,
        root,
        scope_key: key,
    })
}

/// Caller validates the issued operation's scope/target and expiry before entry.
/// On response loss, inspect/reconcile this same selected target; never infer a
/// fresh operation from an absent ordinary operation receipt.
pub fn forget(
    state_root: &Path,
    scope: Scope,
    record_id: &str,
    operation_id: &str,
    expected_revision: u64,
) -> Result<ForgetStatus> {
    validate_target(record_id, operation_id)?;
    crate::memory::validate_reference(&crate::memory::RecordRef {
        record_id: record_id.into(),
        revision: expected_revision,
    })?;
    let access = ExclusiveScope::acquire(state_root, &scope)?;
    let mut store = MemoryStore::open_for_lifecycle(state_root, scope.clone(), &access)?;
    let mut authority = open_authority(&access.0.root, false, true)?;
    begin_forget(
        &mut authority,
        store.lifecycle_connection(),
        &access.0.scope_key,
        record_id,
        operation_id,
        expected_revision,
    )?;
    finish_forget(
        &access,
        &scope,
        record_id,
        &mut authority,
        store.lifecycle_connection(),
    )
}

/// Reconcile committed exclusions only. This cannot create a forgetting intent.
pub fn reconcile(state_root: &Path, scope: Scope, record_id: &str) -> Result<ForgetStatus> {
    crate::memory::validate_reference(&crate::memory::RecordRef {
        record_id: record_id.into(),
        revision: 1,
    })?;
    let access = ExclusiveScope::acquire(state_root, &scope)?;
    let mut authority = open_authority(&access.0.root, false, true)?;
    if status(&authority, &scope, &access.0.scope_key, record_id)?.logically_forgotten {
        let mut store = MemoryStore::open_for_lifecycle(state_root, scope.clone(), &access)?;
        // A known older import may contain additional consumed mutation IDs.
        retain_cleanup_receipts(
            &mut authority,
            store.lifecycle_connection(),
            &access.0.scope_key,
            record_id,
        )?;
        finish_forget(
            &access,
            &scope,
            record_id,
            &mut authority,
            store.lifecycle_connection(),
        )
    } else {
        Err(MemoryError::NotFound)
    }
}

pub fn inspect(state_root: &Path, scope: Scope, record_id: &str) -> Result<ForgetStatus> {
    crate::memory::validate_reference(&crate::memory::RecordRef {
        record_id: record_id.into(),
        revision: 1,
    })?;
    let access = ScopeAccess::shared(state_root, &scope, false)?;
    status(
        &open_authority(&access.root, false, false)?,
        &scope,
        &access.scope_key,
        record_id,
    )
}
/// Exact issued-operation inspection: another operation forgetting the same
/// target does not prove this operation committed. Missing state stays absent.
pub fn inspect_operation(
    state_root: &Path,
    scope: Scope,
    record_id: &str,
    operation_id: &str,
) -> Result<Option<ForgetOperationStatus>> {
    validate_target(record_id, operation_id)?;
    let access = match ScopeAccess::shared(state_root, &scope, false) {
        Err(MemoryError::NotFound) => return Ok(None),
        other => other?,
    };
    let authority = open_authority(&access.root, false, false)?;
    let bound: Option<(String, String, u64)> = authority
        .query_row(
            "SELECT scope_key,record_id,expected_revision FROM forget_ops WHERE operation_id=?1",
            [operation_id],
            |r| Ok((r.get(0)?, r.get(1)?, unsigned_revision(r, 2)?)),
        )
        .optional()?;
    bound
        .map(|(key, record, revision)| {
            if key != access.scope_key || record != record_id {
                return Err(MemoryError::OperationConflict);
            }
            Ok(ForgetOperationStatus {
                operation_id: operation_id.into(),
                expected_revision: revision,
                status: status(&authority, &scope, &key, record_id)?,
            })
        })
        .transpose()
}

fn status(authority: &Connection, scope: &Scope, key: &str, record: &str) -> Result<ForgetStatus> {
    let value:Option<(bool,Option<String>)>=authority.query_row("SELECT cleanup_complete,operation_id FROM exclusions WHERE scope_key=?1 AND record_id=?2",params![key,record],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    Ok(ForgetStatus {
        scope: scope.clone(),
        record_id: record.into(),
        logically_forgotten: value.is_some(),
        cleanup_complete: value.as_ref().is_some_and(|x| x.0),
        pending_operation_id: value.and_then(|x| x.1),
    })
}
fn validate_target(record: &str, operation: &str) -> Result<()> {
    crate::memory::validate_reference(&crate::memory::RecordRef {
        record_id: record.into(),
        revision: 1,
    })?;
    if !operation.starts_with("op-")
        || operation.len() < 4
        || operation.len() > 100
        || !operation
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(MemoryError::InvalidInput(
            "forget requires a bounded server-issued operation identity",
        ));
    }
    Ok(())
}
fn unsigned_revision(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
pub(crate) fn epoch_and_exclusions(root: &Path, scope: &Scope) -> Result<(u64, Vec<String>)> {
    let authority = open_authority(&validate_root(root)?, false, false)?;
    let key = scope_key(scope)?;
    let epoch: Option<i64> = authority
        .query_row(
            "SELECT epoch FROM scope_epochs WHERE scope_key=?1",
            [&key],
            |r| r.get(0),
        )
        .optional()?;
    let mut statement = authority
        .prepare("SELECT record_id FROM exclusions WHERE scope_key=?1 ORDER BY record_id")?;
    let ids = statement
        .query_map([key], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    Ok((
        u64::try_from(epoch.unwrap_or(0)).map_err(|_| MemoryError::UnsupportedSchema)?,
        ids,
    ))
}
pub(crate) fn check_current_exclusions(
    root: &Path,
    scope: &Scope,
    canonical: &Connection,
) -> Result<()> {
    let (_, ids) = epoch_and_exclusions(root, scope)?;
    for id in ids {
        let present:bool=canonical.query_row("SELECT EXISTS(SELECT 1 FROM records WHERE record_id=?1 UNION ALL SELECT 1 FROM revisions WHERE record_id=?1 UNION ALL SELECT 1 FROM mutations WHERE record_id=?1 AND intent_hash<>'')",[id],|r|r.get(0))?;
        if present {
            return Err(MemoryError::InvalidInput(
                "recovery input contains a forgotten lineage",
            ));
        }
    }
    Ok(())
}
pub(crate) fn configured_backup_root(root: &Path) -> Result<Option<PathBuf>> {
    let authority = open_authority(&validate_root(root)?, false, false)?;
    let value: Option<String> = authority
        .query_row(
            "SELECT value FROM lifecycle_config WHERE key='backup_root'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(value.map(PathBuf::from))
}
pub(crate) fn bind_backup_root(root: &Path, backup_root: &Path, create: bool) -> Result<PathBuf> {
    if !backup_root.is_absolute()
        || backup_root
            .components()
            .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(MemoryError::InvalidInput(
            "managed backup root must be explicit and absolute",
        ));
    }
    let root = validate_root(root)?;
    let mut authority = open_authority(&root, false, create)?;
    let selected: Option<String> = authority
        .query_row(
            "SELECT value FROM lifecycle_config WHERE key='backup_root'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(selected) = selected {
        private_directory(backup_root, false)?;
        let canonical = backup_root.canonicalize()?;
        if canonical != Path::new(&selected) {
            return Err(MemoryError::InvalidInput(
                "installation already has another managed backup root; an explicit managed move is required",
            ));
        }
        let stamp: String = authority.query_row(
            "SELECT value FROM lifecycle_config WHERE key='backup_root_identity'",
            [],
            |r| r.get(0),
        )?;
        if stamp != backup_root_identity(&canonical)? {
            return Err(MemoryError::InvalidInput(
                "bound backup root identity changed; coverage remains unknown",
            ));
        }
        return Ok(canonical);
    }
    if !create {
        return Err(MemoryError::NotFound);
    }
    private_directory(backup_root, true)?;
    let canonical = backup_root.canonicalize()?;
    if fs::read_dir(&canonical)?.next().is_some() {
        return Err(MemoryError::InvalidInput(
            "refusing to adopt a nonempty unknown backup root",
        ));
    }
    let tx = authority.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "INSERT INTO lifecycle_config(key,value) VALUES('backup_root',?1)",
        [canonical
            .to_str()
            .ok_or(MemoryError::InvalidInput("backup root must be UTF-8"))?],
    )?;
    tx.execute(
        "INSERT INTO lifecycle_config(key,value) VALUES('backup_root_identity',?1)",
        [backup_root_identity(&canonical)?],
    )?;
    tx.commit()?;
    Ok(canonical)
}

fn backup_root_identity(root: &Path) -> Result<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::metadata(root)?;
        let created = metadata
            .created()?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MemoryError::Clock)?;
        Ok(format!(
            "{}:{}:{}",
            metadata.dev(),
            metadata.ino(),
            created.as_nanos()
        ))
    }
    #[cfg(not(unix))]
    {
        let _ = root;
        Err(MemoryError::InvalidInput(
            "native backup root identity is not qualified on this platform",
        ))
    }
}

pub(crate) fn pending_restore(root: &Path, scope: &Scope) -> Result<Option<String>> {
    Ok(pending_restore_binding(root, scope)?.map(|(operation, _)| operation))
}
pub(crate) fn pending_restore_binding(
    root: &Path,
    scope: &Scope,
) -> Result<Option<(String, String)>> {
    let authority = open_authority(&validate_root(root)?, false, false)?;
    Ok(authority
        .query_row(
            "SELECT operation_id,snapshot_id FROM pending_restores WHERE scope_key=?1",
            [scope_key(scope)?],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?)
}
pub(crate) fn restore_operation_pending(root: &Path, operation: &str) -> Result<bool> {
    let authority = open_authority(&validate_root(root)?, false, false)?;
    Ok(authority.query_row(
        "SELECT EXISTS(SELECT 1 FROM pending_restores WHERE operation_id=?1)",
        [operation],
        |r| r.get(0),
    )?)
}
/// Exact-ID contradiction check for expired, never-started preparation retirement.
/// Missing/corrupt authority is an error; callers separately prove initial absence.
pub(crate) fn forget_operation_exists(root: &Path, operation: &str) -> Result<bool> {
    let authority = open_authority(&validate_root(root)?, false, false)?;
    Ok(authority.query_row(
        "SELECT EXISTS(SELECT 1 FROM forget_ops WHERE operation_id=?1)",
        [operation],
        |row| row.get(0),
    )?)
}
pub(crate) fn set_pending_restore(
    root: &Path,
    scope: &Scope,
    operation: &str,
    target: &str,
) -> Result<()> {
    let mut authority = open_authority(&validate_root(root)?, false, true)?;
    let key = scope_key(scope)?;
    let tx = authority.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let prior: Option<(String, String)> = tx
        .query_row(
            "SELECT operation_id,snapshot_id FROM pending_restores WHERE scope_key=?1",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((bound, bound_target)) = prior {
        if bound != operation || bound_target != target {
            return Err(MemoryError::OperationConflict);
        }
    } else {
        tx.execute(
            "INSERT INTO pending_restores(scope_key,operation_id,snapshot_id) VALUES(?1,?2,?3)",
            params![key, operation, target],
        )?;
    }
    tx.commit()?;
    Ok(())
}
pub(crate) fn clear_pending_restore(root: &Path, scope: &Scope, operation: &str) -> Result<()> {
    let authority = open_authority(&validate_root(root)?, false, true)?;
    authority.execute(
        "DELETE FROM pending_restores WHERE scope_key=?1 AND operation_id=?2",
        params![scope_key(scope)?, operation],
    )?;
    Ok(())
}

fn mutation_ids(canonical: &Connection, record: &str) -> Result<Vec<String>> {
    let mut statement = canonical
        .prepare("SELECT operation_id FROM mutations WHERE record_id=?1 ORDER BY operation_id")?;
    Ok(statement
        .query_map([record], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?)
}
fn begin_forget(
    authority: &mut Connection,
    canonical: &Connection,
    key: &str,
    record: &str,
    operation: &str,
    expected_revision: u64,
) -> Result<()> {
    let ids = mutation_ids(canonical, record)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MemoryError::Clock)?
        .as_secs();
    let now = i64::try_from(now).map_err(|_| MemoryError::Clock)?;
    let tx = authority.transaction_with_behavior(TransactionBehavior::Immediate)?;
    // Ordinary completed operation bindings are bounded. The caller's issued-ID
    // expiry rule remains authoritative after eviction; absence is never fresh.
    tx.execute(
        "DELETE FROM forget_ops WHERE cleanup_complete=1 AND created_at<?1",
        [now.saturating_sub(7 * 24 * 60 * 60)],
    )?;
    let prior: Option<(String, String, u64)> = tx
        .query_row(
            "SELECT scope_key,record_id,expected_revision FROM forget_ops WHERE operation_id=?1",
            [operation],
            |r| Ok((r.get(0)?, r.get(1)?, unsigned_revision(r, 2)?)),
        )
        .optional()?;
    if let Some((bound_scope, bound_record, bound_revision)) = prior {
        if bound_scope != key || bound_record != record || bound_revision != expected_revision {
            return Err(MemoryError::OperationConflict);
        }
    } else {
        let head: Option<u64> = canonical
            .query_row(
                "SELECT head_revision FROM records WHERE record_id=?1",
                [record],
                |r| unsigned_revision(r, 0),
            )
            .optional()?;
        if head != Some(expected_revision) {
            return Err(if head.is_none() {
                MemoryError::NotFound
            } else {
                MemoryError::Conflict
            });
        }
        let count: i64 = tx.query_row("SELECT count(*) FROM forget_ops", [], |r| r.get(0))?;
        if count >= 4096 {
            return Err(MemoryError::InvalidInput(
                "forget receipt capacity exhausted; reconcile pending cleanup before new forgetting",
            ));
        }
        tx.execute("INSERT INTO forget_ops(operation_id,scope_key,record_id,expected_revision,created_at,cleanup_complete) VALUES(?1,?2,?3,?4,?5,0)", params![operation,key,record,i64::try_from(expected_revision).map_err(|_| MemoryError::Conflict)?,now])?;
        tx.execute("INSERT INTO scope_epochs(scope_key,epoch) VALUES(?1,1) ON CONFLICT(scope_key) DO UPDATE SET epoch=epoch+1",[key])?;
    }
    let existing: Option<Option<String>> = tx
        .query_row(
            "SELECT operation_id FROM exclusions WHERE scope_key=?1 AND record_id=?2",
            params![key, record],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(Some(bound)) = existing
        && bound != operation
    {
        return Err(MemoryError::OperationConflict);
    }
    tx.execute("INSERT INTO exclusions(scope_key,record_id,cleanup_complete,operation_id) VALUES(?1,?2,0,?3) ON CONFLICT(scope_key,record_id) DO UPDATE SET cleanup_complete=0,operation_id=excluded.operation_id",params![key,record,operation])?;
    for id in ids {
        tx.execute("INSERT OR IGNORE INTO cleanup_receipts(scope_key,record_id,operation_id) VALUES(?1,?2,?3)",params![key,record,id])?;
    }
    tx.commit()?;
    Ok(())
}
fn retain_cleanup_receipts(
    authority: &mut Connection,
    canonical: &Connection,
    key: &str,
    record: &str,
) -> Result<()> {
    let ids = mutation_ids(canonical, record)?;
    let tx = authority.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "UPDATE exclusions SET cleanup_complete=0 WHERE scope_key=?1 AND record_id=?2",
        params![key, record],
    )?;
    for id in ids {
        tx.execute("INSERT OR IGNORE INTO cleanup_receipts(scope_key,record_id,operation_id) VALUES(?1,?2,?3)",params![key,record,id])?;
    }
    tx.commit()?;
    Ok(())
}
fn finish_forget(
    access: &ExclusiveScope,
    scope: &Scope,
    record: &str,
    authority: &mut Connection,
    canonical: &mut Connection,
) -> Result<ForgetStatus> {
    // A prepared revision/supersession may not yet have a canonical mutation.
    // The service pins shared scope access through receipt issuance, so this
    // exact-target inventory is stable under our exclusive scope guard.
    match OperationStore::open_existing(&access.0.root) {
        Ok(store) => {
            let locked = store.lock()?;
            let ids = locked.memory_target_receipts(scope, record)?;
            let tx = authority.transaction_with_behavior(TransactionBehavior::Immediate)?;
            for id in ids {
                let committed: Option<String> = canonical
                    .query_row(
                        "SELECT record_id FROM mutations WHERE operation_id=?1",
                        [&id],
                        |r| r.get(0),
                    )
                    .optional()?;
                // Completed cross-record supersession belongs to its new lineage.
                if committed.as_deref().is_some_and(|target| target != record) {
                    continue;
                }
                tx.execute("INSERT OR IGNORE INTO cleanup_receipts(scope_key,record_id,operation_id) VALUES(?1,?2,?3)",params![access.0.scope_key,record,id])?;
            }
            tx.commit()?;
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    let cleanup_ids: Vec<String> = {
        let mut statement = authority.prepare(
            "SELECT operation_id FROM cleanup_receipts WHERE scope_key=?1 AND record_id=?2",
        )?;
        statement
            .query_map(params![access.0.scope_key, record], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?
    };
    canonical.execute_batch("PRAGMA secure_delete=ON;")?;
    let tx = canonical.transaction_with_behavior(TransactionBehavior::Immediate)?;
    for id in &cleanup_ids {
        tx.execute("INSERT OR IGNORE INTO mutations(operation_id,intent_hash,record_id,revision) VALUES(?1,'',?2,1)", params![id,record])?;
    }
    tx.execute(
        "DELETE FROM record_fts WHERE rowid IN (SELECT row_id FROM revisions WHERE record_id=?1)",
        [record],
    )?;
    tx.execute("DELETE FROM sources WHERE revision_row_id IN (SELECT row_id FROM revisions WHERE record_id=?1)",[record])?;
    tx.execute("DELETE FROM revisions WHERE record_id=?1", [record])?;
    tx.execute("DELETE FROM records WHERE record_id=?1", [record])?;
    // IDs stay consumed, but neither content nor its derived intent hash remains.
    tx.execute(
        "UPDATE mutations SET intent_hash='' WHERE record_id=?1",
        [record],
    )?;
    // Rebuild derived segments from remaining FTS content; older retired-row
    // tombstones must not retain the forgotten record's indexed terms.
    tx.execute("INSERT INTO record_fts(record_fts) VALUES('rebuild')", [])?;
    tx.commit()?;
    match OperationStore::open_existing(&access.0.root) {
        Ok(store) => {
            let locked = store.lock()?;
            for id in &cleanup_ids {
                locked.remove_consumed(id)?;
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    crate::memory_admin::invalidate_export_stages(&access.0.root, scope)
        .map_err(|error| MemoryError::Io(std::io::Error::other(error)))?;
    crate::memory_backup::reconcile_forgetting(&access.0.root, scope)?;
    let tx = authority.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "DELETE FROM cleanup_receipts WHERE scope_key=?1 AND record_id=?2",
        params![access.0.scope_key, record],
    )?;
    tx.execute(
        "UPDATE forget_ops SET cleanup_complete=1 WHERE scope_key=?1 AND record_id=?2",
        params![access.0.scope_key, record],
    )?;
    tx.execute("UPDATE exclusions SET cleanup_complete=1,operation_id=NULL WHERE scope_key=?1 AND record_id=?2",params![access.0.scope_key,record])?;
    tx.commit()?;
    status(authority, scope, &access.0.scope_key, record)
}

fn open_authority(root: &Path, create: bool, writable: bool) -> Result<Connection> {
    let directory = root.join("memory-lifecycle");
    private_directory(&directory, create)?;
    let path = directory.join("authority.sqlite3");
    drop(open_private(&path, create)?);
    for suffix in ["-journal", "-wal", "-shm"] {
        let sidecar = directory.join(format!("authority.sqlite3{suffix}"));
        match fs::symlink_metadata(&sidecar) {
            Ok(_) => {
                drop(open_private(&sidecar, false)?);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    let flags = if writable {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    } else {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    };
    let mut connection =
        Connection::open_with_flags(path, flags | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
    connection.busy_timeout(Duration::from_millis(250))?;
    connection.set_db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
    connection.set_db_config(
        rusqlite::config::DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA,
        false,
    )?;
    connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
    let mode: String = connection.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
    if mode != "delete" {
        return Err(MemoryError::UnsupportedSchema);
    }
    if create {
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        let app: i64 = tx.query_row("PRAGMA application_id", [], |r| r.get(0))?;
        if version == 0 && app == 0 {
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
                [],
                |r| r.get(0),
            )?;
            if count != 0 {
                return Err(MemoryError::UnsupportedSchema);
            }
            tx.execute_batch("CREATE TABLE exclusions(scope_key TEXT NOT NULL,record_id TEXT NOT NULL,cleanup_complete INTEGER NOT NULL CHECK(cleanup_complete IN(0,1)),operation_id TEXT,PRIMARY KEY(scope_key,record_id)) STRICT; CREATE UNIQUE INDEX pending_forget_operation ON exclusions(operation_id) WHERE operation_id IS NOT NULL; CREATE TABLE scope_epochs(scope_key TEXT PRIMARY KEY,epoch INTEGER NOT NULL CHECK(epoch>=0)) STRICT; CREATE TABLE lifecycle_config(key TEXT PRIMARY KEY,value TEXT NOT NULL) STRICT; CREATE TABLE pending_restores(scope_key TEXT PRIMARY KEY,operation_id TEXT NOT NULL,snapshot_id TEXT NOT NULL) STRICT; CREATE TABLE forget_ops(operation_id TEXT PRIMARY KEY,scope_key TEXT NOT NULL,record_id TEXT NOT NULL,expected_revision INTEGER NOT NULL CHECK(expected_revision>0),created_at INTEGER NOT NULL,cleanup_complete INTEGER NOT NULL CHECK(cleanup_complete IN(0,1))) STRICT; CREATE TABLE cleanup_receipts(scope_key TEXT NOT NULL,record_id TEXT NOT NULL,operation_id TEXT NOT NULL,PRIMARY KEY(scope_key,record_id,operation_id),FOREIGN KEY(scope_key,record_id) REFERENCES exclusions(scope_key,record_id)) STRICT;")?;
            tx.pragma_update(None, "application_id", APPLICATION_ID)?;
            tx.pragma_update(None, "user_version", 1)?;
        }
        tx.commit()?;
    }
    let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let app: i64 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    if version != 1 || app != APPLICATION_ID {
        return Err(MemoryError::UnsupportedSchema);
    }
    Ok(connection)
}
fn validate_root(root: &Path) -> Result<PathBuf> {
    if !root.is_absolute() || root.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(MemoryError::InvalidInput(
            "lifecycle root must be explicit and absolute",
        ));
    }
    private_directory(root, false)?;
    Ok(root.canonicalize()?)
}
fn private_directory(path: &Path, create: bool) -> Result<()> {
    if create {
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(path) {
            Ok(()) => {
                if let Some(parent) = path.parent() {
                    sync_directory(parent)?;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.into()),
        }
    }
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err(MemoryError::InvalidInput(
            "lifecycle state must be a real private directory",
        ));
    }
    private_permissions(&meta)
}
fn open_private(path: &Path, create: bool) -> Result<File> {
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
        Ok(file) => {
            if create {
                file.sync_all()?;
                sync_directory(
                    path.parent()
                        .ok_or(MemoryError::InvalidInput("missing parent"))?,
                )?;
            }
            file
        }
        Err(e) if create && e.kind() == io::ErrorKind::AlreadyExists => {
            OpenOptions::new().read(true).write(true).open(path)?
        }
        Err(e) => return Err(e.into()),
    };
    same_open_file(path, &file)?;
    Ok(file)
}
fn same_open_file(path: &Path, file: &File) -> Result<()> {
    let current = fs::symlink_metadata(path)?;
    let opened = file.metadata()?;
    for meta in [&current, &opened] {
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(MemoryError::InvalidInput(
                "lifecycle file must be regular and unaliased",
            ));
        }
        private_permissions(meta)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if current.nlink() != 1
            || opened.nlink() != 1
            || (current.dev(), current.ino()) != (opened.dev(), opened.ino())
        {
            return Err(MemoryError::InvalidInput("lifecycle file identity changed"));
        }
    }
    Ok(())
}
fn private_permissions(meta: &Metadata) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err(MemoryError::InvalidInput(
                "lifecycle state must deny group and other access",
            ));
        }
    }
    #[cfg(not(unix))]
    let _ = meta;
    Ok(())
}
fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Applicability, ClaimBasis, RecordInput, RecordKind, RecordStatus, Source};
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "mirket-forget-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            private_directory(&path, true).unwrap();
            Self(path.canonicalize().unwrap())
        }
        fn scope(&self) -> Scope {
            Scope::Project("test-project".into())
        }
        fn open(&self) -> MemoryStore {
            MemoryStore::open(&self.0, self.scope()).unwrap()
        }
        fn database(&self) -> PathBuf {
            self.0.join("memory").join(format!(
                "scope-{}.sqlite3",
                scope_key(&self.scope()).unwrap()
            ))
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[cfg(unix)]
    #[test]
    fn scope_drop_unlocks_despite_duplicate_description() {
        let fixture = Fixture::new();
        let first = ScopeAccess::shared(&fixture.0, &fixture.scope(), true).unwrap();
        let second = ScopeAccess::shared(&fixture.0, &fixture.scope(), false).unwrap();
        let duplicate = second._file.as_file().try_clone().unwrap();
        drop(second);
        assert!(ExclusiveScope::acquire(&fixture.0, &fixture.scope()).is_err());
        drop(first);
        let exclusive = ExclusiveScope::acquire(&fixture.0, &fixture.scope()).unwrap();
        let exclusive_duplicate = exclusive.0._file.as_file().try_clone().unwrap();
        drop(exclusive);
        assert!(ScopeAccess::shared(&fixture.0, &fixture.scope(), false).is_ok());
        drop(exclusive_duplicate);
        drop(duplicate);
    }

    fn input(text: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::UserStated,
            content: text.into(),
            sources: vec![Source {
                source_id: "source-1".into(),
                revision: "rev-1".into(),
                locator: "user statement".into(),
                captured_at: 1,
            }],
            status: RecordStatus::Current,
            critical: false,
            applicability: Applicability::default(),
            valid_from: None,
            valid_until: None,
        }
    }
    fn operation() -> String {
        format!("op-life-{}", NEXT.fetch_add(1, Ordering::Relaxed))
    }
    fn copy_database(source: &Connection, path: &Path) {
        let mut target = Connection::open(path).unwrap();
        rusqlite::backup::Backup::new(source, &mut target)
            .unwrap()
            .run_to_completion(32, Duration::ZERO, None)
            .unwrap();
    }

    #[test]
    fn forgetting_removes_all_revisions_search_sources_and_hashes_without_other_scope_changes() {
        let f = Fixture::new();
        let mut store = f.open();
        let first_op = operation();
        let first = store
            .create(&first_op, input("forgetthisinitialvalue"), None)
            .unwrap();
        store
            .revise(
                &operation(),
                &first.record_id,
                1,
                input("forgetthisreplacementvalue"),
            )
            .unwrap();
        let retained = store
            .create(&operation(), input("retained unrelated evidence"), None)
            .unwrap();
        drop(store);
        let other_scope = Scope::Project("other-project".into());
        let mut other = MemoryStore::open(&f.0, other_scope.clone()).unwrap();
        let other_record = other
            .create(&operation(), input("other scope unchanged"), None)
            .unwrap();
        drop(other);
        let other_path = f.0.join("memory").join(format!(
            "scope-{}.sqlite3",
            scope_key(&other_scope).unwrap()
        ));
        let other_bytes = fs::read(&other_path).unwrap();
        let result = forget(&f.0, f.scope(), &first.record_id, &operation(), 2).unwrap();
        assert!(result.cleanup_complete);
        let mut store = f.open();
        assert!(matches!(
            store.read(&first.record_id, None),
            Err(MemoryError::NotFound)
        ));
        assert!(matches!(
            store.history(&first.record_id, None, 10, 32768),
            Err(MemoryError::NotFound)
        ));
        assert!(matches!(
            store.create(
                &first_op,
                input("changed replay must still be unavailable"),
                None
            ),
            Err(MemoryError::MutationUnavailable)
        ));
        assert_eq!(
            store.read(&retained.record_id, None).unwrap().data.content,
            "retained unrelated evidence"
        );
        let conn = store.lifecycle_connection();
        let count:i64=conn.query_row("SELECT count(*) FROM record_fts WHERE record_fts MATCH 'forgetthisinitialvalue OR forgetthisreplacementvalue'",[],|r|r.get(0)).unwrap();
        assert_eq!(count, 0);
        let orphan:i64=conn.query_row("SELECT count(*) FROM sources WHERE revision_row_id NOT IN (SELECT row_id FROM revisions)",[],|r|r.get(0)).unwrap();
        assert_eq!(orphan, 0);
        let hash: String = conn
            .query_row(
                "SELECT intent_hash FROM mutations WHERE operation_id=?1",
                [first_op],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hash, "");
        assert_eq!(fs::read(other_path).unwrap(), other_bytes);
        assert_eq!(
            MemoryStore::open_existing(&f.0, other_scope)
                .unwrap()
                .read(&other_record.record_id, None)
                .unwrap()
                .data
                .content,
            "other scope unchanged"
        );
    }
    #[test]
    fn exclusion_commit_before_cleanup_survives_reopen_and_is_content_free() {
        let f = Fixture::new();
        let mut store = f.open();
        let record = store
            .create(
                &operation(),
                input("unique private forgotten payload"),
                None,
            )
            .unwrap();
        drop(store);
        {
            let access = ExclusiveScope::acquire(&f.0, &f.scope()).unwrap();
            let mut canonical = MemoryStore::open_for_lifecycle(&f.0, f.scope(), &access).unwrap();
            let mut authority = open_authority(&f.0, false, true).unwrap();
            begin_forget(
                &mut authority,
                canonical.lifecycle_connection(),
                &access.0.scope_key,
                &record.record_id,
                &operation(),
                1,
            )
            .unwrap();
        }
        assert!(MemoryStore::open_existing(&f.0, f.scope()).is_err());
        let pending = inspect(&f.0, f.scope(), &record.record_id).unwrap();
        assert!(pending.logically_forgotten);
        assert!(!pending.cleanup_complete);
        let bytes = fs::read(f.0.join("memory-lifecycle/authority.sqlite3")).unwrap();
        assert!(
            !bytes
                .windows(31)
                .any(|w| w == b"unique private forgotten payload")
        );
        assert!(
            reconcile(&f.0, f.scope(), &record.record_id)
                .unwrap()
                .cleanup_complete
        );
        assert!(f.open().read(&record.record_id, None).is_err());
    }
    #[test]
    fn older_sqlite_backup_cannot_resurrect_known_forgotten_lineage() {
        let f = Fixture::new();
        let mut store = f.open();
        let record = store
            .create(&operation(), input("forgotten in old snapshot"), None)
            .unwrap();
        let kept = store
            .create(&operation(), input("retain snapshot neighbor"), None)
            .unwrap();
        let snapshot = f.0.join("external-test-snapshot.sqlite3");
        copy_database(store.lifecycle_connection(), &snapshot);
        drop(store);
        forget(&f.0, f.scope(), &record.record_id, &operation(), 1).unwrap();
        // Model an old import/restore at the database boundary; it cannot replace
        // the external authority. This is not a managed restoration API.
        {
            let access = ExclusiveScope::acquire(&f.0, &f.scope()).unwrap();
            let mut live = MemoryStore::open_for_lifecycle(&f.0, f.scope(), &access).unwrap();
            let old = Connection::open(&snapshot).unwrap();
            rusqlite::backup::Backup::new(&old, live.lifecycle_connection())
                .unwrap()
                .run_to_completion(32, Duration::ZERO, None)
                .unwrap();
        }
        assert!(MemoryStore::open_existing(&f.0, f.scope()).is_err());
        reconcile(&f.0, f.scope(), &record.record_id).unwrap();
        let mut store = f.open();
        assert!(store.read(&record.record_id, None).is_err());
        assert!(store.read(&kept.record_id, None).is_ok());
    }
    #[test]
    fn existing_readers_exclude_forgetting_and_missing_read_creates_nothing() {
        let f = Fixture::new();
        assert!(matches!(
            MemoryStore::open_existing(&f.0, f.scope()),
            Err(MemoryError::NotFound)
        ));
        assert_eq!(fs::read_dir(&f.0).unwrap().count(), 0);
        let mut store = f.open();
        let record = store
            .create(&operation(), input("reader snapshot"), None)
            .unwrap();
        let err = forget(&f.0, f.scope(), &record.record_id, &operation(), 1).unwrap_err();
        assert!(err.to_string().contains("busy"));
        assert!(
            !inspect(&f.0, f.scope(), &record.record_id)
                .unwrap()
                .logically_forgotten
        );
        drop(store);
        assert!(
            forget(&f.0, f.scope(), &record.record_id, &operation(), 1)
                .unwrap()
                .cleanup_complete
        );
    }
    #[test]
    fn cleanup_removes_only_consumed_shared_receipts_and_resumes_after_lock_failure() {
        let f = Fixture::new();
        let operations = OperationStore::open(&f.0).unwrap();
        let (first, unrelated) = {
            let locked = operations.lock().unwrap();
            (
                locked
                    .issue(br#"{"intent_revision":"forgotten-derived-hash"}"#)
                    .unwrap(),
                locked.issue(br#"{"intent_revision":"keep-this"}"#).unwrap(),
            )
        };
        let mut store = f.open();
        let record = store.create(&first, input("private record"), None).unwrap();
        drop(store);
        let locked = operations.lock().unwrap();
        assert!(forget(&f.0, f.scope(), &record.record_id, &operation(), 1).is_err());
        assert!(
            !inspect(&f.0, f.scope(), &record.record_id)
                .unwrap()
                .cleanup_complete
        );
        assert!(locked.read(&first).is_ok());
        drop(locked);
        assert!(
            reconcile(&f.0, f.scope(), &record.record_id)
                .unwrap()
                .cleanup_complete
        );
        let locked = operations.lock().unwrap();
        assert!(locked.read(&first).is_err());
        assert_eq!(
            locked.read(&unrelated).unwrap(),
            br#"{"intent_revision":"keep-this"}"#
        );
    }
    #[test]
    fn prepared_target_receipts_are_scrubbed_but_completed_other_lineages_survive() {
        let f = Fixture::new();
        let mut store = f.open();
        let target = store
            .create(&operation(), input("forgotten target"), None)
            .unwrap();
        let other = store
            .create(&operation(), input("different target"), None)
            .unwrap();
        let operations = OperationStore::open(&f.0).unwrap();
        let issue = |scope: Scope, target: &str, status: &str| {
            operations.lock().unwrap().issue(&serde_json::to_vec(&serde_json::json!({"schema":1,"kind":"memory_mutation","scope":scope,"target":{"record_id":target,"revision":1},"status":status,"intent_revision":"private-derived-intent"})).unwrap()).unwrap()
        };
        let prepared = issue(f.scope(), &target.record_id, "prepared");
        let other_target = issue(f.scope(), &other.record_id, "prepared");
        let other_scope = issue(
            Scope::Project("other-project".into()),
            &target.record_id,
            "prepared",
        );
        let completed_successor = issue(f.scope(), &target.record_id, "unknown");
        let successor = store
            .create(
                &completed_successor,
                input("successor belongs to a new lineage"),
                Some(crate::memory::RecordRef {
                    record_id: target.record_id.clone(),
                    revision: 1,
                }),
            )
            .unwrap();
        drop(store);
        forget(&f.0, f.scope(), &target.record_id, &operation(), 1).unwrap();
        let locked = operations.lock().unwrap();
        assert!(locked.read(&prepared).is_err());
        for id in [&other_target, &other_scope, &completed_successor] {
            assert!(locked.read(id).is_ok());
        }
        drop(locked);
        let mut store = f.open();
        assert!(store.read(&successor.record_id, None).is_ok());
        assert!(matches!(
            store.create(&prepared, input("must not revive"), None),
            Err(MemoryError::MutationUnavailable)
        ));
    }

    #[test]
    fn stale_forget_preparation_does_not_erase_a_concurrent_correction() {
        let f = Fixture::new();
        let mut store = f.open();
        let record = store
            .create(&operation(), input("old value"), None)
            .unwrap();
        store
            .revise(
                &operation(),
                &record.record_id,
                1,
                input("concurrent correction"),
            )
            .unwrap();
        drop(store);
        let stale = operation();
        assert!(matches!(
            forget(&f.0, f.scope(), &record.record_id, &stale, 1),
            Err(MemoryError::Conflict)
        ));
        assert!(
            !inspect(&f.0, f.scope(), &record.record_id)
                .unwrap()
                .logically_forgotten
        );
        assert!(
            inspect_operation(&f.0, f.scope(), &record.record_id, &stale)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            f.open().read(&record.record_id, None).unwrap().data.content,
            "concurrent correction"
        );
        let committed = operation();
        forget(&f.0, f.scope(), &record.record_id, &committed, 2).unwrap();
        let inspected = inspect_operation(&f.0, f.scope(), &record.record_id, &committed)
            .unwrap()
            .unwrap();
        assert_eq!(inspected.expected_revision, 2);
        assert!(inspected.status.cleanup_complete);
        assert!(
            inspect_operation(&f.0, f.scope(), &record.record_id, &stale)
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            forget(&f.0, f.scope(), &record.record_id, &committed, 1),
            Err(MemoryError::OperationConflict)
        ));
        assert!(
            forget(&f.0, f.scope(), &record.record_id, &committed, 2)
                .unwrap()
                .cleanup_complete
        );
    }

    #[test]
    fn lost_exclusion_authority_is_not_silently_recreated_for_existing_memory() {
        let f = Fixture::new();
        drop(f.open());
        fs::remove_file(f.0.join("memory-lifecycle/authority.sqlite3")).unwrap();
        assert!(MemoryStore::open(&f.0, f.scope()).is_err());
        assert!(matches!(
            MemoryStore::open_existing(&f.0, f.scope()),
            Err(MemoryError::InvalidInput(_))
        ));
        assert!(!f.0.join("memory-lifecycle/authority.sqlite3").exists());
        assert!(f.database().exists());
    }
}
