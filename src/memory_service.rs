//! Permission-separated memory intents, commit and inspection. The SQLite
//! mutation receipt commits with canonical data; JSON intent receipts can lag
//! after interruption and are never used as permission to replay a write.
use crate::{
    engine::Engine,
    identity,
    memory::{self, MemoryStore, RecordInput, RecordKind, RecordRef, Scope, SearchRequest},
    operation_store::OperationStore,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selection {
    Project { project_root: PathBuf },
    SharedPreferences,
}
#[derive(Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Mutation {
    Create {
        record: RecordInput,
        supersedes: Option<RecordRef>,
    },
    Revise {
        record_id: String,
        expected_revision: u64,
        record: RecordInput,
    },
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrepareArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub mutation: Mutation,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForgetPrepareArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub record_id: String,
    pub expected_revision: u64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub operation_id: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub request: SearchRequest,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub request: memory::list::ListRequest,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IndexArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub after_row_id: Option<i64>,
    pub limit: usize,
    pub max_bytes: usize,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub record_id: String,
    pub revision: Option<u64>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistoryArgs {
    pub expected_release_id: String,
    pub scope: Selection,
    pub record_id: String,
    pub before_revision: Option<u64>,
    pub limit: usize,
    pub max_bytes: usize,
}
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: u8,
    kind: String,
    owner_id: String,
    release_id: String,
    scope: Scope,
    checkout_id: Option<String>,
    intent_revision: String,
    expires_at: u64,
    status: String,
    authority_at_prepare: AuthorityPresence,
    target: Option<RecordRef>,
    record: Option<RecordRef>,
}
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityPresence {
    canonical: bool,
    lifecycle: bool,
}

fn resolve_existing_selection(
    state_root: &Path,
    selection: &Selection,
) -> Result<(Scope, Option<String>), String> {
    match selection {
        Selection::SharedPreferences => Ok((Scope::SharedPreferences, None)),
        Selection::Project { project_root } => {
            let selected = identity::lookup(state_root, project_root)
                .map_err(error)?
                .ok_or("project has no registered memory scope")?;
            Ok((
                Scope::Project(selected.project_id),
                Some(selected.checkout_id),
            ))
        }
    }
}

fn optional_private_directory(path: &Path) -> Result<bool, String> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err("memory authority directory is substituted".into());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                // SAFETY: geteuid takes no pointers and only observes process identity.
                if metadata.mode() & 0o077 != 0 || metadata.uid() != unsafe { libc::geteuid() } {
                    return Err("memory authority directory is not private and owned".into());
                }
            }
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(error(e)),
    }
}

/// Read-only presence observation. A permanent scope lock or SQLite sidecar
/// without its canonical database is interrupted/missing authority, not a new scope.
fn preparation_authorities(
    root: &Path,
    scope: &Scope,
) -> Result<(Option<MemoryStore>, AuthorityPresence), String> {
    let memory_dir = root.join("memory");
    let lifecycle_dir = root.join("memory-lifecycle");
    let memory_present = optional_private_directory(&memory_dir)?;
    let lifecycle_present = optional_private_directory(&lifecycle_dir)?;
    let key = crate::memory_lifecycle::scope_key(scope).map_err(error)?;
    let canonical_path = memory_dir.join(format!("scope-{key}.sqlite3"));
    let canonical_present = match std::fs::symlink_metadata(&canonical_path) {
        Ok(_) => true,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(e) => return Err(error(e)),
    };
    if !canonical_present {
        let mut remnants = vec![lifecycle_dir.join(format!("scope-{key}.lock"))];
        for suffix in ["-journal", "-wal", "-shm"] {
            remnants.push(memory_dir.join(format!("scope-{key}.sqlite3{suffix}")));
        }
        for path in remnants {
            match std::fs::symlink_metadata(path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                _ => {
                    return Err(
                        "canonical memory is missing but scope/recovery evidence remains".into(),
                    );
                }
            }
        }
    }
    if lifecycle_present {
        // This also validates existing authority format and native private files.
        let (epoch, exclusions) =
            crate::memory_lifecycle::epoch_and_exclusions(root, scope).map_err(error)?;
        if !canonical_present && (epoch != 0 || !exclusions.is_empty()) {
            return Err("missing canonical scope retains lifecycle history".into());
        }
        if crate::memory_lifecycle::pending_restore(root, scope)
            .map_err(error)?
            .is_some()
        {
            return Err("pending restore prevents preparation retirement or issuance".into());
        }
    }
    let canonical = if canonical_present {
        if !memory_present || !lifecycle_present {
            return Err("canonical authority is incomplete".into());
        }
        Some(MemoryStore::open_existing(root, scope.clone()).map_err(error)?)
    } else {
        None
    };
    Ok((
        canonical,
        AuthorityPresence {
            canonical: canonical_present,
            lifecycle: lifecycle_present,
        },
    ))
}

