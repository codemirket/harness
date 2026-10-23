//! Native-owner fault gates. These tests exercise the actual SQLite owners,
//! including process death between durable transitions; they do not dispatch
//! external tools or run the contained embedding image.
use mirket::{agent_runtime::journal::Journal, memory::store::FactStore};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

fn vector(axis: usize) -> Vec<f32> {
    let mut v = vec![0.; 384];
    v[axis % 384] = 1.;
    v
}
fn fact(key: &str, text: &str) -> Value {
    json!({"key":key,"text":text,"kind":"fact","provenance":"native fault fixture","expires_at":null})
}
fn scope() -> String {
    "a".repeat(64)
}

fn start(journal: &mut Journal) -> Value {
    journal.execute("workflow_start", &json!({"run_id":"fault-run","envelope":{"checkpoint_format_version":1,"host_verification_required":true,"provider":"fixture"},"snapshot":{"turn":0,"assignment":"fixed fault fixture"},"deadline":5000}), 100, json!({"results":[]})).unwrap()
}

#[test]
fn acknowledged_fact_survives_child_process_restart() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("facts.sqlite");
    let marker = root.join("ack");
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("child_fact_writer")
        .arg("--nocapture")
        .env("MIRKET_NATIVE_CHILD", "fact")
        .env("MIRKET_NATIVE_PATH", &path)
        .env("MIRKET_NATIVE_MARKER", &marker)
        .status()
        .unwrap();
    assert!(status.success(), "child writer failed: {status}");
    assert!(
        marker.exists(),
        "child did not acknowledge its durable write"
    );
    let mut store = FactStore::open(&path, &scope(), "fixture-384").unwrap();
    let results = store
        .search("restart-marker", Some(&vector(3)), 5, 200)
        .unwrap();
    assert_eq!(results["results"].as_array().unwrap().len(), 1);
    assert_eq!(
        results["results"][0]["memory"],
        "restart-marker durable fact"
    );
}

#[test]
fn killed_child_leaves_pending_effect_without_implicit_replay() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("execution.sqlite");
    let marker = root.join("ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("child_pending_effect")
        .arg("--nocapture")
        .env("MIRKET_NATIVE_CHILD", "effect")
        .env("MIRKET_NATIVE_PATH", &path)
        .env("MIRKET_NATIVE_MARKER", &marker)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        marker.exists(),
        "child did not reach durable effect boundary"
    );
    child.kill().unwrap();
    let _ = child.wait();
    let mut journal = Journal::open(&path, &scope()).unwrap();
    let status = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"fault-run"}),
            200,
            Value::Null,
        )
        .unwrap();
    assert_eq!(status["phase"], "effect_uncertain");
    let after_restart = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"fault-run"}),
            201,
            Value::Null,
        )
        .unwrap();
    assert_eq!(after_restart["phase"], status["phase"]);
    assert_eq!(after_restart["state"], status["state"]);
    assert_eq!(after_restart["checkpoint_id"], status["checkpoint_id"]);
    assert_eq!(after_restart["next"], status["next"]);
    assert_eq!(after_restart["interrupts"], status["interrupts"]);
}

#[test]
fn before_commit_failure_preserves_acknowledged_fact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
    let scope = scope();
    let old_id;
    {
        let mut store = FactStore::open(&path, &scope, "fixture-384").unwrap();
        old_id = store
            .put(
                &fact("before", "acknowledged before failed commit"),
                &vector(0),
                100,
            )
            .unwrap()["memory"]["id"]
            .clone();
        // Invalid vector bytes are rejected before publication. Reopening is
        // the durable boundary that proves no partial fact was acknowledged.
        assert!(
            store
                .put(&fact("failed", "must not publish"), &[1.; 383], 101)
                .is_err()
        );
    }
    let reopened = FactStore::open(&path, &scope, "fixture-384").unwrap();
    assert_eq!(
        reopened.read(None, None, 5, 102).unwrap()["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        reopened
            .read(Some(old_id.as_str().unwrap()), None, 1, 102)
            .unwrap()["results"][0]["id"],
        old_id
    );
}

