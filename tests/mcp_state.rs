use mirket::{
    catalog::Catalog,
    mcp::{BoundedLines, MAX_FRAME_BYTES, MirketServer, protocol_probe},
    paths::Paths,
    state::State,
};
use rmcp::{
    ClientLifecycleMode, ClientServiceExt, ServiceExt,
    model::{CallToolRequestParams, ProtocolVersion, ReadResourceRequestParams},
};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Barrier},
};

struct Fixture {
    _home: tempfile::TempDir,
    paths: Paths,
    project: PathBuf,
    state: State,
    project_id: String,
}
impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let project = paths.home.join("project");
        fs::create_dir(&project).unwrap();
        let state = State::open(&paths).unwrap();
        let registered = state.register_project(&project).unwrap();
        Self {
            _home: home,
            paths,
            project,
            state,
            project_id: registered["id"].as_str().unwrap().into(),
        }
    }
    fn begin(&self, key: &str) -> Value {
        self.state
            .begin(
                &self.project_id,
                "Produce a verified calculation",
                &["engineering-judgment".into()],
                "The saved output reconciles to the input units",
                "A mismatched unit input is rejected",
                key,
            )
            .unwrap()
    }
    fn deliver(&self, id: &str, rev: i64) -> Value {
        let skill = Catalog::embedded()
            .unwrap()
            .read("engineering-judgment", "SKILL.md", 65536)
            .unwrap();
        self.state
            .record_skill(id, rev, "engineering-judgment", &skill.sha256, "read-skill")
            .unwrap()
    }
}
fn id(task: &Value) -> &str {
    task["id"].as_str().unwrap()
}

#[test]
fn unregister_removes_only_selected_coordination_records_and_preserves_project_files() {
    let f = Fixture::new();
    let task = f.begin("start");
    f.deliver(id(&task), 1);
    fs::write(f.project.join("result.txt"), "observed output").unwrap();
    f.state
        .evidence(
            id(&task),
            2,
            "result.txt",
            "acceptance",
            "Inspected output",
            "evidence",
        )
        .unwrap();
    f.state
        .checkpoint(id(&task), 3, "Next verify the failure probe", "checkpoint")
        .unwrap();
    let other_root = f.paths.home.join("unrelated");
    fs::create_dir(&other_root).unwrap();
    fs::write(other_root.join("keep.txt"), "keep unchanged").unwrap();
    let other_project = f.state.register_project(&other_root).unwrap();
    let other_id = other_project["id"].as_str().unwrap();
    let other = f
        .state
        .begin(
            other_id,
            "Keep this work",
            &["engineering-judgment".into()],
            "a",
            "f",
            "start",
        )
        .unwrap();
    let other = f
        .state
        .checkpoint(id(&other), 1, "Unrelated progress", "checkpoint")
        .unwrap();
    let deletion = f.state.unregister_project(&f.project_id).unwrap();
    assert_eq!(
        deletion["deleted"],
        json!({"projects":1,"tasks":1,"skill_deliveries":1,"evidence":1,"checkpoints":1,"operations":4})
    );
    assert_eq!(deletion["project_files"], "unchanged");
    assert_eq!(
        fs::read_to_string(f.project.join("result.txt")).unwrap(),
        "observed output"
    );
    assert_eq!(
        fs::read_to_string(other_root.join("keep.txt")).unwrap(),
        "keep unchanged"
    );
    assert_eq!(f.state.status(id(&other)).unwrap(), other);
    assert_eq!(f.state.project_id(&other_root).unwrap(), other_id);
    assert_eq!(
        f.state
            .checkpoint(id(&other), 1, "Unrelated progress", "checkpoint")
            .unwrap(),
        other,
        "unrelated idempotency receipt survives"
    );
    assert!(f.state.status(id(&task)).is_err());
    assert!(
        f.state
            .unregister_project(&f.project_id)
            .unwrap_err()
            .to_string()
            .contains("not registered")
    );
    // Re-registering the same path starts a clean coordination namespace.
    assert_eq!(
        f.state.register_project(&f.project).unwrap()["id"],
        f.project_id
    );
    let fresh = f.begin("start");
    assert_eq!(fresh["revision"], 1);
    assert!(fresh["skill_deliveries"].as_array().unwrap().is_empty());
    assert!(fresh["evidence"].as_array().unwrap().is_empty());
    assert!(fresh["recent_checkpoints"].as_array().unwrap().is_empty());
}

#[test]
fn unregister_uses_stored_identity_even_after_project_root_moves_or_disappears() {
    let f = Fixture::new();
    let task = f.begin("start");
    let moved = f.paths.home.join("moved-project");
    fs::rename(&f.project, &moved).unwrap();
    let deletion = f.state.unregister_project(&f.project_id).unwrap();
    assert_eq!(deletion["root"], f.project.to_str().unwrap());
    assert_eq!(deletion["deleted"]["tasks"], 1);
    assert!(moved.is_dir());
    assert!(f.state.status(id(&task)).is_err());
    let empty = f.paths.home.join("empty");
    fs::create_dir(&empty).unwrap();
    let registered = f.state.register_project(&empty).unwrap();
    fs::remove_dir(&empty).unwrap();
    assert_eq!(
        f.state
            .unregister_project(registered["id"].as_str().unwrap())
            .unwrap()["deleted"]["projects"],
        1
    );
}

