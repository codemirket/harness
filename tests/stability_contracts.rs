//! Clean 0.0.1 public boundaries and embedding identity.
use mirket::{catalog::Catalog, mcp::MirketServer, paths::Paths, util};
use rmcp::ServiceExt;
use serde_json::Value;

#[tokio::test]
async fn stability_preserves_public_tool_names_and_fact_digest() {
    let baseline: Value =
        serde_json::from_str(include_str!("fixtures/stability/compatibility.json")).unwrap();
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().to_path_buf()).unwrap();
    let server = MirketServer::new(&paths, Catalog::embedded().unwrap()).unwrap();
    let (a, b) = tokio::io::duplex(65536);
    let serving =
        tokio::spawn(async move { server.serve(a).await.unwrap().waiting().await.unwrap() });
    let client = ().serve(b).await.unwrap();
    let tools = client.list_all_tools().await.unwrap();
    for name in baseline["main_mcp_tools"].as_array().unwrap() {
        assert!(
            tools.iter().any(|tool| tool.name == name.as_str().unwrap()),
            "removed public tool: {name}"
        );
    }
    assert_eq!(
        util::sha256(&serde_json::to_vec(&baseline["fact"]).unwrap()),
        baseline["fact_sha256"]
    );
    client.cancel().await.unwrap();
    serving.await.unwrap();
}

#[test]
fn stability_freezes_model_and_release_identity() {
    let baseline: Value =
        serde_json::from_str(include_str!("fixtures/stability/compatibility.json")).unwrap();
    let model: Value = serde_json::from_str(include_str!("../runtime/agent/model.json")).unwrap();
    assert_eq!(
        model["revision"],
        "faf4aa4225822f3bc6376869cb1164e8e3feedd0"
    );
    assert_eq!(model["max_tokens"], 128);
    assert_eq!(model["dimensions"], 384);
    assert_eq!(env!("CARGO_PKG_VERSION"), baseline["release"]);
}
