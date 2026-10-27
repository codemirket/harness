//! Same-home reinstall is a new bounded intent after a proved complete uninstall.
//! It reuses release/native owners and never deletes or initializes durable data.
//! The intent blocks normal startup and the old uninstall until its exact records
//! have been archived after the new installation is verified.

use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const INTENT: &str = "reinstall.json";
const MAX_RECORD_BYTES: u64 = 32 * 1024 * 1024;
const MAX_INTENT_BYTES: u64 = 8192;
const PHASES: [&str; 7] = [
    "stage",
    "activate",
    "global_skills",
    "guidance",
    "registration",
    "verify",
    "archive",
];

#[derive(Clone, Debug, Serialize)]
pub struct ReinstallReport {
    pub status: &'static str,
    pub home: PathBuf,
    pub user_home: PathBuf,
    pub candidate_release_id: Option<String>,
    pub uninstall_plan_sha256: Option<String>,
    pub next_step: &'static str,
    pub preserved_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema_version: u8,
    home: PathBuf,
    user_home: PathBuf,
    candidate_release_id: String,
    prior_release_id: String,
    uninstall_plan_sha256: String,
    uninstall_progress_sha256: String,
    sequence: u64,
    generation: u64,
    phase: u8,
}

/// Only this owner can construct a permit, from a durably saved exact intent.
/// ReleaseStore rechecks it under its writer lock before its existing effects.
pub(crate) struct ReinstallPermit {
    home: PathBuf,
    candidate_release_id: String,
    intent_sha256: String,
}

impl ReinstallPermit {
    pub(crate) fn candidate_release_id(&self) -> &str {
        &self.candidate_release_id
    }
    pub(crate) fn verify_stage(&self, home: &Path, candidate: &str) -> Result<(), String> {
        self.verify(home, candidate, 0)
    }
    pub(crate) fn verify_activation(&self, home: &Path, candidate: &str) -> Result<(), String> {
        self.verify(home, candidate, 1)
    }
    fn verify(&self, home: &Path, candidate: &str, phase: u8) -> Result<(), String> {
        let (intent, digest) = read_intent(home)?.ok_or("reinstall intent disappeared")?;
        if self.home != home
            || self.candidate_release_id != candidate
            || intent.candidate_release_id != candidate
            || intent.phase != phase
            || digest != self.intent_sha256
            || present(&intent_path(home).with_extension("json.next"))?
        {
            return Err(
                "reinstall permit no longer matches its exact saved phase and release".into(),
            );
        }
        verify_bound_records(&intent)?;
        Ok(())
    }
}

impl Installation {
    pub fn reinstall_codex(
        &self,
        source: &Path,
        user_home: &Path,
    ) -> Result<ReinstallReport, String> {
        self.reinstall_inner(source, user_home, false, None)
    }

    pub fn reconcile_reinstall(
        &self,
        source: &Path,
        user_home: &Path,
    ) -> Result<ReinstallReport, String> {
        self.reinstall_inner(source, user_home, true, None)
    }

    /// Inspection never promotes records, creates locks or repairs native state.
    pub fn reinstall_status(&self, user_home: &Path) -> Result<ReinstallReport, String> {
        let paths = CodexPaths::resolve(user_home, &self.home)?;
        let Some((intent, _)) = read_intent(&self.home)? else {
            if let Some(active) = self.store.inspect_active()?
                && let Some(completed) = completed_report(self, &paths, &active)?
            {
                return Ok(completed);
            }
            return Ok(report(&self.home, &paths.user_home, None, false));
        };
        require_user(&intent, &paths)?;
        verify_bound_records(&intent)?;
        Ok(report(&self.home, &paths.user_home, Some(&intent), false))
    }

