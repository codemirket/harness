//! Real browser-backed design persistence, exports and cancellation with private Podman services.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]
use anyhow::{Context, Result, ensure};
use mirket::util::sha256;
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
    process::Stdio,
    time::{Duration, Instant},
};
type Client = RunningService<RoleClient, ()>;
const DOCUMENT: &str = include_str!("../runtime/packages/penpot/examples/field-notes.json");
struct Trace {
    file: PathBuf,
    entries: Vec<Value>,
}
impl Trace {
    fn push(&mut self, entry: Value) -> Result<()> {
        self.entries.push(entry);
        fs::write(&self.file, serde_json::to_vec_pretty(&self.entries)?)?;
        Ok(())
    }
}
async fn call(client: &Client, name: &str, args: Value) -> Result<CallToolResult> {
    Ok(tokio::time::timeout(
        Duration::from_secs(100),
        client.call_tool(
            CallToolRequestParams::new(name.to_owned())
                .with_arguments(args.as_object().context("arguments object")?.clone()),
        ),
    )
    .await??)
}
async fn good(client: &Client, name: &str, args: Value, trace: &mut Trace) -> Result<Value> {
    let started = Instant::now();
    let r = call(client, name, args.clone()).await?;
    let elapsed = started.elapsed().as_secs_f64();
    ensure!(r.is_error != Some(true), "{name}: {r:?}");
    let value = r.structured_content.context("structured result")?;
    trace.push(json!({"tool":name,"arguments":args,"result":value,"call_seconds":elapsed}))?;
    Ok(value)
}
async fn bad(client: &Client, name: &str, args: Value, trace: &mut Trace) -> Result<()> {
    let started = Instant::now();
    let r = call(client, name, args.clone()).await?;
    ensure!(r.is_error == Some(true), "invalid {name} succeeded: {r:?}");
    trace.push(json!({"tool":name,"arguments":args,"expected_error":r,"call_seconds":started.elapsed().as_secs_f64()}))?;
    Ok(())
}
async fn group_alive(group: u32) -> Result<bool> {
    Ok(tokio::process::Command::new("/bin/kill")
        .env_clear()
        .args(["-0", "--", &format!("-{group}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await?
        .success())
}
async fn browser_group(jobs: &Path) -> Result<u32> {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            for entry in fs::read_dir(jobs)? {
                let stage = entry?.path();
                if !stage.join("job.json").is_file() {
                    continue;
                }
                let output = tokio::process::Command::new("/bin/ps")
                    .env_clear()
                    .args(["-axo", "pgid=,command="])
                    .output()
                    .await?;
                for line in String::from_utf8(output.stdout)?.lines().filter(|l| {
                    l.contains(stage.to_str().unwrap()) && l.contains("--user-data-dir=")
                }) {
                    if let Some(group) = line
                        .split_whitespace()
                        .next()
                        .and_then(|g| g.parse::<u32>().ok())
                    {
                        return Ok::<_, anyhow::Error>(group);
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .context("no live browser process group observed")?
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
#[ignore = "requires pinned Penpot installed and started in an explicitly marked disposable home"]
async fn penpot_real_persistence_refinement_exports_failures_and_cancellation() -> Result<()> {
    let home = PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME").context("explicit disposable home")?,
    );
    ensure!(home.is_absolute(), "absolute home required");
    let home = home.canonicalize()?;
    ensure!(
        fs::read(home.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "marker required"
    );
    let parent = if let Some(p) = std::env::var_os("MIRKET_PENPOT_EVIDENCE_DIR") {
        let p = PathBuf::from(p);
        ensure!(
            p.is_absolute() && p.is_dir(),
            "existing absolute evidence directory required"
        );
        p.canonicalize()?
    } else {
        std::env::temp_dir().canonicalize()?
    };
    let temp = tempfile::Builder::new()
        .prefix("penpot-contract-")
        .tempdir_in(parent)?;
    let mut temp = Some(temp);
    let directory = if std::env::var_os("MIRKET_PENPOT_EVIDENCE_DIR").is_some() {
        let directory = temp.take().unwrap().keep();
        eprintln!("Penpot contract evidence: {}", directory.display());
        directory
    } else {
        temp.as_ref().unwrap().path().to_owned()
    };
    let root = directory.join("project");
    let private_jobs = tempfile::Builder::new()
        .prefix("penpot-jobs-")
        .tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let jobs = private_jobs.path();
    fs::create_dir(&root)?;
    // A convincing installation/probe can miss registration failure. Force the
    // actual backend's password-policy rejection and verify safe recovery of a
    // pending, unregistered account without disclosing its private password.
    let scope = sha256(root.canonicalize()?.as_os_str().as_encoded_bytes());
    let accounts = home.join(".mirket/packages/data/penpot/service/browser/accounts");
    fs::create_dir_all(&accounts)?;
    let credentials_file = accounts.join(format!("{scope}.json"));
    let pending_credentials = serde_json::to_vec(&json!({
        "email":format!("{}@mirket.invalid", &scope[..32]),
        "password":"PendingOnlyPasswordWithoutDigitOrSpecial"
    }))?;
    fs::write(&credentials_file, &pending_credentials)?;
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&accounts, fs::Permissions::from_mode(0o700))?;
        fs::set_permissions(&credentials_file, fs::Permissions::from_mode(0o600))?;
    }
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command
        .arg("--home")
        .arg(&home)
        .args(["package", "serve", "penpot", "--project"])
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
        "penpot_create",
        "penpot_inspect",
        "penpot_update",
        "penpot_export",
    ];
    ensure!(
        tools.len() == 4
            && expected.iter().all(|n| tools
                .iter()
                .any(|t| t.name == *n && t.output_schema.is_some())),
        "discovery or schemas incorrect"
    );
    let mut trace = Trace {
        file: root.join("trace.json"),
        entries: vec![],
    };
    let service: Value = serde_json::from_slice(&fs::read(
        home.join(".mirket/packages/data/penpot/service/state.json"),
    )?)?;
    let engine = mirket::podman::Engine::resolve(&mirket::paths::Paths::new(home.clone())?)?;
    ensure!(
        service["engine_owner"] == engine.owner(),
        "Penpot not bound to the private engine"
    );
    let runtime = engine.ready()?;
    trace.push(json!({"discovery":tools,"services":{"engine_owner":engine.owner(),"architecture":runtime["host"]["arch"],"rootless":runtime["host"]["security"]["rootless"],"images":service["images"]},"adapter":"Native managed Node and sandboxed Chromium connect only to the published loopback service; this is a verified native exception."}))?;
    let doc: Value = serde_json::from_str(DOCUMENT)?;
    let created = good(
        &client,
        "penpot_create",
        json!({"document":doc,"output":"field.penpot.json"}),
        &mut trace,
    )
    .await?;
    let recovered = fs::read(&credentials_file)?;
    let account: Value = serde_json::from_slice(&recovered)?;
    let password = account["password"].as_str().context("private password")?;
    ensure!(
        recovered != pending_credentials
            && password.len() >= 43
            && password.chars().any(|c| c.is_ascii_lowercase())
            && password.chars().any(|c| c.is_ascii_uppercase())
            && password.chars().any(|c| c.is_ascii_digit())
            && password.contains('!'),
        "pending account was not safely recovered to the pinned password policy"
    );
    let credentials_identity = sha256(&recovered);
    trace.push(json!({"registration_recovery":{"forced_pending_password_rejection":true,"authenticated_recovery":true,"required_character_classes":true,"private_password_disclosed":false}}))?;
    ensure!(
        created["design"]["nodes"]
            .as_array()
            .is_some_and(|n| n.len() == 9),
        "real editable shapes missing"
    );
    let original = fs::read(root.join("field.penpot.json"))?;
    let identity = sha256(&original);
    // A credential corruption must not turn registration retry into a reset of
    // an existing account. Restore the fixture before evaluating the refusal.
    let mut corrupt_account = account.clone();
    corrupt_account["password"] = json!("WrongPendingPasswordWithoutDigitOrSpecial");
    let corrupt = serde_json::to_vec(&corrupt_account)?;
    fs::write(&credentials_file, &corrupt)?;
    let rejected = call(
        &client,
        "penpot_inspect",
        json!({"source":"field.penpot.json"}),
    )
    .await;
    let preserved = fs::read(&credentials_file)? == corrupt;
    fs::write(&credentials_file, &recovered)?;
    let rejected = rejected?;
    ensure!(
        rejected.is_error == Some(true)
            && serde_json::to_string(&rejected)?.contains("email-already-exists")
            && preserved,
        "existing-account refusal changed private credentials or succeeded"
    );
    trace.push(json!({"registration_existing_account":{"corrupt_private_fixture_preserved":true,"existing_account_refused":true,"original_fixture_restored":true,"error":rejected}}))?;
    let inspected = good(
        &client,
        "penpot_inspect",
        json!({"source":"field.penpot.json"}),
        &mut trace,
    )
    .await?;
    let rev = inspected["design"]["revision"]
        .as_u64()
        .context("current native revision")?;
    let nodes = inspected["design"]["nodes"].as_array().context("nodes")?;
    ensure!(
        nodes
            .iter()
            .any(|n| n["key"] == "button-label" && n["text"] == "EXPLORE THE JOURNAL  →")
            && nodes.iter().filter(|n| n["type"] == "path").count() == 2,
        "typed text/vector paths not retained"
    );
    for (format, ext) in [("png", "png"), ("svg", "svg"), ("penpot", "penpot")] {
        let v=good(&client,"penpot_export",json!({"source":"field.penpot.json","object_key":"poster","format":format,"output":format!("field.{ext}")}),&mut trace).await?;
        ensure!(
            v["artifact"]["bytes"].as_u64().is_some_and(|b| b > 1000),
            "empty or fake export"
        );
        if format == "svg" {
            let svg = fs::read_to_string(root.join("field.svg"))?;
            ensure!(
                svg.contains("data:font/woff;base64,")
                    && !svg.contains("http://127.0.0.1")
                    && !svg.contains("http://localhost"),
                "saved SVG still depends on service fonts"
            );
        }
    }
    let refined=good(&client,"penpot_update",json!({"source":"field.penpot.json","expected_revision":rev,"patches":[{"key":"heading","text":"Go where the\nquiet takes you."},{"key":"sun","opacity":0.7}],"output":"refined.penpot.json"}),&mut trace).await?;
    ensure!(
        refined["design"]["file_id"] != inspected["design"]["file_id"]
            && refined["editable_source"] == "refined.penpot.json",
        "refinement did not preserve source identity"
    );
    let readback = good(
        &client,
        "penpot_inspect",
        json!({"source":"refined.penpot.json"}),
        &mut trace,
    )
    .await?;
    ensure!(
        readback["design"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["key"] == "heading" && n["text"] == "Go where the\nquiet takes you."),
        "refined text not persisted"
    );
    let unchanged = good(
        &client,
        "penpot_inspect",
        json!({"source":"field.penpot.json"}),
        &mut trace,
    )
    .await?;
    ensure!(
        unchanged["design"]["revision"] == rev
            && unchanged["design"]["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["key"] == "heading" && n["text"] == "A quieter kind\nof adventure."),
        "original native document changed"
    );
    good(&client,"penpot_export",json!({"source":"refined.penpot.json","object_key":"poster","format":"png","output":"refined.png"}),&mut trace).await?;
    for args in [
        json!({"source":"field.penpot.json","format":"png","object_key":"poster","output":"field.png"}),
        json!({"source":"field.penpot.json","format":"png","object_key":"poster","output":"field.png","overwrite":true,"expected_sha256":"wrong"}),
        json!({"source":"../field.penpot.json","format":"png","object_key":"poster","output":"failed.png"}),
        json!({"source":"field.penpot.json","format":"png","object_key":"poster","output":"../escape.png"}),
        json!({"source":"field.penpot.json","format":"svg","object_key":"unknown","output":"failed.svg"}),
        json!({"source":"field.penpot.json","format":"png","object_key":"poster","output":"missing/failed.png"}),
        json!({"source":"field.penpot.json","format":"png","object_key":"poster","output":"failed.png","script":"alert(1)"}),
    ] {
        bad(&client, "penpot_export", args, &mut trace).await?;
    }
    bad(&client,"penpot_update",json!({"source":"field.penpot.json","expected_revision":rev+10,"patches":[{"key":"heading","text":"unexpected"}],"output":"failed.penpot.json"}),&mut trace).await?;
    let mut forged: Value = serde_json::from_slice(&original)?;
    forged["project_scope"] = json!("f".repeat(64));
    fs::write(
        root.join("forged.penpot.json"),
        serde_json::to_vec(&forged)?,
    )?;
    bad(
        &client,
        "penpot_inspect",
        json!({"source":"forged.penpot.json"}),
        &mut trace,
    )
    .await?;
    let mut invalid = doc.clone();
    invalid["nodes"][2]["path"] = json!("M0 0LZ");
    bad(
        &client,
        "penpot_create",
        json!({"document":invalid,"output":"malformed.penpot.json"}),
        &mut trace,
    )
    .await?;
    let mut overflow = doc.clone();
    overflow["nodes"][2]["path"] = json!("M1e999 0L0 0L1 1Z");
    bad(
        &client,
        "penpot_create",
        json!({"document":overflow,"output":"overflow.penpot.json"}),
        &mut trace,
    )
    .await?;
    fs::hard_link(
        root.join("field.penpot.json"),
        root.join("source-alias.penpot"),
    )?;
    std::os::unix::fs::symlink(root.join("field.penpot.json"), root.join("linked.penpot"))?;
    for name in ["source-alias.penpot", "linked.penpot"] {
        bad(
            &client,
            "penpot_export",
            json!({"source":"field.penpot.json","format":"penpot","output":name,"overwrite":true}),
            &mut trace,
        )
        .await?;
    }
    ensure!(
        sha256(&fs::read(root.join("field.penpot.json"))?) == identity
            && !root.join("failed.png").exists()
            && !root.join("failed.svg").exists()
            && !root.join("failed.penpot.json").exists()
            && !root.join("malformed.penpot.json").exists()
            && !root.join("overflow.penpot.json").exists(),
        "failure damaged source or published an artifact"
    );
    let handle=client.send_cancellable_request(ClientRequest::CallToolRequest(CallToolRequest::new(CallToolRequestParams::new("penpot_export".to_owned()).with_arguments(json!({"source":"field.penpot.json","object_key":"poster","format":"png","output":"cancelled.png"}).as_object().unwrap().clone()))),PeerRequestOptions::no_options()).await?;
    let group = browser_group(jobs).await?;
    trace.push(json!({"cancellation_started":{"observed_browser_group":group,"private_jobs_root":jobs,"first_browser_process_observed":true}}))?;
    let cancelled_at = Instant::now();
    handle
        .cancel(Some("explicit design contract cancellation".into()))
        .await?;
    let cleanup = tokio::time::timeout(Duration::from_secs(8), async {
        while group_alive(group).await? || fs::read_dir(jobs)?.next().is_some() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok::<_, anyhow::Error>(())
    })
    .await;
    let remaining_staging: Vec<_> = fs::read_dir(jobs)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<_>>()?;
    trace.push(json!({"cancellation_observed":{"elapsed_seconds":cancelled_at.elapsed().as_secs_f64(),"group_alive":group_alive(group).await?,"remaining_staging":remaining_staging,"artifact_published":root.join("cancelled.png").exists(),"eight_second_cleanup_bound_met":matches!(&cleanup, Ok(Ok(())))}}))?;
    cleanup.context("cancelled browser group or staging survived")??;
    ensure!(
        !root.join("cancelled.png").exists(),
        "cancelled artifact was published"
    );
    trace.push(json!({"cancellation":{"observed_browser_group":group,"group_reaped":true,"staging_removed":true,"artifact_published":false}}))?;
    good(
        &client,
        "penpot_inspect",
        json!({"source":"field.penpot.json"}),
        &mut trace,
    )
    .await?;
    // Measure useful warm calls over one native MCP session and already-running
    // services. Each inspection still launches its disposable native browser.
    let measured = Instant::now();
    let mut samples = Vec::new();
    for _ in 0..10 {
        let started = Instant::now();
        let observed = good(
            &client,
            "penpot_inspect",
            json!({"source":"field.penpot.json"}),
            &mut trace,
        )
        .await?;
        samples.push(started.elapsed().as_secs_f64());
        ensure!(
            observed["design"]["revision"] == rev,
            "warm call changed document"
        );
    }
    let total = measured.elapsed().as_secs_f64();
    let mut sorted = samples.clone();
    sorted.sort_by(f64::total_cmp);
    trace.push(json!({"warm_inspect":{"samples_seconds":samples,"p50_seconds":sorted[4],"p95_seconds":sorted[9],"percentile_method":"nearest rank; 10 sequential calls","throughput_calls_per_second":10.0/total,"elapsed_seconds":total,"services_already_running":true,"native_mcp_session_reused":true,"disposable_browser_per_call":true}}))?;
    ensure!(
        sha256(&fs::read(&credentials_file)?) == credentials_identity,
        "working account credentials changed during design operations"
    );
    client.close().await?;
    staging_stays_empty(jobs).await?;
    trace.push(
        json!({"final_cleanup":{"private_jobs_root":jobs,"host_temp":true,"stable_empty_ms":1000}}),
    )?;
    Ok(())
}
