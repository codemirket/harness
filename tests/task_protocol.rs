use mirket::{catalog::Catalog, mcp::MirketServer, paths::Paths, state::State, util};
use rmcp::{ServiceExt, model::CallToolRequestParams};
use serde_json::{Value, json};
use std::{fs, process::Command};

#[tokio::test]
async fn cli_and_mcp_recover_history_and_reject_stale_contracts() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
    let project = home.path().join("project");
    fs::create_dir(&project).unwrap();
    let state = State::open(&paths).unwrap();
    let registered = state.register_project(&project).unwrap();
    let mut task = state
        .begin(
            registered["id"].as_str().unwrap(),
            "Protocol contract",
            &["engineering-judgment".into()],
            "current inputs",
            "stale source",
            "start",
        )
        .unwrap();
    let id = task["id"].as_str().unwrap().to_string();
    for index in 0..12 {
        task = state
            .checkpoint(
                &id,
                task["revision"].as_i64().unwrap(),
                &format!("constraint {index}"),
                &format!("checkpoint-{index}"),
            )
            .unwrap();
    }
    let invoke = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_mirket"))
            .env("MIRKET_DIRECT", "1")
            .arg("--home")
            .arg(home.path())
            .args(args)
            .output()
            .unwrap()
    };
    let output = invoke(&["task", "history", &id, "--limit", "5"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let page: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(page["checkpoints"].as_array().unwrap().len(), 5);
    assert!(page["next_cursor"].is_string());
    for path in ["input.txt", "checker.txt", "result.txt"] {
        fs::write(project.join(path), path).unwrap();
    }
    let binding = |path: &str| json!({"path":path,"sha256":util::sha256(&fs::read(project.join(path)).unwrap())});
    let receipt = json!({"schema_version":1,"task_id":id,"inputs":[binding("input.txt")],"checker":binding("checker.txt"),"result":binding("result.txt"),"outcome":"passed","limitations":["Fixture checks identity, not semantic truth"]});
    fs::write(
        project.join("receipt.json"),
        serde_json::to_vec(&receipt).unwrap(),
    )
    .unwrap();
    let output = invoke(&[
        "task",
        "validate",
        &id,
        "receipt.json",
        "--contract",
        "verification",
        "--completion",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let server = MirketServer::new(&paths, Catalog::embedded().unwrap()).unwrap();
    let (a, b) = tokio::io::duplex(8192);
    let host = tokio::spawn(server.serve(a));
    let mut client = ().serve(b).await.unwrap();
    let mut host = host.await.unwrap().unwrap();
    let call = |name: &'static str, args: Value| {
        CallToolRequestParams::new(name).with_arguments(args.as_object().unwrap().clone())
    };
    let tools = client.list_all_tools().await.unwrap();
    for name in ["task_history", "task_validate"] {
        assert!(tools.iter().any(|tool| tool.name == name));
    }
    let response = client
        .call_tool(call(
            "task_history",
            json!({"task_id":id,"cursor":page["next_cursor"],"limit":20}),
        ))
        .await
        .unwrap();
    assert_eq!(response.is_error, Some(false));
    let history = response.structured_content.unwrap();
    assert_eq!(
        history["checkpoints"].as_array().unwrap().last().unwrap()["note"],
        "constraint 0"
    );
    let response = client
        .call_tool(call(
            "task_validate",
            json!({"task_id":id,"path":"receipt.json","contract":"verification","completion":true}),
        ))
        .await
        .unwrap();
    assert_eq!(response.is_error, Some(false));
    let response = client.call_tool(call("task_evidence", json!({"task_id":id,"revision":task["revision"],"path":"receipt.json","kind":"acceptance","summary":"Input bound protocol fixture","contract":"verification","key":"record"}))).await.unwrap();
    assert_eq!(response.is_error, Some(false));
    assert_eq!(
        response.structured_content.unwrap()["result"]["contract"],
        "verification"
    );
    fs::write(project.join("input.txt"), "changed").unwrap();
    let response = client
        .call_tool(call(
            "task_validate",
            json!({"task_id":id,"path":"receipt.json","contract":"verification","completion":true}),
        ))
        .await
        .unwrap();
    assert_eq!(response.is_error, Some(true));
    assert!(
        !invoke(&[
            "task",
            "validate",
            &id,
            "receipt.json",
            "--contract",
            "verification",
            "--completion"
        ])
        .status
        .success()
    );
    client.close().await.unwrap();
    host.close().await.unwrap();
}