    fn reinstall_inner(
        &self,
        source: &Path,
        user_home: &Path,
        reconcile: bool,
        failure: Option<u8>,
    ) -> Result<ReinstallReport, String> {
        if !cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            return Err("same-home reinstall is qualified only on macOS ARM64".into());
        }
        let paths = CodexPaths::resolve(user_home, &self.home)?;
        let source = VerifiedRelease::open(source).map_err(io_message)?;
        let catalog = release_catalog(&source)?;
        let _native = native_lock(&self.home.join("state/codex-operation.lock"))?;
        let maintenance = MaintenanceGuard::try_acquire(&self.home.join("state"))
            .map_err(|error| format!("reinstall requires idle maintenance: {error}"))?;
        let mut intent = {
            let _writer = self.store.lock_uninstall(&maintenance)?;
            match read_intent(&self.home)? {
                Some((intent, _)) => {
                    require_user(&intent, &paths)?;
                    if intent.candidate_release_id != source.id().as_str() {
                        return Err(
                            "reinstall source differs from the original bound candidate; preserved"
                                .into(),
                        );
                    }
                    if !reconcile {
                        return Err("reinstall is pending; inspect reinstall-status and explicitly run reinstall-reconcile".into());
                    }
                    verify_bound_records(&intent)?;
                    settle_intent(&self.home)?;
                    intent
                }
                None => {
                    if reconcile {
                        return completed_report(self, &paths, &source)?.ok_or_else(|| {
                            "no reinstall intent or matching completed receipt to reconcile".into()
                        });
                    }
                    let completed = uninstall::completed_snapshot(&self.home, &paths.user_home)?;
                    // A completed uninstall knows only its old payload. Require
                    // the managed release/staging containers to be empty too.
                    require_empty_container(&self.home.join("releases"))?;
                    require_empty_container(&self.home.join("staging"))?;
                    let preflight = PreflightReceipts::new()?;
                    preflight_native(
                        &paths,
                        &catalog,
                        source.id().as_str(),
                        &self.home,
                        &preflight.0,
                        None,
                        &qualified_capabilities(source.manifest()),
                    )?;
                    let intent = Intent {
                        schema_version: 1,
                        home: self.home.clone(),
                        user_home: paths.user_home.clone(),
                        candidate_release_id: source.id().as_str().into(),
                        prior_release_id: completed.release_id,
                        uninstall_plan_sha256: completed.plan_sha256,
                        uninstall_progress_sha256: completed.progress_sha256,
                        sequence: next_sequence(&self.home)?,
                        generation: 0,
                        phase: 0,
                    };
                    // Raw records supplied by the owning parser are rechecked
                    // before committing the authorization for any new effect.
                    if read_record(
                        &self.home.join("state").join(uninstall::PLAN),
                        MAX_RECORD_BYTES,
                    )?
                    .as_ref()
                        != Some(&completed.plan_bytes)
                        || read_record(
                            &self.home.join("state").join(uninstall::PROGRESS),
                            MAX_RECORD_BYTES,
                        )?
                        .as_ref()
                            != Some(&completed.progress_bytes)
                    {
                        return Err(
                            "completed uninstall records changed before reinstall intent".into(),
                        );
                    }
                    save_intent(&self.home, &intent)?;
                    intent
                }
            }
        };
        maybe_fail(failure, 9)?;
        while intent.phase < 6 {
            verify_bound_records(&intent)?;
            if intent.phase >= 2 {
                verify_active(self, &intent)?;
            }
            match intent.phase {
                0 => {
                    let permit = permit(&intent)?;
                    let staged = self.store.stage_reinstall(&source, &maintenance, &permit)?;
                    if staged.status != "staged" || staged.release_id != intent.candidate_release_id
                    {
                        return Err("reinstall staging did not select its exact candidate".into());
                    }
                }
                1 => {
                    let permit = permit(&intent)?;
                    let active = self.store.activate_reinstall_held(
                        &self.launcher_path(),
                        &maintenance,
                        &permit,
                    )?;
                    if active.status != "active"
                        || active.release_id != intent.candidate_release_id
                        || active.reason.is_some()
                    {
                        return Err(
                            "reinstall activation needs reconciliation before native effects"
                                .into(),
                        );
                    }
                }
                2 => {
                    let manager = SkillSync::global(
                        &catalog,
                        &paths.user_home,
                        &self.codex_skills_receipt_path(),
                    )?;
                    let inspected = manager.inspect()?;
                    require_receipt_candidate(
                        inspected.release_id.as_deref(),
                        &intent.candidate_release_id,
                    )?;
                    let inspected = if inspected.pending_transaction {
                        manager.verify_receipt_release(&intent.candidate_release_id)?;
                        manager.reconcile()?
                    } else {
                        inspected
                    };
                    require_native_release(
                        inspected.status,
                        inspected.release_id.as_deref(),
                        &intent.candidate_release_id,
                    )?;
                    manager.sync(&intent.candidate_release_id, None, &BTreeSet::new())?;
                }
                3 => {
                    verify_skills(self, &paths, &catalog, &intent)?;
                    let owner =
                        CodexGuidance::open(&paths.guidance, &self.codex_guidance_receipt_path())?;
                    let inspected = owner.inspect()?;
                    require_receipt_candidate(
                        inspected.release_id.as_deref(),
                        &intent.candidate_release_id,
                    )?;
                    let inspected = if inspected.status == "pending" {
                        owner.verify_receipt_release(&intent.candidate_release_id)?;
                        owner.reconcile()?
                    } else {
                        inspected
                    };
                    require_native_release(
                        inspected.status,
                        inspected.release_id.as_deref(),
                        &intent.candidate_release_id,
                    )?;
                    owner.install(&intent.candidate_release_id)?;
                }
                4 => {
                    verify_skills(self, &paths, &catalog, &intent)?;
                    verify_guidance(self, &paths, &intent)?;
                    let owner = CodexConfig::open(&paths.config, &self.codex_receipt_path())?;
                    let inspected = owner.inspect()?;
                    require_receipt_candidate(
                        inspected.managed_release_id.as_deref(),
                        &intent.candidate_release_id,
                    )?;
                    if inspected.pending_transaction {
                        owner.verify_receipt_release(&intent.candidate_release_id)?;
                        owner.reconcile()?;
                    }
                    let inspected = owner.inspect()?;
                    require_native_release(
                        inspected.status,
                        inspected.managed_release_id.as_deref(),
                        &intent.candidate_release_id,
                    )?;
                    owner.register(&CodexRegistrationRequest {
                        release_id: intent.candidate_release_id.clone(),
                        home: self.home.clone(),
                        launcher: self.launcher_path(),
                    })?;
                }
                5 => verify_ready(self, &paths, &catalog, &intent)?,
                _ => unreachable!(),
            }
            maybe_fail(failure, intent.phase)?;
            intent.phase += 1;
            intent.generation += 1;
            let _writer = self.store.lock_uninstall(&maintenance)?;
            save_intent(&self.home, &intent)?;
        }
        // The old uninstall is superseded only inside this committed intent;
        // normal sessions and old-uninstall reconciliation stay blocked until
        // the final exact journal archival removes the live intent marker.
        verify_ready(self, &paths, &catalog, &intent)?;
        maybe_fail(failure, 6)?;
        let _writer = self.store.lock_uninstall(&maintenance)?;
        archive(&intent, failure)?;
        Ok(report(&self.home, &paths.user_home, Some(&intent), true))
    }
}

