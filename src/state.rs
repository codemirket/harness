//! Durable coordination records. These records describe work; the host executes it.
pub mod contracts;
mod identity;
mod identity_recovery;
use crate::{catalog::Catalog, paths::Paths, util};
use anyhow::{Context, Result, bail};
use contracts::ContractKind;
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_EVIDENCE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_CONTRACT_BYTES: u64 = 128 * 1024;
const MAX_SKILLS: usize = 16;
const MAX_EVIDENCE: i64 = 16;
const MAX_CHECKPOINTS: i64 = 256;
const MAX_SNAPSHOT_BYTES: usize = 60 * 1024;
const CONTRACT_SCHEMA: (&str, &str) = (
    "contracts",
    "CREATE TABLE contracts (
    task_id TEXT NOT NULL, path TEXT NOT NULL, kind TEXT NOT NULL,
    contract TEXT NOT NULL CHECK(contract IN ('verification','contributions')),
    roster TEXT, attempts TEXT,
    PRIMARY KEY(task_id,path,kind),
    FOREIGN KEY(task_id,path,kind) REFERENCES evidence(task_id,path,kind) ON DELETE CASCADE
) STRICT",
);
const PAGING_INDEX: (&str, &str) = (
    "tasks_project_created",
    "CREATE INDEX tasks_project_created ON tasks(project_id,created DESC,id DESC)",
);
const SCOPE_SCHEMA: (&str, &str) = (
    "project_scopes",
    "CREATE TABLE project_scopes (project_id TEXT PRIMARY KEY, scope TEXT NOT NULL UNIQUE, root TEXT NOT NULL UNIQUE, identity TEXT NOT NULL) STRICT",
);
const TASKS_FIRST_PAGE: &str = "SELECT id,objective,status,revision,updated,created FROM tasks WHERE project_id=?1 ORDER BY created DESC,id DESC LIMIT ?2";
const TASKS_CURSOR_PAGE: &str = "SELECT id,objective,status,revision,updated,created FROM tasks WHERE project_id=?1 AND (created,id)<(?2,?3) ORDER BY created DESC,id DESC LIMIT ?4";

struct CheckedEvidence {
    bytes: Vec<u8>,
    sha256: String,
    size: u64,
}

const SCHEMA: &[(&str, &str)] = &[
    ("projects", "CREATE TABLE projects (
        id TEXT PRIMARY KEY, root TEXT NOT NULL UNIQUE, identity TEXT NOT NULL,
        created INTEGER NOT NULL
    ) STRICT"),
    ("tasks", "CREATE TABLE tasks (
        id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
        objective TEXT NOT NULL, skills TEXT NOT NULL, acceptance TEXT NOT NULL,
        failure_probe TEXT NOT NULL, status TEXT NOT NULL CHECK(status IN ('active','complete')),
        revision INTEGER NOT NULL, created INTEGER NOT NULL, updated INTEGER NOT NULL
    ) STRICT"),
    ("deliveries", "CREATE TABLE deliveries (
        task_id TEXT NOT NULL REFERENCES tasks(id), skill_id TEXT NOT NULL,
        content_hash TEXT NOT NULL, payload_hash TEXT NOT NULL, delivered INTEGER NOT NULL,
        PRIMARY KEY(task_id,skill_id)
    ) STRICT"),
    ("evidence", "CREATE TABLE evidence (
        task_id TEXT NOT NULL REFERENCES tasks(id), path TEXT NOT NULL,
        kind TEXT NOT NULL CHECK(kind IN ('acceptance','failure_probe')),
        summary TEXT NOT NULL, sha256 TEXT NOT NULL, size INTEGER NOT NULL, recorded INTEGER NOT NULL,
        PRIMARY KEY(task_id,path,kind)
    ) STRICT"),
    ("checkpoints", "CREATE TABLE checkpoints (
        task_id TEXT NOT NULL REFERENCES tasks(id), revision INTEGER NOT NULL,
        note TEXT NOT NULL, created INTEGER NOT NULL, PRIMARY KEY(task_id,revision)
    ) STRICT"),
    ("operations", "CREATE TABLE operations (
        scope TEXT NOT NULL, key TEXT NOT NULL, fingerprint TEXT NOT NULL,
        response TEXT NOT NULL, PRIMARY KEY(scope,key)
    ) STRICT"),
    ("tasks_project", "CREATE INDEX tasks_project ON tasks(project_id,updated)"),
];

#[derive(Clone, Debug)]
pub struct State {
    database: PathBuf,
    paths: Paths,
}

struct GuardedConnection {
    connection: Connection,
    _lease: util::FileLock,
}
impl std::ops::Deref for GuardedConnection {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.connection
    }
}
impl std::ops::DerefMut for GuardedConnection {
    fn deref_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }
}

