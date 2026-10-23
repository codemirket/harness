use mirket::agent_runtime::journal::Journal;
use serde_json::{Value, json};

fn start(journal: &mut Journal) -> Value {
    let planned = journal.execute("workflow_start", &json!({"run_id":"run-one", "envelope":{"checkpoint_format_version":1,"host_verification_required":true,"provider":"fixed"},"snapshot":{"turn":0,"assignment":"immutable guidance"},"deadline":500}),100,json!({"results":[]})).unwrap();
    journal.execute("workflow_resume", &json!({"run_id":"run-one","expected_checkpoint_id":planned["checkpoint_id"],"key":"activate","reply":{"decision":"continue","snapshot":{"turn":0},"effect":{}}}),101,Value::Null).unwrap()
}
fn advance(
    journal: &mut Journal,
    observed: &Value,
    key: &str,
    decision: &str,
    now: u64,
) -> anyhow::Result<Value> {
    if decision == "effect" {
        let prepared = journal.execute("workflow_resume",&json!({"run_id":"run-one","expected_checkpoint_id":observed["checkpoint_id"],"key":format!("{key}:prepared"),"reply":{"decision":"effect_prepared","snapshot":{"turn":1},"effect":{"tool":"remote_send","arguments":{"text":"synthetic"}}}}),now,Value::Null)?;
        return journal.execute("workflow_resume",&json!({"run_id":"run-one","expected_checkpoint_id":prepared["checkpoint_id"],"key":key,"reply":{"decision":"effect_uncertain","snapshot":{"turn":1},"effect":{"tool":"remote_send","outcome":"unknown"}}}),now,Value::Null);
    }
    journal.execute("workflow_resume",&json!({"run_id":"run-one","expected_checkpoint_id":observed["checkpoint_id"],"key":key,"reply":{"decision":decision,"snapshot":{"turn":1},"effect":{"tool":"remote_send","arguments":{"text":"synthetic"}}}}),now,Value::Null)
}

