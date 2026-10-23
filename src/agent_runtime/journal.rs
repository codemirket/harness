//! Native execution journal: bounded replay, compare-and-swap revisions and no tool dispatch.
use crate::util;
use anyhow::{Context, Result, bail, ensure};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde_json::{Value, json};
use std::path::Path;
use std::time::Instant;

const APP: i64 = 0x4d52554e;
const SNAPSHOT_INTERVAL: u64 = 32;
const MAX_EVENTS: u64 = 2048;
const MAX_STATE: usize = 4 * 1024 * 1024;
const MAX_RUN_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_PAUSE: u64 = 86400;

pub fn effective_deadline(state: &Value, now: u64) -> Option<u64> {
    let deadline = state["deadline"].as_u64()?;
    let paused = state.get("paused_at").map_or(Some(0), Value::as_u64)?;
    let used = state.get("pause_used").map_or(Some(0), Value::as_u64)?;
    let allowance = MAX_PAUSE.checked_sub(used)?;
    let effective = if paused != 0 {
        let elapsed = now.checked_sub(paused)?;
        if elapsed > allowance || paused >= deadline {
            return None;
        }
        deadline.checked_add(elapsed)?
    } else {
        deadline
    };
    (effective > now).then_some(effective)
}

// Objects are patched recursively; arrays and primitives are atomic values.
// Explicit path operations distinguish deletion from a legitimate JSON null.
fn bounded_depth(value: &Value, depth: usize) -> bool {
    depth <= 60
        && match value {
            Value::Object(map) => map.values().all(|v| bounded_depth(v, depth + 1)),
            Value::Array(list) => list.iter().all(|v| bounded_depth(v, depth + 1)),
            _ => true,
        }
}

fn difference(before: &Value, after: &Value, path: &mut Vec<String>, ops: &mut Vec<Value>) {
    if before == after {
        return;
    }
    if let (Some(a), Some(b)) = (before.as_object(), after.as_object()) {
        for key in a.keys().filter(|k| !b.contains_key(*k)) {
            path.push(key.clone());
            ops.push(json!({"path":path,"remove":true}));
            path.pop();
        }
        for (key, value) in b {
            path.push(key.clone());
            if let Some(old) = a.get(key) {
                difference(old, value, path, ops);
            } else {
                ops.push(json!({"path":path,"value":value}));
            }
            path.pop();
        }
    } else {
        ops.push(json!({"path":path,"value":after}));
    }
}

fn apply(state: &mut Value, operations: &Value) -> Result<()> {
    for op in operations.as_array().context("invalid journal delta")? {
        let path = op["path"].as_array().context("invalid delta path")?;
        ensure!(path.len() <= 64, "delta path exceeds bound");
        if path.is_empty() {
            *state = op.get("value").context("missing replacement")?.clone();
            continue;
        }
        let mut target = &mut *state;
        for key in &path[..path.len() - 1] {
            target = target
                .get_mut(key.as_str().context("invalid delta key")?)
                .context("missing delta parent")?;
        }
        let key = path.last().unwrap().as_str().context("invalid delta key")?;
        let object = target
            .as_object_mut()
            .context("delta parent is not an object")?;
        if op["remove"] == true {
            ensure!(object.remove(key).is_some(), "missing removed field");
        } else {
            object.insert(
                key.to_owned(),
                op.get("value").context("missing delta value")?.clone(),
            );
        }
    }
    Ok(())
}

