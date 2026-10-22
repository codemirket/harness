// Experiment only: small SQLite CAS journal, deliberately not an execution driver.
// It does not implement LangGraph migration, effect reconciliation, host handoffs,
// retention/export or multi-process coordination. Passing these cases cannot
// authorize replacing the production checkpoint adapter.
use rusqlite::{Connection, params};
use serde_json::{Value, json};

fn transition(connection: &mut Connection, expected: i64, key: &str, payload: &str) -> anyhow::Result<i64> {
    let tx = connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    let (revision, previous_key, previous): (i64, String, String) = tx.query_row("SELECT revision,key,payload FROM checkpoint WHERE id=1", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if key == previous_key {
        anyhow::ensure!(payload == previous, "retry input changed");
        return Ok(revision);
    }
    anyhow::ensure!(revision == expected, "stale checkpoint");
    tx.execute("UPDATE checkpoint SET revision=?1,key=?2,payload=?3 WHERE id=1", params![revision+1,key,payload])?;
    tx.commit()?;
    Ok(revision+1)
}

#[test]
fn journal_prototype_measures_only_storage_not_recovery_parity() {
    let temp = tempfile::tempdir().unwrap(); let path = temp.path().join("prototype.sqlite");
    let mut conn = Connection::open(&path).unwrap();
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE checkpoint(id INTEGER PRIMARY KEY,revision INTEGER,key TEXT,payload TEXT); INSERT INTO checkpoint VALUES(1,0,'','{}');").unwrap();
    let pending = json!({"effect":{"tool":"workspace_write","expected_sha256":"before","content_sha256":"after"},"deadline":500,"turn":7}).to_string();
    assert_eq!(transition(&mut conn,0,"effect-1",&pending).unwrap(),1);
    drop(conn); let mut conn=Connection::open(&path).unwrap();
    assert_eq!(transition(&mut conn,0,"effect-1",&pending).unwrap(),1);
    assert!(transition(&mut conn,0,"effect-2","{}").is_err());
    assert!(transition(&mut conn,1,"effect-1","{}").is_err());
    let saved: String=conn.query_row("SELECT payload FROM checkpoint",[],|r|r.get(0)).unwrap();
    assert_eq!(saved,pending);
    let started=std::time::Instant::now();
    let payload=json!({"transcript":"x".repeat(128*1024)}).to_string();
    for index in 1..=100 { transition(&mut conn,index,&format!("step-{index}"),&payload).unwrap(); }
    let report: Value=json!({"experiment":"native-journal-storage-prototype","transitions":100,"snapshot_bytes":payload.len(),"elapsed_ms":started.elapsed().as_millis(),"storage_cases":["restart","same-key replay","stale revision","changed retry input","pending-effect bytes retained"],"replacement_ready":false,"missing_parity":["effect reconciliation","pause and host-verification state machine","migration of existing LangGraph runs","retention/export integration","cross-process recovery"],"decision":"retain LangGraph; storage cost alone is insufficient evidence"});
    println!("{report}");
    if let Ok(path)=std::env::var("MIRKET_JOURNAL_OUTPUT") {
        use std::io::Write;
        let path=std::path::PathBuf::from(path); assert!(path.is_absolute());
        std::fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap().write_all(report.to_string().as_bytes()).unwrap();
    }
}
