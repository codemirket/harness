//! Installation-wide M0 admission: two heavy slots and eight waiting permits.
//!
//! The caller explicitly selects an existing, dedicated private state directory
//! owned by the installing user. Unix group/other access is refused; std cannot
//! establish Windows ACL ownership, so the installation boundary must qualify
//! that requirement. This is one resource category, not combined reservations.
//!
//! Lockfiles remain in place permanently and are never truncated or removed.
//! Every acquisition opens an independent descriptor. No PID liveness inference,
//! daemon, polling loop or replacement lock inode is used. A rogue actor who can
//! replace files in this private directory can defeat advisory locking; detected
//! symlinks/aliases are rejected but path checks are not hostile-race containment.
//! A lease belongs to its process: it does not establish descendant termination
//! or permit reclaiming resources still owned by an untracked child. The process
//! owner must retain admission until its actual work has stopped.

use crate::file_lock::FileLock;
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions, TryLockError};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub const HEAVY_JOB_SLOTS: usize = 2;
pub const WAITING_JOB_SLOTS: usize = 8;
const WAITING_NAMES: [&str; WAITING_JOB_SLOTS] = [
    "waiting-slot-0.lock",
    "waiting-slot-1.lock",
    "waiting-slot-2.lock",
    "waiting-slot-3.lock",
    "waiting-slot-4.lock",
    "waiting-slot-5.lock",
    "waiting-slot-6.lock",
    "waiting-slot-7.lock",
];
const SLOT_NAMES: [&str; HEAVY_JOB_SLOTS] = ["heavy-slot-0.lock", "heavy-slot-1.lock"];

#[derive(Debug)]
pub enum AdmissionError {
    RootMustBeAbsolute,
    RootMustBeDirectory,
    PrivateStateRequired,
    Symlink,
    InvalidLockFile,
    StateChanged,
    InvalidOperationIdentity,
    Busy,
    WaitingFull,
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeAbsolute => {
                write!(f, "admission requires an explicit absolute state root")
            }
            Self::RootMustBeDirectory => {
                write!(f, "admission state root must be an existing directory")
            }
            Self::PrivateStateRequired => write!(
                f,
                "admission requires private user-owned state without group/other access"
            ),
            Self::Symlink => write!(f, "admission root and lockfiles cannot be symlinks"),
            Self::InvalidLockFile => write!(
                f,
                "admission lockfile must be a private, empty, unaliased regular file"
            ),
            Self::StateChanged => write!(
                f,
                "admission state changed; inspect installation state before retrying"
            ),
            Self::InvalidOperationIdentity => write!(
                f,
                "unresolved ownership requires a bounded operation identity"
            ),
            Self::Busy => write!(f, "both heavy-job slots are busy"),
            Self::WaitingFull => write!(f, "all eight waiting permits are busy"),
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl std::error::Error for AdmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(operation: &'static str, source: io::Error) -> AdmissionError {
    AdmissionError::Io { operation, source }
}

pub struct Admission {
    root: PathBuf,
    root_metadata: Metadata,
}

/// Keeping this value alive holds one OS lock. Drop explicitly unlocks its
/// independently opened descriptor. No File handle is exposed or cloned.
pub struct SlotLease {
    _file: FileLock,
    slot_index: usize,
    root: PathBuf,
    root_metadata: Metadata,
    marker: Option<OwnershipMarker>,
}

/// A waiting permit owns no heavy resource and authorizes no child launch.
/// Drop releases its OS lock; unlike a heavy lease, it never creates a durable
/// ownership marker. The worker must drop it on promotion, cancellation or timeout.
pub struct WaitingLease {
    _file: FileLock,
    slot_index: usize,
}

impl WaitingLease {
    pub fn slot_index(&self) -> usize {
        self.slot_index
    }
}

struct OwnershipMarker {
    metadata: Metadata,
    bytes: Vec<u8>,
}

impl SlotLease {
    pub fn slot_index(&self) -> usize {
        self.slot_index
    }