fn bounded(value: &str, name: &str, max: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > max || value.contains('\0') {
        bail!("{name} must contain 1..={max} bytes and no NUL characters");
    }
    Ok(())
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn identity(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(format!("{}:{}", metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    {
        // The canonical path and creation time are checked together on these hosts.
        let created = metadata
            .created()
            .context("project identity requires a creation time")?;
        Ok(format!(
            "{}",
            created.duration_since(UNIX_EPOCH)?.as_nanos()
        ))
    }
}

fn validate_schema(conn: &Connection, version: i64) -> Result<()> {
    let mut schema = SCHEMA.to_vec();
    if version >= 2 {
        schema.push(CONTRACT_SCHEMA);
    }
    if version >= 3 {
        schema.retain(|(name, _)| *name != "tasks_project");
        schema.push(PAGING_INDEX);
    }
    if version >= 4 {
        schema.push(SCOPE_SCHEMA);
    }
    let mut statement = conn.prepare(
        "SELECT name,sql FROM sqlite_schema WHERE sql IS NOT NULL AND name NOT GLOB 'sqlite_*'",
    )?;
    let mut rows = statement.query([])?;
    let mut count = 0;
    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let sql: String = row.get(1)?;
        let expected = schema
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .with_context(|| format!("incompatible Mirket state schema object: {name}"))?
            .1;
        // SQLite retains the declared constraints in sqlite_schema. Whitespace
        // is insignificant for this fixed schema; changed declarations fail closed.
        if !sql.split_whitespace().eq(expected.split_whitespace()) {
            bail!("incompatible Mirket state schema declaration: {name}");
        }
        count += 1;
    }
    if count != schema.len() {
        bail!("incomplete Mirket state schema");
    }
    Ok(())
}

fn empty_schema(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_schema WHERE sql IS NOT NULL",
        [],
        |row| row.get(0),
    )?;
    if count != 0 {
        bail!("unversioned nonempty Mirket state schema; refusing implicit repair");
    }
    Ok(())
}

fn task_position(project_id: &str, cursor: &str) -> Result<(i64, String)> {
    bounded(cursor, "task cursor", 128)?;
    let mut parts = cursor.split(':');
    let (Some("tasks1"), Some(project), Some(created), Some(task), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        bail!("invalid task cursor; use next_cursor from a task listing");
    };
    let timestamp: i64 = created.parse().context("invalid task cursor timestamp")?;
    if project != project_id
        || timestamp < 0
        || timestamp.to_string() != created
        || !task.strip_prefix("t_").is_some_and(|digest| {
            digest.len() == 32
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    {
        bail!("invalid or cross-project task cursor");
    }
    Ok((timestamp, task.into()))
}

fn history_position(task_id: &str, cursor: &str) -> Result<i64> {
    bounded(cursor, "history cursor", 128)?;
    let mut parts = cursor.split(':');
    let (Some("history1"), Some(task), Some(revision), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        bail!("invalid history cursor; use next_cursor from task history");
    };
    let position: i64 = revision
        .parse()
        .context("invalid history cursor revision")?;
    if task != task_id || position < 1 || position.to_string() != revision {
        bail!("invalid or cross-task history cursor");
    }
    Ok(position)
}

impl State {
    pub fn open(paths: &Paths) -> Result<Self> {
        paths.ensure()?;
        let state = Self {
            database: paths.database(),
            paths: paths.clone(),
        };
        let mut conn = state.connection()?;
        let read = conn.transaction()?;
        let version: i64 = read.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if !(0..=4).contains(&version) {
            bail!("unsupported Mirket state format: {version}");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&state.database, fs::Permissions::from_mode(0o600))?;
        }
        if version == 4 {
            validate_schema(&read, 4)?;
            let journal: String =
                read.pragma_query_value(None, "journal_mode", |row| row.get(0))?;
            if journal != "wal" {
                bail!("Mirket state requires WAL journal mode");
            }
            read.commit()?;
            return Ok(state);
        }
        if version > 0 {
            validate_schema(&read, version)?;
        } else {
            empty_schema(&read)?;
        }
        read.commit()?;

        // WAL mode cannot be changed inside a transaction. Concurrent first
        // openers may briefly contend on that transition before the writer lock.
        let deadline = std::time::Instant::now() + Duration::from_millis(750);
        loop {
            match conn.pragma_update(None, "journal_mode", "WAL") {
                Ok(()) => break,
                Err(error)
                    if error.sqlite_error_code() == Some(rusqlite::ErrorCode::DatabaseBusy)
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(error).context("initialize Mirket WAL journal"),
            }
        }
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: i64 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
        match version {
            0 => {
                empty_schema(&tx)?;
                for (_, sql) in SCHEMA {
                    tx.execute_batch(sql)?;
                }
                tx.execute_batch(CONTRACT_SCHEMA.1)?;
                tx.pragma_update(None, "user_version", 2)?;
            }
            1 => {
                validate_schema(&tx, 1)?;
                tx.execute_batch(CONTRACT_SCHEMA.1)?;
                tx.pragma_update(None, "user_version", 2)?;
            }
            2 => validate_schema(&tx, 2)?,
            3 => validate_schema(&tx, 3)?,
            4 => validate_schema(&tx, 4)?,
            _ => bail!("unsupported Mirket state format: {version}"),
        }
        // Replace the obsolete updated-order index atomically with the cursor
        // ordering. No task, evidence, checkpoint or exact replay is rewritten.
        if version < 3 {
            tx.execute_batch("DROP INDEX tasks_project")?;
            tx.execute_batch(PAGING_INDEX.1)?;
            tx.pragma_update(None, "user_version", 3)?;
        }
        if version < 4 {
            tx.execute_batch(SCOPE_SCHEMA.1)?;
            let projects = {
                let mut q = tx.prepare("SELECT id,root,identity FROM projects")?;
                q.query_map([], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
            };
            for (id, root, identity) in projects {
                tx.execute(
                    "INSERT INTO project_scopes VALUES(?1,?2,?3,?4)",
                    params![id, util::sha256(root.as_bytes()), root, identity],
                )?;
            }
            tx.pragma_update(None, "user_version", 4)?;
        }
        tx.commit()?;
        Ok(state)
    }

    fn connection(&self) -> Result<GuardedConnection> {
        let lease = crate::maintenance::shared(&self.paths)?;
        Ok(GuardedConnection {
            connection: self.unleased_connection()?,
            _lease: lease,
        })
    }

    fn unleased_connection(&self) -> Result<Connection> {
        for path in [
            &self.database,
            &self.database.with_extension("db-wal"),
            &self.database.with_extension("db-shm"),
        ] {
            util::reject_symlink_components(path)?;
        }
        let conn = Connection::open(&self.database)?;
        conn.busy_timeout(Duration::from_millis(750))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        Ok(conn)
    }

    pub(crate) fn validate_backup(path: &Path) -> Result<()> {
        let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        anyhow::ensure!(version == 4, "unsupported backup state schema");
        validate_schema(&conn, version)?;
        anyhow::ensure!(
            conn.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? == "ok",
            "backup SQLite integrity failure"
        );
        anyhow::ensure!(
            conn.prepare("PRAGMA foreign_key_check")?
                .query([])?
                .next()?
                .is_none(),
            "backup foreign key integrity failure"
        );
        let missing: i64 = conn.query_row("SELECT count(*) FROM projects p LEFT JOIN project_scopes s ON s.project_id=p.id WHERE s.scope IS NULL OR s.root!=p.root OR s.identity!=p.identity OR length(s.scope)!=64 OR s.scope GLOB '*[^0-9a-f]*'", [], |r| r.get(0))?;
        anyhow::ensure!(missing == 0, "backup project namespace integrity failure");
        Ok(())
    }

    /// A CLI-only boundary: the MCP server never exposes project registration.
    pub fn register_project(&self, path: &Path) -> Result<Value> {
        let root = path
            .canonicalize()
            .context("project directory must exist")?;
        if !root.is_dir() {
            bail!("project must be a directory");
        }
        util::reject_symlink_components(&root)?;
        let root = root.to_str().context("project path must be UTF-8")?;
        bounded(root, "project path", 4096)?;
        let current_identity = identity::current(Path::new(root))?;
        let mut conn = self.connection()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous: Option<(String, String, i64)> = tx
            .query_row(
                "SELECT id,identity,created FROM projects WHERE root=?1",
                [root],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let id = if let Some((id, previous, registered)) = previous {
            if !identity::matches_registered(Path::new(root), &previous, registered)? {
                bail!("registered project identity changed; use a distinct project directory");
            }
            id
        } else {
            let retained: Option<(String, String)> = tx
                .query_row(
                    "SELECT project_id,identity FROM project_scopes WHERE root=?1",
                    [root],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            let id = if let Some((id, expected)) = retained {
                anyhow::ensure!(
                    identity::matches(Path::new(root), &expected)?,
                    "durable project identity changed; use a distinct directory"
                );
                if expected != current_identity {
                    anyhow::ensure!(
                        tx.execute(
                            "UPDATE project_scopes SET identity=?1 WHERE project_id=?2 AND identity=?3",
                            params![current_identity, id, expected],
                        )? == 1,
                        "durable project identity changed during registration"
                    );
                }
                id
            } else {
                let scope = util::sha256(root.as_bytes());
                let occupied: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM project_scopes WHERE scope=?1)",
                    [&scope],
                    |r| r.get(0),
                )?;
                anyhow::ensure!(
                    !occupied,
                    "path belongs to a relocated project's durable namespace; choose a different root"
                );
                let id = format!("p_{}", &scope[..24]);
                tx.execute(
                    "INSERT INTO project_scopes VALUES(?1,?2,?3,?4)",
                    params![id, scope, root, current_identity],
                )?;
                id
            };
            tx.execute(
                "INSERT INTO projects(id,root,identity,created) VALUES(?1,?2,?3,?4)",
                params![id, root, current_identity, now()],
            )?;
            id
        };
        tx.commit()?;
        Ok(json!({"id":id,"root":root,"registered":true}))
    }

    /// CLI-only removal of coordination records. Project files are never inspected or changed.
    pub fn unregister_project(&self, project_id: &str) -> Result<Value> {
        bounded(project_id, "project id", 64)?;
        let mut conn = self.connection()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let root: String = tx
            .query_row(
                "SELECT root FROM projects WHERE id=?1",
                [project_id],
                |row| row.get(0),
            )
            .optional()?
            .context("project is not registered")?;
        let operations = tx.execute("DELETE FROM operations WHERE scope IN (SELECT id FROM tasks WHERE project_id=?1) OR scope=?2",params![project_id,format!("project:{project_id}")])?;
        let deliveries = tx.execute(
            "DELETE FROM deliveries WHERE task_id IN (SELECT id FROM tasks WHERE project_id=?1)",
            [project_id],
        )?;
        let evidence = tx.execute(
            "DELETE FROM evidence WHERE task_id IN (SELECT id FROM tasks WHERE project_id=?1)",
            [project_id],
        )?;
        let checkpoints = tx.execute(
            "DELETE FROM checkpoints WHERE task_id IN (SELECT id FROM tasks WHERE project_id=?1)",
            [project_id],
        )?;
        let tasks = tx.execute("DELETE FROM tasks WHERE project_id=?1", [project_id])?;
        let projects = tx.execute("DELETE FROM projects WHERE id=?1", [project_id])?;
        tx.commit()?;
        Ok(json!({"ok":true,"project_id":project_id,"root":root,
            "deleted":{"projects":projects,"tasks":tasks,"skill_deliveries":deliveries,"evidence":evidence,"checkpoints":checkpoints,"operations":operations},
            "project_files":"unchanged", "runtime_namespace":"retained; unregister removes MCP registration, not memory identity"}))
    }

    fn project_rows(conn: &Connection, limit: usize, offset: usize) -> Result<Vec<Value>> {
        let mut query =
            conn.prepare("SELECT id,root FROM projects ORDER BY root LIMIT ?1 OFFSET ?2")?;
        Ok(query
            .query_map(params![limit, offset], |row| {
                Ok(json!({"id":row.get::<_,String>(0)?,"root":row.get::<_,String>(1)?}))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Stable namespace for registered roots; legacy CLI-only roots retain path hashing.
    pub fn runtime_scope(&self, path: &Path) -> Result<String> {
        let canonical = crate::paths::canonical_project(path)?;
        let root = canonical.to_str().context("project path must be UTF-8")?;
        let conn = self.connection()?;
        if let Some((scope, expected, registered)) = conn
            .query_row(
                "SELECT s.scope,s.identity,p.created FROM project_scopes s LEFT JOIN projects p ON p.id=s.project_id WHERE s.root=?1",
                [root],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<i64>>(2)?)),
            )
            .optional()?
        {
            util::reject_symlink_components(&canonical)?;
            anyhow::ensure!(
                if let Some(created) = registered {
                    identity::matches_registered(&canonical, &expected, created)?
                } else {
                    identity::matches(&canonical, &expected)?
                },
                "durable project identity changed"
            );
            anyhow::ensure!(
                scope.len() == 64 && scope.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid durable namespace"
            );
            return Ok(scope);
        }
        let scope = util::sha256(root.as_bytes());
        let occupied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM project_scopes WHERE scope=?1)",
            [&scope],
            |r| r.get(0),
        )?;
        anyhow::ensure!(
            !occupied,
            "path belongs to a relocated project's durable namespace"
        );
        Ok(scope)
    }

    /// CLI-only rebinding after an actual directory rename. Never moves application files.
    pub fn relocate(&self, id: &str, expected_root: &Path, destination: &Path) -> Result<Value> {
        let _lease = crate::maintenance::exclusive(&self.paths)?;
        anyhow::ensure!(
            !self.paths.root.join("restore-pending.json").exists(),
            "restore is incomplete"
        );
        util::reject_symlink_components(destination)?;
        let destination = crate::paths::canonical_project(destination)?;
        anyhow::ensure!(
            expected_root.is_absolute() && destination != expected_root,
            "distinct absolute roots required"
        );
        let design_root = self
            .paths
            .root
            .join("execution/podman/staging/design/workspace");
        anyhow::ensure!(
            expected_root.starts_with(&design_root) == destination.starts_with(&design_root),
            "relocation cannot cross the direct design workspace boundary"
        );
        let mut conn = self.unleased_connection()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (root, expected, scope, registered): (String,String,String,i64) = tx.query_row(
            "SELECT p.root,p.identity,s.scope,p.created FROM projects p JOIN project_scopes s ON s.project_id=p.id WHERE p.id=?1", [id],
            |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?.context("project is not registered")?;
        // The original inode/creation identity is retained; copied replacements are not relocation.
        anyhow::ensure!(
            identity::matches_registered(&destination, &expected, registered)?,
            "destination is not the registered directory; rename the original project, do not copy it"
        );
        if Path::new(&root) == destination {
            return Ok(
                json!({"relocated":true,"replayed":true,"id":id,"root":destination,"scope":scope}),
            );
        }
        anyhow::ensure!(
            Path::new(&root) == expected_root,
            "registered root changed; inspect before relocation"
        );
        util::reject_symlink_components(expected_root)?;
        anyhow::ensure!(
            !expected_root.exists(),
            "old project root still exists; rename it before relocation"
        );
        tx.execute(
            "UPDATE projects SET root=?1 WHERE id=?2",
            params![
                destination.to_str().context("UTF-8 destination required")?,
                id
            ],
        )?;
        tx.execute(
            "UPDATE project_scopes SET root=?1 WHERE project_id=?2",
            params![
                destination.to_str().context("UTF-8 destination required")?,
                id
            ],
        )?;
        tx.commit()?;
        Ok(
            json!({"relocated":true,"id":id,"previous_root":root,"root":destination,"scope":scope,
            "worker_recovery":"Historical absolute bindings remain unchanged. Start a new worker at the relocated root; old checkpoints cannot dispatch there.",
            "project_files":"not moved or rewritten"}),
        )
    }

    pub fn project_id(&self, path: &Path) -> Result<String> {
        let canonical = path
            .canonicalize()
            .context("project directory must exist")?;
        let canonical = canonical.to_str().context("project path must be UTF-8")?;
        let conn = self.connection()?;
        let id: String = conn
            .query_row(
                "SELECT id FROM projects WHERE root=?1",
                [canonical],
                |row| row.get(0),
            )
            .optional()?
            .context("project is not registered; run mirket project register --project PATH")?;
        Self::root(&conn, &id)?;
        Ok(id)
    }

    pub fn projects_page(&self, offset: usize, limit: usize) -> Result<Value> {
        if !(1..=10).contains(&limit) || offset > 1_000_000 {
            bail!("project page size must be 1..=10 and offset at most 1000000");
        }
        let conn = self.connection()?;
        let mut projects = Self::project_rows(&conn, limit + 1, offset)?;
        let next_offset = if projects.len() > limit {
            Some(offset + limit)
        } else {
            None
        };
        projects.truncate(limit);
        Ok(json!({"projects":projects,"next_offset":next_offset}))
    }

    pub fn tasks(&self, project_id: &str, cursor: Option<&str>, limit: usize) -> Result<Value> {
        if !(1..=20).contains(&limit) {
            bail!("task page size must be 1..=20");
        }
        let conn = self.connection()?;
        Self::root(&conn, project_id)?;
        let position = cursor
            .map(|value| task_position(project_id, value))
            .transpose()?;
        // Separate statements let SQLite seek directly to a cursor boundary;
        // an optional-position OR predicate otherwise scans the project prefix.
        let mut query = conn.prepare(if position.is_some() {
            TASKS_CURSOR_PAGE
        } else {
            TASKS_FIRST_PAGE
        })?;
        let read_task = |row: &rusqlite::Row<'_>| -> rusqlite::Result<Value> {
            let objective: String = row.get(1)?;
            let preview: String = objective.chars().take(256).collect();
            Ok(
                json!({"id":row.get::<_,String>(0)?,"objective_preview":preview,"status":row.get::<_,String>(2)?,"revision":row.get::<_,i64>(3)?,"updated":row.get::<_,i64>(4)?,"created":row.get::<_,i64>(5)?}),
            )
        };
        let mut tasks = if let Some((created, id)) = &position {
            query
                .query_map(params![project_id, created, id, limit + 1], read_task)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            query
                .query_map(params![project_id, limit + 1], read_task)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        let next_cursor = if tasks.len() > limit {
            tasks.truncate(limit);
            let last = tasks.last().context("missing task page boundary")?;
            Some(format!(
                "tasks1:{project_id}:{}:{}",
                last["created"]
                    .as_i64()
                    .context("invalid task creation time")?,
                last["id"].as_str().context("invalid task id")?
            ))
        } else {
            None
        };
        Ok(json!({"project_id":project_id,"tasks":tasks,"next_cursor":next_cursor}))
    }

    fn root(conn: &Connection, project_id: &str) -> Result<PathBuf> {
        bounded(project_id, "project id", 64)?;
        let (root, expected, registered): (String, String, i64) = conn
            .query_row(
                "SELECT root,identity,created FROM projects WHERE id=?1",
                [project_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?
            .context("project is not registered; run mirket project register through the host")?;
        let path = PathBuf::from(root);
        util::reject_symlink_components(&path)?;
        if !path.is_dir()
            || path.canonicalize()? != path
            || !identity::matches_registered(&path, &expected, registered)?
        {
            bail!("registered project identity changed");
        }
        Ok(path)
    }

    pub fn begin(
        &self,
        project_id: &str,
        objective: &str,
        skills: &[String],
        acceptance: &str,
        failure_probe: &str,
        key: &str,
    ) -> Result<Value> {
        bounded(objective, "objective", 4096)?;
        bounded(acceptance, "acceptance", 2048)?;
        bounded(failure_probe, "failure probe", 2048)?;
        if skills.is_empty() || skills.len() > MAX_SKILLS {
            bail!("select 1..={MAX_SKILLS} skills");
        }
        let mut selected = std::collections::BTreeSet::new();
        let catalog = Catalog::embedded()?;
        for skill in skills {
            bounded(skill, "skill id", 128)?;
            if !selected.insert(skill) {
                bail!("duplicate selected skill: {skill}");
            }
            catalog
                .read(skill, "SKILL.md", 65536)
                .with_context(|| format!("selected skill {skill} must be locally readable"))?;
        }
        let args = json!([
            "begin",
            project_id,
            objective,
            skills,
            acceptance,
            failure_probe
        ]);
        let scope = format!("project:{project_id}");
        self.mutate(&scope,key,&args,|tx| {
            Self::root(tx,project_id)?;
            let id = format!("t_{}", &util::sha256(format!("{project_id}\0{key}").as_bytes())[..32]);
            tx.execute("INSERT INTO tasks(id,project_id,objective,skills,acceptance,failure_probe,status,revision,created,updated) VALUES(?1,?2,?3,?4,?5,?6,'active',1,?7,?7)",
                params![id,project_id,objective,serde_json::to_string(skills)?,acceptance,failure_probe,now()])?;
            Self::snapshot(tx,&id)
        })
    }

    pub fn status(&self, task_id: &str) -> Result<Value> {
        bounded(task_id, "task id", 64)?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        Self::snapshot(&tx, task_id)
    }

    /// Read immutable checkpoints newest first. A cursor resumes strictly before
    /// the last returned revision, so later appends do not shift older pages.
    pub fn task_history(&self, task_id: &str, cursor: Option<&str>, limit: usize) -> Result<Value> {
        bounded(task_id, "task id", 64)?;
        if !(1..=20).contains(&limit) {
            bail!("history page size must be 1..=20");
        }
        let position = cursor
            .map(|value| history_position(task_id, value))
            .transpose()?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let project: String = tx
            .query_row(
                "SELECT project_id FROM tasks WHERE id=?1",
                [task_id],
                |row| row.get(0),
            )
            .optional()?
            .context("task does not exist")?;
        Self::root(&tx, &project)?;
        let mut query = tx.prepare("SELECT revision,note,created FROM checkpoints WHERE task_id=?1 AND (?2 IS NULL OR revision<?2) ORDER BY revision DESC LIMIT ?3")?;
        let rows = query.query_map(params![task_id,position,limit+1], |row| {
            Ok(json!({"revision":row.get::<_,i64>(0)?,"note":row.get::<_,String>(1)?,"created":row.get::<_,i64>(2)?}))
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        let mut checkpoints = Vec::new();
        // Reserve response envelope/cursor space, including JSON escaping.
        let mut remaining = MAX_SNAPSHOT_BYTES - 1024;
        let mut more = false;
        for checkpoint in rows {
            let bytes = serde_json::to_vec(&checkpoint)?.len() + 1;
            if checkpoints.len() == limit || bytes > remaining {
                more = true;
                break;
            }
            remaining -= bytes;
            checkpoints.push(checkpoint);
        }
        let next_cursor = if more {
            let revision = checkpoints
                .last()
                .context("checkpoint exceeds history page budget")?["revision"]
                .as_i64()
                .context("invalid checkpoint revision")?;
            Some(format!("history1:{task_id}:{revision}"))
        } else {
            None
        };
        Ok(json!({"task_id":task_id,"checkpoints":checkpoints,"next_cursor":next_cursor}))
    }

    fn snapshot(conn: &Connection, task_id: &str) -> Result<Value> {
        let mut value = conn.query_row("SELECT id,project_id,objective,skills,acceptance,failure_probe,status,revision,created,updated FROM tasks WHERE id=?1", [task_id], |row| {
            Ok(json!({"id":row.get::<_,String>(0)?,"project_id":row.get::<_,String>(1)?,"objective":row.get::<_,String>(2)?,
                "skills":row.get::<_,String>(3)?,"acceptance":row.get::<_,String>(4)?,"failure_probe":row.get::<_,String>(5)?,
                "status":row.get::<_,String>(6)?,"revision":row.get::<_,i64>(7)?,"created":row.get::<_,i64>(8)?,"updated":row.get::<_,i64>(9)?}))
        }).optional()?.context("task does not exist")?;
        value["skills"] =
            serde_json::from_str(value["skills"].as_str().context("invalid task skills")?)?;
        let mut query = conn.prepare("SELECT skill_id,content_hash,payload_hash,delivered FROM deliveries WHERE task_id=?1 ORDER BY skill_id")?;
        value["skill_deliveries"] = Value::Array(query.query_map([task_id], |row| Ok(json!({"skill_id":row.get::<_,String>(0)?,"content_sha256":row.get::<_,String>(1)?,"payload_sha256":row.get::<_,String>(2)?,"delivered":row.get::<_,i64>(3)?})))?.collect::<rusqlite::Result<Vec<_>>>()?);
        let mut query = conn.prepare("SELECT path,kind,summary,sha256,size,recorded FROM evidence WHERE task_id=?1 ORDER BY kind,path")?;
        value["evidence"] = Value::Array(query.query_map([task_id], |row| Ok(json!({"path":row.get::<_,String>(0)?,"kind":row.get::<_,String>(1)?,"summary":row.get::<_,String>(2)?,"sha256":row.get::<_,String>(3)?,"bytes":row.get::<_,i64>(4)?,"recorded":row.get::<_,i64>(5)?})))?.collect::<rusqlite::Result<Vec<_>>>()?);
        let mut query =
            conn.prepare("SELECT path,kind,contract FROM contracts WHERE task_id=?1")?;
        let rows = query.query_map([task_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (path, kind, contract) = row?;
            for entry in value["evidence"]
                .as_array_mut()
                .context("invalid evidence")?
            {
                if entry["path"] == path && entry["kind"] == kind {
                    entry["contract"] = json!(contract);
                }
            }
        }
        let mut query = conn.prepare("SELECT revision,note,created FROM checkpoints WHERE task_id=?1 ORDER BY revision DESC LIMIT 8")?;
        value["recent_checkpoints"] = Value::Array(query.query_map([task_id], |row| Ok(json!({"revision":row.get::<_,i64>(0)?,"note":row.get::<_,String>(1)?,"created":row.get::<_,i64>(2)?})))?.collect::<rusqlite::Result<Vec<_>>>()?);
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM checkpoints WHERE task_id=?1",
            [task_id],
            |row| row.get(0),
        )?;
        value["checkpoint_count"] = json!(count);
        value["history_has_more"] = json!(count > 8);
        value["authority"] = json!(
            "Mirket records delivered guidance and file identities. The host executes work. Acceptance and failure-probe summaries are caller attestations; hashes do not establish semantic correctness."
        );
        Ok(value)
    }

    fn mutate(
        &self,
        scope: &str,
        key: &str,
        arguments: &Value,
        operation: impl FnOnce(&Transaction<'_>) -> Result<Value>,
    ) -> Result<Value> {
        bounded(key, "idempotency key", 128)?;
        let fingerprint = util::sha256(&serde_json::to_vec(arguments)?);
        let mut conn = self.connection()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous: Option<(String, String)> = tx
            .query_row(
                "SELECT fingerprint,response FROM operations WHERE scope=?1 AND key=?2",
                params![scope, key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((expected, response)) = previous {
            if expected != fingerprint {
                bail!("idempotency key was already used with different arguments");
            }
            return Ok(serde_json::from_str(&response)?);
        }
        let snapshot = operation(&tx)?;
        // Compact output must not make committed state unreadable by status.
        if serde_json::to_vec(&snapshot)?.len() > MAX_SNAPSHOT_BYTES {
            bail!(
                "task record would exceed its 60 KiB serialized budget; shorten checkpoint or evidence text, or start a focused follow-up task"
            );
        }
        let value = Self::mutation_receipt(&snapshot, arguments)?;
        let response = serde_json::to_string(&value)?;
        tx.execute(
            "INSERT INTO operations(scope,key,fingerprint,response) VALUES(?1,?2,?3,?4)",
            params![scope, key, fingerprint, response],
        )?;
        tx.commit()?;
        Ok(value)
    }

    /// Versioned success envelope. Stored older replies intentionally bypass this
    /// conversion: retrying the same key must return the original exact value.
    fn mutation_receipt(snapshot: &Value, arguments: &Value) -> Result<Value> {
        let operation = arguments[0]
            .as_str()
            .context("missing mutation operation")?;
        let result = match operation {
            "begin" => json!({"skills": snapshot["skills"]}),
            "checkpoint" | "reopen" => {
                let checkpoint = &snapshot["recent_checkpoints"][0];
                let note = checkpoint["note"].as_str().context("missing checkpoint")?;
                json!({"checkpoint_revision":checkpoint["revision"],
                    "note_sha256":util::sha256(note.as_bytes()),"note_bytes":note.len()})
            }
            "skill" => snapshot["skill_deliveries"]
                .as_array()
                .context("missing deliveries")?
                .iter()
                .find(|entry| entry["skill_id"] == arguments[2])
                .context("missing recorded delivery")?
                .clone(),
            "evidence" => {
                let mut entry = snapshot["evidence"]
                    .as_array()
                    .context("missing evidence")?
                    .iter()
                    .find(|entry| entry["path"] == arguments[2] && entry["kind"] == arguments[3])
                    .context("missing recorded evidence")?
                    .clone();
                entry
                    .as_object_mut()
                    .context("invalid evidence")?
                    .remove("summary");
                entry
            }
            "complete" => {
                json!({"evidence_count": snapshot["evidence"].as_array().context("missing evidence")?.len()})
            }
            _ => bail!("unknown task mutation operation"),
        };
        Ok(json!({"receipt_version":1,"operation":operation,
            "id":snapshot["id"],"project_id":snapshot["project_id"],
            "revision":snapshot["revision"],"status":snapshot["status"],
            "updated":snapshot["updated"],"result":result}))
    }

    fn expect(conn: &Connection, task_id: &str, revision: i64, status: &str) -> Result<Value> {
        bounded(task_id, "task id", 64)?;
        if revision < 1 {
            bail!("revision must be positive");
        }
        let value = Self::snapshot(conn, task_id)?;
        if value["revision"].as_i64() != Some(revision) {
            bail!("task revision conflict; read task status before retrying");
        }
        if value["status"] != status {
            bail!("task must be {status}");
        }
        Self::root(
            conn,
            value["project_id"].as_str().context("invalid project id")?,
        )?;
        Ok(value)
    }

    fn advance(tx: &Transaction<'_>, task_id: &str) -> Result<Value> {
        tx.execute(
            "UPDATE tasks SET revision=revision+1,updated=?2 WHERE id=?1",
            params![task_id, now()],
        )?;
        Self::snapshot(tx, task_id)
    }

    pub fn checkpoint(&self, task_id: &str, revision: i64, note: &str, key: &str) -> Result<Value> {
        bounded(note, "checkpoint", 2048)?;
        self.mutate(task_id, key, &json!(["checkpoint", revision, note]), |tx| {
            Self::expect(tx, task_id, revision, "active")?;
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM checkpoints WHERE task_id=?1",
                [task_id],
                |row| row.get(0),
            )?;
            if count >= MAX_CHECKPOINTS {
                bail!("task checkpoint limit reached; create a bounded follow-up task");
            }
            tx.execute(
                "INSERT INTO checkpoints(task_id,revision,note,created) VALUES(?1,?2,?3,?4)",
                params![task_id, revision + 1, note, now()],
            )?;
            Self::advance(tx, task_id)
        })
    }

    pub fn record_skill(
        &self,
        task_id: &str,
        revision: i64,
        skill_id: &str,
        content_hash: &str,
        key: &str,
    ) -> Result<Value> {
        bounded(skill_id, "skill id", 128)?;
        let catalog = Catalog::embedded()?;
        let doc = catalog.read(skill_id, "SKILL.md", 65536)?;
        if doc.sha256 != content_hash {
            bail!("skill content identity does not match the current catalog");
        }
        let payload_hash = catalog.payload_digest(skill_id)?.to_owned();
        self.mutate(task_id,key,&json!(["skill",revision,skill_id,content_hash]),|tx| {
            let task = Self::expect(tx,task_id,revision,"active")?;
            if !task["skills"].as_array().context("invalid task skills")?.iter().any(|id|id == skill_id) {
                bail!("skill is not selected for this task");
            }
            tx.execute("INSERT INTO deliveries(task_id,skill_id,content_hash,payload_hash,delivered) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(task_id,skill_id) DO UPDATE SET content_hash=excluded.content_hash,payload_hash=excluded.payload_hash,delivered=excluded.delivered",params![task_id,skill_id,content_hash,payload_hash,now()])?;
            Self::advance(tx,task_id)
        })
    }

    fn file_evidence(root: &Path, relative: &str, max_bytes: u64) -> Result<CheckedEvidence> {
        bounded(relative, "evidence path", 512)?;
        util::safe_relative(Path::new(relative))?;
        let path = root.join(relative);
        util::reject_symlink_components(&path)?;
        if !path.canonicalize()?.starts_with(root) {
            bail!("evidence escaped the registered project");
        }
        let before = fs::metadata(&path)?;
        if !before.is_file() || before.len() > max_bytes {
            bail!("evidence must be a regular file no larger than {max_bytes} bytes");
        }
        let before_identity = identity(&path)?;
        let bytes = util::read_bounded(&path, max_bytes)?;
        let after = fs::metadata(&path)?;
        util::reject_symlink_components(&path)?;
        if before_identity != identity(&path)?
            || before.len() != after.len()
            || before.modified()? != after.modified()?
        {
            bail!("evidence changed while reading; retry after the writer finishes");
        }
        Ok(CheckedEvidence {
            sha256: util::sha256(&bytes),
            size: bytes.len() as u64,
            bytes,
        })
    }

    pub fn evidence(
        &self,
        task_id: &str,
        revision: i64,
        relative_path: &str,
        kind: &str,
        summary: &str,
        key: &str,
    ) -> Result<Value> {
        self.evidence_with_contract(task_id, revision, relative_path, kind, summary, key, None)
    }

    /// Validate an explicitly selected contract without recording or executing it.
    pub fn validate_contract(
        &self,
        task_id: &str,
        path: &str,
        kind: ContractKind,
        completion: bool,
    ) -> Result<Value> {
        let conn = self.connection()?;
        let task = Self::snapshot(&conn, task_id)?;
        let root = Self::root(
            &conn,
            task["project_id"].as_str().context("invalid project id")?,
        )?;
        let checked = Self::file_evidence(&root, path, MAX_CONTRACT_BYTES)?;
        contracts::validate(&root, &task, kind, &checked.bytes, completion)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn evidence_with_contract(
        &self,
        task_id: &str,
        revision: i64,
        relative_path: &str,
        kind: &str,
        summary: &str,
        key: &str,
        contract: Option<ContractKind>,
    ) -> Result<Value> {
        if !matches!(kind, "acceptance" | "failure_probe") {
            bail!("evidence kind must be acceptance or failure_probe");
        }
        bounded(summary, "evidence summary", 1024)?;
        bounded(task_id, "task id", 64)?;
        let arguments = match contract {
            None => json!(["evidence", revision, relative_path, kind, summary]),
            Some(contract) => json!(["evidence", revision, relative_path, kind, summary, contract]),
        };
        if let Some(reply) = self.replay(task_id, key, &arguments)? {
            return Ok(reply);
        }
        let conn = self.connection()?;
        let task = Self::snapshot(&conn, task_id)?;
        let root = Self::root(
            &conn,
            task["project_id"].as_str().context("invalid project id")?,
        )?;
        let max_bytes = if contract.is_some() {
            MAX_CONTRACT_BYTES
        } else {
            MAX_EVIDENCE_BYTES
        };
        let checked = Self::file_evidence(&root, relative_path, max_bytes)?;
        let hash = checked.sha256;
        let size = checked.size;
        let mut attempts = None;
        let roster = if let Some(contract) = contract {
            let bytes = &checked.bytes;
            contracts::validate(&root, &task, contract, bytes, false)?;
            if contract == ContractKind::Contributions {
                let value: Value = serde_json::from_slice(bytes)?;
                let mut generations = serde_json::Map::new();
                for assignment in value["assignments"]
                    .as_array()
                    .context("invalid assignments")?
                {
                    generations.insert(assignment["role"].as_str().context("invalid role")?.into(), json!({"attempt":assignment["attempt"],"host_agent_id":assignment["host_agent_id"]}));
                }
                attempts = Some(serde_json::to_string(&generations)?);
                Some(serde_json::to_string(&value["roster"])?)
            } else {
                None
            }
        } else {
            None
        };
        // File I/O completes before taking SQLite's writer lock.
        self.mutate(task_id,key,&arguments,|tx| {
            Self::expect(tx,task_id,revision,"active")?;
            let previous: Option<(String, Option<String>, Option<String>)> = tx.query_row(
                "SELECT contract,roster,attempts FROM contracts WHERE task_id=?1 AND path=?2 AND kind=?3",
                params![task_id,relative_path,kind], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional()?;
            if let Some((previous, original_roster, previous_attempts)) = previous {
                if contract.map(|kind| kind.as_str()) != Some(previous.as_str()) { bail!("cannot remove or change a recorded evidence contract"); }
                if original_roster != roster { bail!("contributions must retain their originally recorded roster binding"); }
                if let Some(previous_attempts) = previous_attempts {
                    let old: serde_json::Map<String, Value> = serde_json::from_str(&previous_attempts)?;
                    let new: Value = serde_json::from_str(attempts.as_deref().context("missing contribution attempts")?)?;
                    for (role, prior) in old {
                        let before = prior["attempt"].as_u64().context("invalid persisted attempt")?;
                        let after = new[&role]["attempt"].as_u64().context("missing contribution attempt")?;
                        if after < before { bail!("stale contribution attempt for {role}"); }
                        if after == before && !prior["host_agent_id"].is_null() && prior["host_agent_id"] != new[&role]["host_agent_id"] {
                            bail!("changing host_agent_id requires a new attempt for {role}");
                        }
                    }
                }
            }
            let count: i64 = tx.query_row("SELECT count(*) FROM evidence WHERE task_id=?1",[task_id],|row|row.get(0))?;
            let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM evidence WHERE task_id=?1 AND path=?2 AND kind=?3)",params![task_id,relative_path,kind],|row|row.get(0))?;
            if count >= MAX_EVIDENCE && !exists { bail!("task evidence limit reached ({MAX_EVIDENCE})"); }
            tx.execute("INSERT INTO evidence(task_id,path,kind,summary,sha256,size,recorded) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(task_id,path,kind) DO UPDATE SET summary=excluded.summary,sha256=excluded.sha256,size=excluded.size,recorded=excluded.recorded",params![task_id,relative_path,kind,summary,hash,size,now()])?;
            if let Some(contract) = contract {
                tx.execute("INSERT INTO contracts(task_id,path,kind,contract,roster,attempts) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(task_id,path,kind) DO UPDATE SET attempts=excluded.attempts", params![task_id,relative_path,kind,contract.as_str(),roster,attempts])?;
            }
            Self::advance(tx,task_id)
        })
    }

    pub fn complete(&self, task_id: &str, revision: i64, key: &str) -> Result<Value> {
        // An exact retry of a successful operation remains idempotent, even if artifacts later change.
        let arguments = json!(["complete", revision]);
        if let Some(reply) = self.replay(task_id, key, &arguments)? {
            return Ok(reply);
        }
        let conn = self.connection()?;
        let task = Self::expect(&conn, task_id, revision, "active")?;
        let root = Self::root(
            &conn,
            task["project_id"].as_str().context("invalid project id")?,
        )?;
        let catalog = Catalog::embedded()?;
        for skill in task["skills"].as_array().context("invalid task skills")? {
            let skill_id = skill.as_str().context("invalid skill id")?;
            let receipt = task["skill_deliveries"]
                .as_array()
                .context("invalid deliveries")?
                .iter()
                .find(|entry| entry["skill_id"] == skill_id)
                .with_context(|| format!("selected skill has not been delivered: {skill_id}"))?;
            if receipt["payload_sha256"] != catalog.payload_digest(skill_id)? {
                bail!("skill payload changed; read {skill_id} again");
            }
        }
        let evidence = task["evidence"].as_array().context("invalid evidence")?;
        for kind in ["acceptance", "failure_probe"] {
            if !evidence.iter().any(|entry| entry["kind"] == kind) {
                bail!("missing {kind} evidence");
            }
        }
        let mut contract_bytes = 0u64;
        for entry in evidence {
            let relative = entry["path"].as_str().context("invalid evidence path")?;
            let max_bytes = if entry["contract"].is_string() {
                MAX_CONTRACT_BYTES
            } else {
                MAX_EVIDENCE_BYTES
            };
            let checked = Self::file_evidence(&root, relative, max_bytes)?;
            if entry["sha256"] != checked.sha256 || entry["bytes"].as_u64() != Some(checked.size) {
                bail!("evidence is stale: {relative}");
            }
            if let Some(kind) = entry["contract"].as_str() {
                let result =
                    contracts::validate(&root, &task, kind.parse()?, &checked.bytes, true)?;
                contract_bytes += result["bound_bytes"]
                    .as_u64()
                    .context("invalid contract byte count")?;
                if contract_bytes > 64 * 1024 * 1024 {
                    bail!("task contracts exceed 64 MiB cumulative binding budget; split the task");
                }
            }
        }
        self.mutate(task_id, key, &arguments, |tx| {
            Self::expect(tx, task_id, revision, "active")?;
            tx.execute("UPDATE tasks SET status='complete' WHERE id=?1", [task_id])?;
            Self::advance(tx, task_id)
        })
    }

    fn replay(&self, scope: &str, key: &str, arguments: &Value) -> Result<Option<Value>> {
        bounded(key, "idempotency key", 128)?;
        let conn = self.connection()?;
        let previous: Option<(String, String)> = conn
            .query_row(
                "SELECT fingerprint,response FROM operations WHERE scope=?1 AND key=?2",
                params![scope, key],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        match previous {
            None => Ok(None),
            Some((expected, response)) => {
                if expected != util::sha256(&serde_json::to_vec(arguments)?) {
                    bail!("idempotency key was already used with different arguments");
                }
                Ok(Some(serde_json::from_str(&response)?))
            }
        }
    }

    pub fn reopen(&self, task_id: &str, revision: i64, reason: &str, key: &str) -> Result<Value> {
        bounded(reason, "reason", 2048)?;
        self.mutate(task_id, key, &json!(["reopen", revision, reason]), |tx| {
            Self::expect(tx, task_id, revision, "complete")?;
            let count: i64 = tx.query_row(
                "SELECT count(*) FROM checkpoints WHERE task_id=?1",
                [task_id],
                |row| row.get(0),
            )?;
            if count >= MAX_CHECKPOINTS {
                bail!("task checkpoint limit reached; create a bounded follow-up task");
            }
            tx.execute("UPDATE tasks SET status='active' WHERE id=?1", [task_id])?;
            tx.execute(
                "INSERT INTO checkpoints(task_id,revision,note,created) VALUES(?1,?2,?3,?4)",
                params![task_id, revision + 1, reason, now()],
            )?;
            Self::advance(tx, task_id)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_paging_queries_use_ordered_index_and_cursor_range() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let state = State::open(&paths).unwrap();
        let conn = state.connection().unwrap();
        for (sql, cursor) in [(TASKS_FIRST_PAGE, false), (TASKS_CURSOR_PAGE, true)] {
            let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
            let values: Vec<rusqlite::types::Value> = if cursor {
                vec![
                    "p_example".to_owned().into(),
                    10.into(),
                    "t_example".to_owned().into(),
                    21.into(),
                ]
            } else {
                vec!["p_example".to_owned().into(), 21.into()]
            };
            let plan = statement
                .query_map(rusqlite::params_from_iter(values), |row| {
                    row.get::<_, String>(3)
                })
                .unwrap()
                .collect::<rusqlite::Result<Vec<_>>>()
                .unwrap()
                .join("\n");
            // Diagnostic assertions against this repository's locked SQLite,
            // not parsing a query plan as part of the application API.
            assert!(plan.contains("tasks_project_created"), "{plan}");
            assert!(!plan.contains("TEMP B-TREE"), "{plan}");
            if cursor {
                assert!(plan.contains("created") && plan.contains('<'), "{plan}");
            }
        }
    }

    #[test]
    fn checked_evidence_returns_the_validated_bytes_and_applies_the_requested_limit() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let path = root.join("evidence.json");
        let contents = br#"{"ok":true}"#;
        fs::write(&path, contents).unwrap();

        let checked = State::file_evidence(&root, "evidence.json", 128).unwrap();
        assert_eq!(checked.bytes, contents);
        assert_eq!(checked.size, contents.len() as u64);
        assert_eq!(checked.sha256, util::sha256(contents));
        let error = State::file_evidence(&root, "evidence.json", 4)
            .err()
            .expect("the requested evidence limit must reject the file")
            .to_string();
        assert!(error.contains("4 bytes"), "{error}");
    }

    #[test]
    fn project_pages_reach_every_registered_root_without_truncation() {
        let home = tempfile::tempdir().unwrap();
        let root = home.path().canonicalize().unwrap();
        let paths = Paths::new(root.clone()).unwrap();
        let state = State::open(&paths).unwrap();
        for index in 0..129 {
            let project = root.join(format!("project-{index:03}"));
            fs::create_dir(&project).unwrap();
            state.register_project(&project).unwrap();
        }

        let mut offset = 0;
        let mut projects = Vec::new();
        loop {
            let page = state.projects_page(offset, 10).unwrap();
            let rows = page["projects"].as_array().unwrap();
            assert!(rows.len() <= 10);
            projects.extend(rows.iter().cloned());
            match page["next_offset"].as_u64() {
                Some(next) => {
                    assert_eq!(next as usize, offset + 10);
                    offset = next as usize;
                }
                None => break,
            }
        }
        assert_eq!(projects.len(), 129);
        for (index, project) in projects.iter().enumerate() {
            assert_eq!(
                project["root"],
                root.join(format!("project-{index:03}")).to_str().unwrap()
            );
        }
    }
}