pub(super) fn core_status(home: &Path) -> Result<Option<&'static str>, String> {
    read_intent(home)?
        .map(|(intent, _)| {
            verify_bound_records(&intent)?;
            Ok("reinstall_pending")
        })
        .transpose()
}

pub(super) fn require_absent(home: &Path) -> Result<(), String> {
    if present(&intent_path(home))? || present(&intent_path(home).with_extension("json.next"))? {
        return Err(
            "reinstall intent supersedes the old uninstall; inspect/reconcile reinstall instead"
                .into(),
        );
    }
    Ok(())
}

fn report(home: &Path, user: &Path, intent: Option<&Intent>, complete: bool) -> ReinstallReport {
    ReinstallReport {
        status: if complete {
            "reinstalled"
        } else if intent.is_some() {
            "reinstall_pending"
        } else {
            "not_started"
        },
        home: home.into(),
        user_home: user.into(),
        candidate_release_id: intent.map(|value| value.candidate_release_id.clone()),
        uninstall_plan_sha256: intent.map(|value| value.uninstall_plan_sha256.clone()),
        next_step: if complete {
            "complete"
        } else {
            intent.map_or("explicit_reinstall", |value| {
                PHASES[usize::from(value.phase)]
            })
        },
        preserved_paths: vec![home.join("state"), home.join("backups")],
    }
}
fn require_user(intent: &Intent, paths: &CodexPaths) -> Result<(), String> {
    if intent.user_home != paths.user_home {
        return Err("reinstall belongs to another explicit user home".into());
    }
    Ok(())
}
fn require_native_release(
    status: &str,
    release: Option<&str>,
    candidate: &str,
) -> Result<(), String> {
    if status != "missing" && (status != "managed" || release != Some(candidate)) {
        return Err(format!(
            "reinstall refuses {status} native state or a different release; preserved"
        ));
    }
    Ok(())
}

