//! Explicit same-scope transfer intents. A receipt is not proof of an external
//! publication or imported snapshot. Unknown work is never started again.
use super::*;
use crate::memory::transfer::{MAX_STREAM_BYTES, TransferStats};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stamp {
    device: u64,
    inode: u64,
    bytes: u64,
    modified_s: i64,
    modified_ns: i64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Direction {
    Export,
    Import,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Prepared,
    Unknown,
    Writing,
    Staged,
    Complete,
    Invalidated,
    RetiredNotStarted,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferReceipt {
    schema: u8,
    kind: String,
    direction: Direction,
    phase: Phase,
    release_id: String,
    backup_root: PathBuf,
    scope: Scope,
    checkout_id: Option<String>,
    path: PathBuf,
    parent_device: u64,
    parent_inode: u64,
    created_at: u64,
    expires_at: u64,
    scope_epoch: u64,
    stamp: Option<Stamp>,
    sha256: Option<String>,
    stats: Option<TransferStats>,
}

impl MemoryAdmin {
    pub fn transfer_prepare(
        &self,
        selection: &Selection,
        path: &Path,
        importing: bool,
    ) -> Result<serde_json::Value, String> {
        self.check_paths()?;
        let _session = SessionGuard::acquire(&self.state_root).map_err(error)?;
        let (scope, checkout_id) = self.resolve(selection)?;
        let _existing =
            MemoryStore::open_existing(&self.state_root, scope.clone()).map_err(error)?;
        let (parent_device, parent_inode) = selected_parent(path)?;
        if path.starts_with(&self.state_root) || path.starts_with(&self.backup_root) {
            return Err(
                "external transfers must use a path outside managed state and backups".into(),
            );
        }
        let (stamp, sha256) = if importing {
            let mut file = input_file(path, false)?;
            let stamp = file_stamp(&file, false)?;
            let hash = digest(&mut file)?;
            check_file(path, &stamp, false)?;
            if file_stamp(&file, false)? != stamp {
                return Err("transfer input changed during preparation".into());
            }
            (Some(stamp), Some(hash))
        } else {
            require_absent(path)?;
            (None, None)
        };
        let created_at = now()?;
        let scope_epoch = memory_lifecycle::epoch_and_exclusions(&self.state_root, &scope)
            .map_err(error)?
            .0;
        let receipt = TransferReceipt {
            schema: 1,
            kind: "memory_transfer".into(),
            direction: if importing {
                Direction::Import
            } else {
                Direction::Export
            },
            phase: Phase::Prepared,
            release_id: self.release.as_str().into(),
            backup_root: self.backup_root.clone(),
            scope,
            checkout_id,
            path: path.to_owned(),
            parent_device,
            parent_inode,
            created_at,
            expires_at: created_at
                .checked_add(PREPARED_SECONDS)
                .ok_or("transfer clock overflow")?,
            scope_epoch,
            stamp,
            sha256,
            stats: None,
        };
        let store = OperationStore::open(&self.state_root).map_err(error)?;
        let id = store
            .lock()
            .map_err(error)?
            .issue(&serde_json::to_vec(&receipt).map_err(error)?)
            .map_err(error)?;
        transfer_report(&id, &receipt, None)
    }
    pub fn transfer_status(
        &self,
        selection: &Selection,
        id: &str,
    ) -> Result<serde_json::Value, String> {
        let (receipt, _) = self.transfer_read(selection, id)?;
        let authority = self.transfer_authority(id, &receipt)?;
        transfer_report(id, &receipt, authority)
    }
    pub fn transfer_commit(
        &self,
        selection: &Selection,
        id: &str,
    ) -> Result<serde_json::Value, String> {
        self.check_paths()?;
        let maintenance = MaintenanceGuard::try_acquire(&self.state_root).map_err(error)?;
        let (mut receipt, mut payload) = self.transfer_read(selection, id)?;
        if receipt.release_id != self.release.as_str() {
            return Err("transfer commit requires its prepared release".into());
        }
        if receipt.phase == Phase::Complete {
            return self.transfer_status(selection, id);
        }
        if receipt.phase != Phase::Prepared {
            return Err("transfer already started or is unknown; inspect and reconcile the issued ID without replay".into());
        }
        if now()? >= receipt.expires_at {
            return Err("transfer preparation expired without starting".into());
        }
        check_parent(&receipt)?;
        if receipt.direction == Direction::Export {
            check_epoch(&self.state_root, &receipt)?;
        }
        let mut source = if receipt.direction == Direction::Import {
            let file = input_file(&receipt.path, false)?;
            if Some(file_stamp(&file, false)?) != receipt.stamp {
                return Err("prepared transfer input changed".into());
            }
            Some(file)
        } else {
            require_absent(&receipt.path)?;
            None
        };
        if receipt.direction == Direction::Export {
            self.admit_export_stage(id)?;
        }
        receipt.phase = Phase::Unknown;
        payload = self.transfer_update(id, &payload, &receipt, false)?;
        if let Some(source) = source.as_mut() {
            memory_backup::import_snapshot(
                &maintenance,
                &self.backup_root,
                receipt.scope.clone(),
                source,
                memory_backup::ImportInput {
                    operation_id: id,
                    sha256: receipt.sha256.as_deref().ok_or("missing import hash")?,
                    byte_len: receipt
                        .stamp
                        .as_ref()
                        .ok_or("missing import identity")?
                        .bytes,
                },
                self.policy,
            )
            .map_err(error)?;
            if Some(file_stamp(source, false)?) != receipt.stamp {
                return Err(
                    "input changed; inspect the admitted snapshot before reconciling".into(),
                );
            }
            check_parent(&receipt)?;
            receipt.phase = Phase::Complete;
            self.transfer_update(id, &payload, &receipt, true)?;
        } else {
            let stage = stage_path(&receipt, id)?;
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options
                    .mode(0o600)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            let mut file = options.open(&stage).map_err(error)?;
            file.sync_all().map_err(error)?;
            sync_parent(&receipt)?;
            receipt.stamp = Some(file_stamp(&file, false)?);
            receipt.phase = Phase::Writing;
            payload = self.transfer_update(id, &payload, &receipt, false)?;
            let stats = memory_backup::export_scope(&maintenance, receipt.scope.clone(), &mut file)
                .map_err(error)?;
            file.sync_all().map_err(error)?;
            let stamp = file_stamp(&file, false)?;
            let hash = digest(&mut file)?;
            if file_stamp(&file, false)? != stamp {
                return Err("export stage changed during inspection".into());
            }
            check_file(&stage, &stamp, false)?;
            sync_parent(&receipt)?;
            receipt.stamp = Some(stamp);
            receipt.sha256 = Some(hash);
            receipt.stats = Some(stats);
            receipt.phase = Phase::Staged;
            payload = self.transfer_update(id, &payload, &receipt, false)?;
            self.publish_transfer(id, &mut receipt, &payload)?;
        }
        self.transfer_status(selection, id)
    }
    pub fn transfer_reconcile(
        &self,
        selection: &Selection,
        id: &str,
    ) -> Result<serde_json::Value, String> {
        self.check_paths()?;
        let _maintenance = MaintenanceGuard::try_acquire(&self.state_root).map_err(error)?;
        let (mut receipt, payload) = self.transfer_read(selection, id)?;
        if receipt.phase == Phase::Prepared {
            return Err("reconciliation cannot start a prepared transfer".into());
        }
        if matches!(
            receipt.phase,
            Phase::Complete | Phase::Invalidated | Phase::RetiredNotStarted
        ) {
            return self.transfer_status(selection, id);
        }
        match receipt.direction {
            Direction::Import => {
                let bound = memory_backup::inspect_import(
                    &self.state_root,
                    &self.backup_root,
                    receipt.scope.clone(),
                    memory_backup::ImportInput {
                        operation_id: id,
                        sha256: receipt.sha256.as_deref().ok_or("missing import hash")?,
                        byte_len: receipt
                            .stamp
                            .as_ref()
                            .ok_or("missing import identity")?
                            .bytes,
                    },
                )
                .map_err(error)?;
                if bound.as_ref().is_some_and(|saved| saved.eligible) {
                    receipt.phase = Phase::Complete;
                } else {
                    let reconciled = if bound.is_some() {
                        memory_backup::reconcile_backup(
                            &self.state_root,
                            &self.backup_root,
                            receipt.scope.clone(),
                            id,
                        )
                        .map_err(error)?
                    } else {
                        None
                    };
                    // Reconciliation may revalidate an unaffected old-epoch
                    // snapshot; it never reconstructs an absent one.
                    receipt.phase = if reconciled.is_some_and(|saved| saved.eligible) {
                        Phase::Complete
                    } else {
                        Phase::Invalidated
                    };
                }
                self.transfer_update(id, &payload, &receipt, true)?;
            }
            Direction::Export => {
                if receipt.phase == Phase::Unknown {
                    inspect_empty_unbound_stage(&receipt, id)?;
                    receipt.phase = Phase::Invalidated;
                    self.transfer_update(id, &payload, &receipt, true)?;
                } else if receipt.phase == Phase::Writing {
                    remove_owned_stage(&receipt, id)?;
                    receipt.phase = Phase::Invalidated;
                    receipt.sha256 = None;
                    receipt.stats = None;
                    self.transfer_update(id, &payload, &receipt, true)?;
                } else if receipt.phase == Phase::Staged {
                    self.publish_transfer(id, &mut receipt, &payload)?;
                } else {
                    return Err(
                        "no verified staged export binding exists; unknown files are preserved"
                            .into(),
                    );
                }
            }
        }
        self.transfer_status(selection, id)
    }
    pub fn transfer_retire(
        &self,
        selection: &Selection,
        id: &str,
    ) -> Result<serde_json::Value, String> {
        self.check_paths()?;
        let _maintenance = MaintenanceGuard::try_acquire(&self.state_root).map_err(error)?;
        let (mut receipt, payload) = self.transfer_read(selection, id)?;
        if receipt.phase == Phase::RetiredNotStarted {
            return transfer_report(id, &receipt, None);
        }
        if receipt.phase != Phase::Prepared || now()? < receipt.expires_at {
            return Err("only an expired unused transfer preparation can be retired".into());
        }
        if receipt.direction == Direction::Export {
            require_absent(&stage_path(&receipt, id)?)?;
        } else if memory_backup::inspect_import(
            &self.state_root,
            &self.backup_root,
            receipt.scope.clone(),
            memory_backup::ImportInput {
                operation_id: id,
                sha256: receipt.sha256.as_deref().ok_or("missing import hash")?,
                byte_len: receipt
                    .stamp
                    .as_ref()
                    .ok_or("missing import identity")?
                    .bytes,
            },
        )
        .map_err(error)?
        .is_some()
        {
            return Err("bound import prevents preparation retirement".into());
        }
        receipt.phase = Phase::RetiredNotStarted;
        receipt.sha256 = None;
        receipt.stamp = None;
        self.transfer_update(id, &payload, &receipt, true)?;
        transfer_report(id, &receipt, None)
    }
    fn admit_export_stage(&self, current_id: &str) -> Result<(), String> {
        let store = OperationStore::open_existing(&self.state_root).map_err(error)?;
        let locked = store.lock().map_err(error)?;
        for (id, payload) in locked.memory_transfer_receipts(None).map_err(error)? {
            let r: TransferReceipt = serde_json::from_slice(&payload).map_err(error)?;
            if r.schema != 1 || r.kind != "memory_transfer" {
                return Err("unsupported transfer receipt prevents export admission".into());
            }
            if id != current_id
                && r.direction == Direction::Export
                && matches!(r.phase, Phase::Unknown | Phase::Writing | Phase::Staged)
            {
                return Err(format!(
                    "unfinished export {id} retains the installation staging slot; inspect and reconcile it before another export"
                ));
            }
        }
        Ok(())
    }
    fn transfer_read(
        &self,
        selection: &Selection,
        id: &str,
    ) -> Result<(TransferReceipt, Vec<u8>), String> {
        self.check_paths()?;
        let (scope, checkout) = self.resolve(selection)?;
        let payload = OperationStore::open_existing(&self.state_root)
            .map_err(error)?
            .read_existing(id)
            .map_err(error)?;
        let receipt: TransferReceipt = serde_json::from_slice(&payload).map_err(error)?;
        if receipt.schema != 1
            || receipt.kind != "memory_transfer"
            || receipt.scope != scope
            || receipt.checkout_id != checkout
            || receipt.backup_root != self.backup_root
        {
            return Err("transfer receipt differs from selected scope or supported schema".into());
        }
        if receipt.expires_at.checked_sub(receipt.created_at) != Some(PREPARED_SECONDS) {
            return Err("invalid transfer lifetime".into());
        }
        ReleaseId::parse(&receipt.release_id).map_err(error)?;
        // Inspecting a receipt does not follow or create its selected external path.
        Ok((receipt, payload))
    }
    fn transfer_update(
        &self,
        id: &str,
        expected: &[u8],
        receipt: &TransferReceipt,
        complete: bool,
    ) -> Result<Vec<u8>, String> {
        let store = OperationStore::open_existing(&self.state_root).map_err(error)?;
        let locked = store.lock().map_err(error)?;
        if locked.read(id).map_err(error)? != expected {
            return Err("transfer receipt changed after inspection".into());
        }
        let bytes = serde_json::to_vec(receipt).map_err(error)?;
        if complete {
            locked.finalize(id, &bytes)
        } else {
            locked.update(id, &bytes)
        }
        .map_err(error)?;
        Ok(bytes)
    }
    fn transfer_authority(
        &self,
        id: &str,
        receipt: &TransferReceipt,
    ) -> Result<Option<serde_json::Value>, String> {
        match receipt.direction {
            Direction::Import => {
                if matches!(
                    receipt.phase,
                    Phase::Prepared | Phase::Invalidated | Phase::RetiredNotStarted
                ) {
                    return Ok(None);
                }
                let saved = memory_backup::inspect_import(
                    &self.state_root,
                    &self.backup_root,
                    receipt.scope.clone(),
                    memory_backup::ImportInput {
                        operation_id: id,
                        sha256: receipt.sha256.as_deref().ok_or("missing import hash")?,
                        byte_len: receipt
                            .stamp
                            .as_ref()
                            .ok_or("missing import identity")?
                            .bytes,
                    },
                )
                .map_err(error)?;
                saved
                    .filter(|s| s.eligible)
                    .map(|s| serde_json::to_value(s).map_err(error))
                    .transpose()
            }
            Direction::Export => {
                if !matches!(receipt.phase, Phase::Staged | Phase::Complete) {
                    return Ok(None);
                }
                check_parent(receipt)?;
                match fs::symlink_metadata(&receipt.path) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(error(e)),
                    Ok(_) => {
                        verify_export_file(&receipt.path, receipt)?;
                        Ok(Some(
                            serde_json::json!({"path":receipt.path,"sha256":receipt.sha256,"stats":receipt.stats}),
                        ))
                    }
                }
            }
        }
    }
    fn publish_transfer(
        &self,
        id: &str,
        receipt: &mut TransferReceipt,
        payload: &[u8],
    ) -> Result<(), String> {
        check_parent(receipt)?;
        let stage = stage_path(receipt, id)?;
        match fs::symlink_metadata(&receipt.path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                check_epoch(&self.state_root, receipt)?;
                verify_export_file(&stage, receipt)?;
                fs::hard_link(&stage, &receipt.path).map_err(error)?;
            }
            Err(e) => return Err(error(e)),
            Ok(_) => {
                verify_export_file(&receipt.path, receipt)?;
            }
        }
        verify_export_file(&receipt.path, receipt)?;
        sync_parent(receipt)?;
        match fs::symlink_metadata(&stage) {
            Ok(_) => {
                verify_export_file(&stage, receipt)?;
                fs::remove_file(&stage).map_err(error)?;
                sync_parent(receipt)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(error(e)),
        }
        receipt.phase = Phase::Complete;
        self.transfer_update(id, payload, receipt, true)?;
        Ok(())
    }
}
fn transfer_report(
    id: &str,
    r: &TransferReceipt,
    authority: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    Ok(
        serde_json::json!({"operation_id":id,"release_id":r.release_id,"direction":r.direction,"receipt_phase":r.phase,
        "scope":r.scope,"path":r.path,"expires_at":r.expires_at,"confirmed":authority.is_some(),
        "complete":r.phase==Phase::Complete&&authority.is_some(),
        "needs_reconcile":matches!(r.phase,Phase::Unknown|Phase::Writing|Phase::Staged),"authority":authority,
        "notice":"Same-scope recovery only. Import does not activate memory. External exports are user-owned and cannot be erased by later forgetting; missing receipt evidence never authorizes replay."}),
    )
}
fn stage_path(r: &TransferReceipt, id: &str) -> Result<PathBuf, String> {
    if !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("invalid transfer operation ID".into());
    }
    Ok(r.path
        .parent()
        .ok_or("missing transfer parent")?
        .join(format!(".mirket-export-{id}.part")))
}
fn selected_parent(path: &Path) -> Result<(u64, u64), String> {
    if !path.is_absolute()
        || path
            .to_str()
            .is_none_or(|s| s.len() > 4096 || s.contains('\0'))
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err("select an absolute bounded transfer file path without traversal".into());
    }
    let parent = path.parent().ok_or("transfer needs a parent directory")?;
    if parent.canonicalize().map_err(error)? != parent {
        return Err("transfer parent must use its canonical path".into());
    }
    let metadata = fs::symlink_metadata(parent).map_err(error)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("invalid transfer parent".into());
    }
    native_identity(&metadata)
}
#[cfg(unix)]
fn native_identity(metadata: &fs::Metadata) -> Result<(u64, u64), String> {
    use std::os::unix::fs::MetadataExt;
    Ok((metadata.dev(), metadata.ino()))
}
#[cfg(not(unix))]
fn native_identity(_metadata: &fs::Metadata) -> Result<(u64, u64), String> {
    Err("native transfer identity is not qualified on this platform".into())
}
fn check_parent(r: &TransferReceipt) -> Result<(), String> {
    if selected_parent(&r.path)? != (r.parent_device, r.parent_inode) {
        return Err("transfer parent identity changed".into());
    }
    Ok(())
}
fn require_absent(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(error(e)),
        Ok(_) => Err("transfer destination already exists; no overwrite".into()),
    }
}
fn input_file(path: &Path, linked: bool) -> Result<File, String> {
    let before = fs::symlink_metadata(path).map_err(error)?;
    if !before.is_file() || before.file_type().is_symlink() {
        return Err("transfer input must be a regular file".into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).map_err(error)?;
    if native_identity(&before)? != native_identity(&file.metadata().map_err(error)?)? {
        return Err("transfer file changed while opening".into());
    }
    file_stamp(&file, linked)?;
    Ok(file)
}
fn file_stamp(file: &File, linked: bool) -> Result<Stamp, String> {
    let m = file.metadata().map_err(error)?;
    if !m.is_file() || m.len() > MAX_STREAM_BYTES {
        return Err("transfer must be a bounded regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o077 != 0
            || m.nlink() == 0
            || m.nlink() > if linked { 2 } else { 1 }
        {
            return Err(
                "transfer file requires private owner permissions and original native identity"
                    .into(),
            );
        }
        Ok(Stamp {
            device: m.dev(),
            inode: m.ino(),
            bytes: m.len(),
            modified_s: m.mtime(),
            modified_ns: m.mtime_nsec(),
        })
    }
    #[cfg(not(unix))]
    {
        let _ = linked;
        Err("native transfer identity is not qualified on this platform".into())
    }
}
fn check_file(path: &Path, stamp: &Stamp, linked: bool) -> Result<(), String> {
    if file_stamp(&input_file(path, linked)?, linked)? != *stamp {
        return Err("transfer file identity or content metadata changed".into());
    }
    Ok(())
}
fn digest(file: &mut File) -> Result<String, String> {
    file.seek(SeekFrom::Start(0)).map_err(error)?;
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut count = 0u64;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        if Instant::now() >= deadline {
            return Err("transfer hash deadline exceeded".into());
        }
        let n = file.read(&mut buffer).map_err(error)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > MAX_STREAM_BYTES {
            return Err("transfer byte limit exceeded".into());
        }
        hash.update(&buffer[..n]);
    }
    file.seek(SeekFrom::Start(0)).map_err(error)?;
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect())
}
fn verify_export_file(path: &Path, r: &TransferReceipt) -> Result<(), String> {
    let stamp = r.stamp.as_ref().ok_or("missing verified export identity")?;
    let mut file = input_file(path, true)?;
    if file_stamp(&file, true)? != *stamp
        || Some(digest(&mut file)?) != r.sha256
        || file_stamp(&file, true)? != *stamp
    {
        return Err("export identity or saved bytes changed; preserve and inspect".into());
    }
    check_file(path, stamp, true)
}
fn sync_parent(r: &TransferReceipt) -> Result<(), String> {
    check_parent(r)?;
    File::open(r.path.parent().ok_or("missing parent")?)
        .map_err(error)?
        .sync_all()
        .map_err(error)
}

