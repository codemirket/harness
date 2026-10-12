//! Opt-in native editor contract and lifecycle tests in an explicitly disposable home.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]
use anyhow::{Context, Result, ensure};
use mirket::{packages, paths::Paths, util::sha256};
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, CallToolResult},
    service::RunningService,
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Duration,
};

type Client = RunningService<RoleClient, ()>;
const ART: &str = include_str!("fixtures/editors/courier.svg");
fn invoke(home: &Path, args: &[&str]) -> Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(home)
        .arg("--json")
        .args(args)
        .output()?)
}
fn success(home: &Path, args: &[&str]) -> Result<Value> {
    let output = invoke(home, args)?;
    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}
async fn start(home: &Path, project: &Path, id: &str) -> Result<Client> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command
        .arg("--home")
        .arg(home)
        .args(["package", "serve", id, "--project"])
        .arg(project)
        .kill_on_drop(true);
    Ok(tokio::time::timeout(
        Duration::from_secs(60),
        ().serve(TokioChildProcess::new(command)?),
    )
    .await??)
}
async fn call(client: &Client, name: &str, args: Value) -> Result<CallToolResult> {
    Ok(tokio::time::timeout(
        Duration::from_secs(60),
        client.call_tool(
            CallToolRequestParams::new(name.to_owned())
                .with_arguments(args.as_object().context("tool arguments")?.clone()),
        ),
    )
    .await??)
}
async fn good(client: &Client, name: &str, args: Value, trace: &mut Vec<Value>) -> Result<Value> {
    let result = call(client, name, args.clone()).await?;
    ensure!(result.is_error != Some(true), "{name}: {result:?}");
    let value = result
        .structured_content
        .context("editor must return its structured contract")?;
    trace.push(json!({"tool":name,"arguments":args,"result":value}));
    Ok(value)
}
async fn bad(client: &Client, name: &str, args: Value, trace: &mut Vec<Value>) -> Result<()> {
    let result = call(client, name, args.clone()).await?;
    ensure!(
        result.is_error == Some(true),
        "invalid input unexpectedly succeeded: {name}: {result:?}"
    );
    trace.push(json!({"tool":name,"arguments":args,"expected_error":result}));
    Ok(())
}