pub struct Journal {
    connection: Connection,
}
impl Journal {
    pub fn open(path: &Path, scope: &str) -> Result<Self> {
        ensure!(
            scope.len() == 64 && scope.bytes().all(|c| c.is_ascii_hexdigit()),
            "invalid journal scope"
        );
        let connection = super::database::open(path, APP, |tx| {
            tx.execute_batch("CREATE TABLE identity(scope TEXT PRIMARY KEY, revision INTEGER NOT NULL DEFAULT 0);
                CREATE TABLE runs(id TEXT PRIMARY KEY, revision INTEGER NOT NULL, bytes INTEGER NOT NULL);
                CREATE TABLE events(run TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                    revision INTEGER NOT NULL, key TEXT NOT NULL, fingerprint TEXT NOT NULL,
                    snapshot INTEGER NOT NULL, body TEXT NOT NULL, digest TEXT NOT NULL,
                    PRIMARY KEY(run,revision), UNIQUE(run,key));")?;
            tx.execute("INSERT INTO identity(scope) VALUES(?1)", [scope])?;
            Ok(())
        })?;
        ensure!(
            connection.query_row("SELECT count(*) FROM identity", [], |r| r.get::<_, u64>(0))? == 1,
            "journal requires exactly one identity row"
        );
        let stored: String =
            connection.query_row("SELECT scope FROM identity", [], |r| r.get(0))?;
        ensure!(stored == scope, "execution project mismatch");
        Ok(Self { connection })
    }

    /// Validate a copied journal without opening it through the mutating
    /// runtime owner. This is deliberately read-only: backup inspection must
    /// establish logical replay and identity integrity in addition to SQLite's
    /// page-level checks before the file is eligible for restore.
    pub(crate) fn validate_backup(connection: &Connection, scope: &str) -> Result<()> {
        ensure!(
            scope.len() == 64 && scope.bytes().all(|c| c.is_ascii_hexdigit()),
            "invalid journal scope"
        );
        ensure!(
            connection.query_row("SELECT count(*) FROM identity", [], |r| r.get::<_, u64>(0))? == 1,
            "journal requires exactly one identity row"
        );
        let (stored_scope, identity_revision): (String, u64) =
            connection.query_row("SELECT scope,revision FROM identity", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?;
        ensure!(stored_scope == scope, "execution project mismatch");
        let orphan_events: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM events e LEFT JOIN runs r ON r.id=e.run WHERE r.id IS NULL)",
            [],
            |row| row.get(0),
        )?;
        ensure!(!orphan_events, "journal contains orphan events");

        let run_records: Vec<(String, u64, u64)> = {
            let mut runs = connection.prepare("SELECT id,revision,bytes FROM runs ORDER BY id")?;
            let mut run_rows = runs.query([])?;
            let mut records = Vec::new();
            while let Some(run_row) = run_rows.next()? {
                records.push((run_row.get(0)?, run_row.get(1)?, run_row.get(2)?));
            }
            records
        };
        let mut largest_event_revision = 0u64;
        for (id, revision, stored_bytes) in run_records {
            ensure!(
                !id.is_empty()
                    && id.len() <= 64
                    && id
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
                "invalid journal run id"
            );
            ensure!(
                (1..=MAX_EVENTS).contains(&revision),
                "journal run revision exceeds bound"
            );
            ensure!(
                stored_bytes <= MAX_RUN_BYTES,
                "journal run exceeds byte bound"
            );

            let mut events = connection.prepare(
                "SELECT revision,snapshot,body,digest FROM events WHERE run=?1 ORDER BY revision",
            )?;
            let mut event_rows = events.query([&id])?;
            let mut expected_revision = 1u64;
            let mut total_bytes = 0u64;
            let mut state = Value::Null;
            let mut saw_event = false;
            while let Some(event_row) = event_rows.next()? {
                saw_event = true;
                let event_revision: u64 = event_row.get(0)?;
                let snapshot: bool = event_row.get(1)?;
                let body: String = event_row.get(2)?;
                let digest: String = event_row.get(3)?;
                ensure!(
                    event_revision == expected_revision,
                    "journal event sequence gap"
                );
                ensure!(body.len() <= MAX_STATE, "journal record exceeds bound");
                ensure!(
                    util::sha256(body.as_bytes()) == digest,
                    "journal record digest mismatch"
                );
                total_bytes = total_bytes
                    .checked_add(body.len() as u64)
                    .context("journal byte count overflow")?;
                ensure!(
                    total_bytes <= MAX_RUN_BYTES,
                    "journal run exceeds byte bound"
                );
                let value: Value = serde_json::from_str(&body)?;
                ensure!(
                    snapshot
                        == (event_revision == 1
                            || event_revision.is_multiple_of(SNAPSHOT_INTERVAL)),
                    "journal snapshot schedule mismatch"
                );
                if snapshot {
                    ensure!(value.is_object(), "journal snapshot must be an object");
                    state = value;
                } else {
                    apply(&mut state, &value)?;
                }
                ensure!(
                    bounded_depth(&state, 0) && serde_json::to_vec(&state)?.len() <= MAX_STATE,
                    "reconstructed journal state exceeds bound"
                );
                ensure!(
                    state["envelope"]["checkpoint_format_version"] == 1,
                    "unsupported journal checkpoint format"
                );
                ensure!(
                    matches!(
                        state["decision"].as_str(),
                        Some(
                            "planned"
                                | "continue"
                                | "running"
                                | "effect_prepared"
                                | "effect"
                                | "effect_uncertain"
                                | "done"
                                | "pending"
                                | "verified"
                                | "blocked"
                                | "cancelled"
                                | "pause"
                        )
                    ),
                    "invalid journal workflow phase"
                );
                expected_revision += 1;
            }
            ensure!(saw_event, "journal run has no events");
            ensure!(
                expected_revision == revision + 1,
                "journal run revision does not match events"
            );
            ensure!(
                total_bytes == stored_bytes,
                "journal stored byte count does not match events"
            );
            ensure!(
                state["envelope"]["checkpoint_format_version"] == 1,
                "unsupported journal checkpoint format"
            );
            ensure!(
                matches!(
                    state["decision"].as_str(),
                    Some(
                        "planned"
                            | "continue"
                            | "running"
                            | "effect_prepared"
                            | "effect"
                            | "effect_uncertain"
                            | "done"
                            | "pending"
                            | "verified"
                            | "blocked"
                            | "cancelled"
                            | "pause"
                    )
                ),
                "invalid journal workflow phase"
            );
            largest_event_revision = largest_event_revision.max(revision);

            // Re-run the bounded production replay from the latest snapshot;
            // this also checks its persisted head agrees with the full replay.
            let (loaded_revision, loaded_state) =
                Self::load(connection, &id)?.context("journal run disappeared")?;
            ensure!(
                loaded_revision == revision && loaded_state == state,
                "journal replay head mismatch"
            );
        }
        ensure!(
            identity_revision >= largest_event_revision,
            "journal identity revision regressed"
        );
        Ok(())
    }

