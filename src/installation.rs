//! Installation and health reporting for an explicitly selected `.mirket` home.
//!
//! This layer owns the small public setup/doctor contract.  Release identity,
//! payload verification, staging and maintenance-window activation remain in
//! [`crate::release_store::ReleaseStore`].  Setup never downloads files,
//! edits client configuration, or replaces a non-empty/unrecognized home.
//! The explicit Codex orchestration APIs additionally own native setup/update;
//! their reports establish saved state, not Desktop activation.

use crate::{
    codex_config::{
        CodexConfig, CodexConfigReport, CodexReconciliationResult, CodexRegistrationRequest,
        CodexRegistrationResult, CodexRemovalResult,
    },
    codex_guidance::{CodexGuidance, GuidanceReport},
    distribution::{ReleaseComponent, VerifiedRelease},
    maintenance::MaintenanceGuard,
    native_skills::QualifiedCatalog,
    release_store::{ActivationResult, ReleaseStore},
    skill_sync::{ProjectSelection, SkillInspection, SkillSync, SkillSyncReport},
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[path = "installation_uninstall.rs"]
mod uninstall;
pub use uninstall::UninstallReport;
#[path = "installation_reinstall.rs"]
mod reinstall;
pub(crate) use reinstall::ReinstallPermit;
pub use reinstall::ReinstallReport;

const MAX_NATIVE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Serialize)]
pub struct CodexSetupResult {
    pub core: SetupResult,
    pub user_home: PathBuf,
    pub skills: SkillSyncReport,
    pub project_skills: Option<SkillSyncReport>,
    pub guidance: GuidanceReport,
    pub registration: CodexRegistrationResult,
}

#[derive(Serialize)]
pub struct CodexUpdateResult {
    pub home: PathBuf,
    pub user_home: PathBuf,
    pub staged: ActivationResult,
    pub activation: ActivationResult,
    pub skills: Option<SkillSyncReport>,
    pub project_skills: Option<SkillSyncReport>,
    pub guidance: Option<GuidanceReport>,
    pub registration: Option<CodexRegistrationResult>,
}

#[derive(Serialize)]
pub struct CodexDoctorReport {
    pub status: &'static str,
    pub user_home: PathBuf,
    pub core: DoctorReport,
    pub skills: Option<SkillInspection>,
    pub guidance: Option<GuidanceReport>,
    pub registration: Option<CodexConfigReport>,
    pub errors: Vec<String>,
    pub desktop_activation_verified: bool,
    pub uninstall: Option<UninstallReport>,
    pub reinstall: Option<ReinstallReport>,
}

struct CodexPaths {
    user_home: PathBuf,
    directory: PathBuf,
    config: PathBuf,
    guidance: PathBuf,
}

#[derive(Clone, Eq, PartialEq)]
struct SavedFile {
    path: PathBuf,
    bytes: Option<Vec<u8>>,
    mode: Option<u32>,
}

#[derive(Clone, Eq, PartialEq)]
struct NativeSnapshot(Vec<SavedFile>);

#[derive(Clone, Copy, Eq, PartialEq)]
enum NativeStep {
    Skills,
    Guidance,
    Registration,
    ProjectSkills,
}

/// An initialized installation selected by an absolute private home path.
pub struct Installation {
    home: PathBuf,
    store: ReleaseStore,
}

/// Serializes Codex-native mutations with setup and update for one home.
pub struct CodexOperationGuard {
    _file: crate::file_lock::FileLock,
}

/// The result of a fresh local setup attempt.
#[derive(Serialize)]
pub struct SetupResult {
    pub home: PathBuf,
    pub staged: ActivationResult,
    pub activation: ActivationResult,
    pub launcher: Option<PathBuf>,
}

/// A read-only snapshot of the selected installation.
///
/// `manifest_healthy` is `None` when there is no active release yet.  When an
/// active release is present, it is true only after the complete installed
/// manifest and every payload digest/mode have been checked by
/// `VerifiedRelease::open_installed`.
#[derive(Clone, Debug, Serialize)]
pub struct DoctorReport {
    pub home: PathBuf,
    pub status: &'static str,
    pub active_release_id: Option<String>,
    pub active_components: Option<Vec<ReleaseComponent>>,
    pub active_version: Option<String>,
    pub active_target: Option<String>,
    pub manifest_healthy: Option<bool>,
    pub launcher_healthy: Option<bool>,
    pub errors: Vec<String>,
}

impl Installation {
    /// Create a new installation at an empty explicit home.
    pub fn initialize(home: &Path) -> Result<Self, String> {
        let selected = select_home(home)?;
        let store = ReleaseStore::initialize(&selected)?;
        let canonical = selected.canonicalize().map_err(io_message)?;
        Ok(Self {
            home: canonical,
            store,
        })
    }

