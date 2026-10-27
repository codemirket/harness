//! One connection-owned warm embedding supervisor. Request results are not
//! cleanup evidence: resident admission lasts until the native leaf is reaped.
use crate::{
    embedding_job::{self, Input, Output, WarmBody, WarmInference, WarmInit, WarmReply},
    leases::Admission,
    owned_scratch::{OwnedScratch, explicit_path},
    process::{OwnedChild, cancelled, drain_stderr},
};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    sync::{mpsc, oneshot, watch},
    time::Instant,
};
const FRAME: usize = 2 * 1024 * 1024;
const IDLE: Duration = Duration::from_secs(60);
const CLEANUP: Duration = Duration::from_secs(12);
const WRITE: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(25);
// Filesystem ownership probes are more expensive than waiting on the pipes.
// Requests and EOF still wake immediately; idle demand/pressure inspection is
// bounded by this interval plus scheduling and native cleanup.
const IDLE_INSPECTION: Duration = Duration::from_millis(250);

pub(crate) struct SessionConfig {
    pub state_root: PathBuf,
    pub release_id: String,
    pub session_id: String,
    pub package_root: PathBuf,
    pub scratch: OwnedScratch,
    pub current_exe: PathBuf,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Inference {
    pub job_id: String,
    pub input_revision: String,
    pub input: Input,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DispatchState {
    NotStarted,
    MayHaveStarted,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StopReason {
    Requested,
    ParentGone,
    IdleExpired,
    WaitingDemand,
    Pressure,
    Cancelled,
    Deadline,
    Protocol,
    LeafFailure,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StopReport {
    pub reason: StopReason,
    pub cleanup_confirmed: bool,
    pub message: Option<String>,
}
#[derive(Clone, Debug)]
pub(crate) struct SessionError {
    pub message: String,
    pub dispatch: DispatchState,
    pub stopped: Option<StopReport>,
}
impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for SessionError {}
#[derive(Clone, Debug)]
pub(crate) enum SessionStatus {
    Live,
    Stopped(StopReport),
    Unknown,
}
struct Call {
    request: Inference,
    cancel: Arc<AtomicBool>,
    deadline: Instant,
    reply: oneshot::Sender<Result<Output, SessionError>>,
}
pub(crate) struct SessionHandle {
    calls: mpsc::Sender<Call>,
    stop: Arc<AtomicBool>,
    status: watch::Receiver<SessionStatus>,
}
struct CancelDroppedFuture(Option<Arc<AtomicBool>>);
impl Drop for CancelDroppedFuture {
    fn drop(&mut self) {
        if let Some(flag) = &self.0 {
            flag.store(true, Ordering::Release);
        }
    }
}
fn error(message: impl Into<String>, dispatch: DispatchState) -> SessionError {
    SessionError {
        message: message.into(),
        dispatch,
        stopped: None,
    }
}
impl SessionHandle {
    pub async fn start(
        config: SessionConfig,
        cancel: Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<Self, SessionError> {
        let (calls, rx) = mpsc::channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let mut on_drop = CancelDroppedFuture(Some(stop.clone()));
        let (status_tx, status) = watch::channel(SessionStatus::Unknown);
        let (ready, started) = oneshot::channel();
        tokio::spawn(parent_owner(
            config,
            rx,
            stop.clone(),
            status_tx,
            ready,
            cancel,
            deadline,
        ));
        started.await.map_err(|_| {
            error(
                "embedding session owner ended during startup",
                DispatchState::NotStarted,
            )
        })??;
        on_drop.0 = None;
        Ok(Self {
            calls,
            stop,
            status,
        })
    }
    pub async fn infer(
        &self,
        request: Inference,
        cancel: Arc<AtomicBool>,
        deadline: Instant,
    ) -> Result<Output, SessionError> {
        if let SessionStatus::Stopped(stopped) = self.status() {
            return Err(SessionError {
                message: "embedding session already stopped".into(),
                dispatch: DispatchState::NotStarted,
                stopped: Some(stopped),
            });
        }
        if self.stop.load(Ordering::Acquire)
            || cancel.load(Ordering::Acquire)
            || Instant::now() >= deadline
        {
            return Err(error(
                "embedding session retiring or request expired/cancelled",
                DispatchState::NotStarted,
            ));
        }
        let (reply, result) = oneshot::channel();
        let mut on_drop = CancelDroppedFuture(Some(cancel.clone()));
        self.calls
            .try_send(Call {
                request,
                cancel,
                deadline,
                reply,
            })
            .map_err(|_| {
                error(
                    "embedding session is busy or stopped",
                    DispatchState::NotStarted,
                )
            })?;
        let result = match tokio::time::timeout_at(deadline, result).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(error(
                "embedding session request ownership ended",
                DispatchState::MayHaveStarted,
            )),
            Err(_) => {
                // Stop admission synchronously, then allow only the bounded
                // cleanup envelope beyond the absolute execution deadline.
                self.request_shutdown();
                let stopped = self.shutdown().await;
                Err(SessionError {
                    message: "embedding request deadline elapsed".into(),
                    dispatch: DispatchState::MayHaveStarted,
                    stopped: Some(stopped),
                })
            }
        };
        on_drop.0 = None;
        result
    }
    pub fn status(&self) -> SessionStatus {
        self.status.borrow().clone()
    }
    pub async fn stopped(&self) -> StopReport {
        let mut status = self.status.clone();
        loop {
            if let SessionStatus::Stopped(report) = status.borrow().clone() {
                return report;
            }
            if status.changed().await.is_err() {
                return unknown_stop("embedding owner ended without stop proof");
            }
        }
    }
    pub fn request_shutdown(&self) {
        self.stop.store(true, Ordering::Release);
    }
    pub async fn shutdown(&self) -> StopReport {
        self.request_shutdown();
        tokio::time::timeout(CLEANUP, self.stopped())
            .await
            .unwrap_or_else(|_| {
                unknown_stop("embedding cleanup deadline elapsed; ownership task continues")
            })
    }
}
impl Drop for SessionHandle {
    fn drop(&mut self) {
        self.request_shutdown();
    }
}
fn unknown_stop(message: impl Into<String>) -> StopReport {
    StopReport {
        reason: StopReason::Protocol,
        cleanup_confirmed: false,
        message: Some(message.into()),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigWire {
    protocol_version: u8,
    state_root: PathBuf,
    release_id: String,
    session_id: String,
    package_root: PathBuf,
    scratch_dir: PathBuf,
    remaining_ms: u64,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InferWire {
    protocol_version: u8,
    release_id: String,
    session_id: String,
    remaining_ms: u64,
    request: WarmInference,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum SupervisorReply {
    Ready {
        protocol_version: u8,
        release_id: String,
        session_id: String,
    },
    Result {
        protocol_version: u8,
        release_id: String,
        session_id: String,
        sequence: u64,
        job_id: String,
        output: Output,
    },
    Stopped {
        protocol_version: u8,
        release_id: String,
        session_id: String,
        report: StopReport,
    },
}
fn remaining(deadline: Instant) -> Result<u64, String> {
    let ms = deadline
        .saturating_duration_since(Instant::now())
        .as_millis()
        .min(60_000) as u64;
    if ms == 0 {
        Err("embedding deadline exhausted".into())
    } else {
        Ok(ms)
    }
}
fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|_| "embedding frame encoding failed")?;
    bytes.push(b'\n');
    if bytes.len() > FRAME {
        Err("embedding frame limit exceeded".into())
    } else {
        Ok(bytes)
    }
}
async fn write<T: Serialize>(pipe: &mut ChildStdin, value: &T) -> Result<(), String> {
    let bytes = encode(value)?;
    tokio::time::timeout(WRITE, pipe.write_all(&bytes))
        .await
        .map_err(|_| "embedding write deadline exceeded")?
        .map_err(|_| "embedding pipe write failed".into())
}
type Frames = mpsc::Receiver<Result<Vec<u8>, String>>;
fn frames(stdout: tokio::process::ChildStdout) -> Frames {
    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        loop {
            let frame = async {
                let mut frame = Vec::new();
                loop {
                    let available = reader
                        .fill_buf()
                        .await
                        .map_err(|_| "embedding output read failed".to_owned())?;
                    if available.is_empty() {
                        return Err(if frame.is_empty() {
                            "embedding output EOF"
                        } else {
                            "embedding output truncated frame"
                        }
                        .to_owned());
                    }
                    let newline = available.iter().position(|b| *b == b'\n');
                    let count = newline.map_or(available.len(), |i| i + 1);
                    if frame.len() + count > FRAME {
                        return Err("embedding output exceeded limit".to_owned());
                    }
                    frame.extend_from_slice(&available[..count]);
                    reader.consume(count);
                    if newline.is_some() {
                        return Ok(frame);
                    }
                }
            }
            .await;
            let failed = frame.is_err();
            if tx.send(frame).await.is_err() || failed {
                break;
            }
        }
    });
    rx
}
async fn next(frames: &mut Frames) -> Result<Vec<u8>, String> {
    frames
        .recv()
        .await
        .ok_or_else(|| "embedding output closed".to_owned())?
}
fn checked_reply(bytes: &[u8], config: &ConfigWire) -> Result<SupervisorReply, String> {
    let reply: SupervisorReply =
        serde_json::from_slice(bytes).map_err(|_| "embedding supervisor reply invalid")?;
    let (version, release, session) = match &reply {
        SupervisorReply::Ready {
            protocol_version,
            release_id,
            session_id,
        }
        | SupervisorReply::Result {
            protocol_version,
            release_id,
            session_id,
            ..
        }
        | SupervisorReply::Stopped {
            protocol_version,
            release_id,
            session_id,
            ..
        } => (protocol_version, release_id, session_id),
    };
    if *version != 2 || release != &config.release_id || session != &config.session_id {
        return Err("embedding supervisor identity mismatch".into());
    }
    if let SupervisorReply::Stopped { report, .. } = &reply
        && report
            .message
            .as_ref()
            .is_some_and(|message| message.len() > 8192)
    {
        return Err("embedding stop message exceeds limit".into());
    }
    Ok(reply)
}

async fn parent_owner(
    config: SessionConfig,
    mut calls: mpsc::Receiver<Call>,
    stop: Arc<AtomicBool>,
    status: watch::Sender<SessionStatus>,
    ready: oneshot::Sender<Result<(), SessionError>>,
    cancel: Arc<AtomicBool>,
    deadline: Instant,
) {
    let mut ready = Some(ready);
    let mut failed_call = None;
    let mut startup_error = None;
    let mut late_ready = true;
    let mut late_result: Option<WarmInference> = None;
    let wire = ConfigWire {
        protocol_version: 2,
        state_root: config.state_root.clone(),
        release_id: config.release_id.clone(),
        session_id: config.session_id.clone(),
        package_root: config.package_root.clone(),
        scratch_dir: config.scratch.path.clone(),
        remaining_ms: remaining(deadline).unwrap_or(0),
    };
    let launch = (|| -> Result<Child, String> {
        if !cfg!(unix) {
            return Err("warm embedding ownership unsupported on this platform".into());
        }
        if cancel.load(Ordering::Acquire) || wire.remaining_ms == 0 {
            return Err("embedding cancelled or expired before launch".into());
        }
        explicit_path(&config.current_exe, false)?;
        explicit_path(&config.package_root, true)?;
        crate::process::validate_job_dir(&wire.scratch_dir)?;
        let mut command = Command::new(&config.current_exe);
        command
            .arg("__embedding_supervisor")
            .env_clear()
            .env("TMPDIR", &wire.scratch_dir)
            .current_dir(&wire.scratch_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        command
            .spawn()
            .map_err(|_| "embedding supervisor launch failed".into())
    })();
    let mut child = match launch {
        Ok(child) => child,
        Err(message) => {
            let removal = config.scratch.remove_empty();
            let report = StopReport {
                reason: StopReason::LeafFailure,
                cleanup_confirmed: removal.is_ok(),
                message: removal.err(),
            };
            let _ = status.send(SessionStatus::Stopped(report.clone()));
            let _ = ready.take().unwrap().send(Err(SessionError {
                message,
                dispatch: DispatchState::NotStarted,
                stopped: Some(report),
            }));
            return;
        }
    };
    let mut pipe = child.stdin.take();
    let mut replies = frames(child.stdout.take().expect("piped stdout"));
    let mut terminal = None;
    let startup = async {
        write(pipe.as_mut().unwrap(), &wire).await?;
        match checked_reply(&next(&mut replies).await?, &wire)? {
            SupervisorReply::Ready { .. } => Ok(()),
            stopped @ SupervisorReply::Stopped { .. } => {
                let message = match &stopped {
                    SupervisorReply::Stopped { report, .. } => report
                        .message
                        .clone()
                        .unwrap_or_else(|| "embedding session stopped during startup".into()),
                    _ => unreachable!(),
                };
                terminal = Some(stopped);
                Err(message)
            }
            _ => Err("embedding startup reply out of order".into()),
        }
    };
    let startup = tokio::select! {biased; _=cancelled(cancel)=>Err("embedding startup cancelled".into()), _=cancelled(stop.clone())=>Err("embedding startup stopped".into()), result=tokio::time::timeout_at(deadline,startup)=>result.unwrap_or_else(|_|Err("embedding startup deadline exceeded".into()))};
    if let Err(message) = startup {
        startup_error = Some(message);
    } else {
        late_ready = false;
        let _ = status.send(SessionStatus::Live);
        if ready.take().unwrap().send(Ok(())).is_err() {
            stop.store(true, Ordering::Release);
        }
        let mut sequence = 0u64;
        loop {
            let call = tokio::select! {biased; _=cancelled(stop.clone())=>break, frame=next(&mut replies)=>{terminal=frame.and_then(|b|checked_reply(&b,&wire)).ok();break;}, call=calls.recv()=>match call{Some(call)=>call,None=>break}};
            if call.cancel.load(Ordering::Acquire) || remaining(call.deadline).is_err() {
                let _ = call.reply.send(Err(error(
                    "embedding cancelled or expired before dispatch",
                    DispatchState::NotStarted,
                )));
                continue;
            }
            let next_sequence = match sequence.checked_add(1) {
                Some(v) => v,
                None => {
                    let _ = call.reply.send(Err(error(
                        "embedding sequence exhausted",
                        DispatchState::NotStarted,
                    )));
                    break;
                }
            };
            let request = WarmInference {
                sequence: next_sequence,
                job_id: call.request.job_id,
                input_revision: call.request.input_revision,
                input: call.request.input,
            };
            if let Err(message) = embedding_job::validate_warm_input(&request) {
                let _ = call
                    .reply
                    .send(Err(error(message, DispatchState::NotStarted)));
                continue;
            }
            sequence = next_sequence;
            let frame = InferWire {
                protocol_version: 2,
                release_id: wire.release_id.clone(),
                session_id: wire.session_id.clone(),
                remaining_ms: remaining(call.deadline).unwrap_or(0),
                request: request.clone(),
            };
            let work = async {
                write(pipe.as_mut().unwrap(), &frame).await?;
                match checked_reply(&next(&mut replies).await?, &wire)? {
                    SupervisorReply::Result {
                        sequence: got,
                        job_id,
                        output,
                        ..
                    } if got == sequence && job_id == request.job_id => {
                        embedding_job::validate_warm_output(&request, &output)?;
                        Ok(output)
                    }
                    SupervisorReply::Stopped { report, .. } => {
                        terminal = Some(SupervisorReply::Stopped {
                            protocol_version: 2,
                            release_id: wire.release_id.clone(),
                            session_id: wire.session_id.clone(),
                            report,
                        });
                        Err("embedding session stopped during inference".into())
                    }
                    _ => Err("embedding result out of order".into()),
                }
            };
            late_result = Some(request.clone());
            let result = tokio::select! {biased; _=cancelled(call.cancel)=>Err("embedding inference cancelled".into()), _=cancelled(stop.clone())=>Err("embedding session shutdown".into()), result=tokio::time::timeout_at(call.deadline,work)=>result.unwrap_or_else(|_|Err("embedding inference deadline exceeded".into()))};
            match result {
                Ok(output) => {
                    late_result = None;
                    let _ = call.reply.send(Ok(output));
                }
                Err(message) => {
                    failed_call = Some((call.reply, message));
                    break;
                }
            }
            // No completed request cancellation token is retained in idle state.
        }
    }
    // Retirement closes admission immediately, before waiting for process exit.
    calls.close();
    while let Ok(call) = calls.try_recv() {
        let _ = call.reply.send(Err(error(
            "embedding session is retiring",
            DispatchState::NotStarted,
        )));
    }
    drop(pipe.take());
    let cleanup = async {
        let mut reported = match terminal {
            Some(SupervisorReply::Stopped { report, .. }) => Some(Ok(report)),
            Some(_) => Some(Err("embedding unexpected idle response".to_owned())),
            None => None,
        };
        // Cancellation can race a legitimate readiness/result already in the
        // pipe. Discard at most that exact bound frame, then require Stopped.
        while reported.is_none() {
            reported = match next(&mut replies)
                .await
                .and_then(|b| checked_reply(&b, &wire))
            {
                Ok(SupervisorReply::Stopped { report, .. }) => Some(Ok(report)),
                Ok(SupervisorReply::Ready { .. }) if late_ready => {
                    late_ready = false;
                    None
                }
                Ok(SupervisorReply::Result {
                    sequence,
                    job_id,
                    output,
                    ..
                }) if late_result
                    .as_ref()
                    .is_some_and(|r| r.sequence == sequence && r.job_id == job_id) =>
                {
                    let expected = late_result.take().unwrap();
                    match embedding_job::validate_warm_output(&expected, &output) {
                        Ok(()) => None,
                        Err(e) => Some(Err(e)),
                    }
                }
                Ok(_) => Some(Err("embedding stop reply out of order".to_owned())),
                Err(e) => Some(Err(e)),
            };
        }
        let exit = child
            .wait()
            .await
            .map_err(|_| "embedding supervisor reap failed")?;
        if !exit.success() {
            return Err("embedding supervisor exit unconfirmed".into());
        }
        // Reader reports EOF only after the finite supervisor has actually exited.
        match replies.recv().await {
            Some(Err(e)) if e == "embedding output EOF" => (),
            _ => return Err("embedding supervisor trailing output or unconfirmed EOF".into()),
        }
        reported.unwrap()
    };
    tokio::pin!(cleanup);
    let report = tokio::select! {
        report = &mut cleanup => report,
        _ = tokio::time::sleep(CLEANUP) => {
            let report = unknown_stop("embedding cleanup deadline elapsed; owner continues");
            let _ = status.send(SessionStatus::Stopped(report.clone()));
            if let Some(sender) = ready.take() {
                let _ = sender.send(Err(SessionError {
                    message: startup_error.take().unwrap_or_else(|| "embedding startup cleanup unknown".into()),
                    dispatch: DispatchState::NotStarted,
                    stopped: Some(report.clone()),
                }));
            }
            if let Some((sender, message)) = failed_call.take() {
                let _ = sender.send(Err(SessionError {
                    message, dispatch: DispatchState::MayHaveStarted, stopped: Some(report),
                }));
            }
            cleanup.await
        }
    };
    let mut report = report.unwrap_or_else(unknown_stop);
    if report.cleanup_confirmed
        && let Err(message) = config.scratch.remove_empty()
    {
        report.cleanup_confirmed = false;
        report.message = Some(format!("owned scratch retained: {message}"));
    }
    let _ = status.send(SessionStatus::Stopped(report.clone()));
    if let Some(sender) = ready.take() {
        let _ = sender.send(Err(SessionError {
            message: startup_error.unwrap_or_else(|| "embedding startup stopped".into()),
            dispatch: DispatchState::NotStarted,
            stopped: Some(report.clone()),
        }));
    }
    if let Some((sender, message)) = failed_call {
        let _ = sender.send(Err(SessionError {
            message,
            dispatch: DispatchState::MayHaveStarted,
            stopped: Some(report),
        }));
    }
}

#[cfg(unix)]
fn parent_frames(mut input: crate::worker::Lifeline) -> Frames {
    let (tx, rx) = mpsc::channel(1);
    tokio::spawn(async move {
        let mut frame = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let count = match input.read(&mut buffer).await {
                Ok(0) => {
                    let _ = tx.send(Err("parent closed".into())).await;
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                    break;
                }
            };
            for byte in &buffer[..count] {
                frame.push(*byte);
                if frame.len() > FRAME {
                    let _ = tx.send(Err("parent frame exceeded limit".into())).await;
                    return;
                }
                if *byte == b'\n' && tx.send(Ok(std::mem::take(&mut frame))).await.is_err() {
                    return;
                }
            }
        }
    });
    rx
}
#[cfg(unix)]
async fn reply(pipe: &mut crate::worker::Lifeline, value: &SupervisorReply) -> Result<(), String> {
    let bytes = encode(value)?;
    tokio::time::timeout(WRITE, pipe.write_all(&bytes))
        .await
        .map_err(|_| "supervisor response deadline")?
}
fn stop_reply(config: &ConfigWire, report: StopReport) -> SupervisorReply {
    SupervisorReply::Stopped {
        protocol_version: 2,
        release_id: config.release_id.clone(),
        session_id: config.session_id.clone(),
        report,
    }
}
fn warm_reply(bytes: &[u8], config: &ConfigWire) -> Result<WarmBody, String> {
    let reply: WarmReply =
        serde_json::from_slice(bytes).map_err(|_| "native embedding reply invalid")?;
    if reply.protocol_version != 2
        || reply.release_id != config.release_id
        || reply.session_id != config.session_id
    {
        return Err("native embedding identity mismatch".into());
    }
    Ok(reply.body)
}

/// Responsive same-binary supervisor; it never initializes ORT itself.
pub async fn serve_supervisor() -> Result<(), String> {
    #[cfg(not(unix))]
    {
        Err("warm embedding ownership unsupported on this platform".into())
    }
    #[cfg(unix)]
    {
        let mut parent = parent_frames(crate::worker::Lifeline::open(libc::STDIN_FILENO)?);
        let mut stdout = crate::worker::Lifeline::open(libc::STDOUT_FILENO)?;
        let bytes = tokio::time::timeout(WRITE, next(&mut parent))
            .await
            .map_err(|_| "session initialization deadline")??;
        let config: ConfigWire =
            serde_json::from_slice(&bytes).map_err(|_| "session initialization invalid")?;
        if config.protocol_version != 2 || config.remaining_ms == 0 || config.remaining_ms > 60_000
        {
            return Err("session initialization invalid".into());
        }
        crate::release::ReleaseId::parse(&config.release_id)
            .map_err(|_| "invalid session release")?;
        crate::release::ReleaseId::parse(&config.session_id)
            .map_err(|_| "invalid session identity")?;
        let preflight = (|| -> Result<_, String> {
            let maintenance = crate::maintenance::SessionGuard::acquire(&config.state_root)
                .map_err(|e| e.to_string())?;
            crate::process::validate_job_dir(&config.scratch_dir)?;
            explicit_path(&config.package_root, true)?;
            let pressure = pressure::Monitor::new(&config.state_root)?;
            let admission = Admission::open(&config.state_root).map_err(|e| e.to_string())?;
            Ok((maintenance, pressure, admission))
        })();
        let (_maintenance, mut pressure, admission) = match preflight {
            Ok(value) => value,
            Err(message) => {
                let _ = reply(
                    &mut stdout,
                    &stop_reply(
                        &config,
                        StopReport {
                            reason: StopReason::Pressure,
                            cleanup_confirmed: true,
                            message: Some(message),
                        },
                    ),
                )
                .await;
                return Ok(());
            }
        };
        let deadline = Instant::now() + Duration::from_millis(config.remaining_ms);
        let mut waiting = None;
        let lease = loop {
            if Instant::now() >= deadline {
                break Err((
                    StopReason::Deadline,
                    "embedding admission deadline".to_owned(),
                ));
            }
            // Admission samples installation memory and OS pressure before
            // taking a slot. Do not rescan the tree while capacity is full.
            match crate::resource_admission::admit(&config.state_root) {
                Ok(lease) => {
                    if Instant::now() >= deadline {
                        drop(lease);
                        break Err((
                            StopReason::Deadline,
                            "embedding resource inspection deadline".into(),
                        ));
                    }
                    break Ok(lease);
                }
                Err(crate::resource_admission::Error::Busy) => (),
                Err(crate::resource_admission::Error::Deferred(sample)) => {
                    break Err((
                        StopReason::Pressure,
                        format!("memory resources defer embedding load: {sample:?}"),
                    ));
                }
                Err(e) => break Err((StopReason::Protocol, e.to_string())),
            }
            if waiting.is_none() {
                match admission.try_waiting() {
                    Ok(permit) => waiting = Some(permit),
                    Err(e) => break Err((StopReason::Protocol, e.to_string())),
                }
            }
            tokio::select! {biased; _=next(&mut parent)=>break Err((StopReason::ParentGone,"parent closed during admission".into())), _=tokio::time::sleep(POLL)=>()}
        };
        drop(waiting);
        let mut lease = match lease {
            Ok(lease) => lease,
            Err((reason, message)) => {
                let _ = reply(
                    &mut stdout,
                    &stop_reply(
                        &config,
                        StopReport {
                            reason,
                            cleanup_confirmed: true,
                            message: Some(message),
                        },
                    ),
                )
                .await;
                return Ok(());
            }
        };
        // resource_admission::admit just sampled pressure and aggregate RSS;
        // there is no await between taking that slot and launching its leaf.
        lease
            .mark_unresolved(&config.session_id)
            .map_err(|e| e.to_string())?;
        let executable = std::env::current_exe().map_err(|_| "embedding executable unavailable")?;
        let mut command = Command::new(executable);
        command
            .arg("__embedding_leaf")
            .env_clear()
            .env("ORT_DISABLE_TELEMETRY", "1")
            .env("TMPDIR", &config.scratch_dir)
            .env("TMP", &config.scratch_dir)
            .env("TEMP", &config.scratch_dir)
            .current_dir(&config.scratch_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        let spawned = command.spawn();
        let child = match spawned {
            Ok(child) => child,
            Err(_) => {
                let cleaned = lease.clear_after_cleanup().is_ok();
                let _ = reply(
                    &mut stdout,
                    &stop_reply(
                        &config,
                        StopReport {
                            reason: StopReason::LeafFailure,
                            cleanup_confirmed: cleaned,
                            message: Some("embedding leaf launch failed".into()),
                        },
                    ),
                )
                .await;
                return Ok(());
            }
        };
        let mut owned = OwnedChild::new(child)?;
        let mut stdin = owned.child.stdin.take();
        let mut leaves = frames(owned.child.stdout.take().ok_or("leaf output absent")?);
        let stderr = owned.child.stderr.take().ok_or("leaf diagnostics absent")?;
        let diagnostics_failed = Arc::new(AtomicBool::new(false));
        let failed = diagnostics_failed.clone();
        let diagnostics = tokio::spawn(async move {
            let result = drain_stderr(stderr).await;
            if result.is_err() {
                failed.store(true, Ordering::Release);
            }
            result
        });
        let init = WarmInit {
            protocol_version: 2,
            release_id: config.release_id.clone(),
            session_id: config.session_id.clone(),
            package_root: config.package_root.clone(),
        };
        let startup = async {
            write(stdin.as_mut().unwrap(), &init).await?;
            match warm_reply(&next(&mut leaves).await?, &config)? {
                WarmBody::Ready { model }
                    if model
                        == crate::embedding::expected_model_identity()
                            .map_err(|e| e.code().to_owned())? =>
                {
                    Ok(())
                }
                WarmBody::Failed {
                    sequence: 0,
                    job_id,
                    code,
                } if job_id.is_empty()
                    && !code.is_empty()
                    && code.len() <= 128
                    && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') =>
                {
                    Err(format!("embedding failed ({code})"))
                }
                _ => Err("embedding native readiness mismatch".into()),
            }
        };
        let started = tokio::select! {biased; _=next(&mut parent)=>Err((StopReason::ParentGone,"parent closed during model load".to_owned())), _=cancelled(diagnostics_failed.clone())=>Err((StopReason::Protocol,"embedding diagnostics overflow".into())), result=tokio::time::timeout_at(deadline,startup)=>match result{Ok(Ok(()))=>Ok(()),Ok(Err(e))=>Err((StopReason::LeafFailure,e)),Err(_)=>Err((StopReason::Deadline,"embedding load deadline".into()))}};
        let (reason, message) = match started {
            Err(stop) => stop,
            Ok(()) => {
                let ready = SupervisorReply::Ready {
                    protocol_version: 2,
                    release_id: config.release_id.clone(),
                    session_id: config.session_id.clone(),
                };
                if reply(&mut stdout, &ready).await.is_err() {
                    (
                        StopReason::ParentGone,
                        "parent readiness delivery failed".into(),
                    )
                } else {
                    supervisor_loop(
                        &config,
                        &admission,
                        &mut pressure,
                        &mut parent,
                        &mut stdout,
                        stdin.as_mut().unwrap(),
                        &mut leaves,
                        diagnostics_failed,
                        deadline,
                        IDLE,
                    )
                    .await
                }
            }
        };
        // Closing stdin allows a completed or failed leaf to exit without racing
        // a signal against its ordinary exit. Busy native work is forcibly reaped.
        drop(stdin.take());
        let grace =
            if reason == StopReason::LeafFailure && message.starts_with("embedding failed (") {
                // A well-formed failure is the leaf's terminal reply. Let its finite
                // exit finish rather than race a signal against an exiting process.
                Duration::from_secs(5)
            } else {
                Duration::from_millis(100)
            };
        let graceful = tokio::time::timeout(grace, owned.wait()).await;
        let cleanup = match graceful {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => owned.terminate_and_reap().await,
        };
        let mut report = StopReport {
            reason,
            cleanup_confirmed: cleanup.is_ok(),
            message: Some(message),
        };
        if let Err(e) = cleanup {
            report.message = Some(format!("embedding cleanup unconfirmed: {e}"));
        }
        if report.cleanup_confirmed {
            match tokio::time::timeout(WRITE, diagnostics).await {
                Ok(Ok(Ok(_))) => (),
                _ => report.message = Some("embedding diagnostics drain failed after reap".into()),
            }
            if let Err(e) = lease.clear_after_cleanup() {
                report.cleanup_confirmed = false;
                report.message = Some(format!("embedding marker cleanup unconfirmed: {e}"));
            }
        }
        let _ = reply(&mut stdout, &stop_reply(&config, report)).await;
        Ok(())
    }
}
trait Pressure {
    fn elevated(&mut self) -> Result<Option<bool>, String>;
    fn before_dispatch(&mut self) -> Result<Option<bool>, String>;
}
impl Pressure for pressure::Monitor {
    fn elevated(&mut self) -> Result<Option<bool>, String> {
        self.elevated()
    }
    fn before_dispatch(&mut self) -> Result<Option<bool>, String> {
        self.before_dispatch()
    }
}
#[cfg(unix)]
#[allow(clippy::too_many_arguments)]
async fn supervisor_loop(
    config: &ConfigWire,
    admission: &Admission,
    pressure: &mut impl Pressure,
    parent: &mut Frames,
    stdout: &mut crate::worker::Lifeline,
    stdin: &mut ChildStdin,
    leaves: &mut Frames,
    diagnostics_failed: Arc<AtomicBool>,
    first_deadline: Instant,
    idle_budget: Duration,
) -> (StopReason, String) {
    let mut sequence = 1u64;
    let mut idle_until = first_deadline;
    loop {
        if sequence > 1 {
            match pressure.elevated() {
                Ok(Some(false) | None) => (),
                Ok(Some(true)) => {
                    return (
                        StopReason::Pressure,
                        "idle model retired under memory pressure".into(),
                    );
                }
                Err(e) => return (StopReason::Pressure, e),
            }
            match admission.has_waiters() {
                Ok(true) => {
                    return (
                        StopReason::WaitingDemand,
                        "idle model retired for waiting work".into(),
                    );
                }
                Ok(false) => (),
                Err(e) => return (StopReason::Protocol, e.to_string()),
            }
        }
        if Instant::now() >= idle_until {
            return (
                if sequence == 1 {
                    StopReason::Deadline
                } else {
                    StopReason::IdleExpired
                },
                "embedding idle or first-dispatch deadline".into(),
            );
        }
        let bytes = tokio::select! {biased; _=cancelled(diagnostics_failed.clone())=>return (StopReason::Protocol,"embedding diagnostics overflow".into()), _=next(leaves)=>return (StopReason::Protocol,"unsolicited native output or exit".into()), frame=next(parent)=>match frame{Ok(bytes)=>bytes,Err(_)=>return (StopReason::ParentGone,"parent closed".into())}, _=tokio::time::sleep_until(idle_until.min(Instant::now()+IDLE_INSPECTION))=>continue};
        let frame: InferWire = match serde_json::from_slice(&bytes) {
            Ok(frame) => frame,
            Err(_) => return (StopReason::Protocol, "invalid inference frame".into()),
        };
        if frame.protocol_version != 2
            || frame.release_id != config.release_id
            || frame.session_id != config.session_id
            || frame.request.sequence != sequence
            || frame.remaining_ms == 0
            || frame.remaining_ms > 60_000
            || embedding_job::validate_warm_input(&frame.request).is_err()
        {
            return (
                StopReason::Protocol,
                "inference identity or input mismatch".into(),
            );
        }
        sequence = match sequence.checked_add(1) {
            Some(next) => next,
            None => return (StopReason::Protocol, "sequence exhausted".into()),
        };
        let deadline = Instant::now() + Duration::from_millis(frame.remaining_ms);
        let deadline = if frame.request.sequence == 1 {
            deadline.min(first_deadline)
        } else {
            deadline
        };
        loop {
            if Instant::now() >= deadline {
                return (
                    StopReason::Deadline,
                    "embedding resource inspection deadline".into(),
                );
            }
            match pressure.before_dispatch() {
                Ok(Some(false)) => {
                    // A synchronous native snapshot can consume the remaining
                    // budget. Do not let timeout_at poll a write after expiry.
                    if Instant::now() >= deadline {
                        return (
                            StopReason::Deadline,
                            "embedding resource inspection deadline".into(),
                        );
                    }
                    break;
                }
                Ok(Some(true)) => {
                    return (
                        StopReason::Pressure,
                        "embedding dispatch deferred under memory pressure".into(),
                    );
                }
                Err(error) => return (StopReason::Pressure, error),
                Ok(None) => (),
            }
            tokio::select! {
                biased;
                _ = next(parent) => return (StopReason::ParentGone, "parent closed while resource inspection was busy".into()),
                _ = next(leaves) => return (StopReason::Protocol, "unsolicited native output or exit while resource inspection was busy".into()),
                _ = cancelled(diagnostics_failed.clone()) => return (StopReason::Protocol, "embedding diagnostics overflow".into()),
                _ = tokio::time::sleep_until(deadline.min(Instant::now() + POLL)) => (),
            }
        }
        let work = async {
            write(stdin, &frame.request).await?;
            match warm_reply(&next(leaves).await?, config)? {
                WarmBody::Result {
                    sequence,
                    job_id,
                    output,
                } if sequence == frame.request.sequence && job_id == frame.request.job_id => {
                    embedding_job::validate_warm_output(&frame.request, &output)?;
                    Ok(output)
                }
                WarmBody::Failed {
                    sequence,
                    job_id,
                    code,
                } if sequence == frame.request.sequence
                    && job_id == frame.request.job_id
                    && code.len() <= 128
                    && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_') =>
                {
                    Err(format!("embedding failed ({code})"))
                }
                _ => Err("embedding native response mismatch".into()),
            }
        };
        let output = tokio::select! {biased; _=next(parent)=>return (StopReason::ParentGone,"parent closed or sent unexpected active command".into()), _=cancelled(diagnostics_failed.clone())=>return (StopReason::Protocol,"embedding diagnostics overflow".into()), result=tokio::time::timeout_at(deadline,work)=>match result{Ok(Ok(output))=>output,Ok(Err(e))=>return (StopReason::LeafFailure,e),Err(_)=>return (StopReason::Deadline,"embedding inference deadline".into())}};
        let response = SupervisorReply::Result {
            protocol_version: 2,
            release_id: config.release_id.clone(),
            session_id: config.session_id.clone(),
            sequence: frame.request.sequence,
            job_id: frame.request.job_id,
            output,
        };
        if reply(stdout, &response).await.is_err() {
            return (
                StopReason::ParentGone,
                "embedding result delivery failed".into(),
            );
        }
        idle_until = Instant::now() + idle_budget;
    }
}

#[cfg(target_os = "macos")]
#[path = "embedding_session_pressure.rs"]
mod pressure;
#[cfg(not(target_os = "macos"))]
mod pressure {
    pub(super) struct Monitor;
    impl Monitor {
        pub(super) fn new(_: &std::path::Path) -> Result<Self, String> {
            Err("embedding pressure notifications unsupported on this platform".into())
        }
        pub(super) fn elevated(&mut self) -> Result<Option<bool>, String> {
            Err("pressure unsupported".into())
        }
        pub(super) fn before_dispatch(&mut self) -> Result<Option<bool>, String> {
            Err("pressure unsupported".into())
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::leases::AdmissionError;
    use crate::owned_scratch::DirectoryStamp;
    use std::{fs, os::unix::fs::PermissionsExt, sync::atomic::AtomicU64};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        root: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mirket-warm-session-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            let root = root.canonicalize().unwrap();
            for name in ["state", "package"] {
                fs::create_dir(root.join(name)).unwrap();
                fs::set_permissions(root.join(name), fs::Permissions::from_mode(0o700)).unwrap();
            }
            Self { root }
        }
        fn config(&self, mode: &str) -> Option<SessionConfig> {
            let node = std::env::var_os("MIRKET_TEST_NODE").map(PathBuf::from);
            let Some(node) = node else {
                eprintln!("SKIP native fake session: explicit MIRKET_TEST_NODE required");
                return None;
            };
            assert!(node.is_absolute());
            let script = r#"const rl=require('node:readline').createInterface({input:process.stdin});
const crypto=require('node:crypto');const model=__MODEL__;const mode=__MODE__;let config=null,count=0,done=false;
function send(value){process.stdout.write(JSON.stringify(value)+'\n');}
function stop(reason='requested'){if(done)return;done=true;const result={kind:'stopped',protocol_version:2,release_id:config.release_id,session_id:config.session_id,report:{reason,cleanup_confirmed:true,message:null}};process.stdout.write(JSON.stringify(result)+'\n',()=>{if(mode==='trailing')process.stdout.write('{');setTimeout(()=>process.exit(0),(mode==='idle_hang'||mode==='deadline_hang')?600:0);});}
rl.on('line',line=>{const value=JSON.parse(line);if(!config){config=value;if(mode==='refused'){stop('pressure');return;}if(mode==='slow_start')return;send({kind:'ready',protocol_version:2,release_id:config.release_id,session_id:config.session_id});if(mode==='idle'||mode==='idle_hang')setTimeout(()=>stop('idle_expired'),150);return;}
if(mode==='hang'||mode==='deadline_hang')return;count++;const request=value.request;const text=request.input.query;const vector=Array(model.dimensions).fill(0);vector[0]=1;const output={schema:1,input_revision:request.input_revision,query:{model_identity:model,source_sha256:crypto.createHash('sha256').update(text).digest('hex'),input_bytes:Buffer.byteLength(text),input_tokens:count+2,vector},passages:[]};const result={kind:'result',protocol_version:2,release_id:config.release_id,session_id:config.session_id,sequence:request.sequence+(mode==='wrong_sequence'?1:0),job_id:request.job_id,output};if(mode==='late_result'){globalThis.late=result;return;}send(result);});rl.on('close',()=>{if(globalThis.late)send(globalThis.late);if(mode==='slow_start')send({kind:'ready',protocol_version:2,release_id:config.release_id,session_id:config.session_id});if(config)stop();else process.exit(0);});
"#;
            let source = format!(
                "#!{}\n{}",
                node.display(),
                script
                    .replace(
                        "__MODEL__",
                        &serde_json::to_string(
                            &crate::embedding::expected_model_identity().unwrap()
                        )
                        .unwrap()
                    )
                    .replace("__MODE__", &serde_json::to_string(mode).unwrap())
            );
            let executable = self.root.join("fake-supervisor");
            fs::write(&executable, source).unwrap();
            fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
            let state = self.root.join("state");
            let stamp = DirectoryStamp::inspect(&state).unwrap();
            Some(SessionConfig {
                state_root: state.clone(),
                release_id: "warm-test".into(),
                session_id: "session-test".into(),
                package_root: self.root.join("package"),
                scratch: OwnedScratch::create(&state, &stamp, "session-test").unwrap(),
                current_exe: executable,
            })
        }
        fn empty(&self) -> bool {
            let jobs = self.root.join("state/jobs");
            !jobs.exists() || fs::read_dir(jobs).unwrap().next().is_none()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            if self.empty() {
                let _ = fs::remove_dir_all(&self.root);
            }
        }
    }
    fn request() -> Inference {
        let input = Input {
            query: Some("fixture İstanbul".into()),
            passages: vec![],
        };
        Inference {
            job_id: "query-test".into(),
            input_revision: crate::engine::revision(&serde_json::to_vec(&input).unwrap()),
            input,
        }
    }
    fn flag() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }
    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(5)
    }
    #[tokio::test(flavor = "current_thread")]
    async fn same_session_results_do_not_mean_stop_and_completed_cancel_does_not_kill_idle() {
        let f = Fixture::new();
        let Some(config) = f.config("normal") else {
            return;
        };
        let handle = SessionHandle::start(config, flag(), deadline())
            .await
            .unwrap();
        let cancel = flag();
        let first = handle
            .infer(request(), cancel.clone(), deadline())
            .await
            .unwrap();
        cancel.store(true, Ordering::Release);
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(matches!(handle.status(), SessionStatus::Live));
        let second = handle.infer(request(), flag(), deadline()).await.unwrap();
        assert_eq!(first.query.unwrap().input_tokens, 3);
        assert_eq!(second.query.unwrap().input_tokens, 4);
        assert!(!f.empty());
        assert!(handle.shutdown().await.cleanup_confirmed);
        assert!(f.empty());
    }
    #[tokio::test(flavor = "current_thread")]
    async fn autonomous_idle_stop_reaps_and_disposes_scratch_without_next_request() {
        let f = Fixture::new();
        let Some(config) = f.config("idle") else {
            return;
        };
        let handle = SessionHandle::start(config, flag(), deadline())
            .await
            .unwrap();
        let report = tokio::time::timeout(Duration::from_secs(3), handle.stopped())
            .await
            .unwrap();
        assert_eq!(report.reason, StopReason::IdleExpired);
        assert!(report.cleanup_confirmed);
        assert!(f.empty());
    }
    #[tokio::test(flavor = "current_thread")]
    async fn startup_refusal_preserves_typed_confirmed_stop() {
        let f = Fixture::new();
        let Some(config) = f.config("refused") else {
            return;
        };
        let error = match SessionHandle::start(config, flag(), deadline()).await {
            Ok(_) => panic!("unexpected readiness"),
            Err(e) => e,
        };
        assert_eq!(error.dispatch, DispatchState::NotStarted);
        let stop = error.stopped.unwrap();
        assert_eq!(stop.reason, StopReason::Pressure);
        assert!(stop.cleanup_confirmed);
        assert!(f.empty());
    }
    #[tokio::test(flavor = "current_thread")]
    async fn bad_result_and_active_cancellation_wait_for_cleanup() {
        for mode in ["wrong_sequence", "hang"] {
            let f = Fixture::new();
            let Some(config) = f.config(mode) else { return };
            let handle = SessionHandle::start(config, flag(), deadline())
                .await
                .unwrap();
            let cancel = flag();
            let future = handle.infer(request(), cancel.clone(), deadline());
            tokio::pin!(future);
            let result = tokio::select! {result=&mut future=>result,_=tokio::time::sleep(Duration::from_millis(40))=>{cancel.store(true,Ordering::Release);future.await}};
            let error = result.unwrap_err();
            assert_eq!(error.dispatch, DispatchState::MayHaveStarted);
            assert!(error.stopped.unwrap().cleanup_confirmed);
            assert!(f.empty());
        }
    }
    #[tokio::test(flavor = "current_thread")]
    async fn dropped_start_future_still_reaps_and_removes_scratch() {
        let f = Fixture::new();
        let Some(config) = f.config("slow_start") else {
            return;
        };
        let task = tokio::spawn(SessionHandle::start(config, flag(), deadline()));
        tokio::time::sleep(Duration::from_millis(100)).await;
        task.abort();
        let _ = task.await;
        tokio::time::timeout(Duration::from_secs(3), async {
            while !f.empty() {
                tokio::time::sleep(POLL).await;
            }
        })
        .await
        .unwrap();
    }
    #[tokio::test(flavor = "current_thread")]
    async fn late_result_on_cancellation_is_discarded_before_stop_proof() {
        let f = Fixture::new();
        let Some(config) = f.config("late_result") else {
            return;
        };
        let handle = SessionHandle::start(config, flag(), deadline())
            .await
            .unwrap();
        let cancel = flag();
        let call = handle.infer(request(), cancel.clone(), deadline());
        tokio::pin!(call);
        let error = tokio::select! { result=&mut call=>result.unwrap_err(), _=tokio::time::sleep(Duration::from_millis(100))=>{cancel.store(true,Ordering::Release);call.await.unwrap_err()} };
        assert!(
            error.stopped.as_ref().is_some_and(|s| s.cleanup_confirmed),
            "{error:?}"
        );
        assert!(f.empty());
    }
    #[tokio::test(flavor = "current_thread")]
    async fn retiring_session_rejects_undispatched_calls_before_reap() {
        let f = Fixture::new();
        let Some(config) = f.config("idle_hang") else {
            return;
        };
        let handle = SessionHandle::start(config, flag(), deadline())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(250)).await;
        let error = handle
            .infer(
                request(),
                flag(),
                Instant::now() + Duration::from_millis(50),
            )
            .await
            .unwrap_err();
        assert_eq!(error.dispatch, DispatchState::NotStarted);
        assert!(!f.empty(), "stop frame alone cannot dispose scratch");
        assert!(handle.stopped().await.cleanup_confirmed);
        assert!(f.empty());
    }
    #[tokio::test(flavor = "current_thread")]
    async fn execution_deadline_stops_admission_and_awaits_bounded_cleanup() {
        let f = Fixture::new();
        let Some(config) = f.config("deadline_hang") else {
            return;
        };
        let handle = SessionHandle::start(config, flag(), deadline())
            .await
            .unwrap();
        let started = Instant::now();
        let first = handle.infer(request(), flag(), started + Duration::from_millis(40));
        let second = async {
            tokio::time::sleep(Duration::from_millis(150)).await;
            handle.infer(request(), flag(), deadline()).await
        };
        let (first, second) = tokio::time::timeout(Duration::from_secs(3), async {
            tokio::join!(first, second)
        })
        .await
        .unwrap();
        let error = first.unwrap_err();
        assert_eq!(error.dispatch, DispatchState::MayHaveStarted);
        assert!(
            error.stopped.as_ref().is_some_and(|s| s.cleanup_confirmed),
            "{error:?}"
        );
        assert_eq!(second.unwrap_err().dispatch, DispatchState::NotStarted);
        assert!(
            started.elapsed() >= Duration::from_millis(500),
            "must await delayed process exit"
        );
        assert!(f.empty());
    }
    #[tokio::test(flavor = "current_thread")]
    async fn trailing_malformed_output_and_retained_scratch_block_replacement() {
        for mode in ["trailing", "normal"] {
            let f = Fixture::new();
            let Some(config) = f.config(mode) else {
                return;
            };
            let path = config.scratch.path.clone();
            let handle = SessionHandle::start(config, flag(), deadline())
                .await
                .unwrap();
            if mode == "normal" {
                fs::write(path.join("unexpected"), b"preserve").unwrap();
            }
            let report = handle.shutdown().await;
            assert!(!report.cleanup_confirmed, "{report:?}");
            assert!(path.exists());
            if mode == "normal" {
                assert_eq!(fs::read(path.join("unexpected")).unwrap(), b"preserve");
                fs::remove_file(path.join("unexpected")).unwrap();
            }
            fs::remove_dir(path).unwrap();
        }
    }
    struct TestPressure {
        elevated: Arc<AtomicBool>,
        busy_dispatches: u8,
        idle_busy: bool,
        slow_dispatch: bool,
    }
    impl Pressure for TestPressure {
        fn elevated(&mut self) -> Result<Option<bool>, String> {
            if self.idle_busy {
                Ok(None)
            } else {
                Ok(Some(self.elevated.load(Ordering::Acquire)))
            }
        }
        fn before_dispatch(&mut self) -> Result<Option<bool>, String> {
            if self.slow_dispatch {
                std::thread::sleep(Duration::from_millis(30));
            }
            if self.busy_dispatches > 0 {
                self.busy_dispatches -= 1;
                Ok(None)
            } else {
                Ok(Some(self.elevated.load(Ordering::Acquire)))
            }
        }
    }
    #[tokio::test(flavor = "current_thread")]
    async fn real_loop_preserves_first_work_then_reclaims_for_waiter_pressure_or_idle() {
        use std::os::{fd::AsRawFd, unix::net::UnixStream};
        use tokio::io::AsyncReadExt;
        let Some(node) = std::env::var_os("MIRKET_TEST_NODE") else {
            eprintln!("SKIP supervisor pipe fixture: explicit Node required");
            return;
        };
        for mode in [
            "waiting",
            "pressure",
            "expiry",
            "busy",
            "inspection_deadline",
        ] {
            let f = Fixture::new();
            let state = f.root.join("state");
            let admission = Admission::open(&state).unwrap();
            let mut resident = admission.try_acquire().unwrap();
            resident.mark_unresolved("real-loop-test").unwrap();
            let _waiting = if mode == "waiting" {
                Some(admission.try_waiting().unwrap())
            } else {
                None
            };
            let (out_read, out_write) = UnixStream::pair().unwrap();
            out_read.set_nonblocking(true).unwrap();
            let mut output = crate::worker::Lifeline::open(out_write.as_raw_fd()).unwrap();
            let (parent_read, parent_write) = UnixStream::pair().unwrap();
            parent_write.set_nonblocking(true).unwrap();
            let mut parent =
                parent_frames(crate::worker::Lifeline::open(parent_read.as_raw_fd()).unwrap());
            let config = ConfigWire {
                protocol_version: 2,
                state_root: state,
                release_id: "warm-test".into(),
                session_id: "loop-test".into(),
                package_root: f.root.join("package"),
                scratch_dir: f.root.clone(),
                remaining_ms: 5000,
            };
            let source=r#"const rl=require('node:readline').createInterface({input:process.stdin});const crypto=require('node:crypto');const model=__MODEL__;rl.on('line',line=>{require('node:fs').writeFileSync('received','yes');const r=JSON.parse(line);const text=r.input.query;const vector=Array(model.dimensions).fill(0);vector[0]=1;const reply={protocol_version:2,release_id:'warm-test',session_id:'loop-test',body:{kind:'result',sequence:r.sequence,job_id:r.job_id,output:{schema:1,input_revision:r.input_revision,query:{model_identity:model,source_sha256:crypto.createHash('sha256').update(text).digest('hex'),input_bytes:Buffer.byteLength(text),input_tokens:3,vector},passages:[]}}};const data=JSON.stringify(reply)+'\n';setTimeout(()=>{process.stdout.write(data.slice(0,17));setTimeout(()=>process.stdout.write(data.slice(17)),60);},120);});"#.replace("__MODEL__",&serde_json::to_string(&crate::embedding::expected_model_identity().unwrap()).unwrap());
            let mut command = Command::new(&node);
            command
                .arg("-e")
                .arg(source)
                .current_dir(&f.root)
                .env_clear()
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true);
            {
                use std::os::unix::process::CommandExt;
                command.as_std_mut().process_group(0);
            }
            let mut owned = OwnedChild::new(command.spawn().unwrap()).unwrap();
            let mut stdin = owned.child.stdin.take().unwrap();
            let mut leaves = frames(owned.child.stdout.take().unwrap());
            let request = request();
            let wire = InferWire {
                protocol_version: 2,
                release_id: config.release_id.clone(),
                session_id: config.session_id.clone(),
                remaining_ms: if mode == "inspection_deadline" {
                    10
                } else {
                    3000
                },
                request: WarmInference {
                    sequence: 1,
                    job_id: request.job_id,
                    input_revision: request.input_revision,
                    input: request.input,
                },
            };
            let bytes = encode(&wire).unwrap();
            let mut writer = tokio::net::UnixStream::from_std(parent_write).unwrap();
            let send = tokio::spawn(async move {
                writer.write_all(&bytes[..11]).await.unwrap();
                tokio::time::sleep(Duration::from_millis(80)).await;
                writer.write_all(&bytes[11..]).await.unwrap();
                writer
            });
            let elevated = flag();
            let raised = elevated.clone();
            let pressure_change = mode == "pressure";
            let pressure_task = tokio::spawn(async move {
                if pressure_change {
                    tokio::time::sleep(Duration::from_millis(180)).await;
                    raised.store(true, Ordering::Release);
                }
            });
            let mut pressure = TestPressure {
                elevated,
                busy_dispatches: if mode == "busy" { 2 } else { 0 },
                idle_busy: mode == "busy",
                slow_dispatch: mode == "inspection_deadline",
            };
            let result = tokio::time::timeout(
                Duration::from_secs(4),
                supervisor_loop(
                    &config,
                    &admission,
                    &mut pressure,
                    &mut parent,
                    &mut output,
                    &mut stdin,
                    &mut leaves,
                    flag(),
                    deadline(),
                    Duration::from_millis(40),
                ),
            )
            .await
            .unwrap();
            assert_eq!(
                result.0,
                match mode {
                    "waiting" => StopReason::WaitingDemand,
                    "pressure" => StopReason::Pressure,
                    "inspection_deadline" => StopReason::Deadline,
                    _ => StopReason::IdleExpired,
                },
                "{result:?}"
            );
            assert_eq!(
                pressure.busy_dispatches, 0,
                "contended dispatch must obtain a fresh observation before sending work"
            );
            // A valid first result exists despite demand during its cold-start
            // ownership interval; pressure raised during native work waited.
            if mode != "inspection_deadline" {
                let mut read = tokio::net::UnixStream::from_std(out_read).unwrap();
                let mut bytes = vec![0; FRAME];
                let count = read.read(&mut bytes).await.unwrap();
                assert!(matches!(
                    checked_reply(&bytes[..count], &config).unwrap(),
                    SupervisorReply::Result { sequence: 1, .. }
                ));
            }
            let _other = admission.try_acquire().unwrap();
            assert!(matches!(admission.try_acquire(), Err(AdmissionError::Busy)));
            drop(stdin);
            tokio::time::timeout(WRITE, owned.wait())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                f.root.join("received").exists(),
                mode != "inspection_deadline",
                "expired inspection must not send a request to the native leaf"
            );
            resident.clear_after_cleanup().unwrap();
            drop(send.await.unwrap());
            // Lifeline borrows this descriptor. Confirm its reader finished
            // before dropping/reusing the backing test UnixStream descriptor.
            assert_eq!(
                tokio::time::timeout(WRITE, next(&mut parent))
                    .await
                    .unwrap()
                    .unwrap_err(),
                "parent closed"
            );
            pressure_task.await.unwrap();
            drop(output);
            drop(out_write);
            drop(parent_read);
        }
    }
    #[test]
    fn frames_and_remaining_deadlines_are_bounded() {
        assert!(encode(&"x".repeat(FRAME)).is_err());
        assert!(remaining(Instant::now()).is_err());
        assert!(remaining(Instant::now() + Duration::from_secs(600)).unwrap() <= 60_000);
    }
}