#[test]
fn durable_completion_requires_delivered_guidance_and_current_artifacts() {
    let f = Fixture::new();
    let task = f.begin("start");
    assert!(
        f.state
            .complete(id(&task), 1, "finish")
            .unwrap_err()
            .to_string()
            .contains("delivered")
    );
    f.deliver(id(&task), 1);
    assert!(
        f.state
            .complete(id(&task), 2, "finish")
            .unwrap_err()
            .to_string()
            .contains("acceptance")
    );
    fs::write(
        f.project.join("result.txt"),
        "expected:42 actual:42 units:EUR",
    )
    .unwrap();
    fs::write(f.project.join("probe.txt"), "currency mismatch rejected").unwrap();
    f.state
        .evidence(
            id(&task),
            2,
            "result.txt",
            "acceptance",
            "Observed exact units and result",
            "result",
        )
        .unwrap();
    assert!(
        f.state
            .complete(id(&task), 3, "finish")
            .unwrap_err()
            .to_string()
            .contains("failure_probe")
    );
    f.state
        .evidence(
            id(&task),
            3,
            "probe.txt",
            "failure_probe",
            "Observed mismatch rejected",
            "probe",
        )
        .unwrap();
    fs::write(f.project.join("result.txt"), "changed output").unwrap();
    assert!(
        f.state
            .complete(id(&task), 4, "finish")
            .unwrap_err()
            .to_string()
            .contains("stale")
    );
    assert_eq!(f.state.status(id(&task)).unwrap()["status"], "active");
    f.state
        .evidence(
            id(&task),
            4,
            "result.txt",
            "acceptance",
            "Inspected changed output",
            "result-2",
        )
        .unwrap();
    let complete = f.state.complete(id(&task), 5, "finish").unwrap();
    assert_eq!(complete["status"], "complete");
    assert_eq!(complete["revision"], 6);
    assert_eq!(
        State::open(&f.paths).unwrap().status(id(&task)).unwrap(),
        complete
    );
    fs::remove_file(f.project.join("result.txt")).unwrap();
    assert_eq!(
        f.state.complete(id(&task), 5, "finish").unwrap(),
        complete,
        "exact replay returns the operation receipt, not a new verification"
    );
    f.state
        .reopen(
            id(&task),
            6,
            "Artifact removed; produce and verify it again",
            "reopen",
        )
        .unwrap();
    assert!(f.state.complete(id(&task), 7, "finish-again").is_err());
}

#[test]
fn idempotency_and_optimistic_revision_prevent_duplicate_or_lost_updates() {
    let f = Fixture::new();
    let task = f.begin("start");
    assert_eq!(f.begin("start"), task);
    let first = f
        .state
        .checkpoint(id(&task), 1, "decision recorded", "note")
        .unwrap();
    assert_eq!(
        f.state
            .checkpoint(id(&task), 1, "decision recorded", "note")
            .unwrap(),
        first
    );
    assert!(
        f.state
            .checkpoint(id(&task), 1, "different decision", "note")
            .unwrap_err()
            .to_string()
            .contains("idempotency")
    );
    assert!(
        f.state
            .checkpoint(id(&task), 1, "stale decision", "new-note")
            .unwrap_err()
            .to_string()
            .contains("revision conflict")
    );
    let barrier = Arc::new(Barrier::new(2));
    let workers: Vec<_> = (0..2)
        .map(|i| {
            let state = f.state.clone();
            let barrier = barrier.clone();
            let task_id = id(&task).to_owned();
            std::thread::spawn(move || {
                barrier.wait();
                state.checkpoint(&task_id, 2, &format!("decision{i}"), &format!("key{i}"))
            })
        })
        .collect();
    let replies: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(replies.iter().filter(|reply| reply.is_ok()).count(), 1);
    assert!(
        replies
            .iter()
            .find_map(|reply| reply.as_ref().err())
            .unwrap()
            .to_string()
            .contains("revision conflict")
    );
    assert_eq!(f.state.status(id(&task)).unwrap()["revision"], 3);
}