    fn load(connection: &Connection, id: &str) -> Result<Option<(u64, Value)>> {
        let revision: Option<u64> = connection
            .query_row("SELECT revision FROM runs WHERE id=?1", [id], |r| r.get(0))
            .optional()?;
        let Some(revision) = revision else {
            return Ok(None);
        };
        let start: u64 = connection.query_row(
            "SELECT max(revision) FROM events WHERE run=?1 AND snapshot=1",
            [id],
            |r| r.get(0),
        )?;
        ensure!(
            revision >= start && revision - start < SNAPSHOT_INTERVAL,
            "journal replay bound exceeded"
        );
        let mut stmt = connection.prepare("SELECT revision,snapshot,body,digest FROM events WHERE run=?1 AND revision>=?2 ORDER BY revision")?;
        let mut rows = stmt.query(params![id, start])?;
        let mut expected = start;
        let mut state = Value::Null;
        while let Some(row) = rows.next()? {
            ensure!(row.get::<_, u64>(0)? == expected, "journal sequence gap");
            let body: String = row.get(2)?;
            ensure!(body.len() <= MAX_STATE, "journal record exceeds bound");
            ensure!(
                util::sha256(body.as_bytes()) == row.get::<_, String>(3)?,
                "journal record digest mismatch"
            );
            let value: Value = serde_json::from_str(&body)?;
            if row.get::<_, bool>(1)? {
                state = value;
            } else {
                apply(&mut state, &value)?;
            }
            ensure!(
                serde_json::to_vec(&state)?.len() <= MAX_STATE,
                "reconstructed state exceeds bound"
            );
            expected += 1;
        }
        ensure!(expected == revision + 1, "journal head missing");
        Ok(Some((revision, state)))
    }