    /// Open an existing Mirket installation without changing it.
    pub fn open(home: &Path) -> Result<Self, String> {
        let selected = select_home(home)?;
        let store = ReleaseStore::open(&selected)?;
        let canonical = selected.canonicalize().map_err(io_message)?;
        Ok(Self {
            home: canonical,
            store,
        })
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Return the stable launcher path owned by this installation.
    ///
    /// The launcher is a regular copied executable. It never points into a
    /// prunable release directory, so Codex configuration remains stable over
    /// update and rollback operations.
    pub fn launcher_path(&self) -> PathBuf {
        self.home.join("bin").join(if cfg!(windows) {
            "mirket.exe"
        } else {
            "mirket"
        })
    }

    pub fn codex_receipt_path(&self) -> PathBuf {
        self.home.join("state").join("codex-config.json")
    }

    pub fn codex_guidance_receipt_path(&self) -> PathBuf {
        self.home.join("state").join("codex-guidance.json")
    }

    pub fn codex_skills_receipt_path(&self) -> PathBuf {
        self.home.join("state").join("codex-skills.json")
    }

    /// Resolve native skill sources only from the verified active release.
    /// A release containing authored catalog data does not itself qualify any
    /// optional runtime; the capability set reflects packaged, admitted parts.
    pub fn skill_catalog(&self) -> Result<(String, QualifiedCatalog, BTreeSet<String>), String> {
        let active = self.store.open_active()?;
        launcher_health(&self.home, &active.release)?;
        Self::skill_catalog_from_release(&active.release)
    }

    fn skill_catalog_from_release(
        release: &VerifiedRelease,
    ) -> Result<(String, QualifiedCatalog, BTreeSet<String>), String> {
        let manifest = release.manifest();
        if !manifest.components.contains(&ReleaseComponent::Skills) {
            return Err("active release has no native skill catalog".into());
        }
        let catalog = QualifiedCatalog::open(release.root())?;
        catalog.release_shape()?;
        let qualified = qualified_capabilities(manifest);
        Ok((release.id().as_str().into(), catalog, qualified))
    }

    pub fn global_skill_sync(&self, user_home: &Path) -> Result<(String, SkillSync), String> {
        let (release_id, catalog, _) = self.skill_catalog()?;
        let manager = SkillSync::global(&catalog, user_home, &self.codex_skills_receipt_path())?;
        Ok((release_id, manager))
    }

    pub fn project_skill_sync(&self, project_root: &Path) -> Result<(String, SkillSync), String> {
        let (release_id, catalog, _) = self.skill_catalog()?;
        let manager = SkillSync::project(&catalog, project_root)?;
        Ok((release_id, manager))
    }

    pub fn install_codex_guidance(&self, guidance_path: &Path) -> Result<GuidanceReport, String> {
        let _operation = self.lock_codex_operation()?;
        let active = self.store.open_active()?;
        launcher_health(&self.home, &active.release)?;
        CodexGuidance::open(guidance_path, &self.codex_guidance_receipt_path())?
            .install(active.release.id().as_str())
    }

    pub fn inspect_codex_guidance(&self, guidance_path: &Path) -> Result<GuidanceReport, String> {
        CodexGuidance::open(guidance_path, &self.codex_guidance_receipt_path())?.inspect()
    }

    pub fn remove_codex_guidance(&self, guidance_path: &Path) -> Result<GuidanceReport, String> {
        let _operation = self.lock_codex_operation()?;
        CodexGuidance::open(guidance_path, &self.codex_guidance_receipt_path())?.remove()
    }

    pub fn reconcile_codex_guidance(&self, guidance_path: &Path) -> Result<GuidanceReport, String> {
        let _operation = self.lock_codex_operation()?;
        CodexGuidance::open(guidance_path, &self.codex_guidance_receipt_path())?.reconcile()
    }

    /// Register this installation's active release in an explicitly selected
    /// Codex config. The stable launcher must already have been published.
    pub fn register_codex(&self, config_path: &Path) -> Result<CodexRegistrationResult, String> {
        let _operation = self.lock_codex_operation()?;
        let active = self.store.open_active()?;
        launcher_health(&self.home, &active.release)?;
        CodexConfig::open(config_path, &self.codex_receipt_path())?.register(
            &CodexRegistrationRequest {
                release_id: active.release.id().as_str().to_owned(),
                home: self.home.clone(),
                launcher: self.launcher_path(),
            },
        )
    }

    /// Remove only the managed Mirket entry from an explicitly selected Codex
    /// config after verifying its receipt and current digest.
    pub fn remove_codex(&self, config_path: &Path) -> Result<CodexRemovalResult, String> {
        let _operation = self.lock_codex_operation()?;
        CodexConfig::open(config_path, &self.codex_receipt_path())?.remove()
    }

    /// Reconcile an interrupted Codex transaction without replaying unknown
    /// external changes.
    pub fn reconcile_codex(&self, config_path: &Path) -> Result<CodexReconciliationResult, String> {
        let _operation = self.lock_codex_operation()?;
        CodexConfig::open(config_path, &self.codex_receipt_path())?.reconcile()
    }

    /// Verify and stage a complete local release payload.
    pub fn stage(&self, source: &Path) -> Result<ActivationResult, String> {
        self.store.stage(source)
    }

    /// Activate the staged release during an idle maintenance window.
    pub fn activate(&self) -> Result<ActivationResult, String> {
        self.store.activate()
    }

    /// Activate the staged release and publish the stable launcher under one
    /// maintenance transaction. A launcher replacement failure leaves the
    /// existing active release and launcher unchanged.
    pub fn activate_with_launcher(&self) -> Result<ActivationResult, String> {
        self.store.activate_with_launcher(&self.launcher_path())
    }

    /// Initialize, stage and attempt activation of one local payload.
    ///
    /// A busy maintenance window is represented by the returned activation
    /// status `"staged"`; the active release remains unchanged and the caller
    /// may retry activation explicitly.
    pub fn setup(home: &Path, source: &Path) -> Result<SetupResult, String> {
        // Reject a missing or corrupt source before claiming a fresh home.
        let source_release = VerifiedRelease::open(source).map_err(io_message)?;
        let installation = match Self::initialize(home) {
            Ok(installation) => installation,
            Err(initial_error) => {
                let existing = Self::open(home).map_err(|_| initial_error)?;
                let report = Self::doctor(existing.home());
                let launcher_missing = fs::symlink_metadata(existing.launcher_path())
                    .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
                let incomplete = launcher_missing
                    && (report.status == "ready"
                        || (report.active_release_id.as_deref()
                            == Some(source_release.id().as_str())
                            && report.manifest_healthy == Some(true)
                            && report.launcher_healthy == Some(false)));
                if !incomplete {
                    return Err("setup refuses an existing complete or drifted installation".into());
                }
                existing
            }
        };
        let staged = installation.stage(source)?;
        let activation = installation.activate_with_launcher()?;
        let launcher = if activation.status == "active" {
            Some(installation.launcher_path())
        } else {
            None
        };
        Ok(SetupResult {
            home: installation.home,
            staged,
            activation,
            launcher,
        })
    }

    /// Fresh Codex-only installation. Every native destination is inspected
    /// before claiming the installation. Project skills sync after global
    /// owners while the maintenance guard excludes new MCP sessions. A failed
    /// native step preserves the owned core for diagnosis.
    pub fn setup_codex(
        home: &Path,
        source: &Path,
        user_home: &Path,
    ) -> Result<CodexSetupResult, String> {
        Self::setup_codex_with_project(home, source, user_home, None)
    }

    pub fn setup_codex_with_project(
        home: &Path,
        source: &Path,
        user_home: &Path,
        project_root: Option<&Path>,
    ) -> Result<CodexSetupResult, String> {
        Self::setup_codex_inner_with_project(home, source, user_home, project_root, None)
    }

    #[cfg(test)]
    fn setup_codex_inner(
        home: &Path,
        source: &Path,
        user_home: &Path,
        fail: Option<NativeStep>,
    ) -> Result<CodexSetupResult, String> {
        Self::setup_codex_inner_with_project(home, source, user_home, None, fail)
    }

    fn setup_codex_inner_with_project(
        home: &Path,
        source: &Path,
        user_home: &Path,
        project_root: Option<&Path>,
        fail: Option<NativeStep>,
    ) -> Result<CodexSetupResult, String> {
        let paths = CodexPaths::resolve(user_home, home)?;
        let source_release = VerifiedRelease::open(source).map_err(io_message)?;
        let existing = match Self::open(home) {
            Ok(installation) => {
                let report = Self::doctor(installation.home());
                if report.status != "ready" {
                    return Err(
                        "Codex setup refuses an existing complete or drifted installation".into(),
                    );
                }
                Some(installation)
            }
            Err(_) => {
                require_empty_home(home)?;
                None
            }
        };
        let catalog = release_catalog(&source_release)?;
        let qualified = qualified_capabilities(source_release.manifest());
        // These owners inspect the exact real files. A private temporary
        // receipt parent lets fresh setup preflight ownership without writing
        // installation state or inventing a second config/marker parser.
        let preflight = PreflightReceipts::new()?;
        let project_selection = preflight_native(
            &paths,
            &catalog,
            source_release.id().as_str(),
            home,
            &preflight.0,
            project_root,
            &qualified,
        )?;
        paths.create_directory()?;
        let installation = match existing {
            Some(installation) => installation,
            None => Self::initialize(home)?,
        };
        let staged = installation.stage(source)?;
        let _orchestration = installation.codex_operation_lock()?;
        let maintenance = MaintenanceGuard::try_acquire(&installation.home.join("state"))
            .map_err(|error| format!("Codex setup is staged; maintenance is busy: {error}"))?;
        let activation = installation
            .store
            .activate_with_launcher_held(&installation.launcher_path(), &maintenance)?;
        let core = SetupResult {
            home: installation.home.clone(),
            launcher: (activation.status == "active").then(|| installation.launcher_path()),
            staged,
            activation,
        };
        if core.activation.status != "active" {
            return Err("fresh setup did not activate; the owned installation was preserved for an explicit retry".into());
        }
        let active = installation
            .store
            .inspect_active()?
            .ok_or("fresh setup has no active release")?;
        let (release_id, active_catalog, _) = Self::skill_catalog_from_release(&active)?;
        let manager = SkillSync::global(
            &active_catalog,
            &paths.user_home,
            &installation.codex_skills_receipt_path(),
        )?;
        let before = NativeSnapshot::read(&installation, &paths)?;
        let mut known = before.clone();
        let native: Result<_, String> = (|| {
            let skills = manager.sync(&release_id, None, &BTreeSet::new())?;
            known = NativeSnapshot::read(&installation, &paths)?;
            injected_failure(fail, NativeStep::Skills, &installation.home.join("state"))?;
            let guidance =
                CodexGuidance::open(&paths.guidance, &installation.codex_guidance_receipt_path())?
                    .install(&release_id)?;
            known = NativeSnapshot::read(&installation, &paths)?;
            injected_failure(fail, NativeStep::Guidance, &installation.home.join("state"))?;
            let registration =
                CodexConfig::open(&paths.config, &installation.codex_receipt_path())?.register(
                    &CodexRegistrationRequest {
                        release_id: release_id.clone(),
                        home: installation.home.clone(),
                        launcher: installation.launcher_path(),
                    },
                )?;
            known = NativeSnapshot::read(&installation, &paths)?;
            injected_failure(
                fail,
                NativeStep::Registration,
                &installation.home.join("state"),
            )?;
            let project_skills = if let (Some(project_root), Some(selection)) =
                (project_root, project_selection.as_ref())
            {
                let project_manager = SkillSync::project(&active_catalog, project_root)?;
                let result = project_manager.sync(
                    &release_id,
                    Some(&selection.selection),
                    &qualified_capabilities(active.manifest()),
                )?;
                injected_failure(
                    fail,
                    NativeStep::ProjectSkills,
                    &installation.home.join("state"),
                )?;
                Some(result)
            } else {
                None
            };
            Ok((skills, project_skills, guidance, registration))
        })();
        match native {
            Ok((skills, project_skills, guidance, registration)) => Ok(CodexSetupResult {
                core,
                user_home: paths.user_home,
                skills,
                project_skills,
                guidance,
                registration,
            }),
            Err(error) => {
                if let (Some(project_root), Some(preflight)) =
                    (project_root, project_selection.as_ref())
                    && let Err(project_state) = project_state_safe_for_rollback(
                        &active_catalog,
                        project_root,
                        &preflight.inspection,
                    )
                {
                    return Err(format!(
                        "Codex setup partially completed: {error}; project skill state requires reconciliation: {project_state}; installation and native state preserved at {}",
                        installation.home.display()
                    ));
                }
                let rollback =
                    installation.rollback_fresh_native(&paths, &manager, &known, &before);
                Err(match rollback {
                    Ok(()) => format!(
                        "Codex setup failed: {error}; owned native changes rolled back; core installation preserved at {}; run doctor and explicitly install the missing native owners before update",
                        installation.home.display()
                    ),
                    Err(rollback) => format!(
                        "Codex setup partially completed: {error}; rollback refused: {rollback}; installation and pending receipts preserved at {}; run doctor and reconcile the affected native owner before retrying",
                        installation.home.display()
                    ),
                })
            }
        }
    }

    /// Explicit release update. A busy maintenance window leaves the release
    /// staged and returns before any native skill/guidance/config mutation.
    pub fn update_codex(
        &self,
        source: &Path,
        user_home: &Path,
    ) -> Result<CodexUpdateResult, String> {
        self.update_codex_with_project(source, user_home, None)
    }

    pub fn update_codex_with_project(
        &self,
        source: &Path,
        user_home: &Path,
        project_root: Option<&Path>,
    ) -> Result<CodexUpdateResult, String> {
        self.update_codex_inner_with_project(source, user_home, project_root, None)
    }

    #[cfg(test)]
    fn update_codex_inner(
        &self,
        source: &Path,
        user_home: &Path,
        fail: Option<NativeStep>,
    ) -> Result<CodexUpdateResult, String> {
        self.update_codex_inner_with_project(source, user_home, None, fail)
    }

    fn update_codex_inner_with_project(
        &self,
        source: &Path,
        user_home: &Path,
        project_root: Option<&Path>,
        fail: Option<NativeStep>,
    ) -> Result<CodexUpdateResult, String> {
        let paths = CodexPaths::resolve(user_home, &self.home)?;
        let _orchestration = self.codex_operation_lock()?;
        let before_report = Self::doctor_codex(&self.home, &paths.user_home);
        if before_report.status != "healthy" {
            return Err(format!(
                "Codex installation is not healthy; reconcile before update: {}",
                before_report.errors.join("; ")
            ));
        }
        let previous = self
            .store
            .inspect_active()?
            .ok_or("update requires an active release")?;
        let previous_id = previous.id().as_str().to_owned();
        let previous_catalog = release_catalog(&previous)?;
        let source_release = VerifiedRelease::open(source).map_err(io_message)?;
        let catalog = release_catalog(&source_release)?;
        let source_manager = SkillSync::global(
            &catalog,
            &paths.user_home,
            &self.codex_skills_receipt_path(),
        )?;
        source_manager.plan(source_release.id().as_str(), None, &BTreeSet::new())?;
        let qualified = qualified_capabilities(source_release.manifest());
        let project_selection = if let Some(project_root) = project_root {
            Some(preflight_project_skills(
                &catalog,
                source_release.id().as_str(),
                project_root,
                &qualified,
            )?)
        } else {
            None
        };
        let before = NativeSnapshot::read(self, &paths)?;
        let staged = self.stage(source)?;
        let maintenance = match MaintenanceGuard::try_acquire(&self.home.join("state")) {
            Ok(guard) => guard,
            Err(error) => {
                let activation = ActivationResult {
                    status: "staged",
                    release_id: staged.release_id.clone(),
                    reason: Some(error.to_string()),
                };
                return Ok(CodexUpdateResult {
                    home: self.home.clone(),
                    user_home: paths.user_home,
                    staged,
                    activation,
                    skills: None,
                    project_skills: None,
                    guidance: None,
                    registration: None,
                });
            }
        };
        if NativeSnapshot::read(self, &paths)? != before {
            return Err(
                "Codex native files changed during update staging; release remains staged".into(),
            );
        }
        let activation = self
            .store
            .activate_with_launcher_held(&self.launcher_path(), &maintenance)?;
        if activation.status != "active" {
            return Ok(CodexUpdateResult {
                home: self.home.clone(),
                user_home: paths.user_home,
                staged,
                activation,
                skills: None,
                project_skills: None,
                guidance: None,
                registration: None,
            });
        }
        let mut known = before.clone();
        let native: Result<_, String> = (|| {
            let active = self
                .store
                .inspect_active()?
                .ok_or("activated release is missing")?;
            let (release_id, active_catalog, _) = Self::skill_catalog_from_release(&active)?;
            let manager = SkillSync::global(
                &active_catalog,
                &paths.user_home,
                &self.codex_skills_receipt_path(),
            )?;
            let skills = manager.sync(&release_id, None, &BTreeSet::new())?;
            known = NativeSnapshot::read(self, &paths)?;
            injected_failure(fail, NativeStep::Skills, &self.home.join("state"))?;
            let guidance =
                CodexGuidance::open(&paths.guidance, &self.codex_guidance_receipt_path())?
                    .install(&release_id)?;
            known = NativeSnapshot::read(self, &paths)?;
            injected_failure(fail, NativeStep::Guidance, &self.home.join("state"))?;
            let registration = CodexConfig::open(&paths.config, &self.codex_receipt_path())?
                .register(&CodexRegistrationRequest {
                    release_id: release_id.clone(),
                    home: self.home.clone(),
                    launcher: self.launcher_path(),
                })?;
            known = NativeSnapshot::read(self, &paths)?;
            injected_failure(fail, NativeStep::Registration, &self.home.join("state"))?;
            let project_skills = if let (Some(project_root), Some(selection)) =
                (project_root, project_selection.as_ref())
            {
                let project_manager = SkillSync::project(&active_catalog, project_root)?;
                let result = project_manager.sync(
                    &release_id,
                    Some(&selection.selection),
                    &qualified_capabilities(active.manifest()),
                )?;
                injected_failure(fail, NativeStep::ProjectSkills, &self.home.join("state"))?;
                Some(result)
            } else {
                None
            };
            Ok((skills, project_skills, guidance, registration))
        })();
        match native {
            Ok((skills, project_skills, guidance, registration)) => Ok(CodexUpdateResult {
                home: self.home.clone(),
                user_home: paths.user_home,
                staged,
                activation,
                skills: Some(skills),
                project_skills,
                guidance: Some(guidance),
                registration: Some(registration),
            }),
            Err(error) => {
                // Failed owners may have their own pending journal. Refuse to
                // invent its outcome or restore unknown bytes across it.
                if let (Some(project_root), Some(preflight)) =
                    (project_root, project_selection.as_ref())
                    && let Err(project_state) = project_state_safe_for_rollback(
                        &catalog,
                        project_root,
                        &preflight.inspection,
                    )
                {
                    return Err(format!(
                        "Codex update partially completed: {error}; project skill state requires reconciliation: {project_state}; state and receipts preserved at {}",
                        self.home.display()
                    ));
                }
                let rollback = (|| {
                    if NativeSnapshot::read(self, &paths)? != known {
                        return Err("native state differs from the last completed step; inspect/reconcile its owner".into());
                    }
                    let manager = SkillSync::global(
                        &previous_catalog,
                        &paths.user_home,
                        &self.codex_skills_receipt_path(),
                    )?;
                    manager.sync(&previous_id, None, &BTreeSet::new())?;
                    before.restore_non_skill_files(self, &paths, &known)?;
                    self.store.stage_retained_previous(&previous_id)?;
                    let restored = self
                        .store
                        .activate_with_launcher_held(&self.launcher_path(), &maintenance)?;
                    if restored.status != "active" {
                        return Err("previous release is staged but rollback activation is busy; stop sessions and explicitly activate it".into());
                    }
                    Ok::<(), String>(())
                })();
                Err(match rollback {
                    Ok(()) => format!(
                        "Codex update failed: {error}; previous core and native state restored"
                    ),
                    Err(rollback) => format!(
                        "Codex update partially completed: {error}; rollback incomplete: {rollback}; state and receipts preserved at {}; run doctor before another explicit update",
                        self.home.display()
                    ),
                })
            }
        }
    }

    /// Full read-only saved-state inspection. No lock creation, reconciliation,
    /// project sync or Desktop activation is performed by this operation.
    pub fn doctor_codex(home: &Path, user_home: &Path) -> CodexDoctorReport {
        let core = Self::doctor(home);
        let mut report = CodexDoctorReport {
            status: "error",
            user_home: user_home.to_owned(),
            core,
            skills: None,
            guidance: None,
            registration: None,
            errors: Vec::new(),
            desktop_activation_verified: false,
            uninstall: None,
            reinstall: None,
        };
        if report.core.status == "reinstall_pending" {
            match Self::open(home).and_then(|installation| installation.reinstall_status(user_home))
            {
                Ok(reinstall) => {
                    report.status = reinstall.status;
                    report.reinstall = Some(reinstall);
                }
                Err(error) => report.errors.push(error),
            }
            return report;
        }
        if matches!(report.core.status, "uninstall_pending" | "uninstalled") {
            match Self::open(home).and_then(|installation| installation.uninstall_status(user_home))
            {
                Ok(uninstall) => {
                    report.status = uninstall.status;
                    report.uninstall = Some(uninstall);
                }
                Err(error) => report.errors.push(error),
            }
            return report;
        }
        report.errors.extend(report.core.errors.iter().cloned());
        let inspected = (|| {
            let paths = CodexPaths::resolve(user_home, home)?;
            report.user_home = paths.user_home.clone();
            let installation = Self::open(home)?;
            let release = installation
                .store
                .inspect_active()?
                .ok_or("installation has no active release")?;
            launcher_health(&installation.home, &release)?;
            let catalog = release_catalog(&release)?;
            let manager = SkillSync::global(
                &catalog,
                &paths.user_home,
                &installation.codex_skills_receipt_path(),
            )?;
            report.skills = Some(manager.inspect()?);
            report.guidance = Some(installation.inspect_codex_guidance(&paths.guidance)?);
            report.registration = Some(
                CodexConfig::open(&paths.config, &installation.codex_receipt_path())?.inspect()?,
            );
            if report
                .registration
                .as_ref()
                .is_some_and(|registration| registration.status == "managed")
            {
                verify_registration_target(&paths.config, &installation)?;
            }
            Ok::<(), String>(())
        })();
        if let Err(error) = inspected {
            report.errors.push(error);
        }
        if let Some(skills) = &report.skills {
            if skills.status != "managed"
                || skills.skills.len() != 9
                || skills.release_id != report.core.active_release_id
            {
                report.errors.push(format!(
                    "global skills are {} or do not match the active nine-skill release",
                    skills.status
                ));
            }
            report.errors.extend(skills.errors.iter().cloned());
            report
                .errors
                .extend(skills.skills.iter().filter_map(|skill| skill.error.clone()));
        }
        if let Some(guidance) = &report.guidance
            && (guidance.status != "managed"
                || guidance.release_id != report.core.active_release_id)
        {
            report.errors.push(format!(
                "Codex guidance is {} or belongs to a different release",
                guidance.status
            ));
        }
        if let Some(registration) = &report.registration {
            if registration.status != "managed"
                || registration.pending_transaction
                || registration.managed_release_id != report.core.active_release_id
            {
                report.errors.push(format!(
                    "Codex MCP registration is {} or belongs to a different release",
                    registration.status
                ));
            }
            report.errors.extend(registration.errors.iter().cloned());
        }
        if report.core.status == "healthy"
            && report.skills.is_some()
            && report.guidance.is_some()
            && report.registration.is_some()
            && report.errors.is_empty()
        {
            report.status = "healthy";
        }
        report
    }

    fn codex_operation_lock(&self) -> Result<crate::file_lock::FileLock, String> {
        let lock = native_lock(&self.home.join("state/codex-operation.lock"))?;
        reinstall::require_absent(&self.home)?;
        if uninstall::core_status(&self.home)?.is_some() {
            return Err(
                "installation has an uninstall record; use uninstall-status or uninstall-reconcile"
                    .into(),
            );
        }
        Ok(lock)
    }

    pub fn lock_codex_operation(&self) -> Result<CodexOperationGuard, String> {
        Ok(CodexOperationGuard {
            _file: self.codex_operation_lock()?,
        })
    }

    fn rollback_fresh_native(
        &self,
        paths: &CodexPaths,
        manager: &SkillSync,
        known: &NativeSnapshot,
        before: &NativeSnapshot,
    ) -> Result<(), String> {
        if NativeSnapshot::read(self, paths)? != *known {
            return Err("a failed owner has pending or externally changed bytes".into());
        }
        manager.remove_managed()?;
        before.restore_non_skill_files(self, paths, known)
    }

    /// Inspect a home without creating locks or any other files.
    pub fn doctor(home: &Path) -> DoctorReport {
        let display_home = home.to_owned();
        let selected = match select_home(home) {
            Ok(path) => path,
            Err(error) => return DoctorReport::error(display_home, error),
        };
        let store = match ReleaseStore::open(&selected) {
            Ok(store) => store,
            Err(error) => return DoctorReport::error(selected, error),
        };
        let canonical = match selected.canonicalize() {
            Ok(path) => path,
            Err(error) => return DoctorReport::error(selected, io_message(error)),
        };
        let lifecycle = reinstall::core_status(&canonical).and_then(|status| {
            if status.is_some() {
                Ok(status)
            } else {
                uninstall::core_status(&canonical)
            }
        });
        match lifecycle {
            Ok(Some(status)) => {
                return DoctorReport {
                    home: canonical,
                    status,
                    active_release_id: None,
                    active_components: None,
                    active_version: None,
                    active_target: None,
                    manifest_healthy: None,
                    launcher_healthy: None,
                    errors: Vec::new(),
                };
            }
            Ok(None) => {}
            Err(error) => return DoctorReport::error(canonical, error),
        }
        let release = match store.inspect_active() {
            Ok(Some(release)) => release,
            Ok(None) => {
                return DoctorReport {
                    home: canonical,
                    status: "ready",
                    active_release_id: None,
                    active_components: None,
                    active_version: None,
                    active_target: None,
                    manifest_healthy: None,
                    launcher_healthy: None,
                    errors: Vec::new(),
                };
            }
            Err(error) => return DoctorReport::active_error(canonical, error),
        };
        // Keep `store` alive until all selected-home validation is complete;
        // opening it above is the authoritative ownership/private-directory
        // check, and this inspection itself acquires no runtime lock.
        let manifest = release.manifest();
        let launcher_error = launcher_health(&canonical, &release).err();
        let mut errors = Vec::new();
        if let Some(error) = &launcher_error {
            errors.push(error.clone());
        }
        DoctorReport {
            home: canonical,
            status: if !errors.is_empty() {
                "error"
            } else {
                "healthy"
            },
            active_release_id: Some(release.id().as_str().to_owned()),
            active_components: Some(manifest.components.clone()),
            active_version: Some(manifest.version.clone()),
            active_target: Some(manifest.target.clone()),
            manifest_healthy: Some(true),
            launcher_healthy: Some(launcher_error.is_none()),
            errors,
        }
    }
}

fn qualified_capabilities(manifest: &crate::distribution::ReleaseManifest) -> BTreeSet<String> {
    let mut qualified = BTreeSet::new();
    if manifest.components.contains(&ReleaseComponent::Design) {
        qualified.extend([
            "navigation.js-ts".into(),
            "design.tokens".into(),
            "design.vector".into(),
        ]);
    }
    if manifest.components.contains(&ReleaseComponent::Design)
        && manifest.components.contains(&ReleaseComponent::Raster)
    {
        qualified.insert("design.raster".into());
    }
    if manifest.components.contains(&ReleaseComponent::OpenPencil) {
        qualified.insert("design.openpencil".into());
    }
    if manifest.components.contains(&ReleaseComponent::Diagram) {
        qualified.insert("design.diagrams".into());
    }
    if manifest.components.contains(&ReleaseComponent::Video) {
        qualified.insert("video".into());
    }
    qualified
}

impl DoctorReport {
    fn error(home: PathBuf, error: String) -> Self {
        Self {
            home,
            status: "error",
            active_release_id: None,
            active_components: None,
            active_version: None,
            active_target: None,
            manifest_healthy: None,
            launcher_healthy: None,
            errors: vec![error],
        }
    }
    fn active_error(home: PathBuf, error: String) -> Self {
        let mut report = Self::error(home, error);
        report.manifest_healthy = Some(false);
        report
    }
}

impl CodexPaths {
    fn resolve(user_home: &Path, home: &Path) -> Result<Self, String> {
        validate_native_path(user_home)?;
        let metadata = fs::symlink_metadata(user_home).map_err(io_message)?;
        if !metadata.is_dir() {
            return Err("explicit Codex user home must be a real directory".into());
        }
        let user_home = user_home.canonicalize().map_err(io_message)?;
        let directory = user_home.join(".codex");
        let paths = Self {
            config: directory.join("config.toml"),
            guidance: directory.join("AGENTS.md"),
            directory,
            user_home,
        };
        for path in [
            home.to_owned(),
            paths.config.clone(),
            paths.guidance.clone(),
            paths.user_home.join(".agents/skills"),
        ] {
            validate_native_path(&path)?;
        }
        if home == paths.user_home
            || paths.user_home.starts_with(home)
            || home.starts_with(&paths.directory)
            || home.starts_with(paths.user_home.join(".agents"))
        {
            return Err("Mirket installation and Codex native roots must not overlap".into());
        }
        Ok(paths)
    }