#[test]
fn restart_cas_idempotency_and_uncertain_effect_never_reset_state() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
    let scope = "a".repeat(64);
    let mut journal = Journal::open(&path, &scope).unwrap();
    let initial = start(&mut journal);
    assert!(initial["diagnostics"]["serialization_elapsed_us"].is_u64());
    let pending = advance(&mut journal, &initial, "intent", "effect", 101).unwrap();
    assert_eq!(pending["phase"], "effect_uncertain");
    drop(journal);
    let mut journal = Journal::open(&path, &scope).unwrap();
    let observed = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"run-one"}),
            102,
            Value::Null,
        )
        .unwrap();
    assert_eq!(observed["phase"], pending["phase"]);
    assert_eq!(observed["state"], pending["state"]);
    assert_eq!(
        observed["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );
    assert_eq!(
        observed["diagnostics"]["run_stored_bytes"],
        pending["diagnostics"]["run_stored_bytes"]
    );
    let replay = advance(&mut journal, &initial, "intent", "effect", 102).unwrap();
    assert_eq!(replay["state"], pending["state"]);
    assert_eq!(replay["diagnostics"]["replayed_request_revision"], 4);
    assert_eq!(
        replay["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );
    assert!(advance(&mut journal, &initial, "intent", "continue", 102).is_err());
    assert!(advance(&mut journal, &initial, "stale", "continue", 102).is_err());
    assert!(advance(&mut journal, &pending, "invalid", "pause", 102).is_err());
    let final_status = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"run-one"}),
            102,
            Value::Null,
        )
        .unwrap();
    assert_eq!(final_status["state"], pending["state"]);
    assert_eq!(
        final_status["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );
    assert!(Journal::open(&path, &"b".repeat(64)).is_err());
}

#[test]
fn planned_prepared_uncertain_boundaries_and_late_cancellation_are_explicit() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
    let scope = "a".repeat(64);
    let mut journal = Journal::open(&path, &scope).unwrap();
    let planned = journal.execute("workflow_start", &json!({"run_id":"phase-run", "envelope":{"checkpoint_format_version":1},"snapshot":{"turn":0},"deadline":500}),100,Value::Null).unwrap();
    assert_eq!(planned["phase"], "planned");
    let running = journal.execute("workflow_resume", &json!({"run_id":"phase-run","expected_checkpoint_id":planned["checkpoint_id"],"key":"activate","reply":{"decision":"continue","snapshot":{"turn":0},"effect":{}}}),101,Value::Null).unwrap();
    assert_eq!(running["phase"], "running");
    let prepared = journal.execute("workflow_resume", &json!({"run_id":"phase-run","expected_checkpoint_id":running["checkpoint_id"],"key":"prepare","reply":{"decision":"effect_prepared","snapshot":{"turn":1},"effect":{"tool":"remote_send","arguments":{"text":"synthetic"}}}}),102,Value::Null).unwrap();
    assert_eq!(prepared["phase"], "effect_prepared");
    let uncertain = journal.execute("workflow_resume", &json!({"run_id":"phase-run","expected_checkpoint_id":prepared["checkpoint_id"],"key":"uncertain","reply":{"decision":"effect_uncertain","snapshot":{"turn":1},"effect":{"tool":"remote_send","outcome":"unknown"}}}),103,Value::Null).unwrap();
    assert_eq!(uncertain["phase"], "effect_uncertain");
    let cancelled = journal.execute("workflow_resume", &json!({"run_id":"phase-run","expected_checkpoint_id":uncertain["checkpoint_id"],"key":"cancel-late","reply":{"decision":"cancelled","snapshot":{"turn":1},"effect":{}}}),600,Value::Null).unwrap();
    assert_eq!(cancelled["phase"], "cancelled");
    assert_eq!(cancelled["state"]["deadline"], 500);
}

#[test]
fn observed_effect_returns_to_worker_turn_and_failed_dispatch_stays_terminal() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
    let scope = "a".repeat(64);
    let mut journal = Journal::open(&path, &scope).unwrap();
    let planned = journal
        .execute(
            "workflow_start",
            &json!({"run_id":"observed-roundtrip", "envelope":{"checkpoint_format_version":1}, "snapshot":{"turn":0}, "deadline":500}),
            100,
            Value::Null,
        )
        .unwrap();
    let running = journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":planned["checkpoint_id"],"key":"activate","reply":{"decision":"continue","snapshot":{"turn":0},"effect":{}}}),
            101,
            Value::Null,
        )
        .unwrap();
    let prepared = journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":running["checkpoint_id"],"key":"prepare","reply":{"decision":"effect_prepared","snapshot":{"turn":1},"effect":{"tool":"workspace_read","arguments":{"path":"ready.txt"}}}}),
            102,
            Value::Null,
        )
        .unwrap();
    let uncertain = journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":prepared["checkpoint_id"],"key":"dispatch","reply":{"decision":"effect_uncertain","snapshot":{"turn":1},"effect":{"tool":"workspace_read","dispatch":"started","outcome":"unknown"}}}),
            103,
            Value::Null,
        )
        .unwrap();
    let observed = journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":uncertain["checkpoint_id"],"key":"observed","reply":{"decision":"continue","snapshot":{"turn":1,"transcript":[{"tool":"workspace_read","passed":true,"elapsed_ms":4}]},"effect":{"tool":"workspace_read","outcome":"observed"}}}),
            104,
            Value::Null,
        )
        .unwrap();
    assert_eq!(observed["phase"], "running");
    assert_eq!(observed["state"]["effect"]["outcome"], "observed");
    assert_eq!(
        observed["state"]["snapshot"]["transcript"][0]["elapsed_ms"],
        4
    );

    let failed = journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":observed["checkpoint_id"],"key":"prepare-failed","reply":{"decision":"effect_prepared","snapshot":{"turn":2},"effect":{"tool":"workspace_read","arguments":{"path":"missing.txt"}}}}),
            105,
            Value::Null,
        )
        .unwrap();
    let failed_uncertain = journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":failed["checkpoint_id"],"key":"dispatch-failed","reply":{"decision":"effect_uncertain","snapshot":{"turn":2},"effect":{"tool":"workspace_read","dispatch":"started","outcome":"unknown"}}}),
            106,
            Value::Null,
        )
        .unwrap();
    let terminal = journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":failed_uncertain["checkpoint_id"],"key":"blocked-failed","reply":{"decision":"blocked","snapshot":{"turn":2,"transcript":[{"tool":"workspace_read","error":"missing.txt","elapsed_ms":2}]},"effect":{"tool":"workspace_read","dispatch":"started","outcome":"unknown","error":"missing.txt"}}}),
            107,
            Value::Null,
        )
        .unwrap();
    assert_eq!(terminal["phase"], "failed");
    assert_eq!(terminal["state"]["effect"]["outcome"], "unknown");
    assert_eq!(terminal["state"]["effect"]["dispatch"], "started");
    assert_eq!(
        terminal["state"]["snapshot"]["transcript"][0]["elapsed_ms"],
        2
    );
    assert!(journal
        .execute(
            "workflow_resume",
            &json!({"run_id":"observed-roundtrip","expected_checkpoint_id":terminal["checkpoint_id"],"key":"replay","reply":{"decision":"continue","snapshot":{"turn":3},"effect":{}}}),
            108,
            Value::Null,
        )
        .is_err());
}

