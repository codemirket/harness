//! One finite same-binary supervisor owns each leaf child and admission lease.
//!
//! The parent keeps the worker's stdin pipe open after one bounded request.
//! EOF means the parent disappeared or cancelled: the worker cancels and reaps
//! its leaf before releasing admission. There is no daemon and no PID-based reclaim.
//! If the worker itself dies, its durable unresolved marker blocks the slot.
//! Code and design leaves share this owner; arbitrary process trees and native
//! Windows Job Objects remain qualification gates.

use crate::leases::Admission;
use crate::process::AdapterError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

// The escaped selected-source request is bounded at 2 MiB; the supervisor
// adds only configured private paths and its tagged ownership envelope.
const MAX_FRAME: usize = crate::code_navigation::MAX_REQUEST_BYTES + 16_384;
const REQUEST_DEADLINE: Duration = Duration::from_secs(5);
const RESPONSE_DEADLINE: Duration = Duration::from_secs(70);
const FINITE_WORK_BUDGET: Duration = Duration::from_secs(60);
const MAX_PROGRESS_FRAME: usize = 512;
type ProgressCallback<'a> = &'a mut (dyn FnMut(u64) -> Result<(), String> + Send);
type AdmissionProgress =
    tokio::sync::oneshot::Sender<(u64, tokio::sync::oneshot::Sender<Result<(), String>>)>;
