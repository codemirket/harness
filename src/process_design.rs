//! Finite supervisor for the private design Node adapter.
//!
//! The adapter is deliberately one child per operation.  Rust owns the job
//! directory, source snapshot, deadline and staged output; Node only receives
//! two paths inside that directory and a finite operation.  There is no shell,
//! PATH lookup, project configuration, plugin loading or host-runtime
//! fallback.  This module is transport and identity validation only; the
//! worker owns admission and publication.

use crate::design::{
    DesignOperation, DesignOutcome, DesignPayload, DesignRequest, RasterFit, RasterFormat,
    TokenFormat,
};
use crate::process::{
    AdapterError, OwnedChild, cancelled, drain_stderr, read_frame, validate_job_dir,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Component, Path};
use std::process::Stdio;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

const PROTOCOL_VERSION: u32 = 1;
const MAX_FRAME_BYTES: usize = 65_536;
const MAX_INPUT_BYTES: usize = 1_048_576;
const MAX_RASTER_INPUT_BYTES: usize = crate::design::MAX_RASTER_INPUT_BYTES;
const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const MIN_TIMEOUT_MS: u64 = 100;
const MAX_ADAPTER_TIMEOUT_MS: u64 = 120_000;
const MAX_STDERR_BYTES: usize = 16 * 1024;

