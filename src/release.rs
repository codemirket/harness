//! Assemble verified native release manifests without running their assets.
use crate::{
    update::{ReleaseAsset, ReleaseManifest, validate_manifest},
    util::{ensure_directory, read_bounded, reject_symlink_components},
};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

const MAX_ASSET_BYTES: u64 = 256 * 1024 * 1024;

pub fn manifest(inputs: &[PathBuf], output: &Path) -> Result<Value> {
    ensure!(
        (1..=16).contains(&inputs.len()),
        "provide between 1 and 16 input manifests"
    );
    ensure!(
        output.file_name().is_some(),
        "output must name a new manifest file"
    );
    require_new_output(output)?;

    let mut version: Option<String> = None;
    let mut assets = BTreeMap::<String, ReleaseAsset>::new();
    for input in inputs {
        let bytes = read_bounded(input, 1024 * 1024)
            .with_context(|| format!("read release manifest {}", input.display()))?;
        let incoming: ReleaseManifest = serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid release manifest {}", input.display()))?;
        validate_manifest(&incoming)
            .with_context(|| format!("validate release manifest {}", input.display()))?;
        if let Some(expected) = &version {
            ensure!(
                incoming.version == *expected,
                "input manifests have different versions: {expected} and {}",
                incoming.version
            );
        } else {
            version = Some(incoming.version.clone());
        }
        let parent = input
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        for asset in incoming.assets {
            ensure!(
                !assets.contains_key(&asset.target),
                "input manifests repeat target {}",
                asset.target
            );
            verify_local_asset(parent, &asset)?;
            assets.insert(asset.target.clone(), asset);
        }
    }
    let combined = ReleaseManifest {
        version: version.context("no release version was provided")?,
        assets: assets.into_values().collect(),
    };
    validate_manifest(&combined)?;
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    ensure_directory(parent)?;
    let mut staged = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(staged.as_file_mut(), &combined)?;
    staged.write_all(b"\n")?;
    staged.as_file().sync_all()?;
    // Recheck path components after staging; publication itself never replaces an existing file.
    require_new_output(output)?;
    staged
        .persist_noclobber(output)
        .with_context(|| format!("publish new release manifest {}", output.display()))?;
    Ok(json!({"ok":true,"output":output,"version":combined.version,
        "targets":combined.assets.iter().map(|asset|&asset.target).collect::<Vec<_>>()}))
}

