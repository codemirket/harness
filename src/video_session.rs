//! Finite video intents for one connection, using the shared receipt, admission
//! and artifact owners. The registry retains intent ownership; it is not a
//! second scheduler. Unknown or disconnected work is never replayed.
use crate::artifacts::{ArtifactPlan, PublicationMode, VideoFileRevision};
use crate::engine::{Engine, revision};
use crate::job_state::{Effect, JobState, Status};
use crate::operation_store::OperationStore;
use crate::owned_scratch::{DirectoryStamp, OwnedScratch};
use crate::scope::ProjectScope;
use crate::video::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_secs())
        .map_err(err)
}

#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Intent {
    release_id: String,
    owner_id: String,
    project_root: PathBuf,
    source_root: PathBuf,
    entry_file: PathBuf,
    sources: Vec<SourceFile>,
    destination_path: PathBuf,
    profile: VideoProfile,
}
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: u8,
    kind: String,
    intent: Intent,
    intent_revision: String,
    created_at: u64,
    expires_at: u64,
    state: JobState,
    artifact_revision: Option<String>,
    artifact_bytes: Option<usize>,
    scratch_path: Option<PathBuf>,
    scratch_cleanup: Option<String>,
    queue_wait_ms: Option<u64>,
    evidence: Option<Value>,
    error: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    diagnostics: Vec<VideoDiagnostic>,
}
impl Receipt {
    fn issued(&self) -> Self {
        let mut r = self.clone();
        r.state = JobState::default();
        r.artifact_revision = None;
        r.artifact_bytes = None;
        r.scratch_path = None;
        r.scratch_cleanup = None;
        r.queue_wait_ms = None;
        r.evidence = None;
        r.error = None;
        r.diagnostics.clear();
        r
    }
}
struct Pending {
    plan: ArtifactPlan,
    receipt: Receipt,
    bytes: Vec<Vec<u8>>,
}
struct Control {
    cancel: Arc<AtomicBool>,
    handle: tokio::task::JoinHandle<()>,
}
#[derive(Default)]
struct Registry {
    closed: bool,
    pending: HashMap<String, Pending>,
    active: HashMap<String, Control>,
}

