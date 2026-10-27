//! A finite local release manifest: exact bytes and explicit native target.
//!
//! Verification never executes payload files. Digests establish identity and
//! detect ordinary corruption; they do not authenticate a publisher or prove
//! that a native tool works. Selected components have finite inventories, not a
//! general package/plugin format. Files/directories stay
//! immutable after verification. Path-based std checks are not containment
//! against hostile concurrent filesystem replacement.

use crate::release::{AdapterPayload, ReleaseId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

mod chrome_admission;
mod video_candidate;

pub const MANIFEST_NAME: &str = "manifest.json";
const VERSION: &str = "0.0.1";
const MAC_TARGET: &str = "aarch64-apple-darwin";
const WINDOWS_X64_TARGET: &str = "x86_64-pc-windows-msvc";
const WINDOWS_ARM64_TARGET: &str = "aarch64-pc-windows-msvc";
const D2_EXECUTABLE: &str = "runtime/d2/d2";
const D2_LICENSE: &str = "runtime/d2/LICENSE.txt";
const D2_THIRD_PARTY: &str = "runtime/d2/THIRD_PARTY_NOTICES.txt";
const D2_SPDX: &str = "runtime/d2/d2.spdx.json";
const SEMANTIC_ROOT: &str = "runtime/semantic";
const DESIGN_ROOT: &str = "runtime/design";
const RASTER_ROOT: &str = "runtime/design/raster";
const RASTER_MODULE: &str = "runtime/design/raster/node_modules/sharp/dist/index.mjs";
const RASTER_PROPOSAL_SHA256: &str =
    "705cd057ebc0cc25c6cffe92d05d442663dad544dcd62b0e67cc30adae74e41b";
const OPENPENCIL_ROOT: &str = "runtime/openpencil";
const OPENPENCIL_ADAPTER: &str = "adapters/openpencil-main.js";
const OPENPENCIL_RUNTIME_SHA256: &str =
    "a86929a1380cd40bcfdf9a477d79b5a4f9f5fe489ad3d4a2f7e168b76328f0f9";
const DESIGN_ADAPTER: &str = "adapters/design-main.js";
const DESIGN_OPERATIONS: &str = "adapters/design-operations.js";
const CODE_INSPECT_ADAPTER: &str = "adapters/code-inspect-main.js";
const CODE_SOURCE_INSPECT: &str = "adapters/code-source-inspect.js";
const CODE_NAVIGATION: &str = "adapters/code-navigation.js";
const CODE_NAVIGATION_MAIN: &str = "adapters/code-navigation-main.js";
const DESIGN_FONT: &str = "runtime/design/fonts/NotoSans.ttf";
const SKILL_CATALOG: &str = "skills/catalog.json";
const SKILL_PROVENANCE: &str = "skills/PROVENANCE.md";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseComponent {
    Diagram,
    Design,
    OpenPencil,
    Raster,
    Video,
    Semantic,
    Skills,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub schema_version: u32,
    pub version: String,
    pub target: String,
    pub components: Vec<ReleaseComponent>,
    pub files: Vec<ReleaseFile>,
    pub links: Vec<ReleaseLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseFile {
    pub relative_path: String,
    pub byte_len: u64,
    pub sha256: String,
    /// Normalized installed permissions, not the original archive mode.
    pub mode: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseLink {
    pub relative_path: String,
    pub target: String,
}

#[derive(Debug)]
pub enum DistributionError {
    RootMustBeAbsoluteDirectory,
    Symlink,
    InvalidManifest,
    UnsupportedTarget,
    TargetMismatch,
    UnknownEntry,
    TooLarge,
    ChangedDuringRead,
    ContentMismatch,
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for DistributionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootMustBeAbsoluteDirectory => {
                write!(f, "select an explicit absolute release directory")
            }
            Self::Symlink => write!(f, "release contains an unapproved or changed symbolic link"),
            Self::InvalidManifest => {
                write!(
                    f,
                    "release manifest does not match the strict release schema"
                )
            }
            Self::UnsupportedTarget => {
                write!(f, "release target is outside the explicit target inventory")
            }
            Self::TargetMismatch => write!(f, "release target does not match this native platform"),
            Self::UnknownEntry => write!(
                f,
                "release contains a missing, duplicate or unknown payload entry"
            ),
            Self::TooLarge => write!(f, "release exceeds its compiled inventory capacity"),
            Self::ChangedDuringRead => write!(f, "release files changed during verification"),
            Self::ContentMismatch => write!(f, "saved release bytes do not match the manifest"),
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}
impl std::error::Error for DistributionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
fn io_error(operation: &'static str, source: io::Error) -> DistributionError {
    DistributionError::Io { operation, source }
}

// The admitted design inventory was assembled from its own frozen npm lock.
// Development can add a separately inventoried video candidate to
// the shared adapter lock, but cannot silently replace an admitted design
// package or its root dependency. Release layouts still gate their payloads.
fn design_lock_preserved(admitted: &[u8], current: &[u8]) -> bool {
    let (Ok(admitted), Ok(current)) = (
        serde_json::from_slice::<serde_json::Value>(admitted),
        serde_json::from_slice::<serde_json::Value>(current),
    ) else {
        return false;
    };
    let (Some(old), Some(now)) = (admitted.as_object(), current.as_object()) else {
        return false;
    };
    if old.len() != now.len()
        || old.keys().any(|key| !now.contains_key(key))
        || old
            .iter()
            .any(|(key, value)| key != "packages" && now.get(key) != Some(value))
    {
        return false;
    }
    let (Some(old_packages), Some(now_packages)) = (
        old.get("packages").and_then(serde_json::Value::as_object),
        now.get("packages").and_then(serde_json::Value::as_object),
    ) else {
        return false;
    };
    if old_packages
        .iter()
        .any(|(path, package)| !path.is_empty() && now_packages.get(path) != Some(package))
    {
        return false;
    }
    let (Some(mut old_root), Some(now_root)) =
        (old_packages.get("").cloned(), now_packages.get(""))
    else {
        return false;
    };
    if &old_root == now_root {
        return true;
    }
    let Some(dependencies) = old_root
        .get_mut("dependencies")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return false;
    };
    dependencies.insert(
        "@hyperframes/producer".into(),
        serde_json::Value::String("0.8.197".into()),
    );
    &old_root == now_root
}

pub struct VerifiedRelease {
    root: PathBuf,
    id: ReleaseId,
    executable: PathBuf,
    payload: AdapterPayload,
    executable_sha256: String,
    manifest: ReleaseManifest,
    directories: Vec<String>,
}

impl VerifiedRelease {
    pub fn open(root: &Path) -> Result<Self, DistributionError> {
        Self::open_with_file_modes(root, false)
    }

    fn open_with_file_modes(root: &Path, installed: bool) -> Result<Self, DistributionError> {
        let root = selected_root(root)?;
        let (mut manifest, encoded_bytes) = read_manifest(&root)?;
        let layout = if installed && manifest.components.contains(&ReleaseComponent::Skills) {
            // A retained release can have an older authored skill revision. Its
            // catalog and provenance are still bound by the saved manifest and
            // every skill body is bound by that catalog. New source packages
            // continue through the current compiled admission below.
            let catalog = read_bounded_release_file(&root, SKILL_CATALOG, 2 * 1024 * 1024)?;
            let provenance = read_bounded_release_file(&root, SKILL_PROVENANCE, 1024 * 1024)?;
            let layout = layout_with_skill_sources(
                &manifest.target,
                &manifest.components,
                Some((&catalog, &provenance)),
            )?;
            validate_manifest_against_layout(&mut manifest, layout)?
        } else {
            validate_manifest(&mut manifest)?
        };
        layout.capacity.check_manifest(encoded_bytes)?;
        if manifest.target != native_target()? {
            return Err(DistributionError::TargetMismatch);
        }
        verify_inventory(&root, &layout)?;
        verify_payload_files(&root, &manifest.files, installed)?;
        if manifest.components.contains(&ReleaseComponent::Semantic) {
            crate::embedding::verify_package_receipt(&root.join(SEMANTIC_ROOT), &manifest.target)
                .map_err(|_| DistributionError::ContentMismatch)?;
        }
        let id = manifest_identity(&manifest)?;
        let executable_sha256 = manifest
            .files
            .iter()
            .find(|entry| entry.relative_path == layout.executable)
            .ok_or(DistributionError::InvalidManifest)?
            .sha256
            .clone();
        let video = video_candidate::payload(&root, layout.node, &manifest.components);
        Ok(Self {
            executable: root.join(layout.executable),
            executable_sha256,
            payload: AdapterPayload {
                diagram: manifest
                    .components
                    .contains(&ReleaseComponent::Diagram)
                    .then(|| root.join(D2_EXECUTABLE)),
                design: design_payload(&root, layout.node, &manifest.components),
                openpencil: manifest
                    .components
                    .contains(&ReleaseComponent::OpenPencil)
                    .then(|| crate::openpencil::OpenPencilPayload {
                        node: root.join(layout.node),
                        adapter: root.join(OPENPENCIL_ADAPTER),
                        runtime_root: root.join(OPENPENCIL_ROOT),
                        runtime_sha256: OPENPENCIL_RUNTIME_SHA256.into(),
                    }),
                video,
                node: root.join(layout.node),
            },
            root,
            id,
            directories: layout.directories.into_iter().collect(),
            manifest,
        })
    }

    /// A staging source may retain group/other archive permissions. Installed
    /// payloads must use the manifest's exact normalized file modes and 0700 dirs.
    pub(crate) fn open_installed(root: &Path) -> Result<Self, DistributionError> {
        let release = Self::open_with_file_modes(root, true)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let manifest_mode = fs::symlink_metadata(release.root.join(MANIFEST_NAME))
                .map_err(|e| io_error("inspect installed manifest mode", e))?
                .permissions()
                .mode()
                & 0o7777;
            if manifest_mode != 0o600 {
                return Err(DistributionError::ContentMismatch);
            }
            for directory in
                std::iter::once("").chain(release.directories.iter().map(String::as_str))
            {
                let path = release.root.join(directory);
                let mode = fs::symlink_metadata(path)
                    .map_err(|e| io_error("inspect installed directory", e))?
                    .permissions()
                    .mode()
                    & 0o7777;
                if mode != 0o700 {
                    return Err(DistributionError::ContentMismatch);
                }
            }
        }
        Ok(release)
    }
    pub(crate) fn directories(&self) -> &[String] {
        &self.directories
    }

    /// Compare the running core before opening any engine state. A different
    /// digest permits the launcher to select this verified executable; an I/O
    /// or identity error does not. Identical development copies are accepted.
    pub fn running_core_matches(&self) -> Result<bool, DistributionError> {
        let executable =
            std::env::current_exe().map_err(|e| io_error("resolve running core", e))?;
        let parent = executable
            .parent()
            .ok_or(DistributionError::ContentMismatch)?;
        // macOS can report the running path through /var even when its real
        // parent is /private/var. Canonicalize only this OS-reported parent;
        // hash_file still rejects a symlinked executable leaf.
        let parent = parent
            .canonicalize()
            .map_err(|e| io_error("resolve running core parent", e))?;
        let name = executable
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(DistributionError::ContentMismatch)?;
        let (_, digest) = hash_file(
            &parent,
            name,
            UnpinnedFileRole::NativeExecutable.byte_limit(),
        )?;
        Ok(digest == self.executable_sha256)
    }
    pub fn manifest(&self) -> &ReleaseManifest {
        &self.manifest
    }
    pub fn id(&self) -> &ReleaseId {
        &self.id
    }
    pub fn payload(&self) -> AdapterPayload {
        self.payload.clone()
    }
    pub fn executable(&self) -> PathBuf {
        self.executable.clone()
    }
    pub fn embedding_package(&self) -> Option<PathBuf> {
        self.manifest
            .components
            .contains(&ReleaseComponent::Semantic)
            .then(|| self.root.join(SEMANTIC_ROOT))
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Derive the selected admitted inventory and exclusively create its manifest.
/// The caller owns this prepared release directory; no payload is installed,
/// fetched or executed, and an existing manifest is never replaced.
pub fn write_manifest(
    root: &Path,
    version: &str,
    target: &str,
) -> Result<ReleaseId, DistributionError> {
    if version != VERSION {
        return Err(DistributionError::InvalidManifest);
    }
    let root = selected_root(root)?;
    let components = infer_components(&root)?;
    let layout = layout(target, &components)?;
    verify_inventory(&root, &layout)?;
    if components.contains(&ReleaseComponent::Semantic) {
        crate::embedding::verify_package_receipt(&root.join(SEMANTIC_ROOT), target)
            .map_err(|_| DistributionError::ContentMismatch)?;
    }
    let mut files = Vec::with_capacity(layout.files.len());
    let mut total = 0u64;
    for (path, expected) in &layout.files {
        let (byte_len, sha256) = hash_file(&root, path, layout.file_byte_limit(path, expected)?)?;
        total = total
            .checked_add(byte_len)
            .ok_or(DistributionError::TooLarge)?;
        if total > layout.capacity.payload_bytes {
            return Err(DistributionError::TooLarge);
        }
        files.push(ReleaseFile {
            relative_path: path.into(),
            byte_len,
            sha256,
            mode: expected.mode,
        });
    }
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    let manifest = ReleaseManifest {
        schema_version: 3,
        version: version.into(),
        target: target.into(),
        components,
        files,
        links: layout.links.values().cloned().collect(),
    };
    let mut manifest = manifest;
    validate_manifest(&mut manifest)?;
    let id = manifest_identity(&manifest)?;
    let mut bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|_| DistributionError::InvalidManifest)?;
    bytes.push(b'\n');
    layout.capacity.check_manifest(bytes.len())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(root.join(MANIFEST_NAME))
        .map_err(|e| io_error("create release manifest", e))?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| io_error("persist release manifest", e))?;
    #[cfg(unix)]
    File::open(&root)
        .and_then(|directory| directory.sync_all())
        .map_err(|e| io_error("sync release directory", e))?;
    Ok(id)
}

/// Target names are explicit. Listing Windows layouts is not native Windows
/// qualification; open still verifies that the selected target matches its host.
pub fn native_target() -> Result<&'static str, DistributionError> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok(MAC_TARGET),
        ("windows", "x86_64") => Ok(WINDOWS_X64_TARGET),
        ("windows", "aarch64") => Ok(WINDOWS_ARM64_TARGET),
        _ => Err(DistributionError::UnsupportedTarget),
    }
}