const CLEANUP_DEADLINE: Duration = Duration::from_secs(12);

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagramWork {
    pub state_root: PathBuf,
    pub executable: PathBuf,
    pub request: crate::diagram::D2RenderRequest,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignWork {
    pub state_root: PathBuf,
    pub payload: crate::design::DesignLeafPayload,
    pub request: crate::design::DesignRequest,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeInspectWork {
    pub state_root: PathBuf,
    pub payload: crate::code_inspect::Payload,
    pub request: crate::code_inspect::LeafRequest,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeNavigationWork {
    pub state_root: PathBuf,
    pub payload: crate::code_navigation::Payload,
    pub request: crate::code_navigation::LeafRequest,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "request",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum Work {
    Diagram(DiagramWork),
    Design(DesignWork),
    CodeInspect(CodeInspectWork),
    CodeNavigation(CodeNavigationWork),
}
impl Work {
    fn work_budget(&self) -> Duration {
        FINITE_WORK_BUDGET
    }
    fn response_deadline(&self) -> Duration {
        RESPONSE_DEADLINE
    }
    fn state_root(&self) -> &Path {
        match self {
            Self::Diagram(r) => &r.state_root,
            Self::Design(r) => &r.state_root,
            Self::CodeInspect(r) => &r.state_root,
            Self::CodeNavigation(r) => &r.state_root,
        }
    }
    fn release_id(&self) -> &str {
        match self {
            Self::Diagram(r) => &r.request.release_id,
            Self::Design(r) => &r.request.release_id,
            Self::CodeInspect(r) => &r.request.release_id,
            Self::CodeNavigation(r) => &r.request.release_id,
        }
    }
    fn job_id(&self) -> &str {
        match self {
            Self::Diagram(r) => &r.request.job_id,
            Self::Design(r) => &r.request.job_id,
            Self::CodeInspect(r) => &r.request.job_id,
            Self::CodeNavigation(r) => &r.request.job_id,
        }
    }
    fn job_dir(&self) -> Option<&Path> {
        match self {
            Self::Diagram(r) => Some(&r.request.job_root),
            Self::Design(r) => Some(&r.request.job_root),
            Self::CodeInspect(r) => Some(&r.state_root),
            Self::CodeNavigation(r) => Some(&r.state_root),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", deny_unknown_fields)]
enum WorkerResponse {
    #[serde(rename = "code_navigation_result")]
    CodeNavigationResult {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        outcome: crate::code_navigation::Outcome,
    },
    #[serde(rename = "code_inspect_result")]
    CodeInspectResult {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        outcome: crate::code_inspect::Outcome,
    },
    #[serde(rename = "design_result")]
    DesignResult {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        outcome: crate::design::DesignOutcome,
    },
    #[serde(rename = "diagram_result")]
    DiagramResult {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        outcome: crate::diagram::D2RenderOutcome,
    },
    #[serde(rename = "running")]
    Running {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        wait_ms: u64,
    },
    #[serde(rename = "error")]
    Error {
        protocol_version: u32,
        release_id: String,
        job_id: String,
        message: String,
        cleanup_confirmed: bool,
    },
}

fn unknown(message: impl Into<String>) -> AdapterError {
    AdapterError {
        message: message.into(),
        cleanup_confirmed: false,
    }
}

pub async fn run_diagram_owned(
    current_exe: &Path,
    request: DiagramWork,
    cancel: Arc<AtomicBool>,
    on_running: ProgressCallback<'_>,
) -> Result<crate::diagram::D2RenderOutcome, AdapterError> {
    request
        .request
        .validate()
        .map_err(|error| error.to_string())?;
    let bytes = launch(
        current_exe,
        Work::Diagram(request.clone()),
        cancel,
        on_running,
    )
    .await?;
    match serde_json::from_slice::<WorkerResponse>(&bytes)
        .map_err(|_| unknown("D2 worker response is invalid"))?
    {
        WorkerResponse::DiagramResult {
            protocol_version: 1,
            release_id,
            job_id,
            outcome,
        } if release_id == request.request.release_id
            && job_id == request.request.job_id
            && outcome.stderr_bytes <= 16_384
            && outcome.cleanup_confirmed =>
        {
            crate::diagram::validate_result(&request.request, &outcome.result)
                .map_err(|error| unknown(error.to_string()))?;
            Ok(outcome)
        }
        WorkerResponse::Error {
            protocol_version: 1,
            release_id,
            job_id,
            message,
            cleanup_confirmed,
        } if release_id == request.request.release_id && job_id == request.request.job_id => {
            Err(AdapterError {
                message,
                cleanup_confirmed,
            })
        }
        _ => Err(unknown("D2 worker response does not match its request")),
    }
}

pub async fn run_design_owned(
    current_exe: &Path,
    request: DesignWork,
    cancel: Arc<AtomicBool>,
    on_running: ProgressCallback<'_>,
) -> Result<crate::design::DesignOutcome, AdapterError> {
    request
        .request
        .validate()
        .map_err(|error| error.to_string())?;
    let bytes = launch(
        current_exe,
        Work::Design(request.clone()),
        cancel,
        on_running,
    )
    .await?;
    match serde_json::from_slice::<WorkerResponse>(&bytes)
        .map_err(|_| unknown("design worker response is invalid"))?
    {
        WorkerResponse::DesignResult {
            protocol_version: 1,
            release_id,
            job_id,
            outcome,
        } if release_id == request.request.release_id
            && job_id == request.request.job_id
            && outcome.stderr_bytes <= 16_384
            && outcome.output_bytes <= request.request.max_output_bytes
            && outcome.input_bytes <= request.request.operation.max_input_bytes()
            && outcome.cleanup_confirmed
            && crate::design::is_revision(&outcome.output_revision) =>
        {
            Ok(outcome)
        }
        WorkerResponse::Error {
            protocol_version: 1,
            release_id,
            job_id,
            message,
            cleanup_confirmed,
        } if release_id == request.request.release_id && job_id == request.request.job_id => {
            Err(AdapterError {
                message,
                cleanup_confirmed,
            })
        }
        _ => Err(unknown("design worker response does not match its request")),
    }
}

pub async fn run_code_inspect_owned(
    current_exe: &Path,
    request: CodeInspectWork,
    cancel: Arc<AtomicBool>,
) -> Result<crate::code_inspect::Outcome, AdapterError> {
    crate::code_inspect::validate_request(&request.request)?;
    let bytes = launch(
        current_exe,
        Work::CodeInspect(request.clone()),
        cancel,
        &mut |_| Ok(()),
    )
    .await?;
    match serde_json::from_slice::<WorkerResponse>(&bytes)
        .map_err(|_| unknown("code inspection worker response is invalid"))?
    {
        WorkerResponse::CodeInspectResult {
            protocol_version: 1,
            release_id,
            job_id,
            outcome,
        } if release_id == request.request.release_id
            && job_id == request.request.job_id
            && outcome.stderr_bytes <= 16_384 =>
        {
            crate::code_inspect::validate_result(&outcome.result, &request.request)
                .map_err(unknown)?;
            Ok(outcome)
        }
        WorkerResponse::Error {
            protocol_version: 1,
            release_id,
            job_id,
            message,
            cleanup_confirmed,
        } if release_id == request.request.release_id && job_id == request.request.job_id => {
            Err(AdapterError {
                message,
                cleanup_confirmed,
            })
        }
        _ => Err(unknown(
            "code inspection worker response does not match its request",
        )),
    }
}

pub async fn run_code_navigation_owned(
    current_exe: &Path,
    request: CodeNavigationWork,
    cancel: Arc<AtomicBool>,
) -> Result<crate::code_navigation::Outcome, AdapterError> {
    crate::code_navigation::validate_request(&request.request)?;
    let bytes = launch(
        current_exe,
        Work::CodeNavigation(request.clone()),
        cancel,
        &mut |_| Ok(()),
    )
    .await?;
    match serde_json::from_slice::<WorkerResponse>(&bytes)
        .map_err(|_| unknown("code navigation worker response is invalid"))?
    {
        WorkerResponse::CodeNavigationResult {
            protocol_version: 1,
            release_id,
            job_id,
            outcome,
        } if release_id == request.request.release_id
            && job_id == request.request.job_id
            && outcome.stderr_bytes <= 16_384 =>
        {
            crate::code_navigation::validate_result(&outcome.result, &request.request)
                .map_err(unknown)?;
            Ok(outcome)
        }
        WorkerResponse::Error {
            protocol_version: 1,
            release_id,
            job_id,
            message,
            cleanup_confirmed,
        } if release_id == request.request.release_id && job_id == request.request.job_id => {
            Err(AdapterError {
                message,
                cleanup_confirmed,
            })
        }
        _ => Err(unknown(
            "code navigation worker response does not match its request",
        )),
    }
}

async fn launch(
    current_exe: &Path,
    request: Work,
    cancel: Arc<AtomicBool>,
    on_running: ProgressCallback<'_>,
) -> Result<Vec<u8>, AdapterError> {
    if !cfg!(unix) {
        return Err("native worker ownership is not qualified on this platform".into());
    }
    if cancel.load(Ordering::Acquire) {
        return Err("worker cancelled before launch".into());
    }
    if !current_exe.is_absolute() || !current_exe.is_file() {
        return Err("worker requires an explicit existing Mirket executable".into());
    }
    let mut frame =
        serde_json::to_vec(&request).map_err(|_| "worker request could not be encoded")?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME {
        return Err("worker request exceeds its byte limit".into());
    }
    let cwd = request
        .job_dir()
        .ok_or("worker requires an explicit job directory")?;
    if !cwd.is_absolute() {
        return Err("worker requires an absolute job directory".into());
    }
    let mut command = Command::new(current_exe);
    command
        .arg("__worker")
        .env_clear()
        .env("TMPDIR", cwd)
        .env("TMP", cwd)
        .env("TEMP", cwd)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    // Deliberately no kill_on_drop: dropping the parent future must close the
    // lifeline and leave the worker alive long enough to reap its child.
    let mut child = command
        .spawn()
        .map_err(|e| format!("worker launch failed: {e}"))?;
    let mut lifeline = Some(
        child
            .stdin
            .take()
            .ok_or_else(|| unknown("worker lifeline is unavailable"))?,
    );
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| unknown("worker response pipe is unavailable"))?;
    tokio::time::timeout(
        REQUEST_DEADLINE,
        lifeline.as_mut().unwrap().write_all(&frame),
    )
    .await
    .map_err(|_| unknown("worker request deadline exceeded; ownership outcome is unknown"))?
    .map_err(|_| unknown("worker request failed; ownership outcome is unknown"))?;
    let mut reader = BufReader::new(stdout);
    let response = {
        let reply = read_completion(&mut reader, &request, on_running, cancel.clone());
        tokio::pin!(reply);
        tokio::select! {
            biased;
            _ = wait_cancel(cancel) => {
                drop(lifeline.take());
                tokio::time::timeout(CLEANUP_DEADLINE, &mut reply).await
                    .map_err(|_| unknown("worker cancellation cleanup is unconfirmed; inspect ownership marker"))?
            },
            response = tokio::time::timeout(request.response_deadline(), &mut reply) => match response {
                Ok(response) => response,
                Err(_) => {
                    drop(lifeline.take());
                    tokio::time::timeout(CLEANUP_DEADLINE, &mut reply).await
                        .map_err(|_| unknown("worker deadline cleanup is unconfirmed; inspect ownership marker"))?
                }
            },
        }
    };
    // The worker waits for this close after replying. No Tokio stdin blocking
    // reader is used there, so even a malformed parent cannot strand runtime
    // shutdown in a blocked read thread.
    drop(lifeline.take());
    let status = tokio::time::timeout(CLEANUP_DEADLINE, child.wait())
        .await
        .map_err(|_| unknown("worker exit is unconfirmed; inspect ownership marker"))?
        .map_err(|_| unknown("worker exit could not be reaped; inspect ownership marker"))?;
    if !status.success() {
        return Err(unknown("worker exited without a confirmed completion"));
    }
    if !tokio::time::timeout(REQUEST_DEADLINE, reader.fill_buf())
        .await
        .map_err(|_| unknown("worker stdout did not close after exit"))?
        .map_err(|_| unknown("worker stdout could not be inspected"))?
        .is_empty()
    {
        return Err(unknown("worker emitted unexpected trailing output"));
    }
    let completion = response.map_err(unknown)?;
    if let Some(message) = completion.progress_error {
        return Err(AdapterError {
            message,
            cleanup_confirmed: completion.cleanup_confirmed,
        });
    }
    Ok(completion.bytes)
}

struct CompletionFrame {
    bytes: Vec<u8>,
    cleanup_confirmed: bool,
    progress_error: Option<String>,
}

async fn read_completion<R: AsyncBufRead + Unpin>(
    reader: &mut R,
    request: &Work,
    on_running: ProgressCallback<'_>,
    cancel: Arc<AtomicBool>,
) -> Result<CompletionFrame, String> {
    let mut running = false;
    let mut progress_error = None;
    loop {
        let bytes = read_response(reader).await?;
        let response: WorkerResponse =
            serde_json::from_slice(&bytes).map_err(|_| "worker response is invalid")?;
        let cleanup_confirmed = match response {
            WorkerResponse::Running {
                protocol_version: 1,
                release_id,
                job_id,
                wait_ms,
            } if !running
                && release_id == request.release_id()
                && job_id == request.job_id()
                && wait_ms <= 60_000
                && bytes.len() <= MAX_PROGRESS_FRAME =>
            {
                running = true;
                if let Err(message) = on_running(wait_ms) {
                    progress_error = Some(format!(
                        "running receipt update failed: {}",
                        message.chars().take(2048).collect::<String>()
                    ));
                    // The outer launch loop closes the lifeline. Keep reading
                    // the bound terminal reply rather than dropping cleanup.
                    cancel.store(true, Ordering::Release);
                }
                continue;
            }
            WorkerResponse::DiagramResult {
                protocol_version: 1,
                release_id,
                job_id,
                ..
            } if matches!(request, Work::Diagram(_))
                && running
                && release_id == request.release_id()
                && job_id == request.job_id() =>
            {
                true
            }
            WorkerResponse::DesignResult {
                protocol_version: 1,
                release_id,
                job_id,
                ..
            } if matches!(request, Work::Design(_))
                && running
                && release_id == request.release_id()
                && job_id == request.job_id() =>
            {
                true
            }
            WorkerResponse::CodeInspectResult {
                protocol_version: 1,
                release_id,
                job_id,
                ..
            } if matches!(request, Work::CodeInspect(_))
                && running
                && release_id == request.release_id()
                && job_id == request.job_id() =>
            {
                true
            }
            WorkerResponse::CodeNavigationResult {
                protocol_version: 1,
                release_id,
                job_id,
                ..
            } if matches!(request, Work::CodeNavigation(_))
                && running
                && release_id == request.release_id()
                && job_id == request.job_id() =>
            {
                true
            }
            WorkerResponse::Error {
                protocol_version: 1,
                release_id,
                job_id,
                message,
                cleanup_confirmed,
            } if release_id == request.release_id()
                && job_id == request.job_id()
                && message.len() <= 8192 =>
            {
                cleanup_confirmed
            }
            _ => return Err("worker reply identity or progress order is invalid".into()),
        };
        return Ok(CompletionFrame {
            bytes,
            cleanup_confirmed,
            progress_error,
        });
    }
}

/// One category only: a waiter owns no heavy resource or descendant. The
/// native waiting permit is dropped on every return, including promotion.
async fn acquire_finite(
    admission: &Admission,
    cancel: &Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<crate::leases::SlotLease, AdapterError> {
    let mut waiting = None;
    loop {
        if cancel.load(Ordering::Acquire) {
            return Err("job cancelled while queued".into());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err("work deadline exceeded while queued".into());
        }
        match crate::resource_admission::admit(admission.state_root()) {
            Ok(lease) => {
                if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
                    return Err("job cancelled or work deadline exceeded before admission".into());
                }
                return Ok(lease);
            }
            Err(
                crate::resource_admission::Error::Busy
                | crate::resource_admission::Error::Deferred(_),
            ) => (),
            Err(error) => return Err(error.to_string().into()),
        }
        if waiting.is_none() {
            waiting = Some(admission.try_waiting().map_err(|e| e.to_string())?);
        }
        tokio::select! {
            biased;
            _ = wait_cancel(cancel.clone()) => return Err("job cancelled while queued".into()),
            _ = tokio::time::sleep_until(deadline.min(tokio::time::Instant::now() + Duration::from_millis(25))) => (),
        }
    }
}

/// Hidden same-binary entry point. The CLI must dispatch this exact mode before
/// starting its normal MCP server; it never accepts a command to execute.
pub async fn serve_worker() -> Result<(), String> {
    #[cfg(not(unix))]
    {
        return Err("native worker ownership is not qualified on this platform".into());
    }
    #[cfg(unix)]
    {
        let mut lifeline = Lifeline::open(libc::STDIN_FILENO)?;
        let frame = tokio::time::timeout(REQUEST_DEADLINE, lifeline.read_request())
            .await
            .map_err(|_| "worker request deadline exceeded")??;
        let request: Work =
            serde_json::from_slice(&frame).map_err(|_| "worker request is invalid")?;
        let _session_guard = crate::maintenance::SessionGuard::acquire(request.state_root())
            .map_err(|e| e.to_string())?;
        let cancelled = Arc::new(AtomicBool::new(false));
        let (progress_tx, mut progress_rx) = tokio::sync::oneshot::channel();
        let mut progress_open = true;
        let mut stdout = Lifeline::open(libc::STDOUT_FILENO)?;
        let work = execute_work(&request, cancelled.clone(), progress_tx);
        tokio::pin!(work);
        let mut parent_closed = false;
        let result = loop {
            tokio::select! {
                biased;
                _ = lifeline.wait_closed() => {
                    parent_closed = true;
                    cancelled.store(true, Ordering::Release);
                    break work.await;
                },
                progress = &mut progress_rx, if progress_open => {
                    progress_open = false;
                    if let Ok((wait_ms, acknowledged)) = progress {
                        let frame = WorkerResponse::Running { protocol_version: 1,
                            release_id: request.release_id().to_owned(), job_id: request.job_id().to_owned(), wait_ms };
                        let sent = match serde_json::to_vec(&frame) {
                            Ok(mut bytes) if bytes.len() < MAX_PROGRESS_FRAME => {
                                bytes.push(b'\n');
                                tokio::time::timeout(REQUEST_DEADLINE, stdout.write_all(&bytes)).await
                                    .map_err(|_| "worker progress delivery deadline exceeded".to_owned()).and_then(|r| r)
                            },
                            _ => Err("worker progress frame exceeds its limit".into()),
                        };
                        if sent.is_err() { cancelled.store(true, Ordering::Release); }
                        let _ = acknowledged.send(sent);
                    }
                },
                result = &mut work => break result,
            }
        };
        let response = match result {
            Ok(response) => response,
            Err(error) => WorkerResponse::Error {
                protocol_version: 1,
                release_id: request.release_id().to_owned(),
                job_id: request.job_id().to_owned(),
                message: error.message.chars().take(2048).collect(),
                cleanup_confirmed: error.cleanup_confirmed,
            },
        };
        let mut bytes =
            serde_json::to_vec(&response).map_err(|_| "worker response could not be encoded")?;
        bytes.push(b'\n');
        if bytes.len() > MAX_FRAME {
            return Err("worker response exceeds its byte limit".into());
        }
        // One short response fits the ordinary pipe capacity. Failure after
        // parent death is expected and cannot undo the already awaited cleanup.
        let sent = tokio::time::timeout(REQUEST_DEADLINE, stdout.write_all(&bytes)).await;
        if !parent_closed {
            let _ = tokio::time::timeout(REQUEST_DEADLINE, lifeline.wait_closed()).await;
        }
        match sent {
            Ok(Ok(())) => Ok(()),
            _ if parent_closed => Ok(()),
            _ => Err("worker response could not be delivered".into()),
        }
    }
}

async fn execute_work(
    request: &Work,
    cancel: Arc<AtomicBool>,
    progress: AdmissionProgress,
) -> Result<WorkerResponse, AdapterError> {
    let started = tokio::time::Instant::now();
    let deadline = started + request.work_budget();
    let admission = Admission::open(request.state_root()).map_err(|e| e.to_string())?;
    let mut lease = {
        let lease = acquire_finite(&admission, &cancel, deadline).await?;
        let wait_ms = started.elapsed().as_millis().min(60_000) as u64;
        let (acknowledged, ack) = tokio::sync::oneshot::channel();
        progress
            .send((wait_ms, acknowledged))
            .map_err(|_| "worker progress receiver unavailable")?;
        tokio::select! {
            biased;
            _ = wait_cancel(cancel.clone()) => return Err("job cancelled before launch".into()),
            result = tokio::time::timeout_at(deadline, ack) => {
                result.map_err(|_| "work deadline exceeded before launch")?
                    .map_err(|_| "worker progress delivery unconfirmed")??;
            }
        }
        if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
            return Err("job cancelled or work deadline exceeded before launch".into());
        }
        lease
    };
    lease.mark_unresolved(request.job_id()).map_err(|e| {
        unknown(format!(
            "worker ownership marker could not be persisted: {e}"
        ))
    })?;
    let result = match request {
        Work::Diagram(request) => execute_diagram(request, cancel, deadline).await,
        Work::Design(request) => execute_design(request, cancel, deadline).await,
        Work::CodeInspect(request) => execute_code_inspect(request, cancel, deadline).await,
        Work::CodeNavigation(request) => execute_code_navigation(request, cancel, deadline).await,
    };
    if result
        .as_ref()
        .map_or_else(|error| error.cleanup_confirmed, |_| true)
    {
        lease
            .clear_after_cleanup()
            .map_err(|e| unknown(format!("worker cleanup marker could not be cleared: {e}")))?;
    }
    result
}

async fn execute_code_inspect(
    work: &CodeInspectWork,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<WorkerResponse, AdapterError> {
    let outcome = crate::code_inspect::run_leaf(
        &work.payload,
        &work.request,
        &work.state_root,
        cancel,
        deadline,
    )
    .await?;
    Ok(WorkerResponse::CodeInspectResult {
        protocol_version: 1,
        release_id: work.request.release_id.clone(),
        job_id: work.request.job_id.clone(),
        outcome,
    })
}

async fn execute_code_navigation(
    work: &CodeNavigationWork,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<WorkerResponse, AdapterError> {
    let outcome = crate::code_navigation::run_leaf(
        &work.payload,
        &work.request,
        &work.state_root,
        cancel,
        deadline,
    )
    .await?;
    Ok(WorkerResponse::CodeNavigationResult {
        protocol_version: 1,
        release_id: work.request.release_id.clone(),
        job_id: work.request.job_id.clone(),
        outcome,
    })
}

async fn execute_diagram(
    work: &DiagramWork,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<WorkerResponse, AdapterError> {
    use crate::owned_scratch::{DirectoryStamp, OwnedScratch};
    work.request.validate().map_err(|error| error.to_string())?;
    let stamp = DirectoryStamp::inspect(&work.state_root)?;
    let temporary = OwnedScratch::create(
        &work.state_root,
        &stamp,
        &format!("{}-d2-tmp", work.request.job_id),
    )?;
    let outcome = run_d2_leaf(
        &work.executable,
        &work.request,
        &temporary.path,
        cancel,
        deadline,
    )
    .await;
    if outcome.as_ref().map_or_else(
        |error| error.cleanup_confirmed,
        |result| result.cleanup_confirmed,
    ) {
        temporary
            .remove_empty()
            .map_err(|error| unknown(format!("D2 temporary cleanup unconfirmed: {error}")))?;
    }
    let outcome = outcome?;
    if !outcome.cleanup_confirmed {
        return Err(unknown("D2 native cleanup unconfirmed"));
    }
    Ok(WorkerResponse::DiagramResult {
        protocol_version: 1,
        release_id: work.request.release_id.clone(),
        job_id: work.request.job_id.clone(),
        outcome,
    })
}

async fn execute_design(
    work: &DesignWork,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<WorkerResponse, AdapterError> {
    use crate::owned_scratch::{DirectoryStamp, OwnedScratch};
    work.request.validate().map_err(|error| error.to_string())?;
    crate::process::validate_job_dir(&work.request.job_root)?;
    let scope = crate::scope::ProjectScope::open(&work.request.job_root)
        .map_err(|error| format!("design job scope is invalid: {error}"))?;
    let source = scope
        .read_bytes(
            &work.request.input_path,
            work.request.operation.max_input_bytes(),
        )
        .map_err(|error| format!("design source could not be read: {error}"))?
        .bytes;
    if crate::engine::revision(&source) != work.request.source_revision {
        return Err("design source snapshot changed before launch".into());
    }
    match std::fs::symlink_metadata(&work.request.output_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        _ => return Err("design output stage must be absent before launch".into()),
    }
    let stamp = DirectoryStamp::inspect(&work.state_root)?;
    let temporary = OwnedScratch::create(
        &work.state_root,
        &stamp,
        &format!("{}-design-tmp", work.request.job_id),
    )?;
    let outcome: Result<crate::design::DesignOutcome, AdapterError> = async {
        match (&work.payload, &work.request.operation) {
            (
                crate::design::DesignLeafPayload::Standard(payload),
                crate::design::DesignOperation::SvgRenderPng { width, height },
            ) => {
                let stderr_bytes = run_raster_leaf(
                    &payload.font,
                    &work.request,
                    *width,
                    *height,
                    &temporary.path,
                    cancel.clone(),
                    deadline,
                )
                .await?;
                let bytes = scope
                    .read_bytes(&work.request.output_path, work.request.max_output_bytes)
                    .map_err(|error| format!("design PNG stage could not be read: {error}"))?
                    .bytes;
                if bytes.len() < 24
                    || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
                    || u32::from_be_bytes(bytes[16..20].try_into().unwrap()) != *width
                    || u32::from_be_bytes(bytes[20..24].try_into().unwrap()) != *height
                {
                    return Err("design raster leaf produced an invalid PNG stage".into());
                }
                Ok(crate::design::DesignOutcome {
                    output_revision: crate::engine::revision(&bytes),
                    output_bytes: bytes.len(),
                    input_bytes: source.len(),
                    stderr_bytes,
                    cleanup_confirmed: true,
                })
            }
            (
                crate::design::DesignLeafPayload::OpenPencil(payload),
                crate::design::DesignOperation::OpenPencil { .. },
            ) => {
                crate::process_openpencil::run_openpencil_adapter(
                    payload,
                    &work.request,
                    cancel.clone(),
                    deadline,
                )
                .await
            }
            (crate::design::DesignLeafPayload::Standard(payload), _) => {
                crate::process_design::run_design_adapter(
                    payload,
                    &work.request,
                    cancel.clone(),
                    deadline,
                )
                .await
            }
            _ => Err("design operation does not match its selected private payload".into()),
        }
    }
    .await;
    if outcome.as_ref().map_or_else(
        |error| error.cleanup_confirmed,
        |value| value.cleanup_confirmed,
    ) {
        temporary
            .remove_empty()
            .map_err(|error| unknown(format!("design temporary cleanup unconfirmed: {error}")))?;
    }
    let result = outcome?;
    if result.input_bytes != source.len()
        || result.output_bytes > work.request.max_output_bytes
        || !result.cleanup_confirmed
    {
        return Err(unknown(
            "design result does not match its admitted source and bounds",
        ));
    }
    let saved = scope
        .read_bytes(&work.request.output_path, work.request.max_output_bytes)
        .map_err(|error| format!("design stage could not be verified: {error}"))?
        .bytes;
    if saved.len() != result.output_bytes
        || crate::engine::revision(&saved) != result.output_revision
    {
        return Err("design stage changed after worker completion".into());
    }
    Ok(WorkerResponse::DesignResult {
        protocol_version: 1,
        release_id: work.request.release_id.clone(),
        job_id: work.request.job_id.clone(),
        outcome: result,
    })
}

async fn run_raster_leaf(
    font: &Path,
    request: &crate::design::DesignRequest,
    width: u32,
    height: u32,
    temporary: &Path,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<usize, AdapterError> {
    use crate::process::{OwnedChild, cancelled, drain_stderr};
    if !cfg!(unix) {
        return Err("native design raster ownership is not qualified on this platform".into());
    }
    if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
        return Err("design raster cancelled or expired before launch".into());
    }
    crate::design_raster_leaf::verify_font(font)?;
    let executable =
        std::env::current_exe().map_err(|_| "design raster worker executable is unavailable")?;
    let mut command = Command::new(executable);
    command
        .arg("__design_raster")
        .arg(&request.input_path)
        .arg(&request.output_path)
        .arg(font)
        .arg(width.to_string())
        .arg(height.to_string())
        .env_clear()
        .env("HOME", temporary)
        .env("TMPDIR", temporary)
        .env("TMP", temporary)
        .env("TEMP", temporary)
        .current_dir(&request.job_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    let child = command
        .spawn()
        .map_err(|error| format!("private design raster launch failed: {error}"))?;
    let mut owned = OwnedChild::new(child).map_err(unknown)?;
    let stderr = owned
        .child
        .stderr
        .take()
        .ok_or_else(|| unknown("design raster stderr pipe unavailable"))?;
    let execution = async {
        let (status, stderr_bytes) = tokio::try_join!(owned.wait(), drain_stderr(stderr))?;
        if !status.success() {
            return Err("design raster leaf exited unsuccessfully".to_owned());
        }
        Ok::<usize, String>(stderr_bytes)
    };
    let leaf_deadline =
        deadline.min(tokio::time::Instant::now() + Duration::from_millis(request.timeout_ms));
    let result = tokio::select! {
        biased;
        _ = cancelled(cancel) => Err("design raster cancelled before result confirmation".to_owned()),
        value = tokio::time::timeout_at(leaf_deadline, execution) => match value {
            Ok(value) => value,
            Err(_) => Err("design raster deadline exceeded".to_owned()),
        }
    };
    if result.is_err()
        && !owned.is_reaped()
        && let Err(error) = owned.terminate_and_reap().await
    {
        return Err(unknown(format!(
            "design raster cleanup unconfirmed: {error}"
        )));
    }
    result.map_err(AdapterError::from)
}

async fn run_d2_leaf(
    executable: &Path,
    request: &crate::diagram::D2RenderRequest,
    temporary: &Path,
    cancel: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
) -> Result<crate::diagram::D2RenderOutcome, AdapterError> {
    use crate::process::{OwnedChild, cancelled, drain_stderr, validate_job_dir};
    if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        return Err("D2 process ownership is qualified only on macOS ARM64".into());
    }
    request.validate().map_err(|error| error.to_string())?;
    if cancel.load(Ordering::Acquire) || tokio::time::Instant::now() >= deadline {
        return Err("D2 cancelled or expired before launch".into());
    }
    let verified =
        crate::diagram::verify_private_executable(executable).map_err(|error| error.to_string())?;
    validate_job_dir(&request.job_root)?;
    validate_job_dir(temporary)?;
    let metadata = std::fs::symlink_metadata(&request.input_path)
        .map_err(|_| "D2 source snapshot is unavailable")?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > crate::diagram::MAX_SOURCE_BYTES as u64
    {
        return Err("D2 source snapshot must be a bounded regular file".into());
    }
    let scope = crate::scope::ProjectScope::open(&request.job_root)
        .map_err(|error| format!("D2 job scope is invalid: {error}"))?;
    let source = scope
        .read_bytes(&request.input_path, crate::diagram::MAX_SOURCE_BYTES)
        .map_err(|error| format!("D2 source snapshot could not be read: {error}"))?
        .bytes;
    let source_text = std::str::from_utf8(&source).map_err(|_| "D2 source must be UTF-8")?;
    crate::diagram::validate_source(source_text).map_err(|error| error.to_string())?;
    if crate::engine::revision(&source) != request.source_revision {
        return Err("D2 source snapshot changed before launch".into());
    }
    match std::fs::symlink_metadata(&request.output_path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        _ => return Err("D2 output stage must be absent before launch".into()),
    }
    let timeout = request.timeout_ms.div_ceil(1000);
    let mut command = Command::new(&verified.path);
    command
        .arg("--watch=false")
        .arg("--bundle=false")
        .arg(format!("--layout={}", request.layout.cli_value()))
        .arg(format!("--timeout={timeout}"))
        .arg(&request.input_path)
        .arg(&request.output_path)
        .env_clear()
        .env("HOME", temporary)
        .env("XDG_CACHE_HOME", temporary)
        .env("TMPDIR", temporary)
        .env("TMP", temporary)
        .env("TEMP", temporary)
        .current_dir(&request.job_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    let child = command
        .spawn()
        .map_err(|error| format!("private D2 launch failed: {error}"))?;
    let mut owned = OwnedChild::new(child).map_err(unknown)?;
    let stderr = owned
        .child
        .stderr
        .take()
        .ok_or_else(|| unknown("D2 stderr pipe unavailable"))?;
    let execution = async {
        let (status, stderr_bytes) = tokio::try_join!(owned.wait(), drain_stderr(stderr))?;
        if !status.success() {
            return Err("D2 exited unsuccessfully".to_owned());
        }
        let metadata = std::fs::symlink_metadata(&request.output_path)
            .map_err(|_| "D2 output stage missing")?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() == 0
            || metadata.len() > request.max_output_bytes as u64
        {
            return Err("D2 output stage is invalid or exceeds its byte limit".into());
        }
        let bytes = scope
            .read_bytes(&request.output_path, request.max_output_bytes)
            .map_err(|error| format!("D2 output stage could not be read: {error}"))?
            .bytes;
        if bytes.len() != metadata.len() as usize {
            return Err("D2 output changed or exceeds its byte limit".into());
        }
        match request.output_format {
            crate::diagram::D2OutputFormat::Svg => {
                let text = std::str::from_utf8(&bytes).map_err(|_| "D2 SVG is not UTF-8")?;
                if !accepted_d2_svg(text) {
                    return Err("D2 output is not an accepted self-contained SVG".into());
                }
            }
            crate::diagram::D2OutputFormat::Png => {
                crate::diagram::validate_png(&bytes).map_err(str::to_owned)?;
            }
        }
        let output_revision = crate::engine::revision(&bytes);
        let result = crate::diagram::D2RenderResult::Staged {
            output_path: request.output_path.clone(),
            output_revision,
            output_bytes: bytes.len(),
            layout: request.layout,
            output_format: request.output_format,
        };
        crate::diagram::validate_result(request, &result).map_err(|error| error.to_string())?;
        Ok(crate::diagram::D2RenderOutcome {
            result,
            stderr_bytes,
            cleanup_confirmed: true,
        })
    };
    let leaf_deadline =
        deadline.min(tokio::time::Instant::now() + Duration::from_millis(request.timeout_ms));
    let result = tokio::select! {
        biased;
        _ = cancelled(cancel) => Err("D2 cancelled before result confirmation".into()),
        value = tokio::time::timeout_at(leaf_deadline, execution) => match value {
            Ok(value) => value,
            Err(_) => Err("D2 render deadline exceeded".into()),
        },
    };
    if result.is_err()
        && !owned.is_reaped()
        && let Err(error) = owned.terminate_and_reap().await
    {
        return Err(unknown(format!(
            "D2 cleanup unconfirmed: {error}; retain admission for reconciliation"
        )));
    }
    result.map_err(AdapterError::from)
}

fn accepted_d2_svg(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if !(text.starts_with("<?xml version=\"1.0\"") || text.starts_with("<svg"))
        || !text.contains("<svg ")
        || !text.trim_end().ends_with("</svg>")
        || [
            "<script",
            "<foreignobject",
            "<image",
            "<a ",
            "javascript:",
            "href=",
            "@import",
        ]
        .iter()
        .any(|pattern| lower.contains(pattern))
    {
        return false;
    }
    let mut remaining = lower.as_str();
    while let Some(index) = remaining.find("url(") {
        remaining = &remaining[index + 4..];
        let value = remaining.trim_start();
        if !(value.starts_with('#')
            || value.starts_with("\"data:application/font-woff;base64,")
            || value.starts_with("'data:application/font-woff;base64,"))
        {
            return false;
        }
        remaining = value;
    }
    true
}

pub(crate) async fn read_response<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Vec<u8>, String> {
    let mut frame = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .await
            .map_err(|_| "worker response read failed")?;
        if available.is_empty() {
            return Err("worker response ended before completion".into());
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(available.len(), |index| index + 1);
        if frame.len() + count > MAX_FRAME {
            return Err("worker response exceeds its byte limit".into());
        }
        frame.extend_from_slice(&available[..count]);
        reader.consume(count);
        if newline.is_some() {
            return Ok(frame);
        }
    }
}

async fn wait_cancel(cancel: Arc<AtomicBool>) {
    while !cancel.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

#[cfg(unix)]
pub(crate) struct Lifeline {
    fd: i32,
    original_flags: i32,
}

#[cfg(unix)]
impl Lifeline {
    pub(crate) fn open(fd: i32) -> Result<Self, String> {
        // SAFETY: fd0 is this dedicated worker's inherited request pipe. Only
        // this module reads it; nonblocking mode avoids runtime blocking threads.
        let original_flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if original_flags < 0
            || unsafe { libc::fcntl(fd, libc::F_SETFL, original_flags | libc::O_NONBLOCK) } < 0
        {
            return Err("worker lifeline could not be made nonblocking".into());
        }
        Ok(Self { fd, original_flags })
    }

    pub(crate) async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, String> {
        loop {
            // SAFETY: the writable buffer is valid for its exact length and fd0
            // belongs to this worker. No pointer or descriptor escapes.
            let count = unsafe { libc::read(self.fd, buffer.as_mut_ptr().cast(), buffer.len()) };
            if count >= 0 {
                return Ok(count as usize);
            }
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if error.kind() != std::io::ErrorKind::WouldBlock {
                return Err("worker lifeline read failed".into());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    pub(crate) async fn write_all(&mut self, mut bytes: &[u8]) -> Result<(), String> {
        while !bytes.is_empty() {
            // SAFETY: the read-only slice is valid and this is our response fd.
            let count = unsafe { libc::write(self.fd, bytes.as_ptr().cast(), bytes.len()) };
            if count > 0 {
                bytes = &bytes[count as usize..];
                continue;
            }
            if count == 0 {
                return Err("worker response write made no progress".into());
            }
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if error.kind() != std::io::ErrorKind::WouldBlock {
                return Err("worker response write failed".into());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Ok(())
    }

    async fn read_request(&mut self) -> Result<Vec<u8>, String> {
        let mut frame = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let count = self.read(&mut buffer).await?;
            if count == 0 {
                return Err("parent closed before its worker request completed".into());
            }
            if frame.len() + count > MAX_FRAME {
                return Err("worker request exceeds its byte limit".into());
            }
            if let Some(index) = buffer[..count].iter().position(|byte| *byte == b'\n') {
                if index + 1 != count {
                    return Err("worker accepts exactly one request frame".into());
                }
                frame.extend_from_slice(&buffer[..count]);
                return Ok(frame);
            }
            frame.extend_from_slice(&buffer[..count]);
        }
    }

    async fn wait_closed(&mut self) -> Result<(), String> {
        let mut byte = [0u8; 1];
        if self.read(&mut byte).await? == 0 {
            Ok(())
        } else {
            Err("unexpected data after worker request; cancelling".into())
        }
    }
}

#[cfg(unix)]
impl Drop for Lifeline {
    fn drop(&mut self) {
        // SAFETY: restore only the status flags on our still-owned stdin fd.
        let _ = unsafe { libc::fcntl(self.fd, libc::F_SETFL, self.original_flags) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    fn diagram_work() -> Work {
        let root = std::env::temp_dir().join("mirket-worker-protocol-fixture");
        Work::Diagram(DiagramWork {
            state_root: root.clone(),
            executable: root.join("d2"),
            request: crate::diagram::D2RenderRequest {
                protocol_version: crate::diagram::PROTOCOL_VERSION,
                release_id: "test-release".into(),
                job_id: "test-job".into(),
                job_root: root.join("job"),
                input_path: root.join("job/source.d2"),
                source_revision: format!("sha256:{}", "a".repeat(64)),
                output_path: root.join("job/result.svg"),
                layout: crate::diagram::D2Layout::Dagre,
                output_format: crate::diagram::D2OutputFormat::Svg,
                timeout_ms: 30_000,
                max_output_bytes: crate::diagram::MAX_OUTPUT_BYTES,
            },
        })
    }

    fn frames(statuses: &[&str]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for status in statuses {
            let response = match *status {
                "running" => serde_json::json!({"status":"running","protocol_version":1,
                    "release_id":"test-release","job_id":"test-job","wait_ms":17}),
                "wrong_id" => serde_json::json!({"status":"running","protocol_version":1,
                    "release_id":"test-release","job_id":"other","wait_ms":17}),
                "wrong_release" => serde_json::json!({"status":"running","protocol_version":1,
                    "release_id":"other","job_id":"test-job","wait_ms":17}),
                "bad_wait" => serde_json::json!({"status":"running","protocol_version":1,
                    "release_id":"test-release","job_id":"test-job","wait_ms":60_001}),
                "unconfirmed" => serde_json::json!({"status":"error","protocol_version":1,
                    "release_id":"test-release","job_id":"test-job","message":"stopped",
                    "cleanup_confirmed":false}),
                _ => serde_json::json!({"status":"error","protocol_version":1,
                    "release_id":"test-release","job_id":"test-job","message":"stopped",
                    "cleanup_confirmed":true}),
            };
            bytes.extend(serde_json::to_vec(&response).unwrap());
            bytes.push(b'\n');
        }
        bytes
    }

    #[test]
    fn d2_svg_rejects_navigation_and_external_css_urls() {
        let base =
            "<svg xmlns=\"http://www.w3.org/2000/svg\"><style>.n{fill:url(#local)}</style></svg>";
        assert!(accepted_d2_svg(base));
        for unsafe_svg in [
            "<svg xmlns=\"x\"><a href=\"file:///etc/passwd\"/></svg>",
            "<svg xmlns=\"x\"><style>.n{fill:url(../other.svg)}</style></svg>",
            "<svg xmlns=\"x\"><foreignObject/></svg>",
            "<svg xmlns=\"x\"><script/></svg>",
        ] {
            assert!(!accepted_d2_svg(unsafe_svg));
        }
    }

    #[tokio::test]
    async fn worker_frames_are_bounded_and_complete() {
        assert!(
            read_response(&mut BufReader::new(&vec![b'x'; MAX_FRAME + 1][..]))
                .await
                .is_err()
        );
        assert!(
            read_response(&mut BufReader::new(&b"{}"[..]))
                .await
                .is_err()
        );
        assert_eq!(
            read_response(&mut BufReader::new(&b"{}\n"[..]))
                .await
                .unwrap(),
            b"{}\n"
        );
    }

    #[tokio::test]
    async fn progress_order_identity_and_cleanup_knowledge_are_bound() {
        let work = diagram_work();
        let mut callback_count = 0;
        let frame = read_completion(
            &mut BufReader::new(&frames(&["running", "error"])[..]),
            &work,
            &mut |wait_ms| {
                assert_eq!(wait_ms, 17);
                callback_count += 1;
                Ok(())
            },
            Arc::new(AtomicBool::new(false)),
        )
        .await
        .unwrap();
        assert_eq!(callback_count, 1);
        assert!(frame.cleanup_confirmed);
        for sequence in [
            vec!["running", "running", "error"],
            vec!["wrong_id", "error"],
            vec!["wrong_release", "error"],
            vec!["bad_wait", "error"],
            vec!["running"],
        ] {
            assert!(
                read_completion(
                    &mut BufReader::new(&frames(&sequence)[..]),
                    &work,
                    &mut |_| Ok(()),
                    Arc::new(AtomicBool::new(false)),
                )
                .await
                .is_err(),
                "{sequence:?}"
            );
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let result = read_completion(
            &mut BufReader::new(&frames(&["running", "unconfirmed"])[..]),
            &work,
            &mut |_| Err("receipt unavailable".into()),
            cancel.clone(),
        )
        .await
        .unwrap();
        assert!(cancel.load(Ordering::Acquire));
        assert!(!result.cleanup_confirmed);
        assert!(
            result
                .progress_error
                .unwrap()
                .contains("receipt unavailable")
        );
    }
}