fn require_receipt_candidate(release: Option<&str>, candidate: &str) -> Result<(), String> {
    if release.is_some_and(|release| release != candidate) {
        return Err("native receipt belongs to a different reinstall candidate; preserved".into());
    }
    Ok(())
}
fn verify_active(installation: &Installation, intent: &Intent) -> Result<VerifiedRelease, String> {
    let active = installation
        .store
        .inspect_active()?
        .ok_or("reinstall active candidate is missing")?;
    if active.id().as_str() != intent.candidate_release_id {
        return Err("reinstall active release changed; preserved".into());
    }
    launcher_health(&installation.home, &active)?;
    let expected = format!(
        "r-{}",
        intent
            .candidate_release_id
            .strip_prefix("sha256:")
            .ok_or("invalid candidate")?
    );
    let entries = fs::read_dir(installation.home.join("releases"))
        .map_err(io_message)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(io_message)?;
    if entries.len() != 1 || entries[0].file_name() != expected.as_str() {
        return Err("reinstall release inventory changed; preserved".into());
    }
    require_empty_container(&installation.home.join("staging"))?;
    for name in [
        "activation.pending.json",
        "activation.pending.json.next",
        "staging.pending.json",
        "staging.pending.json.next",
    ] {
        if present(&installation.home.join("state").join(name))? {
            return Err(
                "reinstall release owner still has pending work; reconcile activation first".into(),
            );
        }
    }
    if present(&installation.home.join("staged.json"))?
        || present(&installation.home.join("active.json.next"))?
        || present(&installation.home.join("staged.json.next"))?
    {
        return Err("reinstall release pointers have pending or conflicting work".into());
    }
    Ok(active)
}
fn verify_skills(
    installation: &Installation,
    paths: &CodexPaths,
    catalog: &QualifiedCatalog,
    intent: &Intent,
) -> Result<(), String> {
    let inspected = SkillSync::global(
        catalog,
        &paths.user_home,
        &installation.codex_skills_receipt_path(),
    )?
    .inspect()?;
    let expected = catalog
        .catalog
        .global
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    let actual = inspected
        .skills
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<BTreeSet<_>>();
    if inspected.status != "managed"
        || inspected.pending_transaction
        || inspected.release_id.as_deref() != Some(&intent.candidate_release_id)
        || actual != expected
    {
        return Err(
            "reinstalled global skills no longer match the selected candidate; preserved".into(),
        );
    }
    Ok(())
}
fn verify_guidance(
    installation: &Installation,
    paths: &CodexPaths,
    intent: &Intent,
) -> Result<(), String> {
    let inspected =
        CodexGuidance::open(&paths.guidance, &installation.codex_guidance_receipt_path())?
            .inspect()?;
    if inspected.status != "managed"
        || inspected.release_id.as_deref() != Some(&intent.candidate_release_id)
    {
        return Err("reinstalled guidance changed; preserved".into());
    }
    Ok(())
}
fn verify_ready(
    installation: &Installation,
    paths: &CodexPaths,
    catalog: &QualifiedCatalog,
    intent: &Intent,
) -> Result<(), String> {
    verify_bound_records(intent)?;
    verify_active(installation, intent)?;
    verify_skills(installation, paths, catalog, intent)?;
    verify_guidance(installation, paths, intent)?;
    let inspected =
        CodexConfig::open(&paths.config, &installation.codex_receipt_path())?.inspect()?;
    if inspected.status != "managed"
        || inspected.pending_transaction
        || inspected.managed_release_id.as_deref() != Some(&intent.candidate_release_id)
    {
        return Err("reinstalled registration changed; preserved".into());
    }
    verify_registration_target(&paths.config, installation)
}

/// A response may be lost after the final live-marker rename. Reconcile that
/// committed effect from its exact archived receipt and actual current health,
/// without creating another intent or performing any installation effects.
fn completed_report(
    installation: &Installation,
    paths: &CodexPaths,
    candidate: &VerifiedRelease,
) -> Result<Option<ReinstallReport>, String> {
    let intent = history_intents(&installation.home)?
        .into_iter()
        .filter(|intent| {
            intent.user_home == paths.user_home
                && intent.candidate_release_id == candidate.id().as_str()
        })
        .max_by_key(|intent| intent.sequence);
    let Some(intent) = intent else {
        return Ok(None);
    };
    verify_ready(installation, paths, &release_catalog(candidate)?, &intent)?;
    Ok(Some(report(
        &installation.home,
        &paths.user_home,
        Some(&intent),
        true,
    )))
}

fn history_intents(home: &Path) -> Result<Vec<Intent>, String> {
    let history = home.join("state/reinstall-history");
    if !present(&history)? {
        return Ok(Vec::new());
    }
    validate_native_path(&history)?;
    let mut intents = Vec::new();
    let mut sequences = BTreeSet::new();
    let mut count = 0;
    for entry in fs::read_dir(&history).map_err(io_message)? {
        count += 1;
        if count > 32 {
            return Err("reinstall history exceeds its inspection bound; preserved".into());
        }
        let entry = entry.map_err(io_message)?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "invalid reinstall history name")?;
        if !valid_hash(&name) {
            return Err("unknown reinstall history entry; preserved".into());
        }
        let bytes = read_record(&entry.path().join(INTENT), MAX_INTENT_BYTES)?
            .ok_or("incomplete reinstall history exists without a live intent; preserved")?;
        let intent: Intent = serde_json::from_slice(&bytes).map_err(io_message)?;
        validate_intent(&intent, home)?;
        if intent.phase != 6
            || intent.uninstall_plan_sha256 != name
            || !sequences.insert(intent.sequence)
        {
            return Err("invalid or duplicate committed reinstall receipt; preserved".into());
        }
        intents.push(intent);
    }
    Ok(intents)
}

fn next_sequence(home: &Path) -> Result<u64, String> {
    let history = history_intents(home)?;
    if history.len() >= 32 {
        return Err("reinstall history capacity is full; retained receipts preserved".into());
    }
    history
        .iter()
        .map(|value| value.sequence)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| "reinstall history sequence overflow".into())
}

