//! Durable coordination records. These records describe work; the host executes it.
use crate::{catalog::Catalog, paths::Paths, util};
use anyhow::{Context, Result, bail};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MAX_EVIDENCE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SKILLS: usize = 16;
const MAX_EVIDENCE: i64 = 16;
const MAX_CHECKPOINTS: i64 = 256;
const MAX_SNAPSHOT_BYTES: usize = 60 * 1024;

#[derive(Clone, Debug)]
pub struct State {
    database: PathBuf,
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

impl State {
    pub fn open(paths: &Paths) -> Result<Self> {
        paths.ensure()?;
        let state = Self {
            database: paths.database(),
        };
        let conn = state.connection()?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if version != 0 && version != 1 {
            bail!("unsupported Mirket state format: {version}");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&state.database, fs::Permissions::from_mode(0o600))?;
        }
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute_batch("BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY, root TEXT NOT NULL UNIQUE, identity TEXT NOT NULL,
                created INTEGER NOT NULL
            ) STRICT;
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id),
                objective TEXT NOT NULL, skills TEXT NOT NULL, acceptance TEXT NOT NULL,
                failure_probe TEXT NOT NULL, status TEXT NOT NULL CHECK(status IN ('active','complete')),
                revision INTEGER NOT NULL, created INTEGER NOT NULL, updated INTEGER NOT NULL
            ) STRICT;
            CREATE TABLE IF NOT EXISTS deliveries (
                task_id TEXT NOT NULL REFERENCES tasks(id), skill_id TEXT NOT NULL,
                content_hash TEXT NOT NULL, payload_hash TEXT NOT NULL, delivered INTEGER NOT NULL,
                PRIMARY KEY(task_id,skill_id)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS evidence (
                task_id TEXT NOT NULL REFERENCES tasks(id), path TEXT NOT NULL,
                kind TEXT NOT NULL CHECK(kind IN ('acceptance','failure_probe')),
                summary TEXT NOT NULL, sha256 TEXT NOT NULL, size INTEGER NOT NULL, recorded INTEGER NOT NULL,
                PRIMARY KEY(task_id,path,kind)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS checkpoints (
                task_id TEXT NOT NULL REFERENCES tasks(id), revision INTEGER NOT NULL,
                note TEXT NOT NULL, created INTEGER NOT NULL, PRIMARY KEY(task_id,revision)
            ) STRICT;
            CREATE TABLE IF NOT EXISTS operations (
                scope TEXT NOT NULL, key TEXT NOT NULL, fingerprint TEXT NOT NULL,
                response TEXT NOT NULL, PRIMARY KEY(scope,key)
            ) STRICT;
            CREATE INDEX IF NOT EXISTS tasks_project ON tasks(project_id,updated);
            PRAGMA user_version=1;
            COMMIT;")?;
        Ok(state)
    }

    fn connection(&self) -> Result<Connection> {
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
        let current_identity = identity(Path::new(root))?;
        let id = format!("p_{}", &util::sha256(root.as_bytes())[..24]);
        let mut conn = self.connection()?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous: Option<String> = tx
            .query_row("SELECT identity FROM projects WHERE id=?1", [&id], |row| {
                row.get(0)
            })
            .optional()?;
        if let Some(previous) = previous {
            if previous != current_identity {
                bail!("registered project identity changed; use a distinct project directory");
            }
        } else {
            tx.execute(
                "INSERT INTO projects(id,root,identity,created) VALUES(?1,?2,?3,?4)",
                params![id, root, current_identity, now()],
            )?;
        }
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
            "project_files":"unchanged"}))
    }

    pub fn projects(&self) -> Result<Value> {
        let conn = self.connection()?;
        let mut query = conn.prepare("SELECT id,root FROM projects ORDER BY root LIMIT 129")?;
        let rows = query.query_map([], |row| {
            Ok(json!({"id":row.get::<_,String>(0)?,"root":row.get::<_,String>(1)?}))
        })?;
        let mut values = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        let truncated = values.len() > 128;
        values.truncate(128);
        Ok(json!({"projects":values,"truncated":truncated}))
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
        let mut query =
            conn.prepare("SELECT id,root FROM projects ORDER BY root LIMIT ?1 OFFSET ?2")?;
        let mut projects = query
            .query_map(params![limit + 1, offset], |row| {
                Ok(json!({"id":row.get::<_,String>(0)?,"root":row.get::<_,String>(1)?}))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let next_offset = if projects.len() > limit {
            Some(offset + limit)
        } else {
            None
        };
        projects.truncate(limit);
        Ok(json!({"projects":projects,"next_offset":next_offset}))
    }

    pub fn tasks(&self, project_id: &str, offset: usize, limit: usize) -> Result<Value> {
        if !(1..=20).contains(&limit) || offset > 1_000_000 {
            bail!("task page size must be 1..=20 and offset at most 1000000");
        }
        let conn = self.connection()?;
        Self::root(&conn, project_id)?;
        let mut query = conn.prepare("SELECT id,objective,status,revision,updated FROM tasks WHERE project_id=?1 ORDER BY updated DESC,id LIMIT ?2 OFFSET ?3")?;
        let mut tasks = query.query_map(params![project_id,limit+1,offset], |row| {
            let objective: String = row.get(1)?;
            let preview: String = objective.chars().take(256).collect();
            Ok(json!({"id":row.get::<_,String>(0)?,"objective_preview":preview,"status":row.get::<_,String>(2)?,"revision":row.get::<_,i64>(3)?,"updated":row.get::<_,i64>(4)?}))
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        let next_offset = if tasks.len() > limit {
            Some(offset + limit)
        } else {
            None
        };
        tasks.truncate(limit);
        Ok(json!({"project_id":project_id,"tasks":tasks,"next_offset":next_offset}))
    }

    fn root(conn: &Connection, project_id: &str) -> Result<PathBuf> {
        bounded(project_id, "project id", 64)?;
        let (root, expected): (String, String) = conn
            .query_row(
                "SELECT root,identity FROM projects WHERE id=?1",
                [project_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .context("project is not registered; run mirket project register through the host")?;
        let path = PathBuf::from(root);
        util::reject_symlink_components(&path)?;
        if !path.is_dir() || path.canonicalize()? != path || identity(&path)? != expected {
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
        let mut query = conn.prepare("SELECT revision,note,created FROM checkpoints WHERE task_id=?1 ORDER BY revision DESC LIMIT 8")?;
        value["recent_checkpoints"] = Value::Array(query.query_map([task_id], |row| Ok(json!({"revision":row.get::<_,i64>(0)?,"note":row.get::<_,String>(1)?,"created":row.get::<_,i64>(2)?})))?.collect::<rusqlite::Result<Vec<_>>>()?);
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
        let value = operation(&tx)?;
        let response = serde_json::to_string(&value)?;
        if response.len() > MAX_SNAPSHOT_BYTES {
            bail!(
                "task record would exceed its 60 KiB serialized budget; shorten checkpoint or evidence text, or start a focused follow-up task"
            );
        }
        tx.execute(
            "INSERT INTO operations(scope,key,fingerprint,response) VALUES(?1,?2,?3,?4)",
            params![scope, key, fingerprint, response],
        )?;
        tx.commit()?;
        Ok(value)
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

    fn file_evidence(root: &Path, relative: &str) -> Result<(String, u64)> {
        bounded(relative, "evidence path", 512)?;
        util::safe_relative(Path::new(relative))?;
        let path = root.join(relative);
        util::reject_symlink_components(&path)?;
        if !path.canonicalize()?.starts_with(root) {
            bail!("evidence escaped the registered project");
        }
        let before = fs::metadata(&path)?;
        if !before.is_file() || before.len() > MAX_EVIDENCE_BYTES {
            bail!("evidence must be a regular file no larger than 16 MiB");
        }
        let before_identity = identity(&path)?;
        let bytes = util::read_bounded(&path, MAX_EVIDENCE_BYTES)?;
        let after = fs::metadata(&path)?;
        util::reject_symlink_components(&path)?;
        if before_identity != identity(&path)?
            || before.len() != after.len()
            || before.modified()? != after.modified()?
        {
            bail!("evidence changed while reading; retry after the writer finishes");
        }
        Ok((util::sha256(&bytes), bytes.len() as u64))
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
        if !matches!(kind, "acceptance" | "failure_probe") {
            bail!("evidence kind must be acceptance or failure_probe");
        }
        bounded(summary, "evidence summary", 1024)?;
        bounded(task_id, "task id", 64)?;
        let arguments = json!(["evidence", revision, relative_path, kind, summary]);
        if let Some(reply) = self.replay(task_id, key, &arguments)? {
            return Ok(reply);
        }
        let conn = self.connection()?;
        let task = Self::snapshot(&conn, task_id)?;
        let root = Self::root(
            &conn,
            task["project_id"].as_str().context("invalid project id")?,
        )?;
        let (hash, size) = Self::file_evidence(&root, relative_path)?;
        // File I/O completes before taking SQLite's writer lock.
        self.mutate(task_id,key,&arguments,|tx| {
            Self::expect(tx,task_id,revision,"active")?;
            let count: i64 = tx.query_row("SELECT count(*) FROM evidence WHERE task_id=?1",[task_id],|row|row.get(0))?;
            let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM evidence WHERE task_id=?1 AND path=?2 AND kind=?3)",params![task_id,relative_path,kind],|row|row.get(0))?;
            if count >= MAX_EVIDENCE && !exists { bail!("task evidence limit reached ({MAX_EVIDENCE})"); }
            tx.execute("INSERT INTO evidence(task_id,path,kind,summary,sha256,size,recorded) VALUES(?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(task_id,path,kind) DO UPDATE SET summary=excluded.summary,sha256=excluded.sha256,size=excluded.size,recorded=excluded.recorded",params![task_id,relative_path,kind,summary,hash,size,now()])?;
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
        for entry in evidence {
            let relative = entry["path"].as_str().context("invalid evidence path")?;
            let (hash, size) = Self::file_evidence(&root, relative)?;
            if entry["sha256"] != hash || entry["bytes"].as_u64() != Some(size) {
                bail!("evidence is stale: {relative}");
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