#[test]
fn supersession_failure_rolls_back_without_hiding_old_fact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
    let scope = scope();
    let mut store = FactStore::open(&path, &scope, "fixture-384").unwrap();
    let old = store
        .put(&fact("old", "old acknowledged value"), &vector(0), 100)
        .unwrap()["memory"]
        .clone();
    let mut replacement = fact("new", "replacement must not publish");
    replacement["supersedes"] = json!({"id":old["id"],"sha256":"0".repeat(64)});
    assert!(store.put(&replacement, &vector(1), 101).is_err());
    drop(store);
    let reopened = FactStore::open(&path, &scope, "fixture-384").unwrap();
    assert_eq!(
        reopened.read(None, None, 5, 102).unwrap()["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        reopened
            .read(Some(old["id"].as_str().unwrap()), None, 1, 102)
            .unwrap()["results"][0]["id"],
        old["id"]
    );
}

#[test]
fn killed_child_keeps_host_verification_pending() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("verification.sqlite");
    let marker = root.join("verification-ready");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("child_host_verification")
        .arg("--nocapture")
        .env("MIRKET_NATIVE_CHILD", "host")
        .env("MIRKET_NATIVE_PATH", &path)
        .env("MIRKET_NATIVE_MARKER", &marker)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        marker.exists(),
        "child did not reach durable host-verification boundary"
    );
    child.kill().unwrap();
    let _ = child.wait();
    let mut journal = Journal::open(&path, &scope()).unwrap();
    let status = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"fault-run"}),
            200,
            Value::Null,
        )
        .unwrap();
    assert_eq!(status["phase"], "awaiting_verification");
    let reopened = journal
        .execute(
            "workflow_status",
            &json!({"run_id":"fault-run"}),
            201,
            Value::Null,
        )
        .unwrap();
    assert_eq!(reopened["phase"], status["phase"]);
    assert_eq!(reopened["state"], status["state"]);
    assert_eq!(reopened["checkpoint_id"], status["checkpoint_id"]);
    assert_eq!(reopened["next"], status["next"]);
    assert_eq!(reopened["interrupts"], status["interrupts"]);
}

#[test]
fn concurrent_initialization_and_cas_publish_one_winner() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let facts = root.join("facts.sqlite");
    let barrier = Arc::new(Barrier::new(2));
    let mut joins = Vec::new();
    for n in 0..2 {
        let path = facts.clone();
        let gate = barrier.clone();
        joins.push(thread::spawn(move || {
            gate.wait();
            let mut store = FactStore::open(&path, &scope(), "fixture-384").unwrap();
            store
                .put(
                    &fact(&format!("client-{n}"), &format!("client-{n} durable")),
                    &vector(n),
                    100,
                )
                .unwrap();
        }));
    }
    for join in joins {
        join.join().unwrap();
    }
    let store = FactStore::open(&facts, &scope(), "fixture-384").unwrap();
    assert_eq!(
        store.read(None, None, 5, 101).unwrap()["results"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let journal_path = root.join("execution.sqlite");
    let mut initial = Journal::open(&journal_path, &scope()).unwrap();
    let checkpoint = start(&mut initial);
    drop(initial);
    let barrier = Arc::new(Barrier::new(2));
    let mut joins = Vec::new();
    for n in 0..2 {
        let path = journal_path.clone();
        let gate = barrier.clone();
        let expected = checkpoint.clone();
        joins.push(thread::spawn(move || { gate.wait(); let mut journal = Journal::open(&path, &scope()).unwrap(); journal.execute("workflow_resume", &json!({"run_id":"fault-run","expected_checkpoint_id":expected["checkpoint_id"],"key":format!("cas-{n}"),"reply":{"decision":"continue","snapshot":{"turn":1}}}), 101, Value::Null).is_ok() }));
    }
    let winners = joins
        .into_iter()
        .map(|j| j.join().unwrap())
        .filter(|ok| *ok)
        .count();
    assert_eq!(
        winners, 1,
        "CAS must admit exactly one concurrent transition"
    );
}

#[test]
fn corrupted_native_records_fail_closed() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("execution.sqlite");
    let mut journal = Journal::open(&path, &scope()).unwrap();
    let checkpoint = start(&mut journal);
    journal.execute("workflow_resume", &json!({"run_id":"fault-run","expected_checkpoint_id":checkpoint["checkpoint_id"],"key":"corrupt","reply":{"decision":"continue","snapshot":{"turn":1}}}), 101, Value::Null).unwrap();
    drop(journal);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute("UPDATE events SET body='{}' WHERE revision=2", [])
        .unwrap();
    let mut reopened = Journal::open(&path, &scope()).unwrap();
    assert!(
        reopened
            .execute(
                "workflow_status",
                &json!({"run_id":"fault-run"}),
                102,
                Value::Null
            )
            .is_err()
    );
}