fn intent_path(home: &Path) -> PathBuf {
    home.join("state").join(INTENT)
}
fn archive_root(intent: &Intent) -> PathBuf {
    intent
        .home
        .join("state/reinstall-history")
        .join(&intent.uninstall_plan_sha256)
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn valid_release(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(valid_hash)
}
fn present(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io_message(error)),
    }
}
fn read_record(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, String> {
    if !present(path)? {
        return Ok(None);
    }
    validate_native_path(path)?;
    let metadata = fs::symlink_metadata(path).map_err(io_message)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err("reinstall record must be bounded and regular".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.permissions().mode() & 0o7777 != 0o600
            || metadata.nlink() != 1
            || metadata.uid() != unsafe { libc::geteuid() }
        {
            return Err("reinstall record must be private, owned and unaliased".into());
        }
    }
    let mut file = File::open(path).map_err(io_message)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(io_message)?;
    let after = fs::symlink_metadata(path).map_err(io_message)?;
    if bytes.len() as u64 != metadata.len() {
        return Err("reinstall record changed during read".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if (
            metadata.dev(),
            metadata.ino(),
            metadata.mtime(),
            metadata.mtime_nsec(),
            metadata.ctime(),
            metadata.ctime_nsec(),
        ) != (
            after.dev(),
            after.ino(),
            after.mtime(),
            after.mtime_nsec(),
            after.ctime(),
            after.ctime_nsec(),
        ) {
            return Err("reinstall record changed during read".into());
        }
    }
    #[cfg(not(unix))]
    let _ = after;
    Ok(Some(bytes))
}
fn validate_intent(intent: &Intent, home: &Path) -> Result<(), String> {
    if intent.schema_version != 1
        || intent.home != home
        || intent.phase > 6
        || intent.sequence == 0
        || intent.generation != u64::from(intent.phase)
        || !valid_release(&intent.candidate_release_id)
        || !valid_release(&intent.prior_release_id)
        || !valid_hash(&intent.uninstall_plan_sha256)
        || !valid_hash(&intent.uninstall_progress_sha256)
    {
        return Err("invalid reinstall intent; preserved".into());
    }
    CodexPaths::resolve(&intent.user_home, home)?;
    Ok(())
}
fn same_binding(left: &Intent, right: &Intent) -> bool {
    let mut normalized = left.clone();
    normalized.phase = right.phase;
    normalized.generation = right.generation;
    normalized == *right
}
fn read_intent(home: &Path) -> Result<Option<(Intent, String)>, String> {
    let path = intent_path(home);
    let parse = |bytes: Vec<u8>| -> Result<(Intent, String), String> {
        let intent = serde_json::from_slice::<Intent>(&bytes).map_err(io_message)?;
        validate_intent(&intent, home)?;
        Ok((intent, digest(&bytes)))
    };
    let current = read_record(&path, MAX_INTENT_BYTES)?
        .map(parse)
        .transpose()?;
    let next = read_record(&path.with_extension("json.next"), MAX_INTENT_BYTES)?
        .map(parse)
        .transpose()?;
    if let Some((next_intent, _)) = &next {
        if let Some((current_intent, _)) = &current {
            if next_intent != current_intent
                && !(same_binding(current_intent, next_intent)
                    && current_intent.phase.checked_add(1) == Some(next_intent.phase))
            {
                return Err("reinstall temporary record is not its exact known successor".into());
            }
        } else if next_intent.phase != 0 {
            return Err("reinstall temporary record has no predecessor".into());
        }
    }
    Ok(next.or(current))
}
fn save_intent(home: &Path, intent: &Intent) -> Result<(), String> {
    validate_intent(intent, home)?;
    let bytes = serde_json::to_vec(intent).map_err(io_message)?;
    if bytes.len() as u64 > MAX_INTENT_BYTES {
        return Err("reinstall intent exceeds its bound".into());
    }
    let path = intent_path(home);
    let next = path.with_extension("json.next");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file = options.open(&next).map_err(io_message)?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(io_message)?;
    replace_native_path(&next, &path)
}
fn settle_intent(home: &Path) -> Result<(), String> {
    let path = intent_path(home);
    let next = path.with_extension("json.next");
    if present(&next)? {
        replace_native_path(&next, &path)?;
    }
    Ok(())
}
fn permit(intent: &Intent) -> Result<ReinstallPermit, String> {
    let (_, intent_sha256) =
        read_intent(&intent.home)?.ok_or("saved reinstall intent disappeared")?;
    Ok(ReinstallPermit {
        home: intent.home.clone(),
        candidate_release_id: intent.candidate_release_id.clone(),
        intent_sha256,
    })
}
fn require_empty_container(path: &Path) -> Result<(), String> {
    validate_native_path(path)?;
    if fs::read_dir(path).map_err(io_message)?.next().is_some() {
        return Err(
            "reinstall requires an empty owned payload container; unknown files preserved".into(),
        );
    }
    Ok(())
}
fn verify_bound_records(intent: &Intent) -> Result<(), String> {
    for (name, expected) in [
        (uninstall::PLAN, &intent.uninstall_plan_sha256),
        (uninstall::PROGRESS, &intent.uninstall_progress_sha256),
    ] {
        let live = read_record(&intent.home.join("state").join(name), MAX_RECORD_BYTES)?;
        let archived = read_record(&archive_root(intent).join(name), MAX_RECORD_BYTES)?;
        match (live, archived) {
            (Some(bytes), None) if digest(&bytes) == *expected => {}
            (None, Some(bytes)) if intent.phase == 6 && digest(&bytes) == *expected => {}
            _ => return Err("original completed uninstall record changed or is ambiguously duplicated; preserved".into()),
        }
        if present(
            &intent
                .home
                .join("state")
                .join(name)
                .with_extension("json.next"),
        )? {
            return Err("original uninstall has an unexpected temporary record; preserved".into());
        }
    }
    Ok(())
}
fn private_directory(path: &Path) -> Result<(), String> {
    validate_native_path(path)?;
    if !present(path)? {
        create_private_directory(path)?;
        sync_parent(path)?;
    }
    let metadata = fs::symlink_metadata(path).map_err(io_message)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("reinstall history must be a real private directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.permissions().mode() & 0o7777 != 0o700
            || metadata.uid() != unsafe { libc::geteuid() }
        {
            return Err("reinstall history must be private and owned".into());
        }
    }
    Ok(())
}
fn sync_parent(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    File::open(path.parent().ok_or("reinstall path has no parent")?)
        .and_then(|file| file.sync_all())
        .map_err(io_message)?;
    Ok(())
}
fn archive(intent: &Intent, failure: Option<u8>) -> Result<(), String> {
    verify_bound_records(intent)?;
    let root = archive_root(intent);
    private_directory(root.parent().ok_or("history has no parent")?)?;
    private_directory(&root)?;
    for entry in fs::read_dir(&root).map_err(io_message)? {
        let name = entry.map_err(io_message)?.file_name();
        if ![uninstall::PLAN, uninstall::PROGRESS]
            .iter()
            .any(|expected| name == *expected)
        {
            return Err("unknown reinstall history entry; preserved".into());
        }
    }
    for (index, name) in [uninstall::PLAN, uninstall::PROGRESS]
        .into_iter()
        .enumerate()
    {
        let live = intent.home.join("state").join(name);
        let destination = root.join(name);
        if present(&live)? {
            if present(&destination)? {
                return Err("reinstall archive destination appeared; preserved".into());
            }
            fs::rename(&live, &destination).map_err(io_message)?;
            sync_parent(&live)?;
            sync_parent(&destination)?;
        }
        maybe_fail(failure, 7 + index as u8)?;
    }
    verify_bound_records(intent)?;
    // This final rename is the commit point for admitting ordinary startup.
    // Before it, the live intent blocks all ordinary and old-uninstall paths.
    fs::rename(intent_path(&intent.home), root.join(INTENT)).map_err(io_message)?;
    sync_parent(&intent_path(&intent.home))?;
    sync_parent(&root.join(INTENT))?;
    maybe_fail(failure, 10)
}
fn maybe_fail(selected: Option<u8>, point: u8) -> Result<(), String> {
    if selected == Some(point) {
        Err("injected reinstall interruption; inspect and reconcile the original intent".into())
    } else {
        Ok(())
    }
}

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
mod tests {
    use super::*;
    use crate::installation::tests::{Fixture, codex_payload, codex_user};

    fn uninstalled() -> (Fixture, Installation, PathBuf, PathBuf) {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "old core");
        let user = codex_user(&fixture, true);
        let home = fixture.0.join("home");
        Installation::setup_codex(&home, &source, &user).unwrap();
        let installation = Installation::open(&home).unwrap();
        installation.uninstall_codex(&user).unwrap();
        let source = codex_payload(&fixture, "new core");
        (fixture, installation, user, source)
    }

    #[test]
    fn reinstall_preserves_canonical_data_backups_and_unrelated_native_bytes() {
        let (_fixture, installation, user, source) = uninstalled();
        let session =
            crate::maintenance::SessionGuard::acquire(&installation.home.join("state")).unwrap();
        assert!(
            installation
                .reinstall_codex(&source, &user)
                .unwrap_err()
                .contains("idle maintenance")
        );
        assert!(!intent_path(&installation.home).exists());
        drop(session);
        let database = installation.home.join("state/canonical-memory.sqlite");
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection.execute_batch("CREATE TABLE facts(value TEXT); INSERT INTO facts VALUES ('preserved correction');").unwrap();
        drop(connection);
        private_directory(&installation.home.join("backups")).unwrap();
        fs::write(
            installation.home.join("backups/protected"),
            b"protected recovery bytes",
        )
        .unwrap();
        fs::write(installation.home.join("notes.txt"), b"user note").unwrap();
        let config = fs::read_to_string(user.join(".codex/config.toml")).unwrap();
        let result = installation.reinstall_codex(&source, &user).unwrap();
        assert_eq!(result.status, "reinstalled");
        assert_eq!(
            Installation::doctor_codex(&installation.home, &user).status,
            "healthy"
        );
        let connection = rusqlite::Connection::open(&database).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT value FROM facts", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "preserved correction"
        );
        assert_eq!(
            fs::read(installation.home.join("backups/protected")).unwrap(),
            b"protected recovery bytes"
        );
        assert_eq!(
            fs::read(installation.home.join("notes.txt")).unwrap(),
            b"user note"
        );
        assert!(
            fs::read_to_string(user.join(".codex/config.toml"))
                .unwrap()
                .starts_with(&config)
        );
        assert!(!intent_path(&installation.home).exists());
        assert!(
            !installation
                .home
                .join("state")
                .join(uninstall::PLAN)
                .exists()
        );
        assert!(installation.store.open_active().is_ok());
    }

    #[test]
    fn lost_effect_responses_and_partial_archival_resume_the_exact_intent() {
        for point in [9, 0, 1, 2, 4, 6, 7, 8] {
            let (_fixture, installation, user, source) = uninstalled();
            installation
                .reinstall_inner(&source, &user, false, Some(point))
                .unwrap_err();
            if point == 9 {
                let path = intent_path(&installation.home);
                fs::rename(&path, path.with_extension("json.next")).unwrap();
            }
            let (intent, _) = read_intent(&installation.home).unwrap().unwrap();
            assert_eq!(
                installation.reinstall_status(&user).unwrap().status,
                "reinstall_pending"
            );
            assert!(installation.store.bind_active().is_err());
            assert!(installation.stage(&source).is_err());
            assert!(installation.lock_codex_operation().is_err());
            assert_eq!(
                Installation::doctor_codex(&installation.home, &user).status,
                "reinstall_pending"
            );
            assert!(installation.uninstall_codex(&user).is_err());
            assert!(installation.reinstall_codex(&source, &user).is_err());
            let result = installation.reconcile_reinstall(&source, &user).unwrap();
            assert_eq!(result.status, "reinstalled", "point {point}");
            assert_eq!(
                Installation::doctor_codex(&installation.home, &user).status,
                "healthy"
            );
            let history = archive_root(&intent);
            assert_eq!(
                digest(&fs::read(history.join(uninstall::PLAN)).unwrap()),
                intent.uninstall_plan_sha256
            );
            assert_eq!(
                digest(&fs::read(history.join(uninstall::PROGRESS)).unwrap()),
                intent.uninstall_progress_sha256
            );
            assert!(history.join(INTENT).is_file());
        }
    }

    #[test]
    fn pending_uninstall_wrong_candidate_and_native_drift_refuse_without_adoption() {
        let (fixture, installation, user, source) = uninstalled();
        let progress = installation.home.join("state").join(uninstall::PROGRESS);
        let before = fs::read(&progress).unwrap();
        let mut parsed: serde_json::Value = serde_json::from_slice(&before).unwrap();
        parsed["step"] = 3.into();
        fs::write(&progress, serde_json::to_vec(&parsed).unwrap()).unwrap();
        assert!(installation.reinstall_codex(&source, &user).is_err());
        assert!(!intent_path(&installation.home).exists());
        fs::write(&progress, before).unwrap();
        installation
            .reinstall_inner(&source, &user, false, Some(2))
            .unwrap_err();
        let intent_before = fs::read(intent_path(&installation.home)).unwrap();
        let other_user = fixture.0.join("other-user");
        create_private_directory(&other_user).unwrap();
        assert!(
            installation
                .reconcile_reinstall(&source, &other_user)
                .unwrap_err()
                .contains("another explicit user home")
        );
        let source = codex_payload(&fixture, "different candidate core");
        assert!(
            installation
                .reconcile_reinstall(&source, &user)
                .unwrap_err()
                .contains("original bound candidate")
        );
        assert_eq!(
            fs::read(intent_path(&installation.home)).unwrap(),
            intent_before
        );
        let source = codex_payload(&fixture, "new core");
        let edited = user.join(".agents/skills/code-change/SKILL.md");
        fs::write(&edited, b"user changed skill").unwrap();
        assert!(installation.reconcile_reinstall(&source, &user).is_err());
        assert_eq!(fs::read(edited).unwrap(), b"user changed skill");
        assert_eq!(
            fs::read(intent_path(&installation.home)).unwrap(),
            intent_before
        );
    }

    #[test]
    fn changed_uninstall_record_and_unknown_history_refuse_without_clearing_intent() {
        for history in [false, true] {
            let (_fixture, installation, user, source) = uninstalled();
            installation
                .reinstall_inner(&source, &user, false, Some(6))
                .unwrap_err();
            let (intent, _) = read_intent(&installation.home).unwrap().unwrap();
            let path = if history {
                let root = archive_root(&intent);
                private_directory(root.parent().unwrap()).unwrap();
                private_directory(&root).unwrap();
                root.join("unknown")
            } else {
                installation.home.join("state").join(uninstall::PLAN)
            };
            fs::write(&path, b"preserve unknown bytes").unwrap();
            assert!(installation.reconcile_reinstall(&source, &user).is_err());
            assert_eq!(fs::read(&path).unwrap(), b"preserve unknown bytes");
            assert!(intent_path(&installation.home).exists());
        }
    }

    #[test]
    fn final_commit_lost_response_reconciles_from_archive_without_new_effects() {
        let (_fixture, installation, user, source) = uninstalled();
        installation
            .reinstall_inner(&source, &user, false, Some(10))
            .unwrap_err();
        assert!(!intent_path(&installation.home).exists());
        assert_eq!(
            installation.reinstall_status(&user).unwrap().status,
            "reinstalled"
        );
        let before = NativeSnapshot::read(
            &installation,
            &CodexPaths::resolve(&user, &installation.home).unwrap(),
        )
        .unwrap();
        assert_eq!(
            installation
                .reconcile_reinstall(&source, &user)
                .unwrap()
                .status,
            "reinstalled"
        );
        assert!(
            NativeSnapshot::read(
                &installation,
                &CodexPaths::resolve(&user, &installation.home).unwrap()
            )
            .unwrap()
                == before
        );
        assert!(!intent_path(&installation.home).exists());
    }

    #[test]
    fn repeated_same_candidate_reinstalls_keep_distinct_history_and_latest_receipt() {
        let (_fixture, installation, user, source) = uninstalled();
        installation.reinstall_codex(&source, &user).unwrap();
        installation.uninstall_codex(&user).unwrap();
        let second = installation.reinstall_codex(&source, &user).unwrap();
        let history = history_intents(&installation.home).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(
            history
                .iter()
                .map(|intent| intent.sequence)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([1, 2])
        );
        let status = installation.reinstall_status(&user).unwrap();
        assert_eq!(status.status, "reinstalled");
        assert_eq!(status.uninstall_plan_sha256, second.uninstall_plan_sha256);
        assert_eq!(
            installation
                .reconcile_reinstall(&source, &user)
                .unwrap()
                .uninstall_plan_sha256,
            second.uninstall_plan_sha256
        );
    }

    #[test]
    fn foreign_pending_native_receipts_are_refused_before_their_reconciliation() {
        let (_fixture, installation, user, source) = uninstalled();
        let foreign_release = format!("sha256:{}", "f".repeat(64));
        for (phase, rollback) in (2..5).flat_map(|phase| [false, true].map(|old| (phase, old))) {
            installation
                .reinstall_inner(&source, &user, phase != 2 || rollback, Some(phase))
                .unwrap_err();
            let path = match phase {
                2 => installation.codex_skills_receipt_path(),
                3 => installation.codex_guidance_receipt_path(),
                _ => installation.codex_receipt_path(),
            };
            let original = fs::read(&path).unwrap();
            let mut receipt: serde_json::Value = serde_json::from_slice(&original).unwrap();
            if phase == 2 {
                let mut after = receipt["receipt"].clone();
                let before = if rollback {
                    let mut before = after.clone();
                    before["release_id"] = foreign_release.clone().into();
                    after["skills"] = serde_json::json!({});
                    before
                } else {
                    after["release_id"] = foreign_release.clone().into();
                    serde_json::Value::Null
                };
                receipt = serde_json::json!({"state":"pending", "before":before, "after":after});
            } else {
                let mut previous = receipt.clone();
                previous["release_id"] = foreign_release.clone().into();
                receipt["state"] = "pending".into();
                if !rollback {
                    receipt["release_id"] = foreign_release.clone().into();
                }
                if phase == 3 {
                    let current = digest(&fs::read(user.join(".codex/AGENTS.md")).unwrap());
                    receipt["before_present"] = true.into();
                    receipt["before_sha256"] = current.clone().into();
                    receipt["after_sha256"] =
                        if rollback { "0".repeat(64) } else { current }.into();
                    receipt["previous"] = if rollback {
                        previous
                    } else {
                        serde_json::Value::Null
                    };
                } else if rollback {
                    receipt["previous_present"] = true.into();
                    receipt["previous_config_digest"] = format!(
                        "sha256:{}",
                        digest(&fs::read(user.join(".codex/config.toml")).unwrap())
                    )
                    .into();
                    receipt["desired_config_digest"] = format!("sha256:{}", "0".repeat(64)).into();
                    receipt["prior"] = previous;
                }
            }
            let injected = serde_json::to_vec(&receipt).unwrap();
            fs::write(&path, &injected).unwrap();
            assert!(installation.reconcile_reinstall(&source, &user).is_err());
            assert!(
                fs::read(&path).unwrap() == injected,
                "phase {phase}, rollback {rollback} must not reconcile foreign native state"
            );
            fs::write(&path, original).unwrap();
        }
        assert_eq!(
            installation
                .reconcile_reinstall(&source, &user)
                .unwrap()
                .status,
            "reinstalled"
        );
    }
}