    fn create_directory(&self) -> Result<(), String> {
        match fs::symlink_metadata(&self.directory) {
            Ok(meta) if meta.is_dir() && !meta.file_type().is_symlink() => Ok(()),
            Ok(_) => Err("Codex directory is not a real directory".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_private_directory(&self.directory)
            }
            Err(error) => Err(io_message(error)),
        }
    }
}

fn require_empty_home(home: &Path) -> Result<(), String> {
    validate_native_path(home)?;
    match fs::symlink_metadata(home) {
        Ok(meta) if meta.is_dir() && fs::read_dir(home).map_err(io_message)?.next().is_none() => Ok(()),
        Ok(_) => Err("Codex setup requires an absent or empty private installation home; existing installations are preserved".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_message(error)),
    }
}

fn release_catalog(release: &VerifiedRelease) -> Result<QualifiedCatalog, String> {
    if !release
        .manifest()
        .components
        .contains(&ReleaseComponent::Skills)
    {
        return Err("Codex setup/update requires the native Skills release component".into());
    }
    let catalog = QualifiedCatalog::open(release.root())?;
    catalog.release_shape()?;
    Ok(catalog)
}

fn verify_registration_target(path: &Path, installation: &Installation) -> Result<(), String> {
    let snapshot = SavedFile::read(path)?;
    let text = String::from_utf8(snapshot.bytes.ok_or("Codex registration file is missing")?)
        .map_err(io_message)?;
    let document = text.parse::<toml_edit::DocumentMut>().map_err(io_message)?;
    let entry = document
        .get("mcp_servers")
        .and_then(|servers| servers.get("mirket"))
        .ok_or("Codex Mirket registration is missing")?;
    let command = entry
        .get("command")
        .and_then(toml_edit::Item::as_value)
        .and_then(|value| value.as_str());
    let arguments = entry
        .get("args")
        .and_then(toml_edit::Item::as_value)
        .and_then(|value| value.as_array())
        .ok_or("Codex Mirket registration has no arguments")?;
    let actual = arguments
        .iter()
        .map(|value| value.as_str())
        .collect::<Option<Vec<_>>>();
    let launcher = installation.launcher_path();
    if command != launcher.to_str()
        || actual
            != Some(vec![
                "serve",
                "--home",
                installation
                    .home
                    .to_str()
                    .ok_or("installation path must be UTF-8")?,
            ])
    {
        return Err("Codex MCP registration does not launch this installation's stable launcher and explicit home".into());
    }
    Ok(())
}

fn preflight_native(
    paths: &CodexPaths,
    catalog: &QualifiedCatalog,
    release_id: &str,
    home: &Path,
    receipts: &Path,
    project_root: Option<&Path>,
    qualified: &BTreeSet<String>,
) -> Result<Option<ProjectPreflight>, String> {
    SkillSync::global(
        catalog,
        &paths.user_home,
        &home.join("state/codex-skills.json"),
    )?
    .plan(release_id, None, &BTreeSet::new())?;
    if fs::symlink_metadata(&paths.directory).is_ok() {
        let config = CodexConfig::open(&paths.config, &receipts.join("config.json"))?.inspect()?;
        if config.status != "missing" {
            return Err(format!(
                "Codex config preflight refused {} Mirket registration",
                config.status
            ));
        }
        let guidance =
            CodexGuidance::open(&paths.guidance, &receipts.join("guidance.json"))?.inspect()?;
        if guidance.status != "missing" {
            return Err(format!(
                "Codex guidance preflight refused {} managed markers",
                guidance.status
            ));
        }
    }
    // Snapshotting exercises regular-file and size checks even when these
    // absent-parent paths cannot yet be opened by a native owner.
    SavedFile::read(&paths.config)?;
    SavedFile::read(&paths.guidance)?;
    project_root
        .map(|project_root| preflight_project_skills(catalog, release_id, project_root, qualified))
        .transpose()
}

fn preflight_project_skills(
    catalog: &QualifiedCatalog,
    release_id: &str,
    project_root: &Path,
    qualified: &BTreeSet<String>,
) -> Result<ProjectPreflight, String> {
    let selection = ProjectSelection::load(project_root)?
        .ok_or("--project-root requires an existing pinned project skill selection")?;
    let manager = SkillSync::project(catalog, project_root)?;
    let plan = manager.plan(release_id, Some(&selection), qualified)?;
    if !plan.unavailable.is_empty() {
        return Err(format!(
            "project selection requires unavailable private capabilities: {}",
            plan.unavailable
                .iter()
                .map(|entry| format!("{} ({})", entry.id, entry.missing_capabilities.join(", ")))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    let inspection = manager.inspect()?;
    if !matches!(inspection.status, "missing" | "managed") {
        return Err(format!(
            "project skill state is {}; reconcile drift or a pending transaction before setup/update",
            inspection.status
        ));
    }
    Ok(ProjectPreflight {
        selection,
        inspection,
    })
}

struct ProjectPreflight {
    selection: ProjectSelection,
    inspection: SkillInspection,
}

fn project_state_safe_for_rollback(
    catalog: &QualifiedCatalog,
    project_root: &Path,
    expected: &SkillInspection,
) -> Result<(), String> {
    let inspection = SkillSync::project(catalog, project_root)?.inspect()?;
    if inspection != *expected {
        return Err(format!(
            "project skill state changed from preflight ({} / {:?}) to {} / {:?}{}",
            expected.status,
            expected.release_id,
            inspection.status,
            inspection.release_id,
            if inspection.errors.is_empty() {
                String::new()
            } else {
                format!(": {}", inspection.errors.join("; "))
            }
        ));
    }
    Ok(())
}

struct PreflightReceipts(PathBuf);
impl PreflightReceipts {
    fn new() -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let parent = std::env::temp_dir().canonicalize().map_err(io_message)?;
        let path = parent.join(format!(
            "mirket-codex-preflight-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        create_private_directory(&path)?;
        Ok(Self(path))
    }
}
impl Drop for PreflightReceipts {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

impl NativeSnapshot {
    fn read(installation: &Installation, paths: &CodexPaths) -> Result<Self, String> {
        [
            paths.config.clone(),
            paths.guidance.clone(),
            installation.codex_receipt_path(),
            installation.codex_guidance_receipt_path(),
            installation.codex_skills_receipt_path(),
        ]
        .iter()
        .map(|path| SavedFile::read(path))
        .collect::<Result<Vec<_>, _>>()
        .map(Self)
    }

    fn restore_non_skill_files(
        &self,
        installation: &Installation,
        paths: &CodexPaths,
        expected: &Self,
    ) -> Result<(), String> {
        let _config = native_lock(&installation.codex_receipt_path().with_extension("lock"))?;
        let _guidance = native_lock(
            &installation
                .codex_guidance_receipt_path()
                .with_extension("lock"),
        )?;
        let observed = Self::read(installation, paths)?;
        let selected = self
            .0
            .iter()
            .zip(&expected.0)
            .zip(&observed.0)
            .filter(|((before, _), _)| before.path != installation.codex_skills_receipt_path())
            .collect::<Vec<_>>();
        // Complete proof precedes the first restoration. Each replacement also
        // rechecks its own bytes so a later edit remains visible.
        if selected
            .iter()
            .any(|((_, expected), observed)| expected != observed)
        {
            return Err("native rollback found unknown bytes; files and receipts preserved".into());
        }
        for ((before, expected), _) in selected {
            before.restore(expected)?;
        }
        Ok(())
    }
}

impl SavedFile {
    fn read(path: &Path) -> Result<Self, String> {
        validate_native_path(path)?;
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self {
                    path: path.to_owned(),
                    bytes: None,
                    mode: None,
                });
            }
            Err(error) => return Err(io_message(error)),
        };
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_NATIVE_BYTES
        {
            return Err("Codex native file must be bounded and regular".into());
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(io_message)?
            .take(MAX_NATIVE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(io_message)?;
        if bytes.len() as u64 != metadata.len() {
            return Err("Codex native file changed during snapshot".into());
        }
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            Some(metadata.permissions().mode() & 0o777)
        };
        #[cfg(not(unix))]
        let mode = None;
        Ok(Self {
            path: path.to_owned(),
            bytes: Some(bytes),
            mode,
        })
    }

    fn restore(&self, expected: &Self) -> Result<(), String> {
        if Self::read(&self.path)? != *expected {
            return Err("native file changed before rollback; preserved".into());
        }
        if self == expected {
            return Ok(());
        }
        let Some(bytes) = &self.bytes else {
            if expected.bytes.is_some() {
                fs::remove_file(&self.path).map_err(io_message)?;
            }
            return Ok(());
        };
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let temporary = self.path.with_file_name(format!(
            ".mirket-rollback-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(self.mode.unwrap_or(0o600));
        }
        let mut file = options.open(&temporary).map_err(io_message)?;
        let result = (|| {
            file.write_all(bytes)
                .and_then(|_| file.sync_all())
                .map_err(io_message)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(self.mode.unwrap_or(0o600)))
                    .map_err(io_message)?;
            }
            if Self::read(&self.path)? != *expected {
                return Err("native file changed while staging rollback; preserved".into());
            }
            replace_native_path(&temporary, &self.path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

fn native_lock(path: &Path) -> Result<crate::file_lock::FileLock, String> {
    validate_native_path(path)?;
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
            let metadata = fs::symlink_metadata(path).map_err(io_message)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() != 0 {
                return Err("Codex operation lock must be an empty regular file".into());
            }
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(io_message)?
        }
        Err(error) => return Err(io_message(error)),
    };
    file.try_lock()
        .map_err(|_| "another Codex installation operation owns the lock")?;
    Ok(crate::file_lock::FileLock::acquired(file))
}

fn validate_native_path(path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(
            "Codex installation paths must be explicit absolute paths without traversal".into(),
        );
    }
    let mut ancestor = PathBuf::new();
    for part in path.components() {
        ancestor.push(part.as_os_str());
        if !ancestor.is_absolute() {
            continue;
        }
        match fs::symlink_metadata(&ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("symlink in Codex installation path".into());
            }
            Ok(metadata) if ancestor != path && !metadata.is_dir() => {
                return Err("Codex installation ancestor is not a directory".into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_message(error)),
        }
    }
    Ok(())
}

fn create_private_directory(path: &Path) -> Result<(), String> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).map_err(io_message)
}

fn injected_failure(
    selected: Option<NativeStep>,
    current: NativeStep,
    state_root: &Path,
) -> Result<(), String> {
    #[cfg(test)]
    if selected.is_some() {
        assert!(matches!(
            crate::maintenance::SessionGuard::acquire(state_root),
            Err(crate::maintenance::MaintenanceError::Busy)
        ));
    }
    let _ = state_root;
    if selected == Some(current) {
        Err("injected native orchestration failure".into())
    } else {
        Ok(())
    }
}

fn replace_native_path(from: &Path, to: &Path) -> Result<(), String> {
    #[cfg(not(windows))]
    {
        fs::rename(from, to).map_err(io_message)?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let from = from
            .as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        let to = to.as_os_str().encode_wide().chain([0]).collect::<Vec<_>>();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            return Err(io_message(std::io::Error::last_os_error()));
        }
    }
    #[cfg(unix)]
    File::open(to.parent().ok_or("native file has no parent")?)
        .and_then(|file| file.sync_all())
        .map_err(io_message)?;
    Ok(())
}

fn launcher_health(home: &Path, active: &VerifiedRelease) -> Result<(), String> {
    let launcher = home.join("bin").join(if cfg!(windows) {
        "mirket.exe"
    } else {
        "mirket"
    });
    let metadata = fs::symlink_metadata(&launcher).map_err(io_message)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("stable launcher is missing or not a regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err("stable launcher is not executable".into());
        }
    }
    if fs::read(&launcher).map_err(io_message)?
        != fs::read(active.executable()).map_err(io_message)?
    {
        return Err("stable launcher bytes do not match the active core".into());
    }
    Ok(())
}

fn select_home(home: &Path) -> Result<PathBuf, String> {
    if !home.is_absolute()
        || home
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err("installation home must be an absolute path without traversal".into());
    }
    Ok(home.to_owned())
}