    fn describe(revision: u64, state: Value) -> Value {
        let decision = state["decision"].as_str().unwrap_or("continue");
        let (phase, boundary) = match decision {
            "planned" => ("planned", Some("worker_turn")),
            "effect_prepared" => ("effect_prepared", Some("effect_dispatch")),
            "effect" | "effect_uncertain" => ("effect_uncertain", Some("effect")),
            "done" | "pending" => ("awaiting_verification", Some("host_verification")),
            "verified" => ("completed", None),
            "blocked" => ("failed", None),
            "cancelled" => ("cancelled", None),
            "pause" => ("paused", Some("worker_turn")),
            "continue" | "running" => ("running", Some("worker_turn")),
            _ => ("running", Some("worker_turn")),
        };
        let interrupts: Vec<_> = boundary
            .into_iter()
            .map(|kind| json!({"kind":kind,"turn":state["turn"],"effect":state["effect"]}))
            .collect();
        json!({"checkpoint_id":revision.to_string(),"phase":phase,"next":boundary.into_iter().collect::<Vec<_>>(),"interrupts":interrupts,"state":state,"engine":"rust_journal",
            "assurance":"A persisted effect intent does not prove dispatch or completion. Inspect before replay."})
    }

    fn diagnostics(
        connection: &Connection,
        id: &str,
        revision: u64,
        state: Value,
        appended_bytes: u64,
        replayed_request_revision: Option<u64>,
        started: Instant,
    ) -> Result<Value> {
        let run_stored_bytes: u64 =
            connection.query_row("SELECT bytes FROM runs WHERE id=?1", [id], |row| row.get(0))?;
        let mut result = Self::describe(revision, state);
        result["diagnostics"] = json!({
            "checkpoint_elapsed_us": started.elapsed().as_micros().min(u64::MAX as u128) as u64,
            "checkpoint_serialized_bytes_appended": appended_bytes,
            "serialization_elapsed_us": Value::Null,
            "run_stored_bytes": run_stored_bytes,
            "replayed_request_revision": replayed_request_revision,
            "current_revision": revision,
        });
        Ok(result)
    }