    /// Persist before any child launch. The marker deliberately survives Drop,
    /// process death and OS lock release; no PID observation clears it.
    pub fn mark_unresolved(&mut self, operation_id: &str) -> Result<(), AdmissionError> {
        if operation_id.is_empty()
            || operation_id.len() > 256
            || !operation_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
        {
            return Err(AdmissionError::InvalidOperationIdentity);
        }
        self.recheck_state()?;
        if self.marker.is_some() {
            return Err(AdmissionError::StateChanged);
        }
        let path = self.marker_path();
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&path)
            .map_err(|e| io_error("create unresolved ownership marker", e))?;
        let bytes = format!("mirket-unresolved-v1\n{operation_id}\n").into_bytes();
        // Any failure retains the marker. No process is authorized to launch
        // unless both file and directory sync complete successfully.
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| io_error("persist unresolved ownership marker", e))?;
        let metadata = file
            .metadata()
            .map_err(|e| io_error("inspect unresolved ownership marker", e))?;
        let path_metadata = fs::symlink_metadata(&path)
            .map_err(|e| io_error("recheck unresolved ownership marker", e))?;
        validate_marker_metadata(&path_metadata)?;
        if !same_file(&metadata, &path_metadata) {
            return Err(AdmissionError::StateChanged);
        }
        sync_directory(&self.root)?;
        self.marker = Some(OwnershipMarker { metadata, bytes });
        Ok(())
    }

    /// Caller may invoke only after proving its owned child stopped (or before
    /// launch). This clears only this lease's unchanged marker. Existing unknown
    /// markers have no automatic reclaim route in M0.
    pub fn clear_after_cleanup(&mut self) -> Result<(), AdmissionError> {
        self.recheck_state()?;
        let expected = self.marker.as_ref().ok_or(AdmissionError::StateChanged)?;
        let path = self.marker_path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| io_error("inspect ownership marker before cleanup", e))?;
        validate_marker_metadata(&metadata)?;
        if !same_marker(&metadata, &expected.metadata) {
            return Err(AdmissionError::StateChanged);
        }
        let mut file =
            File::open(&path).map_err(|e| io_error("open ownership marker for cleanup", e))?;
        let mut bytes = Vec::new();
        (&mut file)
            .take(512)
            .read_to_end(&mut bytes)
            .map_err(|e| io_error("read ownership marker for cleanup", e))?;
        let after = fs::symlink_metadata(&path)
            .map_err(|e| io_error("recheck ownership marker before cleanup", e))?;
        if bytes != expected.bytes
            || !same_marker(&after, &expected.metadata)
            || !same_marker(
                &file
                    .metadata()
                    .map_err(|e| io_error("recheck open ownership marker", e))?,
                &expected.metadata,
            )
        {
            return Err(AdmissionError::StateChanged);
        }
        drop(file);
        fs::remove_file(&path).map_err(|e| io_error("remove resolved ownership marker", e))?;
        sync_directory(&self.root)?;
        self.marker = None;
        Ok(())
    }

    fn marker_path(&self) -> PathBuf {
        marker_path(&self.root, self.slot_index)
    }

    fn recheck_state(&self) -> Result<(), AdmissionError> {
        if fs::canonicalize(&self.root).map_err(|e| io_error("recheck lease state ancestry", e))?
            != self.root
            || !same_file(&validate_root(&self.root)?, &self.root_metadata)
        {
            return Err(AdmissionError::StateChanged);
        }
        validate_open_file(
            &self.root.join(SLOT_NAMES[self.slot_index]),
            self._file.as_file(),
        )
    }
}

impl Admission {
    pub(crate) fn state_root(&self) -> &Path {
        &self.root
    }

    pub fn open(state_root: &Path) -> Result<Self, AdmissionError> {
        if !state_root.is_absolute() {
            return Err(AdmissionError::RootMustBeAbsolute);
        }
        validate_root(state_root)?;
        let root =
            fs::canonicalize(state_root).map_err(|e| io_error("resolve admission root", e))?;
        let root_metadata = validate_root(&root)?;
        let admission = Self {
            root,
            root_metadata,
        };
        // Validate the complete fixed slot set before returning. This does not
        // acquire a reservation. Existing files are never modified.
        for name in SLOT_NAMES.into_iter().chain(WAITING_NAMES) {
            drop(admission.open_slot(name)?);
        }
        Ok(admission)
    }

