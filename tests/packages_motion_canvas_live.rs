//! Opt-in real Motion Canvas rendering and lifecycle contract in a marked disposable home.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]
use anyhow::{Context, Result, ensure};
use mirket::{paths::Paths, podman::Engine, util::sha256};
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequest, CallToolRequestParams, CallToolResult, ClientRequest},
    service::{PeerRequestOptions, RunningService},
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

type Client = RunningService<RoleClient, ()>;
const SCENE: &str = include_str!("../runtime/packages/motion-canvas/examples/aurora.json");
async fn call(client: &Client, name: &str, args: Value) -> Result<CallToolResult> {
    Ok(tokio::time::timeout(
        Duration::from_secs(100),
        client.call_tool(
            CallToolRequestParams::new(name.to_owned())
                .with_arguments(args.as_object().context("object arguments")?.clone()),
        ),
    )
    .await??)
}
async fn good(client: &Client, name: &str, args: Value, trace: &mut Vec<Value>) -> Result<Value> {
    let result = call(client, name, args.clone()).await?;
    ensure!(result.is_error != Some(true), "{name}: {result:?}");
    let value = result
        .structured_content
        .context("structured editor result required")?;
    trace.push(json!({"tool":name,"arguments":args,"result":value}));
    Ok(value)
}
async fn bad(client: &Client, name: &str, args: Value, trace: &mut Vec<Value>) -> Result<()> {
    let result = call(client, name, args.clone()).await?;
    ensure!(
        result.is_error == Some(true),
        "invalid input succeeded: {name}: {result:?}"
    );
    trace.push(json!({"tool":name,"arguments":args,"expected_error":result}));
    Ok(())
}
async fn engine_read(engine: &Engine, args: Vec<String>) -> Result<String> {
    let engine = engine.clone();
    tokio::task::spawn_blocking(move || engine.run(&args, Duration::from_secs(3))).await?
}
struct RenderWorker {
    cid: String,
    stage: PathBuf,
    guest_processes: String,
}
async fn active_render_worker(engine: &Engine) -> Result<RenderWorker> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            for entry in fs::read_dir(&engine.staging)? {
                let stage = entry?.path();
                if !stage.is_dir() || !stage.join("container-id").is_file() {
                    continue;
                }
                let input = stage.join("input");
                let mut this_render = false;
                for entry in fs::read_dir(&input)? {
                    let job = entry?.path();
                    if let (Ok(manifest), Ok(bytes)) = (
                        fs::read(job.join("launch.json")),
                        fs::read(job.join("job.json")),
                    ) {
                        let manifest: Value = serde_json::from_slice(&manifest)?;
                        let job: Value = serde_json::from_slice(&bytes)?;
                        this_render |= manifest["package"] == "motion-canvas"
                            && job["operation"] == "render"
                            && job["scene"]["frames"] == 900;
                    }
                }
                if !this_render {
                    continue;
                }
                let cid = fs::read_to_string(stage.join("container-id"))?
                    .trim()
                    .to_owned();
                ensure!(
                    cid.len() == 64 && cid.bytes().all(|b| b.is_ascii_hexdigit()),
                    "invalid worker CID"
                );
                let inspected: Value = serde_json::from_str(
                    &engine_read(
                        engine,
                        vec!["container".into(), "inspect".into(), cid.clone()],
                    )
                    .await?,
                )?;
                let worker = &inspected[0];
                ensure!(
                    worker["Id"] == cid
                        && worker["Config"]["Labels"]["org.mirket.execution-owner"]
                            == engine.owner()
                        && worker["Config"]["Labels"]["org.mirket.execution-kind"]
                            == "offline-render"
                        && worker["Config"]["User"] == "1000:1000"
                        && worker["HostConfig"]["ReadonlyRootfs"] == true
                        && worker["HostConfig"]["NetworkMode"] == "none"
                        && worker["Mounts"]
                            .as_array()
                            .is_some_and(|mounts| mounts
                                .iter()
                                .any(|m| m["Destination"] == "/input"
                                    && m["Source"] == input.to_string_lossy().as_ref()
                                    && m["RW"] == false)),
                    "render worker ownership or isolation mismatch"
                );
                let guest_processes =
                    engine_read(engine, vec!["top".into(), cid.clone(), "pid,args".into()]).await?;
                if guest_processes.contains("chrome-headless-shell")
                    && guest_processes.contains("/opt/ffmpeg/bin/ffmpeg")
                {
                    let browser = guest_processes
                        .lines()
                        .find(|line| {
                            line.contains("chrome-headless-shell") && !line.contains("--type=")
                        })
                        .context("guest browser command was not observed")?;
                    ensure!(
                        browser.contains("--no-sandbox")
                            && !browser.contains("--disable-web-security")
                            && !browser.contains("--disable-site-isolation-trials")
                            && !browser.split_whitespace().any(|arg| {
                                arg.strip_prefix("--disable-features=").is_some_and(|features| {
                                    features
                                        .split(',')
                                        .any(|feature| ["IsolateOrigins", "site-per-process"].contains(&feature))
                                })
                            }),
                        "guest browser arguments disagree with the selected container boundary: {browser}"
                    );
                    return Ok::<_, anyhow::Error>(RenderWorker {
                        cid,
                        stage,
                        guest_processes,
                    });
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .context("render never established a live guest Chromium/encoder in an owned worker")?
}
async fn worker_removed(engine: &Engine, worker: &RenderWorker, jobs: &Path) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let found = engine_read(
                engine,
                vec![
                    "container".into(),
                    "ls".into(),
                    "--all".into(),
                    "--filter".into(),
                    format!("id={}", worker.cid),
                    "--no-trunc".into(),
                    "--format={{.ID}}".into(),
                ],
            )
            .await?;
            if found.trim().is_empty()
                && !worker.stage.exists()
                && fs::read_dir(jobs)?.next().is_none()
            {
                return Ok::<_, anyhow::Error>(());
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .context("cancelled worker or staging survived")?
}
fn saved_png(root: &Path, name: &str, result: &Value) -> Result<()> {
    let bytes = fs::read(root.join(name))?;
    ensure!(
        bytes.len() > 24
            && bytes.starts_with(b"\x89PNG\r\n\x1a\n")
            && u32::from_be_bytes(bytes[16..20].try_into()?) == 960
            && u32::from_be_bytes(bytes[20..24].try_into()?) == 540
            && result["artifact"]["sha256"] == sha256(&bytes),
        "saved frame identity or dimensions are invalid"
    );
    Ok(())
}
fn worker_stages(engine: &Engine) -> Result<Vec<PathBuf>> {
    fs::read_dir(&engine.staging)?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().join("container-id").is_file() => Some(Ok(entry.path())),
            Ok(_) => None,
            Err(error) => Some(Err(error.into())),
        })
        .collect()
}
async fn staging_stays_empty(jobs: &Path) -> Result<()> {
    tokio::time::timeout(Duration::from_secs(2), async {
        for _ in 0..50 {
            ensure!(fs::read_dir(jobs)?.next().is_none(), "staging reappeared");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        ensure!(fs::read_dir(jobs)?.next().is_none(), "staging reappeared");
        Ok::<_, anyhow::Error>(())
    })
    .await
    .context("stable staging cleanup exceeded its bound")?
}

#[tokio::test]
#[ignore = "requires pinned installed Motion Canvas in an explicitly marked disposable home"]
async fn motion_canvas_real_rich_scene_failures_and_cancellation() -> Result<()> {
    let home = PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME")
            .context("explicit disposable home required")?,
    );
    ensure!(home.is_absolute(), "absolute test home required");
    let home = home.canonicalize()?;
    ensure!(
        fs::read(home.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "disposable home marker required"
    );
    let parent = if let Some(value) = std::env::var_os("MIRKET_MOTION_CANVAS_EVIDENCE_DIR") {
        let path = PathBuf::from(value);
        ensure!(
            path.is_absolute() && path.is_dir(),
            "existing absolute evidence directory required"
        );
        path.canonicalize()?
    } else {
        std::env::temp_dir().canonicalize()?
    };
    let temporary = tempfile::Builder::new()
        .prefix("motion-canvas-contract-")
        .tempdir_in(parent)?;
    let root = temporary.path().join("project");
    // Native controller jobs and guest staging are observed independently.
    let private_jobs = tempfile::Builder::new()
        .prefix("motion-canvas-jobs-")
        .tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let jobs = private_jobs.path();
    fs::create_dir(&root)?;
    fs::write(root.join("scene.json"), SCENE)?;
    let identity = sha256(SCENE.as_bytes());
    let engine = Engine::resolve(&Paths::new(home.clone())?)?;
    engine.ready()?;
    let existing_workers = worker_stages(&engine)?;
    // Isolated staging allows this test to observe its own jobs and descendants.
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command
        .arg("--home")
        .arg(&home)
        .args(["package", "serve", "motion-canvas", "--project"])
        .arg(&root)
        .env("TMPDIR", jobs)
        .kill_on_drop(true);
    let mut client = tokio::time::timeout(
        Duration::from_secs(60),
        ().serve(TokioChildProcess::new(command)?),
    )
    .await??;
    let tools = client.list_all_tools().await?;
    let expected = [
        "motion_canvas_inspect",
        "motion_canvas_frame",
        "motion_canvas_verify",
        "motion_canvas_render",
    ];
    ensure!(
        tools.len() == expected.len()
            && expected.iter().all(|name| tools
                .iter()
                .any(|tool| tool.name == *name && tool.output_schema.is_some())),
        "discovery or output schema changed"
    );
    let mut trace = vec![json!({"discovery":tools})];
    let inspected = good(
        &client,
        "motion_canvas_inspect",
        json!({"source":"scene.json"}),
        &mut trace,
    )
    .await?;
    ensure!(
        inspected["source_sha256"] == identity
            && inspected["animation"]["source_commit"]
                == "e735a995aa738ca682feb4204a39ee7db7b5d279",
        "editable source identity lost"
    );
    let inspection = &inspected["animation"]["inspection"];
    ensure!(
        inspection["playback_api"] == "Scene2D+PlaybackManager+Stage"
            && inspection["node_count"] == 12
            && inspection["track_count"] == 10
            && inspection["gradients"]
                .as_array()
                .is_some_and(|v| v.len() == 2)
            && inspection["font"]["status"] == "loaded",
        "rich upstream scene was not established"
    );
    let verified = good(
        &client,
        "motion_canvas_verify",
        json!({"source":"scene.json"}),
        &mut trace,
    )
    .await?;
    ensure!(
        verified["animation"]["verification"]["deterministic"] == true
            && verified["animation"]["verification"]["unique_count"] == 5,
        "changing repeatable key frames missing"
    );
    let mut frames = vec![];
    for frame in [0, 75, 149] {
        frames.push(
            good(
                &client,
                "motion_canvas_frame",
                json!({"source":"scene.json","frame":frame,"output":format!("frame-{frame}.png")}),
                &mut trace,
            )
            .await?,
        );
        saved_png(&root, &format!("frame-{frame}.png"), frames.last().unwrap())?;
    }
    ensure!(
        frames[0]["artifact"]["sha256"] != frames[1]["artifact"]["sha256"]
            && frames[1]["artifact"]["sha256"] != frames[2]["artifact"]["sha256"],
        "path, gradient and choreography did not change rendered pixels"
    );
    let rendered = good(
        &client,
        "motion_canvas_render",
        json!({"source":"scene.json","output":"aurora.mp4"}),
        &mut trace,
    )
    .await?;
    ensure!(
        rendered["animation"]["decoded_frames"] == 150
            && rendered["animation"]["codec"] == "h264"
            && rendered["animation"]["decoded_unique_count"]
                .as_u64()
                .is_some_and(|n| n > 100),
        "saved video failed complete decoding or was frozen"
    );
    let original = fs::read(root.join("frame-0.png"))?;
    for args in [
        json!({"source":"scene.json","frame":150,"output":"absent.png"}),
        json!({"source":"scene.json","frame":0,"output":"frame-0.png"}),
        json!({"source":"scene.json","frame":0,"output":"frame-0.png","overwrite":true,"expected_sha256":"wrong"}),
        json!({"source":"../scene.json","frame":0,"output":"absent.png"}),
        json!({"source":"scene.json","frame":0,"output":"../escape.png"}),
        json!({"source":"scene.json","frame":0,"output":"missing/frame.png"}),
        json!({"source":"scene.json","frame":0,"output":"scene.json","overwrite":true}),
    ] {
        bad(&client, "motion_canvas_frame", args, &mut trace).await?;
    }
    ensure!(
        fs::read(root.join("frame-0.png"))? == original && !root.join("absent.png").exists(),
        "failure changed outputs"
    );
    good(&client, "motion_canvas_frame", json!({"source":"scene.json","frame":1,"output":"frame-0.png","overwrite":true,"expected_sha256":frames[0]["artifact"]["sha256"]}), &mut trace).await?;
    fs::hard_link(root.join("scene.json"), root.join("source-alias.mp4"))?;
    std::os::unix::fs::symlink(root.join("scene.json"), root.join("source-link.mp4"))?;
    for name in ["source-alias.mp4", "source-link.mp4"] {
        bad(
            &client,
            "motion_canvas_render",
            json!({"source":"scene.json","output":name,"overwrite":true}),
            &mut trace,
        )
        .await?;
    }
    // The malformed path reaches upstream geometry parsing; it cannot succeed as a generic blank canvas.
    for kind in [
        "unknown-property",
        "overlapping-track",
        "malformed-path",
        "nonfinite-path",
    ] {
        let mut invalid: Value = serde_json::from_str(SCENE)?;
        match kind {
            "unknown-property" => {
                invalid["nodes"][0]["props"]["src"] = json!("https://example.invalid/image.png")
            }
            "overlapping-track" => {
                let track = invalid["tracks"][0].clone();
                invalid["tracks"].as_array_mut().unwrap().push(track);
            }
            "malformed-path" => invalid["nodes"][4]["props"]["data"] = json!("M0 0L"),
            _ => invalid["nodes"][4]["props"]["data"] = json!("M1e999 0L0 20L20 0Z"),
        }
        fs::write(root.join("invalid.json"), serde_json::to_vec(&invalid)?)?;
        bad(
            &client,
            "motion_canvas_render",
            json!({"source":"invalid.json","output":"failed.mp4"}),
            &mut trace,
        )
        .await?;
        ensure!(
            !root.join("failed.mp4").exists(),
            "failed scene published a video"
        );
    }
    ensure!(
        sha256(&fs::read(root.join("scene.json"))?) == identity,
        "editable source was modified"
    );
    let mut long: Value = serde_json::from_str(SCENE)?;
    long["frames"] = json!(900);
    long["width"] = json!(1920);
    long["height"] = json!(1080);
    fs::write(root.join("long.json"), serde_json::to_vec(&long)?)?;
    let request = ClientRequest::CallToolRequest(CallToolRequest::new(
        CallToolRequestParams::new("motion_canvas_render".to_owned()).with_arguments(
            json!({"source":"long.json","output":"cancelled.mp4"})
                .as_object()
                .unwrap()
                .clone(),
        ),
    ));
    let handle = client
        .send_cancellable_request(request, PeerRequestOptions::no_options())
        .await?;
    let worker = active_render_worker(&engine).await?;
    handle
        .cancel(Some("explicit contract cancellation".into()))
        .await?;
    worker_removed(&engine, &worker, jobs).await?;
    ensure!(
        !root.join("cancelled.mp4").exists(),
        "cancelled video was published"
    );
    trace.push(json!({"cancellation":{"observed_worker_cid":worker.cid,"guest_processes":worker.guest_processes,"worker_removed":true,"staging_removed":true,"output_published":false}}));
    let recovered = good(
        &client,
        "motion_canvas_frame",
        json!({"source":"scene.json","frame":75,"output":"recovered.png"}),
        &mut trace,
    )
    .await?;
    saved_png(&root, "recovered.png", &recovered)?;
    ensure!(
        recovered["artifact"]["sha256"] == frames[1]["artifact"]["sha256"],
        "recovery rendered different scene pixels"
    );
    let stages: Vec<_> = worker_stages(&engine)?
        .into_iter()
        .filter(|stage| !existing_workers.contains(stage))
        .collect();
    ensure!(
        stages.len() == 1,
        "expected exactly one recovered MCP worker"
    );
    let final_worker = RenderWorker {
        cid: fs::read_to_string(stages[0].join("container-id"))?
            .trim()
            .into(),
        stage: stages[0].clone(),
        guest_processes: String::new(),
    };
    client.close().await?;
    worker_removed(&engine, &final_worker, jobs).await?;
    staging_stays_empty(jobs).await?;
    trace.push(
        json!({"final_cleanup":{"last_worker_cid":final_worker.cid,"worker_removed":true,"private_jobs_root":jobs,"host_temp":true,"stable_empty_ms":1000}}),
    );
    fs::write(root.join("trace.json"), serde_json::to_vec_pretty(&trace)?)?;
    if std::env::var_os("MIRKET_MOTION_CANVAS_EVIDENCE_DIR").is_some() {
        let saved = temporary.keep();
        eprintln!("Motion Canvas contract evidence: {}", saved.display());
    }
    Ok(())
}
