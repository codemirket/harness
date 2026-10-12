//! Opt-in tests of the pinned upstream producer in a marked disposable home.
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
    time::{Duration, Instant},
};
type Client = RunningService<RoleClient, ()>;
const SCENE: &str = r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Local sound and alpha</title><style>html,body{margin:0;width:320px;height:180px;overflow:hidden;background:transparent}#mark{position:absolute;left:24px;top:54px;width:64px;height:64px;background:#d5f46a;border-radius:12px}</style><script src="/__mirket/gsap.min.js"></script></head><body><div data-composition-id="main" data-start="0" data-width="320" data-height="180" data-duration="1"><div id="mark"></div><audio id="sound" src="tone.wav" data-start="0" data-end="1" data-volume="0.4"></audio></div><script>window.__timelines={main:gsap.timeline({paused:true}).to('#mark',{x:160,rotation:30,duration:1,ease:'none'})};</script></body></html>"##;
fn tone() -> Vec<u8> {
    let size = 48000u32 * 2;
    let mut out = Vec::new();
    out.extend(b"RIFF");
    out.extend((size + 36).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(48000u32.to_le_bytes());
    out.extend(96000u32.to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend(size.to_le_bytes());
    for i in 0..48000 {
        let sample = ((f64::from(i) * 2.0 * std::f64::consts::PI * 440.0 / 48000.0).sin() * 8000.0)
            .round() as i16;
        out.extend(sample.to_le_bytes());
    }
    out
}
async fn call(c: &Client, name: &str, args: Value) -> Result<CallToolResult> {
    Ok(tokio::time::timeout(
        Duration::from_secs(100),
        c.call_tool(
            CallToolRequestParams::new(name.to_owned())
                .with_arguments(args.as_object().context("object arguments")?.clone()),
        ),
    )
    .await??)
}
async fn good(c: &Client, name: &str, args: Value, trace: &mut Vec<Value>) -> Result<Value> {
    let r = call(c, name, args.clone()).await?;
    ensure!(r.is_error != Some(true), "{name}: {r:?}");
    let v = r.structured_content.context("structured result")?;
    ensure!(
        v["animation"]["isolation"] == "podman" && v["animation"]["browser_sandbox"] == false,
        "container result misreported its browser isolation boundary: {v}"
    );
    trace.push(json!({"tool":name,"arguments":args,"result":v}));
    Ok(v)
}
async fn bad(c: &Client, name: &str, args: Value, trace: &mut Vec<Value>) -> Result<()> {
    let r = call(c, name, args.clone()).await?;
    ensure!(
        r.is_error == Some(true),
        "invalid input succeeded: {name}: {r:?}"
    );
    trace.push(json!({"tool":name,"arguments":args,"expected_error":r}));
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
    tokio::time::timeout(Duration::from_secs(8), async {
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
                        this_render |=
                            manifest["package"] == "hyperframes" && job["operation"] == "render";
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
                    && guest_processes.contains("/opt/hyperframes/runner.mjs")
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
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .context("hanging composition never established a real guest browser")?
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
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .context("failed worker or staging survived")?
}
fn saved_png(root: &Path, name: &str, result: &Value) -> Result<()> {
    let bytes = fs::read(root.join(name))?;
    ensure!(
        bytes.len() > 24
            && bytes.starts_with(b"\x89PNG\r\n\x1a\n")
            && u32::from_be_bytes(bytes[16..20].try_into()?) == 320
            && u32::from_be_bytes(bytes[20..24].try_into()?) == 180
            && result["artifact"]["sha256"] == sha256(&bytes),
        "saved PNG identity or dimensions are invalid"
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
#[tokio::test]
#[ignore = "requires installed pinned producer and explicitly marked disposable home"]
async fn hyperframes_real_producer_audio_alpha_and_failures() -> Result<()> {
    let home = PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME")
            .context("explicit disposable home required")?,
    );
    ensure!(home.is_absolute(), "absolute home required");
    let home = home.canonicalize()?;
    ensure!(
        fs::read(home.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "disposable home marker required"
    );
    let parent = if let Some(p) = std::env::var_os("MIRKET_HYPERFRAMES_EVIDENCE_DIR") {
        let p = PathBuf::from(p);
        ensure!(
            p.is_absolute() && p.is_dir(),
            "existing absolute evidence directory required"
        );
        p.canonicalize()?
    } else {
        std::env::temp_dir().canonicalize()?
    };
    let temporary = tempfile::Builder::new()
        .prefix("hyperframes-contract-")
        .tempdir_in(&parent)?;
    let root = temporary.path().canonicalize()?;
    fs::write(root.join("scene.html"), SCENE)?;
    fs::write(root.join("tone.wav"), tone())?;
    fs::write(
        root.join("composition.json"),
        serde_json::to_vec(
            &json!({"version":1,"entry":"scene.html","assets":["tone.wav"],"width":320,"height":180,"fps":30,"frames":30}),
        )?,
    )?;
    let engine = Engine::resolve(&Paths::new(home.clone())?)?;
    engine.ready()?;
    let existing_workers = worker_stages(&engine)?;
    let job_scratch = tempfile::Builder::new()
        .prefix("mirket-hyperframes-jobs-")
        .tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command.env("TMPDIR", job_scratch.path());
    command
        .arg("--home")
        .arg(&home)
        .args(["package", "serve", "hyperframes", "--project"])
        .arg(&root)
        .kill_on_drop(true);
    let mut client = tokio::time::timeout(
        Duration::from_secs(60),
        ().serve(TokioChildProcess::new(command)?),
    )
    .await??;
    let tools = client.list_all_tools().await?;
    ensure!(
        tools.len() == 4 && tools.iter().all(|t| t.output_schema.is_some()),
        "tool/schema contract changed"
    );
    let mut trace = vec![json!({"discovery":tools})];
    let inspected = good(
        &client,
        "composition_inspect",
        json!({"source":"composition.json"}),
        &mut trace,
    )
    .await?;
    ensure!(
        inspected["animation"]["engine"] == "hyperframes",
        "wrong actual runtime"
    );
    good(
        &client,
        "composition_verify",
        json!({"source":"composition.json"}),
        &mut trace,
    )
    .await?;
    let first = good(
        &client,
        "composition_frame",
        json!({"source":"composition.json","frame":0,"output":"first.png"}),
        &mut trace,
    )
    .await?;
    let last = good(
        &client,
        "composition_frame",
        json!({"source":"composition.json","frame":29,"output":"last.png"}),
        &mut trace,
    )
    .await?;
    saved_png(&root, "first.png", &first)?;
    saved_png(&root, "last.png", &last)?;
    ensure!(
        first["artifact"]["sha256"] != last["artifact"]["sha256"],
        "animation did not move"
    );
    for ext in ["mp4", "webm"] {
        let r = good(
            &client,
            "composition_render",
            json!({"source":"composition.json","output":format!("sound.{ext}")}),
            &mut trace,
        )
        .await?;
        ensure!(
            r["animation"]["decoded_frames"] == 30 && r["animation"]["audio_streams"] == 1,
            "video or audio lost"
        );
        if ext == "webm" {
            ensure!(
                r["animation"]["alpha"]["decoded"] == true
                    && r["animation"]["alpha"]["transparent_pixels"]
                        .as_u64()
                        .is_some_and(|n| n > 0)
                    && r["animation"]["alpha"]["opaque_pixels"]
                        .as_u64()
                        .is_some_and(|n| n > 0),
                "alpha was flattened"
            );
        }
    }
    // A declared JSON asset may share the launcher's private manifest name.
    // Make the visible artwork depend on its contents so a skipped copy cannot
    // masquerade as a successful render of the default scene.
    let asset_scene = SCENE.replace(
        "window.__timelines=",
        "const asset=new XMLHttpRequest();asset.open('GET','launch.json',false);asset.send();if(asset.status!==200)throw Error('declared JSON asset missing');const palette=JSON.parse(asset.responseText);document.querySelector('#mark').style.background=palette.color;window.__timelines=",
    );
    fs::write(root.join("asset-scene.html"), &asset_scene)?;
    fs::write(
        root.join("asset-composition.json"),
        serde_json::to_vec(
            &json!({"version":1,"entry":"asset-scene.html","assets":["tone.wav","launch.json"],"width":320,"height":180,"fps":30,"frames":30}),
        )?,
    )?;
    let mut asset_frames = Vec::new();
    for (color, output) in [
        ("#ff774a", "asset-orange.png"),
        ("#9b7dff", "asset-purple.png"),
    ] {
        fs::write(
            root.join("launch.json"),
            serde_json::to_vec(&json!({"color":color}))?,
        )?;
        let frame = good(
            &client,
            "composition_frame",
            json!({"source":"asset-composition.json","frame":29,"output":output}),
            &mut trace,
        )
        .await?;
        saved_png(&root, output, &frame)?;
        asset_frames.push(frame);
    }
    ensure!(
        asset_frames[0]["artifact"]["sha256"] != asset_frames[1]["artifact"]["sha256"]
            && fs::read(root.join("asset-scene.html"))? == asset_scene.as_bytes(),
        "changing the declared JSON palette did not change saved pixels or altered editable HTML"
    );
    trace.push(json!({"declared_launch_json_asset":{"visible_palette_change":true,"html_source_preserved":true,"outputs":["asset-orange.png","asset-purple.png"]}}));
    let original = fs::read(root.join("first.png"))?;
    for args in [
        json!({"source":"composition.json","frame":30,"output":"absent.png"}),
        json!({"source":"composition.json","frame":0,"output":"first.png"}),
        json!({"source":"composition.json","frame":0,"output":"first.png","overwrite":true,"expected_sha256":"wrong"}),
        json!({"source":"composition.json","frame":0,"output":"../escape.png"}),
        json!({"source":"composition.json","frame":0,"output":"scene.html","overwrite":true}),
        json!({"source":"composition.json","frame":0,"output":"absent-parent/frame.png"}),
    ] {
        bad(&client, "composition_frame", args, &mut trace).await?;
    }
    ensure!(
        fs::read(root.join("first.png"))? == original && !root.join("absent.png").exists(),
        "failure changed an output"
    );
    good(&client,"composition_frame",json!({"source":"composition.json","frame":1,"output":"first.png","overwrite":true,"expected_sha256":first["artifact"]["sha256"]}),&mut trace).await?;
    for scene in [
        SCENE.replace("tone.wav", "missing.wav"),
        SCENE.replace("tone.wav", "https://example.invalid/tone.wav"),
        SCENE.replace(
            "window.__timelines=",
            "throw Error('fixture page failure');window.__timelines=",
        ),
        SCENE.replace("data-duration=\"1\"", "data-duration=\"2\""),
    ] {
        fs::write(root.join("scene.html"), scene)?;
        bad(
            &client,
            "composition_render",
            json!({"source":"composition.json","output":"failed.mp4"}),
            &mut trace,
        )
        .await?;
        ensure!(!root.join("failed.mp4").exists(), "failed render published");
    }
    // Observe actual guest rendering before sending the host's cancellation notification.
    fs::write(
        root.join("scene.html"),
        SCENE.replace("window.__timelines=", "while(true){};window.__timelines="),
    )?;
    let handle = client
        .send_cancellable_request(
            ClientRequest::CallToolRequest(CallToolRequest::new(
                CallToolRequestParams::new("composition_render").with_arguments(
                    json!({"source":"composition.json","output":"cancelled.mp4"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )),
            PeerRequestOptions::no_options(),
        )
        .await?;
    let worker = active_render_worker(&engine).await?;
    let cancelled_at = Instant::now();
    handle
        .cancel(Some("cancel actual HyperFrames browser fixture".into()))
        .await?;
    worker_removed(&engine, &worker, job_scratch.path()).await?;
    fs::write(root.join("scene.html"), SCENE)?;
    // Cancellation is a notification: worker cleanup may still hold the
    // job permit when the next request arrives. Retry only that documented state.
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let args = json!({"source":"composition.json","frame":29,"output":"recovered.png"});
            let result = call(&client, "composition_frame", args.clone()).await?;
            if result.is_error == Some(true) {
                let raw = serde_json::to_value(&result)?;
                ensure!(
                    raw["content"].as_array().is_some_and(|items| items
                        .iter()
                        .any(|item| item["text"] == "editor is busy; retry after the current job")),
                    "post-cancellation frame failed: {result:?}"
                );
                trace.push(json!({"cancellation_cleanup_wait":result}));
                tokio::time::sleep(Duration::from_millis(25)).await;
                continue;
            }
            let value = result.structured_content.context("structured result")?;
            saved_png(&root, "recovered.png", &value)?;
            ensure!(
                value["artifact"]["sha256"] == last["artifact"]["sha256"],
                "recovery rendered different composition pixels"
            );
            trace.push(json!({"tool":"composition_frame","arguments":args,"result":value}));
            return Ok::<_, anyhow::Error>(());
        }
    })
    .await??;
    ensure!(
        cancelled_at.elapsed() < Duration::from_secs(10) && !root.join("cancelled.mp4").exists(),
        "cancellation failed to release worker without publication"
    );
    trace.push(
        json!({"cancellation":{"observed_worker_cid":worker.cid,"guest_processes":worker.guest_processes,"worker_removed":true,"staging_removed":true,"no_output":true,"next_render_succeeded":true}}),
    );
    // Let the same authored hang reach the real runtime deadline without cancellation.
    fs::write(
        root.join("scene.html"),
        SCENE.replace("window.__timelines=", "while(true){};window.__timelines="),
    )?;
    let timeout_at = Instant::now();
    let timeout_args = json!({"source":"composition.json","output":"timed-out.mp4"});
    let (failed, observed) = tokio::join!(
        call(&client, "composition_render", timeout_args.clone()),
        active_render_worker(&engine)
    );
    let failed = failed?;
    let timed_worker = observed?;
    ensure!(
        failed.is_error == Some(true),
        "hanging composition succeeded: {failed:?}"
    );
    let diagnostic = serde_json::to_string(&failed)?.to_ascii_lowercase();
    ensure!(
        ["timeout", "timed out", "deadline", "exceeded"]
            .iter()
            .any(|word| diagnostic.contains(word)),
        "hang failed without an actionable time-bound error: {failed:?}"
    );
    worker_removed(&engine, &timed_worker, job_scratch.path()).await?;
    ensure!(
        !root.join("timed-out.mp4").exists(),
        "timed-out composition published output"
    );
    trace.push(json!({"timeout":{"elapsed_ms":timeout_at.elapsed().as_millis(),"observed_worker_cid":timed_worker.cid,"guest_processes":timed_worker.guest_processes,"worker_removed":true,"staging_removed":true,"no_output":true,"error":failed}}));
    fs::write(root.join("scene.html"), SCENE)?;
    let recovered = good(
        &client,
        "composition_frame",
        json!({"source":"composition.json","frame":29,"output":"timeout-recovered.png"}),
        &mut trace,
    )
    .await?;
    saved_png(&root, "timeout-recovered.png", &recovered)?;
    ensure!(
        recovered["artifact"]["sha256"] == last["artifact"]["sha256"],
        "timeout recovery rendered different composition pixels"
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
    worker_removed(&engine, &final_worker, job_scratch.path()).await?;
    ensure!(
        fs::read_dir(&root)?.all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".mirket-editor-")),
        "staging leaked"
    );
    ensure!(
        fs::read_dir(job_scratch.path())?.next().is_none(),
        "native controller staging leaked"
    );
    trace.push(json!({"final_cleanup":{"last_worker_cid":final_worker.cid,"worker_removed":true,"native_staging_removed":true}}));
    fs::write(root.join("trace.json"), serde_json::to_vec_pretty(&trace)?)?;
    if std::env::var_os("MIRKET_HYPERFRAMES_EVIDENCE_DIR").is_some() {
        let saved = temporary.keep();
        eprintln!("HyperFrames contract evidence: {}", saved.display());
    }
    Ok(())
}
