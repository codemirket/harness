//! Native identities for explicitly selected project roots, never remote names.
//!
//! A selected root is the whole selection: no parent search, Git execution,
//! hooks, configuration loading or project-side markers. Unix device/inode/birth
//! time qualifies same-volume moves on the tested filesystem. Cross-volume
//! copies get fresh identities. Windows native identity is not yet qualified.
//! Path checks detect ordinary drift; they are not hostile ancestor-swap
//! containment. The registry must live in a private, caller-owned state root.

use rusqlite::{Connection, OpenFlags, OptionalExtension, TransactionBehavior, params};
use std::{
    fmt, fs,
    io::{self, Read},
    path::{Component, Path, PathBuf},
    time::Duration,
};

#[cfg(unix)]
use std::time::UNIX_EPOCH;

const APPLICATION_ID: i64 = 0x4d494449;
const MAX_GIT_METADATA: u64 = 4096;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedIdentity {
    pub project_id: String,
    pub checkout_id: String,
    pub project_root: PathBuf,
}

#[derive(Debug)]
pub enum IdentityError {
    Invalid(&'static str),
    Io(io::Error),
    Database(rusqlite::Error),
}
impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) => f.write_str(s),
            Self::Io(e) => write!(f, "project identity filesystem: {e}"),
            Self::Database(e) => write!(f, "project identity registry: {e}"),
        }
    }
}
impl std::error::Error for IdentityError {}
impl From<io::Error> for IdentityError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for IdentityError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e)
    }
}
type Result<T> = std::result::Result<T, IdentityError>;

/// Explicit registration may create private state and persist a verified move.
pub fn resolve(state_root: &Path, project_root: &Path) -> Result<ResolvedIdentity> {
    let observed = inspect(project_root)?;
    let mut connection = open_registry(state_root, true)?
        .ok_or(IdentityError::Invalid("identity registry unavailable"))?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let existing = find(&tx, &observed)?;
    let result = if let Some(existing) = existing {
        existing
    } else {
        let project_id: String = match tx
            .query_row(
                "SELECT project_id FROM projects WHERE owner_key=?1",
                [&observed.owner],
                |r| r.get(0),
            )
            .optional()?
        {
            Some(id) => id,
            None => {
                let id: String =
                    tx.query_row("SELECT 'project-' || lower(hex(randomblob(16)))", [], |r| {
                        r.get(0)
                    })?;
                tx.execute(
                    "INSERT INTO projects(project_id,owner_key) VALUES(?1,?2)",
                    params![id, observed.owner],
                )?;
                id
            }
        };
        let checkout_id: String = tx.query_row(
            "SELECT 'checkout-' || lower(hex(randomblob(16)))",
            [],
            |r| r.get(0),
        )?;
        tx.execute("INSERT INTO checkouts(checkout_id,directory_key,project_id,last_path) VALUES(?1,?2,?3,?4)", params![checkout_id, observed.directory, project_id, path_text(&observed.root)?])?;
        ResolvedIdentity {
            project_id,
            checkout_id,
            project_root: observed.root.clone(),
        }
    };
    if inspect(project_root)? != observed {
        return Err(IdentityError::Invalid(
            "project identity changed during registration",
        ));
    }
    tx.execute(
        "UPDATE checkouts SET last_path=?1 WHERE checkout_id=?2",
        params![path_text(&observed.root)?, result.checkout_id],
    )?;
    tx.commit()?;
    Ok(result)
}

/// Does not create state, register a checkout, update observed paths or migrate.
pub fn lookup(state_root: &Path, project_root: &Path) -> Result<Option<ResolvedIdentity>> {
    let observed = inspect(project_root)?;
    let Some(connection) = open_registry(state_root, false)? else {
        return Ok(None);
    };
    let result = find(&connection, &observed)?;
    if inspect(project_root)? != observed {
        return Err(IdentityError::Invalid(
            "project identity changed during lookup",
        ));
    }
    Ok(result)
}

