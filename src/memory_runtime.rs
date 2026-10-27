//! One lazy embedding session per server, using shared resident-work admission.
//!
//! Raw source and vectors stay in bounded anonymous pipes. The parent never
//! verifies model bytes or initializes ORT. Dropping a request signals cancel;
//! an independently owned task still awaits the supervisor and its cleanup.
//! Directory retention is a diagnostic, never permission to reclaim a worker's
//! durable heavy-slot marker. No recursive scratch deletion is performed.
//! The server must await shutdown before stopping Tokio; abrupt runtime/process
//! death relies on the separate supervisor lifeline and durable admission owner.

use crate::{
    embedding,
    embedding_job::{Input, Output},
    embedding_session::{
        DispatchState, Inference, SessionConfig, SessionError, SessionHandle, SessionStatus,
    },
    engine::revision,
    leases::Admission,
    owned_scratch::{DirectoryStamp, OwnedScratch, explicit_path},
    release::ReleaseId,
};
#[cfg(test)]
use crate::{embedding_job::Request, process::AdapterError};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    sync::{Mutex as AsyncMutex, Notify, oneshot},
    task::JoinHandle,
    time::Instant,
};

pub const MAX_ACTIVE_REQUESTS: usize = 8;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(35);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
static NEXT_JOB: AtomicU64 = AtomicU64::new(0);

