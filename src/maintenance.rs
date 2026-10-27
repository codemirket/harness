//! Native maintenance exclusion for the explicitly selected installation state.
//!
//! Every MCP connection and short-lived worker must retain its own shared
//! SessionGuard for its entire lifetime. Activation retains the exclusive
//! MaintenanceGuard through its final state change. Lock order is maintenance,
//! then admission, then scoped mutation; never try to upgrade a held session
//! lock. Both entry points are nonblocking and return Busy instead of waiting.
//!
//! runtime.lock is permanent: never remove, replace or truncate it. Guards own
//! independent descriptors and explicitly unlock when dropped. The
//! state directory is private trusted installation state, not a hostile-path
//! sandbox. Unix permissions/file identity are checked; Windows ACL ownership
//! and native identity require platform qualification. A stale ownership marker
//! blocks activation even after all process locks have been released.

use crate::file_lock::FileLock;
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions, TryLockError};
use std::io;
use std::path::{Component, Path, PathBuf};

const LOCK_NAME: &str = "runtime.lock";
const OWNERSHIP_MARKERS: [&str; 2] = ["heavy-slot-0.unresolved", "heavy-slot-1.unresolved"];

#[derive(Debug)]
pub enum MaintenanceError {
    RootMustBeAbsolute,
    RootMustBeDirectory,
    PrivateStateRequired,
    Symlink,
    InvalidRuntimeLock,
    StateChanged,
    Busy,
    UnresolvedOwnership {
        slot: usize,
    },
    ResourceAccounting(String),
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for MaintenanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeAbsolute => f.write_str(
                "maintenance requires an explicit absolute state root without traversal",
            ),
            Self::RootMustBeDirectory => {
                f.write_str("maintenance state root must be an existing directory")
            }
            Self::PrivateStateRequired => {
                f.write_str("maintenance state and lock must deny group and other access")
            }
            Self::Symlink => {
                f.write_str("maintenance state root and runtime lock cannot be symlinks")
            }
            Self::InvalidRuntimeLock => {
                f.write_str("runtime lock must be a private, empty, unaliased regular file")
            }
            Self::StateChanged => f.write_str(
                "maintenance state changed while taking its lock; inspect installation state",
            ),
            Self::Busy => f.write_str(
                "runtime maintenance lock is busy; retry only after its current owners stop",
            ),
            Self::UnresolvedOwnership { slot } => write!(
                f,
                "heavy-job slot {slot} has unresolved ownership; inspect it before activation"
            ),
            Self::ResourceAccounting(message) => {
                write!(f, "runtime resource registration: {message}")
            }
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl std::error::Error for MaintenanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(operation: &'static str, source: io::Error) -> MaintenanceError {
    MaintenanceError::Io { operation, source }
}

#[derive(Debug)]
struct RuntimeLock {
    _file: FileLock,
    root: PathBuf,
}

/// Hold for one whole connection or worker; this does not authorize its work.
#[derive(Debug)]
pub struct SessionGuard {
    // Drop observational membership while maintenance exclusion still protects
    // this process. Membership cannot release a heavy-job ownership marker.
    #[cfg(target_os = "macos")]
    _resources: crate::resource_admission::ResourceSession,
    lock: RuntimeLock,
}

impl SessionGuard {
    pub fn acquire(state_root: &Path) -> Result<Self, MaintenanceError> {
        let lock = acquire(state_root, false)?;
        #[cfg(target_os = "macos")]
        let resources = crate::resource_admission::ResourceSession::register(&lock.root)
            .map_err(|e| MaintenanceError::ResourceAccounting(e.to_string()))?;
        Ok(Self {
            #[cfg(target_os = "macos")]
            _resources: resources,
            lock,
        })
    }

    pub fn state_root(&self) -> &Path {
        &self.lock.root
    }
}

/// Hold from activation preflight through finalization. Does not activate itself.
#[derive(Debug)]
pub struct MaintenanceGuard {
    lock: RuntimeLock,
}

impl MaintenanceGuard {
    pub fn try_acquire(state_root: &Path) -> Result<Self, MaintenanceError> {
        let lock = acquire(state_root, true)?;
        // All conforming sessions/workers are excluded now. Merely seeing their
        // OS locks released is insufficient if any durable ownership is unknown.
        for (slot, name) in OWNERSHIP_MARKERS.iter().enumerate() {
            match fs::symlink_metadata(lock.root.join(name)) {
                Ok(_) => return Err(MaintenanceError::UnresolvedOwnership { slot }),
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => {
                    return Err(io_error(
                        "inspect unresolved ownership before maintenance",
                        e,
                    ));
                }
            }
        }
        Ok(Self { lock })
    }