    pub fn try_acquire(&self) -> Result<SlotLease, AdmissionError> {
        self.recheck_root()?;
        for (slot_index, name) in SLOT_NAMES.iter().enumerate() {
            let file = self.open_slot(name)?;
            match file.try_lock() {
                Ok(()) => {
                    let file = FileLock::acquired(file);
                    // Detect an ordinary path substitution across lock-taking.
                    // A private-state assumption remains necessary after this.
                    self.recheck_root()?;
                    validate_open_file(&self.root.join(name), file.as_file())?;
                    match fs::symlink_metadata(marker_path(&self.root, slot_index)) {
                        Ok(metadata) => {
                            validate_marker_metadata(&metadata)?;
                            continue;
                        }
                        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
                        Err(error) => {
                            return Err(io_error("inspect unresolved slot ownership", error));
                        }
                    }
                    return Ok(SlotLease {
                        _file: file,
                        slot_index,
                        root: self.root.clone(),
                        root_metadata: self.root_metadata.clone(),
                        marker: None,
                    });
                }
                Err(TryLockError::WouldBlock) => continue,
                Err(TryLockError::Error(error)) => {
                    return Err(io_error("acquire heavy-job slot", error));
                }
            }
        }
        Err(AdmissionError::Busy)
    }

    /// Try once; callers own cancellation, waiting deadline and promotion.
    /// Only native lock contention is capacity exhaustion. Any I/O or identity
    /// failure is returned rather than treating an unknown slot as available.
    pub fn try_waiting(&self) -> Result<WaitingLease, AdmissionError> {
        self.recheck_root()?;
        for (slot_index, name) in WAITING_NAMES.iter().enumerate() {
            let file = self.open_slot(name)?;
            match file.try_lock() {
                Ok(()) => {
                    let file = FileLock::acquired(file);
                    self.recheck_root()?;
                    validate_open_file(&self.root.join(name), file.as_file())?;
                    return Ok(WaitingLease {
                        _file: file,
                        slot_index,
                    });
                }
                Err(TryLockError::WouldBlock) => continue,
                Err(TryLockError::Error(error)) => {
                    return Err(io_error("acquire waiting permit", error));
                }
            }
        }
        Err(AdmissionError::WaitingFull)
    }

    /// Observe live exclusive waiting leases without treating another observer
    /// as demand. Unlocked permanent files and dead owners are not waiters.
    pub fn has_waiters(&self) -> Result<bool, AdmissionError> {
        self.recheck_root()?;
        let mut waiting = false;
        for name in WAITING_NAMES {
            let file = self.open_slot(name)?;
            match file.try_lock_shared() {
                Ok(()) => {
                    let file = FileLock::acquired(file);
                    self.recheck_root()?;
                    validate_open_file(&self.root.join(name), file.as_file())?;
                }
                Err(TryLockError::WouldBlock) => waiting = true,
                Err(TryLockError::Error(error)) => {
                    return Err(io_error("inspect waiting permit", error));
                }
            }
        }
        self.recheck_root()?;
        Ok(waiting)
    }

    fn recheck_root(&self) -> Result<(), AdmissionError> {
        if fs::canonicalize(&self.root).map_err(|e| io_error("recheck state ancestry", e))?
            != self.root
        {
            return Err(AdmissionError::StateChanged);
        }
        let current = validate_root(&self.root)?;
        if !same_file(&self.root_metadata, &current) {
            return Err(AdmissionError::StateChanged);
        }
        Ok(())
    }

    fn open_slot(&self, name: &str) -> Result<File, AdmissionError> {
        self.recheck_root()?;
        let path = self.root.join(name);
        let mut create = OpenOptions::new();
        create.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            create.mode(0o600);
        }
        let file = match create.open(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                validate_lock_metadata(
                    &fs::symlink_metadata(&path)
                        .map_err(|e| io_error("inspect existing slot", e))?,
                )?;
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&path)
                    .map_err(|e| io_error("open existing slot", e))?
            }
            Err(e) => return Err(io_error("create admission slot", e)),
        };
        validate_open_file(&path, &file)?;
        Ok(file)
    }
}