pub struct MemoryRuntime {
    inner: Arc<Inner>,
}
struct Inner {
    state_root: PathBuf,
    state_stamp: DirectoryStamp,
    release: ReleaseId,
    package_root: PathBuf,
    current_exe: PathBuf,
    registry: Mutex<Registry>,
    session: AsyncMutex<SessionSlot>,
    completed: Notify,
    #[cfg(test)]
    runner: Option<TestRunner>,
}
enum SessionSlot {
    Cold,
    Live(SessionHandle),
    Blocked(String),
}
#[derive(Default)]
struct Registry {
    closed: bool,
    active: BTreeMap<String, Control>,
}
struct Control {
    cancel: Arc<AtomicBool>,
    // Dropping this handle detaches; it must never be aborted during cleanup.
    _task: JoinHandle<()>,
}
struct CancelOnDrop(Arc<AtomicBool>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
struct Completion {
    inner: Arc<Inner>,
    job_id: String,
}
impl Drop for Completion {
    fn drop(&mut self) {
        let mut registry = self
            .inner
            .registry
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        registry.active.remove(&self.job_id);
        drop(registry);
        self.inner.completed.notify_waiters();
    }
}

impl MemoryRuntime {
    /// Select one explicit release and private payload for this server. Only
    /// path metadata is inspected here; payload hashes are checked by the leaf
    /// after it obtains shared heavy-work admission. No ambient configuration.
    pub fn new(
        state_root: PathBuf,
        release: ReleaseId,
        package_root: PathBuf,
        current_exe: PathBuf,
    ) -> Result<Self, String> {
        if !cfg!(unix) {
            return Err(
                "native embedding process ownership is not qualified on this platform".into(),
            );
        }
        let state_stamp = DirectoryStamp::inspect(&state_root)?;
        explicit_path(&package_root, true)?;
        explicit_path(&current_exe, false)?;
        Ok(Self {
            inner: Arc::new(Inner {
                state_root,
                state_stamp,
                release,
                package_root,
                current_exe,
                registry: Mutex::new(Registry::default()),
                session: AsyncMutex::new(SessionSlot::Cold),
                completed: Notify::new(),
                #[cfg(test)]
                runner: None,
            }),
        })
    }

    pub async fn execute(&self, input: Input) -> Result<Output, String> {
        let deadline = Instant::now() + REQUEST_TIMEOUT;
        validate_input(&input)?;
        tokio::runtime::Handle::try_current()
            .map_err(|_| "embedding requires an active Tokio runtime")?;
        let input_revision =
            revision(&serde_json::to_vec(&input).map_err(|_| "embedding input encoding failed")?);
        let (sender, receiver) = oneshot::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        {
            // No await while holding this lock: registration and launch are one
            // admission boundary relative to concurrent shutdown and requests.
            let mut registry = self
                .inner
                .registry
                .lock()
                .map_err(|_| "embedding request registry unavailable")?;
            if registry.closed {
                return Err("embedding runtime is shutting down".into());
            }
            if registry.active.len() >= MAX_ACTIVE_REQUESTS {
                return Err("eight embedding requests are already active".into());
            }
            let job_id = job_identity()?;
            let completion = Completion {
                inner: self.inner.clone(),
                job_id: job_id.clone(),
            };
            let owned_cancel = cancel.clone();
            let task = tokio::spawn(async move {
                let result = run(
                    &completion.inner,
                    &completion.job_id,
                    input,
                    input_revision,
                    owned_cancel,
                    deadline,
                )
                .await;
                // A successful inference leaves its resident session owned.
                // Interrupted work awaits cleanup before releasing the mutex.
                drop(completion);
                let _ = sender.send(result);
            });
            registry.active.insert(
                job_id,
                Control {
                    cancel: cancel.clone(),
                    _task: task,
                },
            );
        }
        let _cancel_on_drop = CancelOnDrop(cancel);
        receiver
            .await
            .map_err(|_| "embedding ownership task ended without confirmed cleanup".to_owned())?
    }

    /// Permanently close this instance, cancel all active requests, and wait for
    /// their owned tasks. A deadline reports uncertainty without aborting them.
    pub async fn shutdown(&self) -> Result<(), String> {
        self.shutdown_with_timeout(SHUTDOWN_TIMEOUT).await
    }

    async fn shutdown_with_timeout(&self, timeout: Duration) -> Result<(), String> {
        self.close();
        let wait = async {
            loop {
                let notified = self.inner.completed.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if self
                    .inner
                    .registry
                    .lock()
                    .map_err(|_| "embedding request registry unavailable")?
                    .active
                    .is_empty()
                {
                    break;
                }
                notified.await;
            }
            let mut session = self.inner.session.lock().await;
            match &*session {
                SessionSlot::Cold => Ok(()),
                SessionSlot::Blocked(message) => Err(message.clone()),
                SessionSlot::Live(handle) => {
                    let stopped = handle.shutdown().await;
                    if stopped.cleanup_confirmed {
                        *session = SessionSlot::Cold;
                        Ok(())
                    } else {
                        Err(stopped
                            .message
                            .unwrap_or_else(|| "embedding session cleanup remains unknown".into()))
                    }
                }
            }
        };
        tokio::time::timeout(timeout, wait).await
            .map_err(|_| "embedding shutdown cleanup deadline elapsed; owned tasks continue and uncertain directories are retained".to_owned())?
    }

    fn close(&self) {
        let mut registry = self
            .inner
            .registry
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        registry.closed = true;
        for control in registry.active.values() {
            control.cancel.store(true, Ordering::Release);
        }
        drop(registry);
        if let Ok(session) = self.inner.session.try_lock()
            && let SessionSlot::Live(handle) = &*session
        {
            handle.request_shutdown();
        }
    }
}
impl Drop for MemoryRuntime {
    fn drop(&mut self) {
        self.close();
    }
}

async fn run(
    inner: &Inner,
    job_id: &str,
    input: Input,
    input_revision: String,
    cancel: Arc<AtomicBool>,
    deadline: Instant,
) -> Result<Output, String> {
    // Controlled request-owner tests do not initialize native runtimes. The
    // actual resident-session lifecycle has separate native protocol fixtures.
    #[cfg(test)]
    if let Some(runner) = &inner.runner {
        let directory = OwnedScratch::create(&inner.state_root, &inner.state_stamp, job_id)?;
        let request = Request {
            state_root: inner.state_root.clone(),
            release_id: inner.release.as_str().into(),
            job_id: job_id.into(),
            package_root: inner.package_root.clone(),
            job_dir: directory.path.clone(),
            input,
            input_revision,
        };
        let result = runner(request, cancel).await;
        if result.as_ref().is_err_and(|error| !error.cleanup_confirmed) {
            return Err(format!(
                "embedding job {job_id} cleanup is unconfirmed; directory retained; inspect worker ownership: {}",
                result.unwrap_err().message
            ));
        }
        directory
            .remove_empty()
            .map_err(|error| format!("embedding job {job_id} retained: {error}"))?;
        return result.map_err(|error| error.message);
    }
    inner.state_stamp.matches(&inner.state_root)?;
    let mut session = match inner.session.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            // Only actual contention consumes a waiting permit. Direct idle
            // reuse must not evict itself by manufacturing waiting demand.
            let admission = Admission::open(&inner.state_root).map_err(|e| e.to_string())?;
            let waiting = admission.try_waiting().map_err(|e| e.to_string())?;
            let guard = tokio::select! {
                guard = inner.session.lock() => guard,
                _ = cancelled(&cancel) => return Err("embedding cancelled while queued".into()),
                _ = tokio::time::sleep_until(deadline) => return Err("embedding work deadline elapsed while queued".into()),
            };
            drop(waiting);
            guard
        }
    };
    if cancel.load(Ordering::Acquire) {
        return Err("embedding cancelled before dispatch".into());
    }
    if Instant::now() >= deadline {
        return Err("embedding work deadline elapsed before dispatch".into());
    }
    if let SessionSlot::Live(handle) = &*session {
        match handle.status() {
            SessionStatus::Stopped(stopped) if stopped.cleanup_confirmed => {
                *session = SessionSlot::Cold
            }
            SessionStatus::Stopped(stopped) => {
                return Err(stopped
                    .message
                    .unwrap_or_else(|| "embedding session cleanup remains unknown".into()));
            }
            SessionStatus::Unknown => {
                return Err(
                    "embedding session ownership is unknown; inspect before further work".into(),
                );
            }
            SessionStatus::Live => (),
        }
    }
    if let SessionSlot::Blocked(message) = &*session {
        return Err(message.clone());
    }
    if matches!(*session, SessionSlot::Cold) {
        let session_id = job_identity()?;
        let scratch = OwnedScratch::create(&inner.state_root, &inner.state_stamp, &session_id)?;
        let config = SessionConfig {
            state_root: inner.state_root.clone(),
            release_id: inner.release.as_str().into(),
            session_id,
            package_root: inner.package_root.clone(),
            scratch,
            current_exe: inner.current_exe.clone(),
        };
        match SessionHandle::start(config, cancel.clone(), deadline).await {
            Ok(handle) => *session = SessionSlot::Live(handle),
            Err(error) => {
                if !error
                    .stopped
                    .as_ref()
                    .is_some_and(|stopped| stopped.cleanup_confirmed)
                {
                    *session = SessionSlot::Blocked(error.message.clone());
                }
                return Err(session_error(error));
            }
        }
    }
    let SessionSlot::Live(handle) = &*session else {
        unreachable!("session established above")
    };
    // No retry after dispatch, even if idle eviction races this new request.
    // A later independent request may create a cold session after confirmed stop.
    handle
        .infer(
            Inference {
                job_id: job_id.into(),
                input_revision,
                input,
            },
            cancel,
            deadline,
        )
        .await
        .map_err(session_error)
}

