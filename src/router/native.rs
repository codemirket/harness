//! Narrow native CLI adapters for authenticated model discovery and Claude turns.
//! The router owns all tool execution; these children never receive native tools.

use crate::{mcp::BoundedLines, process::OwnedChild};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Lines},
    process::{ChildStdin, ChildStdout, Command},
    time::{Instant, timeout, timeout_at},
};

const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(30);
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(600);
const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
const MAX_STREAM_BYTES: usize = 16 * 1024 * 1024;
const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;
const MAX_IMAGES: usize = 4;
const EMPTY_MCP: &str = r#"{"mcpServers":{}}"#;

type JsonLines = Lines<BufReader<BoundedLines<ChildStdout>>>;

pub(super) fn native_command(executable: &Path, home: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .current_dir(home)
        .env("HOME", home)
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if std::env::var_os("HOME")
        .and_then(|p| PathBuf::from(p).canonicalize().ok())
        .as_deref()
        != Some(home)
    {
        // Explicit disposable homes must never inherit the real user's auth.
        command
            .env_remove("CLAUDE_CONFIG_DIR")
            .env_remove("CODEX_HOME");
    }
    command
}

/// Recheck account state without creating a thread or requesting inference.
pub async fn codex_authenticated(executable: &Path, home: &Path) -> Result<()> {
    let kind = codex_account_kind(executable, home).await?;
    super::codex::upstream(&kind)?;
    codex_execution_contract(executable, home).await
}
pub(super) async fn codex_execution_contract(executable: &Path, home: &Path) -> Result<()> {
    let mut command = native_command(executable, home);
    command.args(["exec", "--help"]);
    let mut owned = OwnedChild::spawn(&mut command, false)?;
    drop(owned.child().stdin.take());
    let stdout = owned
        .child()
        .stdout
        .take()
        .context("Codex help output missing")?;
    let mut bytes = Vec::new();
    timeout(
        DISCOVERY_TIMEOUT,
        stdout.take(128 * 1024 + 1).read_to_end(&mut bytes),
    )
    .await
    .context("Codex execution contract probe timed out")??;
    owned.finish().await?;
    if bytes.len() > 128 * 1024 {
        bail!("Codex execution contract exceeds limit");
    }
    let help = std::str::from_utf8(&bytes)?;
    for flag in [
        "--ignore-user-config",
        "--ignore-rules",
        "--ephemeral",
        "--output-schema",
        "--sandbox",
        "--model",
    ] {
        if !help.contains(flag) {
            bail!("Codex CLI lacks required execution isolation contract");
        }
    }
    Ok(())
}

pub(super) async fn codex_account_kind(executable: &Path, home: &Path) -> Result<String> {
    let mut command = native_command(executable, home);
    command.args(["app-server", "--listen", "stdio://"]);
    let mut owned = OwnedChild::spawn(&mut command, false)?;
    let (mut stdin, mut lines) = child_pipes(&mut owned)?;
    let deadline = Instant::now() + DISCOVERY_TIMEOUT;
    let mut total = 0;
    let result=async {
        codex_request(&mut stdin,&mut lines,1,"initialize",json!({"clientInfo":{"name":"mirket-discovery","version":"1.0"},"capabilities":{"experimentalApi":true}}),deadline,&mut total).await?;
        send_json(&mut stdin,&json!({"method":"initialized","params":{}})).await?;
        let account=codex_request(&mut stdin,&mut lines,2,"account/read",json!({"refreshToken":false}),deadline,&mut total).await?;
        if account["account"].is_null(){bail!("Codex account is not authenticated");}
        account["account"]["type"].as_str().map(str::to_owned).context("Codex account type missing")
    }.await;
    owned.finish().await?;
    result
}