struct ExpectedFile {
    mode: u32,
    pin: Option<(u64, String)>,
}
struct Layout {
    executable: &'static str,
    node: &'static str,
    directories: BTreeSet<String>,
    files: BTreeMap<String, ExpectedFile>,
    links: BTreeMap<String, ReleaseLink>,
    capacity: ReleaseCapacity,
}

/// Only these unpinned file roles may change with an authored release. Pinned
/// payload files instead use their exact compiled admission size.
#[derive(Clone, Copy)]
enum UnpinnedFileRole {
    NativeExecutable,
    JavaScriptAdapter,
    LicenseNotice,
    ComponentReceipt,
}
impl UnpinnedFileRole {
    const fn byte_limit(self) -> u64 {
        match self {
            Self::NativeExecutable => 256 * 1024 * 1024,
            Self::JavaScriptAdapter => 16 * 1024 * 1024,
            Self::LicenseNotice => 1024 * 1024,
            Self::ComponentReceipt => 64 * 1024,
        }
    }
}

#[derive(Default)]
struct ReleaseCapacity {
    payload_bytes: u64,
    manifest_bytes: usize,
    inventory_entries: usize,
}
impl ReleaseCapacity {
    fn check_manifest(&self, bytes: usize) -> Result<(), DistributionError> {
        if bytes > self.manifest_bytes {
            return Err(DistributionError::TooLarge);
        }
        Ok(())
    }
}
impl Layout {
    fn file_byte_limit(
        &self,
        path: &str,
        expected: &ExpectedFile,
    ) -> Result<u64, DistributionError> {
        if let Some((bytes, _)) = expected.pin {
            return Ok(bytes);
        }
        let role = if path == self.executable || path == self.node {
            UnpinnedFileRole::NativeExecutable
        } else {
            match path {
                DESIGN_ADAPTER
                | DESIGN_OPERATIONS
                | CODE_INSPECT_ADAPTER
                | CODE_SOURCE_INSPECT
                | CODE_NAVIGATION
                | CODE_NAVIGATION_MAIN
                | OPENPENCIL_ADAPTER
                | "adapters/openpencil-operations.js"
                | "adapters/openpencil-runtime.js"
                | "adapters/openpencil-inventory.js" => UnpinnedFileRole::JavaScriptAdapter,
                "runtime/node/LICENSE" => UnpinnedFileRole::LicenseNotice,
                "runtime/semantic/package-receipt.json" => UnpinnedFileRole::ComponentReceipt,
                _ => return Err(DistributionError::InvalidManifest),
            }
        };
        Ok(role.byte_limit())
    }
}

