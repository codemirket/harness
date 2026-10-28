//! Explicitly scoped canonical memory with bounded lexical and semantic recall.
//!
//! Content, source identifiers and locators are data, never instructions or
//! permission grants. No URI is opened, transcript collected, model downloaded
//! or extension loaded. Forgetting and restore eligibility are enforced by the
//! owning lifecycle modules; derived vectors are checked against current heads.

use rusqlite::config::DbConfig;
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, named_params,
    params,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[path = "memory_index.rs"]
pub mod index;
#[path = "memory_list.rs"]
pub mod list;
#[path = "memory_transfer.rs"]
pub(crate) mod transfer;

pub const MAX_CONTENT_BYTES: usize = 16 * 1024;
pub const MAX_RESULTS: usize = 20;
pub const MAX_RESPONSE_BYTES: usize = 32 * 1024;
const SQLITE_VERSION: &str = "3.54.1";
const SQLITE_SOURCE_ID: &str =
    "2026-10-16 14:45:20 555e07ef33f69a0957d24d8e555b702464a131d6f1607f4c3c1e4551e8e6ca76";
const APPLICATION_ID: i64 = 0x4d524b54;
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "project_id", rename_all = "snake_case")]
pub enum Scope {
    Project(String),
    SharedPreferences,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Fact,
    Decision,
    Preference,
    TaskState,
}
impl RecordKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Decision => "decision",
            Self::Preference => "preference",
            Self::TaskState => "task_state",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecordStatus {
    Current,
    Unresolved,
}
impl RecordStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimBasis {
    Observed,
    UserStated,
    Inferred,
}
impl ClaimBasis {
    fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::UserStated => "user_stated",
            Self::Inferred => "inferred",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub source_id: String,
    pub revision: String,
    pub locator: String,
    pub captured_at: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Applicability {
    pub task_key: Option<String>,
    pub checkout_id: Option<String>,
    pub operation: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordInput {
    pub kind: RecordKind,
    pub basis: ClaimBasis,
    pub content: String,
    pub sources: Vec<Source>,
    pub status: RecordStatus,
    pub critical: bool,
    pub applicability: Applicability,
    pub valid_from: Option<u64>,
    /// Exclusive upper bound; absent means no automatic time expiry.
    pub valid_until: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordRef {
    pub record_id: String,
    pub revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub record_id: String,
    pub revision: u64,
    pub data: RecordInput,
    pub recorded_at: u64,
    /// False for retained history or a head explicitly superseded by another ID.
    pub is_current_revision: bool,
    pub supersedes: Option<RecordRef>,
    pub superseded_by: Option<RecordRef>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StatusFilter {
    Current,
    Unresolved,
    Both,
}
impl StatusFilter {
    fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Unresolved => "unresolved",
            Self::Both => "both",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchContext {
    pub task_key: Option<String>,
    pub checkout_id: Option<String>,
    pub operation: Option<String>,
    pub at_time: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchRequest {
    /// Natural-language query: bounded quoted terms form OR/BM25 candidates,
    /// not raw FTS syntax or proof that every term matches the returned record.
    /// Empty text requests applicable heads without lexical filtering.
    pub query: String,
    pub status: StatusFilter,
    pub context: SearchContext,
    pub limit: usize,
    pub max_bytes: usize,
}
impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            status: StatusFilter::Both,
            context: SearchContext::default(),
            limit: MAX_RESULTS,
            max_bytes: MAX_RESPONSE_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchReason {
    CriticalConstraint,
    Lexical,
    Semantic,
    Hybrid,
}
#[derive(Clone, Debug, Serialize)]
pub struct SearchHit {
    pub record: Record,
    pub reason: MatchReason,
}

#[derive(Clone, Debug, Serialize)]
pub struct Coverage {
    pub lexical_only: bool,
    pub semantic_available: bool,
    pub semantic_indexed_eligible: u64,
    pub semantic_missing_eligible: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_warning: Option<String>,
    pub eligible_heads: u64,
    pub lexical_matches: u64,
    /// Only explicitly supplied code/path literals contribute lexical votes to
    /// hybrid ranking. A nearest semantic neighbor is not literal identity.
    pub literal_terms: usize,
    pub literal_matches: u64,
    pub applicable_current_critical: u64,
    pub unclassified_current_critical: u64,
    pub returned_critical: usize,
    pub critical_overflow: bool,
    pub truncated: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct RankedCandidate {
    pub reference: RecordRef,
    pub score: f64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum RankingMode {
    Lexical,
    Semantic,
    Hybrid,
}

#[derive(Clone, Debug, Serialize)]
pub struct SearchResult {
    pub scope: Scope,
    pub hits: Vec<SearchHit>,
    pub coverage: Coverage,
    /// Bounded unmerged ranker evidence for qualification. This is deliberately
    /// not a second public recall payload; MCP returns the bounded merged hits.
    #[serde(skip_serializing)]
    pub ranking: Vec<RankedCandidate>,
}

impl SearchResult {
    pub fn with_semantic_warning(mut self, warning: &str, max_bytes: usize) -> Result<Self> {
        self.coverage.semantic_warning = Some(warning.chars().take(128).collect());
        while serde_json::to_vec(&self)?.len() > max_bytes {
            if self.hits.pop().is_none() {
                return Err(MemoryError::InvalidInput(
                    "recall budget cannot fit degraded coverage metadata",
                ));
            }
            self.coverage.truncated = true;
            self.coverage.returned_critical = self
                .hits
                .iter()
                .filter(|hit| matches!(hit.reason, MatchReason::CriticalConstraint))
                .count();
            self.coverage.critical_overflow =
                self.coverage.applicable_current_critical > self.coverage.returned_critical as u64;
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HistoryPage {
    pub scope: Scope,
    pub record_id: String,
    pub records: Vec<Record>,
    pub has_more: bool,
    pub next_before_revision: Option<u64>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct MutationReceipt {
    pub operation_id: String,
    pub intent_hash: String,
    pub record_id: String,
    pub revision: u64,
    /// False when a later authorized lifecycle operation removed the target.
    /// The consumed operation ID still cannot be replayed as a new mutation.
    pub available: bool,
}

#[derive(Debug)]
pub enum MemoryError {
    InvalidInput(&'static str),
    ScopeMismatch,
    Conflict,
    OperationConflict,
    MutationUnavailable,
    NotFound,
    UnsupportedSqlite,
    UnsupportedSchema,
    Io(io::Error),
    Sql(rusqlite::Error),
    Encoding(serde_json::Error),
    Clock,
}
impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => f.write_str(message),
            Self::ScopeMismatch => f.write_str("memory database belongs to another explicit scope"),
            Self::Conflict => f.write_str("memory revision is stale or superseded; inspect before correcting"),
            Self::OperationConflict => f.write_str("memory operation ID is already bound to another intent"),
            Self::MutationUnavailable => f.write_str("memory operation committed but its exact target is unavailable; it will not be replayed"),
            Self::NotFound => f.write_str("memory record or revision was not found in this scope"),
            Self::UnsupportedSqlite => f.write_str("memory requires the qualified SQLite 3.54.1 source, FTS5 and omitted extension loader"),
            Self::UnsupportedSchema => f.write_str("memory schema is unsupported; no automatic conversion was attempted"),
            Self::Io(e) => write!(f, "memory filesystem operation failed: {e}"),
            Self::Sql(e) => write!(f, "memory database operation failed: {e}"),
            Self::Encoding(_) => f.write_str("memory record encoding is invalid"),
            Self::Clock => f.write_str("memory requires a valid bounded UTC timestamp"),
        }
    }
}
impl std::error::Error for MemoryError {}
impl From<io::Error> for MemoryError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for MemoryError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
    }
}
impl From<serde_json::Error> for MemoryError {
    fn from(e: serde_json::Error) -> Self {
        Self::Encoding(e)
    }
}
type Result<T> = std::result::Result<T, MemoryError>;

pub struct MemoryStore {
    connection: Connection,
    scope: Scope,
    _scope_access: Option<crate::memory_lifecycle::ScopeAccess>,
}

impl MemoryStore {
    pub fn open(state_root: &Path, scope: Scope) -> Result<Self> {
        Self::open_impl(state_root, scope, true)
    }

    /// Inspect an existing canonical store without creating directories,
    /// databases or schemas. SQLite enforces the read-only connection boundary.
    pub fn open_existing(state_root: &Path, scope: Scope) -> Result<Self> {
        Self::open_impl(state_root, scope, false)
    }
    /// Publish rebuildable vectors only into an existing canonical scope. The
    /// index caller must never create a new canonical store after delayed work.
    pub fn open_for_index(state_root: &Path, scope: Scope) -> Result<Self> {
        let access = crate::memory_lifecycle::ScopeAccess::shared(state_root, &scope, false)?;
        let mut store = Self::open_unlocked(state_root, scope, false, true)?;
        access.check_canonical(&store.connection)?;
        crate::memory_backup::check_restore_pending(state_root, &store.scope)?;
        store._scope_access = Some(access);
        Ok(store)
    }

    fn open_impl(state_root: &Path, scope: Scope, create: bool) -> Result<Self> {
        let access = crate::memory_lifecycle::ScopeAccess::shared(state_root, &scope, create)?;
        let mut store = Self::open_unlocked(state_root, scope, create, create)?;
        access.check_canonical(&store.connection)?;
        crate::memory_backup::check_restore_pending(state_root, &store.scope)?;
        store._scope_access = Some(access);
        Ok(store)
    }

    pub(crate) fn open_for_lifecycle(
        state_root: &Path,
        scope: Scope,
        access: &crate::memory_lifecycle::ExclusiveScope,
    ) -> Result<Self> {
        if !access.validates(state_root, &scope)? {
            return Err(MemoryError::ScopeMismatch);
        }
        Self::open_unlocked(state_root, scope, false, true)
    }

    pub(crate) fn lifecycle_connection(&mut self) -> &mut Connection {
        &mut self.connection
    }

    fn open_unlocked(
        state_root: &Path,
        scope: Scope,
        create: bool,
        writable: bool,
    ) -> Result<Self> {
        validate_scope(&scope)?;
        if !state_root.is_absolute()
            || state_root
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(MemoryError::InvalidInput(
                "memory state root must be absolute",
            ));
        }
        existing_directory(state_root)?;
        // Runtime identity is checked before creating any persistent database.
        verify_runtime(&Connection::open_in_memory()?)?;
        let directory = state_root.canonicalize()?.join("memory");
        if create {
            private_directory(&directory, true)?;
        } else {
            existing_directory(&directory)?;
        }
        let scope_json = serde_json::to_string(&scope)?;
        let digest = Sha256::digest(scope_json.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let path = directory.join(format!("scope-{digest}.sqlite3"));
        match fs::symlink_metadata(&path) {
            Ok(_) => private_file(&path)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if !create {
                    return Err(MemoryError::NotFound);
                }
                let mut options = fs::OpenOptions::new();
                options.read(true).write(true).create_new(true);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::OpenOptionsExt;
                    options.mode(0o600);
                }
                match options.open(&path) {
                    Ok(file) => {
                        file.sync_all()?;
                    }
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => (),
                    Err(e) => return Err(e.into()),
                }
                private_file(&path)?;
            }
            Err(e) => return Err(e.into()),
        }
        for suffix in ["-journal", "-wal", "-shm"] {
            let sidecar = PathBuf::from(format!("{}{suffix}", path.to_string_lossy()));
            match fs::symlink_metadata(&sidecar) {
                Ok(_) => private_file(&sidecar)?,
                Err(e) if e.kind() == io::ErrorKind::NotFound => (),
                Err(e) => return Err(e.into()),
            }
        }
        let mut connection = Connection::open_with_flags(
            &path,
            (if writable {
                OpenFlags::SQLITE_OPEN_READ_WRITE
            } else {
                OpenFlags::SQLITE_OPEN_READ_ONLY
            }) | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        verify_runtime(&connection)?;
        connection.busy_timeout(Duration::from_millis(250))?;
        connection.set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)?;
        connection.set_db_config(DbConfig::SQLITE_DBCONFIG_TRUSTED_SCHEMA, false)?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
        let journal_mode: String =
            connection.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
        if journal_mode != "delete" {
            return Err(MemoryError::UnsupportedSchema);
        }
        initialize(&mut connection, &scope_json, create)?;
        Ok(Self {
            connection,
            scope,
            _scope_access: None,
        })
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }
    #[cfg(test)]
    fn database_path(&self) -> &Path {
        Path::new(self.connection.path().expect("file-backed test store"))
    }

    pub fn create(
        &mut self,
        operation_id: &str,
        data: RecordInput,
        supersedes: Option<RecordRef>,
    ) -> Result<Record> {
        validate_operation_id(operation_id)?;
        validate_input(&self.scope, &data)?;
        if let Some(reference) = &supersedes {
            validate_reference(reference)?;
        }
        let intent = intent_hash(&("create", &self.scope, &data, &supersedes))?;
        let recorded = now()?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| MemoryError::Clock)?
            .as_nanos();
        let id = format!(
            "mem-{nanos:x}-{:x}-{:x}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(record) = replay_mutation(&tx, operation_id, &intent)? {
            tx.commit()?;
            return Ok(record);
        }
        if let Some(reference) = &supersedes {
            retire(&tx, reference)?;
        }
        tx.execute(
            "INSERT INTO records(record_id,head_revision) VALUES(?1,1)",
            [&id],
        )?;
        insert_revision(&tx, &id, 1, &data, recorded, supersedes.as_ref())?;
        insert_mutation(&tx, operation_id, &intent, &id, 1)?;
        let result = read_record(&tx, &id, 1)?;
        tx.commit()?;
        Ok(result)
    }

    pub fn revise(
        &mut self,
        operation_id: &str,
        record_id: &str,
        expected_revision: u64,
        data: RecordInput,
    ) -> Result<Record> {
        validate_operation_id(operation_id)?;
        validate_input(&self.scope, &data)?;
        let previous = RecordRef {
            record_id: record_id.into(),
            revision: expected_revision,
        };
        validate_reference(&previous)?;
        let intent = intent_hash(&("revise", &self.scope, record_id, expected_revision, &data))?;
        let next = expected_revision
            .checked_add(1)
            .filter(|n| *n <= i64::MAX as u64)
            .ok_or(MemoryError::Conflict)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(record) = replay_mutation(&tx, operation_id, &intent)? {
            tx.commit()?;
            return Ok(record);
        }
        retire(&tx, &previous)?;
        let changed = tx.execute(
            "UPDATE records SET head_revision=?1 WHERE record_id=?2 AND head_revision=?3",
            params![next as i64, record_id, expected_revision as i64],
        )?;
        if changed != 1 {
            return Err(MemoryError::Conflict);
        }
        insert_revision(&tx, record_id, next, &data, now()?, Some(&previous))?;
        insert_mutation(&tx, operation_id, &intent, record_id, next)?;
        let result = read_record(&tx, record_id, next)?;
        tx.commit()?;
        Ok(result)
    }

    pub fn read_mutation(&mut self, operation_id: &str) -> Result<Option<MutationReceipt>> {
        validate_operation_id(operation_id)?;
        let tx = self.connection.transaction()?;
        let receipt = mutation_receipt(&tx, operation_id)?;
        tx.commit()?;
        Ok(receipt)
    }

    pub fn read(&mut self, record_id: &str, revision: Option<u64>) -> Result<Record> {
        validate_id(record_id)?;
        if let Some(revision) = revision {
            valid_revision(revision)?;
        }
        let tx = self.connection.transaction()?;
        let revision = match revision {
            Some(revision) => revision,
            None => tx
                .query_row(
                    "SELECT head_revision FROM records WHERE record_id=?1",
                    [record_id],
                    |row| unsigned(row, 0),
                )
                .optional()?
                .ok_or(MemoryError::NotFound)?,
        };
        let result = read_record(&tx, record_id, revision)?;
        tx.commit()?;
        Ok(result)
    }

    pub fn history(
        &mut self,
        record_id: &str,
        before_revision: Option<u64>,
        limit: usize,
        max_bytes: usize,
    ) -> Result<HistoryPage> {
        validate_id(record_id)?;
        validate_budget(limit, max_bytes)?;
        if let Some(before) = before_revision {
            valid_revision(before)?;
        }
        let tx = self.connection.transaction()?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM records WHERE record_id=?1)",
            [record_id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(MemoryError::NotFound);
        }
        let revisions: Vec<u64> = {
            let mut statement = tx.prepare("SELECT revision FROM revisions WHERE record_id=?1 AND (?2 IS NULL OR revision<?2) ORDER BY revision DESC LIMIT ?3")?;
            statement
                .query_map(
                    params![
                        record_id,
                        before_revision.map(|v| v as i64),
                        (limit + 1) as i64
                    ],
                    |r| unsigned(r, 0),
                )?
                .collect::<std::result::Result<_, _>>()?
        };
        let has_more = revisions.len() > limit;
        let mut page = HistoryPage {
            scope: self.scope.clone(),
            record_id: record_id.into(),
            records: Vec::new(),
            has_more,
            next_before_revision: None,
            truncated: has_more,
        };
        for revision in revisions.into_iter().take(limit) {
            page.records.push(read_record(&tx, record_id, revision)?);
        }
        loop {
            page.next_before_revision = if page.has_more {
                page.records.last().map(|r| r.revision)
            } else {
                None
            };
            if serde_json::to_vec(&page)?.len() <= max_bytes {
                break;
            }
            page.records.pop();
            page.has_more = true;
            page.truncated = true;
            if page.records.is_empty() {
                return Err(MemoryError::InvalidInput(
                    "history budget cannot fit one complete record; increase it",
                ));
            }
        }
        tx.commit()?;
        Ok(page)
    }

    pub fn search(&mut self, request: SearchRequest) -> Result<SearchResult> {
        self.search_ranked(request, None, RankingMode::Lexical)
    }

    pub fn search_hybrid(
        &mut self,
        request: SearchRequest,
        query: &crate::embedding::QueryEmbedding,
    ) -> Result<SearchResult> {
        self.search_ranked(request, Some(query), RankingMode::Hybrid)
    }

    pub fn search_semantic(
        &mut self,
        request: SearchRequest,
        query: &crate::embedding::QueryEmbedding,
    ) -> Result<SearchResult> {
        self.search_ranked(request, Some(query), RankingMode::Semantic)
    }

    fn search_ranked(
        &mut self,
        request: SearchRequest,
        embedding: Option<&crate::embedding::QueryEmbedding>,
        mode: RankingMode,
    ) -> Result<SearchResult> {
        validate_search_request(&request)?;
        if let Some(query) = embedding {
            validate_query_embedding(&request.query, query)?;
        }
        let query = lexical_query(&request.query)?;
        let at = request.context.at_time.unwrap_or(now()?) as i64;
        let tx = self.connection.transaction()?;
        let common = named_params! { ":at": at, ":task": request.context.task_key, ":checkout": request.context.checkout_id, ":operation": request.context.operation };
        let eligible = format!("is_current=1 AND {VALID_AT} AND {APPLIES}");
        let critical_where = format!("{eligible} AND status='current' AND critical=1");
        let critical_count: u64 = tx.query_row(
            &format!("SELECT count(*) FROM revisions WHERE {critical_where}"),
            common,
            |r| unsigned(r, 0),
        )?;
        let unclassified: u64 = tx.query_row(&format!("SELECT count(*) FROM revisions WHERE is_current=1 AND status='current' AND critical=1 AND {VALID_AT} AND {MAY_APPLY} AND {MISSING_CONTEXT}"), common, |r| unsigned(r, 0))?;
        let critical_ids = row_ids(
            &tx,
            &format!(
                "SELECT row_id FROM revisions WHERE {critical_where} ORDER BY record_id LIMIT {MAX_RESULTS}"
            ),
            common,
        )?;
        let lexical_where = format!("{eligible} AND (:status='both' OR status=:status)");
        let lexical_params = named_params! { ":at": at, ":task": request.context.task_key, ":checkout": request.context.checkout_id, ":operation": request.context.operation, ":status": request.status.as_str() };
        let eligible_heads: u64 = tx.query_row(
            &format!("SELECT count(*) FROM revisions WHERE {lexical_where}"),
            lexical_params,
            |r| unsigned(r, 0),
        )?;
        let (match_count, mut lexical_ids) = if query.is_empty() {
            (
                eligible_heads,
                row_ids(
                    &tx,
                    &format!(
                        "SELECT row_id FROM revisions WHERE {lexical_where} ORDER BY recorded_at DESC,record_id LIMIT {MAX_RESULTS}"
                    ),
                    lexical_params,
                )?,
            )
        } else {
            // Let FTS use its term index first, then apply canonical head,
            // status, time and context eligibility to each matched row.
            // Materializing every eligible revision before MATCH makes a
            // focused query cost proportional to the whole project store.
            let filtered = format!(
                "FROM record_fts JOIN revisions ON revisions.row_id=record_fts.rowid \
                 WHERE record_fts MATCH :query AND {lexical_where}"
            );
            let parameters = named_params! { ":at": at, ":task": request.context.task_key, ":checkout": request.context.checkout_id, ":operation": request.context.operation, ":status": request.status.as_str(), ":query": query };
            let count = tx.query_row(&format!("SELECT count(*) {filtered}"), parameters, |r| {
                unsigned(r, 0)
            })?;
            let ids = row_ids(
                &tx,
                &format!(
                    "SELECT revisions.row_id {filtered} ORDER BY bm25(record_fts),revisions.row_id LIMIT {MAX_RESULTS}"
                ),
                parameters,
            )?;
            (count, ids)
        };
        let literals = query_literals(&request.query);
        let (literal_matches, literal_ids) =
            literal_candidates(&tx, &lexical_where, lexical_params, &literals)?;
        let lexical_rank_count = if mode == RankingMode::Hybrid {
            // General word overlap is useful degraded recall, but the frozen
            // multilingual comparison showed that equal RRF turns incidental
            // function words into misleading full votes. Only supplied literal
            // identities enter this lane; semantic candidates remain evidence,
            // never proof that an absent literal exists.
            lexical_ids = literal_ids;
            literal_matches
        } else {
            match_count
        };
        let (indexed, semantic) = match embedding {
            Some(query) => semantic_candidates(&tx, &lexical_where, lexical_params, query)?,
            None => (0, Vec::new()),
        };
        let ranker_overflow = match mode {
            RankingMode::Lexical => match_count > lexical_ids.len() as u64,
            RankingMode::Semantic => indexed > semantic.len() as u64,
            RankingMode::Hybrid => {
                let union: HashSet<i64> = lexical_ids
                    .iter()
                    .copied()
                    .chain(semantic.iter().map(|(id, _)| *id))
                    .collect();
                union.len() > MAX_RESULTS
                    || lexical_rank_count > lexical_ids.len() as u64
                    || indexed > semantic.len() as u64
            }
        };
        let lexical_scores: Vec<(i64, f64)> = lexical_ids
            .iter()
            .enumerate()
            .map(|(rank, id)| (*id, 1.0 / (rank + 1) as f64))
            .collect();
        let ranked = match mode {
            RankingMode::Lexical => lexical_scores,
            RankingMode::Semantic => semantic,
            RankingMode::Hybrid => fuse_ranks(&lexical_scores, &semantic),
        };
        let ranked = if mode == RankingMode::Hybrid {
            apply_anchor_lane(&tx, &request.query, ranked)?
        } else {
            ranked
        };
        let ranking: Vec<RankedCandidate> = ranked
            .iter()
            .map(|(row_id, score)| {
                tx.query_row(
                    "SELECT record_id,revision FROM revisions WHERE row_id=?1",
                    [row_id],
                    |row| {
                        Ok(RankedCandidate {
                            reference: RecordRef {
                                record_id: row.get(0)?,
                                revision: unsigned(row, 1)?,
                            },
                            score: *score,
                        })
                    },
                )
                .map_err(MemoryError::from)
            })
            .collect::<Result<_>>()?;
        let ranked_ids: Vec<i64> = ranked.iter().map(|(id, _)| *id).collect();
        let mut result = SearchResult {
            scope: self.scope.clone(),
            hits: Vec::new(),
            ranking,
            coverage: Coverage {
                lexical_only: mode == RankingMode::Lexical,
                semantic_available: embedding.is_some(),
                semantic_indexed_eligible: indexed,
                semantic_warning: None,
                semantic_missing_eligible: if embedding.is_some() {
                    eligible_heads.saturating_sub(indexed)
                } else {
                    eligible_heads
                },
                eligible_heads,
                lexical_matches: match_count,
                literal_terms: literals.len(),
                literal_matches,
                applicable_current_critical: critical_count,
                unclassified_current_critical: unclassified,
                returned_critical: 0,
                critical_overflow: false,
                truncated: false,
            },
        };
        let critical_set: HashSet<i64> = critical_ids.iter().copied().collect();
        let mut selected_rows = Vec::new();
        let mut seen = HashSet::new();
        for row_id in critical_ids.into_iter().chain(ranked_ids.iter().copied()) {
            if !seen.insert(row_id) {
                continue;
            }
            let reason = if critical_set.contains(&row_id) {
                MatchReason::CriticalConstraint
            } else {
                match mode {
                    RankingMode::Lexical => MatchReason::Lexical,
                    RankingMode::Semantic => MatchReason::Semantic,
                    RankingMode::Hybrid => MatchReason::Hybrid,
                }
            };
            if result.hits.len() < request.limit {
                result.hits.push(SearchHit {
                    record: read_by_row(&tx, row_id)?,
                    reason,
                });
                selected_rows.push(row_id);
            }
        }
        loop {
            result.coverage.returned_critical = result
                .hits
                .iter()
                .filter(|hit| matches!(hit.reason, MatchReason::CriticalConstraint))
                .count();
            result.coverage.critical_overflow =
                critical_count > result.coverage.returned_critical as u64;
            result.coverage.truncated = ranker_overflow
                || result.coverage.critical_overflow
                || ranked_ids.iter().any(|id| !selected_rows.contains(id));
            if serde_json::to_vec(&result)?.len() <= request.max_bytes {
                break;
            }
            if result.hits.pop().is_none() {
                return Err(MemoryError::InvalidInput(
                    "search budget cannot fit its coverage metadata",
                ));
            }
            selected_rows.pop();
        }
        tx.commit()?;
        Ok(result)
    }
}

fn validate_query_embedding(text: &str, query: &crate::embedding::QueryEmbedding) -> Result<()> {
    let expected = crate::embedding::expected_model_identity()
        .map_err(|_| MemoryError::InvalidInput("compiled embedding identity is invalid"))?;
    let norm: f64 = query.vector.iter().map(|v| f64::from(*v).powi(2)).sum();
    if query.model_identity != expected
        || query.input_bytes != text.len()
        || query.source_sha256 != crate::embedding::source_digest(text)
        || query.input_tokens == 0
        || query.input_tokens > crate::embedding::MAX_INPUT_TOKENS
        || query.vector.len() != crate::embedding::DIMENSIONS
        || query.vector.iter().any(|v| !v.is_finite())
        || (norm - 1.0).abs() > 0.002
    {
        return Err(MemoryError::InvalidInput(
            "semantic query is not a complete normalized embedding of this exact query",
        ));
    }
    Ok(())
}

fn semantic_candidates(
    tx: &Transaction<'_>,
    eligible: &str,
    parameters: &[(&str, &dyn rusqlite::ToSql)],
    query: &crate::embedding::QueryEmbedding,
) -> Result<(u64, Vec<(i64, f64)>)> {
    // Eligibility is canonical SQL before any vector is admitted. Stream one
    // record at a time; only the best twenty candidates stay in memory.
    let mut statement = tx.prepare(&format!(
        "SELECT row_id,content FROM revisions WHERE {eligible} ORDER BY row_id"
    ))?;
    let mut rows = statement.query(parameters)?;
    let mut reader = index::ChunkReader::new(tx, &query.model_identity)?;
    let mut indexed = 0;
    let mut best: Vec<(i64, f64)> = Vec::new();
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let content: String = row.get(1)?;
        let Some(chunks) = reader.read(id, &content)? else {
            continue;
        };
        let mut score = f64::NEG_INFINITY;
        for chunk in chunks {
            score = score.max(cosine_similarity(&chunk.vector, &query.vector)?);
        }
        indexed += 1;
        best.push((id, score));
        best.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        best.truncate(MAX_RESULTS);
    }
    Ok((indexed, best))
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f64> {
    if left.len() != crate::embedding::DIMENSIONS || right.len() != left.len() {
        return Err(MemoryError::InvalidInput(
            "semantic vector has invalid dimensions",
        ));
    }
    let (mut dot, mut left_norm, mut right_norm) = (0.0_f64, 0.0_f64, 0.0_f64);
    for (&a, &b) in left.iter().zip(right) {
        if !a.is_finite() || !b.is_finite() {
            return Err(MemoryError::InvalidInput("semantic vector is not finite"));
        }
        let (a, b) = (f64::from(a), f64::from(b));
        dot += a * b;
        left_norm += a * a;
        right_norm += b * b;
    }
    if left_norm == 0.0 || right_norm == 0.0 {
        return Err(MemoryError::InvalidInput("semantic vector has zero norm"));
    }
    let similarity = dot / (left_norm.sqrt() * right_norm.sqrt());
    if !similarity.is_finite() {
        return Err(MemoryError::InvalidInput(
            "semantic similarity is not finite",
        ));
    }
    Ok(similarity.clamp(-1.0, 1.0))
}

fn fuse_ranks(lexical: &[(i64, f64)], semantic: &[(i64, f64)]) -> Vec<(i64, f64)> {
    let mut fused = std::collections::BTreeMap::<i64, f64>::new();
    for list in [lexical, semantic] {
        for (rank, (id, _)) in list.iter().enumerate() {
            *fused.entry(*id).or_default() += 1.0 / (60.0 + (rank + 1) as f64);
        }
    }
    let mut ranking: Vec<_> = fused.into_iter().collect();
    ranking.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    ranking.truncate(MAX_RESULTS);
    ranking
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnchorIntent {
    Function,
    Identifier,
    Path,
}

/// Rank explicit artifact-shaped evidence only when the query asks for an
/// artifact identity. This is deliberately a bounded tie-break lane: ordinary
/// natural-language queries continue to use the lexical/semantic fusion alone.
fn apply_anchor_lane(
    tx: &Transaction<'_>,
    query: &str,
    ranked: Vec<(i64, f64)>,
) -> Result<Vec<(i64, f64)>> {
    let Some(intent) = anchor_intent(query) else {
        return Ok(ranked);
    };
    let mut adjusted = Vec::with_capacity(ranked.len());
    for (row_id, score) in ranked {
        let content: String = tx.query_row(
            "SELECT content FROM revisions WHERE row_id=?1",
            [row_id],
            |row| row.get(0),
        )?;
        let evidence = anchor_evidence_for_query(&content, intent, query);
        adjusted.push((row_id, score + f64::from(evidence) * 0.01));
    }
    adjusted.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(adjusted)
}

fn anchor_intent(query: &str) -> Option<AnchorIntent> {
    let query = query.to_lowercase();
    // Function/API wording is more specific than the generic path wording.
    if [
        "function",
        "callable",
        "method",
        "api",
        "fonksiyon",
        "metot",
    ]
    .iter()
    .any(|term| query.contains(term))
    {
        return Some(AnchorIntent::Function);
    }
    if [
        "identifier",
        "event name",
        "version suffix",
        "tanımlayıcı",
        "olay",
        "olay adı",
        "tam ad",
    ]
    .iter()
    .any(|term| query.contains(term))
    {
        return Some(AnchorIntent::Identifier);
    }
    if [
        "path",
        "source",
        "filename",
        "stylesheet",
        "template",
        "json",
        "csv",
        "yolu",
        "şablon",
        "kaynak",
    ]
    .iter()
    .any(|term| query.contains(term))
    {
        return Some(AnchorIntent::Path);
    }
    None
}

fn anchor_evidence_for_query(content: &str, intent: AnchorIntent, query: &str) -> u8 {
    let query = query.to_lowercase();
    let requested_extension = ["json", "csv", "css", "docx", "ndjson"]
        .into_iter()
        .find(|extension| query.contains(extension))
        .or_else(|| (query.contains("template") || query.contains("şablon")).then_some("docx"));
    let wants_diacritics =
        query.contains("diacritic") || query.contains("turkish") || query.contains("türkçe");
    let mut evidence = 0u8;
    for token in content.split_whitespace() {
        let token = token.trim_matches(|c: char| "\"'`()[]{}<>,;!?".contains(c));
        if token.is_empty() {
            continue;
        }
        let path = token.contains('/') || token.contains('\\') || has_extension(token);
        let path_extension = token.rsplit_once('.').map(|(_, extension)| extension);
        let extension_matches = requested_extension.is_none_or(|extension| {
            path_extension.is_some_and(|candidate| candidate.eq_ignore_ascii_case(extension))
        });
        let has_diacritic = token
            .chars()
            .any(|character| character.is_alphabetic() && !character.is_ascii());
        let function = token.contains('(')
            && token
                .split_once('(')
                .is_some_and(|(name, _)| code_identifier(name));
        let identifier = code_identifier(token);
        let versioned = token
            .chars()
            .collect::<Vec<_>>()
            .windows(3)
            .any(|window| window[0] == '_' && window[1] == 'v' && window[2].is_ascii_digit());
        let contribution = match intent {
            AnchorIntent::Function if function => 3,
            AnchorIntent::Function if identifier => 1,
            AnchorIntent::Identifier if versioned => 4,
            AnchorIntent::Identifier if identifier => 2,
            AnchorIntent::Path if path && extension_matches && wants_diacritics => {
                if has_diacritic { 6 } else { 0 }
            }
            AnchorIntent::Path if path && extension_matches => 3,
            _ => 0,
        };
        evidence = evidence.saturating_add(contribution);
    }
    evidence.min(8)
}

fn code_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let punctuation = value.contains('_') || value.contains('$');
    let camel = value
        .chars()
        .zip(value.chars().skip(1))
        .any(|(a, b)| a.is_lowercase() && b.is_uppercase());
    (first.is_alphabetic() || first == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        && (punctuation || camel)
}

fn has_extension(value: &str) -> bool {
    let Some((_, extension)) = value.rsplit_once('.') else {
        return false;
    };
    !extension.is_empty()
        && extension.len() <= 12
        && extension.chars().all(|c| c.is_ascii_alphanumeric())
        && value
            .split_once('.')
            .is_some_and(|(stem, _)| stem.chars().any(|c| c.is_alphanumeric()))
}

/// Conservative surface recognition, not a language detector or a synonym map.
/// Keep bytes/case/diacritics intact. Paired backticks can identify a literal
/// containing spaces; unquoted code/path atoms use recognizable punctuation or
/// an internal lower-to-upper transition. Ordinary prose never becomes a vote.
fn query_literals(query: &str) -> Vec<String> {
    fn trim(value: &str) -> &str {
        value
            .trim_matches(|c: char| c.is_whitespace() || "\"'`()[]{}<>,;!?".contains(c))
            .trim_end_matches('.')
    }
    fn code_like(value: &str) -> bool {
        value.contains(['_', '/', '\\', '.', ':'])
            || value
                .chars()
                .zip(value.chars().skip(1))
                .any(|(a, b)| a.is_lowercase() && b.is_uppercase())
    }
    let parts: Vec<_> = query.split('`').collect();
    let mut literals = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        if index % 2 == 1 && index + 1 < parts.len() {
            let value = part.trim();
            if !value.is_empty() && value.len() <= 256 {
                literals.push(value.to_owned());
            }
        } else {
            for word in part.split_whitespace() {
                let value = trim(word);
                if !value.is_empty()
                    && value.len() <= 256
                    && code_like(value)
                    && value
                        .chars()
                        .all(|c| c.is_alphanumeric() || "_-./\\:$".contains(c))
                {
                    literals.push(value.to_owned());
                }
            }
        }
    }
    literals.sort();
    literals.dedup();
    literals
}

fn contains_literal(content: &str, literal: &str) -> bool {
    fn joined(c: char) -> bool {
        c.is_alphanumeric() || "_/\\:-$".contains(c)
    }
    content.match_indices(literal).any(|(offset, _)| {
        let before = content[..offset].chars().next_back();
        let after = &content[offset + literal.len()..];
        let mut suffix = after.chars();
        let next = suffix.next();
        !before.is_some_and(|c| joined(c) || c == '.')
            && !next.is_some_and(|c| joined(c) || (c == '.' && suffix.next().is_some_and(joined)))
    })
}

fn literal_candidates(
    tx: &Transaction<'_>,
    eligible: &str,
    parameters: &[(&str, &dyn rusqlite::ToSql)],
    literals: &[String],
) -> Result<(u64, Vec<i64>)> {
    if literals.is_empty() {
        return Ok((0, Vec::new()));
    }
    let mut statement = tx.prepare(&format!(
        "SELECT row_id,content FROM revisions WHERE {eligible} ORDER BY row_id"
    ))?;
    let mut rows = statement.query(parameters)?;
    let mut count = 0;
    let mut ids = Vec::new();
    while let Some(row) = rows.next()? {
        let content: String = row.get(1)?;
        if literals
            .iter()
            .all(|literal| contains_literal(&content, literal))
        {
            count += 1;
            if ids.len() < MAX_RESULTS {
                ids.push(row.get(0)?);
            }
        }
    }
    Ok((count, ids))
}

const VALID_AT: &str =
    "(valid_from IS NULL OR valid_from<=:at) AND (valid_until IS NULL OR valid_until>:at)";
const APPLIES: &str = "(task_key IS NULL OR task_key=:task) AND (checkout_id IS NULL OR checkout_id=:checkout) AND (operation IS NULL OR operation=:operation)";
const MAY_APPLY: &str = "(task_key IS NULL OR :task IS NULL OR task_key=:task) AND (checkout_id IS NULL OR :checkout IS NULL OR checkout_id=:checkout) AND (operation IS NULL OR :operation IS NULL OR operation=:operation)";
const MISSING_CONTEXT: &str = "((task_key IS NOT NULL AND :task IS NULL) OR (checkout_id IS NOT NULL AND :checkout IS NULL) OR (operation IS NOT NULL AND :operation IS NULL))";

fn initialize(connection: &mut Connection, scope_json: &str, create: bool) -> Result<()> {
    let tx = connection.transaction_with_behavior(if create {
        TransactionBehavior::Immediate
    } else {
        TransactionBehavior::Deferred
    })?;
    let version: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version == 0 {
        if !create {
            return Err(MemoryError::UnsupportedSchema);
        }
        let entries: i64 = tx.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get(0),
        )?;
        if entries != 0 {
            return Err(MemoryError::UnsupportedSchema);
        }
        tx.execute_batch("CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL) STRICT;
            CREATE TABLE mutations(operation_id TEXT PRIMARY KEY,intent_hash TEXT NOT NULL,record_id TEXT NOT NULL,revision INTEGER NOT NULL CHECK(revision>0)) STRICT;
            CREATE TABLE records(record_id TEXT PRIMARY KEY,head_revision INTEGER NOT NULL CHECK(head_revision>0)) STRICT;
            CREATE TABLE revisions(row_id INTEGER PRIMARY KEY,record_id TEXT NOT NULL REFERENCES records(record_id),revision INTEGER NOT NULL CHECK(revision>0),
                kind TEXT NOT NULL CHECK(kind IN ('fact','decision','preference','task_state')),basis TEXT NOT NULL CHECK(basis IN ('observed','user_stated','inferred')),
                content TEXT NOT NULL,status TEXT NOT NULL CHECK(status IN ('current','unresolved')),critical INTEGER NOT NULL CHECK(critical IN (0,1)),
                task_key TEXT,checkout_id TEXT,operation TEXT,valid_from INTEGER,valid_until INTEGER,recorded_at INTEGER NOT NULL,
                supersedes_record_id TEXT,supersedes_revision INTEGER,is_current INTEGER NOT NULL CHECK(is_current IN (0,1)),UNIQUE(record_id,revision)) STRICT;
            CREATE UNIQUE INDEX one_current_revision ON revisions(record_id) WHERE is_current=1;
            CREATE INDEX current_constraints ON revisions(is_current,status,critical);
            CREATE TABLE sources(revision_row_id INTEGER NOT NULL REFERENCES revisions(row_id),ordinal INTEGER NOT NULL,
                source_id TEXT NOT NULL,source_revision TEXT NOT NULL,locator TEXT NOT NULL,captured_at INTEGER NOT NULL,
                PRIMARY KEY(revision_row_id,ordinal)) STRICT;
            CREATE VIRTUAL TABLE record_fts USING fts5(content,tokenize='unicode61');")?;
        index::initialize(&tx)?;
        tx.execute(
            "INSERT INTO metadata(key,value) VALUES('scope',?1)",
            [scope_json],
        )?;
        tx.pragma_update(None, "application_id", APPLICATION_ID)?;
        tx.pragma_update(None, "user_version", 1)?;
    } else if version != 1 {
        return Err(MemoryError::UnsupportedSchema);
    }
    let application: i64 = tx.query_row("PRAGMA application_id", [], |r| r.get(0))?;
    if application != APPLICATION_ID {
        return Err(MemoryError::UnsupportedSchema);
    }
    let stored: String = tx.query_row("SELECT value FROM metadata WHERE key='scope'", [], |r| {
        r.get(0)
    })?;
    if stored != scope_json {
        return Err(MemoryError::ScopeMismatch);
    }
    tx.commit()?;
    Ok(())
}

pub(crate) fn validate_lifecycle_snapshot(
    connection: &mut Connection,
    scope: &Scope,
) -> Result<()> {
    verify_runtime(connection)?;
    initialize(connection, &serde_json::to_string(scope)?, false)
}

fn verify_runtime(connection: &Connection) -> Result<()> {
    let (version, source, fts, omitted): (String, String, i64, i64) = connection.query_row(
        "SELECT sqlite_version(),sqlite_source_id(),sqlite_compileoption_used('ENABLE_FTS5'),sqlite_compileoption_used('OMIT_LOAD_EXTENSION')", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
    if version != SQLITE_VERSION || source != SQLITE_SOURCE_ID || fts != 1 || omitted != 1 {
        return Err(MemoryError::UnsupportedSqlite);
    }
    Ok(())
}

fn validate_operation_id(id: &str) -> Result<()> {
    if id.len() < 4
        || id.len() > 128
        || !id.starts_with("op-")
        || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(MemoryError::InvalidInput(
            "memory requires a bounded server-issued op- identity",
        ));
    }
    Ok(())
}
fn intent_hash(value: &impl Serialize) -> Result<String> {
    Ok(Sha256::digest(serde_json::to_vec(value)?)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
fn mutation_receipt(tx: &Transaction<'_>, operation_id: &str) -> Result<Option<MutationReceipt>> {
    Ok(tx.query_row("SELECT intent_hash,record_id,revision,EXISTS(SELECT 1 FROM revisions r WHERE r.record_id=m.record_id AND r.revision=m.revision) FROM mutations m WHERE operation_id=?1", [operation_id], |row| {
        Ok(MutationReceipt { operation_id: operation_id.into(), intent_hash: row.get(0)?, record_id: row.get(1)?, revision: unsigned(row, 2)?, available: row.get(3)? })
    }).optional()?)
}
fn replay_mutation(
    tx: &Transaction<'_>,
    operation_id: &str,
    intent: &str,
) -> Result<Option<Record>> {
    let Some(receipt) = mutation_receipt(tx, operation_id)? else {
        return Ok(None);
    };
    if !receipt.available {
        return Err(MemoryError::MutationUnavailable);
    }
    if receipt.intent_hash != intent {
        return Err(MemoryError::OperationConflict);
    }
    Ok(Some(read_record(tx, &receipt.record_id, receipt.revision)?))
}
fn insert_mutation(
    tx: &Transaction<'_>,
    operation_id: &str,
    intent: &str,
    record_id: &str,
    revision: u64,
) -> Result<()> {
    tx.execute(
        "INSERT INTO mutations(operation_id,intent_hash,record_id,revision) VALUES(?1,?2,?3,?4)",
        params![operation_id, intent, record_id, revision as i64],
    )?;
    Ok(())
}

fn retire(tx: &Transaction<'_>, reference: &RecordRef) -> Result<()> {
    let row_id: Option<i64> = tx
        .query_row(
            "SELECT row_id FROM revisions WHERE record_id=?1 AND revision=?2 AND is_current=1",
            params![reference.record_id, reference.revision as i64],
            |r| r.get(0),
        )
        .optional()?;
    let row_id = row_id.ok_or(MemoryError::Conflict)?;
    let changed = tx.execute(
        "UPDATE revisions SET is_current=0 WHERE row_id=?1 AND is_current=1",
        [row_id],
    )?;
    if changed != 1 {
        return Err(MemoryError::Conflict);
    }
    tx.execute("DELETE FROM record_fts WHERE rowid=?1", [row_id])?;
    Ok(())
}

fn insert_revision(
    tx: &Transaction<'_>,
    id: &str,
    revision: u64,
    data: &RecordInput,
    recorded: u64,
    supersedes: Option<&RecordRef>,
) -> Result<()> {
    tx.execute("INSERT INTO revisions(record_id,revision,kind,basis,content,status,critical,task_key,checkout_id,operation,valid_from,valid_until,recorded_at,supersedes_record_id,supersedes_revision,is_current)
        VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,1)",
        params![id,revision as i64,data.kind.as_str(),data.basis.as_str(),data.content,data.status.as_str(),data.critical,
            data.applicability.task_key,data.applicability.checkout_id,data.applicability.operation,data.valid_from.map(|v| v as i64),data.valid_until.map(|v| v as i64),recorded as i64,
            supersedes.map(|r| r.record_id.as_str()),supersedes.map(|r| r.revision as i64)])?;
    let row_id = tx.last_insert_rowid();
    for (ordinal, source) in data.sources.iter().enumerate() {
        tx.execute("INSERT INTO sources(revision_row_id,ordinal,source_id,source_revision,locator,captured_at) VALUES(?1,?2,?3,?4,?5,?6)", params![row_id,ordinal as i64,source.source_id,source.revision,source.locator,source.captured_at as i64])?;
    }
    tx.execute(
        "INSERT INTO record_fts(rowid,content) VALUES(?1,?2)",
        params![row_id, data.content],
    )?;
    Ok(())
}

fn read_record(tx: &Transaction<'_>, id: &str, revision: u64) -> Result<Record> {
    let row_id = tx
        .query_row(
            "SELECT row_id FROM revisions WHERE record_id=?1 AND revision=?2",
            params![id, revision as i64],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(MemoryError::NotFound)?;
    read_by_row(tx, row_id)
}

fn read_by_row(tx: &Transaction<'_>, row_id: i64) -> Result<Record> {
    let body: String = tx.query_row("SELECT json_object('record_id',record_id,'revision',revision,'kind',kind,'basis',basis,'content',content,'status',status,'critical',critical,
        'task_key',task_key,'checkout_id',checkout_id,'operation',operation,'valid_from',valid_from,'valid_until',valid_until,'recorded_at',recorded_at,
        'supersedes_record_id',supersedes_record_id,'supersedes_revision',supersedes_revision,'is_current',is_current) FROM revisions WHERE row_id=?1", [row_id], |r| r.get(0))?;
    #[derive(Deserialize)]
    struct Stored {
        record_id: String,
        revision: u64,
        kind: RecordKind,
        basis: ClaimBasis,
        content: String,
        status: RecordStatus,
        critical: u8,
        task_key: Option<String>,
        checkout_id: Option<String>,
        operation: Option<String>,
        valid_from: Option<u64>,
        valid_until: Option<u64>,
        recorded_at: u64,
        supersedes_record_id: Option<String>,
        supersedes_revision: Option<u64>,
        is_current: u8,
    }
    let stored: Stored = serde_json::from_str(&body)?;
    let sources: Vec<Source> = {
        let mut statement = tx.prepare("SELECT source_id,source_revision,locator,captured_at FROM sources WHERE revision_row_id=?1 ORDER BY ordinal")?;
        statement
            .query_map([row_id], |r| {
                Ok(Source {
                    source_id: r.get(0)?,
                    revision: r.get(1)?,
                    locator: r.get(2)?,
                    captured_at: unsigned(r, 3)?,
                })
            })?
            .collect::<std::result::Result<_, _>>()?
    };
    let superseded_by = tx.query_row("SELECT record_id,revision FROM revisions WHERE supersedes_record_id=?1 AND supersedes_revision=?2 ORDER BY row_id LIMIT 1", params![stored.record_id,stored.revision as i64], |r| Ok(RecordRef { record_id:r.get(0)?,revision:unsigned(r, 1)? })).optional()?;
    Ok(Record {
        record_id: stored.record_id,
        revision: stored.revision,
        recorded_at: stored.recorded_at,
        is_current_revision: stored.is_current == 1,
        supersedes: stored
            .supersedes_record_id
            .zip(stored.supersedes_revision)
            .map(|(record_id, revision)| RecordRef {
                record_id,
                revision,
            }),
        superseded_by,
        data: RecordInput {
            kind: stored.kind,
            basis: stored.basis,
            content: stored.content,
            sources,
            status: stored.status,
            critical: stored.critical == 1,
            applicability: Applicability {
                task_key: stored.task_key,
                checkout_id: stored.checkout_id,
                operation: stored.operation,
            },
            valid_from: stored.valid_from,
            valid_until: stored.valid_until,
        },
    })
}

fn row_ids(
    tx: &Transaction<'_>,
    sql: &str,
    parameters: &[(&str, &dyn rusqlite::ToSql)],
) -> Result<Vec<i64>> {
    let mut statement = tx.prepare(sql)?;
    Ok(statement
        .query_map(parameters, |row| row.get(0))?
        .collect::<std::result::Result<_, _>>()?)
}

fn now() -> Result<u64> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| MemoryError::Clock)?
        .as_secs();
    valid_time(seconds)?;
    Ok(seconds)
}
fn valid_time(time: u64) -> Result<()> {
    if time > i64::MAX as u64 {
        Err(MemoryError::InvalidInput(
            "timestamp exceeds SQLite integer range",
        ))
    } else {
        Ok(())
    }
}
fn bounded(value: &str, max: usize, empty: bool) -> Result<()> {
    if (!empty && value.trim().is_empty()) || value.len() > max || value.contains('\0') {
        return Err(MemoryError::InvalidInput(
            "memory field is empty, oversized or contains NUL",
        ));
    }
    Ok(())
}
fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
    {
        return Err(MemoryError::InvalidInput("record identity is invalid"));
    }
    Ok(())
}
fn valid_revision(revision: u64) -> Result<()> {
    if revision == 0 || revision > i64::MAX as u64 {
        Err(MemoryError::InvalidInput("record revision is invalid"))
    } else {
        Ok(())
    }
}
pub fn validate_reference(reference: &RecordRef) -> Result<()> {
    validate_id(&reference.record_id)?;
    valid_revision(reference.revision)
}
pub(crate) fn validate_scope(scope: &Scope) -> Result<()> {
    if let Scope::Project(id) = scope {
        validate_id(id)?;
    }
    Ok(())
}
fn validate_selectors(
    task: &Option<String>,
    checkout: &Option<String>,
    operation: &Option<String>,
) -> Result<()> {
    for value in [task, checkout, operation].into_iter().flatten() {
        bounded(value, 256, false)?;
    }
    Ok(())
}
fn validate_context(context: &SearchContext) -> Result<()> {
    validate_selectors(&context.task_key, &context.checkout_id, &context.operation)?;
    if let Some(t) = context.at_time {
        valid_time(t)?;
    }
    Ok(())
}
pub fn validate_input(scope: &Scope, data: &RecordInput) -> Result<()> {
    if matches!(scope, Scope::SharedPreferences) && data.kind != RecordKind::Preference {
        return Err(MemoryError::InvalidInput(
            "shared memory accepts only explicit shared preferences",
        ));
    }
    bounded(&data.content, MAX_CONTENT_BYTES, false)?;
    if data.sources.is_empty() || data.sources.len() > 8 {
        return Err(MemoryError::InvalidInput(
            "memory requires between one and eight explicit provenance sources",
        ));
    }
    for source in &data.sources {
        bounded(&source.source_id, 512, false)?;
        bounded(&source.revision, 256, false)?;
        bounded(&source.locator, 512, false)?;
        valid_time(source.captured_at)?;
    }
    validate_selectors(
        &data.applicability.task_key,
        &data.applicability.checkout_id,
        &data.applicability.operation,
    )?;
    if let Some(t) = data.valid_from {
        valid_time(t)?;
    }
    if let Some(t) = data.valid_until {
        valid_time(t)?;
    }
    if matches!((data.valid_from,data.valid_until),(Some(from),Some(until)) if from>=until) {
        return Err(MemoryError::InvalidInput(
            "memory validity interval must have a later exclusive end",
        ));
    }
    Ok(())
}
/// Validate recall bounds and selectors without opening canonical state or
/// starting inference. Empty queries remain valid for applicable-head recall.
pub fn validate_search_request(request: &SearchRequest) -> Result<()> {
    validate_budget(request.limit, request.max_bytes)?;
    validate_context(&request.context)?;
    lexical_query(&request.query)?;
    if query_literals(&request.query).len() > 32 {
        return Err(MemoryError::InvalidInput(
            "query exceeds 32 literal identities",
        ));
    }
    Ok(())
}

fn validate_budget(limit: usize, max_bytes: usize) -> Result<()> {
    if limit == 0 || limit > MAX_RESULTS || !(1024..=MAX_RESPONSE_BYTES).contains(&max_bytes) {
        return Err(MemoryError::InvalidInput(
            "memory recall requires 1–20 records and a 1–32 KiB response budget",
        ));
    }
    Ok(())
}
fn lexical_query(query: &str) -> Result<String> {
    bounded(query, 2048, true)?;
    let terms: Vec<_> = query.split_whitespace().collect();
    if terms.len() > 32 {
        return Err(MemoryError::InvalidInput(
            "lexical query exceeds 32 literal terms",
        ));
    }
    Ok(terms
        .into_iter()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" OR "))
}
fn existing_directory(path: &Path) -> Result<()> {
    match private_directory(path, false) {
        Err(MemoryError::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
            Err(MemoryError::NotFound)
        }
        result => result,
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
        return Err(MemoryError::InvalidInput(
            "memory directory must be a real private directory",
        ));
    }
    private_permissions(&m)
}
fn private_file(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    if !m.is_file() || m.file_type().is_symlink() {
        return Err(MemoryError::InvalidInput(
            "memory database and sidecars must be regular private files",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.nlink() != 1 {
            return Err(MemoryError::InvalidInput(
                "memory database aliases are unsupported",
            ));
        }
    }
    private_permissions(&m)
}
fn private_permissions(metadata: &fs::Metadata) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(MemoryError::InvalidInput(
                "memory files and directories must deny group and other access",
            ));
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    Ok(())
}

fn unsigned(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "mirket-memory-test-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            private_directory(&path, true).unwrap();
            Self(path.canonicalize().unwrap())
        }
        fn open(&self, id: &str) -> MemoryStore {
            MemoryStore::open(&self.0, Scope::Project(id.into())).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn operation() -> String {
        format!("op-test-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    fn input(content: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::Observed,
            content: content.into(),
            sources: vec![Source {
                source_id: "fixture:source".into(),
                revision: "sha256:explicit".into(),
                locator: "line:1".into(),
                captured_at: 123,
            }],
            status: RecordStatus::Current,
            critical: false,
            applicability: Applicability::default(),
            valid_from: None,
            valid_until: None,
        }
    }
    fn search(store: &mut MemoryStore, query: &str) -> SearchResult {
        store
            .search(SearchRequest {
                query: query.into(),
                ..SearchRequest::default()
            })
            .unwrap()
    }
    fn reference(record: &Record) -> RecordRef {
        RecordRef {
            record_id: record.record_id.clone(),
            revision: record.revision,
        }
    }

    fn synthetic_vector(axis: usize) -> Vec<f32> {
        let mut vector = vec![0.0; crate::embedding::DIMENSIONS];
        vector[axis] = 1.0;
        vector
    }
    fn index_record(store: &mut MemoryStore, record: &Record, axis: usize) {
        let text = &record.data.content;
        store
            .apply_index(
                &reference(record),
                &index::source_digest(text),
                &crate::embedding::PassageEmbedding {
                    model_identity: crate::embedding::expected_model_identity().unwrap(),
                    input_bytes: text.len(),
                    content_tokens: 2,
                    covered_bytes: text.len(),
                    complete: true,
                    chunks: vec![crate::embedding::ChunkEmbedding {
                        start_byte: 0,
                        end_byte: text.len(),
                        input_tokens: 5,
                        vector: synthetic_vector(axis),
                    }],
                },
            )
            .unwrap();
    }
    fn query_embedding(text: &str) -> crate::embedding::QueryEmbedding {
        crate::embedding::QueryEmbedding {
            model_identity: crate::embedding::expected_model_identity().unwrap(),
            source_sha256: crate::embedding::source_digest(text),
            input_bytes: text.len(),
            input_tokens: 5,
            vector: synthetic_vector(0),
        }
    }
    #[test]
    fn cosine_scores_normalized_vectors_without_native_extension() {
        let x = synthetic_vector(0);
        let y = synthetic_vector(1);
        let mut opposite = x.clone();
        opposite[0] = -1.0;
        let mut diagonal = vec![0.0; crate::embedding::DIMENSIONS];
        diagonal[0] = std::f32::consts::FRAC_1_SQRT_2;
        diagonal[1] = std::f32::consts::FRAC_1_SQRT_2;
        assert!((cosine_similarity(&x, &x).unwrap() - 1.0).abs() < 1e-12);
        assert!(cosine_similarity(&x, &y).unwrap().abs() < 1e-12);
        assert!((cosine_similarity(&x, &opposite).unwrap() + 1.0).abs() < 1e-12);
        assert!(
            (cosine_similarity(&x, &diagonal).unwrap() - std::f64::consts::FRAC_1_SQRT_2).abs()
                < 1e-7
        );
        assert!(cosine_similarity(&x, &[]).is_err());
        assert!(cosine_similarity(&x, &vec![0.0; x.len()]).is_err());
        let mut corrupt = y;
        corrupt[0] = f32::NAN;
        assert!(cosine_similarity(&x, &corrupt).is_err());
    }
    #[test]
    fn semantic_ranking_filters_canonical_scope_head_validity_and_context_first() {
        let fixture = Fixture::new();
        let mut store = fixture.open("semantic-a");
        let valid = store
            .create(&operation(), input("eligible different words"), None)
            .unwrap();
        index_record(&mut store, &valid, 1);
        for mut data in [
            {
                let mut d = input("wrong checkout");
                d.applicability.checkout_id = Some("other".into());
                d
            },
            {
                let mut d = input("expired evidence");
                d.valid_until = Some(50);
                d
            },
            {
                let mut d = input("unresolved evidence");
                d.status = RecordStatus::Unresolved;
                d
            },
        ] {
            data.critical = false;
            let record = store.create(&operation(), data, None).unwrap();
            index_record(&mut store, &record, 0);
        }
        let old = store
            .create(&operation(), input("old evidence"), None)
            .unwrap();
        index_record(&mut store, &old, 0);
        store
            .revise(
                &operation(),
                &old.record_id,
                1,
                input("new unindexed evidence"),
            )
            .unwrap();
        let mut other = fixture.open("semantic-b");
        let record = other
            .create(&operation(), input("other project"), None)
            .unwrap();
        index_record(&mut other, &record, 0);
        let request = SearchRequest {
            query: "needle".into(),
            status: StatusFilter::Current,
            context: SearchContext {
                at_time: Some(100),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = store
            .search_semantic(request, &query_embedding("needle"))
            .unwrap();
        assert_eq!(result.ranking.len(), 1);
        assert_eq!(result.ranking[0].reference, reference(&valid));
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].record.record_id, valid.record_id);
        assert_eq!(result.coverage.semantic_indexed_eligible, 1);
        assert_eq!(result.coverage.semantic_missing_eligible, 1);
    }
    #[test]
    fn hybrid_query_binding_corrupt_cache_and_bounded_diagnostics_are_explicit() {
        let fixture = Fixture::new();
        let mut store = fixture.open("hybrid");
        let record = store
            .create(&operation(), input("needle_id evidence"), None)
            .unwrap();
        index_record(&mut store, &record, 0);
        let request = SearchRequest {
            query: "needle_id".into(),
            ..Default::default()
        };
        let result = store
            .search_hybrid(request.clone(), &query_embedding("needle_id"))
            .unwrap();
        assert_eq!(result.ranking[0].reference, reference(&record));
        assert_eq!(result.ranking[0].score, 2.0 / 61.0);
        let wire = serde_json::to_value(&result).unwrap();
        assert!(wire.get("ranking").is_none());
        assert!(
            store
                .search_hybrid(request.clone(), &query_embedding("noodle"))
                .is_err()
        );
        let mut wrong = query_embedding("needle_id");
        wrong.model_identity.id = "wrong".into();
        assert!(store.search_hybrid(request.clone(), &wrong).is_err());
        store
            .connection
            .execute("UPDATE semantic_chunks SET vector=zeroblob(1536)", [])
            .unwrap();
        let result = store
            .search_hybrid(request, &query_embedding("needle_id"))
            .unwrap();
        assert_eq!(result.coverage.semantic_missing_eligible, 1);
        assert_eq!(result.coverage.semantic_indexed_eligible, 0);
        assert_eq!(result.hits[0].record.record_id, record.record_id);
    }
    #[test]
    fn reciprocal_rank_fusion_deduplicates_and_breaks_ties_stably() {
        let ranked = fuse_ranks(&[(2, 0.9), (1, 0.8)], &[(1, 0.9), (3, 0.8)]);
        assert_eq!(
            ranked.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            fuse_ranks(&[(2, 1.0)], &[(1, 1.0)])
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn anchor_intent_is_limited_to_explicit_artifact_requests() {
        assert_eq!(
            anchor_intent("Which callable removes a frame?"),
            Some(AnchorIntent::Function)
        );
        assert_eq!(
            anchor_intent("Yeni ödeme olayının tam tanımlayıcısı nedir?"),
            Some(AnchorIntent::Identifier)
        );
        assert_eq!(
            anchor_intent("Where is the source file written?"),
            Some(AnchorIntent::Path)
        );
        assert_eq!(anchor_intent("How long is the archive retained?"), None);
    }

    #[test]
    fn anchor_evidence_distinguishes_functions_versions_and_paths() {
        assert!(
            anchor_evidence_for_query(
                "abortPendingFrame(frameId) removes a request",
                AnchorIntent::Function,
                ""
            ) > anchor_evidence_for_query(
                "The queue removes a request",
                AnchorIntent::Function,
                ""
            )
        );
        assert!(
            anchor_evidence_for_query(
                "odeme_bildirimi_v2 is current",
                AnchorIntent::Identifier,
                ""
            ) > anchor_evidence_for_query("Payment retry is bounded", AnchorIntent::Identifier, "")
        );
        assert!(
            anchor_evidence_for_query(
                "receipts/payment-proof.json is current",
                AnchorIntent::Path,
                "",
            ) > anchor_evidence_for_query("The payment proof is current", AnchorIntent::Path, "")
        );
    }

    #[test]
    fn hybrid_anchor_lane_promotes_versioned_identifier_over_semantic_tie() {
        let fixture = Fixture::new();
        let mut store = fixture.open("anchor-lane");
        let distractor = store
            .create(
                &operation(),
                input("Ödeme yanıtı kaybolduğunda aynı işlem kimliğini sorgula."),
                None,
            )
            .unwrap();
        let target = store
            .create(
                &operation(),
                input("Geçerli olay adı odeme_bildirimi_v2 olarak kullanılmalıdır."),
                None,
            )
            .unwrap();
        index_record(&mut store, &distractor, 0);
        index_record(&mut store, &target, 0);
        let query = "Yeni ödeme olayını hangi tam tanımlayıcıyla dinlemeliyiz?";
        let result = store
            .search_hybrid(
                SearchRequest {
                    query: query.into(),
                    ..Default::default()
                },
                &query_embedding(query),
            )
            .unwrap();
        assert_eq!(result.ranking[0].reference, reference(&target));
    }

    #[test]
    fn natural_query_lexical_candidates_use_overlap_without_executing_fts_syntax() {
        let fixture = Fixture::new();
        let mut store = fixture.open("natural-query");
        let relevant = store
            .create(
                &operation(),
                input("The invoice archive is retained for ninety days."),
                None,
            )
            .unwrap();
        let neighbor = store
            .create(
                &operation(),
                input("The archive contains photographs."),
                None,
            )
            .unwrap();
        store
            .create(
                &operation(),
                input("Unrelated deployment instructions."),
                None,
            )
            .unwrap();
        let result = search(&mut store, "How long is the invoice archive retained?");
        assert_eq!(result.ranking[0].reference, reference(&relevant));
        assert!(
            result
                .ranking
                .iter()
                .any(|hit| hit.reference == reference(&neighbor))
        );
        // User text is quoted as terms, never interpreted as MATCH operators.
        assert!(search(&mut store, "NOT nonexistent").hits.is_empty());
        assert_eq!(search(&mut store, "invoice OR nonexistent").hits.len(), 1);
        let mut other = fixture.open("other-natural-query");
        assert!(search(&mut other, "invoice archive").hits.is_empty());
    }

    #[test]
    fn hybrid_does_not_promote_incidental_words_and_reports_literal_identity_separately() {
        let fixture = Fixture::new();
        let mut store = fixture.open("literal-confidence");
        let overlap = store
            .create(&operation(), input("The item is archived."), None)
            .unwrap();
        let meaning = store
            .create(&operation(), input("Saklama süresi doksan gündür."), None)
            .unwrap();
        index_record(&mut store, &overlap, 1);
        index_record(&mut store, &meaning, 0);
        let request = SearchRequest {
            query: "Where is the item retained?".into(),
            ..Default::default()
        };
        let result = store
            .search_hybrid(request.clone(), &query_embedding(&request.query))
            .unwrap();
        assert_eq!(result.coverage.lexical_matches, 1);
        assert_eq!(result.coverage.literal_terms, 0);
        assert_eq!(result.ranking[0].reference, reference(&meaning));
        assert_eq!(result.ranking[0].score, 1.0 / 61.0);
        let exact = store
            .create(
                &operation(),
                input("Use export_invoice for reports/çeyrek özeti.csv."),
                None,
            )
            .unwrap();
        let near = store
            .create(
                &operation(),
                input("Use export_invoices for reports/ceyrek özeti.csv."),
                None,
            )
            .unwrap();
        index_record(&mut store, &near, 0);
        let request = SearchRequest {
            query: "Find export_invoice for `reports/çeyrek özeti.csv`".into(),
            ..Default::default()
        };
        let result = store
            .search_hybrid(request.clone(), &query_embedding(&request.query))
            .unwrap();
        assert_eq!(result.coverage.literal_terms, 2);
        assert_eq!(result.coverage.literal_matches, 1);
        assert!(
            result
                .ranking
                .iter()
                .any(|entry| entry.reference == reference(&exact))
        );
        assert!(!contains_literal("export_invoices", "export_invoice"));
        assert!(!contains_literal("prefix/reports/a.csv", "reports/a.csv"));
        assert!(!contains_literal("reports/a.csv.bak", "reports/a.csv"));
        assert!(contains_literal("Saved reports/a.csv.", "reports/a.csv"));
        assert!(!contains_literal("İşlem", "işlem"));
        assert_eq!(
            query_literals("Call cancelQueuedJob() for src/main.rs."),
            vec!["cancelQueuedJob", "src/main.rs"]
        );
        assert_eq!(
            query_literals("Compare ../config/foo .config/foo .env and `file.`"),
            vec!["../config/foo", ".config/foo", ".env", "file."]
        );
        assert!(!contains_literal("/config/foo", "../config/foo"));
        assert!(!contains_literal("config/foo", ".config/foo"));
    }

    #[test]
    fn scopes_are_separate_and_shared_writes_require_preferences() {
        let fixture = Fixture::new();
        let mut a = fixture.open("opaque-project-a");
        let mut b = fixture.open("opaque-project-b");
        let mut shared = MemoryStore::open(&fixture.0, Scope::SharedPreferences).unwrap();
        let record = a
            .create(&operation(), input("isolated project clue"), None)
            .unwrap();
        assert_ne!(a.database_path(), b.database_path());
        assert!(
            a.database_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("scope-")
        );
        assert!(search(&mut b, "clue").hits.is_empty());
        assert!(matches!(
            b.read(&record.record_id, None),
            Err(MemoryError::NotFound)
        ));
        assert!(matches!(
            shared.create(&operation(), input("global fact refused"), None),
            Err(MemoryError::InvalidInput(_))
        ));
        let mut preference = input("prefer concise responses");
        preference.kind = RecordKind::Preference;
        shared.create(&operation(), preference, None).unwrap();
        assert!(search(&mut a, "concise").hits.is_empty());
        assert_eq!(search(&mut shared, "concise").hits.len(), 1);
        drop(a);
        assert_eq!(
            fixture
                .open("opaque-project-a")
                .read(&record.record_id, None)
                .unwrap()
                .data
                .content,
            "isolated project clue"
        );
    }

    #[test]
    fn concurrent_stale_revisions_cannot_overwrite_each_other() {
        let fixture = Fixture::new();
        let mut store = fixture.open("concurrent");
        let original = store.create(&operation(), input("original"), None).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let handles: Vec<_> = ["first candidate", "second candidate"]
            .into_iter()
            .map(|content| {
                let mut connection = fixture.open("concurrent");
                let id = original.record_id.clone();
                let gate = barrier.clone();
                std::thread::spawn(move || {
                    gate.wait();
                    connection.revise(&operation(), &id, 1, input(content))
                })
            })
            .collect();
        let outcomes: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, Err(MemoryError::Conflict)))
                .count(),
            1
        );
        assert_eq!(store.read(&original.record_id, None).unwrap().revision, 2);
        assert_eq!(
            store
                .history(&original.record_id, None, 20, MAX_RESPONSE_BYTES)
                .unwrap()
                .records
                .len(),
            2
        );
    }

    #[test]
    fn revision_history_and_cross_record_supersession_preserve_provenance() {
        let fixture = Fixture::new();
        let mut store = fixture.open("history");
        let first = store
            .create(&operation(), input("obsolete vocabulary"), None)
            .unwrap();
        let mut changed = input("corrected vocabulary");
        changed.basis = ClaimBasis::UserStated;
        changed.sources[0].revision = "source-revision-two".into();
        let second = store
            .revise(&operation(), &first.record_id, 1, changed)
            .unwrap();
        assert_eq!(second.supersedes, Some(reference(&first)));
        let history = store.read(&first.record_id, Some(1)).unwrap();
        assert!(!history.is_current_revision);
        assert_eq!(history.superseded_by, Some(reference(&second)));
        assert_eq!(history.data.sources, first.data.sources);
        assert!(search(&mut store, "obsolete").hits.is_empty());
        let third = store
            .create(
                &operation(),
                input("replacement vocabulary"),
                Some(reference(&second)),
            )
            .unwrap();
        assert_eq!(
            store.read(&first.record_id, None).unwrap().superseded_by,
            Some(reference(&third))
        );
        assert!(search(&mut store, "corrected").hits.is_empty());
        assert!(matches!(
            store.revise(&operation(), &first.record_id, 2, input("stale")),
            Err(MemoryError::Conflict)
        ));
        let page = store
            .history(&first.record_id, None, 1, MAX_RESPONSE_BYTES)
            .unwrap();
        assert!(page.has_more);
        assert_eq!(page.next_before_revision, Some(2));
        assert_eq!(
            store
                .history(
                    &first.record_id,
                    page.next_before_revision,
                    1,
                    MAX_RESPONSE_BYTES
                )
                .unwrap()
                .records[0]
                .revision,
            1
        );
    }

    #[test]
    fn canonical_filters_precede_lexical_match_and_unresolved_remains_explicit() {
        let fixture = Fixture::new();
        let mut store = fixture.open("filters");
        store
            .create(&operation(), input("matching current"), None)
            .unwrap();
        let mut unresolved = input("matching unresolved");
        unresolved.status = RecordStatus::Unresolved;
        store.create(&operation(), unresolved, None).unwrap();
        let mut expired = input("matching expired");
        expired.valid_until = Some(100);
        store.create(&operation(), expired, None).unwrap();
        let mut future = input("matching future");
        future.valid_from = Some(201);
        store.create(&operation(), future, None).unwrap();
        let mut scoped = input("matching another checkout");
        scoped.applicability.checkout_id = Some("checkout-b".into());
        store.create(&operation(), scoped, None).unwrap();
        let context = SearchContext {
            at_time: Some(200),
            checkout_id: Some("checkout-a".into()),
            ..SearchContext::default()
        };
        let result = store
            .search(SearchRequest {
                query: "matching".into(),
                context: context.clone(),
                ..SearchRequest::default()
            })
            .unwrap();
        assert_eq!(result.hits.len(), 2);
        assert_eq!(result.coverage.lexical_matches, 2);
        let result = store
            .search(SearchRequest {
                query: "matching".into(),
                context,
                status: StatusFilter::Unresolved,
                ..SearchRequest::default()
            })
            .unwrap();
        assert_eq!(result.hits.len(), 1);
        assert_eq!(result.hits[0].record.data.status, RecordStatus::Unresolved);
        assert!(!result.coverage.truncated);
    }

    #[test]
    fn fts_operators_and_separators_are_literal_and_unknown_uris_are_data() {
        let fixture = Fixture::new();
        let mut store = fixture.open("literal");
        let mut data = input("alpha-beta OR quoted marker");
        data.sources[0].locator = "unknown+scheme://host/path?do=not-execute".into();
        let record = store.create(&operation(), data, None).unwrap();
        store
            .create(&operation(), input("beta independent"), None)
            .unwrap();
        assert_eq!(search(&mut store, "alpha-beta").hits.len(), 1);
        assert_eq!(search(&mut store, "alpha OR beta").hits.len(), 2);
        assert_eq!(search(&mut store, "OR").hits.len(), 1);
        for query in ["\"", "*", "NEAR(alpha)"] {
            assert!(search(&mut store, query).hits.is_empty());
        }
        // Quoted terms may overlap the first record, but cannot turn this
        // expression into a SQL tautology that also selects the second record.
        let quoted = search(&mut store, "alpha' OR 1=1 --");
        assert_eq!(quoted.hits.len(), 1);
        assert_eq!(quoted.hits[0].record.record_id, record.record_id);
        assert_eq!(
            store.read(&record.record_id, None).unwrap().data.sources[0].locator,
            "unknown+scheme://host/path?do=not-execute"
        );
    }

    #[test]
    fn applicable_critical_constraints_bypass_rank_but_never_applicability() {
        let fixture = Fixture::new();
        let mut store = fixture.open("critical");
        let mut critical = input("must preserve original source");
        critical.critical = true;
        let universal = store.create(&operation(), critical.clone(), None).unwrap();
        critical.applicability.operation = Some("publish".into());
        store.create(&operation(), critical, None).unwrap();
        store
            .create(&operation(), input("matching ordinary"), None)
            .unwrap();
        let result = search(&mut store, "matching");
        assert_eq!(result.hits.len(), 2);
        assert_eq!(result.hits[0].record.record_id, universal.record_id);
        assert_eq!(result.coverage.applicable_current_critical, 1);
        assert_eq!(result.coverage.unclassified_current_critical, 1);
        assert!(!result.coverage.critical_overflow);
        assert!(!result.coverage.truncated);
        let mismatch = store
            .search(SearchRequest {
                query: "matching".into(),
                context: SearchContext {
                    operation: Some("read".into()),
                    ..SearchContext::default()
                },
                ..SearchRequest::default()
            })
            .unwrap();
        assert_eq!(mismatch.coverage.unclassified_current_critical, 0);
        assert_eq!(mismatch.coverage.applicable_current_critical, 1);
        let overlap = search(&mut store, "preserve");
        assert!(!overlap.coverage.truncated);
        let exact = store
            .search(SearchRequest {
                context: SearchContext {
                    operation: Some("publish".into()),
                    ..SearchContext::default()
                },
                ..SearchRequest::default()
            })
            .unwrap();
        assert_eq!(exact.coverage.returned_critical, 2);
    }

    #[test]
    fn critical_count_and_serialized_byte_budgets_report_omissions() {
        let fixture = Fixture::new();
        let mut store = fixture.open("budgets");
        for n in 0..22 {
            let mut data = input(&format!("constraint {n}"));
            data.critical = true;
            store.create(&operation(), data, None).unwrap();
        }
        let result = search(&mut store, "unmatched");
        assert_eq!(result.hits.len(), MAX_RESULTS);
        assert_eq!(result.coverage.applicable_current_critical, 22);
        assert!(result.coverage.critical_overflow);
        assert!(result.coverage.truncated);
        let bounded = store
            .search(SearchRequest {
                max_bytes: 1024,
                ..SearchRequest::default()
            })
            .unwrap();
        assert!(serde_json::to_vec(&bounded).unwrap().len() <= 1024);
        assert!(bounded.coverage.critical_overflow);
        let mut huge = fixture.open("escaped");
        let mut data = input(&"\n".repeat(MAX_CONTENT_BYTES));
        data.content.push('x');
        assert!(huge.create(&operation(), data, None).is_err());
        let mut data = input(&"\n".repeat(MAX_CONTENT_BYTES - 1));
        data.content.push('x');
        data.critical = true;
        huge.create(&operation(), data, None).unwrap();
        let result = search(&mut huge, "x");
        assert!(result.hits.is_empty());
        assert!(result.coverage.critical_overflow);
        assert!(serde_json::to_vec(&result).unwrap().len() <= MAX_RESPONSE_BYTES);
    }

    #[test]
    fn invalid_inputs_fail_before_writing_and_integer_range_is_checked() {
        let fixture = Fixture::new();
        let mut store = fixture.open("validation");
        for invalid in [input(""), input(&"x".repeat(MAX_CONTENT_BYTES + 1))] {
            assert!(store.create(&operation(), invalid, None).is_err());
        }
        let mut data = input("bad timestamp");
        data.sources[0].captured_at = u64::MAX;
        assert!(store.create(&operation(), data, None).is_err());
        let mut data = input("bad interval");
        data.valid_from = Some(2);
        data.valid_until = Some(1);
        assert!(store.create(&operation(), data, None).is_err());
        assert!(
            store
                .revise(&operation(), "some-id", u64::MAX, input("stale"))
                .is_err()
        );
        assert!(
            store
                .search(SearchRequest {
                    limit: MAX_RESULTS + 1,
                    ..SearchRequest::default()
                })
                .is_err()
        );
        assert!(
            store
                .search(SearchRequest {
                    context: SearchContext {
                        at_time: Some(u64::MAX),
                        ..SearchContext::default()
                    },
                    ..SearchRequest::default()
                })
                .is_err()
        );
        assert!(search(&mut store, "").hits.is_empty());
        assert!(
            MemoryStore::open(&fixture.0, Scope::Project("/private/project/path".into())).is_err()
        );
        assert!(
            store
                .connection
                .query_row("SELECT -1", [], |row| unsigned(row, 0))
                .is_err()
        );
    }

    #[test]
    fn metadata_scope_swap_and_unknown_schema_are_refused() {
        let fixture = Fixture::new();
        let a = fixture.open("scope-a");
        let path_a = a.database_path().to_owned();
        drop(a);
        let b = fixture.open("scope-b");
        let path_b = b.database_path().to_owned();
        drop(b);
        fs::copy(&path_a, &path_b).unwrap();
        assert!(matches!(
            MemoryStore::open(&fixture.0, Scope::Project("scope-b".into())),
            Err(MemoryError::ScopeMismatch)
        ));
        let connection = Connection::open(&path_a).unwrap();
        connection.pragma_update(None, "user_version", 99).unwrap();
        drop(connection);
        assert!(matches!(
            MemoryStore::open(&fixture.0, Scope::Project("scope-a".into())),
            Err(MemoryError::UnsupportedSchema)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_database_and_nonprivate_state_are_refused() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let fixture = Fixture::new();
        let store = fixture.open("links");
        let database = store.database_path().to_owned();
        drop(store);
        let original = fixture.0.join("original.sqlite3");
        fs::rename(&database, &original).unwrap();
        symlink(&original, &database).unwrap();
        assert!(MemoryStore::open(&fixture.0, Scope::Project("links".into())).is_err());
        fs::set_permissions(&fixture.0, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(MemoryStore::open(&fixture.0, Scope::Project("new".into())).is_err());
    }

    #[test]
    fn qualified_runtime_and_no_extension_loader_are_observed() {
        let fixture = Fixture::new();
        let store = fixture.open("runtime");
        verify_runtime(&store.connection).unwrap();
        assert!(
            store
                .connection
                .query_row("SELECT load_extension('not-a-library')", [], |_| Ok(()))
                .is_err()
        );
        let mode: String = store
            .connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(mode, "delete");
    }
    #[test]
    fn response_loss_replays_exact_revision_and_conflicting_id_reuse_is_refused() {
        let fixture = Fixture::new();
        let mut store = fixture.open("receipts");
        let create_id = operation();
        let original = store
            .create(&create_id, input("first value"), None)
            .unwrap();
        let revise_id = operation();
        let corrected = store
            .revise(&revise_id, &original.record_id, 1, input("second value"))
            .unwrap();
        drop(store);
        let mut store = fixture.open("receipts");
        let replay = store
            .create(&create_id, input("first value"), None)
            .unwrap();
        assert_eq!(replay.record_id, original.record_id);
        assert_eq!(replay.revision, 1);
        assert_eq!(replay.data, original.data);
        assert!(!replay.is_current_revision);
        let replay = store
            .revise(&revise_id, &original.record_id, 1, input("second value"))
            .unwrap();
        assert_eq!(replay.revision, corrected.revision);
        assert!(matches!(
            store.create(&create_id, input("changed intent"), None),
            Err(MemoryError::OperationConflict)
        ));
        assert!(matches!(
            store.revise(&create_id, &original.record_id, 2, input("first value")),
            Err(MemoryError::OperationConflict)
        ));
        let receipt = store.read_mutation(&create_id).unwrap().unwrap();
        assert_eq!(receipt.record_id, original.record_id);
        assert_eq!(receipt.revision, 1);
        assert!(receipt.available);
        assert_eq!(receipt.intent_hash.len(), 64);
        assert!(
            !serde_json::to_string(&receipt)
                .unwrap()
                .contains("first value")
        );
        assert!(store.read_mutation("op-unknown").unwrap().is_none());
        assert_eq!(
            store
                .history(&original.record_id, None, 20, MAX_RESPONSE_BYTES)
                .unwrap()
                .records
                .len(),
            2
        );
    }

    #[test]
    fn concurrent_same_operation_commits_once_and_returns_one_identity() {
        let fixture = Fixture::new();
        let barrier = Arc::new(Barrier::new(2));
        let operation_id = operation();
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let mut store = fixture.open("same-operation");
                let id = operation_id.clone();
                let gate = barrier.clone();
                std::thread::spawn(move || {
                    gate.wait();
                    store.create(&id, input("once only"), None).unwrap()
                })
            })
            .collect();
        let records: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(records[0].record_id, records[1].record_id);
        assert_eq!(
            search(&mut fixture.open("same-operation"), "once")
                .hits
                .len(),
            1
        );
    }

    #[test]
    fn unavailable_mutation_target_never_recreates_deleted_memory() {
        let fixture = Fixture::new();
        let mut store = fixture.open("unavailable");
        let id = operation();
        store.create(&id, input("removed later"), None).unwrap();
        // Simulate the future forgetting owner's canonical deletion, preserving
        // content-free replay tombstones. This is not a public forget API.
        store.connection.execute_batch("DELETE FROM sources; DELETE FROM record_fts; DELETE FROM revisions; DELETE FROM records;").unwrap();
        assert!(!store.read_mutation(&id).unwrap().unwrap().available);
        assert!(matches!(
            store.create(&id, input("removed later"), None),
            Err(MemoryError::MutationUnavailable)
        ));
        assert!(search(&mut store, "removed").hits.is_empty());
    }

    #[test]
    fn failed_cas_does_not_reserve_a_mutation_receipt_or_modify_heads() {
        let fixture = Fixture::new();
        let mut store = fixture.open("atomic");
        let record = store.create(&operation(), input("stable"), None).unwrap();
        let failed = operation();
        assert!(matches!(
            store.revise(&failed, &record.record_id, 2, input("invalid")),
            Err(MemoryError::Conflict)
        ));
        assert!(store.read_mutation(&failed).unwrap().is_none());
        assert_eq!(
            store.read(&record.record_id, None).unwrap().data.content,
            "stable"
        );
        assert!(
            store
                .create("not-server-issued", input("invalid id"), None)
                .is_err()
        );
    }
    #[test]
    fn hybrid_reports_overflow_when_rrf_drops_the_twenty_first_union_candidate() {
        use crate::embedding::{ChunkEmbedding, PassageEmbedding, QueryEmbedding};

        let fixture = Fixture::new();
        let mut store = fixture.open("rrf-union-overflow");
        let identity = crate::embedding::expected_model_identity().unwrap();
        let mut vector = vec![0.0; crate::embedding::DIMENSIONS];
        vector[0] = 1.0;
        let mut records = Vec::new();
        for position in 0..21 {
            let content = if position < 20 {
                "needle_id shared lexical content"
            } else {
                "semantic candidate without the literal term"
            };
            let record = store.create(&operation(), input(content), None).unwrap();
            // Equal unit vectors produce a deterministic row-ID tie order:
            // lexical ranks 1..20; semantic ranks 1..19,21. RRF's final tie
            // retains row20 and drops row21 although each source has only20.
            if position != 19 {
                let embedding = PassageEmbedding {
                    model_identity: identity.clone(),
                    input_bytes: content.len(),
                    content_tokens: 4,
                    covered_bytes: content.len(),
                    complete: true,
                    chunks: vec![ChunkEmbedding {
                        start_byte: 0,
                        end_byte: content.len(),
                        input_tokens: 6,
                        vector: vector.clone(),
                    }],
                };
                store
                    .apply_index(
                        &reference(&record),
                        &index::source_digest(content),
                        &embedding,
                    )
                    .unwrap();
            }
            records.push(record);
        }
        let query = QueryEmbedding {
            model_identity: identity,
            source_sha256: crate::embedding::source_digest("needle_id"),
            input_bytes: "needle_id".len(),
            input_tokens: 4,
            vector,
        };
        let result = store
            .search_hybrid(
                SearchRequest {
                    query: "needle_id".into(),
                    ..SearchRequest::default()
                },
                &query,
            )
            .unwrap();
        assert_eq!(result.coverage.lexical_matches, 20);
        assert_eq!(result.coverage.semantic_indexed_eligible, 20);
        assert_eq!(result.coverage.semantic_missing_eligible, 1);
        assert_eq!(result.ranking.len(), 20);
        assert_eq!(result.hits.len(), 20);
        assert!(
            result
                .hits
                .iter()
                .all(|hit| hit.record.record_id != records[20].record_id)
        );
        assert!(
            result
                .hits
                .iter()
                .any(|hit| hit.record.record_id == records[19].record_id)
        );
        assert!(result.coverage.truncated);
    }

    #[test]
    fn read_only_open_does_not_create_or_modify_storage() {
        let fixture = Fixture::new();
        let scope = Scope::Project("read-only".into());
        assert!(matches!(
            MemoryStore::open_existing(&fixture.0, scope.clone()),
            Err(MemoryError::NotFound)
        ));
        assert!(!fixture.0.join("memory").exists());
        let mut writable = MemoryStore::open(&fixture.0, scope.clone()).unwrap();
        let id = operation();
        let record = writable.create(&id, input("readable"), None).unwrap();
        let path = writable.database_path().to_owned();
        drop(writable);
        let bytes = fs::read(&path).unwrap();
        let names = fs::read_dir(fixture.0.join("memory")).unwrap().count();
        let mut readonly = MemoryStore::open_existing(&fixture.0, scope).unwrap();
        assert_eq!(
            readonly.read(&record.record_id, None).unwrap().data.content,
            "readable"
        );
        assert!(readonly.read_mutation(&id).unwrap().unwrap().available);
        assert_eq!(search(&mut readonly, "readable").hits.len(), 1);
        assert!(
            readonly
                .create(&operation(), input("cannot write"), None)
                .is_err()
        );
        assert!(matches!(
            MemoryStore::open_existing(&fixture.0, Scope::Project("absent".into())),
            Err(MemoryError::NotFound)
        ));
        drop(readonly);
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert_eq!(
            fs::read_dir(fixture.0.join("memory")).unwrap().count(),
            names
        );
    }
}