#[test]
fn interrupted_task_reopens_with_same_project_and_partial_evidence() {
    let f = Fixture::new();
    let task = f.begin("start");
    f.deliver(id(&task), 1);
    fs::write(f.project.join("output"), "actual output").unwrap();
    f.state
        .evidence(
            id(&task),
            2,
            "output",
            "acceptance",
            "Inspected persisted output",
            "evidence",
        )
        .unwrap();
    let recovered = State::open(&f.paths).unwrap();
    let snapshot = recovered.status(id(&task)).unwrap();
    assert_eq!(snapshot["status"], "active");
    assert_eq!(snapshot["revision"], 3);
    assert_eq!(snapshot["evidence"][0]["path"], "output");
    assert_eq!(
        snapshot["skill_deliveries"][0]["skill_id"],
        "engineering-judgment"
    );
    assert!(recovered.complete(id(&task), 3, "finish").is_err());
    let listed = recovered.tasks(&f.project_id, None, 10).unwrap();
    assert_eq!(listed["tasks"][0]["id"], task["id"]);
    assert_eq!(listed["next_cursor"], Value::Null);
}

#[test]
fn evidence_retries_replay_receipts_while_completion_checks_current_bytes() {
    let f = Fixture::new();
    let task = f.begin("start");
    fs::write(f.project.join("output"), "observed output").unwrap();
    let receipt = f
        .state
        .evidence(
            id(&task),
            1,
            "output",
            "acceptance",
            "Verified output",
            "record",
        )
        .unwrap();
    fs::remove_file(f.project.join("output")).unwrap();
    assert_eq!(
        f.state
            .evidence(
                id(&task),
                1,
                "output",
                "acceptance",
                "Verified output",
                "record"
            )
            .unwrap(),
        receipt
    );
    assert!(
        f.state
            .evidence(
                id(&task),
                2,
                "output",
                "acceptance",
                "Verified output",
                "new-record"
            )
            .is_err()
    );
}

