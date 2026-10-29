//! Frozen local Video installation inputs. The ordinary layout checks every
//! selected byte and uses Chrome for Testing only for this renderer. Public
//! redistribution remains a separate source/notice decision.

use super::*;

pub(super) const ROOT: &str = "runtime/video";
const ADAPTER: &str = "runtime/video/dist/video-main.js";
pub(super) const ADAPTER_FILES: [&str; 3] = [
    ADAPTER,
    "runtime/video/dist/video-export.js",
    "runtime/video/dist/video-policy.js",
];
const FFMPEG: &str = "runtime/video/bin/ffmpeg";
const FFPROBE: &str = "runtime/video/bin/ffprobe";
const SHARP_PACKAGES: [&str; 6] = [
    "sharp",
    "@img/colour",
    "detect-libc",
    "semver",
    "@img/sharp-darwin-arm64",
    "@img/sharp-libvips-darwin-arm64",
];

#[derive(Deserialize)]
struct Candidate {
    schema: u32,
    status: String,
    release_admitted: bool,
    target: String,
    hyperframes_version: String,
    package_lock_sha256: String,
    source_manifest_sha256: Option<String>,
    package_instances: usize,
    runtime_inventory: RasterInventory,
    media_files: Vec<DesignFile>,
}

pub(super) fn extend(result: &mut Layout) -> Result<(), DistributionError> {
    let source =
        include_bytes!("../../packaging/admission/m3-video-sharp-free-macos.runtime.json.proposed");
    let candidate: Candidate =
        serde_json::from_slice(source).map_err(|_| DistributionError::InvalidManifest)?;
    let admitted_lock =
        include_bytes!("../../packaging/admission/pi-native-base-package-lock.json");
    let current_lock = include_bytes!("../../pi/package-lock.json");
    if !design_lock_preserved(admitted_lock, current_lock)
        || candidate.schema != 1
        || candidate.status != "fresh_current_local_video_candidate_not_release_admitted"
        || candidate.release_admitted
        || candidate.target != MAC_TARGET
        || candidate.hyperframes_version != "0.8.197"
        || candidate.package_instances != 82
        || candidate.package_lock_sha256
            != hex(&Sha256::digest(include_bytes!(
                "../../packaging/admission/pi-native-base-package-lock.json"
            )))
        || candidate.source_manifest_sha256.is_some()
        || candidate.runtime_inventory.regular_files != 5_892
        || candidate.runtime_inventory.files.len() != 5_892
        || candidate.runtime_inventory.regular_bytes != 65_473_567
        || candidate.media_files.len() != 2
    {
        return Err(DistributionError::InvalidManifest);
    }
    let mut bytes = 0u64;
    for entry in candidate.runtime_inventory.files {
        if !safe_relative(&entry.path)
            || !entry.path.starts_with("runtime/video/")
            || entry.path.starts_with("runtime/video/bin/")
            || entry.path.contains("/.bin/")
            || !valid_digest(&entry.sha256)
            || SHARP_PACKAGES.iter().any(|package| {
                entry
                    .path
                    .starts_with(&format!("{ROOT}/node_modules/{package}/"))
            })
        {
            return Err(DistributionError::InvalidManifest);
        }
        bytes = bytes
            .checked_add(entry.bytes)
            .ok_or(DistributionError::TooLarge)?;
        // This is the one selected native esbuild helper. Package CLI/install
        // scripts are data; inventory never executes them or upstream hooks.
        let mode = if entry.path == "runtime/video/node_modules/@esbuild/darwin-arm64/bin/esbuild" {
            0o700
        } else {
            0o600
        };
        insert(result, entry, mode)?;
    }
    if bytes != candidate.runtime_inventory.regular_bytes
        || !result.files.contains_key(ADAPTER)
        || !result
            .files
            .contains_key("runtime/video/node_modules/.mirket-video-policy.json")
    {
        return Err(DistributionError::InvalidManifest);
    }
    for (entry, path) in candidate.media_files.into_iter().zip([FFMPEG, FFPROBE]) {
        if entry.path != path || entry.bytes == 0 || !valid_digest(&entry.sha256) {
            return Err(DistributionError::InvalidManifest);
        }
        insert(result, entry, 0o700)?;
    }
    // Reuse the existing private Node selection, not a second Video runtime.
    let nodes: NodeSelection = serde_json::from_slice(include_bytes!(
        "../../packaging/admission/m0-node.selection.json"
    ))
    .map_err(|_| DistributionError::InvalidManifest)?;
    let mut targets = nodes
        .targets
        .into_iter()
        .filter(|target| target.target == MAC_TARGET);
    let selected = targets.next().ok_or(DistributionError::InvalidManifest)?;
    if targets.next().is_some() || selected.selected_files.len() != 2 {
        return Err(DistributionError::InvalidManifest);
    }
    let mut seen = BTreeSet::new();
    for entry in selected.selected_files {
        if !matches!(
            entry.destination_relative.as_str(),
            "runtime/node/bin/node" | "runtime/node/LICENSE"
        ) || !valid_digest(&entry.sha256)
            || !seen.insert(entry.destination_relative.clone())
        {
            return Err(DistributionError::InvalidManifest);
        }
        let expected = result
            .files
            .get_mut(&entry.destination_relative)
            .ok_or(DistributionError::InvalidManifest)?;
        let pin = (entry.bytes, entry.sha256);
        if expected.pin.as_ref().is_some_and(|prior| prior != &pin) {
            return Err(DistributionError::InvalidManifest);
        }
        expected.pin = Some(pin);
    }
    Ok(())
}

