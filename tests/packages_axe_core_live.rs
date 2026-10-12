//! Actual local DOM states, scoped saved reports and an inaccessible control probe.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]
use anyhow::{Context, Result, ensure};
use rmcp::{
    ServiceExt,
    model::{CallToolRequest, CallToolRequestParams, CallToolResult, ClientRequest},
    service::PeerRequestOptions,
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

const PAGE: &str = r#"<!doctype html><html lang="en"><meta charset="utf-8"><title>Local audit specimen</title>
<style>body{font:18px system-ui;margin:32px;color:#142631;background:#fff}button{padding:12px}dialog{padding:24px}</style>
<main><h1>Account preferences</h1><button id="open">Edit preference</button><dialog id="modal"><h2>Preferences</h2><label for="name">Name</label><input id="name"><button id="unnamed"></button><button id="close">Close</button></dialog></main>
<script>document.querySelector('#open').onclick=()=>document.querySelector('#modal').showModal();document.querySelector('#close').onclick=()=>document.querySelector('#modal').close();</script></html>"#;

struct Server {
    port: u16,
    stop: Arc<AtomicBool>,
    pending: Arc<AtomicBool>,
    release: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Server {
    fn new() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        listener.set_nonblocking(true)?;
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let pending = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let observed = pending.clone();
        let resume = release.clone();
        let thread = std::thread::spawn(move || {
            while !signal.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut socket, _)) => {
                        let _ = socket.set_read_timeout(Some(Duration::from_secs(1)));
                        let mut request = [0; 4096];
                        if let Ok(n) = socket.read(&mut request) {
                            let request = String::from_utf8_lossy(&request[..n]);
                            if request.starts_with("GET /blocked ") {
                                observed.store(true, Ordering::Relaxed);
                                while !resume.load(Ordering::Relaxed)
                                    && !signal.load(Ordering::Relaxed)
                                {
                                    std::thread::sleep(Duration::from_millis(10));
                                }
                            }
                            let body = if request.starts_with("GET /empty ") {
                                "<!doctype html><html><title>Empty</title><body></body></html>"
                            } else if request.starts_with("GET /external ") {
                                "<!doctype html><html><title>Incomplete</title><main>Unloaded media</main><img src='https://example.invalid/missing.png'></html>"
                            } else {
                                PAGE
                            };
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                                body.len()
                            );
                            let _ = socket.write_all(response.as_bytes());
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            port,
            stop,
            pending,
            release,
            thread: Some(thread),
        })
    }
}
struct BrowserGroup(u32);
impl Drop for BrowserGroup {
    fn drop(&mut self) {
        // Clean this deliberately stopped group even if the probe fails.
        let _ = std::process::Command::new("/bin/kill")
            .env_clear()
            .args(["-KILL", "--", &format!("-{}", self.0)])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn structured(result: CallToolResult) -> Result<Value> {
    ensure!(result.is_error != Some(true), "audit error: {result:?}");
    result
        .structured_content
        .context("structured result missing")
}

#[tokio::test]
#[ignore = "real Chromium and explicitly marked disposable home"]
async fn packages_axe_core_live_rendered_states_and_failure_probes() -> Result<()> {
    let home = PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME")
            .context("explicit disposable home required")?,
    );
    ensure!(
        home.is_absolute()
            && fs::read(home.join(".mirket-package-test-home"))?
                == b"disposable package test home\n",
        "disposable home marker required"
    );
    let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let root = project.path().canonicalize()?;
    let jobs = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let server = Server::new()?;
    let url = format!("http://127.0.0.1:{}/", server.port);
    let mut cmd = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    cmd.arg("--home")
        .arg(&home)
        .args(["package", "serve", "axe-core", "--project"])
        .arg(&root)
        .env("TMPDIR", jobs.path())
        .kill_on_drop(true);
    let mut client = tokio::time::timeout(
        Duration::from_secs(90),
        ().serve(TokioChildProcess::new(cmd)?),
    )
    .await??;
    let tools = client.list_all_tools().await?;
    ensure!(
        tools.len() == 1
            && tools[0].name == "accessibility_audit"
            && tools[0].output_schema.is_some(),
        "unexpected tool contract"
    );
    let mut trace = vec![json!({"discovery":tools})];
    for (output, actions, width) in [
        ("initial.json", json!([]), 1280),
        (
            "modal.json",
            json!([{"action":"click","selector":"#open"},{"action":"fill","selector":"#name","value":"Local tester"}]),
            390,
        ),
    ] {
        let arguments = json!({"url":url,"output":output,"actions":actions,"width":width});
        let result = client
            .call_tool(
                CallToolRequestParams::new("accessibility_audit")
                    .with_arguments(arguments.as_object().unwrap().clone()),
            )
            .await?;
        let value = structured(result)?;
        let report: Value = serde_json::from_slice(&fs::read(root.join(output))?)?;
        ensure!(report["testEngine"]["version"] == "4.14.0", "wrong engine");
        let unnamed = report["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"] == "button-name");
        ensure!(
            unnamed == (output == "modal.json"),
            "hidden and visible dialog states were not distinguished"
        );
        trace.push(json!({"arguments":arguments,"result":value,"report":report}));
    }
    for arguments in [
        json!({"url":url,"output":"modal.json"}),
        json!({"url":url,"output":"../escaped.json"}),
        json!({"url":"https://example.com","output":"bad.json"}),
        json!({"url":format!("{url}empty"),"output":"empty.json"}),
        json!({"url":format!("{url}external"),"output":"external.json"}),
        json!({"url":url,"output":"script.json","actions":[{"action":"evaluate","script":"alert(1)"}]}),
    ] {
        let result = client
            .call_tool(
                CallToolRequestParams::new("accessibility_audit")
                    .with_arguments(arguments.as_object().unwrap().clone()),
            )
            .await?;
        ensure!(
            result.is_error == Some(true),
            "failure accepted: {arguments}: {result:?}"
        );
        trace.push(json!({"failure_probe":arguments,"result":result}));
    }
    std::os::unix::fs::symlink("initial.json", root.join("alias.json"))?;
    let args = json!({"url":url,"output":"alias.json","overwrite":true});
    let result = client
        .call_tool(
            CallToolRequestParams::new("accessibility_audit")
                .with_arguments(args.as_object().unwrap().clone()),
        )
        .await?;
    ensure!(result.is_error == Some(true), "symlink write accepted");
    fs::remove_file(root.join("alias.json"))?;
    for path in ["empty.json", "external.json", "bad.json", "script.json"] {
        ensure!(
            !root.join(path).exists(),
            "failed call published unusable evidence"
        );
    }
    // Cancel only after observing the real detached Chromium group. A returned
    // MCP error alone cannot establish that browser writes and staging stopped.
    let handle = client
        .send_cancellable_request(
            ClientRequest::CallToolRequest(CallToolRequest::new(
                CallToolRequestParams::new("accessibility_audit").with_arguments(
                    json!({"url":format!("{url}blocked"),"output":"cancelled.json"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )),
            PeerRequestOptions::no_options(),
        )
        .await?;
    let group = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let output = tokio::process::Command::new("/bin/ps")
                .env_clear()
                .args(["-axo", "pgid=,command="])
                .output()
                .await?;
            for line in String::from_utf8(output.stdout)?.lines() {
                if line.contains(jobs.path().to_str().unwrap())
                    && line.contains("--user-data-dir=")
                    && let Some(group) = line
                        .split_whitespace()
                        .next()
                        .and_then(|pid| pid.parse::<u32>().ok())
                {
                    return Ok::<_, anyhow::Error>(group);
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .context("actual audit browser was not observed")??;
    let group_guard = BrowserGroup(group);
    tokio::time::timeout(Duration::from_secs(10), async {
        while !server.pending.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .context("browser did not reach pending navigation")?;
    ensure!(
        tokio::process::Command::new("/bin/kill")
            .env_clear()
            .args(["-STOP", "--", &format!("-{group}")])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?
            .success(),
        "could not pause owned browser for forced cleanup probe"
    );
    handle
        .cancel(Some("explicit native audit cancellation".into()))
        .await?;
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            let alive = tokio::process::Command::new("/bin/kill")
                .env_clear()
                .args(["-0", "--", &format!("-{group}")])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await?
                .success();
            if !alive && fs::read_dir(jobs.path())?.next().is_none() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        for _ in 0..50 {
            ensure!(
                fs::read_dir(jobs.path())?.next().is_none(),
                "audit staging reappeared"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok::<_, anyhow::Error>(())
    })
    .await
    .context("cancelled native audit browser or staging survived")??;
    ensure!(
        !root.join("cancelled.json").exists(),
        "cancelled audit published output"
    );
    drop(group_guard);
    server.release.store(true, Ordering::Relaxed);
    let recovered = structured(
        client
            .call_tool(
                CallToolRequestParams::new("accessibility_audit").with_arguments(
                    json!({"url":url,"output":"recovered.json"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await?,
    )?;
    let recovery: Value = serde_json::from_slice(&fs::read(root.join("recovered.json"))?)?;
    ensure!(
        recovery["testEngine"]["version"] == "4.14.0",
        "audit recovery unusable"
    );
    trace.push(
        json!({"cancellation":{"observed_browser_group":group,"stalled_browser_close":true,"group_reaped":true,
        "staging_removed_and_stable":true,"artifact_published":false},"recovery":recovered}),
    );
    client.close().await?;
    if let Some(directory) = std::env::var_os("MIRKET_AXE_EVIDENCE_DIR") {
        let directory = PathBuf::from(directory);
        ensure!(
            directory.is_absolute() && directory.is_dir(),
            "absolute evidence directory required"
        );
        fs::write(
            directory.join("trace.json"),
            serde_json::to_vec_pretty(&trace)?,
        )?;
        for name in ["initial.json", "modal.json", "recovered.json"] {
            fs::copy(root.join(name), directory.join(name))?;
        }
    }
    Ok(())
}