/// Retire an exactly issued expired memory preparation without canonical writes.
/// Maintenance excludes sessions; existing authority remains pinned until finalize.
pub fn retire_expired_preparation(
    state_root: &Path,
    expected_release: &crate::release::ReleaseId,
    selection: &Selection,
    operation_id: &str,
) -> Result<Value, String> {
    let maintenance =
        crate::maintenance::MaintenanceGuard::try_acquire(state_root).map_err(error)?;
    let root = maintenance.state_root();
    let (scope, checkout) = resolve_existing_selection(root, selection)?;
    let store = OperationStore::open_existing(root).map_err(error)?;
    // Do not invert scope->receipt locking used by forgetting. Maintenance
    // excludes normal sessions, but keeping the established order is simpler.
    let bytes = store.read_existing(operation_id).map_err(error)?;
    let saved: Receipt = serde_json::from_slice(&bytes).map_err(error)?;
    expected_release.require(&saved.release_id).map_err(error)?;
    if saved.schema != 1
        || !matches!(saved.kind.as_str(), "memory_mutation" | "memory_forget")
        || saved.scope != scope
        || saved.checkout_id != checkout
    {
        return Err("preparation scope, checkout or schema differs".into());
    }
    if now()? <= saved.expires_at {
        return Err("only an expired preparation may be retired".into());
    }
    let mut original = saved.clone();
    if original.status == "expired_not_started" {
        original.status = "prepared".into();
    }
    if original.status != "prepared" {
        return Err("started or unknown memory operation cannot retire as unused".into());
    }
    if original.kind == "memory_mutation" && original.record.is_some() {
        return Err("prepared mutation contains a result; retain authority".into());
    }
    if original.kind == "memory_forget" {
        let target = original
            .target
            .as_ref()
            .ok_or("forget preparation target unavailable")?;
        memory::validate_reference(target).map_err(error)?;
        if original.record.as_ref() != Some(target) || !original.authority_at_prepare.canonical {
            return Err("forget preparation authority differs".into());
        }
    }
    if let Some(target) = &original.target {
        memory::validate_reference(target).map_err(error)?;
    }
    let (mut canonical, current) = preparation_authorities(root, &scope)?;
    if (original.authority_at_prepare.canonical && !current.canonical)
        || (original.authority_at_prepare.lifecycle && !current.lifecycle)
    {
        return Err("previously present memory authority is missing".into());
    }
    if let Some(canonical) = canonical.as_mut()
        && canonical
            .read_mutation(operation_id)
            .map_err(error)?
            .is_some()
    {
        return Err(
            "canonical mutation exists; inspect/reconcile instead of unused retirement".into(),
        );
    }
    if current.lifecycle
        && (crate::memory_lifecycle::forget_operation_exists(root, operation_id).map_err(error)?
            || crate::memory_lifecycle::restore_operation_pending(root, operation_id)
                .map_err(error)?)
    {
        return Err("lifecycle authority exists for this operation; retain recovery".into());
    }
    let locked = store.lock().map_err(error)?;
    if locked.read(operation_id).map_err(error)? != bytes
        || !locked
            .matches_issued_payload(operation_id, &serde_json::to_vec(&original).map_err(error)?)
            .map_err(error)?
    {
        return Err("issued preparation binding changed; retain evidence".into());
    }
    let mut retired = original;
    retired.status = "expired_not_started".into();
    locked
        .finalize(operation_id, &serde_json::to_vec(&retired).map_err(error)?)
        .map_err(error)?;
    Ok(
        json!({"operation_id":operation_id,"release_id":retired.release_id,"scope":retired.scope,
        "checkout_id":retired.checkout_id,"status":retired.status,"replay_allowed":false}),
    )
}
#[derive(Serialize)]
enum Intent {
    Save(Mutation),
    Forget(RecordRef),
}
struct Pending {
    mutation: Intent,
    receipt: Receipt,
}
#[derive(Default)]
pub struct MemoryOptions {
    pub backup_root: Option<PathBuf>,
    pub embedding_runtime: Option<Arc<crate::memory_runtime::MemoryRuntime>>,
}
pub struct MemoryService {
    options: MemoryOptions,
    engine: Arc<Engine>,
    pending: Mutex<HashMap<String, Pending>>,
    mutation_admission: Mutex<()>,
    closed: AtomicBool,
}
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(error)
}
impl MemoryService {
    pub fn new(engine: Arc<Engine>, options: MemoryOptions) -> Self {
        Self {
            engine,
            options,
            pending: Mutex::new(HashMap::new()),
            mutation_admission: Mutex::new(()),
            closed: AtomicBool::new(false),
        }
    }
    fn mutation_guard(&self) -> Result<std::sync::MutexGuard<'_, ()>, String> {
        let guard = self
            .mutation_admission
            .lock()
            .map_err(|_| "memory mutation admission unavailable")?;
        if self.closed.load(Ordering::Acquire) {
            return Err("memory service is shutting down".into());
        }
        Ok(guard)
    }
    fn resolve(
        &self,
        selection: &Selection,
        register: bool,
    ) -> Result<(Scope, Option<String>), String> {
        match selection {
            Selection::SharedPreferences => Ok((Scope::SharedPreferences, None)),
            Selection::Project { project_root } => {
                let selected = if register {
                    identity::resolve(self.engine.state_root(), project_root).map_err(error)?
                } else {
                    identity::lookup(self.engine.state_root(), project_root)
                        .map_err(error)?.ok_or("project has no registered memory scope; prepare an explicit save to register it")?
                };
                Ok((
                    Scope::Project(selected.project_id),
                    Some(selected.checkout_id),
                ))
            }
        }
    }
    pub fn prepare(&self, args: PrepareArgs) -> Result<Value, String> {
        let _admission = self.mutation_guard()?;
        self.engine.guard(&args.expected_release_id)?;
        let (scope, checkout_id) = self.resolve(&args.scope, true)?;
        let mut mutation = args.mutation;
        let record = match &mut mutation {
            Mutation::Create { record, .. } | Mutation::Revise { record, .. } => record,
        };
        if record.kind == RecordKind::TaskState && record.applicability.checkout_id.is_none() {
            record.applicability.checkout_id = checkout_id.clone();
        }
        if record
            .applicability
            .checkout_id
            .as_ref()
            .is_some_and(|id| Some(id) != checkout_id.as_ref())
        {
            return Err("record checkout does not match the explicitly selected checkout".into());
        }
        if scope == Scope::SharedPreferences && record.applicability.task_key.is_some() {
            return Err("task-specific memory belongs to a project, not shared preferences".into());
        }
        memory::validate_input(&scope, record).map_err(error)?;
        match &mutation {
            Mutation::Create {
                supersedes: Some(r),
                ..
            } => memory::validate_reference(r).map_err(error)?,
            Mutation::Revise {
                record_id,
                expected_revision,
                ..
            } => memory::validate_reference(&RecordRef {
                record_id: record_id.clone(),
                revision: *expected_revision,
            })
            .map_err(error)?,
            _ => (),
        }
        // Keep the source scope pinned through receipt issuance. Forgetting
        // cannot finish its receipt sweep between validation and persistence.
        let _source_pin = if matches!(
            &mutation,
            Mutation::Revise { .. }
                | Mutation::Create {
                    supersedes: Some(_),
                    ..
                }
        ) {
            Some(
                MemoryStore::open_existing(self.engine.state_root(), scope.clone())
                    .map_err(error)?,
            )
        } else {
            None
        };
        self.verify_source_checkout(&scope, checkout_id.as_deref(), &mutation)?;
        self.issue_intent(scope, checkout_id, Intent::Save(mutation))
    }
    pub fn prepare_forget(&self, args: ForgetPrepareArgs) -> Result<Value, String> {
        let _admission = self.mutation_guard()?;
        self.engine.guard(&args.expected_release_id)?;
        let (scope, checkout) = self.resolve(&args.scope, false)?;
        let target = RecordRef {
            record_id: args.record_id,
            revision: args.expected_revision,
        };
        let _source_pin =
            MemoryStore::open_existing(self.engine.state_root(), scope.clone()).map_err(error)?;
        memory::validate_reference(&target).map_err(error)?;
        self.verify_forget_target(&scope, checkout.as_deref(), &target)?;
        self.issue_intent(scope, checkout, Intent::Forget(target))
    }
    fn verify_forget_target(
        &self,
        scope: &Scope,
        checkout: Option<&str>,
        target: &RecordRef,
    ) -> Result<(), String> {
        let mut store =
            MemoryStore::open_existing(self.engine.state_root(), scope.clone()).map_err(error)?;
        let record = store.read(&target.record_id, None).map_err(error)?;
        verify_checkout(&record, checkout)?;
        if record.revision != target.revision {
            return Err("forget target revision changed; inspect the latest record".into());
        }
        Ok(())
    }
    fn issue_intent(
        &self,
        scope: Scope,
        checkout_id: Option<String>,
        mutation: Intent,
    ) -> Result<Value, String> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| "memory intent state unavailable")?;
        let current = now()?;
        if self.closed.load(Ordering::Acquire) {
            return Err("memory service is shutting down".into());
        }
        self.retire_unused(&mut pending, Some(current))?;
        if pending.len() >= 64 {
            return Err("memory intent capacity exhausted".into());
        }
        let (_authority_pin, authority_at_prepare) =
            preparation_authorities(self.engine.state_root(), &scope)?;
        let receipt = Receipt {
            schema: 1,
            kind: if matches!(&mutation, Intent::Forget(_)) {
                "memory_forget"
            } else {
                "memory_mutation"
            }
            .into(),
            owner_id: self.engine.owner_id().into(),
            release_id: self.engine.release.as_str().into(),
            scope,
            checkout_id,
            intent_revision: crate::engine::revision(
                &serde_json::to_vec(&mutation).map_err(error)?,
            ),
            expires_at: current.checked_add(300).ok_or("invalid clock")?,
            status: "prepared".into(),
            authority_at_prepare,
            target: match &mutation {
                Intent::Forget(target) => Some(target.clone()),
                Intent::Save(Mutation::Revise {
                    record_id,
                    expected_revision,
                    ..
                }) => Some(RecordRef {
                    record_id: record_id.clone(),
                    revision: *expected_revision,
                }),
                Intent::Save(Mutation::Create { supersedes, .. }) => supersedes.clone(),
            },
            record: match &mutation {
                Intent::Forget(target) => Some(target.clone()),
                Intent::Save(_) => None,
            },
        };
        let store = OperationStore::open(self.engine.state_root()).map_err(error)?;
        let id = store
            .lock()
            .map_err(error)?
            .issue(&serde_json::to_vec(&receipt).map_err(error)?)
            .map_err(error)?;
        let result = json!({"operation_id":id,"release_id":receipt.release_id,"scope":receipt.scope,"checkout_id":receipt.checkout_id,"status":"prepared","expires_at":receipt.expires_at,"intent_revision":receipt.intent_revision});
        pending.insert(id, Pending { mutation, receipt });
        Ok(result)
    }
    /// Only this connection's still-present, never-consumed intents may retire.
    /// The pending mutex serializes this proof with commit's durable Unknown
    /// transition; the operation lock binds verification and finalization.
    fn retire_unused(
        &self,
        pending: &mut HashMap<String, Pending>,
        expired_before: Option<u64>,
    ) -> Result<(), String> {
        let selected: Vec<_> = pending
            .iter()
            .filter(|(_, intent)| {
                intent.receipt.status == "prepared"
                    && expired_before.is_none_or(|time| intent.receipt.expires_at < time)
            })
            .map(|(id, _)| id.clone())
            .collect();
        if selected.is_empty() {
            return Ok(());
        }
        let store = OperationStore::open_existing(self.engine.state_root()).map_err(error)?;
        // Closing clients may retire together. Retry only acquisition, before
        // inspecting or finalizing any intent; ordinary calls remain immediate.
        let locked = if expired_before.is_none() {
            store.lock_for_shutdown()
        } else {
            store.lock()
        }
        .map_err(error)?;
        for id in selected {
            let original = &pending[&id].receipt;
            if original.owner_id != self.engine.owner_id() {
                return Err("unused memory intent owner changed; retain evidence".into());
            }
            let mut retired = original.clone();
            // A single terminal form makes retry independent of whether expiry
            // or graceful shutdown first attempted the same confirmation.
            retired.status = "cancelled".into();
            let original_bytes = serde_json::to_vec(original).map_err(error)?;
            let retired_bytes = serde_json::to_vec(&retired).map_err(error)?;
            let saved = match locked.read(&id) {
                Ok(bytes) => bytes,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // Forgetting may already have consumed this receipt. Drop
                    // only the unused local intent; absence is never permission
                    // to recreate it or a claim about external effect cleanup.
                    pending.remove(&id);
                    continue;
                }
                Err(e) => return Err(error(e)),
            };
            if saved != original_bytes && saved != retired_bytes {
                return Err("unused memory intent receipt changed; retain evidence".into());
            }
            locked.finalize(&id, &retired_bytes).map_err(error)?;
            pending.remove(&id);
        }
        Ok(())
    }
    fn verify_source_checkout(
        &self,
        scope: &Scope,
        checkout: Option<&str>,
        mutation: &Mutation,
    ) -> Result<(), String> {
        let reference = match mutation {
            Mutation::Create { supersedes, .. } => supersedes.clone(),
            Mutation::Revise {
                record_id,
                expected_revision,
                ..
            } => Some(RecordRef {
                record_id: record_id.clone(),
                revision: *expected_revision,
            }),
        };
        if let Some(reference) = reference {
            let mut store = MemoryStore::open_existing(self.engine.state_root(), scope.clone())
                .map_err(error)?;
            let previous = store
                .read(&reference.record_id, Some(reference.revision))
                .map_err(error)?;
            verify_checkout(&previous, checkout)?;
            if !previous.is_current_revision {
                return Err(
                    "memory revision is no longer current; inspect before correcting".into(),
                );
            }
        }
        Ok(())
    }
    fn receipt(&self, args: &OperationArgs) -> Result<Receipt, String> {
        let store = OperationStore::open_existing(self.engine.state_root()).map_err(error)?;
        let bytes = store.read_existing(&args.operation_id).map_err(error)?;
        let receipt: Receipt = serde_json::from_slice(&bytes)
            .map_err(|_| "not a supported memory operation receipt")?;
        if receipt.schema != 1
            || !["memory_mutation", "memory_forget"].contains(&receipt.kind.as_str())
        {
            return Err("unsupported memory receipt".into());
        }
        let (scope, checkout) = self.resolve(&args.scope, false)?;
        if receipt.scope != scope || receipt.checkout_id != checkout {
            return Err("operation belongs to another selected scope or checkout".into());
        }
        Ok(receipt)
    }
    fn save_receipt(
        &self,
        id: &str,
        receipt: &Receipt,
        recovery_complete: bool,
    ) -> Result<(), String> {
        let locked = OperationStore::open(self.engine.state_root())
            .map_err(error)?
            .lock()
            .map_err(error)?;
        let bytes = serde_json::to_vec(receipt).map_err(error)?;
        if recovery_complete {
            locked.finalize(id, &bytes).map_err(error)
        } else {
            locked.update(id, &bytes).map_err(error)
        }
    }
    pub fn inspect(&self, args: OperationArgs) -> Result<Value, String> {
        // Scoped inspection remains usable across release changes, without replay.
        let receipt = self.receipt(&args)?;
        if receipt.kind == "memory_forget" {
            let target = receipt
                .record
                .as_ref()
                .ok_or("forget receipt has no target")?;
            let committed = crate::memory_lifecycle::inspect_operation(
                self.engine.state_root(),
                receipt.scope.clone(),
                &target.record_id,
                &args.operation_id,
            )
            .map_err(error)?;
            let status = if let Some(operation) = &committed {
                if operation.status.cleanup_complete {
                    "forgotten"
                } else {
                    "cleanup_pending"
                }
            } else if receipt.status == "committed" {
                "confirmation_unavailable"
            } else if receipt.status == "prepared" && now()? > receipt.expires_at {
                "expired"
            } else {
                receipt.status.as_str()
            };
            return Ok(
                json!({"operation_id":args.operation_id,"scope":receipt.scope,"checkout_id":receipt.checkout_id,"status":status,"forget":committed,"replay_allowed":false}),
            );
        }
        let mutation =
            match MemoryStore::open_existing(self.engine.state_root(), receipt.scope.clone()) {
                Ok(mut store) => store.read_mutation(&args.operation_id).map_err(error)?,
                Err(memory::MemoryError::NotFound) => None,
                Err(e) => return Err(error(e)),
            };
        let status = if mutation.is_some() {
            "committed"
        } else if receipt.status == "committed" {
            "confirmation_unavailable"
        } else if receipt.status == "prepared" && now()? > receipt.expires_at {
            "expired"
        } else {
            receipt.status.as_str()
        };
        let unavailable = mutation.as_ref().is_some_and(|m| !m.available);
        let intent_available = receipt.owner_id == self.engine.owner_id()
            && receipt.status == "prepared"
            && now()? <= receipt.expires_at
            && self
                .pending
                .lock()
                .map_err(|_| "memory intent state unavailable")?
                .contains_key(&args.operation_id);
        let mut mutation = serde_json::to_value(mutation).map_err(error)?;
        if unavailable {
            mutation["intent_hash"] = Value::Null;
        }
        Ok(
            json!({"operation_id":args.operation_id,"scope":receipt.scope,"checkout_id":receipt.checkout_id,"status":status,"mutation":mutation,"intent_revision":if unavailable {None} else {Some(receipt.intent_revision)},"record":if unavailable {None} else {receipt.record},"intent_available":intent_available,"replay_allowed":false}),
        )
    }
    pub fn commit(&self, args: OperationArgs) -> Result<Value, String> {
        self.commit_intent(args, false)
    }
    pub fn commit_forget(&self, args: OperationArgs) -> Result<Value, String> {
        self.commit_intent(args, true)
    }
    /// An explicit repeated commit can repair only the convenience receipt of
    /// this already committed canonical operation. It never invokes a mutation,
    /// captures another backup, or starts derived indexing.
    fn reconcile_committed_save(
        &self,
        args: &OperationArgs,
        mut receipt: Receipt,
    ) -> Result<Option<Value>, String> {
        let mut store =
            match MemoryStore::open_existing(self.engine.state_root(), receipt.scope.clone()) {
                Ok(store) => store,
                Err(memory::MemoryError::NotFound) => return Ok(None),
                Err(e) => return Err(error(e)),
            };
        let Some(mutation) = store.read_mutation(&args.operation_id).map_err(error)? else {
            return Ok(None);
        };
        if !mutation.available {
            return Ok(None);
        }
        let reference = RecordRef {
            record_id: mutation.record_id.clone(),
            revision: mutation.revision,
        };
        if receipt
            .record
            .as_ref()
            .is_some_and(|record| record != &reference)
        {
            return Err(
                "receipt result differs from its canonical mutation; retain recovery evidence"
                    .into(),
            );
        }
        receipt.status = "committed".into();
        receipt.record = Some(reference.clone());
        // Keep canonical scope access through finalization so forgetting cannot
        // remove the authority between this proof and receipt retirement.
        let receipt_warning = self.save_receipt(&args.operation_id, &receipt, true).err();
        Ok(Some(
            json!({"operation_id":args.operation_id,"scope":receipt.scope,
            "checkout_id":receipt.checkout_id,"status":"committed","mutation":mutation,
            "record":reference,"intent_revision":receipt.intent_revision,"intent_available":false,
            "replay_allowed":false,"receipt_warning":receipt_warning,"reconciled":true}),
        ))
    }
    fn commit_intent(&self, args: OperationArgs, forgetting: bool) -> Result<Value, String> {
        // Hold through canonical mutation, backup and receipt confirmation, not
        // merely intent consumption. Shutdown waits for admitted writes to end.
        let _admission = self.mutation_guard()?;
        self.engine.guard(&args.expected_release_id)?;
        let receipt = self.receipt(&args)?;
        if (receipt.kind == "memory_forget") != forgetting {
            return Err("operation requires its matching save or forgetting commit tool".into());
        }
        if receipt.owner_id != self.engine.owner_id() {
            if !forgetting
                && let Some(confirmed) = self.reconcile_committed_save(&args, receipt.clone())?
            {
                return Ok(confirmed);
            }
            return Err(
                "memory intent belongs to another connection; inspect it without replay".into(),
            );
        }
        let mut intents = self
            .pending
            .lock()
            .map_err(|_| "memory intent state unavailable")?;
        if self.closed.load(Ordering::Acquire) {
            return Err("memory service is shutting down".into());
        }
        let Some(pending) = intents.get_mut(&args.operation_id) else {
            drop(intents);
            if !forgetting && let Some(confirmed) = self.reconcile_committed_save(&args, receipt)? {
                return Ok(confirmed);
            }
            return self.inspect(args);
        };
        if now()? > receipt.expires_at {
            self.retire_unused(&mut intents, Some(now()?))?;
            return Err("memory intent expired; no mutation was started".into());
        }
        if receipt.status != "prepared" || receipt != pending.receipt {
            return Err("memory intent receipt changed; inspect before issuing new intent".into());
        }
        match &pending.mutation {
            Intent::Save(mutation) => self.verify_source_checkout(
                &receipt.scope,
                receipt.checkout_id.as_deref(),
                mutation,
            )?,
            Intent::Forget(target) => {
                self.verify_forget_target(&receipt.scope, receipt.checkout_id.as_deref(), target)?
            }
        }
        pending.receipt.status = "unknown".into();
        self.save_receipt(&args.operation_id, &pending.receipt, false)?;
        // Retain the intent until pre-write validation and durable unknown
        // succeed. Concurrent callers cannot consume it in between these steps.
        let mut pending = intents
            .remove(&args.operation_id)
            .ok_or("memory intent unavailable")?;
        drop(intents);
        if let Intent::Forget(target) = &pending.mutation {
            let forgotten = crate::memory_lifecycle::forget(
                self.engine.state_root(),
                receipt.scope,
                &target.record_id,
                &args.operation_id,
                target.revision,
            )
            .map_err(error)?;
            pending.receipt.status = "committed".into();
            let receipt_warning = self
                .save_receipt(
                    &args.operation_id,
                    &pending.receipt,
                    forgotten.cleanup_complete,
                )
                .err();
            return Ok(
                json!({"operation_id":args.operation_id,"scope":pending.receipt.scope,"status":if forgotten.cleanup_complete {"forgotten"} else {"cleanup_pending"},"forget":forgotten,"receipt_warning":receipt_warning}),
            );
        }
        let Intent::Save(mutation) = pending.mutation else {
            unreachable!()
        };
        let mut store =
            MemoryStore::open(self.engine.state_root(), receipt.scope).map_err(error)?;
        let saved = match mutation {
            Mutation::Create { record, supersedes } => {
                store.create(&args.operation_id, record, supersedes)
            }
            Mutation::Revise {
                record_id,
                expected_revision,
                record,
            } => store.revise(&args.operation_id, &record_id, expected_revision, record),
        }
        .map_err(error)?;
        pending.receipt.status = "committed".into();
        pending.receipt.record = Some(RecordRef {
            record_id: saved.record_id.clone(),
            revision: saved.revision,
        });
        // SQLite already committed; failure to update the convenience receipt is
        // reported as incomplete confirmation, never as an absent mutation.
        let receipt_warning = self
            .save_receipt(&args.operation_id, &pending.receipt, true)
            .err();
        drop(store);
        let backup = match &self.options.backup_root {
            None => json!({"status":"disabled","reason":"no managed backup directory selected"}),
            Some(root) => {
                let id = format!(
                    "op-backup-{}",
                    crate::embedding::source_digest(&args.operation_id)
                );
                match crate::memory_backup::backup(
                    self.engine.state_root(),
                    root,
                    pending.receipt.scope.clone(),
                    &id,
                    crate::memory_backup::BackupKind::Daily,
                    crate::memory_backup::BackupPolicy::default(),
                ) {
                    Ok(snapshot) => json!({"status":"eligible","snapshot":snapshot}),
                    Err(e) => json!({"status":"degraded","warning":e.to_string()}),
                }
            }
        };
        Ok(
            json!({"operation_id":args.operation_id,"scope":pending.receipt.scope,"checkout_id":pending.receipt.checkout_id,"status":"committed","record":saved,"receipt_warning":receipt_warning,"backup":backup}),
        )
    }
    /// Complete only an exclusion already committed by this exact operation.
    /// This explicit cleanup cannot create a new forgetting decision.
    pub fn reconcile_forget(&self, args: OperationArgs) -> Result<Value, String> {
        let _admission = self.mutation_guard()?;
        self.engine.guard(&args.expected_release_id)?;
        let mut receipt = self.receipt(&args)?;
        if receipt.kind != "memory_forget" {
            return Err("not a forgetting operation".into());
        }
        let target = receipt
            .record
            .as_ref()
            .ok_or("forget receipt has no target")?;
        let committed = crate::memory_lifecycle::inspect_operation(
            self.engine.state_root(),
            receipt.scope.clone(),
            &target.record_id,
            &args.operation_id,
        )
        .map_err(error)?
        .ok_or(
            "no committed exclusion for this operation; reconciliation cannot start forgetting",
        )?;
        if committed.expected_revision != target.revision {
            return Err("forgetting operation target changed".into());
        }
        let status = crate::memory_lifecycle::reconcile(
            self.engine.state_root(),
            receipt.scope.clone(),
            &target.record_id,
        )
        .map_err(error)?;
        let receipt_warning = if status.cleanup_complete {
            receipt.status = "committed".into();
            self.save_receipt(&args.operation_id, &receipt, true).err()
        } else {
            None
        };
        Ok(
            json!({"operation_id":args.operation_id,"scope":receipt.scope,"status":if status.cleanup_complete {"forgotten"} else {"cleanup_pending"},"forget":status,"receipt_warning":receipt_warning}),
        )
    }
    fn search_lexical(&self, mut args: SearchArgs) -> Result<Value, String> {
        self.engine.guard(&args.expected_release_id)?;
        let (scope, checkout) = self.resolve(&args.scope, false)?;
        if args
            .request
            .context
            .checkout_id
            .as_ref()
            .is_some_and(|id| Some(id) != checkout.as_ref())
        {
            return Err("search checkout differs from selected checkout".into());
        }
        args.request.context.checkout_id = checkout;
        let mut store =
            MemoryStore::open_existing(self.engine.state_root(), scope).map_err(error)?;
        serde_json::to_value(store.search(args.request).map_err(error)?).map_err(error)
    }
    pub async fn shutdown(&self) -> Result<(), String> {
        let retirement = (|| {
            let _admission = self
                .mutation_admission
                .lock()
                .map_err(|_| "memory mutation admission unavailable")?;
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| "memory intent state unavailable")?;
            self.closed.store(true, Ordering::Release);
            self.retire_unused(&mut pending, None)
        })();
        // A receipt failure must not skip cancellation and reaping of owned
        // model work. Keep failed intent evidence for inspection/retry.
        let runtime = match &self.options.embedding_runtime {
            Some(runtime) => runtime.shutdown().await,
            None => Ok(()),
        };
        match (retirement, runtime) {
            (Ok(()), result) | (result, Ok(())) => result,
            (Err(receipt), Err(runtime)) => Err(format!("{receipt}; {runtime}")),
        }
    }
    pub fn resource_status(&self) -> Value {
        json!({
            "embedding_configured":self.options.embedding_runtime.is_some(),
            "managed_backups_configured":self.options.backup_root.is_some(),
            "heavy_job_slots":crate::leases::HEAVY_JOB_SLOTS,
            "waiting_slots":crate::leases::WAITING_JOB_SLOTS,
            "soft_resident_limit_bytes":crate::resource_admission::SOFT_LIMIT_BYTES,
            "aggregate_accounting":if cfg!(target_os = "macos") { "native_process_tree_rss_at_admission" } else { "unqualified" }
        })
    }
    pub async fn search(self: &Arc<Self>, mut args: SearchArgs) -> Result<Value, String> {
        self.engine.guard(&args.expected_release_id)?;
        memory::validate_search_request(&args.request).map_err(error)?;
        if self.options.embedding_runtime.is_none() || args.request.query.trim().is_empty() {
            let service = self.clone();
            return tokio::task::spawn_blocking(move || service.search_lexical(args))
                .await
                .map_err(|_| "memory search failed")?;
        }
        let service = self.clone();
        let selection = args.scope.clone();
        let selected = tokio::task::spawn_blocking(move || service.resolve(&selection, false))
            .await
            .map_err(|_| "memory scope inspection failed")??;
        let (scope, checkout) = selected;
        if args
            .request
            .context
            .checkout_id
            .as_ref()
            .is_some_and(|id| Some(id) != checkout.as_ref())
        {
            return Err("search checkout differs from selected checkout".into());
        }
        args.request.context.checkout_id = checkout;
        let runtime = self.options.embedding_runtime.as_ref().unwrap();
        let embedded = runtime
            .execute(crate::embedding_job::Input {
                query: Some(args.request.query.clone()),
                passages: Vec::new(),
            })
            .await;
        let service = self.clone();
        tokio::task::spawn_blocking(move || {
            // Reopen after inference: forgetting, correction and restore may
            // have changed canonical eligibility while computation was running.
            let mut store =
                MemoryStore::open_existing(service.engine.state_root(), scope).map_err(error)?;
            let max_bytes = args.request.max_bytes;
            let (result, warning) = match embedded {
                Ok(output) => {
                    let query = output.query.ok_or("embedding query result unavailable")?;
                    (
                        store.search_hybrid(args.request, &query).map_err(error)?,
                        None,
                    )
                }
                Err(warning) => (store.search(args.request).map_err(error)?, Some(warning)),
            };
            let result = match warning {
                Some(warning) => result
                    .with_semantic_warning(&warning, max_bytes)
                    .map_err(error)?,
                None => result,
            };
            serde_json::to_value(result).map_err(error)
        })
        .await
        .map_err(|_| "memory search failed")?
    }
    /// Index a newly committed exact revision. A failed derived computation must
    /// never downgrade the already committed canonical mutation.
    pub async fn refresh_committed(self: &Arc<Self>, value: &mut Value) {
        let Some(runtime) = &self.options.embedding_runtime else {
            return;
        };
        if value["status"] != "committed" {
            return;
        }
        let Ok(record) = serde_json::from_value::<memory::Record>(value["record"].clone()) else {
            return;
        };
        let Ok(scope) = serde_json::from_value::<Scope>(value["scope"].clone()) else {
            return;
        };
        let reference = RecordRef {
            record_id: record.record_id,
            revision: record.revision,
        };
        let digest = memory::index::source_digest(&record.data.content);
        let computed = runtime
            .execute(crate::embedding_job::Input {
                query: None,
                passages: vec![record.data.content],
            })
            .await;
        let service = self.clone();
        let status = match computed {
            Err(e) => Err(e),
            Ok(output) => tokio::task::spawn_blocking(move || {
                let _admission = service.mutation_guard()?;
                let passage = output
                    .passages
                    .first()
                    .ok_or("embedding passage result unavailable")?;
                let mut store = MemoryStore::open_for_index(service.engine.state_root(), scope)
                    .map_err(error)?;
                store
                    .apply_index(&reference, &digest, passage)
                    .map_err(error)
            })
            .await
            .unwrap_or_else(|_| Err("derived memory publication failed".into())),
        };
        value["semantic_index"] = match status {
            Ok(()) => json!({"status":"indexed"}),
            Err(e) => json!({"status":"pending","warning":e}),
        };
    }
    /// Refresh one bounded page of derived vectors. The cursor scans current
    /// heads; it is not a stable snapshot or proof that every head was indexed.
    pub async fn index_refresh(self: &Arc<Self>, args: IndexArgs) -> Result<Value, String> {
        self.engine.guard(&args.expected_release_id)?;
        if !(1..=memory::index::MAX_INDEX_HEADS).contains(&args.limit)
            || !(memory::MAX_CONTENT_BYTES..=memory::index::MAX_INDEX_BYTES)
                .contains(&args.max_bytes)
            || args.after_row_id.is_some_and(|row| row <= 0)
        {
            return Err("invalid bounded index page request".into());
        }
        let runtime = self
            .options
            .embedding_runtime
            .as_ref()
            .ok_or("semantic indexing unavailable: no embedding runtime selected")?;
        let service = self.clone();
        let (scope, batch) = tokio::task::spawn_blocking(move || {
            let _admission = service.mutation_guard()?;
            let (scope, _) = service.resolve(&args.scope, false)?;
            let mut store = MemoryStore::open_existing(service.engine.state_root(), scope.clone())
                .map_err(error)?;
            let batch = store
                .select_index_batch(args.after_row_id, args.limit, args.max_bytes)
                .map_err(error)?;
            // The connection and shared scope pin end before native inference.
            Ok::<_, String>((scope, batch))
        })
        .await
        .map_err(|_| "index page selection failed")??;
        let computed = if batch.items.is_empty() {
            None
        } else {
            Some(
                runtime
                    .execute(crate::embedding_job::Input {
                        query: None,
                        passages: batch
                            .items
                            .iter()
                            .map(|item| item.content.clone())
                            .collect(),
                    })
                    .await,
            )
        };
        let service = self.clone();
        tokio::task::spawn_blocking(move || service.finish_index_page(scope, batch, computed))
            .await
            .map_err(|_| {
                "derived index page result is unavailable; inspect coverage before a fresh pass"
                    .into()
            })
    }

    fn finish_index_page(
        &self,
        scope: Scope,
        batch: memory::index::IndexBatch,
        computed: Option<Result<crate::embedding_job::Output, String>>,
    ) -> Value {
        let mut indexed = Vec::<RecordRef>::new();
        let mut skipped = Vec::<Value>::new();
        let mut failed = Vec::<Value>::new();
        // Computation may finish just before shutdown while this publication
        // is still queued. Admission applies to the write, not only inference.
        let admission = self.mutation_guard();
        let publication = if let Err(warning) = &admission {
            Err(warning.clone())
        } else {
            match computed {
                Some(Ok(output))
                    if output.query.is_none() && output.passages.len() == batch.items.len() =>
                {
                    Ok(output.passages)
                }
                Some(Ok(_)) => Err("embedding batch does not match its selected page".into()),
                Some(Err(warning)) => Err(warning),
                None if batch.items.is_empty() => Ok(Vec::new()),
                None => Err("embedding page result is unavailable".into()),
            }
        };
        match publication {
            Err(warning) => {
                let warning: String = warning.chars().take(256).collect();
                for item in &batch.items {
                    failed.push(json!({"reference":item.reference,"reason":"embedding_unavailable","warning":warning}));
                }
            }
            Ok(passages) if !batch.items.is_empty() => {
                match MemoryStore::open_for_index(self.engine.state_root(), scope.clone()) {
                    Err(error) => {
                        let warning: String = error.to_string().chars().take(256).collect();
                        for item in &batch.items {
                            failed.push(json!({"reference":item.reference,"reason":"canonical_scope_unavailable","warning":warning}));
                        }
                    }
                    Ok(mut store) => {
                        for (item, passage) in batch.items.iter().zip(&passages) {
                            match store.apply_index(&item.reference, &item.source_sha256, passage) {
                                Ok(()) => indexed.push(item.reference.clone()),
                                Err(memory::MemoryError::Conflict) => skipped.push(json!({"reference":item.reference,"reason":"head_changed"})),
                                Err(error) => failed.push(json!({"reference":item.reference,"reason":"publication_unconfirmed","warning":error.to_string().chars().take(256).collect::<String>()})),
                            }
                        }
                    }
                }
            }
            Ok(_) => (),
        }
        // Coverage is observed after publication; concurrent changes may mean
        // a successfully published old revision is no longer a current head.
        let observed = MemoryStore::open_existing(self.engine.state_root(), scope.clone())
            .and_then(|mut store| store.index_coverage());
        let (coverage, mut coverage_warning) = match observed {
            Ok(coverage) => (Some(coverage), None),
            Err(error) => (
                None,
                Some(error.to_string().chars().take(256).collect::<String>()),
            ),
        };
        if let Err(warning) = &admission {
            coverage_warning = Some(warning.clone());
        }
        let restart_required = admission.is_err()
            || !skipped.is_empty()
            || !failed.is_empty()
            || (!batch.has_more
                && coverage
                    .as_ref()
                    .is_some_and(|value| value.missing_heads > 0));
        let status =
            if admission.is_ok() && skipped.is_empty() && failed.is_empty() && coverage.is_some() {
                "page_complete"
            } else if indexed.is_empty() {
                "pending"
            } else {
                "partial"
            };
        json!({
            "status":status,"scope":scope,"model_identity":batch.model_identity,
            "scanned_heads":batch.scanned_heads,"scanned_bytes":batch.scanned_bytes,
            "already_indexed_at_selection":batch.scanned_heads.saturating_sub(batch.items.len()),
            "indexed":indexed,"skipped":skipped,"failed":failed,
            "next_after_row_id":batch.next_after_row_id,"has_more":batch.has_more,
            "coverage":coverage,"coverage_warning":coverage_warning,
            "stable_snapshot":false,"restart_required":restart_required,
        })
    }
    pub fn read(&self, args: ReadArgs) -> Result<Value, String> {
        self.engine.guard(&args.expected_release_id)?;
        let (scope, checkout) = self.resolve(&args.scope, false)?;
        let mut store =
            MemoryStore::open_existing(self.engine.state_root(), scope.clone()).map_err(error)?;
        let record = store.read(&args.record_id, args.revision).map_err(error)?;
        verify_checkout(&record, checkout.as_deref())?;
        Ok(json!({"scope":scope,"record":record}))
    }
    pub fn list(&self, mut args: ListArgs) -> Result<Value, String> {
        self.engine.guard(&args.expected_release_id)?;
        memory::list::validate_list_request(&args.request).map_err(error)?;
        let (scope, checkout) = self.resolve(&args.scope, false)?;
        if args
            .request
            .context
            .checkout_id
            .as_ref()
            .is_some_and(|id| Some(id) != checkout.as_ref())
        {
            return Err("list checkout differs from selected checkout".into());
        }
        args.request.context.checkout_id = checkout;
        let mut store =
            MemoryStore::open_existing(self.engine.state_root(), scope).map_err(error)?;
        serde_json::to_value(store.list_heads(args.request).map_err(error)?).map_err(error)
    }
    pub fn history(&self, args: HistoryArgs) -> Result<Value, String> {
        self.engine.guard(&args.expected_release_id)?;
        let (scope, checkout) = self.resolve(&args.scope, false)?;
        let mut store =
            MemoryStore::open_existing(self.engine.state_root(), scope).map_err(error)?;
        let page = store
            .history(
                &args.record_id,
                args.before_revision,
                args.limit,
                args.max_bytes,
            )
            .map_err(error)?;
        for record in &page.records {
            verify_checkout(record, checkout.as_deref())?;
        }
        serde_json::to_value(page).map_err(error)
    }
}
fn verify_checkout(record: &memory::Record, checkout: Option<&str>) -> Result<(), String> {
    if record
        .data
        .applicability
        .checkout_id
        .as_deref()
        .is_some_and(|id| Some(id) != checkout)
    {
        return Err("memory record belongs to another checkout".into());
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::memory::{Applicability, ClaimBasis, RecordStatus, Source};
    use crate::release::{AdapterPayload, ReleaseId};
    use std::fs;
    use std::os::unix::fs::DirBuilderExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture {
        root: PathBuf,
        service: Arc<MemoryService>,
        selection: Selection,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "mirket-memory-service-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
            let root = root.canonicalize().unwrap();
            for d in ["state", "project"] {
                fs::DirBuilder::new()
                    .mode(0o700)
                    .create(root.join(d))
                    .unwrap();
            }
            fs::write(root.join("node"), "test-only; never executed").unwrap();
            let payload = AdapterPayload::explicit(&root.join("node")).unwrap();
            let engine = Engine::open(
                ReleaseId::parse("test-memory").unwrap(),
                payload,
                root.join("state"),
            )
            .unwrap();
            Self {
                selection: Selection::Project {
                    project_root: root.join("project"),
                },
                service: Arc::new(MemoryService::new(engine, MemoryOptions::default())),
                root,
            }
        }
        fn record(&self, content: &str) -> RecordInput {
            RecordInput {
                kind: RecordKind::Fact,
                basis: ClaimBasis::Observed,
                content: content.into(),
                sources: vec![Source {
                    source_id: "fixture-source".into(),
                    revision: "fixture-rev-1".into(),
                    locator: "line 1".into(),
                    captured_at: 1,
                }],
                status: RecordStatus::Current,
                critical: false,
                applicability: Applicability::default(),
                valid_from: None,
                valid_until: None,
            }
        }
        fn prepare(&self, mutation: Mutation) -> String {
            self.service
                .prepare(PrepareArgs {
                    expected_release_id: "test-memory".into(),
                    scope: self.selection.clone(),
                    mutation,
                })
                .unwrap()["operation_id"]
                .as_str()
                .unwrap()
                .into()
        }
        fn args(&self, id: &str) -> OperationArgs {
            OperationArgs {
                expected_release_id: "test-memory".into(),
                scope: self.selection.clone(),
                operation_id: id.into(),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn reconnect(f: &Fixture) -> Arc<MemoryService> {
        let payload = AdapterPayload::explicit(&f.root.join("node")).unwrap();
        let engine = Engine::open(
            ReleaseId::parse("test-memory").unwrap(),
            payload,
            f.root.join("state"),
        )
        .unwrap();
        Arc::new(MemoryService::new(
            engine,
            MemoryOptions {
                backup_root: f.service.options.backup_root.clone(),
                embedding_runtime: f.service.options.embedding_runtime.clone(),
            },
        ))
    }

    fn maintenance_preparation(
        f: &Fixture,
        name: &str,
        selection: &Selection,
        canonical: bool,
    ) -> (PathBuf, MemoryService, String, Receipt, String) {
        let state = f.root.join(name);
        fs::DirBuilder::new().mode(0o700).create(&state).unwrap();
        let service = MemoryService::new(
            Engine::open(
                ReleaseId::parse("test-memory").unwrap(),
                f.service.engine.payload.clone(),
                state.clone(),
            )
            .unwrap(),
            MemoryOptions::default(),
        );
        if canonical {
            let (scope, _) = service.resolve(selection, true).unwrap();
            MemoryStore::open(&state, scope)
                .unwrap()
                .create("op-seed", f.record("Unrelated preserved record"), None)
                .unwrap();
        }
        let mut input = f.record("Unused prepared content");
        if matches!(selection, Selection::SharedPreferences) {
            input.kind = RecordKind::Preference;
        }
        let live = service
            .prepare(PrepareArgs {
                expected_release_id: "test-memory".into(),
                scope: selection.clone(),
                mutation: Mutation::Create {
                    record: input,
                    supersedes: None,
                },
            })
            .unwrap()["operation_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let mut receipt = service
            .receipt(&OperationArgs {
                expected_release_id: "test-memory".into(),
                scope: selection.clone(),
                operation_id: live.clone(),
            })
            .unwrap();
        receipt.expires_at = now().unwrap() - 1;
        // Intentional clock fixture issued with its original expired bytes.
        let id = OperationStore::open_existing(&state)
            .unwrap()
            .lock()
            .unwrap()
            .issue(&serde_json::to_vec(&receipt).unwrap())
            .unwrap();
        (state, service, id, receipt, live)
    }

    #[test]
    fn maintenance_retires_initial_absent_scopes_without_creating_canonical_authority() {
        let f = Fixture::new();
        let release = ReleaseId::parse("test-memory").unwrap();
        for (index, selection) in [f.selection.clone(), Selection::SharedPreferences]
            .iter()
            .enumerate()
        {
            let (state, service, id, _, unexpired) =
                maintenance_preparation(&f, &format!("retire-{index}"), selection, false);
            assert!(retire_expired_preparation(&state, &release, selection, &id).is_err());
            drop(service); // no graceful retirement: persisted evidence only
            let path = state.join("operations").join(format!("{unexpired}.json"));
            let before = fs::read(&path).unwrap();
            assert!(retire_expired_preparation(&state, &release, selection, &unexpired).is_err());
            assert_eq!(fs::read(path).unwrap(), before);
            assert!(
                retire_expired_preparation(
                    &state,
                    &ReleaseId::parse("other").unwrap(),
                    selection,
                    &id
                )
                .is_err()
            );
            let report = retire_expired_preparation(&state, &release, selection, &id).unwrap();
            assert_eq!(report["status"], "expired_not_started");
            let path = state.join("operations").join(format!("{id}.json"));
            let completed = fs::read(&path).unwrap();
            retire_expired_preparation(&state, &release, selection, &id).unwrap();
            assert_eq!(fs::read(path).unwrap(), completed);
            assert!(!state.join("memory").exists());
            assert!(!state.join("memory-lifecycle").exists());
            assert!(retire_expired_preparation(&state, &release, selection, "op-missing").is_err());
        }
    }

    #[test]
    fn maintenance_memory_refuses_drift_started_missing_corrupt_and_canonical_contradictions() {
        let f = Fixture::new();
        let release = ReleaseId::parse("test-memory").unwrap();
        for variant in 0..7 {
            let (state, service, id, mut receipt, _) =
                maintenance_preparation(&f, &format!("refuse-{variant}"), &f.selection, true);
            let database = state.join("memory").join(format!(
                "scope-{}.sqlite3",
                crate::memory_lifecycle::scope_key(&receipt.scope).unwrap()
            ));
            match variant {
                0 => {
                    receipt.intent_revision = crate::engine::revision(b"drifted content");
                }
                1 => {
                    receipt.status = "unknown".into();
                }
                2 => {
                    fs::remove_file(&database).unwrap();
                }
                3 => {
                    fs::write(&database, b"corrupt canonical authority").unwrap();
                }
                4 => {
                    MemoryStore::open(&state, receipt.scope.clone())
                        .unwrap()
                        .create(&id, f.record("Actually committed"), None)
                        .unwrap();
                }
                5 => {
                    crate::memory_lifecycle::set_pending_restore(
                        &state,
                        &receipt.scope,
                        &id,
                        "snapshot-retained",
                    )
                    .unwrap();
                }
                _ => {
                    fs::remove_file(state.join("memory-lifecycle/authority.sqlite3")).unwrap();
                }
            }
            if variant < 2 {
                OperationStore::open_existing(&state)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .update(&id, &serde_json::to_vec(&receipt).unwrap())
                    .unwrap();
            }
            drop(service);
            let path = state.join("operations").join(format!("{id}.json"));
            let before = fs::read(&path).unwrap();
            assert!(
                retire_expired_preparation(&state, &release, &f.selection, &id).is_err(),
                "variant {variant}"
            );
            assert_eq!(fs::read(path).unwrap(), before);
        }
    }

    #[test]
    fn maintenance_memory_checks_exact_forgetting_authority_and_preserves_other_records() {
        let f = Fixture::new();
        let release = ReleaseId::parse("test-memory").unwrap();
        let (state, service, unused, original, _) =
            maintenance_preparation(&f, "forget-authority", &f.selection, true);
        let mut canonical = MemoryStore::open(&state, original.scope.clone()).unwrap();
        let target = canonical
            .create("op-target", f.record("Forget this target"), None)
            .unwrap();
        drop(canonical);
        let mut forget = original.clone();
        forget.kind = "memory_forget".into();
        let reference = RecordRef {
            record_id: target.record_id.clone(),
            revision: target.revision,
        };
        forget.target = Some(reference.clone());
        forget.record = Some(reference);
        let id = OperationStore::open_existing(&state)
            .unwrap()
            .lock()
            .unwrap()
            .issue(&serde_json::to_vec(&forget).unwrap())
            .unwrap();
        crate::memory_lifecycle::forget(
            &state,
            original.scope.clone(),
            &target.record_id,
            &id,
            target.revision,
        )
        .unwrap();
        drop(service);
        let before = fs::read(state.join("operations").join(format!("{id}.json"))).unwrap();
        assert!(retire_expired_preparation(&state, &release, &f.selection, &id).is_err());
        assert_eq!(
            fs::read(state.join("operations").join(format!("{id}.json"))).unwrap(),
            before
        );
        retire_expired_preparation(&state, &release, &f.selection, &unused).unwrap();
        let mut canonical = MemoryStore::open_existing(&state, original.scope).unwrap();
        assert!(
            canonical
                .read_mutation("op-seed")
                .unwrap()
                .unwrap()
                .available
        );
        assert!(
            !canonical
                .read_mutation("op-target")
                .unwrap()
                .unwrap()
                .available
        );
    }

    #[test]
    fn unused_memory_expiry_reclaims_capacity_without_canonical_work() {
        let f = Fixture::new();
        let mut ids = Vec::new();
        for _ in 0..64 {
            let id = f.prepare(Mutation::Create {
                record: f.record("Never committed"),
                supersedes: None,
            });
            ids.push(id);
        }
        assert_eq!(f.service.pending.lock().unwrap().len(), 64);
        for id in &ids {
            let mut pending = f.service.pending.lock().unwrap();
            let receipt = &mut pending.get_mut(id).unwrap().receipt;
            receipt.expires_at = 0;
            f.service.save_receipt(id, receipt, false).unwrap();
        }
        let fresh = f.prepare(Mutation::Create {
            record: f.record("Still only prepared"),
            supersedes: None,
        });
        assert_eq!(f.service.pending.lock().unwrap().len(), 1);
        assert!(!f.root.join("state/memory").exists());
        for id in ids {
            assert_eq!(
                f.service.inspect(f.args(&id)).unwrap()["status"],
                "cancelled"
            );
            let store = OperationStore::open_existing(f.service.engine.state_root()).unwrap();
            assert!(store.lock().unwrap().update(&id, b"reopen").is_err());
        }
        assert_eq!(
            f.service.inspect(f.args(&fresh)).unwrap()["status"],
            "prepared"
        );
    }

    #[test]
    fn memory_commit_refuses_changed_original_intent_before_consumption() {
        let f = Fixture::new();
        let id = f.prepare(Mutation::Create {
            record: f.record("Original intended content"),
            supersedes: None,
        });
        let original = f.service.receipt(&f.args(&id)).unwrap();
        let mut changed = vec![original.clone(); 4];
        changed[0].release_id = "another-release".into();
        changed[1].expires_at += 1;
        changed[2].target = Some(RecordRef {
            record_id: "mem-unrelated".into(),
            revision: 1,
        });
        changed[3].intent_revision = "different-intent".into();
        let path = f.root.join("state/operations").join(format!("{id}.json"));
        for receipt in changed {
            f.service.save_receipt(&id, &receipt, false).unwrap();
            let before = fs::read(&path).unwrap();
            assert!(
                f.service
                    .commit(f.args(&id))
                    .unwrap_err()
                    .contains("receipt changed")
            );
            assert_eq!(fs::read(&path).unwrap(), before);
            assert!(f.service.pending.lock().unwrap().contains_key(&id));
            assert!(!f.root.join("state/memory").exists());
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn memory_shutdown_retires_only_unstarted_intents_and_keeps_unknown() {
        let f = Fixture::new();
        let prepared = f.prepare(Mutation::Create {
            record: f.record("Never started"),
            supersedes: None,
        });
        let unknown = f.prepare(Mutation::Create {
            record: f.record("Unconfirmed write"),
            supersedes: None,
        });
        {
            let mut pending = f.service.pending.lock().unwrap();
            let receipt = &mut pending.get_mut(&unknown).unwrap().receipt;
            receipt.status = "unknown".into();
            f.service.save_receipt(&unknown, receipt, false).unwrap();
        }
        let path = f
            .root
            .join("state/operations")
            .join(format!("{unknown}.json"));
        let before = fs::read(&path).unwrap();
        f.service.shutdown().await.unwrap();
        assert_eq!(
            f.service.inspect(f.args(&prepared)).unwrap()["status"],
            "cancelled"
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        assert!(f.service.pending.lock().unwrap().contains_key(&unknown));
        assert!(!f.root.join("state/memory").exists());
        assert!(
            f.service
                .prepare(PrepareArgs {
                    expected_release_id: "test-memory".into(),
                    scope: f.selection.clone(),
                    mutation: Mutation::Create {
                        record: f.record("After close"),
                        supersedes: None
                    },
                })
                .unwrap_err()
                .contains("shutting down")
        );
        assert!(
            f.service
                .commit(f.args(&prepared))
                .unwrap_err()
                .contains("shutting down")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn unused_memory_retirement_preserves_drift_and_retries_exact_confirmation() {
        let f = Fixture::new();
        let id = f.prepare(Mutation::Create {
            record: f.record("Never started"),
            supersedes: None,
        });
        let original = f.service.receipt(&f.args(&id)).unwrap();
        let mut changed = original.clone();
        changed.status = "unknown".into();
        f.service.save_receipt(&id, &changed, false).unwrap();
        let path = f.root.join("state/operations").join(format!("{id}.json"));
        let drifted = fs::read(&path).unwrap();
        assert!(f.service.shutdown().await.unwrap_err().contains("changed"));
        assert_eq!(fs::read(&path).unwrap(), drifted);
        assert!(f.service.pending.lock().unwrap().contains_key(&id));
        // Fixture restores the unstarted authority, then models finalization
        // succeeding before its acknowledgement reached this connection.
        f.service.save_receipt(&id, &original, false).unwrap();
        let mut cancelled = original;
        cancelled.status = "cancelled".into();
        f.service.save_receipt(&id, &cancelled, true).unwrap();
        let finalized = fs::read(&path).unwrap();
        f.service.shutdown().await.unwrap();
        assert_eq!(fs::read(&path).unwrap(), finalized);
        assert!(!f.service.pending.lock().unwrap().contains_key(&id));
        assert!(!f.root.join("state/memory").exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn forgetting_scrubs_cancelled_targets_and_shutdown_discards_consumed_intents() {
        let f = Fixture::new();
        let created = f.prepare(Mutation::Create {
            record: f.record("Original source"),
            supersedes: None,
        });
        let saved = f.service.commit(f.args(&created)).unwrap();
        let record_id = saved["record"]["record_id"].as_str().unwrap().to_owned();
        let other = reconnect(&f);
        let retired = other
            .prepare(PrepareArgs {
                expected_release_id: "test-memory".into(),
                scope: f.selection.clone(),
                mutation: Mutation::Revise {
                    record_id: record_id.clone(),
                    expected_revision: 1,
                    record: f.record("Unused correction"),
                },
            })
            .unwrap()["operation_id"]
            .as_str()
            .unwrap()
            .to_owned();
        other.shutdown().await.unwrap();
        let mut expired_receipt = other.receipt(&f.args(&retired)).unwrap();
        expired_receipt.status = "prepared".into();
        expired_receipt.expires_at = now().unwrap() - 1;
        let expired = {
            let store = OperationStore::open_existing(f.service.engine.state_root()).unwrap();
            let locked = store.lock().unwrap();
            let id = locked
                .issue(&serde_json::to_vec(&expired_receipt).unwrap())
                .unwrap();
            expired_receipt.status = "expired_not_started".into();
            locked
                .finalize(&id, &serde_json::to_vec(&expired_receipt).unwrap())
                .unwrap();
            id
        };
        let unconsumed = f.prepare(Mutation::Revise {
            record_id: record_id.clone(),
            expected_revision: 1,
            record: f.record("Another unused correction"),
        });
        let forget = f
            .service
            .prepare_forget(ForgetPrepareArgs {
                expected_release_id: "test-memory".into(),
                scope: f.selection.clone(),
                record_id,
                expected_revision: 1,
            })
            .unwrap()["operation_id"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            f.service.commit_forget(f.args(&forget)).unwrap()["status"],
            "forgotten"
        );
        for id in [&retired, &unconsumed, &expired] {
            assert!(
                !f.root
                    .join("state/operations")
                    .join(format!("{id}.json"))
                    .exists()
            );
        }
        assert!(f.service.pending.lock().unwrap().contains_key(&unconsumed));
        f.service.shutdown().await.unwrap();
        assert!(!f.service.pending.lock().unwrap().contains_key(&unconsumed));
        assert!(
            !f.root
                .join("state/operations")
                .join(format!("{unconsumed}.json"))
                .exists()
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_blocks_registration_and_late_derived_publication() {
        let f = Fixture::new();
        let operation = f.prepare(Mutation::Create {
            record: f.record("Selected before shutdown"),
            supersedes: None,
        });
        f.service.commit(f.args(&operation)).unwrap();
        let (scope, batch) = select_service_index(&f, 32);
        let output = synthetic_index_output(&batch);
        let mut canonical =
            MemoryStore::open_existing(f.service.engine.state_root(), scope.clone()).unwrap();
        let path = PathBuf::from(canonical.lifecycle_connection().path().unwrap());
        drop(canonical);
        let before = fs::read(&path).unwrap();
        f.service.shutdown().await.unwrap();
        let result = f.service.finish_index_page(scope, batch, Some(Ok(output)));
        assert_eq!(result["status"], "pending");
        assert!(result["indexed"].as_array().unwrap().is_empty());
        assert_eq!(fs::read(path).unwrap(), before);
        let new_project = f.root.join("unregistered-after-close");
        fs::create_dir(&new_project).unwrap();
        assert!(
            f.service
                .prepare(PrepareArgs {
                    expected_release_id: "test-memory".into(),
                    scope: Selection::Project {
                        project_root: new_project.clone()
                    },
                    mutation: Mutation::Create {
                        record: f.record("Must not register"),
                        supersedes: None
                    },
                })
                .unwrap_err()
                .contains("shutting down")
        );
        assert!(
            identity::lookup(f.service.engine.state_root(), &new_project)
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lost_confirmation_recovery_across_connections_has_no_secondary_effects() {
        let mut f = Fixture::new();
        let backup_root = f.root.join("backups");
        Arc::get_mut(&mut f.service).unwrap().options.backup_root = Some(backup_root.clone());
        closed_index_runtime(&mut f).await;
        let id = f.prepare(Mutation::Create {
            record: f.record("Canonical authority survives lost convenience confirmation"),
            supersedes: None,
        });
        let receipt_path = f.root.join("state/operations").join(format!("{id}.json"));
        let prepared_bytes = fs::read(&receipt_path).unwrap();
        let committed = f.service.commit(f.args(&id)).unwrap();
        let snapshot = backup_root.join(format!(
            "{}.sqlite3",
            committed["backup"]["snapshot"]["snapshot_id"]
                .as_str()
                .unwrap()
        ));
        let snapshot_bytes = fs::read(&snapshot).unwrap();
        let catalog_bytes = fs::read(backup_root.join("catalog.sqlite3")).unwrap();
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut canonical =
            MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        let canonical_path = PathBuf::from(canonical.lifecycle_connection().path().unwrap());
        let canonical_bytes = fs::read(&canonical_path).unwrap();
        drop(canonical);
        let other = reconnect(&f);
        assert_ne!(other.engine.owner_id(), f.service.engine.owner_id());
        for service in [&f.service, &other] {
            // Model an interrupted convenience write, not an authorized receipt regression.
            fs::write(&receipt_path, &prepared_bytes).unwrap();
            assert_eq!(service.inspect(f.args(&id)).unwrap()["status"], "committed");
            assert_eq!(fs::read(&receipt_path).unwrap(), prepared_bytes);
            let mut recovered = service.commit(f.args(&id)).unwrap();
            assert_eq!(recovered["status"], "committed");
            assert_eq!(recovered["reconciled"], true);
            assert!(recovered["receipt_warning"].is_null());
            assert_eq!(recovered["record"]["revision"], 1);
            assert_eq!(
                recovered["record"]["record_id"],
                committed["record"]["record_id"]
            );
            let before_refresh = recovered.clone();
            service.refresh_committed(&mut recovered).await;
            assert_eq!(recovered, before_refresh);
            assert!(recovered.get("semantic_index").is_none());
            assert_eq!(fs::read(&canonical_path).unwrap(), canonical_bytes);
            assert_eq!(fs::read(&snapshot).unwrap(), snapshot_bytes);
            assert_eq!(
                fs::read(backup_root.join("catalog.sqlite3")).unwrap(),
                catalog_bytes
            );
            assert!(
                OperationStore::open_existing(service.engine.state_root())
                    .unwrap()
                    .lock()
                    .unwrap()
                    .update(&id, b"changed confirmation")
                    .is_err()
            );
        }
        let jobs = f.root.join("state/jobs");
        assert!(!jobs.exists() || fs::read_dir(jobs).unwrap().next().is_none());
    }

    #[test]
    fn recovery_refuses_missing_or_mismatched_canonical_operation_authority() {
        let f = Fixture::new();
        let saved = f.prepare(Mutation::Create {
            record: f.record("Existing canonical record must remain unchanged"),
            supersedes: None,
        });
        let committed = f.service.commit(f.args(&saved)).unwrap();
        let missing = f.prepare(Mutation::Create {
            record: f.record("Uncommitted foreign intent must not be replayed"),
            supersedes: None,
        });
        f.service.pending.lock().unwrap().remove(&missing);
        let other = reconnect(&f);
        let path = f
            .root
            .join("state/operations")
            .join(format!("{missing}.json"));
        let before = fs::read(&path).unwrap();
        assert_eq!(
            f.service.commit(f.args(&missing)).unwrap()["status"],
            "prepared"
        );
        assert!(
            other
                .commit(f.args(&missing))
                .unwrap_err()
                .contains("another connection")
        );
        assert_eq!(fs::read(path).unwrap(), before);

        // Bind a protected receipt to the wrong result while retaining the real
        // issued operation ID. Canonical proof must not bless that disagreement.
        let id = f.prepare(Mutation::Create {
            record: f.record("Second committed record"),
            supersedes: None,
        });
        let path = f.root.join("state/operations").join(format!("{id}.json"));
        let prepared = fs::read(&path).unwrap();
        f.service.commit(f.args(&id)).unwrap();
        fs::write(&path, prepared).unwrap();
        let mut receipt = f.service.receipt(&f.args(&id)).unwrap();
        receipt.record = Some(RecordRef {
            record_id: committed["record"]["record_id"].as_str().unwrap().into(),
            revision: 1,
        });
        f.service.save_receipt(&id, &receipt, false).unwrap();
        let before = fs::read(&path).unwrap();
        for service in [&f.service, &other] {
            assert!(
                service
                    .commit(f.args(&id))
                    .unwrap_err()
                    .contains("differs from its canonical")
            );
            assert_eq!(fs::read(&path).unwrap(), before);
        }
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut canonical =
            MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        assert_eq!(
            canonical
                .search(SearchRequest::default())
                .unwrap()
                .hits
                .len(),
            2
        );
        assert!(canonical.read_mutation(&missing).unwrap().is_none());
    }

    #[test]
    fn unavailable_forgotten_mutation_cannot_finalize_stale_confirmation_or_resurrect() {
        let f = Fixture::new();
        let id = f.prepare(Mutation::Create {
            record: f.record("Content intentionally forgotten"),
            supersedes: None,
        });
        let path = f.root.join("state/operations").join(format!("{id}.json"));
        let stale = fs::read(&path).unwrap();
        let committed = f.service.commit(f.args(&id)).unwrap();
        let record_id = committed["record"]["record_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let forget = f
            .service
            .prepare_forget(ForgetPrepareArgs {
                expected_release_id: "test-memory".into(),
                scope: f.selection.clone(),
                record_id: record_id.clone(),
                expected_revision: 1,
            })
            .unwrap()["operation_id"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            f.service.commit_forget(f.args(&forget)).unwrap()["status"],
            "forgotten"
        );
        assert!(!path.exists());
        // An old external receipt copy is not a fresh operation or content authority.
        fs::write(&path, &stale).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let other = reconnect(&f);
        let result = f.service.commit(f.args(&id)).unwrap();
        assert_eq!(result["mutation"]["available"], false);
        assert_eq!(result["intent_available"], false);
        assert!(result.get("reconciled").is_none());
        assert!(other.commit(f.args(&id)).is_err());
        assert_eq!(fs::read(&path).unwrap(), stale);
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut canonical =
            MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        assert!(!canonical.read_mutation(&id).unwrap().unwrap().available);
        assert!(canonical.read(&record_id, Some(1)).is_err());
        assert!(
            canonical
                .search(SearchRequest::default())
                .unwrap()
                .hits
                .is_empty()
        );
    }

    #[test]
    fn concurrent_commit_and_lost_confirmation_reconcile_one_canonical_revision() {
        let f = Fixture::new();
        let id = f.prepare(Mutation::Create {
            record: f.record("Durable memory value"),
            supersedes: None,
        });
        let receipt_path = f.root.join("state/operations").join(format!("{id}.json"));
        let before_confirmation = fs::read(&receipt_path).unwrap();
        std::thread::scope(|s| {
            let a = s.spawn(|| f.service.commit(f.args(&id)));
            let b = s.spawn(|| f.service.commit(f.args(&id)));
            let results = [a.join().unwrap(), b.join().unwrap()];
            assert!(results.iter().any(|r| r.is_ok()));
        });
        let status = f.service.inspect(f.args(&id)).unwrap();
        assert_eq!(status["status"], "committed");
        assert!(
            OperationStore::open_existing(f.service.engine.state_root())
                .unwrap()
                .lock()
                .unwrap()
                .update(&id, b"reopened")
                .is_err()
        );
        let record_id = status["mutation"]["record_id"].as_str().unwrap();
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut store = MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        assert_eq!(
            store.search(SearchRequest::default()).unwrap().hits.len(),
            1
        );
        assert_eq!(store.read(record_id, None).unwrap().revision, 1);
        // Simulate loss of the convenience receipt write, independently of the
        // committed SQLite authority. Public APIs cannot regress a final receipt.
        fs::write(receipt_path, before_confirmation).unwrap();
        assert_eq!(
            f.service.inspect(f.args(&id)).unwrap()["status"],
            "committed"
        );
        assert_eq!(
            f.service.commit(f.args(&id)).unwrap()["status"],
            "committed"
        );
        assert_eq!(
            store.search(SearchRequest::default()).unwrap().hits.len(),
            1
        );
    }

    #[test]
    fn absent_canonical_database_does_not_turn_json_confirmation_into_commit_proof() {
        let f = Fixture::new();
        let id = f.prepare(Mutation::Create {
            record: f.record("Saved before database loss"),
            supersedes: None,
        });
        let receipt_path = f.root.join("state/operations").join(format!("{id}.json"));
        let prepared = fs::read(&receipt_path).unwrap();
        f.service.commit(f.args(&id)).unwrap();
        let memory = f.service.engine.state_root().join("memory");
        let preserved = f.root.join("preserved-memory");
        fs::rename(&memory, &preserved).unwrap();
        let status = f.service.inspect(f.args(&id)).unwrap();
        assert_eq!(status["status"], "confirmation_unavailable");
        assert_eq!(status["intent_available"], false);
        assert!(status["mutation"].is_null());
        assert_eq!(
            f.service.commit(f.args(&id)).unwrap()["status"],
            "confirmation_unavailable"
        );
        fs::write(&receipt_path, &prepared).unwrap();
        let stale = f.service.commit(f.args(&id)).unwrap();
        assert_eq!(stale["status"], "prepared");
        assert_eq!(stale["intent_available"], false);
        assert!(stale["mutation"].is_null());
        assert!(reconnect(&f).commit(f.args(&id)).is_err());
        assert_eq!(fs::read(receipt_path).unwrap(), prepared);
        assert!(!memory.exists());
        assert!(preserved.exists());
    }

    #[test]
    fn forgetting_has_a_distinct_permission_boundary_and_refuses_stale_intent() {
        let f = Fixture::new();
        let saved_id = f.prepare(Mutation::Create {
            record: f.record("Forget selected content"),
            supersedes: None,
        });
        let saved = f.service.commit(f.args(&saved_id)).unwrap();
        let record_id = saved["record"]["record_id"].as_str().unwrap().to_owned();
        let prepare_forget = |revision| {
            f.service
                .prepare_forget(ForgetPrepareArgs {
                    expected_release_id: "test-memory".into(),
                    scope: f.selection.clone(),
                    record_id: record_id.clone(),
                    expected_revision: revision,
                })
                .unwrap()["operation_id"]
                .as_str()
                .unwrap()
                .to_owned()
        };
        let stale_forget = prepare_forget(1);
        assert!(
            f.service
                .commit(f.args(&stale_forget))
                .unwrap_err()
                .contains("matching save or forgetting")
        );
        assert!(
            f.service
                .reconcile_forget(f.args(&stale_forget))
                .unwrap_err()
                .contains("no committed exclusion")
        );
        let changed = f.prepare(Mutation::Revise {
            record_id: record_id.clone(),
            expected_revision: 1,
            record: f.record("Newer selected content"),
        });
        f.service.commit(f.args(&changed)).unwrap();
        assert!(
            f.service
                .commit_forget(f.args(&stale_forget))
                .unwrap_err()
                .contains("revision changed")
        );
        let forget = prepare_forget(2);
        assert_eq!(
            f.service.commit_forget(f.args(&forget)).unwrap()["status"],
            "forgotten"
        );
        assert_eq!(
            f.service.inspect(f.args(&forget)).unwrap()["status"],
            "forgotten"
        );
        assert_eq!(
            f.service.reconcile_forget(f.args(&forget)).unwrap()["status"],
            "forgotten"
        );
        assert!(f.service.inspect(f.args(&saved_id)).is_err());
        assert!(f.service.inspect(f.args(&changed)).is_err());
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut store = MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        assert!(store.read(&record_id, None).is_err());
        assert!(store.read(&record_id, Some(1)).is_err());
        assert!(
            store
                .search(SearchRequest::default())
                .unwrap()
                .hits
                .is_empty()
        );
    }

    #[test]
    fn forgetting_receipt_stays_protected_until_backup_cleanup_reconciles() {
        let mut f = Fixture::new();
        let backup_root = f.root.join("backups");
        Arc::get_mut(&mut f.service).unwrap().options.backup_root = Some(backup_root.clone());
        let save = f.prepare(Mutation::Create {
            record: f.record("Forgotten content in a temporarily unavailable backup"),
            supersedes: None,
        });
        let saved = f.service.commit(f.args(&save)).unwrap();
        assert_eq!(saved["backup"]["status"], "eligible");
        let forget = f
            .service
            .prepare_forget(ForgetPrepareArgs {
                expected_release_id: "test-memory".into(),
                scope: f.selection.clone(),
                record_id: saved["record"]["record_id"].as_str().unwrap().into(),
                expected_revision: 1,
            })
            .unwrap()["operation_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let moved = f.root.join("temporarily-moved-backups");
        fs::rename(&backup_root, &moved).unwrap();
        assert!(f.service.commit_forget(f.args(&forget)).is_err());
        assert_eq!(
            f.service.inspect(f.args(&forget)).unwrap()["status"],
            "cleanup_pending"
        );
        {
            let locked = OperationStore::open_existing(f.service.engine.state_root())
                .unwrap()
                .lock()
                .unwrap();
            let mut bytes = locked.read(&forget).unwrap();
            bytes.push(b' '); // Same typed intent, different bytes: still protected/mutable.
            locked.update(&forget, &bytes).unwrap();
        }
        fs::rename(moved, &backup_root).unwrap();
        let result = f.service.reconcile_forget(f.args(&forget)).unwrap();
        assert_eq!(result["status"], "forgotten");
        assert!(result["receipt_warning"].is_null());
        let locked = OperationStore::open_existing(f.service.engine.state_root())
            .unwrap()
            .lock()
            .unwrap();
        let mut bytes = locked.read(&forget).unwrap();
        bytes.push(b' ');
        assert!(locked.update(&forget, &bytes).is_err());
    }

    #[test]
    fn stale_correction_cannot_consume_or_overwrite_a_newer_revision() {
        let f = Fixture::new();
        let id = f.prepare(Mutation::Create {
            record: f.record("First value"),
            supersedes: None,
        });
        let saved = f.service.commit(f.args(&id)).unwrap();
        let record_id = saved["record"]["record_id"].as_str().unwrap().to_owned();
        let first = f.prepare(Mutation::Revise {
            record_id: record_id.clone(),
            expected_revision: 1,
            record: f.record("Second value"),
        });
        let stale = f.prepare(Mutation::Revise {
            record_id: record_id.clone(),
            expected_revision: 1,
            record: f.record("Stale value"),
        });
        f.service.commit(f.args(&first)).unwrap();
        assert!(
            f.service
                .commit(f.args(&stale))
                .unwrap_err()
                .contains("no longer current")
        );
        assert!(f.service.pending.lock().unwrap().contains_key(&stale));
        let value = f
            .service
            .read(ReadArgs {
                expected_release_id: "test-memory".into(),
                scope: f.selection.clone(),
                record_id,
                revision: None,
            })
            .unwrap();
        assert_eq!(value["record"]["data"]["content"], "Second value");
    }

    #[test]
    fn stale_release_and_fresh_read_do_not_register_memory_scope() {
        let f = Fixture::new();
        let result = f.service.search_lexical(SearchArgs {
            expected_release_id: "test-memory".into(),
            scope: f.selection.clone(),
            request: SearchRequest::default(),
        });
        assert!(result.unwrap_err().contains("no registered memory"));
        assert!(
            identity::lookup(f.service.engine.state_root(), &f.root.join("project"))
                .unwrap()
                .is_none()
        );
        let result = f.service.prepare(PrepareArgs {
            expected_release_id: "old-release".into(),
            scope: f.selection.clone(),
            mutation: Mutation::Create {
                record: f.record("Must not persist"),
                supersedes: None,
            },
        });
        assert!(result.is_err());
        assert!(
            identity::lookup(f.service.engine.state_root(), &f.root.join("project"))
                .unwrap()
                .is_none()
        );
        assert!(!f.service.engine.state_root().join("memory").exists());
    }

    #[test]
    fn automatic_daily_backup_is_verified_and_replayed_commit_does_not_capture_again() {
        let mut f = Fixture::new();
        let backup_root = f.root.join("backups");
        Arc::get_mut(&mut f.service).unwrap().options.backup_root = Some(backup_root.clone());
        let operation = f.prepare(Mutation::Create {
            record: f.record("Durable record included in the daily recovery point"),
            supersedes: None,
        });
        let committed = f.service.commit(f.args(&operation)).unwrap();
        assert_eq!(committed["status"], "committed");
        assert_eq!(committed["backup"]["status"], "eligible");
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let saved = crate::memory_backup::inspect_backups(
            f.service.engine.state_root(),
            &backup_root,
            scope.clone(),
        )
        .unwrap();
        assert_eq!(saved.len(), 1);
        assert!(saved[0].eligible);
        assert_eq!(
            committed["backup"]["snapshot"]["snapshot_id"],
            saved[0].snapshot_id
        );
        let snapshot_path = backup_root.join(format!("{}.sqlite3", saved[0].snapshot_id));
        let snapshot_bytes = fs::read(&snapshot_path).unwrap();
        let snapshot = rusqlite::Connection::open_with_flags(
            &snapshot_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let content: String = snapshot
            .query_row(
                "SELECT content FROM revisions WHERE record_id=?1 AND is_current=1",
                [committed["record"]["record_id"].as_str().unwrap()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            content,
            "Durable record included in the daily recovery point"
        );
        drop(snapshot);
        let catalog_before = fs::read(backup_root.join("catalog.sqlite3")).unwrap();
        assert_eq!(
            f.service.commit(f.args(&operation)).unwrap()["status"],
            "committed"
        );
        assert_eq!(
            fs::read(backup_root.join("catalog.sqlite3")).unwrap(),
            catalog_before
        );
        assert_eq!(fs::read(snapshot_path).unwrap(), snapshot_bytes);
        let another = f.prepare(Mutation::Create {
            record: f.record("Another canonical record after the daily capture"),
            supersedes: None,
        });
        let another = f.service.commit(f.args(&another)).unwrap();
        assert_eq!(another["status"], "committed");
        assert_eq!(
            another["backup"]["snapshot"]["snapshot_id"],
            saved[0].snapshot_id
        );
        let mut live = MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        assert_eq!(live.search(SearchRequest::default()).unwrap().hits.len(), 2);
        assert_eq!(
            live.read(committed["record"]["record_id"].as_str().unwrap(), None)
                .unwrap()
                .revision,
            1
        );
    }

    #[test]
    fn backup_admission_failure_preserves_committed_status_and_unrelated_records() {
        let mut f = Fixture::new();
        let backup_root = f.root.join("backups");
        Arc::get_mut(&mut f.service).unwrap().options.backup_root = Some(backup_root.clone());
        let first = f.prepare(Mutation::Create {
            record: f.record("Earlier canonical record survives"),
            supersedes: None,
        });
        let first = f.service.commit(f.args(&first)).unwrap();
        assert_eq!(first["backup"]["status"], "eligible");
        let other_scope = Scope::Project("unrelated-service-scope".into());
        let mut other =
            MemoryStore::open(f.service.engine.state_root(), other_scope.clone()).unwrap();
        let unrelated = other
            .create(
                "op-unrelated-record",
                f.record("Other scope survives backup failure"),
                None,
            )
            .unwrap();
        let other_path = PathBuf::from(other.lifecycle_connection().path().unwrap());
        drop(other);
        let unrelated_bytes = fs::read(&other_path).unwrap();
        let catalog_before = fs::read(backup_root.join("catalog.sqlite3")).unwrap();
        let held = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(backup_root.join("catalog.lock"))
            .unwrap();
        held.try_lock().unwrap();
        let operation = f.prepare(Mutation::Create {
            record: f.record("Committed despite unavailable backup admission"),
            supersedes: None,
        });
        let committed = f.service.commit(f.args(&operation)).unwrap();
        assert_eq!(committed["status"], "committed");
        assert_eq!(committed["backup"]["status"], "degraded");
        assert!(
            committed["backup"]["warning"]
                .as_str()
                .unwrap()
                .contains("busy")
        );
        assert_eq!(
            f.service.inspect(f.args(&operation)).unwrap()["status"],
            "committed"
        );
        assert_eq!(
            fs::read(backup_root.join("catalog.sqlite3")).unwrap(),
            catalog_before
        );
        assert_eq!(fs::read(&other_path).unwrap(), unrelated_bytes);
        drop(held);
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut live = MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        assert_eq!(
            live.read(first["record"]["record_id"].as_str().unwrap(), None)
                .unwrap()
                .data
                .content,
            "Earlier canonical record survives"
        );
        assert_eq!(
            live.read(committed["record"]["record_id"].as_str().unwrap(), None)
                .unwrap()
                .data
                .content,
            "Committed despite unavailable backup admission"
        );
        let mut other =
            MemoryStore::open_existing(f.service.engine.state_root(), other_scope).unwrap();
        assert_eq!(
            other.read(&unrelated.record_id, None).unwrap().data.content,
            "Other scope survives backup failure"
        );
    }

    #[test]
    fn delayed_index_open_cannot_recreate_a_missing_canonical_file_or_directory() {
        for remove_directory in [false, true] {
            let f = Fixture::new();
            let operation = f.prepare(Mutation::Create {
                record: f.record("Existing canonical record selected for derived work"),
                supersedes: None,
            });
            let committed = f.service.commit(f.args(&operation)).unwrap();
            let scope = f.service.resolve(&f.selection, false).unwrap().0;
            let mut live =
                MemoryStore::open_for_index(f.service.engine.state_root(), scope.clone()).unwrap();
            let path = PathBuf::from(live.lifecycle_connection().path().unwrap());
            drop(live);
            let removed = if remove_directory {
                path.parent().unwrap().to_path_buf()
            } else {
                path.clone()
            };
            let preserved = f.root.join("preserved-canonical");
            fs::rename(&removed, &preserved).unwrap();
            let preserved_file = if remove_directory {
                preserved.join(path.file_name().unwrap())
            } else {
                preserved.clone()
            };
            let before = fs::read(&preserved_file).unwrap();
            assert!(MemoryStore::open_for_index(f.service.engine.state_root(), scope).is_err());
            assert!(!removed.exists());
            assert!(!path.exists());
            assert_eq!(fs::read(preserved_file).unwrap(), before);
            assert_eq!(
                f.service.inspect(f.args(&operation)).unwrap()["status"],
                "confirmation_unavailable"
            );
            assert_eq!(committed["status"], "committed");
        }
    }

    #[test]
    fn delayed_index_open_obeys_pending_restore_exclusion() {
        let f = Fixture::new();
        let operation = f.prepare(Mutation::Create {
            record: f.record("Canonical record before interrupted restore"),
            supersedes: None,
        });
        f.service.commit(f.args(&operation)).unwrap();
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut live =
            MemoryStore::open_existing(f.service.engine.state_root(), scope.clone()).unwrap();
        let path = PathBuf::from(live.lifecycle_connection().path().unwrap());
        drop(live);
        let before = fs::read(&path).unwrap();
        crate::memory_lifecycle::set_pending_restore(
            f.service.engine.state_root(),
            &scope,
            "op-existing-restore",
            "selected-snapshot",
        )
        .unwrap();
        assert!(MemoryStore::open_existing(f.service.engine.state_root(), scope.clone()).is_err());
        assert!(
            MemoryStore::open_for_index(f.service.engine.state_root(), scope).is_err(),
            "derived publication must obey the same pending restore boundary"
        );
        assert_eq!(fs::read(path).unwrap(), before);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn unavailable_embedding_preserves_committed_status_and_lexical_recall_without_model_work()
     {
        let mut f = Fixture::new();
        let package = f.root.join("unused-package");
        fs::DirBuilder::new().mode(0o700).create(&package).unwrap();
        let runtime = Arc::new(
            crate::memory_runtime::MemoryRuntime::new(
                f.service.engine.state_root().to_path_buf(),
                ReleaseId::parse("test-memory").unwrap(),
                package,
                f.root.join("node"),
            )
            .unwrap(),
        );
        // Close admission before configuring the service: no model or worker can run.
        runtime.shutdown().await.unwrap();
        Arc::get_mut(&mut f.service)
            .unwrap()
            .options
            .embedding_runtime = Some(runtime);
        let operation = f.prepare(Mutation::Create {
            record: f.record("Lexical fallback remains available"),
            supersedes: None,
        });
        let mut committed = f.service.commit(f.args(&operation)).unwrap();
        let record_before = committed["record"].clone();
        f.service.refresh_committed(&mut committed).await;
        assert_eq!(committed["status"], "committed");
        assert_eq!(committed["record"], record_before);
        assert_eq!(committed["semantic_index"]["status"], "pending");
        assert!(
            committed["semantic_index"]["warning"]
                .as_str()
                .unwrap()
                .contains("shutting down")
        );
        let result = f
            .service
            .search(SearchArgs {
                expected_release_id: "test-memory".into(),
                scope: f.selection.clone(),
                request: SearchRequest {
                    query: "fallback".into(),
                    max_bytes: 4096,
                    ..SearchRequest::default()
                },
            })
            .await
            .unwrap();
        assert_eq!(result["hits"].as_array().unwrap().len(), 1);
        assert_eq!(
            result["hits"][0]["record"]["record_id"],
            record_before["record_id"]
        );
        assert_eq!(result["coverage"]["lexical_only"], true);
        assert!(
            result["coverage"]["semantic_warning"]
                .as_str()
                .unwrap()
                .contains("shutting down")
        );
        assert!(serde_json::to_vec(&result).unwrap().len() <= 4096);
        assert!(!f.service.engine.state_root().join("jobs").exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn invalid_search_is_rejected_before_embedding_admission_or_canonical_access() {
        let mut f = Fixture::new();
        let package = f.root.join("unused-package");
        fs::DirBuilder::new().mode(0o700).create(&package).unwrap();
        // The fixture binary has no execute permission and the package is empty:
        // even a regression cannot load a model or run executable fixture code.
        let runtime = Arc::new(
            crate::memory_runtime::MemoryRuntime::new(
                f.service.engine.state_root().to_path_buf(),
                ReleaseId::parse("test-memory").unwrap(),
                package,
                f.root.join("node"),
            )
            .unwrap(),
        );
        Arc::get_mut(&mut f.service)
            .unwrap()
            .options
            .embedding_runtime = Some(runtime);
        let operation = f.prepare(Mutation::Create {
            record: f.record("Canonical content remains unchanged by invalid recall"),
            supersedes: None,
        });
        f.service.commit(f.args(&operation)).unwrap();
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut live = MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        let path = PathBuf::from(live.lifecycle_connection().path().unwrap());
        drop(live);
        let before = fs::read(&path).unwrap();
        let normal = SearchRequest {
            query: "recall".into(),
            ..SearchRequest::default()
        };
        let invalid = vec![
            SearchRequest {
                limit: 0,
                ..normal.clone()
            },
            SearchRequest {
                max_bytes: 1023,
                ..normal.clone()
            },
            SearchRequest {
                query: "literal ".repeat(33),
                ..normal.clone()
            },
            SearchRequest {
                query: "x".repeat(2049),
                ..normal.clone()
            },
            SearchRequest {
                query: "invalid\0query".into(),
                ..normal.clone()
            },
            SearchRequest {
                context: memory::SearchContext {
                    task_key: Some("x".repeat(257)),
                    ..Default::default()
                },
                ..normal.clone()
            },
            SearchRequest {
                context: memory::SearchContext {
                    at_time: Some(u64::MAX),
                    ..Default::default()
                },
                ..normal.clone()
            },
        ];
        memory::validate_search_request(&SearchRequest::default()).unwrap();
        for request in invalid {
            let expected = memory::validate_search_request(&request)
                .unwrap_err()
                .to_string();
            let failed = f
                .service
                .search(SearchArgs {
                    expected_release_id: "test-memory".into(),
                    scope: f.selection.clone(),
                    request,
                })
                .await
                .unwrap_err();
            assert_eq!(failed, expected);
            assert!(
                !f.service.engine.state_root().join("jobs").exists(),
                "invalid recall must not register an embedding job"
            );
            assert_eq!(fs::read(&path).unwrap(), before);
        }
        f.service.shutdown().await.unwrap();
    }

    fn synthetic_index_output(batch: &memory::index::IndexBatch) -> crate::embedding_job::Output {
        crate::embedding_job::Output {
            schema: 1,
            input_revision: "synthetic-unit-result-not-model-evidence".into(),
            query: None,
            passages: batch
                .items
                .iter()
                .map(|item| {
                    let mut vector = vec![0.0; crate::embedding::DIMENSIONS];
                    vector[0] = 1.0;
                    crate::embedding::PassageEmbedding {
                        model_identity: batch.model_identity.clone(),
                        input_bytes: item.content.len(),
                        content_tokens: 1,
                        covered_bytes: item.content.len(),
                        complete: true,
                        chunks: vec![crate::embedding::ChunkEmbedding {
                            start_byte: 0,
                            end_byte: item.content.len(),
                            input_tokens: 3,
                            vector,
                        }],
                    }
                })
                .collect(),
        }
    }
    fn select_service_index(f: &Fixture, limit: usize) -> (Scope, memory::index::IndexBatch) {
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut store =
            MemoryStore::open_existing(f.service.engine.state_root(), scope.clone()).unwrap();
        let batch = store
            .select_index_batch(None, limit, memory::index::MAX_INDEX_BYTES)
            .unwrap();
        (scope, batch)
    }
    fn index_args(
        f: &Fixture,
        after_row_id: Option<i64>,
        limit: usize,
        max_bytes: usize,
    ) -> IndexArgs {
        IndexArgs {
            expected_release_id: "test-memory".into(),
            scope: f.selection.clone(),
            after_row_id,
            limit,
            max_bytes,
        }
    }
    async fn closed_index_runtime(f: &mut Fixture) {
        let package = f.root.join("unused-index-package");
        fs::DirBuilder::new().mode(0o700).create(&package).unwrap();
        let runtime = Arc::new(
            crate::memory_runtime::MemoryRuntime::new(
                f.service.engine.state_root().to_path_buf(),
                ReleaseId::parse("test-memory").unwrap(),
                package,
                f.root.join("node"),
            )
            .unwrap(),
        );
        runtime.shutdown().await.unwrap();
        Arc::get_mut(&mut f.service)
            .unwrap()
            .options
            .embedding_runtime = Some(runtime);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn index_refresh_preflight_never_registers_missing_scope_or_heavy_work() {
        let mut f = Fixture::new();
        for args in [
            index_args(&f, None, 0, 16384),
            index_args(&f, None, 33, 16384),
            index_args(&f, None, 1, 16383),
            index_args(&f, None, 1, 262145),
            index_args(&f, Some(0), 1, 16384),
        ] {
            assert!(
                f.service
                    .index_refresh(args)
                    .await
                    .unwrap_err()
                    .contains("invalid bounded")
            );
        }
        assert!(
            f.service
                .index_refresh(index_args(&f, None, 1, 16384))
                .await
                .unwrap_err()
                .contains("no embedding runtime")
        );
        closed_index_runtime(&mut f).await;
        assert!(
            f.service
                .index_refresh(index_args(&f, None, 1, 16384))
                .await
                .unwrap_err()
                .contains("no registered memory")
        );
        assert!(
            identity::lookup(f.service.engine.state_root(), &f.root.join("project"))
                .unwrap()
                .is_none()
        );
        assert!(!f.service.engine.state_root().join("memory").exists());
        assert!(!f.service.engine.state_root().join("jobs").exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn index_refresh_failed_inference_reports_pending_without_changing_canonical_bytes() {
        let mut f = Fixture::new();
        closed_index_runtime(&mut f).await;
        let operation = f.prepare(Mutation::Create {
            record: f.record("Canonical content survives unavailable indexing"),
            supersedes: None,
        });
        let committed = f.service.commit(f.args(&operation)).unwrap();
        let scope = f.service.resolve(&f.selection, false).unwrap().0;
        let mut store = MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        let path = PathBuf::from(store.lifecycle_connection().path().unwrap());
        drop(store);
        let bytes = fs::read(&path).unwrap();
        let result = f
            .service
            .index_refresh(index_args(&f, None, 1, 16384))
            .await
            .unwrap();
        assert_eq!(result["status"], "pending");
        assert_eq!(
            result["failed"][0]["reference"]["record_id"],
            committed["record"]["record_id"]
        );
        assert_eq!(result["failed"][0]["reason"], "embedding_unavailable");
        assert_eq!(result["coverage"]["indexed_heads"], 0);
        assert_eq!(result["coverage"]["missing_heads"], 1);
        assert_eq!(result["restart_required"], true);
        assert_eq!(fs::read(path).unwrap(), bytes);
        assert!(
            !serde_json::to_string(&result)
                .unwrap()
                .contains("Canonical content survives")
        );
        assert!(!f.service.engine.state_root().join("jobs").exists());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn index_refresh_empty_missing_items_page_reports_continuation_without_inference() {
        let mut f = Fixture::new();
        closed_index_runtime(&mut f).await;
        for content in [
            "First private source content",
            "Second private source content",
        ] {
            let operation = f.prepare(Mutation::Create {
                record: f.record(content),
                supersedes: None,
            });
            f.service.commit(f.args(&operation)).unwrap();
        }
        let (scope, batch) = select_service_index(&f, 1);
        assert!(batch.has_more);
        let cursor = batch.next_after_row_id.unwrap();
        let output = synthetic_index_output(&batch);
        let result = f.service.finish_index_page(scope, batch, Some(Ok(output)));
        assert_eq!(result["status"], "page_complete");
        assert_eq!(result["indexed"].as_array().unwrap().len(), 1);
        assert_eq!(result["coverage"]["current_heads"], 2);
        assert_eq!(result["coverage"]["indexed_heads"], 1);
        assert_eq!(result["next_after_row_id"], cursor);
        assert_eq!(result["stable_snapshot"], false);
        let wire = serde_json::to_string(&result).unwrap();
        assert!(!wire.contains("private source content"));
        assert!(!wire.contains("vector"));
        // Closed runtime would return pending if even this already-indexed page
        // attempted inference. It must still expose the next scan cursor.
        let already = f
            .service
            .index_refresh(index_args(&f, None, 1, 16384))
            .await
            .unwrap();
        assert_eq!(already["status"], "page_complete");
        assert_eq!(already["already_indexed_at_selection"], 1);
        assert!(already["indexed"].as_array().unwrap().is_empty());
        assert_eq!(already["next_after_row_id"], cursor);
        assert_eq!(already["has_more"], true);
        let next = f
            .service
            .index_refresh(index_args(&f, Some(cursor), 1, 16384))
            .await
            .unwrap();
        assert_eq!(next["status"], "pending");
        assert_eq!(next["scanned_heads"], 1);
        assert_eq!(next["has_more"], false);
        assert_eq!(next["coverage"]["missing_heads"], 1);
        assert!(!f.service.engine.state_root().join("jobs").exists());
    }

    #[test]
    fn index_refresh_cas_and_invalid_vector_report_truthful_partial_publication() {
        let f = Fixture::new();
        for content in [
            "Selected source changed while inference ran",
            "Unchanged source accepts derived output",
            "Invalid output cannot change this record",
        ] {
            let operation = f.prepare(Mutation::Create {
                record: f.record(content),
                supersedes: None,
            });
            f.service.commit(f.args(&operation)).unwrap();
        }
        let (scope, batch) = select_service_index(&f, 3);
        let stale = batch.items[0].reference.clone();
        let valid = batch.items[1].reference.clone();
        let invalid = batch.items[2].reference.clone();
        let mut output = synthetic_index_output(&batch);
        output.passages[2].chunks[0].vector.pop();
        let revision = f.prepare(Mutation::Revise {
            record_id: stale.record_id.clone(),
            expected_revision: stale.revision,
            record: f.record("Concurrent canonical correction survives"),
        });
        f.service.commit(f.args(&revision)).unwrap();
        let result = f
            .service
            .finish_index_page(scope.clone(), batch, Some(Ok(output)));
        assert_eq!(result["status"], "partial");
        assert_eq!(result["indexed"], json!([valid]));
        assert_eq!(result["skipped"][0]["reference"], json!(stale));
        assert_eq!(result["skipped"][0]["reason"], "head_changed");
        assert_eq!(result["failed"][0]["reference"], json!(invalid));
        assert_eq!(result["failed"][0]["reason"], "publication_unconfirmed");
        assert_eq!(result["coverage"]["indexed_heads"], 1);
        assert_eq!(result["coverage"]["missing_heads"], 2);
        assert_eq!(result["restart_required"], true);
        let mut store = MemoryStore::open_existing(f.service.engine.state_root(), scope).unwrap();
        assert_eq!(
            store.read(&stale.record_id, None).unwrap().data.content,
            "Concurrent canonical correction survives"
        );
        assert_eq!(
            store.read(&invalid.record_id, None).unwrap().data.content,
            "Invalid output cannot change this record"
        );
        assert!(
            !serde_json::to_string(&result)
                .unwrap()
                .contains("Concurrent canonical correction")
        );
    }

    #[test]
    fn index_refresh_publication_cannot_recreate_missing_canonical_store() {
        let f = Fixture::new();
        let operation = f.prepare(Mutation::Create {
            record: f.record("Do not recreate after scope loss"),
            supersedes: None,
        });
        f.service.commit(f.args(&operation)).unwrap();
        let (scope, batch) = select_service_index(&f, 1);
        let output = synthetic_index_output(&batch);
        let memory = f.service.engine.state_root().join("memory");
        fs::rename(&memory, f.root.join("preserved-index-source")).unwrap();
        let result = f.service.finish_index_page(scope, batch, Some(Ok(output)));
        assert_eq!(result["status"], "pending");
        assert_eq!(result["failed"][0]["reason"], "canonical_scope_unavailable");
        assert!(result["coverage"].is_null());
        assert!(result["coverage_warning"].is_string());
        assert!(!memory.exists());
    }
}