fn insert(result: &mut Layout, entry: DesignFile, mode: u32) -> Result<(), DistributionError> {
    // These are Mirket-authored adapters, like adapters/design-main.js. Their
    // current bytes are bound by the release manifest, not an older vendor
    // admission snapshot. Every third-party file retains its compiled pin.
    let pin = if ADAPTER_FILES.contains(&entry.path.as_str()) {
        None
    } else {
        Some((entry.bytes, entry.sha256))
    };
    if result
        .files
        .insert(entry.path, ExpectedFile { mode, pin })
        .is_some()
    {
        return Err(DistributionError::InvalidManifest);
    }
    Ok(())
}

pub(super) fn payload(
    root: &Path,
    node: &str,
    components: &[ReleaseComponent],
) -> Option<crate::video::VideoPayload> {
    if !components.contains(&ReleaseComponent::Video) {
        return None;
    }
    Some(crate::video::VideoPayload {
        node: root.join(node),
        adapter: root.join(ADAPTER),
        chromium: root.join(super::chrome_admission::executable(MAC_TARGET)),
        ffmpeg: root.join(FFMPEG),
        ffprobe: root.join(FFPROBE),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
                "mirket-video-distribution-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&root).unwrap();
            for (relative, expected) in &layout(MAC_TARGET, &[]).unwrap().files {
                let path = root.join(relative);
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, b"qualification fixture").unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(expected.mode)).unwrap();
                }
                #[cfg(not(unix))]
                let _ = expected;
            }
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn candidate_freezes_the_reduced_closure_and_shared_node_but_no_browser() {
        let mut candidate = layout(MAC_TARGET, &[]).unwrap();
        extend(&mut candidate).unwrap();
        finish_layout(&mut candidate, MAC_TARGET, &[ReleaseComponent::Video]).unwrap();
        assert_eq!(candidate.files.len(), 3 + 5_892 + 2);
        assert_eq!(
            candidate.files["runtime/node/bin/node"]
                .pin
                .as_ref()
                .unwrap()
                .1,
            "6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329"
        );
        assert!(
            candidate
                .files
                .keys()
                .all(|path| !path.starts_with("runtime/browser/"))
        );
        assert!(candidate.files.keys().all(|path| {
            !SHARP_PACKAGES
                .iter()
                .any(|package| path.starts_with(&format!("{ROOT}/node_modules/{package}/")))
        }));
        assert_eq!(candidate.files[FFMPEG].mode, 0o700);
        assert!(candidate.files[FFMPEG].pin.is_some());
        assert!(candidate.files[FFPROBE].pin.is_some());
        for path in ADAPTER_FILES {
            assert!(candidate.files[path].pin.is_none());
            assert_eq!(
                candidate
                    .file_byte_limit(path, &candidate.files[path])
                    .unwrap(),
                UnpinnedFileRole::JavaScriptAdapter.byte_limit()
            );
        }
        assert!(candidate.files.iter().all(|(path, file)| {
            !path.starts_with(ROOT) || ADAPTER_FILES.contains(&path.as_str()) || file.pin.is_some()
        }));
        assert_eq!(
            candidate.files["runtime/video/node_modules/puppeteer/install.mjs"].mode,
            0o600
        );
        assert!(matches!(
            extend(&mut candidate),
            Err(DistributionError::InvalidManifest)
        ));
    }

    #[test]
    fn payload_selects_only_the_private_video_chrome() {
        let root = Path::new("/private/disposable-release");
        let components = [ReleaseComponent::Video];
        assert!(payload(root, "runtime/node/bin/node", &[]).is_none());
        let video = payload(root, "runtime/node/bin/node", &components).unwrap();
        assert_eq!(video.node, root.join("runtime/node/bin/node"));
        assert_eq!(video.adapter, root.join(ADAPTER));
        assert_eq!(
            video.chromium,
            root.join(chrome_admission::executable(MAC_TARGET))
        );
        assert_eq!(video.ffmpeg, root.join(FFMPEG));
        assert_eq!(video.ffprobe, root.join(FFPROBE));
    }

    #[test]
    fn incomplete_video_cannot_create_manifest_or_initialize_installation() {
        assert!(layout(MAC_TARGET, &[ReleaseComponent::Video]).is_ok());
        for target in [WINDOWS_X64_TARGET, WINDOWS_ARM64_TARGET] {
            assert!(matches!(
                layout(target, &[ReleaseComponent::Video]),
                Err(DistributionError::UnsupportedTarget)
            ));
        }
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.0.join(ROOT)).unwrap();
        assert_eq!(
            infer_components(&fixture.0).unwrap(),
            vec![ReleaseComponent::Video]
        );
        assert!(write_manifest(&fixture.0, VERSION, MAC_TARGET).is_err());
        assert!(!fixture.0.join(MANIFEST_NAME).exists());

        let fixture = Fixture::new();
        write_manifest(&fixture.0, VERSION, MAC_TARGET).unwrap();
        assert!(
            VerifiedRelease::open(&fixture.0)
                .unwrap()
                .payload()
                .video
                .is_none()
        );
        let path = fixture.0.join(MANIFEST_NAME);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        manifest["components"] = serde_json::json!(["video"]);
        fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let home = fixture.0.join("must-not-initialize");
        crate::installation::Installation::setup(&home, &fixture.0)
            .err()
            .expect("incomplete Video must fail before claiming a home");
        assert!(!home.exists());
    }
}