fn unconfirmed(message: impl Into<String>) -> AdapterError {
    AdapterError {
        message: message.into(),
        cleanup_confirmed: false,
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ready {
    #[serde(rename = "type")]
    kind: String,
    protocol_version: u32,
    release_id: String,
    operations: Vec<String>,
    max_frame_bytes: usize,
    max_input_bytes: usize,
    max_output_bytes: usize,
    min_deadline_ms: u64,
    max_deadline_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum Completion {
    #[serde(rename = "result")]
    Result {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        operation: String,
        status: String,
        output_path: String,
        input_bytes: usize,
        output_bytes: usize,
        input_sha256: String,
        output_sha256: String,
        format: Option<String>,
        token_count: Option<usize>,
        max_depth: Option<usize>,
        element_count: Option<usize>,
        width: Option<u32>,
        height: Option<u32>,
    },
    #[serde(rename = "error")]
    Error {
        protocol_version: u32,
        release_id: String,
        job_id: Option<String>,
        code: String,
        message: String,
        output_may_exist: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WireOperation {
    TokensCompile(TokenFormat),
    SvgOptimize,
    RasterTransform {
        source_format: RasterFormat,
        output_format: RasterFormat,
        width: u32,
        height: u32,
        fit: RasterFit,
    },
}

impl WireOperation {
    fn name(self) -> &'static str {
        match self {
            Self::TokensCompile(_) => "tokens.compile",
            Self::SvgOptimize => "svg.optimize",
            Self::RasterTransform { .. } => "raster.transform",
        }
    }

    fn format(self) -> Option<&'static str> {
        match self {
            Self::TokensCompile(TokenFormat::Css) => Some("css"),
            Self::TokensCompile(TokenFormat::Json) => Some("json"),
            Self::SvgOptimize => None,
            Self::RasterTransform { .. } => None,
        }
    }
}

fn wire_operation(operation: &DesignOperation) -> Result<WireOperation, String> {
    match operation {
        DesignOperation::TokensCompile {
            format: TokenFormat::Css,
        } => Ok(WireOperation::TokensCompile(TokenFormat::Css)),
        DesignOperation::TokensCompile {
            format: TokenFormat::Json,
        } => Ok(WireOperation::TokensCompile(TokenFormat::Json)),
        DesignOperation::SvgOptimize => Ok(WireOperation::SvgOptimize),
        DesignOperation::RasterTransform {
            source_format,
            output_format,
            width,
            height,
            fit,
        } => Ok(WireOperation::RasterTransform {
            source_format: *source_format,
            output_format: *output_format,
            width: *width,
            height: *height,
            fit: *fit,
        }),
        DesignOperation::SvgRenderPng { .. } => {
            Err("SVG PNG rendering is owned by the Rust renderer".into())
        }
        DesignOperation::OpenPencil { .. } => {
            Err("OpenPencil shape-file work is owned by its exact private adapter".into())
        }
    }
}

fn bounded_path(path: &Path) -> bool {
    path.is_absolute()
        && path
            .to_str()
            .is_some_and(|value| value.len() <= 4096 && !value.contains('\0'))
        && path
            .components()
            .all(|part| !matches!(part, Component::ParentDir | Component::CurDir))
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn sha256_revision(bytes: &[u8]) -> String {
    format!("sha256:{}", hash(bytes))
}

fn read_bounded(path: &Path, limit: usize) -> Result<(Vec<u8>, fs::Metadata), String> {
    let before = fs::symlink_metadata(path).map_err(|_| "design file is unavailable")?;
    if !before.is_file()
        || before.file_type().is_symlink()
        || before.len() > limit as u64
        || fs::canonicalize(path).map_err(|_| "design file cannot be resolved")? != path
    {
        return Err("design file must be a bounded canonical regular file".into());
    }
    let mut file = fs::File::open(path).map_err(|_| "design file cannot be opened")?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "design file cannot be read")?;
    let after = fs::symlink_metadata(path).map_err(|_| "design file disappeared")?;
    if bytes.len() as u64 != before.len()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
        || after.file_type().is_symlink()
    {
        return Err("design file changed during read".into());
    }
    if bytes.len() > limit {
        return Err("design file exceeds its byte limit".into());
    }
    Ok((bytes, after))
}

fn path_text(path: &Path) -> Result<&str, String> {
    if !bounded_path(path) {
        return Err("design path must be an absolute bounded path without traversal".into());
    }
    path.to_str()
        .ok_or_else(|| "design path must be UTF-8".into())
}

fn validate_request(request: &DesignRequest, operation: WireOperation) -> Result<(), String> {
    request.validate()?;
    if request.job_id.len() > 120
        || !request
            .job_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
        || !bounded_path(&request.job_root)
        || !bounded_path(&request.input_path)
        || !bounded_path(&request.output_path)
        || request.input_path == request.output_path
        || request.input_path.parent() != Some(request.job_root.as_path())
        || request.output_path.parent() != Some(request.job_root.as_path())
        || !(MIN_TIMEOUT_MS..=crate::design::MAX_TIMEOUT_MS).contains(&request.timeout_ms)
        || request.max_output_bytes == 0
        || request.max_output_bytes > MAX_OUTPUT_BYTES
    {
        return Err("design request does not match the finite contract".into());
    }
    let expected_input = match operation {
        WireOperation::TokensCompile(_) => "json",
        WireOperation::SvgOptimize => "svg",
        WireOperation::RasterTransform { source_format, .. } => source_format.extension(),
    };
    if request
        .input_path
        .extension()
        .and_then(|value| value.to_str())
        != Some(expected_input)
    {
        return Err("design input extension does not match the operation".into());
    }
    let expected_output = match operation {
        WireOperation::TokensCompile(TokenFormat::Css) => "css",
        WireOperation::TokensCompile(TokenFormat::Json) => "json",
        WireOperation::SvgOptimize => "svg",
        WireOperation::RasterTransform { output_format, .. } => output_format.extension(),
    };
    if request
        .output_path
        .extension()
        .and_then(|value| value.to_str())
        != Some(expected_output)
    {
        return Err("design output extension does not match the operation".into());
    }
    if request.source_revision.len() != 71
        || !request.source_revision.starts_with("sha256:")
        || !request.source_revision[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("design source revision is not a SHA-256 identity".into());
    }
    Ok(())
}

fn validate_ready(frame: &[u8], release_id: &str, raster_enabled: bool) -> Result<(), String> {
    if frame.len() > MAX_FRAME_BYTES {
        return Err("design readiness frame exceeds its byte limit".into());
    }
    let ready: Ready =
        serde_json::from_slice(frame).map_err(|_| "design readiness frame is invalid")?;
    if ready.kind != "ready"
        || ready.protocol_version != PROTOCOL_VERSION
        || ready.release_id != release_id
        || ready.operations
            != if raster_enabled {
                vec!["tokens.compile", "svg.optimize", "raster.transform"]
            } else {
                vec!["tokens.compile", "svg.optimize"]
            }
        || ready.max_frame_bytes != MAX_FRAME_BYTES
        || ready.max_input_bytes
            != if raster_enabled {
                MAX_RASTER_INPUT_BYTES
            } else {
                MAX_INPUT_BYTES
            }
        || ready.max_output_bytes != MAX_OUTPUT_BYTES
        || ready.min_deadline_ms != MIN_TIMEOUT_MS
        || ready.max_deadline_ms != MAX_ADAPTER_TIMEOUT_MS
    {
        return Err("design readiness does not match the selected finite release".into());
    }
    Ok(())
}

fn validate_completion(
    frame: &[u8],
    request: &DesignRequest,
    operation: WireOperation,
    input: &[u8],
    output_path: &Path,
) -> Result<(usize, String), String> {
    if frame.len() > MAX_FRAME_BYTES {
        return Err("design completion frame exceeds its byte limit".into());
    }
    let completion: Completion =
        serde_json::from_slice(frame).map_err(|_| "design completion frame is invalid")?;
    match completion {
        Completion::Result {
            protocol_version,
            release_id,
            job_id,
            operation: actual_operation,
            status,
            output_path: actual_path,
            input_bytes,
            output_bytes,
            input_sha256,
            output_sha256,
            format,
            token_count,
            max_depth,
            element_count,
            width,
            height,
        } => {
            if protocol_version != PROTOCOL_VERSION
                || release_id != request.release_id
                || job_id != request.job_id
                || actual_operation != operation.name()
                || status != "staged"
                || actual_path != path_text(output_path)?
                || input_bytes != input.len()
                || input_sha256 != hash(input)
                || output_bytes == 0
                || output_bytes > request.max_output_bytes
                || output_sha256.len() != 64
                || !output_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err("design result identity, stage or bounds differ from request".into());
            }
            match operation {
                WireOperation::TokensCompile(expected) => {
                    let expected_format = match expected {
                        TokenFormat::Css => "css",
                        TokenFormat::Json => "json",
                    };
                    if format.as_deref() != Some(expected_format)
                        || token_count.is_none()
                        || max_depth.is_none()
                        || element_count.is_some()
                        || width.is_some()
                        || height.is_some()
                    {
                        return Err("design token result metadata differs from request".into());
                    }
                }
                WireOperation::SvgOptimize => {
                    if format.as_deref() != Some("svg")
                        || element_count.is_none()
                        || token_count.is_some()
                        || max_depth.is_some()
                        || width.is_some()
                        || height.is_some()
                    {
                        return Err("design SVG result metadata differs from request".into());
                    }
                }
                WireOperation::RasterTransform {
                    output_format,
                    width: requested_width,
                    height: requested_height,
                    fit,
                    ..
                } => {
                    let dimensions_valid = match (width, height, fit) {
                        (Some(w), Some(h), RasterFit::Fill) => {
                            w == requested_width && h == requested_height
                        }
                        (Some(w), Some(h), RasterFit::Inside) => {
                            w > 0 && h > 0 && w <= requested_width && h <= requested_height
                        }
                        _ => false,
                    };
                    if format.as_deref() != Some(output_format.extension())
                        || !dimensions_valid
                        || token_count.is_some()
                        || max_depth.is_some()
                        || element_count.is_some()
                    {
                        return Err("design raster result metadata differs from request".into());
                    }
                }
            }
            Ok((output_bytes, output_sha256))
        }
        Completion::Error {
            protocol_version,
            release_id,
            job_id,
            code,
            message,
            output_may_exist,
        } => {
            if protocol_version != PROTOCOL_VERSION
                || release_id != request.release_id
                || job_id.as_deref() != Some(request.job_id.as_str())
                || code.is_empty()
                || code.len() > 128
                || !code
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
                || message.len() > 2048
            {
                return Err("design error frame identity or bounds differ from request".into());
            }
            Err(format!(
                "design adapter failed ({code}); private stage may exist: {output_may_exist}"
            ))
        }
    }
}

/// Run exactly one finite design operation in the selected private Node
/// payload.  The caller must keep the worker/admission owner alive until this
/// future returns and must reconcile the private stage when cleanup is not
/// confirmed.
pub async fn run_design_adapter(
    payload: &DesignPayload,
    request: &DesignRequest,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<DesignOutcome, AdapterError> {
    if !cfg!(unix) {
        return Err("design adapter process ownership is not qualified on this platform".into());
    }
    if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
        return Err("design operation cancelled or expired before launch".into());
    }
    let operation = wire_operation(&request.operation).map_err(AdapterError::from)?;
    if matches!(operation, WireOperation::RasterTransform { .. }) && payload.raster_module.is_none()
    {
        return Err("private Sharp raster runtime is not configured".into());
    }
    validate_request(request, operation).map_err(AdapterError::from)?;
    validate_job_dir(&request.job_root).map_err(AdapterError::from)?;
    let input_limit = request.operation.max_input_bytes();
    let (input, input_metadata) =
        read_bounded(&request.input_path, input_limit).map_err(AdapterError::from)?;
    if sha256_revision(&input) != request.source_revision {
        return Err("design source snapshot revision differs".into());
    }
    match fs::symlink_metadata(&request.output_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        _ => return Err("design output stage must be absent before launch".into()),
    }
    let node_metadata = fs::symlink_metadata(&payload.node)
        .map_err(|_| AdapterError::from("design Node payload is unavailable"))?;
    let script_metadata = fs::symlink_metadata(&payload.adapter)
        .map_err(|_| AdapterError::from("design adapter payload is unavailable"))?;
    if !bounded_path(&payload.node)
        || !bounded_path(&payload.adapter)
        || !node_metadata.is_file()
        || node_metadata.file_type().is_symlink()
        || !script_metadata.is_file()
        || script_metadata.file_type().is_symlink()
        || fs::canonicalize(&payload.node).map_err(|_| "design Node payload cannot be resolved")?
            != payload.node
        || fs::canonicalize(&payload.adapter)
            .map_err(|_| "design adapter payload cannot be resolved")?
            != payload.adapter
    {
        return Err("design adapter requires explicit private payload files".into());
    }
    if let Some(module) = &payload.raster_module {
        let metadata = fs::symlink_metadata(module)
            .map_err(|_| AdapterError::from("private Sharp module is unavailable"))?;
        if !bounded_path(module)
            || !metadata.is_file()
            || metadata.file_type().is_symlink()
            || fs::canonicalize(module).map_err(|_| "private Sharp module cannot be resolved")?
                != *module
        {
            return Err("private Sharp module must be an explicit canonical file".into());
        }
    }
    let input_path = path_text(&request.input_path).map_err(AdapterError::from)?;
    let output_path = path_text(&request.output_path).map_err(AdapterError::from)?;
    let mut frame = serde_json::json!({
        "type": "run",
        "protocol_version": PROTOCOL_VERSION,
        "release_id": request.release_id,
        "job_id": request.job_id,
        "input_path": input_path,
        "output_path": output_path,
        "operation": operation.name(),
        "deadline_ms": request.timeout_ms,
    });
    if let Some(format) = operation.format() {
        frame["format"] = serde_json::Value::String(format.into());
    }
    if let WireOperation::RasterTransform {
        source_format,
        output_format,
        width,
        height,
        fit,
    } = operation
    {
        frame["source_format"] = serde_json::Value::String(source_format.extension().into());
        frame["output_format"] = serde_json::Value::String(output_format.extension().into());
        frame["width"] = serde_json::Value::from(width);
        frame["height"] = serde_json::Value::from(height);
        frame["fit"] = serde_json::Value::String(
            match fit {
                RasterFit::Fill => "fill",
                RasterFit::Inside => "inside",
            }
            .into(),
        );
    }
    let mut frame = serde_json::to_vec(&frame)
        .map_err(|_| AdapterError::from("design request could not be encoded"))?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME_BYTES {
        return Err("design request exceeds its frame limit".into());
    }

    let mut command = Command::new(&payload.node);
    command
        .arg(&payload.adapter)
        .arg("--release-id")
        .arg(&request.release_id);
    if let Some(module) = &payload.raster_module {
        command.arg("--raster-module").arg(module);
    }
    command
        .env_clear()
        .env("HOME", &request.job_root)
        .env("TMPDIR", &request.job_root)
        .env("TMP", &request.job_root)
        .env("TEMP", &request.job_root)
        .current_dir(&request.job_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    let child = command
        .spawn()
        .map_err(|error| AdapterError::from(format!("design adapter launch failed: {error}")))?;
    let mut owned = OwnedChild::new(child).map_err(unconfirmed)?;
    let (Some(mut stdin), Some(stdout), Some(stderr)) = (
        owned.child.stdin.take(),
        owned.child.stdout.take(),
        owned.child.stderr.take(),
    ) else {
        return match owned.terminate_and_reap().await {
            Ok(()) => Err("design adapter pipes are unavailable".into()),
            Err(error) => Err(unconfirmed(format!(
                "design adapter pipes unavailable; cleanup unconfirmed: {error}"
            ))),
        };
    };

    let execution = async {
        let protocol = async {
            let mut stdout = BufReader::new(stdout);
            let ready = read_frame(&mut stdout).await?;
            validate_ready(&ready, &request.release_id, payload.raster_module.is_some())?;
            stdin
                .write_all(&frame)
                .await
                .map_err(|_| "design request write failed")?;
            stdin
                .shutdown()
                .await
                .map_err(|_| "design request close failed")?;
            drop(stdin);
            let response = read_frame(&mut stdout).await?;
            let (output_bytes, output_sha256) =
                validate_completion(&response, request, operation, &input, &request.output_path)?;
            let mut extra = [0u8; 1];
            if stdout
                .read(&mut extra)
                .await
                .map_err(|_| "design adapter stdout close failed")?
                != 0
            {
                return Err("design adapter emitted unexpected trailing stdout".into());
            }
            Ok((output_bytes, output_sha256))
        };
        let ((output_bytes, output_sha256), stderr_bytes) =
            tokio::try_join!(protocol, drain_stderr(stderr))?;
        let status = owned.wait().await?;
        if !status.success() {
            return Err("design adapter returned a result but exited unsuccessfully".into());
        }
        if stderr_bytes > MAX_STDERR_BYTES {
            return Err("design adapter stderr exceeds its byte limit".into());
        }
        let (output, _) = read_bounded(&request.output_path, request.max_output_bytes)?;
        if output.len() != output_bytes || hash(&output) != output_sha256 {
            return Err("design output stage differs from the response identity".into());
        }
        let (source_after, source_after_metadata) = read_bounded(&request.input_path, input_limit)?;
        if source_after_metadata.len() != input_metadata.len()
            || source_after != input
            || sha256_revision(&source_after) != request.source_revision
        {
            return Err("design source changed during operation".into());
        }
        Ok(DesignOutcome {
            output_revision: sha256_revision(&output),
            input_bytes: input.len(),
            output_bytes,
            stderr_bytes,
            cleanup_confirmed: true,
        })
    };
    let result = tokio::select! {
        biased;
        _ = cancelled(cancel) => Err("design operation cancelled".to_owned()),
        value = tokio::time::timeout_at(deadline.min(tokio::time::Instant::now() + Duration::from_millis(request.timeout_ms)), execution) => match value {
            Ok(value) => value,
            Err(_) => Err("design operation deadline exceeded".to_owned()),
        },
    };
    if result.is_err()
        && !owned.is_reaped()
        && let Err(error) = owned.terminate_and_reap().await
    {
        return Err(unconfirmed(format!(
            "design adapter cleanup unconfirmed: {error}; retain admission for reconciliation"
        )));
    }
    result.map_err(AdapterError::from)
}
