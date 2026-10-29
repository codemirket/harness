//! The operation owner: finite private children and durable receipts.
//! Receipt reads never turn an unknown or evicted operation into a fresh start.
#[path = "engine_code_navigation.rs"]
mod code_navigation;
#[path = "engine_design.rs"]
mod design;
#[path = "engine_diagram.rs"]
mod diagram;
use crate::file_lock::FileLock;
use crate::{
    artifacts::{ArtifactPlan, PublicationMode},
    job_state::{Effect, JobState, Status},
    operation_store::OperationStore,
    release::{AdapterPayload, ReleaseId},
    scope::ProjectScope,
};
pub use code_navigation::CodeNavigateArgs;
pub use design::DesignPrepareArgs;
pub use diagram::DiagramPrepareArgs;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

pub fn revision(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| "invalid system clock".into())
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

#[derive(Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct JobArgs {
    pub expected_release_id: String,
    pub project_root: PathBuf,
    pub operation_id: String,
}
#[derive(Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CodeInspectArgs {
    pub expected_release_id: String,
    pub project_root: PathBuf,
    pub request: crate::code_inspect::InspectRequest<crate::code_inspect::CodeLanguage>,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OperationKind {
    Diagram,
    Design,
}

enum PendingOperation {
    Diagram(crate::diagram::D2Layout),
    Design(crate::design::DesignOperation),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: u8,
    kind: OperationKind,
    release_id: String,
    owner_id: String,
    project_root: PathBuf,
    source_path: PathBuf,
    destination_path: PathBuf,
    source_revision: String,
    intent_revision: String,
    created_at: u64,
    expires_at: u64,
    state: JobState,
    artifact_revision: Option<String>,
    error: Option<String>,
    #[serde(default)]
    scratch_cleanup: Option<String>,
    queue_wait_ms: Option<u64>,
}
struct Pending {
    plan: ArtifactPlan,
    operation: PendingOperation,
    receipt: Receipt,
}
struct Control {
    cancel: Arc<AtomicBool>,
    handle: tokio::task::JoinHandle<()>,
}
#[derive(Default)]
struct Registry {
    closed: bool,
    active: HashMap<String, Control>,
}
/// Retire one expired, originally issued preparation while all sessions and
/// workers are excluded. This never starts work or reclaims scratch/markers.
pub fn retire_expired_preparation(
    state_root: &Path,
    expected_release: &ReleaseId,
    project_root: &Path,
    operation_id: &str,
) -> Result<Value, String> {
    let maintenance = crate::maintenance::MaintenanceGuard::try_acquire(state_root).map_err(err)?;
    let project = ProjectScope::open(project_root).map_err(err)?;
    let store = OperationStore::open_existing(maintenance.state_root()).map_err(err)?;
    let locked = store.lock().map_err(err)?;
    let bytes = locked.read(operation_id).map_err(err)?;
    let saved: Receipt = serde_json::from_slice(&bytes).map_err(err)?;
    expected_release.require(&saved.release_id).map_err(err)?;
    if saved.schema != 2 || saved.project_root != project.root() {
        return Err("preparation does not belong to the selected project/schema".into());
    }
    if now()? <= saved.expires_at || saved.created_at.checked_add(300) != Some(saved.expires_at) {
        return Err("only an expired preparation may be retired".into());
    }
    let mut original = saved.clone();
    let retry = saved.state.status == Status::Cancelled
        && saved.error.as_deref() == Some("expired preparation retired without starting")
        && saved.scratch_cleanup.as_deref() == Some("not_started");
    if retry {
        original.state.status = Status::Prepared;
        original.error = None;
        original.scratch_cleanup = None;
    }
    if original.state != JobState::default()
        || original.artifact_revision.is_some()
        || original.error.is_some()
        || original.scratch_cleanup.is_some()
        || original.queue_wait_ms.is_some()
        || !locked
            .matches_issued_payload(operation_id, &serde_json::to_vec(&original).map_err(err)?)
            .map_err(err)?
    {
        return Err("preparation is started, changed or unbound; retain recovery evidence".into());
    }
    // A conforming Prepared operation cannot have created a job directory.
    // Even an empty or substituted entry is contradictory evidence, not cleanup.
    let jobs = maintenance.state_root().join("jobs");
    match fs::symlink_metadata(&jobs) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            match fs::symlink_metadata(jobs.join(operation_id)) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                _ => {
                    return Err(
                        "job scratch exists or cannot be inspected; retain preparation".into(),
                    );
                }
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        _ => return Err("job scratch namespace is unavailable; retain preparation".into()),
    }
    let mut retired = original;
    retired.state.cancel();
    retired.error = Some("expired preparation retired without starting".into());
    retired.scratch_cleanup = Some("not_started".into());
    locked
        .finalize(operation_id, &serde_json::to_vec(&retired).map_err(err)?)
        .map_err(err)?;
    Ok(Engine::report(operation_id, &retired))
}
pub struct Engine {
    pub release: ReleaseId,
    pub payload: AdapterPayload,
    state_root: PathBuf,
    store: OperationStore,
    owner_id: String,
    _owner_lock: FileLock,
    _session_guard: crate::maintenance::SessionGuard,
    pending: Mutex<HashMap<String, Pending>>,
    running: Mutex<Registry>,
}
impl Engine {
    pub fn open(
        release: ReleaseId,
        payload: AdapterPayload,
        state_root: PathBuf,
    ) -> Result<Arc<Self>, String> {
        let session_guard = crate::maintenance::SessionGuard::acquire(&state_root).map_err(err)?;
        let store = OperationStore::open(&state_root).map_err(err)?;
        let state_root = state_root.canonicalize().map_err(err)?;
        let (owner_id, owner_lock) = create_owner(&state_root)?;
        Ok(Arc::new(Self {
            release,
            payload,
            state_root,
            store,
            owner_id,
            _owner_lock: owner_lock,
            _session_guard: session_guard,
            pending: Mutex::new(HashMap::new()),
            running: Mutex::new(Registry::default()),
        }))
    }
    pub fn guard(&self, expected: &str) -> Result<(), String> {
        self.release.require(expected).map_err(err)
    }
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }
    pub(crate) fn owner_id(&self) -> &str {
        &self.owner_id
    }
    pub async fn inspect_code(
        self: &Arc<Self>,
        args: CodeInspectArgs,
        request_cancelled: impl std::future::Future<Output = ()> + Send,
    ) -> Result<Value, String> {
        self.inspect_source(
            args.expected_release_id,
            args.project_root,
            args.request.into_shared(),
            request_cancelled,
        )
        .await
    }

    async fn inspect_source(
        self: &Arc<Self>,
        expected_release_id: String,
        project_root: PathBuf,
        request: crate::code_inspect::InspectRequest<crate::code_inspect::Language>,
        request_cancelled: impl std::future::Future<Output = ()> + Send,
    ) -> Result<Value, String> {
        self.guard(&expected_release_id)?;
        crate::code_inspect::validate_input(&request)?;
        let configured = self.payload.design.as_ref().ok_or(
            "private code inspection runtime is not configured; no host Node fallback is used",
        )?;
        let payload = crate::code_inspect::Payload {
            node: configured.node.clone(),
            adapter: configured
                .adapter
                .with_file_name(crate::code_inspect::ADAPTER_NAME),
        };
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let sequence = NEXT
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| "code inspection request identity capacity exhausted")?;
        let job_id = format!("op-code-inspect-{}-{sequence:x}", self.owner_id);
        struct CancelOnDrop(Arc<AtomicBool>);
        impl Drop for CancelOnDrop {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let _cancel_on_drop = CancelOnDrop(cancel.clone());
        let (sender, mut receiver) = tokio::sync::oneshot::channel();
        {
            let mut registry = self.running.lock().map_err(err)?;
            if registry.closed {
                return Err("connection is shutting down; no new inspection may start".into());
            }
            registry
                .active
                .retain(|_, control| !control.handle.is_finished());
            if registry.active.len() >= 8 {
                return Err("connection job capacity is busy".into());
            }
            let this = self.clone();
            let task_id = job_id.clone();
            let task_cancel = cancel.clone();
            let handle = tokio::spawn(async move {
                let result = this
                    .execute_source_request(project_root, request, task_id, payload, task_cancel)
                    .await;
                let _ = sender.send(result);
            });
            registry.active.insert(
                job_id,
                Control {
                    cancel: cancel.clone(),
                    handle,
                },
            );
        }
        tokio::select! {
            biased;
            _ = request_cancelled => {
                cancel.store(true, Ordering::Release);
                receiver.await.map_err(|_| "code inspection ownership task ended without a result".to_owned())??;
                Err("code inspection cancelled after cleanup".into())
            },
            result = &mut receiver => result.map_err(|_| "code inspection ownership task ended without a result".to_owned())?,
        }
    }

    async fn execute_source_request(
        &self,
        project_root: PathBuf,
        request: crate::code_inspect::InspectRequest<crate::code_inspect::Language>,
        job_id: String,
        payload: crate::code_inspect::Payload,
        cancel: Arc<AtomicBool>,
    ) -> Result<Value, String> {
        let release = self.release.as_str().to_owned();
        let snapshot = tokio::task::spawn_blocking(move || {
            crate::code_inspect::read_source(&project_root, request, release, job_id)
        })
        .await
        .map_err(|_| "code inspection source snapshot failed")??;
        if cancel.load(Ordering::Acquire) {
            return Err("code inspection cancelled before dispatch".into());
        }
        let snapshot = match snapshot {
            crate::code_inspect::SourceRead::Current(snapshot) => *snapshot,
            crate::code_inspect::SourceRead::Changed(response) => return Ok(response),
        };
        let executable = std::env::current_exe().map_err(err)?;
        let outcome = crate::worker::run_code_inspect_owned(
            &executable,
            crate::worker::CodeInspectWork {
                state_root: self.state_root.clone(),
                payload,
                request: snapshot.request.clone(),
            },
            cancel.clone(),
        )
        .await
        .map_err(err)?;
        if cancel.load(Ordering::Acquire) {
            return Err("code inspection cancelled after cleanup".into());
        }
        tokio::task::spawn_blocking(move || snapshot.finish(outcome.result))
            .await
            .map_err(|_| "code inspection source verification failed")?
    }

    // Caller holds the registry lock shared with start/cancel/shutdown.
    fn issue_preparation(
        &self,
        plan: ArtifactPlan,
        operation: PendingOperation,
        receipt: Receipt,
    ) -> Result<Value, String> {
        let mut pending = self.pending.lock().map_err(err)?;
        let expired: Vec<_> = pending
            .iter()
            .filter(|(_, intent)| intent.receipt.expires_at < receipt.created_at)
            .map(|(id, _)| id.clone())
            .collect();
        self.retire_unused(
            &mut pending,
            &expired,
            "prepared intent expired without starting",
            false,
        )?;
        let selected_bytes = plan.source_bytes().map_or(0, <[u8]>::len);
        let held_bytes = pending
            .values()
            .try_fold(selected_bytes, |total, item| {
                total.checked_add(item.plan.source_bytes().map_or(0, <[u8]>::len))
            })
            .ok_or("prepared source byte capacity exceeded")?;
        if pending.len() >= 64 || held_bytes > 64 * 1024 * 1024 {
            return Err("prepared-intent capacity exhausted; cancel unused preparations".into());
        }
        let store = self.store.lock().map_err(err)?;
        let id = store
            .issue(&serde_json::to_vec(&receipt).map_err(err)?)
            .map_err(err)?;
        pending.insert(
            id.clone(),
            Pending {
                plan,
                operation,
                receipt: receipt.clone(),
            },
        );
        Ok(Self::report(&id, &receipt))
    }
    /// Caller holds the registry lock, excluding start, cancel and shutdown.
    /// Only a still-owned in-memory intent proves that no task was launched.
    /// A missing/drifted receipt retains both its pending entry and protection.
    fn retire_unused(
        &self,
        pending: &mut HashMap<String, Pending>,
        ids: &[String],
        reason: &str,
        during_shutdown: bool,
    ) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let store = if during_shutdown {
            self.store.lock_bounded()
        } else {
            self.store.lock()
        }
        .map_err(err)?;
        for id in ids {
            let original = &pending
                .get(id)
                .ok_or("unused intent ownership unavailable; retain recovery evidence")?
                .receipt;
            let saved: Receipt =
                serde_json::from_slice(&store.read(id).map_err(err)?).map_err(err)?;
            let mut receipt = original.clone();
            receipt.state.cancel();
            receipt.error = Some(reason.into());
            receipt.scratch_cleanup = Some("not_started".into());
            if saved != *original {
                // The only accepted retry is an exact original-derived terminal
                // payload. Keep its first reason and completion age unchanged.
                receipt.error = saved.error.clone();
                if saved != receipt
                    || !matches!(
                        saved.error.as_deref(),
                        Some(
                            "prepared intent expired without starting"
                                | "connection closed before prepared intent started"
                                | "prepared intent cancelled without starting"
                        )
                    )
                {
                    return Err("unused intent evidence changed; retain protected receipt".into());
                }
            }
            store
                .finalize(id, &serde_json::to_vec(&receipt).map_err(err)?)
                .map_err(err)?;
            pending.remove(id);
        }
        Ok(())
    }
    fn report(id: &str, r: &Receipt) -> Value {
        json!({"operation_id":id,"kind":r.kind,"release_id":r.release_id,"project_root":r.project_root,"source_path":r.source_path,"destination_path":r.destination_path,"source_revision":r.source_revision,"status":r.state.status,"effect":r.state.effect,"artifact_revision":r.artifact_revision,"error":r.error,"expires_at":r.expires_at,"scratch_cleanup":r.scratch_cleanup,"queue_wait_ms":r.queue_wait_ms})
    }
    fn load(&self, args: &JobArgs) -> Result<Receipt, String> {
        let scope = ProjectScope::open(&args.project_root).map_err(err)?;
        let store = self.store.lock().map_err(err)?;
        let receipt: Receipt = serde_json::from_slice(
            &store
                .read(&args.operation_id)
                .map_err(|_| "operation receipt unavailable; never retry this ID as new intent")?,
        )
        .map_err(|_| "operation receipt invalid; inspect output without replay")?;
        if receipt.schema != 2 {
            return Err("operation receipt schema is unsupported; do not replay".into());
        }
        if receipt.project_root != scope.root() {
            return Err("operation belongs to another selected project".into());
        }
        Ok(receipt)
    }
    fn save(&self, id: &str, r: &Receipt) -> Result<(), String> {
        // A status inspection must not turn a running owned job into cancellation.
        // Wait only for bounded native lock contention; never replay a write.
        let store = self.store.lock_bounded().map_err(err)?;
        let current: Receipt =
            serde_json::from_slice(&store.read(id).map_err(err)?).map_err(err)?;
        // A late cancellation must not erase terminal/publication evidence after
        // that evidence allowed disposal of private scratch.
        if (current.state.terminal() && current.state != r.state)
            || (current.state.effect == Effect::Confirmed && r.state.effect != Effect::Confirmed)
            || (current.state.effect == Effect::Unknown && r.state.effect == Effect::NotAttempted)
        {
            return Err("operation state advanced; inspect the retained receipt".into());
        }
        let bytes = serde_json::to_vec(r).map_err(err)?;
        // Terminal execution alone does not establish that its child, stage
        // and publication recovery are resolved. Only this owner can attest
        // that the scratch disposal boundary completed.
        let recovery_complete = r.state.terminal()
            && r.state.effect != Effect::Unknown
            && (r.scratch_cleanup.as_deref() == Some("complete")
                || (current.state.status == Status::Prepared
                    && r.state.status == Status::Cancelled
                    && r.scratch_cleanup.as_deref() == Some("not_started")));
        if recovery_complete {
            store.finalize(id, &bytes).map_err(err)
        } else {
            store.update(id, &bytes).map_err(err)
        }
    }
    pub fn inspect(&self, args: JobArgs) -> Result<Value, String> {
        // Deliberately read-only across release changes. Expected ID is returned,
        // never adopted; this route cannot spawn, publish or replay anything.
        let mut r = self.load(&args)?;
        if !r.state.terminal() && r.owner_id != self.owner_id && self.owner_gone(&r.owner_id)? {
            r.state.owner_lost();
        }
        Ok(Self::report(&args.operation_id, &r))
    }
    pub fn cancel(&self, args: JobArgs) -> Result<Value, String> {
        self.guard(&args.expected_release_id)?;
        // Hold the same admission boundary as start while deciding whether a
        // prepared cancellation has never launched work.
        let registry = self.running.lock().map_err(err)?;
        let mut r = self.load(&args)?;
        if r.release_id != self.release.as_str() {
            return Err("operation was owned by another release; inspect it instead".into());
        }
        if r.state.terminal() {
            return Ok(Self::report(&args.operation_id, &r));
        }
        if r.owner_id != self.owner_id {
            return Err(
                "only the owning connection may cancel this operation; inspect its receipt".into(),
            );
        }
        if let Some(control) = registry.active.get(&args.operation_id) {
            control.cancel.store(true, Ordering::Release);
            r.state.cancel();
            self.save(&args.operation_id, &r)?;
        }
        if r.state.status == Status::Prepared {
            let mut pending = self.pending.lock().map_err(err)?;
            self.retire_unused(
                &mut pending,
                std::slice::from_ref(&args.operation_id),
                "prepared intent cancelled without starting",
                false,
            )?;
            r = self.load(&args)?;
        }
        Ok(Self::report(&args.operation_id, &r))
    }
    pub fn start_diagram(self: &Arc<Self>, args: JobArgs) -> Result<Value, String> {
        self.start_operation(args, OperationKind::Diagram)
    }
    pub fn start_design(self: &Arc<Self>, args: JobArgs) -> Result<Value, String> {
        self.start_operation(args, OperationKind::Design)
    }
    fn start_operation(
        self: &Arc<Self>,
        args: JobArgs,
        kind: OperationKind,
    ) -> Result<Value, String> {
        self.guard(&args.expected_release_id)?;
        // Check, receipt transition and task registration share one boundary
        // with shutdown. No await occurs while this connection lock is held.
        let mut registry = self.running.lock().map_err(err)?;
        if registry.closed {
            return Err("connection is shutting down; no new jobs may start".into());
        }
        let mut r = self.load(&args)?;
        if r.release_id != self.release.as_str() {
            return Err(
                "operation belongs to another release; inspect it before new intent".into(),
            );
        }
        if r.kind != kind {
            return Err("operation kind does not match this start tool".into());
        }
        if r.state.status != Status::Prepared {
            return self.inspect(args);
        }
        if r.owner_id != self.owner_id {
            return Err(
                "prepared intent belongs to another connection; inspect it rather than replaying"
                    .into(),
            );
        }
        let mut pending_intents = self.pending.lock().map_err(err)?;
        let original = &pending_intents
            .get(&args.operation_id)
            .ok_or("prepared intent belongs to another or disconnected session; do not replay it")?
            .receipt;
        if r != *original {
            return Err("prepared intent receipt changed; retain evidence without starting".into());
        }
        if now()? > r.expires_at {
            self.retire_unused(
                &mut pending_intents,
                std::slice::from_ref(&args.operation_id),
                "prepared intent expired without starting",
                false,
            )?;
            return Err(
                "prepared intent expired; inspect the source and prepare new intent".into(),
            );
        }
        registry
            .active
            .retain(|_, control| !control.handle.is_finished());
        if registry.active.len() >= 8 {
            return Err(
                "connection job capacity is busy; inspect existing work before starting more"
                    .into(),
            );
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| "owned jobs require an active Tokio runtime")?;
        let pending = pending_intents
            .remove(&args.operation_id)
            .ok_or("prepared intent unavailable")?;
        drop(pending_intents);
        r.state.start().map_err(err)?;
        self.save(&args.operation_id, &r)?;
        let report = Self::report(&args.operation_id, &r);
        let this = self.clone();
        let id = args.operation_id.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let handle = runtime.spawn(async move {
            let _ = this.execute(&id, pending, &mut r, worker_cancel).await;
            // Failed terminal persistence retains scratch and the last durable
            // running/unknown receipt; neither authorizes a fresh start.
        });
        registry
            .active
            .insert(args.operation_id, Control { cancel, handle });
        Ok(report)
    }
    async fn execute(
        &self,
        id: &str,
        pending: Pending,
        r: &mut Receipt,
        cancel: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let mut scratch = None;
        let outcome = self
            .execute_inner(id, pending, r, cancel, &mut scratch)
            .await;
        self.finish_execution(id, r, scratch, outcome)
    }

    fn finish_execution(
        &self,
        id: &str,
        r: &mut Receipt,
        scratch: Option<JobScratch>,
        outcome: Result<(), String>,
    ) -> Result<(), String> {
        if let Err(message) = &outcome {
            r.error = Some(message.chars().take(2048).collect());
            r.state.fail();
        }
        r.scratch_cleanup = Some(
            if scratch.is_some() {
                "retained: terminal confirmation pending"
            } else {
                "not_owned: no scratch cleanup attempted"
            }
            .into(),
        );
        self.save(id, r)?;
        if let Some(scratch) = scratch {
            r.scratch_cleanup = Some(scratch.dispose(r.state));
            // Cleanup is secondary: never turn a confirmed published artifact
            // into a failed/no-effect result because disposal or its notice failed.
            let _ = self.save(id, r);
        }
        outcome
    }

    fn record_running(
        &self,
        id: &str,
        receipt: &mut Receipt,
        cancel: &AtomicBool,
        wait_ms: u64,
    ) -> Result<(), String> {
        // Cancellation and admission receipts serialize under the same registry
        // lock. A late progress frame cannot overwrite Cancelling with Running.
        let _registry = self.running.lock().map_err(err)?;
        if cancel.load(Ordering::Acquire) {
            receipt.state.cancel();
        } else {
            receipt.state.admitted().map_err(err)?;
        }
        receipt.queue_wait_ms = Some(wait_ms);
        self.save(id, receipt)
    }

    async fn execute_inner(
        &self,
        id: &str,
        pending: Pending,
        r: &mut Receipt,
        cancel: Arc<AtomicBool>,
        owned: &mut Option<JobScratch>,
    ) -> Result<(), String> {
        *owned = Some(match &pending.operation {
            PendingOperation::Diagram(_) => {
                JobScratch::create_named(&self.state_root, id, "source.d2", "result.svg")?
            }
            PendingOperation::Design(operation) => {
                let (source_name, output_name) = match operation {
                    crate::design::DesignOperation::TokensCompile {
                        format: crate::design::TokenFormat::Css,
                    } => ("source.json", "result.css"),
                    crate::design::DesignOperation::TokensCompile {
                        format: crate::design::TokenFormat::Json,
                    } => ("source.json", "result.json"),
                    crate::design::DesignOperation::SvgOptimize => ("source.svg", "result.svg"),
                    crate::design::DesignOperation::SvgRenderPng { .. } => {
                        ("source.svg", "result.png")
                    }
                    crate::design::DesignOperation::RasterTransform { .. } => (
                        match operation.input_extension() {
                            "png" => "source.png",
                            "jpg" => "source.jpg",
                            "webp" => "source.webp",
                            "avif" => "source.avif",
                            _ => unreachable!(),
                        },
                        match operation.output_extension() {
                            "png" => "result.png",
                            "jpg" => "result.jpg",
                            "webp" => "result.webp",
                            "avif" => "result.avif",
                            _ => unreachable!(),
                        },
                    ),
                    crate::design::DesignOperation::OpenPencil { operation } => match operation {
                        crate::openpencil::OpenPencilOperation::Create => {
                            ("source.json", "result.fig")
                        }
                        crate::openpencil::OpenPencilOperation::Patch { .. } => {
                            ("source.fig", "result.fig")
                        }
                        crate::openpencil::OpenPencilOperation::ExportSvg { .. } => {
                            ("source.fig", "result.svg")
                        }
                    },
                };
                JobScratch::create_named(&self.state_root, id, source_name, output_name)?
            }
        });
        let scratch = owned.as_mut().ok_or("job scratch ownership unavailable")?;
        let input = scratch.directory.join(scratch.source_name);
        let output = scratch.directory.join(scratch.output_name);
        scratch.write_source(
            pending
                .plan
                .source_bytes()
                .ok_or("missing selected source")?,
        )?;
        scratch.child_cleanup_confirmed = false;
        let source_revision = r.source_revision.clone();
        let mut on_running = |wait_ms| self.record_running(id, r, &cancel, wait_ms);
        let (expected_revision, output_limit) = match &pending.operation {
            PendingOperation::Diagram(layout) => {
                let executable = self
                    .payload
                    .diagram
                    .clone()
                    .ok_or("private D2 runtime is not configured")?;
                let result = crate::worker::run_diagram_owned(
                    &std::env::current_exe().map_err(err)?,
                    crate::worker::DiagramWork {
                        state_root: self.state_root.clone(),
                        executable,
                        request: crate::diagram::D2RenderRequest {
                            protocol_version: crate::diagram::PROTOCOL_VERSION,
                            release_id: self.release.as_str().into(),
                            job_id: id.into(),
                            job_root: scratch.directory.clone(),
                            input_path: input.clone(),
                            source_revision: source_revision.clone(),
                            output_path: output.clone(),
                            layout: *layout,
                            output_format: crate::diagram::D2OutputFormat::Svg,
                            timeout_ms: 30_000,
                            max_output_bytes: crate::diagram::MAX_OUTPUT_BYTES,
                        },
                    },
                    cancel.clone(),
                    &mut on_running,
                )
                .await;
                scratch.child_cleanup_confirmed = result.as_ref().map_or_else(
                    |error| error.cleanup_confirmed,
                    |value| value.cleanup_confirmed,
                );
                if cancel.load(Ordering::Acquire) {
                    r.state.cancel();
                }
                let outcome = result.map_err(err)?;
                match outcome.result {
                    crate::diagram::D2RenderResult::Staged {
                        output_revision, ..
                    } => (output_revision, crate::diagram::MAX_OUTPUT_BYTES),
                    crate::diagram::D2RenderResult::Refused { reason } => {
                        return Err(format!("D2 render refused: {reason:?}"));
                    }
                }
            }
            PendingOperation::Design(operation) => {
                let payload = match operation {
                    crate::design::DesignOperation::OpenPencil { .. } => {
                        crate::design::DesignLeafPayload::OpenPencil(
                            self.payload
                                .openpencil
                                .clone()
                                .ok_or("private OpenPencil shape-file runtime is not configured")?,
                        )
                    }
                    _ => crate::design::DesignLeafPayload::Standard(
                        self.payload
                            .design
                            .clone()
                            .ok_or("private design runtime is not configured")?,
                    ),
                };
                let result = crate::worker::run_design_owned(
                    &std::env::current_exe().map_err(err)?,
                    crate::worker::DesignWork {
                        state_root: self.state_root.clone(),
                        payload,
                        request: crate::design::DesignRequest {
                            protocol_version: crate::design::PROTOCOL_VERSION,
                            release_id: self.release.as_str().into(),
                            job_id: id.into(),
                            job_root: scratch.directory.clone(),
                            input_path: input.clone(),
                            source_revision: source_revision.clone(),
                            output_path: output.clone(),
                            operation: operation.clone(),
                            timeout_ms: 30_000,
                            max_output_bytes: crate::design::MAX_OUTPUT_BYTES,
                        },
                    },
                    cancel.clone(),
                    &mut on_running,
                )
                .await;
                scratch.child_cleanup_confirmed = result.as_ref().map_or_else(
                    |error| error.cleanup_confirmed,
                    |value| value.cleanup_confirmed,
                );
                if cancel.load(Ordering::Acquire) {
                    r.state.cancel();
                }
                let outcome = result.map_err(err)?;
                if !outcome.cleanup_confirmed {
                    return Err("design worker cleanup is unconfirmed".into());
                }
                (outcome.output_revision, crate::design::MAX_OUTPUT_BYTES)
            }
        };
        let output_stamp = ScratchStamp::file(&output)?;
        let source = ProjectScope::open(&scratch.directory)
            .map_err(err)?
            .read_bytes(&output, output_limit)
            .map_err(err)?;
        if revision(&source.bytes) != expected_revision {
            return Err("adapter output differs from its verified result".into());
        }
        if !output_stamp.same_file(&ScratchStamp::file(&output)?) {
            return Err("scratch output changed during verification".into());
        }
        scratch.output = Some(output_stamp);
        if cancel.load(Ordering::Acquire) {
            r.state.cancel();
            return Err("operation cancelled before publication".into());
        }
        let staged = pending.plan.stage_bytes(&source.bytes).map_err(err)?;
        if cancel.load(Ordering::Acquire) {
            r.state.cancel();
            return Err("cancelled before publication".into());
        }
        r.artifact_revision = Some(revision(&source.bytes));
        r.state.begin_publish().map_err(err)?;
        self.save(id, r)?; // Persist unknown before the first possible external publication.
        let saved = staged.publish().map_err(err)?;
        let actual = saved.inspect_bytes(output_limit).map_err(err)?;
        if revision(&actual) != r.artifact_revision.as_deref().unwrap_or("") {
            return Err("saved revision differs; publication outcome requires inspection".into());
        }
        r.state.confirm().map_err(err)?;
        Ok(())
    }
    fn owner_gone(&self, id: &str) -> Result<bool, String> {
        if id.is_empty()
            || id.len() > 100
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid stored owner identity".into());
        }
        let path = self.state_root.join("owners").join(id);
        match fs::OpenOptions::new().read(true).write(true).open(path) {
            Ok(file) => match file.try_lock() {
                Ok(()) => {
                    let _lock = FileLock::acquired(file);
                    Ok(true)
                }
                Err(fs::TryLockError::WouldBlock) => Ok(false),
                Err(_) => Err("owner status unavailable; do not replay".into()),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(true),
            Err(_) => Err("owner status unavailable; do not replay".into()),
        }
    }
    pub async fn shutdown(&self) {
        let controls: Vec<_> = {
            let mut registry = self.running.lock().unwrap_or_else(|e| e.into_inner());
            registry.closed = true;
            if let Ok(mut pending) = self.pending.lock() {
                let unused: Vec<_> = pending.keys().cloned().collect();
                // Shutdown cannot claim recovery when persistence fails. Leave
                // failed entries protected; a later shutdown can retry them.
                let _ = self.retire_unused(
                    &mut pending,
                    &unused,
                    "connection closed before prepared intent started",
                    true,
                );
            }
            for control in registry.active.values() {
                control.cancel.store(true, Ordering::Release);
            }
            registry
                .active
                .drain()
                .map(|(_, control)| control)
                .collect()
        };
        for c in controls {
            let _ = c.handle.await;
        }
    }
}
/// Narrow ownership evidence for this live execution. It is deliberately not a
/// crash-recovery manifest or permission to scan/reclaim other job directories.
struct ScratchStamp {
    created: Option<SystemTime>,
    modified: Option<SystemTime>,
    bytes: u64,
    #[cfg(unix)]
    native: (u64, u64),
}
impl ScratchStamp {
    fn capture(metadata: &fs::Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Self {
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            bytes: metadata.len(),
            #[cfg(unix)]
            native: (metadata.dev(), metadata.ino()),
        }
    }
    fn directory(path: &Path) -> Result<Self, String> {
        let metadata = fs::symlink_metadata(path).map_err(err)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("scratch directory ownership is unavailable".into());
        }
        check_private(&metadata)?;
        Ok(Self::capture(&metadata))
    }
    fn file(path: &Path) -> Result<Self, String> {
        let metadata = fs::symlink_metadata(path).map_err(err)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("scratch file ownership is unavailable".into());
        }
        check_private(&metadata)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.nlink() != 1 {
                return Err("scratch file aliases are unsupported".into());
            }
        }
        Ok(Self::capture(&metadata))
    }
    fn same_identity(&self, other: &Self) -> bool {
        #[cfg(unix)]
        {
            self.native == other.native && self.created == other.created
        }
        #[cfg(not(unix))]
        {
            let _ = other;
            false
        }
    }
    fn same_file(&self, other: &Self) -> bool {
        self.same_identity(other) && self.modified == other.modified && self.bytes == other.bytes
    }
}
fn check_private(metadata: &fs::Metadata) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("scratch entries must be private".into());
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    Ok(())
}
struct JobScratch {
    state_root: PathBuf,
    state_stamp: ScratchStamp,
    parent_stamp: ScratchStamp,
    directory: PathBuf,
    directory_stamp: ScratchStamp,
    source_name: &'static str,
    output_name: &'static str,
    source: Option<ScratchStamp>,
    output: Option<ScratchStamp>,
    child_cleanup_confirmed: bool,
}
impl JobScratch {
    fn write_source(&mut self, bytes: &[u8]) -> Result<(), String> {
        use std::io::Write;
        let input = self.directory.join(self.source_name);
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&input).map_err(err)?;
        self.source = Some(ScratchStamp::capture(&file.metadata().map_err(err)?));
        file.write_all(bytes).map_err(err)?;
        file.sync_all().map_err(err)?;
        let written = ScratchStamp::capture(&file.metadata().map_err(err)?);
        drop(file);
        if !written.same_file(&ScratchStamp::file(&input)?) {
            return Err("scratch source changed during preparation".into());
        }
        self.source = Some(written);
        Ok(())
    }
    fn create_named(
        state_root: &Path,
        id: &str,
        source_name: &'static str,
        output_name: &'static str,
    ) -> Result<Self, String> {
        let raster_pair = source_name
            .strip_prefix("source.")
            .zip(output_name.strip_prefix("result."))
            .is_some_and(|(source, output)| {
                matches!(source, "png" | "jpg" | "webp" | "avif")
                    && matches!(output, "png" | "jpg" | "webp" | "avif")
            });
        if !raster_pair
            && !matches!(
                (source_name, output_name),
                ("source.txt", "result.txt")
                    | ("source.d2", "result.svg")
                    | ("source.json", "result.css")
                    | ("source.json", "result.json")
                    | ("source.svg", "result.svg")
                    | ("source.svg", "result.png")
                    | ("source.json", "result.fig")
                    | ("source.fig", "result.fig")
                    | ("source.fig", "result.svg")
            )
        {
            return Err("unsupported private scratch file pair".into());
        }
        if !id.starts_with("op-")
            || id.len() > 100
            || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid private job identity".into());
        }
        let state_stamp = ScratchStamp::directory(state_root)?;
        let parent = state_root.join("jobs");
        private_directory(&parent)?;
        let parent_stamp = ScratchStamp::directory(&parent)?;
        let directory = parent.join(id);
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(&directory)
            .map_err(|_| "job scratch reservation failed; existing data was retained")?;
        let directory_stamp = ScratchStamp::directory(&directory)?;
        Ok(Self {
            state_root: state_root.into(),
            state_stamp,
            parent_stamp,
            directory,
            directory_stamp,
            source_name,
            output_name,
            source: None,
            output: None,
            child_cleanup_confirmed: true,
        })
    }
    fn preflight(&self) -> Result<(), &'static str> {
        for (path, expected) in [
            (&self.state_root, &self.state_stamp),
            (&self.state_root.join("jobs"), &self.parent_stamp),
            (&self.directory, &self.directory_stamp),
        ] {
            let actual = ScratchStamp::directory(path)
                .map_err(|_| "retained: scratch directory ownership changed")?;
            if !expected.same_identity(&actual) {
                return Err("retained: scratch directory ownership changed");
            }
        }
        for entry in fs::read_dir(&self.directory)
            .map_err(|_| "retained: scratch directory inspection failed")?
        {
            let entry = entry.map_err(|_| "retained: scratch directory inspection failed")?;
            if entry.file_name() != self.source_name && entry.file_name() != self.output_name {
                return Err("retained: unexpected private scratch entry");
            }
        }
        for (name, expected) in [
            (self.source_name, &self.source),
            (self.output_name, &self.output),
        ] {
            let path = self.directory.join(name);
            match (expected, fs::symlink_metadata(&path)) {
                (None, Err(error)) if error.kind() == std::io::ErrorKind::NotFound => (),
                (Some(expected), Ok(_)) => {
                    let actual = ScratchStamp::file(&path)
                        .map_err(|_| "retained: scratch file ownership changed")?;
                    if !expected.same_file(&actual) {
                        return Err("retained: scratch file ownership changed");
                    }
                }
                (None, Ok(_)) => return Err("retained: unverified or partial scratch output"),
                _ => return Err("retained: scratch file inspection unavailable"),
            }
        }
        Ok(())
    }
    fn dispose(self, state: JobState) -> String {
        if !self.child_cleanup_confirmed {
            return "retained: child cleanup unconfirmed".into();
        }
        if !state.terminal() || state.effect == Effect::Unknown {
            return "retained: publication outcome unresolved".into();
        }
        if let Err(reason) = self.preflight() {
            return reason.into();
        }
        // Keep the checked parent open through deletion so completion cannot
        // be persisted ahead of the directory entry removal.
        let parent = match fs::File::open(self.state_root.join("jobs")) {
            Ok(parent) => parent,
            Err(_) => return "retained: scratch parent flush unavailable".into(),
        };
        if !parent.metadata().ok().is_some_and(|metadata| {
            self.parent_stamp
                .same_identity(&ScratchStamp::capture(&metadata))
        }) {
            return "retained: scratch parent ownership changed".into();
        }
        for (name, present) in [
            (self.source_name, self.source.is_some()),
            (self.output_name, self.output.is_some()),
        ] {
            if present && fs::remove_file(self.directory.join(name)).is_err() {
                return "retained: scratch deletion incomplete".into();
            }
        }
        if fs::remove_dir(&self.directory).is_err() {
            return "retained: scratch directory deletion incomplete".into();
        }
        if parent.sync_all().is_err() {
            return "retained: scratch deletion durability unconfirmed".into();
        }
        "complete".into()
    }
}