#[test]
fn paginated_task_and_project_discovery_survives_lost_session_ids() {
    let f = Fixture::new();
    for index in 0..4 {
        f.begin(&format!("task-{index}"));
    }
    let first = f.state.tasks(&f.project_id, None, 2).unwrap();
    let second = f
        .state
        .tasks(&f.project_id, first["next_cursor"].as_str(), 2)
        .unwrap();
    assert!(first["next_cursor"].is_string());
    assert_eq!(second["next_cursor"], Value::Null);
    for task in first["tasks"].as_array().unwrap() {
        assert!(
            !second["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|other| other["id"] == task["id"])
        );
    }
    assert!(f.state.tasks(&f.project_id, None, 21).is_err());
    assert!(f.state.projects_page(0, 11).is_err());
    assert_eq!(
        f.state.projects_page(0, 1).unwrap()["projects"][0]["id"],
        f.project_id
    );
}

#[test]
fn runtime_state_read_open_does_not_need_the_writer_lock() {
    let f = Fixture::new();
    let task = f.begin("read-with-writer");
    let writer = rusqlite::Connection::open(f.paths.database()).unwrap();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    let reader = State::open(&f.paths).expect("a valid WAL state must open without a writer lock");
    assert_eq!(reader.status(id(&task)).unwrap(), task);
    writer.execute_batch("ROLLBACK").unwrap();
}

#[test]
fn runtime_state_task_pages_do_not_skip_a_task_updated_between_pages() {
    let f = Fixture::new();
    for index in 0..4 {
        f.begin(&format!("pagination-{index}"));
    }
    let before = f.state.tasks(&f.project_id, None, 20).unwrap();
    let first = f.state.tasks(&f.project_id, None, 1).unwrap();
    let updated = before["tasks"].as_array().unwrap().last().unwrap();
    // Make the ordering change deterministic without a wall-clock sleep.
    let database = rusqlite::Connection::open(f.paths.database()).unwrap();
    database.execute("UPDATE tasks SET updated=0", []).unwrap();
    f.state
        .checkpoint(id(updated), 1, "Progress after the first page", "progress")
        .unwrap();
    let mut seen = vec![id(&first["tasks"][0]).to_owned()];
    let mut cursor = first["next_cursor"].as_str().map(str::to_owned);
    while let Some(current) = cursor {
        let page = f.state.tasks(&f.project_id, Some(&current), 1).unwrap();
        seen.extend(
            page["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| id(v).to_owned()),
        );
        cursor = page["next_cursor"].as_str().map(str::to_owned);
    }
    seen.sort();
    let mut expected: Vec<_> = before["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| id(v).to_owned())
        .collect();
    expected.sort();
    assert_eq!(
        seen, expected,
        "every original task must appear exactly once"
    );
}

#[test]
fn runtime_state_cursor_handles_creation_ties_and_inserts_and_is_project_bound() {
    let f = Fixture::new();
    let database = rusqlite::Connection::open(f.paths.database()).unwrap();
    for index in 0..6 {
        let task = f.begin(&format!("ordered-{index}"));
        database
            .execute(
                "UPDATE tasks SET created=?2 WHERE id=?1",
                rusqlite::params![id(&task), 100 + index / 2],
            )
            .unwrap();
    }
    let all = f.state.tasks(&f.project_id, None, 20).unwrap();
    let first = f.state.tasks(&f.project_id, None, 2).unwrap();
    let cursor = first["next_cursor"].as_str().unwrap();
    let inserted = f.begin("newer-insert");
    let other_root = f.project.join("another-project");
    fs::create_dir(&other_root).unwrap();
    let other = f.state.register_project(&other_root).unwrap();
    assert!(
        f.state
            .tasks(other["id"].as_str().unwrap(), Some(cursor), 2)
            .unwrap_err()
            .to_string()
            .contains("cross-project")
    );
    for invalid in [
        "",
        "0",
        "tasks1",
        "tasks1:wrong:0:t_00000000000000000000000000000000",
        &"x".repeat(129),
    ] {
        assert!(
            f.state.tasks(&f.project_id, Some(invalid), 2).is_err(),
            "{invalid}"
        );
    }
    let mut seen: Vec<_> = first["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| id(v).to_owned())
        .collect();
    let mut cursor = Some(cursor.to_owned());
    while let Some(position) = cursor {
        let page = f.state.tasks(&f.project_id, Some(&position), 2).unwrap();
        seen.extend(
            page["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| id(v).to_owned()),
        );
        cursor = page["next_cursor"].as_str().map(str::to_owned);
    }
    let expected: Vec<_> = all["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| id(v).to_owned())
        .collect();
    assert_eq!(seen, expected);
    assert!(!seen.iter().any(|task| task == id(&inserted)));
}

#[test]
fn runtime_state_initialization_rejects_incompatible_schema_without_repair() {
    for mutation in [
        "DROP TABLE checkpoints",
        "ALTER TABLE tasks ADD COLUMN unexpected TEXT",
        "CREATE TRIGGER unexpected AFTER INSERT ON tasks BEGIN DELETE FROM operations; END",
        "CREATE TABLE sqlitexunexpected (value TEXT)",
        "PRAGMA user_version=0",
    ] {
        let f = Fixture::new();
        let database = rusqlite::Connection::open(f.paths.database()).unwrap();
        database.execute_batch(mutation).unwrap();
        let before: String = database.query_row("SELECT group_concat(sql,';') FROM (SELECT sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY name)",[],|row|row.get(0)).unwrap();
        let error = State::open(&f.paths).unwrap_err().to_string();
        assert!(error.contains("schema"), "{mutation}: {error}");
        let after: String = database.query_row("SELECT group_concat(sql,';') FROM (SELECT sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY name)",[],|row|row.get(0)).unwrap();
        assert_eq!(
            before, after,
            "incompatible state must not be repaired implicitly"
        );
    }
}

#[test]
fn runtime_state_concurrent_initializers_share_one_valid_schema() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
    let barrier = Arc::new(Barrier::new(8));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let paths = paths.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                State::open(&paths)
            })
        })
        .collect();
    for thread in threads {
        thread
            .join()
            .unwrap()
            .expect("parallel initialization must succeed");
    }
    State::open(&paths).unwrap();
    let database = rusqlite::Connection::open(paths.database()).unwrap();
    let journal: String = database
        .pragma_query_value(None, "journal_mode", |row| row.get(0))
        .unwrap();
    assert_eq!(journal, "wal");
    assert_eq!(
        database
            .pragma_query_value::<i64, _>(None, "user_version", |row| row.get(0))
            .unwrap(),
        2
    );
}

#[test]
fn runtime_state_cli_task_start_scopes_verification_criteria() {
    let f = Fixture::new();
    let invoke = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_mirket"))
            .env("MIRKET_DIRECT", "1")
            .arg("--home")
            .arg(&f.paths.home)
            .args(args)
            .output()
            .unwrap()
    };
    let start = |key: &str, overrides: &[&str]| {
        let mut args = vec![
            "task",
            "start",
            "--project",
            f.project.to_str().unwrap(),
            "--capability",
            "Accessibility Engineer",
            "--objective",
            "Review accessible markup from source",
            "--key",
            key,
        ];
        args.extend_from_slice(overrides);
        invoke(&args)
    };
    let defaults = start("default-criteria", &[]);
    assert!(
        defaults.status.success(),
        "{}",
        String::from_utf8_lossy(&defaults.stderr)
    );
    let defaults: Value = serde_json::from_slice(&defaults.stdout).unwrap();
    let catalog = Catalog::embedded().unwrap();
    let capability = catalog.capability("Accessibility Engineer").unwrap();
    assert_eq!(defaults["acceptance"], capability.acceptance.join("; "));
    assert_eq!(defaults["failure_probe"], capability.failure_probe);

    let acceptance = "Source-only review: accessible names and form labels match the saved markup.\nKeyboard and assistive-technology behavior remain untested.";
    let failure_probe =
        "A missing explicit accessible name is reported without claiming runtime accessibility.";
    let scoped = start(
        "scoped-criteria",
        &["--acceptance", acceptance, "--failure-probe", failure_probe],
    );
    assert!(
        scoped.status.success(),
        "{}",
        String::from_utf8_lossy(&scoped.stderr)
    );
    let scoped: Value = serde_json::from_slice(&scoped.stdout).unwrap();
    let persisted = invoke(&["task", "status", id(&scoped)]);
    assert!(persisted.status.success());
    let persisted: Value = serde_json::from_slice(&persisted.stdout).unwrap();
    assert_eq!(persisted["acceptance"], acceptance);
    assert_eq!(persisted["failure_probe"], failure_probe);
    assert_eq!(persisted["skills"], json!([capability.lead]));

    for (key, overrides, missing) in [
        (
            "acceptance-only",
            vec!["--acceptance", acceptance],
            "--failure-probe",
        ),
        (
            "failure-only",
            vec!["--failure-probe", failure_probe],
            "--acceptance",
        ),
    ] {
        let rejected = start(key, &overrides);
        assert_eq!(rejected.status.code(), Some(2));
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(error.contains("required arguments"), "{error}");
        assert!(error.contains(missing), "{error}");
    }
    for (key, accept, probe, expected) in [
        (
            "empty-acceptance",
            "",
            failure_probe,
            "acceptance must contain",
        ),
        (
            "empty-probe",
            acceptance,
            " \n",
            "failure probe must contain",
        ),
    ] {
        let rejected = start(key, &["--acceptance", accept, "--failure-probe", probe]);
        assert_eq!(rejected.status.code(), Some(1));
        let error = String::from_utf8_lossy(&rejected.stderr);
        assert!(error.contains(expected), "{error}");
    }
    assert_eq!(
        f.state.tasks(&f.project_id, None, 20).unwrap()["tasks"]
            .as_array()
            .unwrap()
            .len(),
        2,
        "rejected overrides must not create tasks"
    );
}