fn find(connection: &Connection, observed: &Observed) -> Result<Option<ResolvedIdentity>> {
    let found: Option<(String, String, String)> = connection.query_row(
        "SELECT p.project_id,c.checkout_id,p.owner_key FROM checkouts c JOIN projects p USING(project_id) WHERE c.directory_key=?1", [&observed.directory],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
    found.map(|(project_id, checkout_id, owner)| {
        if owner != observed.owner { return Err(IdentityError::Invalid("registered checkout now has a different project owner; explicit identity reconciliation is required")); }
        Ok(ResolvedIdentity { project_id, checkout_id, project_root: observed.root.clone() })
    }).transpose()
}

#[derive(Debug, Eq, PartialEq)]
struct Observed {
    root: PathBuf,
    directory: String,
    owner: String,
    evidence: Vec<(PathBuf, String)>,
}
fn inspect(root: &Path) -> Result<Observed> {
    absolute(root)?;
    let root = fs::canonicalize(root)?;
    let directory = directory_key(&root)?;
    let mut evidence = Vec::new();
    let git = root.join(".git");
    let owner = match fs::symlink_metadata(&git) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => format!("directory:{directory}"),
        Err(e) => return Err(e.into()),
        Ok(m) if m.file_type().is_symlink() => {
            return Err(IdentityError::Invalid(".git symlinks are unsupported"));
        }
        Ok(m) if m.is_dir() => {
            verify_common(&git, &mut evidence)?;
            format!("git:{}", directory_key(&git)?)
        }
        Ok(m) if m.is_file() => {
            let pointer = metadata_text(&git, &mut evidence)?;
            let target = pointer
                .strip_prefix("gitdir: ")
                .ok_or(IdentityError::Invalid("unsupported .git file"))?;
            let admin = referenced(&root, target)?;
            directory_key(&admin)?;
            let common_text = metadata_text(&admin.join("commondir"), &mut evidence)?;
            let common = referenced(&admin, &common_text)?;
            let relative = admin
                .strip_prefix(common.join("worktrees"))
                .map_err(|_| IdentityError::Invalid("unsupported Git directory layout"))?;
            if relative.components().count() != 1
                || !matches!(relative.components().next(), Some(Component::Normal(_)))
            {
                return Err(IdentityError::Invalid("ambiguous linked worktree layout"));
            }
            let backlink = metadata_text(&admin.join("gitdir"), &mut evidence)?;
            if referenced(&admin, &backlink)? != git {
                return Err(IdentityError::Invalid(
                    "Git worktree backlink does not match selected checkout",
                ));
            }
            verify_head(&admin.join("HEAD"), &mut evidence)?;
            verify_common(&common, &mut evidence)?;
            evidence.push((admin.clone(), directory_key(&admin)?));
            format!("git:{}", directory_key(&common)?)
        }
        Ok(_) => return Err(IdentityError::Invalid("unsupported .git file type")),
    };
    Ok(Observed {
        root,
        directory,
        owner,
        evidence,
    })
}
fn verify_common(common: &Path, evidence: &mut Vec<(PathBuf, String)>) -> Result<()> {
    if entry_exists(&common.join("commondir"))? || entry_exists(&common.join("gitdir"))? {
        return Err(IdentityError::Invalid("ambiguous common Git directory"));
    }
    evidence.push((common.to_owned(), directory_key(common)?));
    for name in ["objects", "refs"] {
        let path = common.join(name);
        evidence.push((path.clone(), directory_key(&path)?));
    }
    verify_head(&common.join("HEAD"), evidence)
}
fn verify_head(path: &Path, evidence: &mut Vec<(PathBuf, String)>) -> Result<()> {
    let head = metadata_text(path, evidence)?;
    let valid_ref = head.strip_prefix("ref: refs/").is_some_and(|r| {
        !r.is_empty() && !r.contains("..") && !r.bytes().any(|b| b.is_ascii_whitespace() || b < 32)
    });
    if !valid_ref
        && !((head.len() == 40 || head.len() == 64) && head.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(IdentityError::Invalid("unsupported Git HEAD"));
    }
    Ok(())
}
fn metadata_text(path: &Path, evidence: &mut Vec<(PathBuf, String)>) -> Result<String> {
    no_symlinks(path)?;
    let before = fs::symlink_metadata(path)?;
    if !before.is_file() || before.len() > MAX_GIT_METADATA {
        return Err(IdentityError::Invalid(
            "Git metadata must be a bounded regular file",
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_GIT_METADATA + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_GIT_METADATA || fs::symlink_metadata(path)?.len() != before.len() {
        return Err(IdentityError::Invalid(
            "Git metadata changed or exceeded its bound",
        ));
    }
    let value = String::from_utf8(bytes)
        .map_err(|_| IdentityError::Invalid("Git metadata must be UTF-8"))?;
    let line = value.strip_suffix('\n').unwrap_or(&value);
    if line.is_empty() || line.contains(['\n', '\r', '\0']) {
        return Err(IdentityError::Invalid(
            "Git metadata must contain one bounded line",
        ));
    }
    evidence.push((path.to_owned(), value.clone()));
    Ok(line.to_owned())
}
fn referenced(base: &Path, text: &str) -> Result<PathBuf> {
    let joined = base.join(text);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(IdentityError::Invalid(
                        "Git metadata path escapes filesystem root",
                    ));
                }
            }
            Component::CurDir => (),
            other => normalized.push(other.as_os_str()),
        }
    }
    no_symlinks(&normalized)?;
    Ok(fs::canonicalize(normalized)?)
}
fn no_symlinks(path: &Path) -> Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Err(IdentityError::Invalid(
                "identity metadata symlinks are unsupported",
            ));
        }
    }
    Ok(())
}
fn absolute(path: &Path) -> Result<()> {
    if !path.is_absolute() || path.components().any(|p| matches!(p, Component::ParentDir)) {
        return Err(IdentityError::Invalid(
            "identity requires an explicit absolute path without parent traversal",
        ));
    }
    Ok(())
}
fn path_text(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or(IdentityError::Invalid("identity paths must be UTF-8"))
}
fn directory_key(path: &Path) -> Result<String> {
    no_symlinks(path)?;
    let m = fs::symlink_metadata(path)?;
    if !m.is_dir() {
        return Err(IdentityError::Invalid("identity owner must be a directory"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let created = m
            .created()
            .map_err(|_| {
                IdentityError::Invalid(
                    "native directory birth time unavailable; move identity is unqualified",
                )
            })?
            .duration_since(UNIX_EPOCH)
            .map_err(|_| IdentityError::Invalid("unsupported directory birth time"))?;
        Ok(format!(
            "unix:{}:{}:{}:{}",
            m.dev(),
            m.ino(),
            created.as_secs(),
            created.subsec_nanos()
        ))
    }
    #[cfg(not(unix))]
    {
        let _ = m;
        Err(IdentityError::Invalid(
            "native filesystem identity is not qualified on this platform",
        ))
    }
}

fn open_registry(state_root: &Path, create: bool) -> Result<Option<Connection>> {
    absolute(state_root)?;
    if !create && !entry_exists(state_root)? {
        return Ok(None);
    }
    private_directory(state_root, create)?;
    // Canonicalize the explicitly selected root, then forbid metadata symlinks below it.
    let state_root = fs::canonicalize(state_root)?;
    let directory = state_root.join("identity");
    if !create && !entry_exists(&directory)? {
        return Ok(None);
    }
    private_directory(&directory, create)?;
    let database = directory.join("registry.sqlite3");
    if !create && !entry_exists(&database)? {
        return Ok(None);
    }
    if create {
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&database) {
            Ok(f) => {
                f.sync_all()?;
                fs::File::open(&directory)?.sync_all()?;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.into()),
        }
    }
    private_file(&database)?;
    for suffix in ["-journal", "-wal", "-shm"] {
        let sidecar = directory.join(format!("registry.sqlite3{suffix}"));
        match fs::symlink_metadata(&sidecar) {
            Ok(_) => private_file(&sidecar)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    let flags = if create {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    } else {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    } | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let mut connection = Connection::open_with_flags(database, flags)?;
    connection.busy_timeout(Duration::from_millis(250))?;
    connection.set_db_config(rusqlite::config::DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
    connection.set_db_config(
        rusqlite::config::DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA,
        false,
    )?;
    connection.execute_batch("PRAGMA foreign_keys=ON;")?;
    if create {
        connection.execute_batch("PRAGMA synchronous=FULL;")?;
    }
    let mode: String = connection.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
    if mode != "delete" {
        return Err(IdentityError::Invalid(
            "identity registry requires delete journal mode",
        ));
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
                return Err(IdentityError::Invalid("refusing unknown identity database"));
            }
            tx.execute_batch("CREATE TABLE projects(project_id TEXT PRIMARY KEY,owner_key TEXT NOT NULL UNIQUE) STRICT; CREATE TABLE checkouts(checkout_id TEXT PRIMARY KEY,directory_key TEXT NOT NULL UNIQUE,project_id TEXT NOT NULL REFERENCES projects(project_id),last_path TEXT NOT NULL) STRICT;")?;
            tx.pragma_update(None, "application_id", APPLICATION_ID)?;
            tx.pragma_update(None, "user_version", 1)?;
        }
        tx.commit()?;
    }
    let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let app: i64 = connection.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    if version != 1 || app != APPLICATION_ID {
        return Err(IdentityError::Invalid("unknown identity registry schema"));
    }
    Ok(Some(connection))
}
fn entry_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
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
            Ok(()) => (),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.into()),
        }
    }
    let m = fs::symlink_metadata(path)?;
    if !m.is_dir() || m.file_type().is_symlink() {
        return Err(IdentityError::Invalid(
            "identity state must be a real private directory",
        ));
    }
    private_permissions(&m)
}
fn private_file(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    if !m.is_file() || m.file_type().is_symlink() {
        return Err(IdentityError::Invalid(
            "identity registry must be a private regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.nlink() != 1 {
            return Err(IdentityError::Invalid(
                "identity registry hardlink aliases are unsupported",
            ));
        }
    }
    private_permissions(&m)
}
fn private_permissions(m: &fs::Metadata) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        // SAFETY: geteuid has no arguments, pointer access or failure mode.
        let current_uid = unsafe { libc::geteuid() };
        if m.permissions().mode() & 0o077 != 0 || m.uid() != current_uid {
            return Err(IdentityError::Invalid(
                "identity state must be private and owned by the current user",
            ));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = m;
        Err(IdentityError::Invalid(
            "private identity state is unqualified on this platform",
        ))
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "mirket-identity-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(fs::canonicalize(p).unwrap())
        }
        fn state(&self) -> PathBuf {
            self.0.join("state")
        }
        fn directory(&self, name: &str) -> PathBuf {
            let p = self.0.join(name);
            fs::create_dir(&p).unwrap();
            p
        }
        fn repo(&self, name: &str) -> PathBuf {
            let p = self.directory(name);
            let git = p.join(".git");
            fs::create_dir(&git).unwrap();
            for n in ["objects", "refs", "worktrees"] {
                fs::create_dir(git.join(n)).unwrap();
            }
            fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();
            p
        }
        fn worktree(&self, main: &Path, name: &str) -> PathBuf {
            let p = self.directory(name);
            let admin = main.join(".git/worktrees").join(name);
            fs::create_dir(&admin).unwrap();
            fs::write(p.join(".git"), format!("gitdir: {}\n", admin.display())).unwrap();
            fs::write(admin.join("commondir"), "../..\n").unwrap();
            fs::write(
                admin.join("gitdir"),
                format!("{}\n", p.join(".git").display()),
            )
            .unwrap();
            fs::write(admin.join("HEAD"), "ref: refs/heads/topic\n").unwrap();
            p
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn read_only_lookup_never_registers_or_updates_move() {
        let f = Fixture::new();
        let root = f.directory("project");
        let state = f.state();
        assert_eq!(lookup(&state, &root).unwrap(), None);
        assert!(!state.exists());
        let before = resolve(&state, &root).unwrap();
        let db = state.join("identity/registry.sqlite3");
        let saved = fs::read(&db).unwrap();
        let moved = f.0.join("moved");
        fs::rename(&root, &moved).unwrap();
        let after = lookup(&state, &moved).unwrap().unwrap();
        assert_eq!(before.project_id, after.project_id);
        assert_eq!(before.checkout_id, after.checkout_id);
        assert_eq!(fs::read(db).unwrap(), saved);
        let new = f.directory("new");
        assert_eq!(lookup(&state, &new).unwrap(), None);
    }
    #[test]
    fn verified_worktrees_share_project_but_not_checkout_and_clones_do_not_merge() {
        let f = Fixture::new();
        let main = f.repo("main");
        let a = f.worktree(&main, "a");
        let b = f.worktree(&main, "b");
        let clone = f.repo("clone");
        let m = resolve(&f.state(), &main).unwrap();
        let a = resolve(&f.state(), &a).unwrap();
        let b = resolve(&f.state(), &b).unwrap();
        let c = resolve(&f.state(), &clone).unwrap();
        assert_eq!(m.project_id, a.project_id);
        assert_eq!(a.project_id, b.project_id);
        assert_ne!(m.checkout_id, a.checkout_id);
        assert_ne!(a.checkout_id, b.checkout_id);
        assert_ne!(m.project_id, c.project_id);
    }
    #[test]
    fn moves_preserve_identity_and_reused_paths_do_not() {
        let f = Fixture::new();
        let main = f.repo("main");
        let before = resolve(&f.state(), &main).unwrap();
        let moved = f.0.join("moved");
        fs::rename(&main, &moved).unwrap();
        let after = resolve(&f.state(), &moved).unwrap();
        assert_eq!(before.project_id, after.project_id);
        assert_eq!(before.checkout_id, after.checkout_id);
        let replacement = f.repo("main");
        let new = resolve(&f.state(), &replacement).unwrap();
        assert_ne!(before.project_id, new.project_id);
        assert_ne!(before.checkout_id, new.checkout_id);
        assert_eq!(
            lookup(&f.state(), &moved).unwrap().unwrap().project_id,
            before.project_id
        );
    }
    #[test]
    fn copied_worktree_pointer_and_owner_replacement_refuse() {
        let f = Fixture::new();
        let main = f.repo("main");
        let a = f.worktree(&main, "a");
        let copied = f.directory("copy");
        fs::copy(a.join(".git"), copied.join(".git")).unwrap();
        assert!(resolve(&f.state(), &copied).is_err());
        resolve(&f.state(), &main).unwrap();
        fs::rename(main.join(".git"), f.0.join("oldgit")).unwrap();
        assert!(
            resolve(&f.state(), &main)
                .unwrap_err()
                .to_string()
                .contains("different project owner")
        );
    }
    #[test]
    fn ambiguous_symlinked_and_oversized_metadata_refuse() {
        let f = Fixture::new();
        let main = f.repo("main");
        let a = f.worktree(&main, "a");
        fs::write(main.join(".git/worktrees/a/commondir"), "../..\nextra\n").unwrap();
        assert!(resolve(&f.state(), &a).is_err());
        fs::write(main.join(".git/worktrees/a/commondir"), "../..\n").unwrap();
        fs::write(a.join(".git"), vec![b'x'; MAX_GIT_METADATA as usize + 1]).unwrap();
        assert!(resolve(&f.state(), &a).is_err());
        let linked = f.directory("linked");
        std::os::unix::fs::symlink(main.join(".git"), linked.join(".git")).unwrap();
        assert!(resolve(&f.state(), &linked).is_err());
        assert!(resolve(&f.state(), Path::new("relative")).is_err());
        std::os::unix::fs::symlink(f.0.join("absent"), main.join(".git/commondir")).unwrap();
        assert!(resolve(&f.state(), &main).is_err());
    }
    #[test]
    fn explicit_subdirectory_does_not_inherit_ambient_repository() {
        let f = Fixture::new();
        let main = f.repo("main");
        let sub = main.join("sub");
        fs::create_dir(&sub).unwrap();
        assert_ne!(
            resolve(&f.state(), &main).unwrap().project_id,
            resolve(&f.state(), &sub).unwrap().project_id
        );
    }
    #[test]
    fn dangling_registry_symlink_is_not_an_unregistered_scope() {
        let f = Fixture::new();
        let root = f.directory("project");
        resolve(&f.state(), &root).unwrap();
        let db = f.state().join("identity/registry.sqlite3");
        fs::remove_file(&db).unwrap();
        std::os::unix::fs::symlink(f.0.join("absent"), &db).unwrap();
        assert!(lookup(&f.state(), &root).is_err());
        assert!(resolve(&f.state(), &root).is_err());
    }
    #[test]
    fn unknown_database_and_shared_permissions_refuse() {
        use std::os::unix::fs::PermissionsExt;
        let f = Fixture::new();
        let root = f.directory("project");
        let state = f.state();
        resolve(&state, &root).unwrap();
        let db = state.join("identity/registry.sqlite3");
        let conn = Connection::open(&db).unwrap();
        conn.pragma_update(None, "application_id", 999).unwrap();
        drop(conn);
        assert!(lookup(&state, &root).is_err());
        assert!(resolve(&state, &root).is_err());
        fs::set_permissions(&state, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(lookup(&state, &root).is_err());
    }
}