fn check_epoch(state_root: &Path, r: &TransferReceipt) -> Result<(), String> {
    if memory_lifecycle::epoch_and_exclusions(state_root, &r.scope)
        .map_err(error)?
        .0
        != r.scope_epoch
    {
        return Err("scope forgetting changed after export preparation; this staged archive cannot be published".into());
    }
    Ok(())
}
fn inspect_empty_unbound_stage(r: &TransferReceipt, id: &str) -> Result<(), String> {
    check_parent(r)?;
    match fs::symlink_metadata(stage_path(r, id)?) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        // No plaintext is written until Writing identity is durable. Preserve
        // the unbound empty entry; a name alone never proves deletion ownership.
        Ok(m) if m.is_file() && !m.file_type().is_symlink() && m.len() == 0 => (),
        _ => return Err(
            "unbound nonempty export stage requires inspection; no replay or cleanup confirmation"
                .into(),
        ),
    }
    sync_parent(r)
}
fn remove_owned_stage(r: &TransferReceipt, id: &str) -> Result<(), String> {
    check_parent(r)?;
    let stage = stage_path(r, id)?;
    let file = match input_file(&stage, false) {
        Ok(file) => file,
        Err(e) => {
            if matches!(fs::symlink_metadata(&stage),Err(ref missing) if missing.kind()==std::io::ErrorKind::NotFound)
            {
                return sync_parent(r);
            }
            return Err(e);
        }
    };
    let actual = file_stamp(&file, false)?;
    let expected = r.stamp.as_ref().ok_or("missing export stage ownership")?;
    if actual.device != expected.device || actual.inode != expected.inode {
        return Err("export stage ownership changed; unknown file preserved".into());
    }
    if r.phase == Phase::Staged {
        verify_export_file(&stage, r)?;
    }
    fs::remove_file(&stage).map_err(error)?;
    sync_parent(r)
}
/// Caller holds exclusive scope access after committing forgetting. No prepared
/// transfer can be issued and no maintenance export can run concurrently.
pub(crate) fn invalidate_export_stages(root: &Path, scope: &Scope) -> Result<(), String> {
    let store = match OperationStore::open_existing(root) {
        Ok(store) => store,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(error(e)),
    };
    let locked = store.lock().map_err(error)?;
    for (id, payload) in locked
        .memory_transfer_receipts(Some(scope))
        .map_err(error)?
    {
        let mut r: TransferReceipt = serde_json::from_slice(&payload).map_err(error)?;
        if r.schema != 1 || r.kind != "memory_transfer" || &r.scope != scope {
            return Err("invalid selected transfer receipt".into());
        }
        if r.direction != Direction::Export
            || matches!(
                r.phase,
                Phase::Complete | Phase::Invalidated | Phase::RetiredNotStarted
            )
        {
            continue;
        }
        let stage = stage_path(&r, &id)?;
        match r.phase {
            Phase::Prepared => require_absent(&stage)?,
            Phase::Unknown => inspect_empty_unbound_stage(&r, &id)?,
            Phase::Writing => remove_owned_stage(&r, &id)?,
            Phase::Staged => {
                match fs::symlink_metadata(&r.path) {
                    Ok(_) => {
                        verify_export_file(&r.path, &r)?;
                        // A verified hardlink already transferred this copy to
                        // the user. Remove only its Mirket-owned sibling link.
                        match fs::symlink_metadata(&stage) {
                            Ok(_) => {
                                verify_export_file(&stage, &r)?;
                                fs::remove_file(&stage).map_err(error)?;
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                            Err(e) => return Err(error(e)),
                        }
                        sync_parent(&r)?;
                        r.phase = Phase::Complete;
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        remove_owned_stage(&r, &id)?
                    }
                    Err(e) => return Err(error(e)),
                }
            }
            _ => unreachable!(),
        }
        if r.phase != Phase::Complete {
            r.phase = Phase::Invalidated;
            r.sha256 = None;
            r.stats = None;
            r.stamp = None;
        }
        if locked.read(&id).map_err(error)? != payload {
            return Err("transfer changed during forgetting cleanup".into());
        }
        locked
            .finalize(&id, &serde_json::to_vec(&r).map_err(error)?)
            .map_err(error)?;
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::memory::{Applicability, ClaimBasis, RecordInput, RecordKind, RecordStatus, Source};
    use std::{
        os::unix::fs::{DirBuilderExt, PermissionsExt},
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        base: PathBuf,
        state: PathBuf,
        backup: PathBuf,
        project: PathBuf,
        scope: Scope,
    }
    impl Fixture {
        fn new() -> Self {
            let base = std::env::temp_dir().join(format!(
                "mirket-transfer-admin-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::DirBuilder::new().mode(0o700).create(&base).unwrap();
            let base = base.canonicalize().unwrap();
            let state = base.join("state");
            let backup = base.join("backup");
            let project = base.join("project");
            for p in [&state, &backup, &project] {
                fs::DirBuilder::new().mode(0o700).create(p).unwrap();
            }
            let selected = identity::resolve(&state, &project).unwrap();
            let scope = Scope::Project(selected.project_id);
            drop(MemoryStore::open(&state, scope.clone()).unwrap());
            Self {
                base,
                state,
                backup,
                project,
                scope,
            }
        }
        fn admin(&self) -> MemoryAdmin {
            MemoryAdmin::new(
                self.state.clone(),
                self.backup.clone(),
                ReleaseId::parse("release-transfer").unwrap(),
                BackupPolicy::default(),
            )
            .unwrap()
        }
        fn select(&self) -> Selection {
            Selection::Project {
                project_root: self.project.clone(),
            }
        }
        fn store(&self) -> MemoryStore {
            MemoryStore::open(&self.state, self.scope.clone()).unwrap()
        }
        fn export(&self) -> (String, PathBuf) {
            let path = self.base.join(format!(
                "export-{}.jsonl",
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let a = self.admin();
            let p = a.transfer_prepare(&self.select(), &path, false).unwrap();
            let id = p["operation_id"].as_str().unwrap().to_owned();
            let done = a.transfer_commit(&self.select(), &id).unwrap();
            assert_eq!(done["complete"], true);
            (id, path)
        }
        fn staged(&self, writing: bool) -> (String, PathBuf, PathBuf) {
            let a = self.admin();
            let path = self.base.join(format!(
                "staged-{}.jsonl",
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let p = a.transfer_prepare(&self.select(), &path, false).unwrap();
            let id = p["operation_id"].as_str().unwrap().to_owned();
            let (mut receipt, payload) = a.transfer_read(&self.select(), &id).unwrap();
            let stage = stage_path(&receipt, &id).unwrap();
            let mut options = OpenOptions::new();
            options.read(true).write(true).create_new(true);
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
            let mut file = options.open(&stage).unwrap();
            receipt.stamp = Some(file_stamp(&file, false).unwrap());
            receipt.phase = Phase::Writing;
            let payload = a.transfer_update(&id, &payload, &receipt, false).unwrap();
            let maintenance = MaintenanceGuard::try_acquire(&self.state).unwrap();
            let stats =
                memory_backup::export_scope(&maintenance, self.scope.clone(), &mut file).unwrap();
            drop(maintenance);
            file.sync_all().unwrap();
            if !writing {
                receipt.stamp = Some(file_stamp(&file, false).unwrap());
                receipt.sha256 = Some(digest(&mut file).unwrap());
                receipt.stats = Some(stats);
                receipt.phase = Phase::Staged;
                a.transfer_update(&id, &payload, &receipt, false).unwrap();
            }
            (id, path, stage)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }
    fn input(content: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::Observed,
            content: content.into(),
            status: RecordStatus::Current,
            critical: false,
            applicability: Applicability::default(),
            valid_from: None,
            valid_until: None,
            sources: vec![Source {
                source_id: "fixture".into(),
                revision: "rev-a".into(),
                locator: "lines:1-3".into(),
                captured_at: 1,
            }],
        }
    }
    #[test]
    fn transfer_roundtrip_admits_without_activation_then_reuses_scoped_restore() {
        let f = Fixture::new();
        let saved = f
            .store()
            .create("op-transfer-original", input("earlier evidence"), None)
            .unwrap();
        let (export_id, path) = f.export();
        let a = f.admin();
        assert_eq!(
            a.transfer_commit(&f.select(), &export_id).unwrap()["complete"],
            true
        );
        f.store()
            .revise(
                "op-transfer-newer",
                &saved.record_id,
                1,
                input("newer evidence"),
            )
            .unwrap();
        let p = a.transfer_prepare(&f.select(), &path, true).unwrap();
        let id = p["operation_id"].as_str().unwrap();
        assert!(a.transfer_reconcile(&f.select(), id).is_err());
        let done = a.transfer_commit(&f.select(), id).unwrap();
        assert_eq!(done["complete"], true);
        assert_eq!(
            f.store().read(&saved.record_id, None).unwrap().data.content,
            "newer evidence"
        );
        let snapshot = done["authority"]["snapshot_id"].as_str().unwrap();
        let restore = a.prepare(&f.select(), snapshot).unwrap();
        assert!(
            a.commit(&f.select(), &restore.operation_id)
                .unwrap()
                .complete
        );
        assert_eq!(f.store().read(&saved.record_id, None).unwrap(), saved);
        assert!(
            f.store()
                .read_mutation("op-transfer-original")
                .unwrap()
                .is_none()
        );
        fs::remove_file(path).unwrap();
        let status = a.transfer_status(&f.select(), &export_id).unwrap();
        assert_eq!(status["complete"], false);
        assert_eq!(status["confirmed"], false);
        assert_eq!(
            a.transfer_commit(&f.select(), &export_id).unwrap()["confirmed"],
            false
        );
    }
    #[test]
    fn interrupted_forgetting_revalidates_unaffected_import_without_replaying_it() {
        let f = Fixture::new();
        f.store()
            .create(
                "op-transfer-original",
                input("retained archive evidence"),
                None,
            )
            .unwrap();
        let (_, path) = f.export();
        let later = f
            .store()
            .create("op-transfer-later", input("forget later record"), None)
            .unwrap();
        let a = f.admin();
        let prepared = a.transfer_prepare(&f.select(), &path, true).unwrap();
        let id = prepared["operation_id"].as_str().unwrap();
        let (mut receipt, payload) = a.transfer_read(&f.select(), id).unwrap();
        receipt.phase = Phase::Unknown;
        a.transfer_update(id, &payload, &receipt, false).unwrap();
        let mut source = input_file(&path, false).unwrap();
        let maintenance = MaintenanceGuard::try_acquire(&f.state).unwrap();
        let imported = memory_backup::import_snapshot(
            &maintenance,
            &f.backup,
            f.scope.clone(),
            &mut source,
            memory_backup::ImportInput {
                operation_id: id,
                sha256: receipt.sha256.as_deref().unwrap(),
                byte_len: receipt.stamp.as_ref().unwrap().bytes,
            },
            BackupPolicy::default(),
        )
        .unwrap();
        drop(maintenance);
        let lock = File::open(f.backup.join("catalog.lock")).unwrap();
        lock.lock().unwrap();
        assert!(
            memory_lifecycle::forget(
                &f.state,
                f.scope.clone(),
                &later.record_id,
                "op-transfer-forget-later",
                1
            )
            .is_err()
        );
        lock.unlock().unwrap();
        drop(lock);
        let done = a.transfer_reconcile(&f.select(), id).unwrap();
        assert_eq!(done["complete"], true);
        assert_eq!(done["authority"]["snapshot_id"], imported.snapshot_id);
        assert!(f.store().read(&later.record_id, None).is_err());
        assert!(
            memory_lifecycle::reconcile(&f.state, f.scope.clone(), &later.record_id)
                .unwrap()
                .cleanup_complete
        );
    }
    #[test]
    fn unfinished_export_blocks_another_scope_until_explicit_owned_cleanup() {
        let f = Fixture::new();
        f.store()
            .create("op-transfer-original", input("bounded stage"), None)
            .unwrap();
        let (old, _, stage) = f.staged(true);
        let other = f.base.join("other-project");
        fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
        let selected = identity::resolve(&f.state, &other).unwrap();
        drop(MemoryStore::open(&f.state, Scope::Project(selected.project_id)).unwrap());
        let selection = Selection::Project {
            project_root: other,
        };
        let a = f.admin();
        let out = f.base.join("next.jsonl");
        let p = a.transfer_prepare(&selection, &out, false).unwrap();
        let id = p["operation_id"].as_str().unwrap();
        assert!(
            a.transfer_commit(&selection, id)
                .unwrap_err()
                .contains("installation staging slot")
        );
        assert!(!out.exists());
        assert_eq!(
            a.transfer_status(&selection, id).unwrap()["receipt_phase"],
            "prepared"
        );
        assert_eq!(
            a.transfer_reconcile(&f.select(), &old).unwrap()["receipt_phase"],
            "invalidated"
        );
        assert!(!stage.exists());
        assert_eq!(a.transfer_commit(&selection, id).unwrap()["complete"], true);
    }
    #[test]
    fn unknown_export_without_plaintext_can_be_consumed_but_nonempty_unknown_is_preserved() {
        for nonempty in [false, true] {
            let f = Fixture::new();
            let a = f.admin();
            let out = f.base.join("never-published.jsonl");
            let p = a.transfer_prepare(&f.select(), &out, false).unwrap();
            let id = p["operation_id"].as_str().unwrap();
            let (mut receipt, payload) = a.transfer_read(&f.select(), id).unwrap();
            receipt.phase = Phase::Unknown;
            a.transfer_update(id, &payload, &receipt, false).unwrap();
            let stage = stage_path(&receipt, id).unwrap();
            fs::write(&stage, if nonempty { "unowned contents" } else { "" }).unwrap();
            if nonempty {
                assert!(a.transfer_reconcile(&f.select(), id).is_err());
            } else {
                assert_eq!(
                    a.transfer_reconcile(&f.select(), id).unwrap()["receipt_phase"],
                    "invalidated"
                );
            }
            assert!(stage.exists());
            assert!(!out.exists());
        }
    }
    #[test]
    fn changed_input_and_destination_conflict_do_not_start_transfer() {
        let f = Fixture::new();
        f.store()
            .create("op-transfer-original", input("evidence"), None)
            .unwrap();
        let (_, path) = f.export();
        let a = f.admin();
        let p = a.transfer_prepare(&f.select(), &path, true).unwrap();
        let id = p["operation_id"].as_str().unwrap();
        fs::write(&path, "changed").unwrap();
        assert!(
            a.transfer_commit(&f.select(), id)
                .unwrap_err()
                .contains("changed")
        );
        assert_eq!(
            a.transfer_status(&f.select(), id).unwrap()["receipt_phase"],
            "prepared"
        );
        let out = f.base.join("conflict.jsonl");
        let p = a.transfer_prepare(&f.select(), &out, false).unwrap();
        fs::write(&out, "unrelated owner").unwrap();
        assert!(
            a.transfer_commit(&f.select(), p["operation_id"].as_str().unwrap())
                .unwrap_err()
                .contains("already exists")
        );
        assert_eq!(fs::read_to_string(out).unwrap(), "unrelated owner");
    }
    #[test]
    fn forgetting_removes_owned_unpublished_writing_and_staged_exports() {
        for writing in [true, false] {
            let f = Fixture::new();
            let record = f
                .store()
                .create(
                    "op-transfer-secret",
                    input("erase this private memory"),
                    None,
                )
                .unwrap();
            let (id, path, stage) = f.staged(writing);
            assert!(
                fs::read_to_string(&stage)
                    .unwrap()
                    .contains("erase this private memory")
            );
            assert!(
                memory_lifecycle::forget(
                    &f.state,
                    f.scope.clone(),
                    &record.record_id,
                    "op-transfer-forget",
                    1
                )
                .unwrap()
                .cleanup_complete
            );
            assert!(!stage.exists());
            assert!(!path.exists());
            let a = f.admin();
            assert_eq!(
                a.transfer_status(&f.select(), &id).unwrap()["receipt_phase"],
                "invalidated"
            );
            assert!(
                !a.transfer_reconcile(&f.select(), &id).unwrap()["confirmed"]
                    .as_bool()
                    .unwrap()
            );
            assert!(a.transfer_commit(&f.select(), &id).is_err());
        }
    }
    #[test]
    fn forgetting_preserves_already_published_external_copy_but_removes_stage_link() {
        let f = Fixture::new();
        let record = f
            .store()
            .create("op-transfer-secret", input("external copy limit"), None)
            .unwrap();
        let (id, path, stage) = f.staged(false);
        fs::hard_link(&stage, &path).unwrap();
        assert!(
            memory_lifecycle::forget(
                &f.state,
                f.scope.clone(),
                &record.record_id,
                "op-transfer-forget",
                1
            )
            .unwrap()
            .cleanup_complete
        );
        assert!(!stage.exists());
        assert!(
            fs::read_to_string(path)
                .unwrap()
                .contains("external copy limit")
        );
        assert_eq!(
            f.admin().transfer_status(&f.select(), &id).unwrap()["complete"],
            true
        );
    }
    #[test]
    fn lost_export_response_reconciles_exact_stage_without_reexporting_newer_memory() {
        let f = Fixture::new();
        let record = f
            .store()
            .create("op-transfer-original", input("frozen archive"), None)
            .unwrap();
        let (id, path, stage) = f.staged(false);
        f.store()
            .revise(
                "op-transfer-revision",
                &record.record_id,
                1,
                input("later source"),
            )
            .unwrap();
        let a = f.admin();
        assert_eq!(
            a.transfer_reconcile(&f.select(), &id).unwrap()["complete"],
            true
        );
        assert!(!stage.exists());
        let bytes = fs::read_to_string(path).unwrap();
        assert!(bytes.contains("frozen archive"));
        assert!(!bytes.contains("later source"));
    }
    #[test]
    fn old_import_after_forgetting_is_consumed_without_replay_or_plaintext_residue() {
        let f = Fixture::new();
        let record = f
            .store()
            .create(
                "op-transfer-secret",
                input("forgotten payload sentinel"),
                None,
            )
            .unwrap();
        let (_, path) = f.export();
        memory_lifecycle::forget(
            &f.state,
            f.scope.clone(),
            &record.record_id,
            "op-transfer-forget",
            1,
        )
        .unwrap();
        let a = f.admin();
        let p = a.transfer_prepare(&f.select(), &path, true).unwrap();
        let id = p["operation_id"].as_str().unwrap();
        assert!(a.transfer_commit(&f.select(), id).is_err());
        assert_eq!(
            a.transfer_reconcile(&f.select(), id).unwrap()["receipt_phase"],
            "invalidated"
        );
        assert!(a.transfer_commit(&f.select(), id).is_err());
        assert!(f.store().read(&record.record_id, None).is_err());
        for entry in fs::read_dir(&f.backup).unwrap() {
            let bytes = fs::read(entry.unwrap().path()).unwrap();
            assert!(
                !bytes
                    .windows(26)
                    .any(|w| w == b"forgotten payload sentinel")
            );
        }
    }
    #[test]
    fn expired_preparation_retirement_is_idempotent_and_no_started_effect_is_retired() {
        let f = Fixture::new();
        let a = f.admin();
        let path = f.base.join("unused.jsonl");
        let p = a.transfer_prepare(&f.select(), &path, false).unwrap();
        let id = p["operation_id"].as_str().unwrap();
        assert!(a.transfer_retire(&f.select(), id).is_err());
        let (mut r, payload) = a.transfer_read(&f.select(), id).unwrap();
        r.created_at = now().unwrap() - 600;
        r.expires_at = r.created_at + PREPARED_SECONDS;
        a.transfer_update(id, &payload, &r, false).unwrap();
        assert_eq!(
            a.transfer_retire(&f.select(), id).unwrap()["receipt_phase"],
            "retired_not_started"
        );
        let bytes = fs::read(f.state.join("operations").join(format!("{id}.json"))).unwrap();
        a.transfer_retire(&f.select(), id).unwrap();
        assert_eq!(
            fs::read(f.state.join("operations").join(format!("{id}.json"))).unwrap(),
            bytes
        );
        assert!(!path.exists());
        assert!(a.transfer_commit(&f.select(), id).is_err());
    }
    #[test]
    fn staged_identity_drift_blocks_cleanup_without_deleting_unknown_file() {
        let f = Fixture::new();
        let record = f
            .store()
            .create("op-transfer-secret", input("sensitive fixture"), None)
            .unwrap();
        let (_, _, stage) = f.staged(true);
        fs::rename(&stage, f.base.join("moved-original")).unwrap();
        fs::write(&stage, "unrelated replacement").unwrap();
        fs::set_permissions(&stage, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(
            memory_lifecycle::forget(
                &f.state,
                f.scope.clone(),
                &record.record_id,
                "op-transfer-forget",
                1
            )
            .is_err()
        );
        assert_eq!(fs::read_to_string(stage).unwrap(), "unrelated replacement");
        assert!(f.store().read(&record.record_id, None).is_err());
    }
}
