//! Local staging/activation for a verified release. No download, client configuration,
//! migration or publication. An explicit empty disposable home is required.
//! The activation record is the single commit point; immutable payloads remain
//! separate from writable state. Busy sessions leave the candidate staged.
use crate::file_lock::FileLock;
use crate::{
    distribution::{MANIFEST_NAME, VerifiedRelease},
    maintenance::{MaintenanceGuard, SessionGuard},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};

const OWNER: &[u8] = b"{\"schema_version\":1,\"kind\":\"mirket-installation\"}\n";
const ACTIVATION_JOURNAL: &str = "activation.pending.json";
const LAUNCHER_BACKUP: &str = "launcher.previous";
const STAGING_JOURNAL: &str = "staging.pending.json";
const STAGING_JOURNAL_NEXT: &str = "staging.pending.json.next";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageIdentity {
    device: u64,
    inode: u64,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StagingJournal {
    schema_version: u8,
    candidate_release_id: String,
    directory_identity: Option<StageIdentity>,
    revision: u64,
    phase: StagingPhase,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum StagingPhase {
    Preparing,
    Copying {
        index: usize,
        file_identity: Option<StageIdentity>,
    },
    Linking,
    Ready,
}

struct StageFile {
    relative_path: String,
    byte_len: u64,
    digest: String,
    mode: u32,
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activation {
    pub schema_version: u8,
    pub release_id: String,
    pub previous_release_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivationJournal {
    pub schema_version: u8,
    pub candidate_release_id: String,
    pub previous_release_id: Option<String>,
    pub launcher_path: String,
    pub backup_path: String,
    pub temporary_path: String,
    pub old_launcher_present: bool,
    pub old_launcher_digest: Option<String>,
    pub candidate_launcher_digest: String,
    pub phase: ActivationPhase,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ActivationPhase {
    Prepared,
    LauncherReplaced,
    ActiveCommitted,
}
pub struct ReleaseStore {
    home: PathBuf,
    state: PathBuf,
}
pub struct ActiveRelease {
    pub release: VerifiedRelease,
    pub state_root: PathBuf,
    pub guard: SessionGuard,
}
/// A connection-bound activation pointer. It holds maintenance exclusion but
/// deliberately does not establish payload health; callers must verify the
/// exact release before opening state or dispatching any tool operation.
pub struct BoundActiveRelease {
    pub release_id: String,
    pub release_path: PathBuf,
    pub state_root: PathBuf,
    pub guard: SessionGuard,
}
#[derive(Serialize)]
pub struct ActivationResult {
    pub status: &'static str,
    pub release_id: String,
    pub reason: Option<String>,
}

impl ReleaseStore {
    pub fn initialize(home: &Path) -> Result<Self, String> {
        if !home.is_absolute() {
            return Err("installation home must be absolute".into());
        }
        match fs::symlink_metadata(home) {
            Ok(meta) => {
                if !meta.is_dir() || meta.file_type().is_symlink() {
                    return Err("installation home must be a real directory".into());
                }
                if fs::read_dir(home).map_err(err)?.next().is_some() {
                    return Err("initialization requires an empty home; existing installations are not replaced".into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => private_directory(home)?,
            Err(e) => return Err(err(e)),
        }
        let home = home.canonicalize().map_err(err)?;
        verify_private(&home)?;
        let mut owner = create_file(&home.join("installation.json"))?;
        owner.write_all(OWNER).map_err(err)?;
        owner.sync_all().map_err(err)?;
        for name in ["releases", "staging", "state"] {
            private_directory(&home.join(name))?;
        }
        sync_dir(&home)?;
        Self::open(&home)
    }
    pub fn open(home: &Path) -> Result<Self, String> {
        if !home.is_absolute() {
            return Err("installation home must be absolute".into());
        }
        verify_private(home)?;
        let home = home.canonicalize().map_err(err)?;
        if read_small(&home.join("installation.json"), 1024)? != OWNER {
            return Err("unrecognized installation; no automatic conversion".into());
        }
        for name in ["releases", "staging", "state"] {
            verify_private(&home.join(name))?;
        }
        let state = home.join("state");
        Ok(Self { home, state })
    }
    fn mutation_lock(&self) -> Result<FileLock, String> {
        let lock = self.acquire_mutation_lock()?;
        self.require_not_uninstalling()?;
        Ok(lock)
    }
    fn acquire_mutation_lock(&self) -> Result<FileLock, String> {
        let path = self.home.join("update.lock");
        let file = match create_file(&path) {
            Ok(f) => f,
            Err(_) if path.exists() => {
                let m = fs::symlink_metadata(&path).map_err(err)?;
                verify_lock_metadata(&m)?;
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&path)
                    .map_err(err)?
            }
            Err(e) => return Err(e),
        };
        file.try_lock()
            .map_err(|_| "another update operation owns this home")?;
        let file = FileLock::acquired(file);
        let opened = file.as_file().metadata().map_err(err)?;
        let current = fs::symlink_metadata(&path).map_err(err)?;
        verify_lock_metadata(&opened)?;
        verify_lock_metadata(&current)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (opened.dev(), opened.ino()) != (current.dev(), current.ino()) {
                return Err("update lock changed while acquiring ownership".into());
            }
        }
        Ok(file)
    }
    fn require_not_uninstalling(&self) -> Result<(), String> {
        for name in [
            "uninstall.json",
            "uninstall.json.next",
            "reinstall.json",
            "reinstall.json.next",
        ] {
            if path_present(&self.state.join(name))? {
                return Err("installation has a pending installation lifecycle record; inspect and explicitly reconcile uninstall or reinstall".into());
            }
        }
        Ok(())
    }

    /// The uninstall owner retains both guards until its last removal. A
    /// separate entry permits recovery while ordinary activation/staging are
    /// disabled by the persisted uninstall marker.
    pub(crate) fn lock_uninstall(
        &self,
        maintenance: &MaintenanceGuard,
    ) -> Result<FileLock, String> {
        if maintenance.state_root() != self.state {
            return Err("uninstall maintenance belongs to another installation".into());
        }
        self.acquire_mutation_lock()
    }

    /// Observe only complete, referenced installed payloads. Partial staging or
    /// activation belongs to its existing recovery owner and must finish first.
    pub(crate) fn uninstall_releases(&self) -> Result<Vec<VerifiedRelease>, String> {
        for name in [ACTIVATION_JOURNAL, STAGING_JOURNAL, STAGING_JOURNAL_NEXT] {
            if path_present(&self.state.join(name))? {
                return Err("finish pending release staging/activation before uninstalling".into());
            }
        }
        for name in ["active.json.next", "staged.json.next"] {
            if path_present(&self.home.join(name))? {
                return Err(
                    "pending release pointer must be reconciled before uninstalling".into(),
                );
            }
        }
        if fs::read_dir(self.home.join("staging"))
            .map_err(err)?
            .next()
            .is_some()
        {
            return Err(
                "staging is not empty; preserve it and finish its owning transaction".into(),
            );
        }
        let active = self
            .read_activation("active.json")?
            .ok_or("uninstall requires an active release")?;
        let mut expected = BTreeSet::from([active.release_id]);
        expected.extend(active.previous_release_id);
        if let Some(staged) = self.read_activation("staged.json")? {
            expected.insert(staged.release_id);
        }
        let inventory = self
            .release_inventory()?
            .into_iter()
            .collect::<BTreeSet<_>>();
        if inventory != expected {
            return Err(
                "release inventory differs from active/previous/staged ownership; preserved".into(),
            );
        }
        inventory
            .into_iter()
            .map(|id| {
                let release =
                    VerifiedRelease::open_installed(&self.release_path(&id)?).map_err(err)?;
                if release.id().as_str() != id {
                    return Err("uninstall release identity differs; preserved".into());
                }
                Ok(release)
            })
            .collect()
    }
    fn release_path(&self, id: &str) -> Result<PathBuf, String> {
        let hash = id
            .strip_prefix("sha256:")
            .ok_or("invalid release identity")?;
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err("invalid release identity".into());
        }
        Ok(self.home.join("releases").join(format!("r-{hash}")))
    }
    fn release_inventory(&self) -> Result<Vec<String>, String> {
        let mut ids = Vec::new();
        for entry in fs::read_dir(self.home.join("releases")).map_err(err)? {
            let entry = entry.map_err(err)?;
            if ids.len() == 3 {
                return Err("release inventory exceeds active, previous and one staged payload; inspect retained files".into());
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid release directory name")?;
            let hash = name
                .strip_prefix("r-")
                .ok_or("unknown entry in release store; preserved for inspection")?;
            let id = format!("sha256:{hash}");
            self.release_path(&id)?;
            verify_private(&entry.path())?;
            ids.push(id);
        }
        Ok(ids)
    }
    fn check_stage_capacity(&self, candidate: &str) -> Result<(), String> {
        let inventory = self.release_inventory()?;
        let active = self.read_activation("active.json")?;
        let mut allowed = vec![candidate.to_owned()];
        if let Some(active) = active {
            allowed.push(active.release_id);
            if let Some(previous) = active.previous_release_id {
                allowed.push(previous);
            }
        }
        if inventory.iter().any(|id| !allowed.contains(id)) {
            return Err("unreferenced release remains; finish interrupted activation before staging another payload".into());
        }
        if !inventory.iter().any(|id| id == candidate) && inventory.len() >= 3 {
            return Err("release capacity is full; retained payloads were preserved".into());
        }
        // At most the exact candidate directory may remain. Its ownership and
        // contents are checked against the journal before any recovery writes.
        let expected = format!("r-{}", candidate.trim_start_matches("sha256:"));
        for entry in fs::read_dir(self.home.join("staging")).map_err(err)? {
            if entry.map_err(err)?.file_name() != expected.as_str() {
                return Err("interrupted staging contains an unknown or different candidate; preserved for inspection".into());
            }
        }
        Ok(())
    }
    fn prune_retired(&self, active: &Activation) -> Result<(), String> {
        // Caller holds exclusive maintenance and the update writer lock. Only
        // fully verified owned release trees are eligible; drift stays visible.
        for id in self.release_inventory()? {
            if id == active.release_id || active.previous_release_id.as_ref() == Some(&id) {
                continue;
            }
            let path = self.release_path(&id)?;
            let retired = VerifiedRelease::open_installed(&path).map_err(err)?;
            if retired.id().as_str() != id {
                return Err("retired release identity drifted; preserved".into());
            }
            fs::remove_dir_all(path).map_err(err)?;
        }
        sync_dir(&self.home.join("releases"))
    }
    fn read_staging_journal(
        &self,
        id: &str,
        file_count: usize,
    ) -> Result<(Option<StagingJournal>, bool), String> {
        let current = read_staging_record(&self.state.join(STAGING_JOURNAL))?;
        let next = read_staging_record(&self.state.join(STAGING_JOURNAL_NEXT))?;
        for journal in [&current, &next].into_iter().flatten() {
            self.release_path(&journal.candidate_release_id)?;
            if journal.candidate_release_id != id {
                return Err("interrupted staging belongs to a different candidate; preserved for inspection".into());
            }
            if !cfg!(unix) || journal.directory_identity.is_none() {
                return Err("interrupted staging recovery requires qualified native file identities; preserved for inspection".into());
            }
            let expected_revision = match journal.phase {
                StagingPhase::Preparing => 0,
                StagingPhase::Copying { index, .. } if index < file_count => u64::try_from(index)
                    .map_err(err)?
                    .checked_add(1)
                    .ok_or("staging journal revision overflow")?,
                StagingPhase::Copying { .. } => {
                    return Err("staging journal file index is invalid".into());
                }
                StagingPhase::Linking => u64::try_from(file_count)
                    .map_err(err)?
                    .checked_add(1)
                    .ok_or("staging journal revision overflow")?,
                StagingPhase::Ready => u64::try_from(file_count)
                    .map_err(err)?
                    .checked_add(2)
                    .ok_or("staging journal revision overflow")?,
            };
            if journal.revision != expected_revision {
                return Err(
                    "staging journal phase/revision differs; preserved for inspection".into(),
                );
            }
        }
        if let Some(next) = next {
            if let Some(current) = &current {
                if next != *current && !staging_successor(current, &next) {
                    return Err("staging journal temporary is not its exact successor; preserved for inspection".into());
                }
            } else if next.revision != 0 || next.phase != StagingPhase::Preparing {
                return Err(
                    "staging journal temporary has no predecessor; preserved for inspection".into(),
                );
            }
            Ok((Some(next), true))
        } else {
            Ok((current, false))
        }
    }

    fn check_staging_space(
        &self,
        source: &VerifiedRelease,
        files: &[StageFile],
        existing_payload: &Path,
    ) -> Result<(), String> {
        let mut required = 0u64;
        for file in files {
            let existing = match fs::symlink_metadata(existing_payload.join(&file.relative_path)) {
                Ok(metadata) => metadata.len(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
                Err(error) => return Err(err(error)),
            };
            required = required
                .checked_add(file.byte_len.saturating_sub(existing))
                .ok_or("stage size overflow")?;
        }
        // Candidate temporary + old-launcher backup + a possible restoration
        // temporary. The old stable launcher already consumes disk space.
        let launcher = self.home.join("bin").join(if cfg!(windows) {
            "mirket.exe"
        } else {
            "mirket"
        });
        let old_launcher = match fs::symlink_metadata(&launcher) {
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                metadata.len()
            }
            Ok(_) => return Err("stable launcher is not a regular file".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => return Err(err(error)),
        };
        let candidate_launcher = fs::metadata(source.executable()).map_err(err)?.len();
        let transaction = old_launcher
            .checked_mul(2)
            .and_then(|n| n.checked_add(candidate_launcher))
            .ok_or("launcher recovery size overflow")?;
        let entries = files
            .len()
            .checked_add(source.manifest().links.len())
            .and_then(|n| n.checked_add(source.directories().len()))
            .ok_or("stage entry size overflow")?;
        let metadata = u64::try_from(entries)
            .map_err(err)?
            .checked_mul(4096)
            .and_then(|n| n.checked_add(1024 * 1024))
            .ok_or("stage metadata size overflow")?;
        required = required
            .checked_add(transaction)
            .and_then(|n| n.checked_add(metadata))
            .ok_or("stage size overflow")?;
        let available = available_bytes(&self.home)?;
        if available < required {
            return Err(format!(
                "insufficient staging space: {required} bytes required including metadata and launcher recovery allowance, {available} currently available"
            ));
        }
        Ok(())
    }
    pub fn stage(&self, source: &Path) -> Result<ActivationResult, String> {
        let _mutation = self.mutation_lock()?;
        let source = VerifiedRelease::open(source).map_err(err)?;
        self.stage_verified(&source)
    }

    /// A completed-uninstall intent authorizes only its exact verified candidate.
    /// All copying and staging recovery remain in the normal release owner.
    pub(crate) fn stage_reinstall(
        &self,
        source: &VerifiedRelease,
        maintenance: &MaintenanceGuard,
        permit: &crate::installation::ReinstallPermit,
    ) -> Result<ActivationResult, String> {
        if maintenance.state_root() != self.state {
            return Err("reinstall maintenance belongs to another installation".into());
        }
        let _writer = self.acquire_mutation_lock()?;
        permit.verify_stage(&self.home, source.id().as_str())?;
        self.stage_verified(source)
    }

    pub(crate) fn activate_reinstall_held(
        &self,
        launcher: &Path,
        maintenance: &MaintenanceGuard,
        permit: &crate::installation::ReinstallPermit,
    ) -> Result<ActivationResult, String> {
        if maintenance.state_root() != self.state {
            return Err("reinstall maintenance belongs to another installation".into());
        }
        let _writer = self.acquire_mutation_lock()?;
        let expected = permit.candidate_release_id();
        permit.verify_activation(&self.home, expected)?;
        if let Some(staged) = self.read_activation("staged.json")?
            && staged.release_id != expected
        {
            return Err("reinstall staged release differs from its bound candidate".into());
        }
        if let Some(journal) = read_activation_journal(&self.state)? {
            if journal.candidate_release_id != expected {
                return Err("reinstall activation journal belongs to a different candidate".into());
            }
            if let Some(result) = self.recover_activation_journal(launcher)? {
                return Ok(result);
            }
        }
        // Activation may have fully completed before the reinstall phase record
        // was committed. Reconcile that exact pair instead of activating twice.
        if self.read_activation("staged.json")?.is_none() {
            let active = self
                .read_activation("active.json")?
                .ok_or("reinstall lost its staged and active candidate")?;
            if active.release_id != expected || active.previous_release_id.is_some() {
                return Err("reinstall active pointer differs from its exact candidate".into());
            }
            let candidate =
                VerifiedRelease::open_installed(&self.release_path(expected)?).map_err(err)?;
            if candidate.id().as_str() != expected
                || launcher_snapshot(launcher)?.1 != Some(digest_file(&candidate.executable())?)
            {
                return Err("reinstall active candidate or launcher changed; preserved".into());
            }
            return Ok(ActivationResult {
                status: "active",
                release_id: expected.into(),
                reason: None,
            });
        }
        self.activate_with_launcher_locked(launcher, maintenance)
    }

    /// Roll back only to the release retained by the just-committed active
    /// pointer. It was admitted when originally staged; verify its immutable
    /// installed manifest and bytes rather than applying the new source pins.
    pub(crate) fn stage_retained_previous(&self, id: &str) -> Result<ActivationResult, String> {
        let _mutation = self.mutation_lock()?;
        let active = self
            .read_activation("active.json")?
            .ok_or("rollback requires an active release")?;
        if active.previous_release_id.as_deref() != Some(id) {
            return Err("rollback release is not the retained previous release".into());
        }
        let source = VerifiedRelease::open_installed(&self.release_path(id)?).map_err(err)?;
        if source.id().as_str() != id {
            return Err("retained previous release identity differs".into());
        }
        self.stage_verified(&source)
    }

    fn stage_verified(&self, source: &VerifiedRelease) -> Result<ActivationResult, String> {
        let id = source.id().as_str().to_owned();
        if let Some(journal) = read_activation_journal(&self.state)?
            && journal.candidate_release_id != id
        {
            return Err("pending activation belongs to a different candidate; finish it explicitly before staging another payload".into());
        }
        let target = self.release_path(&id)?;
        // One candidate at a time; never discard an unrelated staged update.
        if let Some(staged) = self.read_activation("staged.json")?
            && staged.release_id != id
        {
            return Err("another release is already staged; resolve it explicitly before staging a different one".into());
        }
        self.check_stage_capacity(&id)?;
        let stage = self
            .home
            .join("staging")
            .join(format!("r-{}", id.trim_start_matches("sha256:")));
        let manifest_bytes = serde_json::to_vec_pretty(source.manifest()).map_err(err)?;
        let files = stage_files(source, &manifest_bytes);
        let (journal, pending_next) = self.read_staging_journal(&id, files.len())?;
        let stage_present = path_present(&stage)?;
        let target_present = path_present(&target)?;
        if let Some(journal) = &journal {
            if stage_present && !target_present {
                verify_stage_tree(&stage, source, &manifest_bytes, &files, journal)?;
            } else if !stage_present && target_present && journal.phase == StagingPhase::Ready {
                // The payload rename committed, but its staged pointer or journal
                // cleanup was interrupted. Verify before completing that record.
                verify_stage_identity(
                    &fs::symlink_metadata(&target).map_err(err)?,
                    &journal.directory_identity,
                )?;
            } else {
                return Err(
                    "staging journal and payload locations differ; preserved for inspection".into(),
                );
            }
        } else if stage_present {
            return Err(
                "interrupted staging has no ownership journal; preserved for inspection".into(),
            );
        }
        if target_present {
            let saved = VerifiedRelease::open_installed(&target).map_err(err)?;
            if saved.id().as_str() != id {
                return Err("existing release directory drifted".into());
            }
            self.check_staging_space(source, &files, &target)?;
        } else {
            self.check_staging_space(source, &files, &stage)?;
            let mut journal = match journal.clone() {
                Some(journal) => journal,
                None => {
                    private_directory(&stage)?;
                    sync_dir(&stage)?;
                    sync_dir(&self.home.join("staging"))?;
                    let journal = StagingJournal {
                        schema_version: 1,
                        candidate_release_id: id.clone(),
                        directory_identity: stage_identity(
                            &fs::symlink_metadata(&stage).map_err(err)?,
                        ),
                        revision: 0,
                        phase: StagingPhase::Preparing,
                    };
                    write_staging_journal(&self.state, &journal)?;
                    journal
                }
            };
            if pending_next {
                replace_path(
                    &self.state.join(STAGING_JOURNAL_NEXT),
                    &self.state.join(STAGING_JOURNAL),
                )?;
                sync_dir(&self.state)?;
            }
            for directory in source.directories() {
                ensure_parents(&stage, &stage.join(directory))?;
            }
            let start = match &journal.phase {
                StagingPhase::Preparing => 0,
                StagingPhase::Copying { index, .. } => *index,
                StagingPhase::Linking | StagingPhase::Ready => files.len(),
            };
            for (index, entry) in files.iter().enumerate().skip(start) {
                let destination = stage.join(&entry.relative_path);
                let continuing = matches!(journal.phase, StagingPhase::Copying { index: current, .. } if current == index);
                let mut output = if continuing {
                    let output = OpenOptions::new()
                        .write(true)
                        .open(&destination)
                        .map_err(err)?;
                    let StagingPhase::Copying { file_identity, .. } = &journal.phase else {
                        unreachable!()
                    };
                    verify_stage_identity(&output.metadata().map_err(err)?, file_identity)?;
                    output
                } else {
                    let output = create_file(&destination)?;
                    // Bind the new file before copying any bytes. If interrupted
                    // before this journal is durable, its unbound entry is kept.
                    output.sync_all().map_err(err)?;
                    sync_dir(destination.parent().ok_or("invalid stage path")?)?;
                    journal.phase = StagingPhase::Copying {
                        index,
                        file_identity: stage_identity(&output.metadata().map_err(err)?),
                    };
                    advance_staging_journal(&self.state, &mut journal)?;
                    output
                };
                if !continuing || output.metadata().map_err(err)?.len() != entry.byte_len {
                    output.set_len(0).map_err(err)?;
                    // The bounded input is type-erased, so io::copy otherwise
                    // uses its small generic buffer even for native files.
                    let mut input = BufReader::with_capacity(
                        64 * 1024,
                        stage_input(source, &manifest_bytes, entry)?,
                    );
                    let copied = std::io::copy(&mut input, &mut output).map_err(err)?;
                    if copied != entry.byte_len || digest_file(&destination)? != entry.digest {
                        return Err(
                            "source payload changed during staging; partial evidence preserved"
                                .into(),
                        );
                    }
                }
                set_stage_mode(&destination, entry.mode)?;
                output.sync_all().map_err(err)?;
            }
            if journal.phase != StagingPhase::Ready {
                if journal.phase != StagingPhase::Linking {
                    journal.phase = StagingPhase::Linking;
                    advance_staging_journal(&self.state, &mut journal)?;
                }
                for link in &source.manifest().links {
                    let destination = stage.join(&link.relative_path);
                    if !path_present(&destination)? {
                        #[cfg(unix)]
                        std::os::unix::fs::symlink(&link.target, &destination).map_err(err)?;
                        #[cfg(not(unix))]
                        return Err(
                            "installed interpreter links are not qualified on this platform".into(),
                        );
                    }
                }
                let staged = VerifiedRelease::open_installed(&stage).map_err(err)?;
                if staged.id().as_str() != id {
                    return Err("staged release identity differs".into());
                }
                for directory in source.directories().iter().rev() {
                    sync_dir(&stage.join(directory))?;
                }
                sync_dir(&stage)?;
                journal.phase = StagingPhase::Ready;
                advance_staging_journal(&self.state, &mut journal)?;
            }
            fs::rename(&stage, &target).map_err(err)?;
            sync_dir(&self.home.join("releases"))?;
            sync_dir(&self.home.join("staging"))?;
        }
        self.write_activation(
            "staged.json",
            &Activation {
                schema_version: 1,
                release_id: id.clone(),
                previous_release_id: None,
            },
        )?;
        // The staged record now names a verified immutable release. Only the
        // two bounded records selected and validated above may be retired.
        if journal.is_some() || !target_present {
            remove_stage_record(&self.state.join(STAGING_JOURNAL))?;
            if pending_next {
                remove_stage_record(&self.state.join(STAGING_JOURNAL_NEXT))?;
            }
            sync_dir(&self.state)?;
        }
        Ok(ActivationResult {
            status: "staged",
            release_id: id,
            reason: None,
        })
    }
    pub fn activate(&self) -> Result<ActivationResult, String> {
        let _mutation = self.mutation_lock()?;
        let staged = self
            .read_activation("staged.json")?
            .ok_or("no staged release; nothing was activated")?;
        let _maintenance = match MaintenanceGuard::try_acquire(&self.state) {
            Ok(g) => g,
            Err(e) => {
                return Ok(ActivationResult {
                    status: "staged",
                    release_id: staged.release_id,
                    reason: Some(e.to_string()),
                });
            }
        };
        let candidate = VerifiedRelease::open_installed(&self.release_path(&staged.release_id)?)
            .map_err(err)?;
        if candidate.id().as_str() != staged.release_id {
            return Err("staged identity mismatch".into());
        }
        let active = self.read_activation("active.json")?;
        let previous = active.as_ref().and_then(|a| {
            if a.release_id == staged.release_id {
                a.previous_release_id.clone()
            } else {
                Some(a.release_id.clone())
            }
        });
        // Refuse unrelated inventory before changing the active pointer.
        self.release_inventory()?;
        let activation = Activation {
            schema_version: 1,
            release_id: staged.release_id.clone(),
            previous_release_id: previous,
        };
        self.write_activation("active.json", &activation)?;
        if let Err(error) = self.prune_retired(&activation) {
            // The pointer is already committed. Keep the staged record so an
            // explicit activation retry can finish cleanup; never report rollback.
            return Ok(ActivationResult {
                status: "active",
                release_id: staged.release_id,
                reason: Some(format!(
                    "activation committed; retired payload cleanup incomplete: {error}"
                )),
            });
        }
        // A crash here leaves the same candidate staged. A later explicit
        // activation recognizes it; ordinary server launch never activates it.
        fs::remove_file(self.home.join("staged.json")).map_err(err)?;
        sync_dir(&self.home)?;
        Ok(ActivationResult {
            status: "active",
            release_id: staged.release_id,
            reason: None,
        })
    }

    /// Activate a staged release while committing the stable launcher and the
    /// active pointer under one maintenance window. A launcher replacement
    /// failure leaves the old active pair intact and keeps the candidate
    /// staged for an explicit retry.
    pub fn activate_with_launcher(&self, launcher_path: &Path) -> Result<ActivationResult, String> {
        let _mutation = self.mutation_lock()?;
        let staged = self.read_activation("staged.json")?;
        let maintenance = match MaintenanceGuard::try_acquire(&self.state) {
            Ok(guard) => guard,
            Err(error) => {
                let staged = staged.ok_or(
                    "an interrupted activation cannot be reconciled while sessions are active",
                )?;
                return Ok(ActivationResult {
                    status: "staged",
                    release_id: staged.release_id,
                    reason: Some(error.to_string()),
                });
            }
        };
        self.activate_with_launcher_locked(launcher_path, &maintenance)
    }

    /// The Codex installer retains one maintenance window while committing
    /// the launcher, native skills, guidance, and registration. Sessions must
    /// not enter between these steps or observe a mixed release.
    pub(crate) fn activate_with_launcher_held(
        &self,
        launcher_path: &Path,
        maintenance: &MaintenanceGuard,
    ) -> Result<ActivationResult, String> {
        if maintenance.state_root() != self.state {
            return Err("maintenance guard belongs to another installation".into());
        }
        let _mutation = self.mutation_lock()?;
        self.activate_with_launcher_locked(launcher_path, maintenance)
    }

    fn activate_with_launcher_locked(
        &self,
        launcher_path: &Path,
        _maintenance: &MaintenanceGuard,
    ) -> Result<ActivationResult, String> {
        validate_launcher_path(&self.home, launcher_path)?;
        if let Some(result) = self.recover_activation_journal(launcher_path)? {
            return Ok(result);
        }
        let staged = self
            .read_activation("staged.json")?
            .ok_or("no staged release; nothing was activated")?;
        let candidate = VerifiedRelease::open_installed(&self.release_path(&staged.release_id)?)
            .map_err(err)?;
        if candidate.id().as_str() != staged.release_id {
            return Err("staged identity mismatch".into());
        }
        let active = self.read_activation("active.json")?;
        let previous = active.as_ref().and_then(|record| {
            if record.release_id == staged.release_id {
                record.previous_release_id.clone()
            } else {
                Some(record.release_id.clone())
            }
        });
        self.release_inventory()?;
        let bin = launcher_path.parent().ok_or("launcher has no parent")?;
        match fs::symlink_metadata(bin) {
            Ok(_) => verify_private(bin)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                private_directory(bin)?;
            }
            Err(error) => return Err(err(error)),
        }
        let old = launcher_snapshot(launcher_path)?;
        let backup = self.state.join(LAUNCHER_BACKUP);
        let temporary = bin.join(format!(".mirket-launcher-next-{}", std::process::id()));
        if fs::symlink_metadata(&backup).is_ok() || fs::symlink_metadata(&temporary).is_ok() {
            return Err(
                "interrupted launcher transaction exists; reconcile it before retrying".into(),
            );
        }
        if old.0 {
            copy_private_file(launcher_path, &backup)?;
        }
        let candidate_digest = digest_file(&candidate.executable())?;
        let journal = ActivationJournal {
            schema_version: 1,
            candidate_release_id: staged.release_id.clone(),
            previous_release_id: previous.clone(),
            launcher_path: launcher_path.to_string_lossy().into_owned(),
            backup_path: backup.to_string_lossy().into_owned(),
            temporary_path: temporary.to_string_lossy().into_owned(),
            old_launcher_present: old.0,
            old_launcher_digest: old.1.clone(),
            candidate_launcher_digest: candidate_digest.clone(),
            phase: ActivationPhase::Prepared,
        };
        if let Err(error) = write_activation_journal(&self.state, &journal) {
            // If no durable journal was installed, the backup has no recovery
            // owner and must not strand the next activation attempt. A journal
            // that did reach disk is preserved for explicit reconciliation.
            let journal_missing = matches!(
                fs::symlink_metadata(self.state.join(ACTIVATION_JOURNAL)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound
            );
            if journal_missing {
                remove_owned_path(&backup)?;
            }
            return Err(error);
        }
        if let Err(error) = copy_and_replace(&candidate.executable(), launcher_path, &temporary) {
            let recovery = self.recover_activation_journal(launcher_path);
            return match recovery {
                Ok(_) => Err(error),
                Err(recovery_error) => Err(format!(
                    "launcher replacement failed and recovery is pending: {error}; {recovery_error}"
                )),
            };
        }
        let mut journal = journal;
        journal.phase = ActivationPhase::LauncherReplaced;
        if let Err(error) = write_activation_journal(&self.state, &journal) {
            let recovery = self.recover_activation_journal(launcher_path);
            return match recovery {
                Ok(_) => Err(error),
                Err(recovery_error) => Err(format!(
                    "launcher replacement journal failed and recovery is pending: {error}; {recovery_error}"
                )),
            };
        }
        let activation = Activation {
            schema_version: 1,
            release_id: staged.release_id.clone(),
            previous_release_id: previous,
        };
        if let Err(error) = self.write_activation("active.json", &activation) {
            let recovery = self.recover_activation_journal(launcher_path);
            return match recovery {
                Ok(None) => Err(error),
                Ok(Some(_)) => Err(format!(
                    "activation pointer failed; launcher was restored: {error}"
                )),
                Err(recovery_error) => Err(format!(
                    "activation pointer failed and launcher recovery is pending: {error}; {recovery_error}"
                )),
            };
        }
        journal.phase = ActivationPhase::ActiveCommitted;
        if let Err(error) = write_activation_journal(&self.state, &journal) {
            return Ok(ActivationResult {
                status: "active",
                release_id: staged.release_id,
                reason: Some(format!(
                    "activation committed; journal completion pending: {error}"
                )),
            });
        }
        if let Err(error) = self.prune_retired(&activation) {
            return Ok(ActivationResult {
                status: "active",
                release_id: staged.release_id,
                reason: Some(format!(
                    "activation committed; retired payload cleanup incomplete: {error}"
                )),
            });
        }
        if let Err(error) = self.finish_activation_journal(&journal) {
            return Ok(ActivationResult {
                status: "active",
                release_id: activation.release_id,
                reason: Some(format!(
                    "activation committed; transaction cleanup incomplete: {error}"
                )),
            });
        }
        if let Err(error) = fs::remove_file(self.home.join("staged.json")).map_err(err) {
            return Ok(ActivationResult {
                status: "active",
                release_id: activation.release_id,
                reason: Some(format!(
                    "activation committed; staged record cleanup incomplete: {error}"
                )),
            });
        }
        if let Err(error) = sync_dir(&self.home) {
            return Ok(ActivationResult {
                status: "active",
                release_id: activation.release_id,
                reason: Some(format!(
                    "activation committed; installation directory sync incomplete: {error}"
                )),
            });
        }
        Ok(ActivationResult {
            status: "active",
            release_id: activation.release_id,
            reason: None,
        })
    }

    fn recover_activation_journal(
        &self,
        launcher_path: &Path,
    ) -> Result<Option<ActivationResult>, String> {
        let Some(journal) = read_activation_journal(&self.state)? else {
            return Ok(None);
        };
        if let Some(staged) = self.read_activation("staged.json")?
            && staged.release_id != journal.candidate_release_id
        {
            return Err("pending activation belongs to a different staged candidate; preserve both intents for explicit reconciliation".into());
        }
        if journal.launcher_path != launcher_path.to_string_lossy().as_ref()
            || journal.backup_path != self.state.join(LAUNCHER_BACKUP).to_string_lossy().as_ref()
        {
            return Err("activation journal belongs to another launcher".into());
        }
        validate_launcher_path(&self.home, Path::new(&journal.launcher_path))?;
        let temporary = Path::new(&journal.temporary_path);
        if temporary.parent() != launcher_path.parent()
            || temporary
                .file_name()
                .and_then(|name| name.to_str())
                .is_none_or(|name| !name.starts_with(".mirket-launcher-next-"))
        {
            return Err("activation journal temporary path is invalid".into());
        }
        let active = self.read_activation("active.json")?;
        let current = launcher_snapshot(launcher_path)?;
        if active.as_ref().map(|record| record.release_id.as_str())
            == Some(journal.candidate_release_id.as_str())
        {
            let candidate =
                VerifiedRelease::open_installed(&self.release_path(&journal.candidate_release_id)?)
                    .map_err(err)?;
            if candidate.id().as_str() != journal.candidate_release_id {
                return Err("activation journal candidate identity no longer verifies".into());
            }
            let candidate_digest = digest_file(&candidate.executable())?;
            if candidate_digest != journal.candidate_launcher_digest {
                return Err("activation journal candidate digest no longer verifies".into());
            }
            if current.1.as_deref() != Some(journal.candidate_launcher_digest.as_str()) {
                if current.1.as_deref() == journal.old_launcher_digest.as_deref() {
                    remove_owned_path(temporary)?;
                    copy_and_replace(&candidate.executable(), launcher_path, temporary)?;
                } else {
                    return Err("active candidate has an unknown launcher state".into());
                }
            }
            self.prune_retired(
                active
                    .as_ref()
                    .ok_or("active candidate record is missing")?,
            )?;
            if let Some(staged) = self.read_activation("staged.json")?
                && staged.release_id == journal.candidate_release_id
            {
                fs::remove_file(self.home.join("staged.json")).map_err(err)?;
            }
            self.finish_activation_journal(&journal)?;
            sync_dir(&self.home)?;
            return Ok(Some(ActivationResult {
                status: "active",
                release_id: journal.candidate_release_id,
                reason: None,
            }));
        }
        let active_matches_previous = active.as_ref().map(|record| record.release_id.as_str())
            == journal.previous_release_id.as_deref();
        if !active_matches_previous {
            return Err("activation journal has an unknown active release".into());
        }
        if current.1.as_deref() == Some(journal.candidate_launcher_digest.as_str()) {
            if journal.old_launcher_present {
                let backup = Path::new(&journal.backup_path);
                let backup_digest = launcher_snapshot(backup)?.1;
                if backup_digest != journal.old_launcher_digest {
                    return Err("launcher backup digest does not verify".into());
                }
                remove_owned_path(temporary)?;
                copy_and_replace(backup, launcher_path, temporary)?;
            } else {
                remove_owned_path(launcher_path)?;
            }
        } else if current.0 != journal.old_launcher_present
            || current.1 != journal.old_launcher_digest
        {
            return Err("activation journal has an unknown launcher state".into());
        }
        self.finish_activation_journal(&journal)?;
        Ok(None)
    }

    fn finish_activation_journal(&self, journal: &ActivationJournal) -> Result<(), String> {
        remove_owned_path(Path::new(&journal.temporary_path))?;
        remove_owned_path(Path::new(&journal.backup_path))?;
        remove_owned_path(&self.state.join(ACTIVATION_JOURNAL))?;
        sync_dir(&self.state)
    }
    pub fn open_active(&self) -> Result<ActiveRelease, String> {
        let bound = self.bind_active()?;
        let release = VerifiedRelease::open_installed(&bound.release_path).map_err(err)?;
        if release.id().as_str() != bound.release_id {
            return Err("active identity mismatch".into());
        }
        Ok(ActiveRelease {
            release,
            state_root: bound.state_root,
            guard: bound.guard,
        })
    }
    pub fn bind_active(&self) -> Result<BoundActiveRelease, String> {
        let guard = SessionGuard::acquire(&self.state).map_err(err)?;
        self.require_not_uninstalling()?;
        let active = self
            .read_activation("active.json")?
            .ok_or("home has no active release")?;
        Ok(BoundActiveRelease {
            release_path: self.release_path(&active.release_id)?,
            release_id: active.release_id,
            state_root: self.state.clone(),
            guard,
        })
    }
    /// Inspect the active pointer and payload without acquiring a session lock.
    /// Doctor uses the same record parser and identity rules as normal startup.
    pub fn inspect_active(&self) -> Result<Option<VerifiedRelease>, String> {
        let Some(active) = self.read_activation("active.json")? else {
            return Ok(None);
        };
        let release = VerifiedRelease::open_installed(&self.release_path(&active.release_id)?)
            .map_err(err)?;
        if release.id().as_str() != active.release_id {
            return Err("active identity mismatch".into());
        }
        Ok(Some(release))
    }
    /// Verify the active payload for an operation already holding exclusive
    /// maintenance. Do not acquire a session guard under that exclusive lock,
    /// or bypass pending installation lifecycle records through doctor access.
    pub fn inspect_active_for_maintenance(
        &self,
        maintenance: &MaintenanceGuard,
    ) -> Result<Option<VerifiedRelease>, String> {
        if maintenance.state_root() != self.state {
            return Err("active release maintenance belongs to another installation".into());
        }
        self.require_not_uninstalling()?;
        self.inspect_active()
    }
    fn read_activation(&self, name: &str) -> Result<Option<Activation>, String> {
        let path = self.home.join(name);
        match fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(err(e)),
            Ok(_) => {
                let value: Activation =
                    serde_json::from_slice(&read_small(&path, 8192)?).map_err(err)?;
                if value.schema_version != 1 {
                    return Err("unsupported activation record".into());
                }
                self.release_path(&value.release_id)?;
                if let Some(previous) = &value.previous_release_id {
                    self.release_path(previous)?;
                }
                Ok(Some(value))
            }
        }
    }
    fn write_activation(&self, name: &str, value: &Activation) -> Result<(), String> {
        let path = self.home.join(name);
        let temp = self.home.join(format!("{name}.next"));
        let bytes = serde_json::to_vec(value).map_err(err)?;
        if temp.exists() {
            if read_small(&temp, 8192)? != bytes {
                return Err(
                    "interrupted activation record differs; preserve it for recovery inspection"
                        .into(),
                );
            }
        } else {
            let mut file = create_file(&temp)?;
            file.write_all(&bytes).map_err(err)?;
            file.sync_all().map_err(err)?;
        }
        replace_path(&temp, &path)?;
        sync_dir(&self.home)
    }
}

fn stage_files(source: &VerifiedRelease, manifest: &[u8]) -> Vec<StageFile> {
    let mut files: Vec<_> = source
        .manifest()
        .files
        .iter()
        .map(|file| StageFile {
            relative_path: file.relative_path.clone(),
            byte_len: file.byte_len,
            digest: format!("sha256:{}", file.sha256),
            mode: file.mode,
        })
        .collect();
    files.push(StageFile {
        relative_path: MANIFEST_NAME.into(),
        byte_len: manifest.len() as u64,
        digest: format!(
            "sha256:{}",
            Sha256::digest(manifest)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        ),
        mode: 0o600,
    });
    files
}

fn stage_input<'a>(
    source: &VerifiedRelease,
    manifest: &'a [u8],
    entry: &StageFile,
) -> Result<Box<dyn Read + 'a>, String> {
    if entry.relative_path == MANIFEST_NAME {
        Ok(Box::new(std::io::Cursor::new(manifest)))
    } else {
        Ok(Box::new(
            File::open(source.root().join(&entry.relative_path))
                .map_err(err)?
                .take(entry.byte_len + 1),
        ))
    }
}

fn path_present(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(err(error)),
    }
}

fn stage_identity(metadata: &fs::Metadata) -> Option<StageIdentity> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(StageIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        None
    }
}

fn verify_stage_identity(
    metadata: &fs::Metadata,
    expected: &Option<StageIdentity>,
) -> Result<(), String> {
    if &stage_identity(metadata) != expected {
        return Err("staging path identity drifted; preserved for inspection".into());
    }
    Ok(())
}

fn verify_stage_file_metadata(metadata: &fs::Metadata) -> Result<(), String> {
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("staging file is not an owned regular file; preserved for inspection".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 || metadata.mode() & 0o077 != 0 {
            return Err(
                "staging file must remain private and unaliased; preserved for inspection".into(),
            );
        }
    }
    Ok(())
}

fn stage_mode(metadata: &fs::Metadata, expected: u32) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o7777 == expected
    }
    #[cfg(not(unix))]
    {
        let _ = (metadata, expected);
        true
    }
}