    /// Memories are resolved before start, outside SQLite. This owner never runs a tool or model.
    pub fn execute(
        &mut self,
        operation: &str,
        args: &Value,
        now: u64,
        memories: Value,
    ) -> Result<Value> {
        let started = Instant::now();
        ensure!(
            bounded_depth(args, 0) && bounded_depth(&memories, 0),
            "workflow JSON nesting exceeds bound"
        );
        let id = args["run_id"].as_str().context("run ID required")?;
        ensure!(
            !id.is_empty()
                && id.len() <= 64
                && id
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
            "invalid run ID"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = Self::load(&tx, id)?;
        if operation == "workflow_status" {
            let (revision, state) = existing.context("unknown workflow run")?;
            return Self::diagnostics(&tx, id, revision, state, 0, None, started);
        }
        if operation == "workflow_forget" {
            if let Some((revision, _)) = existing {
                ensure!(
                    args["expected_checkpoint_id"].as_str() == Some(revision.to_string().as_str()),
                    "workflow changed; inspect before forgetting"
                );
                tx.execute("DELETE FROM runs WHERE id=?1", [id])?;
                tx.execute("UPDATE identity SET revision=revision+1", [])?;
            }
            tx.commit()?;
            return Ok(json!({"deleted":true,"run_id":id,"trace_retained":true}));
        }
        let (revision, previous, mut state, key, fingerprint) = if operation == "workflow_start" {
            if let Some((revision, state)) = existing {
                let fingerprint = util::sha256(&serde_json::to_vec(args)?);
                let recorded: String = tx.query_row(
                    "SELECT fingerprint FROM events WHERE run=?1 AND key='start'",
                    [id],
                    |row| row.get(0),
                )?;
                ensure!(
                    recorded == fingerprint,
                    "workflow start retry changed input"
                );
                ensure!(
                    state["envelope"] == args["envelope"],
                    "workflow ID belongs to different input"
                );
                return Self::diagnostics(&tx, id, revision, state, 0, Some(1), started);
            }
            let deadline = args["deadline"].as_u64().context("deadline required")?;
            ensure!(
                deadline > now && deadline - now <= 7200,
                "workflow requires a future deadline within two hours"
            );
            ensure!(
                args["envelope"].is_object() && args["snapshot"].is_object(),
                "invalid initial state"
            );
            ensure!(
                args["envelope"]["checkpoint_format_version"] == 1,
                "unsupported checkpoint format"
            );
            let state = json!({"envelope":args["envelope"],"snapshot":args["snapshot"],"memories":memories,"turn":0,"deadline":deadline,"decision":"planned","pause_used":0,"paused_at":0,"last_clock":now});
            (
                1,
                Value::Null,
                state,
                "start".to_owned(),
                util::sha256(&serde_json::to_vec(args)?),
            )
        } else {
            let (revision, state) = existing.context("unknown workflow run")?;
            let (key, fingerprint) = if operation == "workflow_resume" {
                let key = args["key"].as_str().context("retry key required")?;
                ensure!(
                    !key.is_empty() && key.len() <= 128 && key != "start",
                    "invalid retry key"
                );
                (
                    key.to_owned(),
                    util::sha256(&serde_json::to_vec(&args["reply"])?),
                )
            } else if operation == "workflow_hostcheck" {
                let fingerprint = util::sha256(&serde_json::to_vec(&args["verification"])?);
                (format!("hostcheck:{fingerprint}"), fingerprint)
            } else {
                bail!("unknown workflow operation");
            };
            let prior: Option<(u64, String)> = tx
                .query_row(
                    "SELECT revision,fingerprint FROM events WHERE run=?1 AND key=?2",
                    params![id, key],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            if let Some((prior_revision, prior)) = prior {
                ensure!(
                    prior == fingerprint,
                    "workflow retry key reused with different input"
                );
                return Self::diagnostics(
                    &tx,
                    id,
                    revision,
                    state,
                    0,
                    Some(prior_revision),
                    started,
                );
            }
            ensure!(
                args["expected_checkpoint_id"].as_str() == Some(revision.to_string().as_str()),
                "workflow changed; inspect current checkpoint"
            );
            ensure!(
                now >= state["last_clock"].as_u64().context("missing clock")?,
                "clock moved backward; inspect workflow"
            );
            let mut next = state.clone();
            if operation == "workflow_hostcheck" {
                ensure!(
                    state["envelope"]["host_verification_required"] == true
                        && matches!(
                            state["decision"].as_str(),
                            Some("done" | "pending" | "verified")
                        ),
                    "workflow is not at host verification"
                );
                ensure!(args["verification"].is_object(), "invalid verification");
                next["verification"] = args["verification"].clone();
                next["decision"] = json!(if args["verification"]["status"] == "host_verified" {
                    "verified"
                } else {
                    "pending"
                });
            } else {
                let current = Self::describe(revision, state.clone());
                let boundary = current["interrupts"][0]["kind"]
                    .as_str()
                    .context("workflow has no pending boundary")?;
                let reply = &args["reply"];
                let decision = reply["decision"]
                    .as_str()
                    .context("transition decision required")?;
                let allowed: &[&str] = match boundary {
                    "worker_turn" => &[
                        "continue",
                        "effect_prepared",
                        "done",
                        "blocked",
                        "pause",
                        "cancelled",
                    ],
                    "effect_dispatch" => &["effect_uncertain", "blocked", "cancelled"],
                    "effect" => &["continue", "blocked", "cancelled"],
                    _ => &[], // Only the validated host-check path can record verification.
                };
                ensure!(allowed.contains(&decision), "invalid workflow transition");
                let cancellation = decision == "cancelled";
                let deadline = if cancellation {
                    state["deadline"].as_u64().context("missing deadline")?
                } else {
                    effective_deadline(&state, now).context("original workflow budget expired")?
                };
                let paused = state["paused_at"]
                    .as_u64()
                    .context("pause state required")?;
                let used = state["pause_used"]
                    .as_u64()
                    .context("pause budget required")?;
                ensure!(
                    reply["snapshot"].is_object(),
                    "workflow snapshot must be an object"
                );
                let turn = reply["snapshot"]["turn"]
                    .as_u64()
                    .context("turn required")?;
                ensure!(
                    turn >= state["turn"].as_u64().context("prior turn required")? && turn <= 256,
                    "turn budget reset or exhausted"
                );
                if decision == "pause" {
                    ensure!(paused == 0, "already paused");
                }
                if decision == "effect_prepared" {
                    ensure!(
                        reply["effect"].is_object() && reply["effect"]["tool"].is_string(),
                        "effect intent required"
                    );
                }
                next["snapshot"] = reply["snapshot"].clone();
                next["snapshot"]["deadline"] = json!(deadline);
                next["effect"] = reply["effect"].clone();
                next["turn"] = json!(turn);
                next["deadline"] = json!(deadline);
                next["paused_at"] = json!(if decision == "pause" { now } else { 0 });
                next["pause_used"] = json!(if cancellation {
                    used
                } else {
                    used + if paused == 0 { 0 } else { now - paused }
                });
                next["decision"] = json!(if decision == "done"
                    && state["envelope"]["host_verification_required"] != true
                {
                    "verified"
                } else {
                    decision
                });
            }
            (revision + 1, state, next, key, fingerprint)
        };
        let recovery = matches!(state["decision"].as_str(), Some("cancelled" | "blocked"))
            || matches!(
                args["reply"]["decision"].as_str(),
                Some("cancelled" | "blocked")
            );
        ensure!(
            revision <= if recovery { MAX_EVENTS } else { MAX_EVENTS - 2 },
            "workflow event budget exhausted; cancellation capacity reserved"
        );
        ensure!(
            bounded_depth(&state, 0),
            "workflow state nesting exceeds bound"
        );
        state["last_clock"] = json!(now);
        let serialization_started = Instant::now();
        let full = serde_json::to_string(&state)?;
        ensure!(full.len() <= MAX_STATE, "workflow state exceeds bound");
        let snapshot = revision == 1 || revision % SNAPSHOT_INTERVAL == 0;
        let body = if snapshot {
            full
        } else {
            let mut ops = Vec::new();
            difference(&previous, &state, &mut Vec::new(), &mut ops);
            serde_json::to_string(&ops)?
        };
        ensure!(body.len() <= MAX_STATE, "workflow delta exceeds bound");
        let serialization_us = serialization_started
            .elapsed()
            .as_micros()
            .min(u64::MAX as u128) as u64;
        let appended_bytes = body.len() as u64;
        if revision == 1 {
            tx.execute("INSERT INTO runs VALUES(?1,0,0)", [id])?;
        }
        let bytes: u64 = tx.query_row("SELECT bytes FROM runs WHERE id=?1", [id], |r| r.get(0))?;
        ensure!(
            bytes + body.len() as u64
                <= if recovery {
                    MAX_RUN_BYTES
                } else {
                    MAX_RUN_BYTES - 2 * MAX_STATE as u64
                },
            "workflow journal quota exhausted"
        );
        tx.execute(
            "INSERT INTO events VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                id,
                revision,
                key,
                fingerprint,
                snapshot,
                body,
                util::sha256(body.as_bytes())
            ],
        )?;
        tx.execute(
            "UPDATE runs SET revision=?2,bytes=?3 WHERE id=?1",
            params![id, revision, bytes + body.len() as u64],
        )?;
        tx.execute("UPDATE identity SET revision=revision+1", [])?;
        tx.commit()?;
        let mut result = Self::diagnostics(
            &self.connection,
            id,
            revision,
            state,
            appended_bytes,
            None,
            started,
        )?;
        result["diagnostics"]["serialization_elapsed_us"] = json!(serialization_us);
        Ok(result)
    }

    pub fn revision(&self) -> Result<u64> {
        Ok(self
            .connection
            .query_row("SELECT revision FROM identity", [], |r| r.get(0))?)
    }
    pub fn count(&self) -> Result<u64> {
        Ok(self
            .connection
            .query_row("SELECT count(*) FROM runs", [], |r| r.get(0))?)
    }
    pub fn get(&self, id: &str) -> Result<Option<Value>> {
        let started = Instant::now();
        Self::load(&self.connection, id)?
            .map(|(rev, state)| {
                Self::diagnostics(&self.connection, id, rev, state, 0, None, started)
            })
            .transpose()
    }
    pub fn next(&self, after: &str) -> Result<Option<(String, Value)>> {
        let id: Option<String> = self
            .connection
            .query_row(
                "SELECT id FROM runs WHERE id>?1 ORDER BY id LIMIT 1",
                [after],
                |r| r.get(0),
            )
            .optional()?;
        id.map(|id| {
            let value = self.get(&id)?.context("journal head disappeared")?;
            Ok((id, value))
        })
        .transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clamp_max_page_count(connection: &Connection) {
        connection
            .pragma_update(None, "wal_autocheckpoint", 1_i64)
            .unwrap();
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .unwrap();
        let pages: i64 = connection
            .pragma_query_value(None, "page_count", |row| row.get(0))
            .unwrap();
        connection
            .pragma_update(None, "max_page_count", pages)
            .unwrap();
    }

    fn restore_max_page_count(connection: &Connection) {
        connection
            .pragma_update(None, "max_page_count", 131072_i64)
            .unwrap();
    }
    #[test]
    fn multiple_identity_namespaces_are_rejected_without_adoption() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
        let scope = "a".repeat(64);
        drop(Journal::open(&path, &scope).unwrap());
        let connection = Connection::open(&path).unwrap();
        connection
            .execute("INSERT INTO identity(scope) VALUES(?1)", ["b".repeat(64)])
            .unwrap();
        assert!(Journal::validate_backup(&connection, &scope).is_err());
        drop(connection);
        assert!(Journal::open(&path, &scope).is_err());
    }

    #[test]
    fn sqlite_full_transition_does_not_advance_revision_and_retries_atomically() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
        let scope = "a".repeat(64);
        let mut journal = Journal::open(&path, &scope).unwrap();
        let checkpoint = journal
            .execute(
                "workflow_start",
                &json!({
                    "run_id":"full-run",
                    "envelope":{"checkpoint_format_version":1},
                    "snapshot":{"turn":0},
                    "deadline":5000
                }),
                100,
                Value::Null,
            )
            .unwrap();
        let before_revision = journal.revision().unwrap();
        let transition = json!({
            "run_id":"full-run",
            "expected_checkpoint_id":checkpoint["checkpoint_id"],
            "key":"sqlite-full-retry",
            "reply":{
                "decision":"continue",
                "snapshot":{"turn":1,"padding":"z".repeat(3_500_000)}
            }
        });
        journal
            .connection
            .pragma_update(None, "journal_mode", "DELETE")
            .unwrap();
        clamp_max_page_count(&journal.connection);
        assert!(
            journal
                .execute("workflow_resume", &transition, 101, Value::Null)
                .is_err()
        );
        assert_eq!(journal.revision().unwrap(), before_revision);
        let unchanged = journal
            .execute(
                "workflow_status",
                &json!({"run_id":"full-run"}),
                102,
                Value::Null,
            )
            .unwrap();
        assert_eq!(unchanged["checkpoint_id"], checkpoint["checkpoint_id"]);
        restore_max_page_count(&journal.connection);
        journal
            .connection
            .pragma_update(None, "journal_mode", "WAL")
            .unwrap();
        let retried = journal
            .execute("workflow_resume", &transition, 101, Value::Null)
            .unwrap();
        assert_eq!(
            retried["checkpoint_id"],
            json!((before_revision + 1).to_string())
        );
        assert_eq!(journal.revision().unwrap(), before_revision + 1);
    }