#[test]
fn dispatch_intent_is_durable_before_external_write_and_crash() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
    let side_effect = temp.path().join("external-write");
    let scope = "a".repeat(64);
    let mut journal = Journal::open(&path, &scope).unwrap();
    let planned = journal.execute(
        "workflow_start",
        &json!({"run_id":"crash-boundary", "envelope":{"checkpoint_format_version":1}, "snapshot":{"turn":0}, "deadline":500}),
        100,
        Value::Null,
    ).unwrap();
    let running = journal.execute(
        "workflow_resume",
        &json!({"run_id":"crash-boundary","expected_checkpoint_id":planned["checkpoint_id"],"key":"activate","reply":{"decision":"continue","snapshot":{"turn":0},"effect":{}}}),
        101,
        Value::Null,
    ).unwrap();
    let prepared = journal.execute(
        "workflow_resume",
        &json!({"run_id":"crash-boundary","expected_checkpoint_id":running["checkpoint_id"],"key":"prepare","reply":{"decision":"effect_prepared","snapshot":{"turn":1},"effect":{"tool":"workspace_write","arguments":{"path":"external-write"}}}}),
        102,
        Value::Null,
    ).unwrap();
    assert_eq!(prepared["phase"], "effect_prepared");
    // This is the dispatch intent written immediately before the external
    // owner is called. A process crash after the next line must recover as
    // uncertain, regardless of whether the write below became visible.
    let dispatching = journal.execute(
        "workflow_resume",
        &json!({"run_id":"crash-boundary","expected_checkpoint_id":prepared["checkpoint_id"],"key":"dispatch","reply":{"decision":"effect_uncertain","snapshot":{"turn":1},"effect":{"tool":"workspace_write","dispatch":"started","outcome":"unknown"}}}),
        103,
        Value::Null,
    ).unwrap();
    assert_eq!(dispatching["phase"], "effect_uncertain");
    std::fs::write(&side_effect, b"written before crash").unwrap();
    drop(journal);

    let mut recovered = Journal::open(&path, &scope).unwrap();
    let observed = recovered
        .execute(
            "workflow_status",
            &json!({"run_id":"crash-boundary"}),
            104,
            Value::Null,
        )
        .unwrap();
    assert!(side_effect.exists());
    assert_eq!(observed["phase"], "effect_uncertain");
    assert_eq!(observed["interrupts"][0]["kind"], "effect");
    assert_eq!(observed["state"]["effect"]["dispatch"], "started");
}