fn set_stage_mode(path: &Path, mode: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(err)?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

// Inspect the entire retained tree before truncating the journal-bound file or
// consuming a complete successor journal. Unknown/corrupt evidence stays intact.
fn verify_stage_tree(
    stage: &Path,
    source: &VerifiedRelease,
    manifest: &[u8],
    files: &[StageFile],
    journal: &StagingJournal,
) -> Result<(), String> {
    verify_private(stage)?;
    let root_metadata = fs::symlink_metadata(stage).map_err(err)?;
    verify_stage_identity(&root_metadata, &journal.directory_identity)?;
    if !stage_mode(&root_metadata, 0o700) {
        return Err("staging directory mode drifted; preserved for inspection".into());
    }
    let completed = match journal.phase {
        StagingPhase::Preparing => 0,
        StagingPhase::Copying { index, .. } if index < files.len() => index,
        StagingPhase::Copying { .. } => return Err("staging journal file index is invalid".into()),
        StagingPhase::Linking | StagingPhase::Ready => files.len(),
    };
    let mut seen = BTreeSet::new();
    let mut directories = vec![stage.to_owned()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).map_err(err)? {
            let entry = entry.map_err(err)?;
            let path = entry.path();
            let relative = path
                .strip_prefix(stage)
                .map_err(err)?
                .components()
                .map(|part| part.as_os_str().to_str().ok_or("invalid staging entry"))
                .collect::<Result<Vec<_>, _>>()?
                .join("/");
            let metadata = fs::symlink_metadata(&path).map_err(err)?;
            if metadata.is_dir()
                && !metadata.file_type().is_symlink()
                && source.directories().contains(&relative)
            {
                verify_private(&path)?;
                if !stage_mode(&metadata, 0o700) {
                    return Err("staging directory mode drifted; preserved for inspection".into());
                }
                directories.push(path);
            } else if let Some((index, file)) = files
                .iter()
                .enumerate()
                .find(|(_, file)| file.relative_path == relative)
            {
                verify_stage_file_metadata(&metadata)?;
                if index < completed {
                    if metadata.len() != file.byte_len
                        || !stage_mode(&metadata, file.mode)
                        || digest_file(&path)? != file.digest
                    {
                        return Err(
                            "completed staging file drifted; preserved for inspection".into()
                        );
                    }
                } else if let StagingPhase::Copying {
                    index: current,
                    ref file_identity,
                } = journal.phase
                {
                    if index != current || file_identity.is_none() {
                        return Err("unbound staging file; preserved for inspection".into());
                    }
                    verify_stage_identity(&metadata, file_identity)?;
                    if metadata.len() > file.byte_len
                        || !(stage_mode(&metadata, file.mode) || stage_mode(&metadata, 0o600))
                    {
                        return Err(
                            "incomplete staging file drifted; preserved for inspection".into()
                        );
                    }
                    let mut actual = File::open(&path).map_err(err)?.take(file.byte_len + 1);
                    let mut expected = stage_input(source, manifest, file)?;
                    let mut left = [0u8; 64 * 1024];
                    let mut right = [0u8; 64 * 1024];
                    loop {
                        let count = actual.read(&mut left).map_err(err)?;
                        if count == 0 {
                            break;
                        }
                        expected.read_exact(&mut right[..count]).map_err(err)?;
                        if left[..count] != right[..count] {
                            return Err(
                                "incomplete staging file bytes drifted; preserved for inspection"
                                    .into(),
                            );
                        }
                    }
                    if metadata.len() == file.byte_len && digest_file(&path)? != file.digest {
                        return Err(
                            "complete staging file digest differs; preserved for inspection".into(),
                        );
                    }
                } else {
                    return Err("unbound staging file; preserved for inspection".into());
                }
            } else if let Some(link) = source
                .manifest()
                .links
                .iter()
                .find(|link| link.relative_path == relative)
            {
                if !matches!(journal.phase, StagingPhase::Linking | StagingPhase::Ready)
                    || !metadata.file_type().is_symlink()
                    || fs::read_link(&path).map_err(err)? != Path::new(&link.target)
                {
                    return Err("staging link drifted; preserved for inspection".into());
                }
            } else {
                return Err("unknown staging entry; preserved for inspection".into());
            }
            seen.insert(relative);
        }
    }
    for (index, file) in files.iter().enumerate() {
        if (index < completed
            || matches!(journal.phase, StagingPhase::Copying { index: current, .. } if current == index))
            && !seen.contains(&file.relative_path)
        {
            return Err("journal-owned staging file is missing; preserved for inspection".into());
        }
    }
    if journal.phase == StagingPhase::Ready
        && (source
            .directories()
            .iter()
            .any(|directory| !seen.contains(directory))
            || source
                .manifest()
                .links
                .iter()
                .any(|link| !seen.contains(&link.relative_path)))
    {
        return Err("ready staging inventory differs; preserved for inspection".into());
    }
    Ok(())
}