#[test]
fn runtime_state_cli_uses_cursor_and_reads_while_a_writer_is_active() {
    let f = Fixture::new();
    for index in 0..3 {
        f.begin(&format!("cli-page-{index}"));
    }
    let invoke = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_mirket"))
            .arg("--home")
            .arg(&f.paths.home)
            .args(args)
            .output()
            .unwrap()
    };
    let root = f.project.to_str().unwrap();
    let first = invoke(&["task", "list", "--project", root, "--limit", "1"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first: Value = serde_json::from_slice(&first.stdout).unwrap();
    let cursor = first["next_cursor"].as_str().unwrap();
    assert!(first.get("next_offset").is_none());
    let next = invoke(&[
        "task",
        "list",
        "--project",
        root,
        "--limit",
        "1",
        "--cursor",
        cursor,
    ]);
    assert!(
        next.status.success(),
        "{}",
        String::from_utf8_lossy(&next.stderr)
    );
    let next: Value = serde_json::from_slice(&next.stdout).unwrap();
    assert_ne!(first["tasks"][0]["id"], next["tasks"][0]["id"]);
    assert!(
        !invoke(&["task", "list", "--project", root, "--offset", "1"])
            .status
            .success()
    );
    let writer = rusqlite::Connection::open(f.paths.database()).unwrap();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    let status = invoke(&["task", "status", id(&first["tasks"][0])]);
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    writer.execute_batch("ROLLBACK").unwrap();
}

#[tokio::test]
async fn runtime_state_mcp_advertises_and_follows_only_task_cursors() {
    let f = Fixture::new();
    for index in 0..3 {
        f.begin(&format!("mcp-page-{index}"));
    }
    let server = MirketServer::new(&f.paths, Catalog::embedded().unwrap()).unwrap();
    let (a, b) = tokio::io::duplex(8192);
    let host = tokio::spawn(server.serve(a));
    let mut client = ().serve(b).await.unwrap();
    let mut host = host.await.unwrap().unwrap();
    let tools = client.list_all_tools().await.unwrap();
    let task_status = tools
        .iter()
        .find(|tool| tool.name == "task_status")
        .unwrap();
    let schema = serde_json::to_value(&task_status.input_schema).unwrap();
    assert!(schema["properties"].get("cursor").is_some());
    assert!(schema["properties"].get("offset").is_none());
    let request = |args: Value| {
        CallToolRequestParams::new("task_status").with_arguments(args.as_object().unwrap().clone())
    };
    let first = client
        .call_tool(request(json!({"project_id":f.project_id,"limit":1})))
        .await
        .unwrap();
    assert_eq!(first.is_error, Some(false));
    let first = first.structured_content.unwrap();
    let second = client
        .call_tool(request(
            json!({"project_id":f.project_id,"limit":1,"cursor":first["next_cursor"]}),
        ))
        .await
        .unwrap();
    assert_eq!(second.is_error, Some(false));
    assert_ne!(
        first["tasks"][0]["id"],
        second.structured_content.unwrap()["tasks"][0]["id"]
    );
    let offset = client
        .call_tool(request(json!({"project_id":f.project_id,"offset":1})))
        .await;
    assert!(offset.is_err() || offset.unwrap().is_error == Some(true));
    let invalid = client
        .call_tool(request(
            json!({"project_id":f.project_id,"cursor":"invalid"}),
        ))
        .await
        .unwrap();
    assert_eq!(invalid.is_error, Some(true));
    client.close().await.unwrap();
    host.close().await.unwrap();
}

#[test]
fn unsupported_state_formats_fail_closed_and_database_is_private() {
    let f = Fixture::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(f.paths.database())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let conn = rusqlite::Connection::open(f.paths.database()).unwrap();
    conn.pragma_update(None, "user_version", 99).unwrap();
    assert!(
        State::open(&f.paths)
            .unwrap_err()
            .to_string()
            .contains("unsupported")
    );
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 99);
}

