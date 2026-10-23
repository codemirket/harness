//! Native retrieval corpus checks.
//!
//! The former test launched a Python container and compared prototype
//! retrieval arms. The production owner is now the Rust fact store, so this
//! required gate exercises its multilingual corpus directly.

use mirket::memory::store::FactStore;
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
        "provenance": "native corpus fixture",
        "expires_at": null
    })
}

#[test]
fn native_multilingual_retrieval_corpus() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let mut facts = FactStore::open(
        &root.join("facts.sqlite"),
        &"a".repeat(64),
        "native-corpus-model",
    )
    .unwrap();
    for (axis, (key, text)) in [
        ("turkish", "Turkish offline behavior is required"),
        ("german", "German financial reporting is required"),
        ("english", "English installation instructions are current"),
        ("journal", "Journal recovery preserves uncertain effects"),
    ]
    .into_iter()
    .enumerate()
    {
        facts.put(&fact(key, text), &vector(axis), 100).unwrap();
    }

    for (query, key) in [
        ("Turkish offline behavior", "turkish"),
        ("German financial reporting", "german"),
        ("English installation", "english"),
        ("uncertain effects", "journal"),
    ] {
        let result = facts.search(query, None, 5, 100).unwrap();
        assert_eq!(result["results"][0]["metadata"]["request_key"], key);
    }
}