#[derive(Deserialize)]
struct D2Admission {
    schema: u32,
    component: String,
    version: String,
    target: String,
    license: String,
    selected_files: Vec<D2SelectedFile>,
    notices: D2Notices,
}
#[derive(Deserialize)]
struct D2SelectedFile {
    destination: String,
    bytes: u64,
    sha256: String,
    mode: String,
}
#[derive(Deserialize)]
struct D2Notices {
    spdx_destination: String,
    spdx_bytes: u64,
    spdx_sha256: String,
    source_license_sha256: String,
    third_party_sha256: String,
}
#[derive(Deserialize)]
struct SemanticAdmission {
    schema: u32,
    runtime_version: String,
    provider: String,
    telemetry: bool,
    model: SemanticModel,
    targets: BTreeMap<String, SemanticTarget>,
}
#[derive(Deserialize)]
struct SemanticModel {
    repo: String,
    revision: String,
    selected: Vec<SemanticFile>,
    contract: serde_json::Value,
}
#[derive(Deserialize)]
struct SemanticTarget {
    assembly_complete: bool,
    files: Vec<SemanticFile>,
}
#[derive(Deserialize)]
struct SemanticFile {
    path: String,
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
struct DesignAdmission {
    schema: u32,
    component: String,
    target: String,
    source_lock_sha256: String,
    file_count: usize,
    total_bytes: u64,
    files: Vec<DesignFile>,
    packages: Vec<DesignPackage>,
    missing_embedded_notices: Vec<String>,
    font: DesignFont,
    notices: Vec<DesignNotice>,
}
#[derive(Deserialize)]
struct DesignFile {
    path: String,
    bytes: u64,
    sha256: String,
}
#[derive(Deserialize)]
struct DesignPackage {
    path: String,
    name: String,
    version: String,
    license: String,
    package_json_sha256: String,
    embedded_notices: Vec<String>,
}
#[derive(Deserialize)]
struct DesignFont {
    path: String,
    bytes: u64,
    sha256: String,
    license: String,
}
#[derive(Deserialize)]
struct DesignNotice {
    source: String,
    destination: String,
    bytes: u64,
    sha256: String,
}

#[derive(Deserialize)]
struct RasterCandidate {
    schema: u32,
    status: String,
    release_admitted: bool,
    target: String,
    node_module: String,
    archive_count: usize,
    archives: Vec<RasterArchive>,
    runtime_inventory: RasterInventory,
}
#[derive(Deserialize)]
struct RasterArchive {
    package: String,
    version: String,
    license_declared: String,
    url: String,
    bytes: u64,
    sha256: String,
    sha512_integrity: String,
}
#[derive(Deserialize)]
struct RasterInventory {
    regular_files: usize,
    regular_bytes: u64,
    files: Vec<DesignFile>,
}
#[derive(Deserialize)]
struct OpenPencilAdmission {
    schema: u32,
    admission_status: String,
    component: String,
    target: String,
    engine_version: String,
    mode: String,
    selection_sha256: String,
    runtime_json_sha256: String,
    runtime_json_bytes: u64,
    file_count: usize,
    total_bytes: u64,
    package_count: usize,
    notice_count: usize,
}
#[derive(Deserialize)]
struct OpenPencilSelection {
    schema: u32,
    status: String,
    name: String,
    engine_version: String,
    packages: Vec<serde_json::Value>,
    files: Vec<OpenPencilFile>,
}
#[derive(Deserialize)]
struct OpenPencilFile {
    path: String,
    bytes: u64,
    sha256: String,
    kind: String,
}
#[derive(Deserialize)]
struct NodeSelection {
    targets: Vec<NodeTarget>,
}
#[derive(Deserialize)]
struct NodeTarget {
    target: String,
    selected_files: Vec<NodeFile>,
}
#[derive(Deserialize)]
struct NodeFile {
    destination_relative: String,
    bytes: u64,
    sha256: String,
}

fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', ':', '\0'])
        && path.len() <= 4096
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && Path::new(path)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn design_payload(
    root: &Path,
    node: &str,
    components: &[ReleaseComponent],
) -> Option<crate::design::DesignPayload> {
    components
        .contains(&ReleaseComponent::Design)
        .then(|| crate::design::DesignPayload {
            node: root.join(node),
            adapter: root.join(DESIGN_ADAPTER),
            font: root.join(DESIGN_FONT),
            raster_module: components
                .contains(&ReleaseComponent::Raster)
                .then(|| root.join(RASTER_MODULE)),
        })
}

/// Freeze the exact upstream archive selection for local installation. No field
/// in a supplied release manifest chooses these paths or changes these pins.
/// Public redistribution remains a separate source/notice decision.
fn extend_raster_candidate(result: &mut Layout) -> Result<(), DistributionError> {
    let proposal =
        include_bytes!("../packaging/admission/m4-raster-prebuilt-macos.runtime.json.proposed");
    if hex(&Sha256::digest(proposal)) != RASTER_PROPOSAL_SHA256 {
        return Err(DistributionError::InvalidManifest);
    }
    let candidate: RasterCandidate =
        serde_json::from_slice(proposal).map_err(|_| DistributionError::InvalidManifest)?;
    if candidate.schema != 1
        || candidate.status != "upstream_archive_candidate_not_release_admitted"
        || candidate.release_admitted
        || candidate.target != MAC_TARGET
        || candidate.node_module != RASTER_MODULE
        || candidate.archive_count != 6
        || candidate.archives.len() != 6
        || candidate.runtime_inventory.regular_files != 125
        || candidate.runtime_inventory.files.len() != 125
        || candidate.runtime_inventory.regular_bytes != 19_698_667
    {
        return Err(DistributionError::InvalidManifest);
    }
    for (archive, (package, version)) in candidate.archives.iter().zip([
        ("sharp", "0.35.5"),
        ("@img/colour", "1.1.0"),
        ("detect-libc", "2.1.2"),
        ("semver", "7.8.5"),
        ("@img/sharp-darwin-arm64", "0.35.5"),
        ("@img/sharp-libvips-darwin-arm64", "1.3.4"),
    ]) {
        if archive.package != package
            || archive.version != version
            || !archive.url.starts_with("https://registry.npmjs.org/")
            || archive.bytes == 0
            || !valid_digest(&archive.sha256)
            || !archive.sha512_integrity.starts_with("sha512-")
            || archive.license_declared.is_empty()
        {
            return Err(DistributionError::InvalidManifest);
        }
    }
    let mut bytes = 0u64;
    for entry in candidate.runtime_inventory.files {
        if !safe_relative(&entry.path)
            || !entry
                .path
                .starts_with("runtime/design/raster/node_modules/")
            || entry.path.contains("/.bin/")
            || !valid_digest(&entry.sha256)
            || entry.bytes == 0
        {
            return Err(DistributionError::InvalidManifest);
        }
        bytes = bytes
            .checked_add(entry.bytes)
            .ok_or(DistributionError::TooLarge)?;
        if result
            .files
            .insert(
                entry.path,
                ExpectedFile {
                    mode: 0o600,
                    pin: Some((entry.bytes, entry.sha256)),
                },
            )
            .is_some()
        {
            return Err(DistributionError::InvalidManifest);
        }
    }
    if bytes != candidate.runtime_inventory.regular_bytes
        || !result.files.contains_key(RASTER_MODULE)
    {
        return Err(DistributionError::InvalidManifest);
    }
    Ok(())
}

fn layout(target: &str, components: &[ReleaseComponent]) -> Result<Layout, DistributionError> {
    layout_with_skill_sources(target, components, None)
}

fn layout_with_skill_sources(
    target: &str,
    components: &[ReleaseComponent],
    skill_sources: Option<(&[u8], &[u8])>,
) -> Result<Layout, DistributionError> {
    if components.len() > 7 || components.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(DistributionError::InvalidManifest);
    }
    let (executable, node) = match target {
        MAC_TARGET => ("bin/mirket", "runtime/node/bin/node"),
        WINDOWS_X64_TARGET | WINDOWS_ARM64_TARGET => ("bin/mirket.exe", "runtime/node/node.exe"),
        _ => return Err(DistributionError::UnsupportedTarget),
    };
    let mut result = Layout {
        executable,
        node,
        directories: BTreeSet::new(),
        files: BTreeMap::new(),
        links: BTreeMap::new(),
        capacity: ReleaseCapacity::default(),
    };
    for (path, mode) in [
        (executable, 0o700),
        (node, 0o700),
        ("runtime/node/LICENSE", 0o600),
    ] {
        result
            .files
            .insert(path.into(), ExpectedFile { mode, pin: None });
    }
    if components.contains(&ReleaseComponent::Raster) {
        if target != MAC_TARGET {
            return Err(DistributionError::UnsupportedTarget);
        }
        if !components.contains(&ReleaseComponent::Design) {
            return Err(DistributionError::InvalidManifest);
        }
        extend_raster_candidate(&mut result)?;
    }
    if components.contains(&ReleaseComponent::Video) {
        if target != MAC_TARGET {
            return Err(DistributionError::UnsupportedTarget);
        }
        chrome_admission::extend(&mut result, target)?;
        video_candidate::extend(&mut result)?;
    }
    if components.contains(&ReleaseComponent::Diagram) {
        if target != MAC_TARGET {
            return Err(DistributionError::UnsupportedTarget);
        }
        let admission: D2Admission =
            serde_json::from_str(include_str!("../packaging/admission/m4-d2.manifest.json"))
                .map_err(|_| DistributionError::InvalidManifest)?;
        if admission.schema != 1
            || admission.component != "d2"
            || admission.version != crate::diagram::D2_VERSION
            || admission.target != MAC_TARGET
            || admission.license != "MPL-2.0"
            || admission.notices.spdx_destination != D2_SPDX
            || !valid_digest(&admission.notices.spdx_sha256)
        {
            return Err(DistributionError::InvalidManifest);
        }
        let mut found = BTreeSet::new();
        for entry in admission.selected_files {
            let mode = match entry.destination.as_str() {
                D2_EXECUTABLE if entry.mode == "0755" => 0o700,
                D2_LICENSE | D2_THIRD_PARTY if entry.mode == "0644" => 0o600,
                _ => return Err(DistributionError::InvalidManifest),
            };
            if !valid_digest(&entry.sha256)
                || !found.insert(entry.destination.clone())
                || (entry.destination == D2_EXECUTABLE
                    && entry.sha256 != crate::diagram::D2_MACOS_ARM64_SHA256)
                || (entry.destination == D2_LICENSE
                    && entry.sha256 != admission.notices.source_license_sha256)
                || (entry.destination == D2_THIRD_PARTY
                    && entry.sha256 != admission.notices.third_party_sha256)
            {
                return Err(DistributionError::InvalidManifest);
            }
            result.files.insert(
                entry.destination,
                ExpectedFile {
                    mode,
                    pin: Some((entry.bytes, entry.sha256)),
                },
            );
        }
        if found
            != BTreeSet::from([
                D2_EXECUTABLE.into(),
                D2_LICENSE.into(),
                D2_THIRD_PARTY.into(),
            ])
        {
            return Err(DistributionError::InvalidManifest);
        }
        result.files.insert(
            D2_SPDX.into(),
            ExpectedFile {
                mode: 0o600,
                pin: Some((admission.notices.spdx_bytes, admission.notices.spdx_sha256)),
            },
        );
    }
    if components.contains(&ReleaseComponent::Design) {
        if target != MAC_TARGET {
            return Err(DistributionError::UnsupportedTarget);
        }
        let admission: DesignAdmission = serde_json::from_str(include_str!(
            "../packaging/admission/m4-design-chromium-macos.runtime.json"
        ))
        .map_err(|_| DistributionError::InvalidManifest)?;
        let admitted_lock = include_bytes!("../packaging/admission/m4-design-package-lock.json");
        let current_lock = include_bytes!("../adapters/package-lock.json");
        if admission.schema != 1
            || admission.component != "design"
            || admission.target != MAC_TARGET
            || admission.source_lock_sha256 != hex(&Sha256::digest(admitted_lock))
            || !design_lock_preserved(admitted_lock, current_lock)
            || admission.file_count != admission.files.len()
            || admission.font.path != DESIGN_FONT
            || admission.font.bytes != 2_049_096
            || admission.font.sha256
                != "bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d"
            || admission.font.license != "OFL-1.1"
        {
            return Err(DistributionError::InvalidManifest);
        }
        let mut bytes = 0u64;
        let mut file_pins = BTreeMap::new();
        for entry in admission.files {
            if !safe_relative(&entry.path)
                || entry.path.starts_with('.')
                || entry.path.contains("/.bin/")
                || entry.path.ends_with(".node")
                || !valid_digest(&entry.sha256)
                || file_pins
                    .insert(entry.path.clone(), (entry.bytes, entry.sha256.clone()))
                    .is_some()
            {
                return Err(DistributionError::InvalidManifest);
            }
            bytes = bytes
                .checked_add(entry.bytes)
                .ok_or(DistributionError::TooLarge)?;
            result.files.insert(
                format!("adapters/node_modules/{}", entry.path),
                ExpectedFile {
                    mode: 0o600,
                    pin: Some((entry.bytes, entry.sha256)),
                },
            );
        }
        if bytes != admission.total_bytes || admission.packages.len() != 126 {
            return Err(DistributionError::InvalidManifest);
        }
        let typescript_subset = [
            (
                "typescript/package.json",
                3_620,
                "822ef7ca6452205657b6288b066481ecf508bfbf43455d715cf7d3ec457561e6",
            ),
            (
                "typescript/lib/typescript.js",
                9_112_572,
                "3ae902c92cc44dace175c0e69e13a4b0899f6983c6121d76b9ab8dd5795e7675",
            ),
            (
                "typescript/LICENSE.txt",
                9_197,
                "a7d00bfd54525bc694b6e32f64c7ebcf5e6b7ae3657be5cc12767bce74654a47",
            ),
            (
                "typescript/ThirdPartyNoticeText.txt",
                37_824,
                "1af3c68039c57e539422da82a4faada506ce6d0ea6f90e0b699d02dbcdb7a90c",
            ),
        ];
        if file_pins
            .keys()
            .filter(|path| path.starts_with("typescript/"))
            .count()
            != typescript_subset.len()
            || typescript_subset.iter().any(|(path, size, sha)| {
                file_pins
                    .get(*path)
                    .is_none_or(|(actual_size, actual_sha)| {
                        actual_size != size || actual_sha != sha
                    })
            })
        {
            return Err(DistributionError::InvalidManifest);
        }
        let mut packages = BTreeMap::new();
        let mut missing_notices = Vec::new();
        for package in admission.packages {
            if !safe_relative(&package.path)
                || package.name.is_empty()
                || package.version.is_empty()
                || !valid_digest(&package.package_json_sha256)
                || !matches!(
                    package.license.as_str(),
                    "MIT"
                        | "Apache-2.0"
                        | "BSD-2-Clause"
                        | "BSD-3-Clause"
                        | "ISC"
                        | "CC0-1.0"
                        | "BlueOak-1.0.0"
                        | "0BSD"
                )
                || packages
                    .insert(package.path.clone(), package.version.clone())
                    .is_some()
                || file_pins
                    .get(&format!("{}/package.json", package.path))
                    .is_none_or(|(_, sha)| *sha != package.package_json_sha256)
            {
                return Err(DistributionError::InvalidManifest);
            }
            if package.embedded_notices.is_empty() {
                missing_notices.push(package.path.clone());
                if !matches!(package.license.as_str(), "MIT" | "Apache-2.0" | "ISC") {
                    return Err(DistributionError::InvalidManifest);
                }
            } else {
                for notice in package.embedded_notices {
                    if !safe_relative(&notice)
                        || !file_pins.contains_key(&format!("{}/{}", package.path, notice))
                    {
                        return Err(DistributionError::InvalidManifest);
                    }
                }
            }
        }
        missing_notices.sort();
        if missing_notices.len() != 9 || missing_notices != admission.missing_embedded_notices {
            return Err(DistributionError::InvalidManifest);
        }
        for (name, version) in [
            ("style-dictionary", "5.7.1"),
            ("svgo", "4.1.0"),
            ("sax", "1.6.1"),
            ("typescript", "5.9.3"),
        ] {
            let manifest = format!("{name}/package.json");
            if packages.get(name).is_none_or(|actual| actual != version)
                || !file_pins.contains_key(&manifest)
            {
                return Err(DistributionError::InvalidManifest);
            }
        }
        for path in [
            DESIGN_ADAPTER,
            DESIGN_OPERATIONS,
            CODE_INSPECT_ADAPTER,
            CODE_SOURCE_INSPECT,
            CODE_NAVIGATION,
            CODE_NAVIGATION_MAIN,
        ] {
            if result
                .files
                .insert(
                    path.into(),
                    ExpectedFile {
                        mode: 0o600,
                        pin: None,
                    },
                )
                .is_some()
            {
                return Err(DistributionError::InvalidManifest);
            }
        }
        result.files.insert(
            DESIGN_FONT.into(),
            ExpectedFile {
                mode: 0o600,
                pin: Some((admission.font.bytes, admission.font.sha256)),
            },
        );
        if admission.notices.len() != 4 {
            return Err(DistributionError::InvalidManifest);
        }
        let mut notices = BTreeSet::new();
        for notice in admission.notices {
            let name = notice
                .destination
                .strip_prefix("notices/design/")
                .ok_or(DistributionError::InvalidManifest)?;
            let source = match name {
                "MIT.txt" => include_bytes!("../packaging/notices/design/MIT.txt").as_slice(),
                "Apache-2.0.txt" => {
                    include_bytes!("../packaging/notices/design/Apache-2.0.txt").as_slice()
                }
                "ISC.txt" => include_bytes!("../packaging/notices/design/ISC.txt").as_slice(),
                "OFL.txt" => include_bytes!("../packaging/notices/design/OFL.txt").as_slice(),
                _ => return Err(DistributionError::InvalidManifest),
            };
            if notice.source != format!("packaging/notices/design/{name}")
                || notice.bytes != source.len() as u64
                || notice.sha256 != hex(&Sha256::digest(source))
                || !notices.insert(name.to_owned())
            {
                return Err(DistributionError::InvalidManifest);
            }
            result.files.insert(
                notice.destination,
                ExpectedFile {
                    mode: 0o600,
                    pin: Some((notice.bytes, notice.sha256)),
                },
            );
        }
    }
    if components.contains(&ReleaseComponent::OpenPencil) {
        if target != MAC_TARGET {
            return Err(DistributionError::UnsupportedTarget);
        }
        let admission: OpenPencilAdmission = serde_json::from_str(include_str!(
            "../packaging/admission/m4-openpencil-macos.runtime.json"
        ))
        .map_err(|_| DistributionError::InvalidManifest)?;
        let selection_bytes =
            include_bytes!("../packaging/admission/openpencil-editable.candidate.json");
        let selection: OpenPencilSelection = serde_json::from_slice(selection_bytes)
            .map_err(|_| DistributionError::InvalidManifest)?;
        if admission.schema != 1
            || admission.admission_status != "admitted"
            || admission.component != "openpencil_editable_file"
            || admission.target != MAC_TARGET
            || admission.engine_version != crate::openpencil::ENGINE_VERSION
            || admission.mode != crate::openpencil::MODE
            || admission.selection_sha256
                != "fd966b3140dfe08a47fbaa4481cebaecd4c1a79856b821edb2be2cf7b238fb0f"
            || admission.selection_sha256 != hex(&Sha256::digest(selection_bytes))
            || admission.runtime_json_sha256 != OPENPENCIL_RUNTIME_SHA256
            || admission.runtime_json_bytes != 98_210
            || admission.file_count != 596
            || admission.total_bytes != 2_433_673
            || admission.package_count != 16
            || admission.notice_count != 21
            || selection.schema != 1
            || selection.status != "candidate_not_release_admission"
            || selection.name != "openpencil-editable-file"
            || selection.engine_version != admission.engine_version
            || selection.packages.len() != admission.package_count
            || selection.files.len() != admission.file_count
        {
            return Err(DistributionError::InvalidManifest);
        }
        let mut bytes = 0u64;
        let mut notices = 0usize;
        let mut paths = BTreeSet::new();
        for entry in selection.files {
            if !safe_relative(&entry.path)
                || !valid_digest(&entry.sha256)
                || entry.bytes == 0
                || !paths.insert(entry.path.clone())
                || !(entry.path.starts_with("node_modules/") || entry.path.starts_with("notices/"))
                || entry.path.contains("canvaskit")
                || entry.path.contains("yoga-layout")
                || entry.path.ends_with(".wasm")
                || !matches!(
                    entry.kind.as_str(),
                    "module" | "package_metadata" | "notice"
                )
                || (entry.kind == "notice") != entry.path.starts_with("notices/")
            {
                return Err(DistributionError::InvalidManifest);
            }
            bytes = bytes
                .checked_add(entry.bytes)
                .ok_or(DistributionError::TooLarge)?;
            notices += usize::from(entry.kind == "notice");
            if result
                .files
                .insert(
                    format!("{OPENPENCIL_ROOT}/{}", entry.path),
                    ExpectedFile {
                        mode: 0o600,
                        pin: Some((entry.bytes, entry.sha256)),
                    },
                )
                .is_some()
            {
                return Err(DistributionError::InvalidManifest);
            }
        }
        if bytes != admission.total_bytes || notices != admission.notice_count {
            return Err(DistributionError::InvalidManifest);
        }
        result.files.insert(
            format!("{OPENPENCIL_ROOT}/runtime.json"),
            ExpectedFile {
                mode: 0o600,
                pin: Some((admission.runtime_json_bytes, admission.runtime_json_sha256)),
            },
        );
        for path in [
            OPENPENCIL_ADAPTER,
            "adapters/openpencil-operations.js",
            "adapters/openpencil-runtime.js",
            "adapters/openpencil-inventory.js",
        ] {
            if result
                .files
                .insert(
                    path.into(),
                    ExpectedFile {
                        mode: 0o600,
                        pin: None,
                    },
                )
                .is_some()
            {
                return Err(DistributionError::InvalidManifest);
            }
        }
    }
    if components.contains(&ReleaseComponent::Semantic) {
        if target != MAC_TARGET {
            return Err(DistributionError::UnsupportedTarget);
        }
        let admission: SemanticAdmission = serde_json::from_str(include_str!(
            "../packaging/admission/m2-semantic-minilm.runtime.json"
        ))
        .map_err(|_| DistributionError::InvalidManifest)?;
        if admission.schema != 1
            || admission.runtime_version != "1.31.0"
            || admission.provider != "CPUExecutionProvider"
            || admission.telemetry
            || admission.model.repo != "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
            || admission.model.revision != "e8f8c211226b894fcb81acc59f3b34ba3efd5f42"
            || admission.model.contract["dimensions"] != 384
            || admission.model.contract["max_tokens"] != 128
        {
            return Err(DistributionError::InvalidManifest);
        }
        let selected = admission
            .targets
            .get(MAC_TARGET)
            .ok_or(DistributionError::InvalidManifest)?;
        if !selected.assembly_complete
            || selected.files.len() != 3
            || admission.model.selected.len() != 8
        {
            return Err(DistributionError::InvalidManifest);
        }
        let mut seen = BTreeSet::new();
        for (prefix, files) in [
            ("", selected.files.as_slice()),
            ("model/", admission.model.selected.as_slice()),
        ] {
            for file in files {
                let relative = format!("{prefix}{}", file.path);
                if !safe_relative(&relative)
                    || !valid_digest(&file.sha256)
                    || file.bytes == 0
                    || !seen.insert(relative.clone())
                {
                    return Err(DistributionError::InvalidManifest);
                }
                let path = format!("{SEMANTIC_ROOT}/{relative}");
                if result
                    .files
                    .insert(
                        path,
                        ExpectedFile {
                            mode: 0o600,
                            pin: Some((file.bytes, file.sha256.clone())),
                        },
                    )
                    .is_some()
                {
                    return Err(DistributionError::InvalidManifest);
                }
            }
        }
        let expected = BTreeSet::from([
            "onnxruntime/ThirdPartyNotices.txt".into(),
            "onnxruntime/LICENSE".into(),
            "onnxruntime/libonnxruntime.1.31.0.dylib".into(),
            "model/onnx/tokenizer.json".into(),
            "model/onnx/tokenizer_config.json".into(),
            "model/onnx/special_tokens_map.json".into(),
            "model/onnx/config.json".into(),
            "model/1_Pooling/config.json".into(),
            "model/sentence_bert_config.json".into(),
            "model/README.md".into(),
            "model/onnx/model_qint8_arm64.onnx".into(),
        ]);
        if seen != expected {
            return Err(DistributionError::InvalidManifest);
        }
        result.files.insert(
            format!("{SEMANTIC_ROOT}/package-receipt.json"),
            ExpectedFile {
                mode: 0o600,
                pin: None,
            },
        );
    }
    if components.contains(&ReleaseComponent::Skills) {
        let (catalog_source, provenance_source): (&[u8], &[u8]) = skill_sources.unwrap_or((
            include_bytes!("../skills/catalog.json"),
            include_bytes!("../skills/PROVENANCE.md"),
        ));
        let catalog: crate::native_skills::SkillCatalog = serde_json::from_slice(catalog_source)
            .map_err(|_| DistributionError::InvalidManifest)?;
        if catalog.schema_version != 1
            || catalog.global.len() != 9
            || catalog.project.len() != 58
            || catalog.profiles.len() != 8
        {
            return Err(DistributionError::InvalidManifest);
        }
        for (path, source) in [
            (SKILL_CATALOG, catalog_source),
            (SKILL_PROVENANCE, provenance_source),
        ] {
            result.files.insert(
                path.into(),
                ExpectedFile {
                    mode: 0o600,
                    pin: Some((source.len() as u64, hex(&Sha256::digest(source)))),
                },
            );
        }
        for entry in catalog.global.iter().chain(&catalog.project) {
            if entry.path != format!("skills/{}/SKILL.md", entry.id)
                || !safe_relative(&entry.path)
                || entry.source_bytes > 128 * 1024
                || !valid_digest(&entry.source_sha256)
                || result.files.contains_key(&entry.path)
            {
                return Err(DistributionError::InvalidManifest);
            }
            result.files.insert(
                entry.path.clone(),
                ExpectedFile {
                    mode: 0o600,
                    pin: Some((entry.source_bytes, entry.source_sha256.clone())),
                },
            );
        }
    }
    finish_layout(&mut result, target, components)?;
    Ok(result)
}