    #[test]
    fn backup_validation_replays_all_events_and_rejects_pre_snapshot_corruption() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
        let scope = "a".repeat(64);
        let mut journal = Journal::open(&path, &scope).unwrap();
        let mut current = journal
            .execute(
                "workflow_start",
                &json!({
                    "run_id":"backup-run",
                    "envelope":{"checkpoint_format_version":1},
                    "snapshot":{"turn":0},
                    "deadline":5000
                }),
                100,
                Value::Null,
            )
            .unwrap();
        for turn in 1..=40 {
            current = journal
                .execute(
                    "workflow_resume",
                    &json!({
                        "run_id":"backup-run",
                        "expected_checkpoint_id":current["checkpoint_id"],
                        "key":format!("turn-{turn}"),
                        "reply":{"decision":"continue","snapshot":{"turn":turn},"effect":{}}
                    }),
                    100 + turn,
                    Value::Null,
                )
                .unwrap();
        }
        assert_eq!(current["checkpoint_id"], "41");
        drop(journal);

        let connection = Connection::open(&path).unwrap();
        Journal::validate_backup(&connection, &scope).unwrap();
        connection
            .execute("UPDATE runs SET bytes=bytes+1 WHERE id='backup-run'", [])
            .unwrap();
        assert!(Journal::validate_backup(&connection, &scope).is_err());
        connection
            .execute("UPDATE runs SET bytes=bytes-1 WHERE id='backup-run'", [])
            .unwrap();
        connection
            .execute(
                "UPDATE events SET body='{}' WHERE run='backup-run' AND revision=2",
                [],
            )
            .unwrap();
        assert!(Journal::validate_backup(&connection, &scope).is_err());
    }
}