    pub fn state_root(&self) -> &Path {
        &self.lock.root
    }
}

fn acquire(state_root: &Path, exclusive: bool) -> Result<RuntimeLock, MaintenanceError> {
    if !state_root.is_absolute()
        || state_root
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(MaintenanceError::RootMustBeAbsolute);
    }
    validate_root(state_root)?;
    let root = fs::canonicalize(state_root).map_err(|e| io_error("resolve maintenance root", e))?;
    let selected = validate_root(&root)?;
    let path = root.join(LOCK_NAME);
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
            validate_lock(
                &fs::symlink_metadata(&path)
                    .map_err(|e| io_error("inspect existing runtime lock", e))?,
            )?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .map_err(|e| io_error("open existing runtime lock", e))?
        }
        Err(e) => return Err(io_error("create runtime lock", e)),
    };
    validate_open_lock(&path, &file)?;
    let result = if exclusive {
        file.try_lock()
    } else {
        file.try_lock_shared()
    };
    match result {
        Ok(()) => (),
        Err(TryLockError::WouldBlock) => return Err(MaintenanceError::Busy),
        Err(TryLockError::Error(e)) => return Err(io_error("acquire runtime lock", e)),
    }
    let file = FileLock::acquired(file);
    if fs::canonicalize(&root).map_err(|e| io_error("recheck maintenance ancestry", e))? != root
        || !same_file(&selected, &validate_root(&root)?)
    {
        return Err(MaintenanceError::StateChanged);
    }
    validate_open_lock(&path, file.as_file())?;
    Ok(RuntimeLock { _file: file, root })
}

fn validate_root(path: &Path) -> Result<Metadata, MaintenanceError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|e| io_error("inspect maintenance root", e))?;
    if metadata.file_type().is_symlink() {
        return Err(MaintenanceError::Symlink);
    }
    if !metadata.is_dir() {
        return Err(MaintenanceError::RootMustBeDirectory);
    }
    require_private(&metadata)?;
    Ok(metadata)
}

fn require_private(metadata: &Metadata) -> Result<(), MaintenanceError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(MaintenanceError::PrivateStateRequired);
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    Ok(())
}

fn validate_lock(metadata: &Metadata) -> Result<(), MaintenanceError> {
    if metadata.file_type().is_symlink() {
        return Err(MaintenanceError::Symlink);
    }
    if !metadata.is_file() || metadata.len() != 0 {
        return Err(MaintenanceError::InvalidRuntimeLock);
    }
    require_private(metadata)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err(MaintenanceError::InvalidRuntimeLock);
        }
    }
    Ok(())
}