fn finish_layout(
    result: &mut Layout,
    target: &str,
    components: &[ReleaseComponent],
) -> Result<(), DistributionError> {
    // Only this compiled selection determines traversal. Manifest input cannot
    // nominate an arbitrary directory, command, executable mode or link target.
    let paths: Vec<_> = result
        .files
        .keys()
        .chain(result.links.keys())
        .chain(result.directories.iter())
        .cloned()
        .collect();
    for path in paths {
        for parent in Path::new(&path).ancestors().skip(1) {
            if let Some(parent) = parent.to_str().filter(|p| !p.is_empty()) {
                result.directories.insert(parent.into());
            }
        }
    }
    let mut folded = BTreeSet::from([MANIFEST_NAME.to_owned()]);
    for path in result
        .files
        .keys()
        .chain(result.links.keys())
        .chain(result.directories.iter())
    {
        if !safe_relative(path) || !folded.insert(path.to_lowercase()) {
            return Err(DistributionError::InvalidManifest);
        }
    }
    result.capacity = layout_capacity(result, target, components)?;
    Ok(())
}

fn layout_capacity(
    layout: &Layout,
    target: &str,
    components: &[ReleaseComponent],
) -> Result<ReleaseCapacity, DistributionError> {
    let mut payload_bytes = 0u64;
    let mut files = Vec::with_capacity(layout.files.len());
    for (path, expected) in &layout.files {
        let byte_limit = layout.file_byte_limit(path, expected)?;
        payload_bytes = payload_bytes
            .checked_add(byte_limit)
            .ok_or(DistributionError::TooLarge)?;
        files.push(ReleaseFile {
            relative_path: path.clone(),
            byte_len: byte_limit,
            sha256: "0".repeat(64),
            mode: expected.mode,
        });
    }
    let envelope = ReleaseManifest {
        schema_version: 3,
        version: VERSION.into(),
        target: target.into(),
        components: components.to_vec(),
        files,
        links: layout.links.values().cloned().collect(),
    };
    // The largest standard pretty-printed body is determined by compiled paths,
    // digests and per-file limits. Allow twice that body for JSON formatting;
    // arbitrary padding is not part of the release identity or its contract.
    let manifest_bytes = serde_json::to_vec_pretty(&envelope)
        .map_err(|_| DistributionError::InvalidManifest)?
        .len()
        .checked_mul(2)
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or(DistributionError::TooLarge)?;
    let inventory_entries = layout
        .files
        .len()
        .checked_add(layout.links.len())
        .and_then(|entries| entries.checked_add(layout.directories.len()))
        .and_then(|entries| entries.checked_add(1)) // manifest, when present
        .ok_or(DistributionError::TooLarge)?;
    Ok(ReleaseCapacity {
        payload_bytes,
        manifest_bytes,
        inventory_entries,
    })
}