fn io_message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distribution::{ReleaseComponent, ReleaseManifest, native_target, write_manifest};
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    pub(super) struct Fixture(pub(super) PathBuf);
    impl Fixture {
        pub(super) fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "mirket-installation-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path.canonicalize().unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn payload(fixture: &Fixture) -> PathBuf {
        let root = fixture.0.join("payload");
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::create_dir_all(root.join("runtime/node/bin")).unwrap();
        let target = native_target().unwrap();
        let executable = if target.contains("windows") {
            "bin/mirket.exe"
        } else {
            "bin/mirket"
        };
        let node = if target.contains("windows") {
            "runtime/node/node.exe"
        } else {
            "runtime/node/bin/node"
        };
        for (path, contents) in [
            (executable, "core"),
            (node, "node"),
            ("runtime/node/LICENSE", "license"),
        ] {
            let path = root.join(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(root.join(executable), fs::Permissions::from_mode(0o700)).unwrap();
            fs::set_permissions(root.join(node), fs::Permissions::from_mode(0o700)).unwrap();
            fs::set_permissions(
                root.join("runtime/node/LICENSE"),
                fs::Permissions::from_mode(0o600),
            )
            .unwrap();
        }
        write_manifest(&root, "0.0.1", target).unwrap();
        root
    }

    #[test]
    fn doctor_reports_fresh_home_without_writing_runtime_locks() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let installation = Installation::initialize(&home).unwrap();
        let before = fs::read_dir(&home).unwrap().count();
        let report = Installation::doctor(installation.home());
        assert_eq!(report.status, "ready");
        assert_eq!(report.manifest_healthy, None);
        assert!(report.errors.is_empty());
        assert_eq!(fs::read_dir(&home).unwrap().count(), before);
        assert!(!home.join("state/runtime.lock").exists());
    }

    #[test]
    fn openpencil_component_qualifies_openpencil_skill_capability() {
        let manifest = ReleaseManifest {
            schema_version: 3,
            version: "0.0.1".into(),
            target: native_target().unwrap().to_string(),
            components: vec![ReleaseComponent::Skills, ReleaseComponent::OpenPencil],
            files: Vec::new(),
            links: Vec::new(),
        };
        let qualified = qualified_capabilities(&manifest);
        assert!(qualified.contains("design.openpencil"));
        assert!(!qualified.contains("design.tokens"));
    }

    #[test]
    fn raster_skill_requires_both_explicit_release_components() {
        let mut manifest = ReleaseManifest {
            schema_version: 3,
            version: "0.0.1".into(),
            target: native_target().unwrap().to_string(),
            components: vec![ReleaseComponent::Design],
            files: Vec::new(),
            links: Vec::new(),
        };
        assert!(!qualified_capabilities(&manifest).contains("design.raster"));
        manifest.components = vec![ReleaseComponent::Raster];
        assert!(!qualified_capabilities(&manifest).contains("design.raster"));
        manifest.components.push(ReleaseComponent::Design);
        assert!(qualified_capabilities(&manifest).contains("design.raster"));
    }

    #[test]
    fn video_skill_requires_the_explicit_video_component() {
        let mut manifest = ReleaseManifest {
            schema_version: 3,
            version: "0.0.1".into(),
            target: native_target().unwrap().to_string(),
            components: vec![ReleaseComponent::Skills],
            files: Vec::new(),
            links: Vec::new(),
        };
        assert!(!qualified_capabilities(&manifest).contains("video"));
        manifest.components.push(ReleaseComponent::Video);
        assert!(qualified_capabilities(&manifest).contains("video"));
    }

    #[test]
    fn setup_stages_activates_and_doctor_reports_exact_manifest() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        let source = payload(&fixture);
        let result = Installation::setup(&home, &source).unwrap();
        assert_eq!(result.staged.status, "staged");
        assert_eq!(result.activation.status, "active");
        let launcher = result.launcher.clone().unwrap();
        assert_eq!(launcher, Installation::open(&home).unwrap().launcher_path());
        assert_eq!(fs::read(launcher).unwrap(), b"core");
        let report = Installation::doctor(&home);
        assert_eq!(report.status, "healthy");
        assert_eq!(report.active_release_id, Some(result.activation.release_id));
        assert_eq!(report.active_components, Some(Vec::new()));
        assert_eq!(report.active_version.as_deref(), Some("0.0.1"));
        assert_eq!(report.active_target.as_deref(), native_target().ok());
        assert_eq!(report.manifest_healthy, Some(true));
        assert_eq!(report.launcher_healthy, Some(true));
        assert!(report.errors.is_empty());
        assert!(!home.join("state/activation.pending.json").exists());
        assert!(!home.join("state/launcher.previous").exists());
    }

    #[test]
    fn doctor_refuses_modified_active_payload_without_repairing_it() {
        let fixture = Fixture::new();
        let source = payload(&fixture);
        let home = fixture.0.join("home");
        Installation::setup(&home, &source).unwrap();
        let active = Installation::open(&home)
            .unwrap()
            .store
            .inspect_active()
            .unwrap()
            .unwrap();
        let installed_node = active.root().join(if cfg!(windows) {
            "runtime/node/node.exe"
        } else {
            "runtime/node/bin/node"
        });
        fs::write(&installed_node, "modified").unwrap();
        let before = fs::read(&installed_node).unwrap();
        let report = Installation::doctor(&home);
        assert_eq!(report.status, "error");
        assert_eq!(report.manifest_healthy, Some(false));
        assert!(report.active_release_id.is_none());
        assert_eq!(fs::read(installed_node).unwrap(), before);
    }

    #[test]
    fn doctor_reports_stable_launcher_drift_without_repairing_it() {
        let fixture = Fixture::new();
        let source = payload(&fixture);
        let home = fixture.0.join("home");
        Installation::setup(&home, &source).unwrap();
        let launcher = Installation::open(&home).unwrap().launcher_path();
        fs::write(&launcher, "drifted").unwrap();
        let before = fs::read(&launcher).unwrap();
        let report = Installation::doctor(&home);
        assert_eq!(report.status, "error");
        assert_eq!(report.manifest_healthy, Some(true));
        assert_eq!(report.launcher_healthy, Some(false));
        assert_eq!(fs::read(launcher).unwrap(), before);
    }

    #[test]
    fn setup_rejects_nonempty_home_without_modification() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        fs::create_dir_all(&home).unwrap();
        fs::write(home.join("keep"), "user data").unwrap();
        let source = payload(&fixture);
        assert!(Installation::setup(&home, &source).is_err());
        assert_eq!(fs::read(home.join("keep")).unwrap(), b"user data");
    }

    #[test]
    fn setup_resumes_only_owned_incomplete_home_with_same_source() {
        let fixture = Fixture::new();
        let source = payload(&fixture);
        let home = fixture.0.join("home");
        let installation = Installation::initialize(&home).unwrap();
        assert_eq!(Installation::doctor(&home).status, "ready");
        let resumed = Installation::setup(&home, &source).unwrap();
        assert_eq!(resumed.activation.status, "active");
        assert_eq!(Installation::doctor(&home).status, "healthy");
        assert!(Installation::setup(&home, &source).is_err());

        let other = fixture.0.join("other-home");
        let incomplete = Installation::initialize(&other).unwrap();
        incomplete.stage(&source).unwrap();
        incomplete.activate().unwrap();
        assert!(!incomplete.launcher_path().exists());
        let recovered = Installation::setup(&other, &source).unwrap();
        assert_eq!(recovered.activation.status, "active");
        assert_eq!(Installation::doctor(&other).status, "healthy");

        let drifted = installation.launcher_path();
        fs::write(&drifted, b"changed").unwrap();
        let before = fs::read(&drifted).unwrap();
        assert!(Installation::setup(&home, &source).is_err());
        assert_eq!(fs::read(&drifted).unwrap(), before);
    }

    #[test]
    fn setup_rejects_invalid_source_before_claiming_home() {
        let fixture = Fixture::new();
        let home = fixture.0.join("home");
        assert!(Installation::setup(&home, &fixture.0.join("missing")).is_err());
        assert!(!home.exists());
    }

    pub(super) fn codex_payload(fixture: &Fixture, core: &str) -> PathBuf {
        let slot = fixture.0.join("payload");
        if slot.exists() {
            fs::remove_dir_all(&slot).unwrap();
        }
        let root = payload(fixture);
        fs::remove_file(root.join(crate::distribution::MANIFEST_NAME)).unwrap();
        let sources = Path::new(env!("CARGO_MANIFEST_DIR"));
        let catalog = QualifiedCatalog::open(sources).unwrap();
        let catalog_files = [
            "skills/catalog.json".to_owned(),
            "skills/PROVENANCE.md".to_owned(),
        ]
        .into_iter()
        .chain(
            catalog
                .catalog
                .global
                .iter()
                .chain(&catalog.catalog.project)
                .flat_map(|entry| {
                    std::iter::once(entry.path.clone()).chain(
                        entry
                            .resources
                            .iter()
                            .map(|resource| format!("skills/{}/{}", entry.id, resource.path)),
                    )
                }),
        );
        for relative in catalog_files {
            let destination = root.join(&relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(sources.join(&relative), &destination).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(destination, fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
        fs::write(
            root.join(if cfg!(windows) {
                "bin/mirket.exe"
            } else {
                "bin/mirket"
            }),
            core,
        )
        .unwrap();
        write_manifest(&root, "0.0.1", native_target().unwrap()).unwrap();
        root
    }

    pub(super) fn codex_user(fixture: &Fixture, populated: bool) -> PathBuf {
        let user = fixture.0.join("user");
        create_private_directory(&user).unwrap();
        if populated {
            create_private_directory(&user.join(".codex")).unwrap();
            fs::write(
                user.join(".codex/config.toml"),
                "# User config\nmodel = 'chosen-model'\n",
            )
            .unwrap();
            fs::write(
                user.join(".codex/AGENTS.md"),
                "User instructions: preserve original wording.\n",
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    user.join(".codex/config.toml"),
                    fs::Permissions::from_mode(0o600),
                )
                .unwrap();
            }
        }
        user
    }

    #[test]
    fn codex_setup_creates_absent_codex_directory_and_full_doctor_is_read_only() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, false);
        let home = fixture.0.join("home");
        let result = Installation::setup_codex(&home, &source, &user).unwrap();
        assert_eq!(result.core.activation.status, "active");
        assert_eq!(result.skills.plan.selected.len(), 9);
        let config = fs::read_to_string(user.join(".codex/config.toml")).unwrap();
        assert!(config.contains("required = true"));
        let installation = Installation::open(&home).unwrap();
        let paths = CodexPaths::resolve(&user, &home).unwrap();
        let before = NativeSnapshot::read(&installation, &paths).unwrap();
        let state_count = fs::read_dir(home.join("state")).unwrap().count();
        let native_count = fs::read_dir(user.join(".codex")).unwrap().count();
        let report = Installation::doctor_codex(&home, &user);
        assert_eq!(report.status, "healthy", "{:?}", report.errors);
        assert!(!report.desktop_activation_verified);
        assert!(NativeSnapshot::read(&installation, &paths).unwrap() == before);
        assert_eq!(
            fs::read_dir(home.join("state")).unwrap().count(),
            state_count
        );
        assert_eq!(
            fs::read_dir(user.join(".codex")).unwrap().count(),
            native_count
        );
        assert!(
            installation.skill_catalog().unwrap().2.is_empty(),
            "Skills-only release must not claim a missing TypeScript runtime"
        );
    }

    #[test]
    fn codex_setup_resumes_owned_staged_home_after_activation_interruption() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, false);
        let home = fixture.0.join("home");
        let interrupted = Installation::initialize(&home).unwrap();
        let staged = interrupted.stage(&source).unwrap();
        assert_eq!(staged.status, "staged");
        assert!(!home.join("active.json").exists());
        assert!(home.join("staged.json").is_file());

        let resumed = Installation::setup_codex(&home, &source, &user).unwrap();
        assert_eq!(resumed.core.activation.status, "active");
        assert_eq!(resumed.core.activation.release_id, staged.release_id);
        assert_eq!(Installation::doctor_codex(&home, &user).status, "healthy");
    }

    #[test]
    fn codex_setup_resumes_same_candidate_after_launcher_publish_before_active_pointer() {
        use sha2::{Digest, Sha256};

        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let original_source = fixture.0.join("original-source");
        fs::rename(&source, &original_source).unwrap();
        let different_source = codex_payload(&fixture, "different");
        let user = codex_user(&fixture, false);
        let home = fixture.0.join("home");
        let interrupted = Installation::initialize(&home).unwrap();
        let staged = interrupted.stage(&original_source).unwrap();
        let candidate = home
            .join("releases")
            .join(format!(
                "r-{}",
                staged.release_id.trim_start_matches("sha256:")
            ))
            .join(if cfg!(windows) {
                "bin/mirket.exe"
            } else {
                "bin/mirket"
            });
        let launcher = interrupted.launcher_path();
        create_private_directory(launcher.parent().unwrap()).unwrap();
        fs::copy(&candidate, &launcher).unwrap();
        let digest = Sha256::digest(fs::read(&candidate).unwrap());
        let digest = format!(
            "sha256:{}",
            digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        let journal = serde_json::json!({
            "schema_version": 1,
            "candidate_release_id": staged.release_id,
            "previous_release_id": null,
            "launcher_path": launcher,
            "backup_path": home.join("state/launcher.previous"),
            "temporary_path": home.join("bin/.mirket-launcher-next-test"),
            "old_launcher_present": false,
            "old_launcher_digest": null,
            "candidate_launcher_digest": digest,
            "phase": "launcher_replaced"
        });
        fs::write(
            home.join("state/activation.pending.json"),
            serde_json::to_vec(&journal).unwrap(),
        )
        .unwrap();
        assert!(!home.join("active.json").exists());
        assert!(home.join("state/activation.pending.json").is_file());

        let journal_before = fs::read(home.join("state/activation.pending.json")).unwrap();
        let launcher_before = fs::read(&launcher).unwrap();
        let error = Installation::setup_codex(&home, &different_source, &user)
            .err()
            .unwrap();
        assert!(error.contains("different candidate"), "{error}");
        assert!(!home.join("active.json").exists());
        assert_eq!(
            fs::read(home.join("state/activation.pending.json")).unwrap(),
            journal_before
        );
        assert_eq!(fs::read(&launcher).unwrap(), launcher_before);

        let resumed = Installation::setup_codex(&home, &original_source, &user).unwrap();
        assert_eq!(resumed.core.activation.status, "active");
        assert_eq!(Installation::doctor_codex(&home, &user).status, "healthy");
        assert!(!home.join("state/activation.pending.json").exists());
    }

    #[test]
    fn codex_setup_does_not_reopen_a_healthy_home_as_fresh() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, false);
        let home = fixture.0.join("home");
        Installation::setup_codex(&home, &source, &user).unwrap();
        let config_before = fs::read(user.join(".codex/config.toml")).unwrap();

        let error = Installation::setup_codex(&home, &source, &user)
            .err()
            .unwrap();
        assert!(error.contains("complete or drifted"), "{error}");
        assert_eq!(Installation::doctor_codex(&home, &user).status, "healthy");
        assert_eq!(
            fs::read(user.join(".codex/config.toml")).unwrap(),
            config_before
        );
    }

    #[test]
    fn codex_setup_preflights_and_syncs_an_opted_in_project_selection() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, false);
        let project = fixture.0.join("project");
        create_private_directory(&project).unwrap();
        let catalog = QualifiedCatalog::open(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["motion-direction".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        let home = fixture.0.join("home");
        let result =
            Installation::setup_codex_with_project(&home, &source, &user, Some(&project)).unwrap();
        assert_eq!(
            result.project_skills.as_ref().map(|report| report.status),
            Some("synced")
        );
        assert!(
            project
                .join(".agents/skills/motion-direction/SKILL.md")
                .is_file()
        );
        assert!(
            project
                .join(".agents/skills/motion-direction/references/storyboard-and-transitions.md")
                .is_file()
        );
        assert_eq!(
            SkillSync::project(&catalog, &project)
                .unwrap()
                .inspect()
                .unwrap()
                .status,
            "managed"
        );
    }

    #[test]
    fn codex_setup_preserves_activation_when_project_receipt_committed_before_error() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, true);
        let project = fixture.0.join("project");
        create_private_directory(&project).unwrap();
        let catalog = QualifiedCatalog::open(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["motion-direction".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        let home = fixture.0.join("home");
        let error = Installation::setup_codex_inner_with_project(
            &home,
            &source,
            &user,
            Some(&project),
            Some(NativeStep::ProjectSkills),
        )
        .err()
        .unwrap();
        assert!(
            error.contains("project skill state requires reconciliation"),
            "{error}"
        );
        let release_id = Installation::doctor(&home).active_release_id.unwrap();
        let project_inspection = SkillSync::project(&catalog, &project)
            .unwrap()
            .inspect()
            .unwrap();
        assert_eq!(project_inspection.status, "managed");
        assert_eq!(
            project_inspection.release_id.as_deref(),
            Some(release_id.as_str())
        );
        assert!(
            project
                .join(".agents/skills/motion-direction/references/storyboard-and-transitions.md")
                .is_file()
        );
        assert_eq!(Installation::doctor_codex(&home, &user).status, "healthy");
    }

    #[test]
    fn codex_setup_rejects_unavailable_project_selection_before_claiming_home() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, false);
        let project = fixture.0.join("project");
        create_private_directory(&project).unwrap();
        let catalog = QualifiedCatalog::open(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["vector-art".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        let home = fixture.0.join("home");
        let error = Installation::setup_codex_with_project(&home, &source, &user, Some(&project))
            .err()
            .unwrap();
        assert!(
            error.contains("unavailable private capabilities"),
            "{error}"
        );
        assert!(!home.exists());
        assert!(!project.join(".agents/skills").exists());
    }

    #[test]
    fn codex_setup_preflights_collisions_before_claiming_installation() {
        for collision in ["config", "guidance", "skill"] {
            let fixture = Fixture::new();
            let source = codex_payload(&fixture, "core");
            let user = codex_user(&fixture, true);
            match collision {
                "config" => fs::write(user.join(".codex/config.toml"), "[mcp_servers.mirket]\ncommand = 'user-server'\n").unwrap(),
                "guidance" => fs::write(user.join(".codex/AGENTS.md"), "<!-- Mirket managed Codex guidance: begin -->\nUser block\n<!-- Mirket managed Codex guidance: end -->\n").unwrap(),
                _ => { fs::create_dir_all(user.join(".agents/skills/task-planning")).unwrap(); fs::write(user.join(".agents/skills/task-planning/SKILL.md"), "User skill.").unwrap(); }
            }
            let before_config = fs::read(user.join(".codex/config.toml")).unwrap();
            let before_guidance = fs::read(user.join(".codex/AGENTS.md")).unwrap();
            let home = fixture.0.join("home");
            assert!(Installation::setup_codex(&home, &source, &user).is_err());
            assert!(!home.exists());
            assert_eq!(
                fs::read(user.join(".codex/config.toml")).unwrap(),
                before_config
            );
            assert_eq!(
                fs::read(user.join(".codex/AGENTS.md")).unwrap(),
                before_guidance
            );
            assert!(!user.join(".agents/skills/runtime-operations").exists());
        }
    }

    #[test]
    fn codex_setup_rolls_back_each_completed_native_step_and_preserves_owned_core() {
        for step in [
            NativeStep::Skills,
            NativeStep::Guidance,
            NativeStep::Registration,
        ] {
            let fixture = Fixture::new();
            let source = codex_payload(&fixture, "core");
            let user = codex_user(&fixture, true);
            let config = fs::read(user.join(".codex/config.toml")).unwrap();
            let guidance = fs::read(user.join(".codex/AGENTS.md")).unwrap();
            let home = fixture.0.join("home");
            let error = Installation::setup_codex_inner(&home, &source, &user, Some(step))
                .err()
                .unwrap();
            assert!(
                error.contains("owned native changes rolled back"),
                "{error}"
            );
            assert_eq!(fs::read(user.join(".codex/config.toml")).unwrap(), config);
            assert_eq!(fs::read(user.join(".codex/AGENTS.md")).unwrap(), guidance);
            assert!(!user.join(".agents/skills/task-planning").exists());
            assert!(!home.join("state/codex-config.json").exists());
            assert!(!home.join("state/codex-guidance.json").exists());
            assert_eq!(Installation::doctor(&home).status, "healthy");
            assert_eq!(Installation::doctor_codex(&home, &user).status, "error");
        }
    }

    #[test]
    fn codex_update_busy_is_staged_without_mutating_native_scopes() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core:first");
        let user = codex_user(&fixture, true);
        let home = fixture.0.join("home");
        let setup = Installation::setup_codex(&home, &source, &user).unwrap();
        let installation = Installation::open(&home).unwrap();
        let paths = CodexPaths::resolve(&user, &home).unwrap();
        let before = NativeSnapshot::read(&installation, &paths).unwrap();
        let source = codex_payload(&fixture, "core:second");
        let session = crate::maintenance::SessionGuard::acquire(&home.join("state")).unwrap();
        let staged = installation.update_codex(&source, &user).unwrap();
        assert_eq!(staged.activation.status, "staged");
        assert!(
            staged.skills.is_none() && staged.guidance.is_none() && staged.registration.is_none()
        );
        assert!(NativeSnapshot::read(&installation, &paths).unwrap() == before);
        assert_eq!(
            Installation::doctor(&home).active_release_id.as_deref(),
            Some(setup.core.activation.release_id.as_str())
        );
        assert_eq!(
            fs::read(installation.launcher_path()).unwrap(),
            b"core:first"
        );
        drop(session);
        let updated = installation.update_codex(&source, &user).unwrap();
        assert_eq!(updated.activation.status, "active");
        assert!(updated.registration.is_some());
        assert_eq!(Installation::doctor_codex(&home, &user).status, "healthy");
        assert_eq!(
            fs::read(installation.launcher_path()).unwrap(),
            b"core:second"
        );
    }

    #[test]
    fn codex_update_preflights_and_syncs_an_existing_project_selection() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core:first");
        let user = codex_user(&fixture, true);
        let project = fixture.0.join("project");
        create_private_directory(&project).unwrap();
        let catalog = QualifiedCatalog::open(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        let home = fixture.0.join("home");
        Installation::setup_codex_with_project(&home, &source, &user, Some(&project)).unwrap();
        let installation = Installation::open(&home).unwrap();
        let source = codex_payload(&fixture, "core:second");
        let result = installation
            .update_codex_with_project(&source, &user, Some(&project))
            .unwrap();
        assert_eq!(result.activation.status, "active");
        assert_eq!(
            result.project_skills.as_ref().map(|report| report.status),
            Some("synced")
        );
        assert_eq!(
            SkillSync::project(&catalog, &project)
                .unwrap()
                .inspect()
                .unwrap()
                .release_id,
            Some(result.activation.release_id)
        );
    }

    #[test]
    fn codex_update_preserves_new_release_when_project_receipt_committed_before_error() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core:first");
        let user = codex_user(&fixture, true);
        let project = fixture.0.join("project");
        create_private_directory(&project).unwrap();
        let catalog = QualifiedCatalog::open(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap();
        let selection =
            ProjectSelection::resolve(&catalog, &[], &["architecture".into()], &[]).unwrap();
        selection.save(&project, &catalog).unwrap();
        let home = fixture.0.join("home");
        let first =
            Installation::setup_codex_with_project(&home, &source, &user, Some(&project)).unwrap();
        let installation = Installation::open(&home).unwrap();
        let source = codex_payload(&fixture, "core:second");
        let next_id = VerifiedRelease::open(&source)
            .unwrap()
            .id()
            .as_str()
            .to_owned();
        let error = installation
            .update_codex_inner_with_project(
                &source,
                &user,
                Some(&project),
                Some(NativeStep::ProjectSkills),
            )
            .err()
            .unwrap();
        assert!(
            error.contains("project skill state requires reconciliation"),
            "{error}"
        );
        assert_ne!(first.core.activation.release_id, next_id);
        assert_eq!(
            Installation::doctor(&home).active_release_id.as_deref(),
            Some(next_id.as_str())
        );
        let project_inspection = SkillSync::project(&catalog, &project)
            .unwrap()
            .inspect()
            .unwrap();
        assert_eq!(project_inspection.status, "managed");
        assert_eq!(
            project_inspection.release_id.as_deref(),
            Some(next_id.as_str())
        );
        assert_eq!(Installation::doctor_codex(&home, &user).status, "healthy");
    }

    #[test]
    fn codex_update_failure_restores_previous_release_and_native_snapshots() {
        for step in [
            NativeStep::Skills,
            NativeStep::Guidance,
            NativeStep::Registration,
        ] {
            let fixture = Fixture::new();
            let source = codex_payload(&fixture, "core:first");
            let user = codex_user(&fixture, true);
            let home = fixture.0.join("home");
            let setup = Installation::setup_codex(&home, &source, &user).unwrap();
            let installation = Installation::open(&home).unwrap();
            let paths = CodexPaths::resolve(&user, &home).unwrap();
            let before = NativeSnapshot::read(&installation, &paths).unwrap();
            let source = codex_payload(&fixture, "core:second");
            let error = installation
                .update_codex_inner(&source, &user, Some(step))
                .err()
                .unwrap();
            assert!(
                error.contains("previous core and native state restored"),
                "{error}"
            );
            assert!(NativeSnapshot::read(&installation, &paths).unwrap() == before);
            assert_eq!(
                Installation::doctor(&home).active_release_id.as_deref(),
                Some(setup.core.activation.release_id.as_str())
            );
            let report = Installation::doctor_codex(&home, &user);
            assert_eq!(report.status, "healthy", "{:?}", report.errors);
        }
    }

    #[test]
    fn codex_update_refuses_managed_drift_and_snapshot_restore_preserves_unknown_edits() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core:first");
        let user = codex_user(&fixture, true);
        let home = fixture.0.join("home");
        let setup = Installation::setup_codex(&home, &source, &user).unwrap();
        let installation = Installation::open(&home).unwrap();
        let paths = CodexPaths::resolve(&user, &home).unwrap();
        let before = NativeSnapshot::read(&installation, &paths).unwrap();
        fs::write(
            user.join(".codex/config.toml"),
            "# New user preference\nmodel = 'new-model'\n",
        )
        .unwrap();
        assert!(
            before
                .restore_non_skill_files(&installation, &paths, &before)
                .is_err()
        );
        assert_eq!(
            fs::read_to_string(user.join(".codex/config.toml")).unwrap(),
            "# New user preference\nmodel = 'new-model'\n"
        );
        let source = codex_payload(&fixture, "core:second");
        assert!(installation.update_codex(&source, &user).is_err());
        assert_eq!(
            Installation::doctor(&home).active_release_id.as_deref(),
            Some(setup.core.activation.release_id.as_str())
        );
        assert!(!home.join("staged.json").exists());
    }

    #[cfg(unix)]
    #[test]
    fn codex_setup_refuses_native_symlinks_without_creating_core_or_changing_target() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, false);
        let external = fixture.0.join("external");
        create_private_directory(&external).unwrap();
        fs::write(external.join("config.toml"), "User-owned target").unwrap();
        symlink(&external, user.join(".codex")).unwrap();
        let home = fixture.0.join("home");
        assert!(Installation::setup_codex(&home, &source, &user).is_err());
        assert!(!home.exists());
        assert_eq!(
            fs::read_to_string(external.join("config.toml")).unwrap(),
            "User-owned target"
        );
    }

    #[test]
    fn full_doctor_rejects_a_matching_receipt_that_launches_another_installation() {
        let fixture = Fixture::new();
        let source = codex_payload(&fixture, "core");
        let user = codex_user(&fixture, false);
        let home = fixture.0.join("home");
        Installation::setup_codex(&home, &source, &user).unwrap();
        let other_user = fixture.0.join("other-user");
        create_private_directory(&other_user).unwrap();
        let other_home = fixture.0.join("other-home");
        Installation::setup_codex(&other_home, &source, &other_user).unwrap();
        fs::copy(
            user.join(".codex/config.toml"),
            other_user.join(".codex/config.toml"),
        )
        .unwrap();
        fs::copy(
            home.join("state/codex-config.json"),
            other_home.join("state/codex-config.json"),
        )
        .unwrap();
        let report = Installation::doctor_codex(&other_home, &other_user);
        assert_eq!(report.status, "error");
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("does not launch this installation")),
            "{:?}",
            report.errors
        );
    }
}
