//! Finite CLI restore intents. Receipt state never authorizes replay of an
//! uncertain effect: only the backup/lifecycle owner's existing durable binding
//! can resume it. No memory body, source digest or snapshot byte hash is copied
//! into this operation namespace. Status performs inspection only.

#[path = "memory_transfer_admin.rs"]
mod transfer;
pub(crate) use transfer::invalidate_export_stages;

pub use crate::memory_service::Selection;
use crate::{
    identity,
    maintenance::{MaintenanceGuard, SessionGuard},
    memory::{MemoryStore, Scope},
    memory_backup::{self, BackupPolicy, RestoreStatus},
    memory_lifecycle,
    operation_store::{LockedStore, OperationStore},
    release::ReleaseId,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const PREPARED_SECONDS: u64 = 300;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptPhase {
    Prepared,
    Unknown,
    Complete,
    ExpiredNotStarted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: u8,
    kind: String,
    release_id: String,
    backup_root: PathBuf,
    scope: Scope,
    checkout_id: Option<String>,
    snapshot_id: String,
    created_at: u64,
    expires_at: u64,
    phase: ReceiptPhase,
}

#[derive(Clone, Debug, Serialize)]
pub struct RestoreReport {
    pub operation_id: String,
    pub release_id: String,
    pub backup_root: PathBuf,
    pub scope: Scope,
    pub checkout_id: Option<String>,
    pub snapshot_id: String,
    pub expires_at: u64,
    pub receipt_phase: ReceiptPhase,
    pub status: String,
    pub authority_phase: Option<String>,
    /// Only the canonical backup authority, never JSON alone, confirms this.
    pub complete: bool,
    pub needs_reconcile: bool,
    pub notice: Option<String>,
}

pub struct MemoryAdmin {
    state_root: PathBuf,
    backup_root: PathBuf,
    state_stamp: RootStamp,
    backup_stamp: RootStamp,
    release: ReleaseId,
    policy: BackupPolicy,
}

impl MemoryAdmin {
    /// No registration, maintenance acquisition, model work or directory creation.
    pub fn new(
        state_root: PathBuf,
        backup_root: PathBuf,
        release: ReleaseId,
        policy: BackupPolicy,
    ) -> Result<Self, String> {
        let state_stamp = private_directory(&state_root)?;
        let backup_stamp = private_directory(&backup_root)?;
        if policy.max_bytes < 128 * 1024 || policy.max_bytes > i64::MAX as u64 {
            return Err("invalid explicit backup byte budget".into());
        }
        Ok(Self {
            state_root,
            backup_root,
            state_stamp,
            backup_stamp,
            release,
            policy,
        })
    }

    pub fn backups(&self, selection: &Selection) -> Result<Vec<memory_backup::BackupInfo>, String> {
        self.check_paths()?;
        let (scope, _) = self.resolve(selection)?;
        memory_backup::inspect_backups(&self.state_root, &self.backup_root, scope).map_err(error)
    }

    pub fn prepare(
        &self,
        selection: &Selection,
        snapshot_id: &str,
    ) -> Result<RestoreReport, String> {
        self.check_paths()?;
        let _session = SessionGuard::acquire(&self.state_root).map_err(error)?;
        let (scope, checkout_id) = self.resolve(selection)?;
        // Pin existing canonical scope through inspection and ID issuance; no
        // forgetting sweep can invalidate its backing snapshot in that interval.
        let _scope = MemoryStore::open_existing(&self.state_root, scope.clone()).map_err(error)?;
        let eligible =
            memory_backup::inspect_backups(&self.state_root, &self.backup_root, scope.clone())
                .map_err(error)?
                .into_iter()
                .any(|saved| saved.snapshot_id == snapshot_id && saved.eligible);
        if !eligible {
            return Err("selected snapshot is not an eligible recovery point in this scope".into());
        }
        let created_at = now()?;
        let receipt = Receipt {
            schema: 1,
            kind: "memory_restore".into(),
            release_id: self.release.as_str().into(),
            backup_root: self.backup_root.clone(),
            scope,
            checkout_id,
            snapshot_id: snapshot_id.into(),
            created_at,
            expires_at: created_at
                .checked_add(PREPARED_SECONDS)
                .ok_or("restore clock overflow")?,
            phase: ReceiptPhase::Prepared,
        };
        let store = OperationStore::open(&self.state_root).map_err(error)?;
        let id = store
            .lock()
            .map_err(error)?
            .issue(&encode(&receipt)?)
            .map_err(error)?;
        self.report(&id, &receipt, None, None)
    }

    /// Inspect current-schema receipts across release changes without rewriting
    /// them. Root/scope bindings still have to match; inspection creates no intent.
    pub fn status(
        &self,
        selection: &Selection,
        operation_id: &str,
    ) -> Result<RestoreReport, String> {
        let receipt = self.read(selection, operation_id)?;
        let authority = self.authority(operation_id, &receipt)?;
        self.report(operation_id, &receipt, authority, None)
    }

    /// Retire one explicitly inspected, expired preparation at an idle
    /// maintenance boundary. This never starts or reconciles a restore. Unknown
    /// recovery remains protected; absence alone is not a no-effect proof.
    pub fn retire_expired(
        &self,
        selection: &Selection,
        operation_id: &str,
    ) -> Result<RestoreReport, String> {
        self.check_paths()?;
        let _maintenance = MaintenanceGuard::try_acquire(&self.state_root).map_err(error)?;
        let (mut receipt, expected_payload) = self.read_payload(selection, operation_id)?;
        match receipt.phase {
            ReceiptPhase::Prepared if now()? >= receipt.expires_at => (),
            ReceiptPhase::Prepared => return Err("prepared restore has not expired".into()),
            ReceiptPhase::ExpiredNotStarted => (),
            ReceiptPhase::Unknown | ReceiptPhase::Complete => {
                return Err("only an expired never-started preparation may be retired; retain unknown or completed restore evidence".into());
            }
        }
        // Check both the selected scope and this operation in any scope. A
        // durable pending binding is recovery authority even before a catalog
        // row exists; another pending restore also makes this scope uncertain.
        if memory_lifecycle::pending_restore_binding(&self.state_root, &receipt.scope)
            .map_err(error)?
            .is_some()
            || memory_lifecycle::restore_operation_pending(&self.state_root, operation_id)
                .map_err(error)?
        {
            return Err("pending restore authority prevents preparation retirement".into());
        }
        if self.authority(operation_id, &receipt)?.is_some() {
            return Err("authoritative restore operation exists; inspect and reconcile rather than retire it as never started".into());
        }
        // Keep an existing, read-only canonical scope pinned through receipt
        // finalization. Missing/corrupt canonical state is not absence evidence.
        // The exact restore marker can survive loss/expiry of catalog evidence.
        let mut canonical =
            MemoryStore::open_existing(&self.state_root, receipt.scope.clone()).map_err(error)?;
        let canonical_operation: bool = canonical.lifecycle_connection().query_row(
            "SELECT EXISTS(SELECT 1 FROM metadata WHERE key='last_restore_operation' AND value=?1)",
            [operation_id], |row| row.get(0),
        ).map_err(error)?;
        if canonical_operation {
            return Err("canonical restore authority prevents preparation retirement".into());
        }
        self.check_paths()?;
        receipt.phase = ReceiptPhase::ExpiredNotStarted;
        let store = OperationStore::open_existing(&self.state_root).map_err(error)?;
        {
            let locked = store.lock().map_err(error)?;
            finalize_expired(&locked, operation_id, &expected_payload, &receipt)?;
        }
        self.report(operation_id, &receipt, None, None)
    }

    pub fn commit(
        &self,
        selection: &Selection,
        operation_id: &str,
    ) -> Result<RestoreReport, String> {
        self.check_paths()?;
        let maintenance = MaintenanceGuard::try_acquire(&self.state_root).map_err(error)?;
        let mut receipt = self.read(selection, operation_id)?;
        if receipt.phase == ReceiptPhase::ExpiredNotStarted {
            return Err("expired restore preparation was retired without starting; this ID cannot start or reconcile a restore".into());
        }
        if receipt.release_id != self.release.as_str() {
            return Err("restore commit requires the release that prepared this intent".into());
        }
        let authority = self.authority(operation_id, &receipt)?;
        if receipt.phase == ReceiptPhase::Complete {
            return self.report(operation_id, &receipt, authority, None);
        }
        if receipt.phase != ReceiptPhase::Prepared || authority.is_some() {
            return Err("restore outcome is already bound or unknown; inspect and reconcile the same issued ID, never replay it as fresh".into());
        }
        if now()? >= receipt.expires_at {
            return Err("prepared restore expired; this ID cannot start a restore".into());
        }
        // Persist uncertainty before calling any effect owner. Drop the receipt
        // lock before taking scoped backup locks; maintenance excludes runtimes.
        let store = OperationStore::open_existing(&self.state_root).map_err(error)?;
        {
            let locked = store.lock().map_err(error)?;
            let current = decode(&locked.read(operation_id).map_err(error)?)?;
            if current != receipt {
                return Err("restore receipt changed before commit".into());
            }
            receipt.phase = ReceiptPhase::Unknown;
            locked
                .update(operation_id, &encode(&receipt)?)
                .map_err(error)?;
        }
        let attempted = memory_backup::restore(
            &maintenance,
            &self.backup_root,
            receipt.scope.clone(),
            &receipt.snapshot_id,
            operation_id,
            self.policy,
        );
        self.finish(operation_id, receipt, attempted)
    }

    /// Expiry prevents a fresh start, not reconciliation of an already durable
    /// restore binding, including one from another release using this supported
    /// receipt schema. No replacement snapshot or fresh intent is accepted here.
    pub fn reconcile(
        &self,
        selection: &Selection,
        operation_id: &str,
    ) -> Result<RestoreReport, String> {
        self.check_paths()?;
        let maintenance = MaintenanceGuard::try_acquire(&self.state_root).map_err(error)?;
        let mut receipt = self.read(selection, operation_id)?;
        if receipt.phase == ReceiptPhase::ExpiredNotStarted {
            return Err("expired restore preparation was retired without starting; this ID cannot start or reconcile a restore".into());
        }
        let authority = self.authority(operation_id, &receipt)?
            .ok_or("no authoritative restore binding exists for this ID; absence is not permission to replay")?;
        if matches!(authority.phase.as_str(), "aborted" | "invalidated") {
            return self.report(operation_id, &receipt, Some(authority), None);
        }
        // Also covers a crash after authority committed but before a prepared
        // operation receipt could be updated by an earlier caller.
        if receipt.phase == ReceiptPhase::Prepared {
            let locked = OperationStore::open_existing(&self.state_root)
                .map_err(error)?
                .lock()
                .map_err(error)?;
            let current = decode(&locked.read(operation_id).map_err(error)?)?;
            if current != receipt {
                return Err("restore receipt changed before reconciliation".into());
            }
            receipt.phase = ReceiptPhase::Unknown;
            locked
                .update(operation_id, &encode(&receipt)?)
                .map_err(error)?;
        }
        let attempted = memory_backup::reconcile_restore(
            &maintenance,
            &self.backup_root,
            receipt.scope.clone(),
            operation_id,
            self.policy,
        );
        self.finish(operation_id, receipt, attempted)
    }

    fn finish(
        &self,
        operation_id: &str,
        mut receipt: Receipt,
        attempted: Result<RestoreStatus, crate::memory::MemoryError>,
    ) -> Result<RestoreReport, String> {
        let authority = self.authority(operation_id, &receipt).map_err(|failure|
            format!("restore outcome is unknown; retain issued ID and inspect before any retry: {failure}"))?;
        let mut notice = attempted.err().map(|failure| {
            format!("restore owner reported: {failure}; inspect the authoritative phase")
        });
        if authority.as_ref().is_some_and(|state| state.complete) {
            let previous = receipt.clone();
            receipt.phase = ReceiptPhase::Complete;
            // Canonical completion and recovery cleanup are separate facts.
            // Keep this receipt protected while any matching pending binding
            // remains, even when the restored data itself is already confirmed.
            let recovery_complete = !self
                .report(operation_id, &receipt, authority.clone(), None)?
                .needs_reconcile;
            let updated = (|| {
                let locked = OperationStore::open_existing(&self.state_root)
                    .map_err(error)?
                    .lock()
                    .map_err(error)?;
                update_owned(
                    &locked,
                    operation_id,
                    &previous,
                    &receipt,
                    recovery_complete,
                )
            })();
            if updated.is_err() {
                receipt = previous;
                notice = Some("canonical restore completion is confirmed, but receipt finalization failed; inspect this same issued ID".into());
            }
        }
        self.report(operation_id, &receipt, authority, notice)
    }

    fn resolve(&self, selection: &Selection) -> Result<(Scope, Option<String>), String> {
        match selection {
            Selection::SharedPreferences => Ok((Scope::SharedPreferences, None)),
            Selection::Project { project_root } => {
                let selected = identity::lookup(&self.state_root, project_root)
                    .map_err(error)?
                    .ok_or("project has no registered memory scope; restore cannot register it")?;
                Ok((
                    Scope::Project(selected.project_id),
                    Some(selected.checkout_id),
                ))
            }
        }
    }
    fn read(&self, selection: &Selection, operation_id: &str) -> Result<Receipt, String> {
        self.read_payload(selection, operation_id)
            .map(|(receipt, _)| receipt)
    }
    fn read_payload(
        &self,
        selection: &Selection,
        operation_id: &str,
    ) -> Result<(Receipt, Vec<u8>), String> {
        self.check_paths()?;
        let (scope, checkout_id) = self.resolve(selection)?;
        let store = OperationStore::open_existing(&self.state_root).map_err(error)?;
        let payload = store.read_existing(operation_id).map_err(error)?;
        let receipt = decode(&payload)?;
        if receipt.backup_root != self.backup_root
            || receipt.scope != scope
            || receipt.checkout_id != checkout_id
        {
            return Err("restore receipt differs from the selected backup root or scope".into());
        }
        Ok((receipt, payload))
    }
    fn check_paths(&self) -> Result<(), String> {
        if private_directory(&self.state_root)? != self.state_stamp
            || private_directory(&self.backup_root)? != self.backup_stamp
        {
            return Err("selected restore state or backup directory identity changed".into());
        }
        Ok(())
    }
    fn authority(
        &self,
        operation_id: &str,
        receipt: &Receipt,
    ) -> Result<Option<RestoreStatus>, String> {
        let authority = memory_backup::inspect_restore(
            &self.state_root,
            &self.backup_root,
            receipt.scope.clone(),
            operation_id,
        )
        .map_err(error)?;
        if authority.as_ref().is_some_and(|status| {
            status.snapshot_id != receipt.snapshot_id
                || status.scope != receipt.scope
                || status.operation_id != operation_id
        }) {
            return Err("authoritative restore binding differs from its issued receipt".into());
        }
        Ok(authority)
    }
    fn report(
        &self,
        operation_id: &str,
        receipt: &Receipt,
        authority: Option<RestoreStatus>,
        notice: Option<String>,
    ) -> Result<RestoreReport, String> {
        let current_time = now()?;
        let pending = memory_lifecycle::pending_restore_binding(&self.state_root, &receipt.scope)
            .map_err(error)?;
        if pending
            .as_ref()
            .is_some_and(|(id, snapshot)| id == operation_id && snapshot != &receipt.snapshot_id)
        {
            return Err("pending restore binding differs from its issued receipt".into());
        }
        let pending_this = pending.as_ref().is_some_and(|(id, _)| id == operation_id);
        if receipt.phase == ReceiptPhase::ExpiredNotStarted && (authority.is_some() || pending_this)
        {
            return Err("expired-not-started receipt conflicts with restore authority; preserve for inspection".into());
        }
        let complete = authority.as_ref().is_some_and(|state| state.complete);
        let authority_phase = authority.as_ref().map(|state| state.phase.clone());
        let status = authority_phase.clone().unwrap_or_else(|| {
            match receipt.phase {
                ReceiptPhase::Prepared if current_time >= receipt.expires_at => "expired",
                ReceiptPhase::Prepared => "prepared",
                ReceiptPhase::Unknown | ReceiptPhase::Complete => "unknown",
                ReceiptPhase::ExpiredNotStarted => "expired_not_started",
            }
            .into()
        });
        let needs_reconcile = pending_this
            || authority_phase
                .as_deref()
                .is_some_and(|phase| matches!(phase, "preparing" | "preserving" | "prepared"));
        let notice = notice.or_else(|| {
            if receipt.phase == ReceiptPhase::ExpiredNotStarted {
                Some("expired preparation was retired without starting a restore; this receipt may expire and cannot authorize replay".into())
            } else {
                (authority.is_none() && receipt.phase != ReceiptPhase::Prepared)
                    .then(|| "no authoritative restore binding is available; this issued ID cannot be replayed as fresh".into())
            }
        });
        Ok(RestoreReport {
            operation_id: operation_id.into(),
            release_id: receipt.release_id.clone(),
            backup_root: receipt.backup_root.clone(),
            scope: receipt.scope.clone(),
            checkout_id: receipt.checkout_id.clone(),
            snapshot_id: receipt.snapshot_id.clone(),
            expires_at: receipt.expires_at,
            receipt_phase: receipt.phase,
            status,
            authority_phase,
            complete,
            needs_reconcile,
            notice,
        })
    }
}

fn finalize_expired(
    locked: &LockedStore,
    id: &str,
    expected_payload: &[u8],
    expired: &Receipt,
) -> Result<(), String> {
    // Exact bytes, not just equivalent decoded fields: an intervening rewrite
    // invalidates the inspection that established no started effect.
    if locked.read(id).map_err(error)? != expected_payload {
        return Err("restore receipt changed before expired preparation retirement".into());
    }
    locked.finalize(id, &encode(expired)?).map_err(error)
}

fn update_owned(
    locked: &LockedStore,
    id: &str,
    expected: &Receipt,
    changed: &Receipt,
    recovery_complete: bool,
) -> Result<(), String> {
    if decode(&locked.read(id).map_err(error)?)? != *expected {
        return Err("restore receipt changed before finalization".into());
    }
    let bytes = encode(changed)?;
    if recovery_complete {
        locked.finalize(id, &bytes).map_err(error)
    } else {
        locked.update(id, &bytes).map_err(error)
    }
}
fn encode(receipt: &Receipt) -> Result<Vec<u8>, String> {
    serde_json::to_vec(receipt).map_err(error)
}
fn decode(bytes: &[u8]) -> Result<Receipt, String> {
    let receipt: Receipt = serde_json::from_slice(bytes)
        .map_err(|_| "operation is not a valid typed restore receipt")?;
    if receipt.schema != 1
        || receipt.kind != "memory_restore"
        || receipt.created_at.checked_add(PREPARED_SECONDS) != Some(receipt.expires_at)
        || !receipt
            .snapshot_id
            .strip_prefix("snapshot-")
            .is_some_and(|id| id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("operation is not a supported restore receipt".into());
    }
    Ok(receipt)
}
fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|time| time.as_secs())
        .map_err(error)
}
fn error(value: impl std::fmt::Display) -> String {
    value.to_string()
}
#[derive(Eq, PartialEq)]
struct RootStamp {
    created: Option<SystemTime>,
    #[cfg(unix)]
    native: (u64, u64),
}
fn private_directory(path: &Path) -> Result<RootStamp, String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("restore requires explicit absolute directory paths".into());
    }
    let metadata = fs::symlink_metadata(path).map_err(error)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || path.canonicalize().map_err(error)? != path
    {
        return Err("restore requires existing canonical directories".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.mode() & 0o077 != 0 || metadata.uid() != unsafe { libc::geteuid() } {
            return Err("restore state and backups must be private and user-owned".into());
        }
    }
    #[cfg(not(unix))]
    return Err("native restore directory ownership is not qualified on this platform".into());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(RootStamp {
            created: metadata.created().ok(),
            native: (metadata.dev(), metadata.ino()),
        })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::memory::{
        Applicability, ClaimBasis, Record, RecordInput, RecordKind, RecordStatus, Source,
    };
    use crate::memory_backup::BackupKind;
    use std::{
        os::unix::fs::DirBuilderExt,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
        backup: PathBuf,
        project: PathBuf,
        scope: Scope,
    }
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "mirket-restore-admin-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
            let root = path.canonicalize().unwrap();
            let project = root.join("project");
            let backup = root.join("backups");
            for path in [&project, &backup] {
                fs::DirBuilder::new().mode(0o700).create(path).unwrap();
            }
            let selected = identity::resolve(&root, &project).unwrap();
            let scope = Scope::Project(selected.project_id);
            drop(MemoryStore::open(&root, scope.clone()).unwrap());
            Self {
                root,
                backup,
                project,
                scope,
            }
        }
        fn selection(&self) -> Selection {
            Selection::Project {
                project_root: self.project.clone(),
            }
        }
        fn admin(&self) -> MemoryAdmin {
            MemoryAdmin::new(
                self.root.clone(),
                self.backup.clone(),
                ReleaseId::parse("release-a").unwrap(),
                BackupPolicy::default(),
            )
            .unwrap()
        }
        fn store(&self) -> MemoryStore {
            MemoryStore::open(&self.root, self.scope.clone()).unwrap()
        }
        fn snapshot(&self) -> String {
            memory_backup::backup(
                &self.root,
                &self.backup,
                self.scope.clone(),
                &operation(),
                BackupKind::Daily,
                BackupPolicy::default(),
            )
            .unwrap()
            .snapshot_id
        }
        fn issued(&self) -> (MemoryAdmin, Record, RestoreReport) {
            let record = self
                .store()
                .create(&operation(), input("original canonical content"), None)
                .unwrap();
            let snapshot = self.snapshot();
            self.store()
                .revise(
                    &operation(),
                    &record.record_id,
                    1,
                    input("newer canonical content"),
                )
                .unwrap();
            let admin = self.admin();
            let report = admin.prepare(&self.selection(), &snapshot).unwrap();
            (admin, record, report)
        }
        fn receipt(&self, id: &str) -> Receipt {
            let store = OperationStore::open_existing(&self.root).unwrap();
            decode(&store.read_existing(id).unwrap()).unwrap()
        }
        fn change_receipt(&self, id: &str, change: impl FnOnce(&mut Receipt)) {
            let store = OperationStore::open_existing(&self.root).unwrap();
            let locked = store.lock().unwrap();
            let mut receipt = decode(&locked.read(id).unwrap()).unwrap();
            change(&mut receipt);
            locked.update(id, &encode(&receipt).unwrap()).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    fn operation() -> String {
        format!("op-admin-test-{}", NEXT.fetch_add(1, Ordering::Relaxed))
    }
    fn input(content: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::Observed,
            content: content.into(),
            sources: vec![Source {
                source_id: "test:explicit".into(),
                revision: "v1".into(),
                locator: "line:1".into(),
                captured_at: 1,
            }],
            status: RecordStatus::Current,
            critical: false,
            applicability: Applicability::default(),
            valid_from: None,
            valid_until: None,
        }
    }

    #[test]
    fn restore_is_bound_preserves_current_and_completed_commit_never_replays() {
        let fixture = Fixture::new();
        let (admin, record, prepared) = fixture.issued();
        assert_eq!(prepared.status, "prepared");
        assert_eq!(prepared.receipt_phase, ReceiptPhase::Prepared);
        assert!(!prepared.complete);
        assert!(
            admin
                .backups(&fixture.selection())
                .unwrap()
                .iter()
                .any(|point| point.snapshot_id == prepared.snapshot_id && point.eligible)
        );
        let bytes = OperationStore::open_existing(&fixture.root)
            .unwrap()
            .read_existing(&prepared.operation_id)
            .unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(!text.contains("original canonical content"));
        assert!(!text.contains("newer canonical content"));
        assert!(!text.contains("sha256"));
        assert!(!text.contains("intent_revision"));
        let completed = admin
            .commit(&fixture.selection(), &prepared.operation_id)
            .unwrap();
        assert!(completed.complete);
        assert!(!completed.needs_reconcile);
        assert_eq!(completed.receipt_phase, ReceiptPhase::Complete);
        assert!(
            OperationStore::open_existing(&fixture.root)
                .unwrap()
                .lock()
                .unwrap()
                .update(&prepared.operation_id, b"reopened")
                .is_err()
        );
        assert_eq!(
            fixture
                .store()
                .read(&record.record_id, None)
                .unwrap()
                .data
                .content,
            "original canonical content"
        );
        assert!(admin.backups(&fixture.selection()).unwrap().len() >= 2);
        fixture
            .store()
            .revise(
                &operation(),
                &record.record_id,
                1,
                input("after completed restore"),
            )
            .unwrap();
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .unwrap()
                .complete
        );
        assert_eq!(
            fixture
                .store()
                .read(&record.record_id, None)
                .unwrap()
                .data
                .content,
            "after completed restore"
        );
    }

    #[test]
    fn status_and_missing_id_inspection_do_not_write_or_register() {
        let fixture = Fixture::new();
        let (admin, _, prepared) = fixture.issued();
        let receipt_path = fixture
            .root
            .join("operations")
            .join(format!("{}.json", prepared.operation_id));
        let before = fs::read(&receipt_path).unwrap();
        assert_eq!(
            admin
                .status(&fixture.selection(), &prepared.operation_id)
                .unwrap()
                .status,
            "prepared"
        );
        assert!(admin.status(&fixture.selection(), "op-missing").is_err());
        assert!(admin.commit(&fixture.selection(), "op-missing").is_err());
        assert!(admin.reconcile(&fixture.selection(), "op-missing").is_err());
        assert!(!fixture.root.join("operations/op-missing.json").exists());
        assert_eq!(fs::read(receipt_path).unwrap(), before);
        let other = fixture.root.join("unregistered");
        fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
        let selection = Selection::Project {
            project_root: other.clone(),
        };
        assert!(admin.backups(&selection).is_err());
        assert!(admin.prepare(&selection, &prepared.snapshot_id).is_err());
        assert!(identity::lookup(&fixture.root, &other).unwrap().is_none());
    }

    #[test]
    fn expired_preparation_is_never_fresh_restore_authority() {
        let fixture = Fixture::new();
        let (admin, record, prepared) = fixture.issued();
        fixture.change_receipt(&prepared.operation_id, |receipt| {
            receipt.created_at = 0;
            receipt.expires_at = PREPARED_SECONDS;
        });
        assert_eq!(
            admin
                .status(&fixture.selection(), &prepared.operation_id)
                .unwrap()
                .status,
            "expired"
        );
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert!(
            admin
                .reconcile(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert_eq!(
            fixture.receipt(&prepared.operation_id).phase,
            ReceiptPhase::Prepared
        );
        assert_eq!(
            fixture
                .store()
                .read(&record.record_id, None)
                .unwrap()
                .revision,
            2
        );
        assert!(
            memory_backup::inspect_restore(
                &fixture.root,
                &fixture.backup,
                fixture.scope.clone(),
                &prepared.operation_id
            )
            .unwrap()
            .is_none()
        );
    }

    fn expire(fixture: &Fixture, id: &str) {
        fixture.change_receipt(id, |receipt| {
            receipt.created_at = 0;
            receipt.expires_at = PREPARED_SECONDS;
        });
    }
    fn envelope(fixture: &Fixture, id: &str) -> Vec<u8> {
        fs::read(fixture.root.join("operations").join(format!("{id}.json"))).unwrap()
    }
    #[test]
    fn expired_never_started_intent_becomes_retirable_once_without_restoring() {
        let fixture = Fixture::new();
        let (admin, record, prepared) = fixture.issued();
        expire(&fixture, &prepared.operation_id);
        let before = envelope(&fixture, &prepared.operation_id);
        let status = admin
            .status(&fixture.selection(), &prepared.operation_id)
            .unwrap();
        assert_eq!(status.status, "expired");
        assert_eq!(
            envelope(&fixture, &prepared.operation_id),
            before,
            "inspection cannot retire intent"
        );
        let backup_before =
            serde_json::to_vec(&admin.backups(&fixture.selection()).unwrap()).unwrap();
        // Retirement can clean a supported old-release preparation without
        // granting a new release permission to execute the old intent.
        let newer = MemoryAdmin::new(
            fixture.root.clone(),
            fixture.backup.clone(),
            ReleaseId::parse("release-b").unwrap(),
            BackupPolicy::default(),
        )
        .unwrap();
        let retired = newer
            .retire_expired(&fixture.selection(), &prepared.operation_id)
            .unwrap();
        assert_eq!(retired.receipt_phase, ReceiptPhase::ExpiredNotStarted);
        assert_eq!(retired.status, "expired_not_started");
        assert!(!retired.complete);
        assert!(!retired.needs_reconcile);
        assert!(retired.authority_phase.is_none());
        assert_eq!(retired.release_id, "release-a");
        assert_eq!(
            fixture
                .store()
                .read(&record.record_id, None)
                .unwrap()
                .revision,
            2
        );
        assert_eq!(
            serde_json::to_vec(&admin.backups(&fixture.selection()).unwrap()).unwrap(),
            backup_before
        );
        assert!(
            admin
                .authority(
                    &prepared.operation_id,
                    &fixture.receipt(&prepared.operation_id)
                )
                .unwrap()
                .is_none()
        );
        let finalized = envelope(&fixture, &prepared.operation_id);
        let stored: serde_json::Value = serde_json::from_slice(&finalized).unwrap();
        assert_eq!(stored["retention"]["state"], "retirable");
        assert!(stored["retention"]["completed_at"].as_u64().is_some());
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert!(
            admin
                .reconcile(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        let repeated = newer
            .retire_expired(&fixture.selection(), &prepared.operation_id)
            .unwrap();
        assert_eq!(repeated.status, "expired_not_started");
        assert_eq!(
            envelope(&fixture, &prepared.operation_id),
            finalized,
            "repeat cannot rewrite or renew retention age"
        );
        assert!(
            OperationStore::open_existing(&fixture.root)
                .unwrap()
                .lock()
                .unwrap()
                .update(&prepared.operation_id, &before)
                .is_err()
        );
    }
    #[test]
    fn unexpired_unknown_and_missing_intents_cannot_be_retired() {
        let fixture = Fixture::new();
        let (admin, _, prepared) = fixture.issued();
        let before = envelope(&fixture, &prepared.operation_id);
        assert!(
            admin
                .retire_expired(&fixture.selection(), &prepared.operation_id)
                .unwrap_err()
                .contains("has not expired")
        );
        assert_eq!(envelope(&fixture, &prepared.operation_id), before);
        expire(&fixture, &prepared.operation_id);
        fixture.change_receipt(&prepared.operation_id, |r| r.phase = ReceiptPhase::Unknown);
        let unknown = envelope(&fixture, &prepared.operation_id);
        assert!(
            admin
                .retire_expired(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert_eq!(envelope(&fixture, &prepared.operation_id), unknown);
        assert!(
            admin
                .retire_expired(&fixture.selection(), "op-missing")
                .is_err()
        );
        assert!(!fixture.root.join("operations/op-missing.json").exists());
    }
    #[test]
    fn retirement_requires_maintenance_and_preserves_unresolved_ownership() {
        let fixture = Fixture::new();
        let (admin, _, prepared) = fixture.issued();
        expire(&fixture, &prepared.operation_id);
        let before = envelope(&fixture, &prepared.operation_id);
        let session = SessionGuard::acquire(&fixture.root).unwrap();
        assert!(
            admin
                .retire_expired(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        drop(session);
        let marker = fixture.root.join("heavy-slot-0.unresolved");
        fs::write(&marker, b"owned").unwrap();
        assert!(
            admin
                .retire_expired(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert_eq!(fs::read(&marker).unwrap(), b"owned");
        assert_eq!(envelope(&fixture, &prepared.operation_id), before);
    }
    #[test]
    fn pending_scope_or_cross_scope_operation_authority_blocks_retirement() {
        for case in ["this_operation", "other_operation", "other_scope"] {
            let fixture = Fixture::new();
            let (admin, _, prepared) = fixture.issued();
            expire(&fixture, &prepared.operation_id);
            let before = envelope(&fixture, &prepared.operation_id);
            let scope = if case == "other_scope" {
                Scope::SharedPreferences
            } else {
                fixture.scope.clone()
            };
            let pending = if case == "other_operation" {
                operation()
            } else {
                prepared.operation_id.clone()
            };
            memory_lifecycle::set_pending_restore(
                &fixture.root,
                &scope,
                &pending,
                &prepared.snapshot_id,
            )
            .unwrap();
            assert!(
                admin
                    .retire_expired(&fixture.selection(), &prepared.operation_id)
                    .is_err(),
                "{case}"
            );
            assert_eq!(envelope(&fixture, &prepared.operation_id), before);
            assert_eq!(
                memory_lifecycle::pending_restore_binding(&fixture.root, &scope).unwrap(),
                Some((pending, prepared.snapshot_id))
            );
        }
    }
    #[test]
    fn catalog_and_canonical_restore_evidence_cannot_be_retired_as_not_started() {
        for catalog in [true, false] {
            let fixture = Fixture::new();
            let (admin, _, prepared) = fixture.issued();
            expire(&fixture, &prepared.operation_id);
            if catalog {
                let maintenance = MaintenanceGuard::try_acquire(&fixture.root).unwrap();
                // Simulate an older caller's effect before its JSON transition.
                memory_backup::restore(
                    &maintenance,
                    &fixture.backup,
                    fixture.scope.clone(),
                    &prepared.snapshot_id,
                    &prepared.operation_id,
                    BackupPolicy::default(),
                )
                .unwrap();
            } else {
                fixture
                    .store()
                    .lifecycle_connection()
                    .execute(
                        "INSERT INTO metadata(key,value) VALUES('last_restore_operation',?1)",
                        [&prepared.operation_id],
                    )
                    .unwrap();
                assert!(
                    admin
                        .authority(
                            &prepared.operation_id,
                            &fixture.receipt(&prepared.operation_id)
                        )
                        .unwrap()
                        .is_none()
                );
            }
            let before = envelope(&fixture, &prepared.operation_id);
            assert!(
                admin
                    .retire_expired(&fixture.selection(), &prepared.operation_id)
                    .is_err()
            );
            assert_eq!(envelope(&fixture, &prepared.operation_id), before);
        }
    }
    #[test]
    fn missing_canonical_or_foreign_receipt_is_preserved_not_treated_as_absence_proof() {
        let fixture = Fixture::new();
        let (admin, _, prepared) = fixture.issued();
        expire(&fixture, &prepared.operation_id);
        let store = OperationStore::open_existing(&fixture.root).unwrap();
        let foreign = br#"{"schema":1,"kind":"text_replace","status":"prepared"}"#;
        let id = store.lock().unwrap().issue(foreign).unwrap();
        assert!(admin.retire_expired(&fixture.selection(), &id).is_err());
        assert_eq!(store.read_existing(&id).unwrap(), foreign);
        let path = fixture
            .store()
            .lifecycle_connection()
            .path()
            .unwrap()
            .to_owned();
        let saved = fixture.root.join("retained-canonical.sqlite3");
        fs::rename(path, &saved).unwrap();
        let before = envelope(&fixture, &prepared.operation_id);
        assert!(
            admin
                .retire_expired(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert_eq!(envelope(&fixture, &prepared.operation_id), before);
        assert!(saved.exists());
    }
    #[test]
    fn retirement_finalization_compares_exact_inspected_payload_bytes() {
        let fixture = Fixture::new();
        let (_, _, prepared) = fixture.issued();
        expire(&fixture, &prepared.operation_id);
        let store = OperationStore::open_existing(&fixture.root).unwrap();
        let locked = store.lock().unwrap();
        let expected = locked.read(&prepared.operation_id).unwrap();
        let mut receipt = decode(&expected).unwrap();
        let rewritten = serde_json::to_vec_pretty(&receipt).unwrap();
        assert_ne!(rewritten, expected);
        // Same decoded fields are still a different inspected payload.
        locked.update(&prepared.operation_id, &rewritten).unwrap();
        receipt.phase = ReceiptPhase::ExpiredNotStarted;
        assert!(finalize_expired(&locked, &prepared.operation_id, &expected, &receipt).is_err());
        assert_eq!(locked.read(&prepared.operation_id).unwrap(), rewritten);
        let stored: serde_json::Value =
            serde_json::from_slice(&envelope(&fixture, &prepared.operation_id)).unwrap();
        assert_eq!(stored["retention"]["state"], "protected");
    }

    #[test]
    fn unknown_before_effect_cannot_be_replayed_without_authority() {
        let fixture = Fixture::new();
        let (admin, record, prepared) = fixture.issued();
        // The selected snapshot disappears after preparation but before the
        // effect owner establishes any durable restore binding.
        fs::remove_file(
            fixture
                .backup
                .join(format!("{}.sqlite3", prepared.snapshot_id)),
        )
        .unwrap();
        let result = admin
            .commit(&fixture.selection(), &prepared.operation_id)
            .unwrap();
        assert_eq!(result.status, "unknown");
        assert!(!result.complete);
        assert!(result.authority_phase.is_none());
        assert_eq!(
            fixture.receipt(&prepared.operation_id).phase,
            ReceiptPhase::Unknown
        );
        assert_eq!(
            fixture
                .store()
                .read(&record.record_id, None)
                .unwrap()
                .revision,
            2
        );
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert!(
            admin
                .reconcile(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
    }

    #[test]
    fn preparing_authority_reconciles_after_expiry_and_across_release() {
        let fixture = Fixture::new();
        let (_, record, prepared) = fixture.issued();
        let admin = MemoryAdmin::new(
            fixture.root.clone(),
            fixture.backup.clone(),
            ReleaseId::parse("release-b").unwrap(),
            BackupPolicy::default(),
        )
        .unwrap();
        fixture.change_receipt(&prepared.operation_id, |receipt| {
            receipt.phase = ReceiptPhase::Unknown;
            receipt.created_at = 0;
            receipt.expires_at = PREPARED_SECONDS;
        });
        // Model the real crash window: external authority committed, catalog
        // restore row not yet written. No new operation may be invented here.
        memory_lifecycle::set_pending_restore(
            &fixture.root,
            &fixture.scope,
            &prepared.operation_id,
            &prepared.snapshot_id,
        )
        .unwrap();
        let store = OperationStore::open_existing(&fixture.root).unwrap();
        let before = store.read_existing(&prepared.operation_id).unwrap();
        let pending = admin
            .status(&fixture.selection(), &prepared.operation_id)
            .unwrap();
        assert_eq!(pending.release_id, "release-a");
        assert_eq!(pending.authority_phase.as_deref(), Some("preparing"));
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert!(pending.needs_reconcile);
        assert!(!pending.complete);
        assert_eq!(store.read_existing(&prepared.operation_id).unwrap(), before);
        let result = admin
            .reconcile(&fixture.selection(), &prepared.operation_id)
            .unwrap();
        assert!(result.complete);
        assert!(!result.needs_reconcile);
        assert_eq!(
            fixture
                .store()
                .read(&record.record_id, None)
                .unwrap()
                .revision,
            1
        );
    }

    #[test]
    fn foreign_receipts_cross_scope_release_and_backup_root_are_preserved() {
        let fixture = Fixture::new();
        let (admin, record, prepared) = fixture.issued();
        let store = OperationStore::open_existing(&fixture.root).unwrap();
        let foreign = br#"{"schema":1,"kind":"text_replace","status":"prepared"}"#;
        let foreign_id = store.lock().unwrap().issue(foreign).unwrap();
        for id in [&foreign_id, &prepared.operation_id] {
            let other_release = MemoryAdmin::new(
                fixture.root.clone(),
                fixture.backup.clone(),
                ReleaseId::parse("release-b").unwrap(),
                BackupPolicy::default(),
            )
            .unwrap();
            assert!(other_release.commit(&fixture.selection(), id).is_err());
            assert!(other_release.reconcile(&fixture.selection(), id).is_err());
            if id == &prepared.operation_id {
                let status = other_release.status(&fixture.selection(), id).unwrap();
                assert_eq!(status.release_id, "release-a");
                assert_eq!(status.status, "prepared");
            }
        }
        assert!(admin.status(&fixture.selection(), &foreign_id).is_err());
        assert!(admin.commit(&fixture.selection(), &foreign_id).is_err());
        assert!(admin.reconcile(&fixture.selection(), &foreign_id).is_err());
        assert_eq!(store.read_existing(&foreign_id).unwrap(), foreign);
        assert!(
            admin
                .status(&Selection::SharedPreferences, &prepared.operation_id)
                .is_err()
        );
        let other_backup = fixture.root.join("other-backup");
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&other_backup)
            .unwrap();
        let other_admin = MemoryAdmin::new(
            fixture.root.clone(),
            other_backup,
            ReleaseId::parse("release-a").unwrap(),
            BackupPolicy::default(),
        )
        .unwrap();
        assert!(
            other_admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert_eq!(
            fixture
                .store()
                .read(&record.record_id, None)
                .unwrap()
                .revision,
            2
        );
        assert_eq!(
            fixture.receipt(&prepared.operation_id).phase,
            ReceiptPhase::Prepared
        );
    }

    #[test]
    fn changed_root_or_snapshot_binding_refuses_without_adopting_replacement() {
        let fixture = Fixture::new();
        let (admin, _, prepared) = fixture.issued();
        let moved = fixture.root.join("old-backup");
        fs::rename(&fixture.backup, &moved).unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&fixture.backup)
            .unwrap();
        assert!(admin.backups(&fixture.selection()).is_err());
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        fs::remove_dir(&fixture.backup).unwrap();
        fs::rename(moved, &fixture.backup).unwrap();
        memory_lifecycle::set_pending_restore(
            &fixture.root,
            &fixture.scope,
            &prepared.operation_id,
            &format!("snapshot-{}", "f".repeat(64)),
        )
        .unwrap();
        assert!(
            admin
                .status(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert!(
            admin
                .reconcile(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert_eq!(
            fixture.receipt(&prepared.operation_id).phase,
            ReceiptPhase::Prepared
        );
    }

    #[test]
    fn live_session_blocks_commit_before_unknown_receipt_transition() {
        let fixture = Fixture::new();
        let (admin, _, prepared) = fixture.issued();
        let session = SessionGuard::acquire(&fixture.root).unwrap();
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .is_err()
        );
        assert_eq!(
            fixture.receipt(&prepared.operation_id).phase,
            ReceiptPhase::Prepared
        );
        drop(session);
        assert!(
            admin
                .commit(&fixture.selection(), &prepared.operation_id)
                .unwrap()
                .complete
        );
    }
}