fn validate_open_lock(path: &Path, file: &File) -> Result<(), MaintenanceError> {
    let current =
        fs::symlink_metadata(path).map_err(|e| io_error("recheck runtime lock path", e))?;
    let opened = file
        .metadata()
        .map_err(|e| io_error("inspect open runtime lock", e))?;
    validate_lock(&current)?;
    validate_lock(&opened)?;
    if !same_file(&current, &opened) {
        return Err(MaintenanceError::StateChanged);
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
        // A bounded fallback check, not a claim of qualified Windows identity.
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
                "mirket-maintenance-test-{}-{}",
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
            Self(path.canonicalize().unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn independent_descriptors_share_sessions_and_exclude_activation() {
        let fixture = Fixture::new();
        let first = SessionGuard::acquire(&fixture.0).unwrap();
        let second = SessionGuard::acquire(&fixture.0).unwrap();
        assert_eq!(first.state_root(), fixture.0);
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::Busy)
        ));
        drop(first);
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::Busy)
        ));
        drop(second);
        let maintenance = MaintenanceGuard::try_acquire(&fixture.0).unwrap();
        assert_eq!(maintenance.state_root(), fixture.0);
        assert!(matches!(
            SessionGuard::acquire(&fixture.0),
            Err(MaintenanceError::Busy)
        ));
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::Busy)
        ));
        drop(maintenance);
        assert!(SessionGuard::acquire(&fixture.0).is_ok());
        assert_eq!(fs::metadata(fixture.0.join(LOCK_NAME)).unwrap().len(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn session_drop_unlocks_despite_duplicate_description() {
        let fixture = Fixture::new();
        let first = SessionGuard::acquire(&fixture.0).unwrap();
        let second = SessionGuard::acquire(&fixture.0).unwrap();
        let duplicate = second.lock._file.as_file().try_clone().unwrap();
        drop(second);
        // The separate first owner still excludes maintenance.
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::Busy)
        ));
        drop(first);
        let maintenance = MaintenanceGuard::try_acquire(&fixture.0).unwrap();
        let maintenance_duplicate = maintenance.lock._file.as_file().try_clone().unwrap();
        drop(maintenance);
        assert!(SessionGuard::acquire(&fixture.0).is_ok());
        drop(maintenance_duplicate);
        drop(duplicate);
    }

    #[test]
    fn either_unresolved_marker_blocks_activation_without_any_live_lock() {
        for (slot, marker) in OWNERSHIP_MARKERS.iter().enumerate() {
            let fixture = Fixture::new();
            // Marker validity does not establish stopped ownership: even an
            // empty/partially persisted entry must block maintenance.
            fs::write(fixture.0.join(marker), []).unwrap();
            assert!(
                matches!(MaintenanceGuard::try_acquire(&fixture.0), Err(MaintenanceError::UnresolvedOwnership { slot: actual }) if actual == slot)
            );
            assert!(fixture.0.join(marker).exists());
            // Inspection sessions stay available to diagnose unresolved work.
            assert!(SessionGuard::acquire(&fixture.0).is_ok());
        }
    }

    #[test]
    fn invalid_roots_and_nonempty_lock_fail_without_modifying_existing_bytes() {
        assert!(matches!(
            SessionGuard::acquire(Path::new(".")),
            Err(MaintenanceError::RootMustBeAbsolute)
        ));
        let fixture = Fixture::new();
        let path = fixture.0.join(LOCK_NAME);
        fs::write(&path, b"unrelated lock content").unwrap();
        assert!(matches!(
            SessionGuard::acquire(&fixture.0),
            Err(MaintenanceError::InvalidRuntimeLock)
        ));
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::InvalidRuntimeLock)
        ));
        assert_eq!(fs::read(path).unwrap(), b"unrelated lock content");
        assert!(matches!(
            SessionGuard::acquire(&fixture.0.join("../")),
            Err(MaintenanceError::RootMustBeAbsolute)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn private_permissions_symlinks_and_lock_aliases_are_checked() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let fixture = Fixture::new();
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(matches!(
            SessionGuard::acquire(&fixture.0),
            Err(MaintenanceError::PrivateStateRequired)
        ));
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o700)).unwrap();
        let aliases = Fixture::new();
        symlink(&fixture.0, aliases.0.join("state")).unwrap();
        assert!(matches!(
            SessionGuard::acquire(&aliases.0.join("state")),
            Err(MaintenanceError::Symlink)
        ));
        let outside = aliases.0.join("outside");
        fs::write(&outside, b"outside remains").unwrap();
        symlink(&outside, fixture.0.join(LOCK_NAME)).unwrap();
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::Symlink)
        ));
        assert_eq!(fs::read(&outside).unwrap(), b"outside remains");
        fs::remove_file(fixture.0.join(LOCK_NAME)).unwrap();
        drop(SessionGuard::acquire(&fixture.0).unwrap());
        fs::hard_link(fixture.0.join(LOCK_NAME), aliases.0.join("lock-alias")).unwrap();
        assert!(matches!(
            SessionGuard::acquire(&fixture.0),
            Err(MaintenanceError::InvalidRuntimeLock)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn dangling_or_directory_marker_fails_closed() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        symlink(
            fixture.0.join("absent"),
            fixture.0.join(OWNERSHIP_MARKERS[0]),
        )
        .unwrap();
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::UnresolvedOwnership { slot: 0 })
        ));
        fs::remove_file(fixture.0.join(OWNERSHIP_MARKERS[0])).unwrap();
        fs::create_dir(fixture.0.join(OWNERSHIP_MARKERS[1])).unwrap();
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::UnresolvedOwnership { slot: 1 })
        ));
    }

    #[test]
    fn subprocess_session_helper() {
        let Some(root) = std::env::var_os("MIRKET_MAINTENANCE_TEST_ROOT") else {
            return;
        };
        let _guard = SessionGuard::acquire(Path::new(&root)).unwrap();
        println!("MIRKET_MAINTENANCE_READY");
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

    fn session_child(fixture: &Fixture) -> ChildGuard {
        let mut child = ChildGuard(
            Command::new(std::env::current_exe().unwrap())
                .arg("subprocess_session_helper")
                .arg("--nocapture")
                .env("MIRKET_MAINTENANCE_TEST_ROOT", &fixture.0)
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
                "child exited before acquiring session"
            );
            if line.trim() == "MIRKET_MAINTENANCE_READY" {
                break;
            }
        }
        child.0.stdout = Some(reader.into_inner());
        child
    }

    #[test]
    fn two_process_sessions_block_activation_until_both_owners_stop() {
        let fixture = Fixture::new();
        let mut first = session_child(&fixture);
        let mut second = session_child(&fixture);
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::Busy)
        ));
        first.0.stdin.take().unwrap().write_all(b"x").unwrap();
        assert!(first.0.wait().unwrap().success());
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::Busy)
        ));
        second.0.stdin.take().unwrap().write_all(b"x").unwrap();
        assert!(second.0.wait().unwrap().success());
        let _exclusive = MaintenanceGuard::try_acquire(&fixture.0).unwrap();
    }

    #[test]
    fn terminated_session_does_not_clear_an_unresolved_marker() {
        let fixture = Fixture::new();
        let mut child = session_child(&fixture);
        fs::write(
            fixture.0.join(OWNERSHIP_MARKERS[1]),
            b"mirket-unresolved-v1\njob-1\n",
        )
        .unwrap();
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        assert!(matches!(
            MaintenanceGuard::try_acquire(&fixture.0),
            Err(MaintenanceError::UnresolvedOwnership { slot: 1 })
        ));
    }
}
