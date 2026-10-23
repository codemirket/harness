use mirket::memory::store::FactStore;
use serde_json::{Value, json};

fn vector(axis: usize) -> Vec<f32> {
    let mut v = vec![0.; 384];
    v[axis] = 1.;
    v
}
fn args(key: &str, text: &str) -> Value {
    json!({"key":key,"text":text,"kind":"fact","provenance":"synthetic fixture","expires_at":null})
}

#[test]
fn facts_publish_replace_forget_and_retry_across_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
    let scope = "a".repeat(64);
    let first;
    let second;
    {
        let mut store = FactStore::open(&path, &scope, "fixture-384").unwrap();
        let a = args("one", "Database is PostgreSQL");
        first = store.put(&a, &vector(0), 100).unwrap()["memory"].clone();
        assert_eq!(store.put(&a, &vector(0), 101).unwrap()["reused"], true);
        assert!(
            store
                .put(&args("one", "different"), &vector(0), 101)
                .is_err()
        );
        let mut replacement = args("two", "Database is SQLite");
        replacement["supersedes"] = json!({"id":first["id"],"sha256":first["sha256"]});
        second = store.put(&replacement, &vector(0), 101).unwrap()["memory"].clone();
        assert!(
            store.read(first["id"].as_str(), None, 5, 102).unwrap()["results"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let mut stale = args("three", "stale update");
        stale["supersedes"] = replacement["supersedes"].clone();
        assert!(store.put(&stale, &vector(0), 102).is_err());
    }
    let mut store = FactStore::open(&path, &scope, "fixture-384").unwrap();
    assert_eq!(
        store.search("database", Some(&vector(0)), 5, 103).unwrap()["results"][0]["id"],
        second["id"]
    );
    assert!(
        store
            .forget(second["id"].as_str().unwrap(), "wrong")
            .is_err()
    );
    store
        .forget(
            second["id"].as_str().unwrap(),
            second["sha256"].as_str().unwrap(),
        )
        .unwrap();
    // Forgetting the current record must not resurrect its superseded predecessor.
    assert_eq!(
        store.read(None, None, 5, 103).unwrap()["results"],
        json!([])
    );
    store
        .forget(
            second["id"].as_str().unwrap(),
            second["sha256"].as_str().unwrap(),
        )
        .unwrap();
    assert!(FactStore::open(&path, &"b".repeat(64), "fixture-384").is_err());
    assert!(FactStore::open(&path, &scope, "other-model").is_err());
}

#[test]
fn expired_prefix_cannot_starve_active_lexical_or_dense_results() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = FactStore::open(
        &dir.path().canonicalize().unwrap().join("facts.sqlite"),
        &"a".repeat(64),
        "fixture",
    )
    .unwrap();
    for i in 0..1000 {
        let mut a = args(&format!("expired-{i}"), "MIRKET-4827 obsolete release");
        a["expires_at"] = json!(101);
        store.put(&a, &vector(0), 100).unwrap();
    }
    let current = store
        .put(
            &args("current", "MIRKET-4827 current release"),
            &vector(0),
            102,
        )
        .unwrap()["memory"]
        .clone();
    for query in [None, Some(vector(0))] {
        let found = store
            .search("MIRKET-4827", query.as_deref(), 5, 102)
            .unwrap();
        assert_eq!(found["results"].as_array().unwrap().len(), 1);
        assert_eq!(found["results"][0]["id"], current["id"]);
        assert_eq!(found["retrieval"]["incomplete"], false);
        if query.is_some() {
            assert_eq!(found["retrieval"]["eligible_vectors_examined"], 1);
        }
    }
}

#[test]
fn invalid_embedding_and_stale_replacement_publish_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = FactStore::open(
        &dir.path().canonicalize().unwrap().join("facts.sqlite"),
        &"a".repeat(64),
        "fixture",
    )
    .unwrap();
    for v in [vec![1.; 3], vec![0.; 384], vec![f32::NAN; 384]] {
        assert!(store.put(&args("bad", "bad vector"), &v, 100).is_err());
    }
    let mut bad = args("bad", "missing predecessor");
    bad["supersedes"] = json!({"id":"missing","sha256":"wrong"});
    assert!(store.put(&bad, &vector(0), 100).is_err());
    assert_eq!(
        store.read(None, None, 5, 100).unwrap()["results"],
        json!([])
    );
    store
        .put(&args("valid", "Türkçe SQLite preference"), &vector(0), 100)
        .unwrap();
    assert_eq!(
        store.search("Türkçe", None, 5, 100).unwrap()["results"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .search("unknown OR NOT \" )", Some(&vector(1)), 5, 100)
            .unwrap()["results"],
        json!([])
    );
}

#[test]
fn foreign_or_future_database_is_not_initialized_or_downgraded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().canonicalize().unwrap().join("foreign.sqlite");
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch("CREATE TABLE important(value TEXT); INSERT INTO important VALUES('preserve'); PRAGMA user_version=99;").unwrap();
    assert!(FactStore::open(&path, &"a".repeat(64), "fixture").is_err());
    assert_eq!(
        conn.query_row("SELECT value FROM important", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "preserve"
    );
    assert_eq!(
        conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        99
    );
}

#[test]
fn forgotten_fact_cannot_be_resurrected_by_a_delayed_retry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
    let mut store = FactStore::open(&path, &"a".repeat(64), "fixture").unwrap();
    let request = args("original", "private synthetic fact");
    let item = store.put(&request, &vector(0), 100).unwrap()["memory"].clone();
    store
        .forget(
            item["id"].as_str().unwrap(),
            item["sha256"].as_str().unwrap(),
        )
        .unwrap();
    drop(store);
    let mut store = FactStore::open(&path, &"a".repeat(64), "fixture").unwrap();
    assert!(store.put(&request, &vector(0), 101).is_err());
    assert_eq!(
        store.read(None, None, 5, 101).unwrap()["results"],
        json!([])
    );
    let conn = rusqlite::Connection::open(&path).unwrap();
    for table in ["facts", "requests", "fact_text"] {
        assert_eq!(
            conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    assert!(
        store
            .put(
                &args("explicit-new", "private synthetic fact"),
                &vector(0),
                101
            )
            .is_ok()
    );
}

#[test]
fn identity_is_singleton_and_query_bounds_are_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
    let scope = "c".repeat(64);
    let store = FactStore::open(&path, &scope, "fixture").unwrap();
    drop(store);

    let conn = rusqlite::Connection::open(&path).unwrap();
    let identity_sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type='table' AND name='identity'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(identity_sql.contains("PRIMARY KEY"));

    // Exercise startup validation against a structurally corrupt identity table.
    conn.execute_batch(
        "ALTER TABLE identity RENAME TO identity_valid;
         CREATE TABLE identity(scope TEXT NOT NULL, model TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 0);
         INSERT INTO identity SELECT scope,model,revision FROM identity_valid;
         INSERT INTO identity SELECT scope,model,revision FROM identity_valid;
         DROP TABLE identity_valid;",
    )
    .unwrap();
    drop(conn);
    assert!(FactStore::open(&path, &scope, "fixture").is_err());

    let fresh = dir.path().canonicalize().unwrap().join("fresh.sqlite");
    let mut store = FactStore::open(&fresh, &scope, "fixture").unwrap();
    assert!(store.search(&"term ".repeat(103), None, 5, 100).is_err());
    assert!(
        store
            .search(
                &(0..129)
                    .map(|i| format!("t{i}"))
                    .collect::<Vec<_>>()
                    .join(" "),
                None,
                5,
                100,
            )
            .is_err()
    );
}