fn marker_path(root: &Path, slot_index: usize) -> PathBuf {
    root.join(format!("heavy-slot-{slot_index}.unresolved"))
}

fn validate_marker_metadata(metadata: &Metadata) -> Result<(), AdmissionError> {
    if metadata.file_type().is_symlink() {
        return Err(AdmissionError::Symlink);
    }
    if !metadata.is_file() || metadata.len() > 512 {
        return Err(AdmissionError::StateChanged);
    }
    require_private(metadata)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(AdmissionError::StateChanged);
        }
    }
    Ok(())
}

fn same_marker(a: &Metadata, b: &Metadata) -> bool {
    if !same_file(a, b) || a.len() != b.len() || a.modified().ok() != b.modified().ok() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (a.ctime(), a.ctime_nsec()) != (b.ctime(), b.ctime_nsec()) {
            return false;
        }
    }
    true
}

fn sync_directory(root: &Path) -> Result<(), AdmissionError> {
    // Directory fsync is qualified on the local Unix slice. Windows needs its
    // own handle/flush qualification before this launch prerequisite can pass.
    File::open(root)
        .and_then(|directory| directory.sync_all())
        .map_err(|e| io_error("sync admission state directory", e))
}

fn validate_root(path: &Path) -> Result<Metadata, AdmissionError> {
    let metadata = fs::symlink_metadata(path).map_err(|e| io_error("inspect admission root", e))?;
    if metadata.file_type().is_symlink() {
        return Err(AdmissionError::Symlink);
    }
    if !metadata.is_dir() {
        return Err(AdmissionError::RootMustBeDirectory);
    }
    require_private(&metadata)?;
    Ok(metadata)
}

fn require_private(metadata: &Metadata) -> Result<(), AdmissionError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(AdmissionError::PrivateStateRequired);
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    Ok(())
}

fn validate_lock_metadata(metadata: &Metadata) -> Result<(), AdmissionError> {
    if metadata.file_type().is_symlink() {
        return Err(AdmissionError::Symlink);
    }
    if !metadata.is_file() || metadata.len() != 0 {
        return Err(AdmissionError::InvalidLockFile);
    }
    require_private(metadata)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(AdmissionError::InvalidLockFile);
        }
    }
    Ok(())
}

fn validate_open_file(path: &Path, file: &File) -> Result<(), AdmissionError> {
    let path_metadata = fs::symlink_metadata(path).map_err(|e| io_error("recheck slot path", e))?;
    let opened_metadata = file
        .metadata()
        .map_err(|e| io_error("inspect open slot", e))?;
    validate_lock_metadata(&path_metadata)?;
    validate_lock_metadata(&opened_metadata)?;
    if !same_file(&path_metadata, &opened_metadata) {
        return Err(AdmissionError::StateChanged);
    }
    Ok(())
}