fn private_directory(path: &Path) -> Result<(), String> {
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    match b.create(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(err(e)),
    }
    let meta = fs::symlink_metadata(path).map_err(err)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("job directory is not a private directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.mode() & 0o077 != 0 {
            return Err("job directory must be private".into());
        }
    }
    Ok(())
}

fn create_owner(state: &Path) -> Result<(String, FileLock), String> {
    use std::sync::atomic::AtomicU64;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let dir = state.join("owners");
    private_directory(&dir)?;
    for _ in 0..64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(err)?
            .as_nanos();
        let id = format!(
            "owner-{nanos:x}-{:x}-{:x}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let mut opts = fs::OpenOptions::new();
        opts.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        match opts.open(dir.join(&id)) {
            Ok(file) => {
                file.try_lock().map_err(err)?;
                return Ok((id, FileLock::acquired(file)));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(err(e)),
        }
    }
    Err("could not reserve unique connection owner".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn maintenance_retires_only_expired_exact_unused_design_preparation() {
        let f = Fixture::new();
        let state = f.root.join("retirement-state");
        private_directory(&state).unwrap();
        let engine = Engine::open(
            f.engine.release.clone(),
            f.engine.payload.clone(),
            state.clone(),
        )
        .unwrap();
        let issued = engine.prepare_design(f.args.clone()).unwrap()["operation_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let job = JobArgs {
            expected_release_id: "test-a".into(),
            project_root: f.args.project_root.clone(),
            operation_id: issued.clone(),
        };
        let mut receipt = engine.load(&job).unwrap();
        receipt.created_at = now().unwrap() - 301;
        receipt.expires_at = receipt.created_at + 300;
        let store = OperationStore::open_existing(&state).unwrap();
        let expired = store
            .lock()
            .unwrap()
            .issue(&serde_json::to_vec(&receipt).unwrap())
            .unwrap();
        assert!(
            retire_expired_preparation(&state, &f.engine.release, &f.args.project_root, &expired)
                .is_err()
        );
        drop(engine); // abrupt owner loss, without graceful retirement
        let live_bytes = fs::read(state.join("operations").join(format!("{issued}.json"))).unwrap();
        assert!(
            retire_expired_preparation(&state, &f.engine.release, &f.args.project_root, &issued)
                .is_err()
        );
        assert_eq!(
            fs::read(state.join("operations").join(format!("{issued}.json"))).unwrap(),
            live_bytes
        );
        assert!(
            retire_expired_preparation(
                &state,
                &ReleaseId::parse("other").unwrap(),
                &f.args.project_root,
                &expired
            )
            .is_err()
        );
        let result =
            retire_expired_preparation(&state, &f.engine.release, &f.args.project_root, &expired)
                .unwrap();
        assert_eq!(result["status"], "cancelled");
        assert_eq!(result["effect"], "not_attempted");
        let path = state.join("operations").join(format!("{expired}.json"));
        let completed = fs::read(&path).unwrap();
        retire_expired_preparation(&state, &f.engine.release, &f.args.project_root, &expired)
            .unwrap();
        assert_eq!(fs::read(path).unwrap(), completed);
        assert!(!state.join("jobs").exists());
        assert!(!f.args.project_root.join("output.css").exists());
        assert!(
            retire_expired_preparation(
                &state,
                &f.engine.release,
                &f.args.project_root,
                "op-missing"
            )
            .is_err()
        );
    }

    #[test]
    fn maintenance_design_retains_drift_started_scratch_and_unknown_effects() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let mut original = f.engine.load(&args).unwrap();
        original.created_at = now().unwrap() - 301;
        original.expires_at = original.created_at + 300;
        let state = f.root.join("retirement-state");
        private_directory(&state).unwrap();
        let store = OperationStore::open(&state).unwrap();
        for mutation in 0..5 {
            let locked = store.lock().unwrap();
            let id = locked
                .issue(&serde_json::to_vec(&original).unwrap())
                .unwrap();
            let mut changed = original.clone();
            match mutation {
                0 => changed.destination_path = f.args.project_root.join("changed-target.txt"),
                1 => changed.intent_revision = revision(b"different intent"),
                2 => changed.state.status = Status::Queued,
                3 => changed.state.effect = Effect::Unknown,
                _ => {
                    private_directory(&state.join("jobs")).unwrap();
                    private_directory(&state.join("jobs").join(&id)).unwrap();
                }
            }
            locked
                .update(&id, &serde_json::to_vec(&changed).unwrap())
                .unwrap();
            drop(locked);
            let path = state.join("operations").join(format!("{id}.json"));
            let before = fs::read(&path).unwrap();
            assert!(
                retire_expired_preparation(&state, &f.engine.release, &f.args.project_root, &id)
                    .is_err()
            );
            assert_eq!(fs::read(path).unwrap(), before);
        }
        assert_eq!(fs::read_dir(state.join("jobs")).unwrap().count(), 1);
    }
    pub(super) struct Fixture {
        root: PathBuf,
        pub(super) engine: Arc<Engine>,
        pub(super) args: DesignPrepareArgs,
    }
    impl Fixture {
        pub(super) fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mirket-engine-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            private_directory(&root).unwrap();
            let root = root.canonicalize().unwrap();
            let state = root.join("state");
            let project = root.join("project");
            private_directory(&state).unwrap();
            private_directory(&project).unwrap();
            fs::write(project.join("input.json"), "private original phrase").unwrap();
            // These tests exercise intent/receipt boundaries only, not child execution.
            let executable = std::env::current_exe().unwrap();
            let mut payload = AdapterPayload::explicit(&executable).unwrap();
            payload.design = Some(crate::design::DesignPayload {
                node: executable.clone(),
                adapter: executable.clone(),
                font: executable.clone(),
                raster_module: None,
            });
            let engine = Engine::open(ReleaseId::parse("test-a").unwrap(), payload, state).unwrap();
            let args = DesignPrepareArgs {
                expected_release_id: "test-a".into(),
                project_root: project,
                source_path: "input.json".into(),
                destination_path: "output.css".into(),
                expected_revision: revision(b"private original phrase"),
                operation: crate::design::DesignOperation::TokensCompile {
                    format: crate::design::TokenFormat::Css,
                },
            };
            Self { root, engine, args }
        }
        pub(super) fn prepare_design(&self) -> JobArgs {
            let value = self.engine.prepare_design(self.args.clone()).unwrap();
            JobArgs {
                expected_release_id: "test-a".into(),
                project_root: self.args.project_root.clone(),
                operation_id: value["operation_id"].as_str().unwrap().into(),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
    #[test]
    fn retired_receipt_gc_never_makes_old_id_a_fresh_intent_after_reconnect() {
        let f = Fixture::new();
        let old = f.prepare_design();
        f.engine.cancel(old.clone()).unwrap();
        let path = f
            .engine
            .state_root
            .join("operations")
            .join(format!("{}.json", old.operation_id));
        let mut envelope: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(envelope["retention"]["state"], "retirable");
        // Clock fixture only: production finalization cannot move completion time.
        envelope["retention"]["completed_at"] = json!(now().unwrap() - 8 * 24 * 60 * 60);
        fs::write(&path, serde_json::to_vec(&envelope).unwrap()).unwrap();
        let reconnected = Engine::open(
            ReleaseId::parse("test-a").unwrap(),
            f.engine.payload.clone(),
            f.engine.state_root.clone(),
        )
        .unwrap();
        let fresh = reconnected.prepare_design(f.args.clone()).unwrap();
        let fresh_id = fresh["operation_id"].as_str().unwrap();
        assert_ne!(fresh_id, old.operation_id);
        assert!(!path.exists());
        assert!(
            reconnected
                .start_design(old.clone())
                .unwrap_err()
                .contains("unavailable")
        );
        assert!(reconnected.inspect(old).is_err());
        let fresh_args = JobArgs {
            expected_release_id: "test-a".into(),
            project_root: f.args.project_root.clone(),
            operation_id: fresh_id.into(),
        };
        assert_eq!(
            reconnected.load(&fresh_args).unwrap().state.status,
            Status::Prepared
        );
        assert!(reconnected.pending.lock().unwrap().contains_key(fresh_id));
        reconnected.cancel(fresh_args).unwrap();
        assert!(!f.args.project_root.join("output.css").exists());
        assert!(!f.engine.state_root.join("jobs").exists());
    }

    #[test]
    fn expired_intent_cannot_be_started_as_fresh() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let mut r = f.engine.load(&args).unwrap();
        r.expires_at = 0;
        // Move both sides of the fixture clock; production binds expiry exactly.
        f.engine
            .pending
            .lock()
            .unwrap()
            .get_mut(&args.operation_id)
            .unwrap()
            .receipt = r.clone();
        f.engine.save(&args.operation_id, &r).unwrap();
        assert!(
            f.engine
                .start_design(args.clone())
                .unwrap_err()
                .contains("expired")
        );
        let receipt = f.engine.load(&args).unwrap();
        assert_eq!(receipt.state.status, Status::Cancelled);
        assert_eq!(receipt.scratch_cleanup.as_deref(), Some("not_started"));
        assert!(
            f.engine
                .store
                .lock()
                .unwrap()
                .update(&args.operation_id, b"changed")
                .is_err()
        );
        assert!(!f.args.project_root.join("output.css").exists());
    }
    #[test]
    fn missing_receipt_and_unknown_schema_never_authorize_start() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let mut r = f.engine.load(&args).unwrap();
        r.schema = 99;
        f.engine.save(&args.operation_id, &r).unwrap();
        assert!(
            f.engine
                .start_design(args.clone())
                .unwrap_err()
                .contains("unsupported")
        );
        fs::remove_file(
            f.engine
                .state_root
                .join("operations")
                .join(format!("{}.json", args.operation_id)),
        )
        .unwrap();
        assert!(
            f.engine
                .start_design(args)
                .unwrap_err()
                .contains("unavailable")
        );
        assert!(!f.args.project_root.join("output.css").exists());
    }
    #[test]
    fn minimal_receipt_does_not_persist_design_source() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let bytes = f
            .engine
            .store
            .lock()
            .unwrap()
            .read(&args.operation_id)
            .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("private original phrase"));
        assert!(text.contains("sha256:"));
    }
    #[test]
    fn stale_release_prepare_has_no_receipt_or_job_effect() {
        let f = Fixture::new();
        let mut args = f.args.clone();
        args.expected_release_id = "old".into();
        assert!(f.engine.prepare_design(args).is_err());
        assert_eq!(
            fs::read_dir(f.engine.state_root.join("operations"))
                .unwrap()
                .count(),
            0
        );
        assert!(!f.engine.state_root.join("jobs").exists());
    }
    #[cfg(unix)]
    fn scratch(fixture: &Fixture, id: &str, verified_output: bool) -> JobScratch {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut scratch =
            JobScratch::create_named(&fixture.engine.state_root, id, "source.json", "result.css")
                .unwrap();
        for name in if verified_output {
            vec!["source.json", "result.css"]
        } else {
            vec!["source.json"]
        } {
            let path = scratch.directory.join(name);
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
                .unwrap();
            file.write_all(b"owned private bytes").unwrap();
            file.sync_all().unwrap();
            let stamp = ScratchStamp::file(&path).unwrap();
            if name == "source.json" {
                scratch.source = Some(stamp);
            } else {
                scratch.output = Some(stamp);
            }
        }
        scratch
    }

    #[test]
    fn expired_pending_capacity_recovers_with_retirable_receipts_without_replaying() {
        let f = Fixture::new();
        let operations: Vec<_> = (0..64).map(|_| f.prepare_design()).collect();
        assert!(
            f.engine
                .prepare_design(f.args.clone())
                .unwrap_err()
                .contains("capacity")
        );
        for args in &operations {
            let mut receipt = f.engine.load(args).unwrap();
            receipt.expires_at = 0;
            f.engine.save(&args.operation_id, &receipt).unwrap();
        }
        for pending in f.engine.pending.lock().unwrap().values_mut() {
            pending.receipt.expires_at = 0;
        }
        let fresh = f.prepare_design();
        assert_eq!(f.engine.pending.lock().unwrap().len(), 1);
        assert_ne!(fresh.operation_id, operations[0].operation_id);
        for args in &operations {
            assert_eq!(f.engine.load(args).unwrap().state.status, Status::Cancelled);
            assert!(
                f.engine
                    .store
                    .lock()
                    .unwrap()
                    .update(&args.operation_id, b"changed")
                    .is_err()
            );
        }
        assert_eq!(
            f.engine.start_design(operations[0].clone()).unwrap()["status"],
            "cancelled"
        );
        assert_eq!(f.engine.pending.lock().unwrap().len(), 1);
        assert!(!f.engine.state_root.join("jobs").exists());
    }

    #[cfg(unix)]
    #[test]
    fn terminal_confirmed_cleanup_preserves_outputs_receipts_and_other_jobs() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let owned = scratch(&f, &args.operation_id, true);
        let path = owned.directory.clone();
        let other = scratch(&f, "op-other-owner", true);
        let other_path = other.directory.clone();
        fs::write(f.args.project_root.join("output.css"), "published artifact").unwrap();
        let mut receipt = f.engine.load(&args).unwrap();
        receipt.state.start().unwrap();
        receipt.state.admitted().unwrap();
        receipt.state.begin_publish().unwrap();
        receipt.state.confirm().unwrap();
        receipt.artifact_revision = Some(revision(b"published artifact"));
        f.engine
            .finish_execution(&args.operation_id, &mut receipt, Some(owned), Ok(()))
            .unwrap();
        assert!(!path.exists());
        assert!(other_path.join("source.json").exists());
        assert_eq!(
            fs::read_to_string(f.args.project_root.join("output.css")).unwrap(),
            "published artifact"
        );
        assert_eq!(
            fs::read_to_string(f.args.project_root.join("input.json")).unwrap(),
            "private original phrase"
        );
        let saved = f.engine.load(&args).unwrap();
        assert_eq!(saved.state.effect, Effect::Confirmed);
        assert_eq!(saved.state.status, Status::Succeeded);
        assert_eq!(saved.scratch_cleanup.as_deref(), Some("complete"));
        assert_eq!(saved.artifact_revision, receipt.artifact_revision);
        assert!(
            f.engine
                .store
                .lock()
                .unwrap()
                .update(&args.operation_id, b"reopened")
                .is_err()
        );
    }

    #[cfg(unix)]
    #[test]
    fn confirmed_child_failure_without_output_disposes_known_source() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let owned = scratch(&f, &args.operation_id, false);
        let path = owned.directory.clone();
        let mut receipt = f.engine.load(&args).unwrap();
        receipt.state.start().unwrap();
        receipt.state.admitted().unwrap();
        assert!(
            f.engine
                .finish_execution(
                    &args.operation_id,
                    &mut receipt,
                    Some(owned),
                    Err("adapter refused before output".into())
                )
                .is_err()
        );
        assert!(!path.exists());
        let saved = f.engine.load(&args).unwrap();
        assert_eq!(saved.state.status, Status::Failed);
        assert_eq!(saved.state.effect, Effect::NotAttempted);
        assert_eq!(saved.scratch_cleanup.as_deref(), Some("complete"));
    }

    #[cfg(unix)]
    #[test]
    fn unknown_child_or_publication_retains_all_recovery_material() {
        for child_unknown in [true, false] {
            let f = Fixture::new();
            let args = f.prepare_design();
            let mut owned = scratch(&f, &args.operation_id, true);
            owned.child_cleanup_confirmed = !child_unknown;
            let path = owned.directory.clone();
            let mut receipt = f.engine.load(&args).unwrap();
            receipt.state.start().unwrap();
            receipt.state.admitted().unwrap();
            if !child_unknown {
                receipt.state.begin_publish().unwrap();
            }
            assert!(
                f.engine
                    .finish_execution(
                        &args.operation_id,
                        &mut receipt,
                        Some(owned),
                        Err("unknown outcome".into())
                    )
                    .is_err()
            );
            assert!(path.join("source.json").exists());
            assert!(path.join("result.css").exists());
            let saved = f.engine.load(&args).unwrap();
            assert!(
                saved
                    .scratch_cleanup
                    .as_ref()
                    .unwrap()
                    .starts_with("retained:")
            );
            // Unresolved recovery remains mutable by its owner, not an
            // immutable completion eligible for storage-pressure eviction.
            let mut recovering = saved;
            recovering.error = Some("recovery evidence still protected".into());
            f.engine.save(&args.operation_id, &recovering).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn partial_output_and_unexpected_entries_are_not_deleted() {
        for partial in [true, false] {
            let f = Fixture::new();
            let args = f.prepare_design();
            let owned = scratch(&f, &args.operation_id, false);
            let path = owned.directory.clone();
            fs::write(
                path.join(if partial {
                    "result.css"
                } else {
                    "unrecognized.txt"
                }),
                b"unverified bytes",
            )
            .unwrap();
            let mut receipt = f.engine.load(&args).unwrap();
            receipt.state.start().unwrap();
            receipt.state.admitted().unwrap();
            f.engine
                .finish_execution(
                    &args.operation_id,
                    &mut receipt,
                    Some(owned),
                    Err("failure".into()),
                )
                .unwrap_err();
            assert!(path.join("source.json").exists());
            assert_eq!(fs::read_dir(&path).unwrap().count(), 2);
            assert!(
                f.engine
                    .load(&args)
                    .unwrap()
                    .scratch_cleanup
                    .unwrap()
                    .contains(if partial { "partial" } else { "unexpected" })
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn terminal_receipt_failure_retains_scratch_before_disposal() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let owned = scratch(&f, &args.operation_id, true);
        let path = owned.directory.clone();
        let mut receipt = f.engine.load(&args).unwrap();
        receipt.state.start().unwrap();
        receipt.state.admitted().unwrap();
        fs::write(
            f.engine.state_root.join("operations/unresolved.tmp"),
            b"preserve",
        )
        .unwrap();
        assert!(
            f.engine
                .finish_execution(
                    &args.operation_id,
                    &mut receipt,
                    Some(owned),
                    Err("failure".into())
                )
                .is_err()
        );
        assert!(path.join("source.json").exists());
        assert!(path.join("result.css").exists());
        assert_eq!(f.engine.load(&args).unwrap().state.status, Status::Prepared);
        assert_eq!(
            fs::read(f.engine.state_root.join("operations/unresolved.tmp")).unwrap(),
            b"preserve"
        );
    }

    #[cfg(unix)]
    #[test]
    fn substitution_and_aliases_refuse_disposal_without_touching_targets() {
        use std::os::unix::fs::symlink;
        for alias in ["symlink", "hardlink", "replacement"] {
            let f = Fixture::new();
            let args = f.prepare_design();
            let owned = scratch(&f, &args.operation_id, true);
            let path = owned.directory.clone();
            let output = path.join("result.css");
            let outside = f.args.project_root.join("do-not-remove.txt");
            fs::write(&outside, "independent bytes").unwrap();
            fs::remove_file(&output).unwrap();
            match alias {
                "symlink" => symlink(&outside, &output).unwrap(),
                "hardlink" => fs::hard_link(&outside, &output).unwrap(),
                _ => {
                    fs::write(&output, "substituted bytes").unwrap();
                }
            }
            assert!(
                owned
                    .dispose(JobState {
                        status: Status::Succeeded,
                        effect: Effect::Confirmed
                    })
                    .starts_with("retained:")
            );
            assert!(path.join("source.json").exists());
            assert!(fs::symlink_metadata(&output).is_ok());
            assert_eq!(fs::read_to_string(outside).unwrap(), "independent bytes");
        }
    }

    #[cfg(unix)]
    #[test]
    fn existing_job_directory_is_never_adopted_or_cleaned() {
        let f = Fixture::new();
        let original = scratch(&f, "op-existing", true);
        assert!(
            JobScratch::create_named(
                &f.engine.state_root,
                "op-existing",
                "source.json",
                "result.css"
            )
            .is_err()
        );
        assert!(original.directory.join("source.json").exists());
        assert!(
            JobScratch::create_named(
                &f.engine.state_root,
                "../outside",
                "source.json",
                "result.css"
            )
            .is_err()
        );
    }

    #[test]
    fn late_cancel_cannot_erase_terminal_publication_evidence() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let mut stale = f.engine.load(&args).unwrap();
        stale.state.start().unwrap();
        stale.state.admitted().unwrap();
        f.engine.save(&args.operation_id, &stale).unwrap();
        let mut terminal = stale.clone();
        terminal.state.begin_publish().unwrap();
        terminal.state.confirm().unwrap();
        terminal.artifact_revision = Some(revision(b"saved"));
        f.engine.save(&args.operation_id, &terminal).unwrap();
        stale.state.cancel();
        assert!(f.engine.save(&args.operation_id, &stale).is_err());
        let actual = f.engine.load(&args).unwrap();
        assert_eq!(actual.state.status, Status::Succeeded);
        assert_eq!(actual.state.effect, Effect::Confirmed);
        assert_eq!(actual.artifact_revision, terminal.artifact_revision);
    }

    #[cfg(unix)]
    #[test]
    fn retained_scratch_does_not_downgrade_a_published_success() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let owned = scratch(&f, &args.operation_id, true);
        fs::write(owned.directory.join("unexpected"), b"preserve").unwrap();
        let mut receipt = f.engine.load(&args).unwrap();
        receipt.state.start().unwrap();
        receipt.state.admitted().unwrap();
        receipt.state.begin_publish().unwrap();
        receipt.state.confirm().unwrap();
        f.engine
            .finish_execution(&args.operation_id, &mut receipt, Some(owned), Ok(()))
            .unwrap();
        let actual = f.engine.load(&args).unwrap();
        assert_eq!(actual.state.status, Status::Succeeded);
        assert_eq!(actual.state.effect, Effect::Confirmed);
        assert!(actual.scratch_cleanup.unwrap().contains("unexpected"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cancelled_before_worker_launch_cleans_private_source_after_receipt() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let pending = f
            .engine
            .pending
            .lock()
            .unwrap()
            .remove(&args.operation_id)
            .unwrap();
        let mut receipt = f.engine.load(&args).unwrap();
        receipt.state.start().unwrap();
        receipt.state.admitted().unwrap();
        f.engine.save(&args.operation_id, &receipt).unwrap();
        let result = f
            .engine
            .execute(
                &args.operation_id,
                pending,
                &mut receipt,
                Arc::new(AtomicBool::new(true)),
            )
            .await;
        assert!(result.is_err());
        assert!(
            !f.engine
                .state_root
                .join("jobs")
                .join(&args.operation_id)
                .exists()
        );
        let actual = f.engine.load(&args).unwrap();
        assert_eq!(actual.state.status, Status::Cancelled);
        assert_eq!(actual.scratch_cleanup.as_deref(), Some("complete"));
        assert!(!f.args.project_root.join("output.css").exists());
    }

    #[tokio::test]
    async fn concurrent_starts_reserve_exactly_eight_before_any_task_runs() {
        let f = Fixture::new();
        let args: Vec<_> = (0..12).map(|_| f.prepare_design()).collect();
        let barrier = Arc::new(std::sync::Barrier::new(args.len()));
        let runtime = tokio::runtime::Handle::current();
        let starts: Vec<_> = args
            .iter()
            .cloned()
            .map(|args| {
                let engine = f.engine.clone();
                let barrier = barrier.clone();
                let runtime = runtime.clone();
                std::thread::spawn(move || {
                    let _entered = runtime.enter();
                    barrier.wait();
                    engine.start_design(args)
                })
            })
            .collect();
        // This is the current-thread runtime: no child task can execute until
        // the test yields. Native threads still race actual start registration.
        let results: Vec<_> = starts.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 8);
        assert_eq!(f.engine.running.lock().unwrap().active.len(), 8);
        for (args, result) in args.iter().zip(&results) {
            if let Err(message) = result {
                assert!(message.contains("capacity"));
                assert_eq!(f.engine.load(args).unwrap().state.status, Status::Prepared);
                assert!(
                    f.engine
                        .pending
                        .lock()
                        .unwrap()
                        .contains_key(&args.operation_id)
                );
            }
        }
        f.engine.shutdown().await;
        assert!(f.engine.running.lock().unwrap().active.is_empty());
        for (args, result) in args.iter().zip(results) {
            if result.is_ok() {
                assert_eq!(f.engine.load(args).unwrap().state.status, Status::Cancelled);
            }
        }
        assert!(!f.args.project_root.join("output.css").exists());
    }

    #[tokio::test]
    async fn shutdown_retires_only_owned_unused_intents_and_preserves_unknown_effects() {
        let f = Fixture::new();
        let unused = f.prepare_design();
        let uncertain = f.prepare_design();
        // An executing operation is no longer owned by the pending-intent map.
        f.engine
            .pending
            .lock()
            .unwrap()
            .remove(&uncertain.operation_id);
        let mut receipt = f.engine.load(&uncertain).unwrap();
        receipt.state.status = Status::Interrupted;
        receipt.state.effect = Effect::Unknown;
        f.engine.save(&uncertain.operation_id, &receipt).unwrap();
        let unknown_path = f
            .engine
            .state_root
            .join("operations")
            .join(format!("{}.json", uncertain.operation_id));
        let unknown_before = fs::read(&unknown_path).unwrap();
        f.engine.shutdown().await;
        assert!(f.engine.pending.lock().unwrap().is_empty());
        assert_eq!(
            f.engine.load(&unused).unwrap().state.status,
            Status::Cancelled
        );
        assert!(
            f.engine
                .store
                .lock()
                .unwrap()
                .update(&unused.operation_id, b"changed")
                .is_err()
        );
        assert_eq!(fs::read(unknown_path).unwrap(), unknown_before);
        // Same payload remains mutable: unknown effects must not become GC candidates.
        let store = f.engine.store.lock().unwrap();
        let bytes = store.read(&uncertain.operation_id).unwrap();
        let mut changed = bytes.clone();
        changed.push(b' ');
        store.update(&uncertain.operation_id, &changed).unwrap();
        drop(store);
        assert!(
            f.engine
                .prepare_design(f.args.clone())
                .unwrap_err()
                .contains("shutting down")
        );
        assert!(!f.engine.state_root.join("jobs").exists());
    }

    #[tokio::test]
    async fn prepared_identity_drift_never_starts_or_retires_and_keeps_original_pending() {
        for field in [
            "destination",
            "source",
            "project",
            "intent_revision",
            "source_revision",
            "created_at",
            "expires_at",
        ] {
            let f = Fixture::new();
            let args = f.prepare_design();
            let original = f.engine.load(&args).unwrap();
            let mut drifted = original.clone();
            match field {
                "destination" => drifted.destination_path = f.args.project_root.join("other.txt"),
                "source" => drifted.source_path = f.args.project_root.join("other-source.txt"),
                "project" => drifted.project_root = f.root.clone(),
                "intent_revision" => drifted.intent_revision = revision(b"different intent"),
                "source_revision" => drifted.source_revision = revision(b"different source"),
                "created_at" => drifted.created_at -= 1,
                "expires_at" => drifted.expires_at += 1,
                _ => unreachable!(),
            }
            f.engine.save(&args.operation_id, &drifted).unwrap();
            let path = f
                .engine
                .state_root
                .join("operations")
                .join(format!("{}.json", args.operation_id));
            let before = fs::read(&path).unwrap();
            assert!(f.engine.start_design(args.clone()).is_err(), "{field}");
            assert!(f.engine.cancel(args.clone()).is_err(), "{field}");
            f.engine.shutdown().await;
            assert_eq!(fs::read(path).unwrap(), before, "{field}");
            assert!(
                f.engine
                    .pending
                    .lock()
                    .unwrap()
                    .get(&args.operation_id)
                    .is_some_and(|pending| pending.receipt == original),
                "{field}"
            );
            // Drift remains protected rather than being accepted as completion.
            let store = f.engine.store.lock().unwrap();
            let mut bytes = store.read(&args.operation_id).unwrap();
            bytes.push(b' ');
            store.update(&args.operation_id, &bytes).unwrap();
            assert!(!f.engine.state_root.join("jobs").exists());
            assert!(!f.args.project_root.join("output.css").exists());
            assert!(!f.args.project_root.join("other.txt").exists());
        }
    }

    #[tokio::test]
    async fn failed_unused_retirement_preserves_pending_and_protected_evidence_for_retry() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let original = f.engine.load(&args).unwrap();
        let mut drifted = original.clone();
        drifted.state.effect = Effect::Unknown;
        f.engine.save(&args.operation_id, &drifted).unwrap();
        f.engine.shutdown().await;
        assert!(
            f.engine
                .pending
                .lock()
                .unwrap()
                .contains_key(&args.operation_id)
        );
        assert_eq!(f.engine.load(&args).unwrap().state.effect, Effect::Unknown);
        // Test fixture repair using the protected store API, not a production
        // permission to regress uncertain effect evidence.
        f.engine
            .store
            .lock()
            .unwrap()
            .update(&args.operation_id, &serde_json::to_vec(&original).unwrap())
            .unwrap();
        f.engine.shutdown().await;
        assert!(f.engine.pending.lock().unwrap().is_empty());
        assert_eq!(
            f.engine.load(&args).unwrap().state.status,
            Status::Cancelled
        );
        assert!(
            f.engine
                .store
                .lock()
                .unwrap()
                .update(&args.operation_id, b"changed")
                .is_err()
        );
    }

    #[tokio::test]
    async fn unused_retirement_retries_lost_acknowledgement_without_rewriting_completion() {
        let f = Fixture::new();
        let args = f.prepare_design();
        let mut receipt = f.engine.load(&args).unwrap();
        receipt.state.cancel();
        receipt.error = Some("prepared intent expired without starting".into());
        receipt.scratch_cleanup = Some("not_started".into());
        // Model durable finalization followed by a lost acknowledgement: Pending
        // is intentionally still present, so shutdown must retry the same proof.
        f.engine
            .store
            .lock()
            .unwrap()
            .finalize(&args.operation_id, &serde_json::to_vec(&receipt).unwrap())
            .unwrap();
        let path = f
            .engine
            .state_root
            .join("operations")
            .join(format!("{}.json", args.operation_id));
        let completed = fs::read(&path).unwrap();
        f.engine.shutdown().await;
        assert!(f.engine.pending.lock().unwrap().is_empty());
        assert_eq!(fs::read(path).unwrap(), completed);
        assert!(!f.engine.state_root.join("jobs").exists());
    }

    #[tokio::test]
    async fn concurrent_start_and_shutdown_never_leave_an_untracked_task() {
        use std::future::Future;
        use std::task::{Context, Waker};
        for _ in 0..16 {
            let f = Fixture::new();
            let args = f.prepare_design();
            let thread_args = args.clone();
            let barrier = Arc::new(std::sync::Barrier::new(2));
            let thread_barrier = barrier.clone();
            let engine = f.engine.clone();
            let runtime = tokio::runtime::Handle::current();
            let starter = std::thread::spawn(move || {
                let _entered = runtime.enter();
                thread_barrier.wait();
                engine.start_design(thread_args)
            });
            let mut shutdown = std::pin::pin!(f.engine.shutdown());
            barrier.wait();
            // Poll shutdown's synchronous close/cancel transition concurrently
            // with start, before allowing the current-thread executor to run jobs.
            let pending = shutdown
                .as_mut()
                .poll(&mut Context::from_waker(Waker::noop()))
                .is_pending();
            let result = starter.join().unwrap();
            if pending {
                shutdown.await;
            }
            let registry = f.engine.running.lock().unwrap();
            assert!(registry.closed);
            assert!(registry.active.is_empty());
            drop(registry);
            let receipt = f.engine.load(&args).unwrap();
            match result {
                Ok(_) => assert_eq!(receipt.state.status, Status::Cancelled),
                Err(message) => {
                    assert!(message.contains("shutting down"));
                    assert_eq!(receipt.state.status, Status::Cancelled);
                    assert_eq!(receipt.scratch_cleanup.as_deref(), Some("not_started"));
                }
            }
            assert!(
                f.engine
                    .start_design(args)
                    .unwrap_err()
                    .contains("shutting down")
            );
            assert!(!f.args.project_root.join("output.css").exists());
        }
    }
    #[test]
    fn running_progress_waits_for_transient_receipt_inspection() {
        let fixture = Fixture::new();
        let args = fixture.prepare_design();
        let mut receipt = fixture.engine.load(&args).unwrap();
        receipt.state.start().unwrap();
        fixture.engine.save(&args.operation_id, &receipt).unwrap();
        let held = fixture.engine.store.lock().unwrap();
        let reader = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(60));
            drop(held);
        });
        let result = fixture.engine.record_running(
            &args.operation_id,
            &mut receipt,
            &AtomicBool::new(false),
            1,
        );
        reader.join().unwrap();
        result.expect("ordinary receipt inspection must not cancel the running worker");
        assert_eq!(
            fixture.engine.load(&args).unwrap().state.status,
            Status::Running
        );
    }

    #[test]
    fn running_progress_persists_wait_but_never_regresses_cancellation() {
        for cancelled in [false, true] {
            let fixture = Fixture::new();
            let args = fixture.prepare_design();
            let mut receipt = fixture.engine.load(&args).unwrap();
            receipt.state.start().unwrap();
            fixture.engine.save(&args.operation_id, &receipt).unwrap();
            assert_eq!(
                fixture.engine.inspect(args.clone()).unwrap()["status"],
                "queued"
            );
            let cancel = AtomicBool::new(cancelled);
            fixture
                .engine
                .record_running(&args.operation_id, &mut receipt, &cancel, 125)
                .unwrap();
            let current = fixture.engine.load(&args).unwrap();
            assert_eq!(current.queue_wait_ms, Some(125));
            assert_eq!(
                current.state.status,
                if cancelled {
                    Status::Cancelling
                } else {
                    Status::Running
                }
            );
            assert_eq!(current.state.effect, Effect::NotAttempted);
            if !cancelled {
                assert!(
                    fixture
                        .engine
                        .record_running(&args.operation_id, &mut receipt, &cancel, 126)
                        .is_err()
                );
                assert_eq!(fixture.engine.load(&args).unwrap().queue_wait_ms, Some(125));
            }
        }
    }
}