fn read_staging_record(path: &Path) -> Result<Option<StagingJournal>, String> {
    if !path_present(path)? {
        return Ok(None);
    }
    let metadata = fs::symlink_metadata(path).map_err(err)?;
    verify_stage_file_metadata(&metadata)?;
    if !stage_mode(&metadata, 0o600) {
        return Err("staging journal mode differs; preserved for inspection".into());
    }
    let journal: StagingJournal = serde_json::from_slice(&read_small(path, 4096)?).map_err(err)?;
    if journal.schema_version != 1 {
        return Err("unsupported staging journal schema".into());
    }
    Ok(Some(journal))
}

fn staging_successor(current: &StagingJournal, next: &StagingJournal) -> bool {
    current.candidate_release_id == next.candidate_release_id
        && current.directory_identity == next.directory_identity
        && current.revision.checked_add(1) == Some(next.revision)
        && match (&current.phase, &next.phase) {
            (StagingPhase::Preparing, StagingPhase::Copying { index: 0, .. }) => true,
            (StagingPhase::Copying { index: a, .. }, StagingPhase::Copying { index: b, .. }) => {
                a.checked_add(1) == Some(*b)
            }
            (StagingPhase::Copying { .. }, StagingPhase::Linking)
            | (StagingPhase::Linking, StagingPhase::Ready) => true,
            _ => false,
        }
}