fn session_error(error: SessionError) -> String {
    match error.dispatch {
        DispatchState::NotStarted => format!("embedding inference not started: {}", error.message),
        DispatchState::MayHaveStarted => format!(
            "embedding inference may have started; request was not replayed: {}",
            error.message
        ),
    }
}

async fn cancelled(cancel: &AtomicBool) {
    while !cancel.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn validate_input(input: &Input) -> Result<(), String> {
    if (input.query.is_none() && input.passages.is_empty())
        || input.passages.len() > embedding::MAX_PASSAGES
    {
        return Err("embedding requires a query or between one and thirty-two passages".into());
    }
    let mut bytes = 0;
    for text in input.query.iter().chain(input.passages.iter()) {
        if text.trim().is_empty() || text.len() > embedding::MAX_INPUT_BYTES || text.contains('\0')
        {
            return Err("embedding input must be nonempty bounded UTF-8 text without NUL".into());
        }
        bytes += text.len();
    }
    if bytes > embedding::MAX_BATCH_BYTES {
        return Err("embedding input exceeds its aggregate byte limit".into());
    }
    Ok(())
}

fn job_identity() -> Result<String, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "embedding requires a valid clock")?
        .as_nanos();
    Ok(format!(
        "op-embed-{nanos:x}-{:x}-{:x}",
        std::process::id(),
        NEXT_JOB.fetch_add(1, Ordering::Relaxed)
    ))
}