fn require_new_output(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => bail!("release manifest output already exists: {}", path.display()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    reject_symlink_components(path)
}

fn verify_local_asset(parent: &Path, asset: &ReleaseAsset) -> Result<()> {
    let url = reqwest::Url::parse(&asset.url)?;
    let name = url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        .context("release asset URL has no filename")?;
    // Native dist names are plain ASCII. Reject encoded delimiters and platform-specific path forms.
    ensure!(
        !name.is_empty()
            && name.len() <= 255
            && name != "."
            && name != ".."
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')),
        "release asset must use a simple local filename"
    );
    let path = parent.join(name);
    reject_symlink_components(&path)?;
    let parent = parent
        .canonicalize()
        .context("release asset directory does not exist")?;
    ensure!(
        path.canonicalize()?.parent() == Some(parent.as_path()),
        "release asset escaped its manifest directory"
    );
    let before = fs::symlink_metadata(&path)?;
    ensure!(
        before.is_file() && before.len() > 0 && before.len() <= MAX_ASSET_BYTES,
        "release asset must be a nonempty regular file no larger than 256 MiB"
    );
    let mut file = fs::File::open(&path)?;
    let opened = file.metadata()?;
    ensure!(
        opened.is_file() && opened.len() == before.len(),
        "release asset changed before verification"
    );
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        ensure!(total <= MAX_ASSET_BYTES, "release asset exceeds 256 MiB");
        digest.update(&buffer[..count]);
    }
    let after = file.metadata()?;
    reject_symlink_components(&path)?;
    ensure!(
        total == opened.len()
            && after.len() == opened.len()
            && after.modified()? == opened.modified()?,
        "release asset changed during verification"
    );
    let actual = format!("{:x}", digest.finalize());
    ensure!(
        actual.eq_ignore_ascii_case(&asset.sha256),
        "release asset SHA-256 differs from its manifest: {}",
        path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::sha256;

    fn input(root: &Path, directory: &str, target: &str, version: &str) -> PathBuf {
        let directory = root.join(directory);
        fs::create_dir_all(&directory).unwrap();
        let name = format!(
            "mirket-{target}{}",
            if target.contains("windows") {
                ".exe"
            } else {
                ""
            }
        );
        let bytes = format!("native test artifact for {target} {version}");
        fs::write(directory.join(&name), &bytes).unwrap();
        let manifest = ReleaseManifest {
            version: version.into(),
            assets: vec![ReleaseAsset {
                target: target.into(),
                url: format!(
                    "https://github.com/codemirket/harness/releases/download/mirket-v{version}/{name}"
                ),
                sha256: sha256(bytes.as_bytes()),
            }],
        };
        let path = directory.join("mirket-release.json");
        fs::write(&path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
        path
    }

    #[test]
    fn merge_verifies_local_assets_preserves_inputs_and_sorts_targets() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let a = input(&root, "mac", "aarch64-apple-darwin", "1.0.0");
        let b = input(&root, "windows", "x86_64-pc-windows-msvc", "1.0.0");
        let original_a = fs::read(&a).unwrap();
        let original_b = fs::read(&b).unwrap();
        let output = root.join("combined/mirket-release.json");
        let report = manifest(&[b.clone(), a.clone()], &output).unwrap();
        assert_eq!(report["version"], "1.0.0");
        assert_eq!(
            report["targets"],
            json!(["aarch64-apple-darwin", "x86_64-pc-windows-msvc"])
        );
        let combined: ReleaseManifest = serde_json::from_slice(&fs::read(output).unwrap()).unwrap();
        validate_manifest(&combined).unwrap();
        assert_eq!(combined.assets.len(), 2);
        assert_eq!(fs::read(&a).unwrap(), original_a);
        assert_eq!(fs::read(&b).unwrap(), original_b);
        assert_eq!(
            fs::read_to_string(a.parent().unwrap().join("mirket-aarch64-apple-darwin")).unwrap(),
            "native test artifact for aarch64-apple-darwin 1.0.0"
        );
        assert_eq!(
            fs::read_to_string(
                b.parent()
                    .unwrap()
                    .join("mirket-x86_64-pc-windows-msvc.exe")
            )
            .unwrap(),
            "native test artifact for x86_64-pc-windows-msvc 1.0.0"
        );
    }

    #[test]
    fn duplicate_targets_mixed_versions_and_invalid_inputs_publish_nothing() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let a = input(&root, "mac", "aarch64-apple-darwin", "1.0.0");
        let duplicate = input(&root, "other-mac", "aarch64-apple-darwin", "1.0.0");
        let different = input(&root, "windows", "x86_64-pc-windows-msvc", "1.0.1");
        let output = root.join("combined.json");
        assert!(
            manifest(&[a.clone(), duplicate], &output)
                .unwrap_err()
                .to_string()
                .contains("repeat target")
        );
        assert!(
            manifest(&[a.clone(), different], &output)
                .unwrap_err()
                .to_string()
                .contains("different versions")
        );
        assert!(manifest(&[], &output).is_err());
        assert!(manifest(&vec![a; 17], &output).is_err());
        assert!(!output.exists());
    }

    #[test]
    fn tampered_missing_and_empty_assets_are_rejected_without_execution() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let a = input(&root, "mac", "aarch64-apple-darwin", "1.0.0");
        let artifact = a.parent().unwrap().join("mirket-aarch64-apple-darwin");
        let output = root.join("combined.json");
        fs::write(&artifact, "changed bytes").unwrap();
        assert!(
            manifest(std::slice::from_ref(&a), &output)
                .unwrap_err()
                .to_string()
                .contains("SHA-256")
        );
        fs::write(&artifact, "").unwrap();
        assert!(manifest(std::slice::from_ref(&a), &output).is_err());
        fs::remove_file(&artifact).unwrap();
        assert!(manifest(&[a], &output).is_err());
        assert!(!output.exists());
    }

    #[test]
    fn existing_output_is_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let a = input(&root, "mac", "aarch64-apple-darwin", "1.0.0");
        let output = root.join("combined.json");
        fs::write(&output, "existing content").unwrap();
        assert!(manifest(&[a], &output).is_err());
        assert_eq!(fs::read_to_string(output).unwrap(), "existing content");
    }

    #[test]
    fn encoded_path_delimiters_cannot_select_files_outside_the_manifest_directory() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let a = input(&root, "mac", "aarch64-apple-darwin", "1.0.0");
        let mut data: ReleaseManifest = serde_json::from_slice(&fs::read(&a).unwrap()).unwrap();
        data.assets[0].url="https://github.com/codemirket/harness/releases/download/mirket-v1.0.0/%2e%2e%2foutside".into();
        fs::write(&a, serde_json::to_vec(&data).unwrap()).unwrap();
        let output = root.join("combined.json");
        assert!(manifest(&[a], &output).is_err());
        assert!(!output.exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_assets_and_dangling_output_symlinks_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let a = input(&root, "mac", "aarch64-apple-darwin", "1.0.0");
        let artifact = a.parent().unwrap().join("mirket-aarch64-apple-darwin");
        let outside = root.join("outside");
        fs::rename(&artifact, &outside).unwrap();
        std::os::unix::fs::symlink(&outside, &artifact).unwrap();
        let output = root.join("combined.json");
        assert!(manifest(std::slice::from_ref(&a), &output).is_err());
        assert!(!output.exists());
        std::os::unix::fs::symlink(root.join("missing"), &output).unwrap();
        assert!(
            manifest(&[a], &output)
                .unwrap_err()
                .to_string()
                .contains("already exists")
        );
        assert!(
            fs::symlink_metadata(output)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(outside.is_file());
    }
}