pub struct VideoManager {
    engine: Arc<Engine>,
    registry: Mutex<Registry>,
    payload_identity: Mutex<Option<Vec<String>>>,
}
impl VideoManager {
    pub fn new(engine: Arc<Engine>) -> Arc<Self> {
        Arc::new(Self {
            engine,
            registry: Mutex::new(Registry::default()),
            payload_identity: Mutex::new(None),
        })
    }
    fn store(&self) -> Result<OperationStore, String> {
        OperationStore::open_existing(self.engine.state_root()).map_err(err)
    }
    fn save(&self, id: &str, r: &Receipt) -> Result<(), String> {
        let s = self.store()?.lock().map_err(err)?;
        let current: Receipt = serde_json::from_slice(&s.read(id).map_err(err)?).map_err(err)?;
        if (current.state.terminal() && current.state != r.state)
            || (current.state.effect == Effect::Confirmed && r.state.effect != Effect::Confirmed)
            || (current.state.effect == Effect::Unknown && r.state.effect == Effect::NotAttempted)
        {
            return Err("video operation state advanced; inspect the retained receipt".into());
        }
        let bytes = serde_json::to_vec(r).map_err(err)?;
        if r.state.terminal()
            && r.state.effect != Effect::Unknown
            && (r.scratch_cleanup.as_deref() == Some("removed_verified_owned_files")
                || (r.scratch_path.is_none()
                    && r.scratch_cleanup.as_deref() == Some("not_started")))
        {
            s.finalize(id, &bytes)
        } else {
            s.update(id, &bytes)
        }
        .map_err(err)
    }
    fn begin_child_ownership(
        &self,
        id: &str,
        r: &mut Receipt,
        lease: &mut crate::leases::SlotLease,
    ) -> Result<(), String> {
        // Persist the prelaunch state before creating the durable heavy-slot
        // marker. A contended receipt lock must never strand a marker for work
        // that could not yet launch.
        r.scratch_cleanup = Some("launch_pending".into());
        self.save(id, r)?;
        if let Err(error) = lease.mark_unresolved(id) {
            // Marker creation may fail after writing its file. Retain the
            // scratch and protected receipt until that ownership is inspected.
            r.scratch_cleanup = Some("marker_unconfirmed".into());
            return Err(err(error));
        }
        r.scratch_cleanup = Some("children_unconfirmed".into());
        Ok(())
    }
    fn load(&self, a: &VideoJobArgs) -> Result<Receipt, String> {
        self.engine.guard(&a.expected_release_id)?;
        let scope = ProjectScope::open(&a.project_root).map_err(err)?;
        let s = self.store()?.lock().map_err(err)?;
        let r: Receipt = serde_json::from_slice(&s.read(&a.operation_id).map_err(
            |_| "video receipt unavailable; missing evidence does not authorize replay",
        )?)
        .map_err(|_| "not a video receipt")?;
        if r.schema != 1
            || r.kind != "video_export"
            || r.intent.release_id != self.engine.release.as_str()
            || r.intent.project_root != scope.root()
            || r.created_at.checked_add(300) != Some(r.expires_at)
            || r.intent_revision != revision(&serde_json::to_vec(&r.intent).map_err(err)?)
            || !s
                .matches_issued_payload(
                    &a.operation_id,
                    &serde_json::to_vec(&r.issued()).map_err(err)?,
                )
                .map_err(err)?
        {
            return Err(
                "video receipt differs from its original issued scope/release/intent".into(),
            );
        }
        Ok(r)
    }
    fn report(&self, id: &str, r: &Receipt) -> Value {
        let mut state = r.state;
        if r.intent.owner_id != self.engine.owner_id() {
            state.owner_lost();
        }
        json!({"operation_id":id,"release_id":r.intent.release_id,"project_root":r.intent.project_root,
            "destination_path":r.intent.destination_path,"profile":r.intent.profile,"sources":r.intent.sources,
            "status":state.status,"effect":state.effect,"artifact_revision":r.artifact_revision,
            "artifact_bytes":r.artifact_bytes,"queue_wait_ms":r.queue_wait_ms,"scratch_path":r.scratch_path,
            "scratch_cleanup":r.scratch_cleanup,"output_budget_bytes":MAX_OUTPUT_BYTES,"scratch_budget_bytes":MAX_WORK_BYTES,"evidence":r.evidence,"error":r.error,"diagnostics":r.diagnostics,"expires_at":r.expires_at})
    }
    pub fn prepare(&self, a: VideoPrepareArgs) -> Result<Value, String> {
        self.engine.guard(&a.expected_release_id)?;
        let payload = self
            .engine
            .payload
            .video
            .as_ref()
            .ok_or("private video runtime is not configured")?;
        let current = crate::process_video::payload_identity(payload)?;
        let mut identity = self.payload_identity.lock().map_err(err)?;
        if identity
            .as_ref()
            .is_some_and(|expected| *expected != current)
        {
            return Err("video payload changed after its first preparation".into());
        }
        *identity = Some(current);
        drop(identity);
        a.profile.validate()?;
        if !crate::design::is_revision(&a.expected_revision)
            || a.assets.len() >= MAX_FILES
            || a.source_path.extension().and_then(|v| v.to_str()) != Some("html")
            || a.destination_path.extension().and_then(|v| v.to_str())
                != Some(a.profile.format.extension())
        {
            return Err("video requires an HTML revision, bounded selected assets and a profile-matched new output".into());
        }
        let mut registry = self.registry.lock().map_err(err)?;
        if registry.closed {
            return Err("connection is shutting down".into());
        }
        let held: usize = registry
            .pending
            .values()
            .flat_map(|p| &p.bytes)
            .map(Vec::len)
            .sum();
        if registry.pending.len() >= 8 || held >= 64 * 1024 * 1024 {
            return Err("video preparation capacity exhausted; cancel unused preparations".into());
        }
        let scope = ProjectScope::open(&a.project_root).map_err(err)?;
        let source = scope.resolve_existing_file(&a.source_path).map_err(err)?;
        let source_root = source
            .parent()
            .ok_or("HTML source directory missing")?
            .to_owned();
        let entry_file = PathBuf::from(source.file_name().ok_or("HTML source name missing")?);
        if !relative(&entry_file) {
            return Err("HTML entry name is invalid".into());
        }
        let destination = if a.destination_path.is_absolute() {
            a.destination_path
                .strip_prefix(&a.project_root)
                .map(|path| scope.root().join(path))
                .unwrap_or(a.destination_path.clone())
        } else {
            a.destination_path.clone()
        };
        let plan = ArtifactPlan::prepare(
            scope.root(),
            Some(&source),
            &destination,
            PublicationMode::NewCopy,
            MAX_SOURCE_BYTES,
        )
        .map_err(err)?;
        let selected = plan
            .source_bytes()
            .ok_or("HTML selection unavailable")?
            .to_vec();
        if selected.len() > MAX_SOURCE_BYTES || revision(&selected) != a.expected_revision {
            return Err("HTML source changed or exceeds the source budget".into());
        }
        let mut selected_bytes = selected.len();
        let mut files = vec![(
            SourceFile {
                path: entry_file.clone(),
                revision: a.expected_revision,
                bytes: selected.len(),
            },
            selected,
        )];
        for asset in a.assets {
            if !relative(&asset.path)
                || !crate::design::is_revision(&asset.expected_revision)
                || files.iter().any(|(file, _)| file.path == asset.path)
            {
                return Err("video assets must be unique, relative, revision-bound files".into());
            }
            let value = scope
                .read_bytes(
                    &source_root.join(&asset.path),
                    MAX_SOURCE_BYTES - selected_bytes,
                )
                .map_err(err)?;
            if revision(&value.bytes) != asset.expected_revision {
                return Err("video asset revision changed".into());
            }
            selected_bytes += value.bytes.len();
            files.push((
                SourceFile {
                    path: asset.path,
                    revision: asset.expected_revision,
                    bytes: value.bytes.len(),
                },
                value.bytes,
            ));
        }
        files.sort_by(|a, b| a.0.path.cmp(&b.0.path));
        let total: usize = files.iter().map(|(_, b)| b.len()).sum();
        if total > MAX_SOURCE_BYTES || held + total > 64 * 1024 * 1024 {
            return Err("selected video sources exceed the byte budget".into());
        }
        if a.profile
            .captions_path
            .as_ref()
            .is_some_and(|path| !files.iter().any(|(f, _)| &f.path == path))
        {
            return Err("caption cues must be a selected source asset".into());
        }
        let intent = Intent {
            release_id: self.engine.release.as_str().into(),
            owner_id: self.engine.owner_id().into(),
            project_root: scope.root().into(),
            source_root,
            entry_file,
            sources: files.iter().map(|(f, _)| f.clone()).collect(),
            destination_path: plan.destination().into(),
            profile: a.profile,
        };
        let created_at = now()?;
        let r = Receipt {
            schema: 1,
            kind: "video_export".into(),
            intent_revision: revision(&serde_json::to_vec(&intent).map_err(err)?),
            intent,
            created_at,
            expires_at: created_at.checked_add(300).ok_or("invalid clock")?,
            state: JobState::default(),
            artifact_revision: None,
            artifact_bytes: None,
            scratch_path: None,
            scratch_cleanup: None,
            queue_wait_ms: None,
            evidence: None,
            error: None,
            diagnostics: vec![],
        };
        let id = self
            .store()?
            .lock()
            .map_err(err)?
            .issue(&serde_json::to_vec(&r).map_err(err)?)
            .map_err(err)?;
        let report = self.report(&id, &r);
        registry.pending.insert(
            id,
            Pending {
                plan,
                receipt: r,
                bytes: files.into_iter().map(|(_, b)| b).collect(),
            },
        );
        Ok(report)
    }
    pub fn start(self: &Arc<Self>, a: VideoJobArgs) -> Result<Value, String> {
        let mut registry = self.registry.lock().map_err(err)?;
        if registry.closed {
            return Err("connection is shutting down".into());
        }
        let mut r = self.load(&a)?;
        if r.state.status != Status::Prepared {
            return Ok(self.report(&a.operation_id, &r));
        }
        if registry.pending.get(&a.operation_id).map(|p| &p.receipt) != Some(&r) {
            return Err(
                "video preparation belongs to another/disconnected session; inspect without replay"
                    .into(),
            );
        }
        if now()? > r.expires_at {
            return Err("video preparation expired; cancel it and inspect current sources".into());
        }
        registry.active.retain(|_, c| !c.handle.is_finished());
        if registry.active.len() >= 8 {
            return Err("video owner has eight active/waiting jobs".into());
        }
        r.state.start().map_err(str::to_owned)?;
        self.save(&a.operation_id, &r)?;
        let p = registry
            .pending
            .remove(&a.operation_id)
            .ok_or("video preparation missing")?;
        let report = self.report(&a.operation_id, &r);
        let cancel = Arc::new(AtomicBool::new(false));
        let child_cancel = cancel.clone();
        let manager = self.clone();
        let id = a.operation_id;
        let owned_id = id.clone();
        let handle = tokio::spawn(async move {
            manager.execute(owned_id, p, r, child_cancel).await;
        });
        registry.active.insert(id, Control { cancel, handle });
        Ok(report)
    }
    pub fn status(&self, a: VideoJobArgs) -> Result<Value, String> {
        let r = self.load(&a)?;
        Ok(self.report(&a.operation_id, &r))
    }
    pub fn result(&self, a: VideoJobArgs) -> Result<Value, String> {
        let r = self.load(&a)?;
        let mut report = self.report(&a.operation_id, &r);
        if let Some(expected) = &r.artifact_revision {
            let observation = ProjectScope::open(&a.project_root).map_err(err).and_then(|scope|
                VideoFileRevision::inspect(scope.root(), &r.intent.destination_path, MAX_OUTPUT_BYTES).map_err(err))
                .map(|saved| json!({"revision":saved.revision,"bytes":saved.byte_len,"matches_receipt":saved.revision==*expected}));
            report["saved_artifact_observation"] = match observation {
                Ok(v) => v,
                Err(e) => json!({"matches_receipt":false,"error":e}),
            };
        }
        Ok(report)
    }
    pub fn cancel(&self, a: VideoJobArgs) -> Result<Value, String> {
        let mut registry = self.registry.lock().map_err(err)?;
        let mut r = self.load(&a)?;
        if r.intent.owner_id != self.engine.owner_id() {
            return Err("video job belongs to another owner; inspect its receipt".into());
        }
        if r.state.status == Status::Prepared {
            let p = registry
                .pending
                .get(&a.operation_id)
                .ok_or("video preparation ownership unavailable")?;
            if p.receipt != r {
                return Err("video preparation changed".into());
            }
            r.state.cancel();
            r.scratch_cleanup = Some("not_started".into());
            self.save(&a.operation_id, &r)?;
            registry.pending.remove(&a.operation_id);
        } else if let Some(c) = registry.active.get(&a.operation_id) {
            c.cancel.store(true, Ordering::Release);
        }
        // Running receipt writes belong to the executing owner; the atomic flag
        // avoids overwriting a simultaneous commit with a stale cancel read.
        Ok(self.report(&a.operation_id, &r))
    }
    pub async fn shutdown(&self) {
        let handles = {
            let Ok(mut r) = self.registry.lock() else {
                return;
            };
            r.closed = true;
            for (id, p) in r.pending.drain() {
                let mut receipt = p.receipt;
                receipt.state.cancel();
                receipt.scratch_cleanup = Some("not_started".into());
                let _ = self.save(&id, &receipt);
            }
            r.active
                .drain()
                .map(|(_, c)| {
                    c.cancel.store(true, Ordering::Release);
                    c.handle
                })
                .collect::<Vec<_>>()
        };
        for handle in handles {
            let _ = handle.await;
        }
    }
    fn verify_sources(&self, intent: &Intent) -> Result<(), String> {
        let scope = ProjectScope::open(&intent.project_root).map_err(err)?;
        for f in &intent.sources {
            let bytes = scope
                .read_bytes(&intent.source_root.join(&f.path), MAX_SOURCE_BYTES)
                .map_err(err)?
                .bytes;
            if bytes.len() != f.bytes || revision(&bytes) != f.revision {
                return Err("video source tree changed; original output preserved".into());
            }
        }
        Ok(())
    }
    async fn execute(
        self: Arc<Self>,
        id: String,
        p: Pending,
        mut r: Receipt,
        cancel: Arc<AtomicBool>,
    ) {
        let mut scratch = None;
        let result = self
            .execute_inner(&id, p, &mut r, cancel.clone(), &mut scratch)
            .await;
        if let Err(error) = result {
            if cancel.load(Ordering::Acquire) {
                r.state.cancel();
            }
            r.state.fail();
            r.error = Some(error);
        }
        if scratch.is_none() && r.scratch_path.is_none() && r.state.effect == Effect::NotAttempted {
            r.scratch_cleanup = Some("not_started".into());
        }
        if self.save(&id, &r).is_ok()
            && r.state.effect != Effect::Unknown
            && let Some(scratch) = scratch
            && !matches!(
                r.scratch_cleanup.as_deref(),
                Some("children_unconfirmed" | "marker_unconfirmed")
            )
        {
            r.scratch_cleanup = Some(
                match tokio::task::spawn_blocking(move || scratch.cleanup()).await {
                    Ok(Ok(())) => "removed_verified_owned_files".into(),
                    Ok(Err(error)) => format!("retained_cleanup_refused:{error}"),
                    Err(error) => format!("retained_cleanup_worker_failed:{error}"),
                },
            );
            let _ = self.save(&id, &r);
        }
    }
    async fn execute_inner(
        &self,
        id: &str,
        p: Pending,
        r: &mut Receipt,
        cancel: Arc<AtomicBool>,
        owned: &mut Option<VideoScratch>,
    ) -> Result<(), String> {
        let deadline = tokio::time::Instant::now() + Duration::from_millis(MAX_TIMEOUT_MS);
        let started = tokio::time::Instant::now();
        let admission = crate::leases::Admission::open(self.engine.state_root()).map_err(err)?;
        let mut waiting = None;
        let mut lease = loop {
            if cancel.load(Ordering::Acquire) {
                return Err("video cancelled while queued".into());
            }
            if tokio::time::Instant::now() >= deadline {
                return Err("video deadline exceeded while queued".into());
            }
            match crate::resource_admission::admit(self.engine.state_root()) {
                Ok(lease) => break lease,
                Err(
                    crate::resource_admission::Error::Busy
                    | crate::resource_admission::Error::Deferred(_),
                ) => (),
                Err(error) => return Err(error.to_string()),
            }
            if waiting.is_none() {
                waiting = Some(admission.try_waiting().map_err(err)?);
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        };
        drop(waiting);
        self.verify_sources(&r.intent)?;
        if cancel.load(Ordering::Acquire) {
            return Err("video cancelled before launch".into());
        }
        r.state.admitted().map_err(str::to_owned)?;
        r.queue_wait_ms = Some(started.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
        self.save(id, r)?;
        let stamp = DirectoryStamp::inspect(self.engine.state_root())?;
        let scratch =
            OwnedScratch::create(self.engine.state_root(), &stamp, &format!("{id}-video"))?;
        let scratch_path = scratch.path.clone();
        r.scratch_path = Some(scratch_path.clone());
        r.scratch_cleanup = Some("ownership_unconfirmed".into());
        let scratch_stamp = DirectoryStamp::inspect(&scratch_path)?;
        *owned = Some(VideoScratch {
            owned: scratch,
            stamp: scratch_stamp,
        });
        r.scratch_cleanup = Some("not_started".into());
        self.save(id, r)?;
        let input_root = scratch_path.join("source");
        private_dir(&input_root)?;
        for (source, bytes) in r.intent.sources.iter().zip(p.bytes) {
            let output = input_root.join(&source.path);
            if let Some(parent) = output.parent() {
                private_dirs(&input_root, parent)?;
            }
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&output).map_err(err)?;
            file.write_all(&bytes)
                .and_then(|_| file.sync_all())
                .map_err(err)?;
        }
        let payload = self
            .engine
            .payload
            .video
            .as_ref()
            .ok_or("private video runtime unavailable")?;
        if self.payload_identity.lock().map_err(err)?.as_ref()
            != Some(&crate::process_video::payload_identity(payload)?)
        {
            return Err(
                "video executable/adapter payload changed after connection initialization".into(),
            );
        }
        let request = VideoRequest {
            protocol_version: 1,
            release_id: r.intent.release_id.clone(),
            job_id: id.into(),
            job_root: scratch_path.clone(),
            input_root,
            entry_file: r.intent.entry_file.clone(),
            sources: r.intent.sources.clone(),
            output_path: scratch_path
                .join(format!("result.{}", r.intent.profile.format.extension())),
            profile: r.intent.profile.clone(),
            timeout_ms: MAX_TIMEOUT_MS,
            max_output_bytes: MAX_OUTPUT_BYTES,
        };
        self.begin_child_ownership(id, r, &mut lease)?;
        let outcome =
            crate::process_video::run_video_adapter(payload, &request, cancel.clone(), deadline)
                .await;
        if outcome.as_ref().is_ok_and(|o| o.cleanup_confirmed)
            || outcome.as_ref().is_err_and(|e| e.cleanup_confirmed)
        {
            lease.clear_after_cleanup().map_err(err)?;
            r.scratch_cleanup = Some("children_stopped".into());
            self.save(id, r)?;
        }
        let outcome = outcome.map_err(|error| {
            r.diagnostics = error.diagnostics.clone();
            error.to_string()
        })?;
        if !outcome.cleanup_confirmed {
            return Err("video process cleanup unconfirmed; admission marker retained".into());
        }
        if cancel.load(Ordering::Acquire) {
            r.state.cancel();
            return Err("video cancelled before publication".into());
        }
        self.verify_sources(&r.intent)?;
        let stage_path = request.output_path.clone();
        let stage_profile = r.intent.profile.clone();
        let stage_revision = outcome.output_revision.clone();
        let stage_bytes = outcome.output_bytes;
        let stage_cancel = cancel.clone();
        let (staged, output) = tokio::task::spawn_blocking(move || {
            let keep_running =
                || !stage_cancel.load(Ordering::Acquire) && tokio::time::Instant::now() < deadline;
            let output = VideoFileRevision::inspect_checked(
                &scratch_path,
                &stage_path,
                MAX_OUTPUT_BYTES,
                &keep_running,
            )
            .map_err(err)?;
            if output.byte_len != stage_bytes || output.revision != stage_revision {
                return Err("video stage differs from verified adapter output".to_owned());
            }
            crate::process_video::validate_artifact_file(&output, &stage_profile, &keep_running)?;
            let staged = p
                .plan
                .stage_video_file(&output, &keep_running)
                .map_err(err)?;
            Ok((staged, output))
        })
        .await
        .map_err(err)??;
        self.verify_sources(&r.intent)?;
        r.artifact_revision = Some(outcome.output_revision);
        r.artifact_bytes = Some(output.byte_len);
        r.evidence = Some(outcome.evidence);
        if cancel.load(Ordering::Acquire) {
            r.state.cancel();
            return Err("video cancelled before artifact commit".into());
        }
        r.state.begin_publish().map_err(str::to_owned)?;
        self.save(id, r)?;
        let saved = tokio::task::spawn_blocking(move || {
            staged.publish(&|| {
                !cancel.load(Ordering::Acquire) && tokio::time::Instant::now() < deadline
            })
        })
        .await
        .map_err(err)?
        .map_err(err)?;
        if saved.revision != r.artifact_revision.as_deref().unwrap_or("") {
            return Err("saved video differs from commit receipt".into());
        }
        r.state.confirm().map_err(str::to_owned)?;
        self.save(id, r)?;
        Ok(())
    }
}
fn private_dir(path: &Path) -> Result<(), String> {
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b.create(path).map_err(err)
}
fn private_dirs(root: &Path, path: &Path) -> Result<(), String> {
    let relative = path.strip_prefix(root).map_err(err)?;
    let mut current = root.to_owned();
    for c in relative.components() {
        current.push(c);
        match fs::symlink_metadata(&current) {
            Ok(m) if m.is_dir() && !m.file_type().is_symlink() => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => private_dir(&current)?,
            _ => return Err("video source staging ancestor changed".into()),
        }
    }
    Ok(())
}

struct VideoScratch {
    owned: OwnedScratch,
    stamp: DirectoryStamp,
}
impl VideoScratch {
    fn cleanup(self) -> Result<(), String> {
        let root = &self.owned.path;
        self.stamp.matches(root)?;
        let mut pending = vec![root.clone()];
        let mut dirs = vec![];
        let mut files = vec![];
        let mut total = 0u64;
        while let Some(dir) = pending.pop() {
            let stamp = DirectoryStamp::inspect(&dir)?;
            dirs.push((dir.clone(), stamp));
            for entry in fs::read_dir(&dir).map_err(err)? {
                if dirs.len() + files.len() + pending.len() > 25_000 {
                    return Err("work-tree entry budget exceeded; evidence retained".into());
                }
                let path = entry.map_err(err)?.path();
                let meta = fs::symlink_metadata(&path).map_err(err)?;
                if meta.file_type().is_symlink() {
                    return Err("work-tree symlink retained".into());
                }
                if meta.is_dir() {
                    pending.push(path);
                } else if meta.is_file() {
                    total = total
                        .checked_add(meta.len())
                        .ok_or("work-tree extent overflow")?;
                    if total > MAX_WORK_BYTES {
                        return Err("work-tree byte budget exceeded; evidence retained".into());
                    }
                    files.push((path.clone(), crate::process_video::file_identity(&path)?));
                } else {
                    return Err("work-tree special file retained".into());
                }
            }
        }
        // Freeze the entire exact file/directory inventory before disposal.
        // A newly added entry makes empty-directory removal refuse; no recursive
        // path-only deletion adopts unexpected bytes.
        for (dir, stamp) in &dirs {
            stamp.matches(dir)?;
        }
        for (file, identity) in files {
            self.stamp.matches(root)?;
            if crate::process_video::file_identity(&file)? != identity {
                return Err("work-tree file changed; evidence retained".into());
            }
            fs::remove_file(&file).map_err(err)?;
        }
        dirs.sort_by_key(|(dir, _)| std::cmp::Reverse(dir.components().count()));
        for (dir, stamp) in dirs {
            if dir == *root {
                continue;
            }
            stamp.matches(&dir)?;
            fs::remove_dir(&dir).map_err(err)?;
        }
        self.owned.remove_empty()
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::release::{AdapterPayload, ReleaseId};
    fn scratch() -> (PathBuf, VideoScratch) {
        let state = std::env::temp_dir().join(format!(
            "mirket-video-scratch-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        private_dir(&state).unwrap();
        let state = fs::canonicalize(state).unwrap();
        let stamp = DirectoryStamp::inspect(&state).unwrap();
        let owned = OwnedScratch::create(&state, &stamp, "video-cleanup").unwrap();
        let stamp = DirectoryStamp::inspect(&owned.path).unwrap();
        (state, VideoScratch { owned, stamp })
    }
    #[test]
    fn confirmed_scratch_cleanup_removes_only_its_verified_inventory() {
        let (state, scratch) = scratch();
        let path = scratch.owned.path.clone();
        private_dir(&path.join("source")).unwrap();
        fs::write(path.join("source/index.html"), "editable original copy").unwrap();
        fs::write(path.join("result.mp4"), "checked stage").unwrap();
        scratch.cleanup().unwrap();
        assert!(!path.exists());
        assert!(state.join("jobs").is_dir());
        fs::remove_dir_all(state).unwrap();
    }
    #[test]
    fn unexpected_scratch_link_retains_all_evidence_and_preserves_target() {
        let (state, scratch) = scratch();
        let path = scratch.owned.path.clone();
        let outside = state.join("unrelated.txt");
        fs::write(&outside, "preserve").unwrap();
        fs::write(path.join("owned.txt"), "retain").unwrap();
        std::os::unix::fs::symlink(&outside, path.join("unexpected")).unwrap();
        assert!(scratch.cleanup().unwrap_err().contains("symlink"));
        assert_eq!(fs::read_to_string(&outside).unwrap(), "preserve");
        assert_eq!(
            fs::read_to_string(path.join("owned.txt")).unwrap(),
            "retain"
        );
        fs::remove_dir_all(state).unwrap();
    }
    #[test]
    fn terminal_receipts_remain_protected_until_effect_and_cleanup_are_resolved() {
        let state = std::env::temp_dir().join(format!(
            "mirket-video-retention-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        private_dir(&state).unwrap();
        let state = fs::canonicalize(state).unwrap();
        let binary = std::env::current_exe().unwrap();
        let payload = AdapterPayload::explicit(&binary).unwrap();
        let engine = Engine::open(
            ReleaseId::parse("video-retention-test").unwrap(),
            payload,
            state.clone(),
        )
        .unwrap();
        let manager = VideoManager::new(engine.clone());
        let issued = Receipt {
            schema: 1,
            kind: "video_export".into(),
            intent: Intent {
                release_id: engine.release.as_str().into(),
                owner_id: engine.owner_id().into(),
                project_root: state.clone(),
                source_root: state.clone(),
                entry_file: "index.html".into(),
                sources: vec![],
                destination_path: state.join("output.mp4"),
                profile: VideoProfile {
                    width: 640,
                    height: 360,
                    fps: FrameRate {
                        numerator: 12,
                        denominator: 1,
                    },
                    frame_count: 24,
                    format: VideoFormat::Mp4,
                    captions_path: None,
                },
            },
            intent_revision: "sha256:placeholder".into(),
            created_at: now().unwrap(),
            expires_at: now().unwrap() + 300,
            state: JobState::default(),
            artifact_revision: None,
            artifact_bytes: None,
            scratch_path: None,
            scratch_cleanup: None,
            queue_wait_ms: None,
            evidence: None,
            error: None,
            diagnostics: vec![],
        };
        let id = manager
            .store()
            .unwrap()
            .lock()
            .unwrap()
            .issue(&serde_json::to_vec(&issued).unwrap())
            .unwrap();
        let mut unknown = issued.clone();
        unknown.state.status = Status::Failed;
        unknown.state.effect = Effect::Unknown;
        unknown.scratch_path = Some(state.join("jobs/retained"));
        unknown.scratch_cleanup = Some("children_unconfirmed".into());
        manager.save(&id, &unknown).unwrap();
        unknown.error = Some("publication may have happened".into());
        manager.save(&id, &unknown).unwrap();

        let second = manager
            .store()
            .unwrap()
            .lock()
            .unwrap()
            .issue(&serde_json::to_vec(&issued).unwrap())
            .unwrap();
        let mut complete = issued;
        complete.state.status = Status::Succeeded;
        complete.state.effect = Effect::Confirmed;
        complete.scratch_path = Some(state.join("jobs/removed"));
        complete.scratch_cleanup = Some("children_stopped".into());
        manager.save(&second, &complete).unwrap();
        complete.scratch_cleanup = Some("removed_verified_owned_files".into());
        manager.save(&second, &complete).unwrap();
        complete.error = Some("late mutation".into());
        assert!(manager.save(&second, &complete).is_err());

        let third = manager
            .store()
            .unwrap()
            .lock()
            .unwrap()
            .issue(&serde_json::to_vec(&complete.issued()).unwrap())
            .unwrap();
        let mut launching = complete.issued();
        launching.state.status = Status::Running;
        launching.scratch_path = Some(state.join("jobs/prelaunch"));
        let admission = crate::leases::Admission::open(&state).unwrap();
        let mut lease = admission.try_acquire().unwrap();
        let marker = state.join(format!("heavy-slot-{}.unresolved", lease.slot_index()));
        let held = manager.store().unwrap().lock().unwrap();
        assert!(
            manager
                .begin_child_ownership(&third, &mut launching, &mut lease)
                .is_err()
        );
        assert!(
            !marker.exists(),
            "receipt contention must not create a heavy marker"
        );
        drop(held);
        drop(lease);
        drop(manager);
        drop(engine);
        fs::remove_dir_all(state).unwrap();
    }
}