#[tokio::test]
#[ignore = "runs signed native applications in an explicitly marked disposable home"]
async fn packages_editors_live_artifacts_failures_and_update_replay() -> Result<()> {
    let home = PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME").context("set an absolute disposable home")?,
    );
    ensure!(home.is_absolute(), "absolute test home required");
    let home = home.canonicalize()?;
    ensure!(
        fs::read(home.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "disposable marker required"
    );
    let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let root = project.path().canonicalize()?;
    let project_arg = root.to_str().context("UTF8 project")?;
    let mut trace = vec![];
    let mut inkscape = start(&home, &root, "inkscape").await?;
    {
        let tools = inkscape.list_all_tools().await?;
        ensure!(
            tools.len() == 3 && tools.iter().all(|t| t.output_schema.is_some()),
            "discovery/schema contract changed"
        );
        trace.push(json!({"discovery":tools}));
    }
    let saved = good(
        &inkscape,
        "svg_save",
        json!({"path":"courier.svg","svg":ART}),
        &mut trace,
    )
    .await?;
    ensure!(
        fs::read_to_string(root.join("courier.svg"))? == ART,
        "editable markup changed"
    );
    good(
        &inkscape,
        "svg_inspect",
        json!({"path":"courier.svg"}),
        &mut trace,
    )
    .await?;
    bad(
        &inkscape,
        "svg_save",
        json!({"path":"courier.svg","svg":ART}),
        &mut trace,
    )
    .await?;
    bad(
        &inkscape,
        "svg_save",
        json!({"path":"courier.svg","svg":ART,"overwrite":true,"expected_sha256":"wrong"}),
        &mut trace,
    )
    .await?;
    // A real refinement preserves object identity and checks the revision being replaced.
    let refined = ART.replace("#73AB95", "#79B19B");
    good(&inkscape,"svg_save",json!({"path":"courier.svg","svg":refined,"overwrite":true,"expected_sha256":saved["artifact"]["sha256"]}),&mut trace).await?;
    if root.join("Courier.svg").exists() {
        let source_before = fs::read(root.join("courier.svg"))?;
        bad(
            &inkscape,
            "svg_export",
            json!({"source":"courier.svg","output":"Courier.svg","format":"svg","overwrite":true}),
            &mut trace,
        )
        .await?;
        ensure!(
            fs::read(root.join("courier.svg"))? == source_before,
            "case alias replaced source"
        );
    }
    for (format, output, width, id) in [
        ("png", "courier.png", Some(1280), None),
        ("png", "courier-small.png", Some(320), None),
        ("png", "aircraft.png", Some(256), Some("aircraft")),
        ("svg", "courier-export.svg", None, None),
        ("pdf", "courier.pdf", None, None),
    ] {
        let mut args = json!({"source":"courier.svg","output":output,"format":format});
        if let Some(w) = width {
            args["width"] = json!(w);
        }
        if let Some(id) = id {
            args["object_id"] = json!(id);
        }
        good(&inkscape, "svg_export", args, &mut trace).await?;
        ensure!(
            fs::metadata(root.join(output))?.len() > 128,
            "usable artifact missing"
        );
    }
    for path in [
        "../escape.svg",
        "/tmp/escape.svg",
        ".hidden.svg",
        "missing/out.svg",
    ] {
        bad(
            &inkscape,
            "svg_save",
            json!({"path":path,"svg":ART}),
            &mut trace,
        )
        .await?;
    }
    std::os::unix::fs::symlink("courier.svg", root.join("alias.svg"))?;
    bad(
        &inkscape,
        "svg_inspect",
        json!({"path":"alias.svg"}),
        &mut trace,
    )
    .await?;
    fs::remove_file(root.join("alias.svg"))?;
    for markup in [
        "<svg>",
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><script>alert(1)</script></svg>"#,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><image href="file:///etc/passwd"/></svg>"#,
    ] {
        bad(
            &inkscape,
            "svg_save",
            json!({"path":"invalid.svg","svg":markup}),
            &mut trace,
        )
        .await?;
        ensure!(
            !root.join("invalid.svg").exists(),
            "failed validation published an artifact"
        );
    }
    bad(
        &inkscape,
        "svg_export",
        json!({"source":"courier.svg","output":"wrong.png","format":"png","object_id":"absent"}),
        &mut trace,
    )
    .await?;
    inkscape.close().await?;
    ensure!(
        !fs::read_dir(&root)?.any(|e| e.is_ok_and(|e| e
            .file_name()
            .to_string_lossy()
            .starts_with(".mirket-editor-"))),
        "private workspaces leaked"
    );
    {
        let id = "inkscape";
        trace.push(success(&home, &["package", "show", id])?);
        trace.push(success(
            &home,
            &[
                "package",
                "probe",
                id,
                "--schemas",
                "--project",
                project_arg,
            ],
        )?);
    }
    trace.push(success(
        &home,
        &[
            "package",
            "call",
            "inkscape",
            "svg_inspect",
            "--arguments",
            "{\"path\":\"courier.svg\"}",
            "--project",
            project_arg,
        ],
    )?);
    let settings = home.join(".mirket/setup.json");
    let options = serde_json::from_slice::<Value>(&fs::read(&settings)?)?["options"].clone();
    ensure!(
        options["targets"] == json!(["codex", "claude"]),
        "both disposable host configurations must be selected"
    );
    let binary = env!("CARGO_BIN_EXE_mirket");
    trace.push(success(
        &home,
        &[
            "update",
            "--from",
            binary,
            "--sha256",
            &sha256(&fs::read(binary)?),
        ],
    )?);
    ensure!(
        serde_json::from_slice::<Value>(&fs::read(&settings)?)?["options"] == options,
        "update changed saved choices"
    );
    let paths = Paths::new(home.clone())?;
    let launch = packages::launch(&paths, "inkscape", &root)?;
    let drift = launch
        .command
        .parent()
        .context("executable parent")?
        .join("unexpected-fixture");
    let watched = [
        settings,
        home.join(".codex/config.toml"),
        home.join(".claude.json"),
    ];
    let snapshots = watched
        .iter()
        .map(fs::read)
        .collect::<std::io::Result<Vec<_>>>()?;
    fs::write(&drift, b"managed-copy drift\n")?;
    let refused_probe = invoke(
        &home,
        &["package", "probe", "inkscape", "--project", project_arg],
    )?;
    let refused_setup = invoke(&home, &["setup", "--yes"])?;
    fs::remove_file(drift)?;
    for result in [refused_probe, refused_setup] {
        ensure!(
            !result.status.success() && String::from_utf8_lossy(&result.stderr).contains("drifted"),
            "managed drift was accepted"
        );
    }
    for (path, before) in watched.iter().zip(snapshots) {
        ensure!(
            fs::read(path)? == before,
            "failed setup changed active settings"
        );
    }
    trace.push(success(&home, &["doctor"])?);
    if let Some(output) = std::env::var_os("MIRKET_PACKAGE_TEST_OUTPUT") {
        let output = PathBuf::from(output);
        ensure!(
            output.is_absolute() && output.is_dir(),
            "existing absolute evidence directory required"
        );
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                fs::copy(entry.path(), output.join(entry.file_name()))?;
            }
        }
        fs::write(
            output.join("editor-tool-trace.json"),
            serde_json::to_vec_pretty(&trace)?,
        )?;
    }
    Ok(())
}
