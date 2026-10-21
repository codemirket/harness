//! Opt-in integration: installed pinned image, disposable state, no user memories.
use mirket::{backup, paths::Paths, state::State};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::io::AsyncWriteExt;

async fn call(
    engine_home: &Paths,
    fixture: &Paths,
    scope: &str,
    operation: &str,
    args: Value,
) -> Value {
    let (engine, mut argv, name) =
        mirket::packages::agent::command(engine_home, Some(scope)).unwrap();
    // Keep production image/containment/engine settings, replacing only the synthetic data mount.
    let empty = mirket::packages::agent::data(engine_home).join(scope);
    assert!(fs::read_dir(&empty).unwrap().next().is_none());
    fs::remove_dir(&empty).unwrap();
    let directory = mirket::packages::agent::data(fixture).join(scope);
    fs::create_dir_all(&directory).unwrap();
    let volume = argv.iter().position(|a| a == "--volume").unwrap() + 1;
    argv[volume] = format!("{}:/data/{scope}:rw", directory.display());
    let mut command = tokio::process::Command::from(engine.command(&argv));
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let mut child = command.spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    let request = json!({"operation":operation,"scope":scope,"arguments":args});
    input
        .write_all(format!("{request}\n").as_bytes())
        .await
        .unwrap();
    drop(input);
    let result = tokio::time::timeout(Duration::from_secs(120), child.wait_with_output()).await;
    // Cleanup the exact generated test container on failure; never another installation service.
    let _ = engine.run(
        &["rm".into(), "--force".into(), name],
        Duration::from_secs(10),
    );
    let output = result.expect("contained fixture deadline").unwrap();
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result.get("error").is_none(), "{result}");
    assert_eq!(result["runtime"]["versions"]["mem0ai"], "2.2.1");
    assert_eq!(result["runtime"]["versions"]["langgraph"], "1.2.16");
    result["result"].clone()
}
fn sqlite_rows(path: &Path, table: &str) -> i64 {
    let db = rusqlite::Connection::open(path).unwrap();
    db.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
#[tokio::test]
#[ignore = "explicit registered engine; exercises real pinned Mem0/LangGraph against disposable state"]
async fn backup_runtime_recovers_history_pending_effect_and_relocated_memory() {
    let home = std::env::var_os("MIRKET_TEST_ENGINE_HOME").expect("explicit test engine home");
    let engine_home = Paths::new(PathBuf::from(home)).unwrap();
    let staging = engine_home.root.join("execution/podman/staging");
    let fixture = tempfile::Builder::new()
        .prefix("backup-fixture-")
        .tempdir_in(staging)
        .unwrap();
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
    let start=call(&engine_home,&paths,&scope,"workflow_start",json!({"run_id":run_id,"envelope":{"request":{"objective":"Exercise recovery","memory_recall":"off"},"project":project,"source_sha256":env!("MIRKET_SOURCE_SHA256"),"execution_sha256":env!("MIRKET_EXECUTION_SHA256"),"host_verification_required":true},"snapshot":{"turn":0},"deadline":mirket::router::durable::now()+3600})).await;
    let pending=call(&engine_home,&paths,&scope,"workflow_resume",json!({"run_id":run_id,"expected_checkpoint_id":start["checkpoint_id"],"key":"effect","reply":{"decision":"effect","snapshot":{"turn":1},"effect":{"tool":"external-effect","may_mutate":true}}})).await;
    let run = paths.root.join("runs").join(run_id);
    fs::create_dir_all(&run).unwrap();
    fs::write(run.join("trace.json"), b"synthetic pending-effect evidence").unwrap();
    let directory = mirket::packages::agent::data(&paths).join(&scope);
    let history = sqlite_rows(&directory.join("history.sqlite"), "history");
    let checkpoints = sqlite_rows(&directory.join("workflows.sqlite"), "checkpoints");
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
        sqlite_rows(&directory.join("history.sqlite"), "history"),
        history
    );
    assert_eq!(
        sqlite_rows(&directory.join("workflows.sqlite"), "checkpoints"),
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