fn write_staging_journal(state: &Path, journal: &StagingJournal) -> Result<(), String> {
    let bytes = serde_json::to_vec(journal).map_err(err)?;
    if bytes.len() > 4096 {
        return Err("staging journal exceeds its bound".into());
    }
    let temporary = state.join(STAGING_JOURNAL_NEXT);
    // Never discard a torn or unrelated temporary record.
    let mut file = create_file(&temporary)?;
    file.write_all(&bytes).map_err(err)?;
    file.sync_all().map_err(err)?;
    replace_path(&temporary, &state.join(STAGING_JOURNAL))?;
    sync_dir(state)
}

fn advance_staging_journal(state: &Path, journal: &mut StagingJournal) -> Result<(), String> {
    journal.revision = journal
        .revision
        .checked_add(1)
        .ok_or("staging journal revision overflow")?;
    write_staging_journal(state, journal)
}

fn remove_stage_record(path: &Path) -> Result<(), String> {
    if path_present(path)? {
        read_staging_record(path)?;
        fs::remove_file(path).map_err(err)?;
    }
    Ok(())
}

fn validate_launcher_path(home: &Path, launcher: &Path) -> Result<(), String> {
    let expected = home.join("bin").join(if cfg!(windows) {
        "mirket.exe"
    } else {
        "mirket"
    });
    let expected_parent = home.join("bin");
    if launcher != expected.as_path() {
        return Err("launcher path is outside the selected installation".into());
    }
    if launcher.parent() != Some(expected_parent.as_path()) {
        return Err("launcher parent is invalid".into());
    }
    Ok(())
}