async fn codex_request(
    stdin: &mut ChildStdin,
    lines: &mut JsonLines,
    id: u64,
    method: &str,
    params: Value,
    deadline: Instant,
    total: &mut usize,
) -> Result<Value> {
    send_json(stdin, &json!({"id":id,"method":method,"params":params})).await?;
    loop {
        let frame = next_json(lines, deadline, total).await?;
        if frame.get("id").and_then(Value::as_u64) != Some(id) {
            continue;
        }
        if frame.get("error").is_some() {
            bail!("Codex discovery request failed");
        }
        return frame
            .get("result")
            .cloned()
            .context("Codex discovery response omitted result");
    }
}

/// Discover the signed-in Codex app-server model menu, including all pages.
/// This read-only catalog does not grant execution or prove a native tool floor.
pub async fn discover_codex(executable: &Path, home: &Path) -> Result<Vec<Value>> {
    let mut command = native_command(executable, home);
    command.args(["app-server", "--listen", "stdio://"]);
    let mut owned =
        OwnedChild::spawn(&mut command, false).context("start Codex discovery child")?;
    let (mut stdin, mut lines) = child_pipes(&mut owned)?;
    let deadline = Instant::now() + DISCOVERY_TIMEOUT;
    let mut total = 0;
    let result = async {
        codex_request(
            &mut stdin,
            &mut lines,
            1,
            "initialize",
            json!({
                "clientInfo":{"name":"mirket-discovery","version":"1.0"},
                "capabilities":{"experimentalApi":true}
            }),
            deadline,
            &mut total,
        )
        .await?;
        send_json(&mut stdin, &json!({"method":"initialized","params":{}})).await?;
        let account = codex_request(
            &mut stdin,
            &mut lines,
            2,
            "account/read",
            json!({"refreshToken":false}),
            deadline,
            &mut total,
        )
        .await?;
        if account["account"].is_null() {
            bail!("Codex account is not authenticated");
        }
        super::codex::upstream(
            account["account"]["type"]
                .as_str()
                .context("Codex account type missing")?,
        )?;
        let mut models = Vec::new();
        let mut cursor: Option<String> = None;
        for page in 0..20u64 {
            let mut params = json!({"limit":100,"includeHidden":true});
            if let Some(value) = cursor.as_deref() {
                params["cursor"] = Value::String(value.into());
            }
            let response = codex_request(
                &mut stdin,
                &mut lines,
                page + 3,
                "model/list",
                params,
                deadline,
                &mut total,
            )
            .await?;
            let entries = response["data"]
                .as_array()
                .context("Codex model list omitted data array")?;
            models.extend(entries.iter().cloned());
            if models.len() > 2000 {
                bail!("Codex model discovery exceeded page limit");
            }
            match response.get("nextCursor").and_then(Value::as_str) {
                Some(next) if !next.is_empty() => cursor = Some(next.to_owned()),
                _ => return Ok(models),
            }
        }
        bail!("Codex model discovery exceeded pagination limit")
    }
    .await;
    owned.finish().await.context("stop Codex discovery child")?;
    let models = result?;
    if models.is_empty() {
        bail!("Codex discovery returned no models");
    }
    Ok(models)
}

fn claude_command(executable: &Path, home: &Path) -> Command {
    let mut command = native_command(executable, home);
    command.args([
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--restricted",
        "--safe-mode",
        "--tools",
        "",
        "--strict-mcp-config",
        "--mcp-config",
        EMPTY_MCP,
        "--disable-slash-commands",
        "--permission-mode",
        "dontAsk",
        "--permission-prompts",
        "none",
        "--no-session-persistence",
        "--no-chrome",
        "--setting-sources",
        "",
    ]);
    command
}