fn maximum_manifest_bytes() -> Result<usize, DistributionError> {
    static MAXIMUM: OnceLock<usize> = OnceLock::new();
    if let Some(maximum) = MAXIMUM.get() {
        return Ok(*maximum);
    }
    // This pre-parse ceiling comes only from compiled target/component layouts.
    // Neither a manifest nor a directory can nominate a larger inventory.
    let components = [
        ReleaseComponent::Diagram,
        ReleaseComponent::Design,
        ReleaseComponent::OpenPencil,
        ReleaseComponent::Semantic,
        ReleaseComponent::Skills,
    ];
    let mut maximum = layout(MAC_TARGET, &components)?.capacity.manifest_bytes;
    for target in [WINDOWS_X64_TARGET, WINDOWS_ARM64_TARGET] {
        maximum = maximum.max(
            layout(target, &[ReleaseComponent::Skills])?
                .capacity
                .manifest_bytes,
        );
    }
    let _ = MAXIMUM.set(maximum);
    Ok(maximum)
}

fn infer_components(root: &Path) -> Result<Vec<ReleaseComponent>, DistributionError> {
    let diagram = match fs::symlink_metadata(root.join("runtime/d2")) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(io_error("inspect optional diagram component", e)),
    };
    let design = match fs::symlink_metadata(root.join(DESIGN_ROOT)) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(io_error("inspect optional design component", e)),
    };
    let openpencil = match fs::symlink_metadata(root.join(OPENPENCIL_ROOT)) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(io_error("inspect optional OpenPencil component", e)),
    };
    let raster = match fs::symlink_metadata(root.join(RASTER_ROOT)) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(io_error("inspect optional raster component", e)),
    };
    let video = match fs::symlink_metadata(root.join(video_candidate::ROOT)) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(io_error("inspect optional video component", e)),
    };
    let semantic = match fs::symlink_metadata(root.join(SEMANTIC_ROOT)) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(io_error("inspect optional semantic component", e)),
    };
    let skills = match fs::symlink_metadata(root.join("skills")) {
        Ok(_) => true,
        Err(e) if e.kind() == io::ErrorKind::NotFound => false,
        Err(e) => return Err(io_error("inspect optional skill component", e)),
    };
    let mut components = Vec::new();
    if diagram {
        components.push(ReleaseComponent::Diagram);
    }
    if design {
        components.push(ReleaseComponent::Design);
    }
    if openpencil {
        components.push(ReleaseComponent::OpenPencil);
    }
    if raster {
        components.push(ReleaseComponent::Raster);
    }
    if video {
        components.push(ReleaseComponent::Video);
    }
    if semantic {
        components.push(ReleaseComponent::Semantic);
    }
    if skills {
        components.push(ReleaseComponent::Skills);
    }
    Ok(components)
}

fn validate_manifest(manifest: &mut ReleaseManifest) -> Result<Layout, DistributionError> {
    let layout = layout(&manifest.target, &manifest.components)?;
    validate_manifest_against_layout(manifest, layout)
}

fn validate_manifest_against_layout(
    manifest: &mut ReleaseManifest,
    layout: Layout,
) -> Result<Layout, DistributionError> {
    if manifest.schema_version != 3 || manifest.version != VERSION {
        return Err(DistributionError::InvalidManifest);
    }
    manifest
        .files
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    manifest
        .links
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    if manifest.files.len() != layout.files.len()
        || manifest.links != layout.links.values().cloned().collect::<Vec<_>>()
    {
        return Err(DistributionError::UnknownEntry);
    }
    let mut total = 0u64;
    for (entry, (expected_path, expected)) in manifest.files.iter().zip(&layout.files) {
        if &entry.relative_path != expected_path || entry.mode != expected.mode {
            return Err(DistributionError::UnknownEntry);
        }
        if !valid_digest(&entry.sha256) {
            return Err(DistributionError::InvalidManifest);
        }
        if entry.byte_len > layout.file_byte_limit(expected_path, expected)? {
            return Err(DistributionError::TooLarge);
        }
        if let Some((len, digest)) = &expected.pin
            && (entry.byte_len != *len || entry.sha256 != *digest)
        {
            return Err(DistributionError::ContentMismatch);
        }
        total = total
            .checked_add(entry.byte_len)
            .ok_or(DistributionError::TooLarge)?;
        if total > layout.capacity.payload_bytes {
            return Err(DistributionError::TooLarge);
        }
    }
    Ok(layout)
}

fn manifest_identity(manifest: &ReleaseManifest) -> Result<ReleaseId, DistributionError> {
    // Typed field order plus sorted file entries defines the M0 canonical body.
    // Whitespace in saved JSON is irrelevant. There is no self-hash field.
    let body = serde_json::to_vec(manifest).map_err(|_| DistributionError::InvalidManifest)?;
    ReleaseId::parse(&format!("sha256:{}", hex(&Sha256::digest(&body))))
        .map_err(|_| DistributionError::InvalidManifest)
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(DIGITS[(byte >> 4) as usize] as char);
        result.push(DIGITS[(byte & 15) as usize] as char);
    }
    result
}

fn selected_root(root: &Path) -> Result<PathBuf, DistributionError> {
    if !root.is_absolute() {
        return Err(DistributionError::RootMustBeAbsoluteDirectory);
    }
    let metadata = fs::symlink_metadata(root).map_err(|e| io_error("inspect release root", e))?;
    if metadata.file_type().is_symlink() {
        return Err(DistributionError::Symlink);
    }
    if !metadata.is_dir() {
        return Err(DistributionError::RootMustBeAbsoluteDirectory);
    }
    fs::canonicalize(root).map_err(|e| io_error("resolve release root", e))
}

fn read_bounded_release_file(
    root: &Path,
    relative: &str,
    maximum: u64,
) -> Result<Vec<u8>, DistributionError> {
    let path = checked_path(root, relative)?;
    let before =
        fs::symlink_metadata(&path).map_err(|error| io_error("inspect release metadata", error))?;
    if !before.is_file() {
        return Err(DistributionError::UnknownEntry);
    }
    if before.len() > maximum {
        return Err(DistributionError::TooLarge);
    }
    let mut file = File::open(&path).map_err(|error| io_error("open release metadata", error))?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(maximum.checked_add(1).ok_or(DistributionError::TooLarge)?)
        .read_to_end(&mut bytes)
        .map_err(|error| io_error("read release metadata", error))?;
    if u64::try_from(bytes.len()).map_err(|_| DistributionError::TooLarge)? > maximum {
        return Err(DistributionError::TooLarge);
    }
    recheck_file(root, relative, &file, &before)?;
    Ok(bytes)
}

fn checked_path(root: &Path, relative: &str) -> Result<PathBuf, DistributionError> {
    if fs::canonicalize(root).map_err(|e| io_error("recheck release ancestry", e))? != root {
        return Err(DistributionError::Symlink);
    }
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(DistributionError::UnknownEntry);
    }
    let mut path = root.to_owned();
    for part in relative.components() {
        path.push(part);
        if fs::symlink_metadata(&path)
            .map_err(|e| io_error("inspect payload path", e))?
            .file_type()
            .is_symlink()
        {
            return Err(DistributionError::Symlink);
        }
    }
    Ok(path)
}

fn verify_inventory(root: &Path, layout: &Layout) -> Result<(), DistributionError> {
    let mut seen = BTreeSet::new();
    for directory in std::iter::once("").chain(layout.directories.iter().map(String::as_str)) {
        let path = if directory.is_empty() {
            root.to_owned()
        } else {
            checked_path(root, directory)?
        };
        if !fs::symlink_metadata(&path)
            .map_err(|e| io_error("inspect release directory", e))?
            .is_dir()
        {
            return Err(DistributionError::UnknownEntry);
        }
        for entry in fs::read_dir(&path).map_err(|e| io_error("inspect release inventory", e))? {
            if seen.len() >= layout.capacity.inventory_entries {
                return Err(DistributionError::TooLarge);
            }
            let entry = entry.map_err(|e| io_error("read release entry", e))?;
            let entry_path = entry.path();
            let relative = entry_path
                .strip_prefix(root)
                .map_err(|_| DistributionError::UnknownEntry)?;
            let name = relative
                .components()
                .map(|part| match part {
                    Component::Normal(part) => part.to_str().ok_or(DistributionError::UnknownEntry),
                    _ => Err(DistributionError::UnknownEntry),
                })
                .collect::<Result<Vec<_>, _>>()?
                .join("/");
            let name = name.as_str();
            if !safe_relative(name) || !seen.insert(name.to_owned()) {
                return Err(DistributionError::UnknownEntry);
            }
            let metadata = fs::symlink_metadata(&entry_path)
                .map_err(|e| io_error("inspect release entry", e))?;
            if let Some(link) = layout.links.get(name) {
                if !metadata.file_type().is_symlink()
                    || fs::read_link(&entry_path)
                        .map_err(|e| io_error("inspect interpreter alias", e))?
                        != Path::new(&link.target)
                {
                    return Err(DistributionError::Symlink);
                }
            } else if metadata.file_type().is_symlink() {
                return Err(DistributionError::Symlink);
            } else if layout.directories.contains(name) {
                if !metadata.is_dir() {
                    return Err(DistributionError::UnknownEntry);
                }
            } else if let Some(expected) = layout.files.get(name) {
                if !metadata.is_file() {
                    return Err(DistributionError::UnknownEntry);
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    // Source 775/755 becomes installed 700, and 664/644 becomes 600.
                    // Owner permissions must already match: never invent execute
                    // permission or preserve set-id/sticky bits from an archive.
                    if mode & 0o700 != expected.mode || mode & 0o7000 != 0 {
                        return Err(DistributionError::ContentMismatch);
                    }
                }
                #[cfg(not(unix))]
                let _ = expected;
            } else if name == MANIFEST_NAME {
                if !metadata.is_file() {
                    return Err(DistributionError::InvalidManifest);
                }
            } else {
                return Err(DistributionError::UnknownEntry);
            }
        }
    }
    if layout
        .files
        .keys()
        .chain(layout.links.keys())
        .chain(layout.directories.iter())
        .any(|path| !seen.contains(path))
    {
        return Err(DistributionError::UnknownEntry);
    }
    Ok(())
}

fn read_manifest(root: &Path) -> Result<(ReleaseManifest, usize), DistributionError> {
    let path = checked_path(root, MANIFEST_NAME)?;
    let before = fs::symlink_metadata(&path).map_err(|e| io_error("inspect manifest", e))?;
    if !before.is_file() {
        return Err(DistributionError::InvalidManifest);
    }
    let maximum = maximum_manifest_bytes()?;
    let read_limit = u64::try_from(maximum)
        .ok()
        .and_then(|bytes| bytes.checked_add(1))
        .ok_or(DistributionError::TooLarge)?;
    if before.len() >= read_limit {
        return Err(DistributionError::TooLarge);
    }
    let mut file = File::open(&path).map_err(|e| io_error("open manifest", e))?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|e| io_error("read manifest", e))?;
    if bytes.len() > maximum {
        return Err(DistributionError::TooLarge);
    }
    recheck_file(root, MANIFEST_NAME, &file, &before)?;
    let manifest =
        serde_json::from_slice(&bytes).map_err(|_| DistributionError::InvalidManifest)?;
    Ok((manifest, bytes.len()))
}

fn verify_payload_files(
    root: &Path,
    files: &[ReleaseFile],
    installed: bool,
) -> Result<(), DistributionError> {
    // Verification remains complete and connection-local. Bound read-only work
    // without caching file metadata, skipping bytes, or opening any runtime.
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(4);
    verify_payload_files_with_workers(root, files, installed, workers)
}

