//! Desktop contracts always run; production-package fixtures require explicit opt-in.
use mirket::{catalog::Catalog, lifecycle, mcp::MirketServer, paths::Paths, util};
use rmcp::{ServiceExt, model::CallToolRequestParams};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

fn resolved<'a>(root: &'a Value, value: &'a Value) -> &'a Value {
    if let Some(reference) = value["$ref"].as_str() {
        return root.pointer(reference.strip_prefix('#').unwrap()).unwrap();
    }
    if let Some(choices) = value["anyOf"].as_array() {
        return resolved(root, choices.iter().find(|v| v["type"] != "null").unwrap());
    }
    value
}

async fn assert_contracts(client: &rmcp::service::RunningService<rmcp::RoleClient, ()>) {
    let tools = client.list_all_tools().await.unwrap();
    let schema = serde_json::to_value(
        &tools
            .iter()
            .find(|t| t.name == "runtime_data")
            .unwrap()
            .input_schema,
    )
    .unwrap();
    let cursor = resolved(&schema, &schema["properties"]["cursor"]);
    assert_eq!(
        cursor["type"], "object",
        "desktop must not infer a string cursor"
    );
    assert_eq!(
        resolved(&schema, &cursor["properties"]["phase"])["enum"],
        json!(["memory", "workflow"])
    );
    let schema = serde_json::to_value(
        &tools
            .iter()
            .find(|t| t.name == "runtime_data_apply")
            .unwrap()
            .input_schema,
    )
    .unwrap();
    let plan = resolved(&schema, &schema["properties"]["plan"]);
    assert_eq!(plan["type"], "object");
    assert_eq!(plan["additionalProperties"], false);
    assert_eq!(plan["properties"]["memories"]["type"], "array");
    assert_eq!(plan["properties"]["workflows"]["type"], "array");
}

#[tokio::test]
async fn desktop_lifecycle_schemas_and_roundtrips_are_typed() {
    let temp = tempfile::tempdir().unwrap();
    let paths = Paths::new(temp.path().to_path_buf()).unwrap();
    let server = MirketServer::new(&paths, Catalog::embedded().unwrap()).unwrap();
    let (a, b) = tokio::io::duplex(65536);
    let task = tokio::spawn(async move { server.serve(a).await.unwrap().waiting().await.unwrap() });
    let client = ().serve(b).await.unwrap();
    assert_contracts(&client).await;
    for cursor in [
        json!({"phase":"memory","after":null}),
        json!({"phase":"workflow","after":"routed-fixture","sha256":"a".repeat(64)}),
    ] {
        let typed: lifecycle::Cursor = serde_json::from_value(cursor.clone()).unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), cursor);
    }
    for invalid in [
        json!("{\"phase\":\"memory\",\"after\":null}"),
        json!({"phase":"unknown","after":null}),
        json!({"phase":"memory","after":null,"extra":true}),
    ] {
        assert!(serde_json::from_value::<lifecycle::Cursor>(invalid).is_err());
    }
    let plan = json!({"scope":"a".repeat(64),"policy":{"expired_before":0,"completed_before":0},"memories":[{"id":"fact","sha256":"b".repeat(64)}],"workflows":[{"run_id":"routed-fixture","checkpoint_id":"checkpoint"}]});
    assert_eq!(
        serde_json::to_value(
            serde_json::from_value::<lifecycle::RetentionPlan>(plan.clone()).unwrap()
        )
        .unwrap(),
        plan
    );
    assert!(serde_json::from_value::<lifecycle::RetentionPlan>(json!({"workflows":[]})).is_err());
    client.cancel().await.unwrap();
    task.await.unwrap();
}

struct RuntimeFixture {
    _temp: tempfile::TempDir,
    project: PathBuf,
    data: PathBuf,
}
impl RuntimeFixture {
    fn new(paths: &Paths) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("acceptance-")
            .tempdir_in(paths.root.join("execution/podman/staging"))
            .unwrap();
        let project = temp.path().canonicalize().unwrap();
        let scope = mirket::state::State::open(paths)
            .unwrap()
            .runtime_scope(&project)
            .unwrap();
        let data = mirket::packages::agent::data(paths).join(scope);
        assert!(!data.exists());
        Self {
            _temp: temp,
            project,
            data,
        }
    }
}
impl Drop for RuntimeFixture {
    fn drop(&mut self) {
        if self.data.exists() {
            fs::remove_dir_all(&self.data).expect("remove only this disposable runtime scope");
        }
    }
}