#[test]
fn child_fact_writer() {
    if std::env::var("MIRKET_NATIVE_CHILD").ok().as_deref() != Some("fact") {
        return;
    }
    let path = PathBuf::from(std::env::var_os("MIRKET_NATIVE_PATH").unwrap());
    let marker = PathBuf::from(std::env::var_os("MIRKET_NATIVE_MARKER").unwrap());
    let mut store = FactStore::open(&path, &scope(), "fixture-384").unwrap();
    store
        .put(
            &fact("child", "restart-marker durable fact"),
            &vector(3),
            100,
        )
        .unwrap();
    fs::write(marker, b"ack").unwrap();
}

#[test]
fn child_host_verification() {
    if std::env::var("MIRKET_NATIVE_CHILD").ok().as_deref() != Some("host") {
        return;
    }
    let path = PathBuf::from(std::env::var_os("MIRKET_NATIVE_PATH").unwrap());
    let marker = PathBuf::from(std::env::var_os("MIRKET_NATIVE_MARKER").unwrap());
    let mut journal = Journal::open(&path, &scope()).unwrap();
    let planned = start(&mut journal);
    let running = journal.execute("workflow_resume", &json!({"run_id":"fault-run","expected_checkpoint_id":planned["checkpoint_id"],"key":"activate","reply":{"decision":"continue","snapshot":{"turn":0},"effect":{}}}), 101, Value::Null).unwrap();
    let done = journal.execute("workflow_resume", &json!({"run_id":"fault-run","expected_checkpoint_id":running["checkpoint_id"],"key":"done","reply":{"decision":"done","snapshot":{"turn":1},"effect":{}}}), 102, Value::Null).unwrap();
    assert_eq!(done["phase"], "awaiting_verification");
    fs::write(marker, b"ready").unwrap();
    thread::sleep(Duration::from_secs(60));
}

#[test]
fn child_pending_effect() {
    if std::env::var("MIRKET_NATIVE_CHILD").ok().as_deref() != Some("effect") {
        return;
    }
    let path = PathBuf::from(std::env::var_os("MIRKET_NATIVE_PATH").unwrap());
    let marker = PathBuf::from(std::env::var_os("MIRKET_NATIVE_MARKER").unwrap());
    let mut journal = Journal::open(&path, &scope()).unwrap();
    let checkpoint = start(&mut journal);
    let prepared = journal.execute("workflow_resume", &json!({"run_id":"fault-run","expected_checkpoint_id":checkpoint["checkpoint_id"],"key":"prepare","reply":{"decision":"effect_prepared","snapshot":{"turn":1},"effect":{"tool":"external","arguments":{"value":"one"}}}}), 101, Value::Null).unwrap();
    journal.execute("workflow_resume", &json!({"run_id":"fault-run","expected_checkpoint_id":prepared["checkpoint_id"],"key":"effect","reply":{"decision":"effect_uncertain","snapshot":{"turn":1},"effect":{"tool":"external","arguments":{"value":"one"}}}}), 102, Value::Null).unwrap();
    fs::write(marker, b"ready").unwrap();
    thread::sleep(Duration::from_secs(60));
}
