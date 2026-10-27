//! Explicit uninstall of verified payloads and receipt-owned native integration.
//! The immutable plan and small progress record survive partial effects. State,
//! backups, project skill selections and unrecognized home files are preserved.
//! These checks are ownership/recovery controls, not a hostile-filesystem sandbox.

use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub(super) const PLAN: &str = "uninstall.json";
pub(super) const PROGRESS: &str = "uninstall.progress.json";
const MAX_PLAN_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ENTRIES: usize = 65_536;

#[derive(Clone, Debug, Serialize)]
pub struct UninstallReport {
    pub status: &'static str,
    pub home: PathBuf,
    pub user_home: PathBuf,
    pub release_id: Option<String>,
    pub next_step: &'static str,
    pub removed_payload_entries: usize,
    pub total_payload_entries: usize,
    pub preserved_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct CompletedUninstallSnapshot {
    pub(super) release_id: String,
    pub(super) plan_sha256: String,
    pub(super) progress_sha256: String,
    pub(super) plan_bytes: Vec<u8>,
    pub(super) progress_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    device: u64,
    inode: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum EntryKind {
    File {
        bytes: u64,
        sha256: String,
        mode: u32,
    },
    Link {
        target: PathBuf,
    },
    Directory,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    identity: Identity,
    content: EntryKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema_version: u8,
    home: PathBuf,
    user_home: PathBuf,
    release_id: String,
    release_ids: BTreeSet<String>,
    global_skills: Vec<String>,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Progress {
    schema_version: u8,
    plan_sha256: String,
    generation: u64,
    // 0 registration, 1 guidance, 2 skills, 3 payload, 4 complete.
    step: u8,
    index: usize,
}

impl Installation {
    /// Explicitly remove this installation's integration and verified payloads.
    /// Durable data is never an uninstall input or deletion destination.
    pub fn uninstall_codex(&self, user_home: &Path) -> Result<UninstallReport, String> {
        self.uninstall_inner(user_home, false, None)
    }

    /// Reconcile the original plan; never create a replacement operation or
    /// infer ownership from a missing receipt/path alone.
    pub fn reconcile_uninstall(&self, user_home: &Path) -> Result<UninstallReport, String> {
        self.uninstall_inner(user_home, true, None)
    }

    /// Read-only inspection, including incomplete atomic-record writes.
    pub fn uninstall_status(&self, user_home: &Path) -> Result<UninstallReport, String> {
        let paths = CodexPaths::resolve(user_home, &self.home)?;
        let Some((plan, digest)) = read_plan(&self.home)? else {
            return Ok(report(&self.home, &paths.user_home, None, None));
        };
        require_user(&plan, &paths)?;
        let progress = read_progress(&self.home, &digest, plan.entries.len())?;
        verify_completed_native(self, &paths, &plan, progress.step)?;
        if progress.step == 4 {
            completed_snapshot(&self.home, &paths.user_home)?;
        }
        Ok(report(
            &self.home,
            &paths.user_home,
            Some(&plan),
            Some(&progress),
        ))
    }

    fn uninstall_inner(
        &self,
        user_home: &Path,
        reconcile: bool,
        fail_after: Option<usize>,
    ) -> Result<UninstallReport, String> {
        super::reinstall::require_absent(&self.home)?;
        if !cfg!(target_os = "macos") {
            return Err("uninstall filesystem ownership is qualified only on macOS".into());
        }
        let paths = CodexPaths::resolve(user_home, &self.home)?;
        let _native = native_lock(&self.home.join("state/codex-operation.lock"))?;
        // A reinstall can persist its intent between the early admission check
        // and this lock. Recheck under the same exclusion as its intent writer.
        super::reinstall::require_absent(&self.home)?;
        let maintenance = MaintenanceGuard::try_acquire(&self.home.join("state"))
            .map_err(|error| format!("uninstall requires idle maintenance: {error}"))?;
        let _writer = self.store.lock_uninstall(&maintenance)?;
        let existing = read_plan(&self.home)?;
        let (plan, digest) = match existing {
            Some((plan, digest)) => {
                require_user(&plan, &paths)?;
                let progress = read_progress(&self.home, &digest, plan.entries.len())?;
                if !reconcile && progress.step != 4 {
                    return Err("uninstall is pending; inspect uninstall-status and explicitly run uninstall-reconcile".into());
                }
                (plan, digest)
            }
            None if reconcile => return Err("no uninstall plan to reconcile".into()),
            None => {
                let releases = self.store.uninstall_releases()?;
                let active = self
                    .store
                    .inspect_active()?
                    .ok_or("uninstall has no active release")?;
                launcher_health(&self.home, &active)?;
                let catalog = release_catalog(&active)?;
                preflight_native_removal(self, &paths, &catalog, active.id().as_str())?;
                let plan = removal_plan(self, &paths, &active, &releases, &catalog)?;
                let bytes = serde_json::to_vec(&plan).map_err(io_message)?;
                if bytes.len() as u64 > MAX_PLAN_BYTES {
                    return Err("uninstall inventory exceeds its bound; preserved".into());
                }
                create_record(&self.home.join("state").join(PLAN), &bytes)?;
                (plan, hash(&bytes))
            }
        };
        settle_record(&self.home.join("state").join(PLAN))?;
        let mut progress = read_progress(&self.home, &digest, plan.entries.len())?;
        settle_record(&self.home.join("state").join(PROGRESS))?;
        verify_completed_native(self, &paths, &plan, progress.step)?;
        if progress.step == 4 {
            verify_removed_prefix(&self.home, &plan, plan.entries.len())?;
            return Ok(report(
                &self.home,
                &paths.user_home,
                Some(&plan),
                Some(&progress),
            ));
        }
        // Recheck the entire outstanding payload before the next native effect.
        // Only the single armed deletion may be absent after a lost response.
        verify_payload_progress(&self.home, &plan, &progress)?;
        let catalog = if progress.step <= 2 {
            let active = self
                .store
                .inspect_active()?
                .ok_or("pending uninstall lost its active release")?;
            if active.id().as_str() != plan.release_id {
                return Err("pending uninstall active release changed; preserved".into());
            }
            Some(release_catalog(&active)?)
        } else {
            None
        };
        while progress.step < 3 {
            match progress.step {
                0 => remove_registration(self, &paths, &plan.release_id)?,
                1 => remove_guidance(self, &paths, &plan.release_id)?,
                2 => {
                    let manager = SkillSync::global(
                        catalog.as_ref().ok_or("uninstall catalog unavailable")?,
                        &paths.user_home,
                        &self.codex_skills_receipt_path(),
                    )?;
                    let mut inspected = manager.inspect()?;
                    require_receipt_release(inspected.release_id.as_deref(), &plan.release_id)?;
                    if inspected.pending_transaction {
                        manager.verify_receipt_release(&plan.release_id)?;
                        inspected = manager.reconcile()?;
                    }
                    require_skills(&inspected, &plan.release_id)?;
                    manager.remove_managed()?;
                    let removed = manager.finish_removal()?;
                    if removed.status != "missing" {
                        return Err("global skill removal did not finish; preserved".into());
                    }
                }
                _ => unreachable!(),
            }
            // Failure after the effect and before its progress commit exercises
            // the same uncertain boundary as process loss.
            maybe_fail(fail_after, usize::from(progress.step))?;
            progress.step += 1;
            advance(&self.home, &mut progress)?;
        }
        while progress.index < plan.entries.len() {
            let entry = &plan.entries[progress.index];
            let path = self.home.join(&entry.path);
            if present(&path)? {
                verify_entry(&self.home, entry)?;
                match entry.content {
                    EntryKind::Directory => fs::remove_dir(&path),
                    _ => fs::remove_file(&path),
                }
                .map_err(|error| format!("uninstall retained {}: {error}", path.display()))?;
                sync_parent(&path)?;
            }
            maybe_fail(fail_after, 3 + progress.index)?;
            progress.index += 1;
            advance(&self.home, &mut progress)?;
        }
        verify_removed_prefix(&self.home, &plan, plan.entries.len())?;
        verify_completed_native(self, &paths, &plan, 3)?;
        progress.step = 4;
        advance(&self.home, &mut progress)?;
        Ok(report(
            &self.home,
            &paths.user_home,
            Some(&plan),
            Some(&progress),
        ))
    }
}

/// Read-only binding for an explicit subsequent lifecycle owner. Callers retain
/// installation/native and maintenance exclusion while using this snapshot.
/// No marker is removed, renamed or implicitly made eligible for activation.
pub(super) fn completed_snapshot(
    home: &Path,
    user_home: &Path,
) -> Result<CompletedUninstallSnapshot, String> {
    let paths = CodexPaths::resolve(user_home, home)?;
    let installation = Installation::open(home)?;
    let (plan, digest) = read_plan(home)?.ok_or("completed uninstall is missing")?;
    require_user(&plan, &paths)?;
    let progress = read_progress(home, &digest, plan.entries.len())?;
    if progress.step != 4 {
        return Err("uninstall is not complete; reconcile the original operation".into());
    }
    verify_removed_prefix(home, &plan, plan.entries.len())?;
    verify_completed_native(&installation, &paths, &plan, 4)?;
    let state = home.join("state");
    for name in [PLAN, PROGRESS] {
        if present(&state.join(name).with_extension("json.next"))? {
            return Err(
                "uninstall has a pending record write; explicitly reconcile it first".into(),
            );
        }
    }
    let plan_bytes =
        record_bytes(&state.join(PLAN), MAX_PLAN_BYTES)?.ok_or("uninstall plan disappeared")?;
    let progress_bytes =
        record_bytes(&state.join(PROGRESS), 4096)?.ok_or("uninstall progress disappeared")?;
    if hash(&plan_bytes) != digest
        || serde_json::from_slice::<Progress>(&progress_bytes).map_err(io_message)? != progress
    {
        return Err("completed uninstall changed during snapshot; preserved".into());
    }
    Ok(CompletedUninstallSnapshot {
        release_id: plan.release_id,
        plan_sha256: digest,
        progress_sha256: hash(&progress_bytes),
        plan_bytes,
        progress_bytes,
    })
}

pub(super) fn core_status(home: &Path) -> Result<Option<&'static str>, String> {
    read_plan(home)?
        .map(|(plan, digest)| {
            let progress = read_progress(home, &digest, plan.entries.len())?;
            if progress.step == 4 {
                verify_removed_prefix(home, &plan, plan.entries.len())?;
                let installation = Installation::open(home)?;
                let paths = CodexPaths::resolve(&plan.user_home, home)?;
                verify_completed_native(&installation, &paths, &plan, 4)?;
                Ok("uninstalled")
            } else {
                Ok("uninstall_pending")
            }
        })
        .transpose()
}

fn report(
    home: &Path,
    user: &Path,
    plan: Option<&Plan>,
    progress: Option<&Progress>,
) -> UninstallReport {
    let step = progress.map_or(0, |value| value.step);
    UninstallReport {
        status: if plan.is_none() {
            "not_started"
        } else if step == 4 {
            "uninstalled"
        } else {
            "uninstall_pending"
        },
        home: home.to_owned(),
        user_home: user.to_owned(),
        release_id: plan.map(|value| value.release_id.clone()),
        next_step: if plan.is_none() {
            "explicit_uninstall"
        } else {
            [
                "registration",
                "guidance",
                "global_skills",
                "payload",
                "complete",
            ][usize::from(step)]
        },
        removed_payload_entries: progress.map_or(0, |value| value.index),
        total_payload_entries: plan.map_or(0, |value| value.entries.len()),
        preserved_paths: vec![home.join("state"), home.join("backups")],
    }
}

fn preflight_native_removal(
    installation: &Installation,
    paths: &CodexPaths,
    catalog: &QualifiedCatalog,
    id: &str,
) -> Result<(), String> {
    let registration =
        CodexConfig::open(&paths.config, &installation.codex_receipt_path())?.inspect()?;
    require_registration(installation, paths, &registration, id)?;
    let guidance =
        CodexGuidance::open(&paths.guidance, &installation.codex_guidance_receipt_path())?
            .inspect()?;
    require_guidance(&guidance, id)?;
    require_skills(
        &SkillSync::global(
            catalog,
            &paths.user_home,
            &installation.codex_skills_receipt_path(),
        )?
        .inspect()?,
        id,
    )
}

fn require_registration(
    installation: &Installation,
    paths: &CodexPaths,
    value: &CodexConfigReport,
    id: &str,
) -> Result<(), String> {
    require_receipt_release(value.managed_release_id.as_deref(), id)?;
    if value.pending_transaction || !matches!(value.status, "managed" | "missing") {
        return Err(format!(
            "uninstall refuses {} Codex registration; reconcile its owner first",
            value.status
        ));
    }
    if value.status == "managed" {
        if value.managed_release_id.as_deref() != Some(id) {
            return Err("Codex receipt belongs to another release".into());
        }
        verify_registration_target(&paths.config, installation)?;
    }
    Ok(())
}
fn require_guidance(value: &GuidanceReport, id: &str) -> Result<(), String> {
    require_receipt_release(value.release_id.as_deref(), id)?;
    if !matches!(value.status, "managed" | "missing")
        || value.status == "managed" && value.release_id.as_deref() != Some(id)
    {
        return Err(format!(
            "uninstall refuses {} guidance or mismatched release",
            value.status
        ));
    }
    Ok(())
}
fn require_skills(value: &SkillInspection, id: &str) -> Result<(), String> {
    require_receipt_release(value.release_id.as_deref(), id)?;
    if value.pending_transaction || !matches!(value.status, "managed" | "missing") {
        return Err(format!(
            "uninstall refuses {} global skills or mismatched release",
            value.status
        ));
    }
    Ok(())
}
fn require_receipt_release(release: Option<&str>, id: &str) -> Result<(), String> {
    if release.is_some_and(|release| release != id) {
        return Err("native receipt belongs to another uninstall release; preserved".into());
    }
    Ok(())
}
fn remove_registration(
    installation: &Installation,
    paths: &CodexPaths,
    id: &str,
) -> Result<(), String> {
    let owner = CodexConfig::open(&paths.config, &installation.codex_receipt_path())?;
    let mut inspected = owner.inspect()?;
    require_receipt_release(inspected.managed_release_id.as_deref(), id)?;
    if inspected.pending_transaction {
        owner.verify_receipt_release(id)?;
        owner.reconcile()?;
        inspected = owner.inspect()?;
    }
    require_registration(installation, paths, &inspected, id)?;
    if inspected.status == "managed" {
        owner.remove()?;
    }
    Ok(())
}
fn remove_guidance(
    installation: &Installation,
    paths: &CodexPaths,
    id: &str,
) -> Result<(), String> {
    let owner = CodexGuidance::open(&paths.guidance, &installation.codex_guidance_receipt_path())?;
    let mut inspected = owner.inspect()?;
    require_receipt_release(inspected.release_id.as_deref(), id)?;
    if inspected.status == "pending" {
        owner.verify_receipt_release(id)?;
        inspected = owner.reconcile()?;
    }
    require_guidance(&inspected, id)?;
    if inspected.status == "managed" {
        owner.remove()?;
    }
    Ok(())
}

fn verify_completed_native(
    installation: &Installation,
    paths: &CodexPaths,
    plan: &Plan,
    step: u8,
) -> Result<(), String> {
    if step >= 1 {
        let inspected =
            CodexConfig::open(&paths.config, &installation.codex_receipt_path())?.inspect()?;
        if inspected.status != "missing" || inspected.pending_transaction {
            return Err(
                "removed Codex registration reappeared or has uncertain state; preserved".into(),
            );
        }
    }
    if step >= 2
        && CodexGuidance::open(&paths.guidance, &installation.codex_guidance_receipt_path())?
            .inspect()?
            .status
            != "missing"
    {
        return Err("removed Codex guidance reappeared or has uncertain state; preserved".into());
    }
    if step >= 3 {
        SkillSync::inspect_global_removal(
            &paths.user_home,
            &installation.codex_skills_receipt_path(),
            &plan.global_skills,
        )?;
    }
    Ok(())
}

fn removal_plan(
    installation: &Installation,
    paths: &CodexPaths,
    active: &VerifiedRelease,
    releases: &[VerifiedRelease],
    catalog: &QualifiedCatalog,
) -> Result<Plan, String> {
    let home = &installation.home;
    let mut entries = Vec::new();
    // Keep the stable executable usable throughout payload removal. A lost
    // response after its final unlink is reconciled by the verified external
    // release/installer binary; never claim an installed recovery executable
    // still exists at that last boundary.
    let launcher = snapshot_entry(home, &installation.launcher_path())?;
    for path in [home.join("active.json"), home.join("staged.json")] {
        if present(&path)? {
            entries.push(snapshot_entry(home, &path)?);
        }
    }
    let mut directories = BTreeMap::new();
    let mut release_ids = BTreeSet::new();
    for release in releases {
        release_ids.insert(release.id().as_str().to_owned());
        for file in &release.manifest().files {
            let entry = snapshot_entry(home, &release.root().join(&file.relative_path))?;
            if entry.content
                != (EntryKind::File {
                    bytes: file.byte_len,
                    sha256: file.sha256.clone(),
                    mode: file.mode,
                })
            {
                return Err("uninstall file differs from its verified manifest; preserved".into());
            }
            entries.push(entry);
        }
        for link in &release.manifest().links {
            let entry = snapshot_entry(home, &release.root().join(&link.relative_path))?;
            if entry.content
                != (EntryKind::Link {
                    target: PathBuf::from(&link.target),
                })
            {
                return Err("uninstall link differs from its verified manifest; preserved".into());
            }
            entries.push(entry);
        }
        entries.push(snapshot_entry(
            home,
            &release.root().join(crate::distribution::MANIFEST_NAME),
        )?);
        let reopened = VerifiedRelease::open_installed(release.root()).map_err(io_message)?;
        if reopened.id().as_str() != release.id().as_str() {
            return Err("uninstall release changed during inventory capture; preserved".into());
        }
        for directory in std::iter::once("").chain(release.directories().iter().map(String::as_str))
        {
            let path = release.root().join(directory);
            directories.insert(path.clone(), snapshot_entry(home, &path)?);
        }
    }
    let mut directories = directories.into_values().collect::<Vec<_>>();
    directories.sort_by(|a, b| {
        b.path
            .matches('/')
            .count()
            .cmp(&a.path.matches('/').count())
            .then(a.path.cmp(&b.path))
    });
    entries.extend(directories);
    entries.push(launcher);
    let plan = Plan {
        schema_version: 1,
        home: home.clone(),
        user_home: paths.user_home.clone(),
        release_id: active.id().as_str().into(),
        release_ids,
        global_skills: catalog
            .catalog
            .global
            .iter()
            .map(|entry| entry.id.clone())
            .collect(),
        entries,
    };
    validate_plan(&plan, home)?;
    Ok(plan)
}

fn snapshot_entry(home: &Path, path: &Path) -> Result<Entry, String> {
    let metadata = fs::symlink_metadata(path).map_err(io_message)?;
    let content = if metadata.is_file() {
        EntryKind::File {
            bytes: metadata.len(),
            sha256: file_hash(path, metadata.len())?,
            mode: file_mode(&metadata),
        }
    } else if metadata.file_type().is_symlink() {
        EntryKind::Link {
            target: fs::read_link(path).map_err(io_message)?,
        }
    } else if metadata.is_dir() {
        EntryKind::Directory
    } else {
        return Err("uninstall refuses a special payload file".into());
    };
    let entry = Entry {
        path: path
            .strip_prefix(home)
            .map_err(io_message)?
            .to_str()
            .ok_or("uninstall paths must be UTF-8")?
            .into(),
        identity: identity(&metadata)?,
        content,
    };
    verify_entry(home, &entry)?;
    Ok(entry)
}
fn identity(metadata: &fs::Metadata) -> Result<Identity, String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(Identity {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        Err("native uninstall file identities are unqualified".into())
    }
}
fn file_mode(metadata: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o7777
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        0
    }
}
fn file_hash(path: &Path, expected_bytes: u64) -> Result<String, String> {
    let mut file = File::open(path).map_err(io_message)?;
    let before = file.metadata().map_err(io_message)?;
    if !before.is_file() || before.len() != expected_bytes {
        return Err("uninstall payload size changed".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.nlink() != 1 {
            return Err("uninstall payload has aliases".into());
        }
    }
    let mut digest = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(io_message)?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or("uninstall size overflow")?;
        if total > expected_bytes {
            return Err("uninstall payload grew while reading".into());
        }
        digest.update(&buffer[..count]);
    }
    let after = fs::symlink_metadata(path).map_err(io_message)?;
    if total != expected_bytes
        || identity(&before)? != identity(&after)?
        || file_mode(&before) != file_mode(&after)
    {
        return Err("uninstall payload changed while reading".into());
    }
    Ok(hex(&digest.finalize()))
}
fn verify_entry(home: &Path, entry: &Entry) -> Result<(), String> {
    let path = home.join(&entry.path);
    validate_native_path(path.parent().ok_or("uninstall path has no parent")?)?;
    let metadata = fs::symlink_metadata(&path).map_err(io_message)?;
    if identity(&metadata)? != entry.identity {
        return Err(format!(
            "uninstall payload identity changed: {}",
            path.display()
        ));
    }
    let matches = match &entry.content {
        EntryKind::File {
            bytes,
            sha256,
            mode,
        } => {
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && file_mode(&metadata) == *mode
                && file_hash(&path, *bytes)? == *sha256
        }
        EntryKind::Link { target } => {
            metadata.file_type().is_symlink()
                && fs::read_link(&path).map_err(io_message)? == *target
        }
        EntryKind::Directory => {
            metadata.is_dir() && !metadata.file_type().is_symlink() && file_mode(&metadata) == 0o700
        }
    };
    if !matches {
        return Err(format!("uninstall payload drifted: {}", path.display()));
    }
    Ok(())
}
fn verify_removed_prefix(home: &Path, plan: &Plan, count: usize) -> Result<(), String> {
    for entry in plan.entries.iter().take(count) {
        if present(&home.join(&entry.path))? {
            return Err(format!(
                "removed uninstall path reappeared; preserved: {}",
                entry.path
            ));
        }
    }
    Ok(())
}
fn verify_payload_progress(home: &Path, plan: &Plan, progress: &Progress) -> Result<(), String> {
    let index = if progress.step < 3 { 0 } else { progress.index };
    verify_removed_prefix(home, plan, index)?;
    for (position, entry) in plan.entries.iter().enumerate().skip(index) {
        if progress.step == 3 && position == index && !present(&home.join(&entry.path))? {
            continue;
        }
        verify_entry(home, entry)?;
    }
    let outstanding = plan
        .entries
        .iter()
        .skip(index)
        .map(|entry| home.join(&entry.path))
        .collect::<BTreeSet<_>>();
    // A new payload child is not authorized merely because an older plan knew
    // its parent. Refuse it before resuming any native or payload effects.
    for entry in plan
        .entries
        .iter()
        .skip(index)
        .filter(|entry| matches!(entry.content, EntryKind::Directory))
    {
        let path = home.join(&entry.path);
        if !present(&path)? {
            continue;
        }
        for child in fs::read_dir(path).map_err(io_message)? {
            if !outstanding.contains(&child.map_err(io_message)?.path()) {
                return Err("uninstall payload contains an unknown entry; preserved".into());
            }
        }
    }
    Ok(())
}

fn validate_plan(plan: &Plan, home: &Path) -> Result<(), String> {
    if plan.schema_version != 1
        || plan.home != home
        || plan.entries.is_empty()
        || plan.entries.len() > MAX_ENTRIES
        || plan.release_ids.is_empty()
        || plan.release_ids.len() > 3
        || !plan.release_ids.contains(&plan.release_id)
        || plan.global_skills.len() != 9
    {
        return Err("invalid uninstall plan; preserved".into());
    }
    let mut ids = BTreeSet::new();
    for id in &plan.global_skills {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            || !ids.insert(id)
        {
            return Err("invalid uninstall global skill identity".into());
        }
    }
    for id in &plan.release_ids {
        if !id.strip_prefix("sha256:").is_some_and(valid_hash) {
            return Err("invalid uninstall release identity".into());
        }
    }
    let mut seen = BTreeSet::new();
    let launcher = if cfg!(windows) {
        "bin/mirket.exe"
    } else {
        "bin/mirket"
    };
    if plan.entries.last().map(|entry| entry.path.as_str()) != Some(launcher) {
        return Err(
            "uninstall must retain its stable launcher until the final payload removal".into(),
        );
    }
    for entry in &plan.entries {
        let path = Path::new(&entry.path);
        let parts = path.components().collect::<Vec<_>>();
        if entry.path.is_empty()
            || path.is_absolute()
            || entry.path.contains('\\')
            || parts
                .iter()
                .any(|part| !matches!(part, Component::Normal(_)))
            || !seen.insert(&entry.path)
        {
            return Err("invalid uninstall destination; preserved".into());
        }
        let fixed =
            matches!(entry.path.as_str(), "active.json" | "staged.json") || entry.path == launcher;
        let release = parts.len() >= 2
            && parts[0].as_os_str() == "releases"
            && parts[1]
                .as_os_str()
                .to_str()
                .and_then(|name| name.strip_prefix("r-"))
                .is_some_and(|digest| plan.release_ids.contains(&format!("sha256:{digest}")));
        if !fixed && !release || fixed && !matches!(entry.content, EntryKind::File { .. }) {
            return Err("uninstall destination is outside fixed payload ownership".into());
        }
        if let EntryKind::File { sha256, .. } = &entry.content
            && !valid_hash(sha256)
        {
            return Err("invalid uninstall file digest".into());
        }
    }
    Ok(())
}
fn require_user(plan: &Plan, paths: &CodexPaths) -> Result<(), String> {
    if plan.user_home != paths.user_home {
        return Err("uninstall belongs to a different explicit user home".into());
    }
    Ok(())
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn present(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io_message(error)),
    }
}
fn record_bytes(path: &Path, limit: u64) -> Result<Option<Vec<u8>>, String> {
    if !present(path)? {
        return Ok(None);
    }
    validate_native_path(path)?;
    let metadata = fs::symlink_metadata(path).map_err(io_message)?;
    if !metadata.is_file() || metadata.len() > limit || file_mode(&metadata) != 0o600 {
        return Err("uninstall record must be a bounded private regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err("uninstall record cannot have aliases".into());
        }
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(io_message)?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(io_message)?;
    if bytes.len() as u64 != metadata.len()
        || identity(&metadata)? != identity(&fs::symlink_metadata(path).map_err(io_message)?)?
    {
        return Err("uninstall record changed while reading".into());
    }
    Ok(Some(bytes))
}
fn read_plan(home: &Path) -> Result<Option<(Plan, String)>, String> {
    let path = home.join("state").join(PLAN);
    let current = record_bytes(&path, MAX_PLAN_BYTES)?;
    let next = record_bytes(&path.with_extension("json.next"), MAX_PLAN_BYTES)?;
    if let (Some(current), Some(next)) = (&current, &next)
        && current != next
    {
        return Err("uninstall plan write differs; preserved for inspection".into());
    }
    let Some(bytes) = current.or(next) else {
        return Ok(None);
    };
    let plan = serde_json::from_slice::<Plan>(&bytes).map_err(io_message)?;
    validate_plan(&plan, home)?;
    Ok(Some((plan, hash(&bytes))))
}
fn read_progress(home: &Path, digest: &str, entries: usize) -> Result<Progress, String> {
    let path = home.join("state").join(PROGRESS);
    let parse = |bytes: &[u8]| -> Result<Progress, String> {
        let progress = serde_json::from_slice::<Progress>(bytes).map_err(io_message)?;
        if progress.schema_version != 1
            || progress.plan_sha256 != digest
            || progress.step > 4
            || progress.index > entries
            || progress.step < 3 && progress.index != 0
            || progress.step == 4 && progress.index != entries
        {
            return Err("invalid uninstall progress; preserved".into());
        }
        Ok(progress)
    };
    let current = record_bytes(&path, 4096)?
        .map(|bytes| parse(&bytes))
        .transpose()?
        .unwrap_or(Progress {
            schema_version: 1,
            plan_sha256: digest.into(),
            generation: 0,
            step: 0,
            index: 0,
        });
    if let Some(bytes) = record_bytes(&path.with_extension("json.next"), 4096)? {
        let next = parse(&bytes)?;
        let successor = current.generation.checked_add(1) == Some(next.generation)
            && (next.step == current.step
                && next.step == 3
                && current.index.checked_add(1) == Some(next.index)
                || current.step.checked_add(1) == Some(next.step) && next.index == current.index);
        if next != current && !successor {
            return Err("uninstall progress write is not its known successor; preserved".into());
        }
        return Ok(next);
    }
    Ok(current)
}
fn create_record(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("json.next");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary).map_err(io_message)?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(io_message)?;
    if present(path)? {
        return Err("uninstall record destination appeared; preserved".into());
    }
    replace_native_path(&temporary, path)
}
fn settle_record(path: &Path) -> Result<(), String> {
    let next = path.with_extension("json.next");
    if present(&next)? {
        replace_native_path(&next, path)?;
    }
    Ok(())
}
fn advance(home: &Path, progress: &mut Progress) -> Result<(), String> {
    progress.generation = progress
        .generation
        .checked_add(1)
        .ok_or("uninstall generation overflow")?;
    let path = home.join("state").join(PROGRESS);
    let bytes = serde_json::to_vec(progress).map_err(io_message)?;
    if !present(&path)? {
        return create_record(&path, &bytes);
    }
    let temporary = path.with_extension("json.next");
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary).map_err(io_message)?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(io_message)?;
    replace_native_path(&temporary, &path)
}
fn sync_parent(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    File::open(path.parent().ok_or("uninstall path has no parent")?)
        .and_then(|file| file.sync_all())
        .map_err(io_message)?;
    Ok(())
}
fn maybe_fail(selected: Option<usize>, step: usize) -> Result<(), String> {
    if selected == Some(step) {
        Err("injected uninstall interruption; inspect and reconcile the original plan".into())
    } else {
        Ok(())
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use crate::installation::tests::{Fixture, codex_payload, codex_user};

    #[test]
    #[ignore = "requires an explicit verified real source package and a new private qualification root; leaves final-unlink recovery evidence"]
    fn real_release_final_launcher_unlink_fixture() {
        let source = PathBuf::from(
            std::env::var_os("MIRKET_UNINSTALL_QUALIFICATION_SOURCE")
                .expect("explicit source package"),
        );
        let root = PathBuf::from(
            std::env::var_os("MIRKET_UNINSTALL_QUALIFICATION_ROOT")
                .expect("explicit new evidence root"),
        );
        validate_native_path(&root).unwrap();
        assert!(!root.exists(), "never replace prior qualification evidence");
        create_private_directory(&root).unwrap();
        let user = root.join("user-home");
        create_private_directory(&user).unwrap();
        let home = user.join(".mirket");
        let setup = Installation::setup_codex(&home, &source, &user).unwrap();
        assert_eq!(setup.core.activation.status, "active");
        let installation = Installation::open(&home).unwrap();
        fs::write(
            home.join("state/preserved-fact-evidence"),
            b"protected fixture data",
        )
        .unwrap();
        let active = installation.store.inspect_active().unwrap().unwrap();
        let paths = CodexPaths::resolve(&user, &home).unwrap();
        let catalog = release_catalog(&active).unwrap();
        let plan = removal_plan(
            &installation,
            &paths,
            &active,
            &installation.store.uninstall_releases().unwrap(),
            &catalog,
        )
        .unwrap();
        installation
            .uninstall_inner(&user, false, Some(3 + plan.entries.len() - 1))
            .unwrap_err();
        assert!(!installation.launcher_path().exists());
        assert!(
            fs::read_dir(home.join("releases"))
                .unwrap()
                .next()
                .is_none()
        );
        let status = installation.uninstall_status(&user).unwrap();
        assert_eq!(status.status, "uninstall_pending");
        fs::write(
            root.join("after-final-unlink.json"),
            serde_json::to_vec_pretty(&status).unwrap(),
        )
        .unwrap();
    }

    fn installed() -> (Fixture, Installation, PathBuf, PathBuf) {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, true);
        let home = fixture.0.join("home");
        Installation::setup_codex(&home, &source, &user).unwrap();
        let installation = Installation::open(&home).unwrap();
        (fixture, installation, user, source)
    }

    #[test]
    fn uninstall_removes_only_owned_integration_and_payload_preserving_durable_and_unrelated_bytes()
    {
        let (_fixture, installation, user, source) = installed();
        let home = installation.home();
        let personal = user.join(".agents/skills/personal-method");
        fs::create_dir(&personal).unwrap();
        fs::write(personal.join("SKILL.md"), "User owned skill\n").unwrap();
        for (path, bytes) in [
            (
                home.join("state/canonical-memory.db"),
                b"canonical facts".as_slice(),
            ),
            (
                home.join("state/recovery-point"),
                b"protected recovery".as_slice(),
            ),
            (home.join("notes.txt"), b"unknown home file".as_slice()),
        ] {
            fs::write(path, bytes).unwrap();
        }
        fs::create_dir(home.join("backups")).unwrap();
        fs::write(
            home.join("backups/pre-migration.sqlite"),
            b"protected backup",
        )
        .unwrap();
        let result = installation.uninstall_codex(&user).unwrap();
        assert_eq!(result.status, "uninstalled");
        assert_eq!(result.removed_payload_entries, result.total_payload_entries);
        assert!(!installation.launcher_path().exists());
        assert!(!home.join("active.json").exists());
        assert_eq!(fs::read_dir(home.join("releases")).unwrap().count(), 0);
        assert_eq!(
            fs::read(home.join("state/canonical-memory.db")).unwrap(),
            b"canonical facts"
        );
        assert_eq!(
            fs::read(home.join("state/recovery-point")).unwrap(),
            b"protected recovery"
        );
        assert_eq!(
            fs::read(home.join("backups/pre-migration.sqlite")).unwrap(),
            b"protected backup"
        );
        assert_eq!(
            fs::read(home.join("notes.txt")).unwrap(),
            b"unknown home file"
        );
        assert_eq!(
            fs::read(personal.join("SKILL.md")).unwrap(),
            b"User owned skill\n"
        );
        let config = fs::read_to_string(user.join(".codex/config.toml")).unwrap();
        assert!(config.contains("# User config\nmodel = 'chosen-model'"));
        assert!(!config.contains("mcp_servers.mirket"));
        assert_eq!(
            fs::read_to_string(user.join(".codex/AGENTS.md")).unwrap(),
            "User instructions: preserve original wording.\n"
        );
        assert_eq!(
            fs::read_dir(user.join(".agents/skills")).unwrap().count(),
            1
        );
        assert!(installation.store.bind_active().is_err());
        assert!(installation.stage(&source).is_err());
        assert_eq!(
            Installation::doctor_codex(home, &user).status,
            "uninstalled"
        );
        assert_eq!(
            installation.uninstall_codex(&user).unwrap().status,
            "uninstalled"
        );
    }

    #[test]
    fn uninstall_busy_and_wrong_user_refuse_without_creating_a_plan_or_changing_native_files() {
        let (fixture, installation, user, _) = installed();
        let paths = CodexPaths::resolve(&user, installation.home()).unwrap();
        let before = NativeSnapshot::read(&installation, &paths).unwrap();
        let session = installation.store.open_active().unwrap();
        assert!(
            installation
                .uninstall_codex(&user)
                .unwrap_err()
                .contains("idle maintenance")
        );
        assert!(!installation.home.join("state").join(PLAN).exists());
        assert!(NativeSnapshot::read(&installation, &paths).unwrap() == before);
        drop(session);
        installation
            .uninstall_inner(&user, false, Some(0))
            .unwrap_err();
        let wrong = fixture.0.join("other-user");
        create_private_directory(&wrong).unwrap();
        assert!(
            installation
                .reconcile_uninstall(&wrong)
                .unwrap_err()
                .contains("different explicit user home")
        );
    }

    #[test]
    fn uninstall_preflight_refuses_skill_payload_and_unknown_release_drift_before_native_effects() {
        for drift in ["skill", "registration", "guidance", "payload", "unknown"] {
            let (_fixture, installation, user, _) = installed();
            let active = installation.store.inspect_active().unwrap().unwrap();
            match drift {
                "skill" => fs::write(
                    user.join(".agents/skills/code-change/SKILL.md"),
                    "User edit",
                )
                .unwrap(),
                "payload" => {
                    fs::write(active.root().join("runtime/node/LICENSE"), "changed").unwrap()
                }
                "registration" => {
                    let path = user.join(".codex/config.toml");
                    let before = fs::read_to_string(&path).unwrap();
                    let changed = before.replace("required = true", "required = false");
                    assert_ne!(changed, before);
                    fs::write(path, changed).unwrap();
                }
                "guidance" => {
                    let path = user.join(".codex/AGENTS.md");
                    let before = fs::read_to_string(&path).unwrap();
                    fs::write(
                        path,
                        before.replace("For project memory", "For changed project memory"),
                    )
                    .unwrap();
                }
                _ => fs::write(active.root().join("unknown.txt"), "preserve").unwrap(),
            }
            let paths = CodexPaths::resolve(&user, installation.home()).unwrap();
            let config = fs::read(&paths.config).unwrap();
            let guidance = fs::read(&paths.guidance).unwrap();
            assert!(installation.uninstall_codex(&user).is_err(), "{drift}");
            assert!(!installation.home.join("state").join(PLAN).exists());
            assert_eq!(fs::read(&paths.config).unwrap(), config);
            assert_eq!(fs::read(&paths.guidance).unwrap(), guidance);
            assert!(installation.launcher_path().exists());
        }
    }

    #[test]
    fn interrupted_native_and_armed_payload_effects_resume_the_same_plan_without_replay() {
        for failure in [0, 1, 2, 3, 5] {
            let (_fixture, installation, user, source) = installed();
            installation
                .uninstall_inner(&user, false, Some(failure))
                .unwrap_err();
            let plan = fs::read(installation.home.join("state").join(PLAN)).unwrap();
            let status = installation.uninstall_status(&user).unwrap();
            assert_eq!(status.status, "uninstall_pending");
            assert_eq!(
                Installation::doctor_codex(installation.home(), &user).status,
                "uninstall_pending"
            );
            assert!(installation.uninstall_codex(&user).is_err());
            assert!(installation.store.bind_active().is_err());
            assert!(installation.stage(&source).is_err());
            assert!(installation.lock_codex_operation().is_err());
            let state_before = fs::read_dir(installation.home.join("state"))
                .unwrap()
                .count();
            installation.uninstall_status(&user).unwrap();
            assert_eq!(
                fs::read_dir(installation.home.join("state"))
                    .unwrap()
                    .count(),
                state_before
            );
            let result = installation.reconcile_uninstall(&user).unwrap();
            assert_eq!(result.status, "uninstalled", "failure {failure}");
            assert_eq!(
                fs::read(installation.home.join("state").join(PLAN)).unwrap(),
                plan
            );
            assert_eq!(
                fs::read_dir(installation.home.join("releases"))
                    .unwrap()
                    .count(),
                0
            );
        }
    }

    #[test]
    fn reconcile_preserves_new_unknown_entries_and_changed_unremoved_payload() {
        for unknown in [true, false] {
            let (_fixture, installation, user, _) = installed();
            let active_root = installation
                .store
                .inspect_active()
                .unwrap()
                .unwrap()
                .root()
                .to_owned();
            installation
                .uninstall_inner(&user, false, Some(3))
                .unwrap_err();
            let changed = active_root.join(if unknown {
                "unknown.txt"
            } else {
                "runtime/node/LICENSE"
            });
            fs::write(&changed, b"do not delete").unwrap();
            assert!(installation.reconcile_uninstall(&user).is_err());
            assert_eq!(fs::read(changed).unwrap(), b"do not delete");
            assert!(installation.launcher_path().exists());
            assert_eq!(
                installation.uninstall_status(&user).unwrap().status,
                "uninstall_pending"
            );
        }
    }

    #[test]
    fn progress_next_is_reconciled_only_for_the_exact_plan_and_known_successor() {
        let (_fixture, installation, user, _) = installed();
        installation
            .uninstall_inner(&user, false, Some(0))
            .unwrap_err();
        let (plan, digest) = read_plan(installation.home()).unwrap().unwrap();
        let mut progress = read_progress(installation.home(), &digest, plan.entries.len()).unwrap();
        progress.step = 1;
        progress.generation = 1;
        let next = installation.home.join("state/uninstall.progress.json.next");
        fs::write(&next, serde_json::to_vec(&progress).unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&next, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert_eq!(
            installation.uninstall_status(&user).unwrap().next_step,
            "guidance"
        );
        assert_eq!(
            installation.reconcile_uninstall(&user).unwrap().status,
            "uninstalled"
        );
        assert!(!next.exists());
        progress.plan_sha256 = "f".repeat(64);
        fs::write(&next, serde_json::to_vec(&progress).unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&next, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let bytes = fs::read(&next).unwrap();
        assert!(installation.reconcile_uninstall(&user).is_err());
        assert_eq!(fs::read(next).unwrap(), bytes);
    }

    #[test]
    fn completed_status_and_doctor_refuse_a_reappeared_removed_path_without_writing() {
        let (_fixture, installation, user, _) = installed();
        installation.uninstall_codex(&user).unwrap();
        fs::write(installation.launcher_path(), b"unknown replacement").unwrap();
        let record = installation.home.join("state").join(PROGRESS);
        let before = fs::read(&record).unwrap();
        assert!(
            installation
                .uninstall_status(&user)
                .unwrap_err()
                .contains("reappeared")
        );
        let doctor = Installation::doctor_codex(installation.home(), &user);
        assert_eq!(doctor.status, "error");
        assert!(
            doctor
                .errors
                .iter()
                .any(|error| error.contains("reappeared"))
        );
        assert!(installation.reconcile_uninstall(&user).is_err());
        assert_eq!(
            fs::read(installation.launcher_path()).unwrap(),
            b"unknown replacement"
        );
        assert_eq!(fs::read(record).unwrap(), before);
    }

    #[test]
    fn complete_initial_plan_write_is_inspectable_and_only_promoted_by_explicit_reconcile() {
        let (_fixture, installation, user, _) = installed();
        installation
            .uninstall_inner(&user, false, Some(0))
            .unwrap_err();
        let record = installation.home.join("state").join(PLAN);
        let next = record.with_extension("json.next");
        fs::rename(&record, &next).unwrap();
        let before = fs::read(&next).unwrap();
        assert_eq!(
            installation.uninstall_status(&user).unwrap().status,
            "uninstall_pending"
        );
        assert!(!record.exists());
        assert_eq!(fs::read(&next).unwrap(), before);
        assert_eq!(
            installation.reconcile_uninstall(&user).unwrap().status,
            "uninstalled"
        );
        assert!(!next.exists());
        assert_eq!(fs::read(record).unwrap(), before);
    }

    #[test]
    fn reconcile_refuses_reintroduced_completed_native_state_before_deleting_payloads() {
        let (_fixture, installation, user, _) = installed();
        let config_path = user.join(".codex/config.toml");
        let registered = fs::read(&config_path).unwrap();
        installation
            .uninstall_inner(&user, false, Some(1))
            .unwrap_err();
        // Registration completion was committed before the guidance effect.
        fs::write(&config_path, &registered).unwrap();
        assert!(
            installation
                .reconcile_uninstall(&user)
                .unwrap_err()
                .contains("registration reappeared")
        );
        assert!(installation.launcher_path().exists());
        assert_eq!(fs::read(&config_path).unwrap(), registered);
        assert!(installation.uninstall_status(&user).is_err());
    }

    #[test]
    fn completed_status_refuses_reintroduced_global_skill_receipt_without_deleting_it() {
        let (_fixture, installation, user, _) = installed();
        let receipt = installation.codex_skills_receipt_path();
        let original = fs::read(&receipt).unwrap();
        installation.uninstall_codex(&user).unwrap();
        assert!(!receipt.exists());
        fs::write(&receipt, &original).unwrap();
        assert!(
            installation
                .uninstall_status(&user)
                .unwrap_err()
                .contains("receipt or transaction reappeared")
        );
        assert_eq!(
            Installation::doctor_codex(installation.home(), &user).status,
            "error"
        );
        assert_eq!(fs::read(&receipt).unwrap(), original);
    }

    #[test]
    fn foreign_pending_native_receipts_are_preserved_before_reconciliation() {
        for (phase, rollback) in (0..3).flat_map(|phase| [false, true].map(|old| (phase, old))) {
            let (_fixture, installation, user, _) = installed();
            let paths = CodexPaths::resolve(&user, &installation.home).unwrap();
            let active = installation.store.inspect_active().unwrap().unwrap();
            let catalog = release_catalog(&active).unwrap();
            let plan = removal_plan(
                &installation,
                &paths,
                &active,
                &installation.store.uninstall_releases().unwrap(),
                &catalog,
            )
            .unwrap();
            let plan_bytes = serde_json::to_vec(&plan).unwrap();
            create_record(&installation.home.join("state").join(PLAN), &plan_bytes).unwrap();
            let mut progress =
                read_progress(&installation.home, &hash(&plan_bytes), plan.entries.len()).unwrap();
            if phase >= 1 {
                remove_registration(&installation, &paths, &plan.release_id).unwrap();
                progress.step = 1;
                advance(&installation.home, &mut progress).unwrap();
            }
            if phase >= 2 {
                remove_guidance(&installation, &paths, &plan.release_id).unwrap();
                progress.step = 2;
                advance(&installation.home, &mut progress).unwrap();
            }
            let receipt_path = match phase {
                0 => installation.codex_receipt_path(),
                1 => installation.codex_guidance_receipt_path(),
                _ => installation.codex_skills_receipt_path(),
            };
            let mut receipt: serde_json::Value =
                serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
            let foreign_release = format!("sha256:{}", "f".repeat(64));
            if phase == 2 {
                let mut after = receipt["receipt"].clone();
                let before = if rollback {
                    let mut before = after.clone();
                    before["release_id"] = foreign_release.into();
                    after["skills"] = serde_json::json!({});
                    before
                } else {
                    after["release_id"] = foreign_release.into();
                    serde_json::Value::Null
                };
                receipt = serde_json::json!({"state":"pending", "before":before, "after":after});
            } else {
                let mut previous = receipt.clone();
                previous["release_id"] = foreign_release.clone().into();
                receipt["state"] = "pending".into();
                if !rollback {
                    receipt["release_id"] = foreign_release.into();
                }
                if phase == 1 {
                    let current = hash(&fs::read(&paths.guidance).unwrap());
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
                    receipt["previous_config_digest"] =
                        format!("sha256:{}", hash(&fs::read(&paths.config).unwrap())).into();
                    receipt["desired_config_digest"] = format!("sha256:{}", "0".repeat(64)).into();
                    receipt["prior"] = previous;
                }
            }
            let injected = serde_json::to_vec(&receipt).unwrap();
            fs::write(&receipt_path, &injected).unwrap();
            let native_before = NativeSnapshot::read(&installation, &paths).unwrap();
            assert!(installation.reconcile_uninstall(&user).is_err());
            assert!(
                fs::read(&receipt_path).unwrap() == injected,
                "phase {phase}, rollback {rollback} reconciled a foreign receipt before checking its release"
            );
            assert!(NativeSnapshot::read(&installation, &paths).unwrap() == native_before);
            assert_eq!(
                fs::read(installation.home.join("state").join(PLAN)).unwrap(),
                plan_bytes
            );
            assert!(installation.launcher_path().exists());
        }
    }

    #[test]
    fn empty_foreign_global_skill_receipt_is_preserved_before_uninstall_admission() {
        let (_fixture, installation, user, _) = installed();
        let receipt_path = installation.codex_skills_receipt_path();
        let mut receipt: serde_json::Value =
            serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
        receipt["receipt"]["release_id"] = format!("sha256:{}", "f".repeat(64)).into();
        receipt["receipt"]["skills"] = serde_json::json!({});
        let injected = serde_json::to_vec(&receipt).unwrap();
        fs::write(&receipt_path, &injected).unwrap();
        assert!(installation.uninstall_codex(&user).is_err());
        assert_eq!(fs::read(receipt_path).unwrap(), injected);
        assert!(!installation.home.join("state").join(PLAN).exists());
        assert!(installation.launcher_path().exists());
    }

    #[test]
    fn stable_launcher_is_last_and_final_lost_response_uses_external_recovery_owner() {
        let (_fixture, installation, user, _) = installed();
        installation
            .uninstall_inner(&user, false, Some(0))
            .unwrap_err();
        let (plan, _) = read_plan(installation.home()).unwrap().unwrap();
        assert_eq!(plan.entries.last().unwrap().path, "bin/mirket");
        let before_launcher = plan.entries.len() - 2;
        installation
            .uninstall_inner(&user, true, Some(3 + before_launcher))
            .unwrap_err();
        assert!(
            installation.launcher_path().exists(),
            "the installed command remains available through other payload removals"
        );
        installation
            .uninstall_inner(&user, true, Some(3 + plan.entries.len() - 1))
            .unwrap_err();
        assert!(!installation.launcher_path().exists());
        assert_eq!(
            installation.uninstall_status(&user).unwrap().status,
            "uninstall_pending"
        );
        // This test process is the external owner. Production uses the exact
        // verified source-release/installer binary at this final boundary.
        assert_eq!(
            installation.reconcile_uninstall(&user).unwrap().status,
            "uninstalled"
        );
    }
}