#[tokio::test]
#[ignore = "explicit prepared ARM64 engine; real runtimes with disposable synthetic state"]
async fn prepared_packages_pass_desktop_acceptance() {
    let paths = Paths::new(PathBuf::from(
        std::env::var_os("MIRKET_TEST_ENGINE_HOME").expect("explicit engine home"),
    ))
    .unwrap();
    let binary =
        PathBuf::from(std::env::var_os("MIRKET_TEST_BINARY").expect("explicit candidate binary"));
    // Fresh actual CLI/MCP process, isolated from installed tasks and client settings.
    let home = tempfile::tempdir().unwrap();
    let mut command = tokio::process::Command::new(binary);
    command
        .arg("--home")
        .arg(home.path())
        .args(["mcp", "serve"])
        .kill_on_drop(true);
    let client = ().serve(rmcp::transport::TokioChildProcess::new(command).unwrap()).await.unwrap();
    assert_contracts(&client).await;
    let result = client
        .call_tool(CallToolRequestParams::new("runtime_status"))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(
        result["process"]["source_sha256"],
        env!("MIRKET_SOURCE_SHA256")
    );
    for name in [
        "skill-catalog",
        "context-management",
        "engineering-judgment",
    ] {
        let result = client
            .call_tool(
                CallToolRequestParams::new("skill_read")
                    .with_arguments(json!({"skill_id":name}).as_object().unwrap().clone()),
            )
            .await
            .unwrap();
        assert_eq!(
            result.structured_content.unwrap()["sha256"],
            Catalog::embedded()
                .unwrap()
                .read(name, "SKILL.md", 65536)
                .unwrap()
                .sha256
        );
    }
    client.cancel().await.unwrap();

    let fixture = RuntimeFixture::new(&paths);
    let project = &fixture.project;
    let mut session = mirket::agent_runtime::Session::open(&paths, Some(project))
        .await
        .unwrap();
    let mut ids = std::collections::BTreeSet::new();
    for index in 0..23 {
        let reply = session.call(Some(project),"memory_put",json!({"text":format!("Synthetic acceptance fact number {index}."),"kind":"fact","provenance":"Release acceptance fixture","key":format!("fact-{index}")})).await.unwrap();
        ids.insert(reply["result"]["memory"]["id"].as_str().unwrap().to_owned());
    }
    session.close().await.unwrap();
    let mut seen = std::collections::BTreeSet::new();
    let mut cursor = None;
    let mut pages = 0;
    loop {
        let reply = lifecycle::inspect(
            &paths,
            lifecycle::Inspect {
                project: project.clone(),
                action: lifecycle::Action::Export,
                cursor,
                policy: None,
            },
        )
        .await
        .unwrap();
        pages += 1;
        for item in reply["result"]["items"].as_array().unwrap() {
            assert!(seen.insert(item["value"]["id"].as_str().unwrap().to_owned()));
        }
        cursor = serde_json::from_value(reply["result"]["next_cursor"].clone()).unwrap();
        if cursor.is_none() {
            break;
        }
        assert!(pages < 3);
    }
    assert_eq!(pages, 2);
    assert_eq!(seen, ids);
    let (a, b, c) = tokio::join!(
        mirket::agent_runtime::call(&paths, Some(project), "memory_read", json!({"limit":5})),
        mirket::agent_runtime::call(&paths, Some(project), "memory_read", json!({"limit":5})),
        lifecycle::inspect(
            &paths,
            lifecycle::Inspect {
                project: project.clone(),
                action: lifecycle::Action::Plan,
                cursor: Some(lifecycle::Cursor {
                    phase: lifecycle::Phase::Memory,
                    after: None,
                    sha256: None
                }),
                policy: Some(lifecycle::Policy {
                    expired_before: 0,
                    completed_before: 0
                })
            }
        )
    );
    for reply in [a.unwrap(), b.unwrap()] {
        assert_eq!(reply["result"]["results"].as_array().unwrap().len(), 5);
        assert!(
            reply["result"]["results"]
                .as_array()
                .unwrap()
                .iter()
                .all(|i| ids.contains(i["id"].as_str().unwrap()))
        );
    }
    let preview = c.unwrap();
    let plan: lifecycle::RetentionPlan =
        serde_json::from_value(preview["result"]["plan"].clone()).unwrap();
    assert!(plan.memories.is_empty() && plan.workflows.is_empty());
    // Exact typed plan must keep the digest of the original wire payload.
    let applied = lifecycle::apply(
        &paths,
        lifecycle::Apply {
            project: project.clone(),
            plan,
            expected_sha256: preview["result"]["sha256"].as_str().unwrap().into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(applied["result"]["applied"], true);

    // Exercise real checkpoint IPC and restart with the Rust archive owner.
    let recovery = RuntimeFixture::new(&paths);
    util::ensure_directory(&paths.root.join("runs")).unwrap();
    let run = tempfile::Builder::new()
        .prefix("routed-")
        .tempdir_in(paths.root.join("runs"))
        .unwrap();
    let deadline = mirket::router::durable::now() + 600;
    let snapshot =
        json!({"turn":1,"deadline":deadline,"transcript":[{"result":"evidence".repeat(32768)}]});
    let envelope = json!({"checkpoint_format_version":mirket::router::durable::checkpoint_format_version(),
        "request":{"memory_recall":"off","persistence":"durable"},"host_verification_required":false});
    let mut driver = mirket::router::durable::Driver::start(
        &paths,
        &recovery.project,
        run.path(),
        envelope,
        snapshot.clone(),
        deadline,
    )
    .await
    .unwrap();
    driver
        .advance("continue", snapshot.clone(), json!({}))
        .await
        .unwrap();
    let checkpoint = driver.summary()["checkpoint_id"]
        .as_str()
        .unwrap()
        .to_owned();
    driver.close().await.unwrap();
    let request = mirket::router::durable::RunRequest {
        project: recovery.project.clone(),
        run_id: run
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        expected_checkpoint_id: Some(checkpoint),
    };
    let resumed = mirket::router::durable::Driver::resume(&paths, &recovery.project, &request)
        .await
        .unwrap();
    assert_eq!(
        resumed.state().unwrap()["snapshot"]["transcript"],
        snapshot["transcript"]
    );
    resumed.close().await.unwrap();
    let blob = fs::read_dir(run.path().join("observations"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(blob, b"{}").unwrap();
    assert!(
        mirket::router::durable::Driver::resume(&paths, &recovery.project, &request)
            .await
            .is_err()
    );

    let workspace = paths.root.join("execution/podman/staging/design/workspace");
    let design = tempfile::Builder::new()
        .prefix("acceptance-")
        .tempdir_in(&workspace)
        .unwrap();
    let design_project = design.path().join("project");
    fs::create_dir_all(design_project.join("design")).unwrap();
    fs::write(
        design_project.join("design/tokens.json"),
        r##"{"color":{"accent":{"$type":"color","$value":"#276a83"}}}"##,
    )
    .unwrap();
    // Candidate protocol acceptance must not replace the user's running stack.
    let candidate_image = mirket::packages::design_candidate_image(&paths).unwrap();
    let engine = mirket::podman::Engine::resolve(&paths).unwrap();
    struct TestContainer(mirket::podman::Engine, String);
    impl Drop for TestContainer {
        fn drop(&mut self) {
            let _ = self.0.run(
                &["rm".into(), "--force".into(), self.1.clone()],
                std::time::Duration::from_secs(15),
            );
        }
    }
    let call = async |name: &'static str, arguments: Value| {
        let container = format!(
            "mirket-acceptance-{}",
            &util::sha256(format!("{}-{name}", design.path().display()).as_bytes())[..16]
        );
        let _cleanup = TestContainer(engine.clone(), container.clone());
        let mut args = Vec::new();
        if cfg!(target_os = "macos") || cfg!(windows) {
            args.extend(["--connection".into(), engine.connection.clone()]);
        }
        args.extend([
            "run".into(),
            "--rm".into(),
            "-i".into(),
            "--init".into(),
            "--pull=never".into(),
            "--name".into(),
            container,
            "--network=none".into(),
            "--userns=keep-id:uid=1000,gid=1000".into(),
            "--user=1000:1000".into(),
            "--read-only".into(),
            "--cap-drop=ALL".into(),
            "--security-opt=no-new-privileges".into(),
            "--pids-limit=128".into(),
            "--memory=1g".into(),
            "--memory-swap=1g".into(),
            "--tmpfs=/tmp:rw,size=256m,mode=1777".into(),
            "--volume".into(),
            format!("{}:/workspace:rw", design.path().display()),
            "--workdir=/workspace/project".into(),
            "--env=MIRKET_DESIGN_WORKSPACE=/workspace/project".into(),
            "--env=MIRKET_SCOPED_WORKER=1".into(),
            "--entrypoint=node".into(),
            candidate_image.clone(),
            "/opt/design/server.mjs".into(),
        ]);
        let spec = mirket::package_mcp::LaunchSpec {
            command: engine.binary.clone(),
            args,
            env: engine.env.clone(),
            cwd: design_project.clone(),
            allowed_tools: Some([name.to_owned()].into_iter().collect()),
            denied_tools: Default::default(),
            startup_timeout: std::time::Duration::from_secs(120),
            request_timeout: std::time::Duration::from_secs(120),
            tool_timeouts: Default::default(),
        };
        let reply =
            mirket::package_mcp::call(spec, name.into(), arguments.as_object().unwrap().clone())
                .await
                .unwrap();
        assert_ne!(reply.is_error, Some(true), "{reply:?}");
        reply.structured_content.unwrap()
    };
    let tokens = call("tokens_compile", json!({"source":"design/tokens.json"})).await;
    assert_eq!(
        tokens["runtime"]["files"]["server.mjs"],
        util::sha256(include_bytes!("../runtime/design/tools/server.mjs"))
    );
    assert_eq!(tokens["runtime"]["scope"], "project");
    let css =
        fs::read_to_string(design_project.join(tokens["outputs"]["tokens.css"].as_str().unwrap()))
            .unwrap();
    assert!(css.contains("--color-accent: #276a83;"));
    fs::write(design_project.join("source.svg"),r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="64" height="64" fill="#276a83"/></svg>"##).unwrap();
    let vector = call(
        "vector_render",
        json!({"source":"source.svg","output":"result.svg","width":64,"height":64}),
    )
    .await;
    assert_eq!(vector["runtime"], tokens["runtime"]);
    let png = fs::read(design_project.join(vector["preview"]["path"].as_str().unwrap())).unwrap();
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 64);
    assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 64);
    assert!(
        fs::read_to_string(design_project.join("result.svg"))
            .unwrap()
            .contains("#276a83")
    );
}