fn launcher_snapshot(path: &Path) -> Result<(bool, Option<String>), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("stable launcher is not a regular file".into());
            }
            Ok((true, Some(digest_file(path)?)))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok((false, None)),
        Err(error) => Err(err(error)),
    }
}

fn copy_private_file(source: &Path, destination: &Path) -> Result<(), String> {
    if fs::symlink_metadata(destination).is_ok() {
        return Err("managed launcher backup already exists".into());
    }
    let result = (|| {
        let mut input = File::open(source).map_err(err)?;
        let mut output = create_file(destination)?;
        std::io::copy(&mut input, &mut output).map_err(err)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(destination, fs::Permissions::from_mode(0o700)).map_err(err)?;
        }
        output.sync_all().map_err(err)
    })();
    if result.is_err() {
        let _ = remove_owned_path(destination);
    }
    result
}

fn copy_and_replace(source: &Path, destination: &Path, temporary: &Path) -> Result<(), String> {
    if fs::symlink_metadata(temporary).is_ok() {
        return Err("managed launcher temporary already exists".into());
    }
    copy_private_file(source, temporary)?;
    let source_digest = digest_file(source)?;
    if digest_file(temporary)? != source_digest {
        let _ = fs::remove_file(temporary);
        return Err("launcher copy digest mismatch".into());
    }
    if let Err(error) = replace_path(temporary, destination) {
        let _ = fs::remove_file(temporary);
        return Err(error);
    }
    if digest_file(destination)? != source_digest {
        return Err("launcher replacement digest mismatch".into());
    }
    Ok(())
}

fn remove_owned_path(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("managed recovery path is not a regular file".into());
            }
            fs::remove_file(path).map_err(err)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(err(error)),
    }
}

fn digest_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(err)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(err)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!(
        "sha256:{}",
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ))
}

fn read_activation_journal(state: &Path) -> Result<Option<ActivationJournal>, String> {
    let path = state.join(ACTIVATION_JOURNAL);
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(err(error)),
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("activation journal is not a regular file".into());
            }
            let journal: ActivationJournal =
                serde_json::from_slice(&read_small(&path, 16 * 1024)?).map_err(err)?;
            if journal.schema_version != 1 {
                return Err("unsupported activation journal schema".into());
            }
            Ok(Some(journal))
        }
    }
}

fn write_activation_journal(state: &Path, journal: &ActivationJournal) -> Result<(), String> {
    let path = state.join(ACTIVATION_JOURNAL);
    let temp = state.join("activation.pending.json.next");
    let bytes = serde_json::to_vec(journal).map_err(err)?;
    if fs::symlink_metadata(&temp).is_ok() {
        if read_small(&temp, 16 * 1024)? != bytes {
            return Err(
                "activation journal temporary differs; preserve it for recovery inspection".into(),
            );
        }
        // An identical private temp is recoverable evidence from this exact
        // journal write; consume it before creating the next durable copy.
        remove_owned_path(&temp)?;
    }
    let result = (|| {
        let mut file = create_file(&temp)?;
        file.write_all(&bytes).map_err(err)?;
        file.sync_all().map_err(err)?;
        replace_path(&temp, &path)?;
        sync_dir(state)
    })();
    if result.is_err() {
        let _ = remove_owned_path(&temp);
    }
    result
}

fn replace_path(from: &Path, to: &Path) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        fs::rename(from, to).map_err(err)
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let from: Vec<u16> = from.as_os_str().encode_wide().chain([0]).collect();
        let to: Vec<u16> = to.as_os_str().encode_wide().chain([0]).collect();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            return Err(err(std::io::Error::last_os_error()));
        }
        Ok(())
    }
}

