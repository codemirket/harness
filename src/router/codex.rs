//! Native Codex authentication with an owned, text-only Responses transport.
//!
//! The child never receives an upstream tool call: requests are reconstructed
//! from the router's input, and the entire bounded SSE response is validated
//! before any response bytes are released. No credentials are copied to disk.
use super::{execution, native, responses::validate_item};
use crate::{paths::Paths, process::OwnedChild, util};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    time::{Instant, timeout_at},
};

const DEADLINE: Duration = Duration::from_secs(600);
const MAX_BODY: usize = 16 * 1024 * 1024;
const MAX_HEADERS: usize = 64 * 1024;

pub(super) fn upstream(account: &str) -> Result<&'static str> {
    match account {
        "chatgpt" => Ok("https://chatgpt.com/backend-api/codex/responses"),
        "apiKey" => Ok("https://api.openai.com/v1/responses"),
        _ => bail!("unsupported Codex authentication kind"),
    }
}

fn request_body(
    model: &str,
    effort: Option<&str>,
    max: u64,
    prompt: &str,
    images: &[PathBuf],
) -> Result<Value> {
    ensure!(!prompt.trim().is_empty(), "Codex prompt is empty");
    ensure!(images.len() <= 4, "too many Codex images");
    let mut content = vec![json!({"type":"input_text","text":prompt})];
    for path in images {
        let bytes = util::read_bounded(path, 4 * 1024 * 1024)?;
        let mime = native::image_media_type(&bytes).context("unsupported Codex image type")?;
        content.push(json!({"type":"input_image","image_url":format!("data:{mime};base64,{}",execution::base64(&bytes))}));
    }
    let mut body = json!({
        "model":model,
        "instructions":"Return exactly one Mirket action using the supplied JSON schema, in your final answer. Commentary and partial answers are never dispatched. Do not issue actions in commentary or expect tool results within this response; Mirket dispatches the final action and supplies its result on the next turn. All tool dispatch belongs to Mirket.",
        "input":[{"role":"user","content":content}],
        "tools":[],"tool_choice":"none","parallel_tool_calls":false,
        "store":false,"stream":true,"max_output_tokens":max,
        "text":{"format":{"type":"json_schema","name":"mirket_action","strict":true,"schema":execution::action_schema()}}
    });
    if let Some(effort) = effort {
        body["reasoning"] = json!({"effort":effort});
    }
    ensure!(
        serde_json::to_vec(&body)?.len() <= MAX_BODY,
        "Codex request exceeds limit"
    );
    Ok(body)
}

