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
    let listed = recovered.tasks(&f.project_id, 0, 10).unwrap();
    assert_eq!(listed["tasks"][0]["id"], task["id"]);
    assert_eq!(listed["next_offset"], Value::Null);
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
    let first = f.state.tasks(&f.project_id, 0, 2).unwrap();
    let second = f.state.tasks(&f.project_id, 2, 2).unwrap();
    assert_eq!(first["next_offset"], 2);
    assert_eq!(second["next_offset"], Value::Null);
    for task in first["tasks"].as_array().unwrap() {
        assert!(
            !second["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|other| other["id"] == task["id"])
        );
    }
    assert!(f.state.tasks(&f.project_id, 0, 21).is_err());
    assert!(f.state.projects_page(0, 11).is_err());
    assert_eq!(
        f.state.projects_page(0, 1).unwrap()["projects"][0]["id"],
        f.project_id
    );
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
    assert_eq!(tools.len(), 9);
    assert!(
        !tools
            .iter()
            .any(|tool| tool.name.contains("exec") || tool.name.contains("register"))
    );
    let call = |name: &'static str, args: Value| {
        CallToolRequestParams::new(name).with_arguments(args.as_object().unwrap().clone())
    };
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