fn same_file(a: &Metadata, b: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        (a.dev(), a.ino()) == (b.dev(), b.ino())
    }
    #[cfg(not(unix))]
    {
        // Native Windows file identity is still a qualification gap. This
        // metadata check is not presented as equivalent to Unix dev/inode.
        a.created().ok() == b.created().ok() && a.is_dir() == b.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::process::{Child, Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "mirket-leases-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&path).unwrap();
            Self(fs::canonicalize(path).unwrap())
        }
        fn admission(&self) -> Admission {
            Admission::open(&self.0).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn separate_descriptors_allow_exactly_two_leases_and_drop_releases() {
        let fixture = Fixture::new();
        let one = fixture.admission();
        let two = fixture.admission();
        let lease_a = one.try_acquire().unwrap();
        let lease_b = two.try_acquire().unwrap();
        assert_ne!(lease_a.slot_index(), lease_b.slot_index());
        assert!(matches!(one.try_acquire(), Err(AdmissionError::Busy)));
        assert!(matches!(two.try_acquire(), Err(AdmissionError::Busy)));
        let freed = lease_a.slot_index();
        drop(lease_a);
        assert_eq!(two.try_acquire().unwrap().slot_index(), freed);
        drop(lease_b);
        for name in SLOT_NAMES {
            assert_eq!(fs::metadata(fixture.0.join(name)).unwrap().len(), 0);
        }
    }

    #[test]
    fn waiter_observation_counts_only_live_exclusive_leases() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        assert!(!admission.has_waiters().unwrap());
        let observer = admission.open_slot(WAITING_NAMES[0]).unwrap();
        observer.try_lock_shared().unwrap();
        let observer = FileLock::acquired(observer);
        assert!(
            !admission.has_waiters().unwrap(),
            "another shared observer is not demand"
        );
        let waiter = admission.try_waiting().unwrap();
        assert!(admission.has_waiters().unwrap());
        drop(waiter);
        assert!(!admission.has_waiters().unwrap());
        drop(observer);
        assert!(!admission.has_waiters().unwrap());
    }

    #[test]
    fn root_selection_and_invalid_slot_do_not_fall_back_to_capacity() {
        assert!(matches!(
            Admission::open(Path::new(".")),
            Err(AdmissionError::RootMustBeAbsolute)
        ));
        let fixture = Fixture::new();
        let admission = fixture.admission();
        fs::write(fixture.0.join(SLOT_NAMES[0]), "unrelated bytes").unwrap();
        assert!(matches!(
            admission.try_acquire(),
            Err(AdmissionError::InvalidLockFile)
        ));
        assert_eq!(
            fs::read_to_string(fixture.0.join(SLOT_NAMES[0])).unwrap(),
            "unrelated bytes"
        );
        assert!(matches!(
            Admission::open(&fixture.0),
            Err(AdmissionError::InvalidLockFile)
        ));
    }

    #[test]
    fn unresolved_marker_survives_drop_and_blocks_only_its_slot() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let mut unresolved = admission.try_acquire().unwrap();
        let reserved = unresolved.slot_index();
        unresolved.mark_unresolved("operation-1").unwrap();
        drop(unresolved);
        let remaining = admission.try_acquire().unwrap();
        assert_ne!(remaining.slot_index(), reserved);
        assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
        assert!(marker_path(&fixture.0, reserved).exists());
    }

    #[test]
    fn proven_cleanup_clears_own_marker_and_restores_capacity() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let mut lease = admission.try_acquire().unwrap();
        let reserved = lease.slot_index();
        assert!(matches!(
            lease.mark_unresolved("../invalid"),
            Err(AdmissionError::InvalidOperationIdentity)
        ));
        lease.mark_unresolved("operation-2").unwrap();
        assert!(lease.mark_unresolved("operation-3").is_err());
        lease.clear_after_cleanup().unwrap();
        assert!(!marker_path(&fixture.0, reserved).exists());
        drop(lease);
        let first = admission.try_acquire().unwrap();
        let second = admission.try_acquire().unwrap();
        assert_ne!(first.slot_index(), second.slot_index());
    }

    #[test]
    fn marker_drift_is_not_removed_as_owned_cleanup() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let mut lease = admission.try_acquire().unwrap();
        lease.mark_unresolved("operation-4").unwrap();
        let path = marker_path(&fixture.0, lease.slot_index());
        fs::write(&path, "another operation").unwrap();
        assert!(matches!(
            lease.clear_after_cleanup(),
            Err(AdmissionError::StateChanged)
        ));
        assert_eq!(fs::read_to_string(path).unwrap(), "another operation");
    }

    #[cfg(unix)]
    #[test]
    fn unresolved_marker_symlink_is_not_followed_or_reclaimed() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let target = fixture.0.join("outside-marker");
        fs::write(&target, "keep").unwrap();
        symlink(&target, marker_path(&fixture.0, 0)).unwrap();
        assert!(matches!(
            admission.try_acquire(),
            Err(AdmissionError::Symlink)
        ));
        assert_eq!(fs::read_to_string(target).unwrap(), "keep");
    }

    #[cfg(unix)]
    #[test]
    fn insecure_state_symlinks_and_hard_link_aliases_are_rejected() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let fixture = Fixture::new();
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            Admission::open(&fixture.0),
            Err(AdmissionError::PrivateStateRequired)
        ));
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o700)).unwrap();
        let other = Fixture::new();
        symlink(&other.0, fixture.0.join("root-alias")).unwrap();
        assert!(matches!(
            Admission::open(&fixture.0.join("root-alias")),
            Err(AdmissionError::Symlink)
        ));
        symlink(other.0.join("external"), fixture.0.join(SLOT_NAMES[0])).unwrap();
        assert!(matches!(
            Admission::open(&fixture.0),
            Err(AdmissionError::Symlink)
        ));
        assert!(!other.0.join("external").exists());
        fs::remove_file(fixture.0.join(SLOT_NAMES[0])).unwrap();
        let admission = fixture.admission();
        fs::remove_file(fixture.0.join(SLOT_NAMES[1])).unwrap();
        fs::hard_link(fixture.0.join(SLOT_NAMES[0]), fixture.0.join(SLOT_NAMES[1])).unwrap();
        assert!(matches!(
            admission.try_acquire(),
            Err(AdmissionError::InvalidLockFile)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn ancestor_retargeting_same_state_inode_is_refused() {
        use std::os::unix::fs::{DirBuilderExt, symlink};
        let fixture = Fixture::new();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(fixture.0.join("parent"))
            .unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(fixture.0.join("parent/state"))
            .unwrap();
        let admission = Admission::open(&fixture.0.join("parent/state")).unwrap();
        let _existing = admission.try_acquire().unwrap();
        fs::rename(fixture.0.join("parent"), fixture.0.join("moved")).unwrap();
        symlink(fixture.0.join("moved"), fixture.0.join("parent")).unwrap();
        assert!(matches!(
            admission.try_acquire(),
            Err(AdmissionError::StateChanged)
        ));
    }

    #[test]
    fn subprocess_lock_helper() {
        let Some(root) = std::env::var_os("MIRKET_LEASE_TEST_ROOT") else {
            return;
        };
        let admission = Admission::open(Path::new(&root)).unwrap();
        let _first = admission.try_acquire().unwrap();
        let _second = admission.try_acquire().unwrap();
        assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
        println!("MIRKET_TEST_LEASES_READY");
        std::io::stdout().flush().unwrap();
        let mut signal = [0];
        std::io::stdin().read_exact(&mut signal).unwrap();
    }

    struct ChildGuard(Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    fn holding_child(fixture: &Fixture) -> ChildGuard {
        holding_child_kind(fixture, false)
    }

    fn holding_child_kind(fixture: &Fixture, waiting: bool) -> ChildGuard {
        let mut child = ChildGuard(
            Command::new(std::env::current_exe().unwrap())
                .arg(if waiting {
                    "subprocess_waiting_helper"
                } else {
                    "subprocess_lock_helper"
                })
                .arg("--nocapture")
                .env(
                    if waiting {
                        "MIRKET_WAITING_TEST_ROOT"
                    } else {
                        "MIRKET_LEASE_TEST_ROOT"
                    },
                    &fixture.0,
                )
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        );
        let mut reader = BufReader::new(child.0.stdout.take().unwrap());
        loop {
            let mut line = String::new();
            assert_ne!(
                reader.read_line(&mut line).unwrap(),
                0,
                "child ended before holding its leases"
            );
            if line.trim() == "MIRKET_TEST_LEASES_READY" {
                break;
            }
        }
        // Keep remaining harness output drainable so its normal exit cannot
        // receive a broken pipe after the readiness handshake.
        child.0.stdout = Some(reader.into_inner());
        child
    }

    #[test]
    fn independent_processes_share_two_slots_and_normal_exit_releases_them() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let mut child = holding_child(&fixture);
        assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
        child.0.stdin.take().unwrap().write_all(b"x").unwrap();
        assert!(child.0.wait().unwrap().success());
        let _one = admission.try_acquire().unwrap();
        let _two = admission.try_acquire().unwrap();
        assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
    }

    #[test]
    fn os_releases_locks_after_holder_process_is_terminated() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let mut child = holding_child(&fixture);
        assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        let _one = admission.try_acquire().unwrap();
        let _two = admission.try_acquire().unwrap();
    }

    #[test]
    fn waiting_permits_bound_eight_independently_of_two_heavy_slots() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let _heavy_a = admission.try_acquire().unwrap();
        let _heavy_b = admission.try_acquire().unwrap();
        let mut waiting: Vec<_> = (0..WAITING_JOB_SLOTS)
            .map(|_| fixture.admission().try_waiting().unwrap())
            .collect();
        assert!(matches!(
            admission.try_waiting(),
            Err(AdmissionError::WaitingFull)
        ));
        assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
        let released = waiting.pop().unwrap();
        let index = released.slot_index();
        drop(released);
        let replacement = admission.try_waiting().unwrap();
        assert_eq!(replacement.slot_index(), index);
        assert!(matches!(
            admission.try_waiting(),
            Err(AdmissionError::WaitingFull)
        ));
        for name in WAITING_NAMES {
            assert_eq!(fs::metadata(fixture.0.join(name)).unwrap().len(), 0);
        }
        assert!(!marker_path(&fixture.0, 0).exists());
        assert!(!marker_path(&fixture.0, 1).exists());
    }

    #[cfg(unix)]
    #[test]
    fn waiting_slot_drift_symlink_permissions_and_alias_are_errors_not_capacity() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let path = fixture.0.join(WAITING_NAMES[0]);
        fs::write(&path, b"unrelated").unwrap();
        assert!(matches!(
            admission.try_waiting(),
            Err(AdmissionError::InvalidLockFile)
        ));
        assert_eq!(fs::read(&path).unwrap(), b"unrelated");
        fs::remove_file(&path).unwrap();
        symlink(fixture.0.join("outside"), &path).unwrap();
        assert!(matches!(
            admission.try_waiting(),
            Err(AdmissionError::Symlink)
        ));
        assert!(!fixture.0.join("outside").exists());
        fs::remove_file(&path).unwrap();
        drop(admission.try_waiting().unwrap());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            admission.try_waiting(),
            Err(AdmissionError::PrivateStateRequired)
        ));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::hard_link(&path, fixture.0.join("alias")).unwrap();
        assert!(matches!(
            admission.try_waiting(),
            Err(AdmissionError::InvalidLockFile)
        ));
        assert!(matches!(
            Admission::open(&fixture.0),
            Err(AdmissionError::InvalidLockFile)
        ));
    }

    #[test]
    fn subprocess_waiting_helper() {
        let Some(root) = std::env::var_os("MIRKET_WAITING_TEST_ROOT") else {
            return;
        };
        let admission = Admission::open(Path::new(&root)).unwrap();
        let _waiting: Vec<_> = (0..WAITING_JOB_SLOTS)
            .map(|_| admission.try_waiting().unwrap())
            .collect();
        assert!(matches!(
            admission.try_waiting(),
            Err(AdmissionError::WaitingFull)
        ));
        println!("MIRKET_TEST_LEASES_READY");
        std::io::stdout().flush().unwrap();
        let mut signal = [0];
        std::io::stdin().read_exact(&mut signal).unwrap();
    }

    #[test]
    fn independent_process_waiter_release_and_death_restore_all_eight_permits() {
        let fixture = Fixture::new();
        let admission = fixture.admission();
        let _heavy_a = admission.try_acquire().unwrap();
        let _heavy_b = admission.try_acquire().unwrap();
        for abrupt in [false, true] {
            let mut child = holding_child_kind(&fixture, true);
            assert!(matches!(
                admission.try_waiting(),
                Err(AdmissionError::WaitingFull)
            ));
            if abrupt {
                child.0.kill().unwrap();
                child.0.wait().unwrap();
            } else {
                child.0.stdin.take().unwrap().write_all(b"x").unwrap();
                assert!(child.0.wait().unwrap().success());
            }
            let _waiting: Vec<_> = (0..WAITING_JOB_SLOTS)
                .map(|_| admission.try_waiting().unwrap())
                .collect();
            assert!(matches!(
                admission.try_waiting(),
                Err(AdmissionError::WaitingFull)
            ));
            assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
            assert!(!marker_path(&fixture.0, 0).exists());
            assert!(!marker_path(&fixture.0, 1).exists());
        }
    }
}