/// Fail closed on new event kinds, incomplete streams, changed model identity,
/// and any executable output. Validation happens before forwarding even a byte.
fn validate_stream(bytes: &[u8], model: &str, observation: &mut Value) -> Result<(Value, Vec<u8>)> {
    let stream = std::str::from_utf8(bytes).context("Codex stream is not UTF-8")?;
    let mut completed = None;
    let mut normalized = Vec::new();
    let stream = stream.replace("\r\n", "\n").replace('\r', "\n");
    ensure!(stream.ends_with("\n\n"), "Codex SSE stream is truncated");
    for event in stream.split("\n\n") {
        let mut data = Vec::new();
        let mut event_name = None;
        for line in event.lines() {
            if line.starts_with(':') {
                continue;
            }
            let (field, value) = line.split_once(':').unwrap_or((line, ""));
            let value = value.strip_prefix(' ').unwrap_or(value);
            match field {
                "data" => data.push(value),
                "event" => event_name = Some(value),
                _ => {} // Re-encoding discards non-data fields consistently.
            }
        }
        let payload = data.join("\n");
        if payload.is_empty() {
            continue;
        }
        if payload == "[DONE]" {
            ensure!(completed.is_some(), "Codex stream ended before completion");
            continue;
        }
        let frame: Value = serde_json::from_str(&payload).context("invalid Codex SSE frame")?;
        if let Some(name) = event_name {
            ensure!(frame["type"] == name, "Codex SSE event type mismatch");
        }
        ensure!(
            completed.is_none(),
            "Codex output continued after completion"
        );
        match frame["type"].as_str() {
            Some("response.created" | "response.in_progress") => {
                if let Some(identity) = frame["response"]["model"].as_str() {
                    ensure!(identity == model, "Codex response model identity changed");
                }
                if let Some(items) = frame["response"]["output"].as_array() {
                    for item in items {
                        validate_item(item)?;
                    }
                }
            }
            Some("response.output_item.added" | "response.output_item.done") => {
                validate_item(&frame["item"])?
            }
            Some("response.content_part.added" | "response.content_part.done") => {
                ensure!(
                    frame["part"]["type"] == "output_text",
                    "non-text Codex content rejected"
                );
            }
            Some(
                "response.output_text.delta"
                | "response.output_text.done"
                | "response.reasoning_summary_part.added"
                | "response.reasoning_summary_part.done"
                | "response.reasoning_summary_text.delta"
                | "response.reasoning_summary_text.done"
                | "response.reasoning_text.delta"
                | "response.reasoning_text.done"
                | "response.metadata",
            ) => {}
            Some("response.completed") => {
                let response = &frame["response"];
                ensure!(
                    response["model"] == model,
                    "Codex completion model identity changed or missing"
                );
                ensure!(
                    response["status"] == "completed",
                    "Codex response incomplete"
                );
                observation["response_sha256"] =
                    json!(util::sha256(&serde_json::to_vec(response)?));
                observation["usage"] = super::execution::usage_receipt(response);
                completed = Some(super::responses::action(&response["output"])?);
            }
            _ => bail!("Codex native tool, error or unknown event rejected before dispatch"),
        }
        normalized.extend_from_slice(
            format!(
                "event: {}\ndata: {}\n\n",
                frame["type"].as_str().context("Codex event type missing")?,
                serde_json::to_string(&frame)?
            )
            .as_bytes(),
        );
    }
    let action = completed.context("Codex stream omitted successful completion")?;
    Ok((action, normalized))
}

async fn read_request(
    stream: TcpStream,
) -> Result<(TcpStream, String, BTreeMap<String, String>, Vec<u8>)> {
    let mut reader = BufReader::new(stream);
    let mut first = String::new();
    ensure!(
        (&mut reader)
            .take((MAX_HEADERS + 1) as u64)
            .read_line(&mut first)
            .await?
            <= MAX_HEADERS,
        "Codex request line too large"
    );
    let mut headers = BTreeMap::new();
    let mut total = first.len();
    loop {
        let mut line = String::new();
        // read_until is capped so a missing newline cannot allocate indefinitely.
        let count = (&mut reader)
            .take((MAX_HEADERS - total + 1) as u64)
            .read_line(&mut line)
            .await?;
        ensure!(count > 0, "Codex request header truncated");
        total += count;
        ensure!(total <= MAX_HEADERS, "Codex headers exceed limit");
        if line == "\r\n" {
            break;
        }
        let (key, value) = line
            .split_once(':')
            .context("invalid Codex request header")?;
        ensure!(
            headers
                .insert(key.to_ascii_lowercase(), value.trim().to_owned())
                .is_none(),
            "duplicate Codex request header"
        );
    }
    ensure!(
        !headers.contains_key("transfer-encoding"),
        "chunked Codex request unsupported"
    );
    let length = headers
        .get("content-length")
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(0);
    ensure!(length <= MAX_BODY, "Codex request exceeds limit");
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;
    Ok((reader.into_inner(), first, headers, body))
}

