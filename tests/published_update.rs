use anyhow::{Context, Result, bail, ensure};
use mirket::{
    update::{ReleaseManifest, target_triple, validate_manifest},
    util::sha256,
};
use reqwest::{blocking::Client, redirect::Policy};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::Path,
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

const MANIFEST_URL: &str =
    "https://github.com/codemirket/harness/releases/latest/download/mirket-release.json";
const OUTPUT_LIMIT: u64 = 1024 * 1024;

fn published_manifest() -> Result<ReleaseManifest> {
    let client = Client::builder()
        .user_agent(concat!("mirket-release-test/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(90))
        .redirect(Policy::custom(|attempt| {
            if attempt.previous().len() >= 5
                || attempt.url().scheme() != "https"
                || !matches!(
                    attempt.url().host_str(),
                    Some(
                        "github.com"
                            | "release-assets.githubusercontent.com"
                            | "objects.githubusercontent.com"
                    )
                )
            {
                return attempt.error("unexpected official release redirect");
            }
            attempt.follow()
        }))
        .build()?;
    let response = client
        .get(MANIFEST_URL)
        .send()
        .context("download the published GitHub release manifest")?
        .error_for_status()
        .context("a public latest release with mirket-release.json is required")?;
    ensure!(
        response
            .content_length()
            .is_none_or(|length| length <= OUTPUT_LIMIT),
        "release manifest exceeds 1 MiB"
    );
    let mut bytes = Vec::new();
    response.take(OUTPUT_LIMIT + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= OUTPUT_LIMIT, "manifest exceeds 1 MiB");
    let manifest: ReleaseManifest = serde_json::from_slice(&bytes)?;
    validate_manifest(&manifest)?;
    ensure!(
        manifest.version == env!("CARGO_PKG_VERSION"),
        "latest release {} does not match source {}; test the published source revision",
        manifest.version,
        env!("CARGO_PKG_VERSION")
    );
    Ok(manifest)
}

fn invoke(binary: &Path, home: &Path, args: &[&str], timeout: Duration) -> Result<Output> {
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = Command::new(binary)
        .env("MIRKET_DIRECT", "1")
        .arg("--home")
        .arg(home)
        .args(args)
        .stdin(Stdio::null())
        .stdout(stdout.try_clone()?)
        .stderr(stderr.try_clone()?)
        .spawn()
        .with_context(|| format!("start {} {args:?}", binary.display()))?;
    let result = (|| {
        let started = Instant::now();
        loop {
            ensure!(
                stdout.metadata()?.len() <= OUTPUT_LIMIT
                    && stderr.metadata()?.len() <= OUTPUT_LIMIT,
                "{args:?} exceeded the 1 MiB output limit"
            );
            if let Some(status) = child.try_wait()? {
                stdout.seek(SeekFrom::Start(0))?;
                stderr.seek(SeekFrom::Start(0))?;
                let mut out = Vec::new();
                let mut err = Vec::new();
                stdout.by_ref().take(OUTPUT_LIMIT).read_to_end(&mut out)?;
                stderr.by_ref().take(OUTPUT_LIMIT).read_to_end(&mut err)?;
                return Ok(Output {
                    status,
                    stdout: out,
                    stderr: err,
                });
            }
            if started.elapsed() >= timeout {
                bail!("{args:?} exceeded {} seconds", timeout.as_secs());
            }
            thread::sleep(Duration::from_millis(25));
        }
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}; stderr: {}; stdout: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr[..output.stderr.len().min(4096)]),
        String::from_utf8_lossy(&output.stdout[..output.stdout.len().min(4096)])
    )
}

fn successful_json(binary: &Path, home: &Path, args: &[&str], timeout: Duration) -> Result<Value> {
    let output = invoke(binary, home, args, timeout)?;
    ensure!(
        output.status.success(),
        "{args:?}: {}",
        diagnostics(&output)
    );
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{args:?} did not return JSON: {}", diagnostics(&output)))
}

#[test]
#[ignore = "downloads and installs the public latest release in a disposable home"]
fn published_update_restores_setup_and_installs_verified_release() -> Result<()> {
    let manifest = published_manifest()?;
    let target = target_triple()?;
    ensure!(
        target == env!("MIRKET_TARGET"),
        "test target does not match build"
    );
    let asset = manifest
        .assets
        .iter()
        .find(|asset| asset.target == target)
        .with_context(|| format!("published release has no artifact for {target}"))?;
    let expected_digest = asset.sha256.to_ascii_lowercase();
    let source = Path::new(env!("CARGO_BIN_EXE_mirket"));
    ensure!(
        sha256(&fs::read(source)?) != expected_digest,
        "run this probe with the debug test build to exercise actual binary replacement"
    );

    let temporary = tempfile::tempdir()?;
    let home = temporary.path().canonicalize()?;
    fs::create_dir(home.join(".codex"))?;
    fs::write(
        home.join(".codex/config.toml"),
        "# Preserve this personal configuration\nmodel = \"published-update-sentinel\"\napproval_policy = \"on-request\"\n[mcp_servers.personal]\ncommand = \"personal-tool-sentinel\"\n",
    )?;
    fs::write(
        home.join(".claude.json"),
        serde_json::to_vec_pretty(&json!({
            "theme":"dark",
            "oauthAccount":{"accountUuid":"published-update-sentinel"},
            "mcpServers":{"personal":{"type":"stdio","command":"personal-tool-sentinel"}}
        }))?,
    )?;
    let local_timeout = Duration::from_secs(60);
    successful_json(
        source,
        &home,
        &[
            "setup",
            "--target",
            "all",
            "--microsoft-learn",
            "--yes",
            "--json",
        ],
        local_timeout,
    )?;
    let healthy = successful_json(source, &home, &["doctor", "--json"], local_timeout)?;
    ensure!(healthy["ok"] == true, "initial doctor is not healthy");
    let receipt_path = home.join(".mirket/setup.json");
    let before: Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    ensure!(
        before["options"] == json!({"targets":["codex","claude"],"microsoft_learn":true}),
        "setup did not save both clients and Microsoft Learn"
    );
    let codex_config = fs::read(home.join(".codex/config.toml"))?;
    let claude_config = fs::read(home.join(".claude.json"))?;
    let codex: toml_edit::DocumentMut = std::str::from_utf8(&codex_config)?.parse()?;
    ensure!(
        codex["model"].as_str() == Some("published-update-sentinel")
            && codex["approval_policy"].as_str() == Some("on-request")
            && codex["mcp_servers"]["personal"]["command"].as_str()
                == Some("personal-tool-sentinel"),
        "setup changed unrelated Codex configuration"
    );
    let claude: Value = serde_json::from_slice(&claude_config)?;
    ensure!(
        claude["theme"] == "dark"
            && claude["oauthAccount"]["accountUuid"] == "published-update-sentinel"
            && claude["mcpServers"]["personal"]["command"] == "personal-tool-sentinel",
        "setup changed unrelated Claude configuration"
    );
    ensure!(
        codex["mcp_servers"]["microsoft-learn"]["url"].as_str()
            == Some("https://learn.microsoft.com/api/mcp")
            && claude["mcpServers"]["microsoft-learn"]["url"]
                == "https://learn.microsoft.com/api/mcp",
        "setup did not enable Microsoft Learn in both clients"
    );

    let removed_relative = before["files"]
        .as_object()
        .context("setup receipt omitted managed files")?
        .keys()
        .find(|path| path.starts_with(".agents/skills/") && path.ends_with("/SKILL.md"))
        .context("setup receipt omitted Codex global skill files")?;
    let removed = home.join(removed_relative);
    let removed_bytes = fs::read(&removed)?;
    ensure!(
        before["files"][removed_relative]["sha256"] == sha256(&removed_bytes),
        "selected failure probe is not an intact managed skill"
    );
    fs::remove_file(&removed)?;
    let broken = invoke(source, &home, &["doctor", "--json"], local_timeout)?;
    ensure!(
        !broken.status.success(),
        "doctor accepted a missing managed skill"
    );
    let broken_report: Value = serde_json::from_slice(&broken.stdout).with_context(|| {
        format!(
            "failure doctor returned no report: {}",
            diagnostics(&broken)
        )
    })?;
    ensure!(
        broken_report["ok"] == false
            && broken_report["installation"]["checks"]
                .as_array()
                .context("doctor omitted installation checks")?
                .iter()
                .any(|check| check["name"] == "managed files" && check["status"] == "fail"),
        "doctor did not identify missing managed files"
    );

    let managed = home.join(".mirket/bin").join(if cfg!(windows) {
        "mirket.exe"
    } else {
        "mirket"
    });
    let updated = successful_json(
        &managed,
        &home,
        &["update", "--json"],
        Duration::from_secs(360),
    )?;
    ensure!(
        updated["updated"] == true
            && updated["reconfigured"] == true
            && updated["current_version"] == manifest.version
            && updated["available_version"] == manifest.version,
        "online update did not replace the binary and reapply the saved release: {updated}"
    );
    let after: Value = serde_json::from_slice(&fs::read(&receipt_path)?)?;
    ensure!(
        after["options"] == before["options"],
        "update changed saved setup choices"
    );
    ensure!(
        fs::read(&removed)? == removed_bytes,
        "update did not restore the missing skill"
    );
    ensure!(
        fs::read(home.join(".codex/config.toml"))? == codex_config
            && fs::read(home.join(".claude.json"))? == claude_config,
        "update changed unrelated client settings or the saved MCP configuration"
    );
    ensure!(
        home.join(".codex/AGENTS.md").is_file() && home.join(".claude/CLAUDE.md").is_file(),
        "update removed a selected client's instructions"
    );
    ensure!(
        sha256(&fs::read(&managed)?) == expected_digest,
        "installed managed binary differs from the official published SHA-256"
    );
    let healthy = successful_json(&managed, &home, &["doctor", "--json"], local_timeout)?;
    ensure!(
        healthy["ok"] == true && healthy["mcp"]["ok"] == true,
        "published runtime doctor failed"
    );
    let info = successful_json(&managed, &home, &["info", "--json"], local_timeout)?;
    ensure!(
        info == json!({"name":"mirket","version":manifest.version,"target":target}),
        "published runtime reported an unexpected identity: {info}"
    );
    let version = invoke(&managed, &home, &["--version"], local_timeout)?;
    ensure!(
        version.status.success()
            && std::str::from_utf8(&version.stdout)?.trim()
                == format!("mirket {}", manifest.version),
        "published runtime version failed: {}",
        diagnostics(&version)
    );
    Ok(())
}