#[cfg(test)]
type TestRunner = Arc<
    dyn Fn(
            Request,
            Arc<AtomicBool>,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Output, AdapterError>> + Send>,
        > + Send
        + Sync,
>;

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mirket-memory-runtime-{}-{}",
                std::process::id(),
                NEXT_JOB.fetch_add(1, Ordering::Relaxed)
            ));
            fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
            let root = root.canonicalize().unwrap();
            fs::DirBuilder::new()
                .mode(0o700)
                .create(root.join("package"))
                .unwrap();
            fs::write(
                root.join("mirket"),
                b"metadata-only fixture; never executed",
            )
            .unwrap();
            Self(root)
        }
        fn runtime(&self, runner: TestRunner) -> MemoryRuntime {
            let mut runtime = MemoryRuntime::new(
                self.0.clone(),
                ReleaseId::parse("test-release").unwrap(),
                self.0.join("package"),
                self.0.join("mirket"),
            )
            .unwrap();
            Arc::get_mut(&mut runtime.inner).unwrap().runner = Some(runner);
            runtime
        }
        fn queued_runtime(&self) -> MemoryRuntime {
            MemoryRuntime::new(
                self.0.clone(),
                ReleaseId::parse("test-release").unwrap(),
                self.0.join("package"),
                self.0.join("mirket"),
            )
            .unwrap()
        }
        fn jobs(&self) -> Vec<PathBuf> {
            match fs::read_dir(self.0.join("jobs")) {
                Ok(entries) => entries.map(|entry| entry.unwrap().path()).collect(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => vec![],
                Err(error) => panic!("fixture jobs unavailable: {error}"),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn input() -> Input {
        Input {
            query: Some("private query never staged".into()),
            passages: vec![],
        }
    }
    fn output(request: &Request) -> Output {
        // Ownership tests substitute only the already-validated worker result;
        // protocol/vector correctness is tested by embedding_job separately.
        Output {
            schema: 1,
            input_revision: request.input_revision.clone(),
            query: None,
            passages: vec![],
        }
    }
    fn active(runtime: &MemoryRuntime) -> usize {
        runtime.inner.registry.lock().unwrap().active.len()
    }
    async fn wait_cancel(flag: Arc<AtomicBool>) {
        while !flag.load(Ordering::Acquire) {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    fn controlled_runner(
        started: tokio::sync::mpsc::UnboundedSender<PathBuf>,
        cancelled: Arc<Notify>,
        release: Arc<Notify>,
    ) -> TestRunner {
        Arc::new(move |request, flag| {
            let started = started.clone();
            let cancelled = cancelled.clone();
            let release = release.clone();
            Box::pin(async move {
                started.send(request.job_dir.clone()).unwrap();
                wait_cancel(flag).await;
                cancelled.notify_one();
                release.notified().await;
                Err(AdapterError {
                    message: "cancelled after confirmed reap".into(),
                    cleanup_confirmed: true,
                })
            })
        })
    }

    #[tokio::test(flavor = "current_thread")]
    async fn queued_cancellation_releases_global_waiter_without_model_or_heavy_ownership() {
        let fixture = Fixture::new();
        let runtime = Arc::new(fixture.queued_runtime());
        let guard = runtime.inner.session.lock().await;
        let caller = tokio::spawn({
            let runtime = runtime.clone();
            async move { runtime.execute(input()).await }
        });
        let admission = Admission::open(&fixture.0).unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            while !admission.has_waiters().unwrap() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let first = admission.try_acquire().unwrap();
        let second = admission.try_acquire().unwrap();
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        tokio::time::timeout(Duration::from_secs(1), async {
            while active(&runtime) != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(!admission.has_waiters().unwrap());
        assert!(fixture.jobs().is_empty());
        assert!(matches!(*guard, SessionSlot::Cold));
        drop((first, second, guard));
        runtime.shutdown().await.unwrap();
        let permits: Vec<_> = (0..8).map(|_| admission.try_waiting().unwrap()).collect();
        assert!(matches!(
            admission.try_waiting(),
            Err(crate::leases::AdmissionError::WaitingFull)
        ));
        drop(permits);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn local_queue_obeys_original_work_deadline_without_dispatch() {
        let fixture = Fixture::new();
        let runtime = fixture.queued_runtime();
        let guard = runtime.inner.session.lock().await;
        let input = input();
        let digest = revision(&serde_json::to_vec(&input).unwrap());
        let result = run(
            &runtime.inner,
            "op-queued-deadline",
            input,
            digest,
            Arc::new(AtomicBool::new(false)),
            Instant::now() + Duration::from_millis(20),
        )
        .await;
        assert!(
            result
                .unwrap_err()
                .contains("deadline elapsed while queued")
        );
        let admission = Admission::open(&fixture.0).unwrap();
        assert!(!admission.has_waiters().unwrap());
        assert!(fixture.jobs().is_empty());
        assert!(matches!(*guard, SessionSlot::Cold));
        drop(guard);
        runtime.shutdown().await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn success_keeps_input_in_memory_and_removes_empty_owned_directory() {
        let fixture = Fixture::new();
        let runtime = fixture.runtime(Arc::new(|request, _| {
            Box::pin(async move {
                assert!(request.job_id.starts_with("op-embed-"));
                assert_eq!(fs::read_dir(&request.job_dir).unwrap().count(), 0);
                assert_eq!(
                    request.input_revision,
                    revision(&serde_json::to_vec(&request.input).unwrap())
                );
                Ok(output(&request))
            })
        }));
        assert!(!fixture.0.join("jobs").exists());
        runtime.execute(input()).await.unwrap();
        assert!(fixture.jobs().is_empty());
        assert_eq!(active(&runtime), 0);
        runtime.shutdown().await.unwrap();
        assert!(
            runtime
                .execute(input())
                .await
                .unwrap_err()
                .contains("shutting down")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dropped_caller_signals_cancel_but_keeps_owned_task_until_reap() {
        let fixture = Fixture::new();
        let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel();
        let cancelled = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let runtime = Arc::new(fixture.runtime(controlled_runner(
            started_tx,
            cancelled.clone(),
            release.clone(),
        )));
        let caller = tokio::spawn({
            let runtime = runtime.clone();
            async move { runtime.execute(input()).await }
        });
        let directory = started_rx.recv().await.unwrap();
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        tokio::time::timeout(Duration::from_secs(1), cancelled.notified())
            .await
            .unwrap();
        assert!(directory.is_dir());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 0);
        assert_eq!(active(&runtime), 1);
        release.notify_one();
        runtime
            .shutdown_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert!(!directory.exists());
        assert_eq!(active(&runtime), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_timeout_does_not_abort_the_cleanup_task() {
        let fixture = Fixture::new();
        let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel();
        let cancelled = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let runtime = Arc::new(fixture.runtime(controlled_runner(
            started_tx,
            cancelled.clone(),
            release.clone(),
        )));
        let caller = tokio::spawn({
            let runtime = runtime.clone();
            async move { runtime.execute(input()).await }
        });
        let directory = started_rx.recv().await.unwrap();
        assert!(
            runtime
                .shutdown_with_timeout(Duration::from_millis(10))
                .await
                .is_err()
        );
        assert_eq!(active(&runtime), 1);
        assert!(directory.exists());
        assert!(!caller.is_finished());
        release.notify_one();
        runtime
            .shutdown_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        assert!(
            caller
                .await
                .unwrap()
                .unwrap_err()
                .contains("confirmed reap")
        );
        assert!(!directory.exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn capacity_and_input_bounds_precede_new_scratch_or_worker_launch() {
        let fixture = Fixture::new();
        let (started_tx, mut started_rx) = tokio::sync::mpsc::unbounded_channel();
        let runtime = Arc::new(fixture.runtime(Arc::new(move |request, flag| {
            let started = started_tx.clone();
            Box::pin(async move {
                started.send(request.job_id).unwrap();
                wait_cancel(flag).await;
                Err(AdapterError {
                    message: "cancelled".into(),
                    cleanup_confirmed: true,
                })
            })
        })));
        for invalid in [
            Input {
                query: None,
                passages: vec![],
            },
            Input {
                query: Some("x".repeat(embedding::MAX_INPUT_BYTES + 1)),
                passages: vec![],
            },
            Input {
                query: None,
                passages: vec!["x".repeat(embedding::MAX_INPUT_BYTES); 17],
            },
            Input {
                query: None,
                passages: vec!["x".into(); 33],
            },
            Input {
                query: Some("nul\0value".into()),
                passages: vec![],
            },
        ] {
            assert!(runtime.execute(invalid).await.is_err());
        }
        assert!(!fixture.0.join("jobs").exists());
        let mut callers = Vec::new();
        for _ in 0..MAX_ACTIVE_REQUESTS {
            callers.push(tokio::spawn({
                let runtime = runtime.clone();
                async move { runtime.execute(input()).await }
            }));
        }
        for _ in 0..MAX_ACTIVE_REQUESTS {
            started_rx.recv().await.unwrap();
        }
        assert_eq!(active(&runtime), 8);
        assert_eq!(fixture.jobs().len(), 8);
        assert!(
            runtime
                .execute(input())
                .await
                .unwrap_err()
                .contains("eight")
        );
        assert_eq!(fixture.jobs().len(), 8);
        runtime
            .shutdown_with_timeout(Duration::from_secs(1))
            .await
            .unwrap();
        for caller in callers {
            assert!(caller.await.unwrap().is_err());
        }
        assert!(fixture.jobs().is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn unknown_cleanup_retains_empty_directory_without_new_marker_authority() {
        let fixture = Fixture::new();
        let runtime = fixture.runtime(Arc::new(|_, _| {
            Box::pin(async {
                Err(AdapterError {
                    message: "worker ownership unknown".into(),
                    cleanup_confirmed: false,
                })
            })
        }));
        assert!(
            runtime
                .execute(input())
                .await
                .unwrap_err()
                .contains("cleanup is unconfirmed")
        );
        let jobs = fixture.jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(fs::read_dir(&jobs[0]).unwrap().count(), 0);
        assert_eq!(active(&runtime), 0);
        runtime.shutdown().await.unwrap();
        assert!(jobs[0].exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn unexpected_file_is_retained_instead_of_recursively_deleted() {
        let fixture = Fixture::new();
        let runtime = fixture.runtime(Arc::new(|request, _| {
            Box::pin(async move {
                fs::write(request.job_dir.join("unexpected"), b"retain this fixture").unwrap();
                Ok(output(&request))
            })
        }));
        assert!(
            runtime
                .execute(input())
                .await
                .unwrap_err()
                .contains("retained")
        );
        let jobs = fixture.jobs();
        assert_eq!(jobs.len(), 1);
        assert_eq!(
            fs::read(jobs[0].join("unexpected")).unwrap(),
            b"retain this fixture"
        );
        assert_eq!(active(&runtime), 0);
    }

    #[test]
    fn paths_permissions_existing_jobs_and_replaced_directory_are_not_adopted() {
        let fixture = Fixture::new();
        let runtime = fixture.runtime(Arc::new(|_, _| Box::pin(async { unreachable!() })));
        assert!(
            MemoryRuntime::new(
                PathBuf::from("relative"),
                ReleaseId::parse("r").unwrap(),
                fixture.0.join("package"),
                fixture.0.join("mirket")
            )
            .is_err()
        );
        let first = OwnedScratch::create(
            &runtime.inner.state_root,
            &runtime.inner.state_stamp,
            "op-test-existing",
        )
        .unwrap();
        assert!(
            OwnedScratch::create(
                &runtime.inner.state_root,
                &runtime.inner.state_stamp,
                "op-test-existing"
            )
            .is_err()
        );
        let saved = fixture.0.join("retained-original");
        fs::rename(&first.path, &saved).unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(&first.path)
            .unwrap();
        let replacement = first.path.clone();
        assert!(first.remove_empty().is_err());
        assert!(saved.exists());
        assert!(replacement.exists());
        fs::set_permissions(fixture.0.join("jobs"), fs::Permissions::from_mode(0o755)).unwrap();
        assert!(
            OwnedScratch::create(
                &runtime.inner.state_root,
                &runtime.inner.state_stamp,
                "op-new"
            )
            .is_err()
        );
        fs::set_permissions(fixture.0.join("jobs"), fs::Permissions::from_mode(0o700)).unwrap();
        let alias = fixture.0.join("alias");
        std::os::unix::fs::symlink(fixture.0.join("package"), &alias).unwrap();
        assert!(
            MemoryRuntime::new(
                fixture.0.clone(),
                ReleaseId::parse("r").unwrap(),
                alias,
                fixture.0.join("mirket")
            )
            .is_err()
        );
    }
}
