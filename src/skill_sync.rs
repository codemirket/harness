//! Receipt-owned copies of release skills in explicit Codex native scopes.
//!
//! Planning and inspection do not write. A project opt-in is a separate action;
//! profiles explain a pinned selection and are never re-resolved during sync.
//! The transaction rejects detected symlinks and concurrent byte changes. As
//! with other path-based owners, this is not containment against a hostile
//! process replacing ancestors between native filesystem calls.

use crate::{file_lock::FileLock, native_skills::QualifiedCatalog};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PACKAGE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PACKAGE_ENTRIES: usize = 256;
const MAX_STATE_BYTES: u64 = 2 * 1024 * 1024;

/// Capability IDs are opaque catalog values. The current release uses browser,
/// navigation.js-ts, design.tokens, design.openpencil,
/// design.vector, design.raster, design.diagrams, and video. Only the caller's
/// verified qualification set admits them; a catalog entry is not readiness.
#[derive(Clone, Debug)]
pub struct SkillSync {
    catalog: QualifiedCatalog,
    scope: &'static str,
    root: PathBuf,
    destination: PathBuf,
    receipt_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSelection {
    pub schema_version: u8,
    pub profiles: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub selected: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionWriteOutcome {
    Unchanged,
    Replaced,
}

#[derive(Debug)]
pub struct SelectionWriteError {
    pub message: String,
    /// True only after this operation replaced the selection file.
    pub replaced: bool,
}

impl From<String> for SelectionWriteError {
    fn from(message: String) -> Self {
        Self {
            message,
            replaced: false,
        }
    }
}

impl From<&str> for SelectionWriteError {
    fn from(message: &str) -> Self {
        Self::from(message.to_owned())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SkillChange {
    pub id: String,
    pub action: &'static str,
    pub destination: PathBuf,
    pub source_sha256: Option<String>,
    pub previous_sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnavailableSkill {
    pub id: String,
    pub missing_capabilities: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SkillPlan {
    pub scope: &'static str,
    pub destination: PathBuf,
    pub receipt_path: PathBuf,
    pub release_id: String,
    pub selected: Vec<String>,
    pub changes: Vec<SkillChange>,
    pub unavailable: Vec<UnavailableSkill>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SkillSyncReport {
    pub status: &'static str,
    pub plan: SkillPlan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ManagedSkillStatus {
    pub id: String,
    pub status: &'static str,
    pub source_sha256: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SkillInspection {
    pub scope: &'static str,
    pub destination: PathBuf,
    pub status: &'static str,
    pub release_id: Option<String>,
    pub pending_transaction: bool,
    pub skills: Vec<ManagedSkillStatus>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRecord {
    bytes: u64,
    sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageRecord {
    source_sha256: String,
    directories: BTreeSet<String>,
    files: BTreeMap<String, FileRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u8,
    scope: String,
    destination: PathBuf,
    release_id: String,
    skills: BTreeMap<String, PackageRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
enum ReceiptFile {
    Committed {
        receipt: Receipt,
    },
    Pending {
        before: Option<Receipt>,
        after: Receipt,
    },
}

struct Package {
    record: PackageRecord,
    bytes: BTreeMap<String, Vec<u8>>,
}

struct Prepared {
    plan: SkillPlan,
    before: Option<Receipt>,
    after: Receipt,
    packages: BTreeMap<String, Package>,
}

impl ProjectSelection {
    pub fn resolve(
        catalog: &QualifiedCatalog,
        profiles: &[String],
        include: &[String],
        exclude: &[String],
    ) -> Result<Self, String> {
        let profiles = unique(profiles, "profile")?;
        let include = unique(include, "included skill")?;
        let exclude = unique(exclude, "excluded skill")?;
        if include.iter().any(|id| exclude.contains(id)) {
            return Err("a skill cannot be explicitly included and excluded".into());
        }
        let mut selected = BTreeSet::new();
        for name in &profiles {
            let profile = catalog
                .catalog
                .profiles
                .get(name)
                .ok_or_else(|| format!("unknown skill profile: {name}"))?;
            selected.extend(profile.skills.iter().cloned());
        }
        selected.extend(include.iter().cloned());
        selected.retain(|id| !exclude.contains(id));
        let selection = Self {
            schema_version: 1,
            profiles,
            include,
            exclude,
            selected: selected.into_iter().collect(),
        };
        selection.validate(catalog)?;
        Ok(selection)
    }

    /// Read a pinned opt-in without creating `.mirket` or any lock.
    pub fn load(project_root: &Path) -> Result<Option<Self>, String> {
        let Some(bytes) = Self::saved_bytes(project_root)? else {
            return Ok(None);
        };
        let selection: Self = serde_json::from_slice(&bytes).map_err(str_error)?;
        selection.validate_shape()?;
        Ok(Some(selection))
    }

    /// Capture exact bounded selection bytes without creating state or a lock.
    /// Callers must still validate the selection before relying on its IDs.
    pub fn saved_bytes(project_root: &Path) -> Result<Option<Vec<u8>>, String> {
        let root = select_root(project_root)?;
        let path = root.join(".mirket/project.json");
        validate_path(&path)?;
        read_optional(&path, MAX_STATE_BYTES)
    }

    /// Initial CLI activation uses the exact bytes captured before planning.
    /// An existing equal selection is idempotent; another pin requires update.
    pub fn save_for_sync(
        &self,
        project_root: &Path,
        catalog: &QualifiedCatalog,
        prior_exact_bytes: Option<&[u8]>,
    ) -> Result<SelectionWriteOutcome, SelectionWriteError> {
        self.write_for_sync(project_root, catalog, prior_exact_bytes, false)
    }

    /// Update CLI activation requires an existing exact prior pin. Both paths
    /// compare those bytes again under the selection lock before replacement.
    pub fn replace_for_sync(
        &self,
        project_root: &Path,
        catalog: &QualifiedCatalog,
        prior_exact_bytes: &[u8],
    ) -> Result<SelectionWriteOutcome, SelectionWriteError> {
        self.write_for_sync(project_root, catalog, Some(prior_exact_bytes), true)
    }

    fn write_for_sync(
        &self,
        project_root: &Path,
        catalog: &QualifiedCatalog,
        prior_exact_bytes: Option<&[u8]>,
        update: bool,
    ) -> Result<SelectionWriteOutcome, SelectionWriteError> {
        self.validate(catalog)?;
        if prior_exact_bytes.is_some_and(|bytes| bytes.len() as u64 > MAX_STATE_BYTES) {
            return Err("project skill selection exceeds its byte limit".into());
        }
        let previous = prior_exact_bytes
            .map(|bytes| serde_json::from_slice::<Self>(bytes).map_err(str_error))
            .transpose()?;
        if let Some(previous) = &previous {
            previous.validate_shape()?;
        }
        if update && previous.is_none() {
            return Err("project has no saved selection to update".into());
        }
        if !update && previous.as_ref().is_some_and(|previous| previous != self) {
            return Err("project skill selection already exists; review an explicit update".into());
        }
        let root = select_root(project_root)?;
        let path = root.join(".mirket/project.json");
        validate_path(&path)?;
        if read_optional(&path, MAX_STATE_BYTES)?.as_deref() != prior_exact_bytes {
            return Err("project skill selection changed before synchronization".into());
        }
        if previous.as_ref() == Some(self) {
            return Ok(SelectionWriteOutcome::Unchanged);
        }
        let mut created = Vec::new();
        ensure_directory(
            path.parent().ok_or("project selection has no parent")?,
            &mut created,
        )?;
        let result = (|| {
            let _lock = acquire_lock(&path.with_extension("lock"))?;
            if read_optional(&path, MAX_STATE_BYTES)?.as_deref() != prior_exact_bytes {
                return Err(SelectionWriteError::from(
                    "project skill selection changed before replacement",
                ));
            }
            let bytes = serde_json::to_vec_pretty(self).map_err(str_error)?;
            let mut replaced = false;
            atomic_replace_tracked(&path, &bytes, &mut replaced)
                .map_err(|message| SelectionWriteError { message, replaced })?;
            Ok(SelectionWriteOutcome::Replaced)
        })();
        if result.is_err() {
            remove_empty_directories(&created);
        }
        result
    }

    /// Restore a failed activation's previous exact bytes only while the pin
    /// still equals the canonical bytes written by `save`/`replace`. Unknown
    /// edits and symlink substitutions refuse recovery. Directories survive;
    /// a missing prior pin removes only `project.json`. An already restored
    /// exact state succeeds without writing or creating a lock.
    pub fn restore_exact(
        project_root: &Path,
        just_written: &Self,
        prior_exact_bytes: Option<&[u8]>,
    ) -> Result<(), String> {
        just_written.validate_shape()?;
        let expected = serde_json::to_vec_pretty(just_written).map_err(str_error)?;
        if expected.len() as u64 > MAX_STATE_BYTES
            || prior_exact_bytes.is_some_and(|bytes| bytes.len() as u64 > MAX_STATE_BYTES)
        {
            return Err("project skill selection rollback exceeds its byte limit".into());
        }
        if let Some(bytes) = prior_exact_bytes {
            let prior: Self = serde_json::from_slice(bytes).map_err(str_error)?;
            prior.validate_shape()?;
        }
        let root = select_root(project_root)?;
        let path = root.join(".mirket/project.json");
        validate_path(&path)?;
        let current = read_optional(&path, MAX_STATE_BYTES)?;
        if current.as_deref() == prior_exact_bytes {
            return Ok(());
        }
        if current.as_deref() != Some(expected.as_slice()) {
            return Err("project skill selection changed; rollback preserved current bytes".into());
        }
        let _lock = acquire_lock(&path.with_extension("lock"))?;
        let current = read_optional(&path, MAX_STATE_BYTES)?;
        if current.as_deref() == prior_exact_bytes {
            return Ok(());
        }
        if current.as_deref() != Some(expected.as_slice()) {
            return Err("project skill selection changed before rollback; preserved".into());
        }
        if let Some(bytes) = prior_exact_bytes {
            atomic_replace(&path, bytes)
        } else {
            fs::remove_file(&path).map_err(str_error)?;
            #[cfg(unix)]
            File::open(path.parent().ok_or("project selection has no parent")?)
                .and_then(|file| file.sync_all())
                .map_err(str_error)?;
            Ok(())
        }
    }

    /// Explicit initial opt-in. An existing different selection is a conflict;
    /// callers must use `replace` with the previously reviewed value to update.
    pub fn save(&self, project_root: &Path, catalog: &QualifiedCatalog) -> Result<(), String> {
        self.write_selection(project_root, catalog, None)
    }

    /// Explicit compare-and-replace of an approved selection. A user edit since
    /// the plan was read refuses the update instead of silently changing pins.
    pub fn replace(
        &self,
        project_root: &Path,
        catalog: &QualifiedCatalog,
        expected: &Self,
    ) -> Result<(), String> {
        self.write_selection(project_root, catalog, Some(expected))
    }

    fn write_selection(
        &self,
        project_root: &Path,
        catalog: &QualifiedCatalog,
        expected: Option<&Self>,
    ) -> Result<(), String> {
        self.validate(catalog)?;
        let root = select_root(project_root)?;
        let path = root.join(".mirket/project.json");
        validate_path(&path)?;
        let before = read_optional(&path, MAX_STATE_BYTES)?;
        let current = Self::load(&root)?;
        if current.as_ref() == Some(self) {
            return Ok(());
        }
        if current.as_ref() != expected {
            return Err(
                "project skill selection changed or already exists; review an explicit update"
                    .into(),
            );
        }
        let mut created = Vec::new();
        ensure_directory(
            path.parent().ok_or("project selection has no parent")?,
            &mut created,
        )?;
        let result = (|| {
            let _lock = acquire_lock(&path.with_extension("lock"))?;
            if read_optional(&path, MAX_STATE_BYTES)? != before {
                return Err("project skill selection changed before replacement".into());
            }
            atomic_replace(&path, &serde_json::to_vec_pretty(self).map_err(str_error)?)
        })();
        if result.is_err() {
            remove_empty_directories(&created);
        }
        result
    }

    fn validate_shape(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err("unsupported project skill selection schema".into());
        }
        for (values, label) in [
            (&self.profiles, "profile"),
            (&self.include, "included skill"),
            (&self.exclude, "excluded skill"),
            (&self.selected, "selected skill"),
        ] {
            unique(values, label)?;
        }
        if self.selected.iter().any(|id| self.exclude.contains(id)) {
            return Err("an excluded skill appears in the pinned selection".into());
        }
        Ok(())
    }

    fn validate(&self, catalog: &QualifiedCatalog) -> Result<(), String> {
        self.validate_shape()?;
        for name in &self.profiles {
            if !catalog.catalog.profiles.contains_key(name) {
                return Err(format!("unknown skill profile: {name}"));
            }
        }
        for id in self
            .include
            .iter()
            .chain(&self.exclude)
            .chain(&self.selected)
        {
            if !catalog.catalog.project.iter().any(|entry| &entry.id == id) {
                return Err(format!("unknown project skill: {id}"));
            }
        }
        Ok(())
    }
}

impl SkillSync {
    /// `user_home` is explicit and never inferred from process environment.
    pub fn global(
        catalog: &QualifiedCatalog,
        user_home: &Path,
        receipt_path: &Path,
    ) -> Result<Self, String> {
        let root = select_root(user_home)?;
        let destination = root.join(".agents/skills");
        validate_path(receipt_path)?;
        if receipt_path.starts_with(&destination) {
            return Err("skill receipt cannot live inside its managed destination".into());
        }
        let manager = Self {
            catalog: catalog.clone(),
            scope: "global",
            root,
            destination,
            receipt_path: receipt_path.to_owned(),
        };
        manager.validate_paths()?;
        Ok(manager)
    }

    pub fn project(catalog: &QualifiedCatalog, project_root: &Path) -> Result<Self, String> {
        let root = select_root(project_root)?;
        let manager = Self {
            catalog: catalog.clone(),
            scope: "project",
            destination: root.join(".agents/skills"),
            receipt_path: root.join(".mirket/skills-receipt.json"),
            root,
        };
        manager.validate_paths()?;
        Ok(manager)
    }

    pub fn plan(
        &self,
        release_id: &str,
        selection: Option<&ProjectSelection>,
        qualified_capabilities: &BTreeSet<String>,
    ) -> Result<SkillPlan, String> {
        self.prepare(release_id, selection, qualified_capabilities, false)
            .map(|prepared| prepared.plan)
    }

    pub fn sync(
        &self,
        release_id: &str,
        selection: Option<&ProjectSelection>,
        qualified_capabilities: &BTreeSet<String>,
    ) -> Result<SkillSyncReport, String> {
        if self.scope == "project" && ProjectSelection::load(&self.root)?.as_ref() != selection {
            return Err(
                "project sync requires the saved, explicitly approved pinned selection".into(),
            );
        }
        let _project_operation = self.project_operation_guard()?;
        self.sync_inner(release_id, selection, qualified_capabilities, false, None)
    }

    /// Treat a project pin and its native activation as one serialized action
    /// across all Mirket installation homes targeting that project. The outer
    /// project lock also covers no-op sync acknowledgements and failure recovery.
    pub fn select_and_sync(
        &self,
        release_id: &str,
        selection: &ProjectSelection,
        qualified_capabilities: &BTreeSet<String>,
        update: bool,
    ) -> Result<SkillSyncReport, String> {
        if self.scope != "project" {
            return Err("selection activation requires a project skill scope".into());
        }
        let _project_operation = self.project_operation_guard()?;
        let before = self.inspect()?;
        let prior_selection = ProjectSelection::saved_bytes(&self.root)?;
        let write = if update {
            let prior = prior_selection
                .as_deref()
                .ok_or("project has no saved selection to update")?;
            selection.replace_for_sync(&self.root, &self.catalog, prior)
        } else {
            selection.save_for_sync(&self.root, &self.catalog, prior_selection.as_deref())
        };
        let (result, replaced) = match write {
            Ok(outcome) => (
                self.sync_inner(
                    release_id,
                    Some(selection),
                    qualified_capabilities,
                    false,
                    None,
                ),
                outcome == SelectionWriteOutcome::Replaced,
            ),
            Err(error) => (Err(error.message), error.replaced),
        };
        match result {
            Ok(report) => Ok(report),
            Err(error) => {
                if replaced {
                    self.restore_selection_if_native_unchanged(
                        &before,
                        selection,
                        prior_selection.as_deref(),
                    )
                    .map_err(|restore| {
                        format!("{error}; previous selection could not be restored: {restore}")
                    })?;
                }
                Err(error)
            }
        }
    }

    /// Receipt-checked uninstall/rollback; unmanaged and edited trees survive.
    pub fn remove_managed(&self) -> Result<SkillSyncReport, String> {
        let _project_operation = self.project_operation_guard()?;
        let release_id = match self.read_receipt()? {
            Some(ReceiptFile::Committed { receipt }) => receipt.release_id,
            Some(ReceiptFile::Pending { .. }) => {
                return Err("pending skill transaction must be reconciled first".into());
            }
            None => "uninstalled".into(),
        };
        self.sync_inner(&release_id, None, &BTreeSet::new(), true, None)
    }

    /// Retire only an already completed empty removal receipt. The same
    /// destination and receipt locks exclude a competing conforming writer;
    /// pending transactions, new copies and unknown receipt bytes survive.
    /// A crash after receipt removal is reconciled as the same missing state.
    pub fn finish_removal(&self) -> Result<SkillInspection, String> {
        let _project_operation = self.project_operation_guard()?;
        self.validate_paths()?;
        let _destination_lock = acquire_lock(&self.destination_lock_path())?;
        let _lock = acquire_lock(&self.lock_path())?;
        let inspected = self.inspect()?;
        if inspected.status == "missing" {
            if path_present(&self.transaction_path())? {
                return Err("skill removal has an unowned transaction directory; preserved".into());
            }
            return Ok(inspected);
        }
        if inspected.status != "managed"
            || inspected.pending_transaction
            || !inspected.skills.is_empty()
        {
            return Err("skill receipt retirement requires a completed empty removal".into());
        }
        let before = read_optional(&self.receipt_path, MAX_STATE_BYTES)?;
        match self.read_receipt()? {
            Some(ReceiptFile::Committed { receipt }) if receipt.skills.is_empty() => {}
            _ => return Err("removed skill receipt changed; preserved".into()),
        }
        if read_optional(&self.receipt_path, MAX_STATE_BYTES)? != before {
            return Err("removed skill receipt bytes changed; preserved".into());
        }
        fs::remove_file(&self.receipt_path).map_err(str_error)?;
        #[cfg(unix)]
        File::open(
            self.receipt_path
                .parent()
                .ok_or("skill receipt has no parent")?,
        )
        .and_then(|file| file.sync_all())
        .map_err(str_error)?;
        self.inspect()
    }

    /// Read-only verification after the release/catalog has been removed.
    /// Only the former global IDs are observed; user skills are never adopted.
    pub fn inspect_global_removal(
        user_home: &Path,
        receipt_path: &Path,
        removed_ids: &[String],
    ) -> Result<(), String> {
        let root = select_root(user_home)?;
        validate_path(receipt_path)?;
        let destination = root.join(".agents/skills");
        validate_path(&destination)?;
        if receipt_path.starts_with(&destination) {
            return Err("skill receipt cannot live inside its managed destination".into());
        }
        if path_present(receipt_path)?
            || path_present(&transaction_path_for(&destination, &root, receipt_path))?
        {
            return Err("removed global skill receipt or transaction reappeared; preserved".into());
        }
        for id in removed_ids {
            validate_id(id)?;
            if path_present(&destination.join(id))? {
                return Err(format!("removed global skill reappeared; preserved: {id}"));
            }
        }
        Ok(())
    }

    pub fn inspect(&self) -> Result<SkillInspection, String> {
        self.validate_paths()?;
        let state = self.read_receipt()?;
        let mut report = SkillInspection {
            scope: self.scope,
            destination: self.destination.clone(),
            status: "missing",
            release_id: None,
            pending_transaction: false,
            skills: Vec::new(),
            errors: Vec::new(),
        };
        let receipt = match state {
            None => {
                let ids = if self.scope == "global" {
                    self.catalog
                        .catalog
                        .global
                        .iter()
                        .map(|entry| entry.id.clone())
                        .collect()
                } else {
                    ProjectSelection::load(&self.root)?
                        .map(|selection| selection.selected)
                        .unwrap_or_default()
                };
                for id in ids {
                    if path_present(&self.destination.join(&id))? {
                        report.status = "conflict";
                        report
                            .errors
                            .push(format!("unmanaged native skill collision: {id}"));
                    }
                }
                return Ok(report);
            }
            Some(ReceiptFile::Committed { receipt }) => receipt,
            Some(ReceiptFile::Pending { after, .. }) => {
                report.pending_transaction = true;
                report.status = "pending";
                after
            }
        };
        report.release_id = Some(receipt.release_id.clone());
        for (id, expected) in &receipt.skills {
            let observed = snapshot_optional(&self.destination.join(id));
            let error = match observed {
                Ok(Some(package)) if package.record == *expected => None,
                Ok(_) => Some("managed skill is missing or differs from its receipt".into()),
                Err(error) => Some(error),
            };
            report.skills.push(ManagedSkillStatus {
                id: id.clone(),
                status: if error.is_none() {
                    "managed"
                } else {
                    "drifted"
                },
                source_sha256: expected.source_sha256.clone(),
                error,
            });
        }
        if !report.pending_transaction {
            report.status = if report.skills.iter().any(|skill| skill.error.is_some()) {
                "drifted"
            } else {
                "managed"
            };
            if path_present(&self.transaction_path())? {
                report.status = "conflict";
                report
                    .errors
                    .push("skill transaction directory exists without a pending receipt".into());
            }
            if self.scope == "project" {
                let selection = ProjectSelection::load(&self.root).and_then(|selection| {
                    let selection = selection.ok_or("saved project skill selection is missing")?;
                    selection.validate(&self.catalog)?;
                    Ok(selection)
                });
                let error = match selection {
                    Ok(selection)
                        if selection.selected.iter().collect::<BTreeSet<_>>()
                            == receipt.skills.keys().collect::<BTreeSet<_>>() =>
                    {
                        None
                    }
                    Ok(_) => Some(
                        "saved project skill selection differs from the committed native copies"
                            .into(),
                    ),
                    Err(error) => Some(format!("invalid saved project skill selection: {error}")),
                };
                if let Some(error) = error {
                    report.status = "conflict";
                    report.errors.push(error);
                }
            }
        }
        Ok(report)
    }

    /// Roll back this operation's pin only while the native receipt and copies
    /// still match the pre-activation observation. The destination, receipt and
    /// selection locks use the same order as sync, so another installation home
    /// cannot commit project skills between the evidence check and restoration.
    fn restore_selection_if_native_unchanged(
        &self,
        before: &SkillInspection,
        just_written: &ProjectSelection,
        prior_exact_bytes: Option<&[u8]>,
    ) -> Result<bool, String> {
        if self.scope != "project"
            || before.scope != "project"
            || before.destination != self.destination
        {
            return Err("selection recovery requires this project skill scope".into());
        }
        self.validate_paths()?;
        let mut created = Vec::new();
        let result = (|| {
            ensure_directory(
                self.destination
                    .parent()
                    .ok_or("skill destination has no parent")?,
                &mut created,
            )?;
            ensure_directory(
                self.receipt_path
                    .parent()
                    .ok_or("skill receipt has no parent")?,
                &mut created,
            )?;
            let _destination_lock = acquire_lock(&self.destination_lock_path())?;
            let _receipt_lock = acquire_lock(&self.lock_path())?;
            let after = self.inspect()?;
            if before.pending_transaction
                || after.pending_transaction
                || before.release_id != after.release_id
                || before.skills != after.skills
            {
                return Ok(false);
            }
            ProjectSelection::restore_exact(&self.root, just_written, prior_exact_bytes)?;
            Ok(true)
        })();
        if result.is_err() {
            remove_empty_directories(&created);
        }
        result
    }

    /// Check both possible receipts before lifecycle-owned reconciliation.
    pub(crate) fn verify_receipt_release(&self, expected: &str) -> Result<(), String> {
        self.validate_paths()?;
        let matches = match self.read_receipt()? {
            None => true,
            Some(ReceiptFile::Committed { receipt }) => receipt.release_id == expected,
            Some(ReceiptFile::Pending { before, after }) => {
                after.release_id == expected
                    && before
                        .as_ref()
                        .is_none_or(|before| before.release_id == expected)
            }
        };
        if !matches {
            return Err(
                "native receipt transition belongs to another lifecycle release; preserved".into(),
            );
        }
        Ok(())
    }

    /// Finish an entirely applied transaction, otherwise restore its known
    /// before state. Unknown external edits refuse all recovery writes.
    pub fn reconcile(&self) -> Result<SkillInspection, String> {
        self.validate_paths()?;
        if self.read_receipt()?.is_none() {
            return self.inspect();
        }
        let _project_operation = self.project_operation_guard()?;
        let _destination_lock = acquire_lock(&self.destination_lock_path())?;
        let _lock = acquire_lock(&self.lock_path())?;
        if let Some(ReceiptFile::Pending { before, after }) = self.read_receipt()? {
            self.validate_transaction(&before, &after)?;
            if self.matches_transition(before.as_ref(), &after)? {
                self.cleanup_transaction(&before, &after)?;
                self.write_receipt(&ReceiptFile::Committed { receipt: after })?;
            } else {
                self.rollback(&before, &after)?;
            }
        }
        self.inspect()
    }

    fn sync_inner(
        &self,
        release_id: &str,
        selection: Option<&ProjectSelection>,
        qualified: &BTreeSet<String>,
        remove: bool,
        fail_after: Option<usize>,
    ) -> Result<SkillSyncReport, String> {
        if self.scope == "project"
            && !remove
            && ProjectSelection::load(&self.root)?.as_ref() != selection
        {
            return Err(
                "project sync requires the saved, explicitly approved pinned selection".into(),
            );
        }
        let initial = self.prepare(release_id, selection, qualified, remove)?;
        ensure_available(&initial.plan)?;
        if initial.before.as_ref() == Some(&initial.after) {
            return Ok(SkillSyncReport {
                status: "unchanged",
                plan: initial.plan,
            });
        }
        if remove && initial.before.is_none() {
            return Ok(SkillSyncReport {
                status: "missing",
                plan: initial.plan,
            });
        }
        // All source packages, managed copies and destinations were validated
        // above before directory/lock creation. Recheck under the native lock.
        let mut created = Vec::new();
        ensure_directory(
            self.receipt_path
                .parent()
                .ok_or("skill receipt has no parent")?,
            &mut created,
        )?;
        ensure_directory(&self.destination, &mut created)?;
        let result = (|| {
            let _destination_lock = acquire_lock(&self.destination_lock_path())?;
            let _lock = acquire_lock(&self.lock_path())?;
            let _selection_lock = if self.scope == "project" && !remove {
                Some(acquire_lock(&self.root.join(".mirket/project.lock"))?)
            } else {
                None
            };
            if self.scope == "project"
                && !remove
                && ProjectSelection::load(&self.root)?.as_ref() != selection
            {
                return Err("project skill selection changed before synchronization".into());
            }
            let prepared = self.prepare(release_id, selection, qualified, remove)?;
            ensure_available(&prepared.plan)?;
            let pending = ReceiptFile::Pending {
                before: prepared.before.clone(),
                after: prepared.after.clone(),
            };
            self.write_receipt(&pending)?;
            let applied = self.apply(&prepared, fail_after);
            if let Err(error) = applied {
                return match self.rollback(&prepared.before, &prepared.after) {
                    Ok(()) => Err(format!("skill sync rolled back: {error}")),
                    Err(rollback) => Err(format!(
                        "skill sync failed: {error}; recovery refused: {rollback}; reconcile the pending transaction"
                    )),
                };
            }
            // From this point the complete after state was verified. Cleanup can
            // be retried by reconcile even after some backups are removed.
            self.cleanup_transaction(&prepared.before, &prepared.after)
                .map_err(|error| format!("skills applied; reconcile pending cleanup: {error}"))?;
            self.write_receipt(&ReceiptFile::Committed {
                receipt: prepared.after,
            })
            .map_err(|error| format!("skills applied; reconcile pending receipt: {error}"))?;
            Ok(SkillSyncReport {
                status: if remove { "removed" } else { "synced" },
                plan: prepared.plan,
            })
        })();
        if result.is_err() {
            remove_empty_directories(&created);
        }
        result
    }

    fn prepare(
        &self,
        release_id: &str,
        selection: Option<&ProjectSelection>,
        qualified: &BTreeSet<String>,
        remove: bool,
    ) -> Result<Prepared, String> {
        validate_release_id(release_id)?;
        self.validate_paths()?;
        let before = match self.read_receipt()? {
            Some(ReceiptFile::Committed { receipt }) => Some(receipt),
            Some(ReceiptFile::Pending { .. }) => {
                return Err("pending skill transaction must be reconciled first".into());
            }
            None => None,
        };
        if path_present(&self.transaction_path())? {
            return Err("skill transaction directory already exists; inspect before sync".into());
        }
        if !self.matches_receipt(before.as_ref())? {
            return Err("managed skill copy has drifted; refusing replacement or removal".into());
        }
        let ids = if remove {
            Vec::new()
        } else if self.scope == "global" {
            if selection.is_some() {
                return Err("global skills do not accept a project selection".into());
            }
            self.catalog
                .catalog
                .global
                .iter()
                .map(|entry| entry.id.clone())
                .collect()
        } else {
            let selection =
                selection.ok_or("project skills require an explicit pinned selection")?;
            selection.validate(&self.catalog)?;
            selection.selected.clone()
        };
        let mut unavailable = Vec::new();
        let mut packages = BTreeMap::new();
        for id in ids {
            let entry = self
                .catalog
                .entry(&id)
                .ok_or_else(|| format!("unknown skill: {id}"))?;
            let missing: Vec<_> = entry
                .required_capabilities
                .iter()
                .filter(|id| !qualified.contains(*id))
                .cloned()
                .collect();
            if !missing.is_empty() {
                unavailable.push(UnavailableSkill {
                    id: id.clone(),
                    missing_capabilities: missing,
                });
            }
            let source = self.catalog.root.join("skills").join(&id);
            validate_path(&source)?;
            let package = snapshot(&source)?;
            if !package.record.files.contains_key("SKILL.md") {
                return Err(format!("skill {id} has no native SKILL.md"));
            }
            let native_body = &package.record.files["SKILL.md"];
            if native_body.sha256 != entry.source_sha256 || native_body.bytes != entry.source_bytes
            {
                return Err(format!(
                    "skill source differs from the qualified catalog: {id}"
                ));
            }
            let destination = self.destination.join(&id);
            if path_present(&destination)?
                && !before
                    .as_ref()
                    .is_some_and(|receipt| receipt.skills.contains_key(&id))
            {
                return Err(format!("unmanaged native skill collision: {id}"));
            }
            packages.insert(id, package);
        }
        let after = Receipt {
            schema_version: 1,
            scope: self.scope.into(),
            destination: self.destination.clone(),
            release_id: release_id.into(),
            skills: packages
                .iter()
                .map(|(id, package)| (id.clone(), package.record.clone()))
                .collect(),
        };
        let changes = skill_ids(before.as_ref(), &after)
            .into_iter()
            .map(|id| {
                let previous = before.as_ref().and_then(|receipt| receipt.skills.get(&id));
                let next = after.skills.get(&id);
                let action = match (previous, next) {
                    (None, Some(_)) => "install",
                    (Some(_), None) => "remove",
                    (Some(a), Some(b)) if a == b => "unchanged",
                    _ => "update",
                };
                SkillChange {
                    destination: self.destination.join(&id),
                    id,
                    action,
                    source_sha256: next.map(|record| record.source_sha256.clone()),
                    previous_sha256: previous.map(|record| record.source_sha256.clone()),
                }
            })
            .collect();
        let selected = after.skills.keys().cloned().collect::<Vec<_>>();
        let warnings = if self.scope == "project" && selected.len() > 12 {
            vec!["more than twelve selected specialists may crowd native skill discovery".into()]
        } else {
            Vec::new()
        };
        Ok(Prepared {
            plan: SkillPlan {
                scope: self.scope,
                destination: self.destination.clone(),
                receipt_path: self.receipt_path.clone(),
                release_id: release_id.into(),
                selected,
                changes,
                unavailable,
                warnings,
            },
            before,
            after,
            packages,
        })
    }

    fn apply(&self, prepared: &Prepared, fail_after: Option<usize>) -> Result<(), String> {
        let transaction = self.transaction_path();
        private_directory(&transaction)?;
        private_directory(&transaction.join("after"))?;
        private_directory(&transaction.join("before"))?;
        for change in prepared
            .plan
            .changes
            .iter()
            .filter(|change| change.action != "unchanged")
        {
            if let Some(package) = prepared.packages.get(&change.id) {
                write_package(&transaction.join("after").join(&change.id), package)?;
            }
        }
        if !self.matches_receipt(prepared.before.as_ref())? {
            return Err("managed skills changed during staging".into());
        }
        let mut count = 0;
        for change in prepared
            .plan
            .changes
            .iter()
            .filter(|change| change.action != "unchanged")
        {
            let destination = self.destination.join(&change.id);
            // A previously absent destination might have appeared while the
            // packages were staged; never claim it by rename semantics.
            let old = prepared
                .before
                .as_ref()
                .and_then(|receipt| receipt.skills.get(&change.id));
            if record_at(&destination)? != old.cloned() {
                return Err(format!(
                    "skill destination changed before replacement: {}",
                    change.id
                ));
            }
            if old.is_some() {
                fs::rename(&destination, transaction.join("before").join(&change.id))
                    .map_err(str_error)?;
            }
            if prepared.after.skills.contains_key(&change.id) {
                fs::rename(transaction.join("after").join(&change.id), &destination)
                    .map_err(str_error)?;
            }
            count += 1;
            if fail_after == Some(count) {
                return Err("injected replacement failure".into());
            }
        }
        if !self.matches_transition(prepared.before.as_ref(), &prepared.after)? {
            return Err("skill verification failed after replacement".into());
        }
        Ok(())
    }

    fn rollback(&self, before: &Option<Receipt>, after: &Receipt) -> Result<(), String> {
        self.validate_transaction(before, after)?;
        let ids = skill_ids(before.as_ref(), after);
        // Recovery is all-or-nothing with respect to observed state: inspect
        // every destination and needed backup before the first recovery write.
        for id in &ids {
            let old = before.as_ref().and_then(|receipt| receipt.skills.get(id));
            let new = after.skills.get(id);
            let current = record_at(&self.destination.join(id))?;
            if current.as_ref() != old && current.as_ref() != new && current.is_some() {
                return Err(format!("unknown skill state during rollback: {id}"));
            }
            if current.as_ref() != old
                && old.is_some()
                && record_at(&self.transaction_path().join("before").join(id))?.as_ref() != old
            {
                return Err(format!("original skill backup is unavailable: {id}"));
            }
        }
        for id in ids.iter().rev() {
            let old = before.as_ref().and_then(|receipt| receipt.skills.get(id));
            let destination = self.destination.join(id);
            if record_at(&destination)?.as_ref() != old {
                if path_present(&destination)? {
                    fs::remove_dir_all(&destination).map_err(str_error)?;
                }
                if old.is_some() {
                    fs::rename(
                        self.transaction_path().join("before").join(id),
                        &destination,
                    )
                    .map_err(str_error)?;
                }
            }
        }
        self.cleanup_transaction(before, after)?;
        match before {
            Some(receipt) => self.write_receipt(&ReceiptFile::Committed {
                receipt: receipt.clone(),
            })?,
            None => fs::remove_file(&self.receipt_path).map_err(str_error)?,
        }
        Ok(())
    }

    fn validate_transaction(
        &self,
        before: &Option<Receipt>,
        after: &Receipt,
    ) -> Result<(), String> {
        self.validate_paths()?;
        if let Some(before) = before {
            self.validate_receipt(before)?;
        }
        self.validate_receipt(after)?;
        let transaction = self.transaction_path();
        if !path_present(&transaction)? {
            return Ok(());
        }
        validate_path(&transaction)?;
        require_directory(&transaction)?;
        for entry in fs::read_dir(&transaction).map_err(str_error)? {
            let entry = entry.map_err(str_error)?;
            let name = entry.file_name();
            let expected = match name.to_str() {
                Some("before") => before.as_ref().map(|receipt| &receipt.skills),
                Some("after") => Some(&after.skills),
                _ => return Err("unknown entry in skill transaction directory".into()),
            };
            require_directory(&entry.path())?;
            for package in fs::read_dir(entry.path()).map_err(str_error)? {
                let package = package.map_err(str_error)?;
                let id = package
                    .file_name()
                    .into_string()
                    .map_err(|_| "non-UTF-8 transaction entry")?;
                let record = expected
                    .and_then(|skills| skills.get(&id))
                    .ok_or("unknown skill in transaction directory")?;
                let observed =
                    record_at(&package.path())?.ok_or("transaction package disappeared")?;
                let matches = if name == "after" {
                    // A failed stage may contain only complete source files;
                    // write_new removes its own partial write on error.
                    observed.directories.is_subset(&record.directories)
                        && observed
                            .files
                            .iter()
                            .all(|(path, file)| record.files.get(path) == Some(file))
                } else {
                    observed == *record
                };
                if !matches {
                    return Err(format!("skill transaction package changed: {id}"));
                }
            }
        }
        Ok(())
    }

    fn cleanup_transaction(&self, before: &Option<Receipt>, after: &Receipt) -> Result<(), String> {
        self.validate_transaction(before, after)?;
        let transaction = self.transaction_path();
        if path_present(&transaction)? {
            fs::remove_dir_all(transaction).map_err(str_error)?;
        }
        Ok(())
    }

    fn matches_receipt(&self, receipt: Option<&Receipt>) -> Result<bool, String> {
        let Some(receipt) = receipt else {
            return Ok(true);
        };
        for (id, expected) in &receipt.skills {
            if record_at(&self.destination.join(id))?.as_ref() != Some(expected) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn matches_transition(
        &self,
        before: Option<&Receipt>,
        after: &Receipt,
    ) -> Result<bool, String> {
        for id in skill_ids(before, after) {
            if record_at(&self.destination.join(&id))?.as_ref() != after.skills.get(&id) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn validate_paths(&self) -> Result<(), String> {
        select_root(&self.root)?;
        for path in [
            &self.destination,
            &self.receipt_path,
            &self.lock_path(),
            &self.destination_lock_path(),
            &self.transaction_path(),
        ] {
            validate_path(path)?;
        }
        if path_present(&self.destination)? {
            require_directory(&self.destination)?;
        }
        Ok(())
    }

    fn lock_path(&self) -> PathBuf {
        self.receipt_path.with_extension("lock")
    }
    fn project_operation_guard(&self) -> Result<Option<FileLock>, String> {
        if self.scope != "project" {
            return Ok(None);
        }
        let mut created = Vec::new();
        let operation_dir = self.root.join(".mirket");
        ensure_directory(&operation_dir, &mut created)?;
        match acquire_lock(&operation_dir.join("project-skills-operation.lock")) {
            Ok(guard) => Ok(Some(guard)),
            Err(error) => {
                remove_empty_directories(&created);
                Err(error)
            }
        }
    }
    fn destination_lock_path(&self) -> PathBuf {
        self.destination
            .parent()
            .unwrap_or(&self.root)
            .join(".mirket-skills-destination.lock")
    }
    fn transaction_path(&self) -> PathBuf {
        // Stage beside the native destination, so directory swaps remain on
        // its filesystem even when the installation receipt lives elsewhere.
        transaction_path_for(&self.destination, &self.root, &self.receipt_path)
    }

    fn read_receipt(&self) -> Result<Option<ReceiptFile>, String> {
        let Some(bytes) = read_optional(&self.receipt_path, MAX_STATE_BYTES)? else {
            return Ok(None);
        };
        let state: ReceiptFile = serde_json::from_slice(&bytes).map_err(str_error)?;
        match &state {
            ReceiptFile::Committed { receipt } => self.validate_receipt(receipt)?,
            ReceiptFile::Pending { before, after } => {
                if let Some(before) = before {
                    self.validate_receipt(before)?;
                }
                self.validate_receipt(after)?;
            }
        }
        Ok(Some(state))
    }

    fn validate_receipt(&self, receipt: &Receipt) -> Result<(), String> {
        if receipt.schema_version != 1
            || receipt.scope != self.scope
            || receipt.destination != self.destination
            || receipt.skills.len() > 128
        {
            return Err("skill receipt belongs to a different scope or unsupported schema".into());
        }
        validate_release_id(&receipt.release_id)?;
        for (id, record) in &receipt.skills {
            validate_id(id)?;
            validate_package_record(record)?;
        }
        Ok(())
    }

    fn write_receipt(&self, state: &ReceiptFile) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(state).map_err(str_error)?;
        if bytes.len() as u64 > MAX_STATE_BYTES {
            return Err("skill receipt exceeds size limit".into());
        }
        atomic_replace(&self.receipt_path, &bytes)
    }
}

fn transaction_path_for(destination: &Path, root: &Path, receipt_path: &Path) -> PathBuf {
    destination.parent().unwrap_or(root).join(format!(
        ".mirket-skills-{}",
        &hash(receipt_path.to_string_lossy().as_bytes())[..24]
    ))
}

fn ensure_available(plan: &SkillPlan) -> Result<(), String> {
    if plan.unavailable.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "selected skills require unqualified private capabilities: {}",
            plan.unavailable
                .iter()
                .map(|entry| format!("{} ({})", entry.id, entry.missing_capabilities.join(", ")))
                .collect::<Vec<_>>()
                .join("; ")
        ))
    }
}

fn skill_ids(before: Option<&Receipt>, after: &Receipt) -> BTreeSet<String> {
    before
        .into_iter()
        .flat_map(|receipt| receipt.skills.keys())
        .chain(after.skills.keys())
        .cloned()
        .collect()
}

fn unique(values: &[String], label: &str) -> Result<Vec<String>, String> {
    let mut selected = BTreeSet::new();
    for value in values {
        validate_id(value)?;
        if !selected.insert(value.clone()) {
            return Err(format!("duplicate {label}: {value}"));
        }
    }
    Ok(selected.into_iter().collect())
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 64
        || id.starts_with('-')
        || id.ends_with('-')
        || id.contains("--")
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(format!("invalid skill/profile ID: {id}"));
    }
    Ok(())
}

fn validate_release_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 256 || id.chars().any(char::is_control) {
        return Err("invalid skill release identity".into());
    }
    Ok(())
}

fn select_root(path: &Path) -> Result<PathBuf, String> {
    validate_path(path)?;
    require_directory(path)?;
    path.canonicalize().map_err(str_error)
}

fn validate_path(path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err("native skill paths must be absolute without traversal".into());
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        // A Windows drive prefix alone is not absolute; inspect only once
        // its root component has been accumulated.
        if !current.is_absolute() {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!(
                    "symlink in native skill path: {}",
                    current.display()
                ));
            }
            Ok(meta) if current != path && !meta.is_dir() => {
                return Err(format!(
                    "native skill ancestor is not a directory: {}",
                    current.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(str_error(error)),
        }
    }
    Ok(())
}

fn require_directory(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(str_error)?;
    if meta.file_type().is_symlink() || !meta.is_dir() {
        return Err(format!(
            "native skill directory is not real: {}",
            path.display()
        ));
    }
    Ok(())
}

fn path_present(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(str_error(error)),
    }
}

fn read_optional(path: &Path, max: u64) -> Result<Option<Vec<u8>>, String> {
    if !path_present(path)? {
        return Ok(None);
    }
    validate_path(path)?;
    let before = fs::symlink_metadata(path).map_err(str_error)?;
    if !before.is_file() || before.len() > max {
        return Err(format!(
            "native skill file is not bounded and regular: {}",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(str_error)?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(str_error)?;
    let after = fs::symlink_metadata(path).map_err(str_error)?;
    if after.file_type().is_symlink()
        || !after.is_file()
        || after.len() != before.len()
        || bytes.len() as u64 != before.len()
        || bytes.len() as u64 > max
    {
        return Err("native skill file changed during read".into());
    }
    Ok(Some(bytes))
}

fn snapshot_optional(path: &Path) -> Result<Option<Package>, String> {
    if path_present(path)? {
        snapshot(path).map(Some)
    } else {
        Ok(None)
    }
}

fn record_at(path: &Path) -> Result<Option<PackageRecord>, String> {
    snapshot_optional(path).map(|package| package.map(|package| package.record))
}

fn snapshot(path: &Path) -> Result<Package, String> {
    validate_path(path)?;
    require_directory(path)?;
    let mut package = Package {
        record: PackageRecord {
            source_sha256: String::new(),
            directories: BTreeSet::new(),
            files: BTreeMap::new(),
        },
        bytes: BTreeMap::new(),
    };
    visit_package(path, path, &mut package)?;
    package.record.source_sha256 = package_hash(&package.record)?;
    validate_package_record(&package.record)?;
    Ok(package)
}

fn visit_package(root: &Path, path: &Path, package: &mut Package) -> Result<(), String> {
    for entry in fs::read_dir(path).map_err(str_error)? {
        let entry = entry.map_err(str_error)?;
        let path = entry.path();
        let relative = path.strip_prefix(root).map_err(str_error)?;
        let name = relative
            .components()
            .map(|part| {
                part.as_os_str()
                    .to_str()
                    .ok_or("non-UTF-8 skill package path")
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("/");
        validate_relative(&name)?;
        let meta = fs::symlink_metadata(&path).map_err(str_error)?;
        if meta.file_type().is_symlink() {
            return Err(format!("symlink in skill package: {name}"));
        }
        if meta.is_dir() {
            package.record.directories.insert(name);
        } else if meta.is_file() {
            let bytes =
                read_optional(&path, MAX_FILE_BYTES)?.ok_or("skill package file disappeared")?;
            package.record.files.insert(
                name.clone(),
                FileRecord {
                    bytes: bytes.len() as u64,
                    sha256: hash(&bytes),
                },
            );
            package.bytes.insert(name, bytes);
        } else {
            return Err("skill package contains a special file".into());
        }
        if package.record.files.len() + package.record.directories.len() > MAX_PACKAGE_ENTRIES
            || package
                .record
                .files
                .values()
                .map(|file| file.bytes)
                .sum::<u64>()
                > MAX_PACKAGE_BYTES
        {
            return Err("skill package exceeds its bounded file/byte limits".into());
        }
        if meta.is_dir() {
            visit_package(root, &path, package)?;
        }
    }
    Ok(())
}

fn validate_relative(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.len() > 512
        || path.contains('\\')
        || path.split('/').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.contains(':')
                || part.chars().any(char::is_control)
        })
    {
        return Err("invalid relative native skill package path".into());
    }
    Ok(())
}

fn validate_package_record(record: &PackageRecord) -> Result<(), String> {
    if record.files.len() + record.directories.len() > MAX_PACKAGE_ENTRIES
        || record
            .files
            .values()
            .any(|file| file.bytes > MAX_FILE_BYTES)
        || record.files.values().map(|file| file.bytes).sum::<u64>() > MAX_PACKAGE_BYTES
        || package_hash(record)? != record.source_sha256
    {
        return Err("invalid skill package receipt/hash".into());
    }
    for path in record.files.keys().chain(record.directories.iter()) {
        validate_relative(path)?;
    }
    for (path, file) in &record.files {
        if file.bytes > MAX_FILE_BYTES
            || file.sha256.len() != 64
            || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || record.directories.contains(path)
        {
            return Err("invalid skill package file receipt".into());
        }
    }
    Ok(())
}

fn package_hash(record: &PackageRecord) -> Result<String, String> {
    serde_json::to_vec(&(&record.directories, &record.files))
        .map(|bytes| hash(&bytes))
        .map_err(str_error)
}

fn write_package(path: &Path, package: &Package) -> Result<(), String> {
    private_directory(path)?;
    for directory in &package.record.directories {
        private_directory(&path.join(directory))?;
    }
    for (relative, bytes) in &package.bytes {
        write_new(&path.join(relative), bytes)?;
    }
    if record_at(path)?.as_ref() != Some(&package.record) {
        return Err("staged skill bytes differ from source".into());
    }
    Ok(())
}

fn ensure_directory(path: &Path, created: &mut Vec<PathBuf>) -> Result<(), String> {
    validate_path(path)?;
    if path_present(path)? {
        return require_directory(path);
    }
    ensure_directory(
        path.parent()
            .ok_or("native skill directory has no parent")?,
        created,
    )?;
    private_directory(path)?;
    created.push(path.to_owned());
    Ok(())
}

fn private_directory(path: &Path) -> Result<(), String> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).map_err(str_error)
}

fn remove_empty_directories(paths: &[PathBuf]) {
    for path in paths.iter().rev() {
        let _ = fs::remove_dir(path);
    }
}

fn acquire_lock(path: &Path) -> Result<FileLock, String> {
    validate_path(path)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = match options.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let meta = fs::symlink_metadata(path).map_err(str_error)?;
            if meta.file_type().is_symlink() || !meta.is_file() || meta.len() != 0 {
                return Err("native skill lock must be an empty regular file".into());
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(str_error)?
        }
        Err(error) => return Err(str_error(error)),
    };
    file.try_lock()
        .map_err(|_| "another native skill operation owns the lock")?;
    Ok(FileLock::acquired(file))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(str_error)?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(str_error(error));
    }
    Ok(())
}

fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut renamed = false;
    atomic_replace_tracked(path, bytes, &mut renamed)
}

fn atomic_replace_tracked(path: &Path, bytes: &[u8], renamed: &mut bool) -> Result<(), String> {
    atomic_replace_tracked_with(path, bytes, renamed, || {
        #[cfg(unix)]
        File::open(path.parent().ok_or("native skill file has no parent")?)
            .and_then(|file| file.sync_all())
            .map_err(str_error)?;
        Ok(())
    })
}

fn atomic_replace_tracked_with(
    path: &Path,
    bytes: &[u8],
    renamed: &mut bool,
    after_rename: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    validate_path(path)?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let temp = path.with_file_name(format!(
        ".{}-next-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("skills"),
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    write_new(&temp, bytes)?;
    #[cfg(not(windows))]
    let replaced = fs::rename(&temp, path).map_err(str_error);
    #[cfg(windows)]
    let replaced = {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let from = temp
            .as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        let to = path
            .as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            Err(str_error(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    };
    if replaced.is_err() {
        let _ = fs::remove_file(&temp);
    }
    replaced?;
    *renamed = true;
    after_rename()
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn str_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_skills::QualifiedCatalog;

    #[test]
    fn finish_removal_requires_an_empty_committed_owner_and_is_idempotent_after_lost_response() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        let owned = fs::read(&manager.receipt_path).unwrap();
        assert!(manager.finish_removal().is_err());
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), owned);
        manager.remove_managed().unwrap();
        let empty = fs::read(&manager.receipt_path).unwrap();
        fs::create_dir(manager.transaction_path()).unwrap();
        assert!(manager.finish_removal().is_err());
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), empty);
        fs::remove_dir(manager.transaction_path()).unwrap();
        assert_eq!(manager.finish_removal().unwrap().status, "missing");
        // The effect completed but its acknowledgement was lost.
        assert_eq!(manager.finish_removal().unwrap().status, "missing");
        let ids = vec![
            "task-planning".to_owned(),
            "outcome-verification".to_owned(),
        ];
        SkillSync::inspect_global_removal(&manager.root, &manager.receipt_path, &ids).unwrap();
        fs::create_dir(manager.transaction_path()).unwrap();
        assert!(
            SkillSync::inspect_global_removal(&manager.root, &manager.receipt_path, &ids).is_err()
        );
        assert!(manager.finish_removal().is_err());
        fs::remove_dir(manager.transaction_path()).unwrap();
        fs::write(&manager.receipt_path, b"unknown receipt edit").unwrap();
        assert!(manager.finish_removal().is_err());
        assert!(
            SkillSync::inspect_global_removal(&manager.root, &manager.receipt_path, &ids).is_err()
        );
        assert_eq!(
            fs::read(&manager.receipt_path).unwrap(),
            b"unknown receipt edit"
        );
    }

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "mirket-skill-sync-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("release/skills")).unwrap();
            for path in ["user", "project", "state"] {
                fs::create_dir(root.join(path)).unwrap();
            }
            let fixture = Self(root.canonicalize().unwrap());
            for (id, description) in [
                ("task-planning", "Plan dependent work"),
                ("outcome-verification", "Verify saved results"),
                ("architecture", "Choose module boundaries"),
                ("browser-qa", "Test browser journeys"),
            ] {
                let path = fixture.0.join("release/skills").join(id);
                fs::create_dir(&path).unwrap();
                fs::write(path.join("SKILL.md"), format!("---\nname: {id}\ndescription: {description}\n---\n\n# {id}\n\nOriginal method.\n")).unwrap();
            }
            fs::create_dir(fixture.0.join("release/skills/task-planning/references")).unwrap();
            fs::write(
                fixture
                    .0
                    .join("release/skills/task-planning/references/method.md"),
                "Reference with a distinct source.\n",
            )
            .unwrap();
            let entry = |id: &str, description: &str, required: &[&str]| {
                serde_json::json!({
                    "id": id, "description": description, "path": format!("skills/{id}/SKILL.md"),
                    "required_capabilities": required, "conditional_capabilities": [], "source_sha256": "", "source_bytes": 0, "provenance": {}, "evaluation": {}
                })
            };
            let catalog = serde_json::json!({
                "schema_version": 1,
                "authoring_date": "2026-10-25", "status": "unit fixture", "authoring_basis": [],
                "capability_vocabulary": ["browser"], "reference_sources": [], "evaluation_contract": {},
                "global": [entry("task-planning", "Plan dependent work", &[]), entry("outcome-verification", "Verify saved results", &[])],
                "project": [entry("architecture", "Choose module boundaries", &[]), entry("browser-qa", "Test browser journeys", &["browser"])],
                "profiles": { "library-cli": {"skills": ["architecture"], "conditional": ["browser-qa"]} }
            });
            fs::write(
                fixture.0.join("release/skills/catalog.json"),
                serde_json::to_vec(&catalog).unwrap(),
            )
            .unwrap();
            fixture
        }

        fn catalog(&self) -> QualifiedCatalog {
            // Catalog provenance/shape validation belongs to native_skills;
            // these small package fixtures exercise only the copy owner.
            let root = self.0.join("release");
            let mut catalog: crate::native_skills::SkillCatalog =
                serde_json::from_slice(&fs::read(root.join("skills/catalog.json")).unwrap())
                    .unwrap();
            for entry in catalog.global.iter_mut().chain(&mut catalog.project) {
                let bytes = fs::read(root.join(&entry.path)).unwrap();
                entry.source_sha256 = hash(&bytes);
                entry.source_bytes = bytes.len() as u64;
            }
            QualifiedCatalog { root, catalog }
        }
        fn global(&self) -> SkillSync {
            SkillSync::global(
                &self.catalog(),
                &self.0.join("user"),
                &self.0.join("state/global-skills.json"),
            )
            .unwrap()
        }
        fn project(&self) -> SkillSync {
            SkillSync::project(&self.catalog(), &self.0.join("project")).unwrap()
        }
        fn update_source(&self, id: &str) {
            let path = self.0.join("release/skills").join(id).join("SKILL.md");
            let mut body = fs::read_to_string(&path).unwrap();
            body.push_str("\nA revised, observable method.\n");
            fs::write(path, body).unwrap();
        }
        fn begin(&self, manager: &SkillSync, prepared: &Prepared) {
            let mut created = Vec::new();
            ensure_directory(manager.receipt_path.parent().unwrap(), &mut created).unwrap();
            ensure_directory(&manager.destination, &mut created).unwrap();
            manager
                .write_receipt(&ReceiptFile::Pending {
                    before: prepared.before.clone(),
                    after: prepared.after.clone(),
                })
                .unwrap();
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn read_only_plan_then_exact_global_copy_and_idempotent_update() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        let plan = manager.plan("release:a", None, &BTreeSet::new()).unwrap();
        assert_eq!(plan.selected.len(), 2);
        assert!(plan.changes.iter().all(|change| change.action == "install"));
        assert!(!fixture.0.join("user/.agents").exists());
        assert!(!manager.receipt_path.exists());
        assert_eq!(manager.inspect().unwrap().status, "missing");
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        assert_eq!(
            fs::read(
                manager
                    .destination
                    .join("task-planning/references/method.md")
            )
            .unwrap(),
            fs::read(
                fixture
                    .0
                    .join("release/skills/task-planning/references/method.md")
            )
            .unwrap()
        );
        let receipt = fs::read(&manager.receipt_path).unwrap();
        assert_eq!(
            manager
                .sync("release:a", None, &BTreeSet::new())
                .unwrap()
                .status,
            "unchanged"
        );
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), receipt);
        fs::create_dir(manager.destination.join("user-authored")).unwrap();
        fs::write(
            manager.destination.join("user-authored/notes.md"),
            "Keep this.",
        )
        .unwrap();
        fixture.update_source("task-planning");
        let updated = fixture
            .global()
            .sync("release:b", None, &BTreeSet::new())
            .unwrap();
        assert_eq!(
            updated
                .plan
                .changes
                .iter()
                .find(|change| change.id == "task-planning")
                .unwrap()
                .action,
            "update"
        );
        assert_eq!(
            manager.inspect().unwrap().release_id.as_deref(),
            Some("release:b")
        );
        assert_eq!(
            fs::read_to_string(manager.destination.join("user-authored/notes.md")).unwrap(),
            "Keep this."
        );
    }

    #[test]
    fn selection_is_explicit_pinned_and_capability_checked() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let manager = fixture.project();
        let selection = ProjectSelection::resolve(
            &catalog,
            &["library-cli".into()],
            &["browser-qa".into()],
            &[],
        )
        .unwrap();
        let plan = manager
            .plan("release:a", Some(&selection), &BTreeSet::new())
            .unwrap();
        assert_eq!(plan.unavailable[0].id, "browser-qa");
        assert!(!fixture.0.join("project/.mirket").exists());
        assert!(
            manager
                .sync(
                    "release:a",
                    Some(&selection),
                    &BTreeSet::from(["browser".into()])
                )
                .unwrap_err()
                .contains("saved")
        );
        selection
            .save(&fixture.0.join("project"), &catalog)
            .unwrap();
        assert!(!manager.destination.exists());
        assert!(
            manager
                .sync("release:a", Some(&selection), &BTreeSet::new())
                .unwrap_err()
                .contains("unqualified")
        );
        assert!(!manager.destination.exists());
        manager
            .sync(
                "release:a",
                Some(&selection),
                &BTreeSet::from(["browser".into()]),
            )
            .unwrap();
        let mut changed_recipes = catalog.clone();
        changed_recipes
            .catalog
            .profiles
            .get_mut("library-cli")
            .unwrap()
            .skills = vec!["browser-qa".into()];
        let loaded = ProjectSelection::load(&fixture.0.join("project"))
            .unwrap()
            .unwrap();
        let pinned = SkillSync::project(&changed_recipes, &fixture.0.join("project"))
            .unwrap()
            .plan(
                "release:b",
                Some(&loaded),
                &BTreeSet::from(["browser".into()]),
            )
            .unwrap();
        assert_eq!(pinned.selected, vec!["architecture", "browser-qa"]);
        let disabled = ProjectSelection::resolve(&catalog, &[], &[], &[]).unwrap();
        assert!(disabled.save(&fixture.0.join("project"), &catalog).is_err());
        disabled
            .replace(&fixture.0.join("project"), &catalog, &loaded)
            .unwrap();
        assert!(
            loaded
                .replace(&fixture.0.join("project"), &catalog, &loaded)
                .is_err()
        );
        manager
            .sync("release:b", Some(&disabled), &BTreeSet::new())
            .unwrap();
        assert!(!manager.destination.join("architecture").exists());
        assert!(!manager.destination.join("browser-qa").exists());
    }

    #[test]
    fn failed_project_sync_can_restore_the_exact_previous_selection() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let manager = fixture.project();
        let previous =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        previous.save(&project, &catalog).unwrap();
        manager
            .sync("release:a", Some(&previous), &BTreeSet::new())
            .unwrap();
        // Preserve a valid user's formatting, not a reconstructed selection.
        let mut prior_bytes = serde_json::to_vec(&previous).unwrap();
        prior_bytes.push(b'\n');
        fs::write(project.join(".mirket/project.json"), &prior_bytes).unwrap();
        assert_eq!(
            ProjectSelection::saved_bytes(&project).unwrap().unwrap(),
            prior_bytes
        );
        let before = manager.inspect().unwrap();
        let before_receipt = fs::read(&manager.receipt_path).unwrap();
        let next = ProjectSelection::resolve(&catalog, &[], &["browser-qa".into()], &[]).unwrap();
        assert_eq!(
            next.replace_for_sync(&project, &catalog, &prior_bytes)
                .unwrap(),
            SelectionWriteOutcome::Replaced
        );
        let error = manager
            .sync_inner(
                "release:b",
                Some(&next),
                &BTreeSet::from(["browser".into()]),
                false,
                Some(1),
            )
            .unwrap_err();
        assert!(error.contains("rolled back"));
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), before_receipt);
        assert_eq!(manager.inspect().unwrap().status, "conflict");
        assert!(
            manager
                .restore_selection_if_native_unchanged(&before, &next, Some(&prior_bytes))
                .unwrap()
        );
        assert_eq!(
            ProjectSelection::saved_bytes(&project).unwrap().unwrap(),
            prior_bytes
        );
        assert_eq!(manager.inspect().unwrap().status, "managed");
        // A lost restoration acknowledgement is safe to reconcile.
        ProjectSelection::restore_exact(&project, &next, Some(&prior_bytes)).unwrap();
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), before_receipt);
    }

    #[test]
    fn failed_activation_does_not_restore_over_another_home_native_commit() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let manager = fixture.project();
        let previous =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        previous.save(&project, &catalog).unwrap();
        manager
            .sync("release:a", Some(&previous), &BTreeSet::new())
            .unwrap();
        let before = manager.inspect().unwrap();
        let prior_bytes = ProjectSelection::saved_bytes(&project).unwrap().unwrap();
        let next = ProjectSelection::resolve(&catalog, &[], &["browser-qa".into()], &[]).unwrap();
        next.replace_for_sync(&project, &catalog, &prior_bytes)
            .unwrap();
        assert!(
            manager
                .sync_inner(
                    "release:b",
                    Some(&next),
                    &BTreeSet::from(["browser".into()]),
                    false,
                    Some(1),
                )
                .unwrap_err()
                .contains("rolled back")
        );
        // This models a second installation home committing B after our failed
        // sync but before our recovery decision.
        fixture
            .project()
            .sync(
                "release:b",
                Some(&next),
                &BTreeSet::from(["browser".into()]),
            )
            .unwrap();
        assert!(
            !manager
                .restore_selection_if_native_unchanged(&before, &next, Some(&prior_bytes))
                .unwrap()
        );
        assert_eq!(
            ProjectSelection::saved_bytes(&project).unwrap().unwrap(),
            serde_json::to_vec_pretty(&next).unwrap()
        );
        assert_eq!(manager.inspect().unwrap().status, "managed");
    }

    #[test]
    fn unchanged_project_sync_cannot_acknowledge_an_inflight_selection() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let manager = fixture.project();
        let previous =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        previous.save(&project, &catalog).unwrap();
        manager
            .sync("release:a", Some(&previous), &BTreeSet::new())
            .unwrap();
        let next = ProjectSelection::resolve(&catalog, &["library-cli".into()], &[], &[]).unwrap();
        assert_eq!(previous.selected, next.selected);
        let prior_bytes = ProjectSelection::saved_bytes(&project).unwrap().unwrap();
        let _operation = manager.project_operation_guard().unwrap();
        next.replace_for_sync(&project, &catalog, &prior_bytes)
            .unwrap();
        let refused = fixture
            .project()
            .sync("release:a", Some(&next), &BTreeSet::new())
            .unwrap_err();
        assert!(refused.contains("another native skill operation owns the lock"));
        drop(_operation);
        assert_eq!(
            fixture
                .project()
                .sync("release:a", Some(&next), &BTreeSet::new())
                .unwrap()
                .status,
            "unchanged"
        );
    }

    #[test]
    fn initial_selection_rollback_removes_only_the_owned_pin() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        assert!(ProjectSelection::saved_bytes(&project).unwrap().is_none());
        ProjectSelection::restore_exact(&project, &selection, None).unwrap();
        assert!(!project.join(".mirket").exists());
        assert_eq!(
            selection.save_for_sync(&project, &catalog, None).unwrap(),
            SelectionWriteOutcome::Replaced
        );
        let unrelated = project.join(".mirket/keep.txt");
        fs::write(&unrelated, "Unrelated project state.\n").unwrap();
        ProjectSelection::restore_exact(&project, &selection, None).unwrap();
        assert!(ProjectSelection::saved_bytes(&project).unwrap().is_none());
        assert!(project.join(".mirket").is_dir());
        assert!(project.join(".mirket/project.lock").is_file());
        assert_eq!(
            fs::read_to_string(&unrelated).unwrap(),
            "Unrelated project state.\n"
        );
        ProjectSelection::restore_exact(&project, &selection, None).unwrap();
    }

    #[test]
    fn exact_selection_write_refuses_intervening_bytes_before_own_effect() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let path = project.join(".mirket/project.json");
        let previous =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        let requested =
            ProjectSelection::resolve(&catalog, &[], &["browser-qa".into()], &[]).unwrap();
        previous.save(&project, &catalog).unwrap();
        let snapshot = ProjectSelection::saved_bytes(&project).unwrap().unwrap();

        // A formatting-only user edit is still a different exact prior pin.
        let formatted = serde_json::to_vec(&previous).unwrap();
        fs::write(&path, &formatted).unwrap();
        let error = requested
            .replace_for_sync(&project, &catalog, &snapshot)
            .unwrap_err();
        assert!(!error.replaced);
        assert_eq!(fs::read(&path).unwrap(), formatted);

        // Even the requested canonical bytes can belong to another writer;
        // this failed comparison must not issue a rollback token.
        let other_writer = serde_json::to_vec_pretty(&requested).unwrap();
        fs::write(&path, &other_writer).unwrap();
        let error = requested
            .replace_for_sync(&project, &catalog, &snapshot)
            .unwrap_err();
        assert!(!error.replaced);
        assert_eq!(fs::read(&path).unwrap(), other_writer);
    }

    #[test]
    fn tracked_replace_distinguishes_post_rename_sync_failure() {
        let fixture = Fixture::new();
        let path = fixture.0.join("state/pin.json");
        fs::write(&path, b"old").unwrap();
        let mut renamed = false;
        let error = atomic_replace_tracked_with(&path, b"new", &mut renamed, || {
            Err("injected parent sync failure".into())
        })
        .unwrap_err();
        assert!(error.contains("injected parent sync failure"));
        assert!(renamed);
        assert_eq!(fs::read(&path).unwrap(), b"new");
    }

    #[test]
    fn selection_rollback_preserves_unknown_bytes_and_enforces_bounds() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let path = project.join(".mirket/project.json");
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        // Even semantically identical formatting is an unknown byte change.
        let edited = serde_json::to_vec(&selection).unwrap();
        fs::write(&path, &edited).unwrap();
        assert!(ProjectSelection::restore_exact(&project, &selection, None).is_err());
        assert_eq!(fs::read(&path).unwrap(), edited);
        let oversized = vec![b' '; MAX_STATE_BYTES as usize + 1];
        assert!(ProjectSelection::restore_exact(&project, &selection, Some(&oversized)).is_err());
        assert_eq!(fs::read(&path).unwrap(), edited);
        fs::write(&path, &oversized).unwrap();
        assert!(ProjectSelection::saved_bytes(&project).is_err());
        assert!(ProjectSelection::restore_exact(&project, &selection, None).is_err());
        assert_eq!(fs::metadata(&path).unwrap().len(), oversized.len() as u64);
    }

    #[cfg(unix)]
    #[test]
    fn selection_rollback_refuses_file_and_parent_symlink_substitutions() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let path = project.join(".mirket/project.json");
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        let expected = fs::read(&path).unwrap();
        let outside = fixture.0.join("state/outside-selection.json");
        fs::write(&outside, &expected).unwrap();
        fs::remove_file(&path).unwrap();
        symlink(&outside, &path).unwrap();
        assert!(ProjectSelection::saved_bytes(&project).is_err());
        assert!(ProjectSelection::restore_exact(&project, &selection, None).is_err());
        assert!(
            fs::symlink_metadata(&path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(&outside).unwrap(), expected);
        fs::remove_file(&path).unwrap();
        let saved_directory = project.join("saved-mirket");
        fs::rename(project.join(".mirket"), &saved_directory).unwrap();
        fs::write(saved_directory.join("project.json"), &expected).unwrap();
        symlink(&saved_directory, project.join(".mirket")).unwrap();
        assert!(ProjectSelection::saved_bytes(&project).is_err());
        assert!(ProjectSelection::restore_exact(&project, &selection, None).is_err());
        assert_eq!(
            fs::read(saved_directory.join("project.json")).unwrap(),
            expected
        );
    }

    #[test]
    fn project_inspection_reports_invalid_or_unapplied_selection_without_writing() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let manager = fixture.project();
        let path = project.join(".mirket/project.json");
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        manager
            .sync("release:a", Some(&selection), &BTreeSet::new())
            .unwrap();
        let receipt = fs::read(&manager.receipt_path).unwrap();
        let package = snapshot(&manager.destination.join("architecture"))
            .unwrap()
            .record;
        let next = ProjectSelection::resolve(&catalog, &[], &["browser-qa".into()], &[]).unwrap();
        next.replace(&project, &catalog, &selection).unwrap();
        let report = manager.inspect().unwrap();
        assert_eq!(report.status, "conflict");
        assert!(report.errors[0].contains("differs from the committed"));
        for invalid in [
            b"not JSON".as_slice(),
            br#"{"schema_version":1,"profiles":[],"include":[],"exclude":[],"selected":["unknown-skill"]}"#,
        ] {
            fs::write(&path, invalid).unwrap();
            let report = manager.inspect().unwrap();
            assert_eq!(report.status, "conflict");
            assert!(report.errors[0].contains("invalid saved project skill selection"));
            assert_eq!(fs::read(&path).unwrap(), invalid);
        }
        fs::remove_file(&path).unwrap();
        let report = manager.inspect().unwrap();
        assert_eq!(report.status, "conflict");
        assert!(report.errors[0].contains("selection is missing"));
        assert!(!path.exists());
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), receipt);
        assert_eq!(
            snapshot(&manager.destination.join("architecture"))
                .unwrap()
                .record,
            package
        );
    }

    #[test]
    fn pending_project_receipt_can_reconcile_despite_invalid_selection() {
        let fixture = Fixture::new();
        let catalog = fixture.catalog();
        let project = fixture.0.join("project");
        let manager = fixture.project();
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        let prepared = manager
            .prepare("release:a", Some(&selection), &BTreeSet::new(), false)
            .unwrap();
        fixture.begin(&manager, &prepared);
        manager.apply(&prepared, None).unwrap();
        fs::write(project.join(".mirket/project.json"), b"unknown edit").unwrap();
        assert_eq!(manager.inspect().unwrap().status, "pending");
        let report = manager.reconcile().unwrap();
        assert_eq!(report.status, "conflict");
        assert!(!report.pending_transaction);
        assert!(matches!(
            manager.read_receipt().unwrap(),
            Some(ReceiptFile::Committed { .. })
        ));
        assert!(manager.destination.join("architecture/SKILL.md").is_file());
        assert_eq!(
            fs::read(project.join(".mirket/project.json")).unwrap(),
            b"unknown edit"
        );
    }

    #[test]
    fn unmanaged_collision_is_rejected_before_any_other_skill_or_receipt_write() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        fs::create_dir_all(manager.destination.join("task-planning")).unwrap();
        fs::write(
            manager.destination.join("task-planning/SKILL.md"),
            "User skill.",
        )
        .unwrap();
        assert!(
            manager
                .sync("release:a", None, &BTreeSet::new())
                .unwrap_err()
                .contains("collision")
        );
        assert_eq!(manager.inspect().unwrap().status, "conflict");
        assert!(!manager.destination.join("outcome-verification").exists());
        assert!(!manager.receipt_path.exists());
        assert!(!manager.lock_path().exists());
        assert_eq!(
            fs::read_to_string(manager.destination.join("task-planning/SKILL.md")).unwrap(),
            "User skill."
        );
    }

    #[test]
    fn independent_receipts_cannot_mutate_one_native_destination_concurrently() {
        let fixture = Fixture::new();
        let first = fixture.global();
        let second = SkillSync::global(
            &fixture.catalog(),
            &fixture.0.join("user"),
            &fixture.0.join("state/other-installation.json"),
        )
        .unwrap();
        let mut created = Vec::new();
        ensure_directory(&first.destination, &mut created).unwrap();
        assert_eq!(
            first.destination_lock_path(),
            second.destination_lock_path()
        );
        let guard = acquire_lock(&first.destination_lock_path()).unwrap();
        assert!(second.sync("release:a", None, &BTreeSet::new()).is_err());
        assert!(!second.receipt_path.exists());
        drop(guard);

        first.sync("release:a", None, &BTreeSet::new()).unwrap();
        assert!(
            second
                .sync("release:a", None, &BTreeSet::new())
                .unwrap_err()
                .contains("collision")
        );
        assert_eq!(first.inspect().unwrap().status, "managed");
        assert!(!second.receipt_path.exists());
    }

    #[test]
    fn drift_in_any_managed_package_blocks_updates_and_removal() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        let before = fs::read(manager.destination.join("outcome-verification/SKILL.md")).unwrap();
        fs::write(
            manager.destination.join("task-planning/user-addition.txt"),
            "User note.",
        )
        .unwrap();
        fixture.update_source("outcome-verification");
        assert_eq!(manager.inspect().unwrap().status, "drifted");
        assert!(
            fixture
                .global()
                .sync("release:b", None, &BTreeSet::new())
                .is_err()
        );
        assert!(manager.remove_managed().is_err());
        assert_eq!(
            fs::read(manager.destination.join("outcome-verification/SKILL.md")).unwrap(),
            before
        );
        assert_eq!(
            fs::read_to_string(manager.destination.join("task-planning/user-addition.txt"))
                .unwrap(),
            "User note."
        );
    }

    #[test]
    fn partial_initial_and_update_failures_restore_all_owned_state() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        assert!(
            manager
                .sync_inner("release:a", None, &BTreeSet::new(), false, Some(1))
                .unwrap_err()
                .contains("rolled back")
        );
        assert!(!manager.receipt_path.exists());
        assert!(!manager.destination.join("outcome-verification").exists());
        assert!(!manager.transaction_path().exists());
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        let receipt = fs::read(&manager.receipt_path).unwrap();
        let original = record_at(&manager.destination.join("outcome-verification")).unwrap();
        fixture.update_source("task-planning");
        fixture.update_source("outcome-verification");
        assert!(
            fixture
                .global()
                .sync_inner("release:b", None, &BTreeSet::new(), false, Some(1))
                .unwrap_err()
                .contains("rolled back")
        );
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), receipt);
        assert_eq!(
            record_at(&manager.destination.join("outcome-verification")).unwrap(),
            original
        );
        assert_eq!(manager.inspect().unwrap().status, "managed");
    }

    #[test]
    fn interrupted_partial_apply_rolls_back_and_complete_apply_commits() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        let prepared = manager
            .prepare("release:a", None, &BTreeSet::new(), false)
            .unwrap();
        fixture.begin(&manager, &prepared);
        assert!(manager.apply(&prepared, Some(1)).is_err());
        assert_eq!(manager.inspect().unwrap().status, "pending");
        assert_eq!(manager.reconcile().unwrap().status, "missing");
        assert!(!manager.destination.join("outcome-verification").exists());
        let prepared = manager
            .prepare("release:a", None, &BTreeSet::new(), false)
            .unwrap();
        fixture.begin(&manager, &prepared);
        manager.apply(&prepared, None).unwrap();
        assert_eq!(manager.reconcile().unwrap().status, "managed");
        assert!(!manager.transaction_path().exists());
        assert!(!manager.inspect().unwrap().pending_transaction);
    }

    #[test]
    fn interrupted_removal_must_observe_absence_before_committing() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        let prepared = manager
            .prepare("release:a", None, &BTreeSet::new(), true)
            .unwrap();
        fixture.begin(&manager, &prepared);
        // The receipt alone must not turn a never-applied removal into success.
        assert_eq!(manager.reconcile().unwrap().skills.len(), 2);
        let prepared = manager
            .prepare("release:a", None, &BTreeSet::new(), true)
            .unwrap();
        fixture.begin(&manager, &prepared);
        manager.apply(&prepared, None).unwrap();
        let report = manager.reconcile().unwrap();
        assert!(report.skills.is_empty());
        assert!(!manager.destination.join("task-planning").exists());
    }

    #[test]
    fn partial_staging_can_be_removed_but_unknown_pending_edits_are_preserved() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        let prepared = manager
            .prepare("release:a", None, &BTreeSet::new(), false)
            .unwrap();
        fixture.begin(&manager, &prepared);
        fs::create_dir_all(
            manager
                .transaction_path()
                .join("after/task-planning/references"),
        )
        .unwrap();
        fs::write(
            manager
                .transaction_path()
                .join("after/task-planning/references/method.md"),
            &prepared.packages["task-planning"].bytes["references/method.md"],
        )
        .unwrap();
        assert_eq!(manager.reconcile().unwrap().status, "missing");
        let prepared = manager
            .prepare("release:a", None, &BTreeSet::new(), false)
            .unwrap();
        fixture.begin(&manager, &prepared);
        manager.apply(&prepared, None).unwrap();
        fs::write(
            manager.destination.join("task-planning/SKILL.md"),
            "Unknown concurrent edit.",
        )
        .unwrap();
        let receipt = fs::read(&manager.receipt_path).unwrap();
        assert!(manager.reconcile().is_err());
        assert_eq!(fs::read(&manager.receipt_path).unwrap(), receipt);
        assert_eq!(
            fs::read_to_string(manager.destination.join("task-planning/SKILL.md")).unwrap(),
            "Unknown concurrent edit."
        );
    }

    #[test]
    fn uninstall_removes_only_unchanged_receipted_packages() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        fs::create_dir(manager.destination.join("unrelated")).unwrap();
        fs::write(manager.destination.join("unrelated/SKILL.md"), "Unrelated.").unwrap();
        assert_eq!(manager.remove_managed().unwrap().status, "removed");
        assert!(!manager.destination.join("task-planning").exists());
        assert_eq!(
            fs::read_to_string(manager.destination.join("unrelated/SKILL.md")).unwrap(),
            "Unrelated."
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_destination_source_and_project_selection_are_refused() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        symlink(fixture.0.join("state"), fixture.0.join("user/.agents")).unwrap();
        assert!(
            SkillSync::global(
                &fixture.catalog(),
                &fixture.0.join("user"),
                &fixture.0.join("state/global-skills.json")
            )
            .is_err()
        );
        fs::remove_file(fixture.0.join("user/.agents")).unwrap();
        symlink(
            fixture.0.join("state"),
            fixture
                .0
                .join("release/skills/task-planning/references/escape"),
        )
        .unwrap();
        assert!(
            fixture
                .global()
                .plan("release:a", None, &BTreeSet::new())
                .is_err()
        );
        assert!(!fixture.0.join("user/.agents").exists());
        let selection =
            ProjectSelection::resolve(&fixture.catalog(), &[], &["architecture".into()], &[])
                .unwrap();
        symlink(fixture.0.join("state"), fixture.0.join("project/.mirket")).unwrap();
        assert!(
            selection
                .save(&fixture.0.join("project"), &fixture.catalog())
                .is_err()
        );
        assert!(!fixture.0.join("state/project.json").exists());
    }

    #[test]
    fn receipt_binding_and_selection_schema_cannot_redirect_owned_writes() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        let Some(ReceiptFile::Committed { mut receipt }) = manager.read_receipt().unwrap() else {
            panic!("missing receipt");
        };
        receipt.destination = fixture.0.join("project");
        manager
            .write_receipt(&ReceiptFile::Committed { receipt })
            .unwrap();
        assert!(manager.remove_managed().is_err());
        assert!(manager.destination.join("task-planning").exists());
        fs::create_dir(fixture.0.join("project/.mirket")).unwrap();
        fs::write(fixture.0.join("project/.mirket/project.json"), r#"{"schema_version":1,"profiles":[],"include":[],"exclude":[],"selected":[],"legacy_path":"escape"}"#).unwrap();
        assert!(ProjectSelection::load(&fixture.0.join("project")).is_err());
    }

    #[test]
    fn rechecks_source_identity_after_catalog_qualification_before_any_write() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        fixture.update_source("task-planning");
        let error = manager
            .plan("release:a", None, &BTreeSet::new())
            .unwrap_err();
        assert!(error.contains("source differs from the qualified catalog"));
        assert!(!fixture.0.join("user/.agents").exists());
        assert!(!manager.receipt_path.exists());
    }

    #[test]
    fn malformed_receipt_sizes_fail_closed_without_integer_overflow() {
        let fixture = Fixture::new();
        let manager = fixture.global();
        manager.sync("release:a", None, &BTreeSet::new()).unwrap();
        let Some(ReceiptFile::Committed { mut receipt }) = manager.read_receipt().unwrap() else {
            panic!("missing receipt");
        };
        let package = receipt.skills.get_mut("task-planning").unwrap();
        for file in package.files.values_mut() {
            file.bytes = u64::MAX;
        }
        package.source_sha256 = package_hash(package).unwrap();
        manager
            .write_receipt(&ReceiptFile::Committed { receipt })
            .unwrap();
        assert!(manager.inspect().is_err());
        assert!(manager.remove_managed().is_err());
        assert!(manager.destination.join("task-planning/SKILL.md").exists());
    }
}
