//! Native simplification regression.
//!
//! The former ignored test launched a Python container and asserted the shape
//! of a four-arm prototype report. Production retrieval and recovery now have
//! Rust owners, so this required test exercises the two state boundaries
//! together with a disposable native corpus.

use mirket::{agent_runtime::journal::Journal, memory::store::FactStore};
use serde_json::{Value, json};

fn vector(axis: usize) -> Vec<f32> {
    let mut result = vec![0.0; 384];
    result[axis] = 1.0;
    result
}

fn fact(key: &str, text: &str) -> Value {
    json!({
        "key": key,
        "text": text,
        "kind": "fact",
        "provenance": "native simplification fixture",
        "expires_at": null
    })
}

#[test]
fn native_retrieval_and_recovery_remain_distinct() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut facts = FactStore::open(
        &root.join("facts.sqlite"),
        &"a".repeat(64),
        "native-simplification-model",
    )
    .unwrap();
    facts
        .put(
            &fact("turkish", "Turkish offline behavior is required"),
            &vector(0),
            100,
        )
        .unwrap();
    facts
        .put(
            &fact("german", "German financial reporting is required"),
            &vector(1),
            100,
        )
        .unwrap();
    let retrieval = facts
        .search("Turkish offline behavior", None, 5, 100)
        .unwrap();
    assert_eq!(
        retrieval["results"][0]["metadata"]["request_key"],
        "turkish"
    );

    let mut journal = Journal::open(&root.join("execution.sqlite"), &"a".repeat(64)).unwrap();
    let started = journal
        .execute(
            "workflow_start",
            &json!({
                "run_id":"native-simplification",
                "envelope":{"checkpoint_format_version":1},
                "snapshot":{"turn":0},
                "deadline":500
            }),
            100,
            json!({}),
        )
        .unwrap();
    let prepared = journal
        .execute(
            "workflow_resume",
            &json!({
                "run_id":"native-simplification",
                "expected_checkpoint_id":started["checkpoint_id"],
                "key":"prepare",
                "reply":{"decision":"effect_prepared","snapshot":{"turn":1},"effect":{"tool":"remote_send"}}
            }),
            110,
            Value::Null,
        )
        .unwrap();
    let uncertain = journal
        .execute(
            "workflow_resume",
            &json!({
                "run_id":"native-simplification",
                "expected_checkpoint_id":prepared["checkpoint_id"],
                "key":"effect",
                "reply":{"decision":"effect_uncertain","snapshot":{"turn":1},"effect":{"tool":"remote_send"}}
            }),
            111,
            Value::Null,
        )
        .unwrap();
    assert_eq!(uncertain["phase"], "effect_uncertain");
    assert_eq!(
        retrieval["results"][0]["memory"],
        "Turkish offline behavior is required"
    );
}