fn verify_payload_files_with_workers(
    root: &Path,
    files: &[ReleaseFile],
    installed: bool,
    workers: usize,
) -> Result<(), DistributionError> {
    let verify = |entry: &ReleaseFile| {
        let (byte_len, digest) = hash_file_with_mode(
            root,
            &entry.relative_path,
            entry.byte_len,
            installed.then_some(entry.mode),
        )?;
        if byte_len != entry.byte_len || digest != entry.sha256 {
            return Err(DistributionError::ContentMismatch);
        }
        Ok(())
    };
    if workers <= 1 || files.len() < 64 {
        return files.iter().try_for_each(verify);
    }
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let verify = &verify;
            let next = &next;
            handles.push(
                std::thread::Builder::new()
                    .name("mirket-release-verify".into())
                    .spawn_scoped(scope, move || {
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let entry = files.get(index)?;
                            if let Err(error) = verify(entry) {
                                return Some((index, error));
                            }
                        }
                    })
                    .map_err(|e| io_error("start release verification worker", e))?,
            );
        }
        // Scheduling cannot change the reported failure: preserve the first
        // manifest entry that a serial full verification would reject.
        let mut first_error = None;
        for handle in handles {
            let failure = handle.join().map_err(|_| {
                io_error(
                    "join release verification worker",
                    io::Error::other("release verification worker panicked"),
                )
            })?;
            if let Some((index, error)) = failure
                && first_error.as_ref().is_none_or(|(first, _)| index < *first)
            {
                first_error = Some((index, error));
            }
        }
        first_error.map_or(Ok(()), |(_, error)| Err(error))
    })
}

fn hash_file(
    root: &Path,
    relative: &str,
    byte_limit: u64,
) -> Result<(u64, String), DistributionError> {
    hash_file_with_mode(root, relative, byte_limit, None)
}

fn hash_file_with_mode(
    root: &Path,
    relative: &str,
    byte_limit: u64,
    installed_mode: Option<u32>,
) -> Result<(u64, String), DistributionError> {
    let path = checked_path(root, relative)?;
    let before = fs::symlink_metadata(&path).map_err(|e| io_error("inspect payload file", e))?;
    if !before.is_file() {
        return Err(DistributionError::UnknownEntry);
    }
    if before.len() > byte_limit {
        return Err(DistributionError::TooLarge);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if installed_mode.is_some_and(|mode| before.permissions().mode() & 0o7777 != mode) {
            return Err(DistributionError::ContentMismatch);
        }
    }
    #[cfg(not(unix))]
    let _ = installed_mode;
    let mut file = File::open(&path).map_err(|e| io_error("open payload file", e))?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65_536];
    let mut total = 0u64;
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| io_error("hash payload file", e))?;
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or(DistributionError::TooLarge)?;
        if total > byte_limit {
            return Err(DistributionError::TooLarge);
        }
        digest.update(&buffer[..count]);
    }
    // The opened and saved file identities, including Unix ctime, are checked
    // after the complete read. A mode change during hashing is still rejected.
    recheck_file(root, relative, &file, &before)?;
    if total != before.len() {
        return Err(DistributionError::ChangedDuringRead);
    }
    Ok((total, hex(&digest.finalize())))
}