#[test]
fn project_registry_and_file_evidence_reject_identity_and_path_escape() {
    let f = Fixture::new();
    assert!(
        f.state
            .begin(
                "p_unregistered",
                "x",
                &["engineering-judgment".into()],
                "a",
                "f",
                "k"
            )
            .is_err()
    );
    let task = f.begin("start");
    fs::write(f.paths.home.join("outside.txt"), "outside").unwrap();
    for path in [
        "../outside.txt",
        f.paths.home.join("outside.txt").to_str().unwrap(),
    ] {
        assert!(
            f.state
                .evidence(id(&task), 1, path, "acceptance", "outside", "escape")
                .is_err()
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(f.paths.home.join("outside.txt"), f.project.join("link.txt"))
            .unwrap();
        assert!(
            f.state
                .evidence(id(&task), 1, "link.txt", "acceptance", "symlink", "link")
                .is_err()
        );
        let _socket = std::os::unix::net::UnixListener::bind(f.project.join("socket")).unwrap();
        assert!(
            f.state
                .evidence(
                    id(&task),
                    1,
                    "socket",
                    "acceptance",
                    "not a regular file",
                    "socket"
                )
                .unwrap_err()
                .to_string()
                .contains("regular file")
        );
    }
    fs::rename(&f.project, f.paths.home.join("moved-project")).unwrap();
    fs::create_dir(&f.project).unwrap();
    assert!(
        f.state
            .checkpoint(
                id(&task),
                1,
                "new directory is not the registered one",
                "changed-root"
            )
            .unwrap_err()
            .to_string()
            .contains("identity")
    );
    assert!(f.state.register_project(&f.project).is_err());
}

#[test]
fn state_validates_input_and_skill_identity_for_every_caller() {
    let f = Fixture::new();
    for skills in [
        vec![],
        vec!["not-a-skill".into()],
        vec!["engineering-judgment".into(); 2],
    ] {
        assert!(
            f.state
                .begin(&f.project_id, "x", &skills, "a", "f", "invalid")
                .is_err()
        );
    }
    assert!(
        f.state
            .begin(
                &f.project_id,
                &"x".repeat(4097),
                &["engineering-judgment".into()],
                "a",
                "f",
                "invalid"
            )
            .is_err()
    );
    let task = f.begin("start");
    assert!(
        f.state
            .record_skill(
                id(&task),
                1,
                "engineering-judgment",
                &"0".repeat(64),
                "bad-hash"
            )
            .is_err()
    );
    assert!(
        f.state
            .checkpoint(id(&task), 1, &"x".repeat(2049), "large-note")
            .is_err()
    );
    let large = fs::File::create(f.project.join("large")).unwrap();
    large.set_len(16 * 1024 * 1024 + 1).unwrap();
    assert!(
        f.state
            .evidence(id(&task), 1, "large", "acceptance", "too large", "large")
            .is_err()
    );
    assert_eq!(f.state.status(id(&task)).unwrap()["revision"], 1);
}

#[test]
fn serialized_response_budget_rolls_back_writes_before_they_become_unreadable() {
    let f = Fixture::new();
    let task = f.begin("start");
    let mut previous = task.clone();
    let mut rejected = false;
    for index in 0..8 {
        let revision = previous["revision"].as_i64().unwrap();
        match f.state.checkpoint(
            id(&task),
            revision,
            &"\u{1}".repeat(2048),
            &format!("note-{index}"),
        ) {
            Ok(value) => previous = value,
            Err(error) => {
                assert!(error.to_string().contains("serialized budget"));
                assert_eq!(f.state.status(id(&task)).unwrap(), previous);
                f.state
                    .checkpoint(
                        id(&task),
                        revision,
                        "Shorter observable next step",
                        &format!("note-{index}"),
                    )
                    .unwrap();
                rejected = true;
                break;
            }
        }
    }
    assert!(rejected);
    assert!(
        serde_json::to_vec(&f.state.status(id(&task)).unwrap())
            .unwrap()
            .len()
            <= mirket::mcp::MAX_RESULT_BYTES
    );
}

#[tokio::test]
async fn stdio_frame_boundary_limits_single_lines_without_limiting_session_bytes() {
    use tokio::io::AsyncReadExt;
    let huge = vec![b'x'; MAX_FRAME_BYTES + 1];
    let mut reader = BoundedLines::new(huge.as_slice());
    assert_eq!(
        reader
            .read_to_end(&mut Vec::new())
            .await
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::InvalidData
    );
    let mut several = Vec::new();
    for _ in 0..3 {
        several.extend(vec![b'x'; MAX_FRAME_BYTES / 2]);
        several.push(b'\n');
    }
    let mut reader = BoundedLines::new(several.as_slice());
    let mut received = Vec::new();
    reader.read_to_end(&mut received).await.unwrap();
    assert_eq!(received, several);
}

#[tokio::test]
async fn official_mcp_client_discovers_calls_and_reads_scoped_resources() {
    let f = Fixture::new();
    let server = MirketServer::new(&f.paths, Catalog::embedded().unwrap()).unwrap();
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let handle = tokio::spawn(async move {
        server
            .serve(server_transport)
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap()
    });
    let client = ().serve(client_transport).await.unwrap();
    let tools = client.list_all_tools().await.unwrap();
    assert_eq!(tools.len(), 14);
    assert!(
        !tools
            .iter()
            .any(|tool| tool.name.contains("exec") || tool.name.contains("register"))
    );
    let call = |name: &'static str, args: Value| {
        CallToolRequestParams::new(name).with_arguments(args.as_object().unwrap().clone())
    };
    let facets = client
        .call_tool(call("team_facets", json!({})))
        .await
        .unwrap();
    assert_eq!(facets.is_error, Some(false));
    assert!(
        facets.structured_content.unwrap()["facets"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["id"] == "web-interface")
    );
    let team = client.call_tool(call("team_plan", json!({
        "objective":"Build a recoverable checkout", "primary":"Frontend Engineer",
        "facets":[{"id":"web-interface","reason":"New screen and state behavior"},{"id":"service-contract","reason":"Order retry contract changes"}]
    }))).await.unwrap();
    assert_eq!(team.is_error, Some(false));
    let team = team.structured_content.unwrap();
    assert_eq!(team["assignments"].as_array().unwrap().len(), 4);
    assert_eq!(team["execution"]["status"], "not_dispatched");
    assert!(
        team["task_start"]["skills"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "agent-coordination")
    );
    let invalid_team = client
        .call_tool(call(
            "team_plan",
            json!({
                "objective":"x","primary":"CFO","facets":[{"id":"workbook","reason":""}]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(invalid_team.is_error, Some(true));
    let described = client
        .call_tool(call("catalog_show", json!({"skill_id":"ecc-rust-testing"})))
        .await
        .unwrap();
    assert_eq!(described.is_error, Some(false));
    let described = described.structured_content.unwrap();
    assert_eq!(described["guidance_access"]["body_readable"], false);
    assert!(described["source_pin"]["commit"].as_str().unwrap().len() == 40);
    assert!(described["caveats"].is_array());
    for skill_id in ["ecc-rust-testing", "missing-skill"] {
        let result = client
            .call_tool(call("skill_read", json!({"skill_id":skill_id})))
            .await
            .unwrap();
        assert_eq!(result.is_error, Some(true));
    }
    for skill_id in ["missing-skill", ""] {
        let result = client
            .call_tool(call("catalog_show", json!({"skill_id":skill_id})))
            .await
            .unwrap();
        assert_eq!(result.is_error, Some(true));
    }
    let plan_tool = tools
        .iter()
        .find(|tool| tool.name == "capability_plan")
        .unwrap();
    assert_eq!(
        plan_tool.input_schema["properties"]["supporting"]["maxItems"],
        12
    );
    let support_skill = client
        .call_tool(call(
            "capability_plan",
            json!({
                "capability": "Backend Engineer", "supporting": ["security-judgment"]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(support_skill.is_error, Some(true));
    assert!(
        serde_json::to_string(&support_skill)
            .unwrap()
            .contains("skill_read")
    );
    let composed = client
        .call_tool(call(
            "capability_plan",
            json!({
                "capability": "Backend Engineer", "supporting": ["Security Engineer"]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(composed.is_error, Some(false));
    assert_eq!(
        composed.structured_content.unwrap()["read_sequence"][1]["id"],
        "security-judgment"
    );
    for (count, rejected) in [(12, false), (13, true)] {
        let bounded = client
            .call_tool(call(
                "capability_plan",
                json!({
                    "capability": "Backend Engineer", "supporting": vec!["Security Engineer"; count]
                }),
            ))
            .await
            .unwrap();
        assert_eq!(bounded.is_error, Some(rejected), "supporting count {count}");
    }
    let started=client.call_tool(call("task_start",json!({"project_id":f.project_id,"objective":"test task","skills":["engineering-judgment"],"acceptance":"saved output matches expected units","failure_probe":"malformed units rejected","key":"start"}))).await.unwrap();
    assert_eq!(started.is_error, Some(false));
    let task = started.structured_content.unwrap();
    let read=client.call_tool(call("skill_read",json!({"skill_id":"engineering-judgment","task_id":task["id"],"revision":1,"key":"read"}))).await.unwrap();
    assert_eq!(read.is_error, Some(false));
    let read = read.structured_content.unwrap();
    assert!(read["content"].as_str().unwrap().len() > 100);
    assert_eq!(read["delivery"]["revision"], 2);
    assert_eq!(
        f.state.status(id(&task)).unwrap()["skill_deliveries"][0]["skill_id"],
        "engineering-judgment"
    );
    let result = client
        .call_tool(call(
            "task_finish",
            json!({"task_id":task["id"],"revision":2,"action":"complete","key":"finish"}),
        ))
        .await
        .unwrap();
    assert_eq!(
        result.is_error,
        Some(true),
        "completion cannot invent evidence"
    );
    let malformed = client
        .call_tool(call("catalog_search", json!({"query":42})))
        .await;
    assert!(malformed.is_err() || malformed.unwrap().is_error == Some(true));
    let unexpected = client
        .call_tool(call(
            "catalog_search",
            json!({"query":"design","command":"touch /tmp/unexpected"}),
        ))
        .await;
    assert!(unexpected.is_err() || unexpected.unwrap().is_error == Some(true));
    let too_large = client
        .call_tool(call("catalog_search", json!({"query":"a".repeat(513)})))
        .await
        .unwrap();
    assert_eq!(too_large.is_error, Some(true));
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new("file:///etc/passwd"))
            .await
            .is_err()
    );
    assert!(
        client
            .read_resource(ReadResourceRequestParams::new(
                "mirket://skills/engineering-judgment/../../Cargo.toml"
            ))
            .await
            .is_err()
    );
    let resource = client
        .read_resource(ReadResourceRequestParams::new(
            "mirket://skills/engineering-judgment/SKILL.md",
        ))
        .await
        .unwrap();
    assert!(
        serde_json::to_string(&resource)
            .unwrap()
            .contains("engineering-judgment")
    );
    client.cancel().await.unwrap();
    handle.await.unwrap();
}

#[tokio::test]
async fn actual_cli_serves_mcp_protocol_over_child_stdio() {
    let home = tempfile::tempdir().unwrap();
    let report = protocol_probe(
        std::path::Path::new(env!("CARGO_BIN_EXE_mirket")),
        home.path(),
        8,
    )
    .await
    .unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["concurrent_requests"], 4);
    assert_eq!(report["malformed_input_rejected"], true);
    assert!(report["request_p95_ms"].as_f64().unwrap().is_finite());
}

#[tokio::test]
async fn current_mcp_discovery_lifecycle_serves_registered_projects() {
    let f = Fixture::new();
    let server = MirketServer::new(&f.paths, Catalog::embedded().unwrap()).unwrap();
    let (server_transport, client_transport) = tokio::io::duplex(8192);
    let handle = tokio::spawn(async move {
        server
            .serve(server_transport)
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap()
    });
    let client = ()
        .serve_with_lifecycle(
            client_transport,
            ClientLifecycleMode::Discover {
                preferred_versions: vec![ProtocolVersion::V_2026_07_28],
            },
        )
        .await
        .unwrap();
    assert_eq!(
        client.peer_info().unwrap().protocol_version,
        ProtocolVersion::V_2026_07_28
    );
    let result = client
        .call_tool(CallToolRequestParams::new("project_list"))
        .await
        .unwrap();
    assert_eq!(result.is_error, Some(false));
    assert_eq!(
        result.structured_content.unwrap()["projects"][0]["id"],
        f.project_id
    );
    client.cancel().await.unwrap();
    handle.await.unwrap();
}

#[test]
#[ignore = "downloads an explicitly selected public upstream payload; run when network verification is requested"]
fn pinned_upstream_fetch_uses_cli_and_rejects_modified_cache() {
    let home = tempfile::tempdir().unwrap();
    let skill = "vercel-composition-patterns";
    let fetch = || {
        std::process::Command::new(env!("CARGO_BIN_EXE_mirket"))
            .arg("--home")
            .arg(home.path())
            .args(["catalog", "fetch", skill])
            .output()
            .unwrap()
    };
    let output = fetch();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    let catalog = Catalog::embedded().unwrap();
    let definition = catalog.skill(skill).unwrap();
    assert_eq!(
        result["sha256"].as_str(),
        definition
            .installed_sha256
            .as_deref()
            .or(definition.sha256.as_deref())
    );
    assert_eq!(result["executed"], false);
    assert!(result["files"].as_u64().unwrap() > 1);
    let body = home
        .path()
        .join(".mirket/cache/skills")
        .join(result["sha256"].as_str().unwrap())
        .join("SKILL.md");
    fs::write(&body, "unreviewed cache change").unwrap();
    let rejected = fetch();
    assert!(!rejected.status.success());
    assert_eq!(fs::read_to_string(body).unwrap(), "unreviewed cache change");
}
