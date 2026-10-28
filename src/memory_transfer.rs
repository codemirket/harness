//! Strict, bounded canonical JSON Lines interchange. The caller owns locks,
//! staged files, stream hashes, lineage exclusions and atomic publication.
//! Import accepts a new empty connection, never an external SQLite database.
use super::*;
use std::io::{BufRead, BufReader, Read, Write};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Instant;

pub(crate) const MAX_STREAM_BYTES: u64 = 512 * 1024 * 1024;
// Covers a maximum-size canonical record even when each byte needs a six-byte
// JSON escape (content, eight provenance sources and three selectors).
pub(crate) const MAX_LINE_BYTES: usize = 256 * 1024;
pub(crate) const MAX_REVISIONS: u64 = 1_000_000;

const DEADLINE: Duration = Duration::from_secs(120);
// A finite joined watchdog interrupts native SQL, including one long statement.
// Reader/writer calls remain caller-owned; the deadline is checked between calls.
struct Deadline {
    end: Instant,
    stop: Arc<(Mutex<bool>, Condvar)>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Deadline {
    fn new(connection: &Connection) -> Result<Self> {
        Self::with_duration(connection, DEADLINE)
    }
    fn with_duration(connection: &Connection, duration: Duration) -> Result<Self> {
        let end = Instant::now() + duration;
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let child = stop.clone();
        let interrupt = connection.get_interrupt_handle();
        let handle = thread::Builder::new()
            .name("memory-transfer-deadline".into())
            .spawn(move || {
                let (lock, wake) = &*child;
                let Ok(mut stopped) = lock.lock() else {
                    interrupt.interrupt();
                    return;
                };
                while !*stopped {
                    let remaining = end.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        interrupt.interrupt();
                        break;
                    }
                    match wake.wait_timeout(stopped, remaining) {
                        Ok((guard, _)) => stopped = guard,
                        Err(_) => {
                            interrupt.interrupt();
                            break;
                        }
                    }
                }
            })?;
        Ok(Self {
            end,
            stop,
            thread: Some(handle),
        })
    }
    fn check(&self) -> Result<()> {
        if Instant::now() >= self.end {
            Err(invalid("memory transfer deadline exceeded"))
        } else {
            Ok(())
        }
    }
}
impl Drop for Deadline {
    fn drop(&mut self) {
        let (lock, wake) = &*self.stop;
        if let Ok(mut stop) = lock.lock() {
            *stop = true;
            wake.notify_all();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransferStats {
    pub bytes: u64,
    pub records: u64,
    pub revisions: u64,
    /// Opaque links to absent records; never materialized or treated as authority.
    pub dangling_supersedes: u64,
    /// Retired heads with no surviving successor, possible after forgetting.
    pub retired_heads_without_successor: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "project_id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum WireScope {
    Project(String),
    SharedPreferences,
}
impl From<&Scope> for WireScope {
    fn from(scope: &Scope) -> Self {
        match scope {
            Scope::Project(id) => Self::Project(id.clone()),
            Scope::SharedPreferences => Self::SharedPreferences,
        }
    }
}
impl From<WireScope> for Scope {
    fn from(scope: WireScope) -> Self {
        match scope {
            WireScope::Project(id) => Self::Project(id),
            WireScope::SharedPreferences => Self::SharedPreferences,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Line {
    Header {
        version: u32,
        scope: WireScope,
    },
    Revision {
        head_revision: u64,
        record: Box<Record>,
    },
    Footer {
        records: u64,
        revisions: u64,
    },
}
fn invalid(message: &'static str) -> MemoryError {
    MemoryError::InvalidInput(message)
}

struct Lines<R> {
    reader: BufReader<R>,
    bytes: u64,
}
impl<R: Read> Lines<R> {
    fn next(&mut self) -> Result<Option<Line>> {
        let mut line = Vec::new();
        (&mut self.reader)
            .take(MAX_LINE_BYTES as u64 + 1)
            .read_until(b'\n', &mut line)?;
        self.bytes = self
            .bytes
            .checked_add(line.len() as u64)
            .ok_or(invalid("transfer byte limit exceeded"))?;
        if self.bytes > MAX_STREAM_BYTES || line.len() > MAX_LINE_BYTES {
            return Err(invalid("transfer byte limit exceeded"));
        }
        if line.is_empty() {
            return Ok(None);
        }
        if line.last() != Some(&b'\n') {
            return Err(invalid("transfer line is truncated"));
        }
        Ok(Some(serde_json::from_slice(&line)?))
    }
}
fn write_line(writer: &mut impl Write, line: &Line, stats: &mut TransferStats) -> Result<()> {
    let mut bytes = serde_json::to_vec(line)?;
    bytes.push(b'\n');
    if bytes.len() > MAX_LINE_BYTES || stats.bytes + bytes.len() as u64 > MAX_STREAM_BYTES {
        return Err(invalid("transfer byte limit exceeded"));
    }
    writer.write_all(&bytes)?;
    stats.bytes += bytes.len() as u64;
    Ok(())
}
fn validate_record(scope: &Scope, r: &Record, head: u64) -> Result<()> {
    validate_reference(&RecordRef {
        record_id: r.record_id.clone(),
        revision: r.revision,
    })?;
    valid_revision(head)?;
    valid_time(r.recorded_at)?;
    validate_input(scope, &r.data)?;
    if r.revision > head || (r.is_current_revision && r.revision != head) {
        return Err(invalid("invalid transfer head or current revision"));
    }
    for reference in [&r.supersedes, &r.superseded_by].into_iter().flatten() {
        validate_reference(reference)?;
    }
    if r.data.kind == RecordKind::TaskState && r.data.applicability.checkout_id.is_none() {
        return Err(invalid("task state requires an explicit checkout"));
    }
    if *scope == Scope::SharedPreferences
        && (r.data.applicability.task_key.is_some() || r.data.applicability.checkout_id.is_some())
    {
        return Err(invalid(
            "shared preferences cannot contain project selectors",
        ));
    }
    if r.revision > 1 {
        if r.supersedes
            != Some(RecordRef {
                record_id: r.record_id.clone(),
                revision: r.revision - 1,
            })
        {
            return Err(invalid("revision history is not contiguous"));
        }
    } else if r
        .supersedes
        .as_ref()
        .is_some_and(|s| s.record_id == r.record_id)
    {
        return Err(invalid("initial revision cannot supersede itself"));
    }
    Ok(())
}

/// Validate canonical graph without retaining its contents in Rust memory.
fn validate_graph(tx: &Transaction<'_>, deadline: &Deadline) -> Result<(u64, u64)> {
    deadline.check()?;
    tx.execute_batch("CREATE TEMP TABLE transfer_edges AS SELECT record_id,revision,supersedes_record_id,supersedes_revision FROM revisions;
        CREATE UNIQUE INDEX transfer_edge_origin ON transfer_edges(record_id,revision);
        CREATE INDEX transfer_edge_target ON transfer_edges(supersedes_record_id,supersedes_revision);")?;
    deadline.check()?;
    let bad: bool = tx.query_row("SELECT EXISTS(
        SELECT 1 FROM records r LEFT JOIN revisions v ON v.record_id=r.record_id
        GROUP BY r.record_id HAVING min(v.revision) IS NOT 1 OR max(v.revision) IS NOT r.head_revision OR count(v.row_id) != r.head_revision
        UNION ALL SELECT 1 FROM revisions v LEFT JOIN records r ON r.record_id=v.record_id WHERE r.record_id IS NULL
        UNION ALL SELECT 1 FROM sources s LEFT JOIN revisions v ON v.row_id=s.revision_row_id WHERE v.row_id IS NULL
        UNION ALL SELECT 1 FROM revisions v LEFT JOIN revisions p ON p.record_id=v.supersedes_record_id AND p.revision=v.supersedes_revision
        WHERE (v.supersedes_record_id IS NULL) != (v.supersedes_revision IS NULL)
           OR (v.supersedes_record_id IS NOT NULL AND p.row_id IS NULL AND EXISTS(SELECT 1 FROM records present WHERE present.record_id=v.supersedes_record_id))
           OR (v.revision>1 AND (v.supersedes_record_id IS NOT v.record_id OR v.supersedes_revision IS NOT v.revision-1))
           OR (v.revision=1 AND v.supersedes_record_id=v.record_id)
        UNION ALL SELECT 1 FROM revisions WHERE supersedes_record_id IS NOT NULL GROUP BY supersedes_record_id,supersedes_revision HAVING count(*)>1
        UNION ALL SELECT 1 FROM revisions v JOIN records r ON r.record_id=v.record_id
        WHERE v.is_current=1 AND (v.revision!=r.head_revision OR EXISTS(SELECT 1 FROM transfer_edges n WHERE n.supersedes_record_id=v.record_id AND n.supersedes_revision=v.revision))
        UNION ALL SELECT 1 FROM revisions v JOIN records p ON p.record_id=v.supersedes_record_id
        WHERE v.record_id!=v.supersedes_record_id AND v.supersedes_revision!=p.head_revision
    )", [], |r| r.get(0))?;
    if bad {
        return Err(invalid("transfer contains invalid canonical history"));
    }
    // Each predecessor has at most one successor. A roots-to-successors walk
    // must visit every revision; otherwise a disconnected cycle exists.
    deadline.check()?;
    let reachable: u64 = tx.query_row("WITH RECURSIVE reachable(id,rev) AS (
        SELECT v.record_id,v.revision FROM revisions v WHERE NOT EXISTS(SELECT 1 FROM revisions p WHERE p.record_id=v.supersedes_record_id AND p.revision=v.supersedes_revision)
        UNION ALL SELECT v.record_id,v.revision FROM transfer_edges v JOIN reachable p ON v.supersedes_record_id=p.id AND v.supersedes_revision=p.rev
        ) SELECT count(*) FROM reachable", [], |r| unsigned(r,0))?;
    deadline.check()?;
    let count: u64 = tx.query_row("SELECT count(*) FROM revisions", [], |r| unsigned(r, 0))?;
    if reachable != count {
        return Err(invalid("transfer history contains a cycle"));
    }
    deadline.check()?;
    let dangling = tx.query_row("SELECT count(*) FROM revisions v WHERE v.supersedes_record_id IS NOT NULL AND NOT EXISTS(SELECT 1 FROM records p WHERE p.record_id=v.supersedes_record_id)", [], |r|unsigned(r,0))?;
    deadline.check()?;
    let retired = tx.query_row("SELECT count(*) FROM revisions v JOIN records r ON r.record_id=v.record_id AND r.head_revision=v.revision WHERE v.is_current=0 AND NOT EXISTS(SELECT 1 FROM transfer_edges n WHERE n.supersedes_record_id=v.record_id AND n.supersedes_revision=v.revision)", [], |r|unsigned(r,0))?;
    Ok((dangling, retired))
}

// Same canonical fields as read_record; indexed temporary successor lookup
// avoids its unindexed per-record reverse-link scan for million-revision exports.
fn read_export_record(tx: &Transaction<'_>, id: &str, revision: u64) -> Result<Record> {
    let body:String=tx.query_row("SELECT json_object(
        'record_id',v.record_id,'revision',v.revision,'recorded_at',v.recorded_at,
        'is_current_revision',json(CASE v.is_current WHEN 1 THEN 'true' ELSE 'false' END),
        'supersedes',json(CASE WHEN v.supersedes_record_id IS NULL THEN NULL ELSE json_object('record_id',v.supersedes_record_id,'revision',v.supersedes_revision) END),
        'superseded_by',json((SELECT json_object('record_id',n.record_id,'revision',n.revision) FROM transfer_edges n WHERE n.supersedes_record_id=v.record_id AND n.supersedes_revision=v.revision)),
        'data',json_object('kind',v.kind,'basis',v.basis,'content',v.content,'status',v.status,
        'critical',json(CASE v.critical WHEN 1 THEN 'true' ELSE 'false' END),
        'valid_from',v.valid_from,'valid_until',v.valid_until,
        'applicability',json_object('task_key',v.task_key,'checkout_id',v.checkout_id,'operation',v.operation),
        'sources',json((SELECT json_group_array(json(body)) FROM (
            SELECT json_object('source_id',s.source_id,'revision',s.source_revision,'locator',s.locator,'captured_at',s.captured_at) body
            FROM sources s WHERE s.revision_row_id=v.row_id ORDER BY s.ordinal)))))
        FROM revisions v WHERE v.record_id=?1 AND v.revision=?2",params![id,revision as i64],|r|r.get(0))?;
    Ok(serde_json::from_str(&body)?)
}

/// Stable read transaction. Output may be partial on error; only the caller can
/// publish a fully successful staged stream. No flush or filesystem publication.
pub(crate) fn export(
    connection: &mut Connection,
    scope: &Scope,
    mut writer: impl Write,
) -> Result<TransferStats> {
    let deadline = Deadline::new(connection)?;
    validate_scope(scope)?;
    verify_runtime(connection)?;
    initialize(connection, &serde_json::to_string(scope)?, false)?;
    let tx = connection.transaction()?;
    let count: u64 = tx.query_row("SELECT count(*) FROM revisions", [], |r| unsigned(r, 0))?;
    if count > MAX_REVISIONS {
        return Err(invalid("transfer revision limit exceeded"));
    }
    let (dangling_supersedes, retired_heads_without_successor) = validate_graph(&tx, &deadline)?;
    let mut stats = TransferStats {
        dangling_supersedes,
        retired_heads_without_successor,
        ..TransferStats::default()
    };
    write_line(
        &mut writer,
        &Line::Header {
            version: 1,
            scope: scope.into(),
        },
        &mut stats,
    )?;
    {
        let mut statement = tx.prepare("SELECT v.record_id,v.revision,r.head_revision FROM revisions v JOIN records r ON r.record_id=v.record_id ORDER BY v.record_id COLLATE BINARY,v.revision")?;
        let mut rows = statement.query([])?;
        let mut previous = String::new();
        while let Some(row) = rows.next()? {
            deadline.check()?;
            let id: String = row.get(0)?;
            let revision = unsigned(row, 1)?;
            let head = unsigned(row, 2)?;
            let record = read_export_record(&tx, &id, revision)?;
            validate_record(scope, &record, head)?;
            // Source ordinals are part of the canonical sequence, not arbitrary
            // sparse positions that could be silently compacted by read_record.
            let bad: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM sources s JOIN revisions v ON v.row_id=s.revision_row_id WHERE v.record_id=?1 AND v.revision=?2 GROUP BY s.revision_row_id HAVING min(s.ordinal)!=0 OR max(s.ordinal)!=count(*)-1)", params![id,revision as i64], |r|r.get(0))?;
            if bad {
                return Err(invalid("source ordinals are not contiguous"));
            }
            if previous != id {
                stats.records += 1;
                previous = id;
            }
            stats.revisions += 1;
            write_line(
                &mut writer,
                &Line::Revision {
                    head_revision: head,
                    record: Box::new(record),
                },
                &mut stats,
            )?;
        }
    }
    let footer = Line::Footer {
        records: stats.records,
        revisions: stats.revisions,
    };
    write_line(&mut writer, &footer, &mut stats)?;
    tx.execute_batch("DROP TABLE transfer_edges;")?;
    deadline.check()?;
    tx.commit()?;
    Ok(stats)
}

/// Initialize only an empty supplied database. On malformed input all imported
/// data rolls back; the known empty schema may remain. Caller discards its stage.
/// This codec does not authorize import into live state or check external forget
/// lineage / checkout registration; the publication owner must perform those.
pub(crate) fn import(
    connection: &mut Connection,
    scope: &Scope,
    reader: impl Read,
) -> Result<TransferStats> {
    let deadline = Deadline::new(connection)?;
    validate_scope(scope)?;
    verify_runtime(connection)?;
    let entries: u64 =
        connection.query_row("SELECT count(*) FROM sqlite_schema", [], |r| unsigned(r, 0))?;
    if entries != 0 {
        return Err(invalid("transfer import requires a new empty database"));
    }
    let mut lines = Lines {
        reader: BufReader::new(reader),
        bytes: 0,
    };
    match lines.next()? {
        Some(Line::Header {
            version: 1,
            scope: received,
        }) => {
            if Scope::from(received) != *scope {
                return Err(invalid("transfer header scope mismatch"));
            }
        }
        _ => return Err(invalid("transfer header version or scope mismatch")),
    }
    deadline.check()?;
    initialize(connection, &serde_json::to_string(scope)?, true)?;
    let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute_batch("CREATE TEMP TABLE transfer_successors(record_id TEXT NOT NULL,revision INTEGER NOT NULL,next_id TEXT,next_revision INTEGER,PRIMARY KEY(record_id,revision)) STRICT;")?;
    let mut stats = TransferStats::default();
    let mut previous: Option<(String, u64, u64)> = None;
    loop {
        deadline.check()?;
        match lines.next()?.ok_or(invalid("transfer footer missing"))? {
            Line::Revision {
                head_revision,
                record: r,
            } => {
                if stats.revisions == MAX_REVISIONS {
                    return Err(invalid("transfer revision limit exceeded"));
                }
                validate_record(scope, &r, head_revision)?;
                let same = previous.as_ref().is_some_and(|p| p.0 == r.record_id);
                if same {
                    let p = previous.as_ref().expect("checked previous");
                    if r.revision != p.1 + 1 || head_revision != p.2 {
                        return Err(invalid("transfer revisions or heads are inconsistent"));
                    }
                } else {
                    if r.revision != 1
                        || previous
                            .as_ref()
                            .is_some_and(|p| p.0 >= r.record_id || p.1 != p.2)
                    {
                        return Err(invalid("transfer record ordering or history is invalid"));
                    }
                    tx.execute(
                        "INSERT INTO records(record_id,head_revision) VALUES(?1,?2)",
                        params![r.record_id, head_revision as i64],
                    )?;
                    stats.records += 1;
                }
                insert_revision(
                    &tx,
                    &r.record_id,
                    r.revision,
                    &r.data,
                    r.recorded_at,
                    r.supersedes.as_ref(),
                )?;
                if !r.is_current_revision {
                    retire(
                        &tx,
                        &RecordRef {
                            record_id: r.record_id.clone(),
                            revision: r.revision,
                        },
                    )?;
                }
                tx.execute(
                    "INSERT INTO transfer_successors VALUES(?1,?2,?3,?4)",
                    params![
                        r.record_id,
                        r.revision as i64,
                        r.superseded_by.as_ref().map(|v| v.record_id.as_str()),
                        r.superseded_by.as_ref().map(|v| v.revision as i64)
                    ],
                )?;
                previous = Some((r.record_id, r.revision, head_revision));
                stats.revisions += 1;
            }
            Line::Footer { records, revisions } => {
                if records != stats.records
                    || revisions != stats.revisions
                    || previous.as_ref().is_some_and(|p| p.1 != p.2)
                {
                    return Err(invalid("transfer footer or final history mismatch"));
                }
                if lines.next()?.is_some() {
                    return Err(invalid("transfer has trailing records"));
                }
                break;
            }
            Line::Header { .. } => return Err(invalid("duplicate transfer header")),
        }
    }
    (
        stats.dangling_supersedes,
        stats.retired_heads_without_successor,
    ) = validate_graph(&tx, &deadline)?;
    deadline.check()?;
    let wrong: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM transfer_successors s LEFT JOIN transfer_edges v ON v.supersedes_record_id=s.record_id AND v.supersedes_revision=s.revision WHERE s.next_id IS NOT v.record_id OR s.next_revision IS NOT v.revision)", [], |r|r.get(0))?;
    if wrong {
        return Err(invalid("transfer successor claims do not match history"));
    }
    tx.execute_batch("DROP TABLE transfer_successors;")?;
    stats.bytes = lines.bytes;
    tx.execute_batch("DROP TABLE transfer_edges;")?;
    deadline.check()?;
    tx.commit()?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn scope() -> Scope {
        Scope::Project("transfer-project".into())
    }
    fn input(text: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::Observed,
            content: text.into(),
            sources: vec![
                Source {
                    source_id: "source-2".into(),
                    revision: "revision-2".into(),
                    locator: "line:20".into(),
                    captured_at: 20,
                },
                Source {
                    source_id: "source-1".into(),
                    revision: "revision-1".into(),
                    locator: "line:10".into(),
                    captured_at: 10,
                },
            ],
            status: RecordStatus::Current,
            critical: true,
            applicability: Applicability::default(),
            valid_from: Some(10),
            valid_until: Some(1000),
        }
    }
    fn fixture() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        initialize(&mut c, &serde_json::to_string(&scope()).unwrap(), true).unwrap();
        let tx = c.transaction().unwrap();
        tx.execute("INSERT INTO records VALUES('a',2),('b',1),('task',1)", [])
            .unwrap();
        insert_revision(&tx, "a", 1, &input("original"), 100, None).unwrap();
        retire(
            &tx,
            &RecordRef {
                record_id: "a".into(),
                revision: 1,
            },
        )
        .unwrap();
        insert_revision(
            &tx,
            "a",
            2,
            &input("corrected"),
            101,
            Some(&RecordRef {
                record_id: "a".into(),
                revision: 1,
            }),
        )
        .unwrap();
        retire(
            &tx,
            &RecordRef {
                record_id: "a".into(),
                revision: 2,
            },
        )
        .unwrap();
        insert_revision(
            &tx,
            "b",
            1,
            &input("replacement"),
            102,
            Some(&RecordRef {
                record_id: "a".into(),
                revision: 2,
            }),
        )
        .unwrap();
        let mut task = input("unfinished work");
        task.kind = RecordKind::TaskState;
        task.status = RecordStatus::Unresolved;
        task.applicability = Applicability {
            task_key: Some("task-key".into()),
            checkout_id: Some("checkout-id".into()),
            operation: Some("inspect".into()),
        };
        insert_revision(&tx, "task", 1, &task, 103, None).unwrap();
        tx.execute(
            "INSERT INTO mutations VALUES('op-not-portable','hash','a',1)",
            [],
        )
        .unwrap();
        tx.execute(
            "INSERT INTO metadata VALUES('last_restore_operation','op-not-portable')",
            [],
        )
        .unwrap();
        tx.commit().unwrap();
        c
    }
    fn stream(c: &mut Connection) -> Vec<u8> {
        let mut output = Vec::new();
        export(c, &scope(), &mut output).unwrap();
        output
    }
    fn values(bytes: &[u8]) -> Vec<Value> {
        bytes
            .split(|b| *b == b'\n')
            .filter(|s| !s.is_empty())
            .map(|s| serde_json::from_slice(s).unwrap())
            .collect()
    }
    fn encode(values: &[Value]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|v| {
                let mut b = serde_json::to_vec(v).unwrap();
                b.push(b'\n');
                b
            })
            .collect()
    }
    fn rejects(bytes: &[u8]) {
        let mut target = Connection::open_in_memory().unwrap();
        assert!(import(&mut target, &scope(), bytes).is_err());
        let exists: bool = target
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name='records')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        if exists {
            assert_eq!(
                target
                    .query_row("SELECT count(*) FROM records", [], |r| r.get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
    }
    #[test]
    fn roundtrip_preserves_history_task_provenance_and_only_rebuilds_current_fts() {
        let mut original = fixture();
        let bytes = stream(&mut original);
        let tx = original.transaction().unwrap();
        for value in values(&bytes).iter().filter(|v| v["type"] == "revision") {
            let id = value["record"]["record_id"].as_str().unwrap();
            let rev = value["record"]["revision"].as_u64().unwrap();
            assert_eq!(
                serde_json::to_value(read_record(&tx, id, rev).unwrap()).unwrap(),
                value["record"]
            );
        }
        tx.commit().unwrap();
        let mut target = Connection::open_in_memory().unwrap();
        let stats = import(&mut target, &scope(), bytes.as_slice()).unwrap();
        assert_eq!(
            (stats.records, stats.revisions, stats.bytes),
            (3, 4, bytes.len() as u64)
        );
        assert_eq!(stream(&mut target), bytes);
        assert_eq!(
            target
                .query_row("SELECT count(*) FROM mutations", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            target
                .query_row(
                    "SELECT count(*) FROM metadata WHERE key='last_restore_operation'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            target
                .query_row("SELECT count(*) FROM record_fts", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            target
                .query_row(
                    "SELECT count(*) FROM record_fts WHERE record_fts MATCH 'original'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        let tx = target.transaction().unwrap();
        let task = read_record(&tx, "task", 1).unwrap();
        assert_eq!(
            task.data.applicability.checkout_id.as_deref(),
            Some("checkout-id")
        );
        assert_eq!(
            task.data
                .sources
                .iter()
                .map(|s| s.source_id.as_str())
                .collect::<Vec<_>>(),
            vec!["source-2", "source-1"]
        );
    }
    #[test]
    fn maximum_escaped_canonical_record_roundtrips_without_exceeding_line_bound() {
        let mut original = Connection::open_in_memory().unwrap();
        initialize(
            &mut original,
            &serde_json::to_string(&scope()).unwrap(),
            true,
        )
        .unwrap();
        let mut data = input(&"\u{1}".repeat(MAX_CONTENT_BYTES));
        data.sources = (0..8)
            .map(|ordinal| Source {
                source_id: "\u{1}".repeat(512),
                revision: "\u{1}".repeat(256),
                locator: "\u{1}".repeat(512),
                captured_at: ordinal,
            })
            .collect();
        data.applicability = Applicability {
            task_key: Some("\u{1}".repeat(256)),
            checkout_id: Some("\u{1}".repeat(256)),
            operation: Some("\u{1}".repeat(256)),
        };
        validate_input(&scope(), &data).unwrap();
        let tx = original.transaction().unwrap();
        tx.execute("INSERT INTO records VALUES('escaped',1)", [])
            .unwrap();
        insert_revision(&tx, "escaped", 1, &data, 100, None).unwrap();
        tx.commit().unwrap();

        let bytes = stream(&mut original);
        let longest_line = bytes
            .split_inclusive(|byte| *byte == b'\n')
            .map(<[u8]>::len)
            .max()
            .unwrap();
        assert!(
            longest_line > 128 * 1024,
            "regression must exceed the old limit"
        );
        assert!(longest_line <= MAX_LINE_BYTES);
        let mut target = Connection::open_in_memory().unwrap();
        import(&mut target, &scope(), bytes.as_slice()).unwrap();
        assert_eq!(stream(&mut target), bytes);
    }

    #[test]
    fn post_forget_links_and_retired_heads_roundtrip_without_resurrection() {
        for forgotten in ["a", "b"] {
            let mut c = fixture();
            // The same canonical deletion performed by the lifecycle owner;
            // external exclusion authority is deliberately outside this codec.
            c.execute("DELETE FROM record_fts WHERE rowid IN (SELECT row_id FROM revisions WHERE record_id=?1)",[forgotten]).unwrap();
            c.execute("DELETE FROM sources WHERE revision_row_id IN (SELECT row_id FROM revisions WHERE record_id=?1)",[forgotten]).unwrap();
            c.execute("DELETE FROM revisions WHERE record_id=?1", [forgotten])
                .unwrap();
            c.execute("DELETE FROM records WHERE record_id=?1", [forgotten])
                .unwrap();
            let bytes = stream(&mut c);
            let mut target = Connection::open_in_memory().unwrap();
            let stats = import(&mut target, &scope(), bytes.as_slice()).unwrap();
            assert_eq!(stats.dangling_supersedes, u64::from(forgotten == "a"));
            assert_eq!(
                stats.retired_heads_without_successor,
                u64::from(forgotten == "b")
            );
            assert_eq!(
                target
                    .query_row(
                        "SELECT count(*) FROM records WHERE record_id=?1",
                        [forgotten],
                        |r| r.get::<_, i64>(0)
                    )
                    .unwrap(),
                0
            );
            assert_eq!(stream(&mut target), bytes);
        }
    }
    #[test]
    fn canonical_record_roundtrip_rejects_unknown_authority_fields() {
        let value = values(&stream(&mut fixture()))[1]["record"].clone();
        let record: Record = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(record).unwrap(), value);
        let mut changed = value;
        changed["operation_id"] = json!("op-forged");
        assert!(serde_json::from_value::<Record>(changed).is_err());
    }
    #[test]
    fn strict_objects_reject_unknown_fields_and_operation_authority() {
        let base = values(&stream(&mut fixture()));
        for path in [
            vec!["record"],
            vec!["record", "data"],
            vec!["record", "data", "applicability"],
        ] {
            let mut v = base.clone();
            let mut object = &mut v[1];
            for key in path {
                object = &mut object[key];
            }
            object["operation_id"] = json!("op-forged");
            rejects(&encode(&v));
        }
        let mut v = base.clone();
        v[0]["scope"]["authority"] = json!(true);
        rejects(&encode(&v));
        let mut v = base.clone();
        v[1]["record"]["data"]["sources"][0]["sql"] = json!("DROP TABLE records");
        rejects(&encode(&v));
        let mut v = base;
        v.last_mut().unwrap()["last_restore_operation"] = json!("op-forged");
        rejects(&encode(&v));
    }
    #[test]
    fn malformed_truncated_duplicate_and_trailing_streams_are_rejected() {
        let bytes = stream(&mut fixture());
        let base = values(&bytes);
        rejects(&bytes[..bytes.len() - 1]);
        rejects(&encode(&base[..base.len() - 1]));
        rejects(b"not json\n");
        let mut v = base.clone();
        v.push(json!({"type":"footer","records":3,"revisions":4}));
        rejects(&encode(&v));
        let mut v = base.clone();
        v[2] = v[1].clone();
        rejects(&encode(&v));
        let mut v = base;
        v.last_mut().unwrap()["records"] = json!(100);
        rejects(&encode(&v));
        rejects(b"{\"type\":\"header\",\"version\":1,\"version\":1,\"scope\":{\"kind\":\"project\",\"project_id\":\"transfer-project\"}}\n");
    }
    #[test]
    fn invalid_history_scope_and_checkout_forms_are_rejected() {
        let base = values(&stream(&mut fixture()));
        for (line, key, value) in [
            (1, "head_revision", json!(3)),
            (2, "head_revision", json!(1)),
        ] {
            let mut v = base.clone();
            v[line][key] = value;
            rejects(&encode(&v));
        }
        let mut v = base.clone();
        v[1]["record"]["is_current_revision"] = json!(true);
        rejects(&encode(&v));
        let mut v = base.clone();
        v[2]["record"]["supersedes"]["revision"] = json!(99);
        rejects(&encode(&v));
        let mut v = base.clone();
        v[1]["record"]["superseded_by"] = Value::Null;
        rejects(&encode(&v));
        let mut v = base.clone();
        v[4]["record"]["data"]["applicability"]["checkout_id"] = Value::Null;
        rejects(&encode(&v));
        let mut v = base;
        v[0]["scope"]["project_id"] = json!("other-project");
        rejects(&encode(&v));
    }
    #[test]
    fn cycles_and_two_successors_are_rejected() {
        let base = values(&stream(&mut fixture()));
        let mut v = base.clone();
        v[1]["record"]["supersedes"] = json!({"record_id":"b","revision":1});
        v[3]["record"]["is_current_revision"] = json!(false);
        v[3]["record"]["superseded_by"] = json!({"record_id":"a","revision":1});
        rejects(&encode(&v));
        let mut v = base;
        v[4]["record"]["supersedes"] = json!({"record_id":"a","revision":2});
        rejects(&encode(&v));
    }
    #[test]
    fn resource_limits_and_nonempty_destination_fail_closed() {
        let bytes = stream(&mut fixture());
        let mut target = Connection::open_in_memory().unwrap();
        target.execute_batch("CREATE TABLE unrelated(x)").unwrap();
        assert!(import(&mut target, &scope(), bytes.as_slice()).is_err());
        rejects(&vec![b'x'; MAX_LINE_BYTES + 1]);
        let mut lines = Lines {
            reader: BufReader::new(b"{}\n".as_slice()),
            bytes: MAX_STREAM_BYTES - 1,
        };
        assert!(lines.next().is_err());
        let mut stats = TransferStats {
            bytes: MAX_STREAM_BYTES,
            ..TransferStats::default()
        };
        let mut output = Vec::new();
        assert!(
            write_line(
                &mut output,
                &Line::Footer {
                    records: 0,
                    revisions: 0
                },
                &mut stats
            )
            .is_err()
        );
        assert!(output.is_empty());
        assert_eq!(MAX_REVISIONS, 1_000_000);
    }
    #[test]
    fn text_is_data_and_caller_writer_failures_are_not_success() {
        let mut c = fixture();
        c.execute(
            "UPDATE revisions SET content=?1 WHERE record_id='b'",
            ["DROP TABLE records; -- harmless source text"],
        )
        .unwrap();
        let bytes = stream(&mut c);
        let mut target = Connection::open_in_memory().unwrap();
        import(&mut target, &scope(), bytes.as_slice()).unwrap();
        assert_eq!(
            target
                .query_row("SELECT count(*) FROM records", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            3
        );
        struct Fail;
        impl Write for Fail {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("fixture writer failure"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert!(export(&mut c, &scope(), Fail).is_err());
    }
    #[test]
    fn thousand_revision_chain_streams_with_indexed_graph_validation() {
        let mut c = Connection::open_in_memory().unwrap();
        initialize(&mut c, &serde_json::to_string(&scope()).unwrap(), true).unwrap();
        let tx = c.transaction().unwrap();
        tx.execute("INSERT INTO records VALUES('chain',1000)", [])
            .unwrap();
        for rev in 1..=1000 {
            let previous = (rev > 1).then(|| RecordRef {
                record_id: "chain".into(),
                revision: rev - 1,
            });
            if let Some(previous) = &previous {
                retire(&tx, previous).unwrap();
            }
            insert_revision(
                &tx,
                "chain",
                rev,
                &input("retained chain"),
                100 + rev,
                previous.as_ref(),
            )
            .unwrap();
        }
        tx.commit().unwrap();
        let bytes = stream(&mut c);
        let mut target = Connection::open_in_memory().unwrap();
        let stats = import(&mut target, &scope(), bytes.as_slice()).unwrap();
        assert_eq!((stats.records, stats.revisions), (1, 1000));
        assert_eq!(stream(&mut target), bytes);
    }
    #[test]
    fn watchdog_interrupts_native_sql_and_joins_without_waiting_full_deadline() {
        let c = Connection::open_in_memory().unwrap();
        let deadline = Deadline::with_duration(&c, Duration::from_millis(20)).unwrap();
        let result=c.query_row("WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<1000000000) SELECT sum(x) FROM n",[],|r|r.get::<_,i64>(0));
        assert!(
            matches!(result,Err(rusqlite::Error::SqliteFailure(ref e,_)) if e.code==rusqlite::ErrorCode::OperationInterrupted)
        );
        assert!(deadline.check().is_err());
        drop(deadline);
        let deadline = Deadline::new(&c).unwrap();
        drop(deadline); // Must notify/join immediately, not wait 120 seconds.
        assert_eq!(
            c.query_row("SELECT 1", [], |r| r.get::<_, i64>(0)).unwrap(),
            1
        );
    }
    #[test]
    fn empty_and_shared_preference_streams_preserve_scope_and_reject_task_selectors() {
        let scope = Scope::SharedPreferences;
        let mut c = Connection::open_in_memory().unwrap();
        initialize(&mut c, &serde_json::to_string(&scope).unwrap(), true).unwrap();
        let mut bytes = Vec::new();
        assert_eq!(export(&mut c, &scope, &mut bytes).unwrap().revisions, 0);
        let mut target = Connection::open_in_memory().unwrap();
        assert_eq!(
            import(&mut target, &scope, bytes.as_slice())
                .unwrap()
                .records,
            0
        );
        let tx = c.transaction().unwrap();
        let mut preference = input("explicit shared preference");
        preference.kind = RecordKind::Preference;
        tx.execute("INSERT INTO records VALUES('preference',1)", [])
            .unwrap();
        insert_revision(&tx, "preference", 1, &preference, 10, None).unwrap();
        tx.commit().unwrap();
        bytes.clear();
        export(&mut c, &scope, &mut bytes).unwrap();
        let mut target = Connection::open_in_memory().unwrap();
        import(&mut target, &scope, bytes.as_slice()).unwrap();
        let mut v = values(&bytes);
        v[1]["record"]["data"]["applicability"]["task_key"] = json!("project-task");
        let mut target = Connection::open_in_memory().unwrap();
        assert!(import(&mut target, &scope, encode(&v).as_slice()).is_err());
        let mut v = values(&bytes);
        v[0]["scope"]["project_id"] = json!("forged-project");
        let mut target = Connection::open_in_memory().unwrap();
        assert!(import(&mut target, &scope, encode(&v).as_slice()).is_err());
    }
}