fn verify_lock_metadata(meta: &fs::Metadata) -> Result<(), String> {
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() != 0 {
        return Err("update lock must be an empty regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.mode() & 0o077 != 0 || meta.nlink() != 1 {
            return Err("update lock must be private and unaliased".into());
        }
    }
    Ok(())
}
fn read_small(path: &Path, max: usize) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(err)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > max as u64 {
        return Err("managed record is invalid or exceeds its limit".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(err)?
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    if bytes.len() > max {
        return Err("managed record grew beyond its limit".into());
    }
    Ok(bytes)
}
fn create_file(path: &Path) -> Result<File, String> {
    let mut o = OpenOptions::new();
    o.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    o.open(path).map_err(err)
}
fn private_directory(path: &Path) -> Result<(), String> {
    let mut b = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        b.mode(0o700);
    }
    b.create(path).map_err(err)
}
fn verify_private(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(err)?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Err("managed directory is not a real private directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if meta.mode() & 0o077 != 0 {
            return Err("managed directory must be private".into());
        }
    }
    Ok(())
}
fn ensure_parents(root: &Path, parent: &Path) -> Result<(), String> {
    let relative = parent.strip_prefix(root).map_err(err)?;
    let mut current = root.to_owned();
    for part in relative.components() {
        if !matches!(part, std::path::Component::Normal(_)) {
            return Err("invalid stage parent".into());
        }
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(_) => verify_private(&current)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => private_directory(&current)?,
            Err(e) => return Err(err(e)),
        }
    }
    Ok(())
}
fn sync_dir(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        File::open(path).and_then(|f| f.sync_all()).map_err(err)?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

// An observation under the single update writer lock, not a guarantee against
// unrelated disk consumers. Copy/sync failures preserve the active release.
fn available_bytes(path: &Path) -> Result<u64, String> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(err)?;
        let mut info = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: NUL-terminated stable path and valid writable statvfs storage.
        if unsafe { libc::statvfs(path.as_ptr(), info.as_mut_ptr()) } != 0 {
            return Err(err(std::io::Error::last_os_error()));
        }
        // SAFETY: statvfs succeeded and initialized the complete structure.
        let info = unsafe { info.assume_init() };
        u64::from(info.f_bavail)
            .checked_mul(info.f_frsize)
            .ok_or("free-space size overflow".into())
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut free = 0u64;
        // SAFETY: valid terminated UTF-16 and writable u64 result; unused outputs are nullable.
        if unsafe {
            windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        } == 0
        {
            return Err(err(std::io::Error::last_os_error()));
        }
        Ok(free)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Err("native free-space observation is unavailable".into())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "mirket-release-store-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            private_directory(&path).unwrap();
            Self(path.canonicalize().unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn payload(fixture: &Fixture, label: &str) -> (PathBuf, String) {
        let root = fixture.0.join(label);
        private_directory(&root).unwrap();
        let target = crate::distribution::native_target().unwrap();
        for relative in [
            "bin/mirket",
            "runtime/node/bin/node",
            "runtime/node/LICENSE",
        ] {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, format!("{label}:{relative}")).unwrap();
            let mode = if relative == "bin/mirket" || relative == "runtime/node/bin/node" {
                0o700
            } else {
                0o600
            };
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        }
        let id = crate::distribution::write_manifest(&root, "0.0.1", target).unwrap();
        (root, id.as_str().to_owned())
    }

    #[test]
    fn maintenance_active_inspection_preserves_lifecycle_refusals_and_guard_binding() {
        let fixture = Fixture::new();
        let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
        let (source, id) = payload(&fixture, "source");
        store.stage(&source).unwrap();
        store.activate().unwrap();
        let maintenance = MaintenanceGuard::try_acquire(&store.state).unwrap();
        assert_eq!(
            store
                .inspect_active_for_maintenance(&maintenance)
                .unwrap()
                .unwrap()
                .id()
                .as_str(),
            id
        );

        let other = ReleaseStore::initialize(&fixture.0.join("other-home")).unwrap();
        let other_guard = MaintenanceGuard::try_acquire(&other.state).unwrap();
        let error = store
            .inspect_active_for_maintenance(&other_guard)
            .err()
            .expect("another installation's guard must refuse");
        assert!(error.contains("another installation"), "{error}");
        assert!(
            other
                .inspect_active_for_maintenance(&other_guard)
                .unwrap()
                .is_none()
        );

        let active_before = fs::read(store.home.join("active.json")).unwrap();
        for name in [
            "uninstall.json",
            "uninstall.json.next",
            "reinstall.json",
            "reinstall.json.next",
        ] {
            let path = store.state.join(name);
            let mut file = create_file(&path).unwrap();
            file.write_all(b"preserved pending lifecycle").unwrap();
            drop(file);
            let error = store
                .inspect_active_for_maintenance(&maintenance)
                .err()
                .expect("pending lifecycle record must refuse");
            assert!(
                error.contains("pending installation lifecycle record"),
                "{error}"
            );
            assert_eq!(fs::read(&path).unwrap(), b"preserved pending lifecycle");
            assert_eq!(
                fs::read(store.home.join("active.json")).unwrap(),
                active_before
            );
            fs::remove_file(path).unwrap();
        }
    }

    fn interrupted_stage(
        store: &ReleaseStore,
        root: &Path,
        index: usize,
        prefix: usize,
    ) -> (PathBuf, StagingJournal, Vec<StageFile>) {
        let source = VerifiedRelease::open(root).unwrap();
        let manifest = serde_json::to_vec_pretty(source.manifest()).unwrap();
        let files = stage_files(&source, &manifest);
        let stage = store.home.join("staging").join(format!(
            "r-{}",
            source.id().as_str().trim_start_matches("sha256:")
        ));
        private_directory(&stage).unwrap();
        for directory in source.directories() {
            ensure_parents(&stage, &stage.join(directory)).unwrap();
        }
        for (current, entry) in files.iter().enumerate().take(index + 1) {
            let path = stage.join(&entry.relative_path);
            let mut output = create_file(&path).unwrap();
            let mut input = stage_input(&source, &manifest, entry).unwrap();
            if current < index {
                std::io::copy(&mut input, &mut output).unwrap();
                set_stage_mode(&path, entry.mode).unwrap();
            } else {
                std::io::copy(&mut input.take(prefix as u64), &mut output).unwrap();
            }
            output.sync_all().unwrap();
        }
        let journal = StagingJournal {
            schema_version: 1,
            candidate_release_id: source.id().as_str().into(),
            directory_identity: stage_identity(&fs::symlink_metadata(&stage).unwrap()),
            revision: index as u64 + 1,
            phase: StagingPhase::Copying {
                index,
                file_identity: stage_identity(
                    &fs::symlink_metadata(stage.join(&files[index].relative_path)).unwrap(),
                ),
            },
        };
        write_staging_journal(&store.state, &journal).unwrap();
        (stage, journal, files)
    }

    #[test]
    fn same_candidate_retry_reuses_completed_files_and_repairs_only_bound_partial() {
        use std::os::unix::fs::MetadataExt;
        let fixture = Fixture::new();
        let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
        let (source, id) = payload(&fixture, "source");
        let (stage, _, files) = interrupted_stage(&store, &source, 1, 3);
        let completed = fs::symlink_metadata(stage.join(&files[0].relative_path))
            .unwrap()
            .ino();
        let partial = fs::symlink_metadata(stage.join(&files[1].relative_path))
            .unwrap()
            .ino();
        let result = store.stage(&source).unwrap();
        assert_eq!(result.release_id, id);
        let target = store.release_path(&id).unwrap();
        assert_eq!(
            fs::symlink_metadata(target.join(&files[0].relative_path))
                .unwrap()
                .ino(),
            completed
        );
        assert_eq!(
            fs::symlink_metadata(target.join(&files[1].relative_path))
                .unwrap()
                .ino(),
            partial
        );
        assert_eq!(
            VerifiedRelease::open_installed(&target)
                .unwrap()
                .id()
                .as_str(),
            id
        );
        assert!(!stage.exists());
        assert!(!store.state.join(STAGING_JOURNAL).exists());
        assert!(!store.state.join(STAGING_JOURNAL_NEXT).exists());
        assert!(store.read_activation("active.json").unwrap().is_none());
    }

    #[test]
    fn staging_and_recovery_preserve_bytes_across_copy_buffer_boundaries() {
        use std::os::unix::fs::MetadataExt;
        let fixture = Fixture::new();
        let (source, _) = payload(&fixture, "source");
        let expected: Vec<u8> = (0..(2 * 65_536 + 31))
            .map(|index| ((index * 37) % 251) as u8)
            .collect();
        fs::write(source.join("runtime/node/LICENSE"), &expected).unwrap();
        fs::remove_file(source.join(MANIFEST_NAME)).unwrap();
        let id = crate::distribution::write_manifest(
            &source,
            "0.0.1",
            crate::distribution::native_target().unwrap(),
        )
        .unwrap();
        for (case, prefix) in [None, Some(65_535), Some(65_536), Some(65_537)]
            .into_iter()
            .enumerate()
        {
            let store = ReleaseStore::initialize(&fixture.0.join(format!("home-{case}"))).unwrap();
            let partial_inode = prefix.map(|prefix| {
                let (stage, _, files) = interrupted_stage(&store, &source, 1, prefix);
                assert_eq!(files[1].relative_path, "runtime/node/LICENSE");
                fs::metadata(stage.join(&files[1].relative_path))
                    .unwrap()
                    .ino()
            });
            let result = store.stage(&source).unwrap();
            assert_eq!(result.release_id, id.as_str());
            let saved = store.release_path(id.as_str()).unwrap();
            let license = saved.join("runtime/node/LICENSE");
            assert_eq!(fs::read(&license).unwrap(), expected, "prefix {prefix:?}");
            if let Some(inode) = partial_inode {
                assert_eq!(fs::metadata(&license).unwrap().ino(), inode);
            }
            assert_eq!(
                VerifiedRelease::open_installed(&saved)
                    .unwrap()
                    .id()
                    .as_str(),
                id.as_str()
            );
        }
    }

    #[test]
    fn recovered_staging_preserves_active_previous_and_live_session_exclusion() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (previous, _) = payload(&fixture, "previous");
        let (active, active_id) = payload(&fixture, "active");
        let (candidate, candidate_id) = payload(&fixture, "candidate");
        let launcher = home.join("bin/mirket");
        for source in [&previous, &active] {
            store.stage(source).unwrap();
            store.activate_with_launcher(&launcher).unwrap();
        }
        let active_record = fs::read(home.join("active.json")).unwrap();
        let launcher_bytes = fs::read(&launcher).unwrap();
        let session = store.open_active().unwrap();
        assert_eq!(session.release.id().as_str(), active_id);
        interrupted_stage(&store, &candidate, 1, 2);
        store.stage(&candidate).unwrap();
        assert_eq!(fs::read(home.join("active.json")).unwrap(), active_record);
        assert_eq!(fs::read(&launcher).unwrap(), launcher_bytes);
        let busy = store.activate_with_launcher(&launcher).unwrap();
        assert_eq!(busy.status, "staged");
        assert_eq!(busy.release_id, candidate_id);
        assert_eq!(fs::read(home.join("active.json")).unwrap(), active_record);
        assert_eq!(fs::read(&launcher).unwrap(), launcher_bytes);
        assert_eq!(store.release_inventory().unwrap().len(), 3);
        drop(session);
        assert_eq!(
            store.activate_with_launcher(&launcher).unwrap().status,
            "active"
        );
    }

    #[test]
    fn bound_catalog_session_excludes_activation_but_does_not_trust_payload() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (source, id) = payload(&fixture, "source");
        store.stage(&source).unwrap();
        assert_eq!(
            store
                .activate_with_launcher(&home.join("bin/mirket"))
                .unwrap()
                .status,
            "active"
        );
        let bound = store.bind_active().unwrap();
        assert_eq!(bound.release_id, id);
        assert!(crate::maintenance::MaintenanceGuard::try_acquire(&store.state).is_err());
        fs::write(bound.release_path.join("runtime/node/LICENSE"), b"changed").unwrap();
        assert!(VerifiedRelease::open_installed(&bound.release_path).is_err());
        drop(bound);
        assert!(crate::maintenance::MaintenanceGuard::try_acquire(&store.state).is_ok());
    }

    #[test]
    fn interrupted_stage_refuses_corruption_unknown_entries_and_file_substitution() {
        for case in [
            "completed",
            "partial",
            "unknown",
            "backslash",
            "inode",
            "alias",
            "missing",
            "mode",
        ] {
            let fixture = Fixture::new();
            let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
            let (source, _) = payload(&fixture, "source");
            let (stage, _, files) = interrupted_stage(&store, &source, 1, 3);
            let completed = stage.join(&files[0].relative_path);
            let partial = stage.join(&files[1].relative_path);
            let watched = match case {
                "completed" => {
                    fs::write(&completed, "corrupt").unwrap();
                    completed
                }
                "partial" => {
                    fs::write(&partial, "bad").unwrap();
                    partial.clone()
                }
                "unknown" => {
                    let path = stage.join("user-data");
                    fs::write(&path, "preserve").unwrap();
                    path
                }
                "backslash" => {
                    let path = stage.join(files[0].relative_path.replace('/', "\\"));
                    fs::write(&path, fs::read(&completed).unwrap()).unwrap();
                    fs::set_permissions(&path, fs::Permissions::from_mode(files[0].mode)).unwrap();
                    path
                }
                "inode" => {
                    let bytes = fs::read(&partial).unwrap();
                    fs::rename(&partial, fixture.0.join("old-inode")).unwrap();
                    fs::write(&partial, bytes).unwrap();
                    fs::set_permissions(&partial, fs::Permissions::from_mode(0o600)).unwrap();
                    partial.clone()
                }
                "alias" => {
                    fs::hard_link(&partial, fixture.0.join("alias")).unwrap();
                    partial.clone()
                }
                "missing" => {
                    fs::remove_file(&partial).unwrap();
                    completed
                }
                "mode" => {
                    fs::set_permissions(&partial, fs::Permissions::from_mode(0o644)).unwrap();
                    partial.clone()
                }
                _ => unreachable!(),
            };
            let before = fs::read(&watched).unwrap();
            let journal = fs::read(store.state.join(STAGING_JOURNAL)).unwrap();
            assert!(store.stage(&source).is_err(), "{case}");
            assert_eq!(fs::read(&watched).unwrap(), before, "{case}");
            assert_eq!(
                fs::read(store.state.join(STAGING_JOURNAL)).unwrap(),
                journal,
                "{case}"
            );
            assert!(store.read_activation("staged.json").unwrap().is_none());
        }
    }

    #[test]
    fn different_candidate_and_unjournaled_stage_are_preserved() {
        let fixture = Fixture::new();
        let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
        let (source, _) = payload(&fixture, "source");
        let (other, _) = payload(&fixture, "other");
        let (stage, _, files) = interrupted_stage(&store, &source, 0, 2);
        let partial = stage.join(&files[0].relative_path);
        let before = fs::read(&partial).unwrap();
        assert!(
            store
                .stage(&other)
                .err()
                .unwrap()
                .contains("different candidate")
        );
        fs::remove_file(store.state.join(STAGING_JOURNAL)).unwrap();
        assert!(
            store
                .stage(&source)
                .err()
                .unwrap()
                .contains("no ownership journal")
        );
        assert_eq!(fs::read(partial).unwrap(), before);
    }

    #[test]
    fn complete_initial_preparing_temporary_resumes_but_changed_stage_root_refuses() {
        for changed_root in [false, true] {
            let fixture = Fixture::new();
            let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
            let (source, id) = payload(&fixture, "source");
            let stage = store
                .home
                .join("staging")
                .join(format!("r-{}", id.trim_start_matches("sha256:")));
            private_directory(&stage).unwrap();
            let journal = StagingJournal {
                schema_version: 1,
                candidate_release_id: id,
                directory_identity: stage_identity(&fs::symlink_metadata(&stage).unwrap()),
                revision: 0,
                phase: StagingPhase::Preparing,
            };
            let mut temporary = create_file(&store.state.join(STAGING_JOURNAL_NEXT)).unwrap();
            temporary
                .write_all(&serde_json::to_vec(&journal).unwrap())
                .unwrap();
            temporary.sync_all().unwrap();
            if changed_root {
                fs::rename(&stage, fixture.0.join("old-stage")).unwrap();
                private_directory(&stage).unwrap();
                assert!(
                    store
                        .stage(&source)
                        .err()
                        .unwrap()
                        .contains("identity drifted")
                );
                assert!(store.state.join(STAGING_JOURNAL_NEXT).exists());
                assert!(fs::read_dir(stage).unwrap().next().is_none());
            } else {
                store.stage(&source).unwrap();
                assert!(!store.state.join(STAGING_JOURNAL_NEXT).exists());
            }
        }
    }

    #[test]
    fn ready_stage_and_published_payload_resume_without_activating() {
        for published in [false, true] {
            let fixture = Fixture::new();
            let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
            let (source, id) = payload(&fixture, "source");
            store.stage(&source).unwrap();
            let target = store.release_path(&id).unwrap();
            let stage = store.home.join("staging").join(target.file_name().unwrap());
            let selected = VerifiedRelease::open(&source).unwrap();
            let manifest = serde_json::to_vec_pretty(selected.manifest()).unwrap();
            let journal = StagingJournal {
                schema_version: 1,
                candidate_release_id: id.clone(),
                directory_identity: stage_identity(&fs::symlink_metadata(&target).unwrap()),
                revision: stage_files(&selected, &manifest).len() as u64 + 2,
                phase: StagingPhase::Ready,
            };
            if !published {
                fs::rename(&target, &stage).unwrap();
            }
            fs::remove_file(store.home.join("staged.json")).unwrap();
            write_staging_journal(&store.state, &journal).unwrap();
            store.stage(&source).unwrap();
            assert_eq!(
                store
                    .read_activation("staged.json")
                    .unwrap()
                    .unwrap()
                    .release_id,
                id
            );
            assert!(store.read_activation("active.json").unwrap().is_none());
            assert!(!store.state.join(STAGING_JOURNAL).exists());
        }
    }

    #[test]
    fn full_current_file_is_hashed_and_complete_successor_can_be_consumed() {
        let fixture = Fixture::new();
        let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
        let (source, _) = payload(&fixture, "source");
        let (stage, mut journal, files) = interrupted_stage(&store, &source, 0, usize::MAX);
        let path = stage.join(&files[0].relative_path);
        let original = fs::read(&path).unwrap();
        fs::write(&path, vec![b'x'; original.len()]).unwrap();
        assert!(store.stage(&source).is_err());
        assert_eq!(fs::read(&path).unwrap(), vec![b'x'; original.len()]);
        fs::write(&path, original).unwrap();
        set_stage_mode(&path, files[0].mode).unwrap();
        let next_path = stage.join(&files[1].relative_path);
        let mut file = create_file(&next_path).unwrap();
        file.write_all(b"sou").unwrap();
        file.sync_all().unwrap();
        journal.revision += 1;
        journal.phase = StagingPhase::Copying {
            index: 1,
            file_identity: stage_identity(&file.metadata().unwrap()),
        };
        let mut next = create_file(&store.state.join(STAGING_JOURNAL_NEXT)).unwrap();
        next.write_all(&serde_json::to_vec(&journal).unwrap())
            .unwrap();
        next.sync_all().unwrap();
        store.stage(&source).unwrap();
        assert!(!store.state.join(STAGING_JOURNAL_NEXT).exists());
    }

    #[test]
    fn torn_or_unrelated_successor_journal_and_unbound_file_remain_untouched() {
        for case in ["torn", "revision", "unbound"] {
            let fixture = Fixture::new();
            let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
            let (source, _) = payload(&fixture, "source");
            let (stage, mut journal, files) = interrupted_stage(&store, &source, 0, 2);
            let watched = if case == "unbound" {
                let path = stage.join(&files[1].relative_path);
                fs::write(&path, "unbound user bytes").unwrap();
                path
            } else {
                let path = store.state.join(STAGING_JOURNAL_NEXT);
                let bytes = if case == "torn" {
                    b"{\"schema_version\":".to_vec()
                } else {
                    journal.revision += 3;
                    serde_json::to_vec(&journal).unwrap()
                };
                let mut file = create_file(&path).unwrap();
                file.write_all(&bytes).unwrap();
                path
            };
            let before = fs::read(&watched).unwrap();
            assert!(store.stage(&source).is_err(), "{case}");
            assert_eq!(fs::read(&watched).unwrap(), before);
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn staging_normalizes_source_modes_and_rechecks_installed_mode_drift() {
        let fixture = Fixture::new();
        let store = ReleaseStore::initialize(&fixture.0.join("home")).unwrap();
        let (source, id) = payload(&fixture, "source");
        let verified = VerifiedRelease::open(&source).unwrap();
        for entry in &verified.manifest().files {
            let source_mode = if entry.mode == 0o700 { 0o775 } else { 0o664 };
            fs::set_permissions(
                source.join(&entry.relative_path),
                fs::Permissions::from_mode(source_mode),
            )
            .unwrap();
        }
        store.stage(&source).unwrap();
        let installed = store.release_path(&id).unwrap();
        let saved = VerifiedRelease::open_installed(&installed).unwrap();
        assert_eq!(saved.id().as_str(), id);
        for entry in &saved.manifest().files {
            assert_eq!(
                fs::metadata(installed.join(&entry.relative_path))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o7777,
                entry.mode
            );
        }
        let manifest = installed.join(MANIFEST_NAME);
        fs::set_permissions(&manifest, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(store.activate().is_err());
        assert!(store.read_activation("active.json").unwrap().is_none());
        fs::set_permissions(manifest, fs::Permissions::from_mode(0o600)).unwrap();
        store.activate().unwrap();
        fs::set_permissions(
            installed.join("bin/mirket"),
            fs::Permissions::from_mode(0o775),
        )
        .unwrap();
        assert!(store.open_active().is_err());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn successive_updates_keep_only_active_and_previous_and_preserve_unknown_stage() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (a, a_id) = payload(&fixture, "a");
        let (b, b_id) = payload(&fixture, "b");
        let (c, c_id) = payload(&fixture, "c");
        for source in [&a, &b, &c] {
            store.stage(source).unwrap();
            let result = store.activate().unwrap();
            assert_eq!(result.status, "active");
            assert!(result.reason.is_none());
        }
        let active = store.read_activation("active.json").unwrap().unwrap();
        assert_eq!(active.release_id, c_id);
        assert_eq!(active.previous_release_id.as_deref(), Some(b_id.as_str()));
        assert!(!store.release_path(&a_id).unwrap().exists());
        assert_eq!(store.release_inventory().unwrap().len(), 2);
        let partial = home.join("staging/unknown");
        fs::write(&partial, "preserve interrupted evidence").unwrap();
        assert!(
            store
                .stage(&a)
                .err()
                .unwrap()
                .contains("interrupted staging")
        );
        assert_eq!(fs::read(partial).unwrap(), b"preserve interrupted evidence");
        assert_eq!(
            store
                .read_activation("active.json")
                .unwrap()
                .unwrap()
                .release_id,
            c_id
        );
    }

    #[test]
    fn activation_with_launcher_commits_the_active_pair_and_cleans_journal() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (source, id) = payload(&fixture, "source");
        store.stage(&source).unwrap();

        let launcher = home.join("bin/mirket");
        let result = store.activate_with_launcher(&launcher).unwrap();
        assert_eq!(result.status, "active");
        assert_eq!(result.release_id, id);
        assert!(result.reason.is_none());
        assert_eq!(
            fs::read(&launcher).unwrap(),
            fs::read(source.join("bin/mirket")).unwrap()
        );
        assert_eq!(
            store
                .read_activation("active.json")
                .unwrap()
                .unwrap()
                .release_id,
            id
        );
        assert!(!home.join("staged.json").exists());
        assert!(!home.join("state/activation.pending.json").exists());
        assert!(!home.join("state/launcher.previous").exists());
    }

    #[test]
    fn repeating_active_release_preserves_its_retained_predecessor() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (first, first_id) = payload(&fixture, "first");
        let (second, second_id) = payload(&fixture, "second");
        let launcher = home.join("bin/mirket");
        for source in [&first, &second] {
            store.stage(source).unwrap();
            store.activate_with_launcher(&launcher).unwrap();
        }
        let active_before = fs::read(home.join("active.json")).unwrap();
        let predecessor = store.release_path(&first_id).unwrap();
        let predecessor_manifest = fs::read(predecessor.join(MANIFEST_NAME)).unwrap();

        store.stage(&second).unwrap();
        let result = store.activate_with_launcher(&launcher).unwrap();

        assert_eq!(result.status, "active");
        assert_eq!(result.release_id, second_id);
        assert!(result.reason.is_none());
        assert_eq!(fs::read(home.join("active.json")).unwrap(), active_before);
        assert_eq!(
            fs::read(predecessor.join(MANIFEST_NAME)).unwrap(),
            predecessor_manifest
        );
        assert_eq!(
            store
                .release_inventory()
                .unwrap()
                .into_iter()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([first_id, second_id])
        );
        assert_eq!(
            fs::read(&launcher).unwrap(),
            fs::read(second.join("bin/mirket")).unwrap()
        );
        assert!(!home.join("staged.json").exists());
        assert!(!home.join("state/activation.pending.json").exists());
    }

    #[test]
    fn pending_activation_refuses_a_different_candidate_before_mutation() {
        for phase in ["stage", "activate"] {
            let fixture = Fixture::new();
            let home = fixture.0.join("home");
            let store = ReleaseStore::initialize(&home).unwrap();
            let (first, first_id) = payload(&fixture, "first");
            let (second, second_id) = payload(&fixture, "second");
            let (third, third_id) = payload(&fixture, "third");
            let launcher = home.join("bin/mirket");
            for source in [&first, &second] {
                store.stage(source).unwrap();
                store.activate_with_launcher(&launcher).unwrap();
            }
            // The activate case also covers a pre-existing conflicting staged
            // intent, such as one admitted by an older core before this guard.
            if phase == "activate" {
                store.stage(&third).unwrap();
            }
            let backup = home.join("state/launcher.previous");
            let temporary = home.join("bin/.mirket-launcher-next-candidate-test");
            copy_private_file(&first.join("bin/mirket"), &backup).unwrap();
            write_activation_journal(
                &store.state,
                &ActivationJournal {
                    schema_version: 1,
                    candidate_release_id: second_id.clone(),
                    previous_release_id: Some(first_id.clone()),
                    launcher_path: launcher.to_string_lossy().into_owned(),
                    backup_path: backup.to_string_lossy().into_owned(),
                    temporary_path: temporary.to_string_lossy().into_owned(),
                    old_launcher_present: true,
                    old_launcher_digest: Some(digest_file(&backup).unwrap()),
                    candidate_launcher_digest: digest_file(&launcher).unwrap(),
                    phase: ActivationPhase::ActiveCommitted,
                },
            )
            .unwrap();
            let active_before = fs::read(home.join("active.json")).unwrap();
            let staged_before = fs::read(home.join("staged.json")).ok();
            let journal_path = store.state.join(ACTIVATION_JOURNAL);
            let journal_before = fs::read(&journal_path).unwrap();
            let launcher_before = fs::read(&launcher).unwrap();
            let backup_before = fs::read(&backup).unwrap();
            let releases_before = store
                .release_inventory()
                .unwrap()
                .into_iter()
                .collect::<BTreeSet<_>>();
            let candidate = store.release_path(&third_id).unwrap();
            let candidate_before = fs::read(candidate.join(MANIFEST_NAME)).ok();

            let error = if phase == "stage" {
                store.stage(&third)
            } else {
                store.activate_with_launcher(&launcher)
            }
            .err()
            .expect("a different pending activation candidate must refuse");

            assert!(error.contains("different"), "{phase}: {error}");
            assert_eq!(fs::read(home.join("active.json")).unwrap(), active_before);
            assert_eq!(fs::read(home.join("staged.json")).ok(), staged_before);
            assert_eq!(fs::read(&journal_path).unwrap(), journal_before);
            assert_eq!(fs::read(&launcher).unwrap(), launcher_before);
            assert_eq!(fs::read(&backup).unwrap(), backup_before);
            assert_eq!(
                store
                    .release_inventory()
                    .unwrap()
                    .into_iter()
                    .collect::<BTreeSet<_>>(),
                releases_before
            );
            assert_eq!(
                fs::read(candidate.join(MANIFEST_NAME)).ok(),
                candidate_before
            );
            assert!(fs::read_dir(home.join("staging")).unwrap().next().is_none());

            if phase == "stage" {
                // An explicit retry of the original candidate remains valid.
                store.stage(&second).unwrap();
                let result = store.activate_with_launcher(&launcher).unwrap();
                assert_eq!(result.status, "active");
                assert_eq!(result.release_id, second_id);
                assert!(!journal_path.exists());
                assert!(!home.join("staged.json").exists());
                assert!(store.release_path(&first_id).unwrap().exists());
                assert!(!candidate.exists());
            }
        }
    }

    #[test]
    fn interrupted_launcher_replacement_recovers_old_pair_before_retrying_candidate() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (first, first_id) = payload(&fixture, "first");
        let (second, second_id) = payload(&fixture, "second");
        let launcher = home.join("bin/mirket");
        store.stage(&first).unwrap();
        store.activate_with_launcher(&launcher).unwrap();
        let old_launcher = fs::read(&launcher).unwrap();
        store.stage(&second).unwrap();

        let backup = home.join("state/launcher.previous");
        let temporary = home.join("bin/.mirket-launcher-next-recovery-test");
        copy_private_file(&launcher, &backup).unwrap();
        let candidate = store.release_path(&second_id).unwrap().join("bin/mirket");
        copy_and_replace(&candidate, &launcher, &temporary).unwrap();
        let journal = ActivationJournal {
            schema_version: 1,
            candidate_release_id: second_id.clone(),
            previous_release_id: Some(first_id.clone()),
            launcher_path: launcher.to_string_lossy().into_owned(),
            backup_path: backup.to_string_lossy().into_owned(),
            temporary_path: temporary.to_string_lossy().into_owned(),
            old_launcher_present: true,
            old_launcher_digest: Some(digest_file(&backup).unwrap()),
            candidate_launcher_digest: digest_file(&candidate).unwrap(),
            phase: ActivationPhase::LauncherReplaced,
        };
        write_activation_journal(&home.join("state"), &journal).unwrap();

        let result = store.activate_with_launcher(&launcher).unwrap();
        assert_eq!(result.status, "active");
        assert_eq!(result.release_id, second_id);
        assert_eq!(
            fs::read(&launcher).unwrap(),
            fs::read(fixture.0.join("second/bin/mirket")).unwrap()
        );
        assert_ne!(fs::read(&launcher).unwrap(), old_launcher);
        assert!(!home.join("state/activation.pending.json").exists());
        assert!(!home.join("state/launcher.previous").exists());
    }

    #[test]
    fn committed_activation_journal_recovers_without_a_staged_record() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (first, first_id) = payload(&fixture, "first");
        let (second, second_id) = payload(&fixture, "second");
        let launcher = home.join("bin/mirket");
        store.stage(&first).unwrap();
        store.activate_with_launcher(&launcher).unwrap();
        store.stage(&second).unwrap();

        let backup = home.join("state/launcher.previous");
        let temporary = home.join("bin/.mirket-launcher-next-committed-test");
        copy_private_file(&launcher, &backup).unwrap();
        let candidate = store.release_path(&second_id).unwrap().join("bin/mirket");
        copy_and_replace(&candidate, &launcher, &temporary).unwrap();
        store
            .write_activation(
                "active.json",
                &Activation {
                    schema_version: 1,
                    release_id: second_id.clone(),
                    previous_release_id: Some(first_id.clone()),
                },
            )
            .unwrap();
        fs::remove_file(home.join("staged.json")).unwrap();
        write_activation_journal(
            &home.join("state"),
            &ActivationJournal {
                schema_version: 1,
                candidate_release_id: second_id.clone(),
                previous_release_id: Some(first_id),
                launcher_path: launcher.to_string_lossy().into_owned(),
                backup_path: backup.to_string_lossy().into_owned(),
                temporary_path: temporary.to_string_lossy().into_owned(),
                old_launcher_present: true,
                old_launcher_digest: Some(digest_file(&backup).unwrap()),
                candidate_launcher_digest: digest_file(&candidate).unwrap(),
                phase: ActivationPhase::ActiveCommitted,
            },
        )
        .unwrap();
        let installed = store.release_path(&second_id).unwrap();
        let license = installed.join("runtime/node/LICENSE");
        let manifest_path = installed.join(MANIFEST_NAME);
        let original_license = fs::read(&license).unwrap();
        let original_manifest = fs::read(&manifest_path).unwrap();
        let changed_license = b"changed license after active pointer";
        fs::write(&license, changed_license).unwrap();
        let mut changed_manifest: serde_json::Value =
            serde_json::from_slice(&original_manifest).unwrap();
        let entry = changed_manifest["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["relative_path"] == "runtime/node/LICENSE")
            .unwrap();
        entry["byte_len"] = serde_json::json!(changed_license.len());
        entry["sha256"] = serde_json::json!(
            Sha256::digest(changed_license)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        fs::write(
            &manifest_path,
            serde_json::to_vec(&changed_manifest).unwrap(),
        )
        .unwrap();
        assert_ne!(
            VerifiedRelease::open_installed(&installed)
                .unwrap()
                .id()
                .as_str(),
            second_id
        );
        let launcher_bytes = fs::read(&launcher).unwrap();
        let error = store.activate_with_launcher(&launcher).err().unwrap();
        assert!(
            error.contains("candidate identity no longer verifies"),
            "{error}"
        );
        assert_eq!(fs::read(&launcher).unwrap(), launcher_bytes);
        assert!(home.join("state/activation.pending.json").exists());
        fs::write(&license, original_license).unwrap();
        fs::write(&manifest_path, original_manifest).unwrap();
        let result = store.activate_with_launcher(&launcher).unwrap();
        assert_eq!(result.status, "active");
        assert_eq!(result.release_id, second_id);
        assert!(!home.join("state/activation.pending.json").exists());
        assert!(!home.join("state/launcher.previous").exists());
    }

    #[test]
    fn busy_activation_keeps_candidate_staged_and_active_pair_unchanged() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (source, id) = payload(&fixture, "source");
        store.stage(&source).unwrap();
        let session = SessionGuard::acquire(&home.join("state")).unwrap();

        let launcher = home.join("bin/mirket");
        let result = store.activate_with_launcher(&launcher).unwrap();
        assert_eq!(result.status, "staged");
        assert_eq!(result.release_id, id);
        assert!(result.reason.is_some());
        assert!(!home.join("active.json").exists());
        assert!(!launcher.exists());
        assert!(home.join("staged.json").exists());
        drop(session);
    }

    #[test]
    fn preexisting_launcher_backup_refuses_update_without_changing_active_pair() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let store = ReleaseStore::initialize(&home).unwrap();
        let (first, first_id) = payload(&fixture, "first");
        let (second, second_id) = payload(&fixture, "second");
        store.stage(&first).unwrap();
        let launcher = home.join("bin/mirket");
        store.activate_with_launcher(&launcher).unwrap();
        let old_launcher = fs::read(&launcher).unwrap();
        store.stage(&second).unwrap();

        private_directory(&home.join("state/launcher.previous")).unwrap();
        let error = store.activate_with_launcher(&launcher).err().unwrap();
        assert!(error.contains("interrupted launcher transaction"));
        assert_eq!(
            store
                .read_activation("active.json")
                .unwrap()
                .unwrap()
                .release_id,
            first_id
        );
        assert_eq!(fs::read(&launcher).unwrap(), old_launcher);
        assert_eq!(
            store
                .read_activation("staged.json")
                .unwrap()
                .unwrap()
                .release_id,
            second_id
        );
        fs::remove_dir(home.join("state/launcher.previous")).unwrap();
    }

    #[test]
    fn lock_alias_and_contents_are_refused_without_modification() {
        let fixture = Fixture::new();
        let store = ReleaseStore::initialize(&fixture.0).unwrap();
        let path = fixture.0.join("update.lock");
        drop(store.mutation_lock().unwrap());
        let held = store.mutation_lock().unwrap();
        assert!(store.mutation_lock().is_err());
        drop(held);
        let alias = fixture.0.join("lock-alias");
        fs::hard_link(&path, &alias).unwrap();
        assert!(store.mutation_lock().unwrap_err().contains("unaliased"));
        fs::remove_file(alias).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(store.mutation_lock().unwrap_err().contains("private"));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, "unrelated content").unwrap();
        assert!(store.mutation_lock().is_err());
        assert_eq!(fs::read(&path).unwrap(), b"unrelated content");
        fs::remove_file(&path).unwrap();
        symlink(fixture.0.join("missing"), &path).unwrap();
        assert!(store.mutation_lock().is_err());
        assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    }

    #[test]
    fn initialization_preserves_unrecognized_nonempty_home() {
        let fixture = Fixture::new();
        fs::write(fixture.0.join("user-file"), "preserved").unwrap();
        assert!(ReleaseStore::initialize(&fixture.0).is_err());
        assert!(!fixture.0.join("installation.json").exists());
        assert_eq!(fs::read(fixture.0.join("user-file")).unwrap(), b"preserved");
    }
}
