//! Verified native payload setup and read-only health for an explicit pi-owned home.
//! Pi owns package registration and resources. This layer never edits host settings.
use crate::{
    distribution::{ReleaseComponent, VerifiedRelease},
    native_skills::QualifiedCatalog,
    release_store::{ActivationResult, ReleaseStore},
};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

pub struct Installation {
    home: PathBuf,
    store: ReleaseStore,
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
    /// prunable release directory, so the pi runtime binding remains stable over
    /// update and rollback operations.
    pub fn launcher_path(&self) -> PathBuf {
        self.home.join("bin").join(if cfg!(windows) {
            "mirket.exe"
        } else {
            "mirket"
        })
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
}
