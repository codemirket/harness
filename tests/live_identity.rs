use mirket::{catalog::Catalog, mcp::MirketServer, paths::Paths};
use rmcp::{ServiceExt, model::CallToolRequestParams};
use serde_json::json;
use std::fs;

#[tokio::test]
async fn connected_identity_does_not_change_when_the_installed_binary_is_replaced() {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().to_path_buf()).unwrap();
    fs::create_dir_all(paths.binary().parent().unwrap()).unwrap();
    fs::copy(std::env::current_exe().unwrap(), paths.binary()).unwrap();
    let catalog = Catalog::embedded().unwrap();
    let expected_catalog = catalog.fingerprint();
    let server = MirketServer::new(&paths, catalog).unwrap();
    let (a, b) = tokio::io::duplex(65536);
    let handle =
        tokio::spawn(async move { server.serve(a).await.unwrap().waiting().await.unwrap() });
    let client = ().serve(b).await.unwrap();
    let before = client
        .call_tool(CallToolRequestParams::new("runtime_status"))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(before["catalog"]["sha256"], expected_catalog);
    assert_eq!(before["installed_binary"]["matches_process"], true);
    assert_eq!(before["recommended_actions"], json!([]));
    assert!(
        client
            .peer_info()
            .unwrap()
            .server_info
            .as_ref()
            .unwrap()
            .version
            .contains('+')
    );
    fs::write(paths.binary(), b"replacement executable fixture").unwrap();
    let after = client
        .call_tool(CallToolRequestParams::new("runtime_status"))
        .await
        .unwrap()
        .structured_content
        .unwrap();
    assert_eq!(after["process"], before["process"]);
    assert_eq!(after["installed_binary"]["matches_process"], false);
    assert!(
        after["recommended_actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a.as_str().unwrap().contains("Reconnect this client"))
    );
    let args = json!({"project":paths.home,"run_id":"routed-fixture"});
    let rejected = client
        .call_tool(
            CallToolRequestParams::new("verification_handoff")
                .with_arguments(args.as_object().unwrap().clone()),
        )
        .await
        .unwrap();
    assert_eq!(rejected.is_error, Some(true));
    client.cancel().await.unwrap();
    handle.await.unwrap();
}

#[test]
fn alternative_workflows_are_explicit_and_profile_selections_are_preserved() {
    let catalog = Catalog::embedded().unwrap();
    let ordinary = catalog.search("writing plans", 0, 100).unwrap();
    assert!(
        !ordinary["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["id"] == "superpowers-writing-plans")
    );
    let explicit = catalog.search("superpowers-writing-plans", 0, 10).unwrap();
    assert_eq!(explicit["items"][0]["id"], "superpowers-writing-plans");
    assert!(
        !catalog.search("superpowers", 0, 100).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    for target in ["codex", "claude"] {
        let selected = catalog
            .selection(&["execution-superpowers".into()], &[], target)
            .unwrap();
        assert!(selected.iter().any(|s| s.id == "superpowers-writing-plans"));
    }
    assert_eq!(
        catalog
            .profiles()
            .values()
            .filter(|p| p.kind == "starter")
            .count(),
        8
    );
    assert_eq!(catalog.profiles().len(), 63);
}