/// The model menu can be returned before a usable login is established.
/// Check the native account state without retaining its identifying fields.
pub async fn claude_authenticated(executable: &Path, home: &Path) -> Result<()> {
    let mut command = native_command(executable, home);
    command.args(["auth", "status", "--json"]);
    let mut owned =
        OwnedChild::spawn(&mut command, false).context("start Claude auth check child")?;
    drop(owned.child().stdin.take());
    let stdout = owned
        .child()
        .stdout
        .take()
        .context("native auth stdout unavailable")?;
    let mut bytes = Vec::new();
    let result = timeout(
        DISCOVERY_TIMEOUT,
        stdout.take(128 * 1024 + 1).read_to_end(&mut bytes),
    )
    .await
    .context("native auth check timed out")?
    .context("read native auth response")
    .and_then(|_| {
        if bytes.len() > 128 * 1024 {
            bail!("native auth response exceeds limit");
        }
        let status: Value =
            serde_json::from_slice(&bytes).context("native auth response is not JSON")?;
        if status["loggedIn"] == true {
            Ok(())
        } else {
            bail!("Claude CLI is not authenticated")
        }
    });
    owned
        .finish()
        .await
        .context("stop Claude auth check child")?;
    result
}

pub(super) fn child_pipes(owned: &mut OwnedChild) -> Result<(ChildStdin, JsonLines)> {
    let stdin = owned
        .child()
        .stdin
        .take()
        .context("native child stdin unavailable")?;
    let stdout = owned
        .child()
        .stdout
        .take()
        .context("native child stdout unavailable")?;
    let lines = BufReader::new(BoundedLines::with_limit(stdout, MAX_FRAME_BYTES)).lines();
    Ok((stdin, lines))
}

async fn send_json(stdin: &mut ChildStdin, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec(value).context("encode native protocol request")?;
    if bytes.len() > MAX_INPUT_BYTES {
        bail!("native protocol request exceeds input limit");
    }
    bytes.push(b'\n');
    timeout(Duration::from_secs(60), async {
        stdin
            .write_all(&bytes)
            .await
            .context("write native protocol request")?;
        stdin.flush().await.context("flush native protocol request")
    })
    .await
    .context("native protocol write timed out")?
}

pub(super) async fn next_json(
    lines: &mut JsonLines,
    deadline: Instant,
    total: &mut usize,
) -> Result<Value> {
    let line = timeout_at(deadline, lines.next_line())
        .await
        .context("native child response timed out")??
        .context("native child closed its response stream")?;
    *total = total.saturating_add(line.len());
    if *total > MAX_STREAM_BYTES {
        bail!("native child response exceeds output limit");
    }
    serde_json::from_str(&line).context("native child emitted invalid JSON")
}

async fn claude_initialize(
    stdin: &mut ChildStdin,
    lines: &mut JsonLines,
    deadline: Instant,
    total: &mut usize,
) -> Result<Value> {
    send_json(
        stdin,
        &json!({
            "type":"control_request", "request_id":"mirket_initialize",
            "request":{"subtype":"initialize"}
        }),
    )
    .await?;
    loop {
        let frame = next_json(lines, deadline, total).await?;
        if frame["type"] != "control_response" {
            continue;
        }
        let response = &frame["response"];
        if response["request_id"] != "mirket_initialize" {
            continue;
        }
        if response["subtype"] != "success" {
            bail!("Claude initialization refused by native client");
        }
        return response
            .get("response")
            .cloned()
            .context("Claude initialization omitted response");
    }
}

/// Discover the current authenticated Claude Code menu without model inference.
pub async fn discover_claude(executable: &Path, home: &Path) -> Result<Vec<Value>> {
    claude_authenticated(executable, home).await?;
    let mut command = claude_command(executable, home);
    let mut owned =
        OwnedChild::spawn(&mut command, false).context("start Claude discovery child")?;
    let (mut stdin, mut lines) = child_pipes(&mut owned)?;
    let mut total = 0;
    let result = claude_initialize(
        &mut stdin,
        &mut lines,
        Instant::now() + DISCOVERY_TIMEOUT,
        &mut total,
    )
    .await;
    owned
        .finish()
        .await
        .context("stop Claude discovery child")?;
    let response = result?;
    let models = response["models"]
        .as_array()
        .context("Claude discovery omitted model array")?;
    if models.is_empty() {
        bail!("Claude discovery returned no selectable models");
    }
    Ok(models.clone())
}