async fn serve(
    listener: TcpListener,
    capability: &str,
    url: &str,
    model: &str,
    body: Value,
    observation: &mut Value,
) -> Result<Value> {
    let mut authenticated_requests = 0;
    loop {
        let (stream, _) = listener.accept().await?;
        let Ok(Ok((mut stream, first, headers, received))) =
            tokio::time::timeout(Duration::from_secs(5), read_request(stream)).await
        else {
            continue;
        };
        if headers.get("x-mirket-capability").map(String::as_str) != Some(capability) {
            continue;
        }
        authenticated_requests += 1;
        ensure!(
            authenticated_requests <= 8,
            "Codex transport request limit exceeded"
        );
        if first.starts_with("GET /sampler/models?") {
            stream
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await?;
            continue;
        }
        if first != "POST /sampler/responses HTTP/1.1\r\n" {
            let _ = stream
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await;
            continue;
        }
        let native_body: Value =
            serde_json::from_slice(&received).context("invalid native Codex request")?;
        ensure!(
            native_body["model"] == model,
            "native Codex selected another model"
        );
        let auth = headers
            .get("authorization")
            .context("native Codex authorization missing")?;
        ensure!(
            auth.starts_with("Bearer "),
            "invalid native Codex authorization"
        );
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(DEADLINE)
            .build()?;
        let mut request = client.post(url).header("authorization", auth).json(&body);
        for key in [
            "chatgpt-account-id",
            "originator",
            "user-agent",
            "openai-beta",
        ] {
            if let Some(value) = headers.get(key) {
                request = request.header(key, value);
            }
        }
        let mut response = request
            .send()
            .await
            .context("Codex inference transport failed")?;
        ensure!(
            response.status().is_success(),
            "Codex inference target returned status {}",
            response.status()
        );
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            ensure!(
                bytes.len().saturating_add(chunk.len()) <= MAX_BODY,
                "Codex inference response exceeds limit"
            );
            bytes.extend_from_slice(&chunk);
        }
        let (action, bytes) = validate_stream(&bytes, model, observation)?;
        stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",bytes.len()).as_bytes()).await?;
        stream.write_all(&bytes).await?;
        stream.shutdown().await?;
        return Ok(action);
    }
}

enum ChildEvent {
    Progress,
    Message(String),
    Completed,
}