#[test]
fn retries_report_original_revision_and_hostcheck_is_idempotent() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
    let scope = "a".repeat(64);
    let mut journal = Journal::open(&path, &scope).unwrap();
    let start_args = json!({"run_id":"retry-run", "envelope":{"checkpoint_format_version":1,"host_verification_required":true}, "snapshot":{"turn":0}, "deadline":500});
    let planned = journal
        .execute("workflow_start", &start_args, 100, Value::Null)
        .unwrap();
    let start_retry = journal
        .execute("workflow_start", &start_args, 100, Value::Null)
        .unwrap();
    assert_eq!(start_retry["checkpoint_id"], planned["checkpoint_id"]);
    assert_eq!(start_retry["diagnostics"]["replayed_request_revision"], 1);
    assert_eq!(
        start_retry["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );
    let mut changed_start = start_args.clone();
    changed_start["snapshot"]["turn"] = json!(1);
    assert!(
        journal
            .execute("workflow_start", &changed_start, 100, Value::Null)
            .is_err()
    );

    let running = journal
        .execute("workflow_resume", &json!({"run_id":"retry-run","expected_checkpoint_id":planned["checkpoint_id"],"key":"activate","reply":{"decision":"continue","snapshot":{"turn":0},"effect":{}}}), 101, Value::Null)
        .unwrap();
    let first = journal
        .execute("workflow_resume", &json!({"run_id":"retry-run","expected_checkpoint_id":running["checkpoint_id"],"key":"step","reply":{"decision":"continue","snapshot":{"turn":1},"effect":{}}}), 102, Value::Null)
        .unwrap();
    let current = journal
        .execute("workflow_resume", &json!({"run_id":"retry-run","expected_checkpoint_id":first["checkpoint_id"],"key":"next","reply":{"decision":"done","snapshot":{"turn":2},"effect":{}}}), 103, Value::Null)
        .unwrap();
    let replay = journal
        .execute("workflow_resume", &json!({"run_id":"retry-run","expected_checkpoint_id":first["checkpoint_id"],"key":"step","reply":{"decision":"continue","snapshot":{"turn":1},"effect":{}}}), 104, Value::Null)
        .unwrap();
    assert_eq!(replay["checkpoint_id"], current["checkpoint_id"]);
    assert_eq!(replay["diagnostics"]["replayed_request_revision"], 3);
    assert_eq!(replay["diagnostics"]["current_revision"], 4);
    assert_eq!(
        replay["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );

    let verified = journal
        .execute("workflow_hostcheck", &json!({"run_id":"retry-run","expected_checkpoint_id":current["checkpoint_id"],"verification":{"status":"host_verified"}}), 105, Value::Null)
        .unwrap();
    let verified_retry = journal
        .execute("workflow_hostcheck", &json!({"run_id":"retry-run","expected_checkpoint_id":current["checkpoint_id"],"verification":{"status":"host_verified"}}), 105, Value::Null)
        .unwrap();
    assert_eq!(verified_retry["checkpoint_id"], verified["checkpoint_id"]);
    assert_eq!(
        verified_retry["diagnostics"]["replayed_request_revision"],
        5
    );
    assert_eq!(
        verified_retry["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );
}

#[test]
fn pauses_share_one_budget_and_host_verification_can_be_revoked() {
    let temp = tempfile::tempdir().unwrap();
    let mut journal = Journal::open(
        &temp.path().canonicalize().unwrap().join("execution.sqlite"),
        &"a".repeat(64),
    )
    .unwrap();
    let initial = start(&mut journal);
    let paused = advance(&mut journal, &initial, "pause", "pause", 110).unwrap();
    assert!(advance(&mut journal, &paused, "backward", "continue", 109).is_err());
    let resumed = advance(&mut journal, &paused, "resume", "continue", 160).unwrap();
    assert_eq!(resumed["state"]["pause_used"], 50);
    assert_eq!(resumed["state"]["deadline"], 550);
    let done = advance(&mut journal, &resumed, "done", "done", 161).unwrap();
    assert_eq!(done["phase"], "awaiting_verification");
    assert!(advance(&mut journal, &done, "forged", "verified", 162).is_err());
    let verified=journal.execute("workflow_hostcheck",&json!({"run_id":"run-one","expected_checkpoint_id":done["checkpoint_id"],"verification":{"status":"host_verified"}}),600,Value::Null).unwrap();
    assert_eq!(verified["phase"], "completed");
    let revoked=journal.execute("workflow_hostcheck",&json!({"run_id":"run-one","expected_checkpoint_id":verified["checkpoint_id"],"verification":{"status":"stale"}}),601,Value::Null).unwrap();
    assert_eq!(revoked["phase"], "awaiting_verification");
    assert_eq!(revoked["state"]["envelope"]["provider"], "fixed");
}

#[test]
fn periodic_snapshots_bound_replay_and_reduce_repeated_context_storage() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
    let mut journal = Journal::open(&path, &"a".repeat(64)).unwrap();
    let mut current=journal.execute("workflow_start",&json!({"run_id":"run-one","envelope":{"checkpoint_format_version":1,"assignment":"g".repeat(100_000)},"snapshot":{"turn":0},"deadline":500}),100,json!({"results":[]})).unwrap();
    for i in 0..100 {
        current = advance(
            &mut journal,
            &current,
            &format!("step-{i}"),
            "continue",
            101 + i,
        )
        .unwrap();
    }
    drop(journal);
    let mut journal = Journal::open(&path, &"a".repeat(64)).unwrap();
    let status = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"run-one"}),
            202,
            Value::Null,
        )
        .unwrap();
    assert_eq!(status["phase"], current["phase"]);
    assert_eq!(status["state"], current["state"]);
    assert_eq!(
        status["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );
    let connection = rusqlite::Connection::open(&path).unwrap();
    let stored: u64 = connection
        .query_row("SELECT bytes FROM runs", [], |r| r.get(0))
        .unwrap();
    assert!(
        stored < 1_000_000,
        "stored {stored} bytes instead of bounded snapshots/deltas"
    );
    // A valid SQLite file with corrupted event content must not be resumed.
    connection
        .execute("UPDATE events SET body='{}' WHERE revision=101", [])
        .unwrap();
    assert!(
        journal
            .execute(
                "workflow_status",
                &json!({"run_id":"run-one"}),
                202,
                Value::Null
            )
            .is_err()
    );
}

#[test]
fn unsupported_database_and_wrong_revision_delete_preserve_data() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap().join("execution.sqlite");
    let mut journal = Journal::open(&path, &"a".repeat(64)).unwrap();
    let initial = start(&mut journal);
    assert!(
        journal
            .execute(
                "workflow_forget",
                &json!({"run_id":"run-one","expected_checkpoint_id":"0"}),
                101,
                Value::Null
            )
            .is_err()
    );
    let status = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"run-one"}),
            101,
            Value::Null,
        )
        .unwrap();
    assert_eq!(status["state"], initial["state"]);
    assert_eq!(
        status["diagnostics"]["checkpoint_serialized_bytes_appended"],
        0
    );
    drop(journal);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    assert!(Journal::open(&path, &"a".repeat(64)).is_err());
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM runs", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}
