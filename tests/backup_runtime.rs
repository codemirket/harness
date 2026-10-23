//! Native backup integration: disposable state, real SQLite WAL and pending effects.
use mirket::{backup, paths::Paths, state::State};
use serde_json::{Value, json};
use std::{fs, path::Path};

async fn call(
    _engine_home: &Paths,
    fixture: &Paths,
    scope: &str,
    operation: &str,
    args: Value,
) -> Value {
    let directory = mirket::packages::agent::data(fixture).join(scope);
    fs::create_dir_all(&directory).unwrap();
    let now = mirket::router::durable::now();
    if operation.starts_with("workflow_") {
        return mirket::agent_runtime::journal::Journal::open(
            &directory.join("execution.sqlite"),
            scope,
        )
        .unwrap()
        .execute(operation, &args, now, json!({"results":[]}))
        .unwrap();
    }
    let model = mirket::util::sha256(include_bytes!("../runtime/agent/model.json"));
    let mut store =
        mirket::memory::store::FactStore::open(&directory.join("facts.sqlite"), scope, &model)
            .unwrap();
    let mut vector = vec![0.; 384];
    vector[0] = 1.;
    match operation {
        "memory_put" => store.put(&args, &vector, now).unwrap(),
        "memory_read" => match args["query"].as_str() {
            Some(q) => store.search(q, Some(&vector), 5, now).unwrap(),
            None => store.read(None, None, 5, now).unwrap(),
        },
        _ => panic!("unsupported fixture operation"),
    }
}
fn sqlite_rows(path: &Path, table: &str) -> i64 {
    let db = rusqlite::Connection::open(path).unwrap();
    db.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
#[tokio::test]
async fn backup_runtime_recovers_history_pending_effect_and_relocated_memory() {
    let fixture = tempfile::tempdir().unwrap();
    let engine_home = Paths::new(fixture.path().canonicalize().unwrap()).unwrap();
    let paths = Paths::new(fixture.path().canonicalize().unwrap()).unwrap();
    let project = paths.home.join("project");
    fs::create_dir(&project).unwrap();
    let state = State::open(&paths).unwrap();
    let registration = state.register_project(&project).unwrap();
    let id = registration["id"].as_str().unwrap();
    let scope = state.runtime_scope(&project).unwrap();
    let first=call(&engine_home,&paths,&scope,"memory_put",json!({"text":"The approved brand color is orange.","kind":"decision","provenance":"Synthetic backup fixture","key":"old"})).await["memory"].clone();
    let new=call(&engine_home,&paths,&scope,"memory_put",json!({"text":"The approved brand color is cobalt blue.","kind":"decision","provenance":"Synthetic backup fixture","key":"new","supersedes":{"id":first["id"],"sha256":first["sha256"]}})).await["memory"].clone();
    let run_id = "routed-backup-fixture";
    let start=call(&engine_home,&paths,&scope,"workflow_start",json!({"run_id":run_id,"envelope":{"checkpoint_format_version":1,"request":{"objective":"Exercise recovery","memory_recall":"off"},"project":project,"source_sha256":env!("MIRKET_SOURCE_SHA256"),"execution_sha256":env!("MIRKET_EXECUTION_SHA256"),"host_verification_required":true},"snapshot":{"turn":0},"deadline":mirket::router::durable::now()+3600})).await;
    let prepared=call(&engine_home,&paths,&scope,"workflow_resume",json!({"run_id":run_id,"expected_checkpoint_id":start["checkpoint_id"],"key":"effect","reply":{"decision":"effect_prepared","snapshot":{"turn":1},"effect":{"tool":"external-effect","may_mutate":true}}})).await;
    let pending=call(&engine_home,&paths,&scope,"workflow_resume",json!({"run_id":run_id,"expected_checkpoint_id":prepared["checkpoint_id"],"key":"dispatch","reply":{"decision":"effect_uncertain","snapshot":{"turn":1},"effect":prepared["state"]["effect"]}})).await;
    let run = paths.root.join("runs").join(run_id);
    fs::create_dir_all(&run).unwrap();
    fs::write(run.join("trace.json"), b"synthetic pending-effect evidence").unwrap();
    let directory = mirket::packages::agent::data(&paths).join(&scope);
    let history = sqlite_rows(&directory.join("facts.sqlite"), "facts");
    let checkpoints = sqlite_rows(&directory.join("execution.sqlite"), "events");
    assert!(history >= 2 && checkpoints > 1);
    let archive = paths.home.join("recovery.tar");
    let receipt = backup::create(&paths, &archive).unwrap();
    let hash = receipt["sha256"].as_str().unwrap();
    // Data loss only inside the disposable fixture; original project and backup remain.
    fs::remove_dir_all(mirket::packages::agent::data(&paths)).unwrap();
    fs::remove_dir_all(paths.root.join("runs")).unwrap();
    for name in ["state.db", "state.db-wal", "state.db-shm"] {
        let p = paths.root.join(name);
        if p.exists() {
            fs::remove_file(p).unwrap();
        }
    }
    backup::restore(&paths, &archive, hash).unwrap();
    assert_eq!(
        sqlite_rows(&directory.join("facts.sqlite"), "facts"),
        history
    );
    assert_eq!(
        sqlite_rows(&directory.join("execution.sqlite"), "events"),
        checkpoints
    );
    let restored = call(
        &engine_home,
        &paths,
        &scope,
        "workflow_status",
        json!({"run_id":run_id}),
    )
    .await;
    assert_eq!(restored["checkpoint_id"], pending["checkpoint_id"]);
    assert_eq!(restored["interrupts"][0]["kind"], "effect");
    assert_eq!(
        fs::read(run.join("trace.json")).unwrap(),
        b"synthetic pending-effect evidence"
    );
    let moved = paths.home.join("renamed");
    fs::rename(&project, &moved).unwrap();
    let state = State::open(&paths).unwrap();
    state.relocate(id, &project, &moved).unwrap();
    let relocated = state.runtime_scope(&moved).unwrap();
    assert_eq!(scope, relocated);
    let memories = call(&engine_home, &paths, &relocated, "memory_read", json!({})).await;
    assert_eq!(memories["results"].as_array().unwrap().len(), 1);
    assert_eq!(memories["results"][0]["id"], new["id"]);
    let found = call(
        &engine_home,
        &paths,
        &relocated,
        "memory_read",
        json!({"query":"What is the approved brand color?"}),
    )
    .await;
    assert_eq!(found["results"][0]["id"], new["id"]);
    assert_ne!(restored["state"]["envelope"]["project"], json!(moved));
}