fn claude_failure(frame: &Value) -> &'static str {
    let subtype = frame["subtype"].as_str().unwrap_or_default();
    let error = frame["error"].as_str().unwrap_or_default();
    let result = frame["result"].as_str().unwrap_or_default();
    let description = format!("{subtype} {error} {result}").to_ascii_lowercase();
    if description.contains("rate_limit")
        || description.contains("rate limit")
        || description.contains("429")
    {
        "Claude target rate limit"
    } else if description.contains("overloaded") || description.contains("529") {
        "Claude target overloaded"
    } else if description.contains("unavailable")
        || description.contains("503")
        || description.contains("502")
        || description.contains("504")
    {
        "Claude target unavailable"
    } else if description.contains("auth")
        || description.contains("login")
        || description.contains("credential")
    {
        "Claude authentication unavailable"
    } else {
        "Claude action failed or was refused"
    }
}

pub(super) fn image_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

fn user_frame(prompt: &str, images: &[PathBuf]) -> Result<Value> {
    if prompt.trim().is_empty() {
        bail!("Claude prompt is empty");
    }
    if images.len() > MAX_IMAGES {
        bail!("too many Claude image inputs");
    }
    let mut content = vec![json!({"type":"text","text":prompt})];
    for path in images {
        let bytes = crate::util::read_bounded(path, MAX_IMAGE_BYTES as u64)
            .context("read bounded Claude image input")?;
        let media_type = image_media_type(&bytes).context("unsupported Claude image type")?;
        content.push(json!({"type":"image","source":{"type":"base64","media_type":media_type,"data":crate::router::execution::base64(&bytes)}}));
    }
    Ok(
        json!({"type":"user","message":{"role":"user","content":content},"parent_tool_use_id":null,"session_id":""}),
    )
}

fn validate_claude_selection(
    initialization: &Value,
    model: &str,
    effort: Option<&str>,
) -> Result<()> {
    let available = initialization["models"]
        .as_array()
        .context("Claude action omitted model menu")?;
    let selected = available
        .iter()
        .find(|entry| entry["resolvedModel"] == model)
        .context("selected Claude model unavailable by exact identity")?;
    if let Some(level) = effort
        && (selected["supportsEffort"] != true
            || !selected["supportedEffortLevels"]
                .as_array()
                .is_some_and(|levels| levels.iter().any(|entry| entry == level)))
    {
        bail!("required effort unavailable for selected Claude model");
    }
    Ok(())
}

/// Run one Claude action turn with no native tool capability and strict JSON output.
pub async fn claude_complete(
    executable: &Path,
    home: &Path,
    model: &str,
    effort: Option<&str>,
    max_tokens: u64,
    prompt: &str,
    images: &[PathBuf],
) -> Result<Value> {
    if model.trim().is_empty() || max_tokens == 0 {
        bail!("Claude model or output limit is missing");
    }
    let user = user_frame(prompt, images)?;
    let mut command = claude_command(executable, home);
    let settings = json!({
        "availableModels":[model],
        "availableModelsMatch":"exact",
        "enforceAvailableModels":true,
        "switchModelsOnFlag":false,
        "fallbackModel":[],
        "hooks":{}
    });
    command
        .arg("--model")
        .arg(model)
        .arg("--json-schema")
        .arg(crate::router::execution::action_schema().to_string())
        .arg("--settings")
        .arg(settings.to_string())
        .env("CLAUDE_CODE_MAX_OUTPUT_TOKENS", max_tokens.to_string());
    if let Some(level) = effort {
        command.arg("--effort").arg(level);
    }
    let mut owned = OwnedChild::spawn(&mut command, false).context("start Claude action child")?;
    let (mut stdin, mut lines) = child_pipes(&mut owned)?;
    let deadline = Instant::now() + COMPLETION_TIMEOUT;
    let mut total = 0;
    let result = async {
        let initialization =
            claude_initialize(&mut stdin, &mut lines, deadline, &mut total).await?;
        validate_claude_selection(&initialization, model, effort)?;
        send_json(&mut stdin, &user).await?;
        let mut seen_model = false;
        let mut seen_init = false;
        loop {
            let frame = next_json(&mut lines, deadline, &mut total).await?;
            match frame["type"].as_str() {
                Some("error") => bail!(claude_failure(&frame)),
                Some("system") if frame["subtype"] == "init" => {
                    let tools = frame["tools"]
                        .as_array()
                        .context("Claude init omitted tool list")?;
                    if tools.len() != 1 || tools[0] != "StructuredOutput" {
                        bail!("Claude action exposed unexpected native tools");
                    }
                    if !frame["mcp_servers"].as_array().is_some_and(Vec::is_empty) {
                        bail!("Claude action exposed unexpected MCP servers");
                    }
                    seen_init = true;
                }
                Some("assistant") => {
                    let actual = frame["message"]["model"]
                        .as_str()
                        .context("Claude assistant omitted model identity")?;
                    if actual != model {
                        bail!("Claude changed from selected model");
                    }
                    seen_model = true;
                }
                Some("result") => {
                    if frame["is_error"] == true || frame["subtype"] != "success" {
                        bail!(claude_failure(&frame));
                    }
                    if !seen_init {
                        bail!("Claude action returned without tool-boundary confirmation");
                    }
                    if !seen_model {
                        bail!("Claude action returned without verifiable model identity");
                    }
                    if let Some(output) = frame.get("structured_output") {
                        if !output.is_object() {
                            bail!("Claude structured output is not an object");
                        }
                        return Ok(output.clone());
                    }
                    bail!("Claude action omitted structured output");
                }
                _ => {}
            }
        }
    }
    .await;
    owned.finish().await.context("stop Claude action child")?;
    result
}

