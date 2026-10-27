//! Exact private OpenPencil editable-file leaf under the existing design worker.
//! The worker owns admission and publication; this module only verifies one
//! bounded adapter exchange and its staged bytes. It is not a sandbox.

use crate::design::{DesignOperation, DesignOutcome, DesignRequest};
use crate::openpencil::{ENGINE_VERSION, MODE, OpenPencilOperation, OpenPencilPayload};
use crate::process::{
    AdapterError, OwnedChild, cancelled, drain_stderr, read_frame, validate_job_dir,
};
use crate::scope::ProjectScope;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

const MAX_FRAME_BYTES: usize = 65_536;

#[derive(Deserialize)]
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
    engine_version: String,
    mode: String,
}

#[derive(Deserialize)]
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
        format: String,
        node_count: usize,
        engine_version: String,
        mode: String,
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

fn unconfirmed(message: impl Into<String>) -> AdapterError {
    AdapterError {
        message: message.into(),
        cleanup_confirmed: false,
    }
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn ready_matches(frame: &[u8], release: &str) -> Result<(), String> {
    let value: Ready =
        serde_json::from_slice(frame).map_err(|_| "OpenPencil readiness frame is invalid")?;
    if value.kind != "ready"
        || value.protocol_version != 1
        || value.release_id != release
        || value.operations
            != [
                "openpencil.create",
                "openpencil.patch",
                "openpencil.export_svg",
            ]
        || value.max_frame_bytes != MAX_FRAME_BYTES
        || value.max_input_bytes != crate::openpencil::MAX_INPUT_BYTES
        || value.max_output_bytes != crate::openpencil::MAX_OUTPUT_BYTES
        || value.min_deadline_ms != crate::openpencil::MIN_TIMEOUT_MS
        || value.max_deadline_ms != crate::openpencil::MAX_TIMEOUT_MS
        || value.engine_version != ENGINE_VERSION
        || value.mode != MODE
    {
        return Err("OpenPencil readiness differs from selected private release".into());
    }
    Ok(())
}

fn completion_matches(
    frame: &[u8],
    request: &DesignRequest,
    operation: &OpenPencilOperation,
    input: &[u8],
) -> Result<(usize, String), String> {
    let value: Completion =
        serde_json::from_slice(frame).map_err(|_| "OpenPencil completion frame is invalid")?;
    match value {
        Completion::Result {
            protocol_version,
            release_id,
            job_id,
            operation: actual_operation,
            status,
            output_path,
            input_bytes,
            output_bytes,
            input_sha256,
            output_sha256,
            format,
            node_count,
            engine_version,
            mode,
        } => {
            if protocol_version != 1
                || release_id != request.release_id
                || job_id != request.job_id
                || actual_operation != operation.wire_name()
                || status != "staged"
                || output_path != request.output_path.to_string_lossy()
                || input_bytes != input.len()
                || input_sha256 != digest(input)
                || output_bytes == 0
                || output_bytes > request.max_output_bytes
                || output_sha256.len() != 64
                || !output_sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                || format != operation.output_extension()
                || node_count == 0
                || node_count > crate::openpencil::MAX_NODES
                || engine_version != ENGINE_VERSION
                || mode != MODE
            {
                return Err("OpenPencil result does not match its finite request".into());
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
            if protocol_version != 1
                || release_id != request.release_id
                || job_id.as_deref() != Some(&request.job_id)
                || code.is_empty()
                || code.len() > 100
                || !code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || message.len() > 256
            {
                return Err("OpenPencil refusal identity or bounds differ".into());
            }
            Err(format!(
                "OpenPencil refused ({code}); private stage may exist: {output_may_exist}"
            ))
        }
    }
}

fn operation_frame(request: &DesignRequest, operation: &OpenPencilOperation) -> Value {
    let mut frame = json!({
        "type":"run", "protocol_version":1, "release_id":request.release_id,
        "job_id":request.job_id, "job_root":request.job_root,
        "input_path":request.input_path, "source_revision":request.source_revision,
        "output_path":request.output_path, "operation":operation.wire_name(),
        "deadline_ms":request.timeout_ms,
    });
    match operation {
        OpenPencilOperation::Create => {}
        OpenPencilOperation::Patch { edits } => frame["edits"] = json!(edits),
        OpenPencilOperation::ExportSvg { target_path } => frame["target_path"] = json!(target_path),
    }
    frame
}

/// Execute exactly one editable-file operation; the caller retains the shared
/// design worker and its admission lease until child cleanup is confirmed.
pub async fn run_openpencil_adapter(
    payload: &OpenPencilPayload,
    request: &DesignRequest,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<DesignOutcome, AdapterError> {
    if !cfg!(unix) {
        return Err("OpenPencil process ownership is not qualified on this platform".into());
    }
    if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
        return Err("OpenPencil operation cancelled or expired before launch".into());
    }
    request.validate()?;
    let DesignOperation::OpenPencil { operation } = &request.operation else {
        return Err("OpenPencil leaf requires an OpenPencil operation".into());
    };
    validate_job_dir(&request.job_root)?;
    for file in [&payload.node, &payload.adapter] {
        crate::owned_scratch::explicit_path(file, false)?;
    }
    crate::owned_scratch::explicit_path(&payload.runtime_root, true)?;
    if payload.runtime_sha256.len() != 64
        || !payload
            .runtime_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("OpenPencil runtime manifest digest is invalid".into());
    }
    let runtime_manifest = payload.runtime_root.join("runtime.json");
    let manifest_meta = crate::owned_scratch::explicit_path(&runtime_manifest, false)?;
    if manifest_meta.len() > 1024 * 1024
        || digest(
            &fs::read(&runtime_manifest).map_err(|_| "OpenPencil runtime manifest unavailable")?,
        ) != payload.runtime_sha256
    {
        return Err("OpenPencil runtime manifest differs from its selected digest".into());
    }
    let scope =
        ProjectScope::open(&request.job_root).map_err(|_| "OpenPencil job scope is unavailable")?;
    let source = scope
        .read_bytes(&request.input_path, crate::openpencil::MAX_INPUT_BYTES)
        .map_err(|_| "OpenPencil source is unavailable")?
        .bytes;
    if crate::engine::revision(&source) != request.source_revision {
        return Err("OpenPencil source snapshot changed before launch".into());
    }
    match fs::symlink_metadata(&request.output_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        _ => return Err("OpenPencil output stage must be absent before launch".into()),
    }
    let mut frame = serde_json::to_vec(&operation_frame(request, operation))
        .map_err(|_| "OpenPencil request could not be encoded")?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME_BYTES {
        return Err("OpenPencil edit request exceeds its frame limit".into());
    }
    let mut command = Command::new(&payload.node);
    command
        .arg("--max-old-space-size=192")
        .arg(&payload.adapter)
        .arg("--release-id")
        .arg(&request.release_id)
        .arg("--runtime-root")
        .arg(&payload.runtime_root)
        .arg("--runtime-sha256")
        .arg(&payload.runtime_sha256)
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
    let mut owned = OwnedChild::new(
        command
            .spawn()
            .map_err(|error| format!("OpenPencil adapter launch failed: {error}"))?,
    )
    .map_err(unconfirmed)?;
    let (Some(mut stdin), Some(stdout), Some(stderr)) = (
        owned.child.stdin.take(),
        owned.child.stdout.take(),
        owned.child.stderr.take(),
    ) else {
        return match owned.terminate_and_reap().await {
            Ok(()) => Err("OpenPencil adapter pipes are unavailable".into()),
            Err(error) => Err(unconfirmed(format!(
                "OpenPencil pipe cleanup unconfirmed: {error}"
            ))),
        };
    };
    let exchange = async {
        let protocol = async {
            let mut stdout = BufReader::new(stdout);
            let ready = read_frame(&mut stdout).await?;
            ready_matches(&ready, &request.release_id)?;
            stdin
                .write_all(&frame)
                .await
                .map_err(|_| "OpenPencil request write failed")?;
            stdin
                .shutdown()
                .await
                .map_err(|_| "OpenPencil request close failed")?;
            drop(stdin);
            let response = read_frame(&mut stdout).await?;
            let result = completion_matches(&response, request, operation, &source)?;
            let mut extra = [0u8; 1];
            if stdout
                .read(&mut extra)
                .await
                .map_err(|_| "OpenPencil stdout close failed")?
                != 0
            {
                return Err("OpenPencil adapter emitted trailing stdout".into());
            }
            Ok(result)
        };
        let ((output_bytes, output_sha256), stderr_bytes) =
            tokio::try_join!(protocol, drain_stderr(stderr))?;
        let status = owned.wait().await?;
        if !status.success() {
            return Err("OpenPencil adapter returned a result but exited unsuccessfully".into());
        }
        let output = scope
            .read_bytes(&request.output_path, request.max_output_bytes)
            .map_err(|_| "OpenPencil staged output could not be read")?
            .bytes;
        if output.len() != output_bytes || digest(&output) != output_sha256 {
            return Err("OpenPencil output stage differs from its response identity".into());
        }
        let source_after = scope
            .read_bytes(&request.input_path, crate::openpencil::MAX_INPUT_BYTES)
            .map_err(|_| "OpenPencil source disappeared during operation")?
            .bytes;
        if source_after != source
            || crate::engine::revision(&source_after) != request.source_revision
        {
            return Err("OpenPencil source changed during operation".into());
        }
        Ok(DesignOutcome {
            output_revision: crate::engine::revision(&output),
            output_bytes,
            input_bytes: source.len(),
            stderr_bytes,
            cleanup_confirmed: true,
        })
    };
    let result = tokio::select! {
        biased;
        _ = cancelled(cancel) => Err("OpenPencil operation cancelled".to_owned()),
        value = tokio::time::timeout_at(deadline.min(tokio::time::Instant::now() + Duration::from_millis(request.timeout_ms)), exchange) => match value {
            Ok(value) => value,
            Err(_) => Err("OpenPencil operation deadline exceeded".to_owned()),
        },
    };
    if result.is_err()
        && !owned.is_reaped()
        && let Err(error) = owned.terminate_and_reap().await
    {
        return Err(unconfirmed(format!(
            "OpenPencil adapter cleanup unconfirmed: {error}; retain admission for reconciliation"
        )));
    }
    result.map_err(AdapterError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn ready_requires_exact_editable_file_surface() {
        let frame = serde_json::to_vec(&json!({
            "type":"ready", "protocol_version":1, "release_id":"release-1",
            "operations":["openpencil.create","openpencil.patch","openpencil.export_svg"],
            "max_frame_bytes":65536, "max_input_bytes":1048576,
            "max_output_bytes":16777216, "min_deadline_ms":100,
            "max_deadline_ms":60000, "engine_version":"0.16.0", "mode":"editable_file"
        }))
        .unwrap();
        assert!(ready_matches(&frame, "release-1").is_ok());
        assert!(ready_matches(&frame, "release-2").is_err());
    }

    /// Run this with exact private candidate paths to check the Rust-owned
    /// process boundary as well as the independently tested Node leaf.
    #[tokio::test]
    async fn private_editable_file_round_trip_when_selected() {
        let Some(node) = std::env::var_os("MIRKET_TEST_OPENPENCIL_NODE") else {
            return;
        };
        let adapter = std::env::var_os("MIRKET_TEST_OPENPENCIL_ADAPTER").unwrap();
        let runtime_root = std::env::var_os("MIRKET_TEST_OPENPENCIL_RUNTIME").unwrap();
        let runtime_sha256 = std::env::var("MIRKET_TEST_OPENPENCIL_SHA256").unwrap();
        let payload = OpenPencilPayload {
            node: PathBuf::from(node),
            adapter: PathBuf::from(adapter),
            runtime_root: PathBuf::from(runtime_root),
            runtime_sha256,
        };
        let parent = std::env::temp_dir().canonicalize().unwrap();
        let id = format!(
            "mirket-openpencil-supervisor-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let job_root = parent.join(id);
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&job_root).unwrap();
        let input_path = job_root.join("source.json");
        let output_path = job_root.join("result.fig");
        let input = br##"{"page":"Shapes","nodes":[{"kind":"frame","name":"Card","width":320,"height":180,"fill":"#ffffff","children":[{"kind":"rectangle","name":"Box","x":20,"y":30,"width":120,"height":40,"fill":"#1463ff"}]}]}"##;
        fs::write(&input_path, input).unwrap();
        let request = DesignRequest {
            protocol_version: 1,
            release_id: "openpencil-candidate".into(),
            job_id: "job-1".into(),
            job_root: job_root.clone(),
            input_path: input_path.clone(),
            source_revision: crate::engine::revision(input),
            output_path: output_path.clone(),
            operation: DesignOperation::OpenPencil {
                operation: OpenPencilOperation::Create,
            },
            timeout_ms: 30_000,
            max_output_bytes: crate::openpencil::MAX_OUTPUT_BYTES,
        };
        let result = run_openpencil_adapter(
            &payload,
            &request,
            Arc::new(AtomicBool::new(false)),
            tokio::time::Instant::now() + Duration::from_secs(40),
        )
        .await
        .unwrap();
        assert!(result.cleanup_confirmed);
        assert_eq!(result.input_bytes, input.len());
        assert_eq!(
            result.output_revision,
            crate::engine::revision(&fs::read(&output_path).unwrap())
        );
        assert!(result.output_bytes > 100);
        fs::remove_dir_all(job_root).unwrap();
    }
}