fn child_event(frame: &Value) -> Result<ChildEvent> {
    match frame["type"].as_str() {
        Some("thread.started" | "turn.started") => Ok(ChildEvent::Progress),
        Some("item.completed") if frame["item"]["type"] == "agent_message" => {
            let text = frame["item"]["text"]
                .as_str()
                .context("Codex action text missing")?
                .to_owned();
            Ok(ChildEvent::Message(text))
        }
        Some("item.started" | "item.updated" | "item.completed")
            if frame["item"]["type"] == "reasoning" =>
        {
            Ok(ChildEvent::Progress)
        }
        // Exec's ErrorItem is explicitly a non-fatal diagnostic. In contrast,
        // top-level error and turn.failed remain fatal. Disabled Code Mode
        // emits this item even though the isolated text turn is still valid.
        Some("item.completed") if frame["item"]["type"] == "error" => Ok(ChildEvent::Progress),
        Some("turn.completed") => Ok(ChildEvent::Completed),
        _ => bail!("Codex child attempted native execution or failed"),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn complete(
    executable: &Path,
    home: &Path,
    model: &str,
    effort: Option<&str>,
    max: u64,
    prompt: &str,
    images: &[PathBuf],
    observation: &mut Value,
) -> Result<Value> {
    let url = upstream(&native::codex_account_kind(executable, home).await?)?;
    let body = request_body(model, effort, max, prompt, images)?;
    let scratch = tempfile::Builder::new()
        .prefix("mirket-codex-sampler-")
        .tempdir_in(Paths::new(home.to_owned())?.scratch()?)?;
    let capability_file = tempfile::Builder::new()
        .prefix("cap-")
        .rand_bytes(32)
        .tempfile_in(scratch.path())?;
    let capability = capability_file
        .path()
        .file_name()
        .context("capability missing")?
        .to_string_lossy()
        .into_owned();
    let schema = scratch.path().join("action.json");
    let instructions = scratch.path().join("instructions.txt");
    std::fs::write(&schema, serde_json::to_vec(&execution::action_schema())?)?;
    std::fs::write(&instructions, "Return the requested schema-bound action.")?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let endpoint = format!("http://{}/sampler", listener.local_addr()?);
    let mut command = native::native_command(executable, home);
    command.env("MIRKET_CODEX_TRANSPORT_CAPABILITY", &capability);
    command
        .current_dir(scratch.path())
        .args([
            "exec",
            "--ignore-user-config",
            "--ignore-rules",
            "--skip-git-repo-check",
            "--ephemeral",
            "--json",
            "--sandbox",
            "read-only",
            "--model",
            model,
            "--output-schema",
        ])
        .arg(&schema);
    let provider = format!(
        "{{name=\"Mirket sampler\",base_url={},wire_api=\"responses\",requires_openai_auth=true,request_max_retries=0,stream_max_retries=0,supports_websockets=false,env_http_headers={{x-mirket-capability=\"MIRKET_CODEX_TRANSPORT_CAPABILITY\"}}}}",
        serde_json::to_string(&endpoint)?
    );
    for (key, value) in [
        ("model_provider", "\"mirket_sampler\"".to_owned()),
        ("model_providers.mirket_sampler", provider),
        (
            "model_instructions_file",
            serde_json::to_string(&instructions)?,
        ),
        ("sqlite_home", serde_json::to_string(scratch.path())?),
        ("log_dir", serde_json::to_string(scratch.path())?),
        ("project_doc_max_bytes", "0".into()),
        ("web_search", "\"disabled\"".into()),
        ("tools.update_plan.enabled", "false".into()),
        (
            "tools.experimental_request_user_input.enabled",
            "false".into(),
        ),
    ] {
        command.args(["-c", &format!("{key}={value}")]);
    }
    // These controls prevent ambient startup effects. The transport, rather
    // than this version-sensitive feature list, enforces the complete tool floor.
    for feature in [
        "hooks",
        "apps",
        "plugins",
        "shell_tool",
        "unified_exec",
        "multi_agent",
        "multi_agent_v2",
        "code_mode",
        "code_mode_host",
        "image_generation",
        "enable_request_compression",
    ] {
        command.args(["--disable", feature]);
    }
    if let Some(effort) = effort {
        command.args([
            "-c",
            &format!("model_reasoning_effort={}", serde_json::to_string(effort)?),
        ]);
    }
    command.arg("Return the requested action.");
    let mut owned = OwnedChild::spawn(&mut command, false)?;
    let (stdin, mut lines) = native::child_pipes(&mut owned)?;
    drop(stdin);
    let deadline = Instant::now() + DEADLINE;
    let result = timeout_at(deadline, async {
        let child = async {
            let mut total = 0;
            let mut action = None;
            loop {
                let frame = native::next_json(&mut lines, deadline, &mut total).await?;
                match child_event(&frame)? {
                    ChildEvent::Progress => {}
                    ChildEvent::Message(text) => {
                        // Exec may omit phase metadata. The transport validates
                        // every item and requires the final message to be the
                        // sole action; compare that last message after completion.
                        action = Some(text);
                    }
                    ChildEvent::Completed => {
                        return serde_json::from_str::<Value>(
                            &action.context("Codex child omitted action")?,
                        )
                        .context("Codex child final answer is not JSON");
                    }
                }
            }
        };
        let (child_action, transport_action) = tokio::try_join!(
            child,
            serve(listener, &capability, url, model, body, observation)
        )?;
        ensure!(
            child_action == transport_action,
            "Codex child changed the validated action"
        );
        Ok(child_action)
    })
    .await
    .context("Codex completion timed out")?;
    owned.finish().await?;
    result
}

#[cfg(test)]
include!("tests/codex.rs");