fn recheck_file(
    root: &Path,
    relative: &str,
    file: &File,
    before: &Metadata,
) -> Result<(), DistributionError> {
    let path = checked_path(root, relative)?;
    let after = fs::symlink_metadata(path).map_err(|e| io_error("recheck payload file", e))?;
    let opened = file
        .metadata()
        .map_err(|e| io_error("recheck open payload file", e))?;
    for candidate in [&after, &opened] {
        if before.len() != candidate.len() || before.modified().ok() != candidate.modified().ok() {
            return Err(DistributionError::ChangedDuringRead);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if (
                before.dev(),
                before.ino(),
                before.ctime(),
                before.ctime_nsec(),
            ) != (
                candidate.dev(),
                candidate.ino(),
                candidate.ctime(),
                candidate.ctime_nsec(),
            ) {
                return Err(DistributionError::ChangedDuringRead);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn design_lock_keeps_the_admitted_package_closure_exact() {
        let admitted = include_bytes!("../packaging/admission/m4-design-package-lock.json");
        let current = include_bytes!("../adapters/package-lock.json");
        assert!(design_lock_preserved(admitted, admitted));
        assert!(design_lock_preserved(admitted, current));
        let mut changed: serde_json::Value = serde_json::from_slice(current).unwrap();
        changed["packages"]["node_modules/typescript"]["version"] = serde_json::json!("0.0.0");
        assert!(!design_lock_preserved(
            admitted,
            &serde_json::to_vec(&changed).unwrap()
        ));
        let mut changed: serde_json::Value = serde_json::from_slice(current).unwrap();
        changed["packages"][""]["dependencies"]["unadmitted-package"] = serde_json::json!("1.0.0");
        assert!(!design_lock_preserved(
            admitted,
            &serde_json::to_vec(&changed).unwrap()
        ));
        let mut changed: serde_json::Value = serde_json::from_slice(current).unwrap();
        changed["packages"]["node_modules/style-dictionary"]["integrity"] =
            serde_json::json!("sha512-forged");
        assert!(!design_lock_preserved(
            admitted,
            &serde_json::to_vec(&changed).unwrap()
        ));
    }

    struct Fixture {
        root: PathBuf,
        target: &'static str,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "mirket-distribution-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let target = native_target().unwrap_or(MAC_TARGET);
            let layout = layout(target, &[]).unwrap();
            for directory in &layout.directories {
                fs::create_dir_all(root.join(directory)).unwrap();
            }
            for (index, (relative, expected)) in layout.files.iter().enumerate() {
                fs::write(
                    root.join(relative),
                    format!("verification fixture {index}; never execute\n"),
                )
                .unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(
                        root.join(relative),
                        fs::Permissions::from_mode(expected.mode),
                    )
                    .unwrap();
                }
                #[cfg(not(unix))]
                let _ = expected;
            }
            Self { root, target }
        }
        fn manifest(&self) -> ReleaseManifest {
            read_manifest(&self.root).unwrap().0
        }
        fn write(&self) -> ReleaseId {
            write_manifest(&self.root, VERSION, self.target).unwrap()
        }
        fn mutate_manifest(&self, update: impl FnOnce(&mut serde_json::Value)) {
            let mut value: serde_json::Value =
                serde_json::from_slice(&fs::read(self.root.join(MANIFEST_NAME)).unwrap()).unwrap();
            update(&mut value);
            fs::write(
                self.root.join(MANIFEST_NAME),
                serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn parallel_payload_verification_keeps_exact_bytes_modes_and_first_failure() {
        let fixture = Fixture::new();
        let mut files = Vec::new();
        for index in 0..64 {
            let relative_path = format!("parallel-{index:03}");
            let path = fixture.root.join(&relative_path);
            fs::write(&path, b"abc").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
            }
            files.push(ReleaseFile {
                relative_path,
                byte_len: 3,
                sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
                mode: 0o600,
            });
        }
        for workers in [1, 4] {
            verify_payload_files_with_workers(&fixture.root, &files, true, workers).unwrap();
        }
        let last = fixture.root.join(&files[63].relative_path);
        fs::write(&last, b"abd").unwrap();
        for workers in [1, 4] {
            assert!(matches!(
                verify_payload_files_with_workers(&fixture.root, &files, false, workers),
                Err(DistributionError::ContentMismatch)
            ));
        }
        fs::write(&last, b"abc").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let before = fs::symlink_metadata(&last).unwrap();
            let opened = File::open(&last).unwrap();
            fs::set_permissions(&last, fs::Permissions::from_mode(0o640)).unwrap();
            assert!(matches!(
                recheck_file(&fixture.root, &files[63].relative_path, &opened, &before),
                Err(DistributionError::ChangedDuringRead)
            ));
            for workers in [1, 4] {
                verify_payload_files_with_workers(&fixture.root, &files, false, workers).unwrap();
                assert!(matches!(
                    verify_payload_files_with_workers(&fixture.root, &files, true, workers),
                    Err(DistributionError::ContentMismatch)
                ));
            }
            fs::set_permissions(&last, fs::Permissions::from_mode(0o600)).unwrap();
        }
        // Two distinct failures must retain manifest order despite scheduling.
        fs::remove_file(last).unwrap();
        let first = fixture.root.join(&files[2].relative_path);
        fs::write(&first, b"abd").unwrap();
        for workers in [1, 4] {
            assert!(matches!(
                verify_payload_files_with_workers(&fixture.root, &files, false, workers),
                Err(DistributionError::ContentMismatch)
            ));
        }
        fs::write(first, b"abc").unwrap();
        for workers in [1, 4] {
            assert!(matches!(
                verify_payload_files_with_workers(&fixture.root, &files, false, workers),
                Err(DistributionError::Io { .. })
            ));
        }
    }

    #[test]
    fn exact_saved_payload_has_stable_typed_identity_without_execution() {
        if native_target().is_err() {
            return;
        }
        let fixture = Fixture::new();
        let expected = fixture.write();
        let verified = VerifiedRelease::open(&fixture.root).unwrap();
        assert_eq!(verified.id(), &expected);
        assert_eq!(verified.root(), fixture.root);
        assert!(verified.executable().is_absolute());
        assert!(verified.payload().node.is_absolute());
        assert!(verified.payload().node.is_file());
        let mut manifest = fixture.manifest();
        manifest.files.reverse();
        fs::write(
            fixture.root.join(MANIFEST_NAME),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(
            VerifiedRelease::open(&fixture.root).unwrap().id(),
            &expected
        );
        assert!(write_manifest(&fixture.root, VERSION, fixture.target).is_err());
        assert_eq!(
            VerifiedRelease::open(&fixture.root).unwrap().id(),
            &expected
        );
    }

    #[test]
    fn skills_component_accepts_only_the_compiled_authored_bytes() {
        if native_target().is_err() {
            return;
        }
        let fixture = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        let skills_layout = layout(fixture.target, &[ReleaseComponent::Skills]).unwrap();
        for relative in skills_layout
            .files
            .keys()
            .filter(|path| path.starts_with("skills/"))
        {
            let destination = fixture.root.join(relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(source.join(relative), destination).unwrap();
        }
        let id = fixture.write();
        let verified = VerifiedRelease::open(&fixture.root).unwrap();
        assert_eq!(verified.id(), &id);
        assert_eq!(
            verified.manifest().components,
            vec![ReleaseComponent::Skills]
        );
        let body = fixture.root.join("skills/runtime-operations/SKILL.md");
        fs::write(&body, b"changed instruction").unwrap();
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::ContentMismatch)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn retained_skill_revision_uses_its_manifest_and_refuses_drift() {
        use std::os::unix::fs::PermissionsExt;
        if native_target().is_err() {
            return;
        }
        let fixture = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        let skills_layout = layout(fixture.target, &[ReleaseComponent::Skills]).unwrap();
        for relative in skills_layout
            .files
            .keys()
            .filter(|path| path.starts_with("skills/"))
        {
            let destination = fixture.root.join(relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(source.join(relative), destination).unwrap();
            fs::set_permissions(
                fixture.root.join(relative),
                fs::Permissions::from_mode(0o600),
            )
            .unwrap();
        }
        fixture.write();
        let body_path = fixture.root.join("skills/vector-art/SKILL.md");
        let mut body = fs::read(&body_path).unwrap();
        body.extend_from_slice(b"\nRetained revision fixture.\n");
        fs::write(&body_path, &body).unwrap();
        let catalog_path = fixture.root.join(SKILL_CATALOG);
        let mut catalog: serde_json::Value =
            serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
        let entry = catalog["project"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|entry| entry["id"] == "vector-art")
            .unwrap();
        entry["source_bytes"] = serde_json::json!(body.len());
        entry["source_sha256"] = serde_json::json!(hex(&Sha256::digest(&body)));
        let catalog_bytes = serde_json::to_vec_pretty(&catalog).unwrap();
        fs::write(&catalog_path, &catalog_bytes).unwrap();
        let mut manifest = fixture.manifest();
        for (relative, bytes) in [
            ("skills/vector-art/SKILL.md", body.as_slice()),
            (SKILL_CATALOG, catalog_bytes.as_slice()),
        ] {
            let entry = manifest
                .files
                .iter_mut()
                .find(|entry| entry.relative_path == relative)
                .unwrap();
            entry.byte_len = bytes.len() as u64;
            entry.sha256 = hex(&Sha256::digest(bytes));
        }
        fs::write(
            fixture.root.join(MANIFEST_NAME),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        fs::set_permissions(&fixture.root, fs::Permissions::from_mode(0o700)).unwrap();
        for directory in &skills_layout.directories {
            fs::set_permissions(
                fixture.root.join(directory),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
        let retained = VerifiedRelease::open_installed(&fixture.root).unwrap();
        assert_eq!(retained.id(), &manifest_identity(&manifest).unwrap());
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::ContentMismatch)
        ));
        fs::write(&body_path, b"unrecorded edit").unwrap();
        assert!(matches!(
            VerifiedRelease::open_installed(&fixture.root),
            Err(DistributionError::ContentMismatch)
        ));
    }

    #[test]
    fn changed_or_missing_saved_files_cannot_verify() {
        if native_target().is_err() {
            return;
        }
        let fixture = Fixture::new();
        fixture.write();
        let path = fixture.root.join("runtime/node/LICENSE");
        let mut bytes = fs::read(&path).unwrap();
        bytes[0] ^= 1;
        fs::write(&path, &bytes).unwrap();
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::ContentMismatch)
        ));
        fs::remove_file(path).unwrap();
        assert!(VerifiedRelease::open(&fixture.root).is_err());
    }

    #[test]
    fn strict_schema_rejects_unknown_fields_duplicates_and_traversal() {
        for mutation in [0, 1, 2, 3, 4, 5] {
            let fixture = Fixture::new();
            fixture.write();
            fixture.mutate_manifest(|manifest| match mutation {
                0 => manifest["unexpected"] = serde_json::json!(true),
                1 => manifest["files"][0]["relative_path"] = serde_json::json!("../outside"),
                2 => manifest["files"][1] = manifest["files"][0].clone(),
                3 => manifest["files"][0]["sha256"] = serde_json::json!("ABCDEF".repeat(11)),
                4 => manifest["schema_version"] = serde_json::json!(1),
                _ => manifest["files"][0]["command"] = serde_json::json!("not allowed"),
            });
            assert!(VerifiedRelease::open(&fixture.root).is_err());
        }
        let fixture = Fixture::new();
        assert!(matches!(
            write_manifest(&fixture.root, "other-version", fixture.target),
            Err(DistributionError::InvalidManifest)
        ));
        assert!(matches!(
            write_manifest(&fixture.root, VERSION, "automatic"),
            Err(DistributionError::UnsupportedTarget)
        ));
    }

    #[test]
    fn target_is_explicit_and_must_match_native_selection() {
        if native_target().is_err() {
            return;
        }
        let fixture = Fixture::new();
        fixture.write();
        let other_target = if fixture.target == MAC_TARGET {
            WINDOWS_X64_TARGET
        } else {
            MAC_TARGET
        };
        let other_layout = layout(other_target, &[]).unwrap();
        fixture.mutate_manifest(|manifest| {
            manifest["target"] = serde_json::json!(other_target);
            let files: Vec<_> = other_layout.files.keys().collect();
            for (index, path) in files.iter().enumerate() {
                manifest["files"][index]["relative_path"] = serde_json::json!(path);
            }
        });
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::TargetMismatch)
        ));
    }

    #[test]
    fn manifest_and_declared_payload_bounds_are_enforced() {
        let fixture = Fixture::new();
        fixture.write();
        let layout = layout(fixture.target, &[]).unwrap();
        let (path, expected) = layout.files.first_key_value().unwrap();
        let limit = layout.file_byte_limit(path, expected).unwrap();
        fixture.mutate_manifest(|manifest| {
            manifest["files"][0]["byte_len"] = serde_json::json!(limit + 1)
        });
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::TooLarge)
        ));
        fs::write(
            fixture.root.join(MANIFEST_NAME),
            vec![b' '; maximum_manifest_bytes().unwrap() + 1],
        )
        .unwrap();
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::TooLarge)
        ));
        // Sparse length mutation exercises the actual-file cap without creating
        // a large fixture body or reading hundreds of megabytes.
        let fixture = Fixture::new();
        OpenOptions::new()
            .write(true)
            .open(fixture.root.join("runtime/node/LICENSE"))
            .unwrap()
            .set_len(UnpinnedFileRole::LicenseNotice.byte_limit() + 1)
            .unwrap();
        assert!(matches!(
            write_manifest(&fixture.root, VERSION, fixture.target),
            Err(DistributionError::TooLarge)
        ));
        assert!(!fixture.root.join(MANIFEST_NAME).exists());
    }

    #[test]
    fn manifest_padding_cannot_borrow_another_components_parse_capacity() {
        let fixture = Fixture::new();
        fixture.write();
        let selected_limit = layout(fixture.target, &[]).unwrap().capacity.manifest_bytes;
        assert!(selected_limit < maximum_manifest_bytes().unwrap());
        let mut bytes = fs::read(fixture.root.join(MANIFEST_NAME)).unwrap();
        bytes.resize(selected_limit + 1, b' ');
        fs::write(fixture.root.join(MANIFEST_NAME), bytes).unwrap();
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::TooLarge)
        ));
    }

    #[test]
    fn compiled_inventory_capacity_scales_without_relaxing_file_identity() {
        // A synthetic compiled selection, not an admitted browser component.
        // Its cardinality, nesting and pinned native file exceed the former
        // global 8192-entry/1024-directory/512-MiB/256-MiB-file limits.
        let mut selected = layout(MAC_TARGET, &[]).unwrap();
        for index in 0..15_000 {
            selected.files.insert(
                format!("runtime/synthetic/dir{}/file{index}", index / 10),
                ExpectedFile {
                    mode: 0o600,
                    pin: Some((100_000, "0".repeat(64))),
                },
            );
        }
        let native_path = "runtime/synthetic/native";
        selected.files.insert(
            native_path.into(),
            ExpectedFile {
                mode: 0o700,
                pin: Some((768 * 1024 * 1024, "1".repeat(64))),
            },
        );
        finish_layout(&mut selected, MAC_TARGET, &[]).unwrap();
        assert!(selected.files.len() > 8192);
        assert!(selected.directories.len() > 1024);
        assert!(selected.capacity.payload_bytes > 2 * 1024 * 1024 * 1024);
        assert!(selected.capacity.manifest_bytes > 2 * 1024 * 1024);
        assert_eq!(
            selected
                .file_byte_limit(native_path, &selected.files[native_path])
                .unwrap(),
            768 * 1024 * 1024
        );
        // The runtime parser's ceiling still comes from actual admitted layouts.
        assert!(selected.capacity.manifest_bytes > maximum_manifest_bytes().unwrap());
    }

    #[test]
    fn capacity_rejects_overflow_unbudgeted_roles_and_compiled_path_collisions() {
        for path in [
            "runtime/node/Bin/extra",
            "runtime/node",
            "manifest.json",
            "MANIFEST.JSON",
            "../outside",
            "runtime\\outside",
        ] {
            let mut selected = layout(MAC_TARGET, &[]).unwrap();
            selected.files.insert(
                path.into(),
                ExpectedFile {
                    mode: 0o600,
                    pin: Some((1, "0".repeat(64))),
                },
            );
            assert!(matches!(
                finish_layout(&mut selected, MAC_TARGET, &[]),
                Err(DistributionError::InvalidManifest)
            ));
        }
        let mut selected = layout(MAC_TARGET, &[]).unwrap();
        selected.files.insert(
            "runtime/synthetic/oversized".into(),
            ExpectedFile {
                mode: 0o600,
                pin: Some((u64::MAX, "0".repeat(64))),
            },
        );
        assert!(matches!(
            finish_layout(&mut selected, MAC_TARGET, &[]),
            Err(DistributionError::TooLarge)
        ));
        let mut selected = layout(MAC_TARGET, &[]).unwrap();
        selected.files.insert(
            "runtime/synthetic/unbudgeted".into(),
            ExpectedFile {
                mode: 0o600,
                pin: None,
            },
        );
        assert!(matches!(
            finish_layout(&mut selected, MAC_TARGET, &[]),
            Err(DistributionError::InvalidManifest)
        ));
    }

    #[test]
    fn actual_file_reads_use_the_selected_file_limit() {
        let fixture = Fixture::new();
        let path = "runtime/node/LICENSE";
        let bytes = fs::read(fixture.root.join(path)).unwrap();
        let (len, digest) = hash_file(&fixture.root, path, bytes.len() as u64).unwrap();
        assert_eq!(len, bytes.len() as u64);
        assert_eq!(digest, hex(&Sha256::digest(&bytes)));
        assert!(matches!(
            hash_file(&fixture.root, path, bytes.len() as u64 - 1),
            Err(DistributionError::TooLarge)
        ));
    }

    #[test]
    fn unlisted_disk_content_is_not_silently_accepted() {
        let fixture = Fixture::new();
        fs::write(fixture.root.join("extra.js"), "unexpected").unwrap();
        assert!(matches!(
            write_manifest(&fixture.root, VERSION, fixture.target),
            Err(DistributionError::UnknownEntry)
        ));
        #[cfg(unix)]
        {
            fs::remove_file(fixture.root.join("extra.js")).unwrap();
            fs::write(
                fixture.root.join("obsolete\\payload.js"),
                "hidden alias spelling",
            )
            .unwrap();
            assert!(matches!(
                write_manifest(&fixture.root, VERSION, fixture.target),
                Err(DistributionError::UnknownEntry)
            ));
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_file_or_directory_is_rejected_without_following_it() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        fixture.write();
        let script = fixture.root.join("runtime/node/LICENSE");
        fs::remove_file(&script).unwrap();
        symlink(fixture.root.join("bin/mirket"), script).unwrap();
        assert!(matches!(
            VerifiedRelease::open(&fixture.root),
            Err(DistributionError::Symlink)
        ));
        let fixture = Fixture::new();
        fs::remove_dir_all(fixture.root.join("runtime")).unwrap();
        symlink(fixture.root.join("bin"), fixture.root.join("runtime")).unwrap();
        assert!(matches!(
            write_manifest(&fixture.root, VERSION, fixture.target),
            Err(DistributionError::Symlink)
        ));
    }

    fn manifest_for(components: Vec<ReleaseComponent>) -> ReleaseManifest {
        let layout = layout(MAC_TARGET, &components).unwrap();
        ReleaseManifest {
            schema_version: 3,
            version: VERSION.into(),
            target: MAC_TARGET.into(),
            components,
            files: layout
                .files
                .iter()
                .map(|(path, expected)| {
                    let (byte_len, sha256) = expected
                        .pin
                        .clone()
                        .unwrap_or((0, hex(&Sha256::digest([]))));
                    ReleaseFile {
                        relative_path: path.clone(),
                        byte_len,
                        sha256,
                        mode: expected.mode,
                    }
                })
                .collect(),
            links: layout.links.values().cloned().collect(),
        }
    }

    fn admitted_manifest() -> ReleaseManifest {
        manifest_for(vec![ReleaseComponent::Design])
    }

    #[test]
    fn raster_selection_has_exact_separate_pins_for_local_installation() {
        let components = [ReleaseComponent::Design, ReleaseComponent::Raster];
        let mut candidate = layout(MAC_TARGET, &[ReleaseComponent::Design]).unwrap();
        let design_files = candidate.files.len();
        extend_raster_candidate(&mut candidate).unwrap();
        finish_layout(&mut candidate, MAC_TARGET, &components).unwrap();
        assert_eq!(candidate.files.len(), design_files + 125);
        assert_eq!(
            candidate
                .files
                .keys()
                .filter(|path| path.starts_with("runtime/design/raster/"))
                .count(),
            125
        );
        for (path, bytes, digest) in [
            (
                "runtime/design/raster/node_modules/@img/sharp-darwin-arm64/lib/sharp-darwin-arm64-0.35.5.node",
                280_256,
                "791921f975371be33aed727e06283282cc001c960792eb52505b579fdaaf1b55",
            ),
            (
                "runtime/design/raster/node_modules/@img/sharp-libvips-darwin-arm64/lib/libvips-cpp.8.18.7.dylib",
                18_231_688,
                "c1a5b5633b21aeb8a229abf7d062e07e4d778c0b44ed43cd1f93362bda58c322",
            ),
        ] {
            let selected = &candidate.files[path];
            assert_eq!(selected.mode, 0o600);
            assert_eq!(selected.pin, Some((bytes, digest.into())));
        }
        assert!(candidate.files.contains_key(RASTER_MODULE));
        assert!(layout(MAC_TARGET, &components).is_ok());
        assert!(matches!(
            layout(MAC_TARGET, &[ReleaseComponent::Raster]),
            Err(DistributionError::InvalidManifest)
        ));
        for target in [WINDOWS_X64_TARGET, WINDOWS_ARM64_TARGET] {
            assert!(matches!(
                layout(target, &components),
                Err(DistributionError::UnsupportedTarget)
            ));
        }
        // Extending the frozen selection twice is a duplicate, never an
        // override of an existing component's pins.
        assert!(matches!(
            extend_raster_candidate(&mut candidate),
            Err(DistributionError::InvalidManifest)
        ));
    }

    #[test]
    fn raster_payload_has_only_the_fixed_private_module_path() {
        let root = Path::new("/private/disposable-release");
        let payload = design_payload(
            root,
            "runtime/node/bin/node",
            &[ReleaseComponent::Design, ReleaseComponent::Raster],
        )
        .unwrap();
        assert_eq!(payload.raster_module, Some(root.join(RASTER_MODULE)));
        assert_eq!(payload.node, root.join("runtime/node/bin/node"));
        assert_eq!(payload.adapter, root.join(DESIGN_ADAPTER));
        assert!(
            design_payload(root, "runtime/node/bin/node", &[ReleaseComponent::Design])
                .unwrap()
                .raster_module
                .is_none()
        );
        assert!(
            design_payload(root, "runtime/node/bin/node", &[ReleaseComponent::Raster]).is_none()
        );
    }

    #[test]
    fn incomplete_raster_payload_cannot_create_a_manifest_or_installation() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.root.join(RASTER_ROOT)).unwrap();
        assert_eq!(
            infer_components(&fixture.root).unwrap(),
            vec![ReleaseComponent::Design, ReleaseComponent::Raster]
        );
        assert!(write_manifest(&fixture.root, VERSION, MAC_TARGET).is_err());
        assert!(!fixture.root.join(MANIFEST_NAME).exists());

        let fixture = Fixture::new();
        fixture.write();
        fixture.mutate_manifest(|manifest| {
            manifest["components"] = serde_json::json!(["design", "raster"]);
        });
        let home = fixture.root.join("must-not-initialize");
        crate::installation::Installation::setup(&home, &fixture.root)
            .err()
            .expect("incomplete Raster must fail before claiming a home");
        assert!(!home.exists());
    }

    #[test]
    #[ignore = "requires an explicitly assembled 125-file upstream Sharp fixture"]
    fn raster_private_fixture_matches_every_frozen_candidate_pin() {
        let root = PathBuf::from(
            std::env::var_os("MIRKET_RASTER_QUALIFICATION_FIXTURE")
                .expect("select the private restored fixture explicitly"),
        );
        let root = selected_root(&root).unwrap();
        let mut candidate = layout(MAC_TARGET, &[]).unwrap();
        let core: BTreeSet<_> = candidate.files.keys().cloned().collect();
        extend_raster_candidate(&mut candidate).unwrap();
        let files: Vec<_> = candidate
            .files
            .into_iter()
            .filter(|(path, _)| !core.contains(path))
            .map(|(relative_path, expected)| {
                let (byte_len, sha256) = expected.pin.unwrap();
                ReleaseFile {
                    relative_path,
                    byte_len,
                    sha256,
                    mode: expected.mode,
                }
            })
            .collect();
        assert_eq!(files.len(), 125);
        assert_eq!(
            files.iter().map(|file| file.byte_len).sum::<u64>(),
            19_698_667
        );
        verify_payload_files(&root, &files, false).unwrap();
    }

    #[test]
    fn semantic_component_has_exact_pins_and_rejects_partial_or_unsupported_layouts() {
        let manifest = manifest_for(vec![ReleaseComponent::Semantic]);
        let semantic: Vec<_> = manifest
            .files
            .iter()
            .filter(|entry| entry.relative_path.starts_with("runtime/semantic/"))
            .collect();
        assert_eq!(semantic.len(), 12);
        assert_eq!(
            semantic.iter().filter(|entry| entry.byte_len > 0).count(),
            11
        );
        assert!(semantic.iter().any(|entry| {
            entry.relative_path == "runtime/semantic/package-receipt.json" && entry.byte_len == 0
        }));
        assert!(semantic.iter().all(|entry| valid_digest(&entry.sha256)));
        assert!(semantic.iter().any(|entry| {
            entry.relative_path == "runtime/semantic/model/onnx/model_qint8_arm64.onnx"
                && entry.sha256
                    == "783fea82d71a58179b830a4dbd2d58447e640609e98eedf9ffa12622d375a672"
        }));
        let mut valid = manifest.clone();
        validate_manifest(&mut valid).unwrap();
        let mut missing = manifest.clone();
        missing.files.retain(|entry| {
            entry.relative_path != "runtime/semantic/model/onnx/model_qint8_arm64.onnx"
        });
        assert!(matches!(
            validate_manifest(&mut missing),
            Err(DistributionError::UnknownEntry)
        ));
        assert!(layout(WINDOWS_X64_TARGET, &[ReleaseComponent::Semantic]).is_err());
        assert!(layout(WINDOWS_ARM64_TARGET, &[ReleaseComponent::Semantic]).is_err());
        assert!(
            layout(
                MAC_TARGET,
                &[ReleaseComponent::Semantic, ReleaseComponent::Semantic]
            )
            .is_err()
        );
        assert!(
            layout(
                MAC_TARGET,
                &[ReleaseComponent::Semantic, ReleaseComponent::Design]
            )
            .is_err()
        );
    }

    #[test]
    fn admitted_design_profile_is_finite_and_pinned() {
        let mut manifest = admitted_manifest();
        let selected = validate_manifest(&mut manifest).unwrap();
        assert!(manifest.files.len() > 5_000);
        assert!(manifest.links.is_empty());
        assert!(selected.files["runtime/node/bin/node"].pin.is_none());
        assert!(selected.files[DESIGN_FONT].pin.is_some());
        let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        assert!(bytes.len() + 1 < selected.capacity.manifest_bytes);
    }

    #[test]
    fn all_admitted_components_compose_with_exact_inventory_capacity() {
        let mut manifest = manifest_for(vec![
            ReleaseComponent::Diagram,
            ReleaseComponent::Design,
            ReleaseComponent::OpenPencil,
            ReleaseComponent::Semantic,
            ReleaseComponent::Skills,
        ]);
        let selected = validate_manifest(&mut manifest).unwrap();
        assert!(manifest.files.len() > 6_000);
        assert_eq!(
            selected.capacity.inventory_entries,
            manifest.files.len() + manifest.links.len() + selected.directories.len() + 1
        );
        assert_eq!(
            selected.capacity.manifest_bytes,
            maximum_manifest_bytes().unwrap()
        );
        selected
            .capacity
            .check_manifest(serde_json::to_vec_pretty(&manifest).unwrap().len() + 1)
            .unwrap();
        manifest.files.push(ReleaseFile {
            relative_path: "runtime/unlisted".into(),
            byte_len: 1,
            sha256: "0".repeat(64),
            mode: 0o600,
        });
        assert!(matches!(
            validate_manifest(&mut manifest),
            Err(DistributionError::UnknownEntry)
        ));
    }

    #[test]
    fn third_party_pins_and_modes_cannot_be_redeclared() {
        let original = admitted_manifest();
        for path in [
            "adapters/node_modules/typescript/lib/typescript.js",
            DESIGN_FONT,
        ] {
            let mut changed = original.clone();
            changed
                .files
                .iter_mut()
                .find(|file| file.relative_path == path)
                .unwrap()
                .sha256 = "0".repeat(64);
            assert!(matches!(
                validate_manifest(&mut changed),
                Err(DistributionError::ContentMismatch)
            ));
        }
        let mut changed = original.clone();
        changed
            .files
            .iter_mut()
            .find(|file| file.relative_path == "runtime/node/bin/node")
            .unwrap()
            .mode = 0o600;
        assert!(validate_manifest(&mut changed).is_err());
        let mut changed = original.clone();
        changed
            .files
            .iter_mut()
            .find(|file| file.relative_path == DESIGN_ADAPTER)
            .unwrap()
            .sha256 = "1".repeat(64);
        validate_manifest(&mut changed).unwrap();
        assert_ne!(
            manifest_identity(&changed).unwrap(),
            manifest_identity(&original).unwrap()
        );
    }

    #[test]
    fn retired_browser_and_sql_trees_cannot_fall_back_to_core() {
        for path in ["runtime/sql/obsolete", "runtime/browser/unselected"] {
            let fixture = Fixture::new();
            fs::create_dir_all(fixture.root.join(path).parent().unwrap()).unwrap();
            fs::write(fixture.root.join(path), b"unlisted").unwrap();
            assert!(infer_components(&fixture.root).unwrap().is_empty());
            assert!(write_manifest(&fixture.root, VERSION, MAC_TARGET).is_err());
            assert!(!fixture.root.join(MANIFEST_NAME).exists());
        }
        let original = admitted_manifest();
        let mut stale = serde_json::to_value(&original).unwrap();
        for retired in ["sql", "browser", "documents", "python_navigation", "c_cpp"] {
            stale["components"] = serde_json::json!([retired]);
            assert!(serde_json::from_value::<ReleaseManifest>(stale.clone()).is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn source_modes_are_normalized_but_installed_modes_are_exact() {
        use std::os::unix::fs::PermissionsExt;
        if native_target().is_err() {
            return;
        }
        let fixture = Fixture::new();
        let layout = layout(fixture.target, &[]).unwrap();
        for (path, expected) in &layout.files {
            let source_mode = if expected.mode == 0o700 { 0o775 } else { 0o664 };
            fs::set_permissions(
                fixture.root.join(path),
                fs::Permissions::from_mode(source_mode),
            )
            .unwrap();
        }
        let id = fixture.write();
        assert_eq!(VerifiedRelease::open(&fixture.root).unwrap().id(), &id);
        assert!(VerifiedRelease::open_installed(&fixture.root).is_err());
        for (path, expected) in &layout.files {
            fs::set_permissions(
                fixture.root.join(path),
                fs::Permissions::from_mode(expected.mode),
            )
            .unwrap();
        }
        for path in std::iter::once("").chain(layout.directories.iter().map(String::as_str)) {
            fs::set_permissions(fixture.root.join(path), fs::Permissions::from_mode(0o700))
                .unwrap();
        }
        assert_eq!(
            VerifiedRelease::open_installed(&fixture.root).unwrap().id(),
            &id
        );
        let manifest = fixture.root.join(MANIFEST_NAME);
        fs::set_permissions(&manifest, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(VerifiedRelease::open_installed(&fixture.root).is_err());
        fs::set_permissions(manifest, fs::Permissions::from_mode(0o600)).unwrap();
        fs::set_permissions(
            fixture.root.join(layout.executable),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        assert!(VerifiedRelease::open(&fixture.root).is_err());
    }
}