/// Codex authenticates its request; the owned transport removes all native tool
/// surfaces and validates the complete response before exposing it to the child.
#[allow(clippy::too_many_arguments)]
pub async fn codex_complete(
    executable: &Path,
    home: &Path,
    model: &str,
    effort: Option<&str>,
    max: u64,
    prompt: &str,
    images: &[PathBuf],
) -> Result<Value> {
    super::codex::complete(executable, home, model, effort, max, prompt, images).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_inputs_are_regular_bounded_files() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let path = dir.path().join("image.png");
        std::fs::write(&path, b"\x89PNG\r\n\x1a\n").unwrap();
        assert!(user_frame("inspect", std::slice::from_ref(&path)).is_ok());
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(MAX_IMAGE_BYTES as u64 + 1)
            .unwrap();
        assert!(user_frame("inspect", std::slice::from_ref(&path)).is_err());
        assert!(user_frame("inspect", &[dir.path().to_path_buf()]).is_err());
        #[cfg(unix)]
        {
            std::fs::write(&path, b"\x89PNG\r\n\x1a\n").unwrap();
            let link = dir.path().join("link.png");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(user_frame("inspect", &[link]).is_err());
        }
    }

    #[test]
    fn action_requires_current_exact_identity_and_effort_before_inference() {
        let current = json!({"models":[{
            "value":"native-alias", "resolvedModel":"dynamic-resolved-model",
            "supportsEffort":true, "supportedEffortLevels":["medium","high"]
        }]});
        assert!(
            validate_claude_selection(&current, "dynamic-resolved-model", Some("high")).is_ok()
        );
        assert!(validate_claude_selection(&current, "native-alias", Some("high")).is_err());
        let mut changed = current.clone();
        changed["models"][0]["supportsEffort"] = json!(false);
        assert!(
            validate_claude_selection(&changed, "dynamic-resolved-model", Some("high")).is_err()
        );
        changed = current.clone();
        changed["models"][0]["supportedEffortLevels"] = json!(["medium"]);
        assert!(
            validate_claude_selection(&changed, "dynamic-resolved-model", Some("high")).is_err()
        );
        changed = current;
        changed["models"][0]["resolvedModel"] = json!("new-dynamic-model");
        assert!(
            validate_claude_selection(&changed, "dynamic-resolved-model", Some("high")).is_err()
        );
    }
}
