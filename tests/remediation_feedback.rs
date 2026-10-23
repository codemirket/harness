//! Behavioral regressions retained from the pre-migration feedback suite.
//! These tests call the native Rust owners directly; they do not inspect source
//! text or import the retired runtime.
use mirket::{
    agent_runtime::{journal::Journal, queries},
    memory::store::FactStore,
};
use serde_json::{Value, json};

fn vector(axis: usize) -> Vec<f32> {
    let mut result = vec![0.0; 384];
    result[axis] = 1.0;
    result
}

fn fact(key: &str, text: &str) -> Value {
    json!({"key": key, "text": text, "kind": "fact", "provenance": "feedback fixture", "expires_at": null})
}

fn store(path: &std::path::Path) -> FactStore {
    FactStore::open(path, &"a".repeat(64), "feedback-model").unwrap()
}

#[test]
fn late_objective_requirements_and_explicit_queries_are_retrievable() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut facts = store(&root.join("facts.sqlite"));
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

    // The late marker must survive objective chunking and be passed to the
    // native lexical owner. This exercises the actual planner, rather than a
    // source-text assertion or a hand-picked search string.
    let objective_request = json!({
        "objective": format!("{} Turkish offline behavior is required", "context ".repeat(40)),
        "task_needs": []
    });
    let (planned, incomplete, automatic) = queries::plan(&objective_request).unwrap();
    assert!(automatic);
    assert!(!incomplete);
    let late_query = planned
        .iter()
        .find(|query| query.contains("Turkish offline behavior"))
        .unwrap();
    let result = facts.search(late_query, None, 5, 100).unwrap();
    assert_eq!(result["results"][0]["metadata"]["request_key"], "turkish");

    // Explicit queries remain in caller order and do not gain objective
    // chunks. The second query must still retrieve the late requirement.
    let explicit_request = json!({
        "objective": "ignored objective",
        "memory_queries": ["late requirement", "German financial reporting"]
    });
    let (explicit, incomplete, automatic) = queries::plan(&explicit_request).unwrap();
    assert_eq!(explicit, ["late requirement", "German financial reporting"]);
    assert!(!incomplete);
    assert!(!automatic);
    let result = facts.search(&explicit[1], None, 5, 100).unwrap();
    assert_eq!(result["results"][0]["metadata"]["request_key"], "german");
}

#[test]
fn query_planner_reports_bounded_truncation() {
    let request = json!({
        "objective": "x".repeat(192 * 12),
        "task_needs": (0..8)
            .map(|index| json!({"reason": format!("need-{index} {}", "z".repeat(220))}))
            .collect::<Vec<_>>()
    });
    let (queries, incomplete, automatic) = queries::plan(&request).unwrap();
    assert!(automatic);
    assert!(queries.len() <= 8);
    assert!(incomplete);
}

#[test]
fn empty_retrieval_cannot_pass_the_native_baseline() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut empty = store(&root.join("empty.sqlite"));
    assert!(
        empty.search("known baseline fact", None, 5, 100).unwrap()["results"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let mut populated = store(&root.join("populated.sqlite"));
    for index in 0..23 {
        populated
            .put(
                &fact(&format!("fact-{index}"), &format!("baseline fact {index}")),
                &vector(index % 2),
                100,
            )
            .unwrap();
    }
    let result = populated.search("baseline fact", None, 3, 100).unwrap();
    assert_eq!(result["results"].as_array().unwrap().len(), 3);
    assert!(
        result["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["memory"].as_str().unwrap().contains("baseline fact"))
    );
}

#[test]
fn pause_budget_and_uncertain_effect_feedback_remain_distinct() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut journal = Journal::open(&root.join("execution.sqlite"), &"a".repeat(64)).unwrap();
    let start = journal
        .execute(
            "workflow_start",
            &json!({
                "run_id":"feedback-run", "envelope":{"checkpoint_format_version":1},
                "snapshot":{"turn":0}, "deadline":500
            }),
            100,
            json!({}),
        )
        .unwrap();
    let paused = journal.execute("workflow_resume", &json!({
        "run_id":"feedback-run", "expected_checkpoint_id":start["checkpoint_id"], "key":"pause",
        "reply":{"decision":"pause", "snapshot":{"turn":1}}
    }), 110, Value::Null).unwrap();
    let resumed = journal.execute("workflow_resume", &json!({
        "run_id":"feedback-run", "expected_checkpoint_id":paused["checkpoint_id"], "key":"resume",
        "reply":{"decision":"continue", "snapshot":{"turn":2}}
    }), 160, Value::Null).unwrap();
    assert_eq!(resumed["state"]["pause_used"], 50);
    let prepared = journal.execute("workflow_resume", &json!({
        "run_id":"feedback-run", "expected_checkpoint_id":resumed["checkpoint_id"], "key":"effect",
        "reply":{"decision":"effect_prepared", "snapshot":{"turn":3}, "effect":{"tool":"remote_send"}}
    }), 161, Value::Null).unwrap();
    let uncertain=journal.execute("workflow_resume",&json!({"run_id":"feedback-run","expected_checkpoint_id":prepared["checkpoint_id"],"key":"dispatch","reply":{"decision":"effect_uncertain","snapshot":{"turn":3},"effect":prepared["state"]["effect"]}}),162,Value::Null).unwrap();
    assert_eq!(uncertain["phase"], "effect_uncertain");
    assert_ne!(uncertain["state"]["pause_used"], 0);
}
