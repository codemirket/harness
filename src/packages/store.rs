use super::{Registry, archive, notice};
use crate::{paths::Paths, util};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
    schema_version: u32,
    id: String,
    version: String,
    target: String,
    definition: String,
    pub files: BTreeMap<String, EntryIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<super::container::ImageIdentity>,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum EntryIdentity {
    File(FileIdentity),
    Link(LinkIdentity),
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LinkIdentity {
    symlink: String,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileIdentity {
    pub sha256: String,
    bytes: u64,
    executable: bool,
}

pub(super) fn file_identity(path: &Path, max: u64) -> Result<FileIdentity> {
    util::reject_symlink_components(path)?;
    let before = fs::symlink_metadata(path)?;
    if !before.is_file() || before.len() > max {
        bail!("package file is not regular or exceeds size limit");
    }
    let file = fs::File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > max {
        bail!("package file is not regular or exceeds size limit");
    }
    let mut hasher = Sha256::new();
    let mut reader = file.take(max + 1);
    let mut buffer = [0u8; 64 * 1024];
    let mut count = 0;
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        hasher.update(&buffer[..n]);
    }
    if count != meta.len() || count > max {
        bail!("package file size changed while hashing");
    }
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = false;
    Ok(FileIdentity {
        sha256: format!("{:x}", hasher.finalize()),
        bytes: count,
        executable,
    })
}

fn inventory(
    root: &Path,
    application_roots: &[PathBuf],
) -> Result<BTreeMap<String, EntryIdentity>> {
    let mut result = BTreeMap::new();
    let mut total = 0u64;
    for file in archive::files(root, application_roots)? {
        let identity = if fs::symlink_metadata(&file)?.file_type().is_symlink() {
            let app = application_roots
                .iter()
                .find(|app| file.starts_with(app))
                .context("package link has no reviewed application root")?;
            EntryIdentity::Link(LinkIdentity {
                symlink: super::dmg::link_target(app, &file)?,
            })
        } else {
            let identity = file_identity(&file, 2 * 1024 * 1024 * 1024)?;
            total = total
                .checked_add(identity.bytes)
                .context("package size overflow")?;
            EntryIdentity::File(identity)
        };
        if total > 4 * 1024 * 1024 * 1024 {
            bail!("package exceeds installed size limit");
        }
        let relative = archive::relative_name(root, &file)?;
        result.insert(relative, identity);
    }
    if result.is_empty() {
        bail!("package contains no files");
    }
    Ok(result)
}

fn application_roots(root: &Path, package: &super::Package, target: &str) -> Result<Vec<PathBuf>> {
    let platform = package
        .platforms
        .get(target)
        .context("package target is unavailable")?;
    Ok(package
        .common
        .iter()
        .chain(&platform.artifacts)
        .filter(|artifact| artifact.format == "dmg-app")
        .map(|artifact| root.join(&artifact.destination))
        .collect())
}

pub(super) fn verify(paths: &Paths, registry: &Registry, id: &str) -> Result<Receipt> {
    verify_target(paths, registry, id, crate::update::target_triple()?)
}
pub(super) fn verify_target(
    paths: &Paths,
    registry: &Registry,
    id: &str,
    target: &str,
) -> Result<Receipt> {
    let root = registry.root_for(paths, id, target)?;
    let bytes = util::read_bounded(&root.join(archive::RECEIPT), 16 * 1024 * 1024)
        .with_context(|| format!("package {id} is missing or incomplete; run mirket setup"))?;
    let receipt: Receipt = serde_json::from_slice(&bytes).context("invalid package receipt")?;
    let package = registry.get(id)?;
    if receipt.schema_version != 1
        || receipt.id != id
        || receipt.version != package.version
        || receipt.target != target
        || receipt.definition != registry.identity(id, target)?
        || receipt.files.len() > archive::MAX_FILES
    {
        bail!("package {id} receipt differs from curated definition");
    }
    for path in receipt.files.keys() {
        archive::normal_path(path)?;
    }
    if receipt.files != inventory(&root, &application_roots(&root, package, target)?)? {
        bail!("package {id} content has drifted; preserve or reconcile it before setup or launch");
    }
    if let Some(image) = &receipt.container {
        super::container::verify(paths, image)?;
    }
    if package.container.is_some() && target.contains("linux") && receipt.container.is_none() {
        bail!("container image receipt is missing");
    }
    Ok(receipt)
}

#[cfg(test)]
pub(super) fn install(paths: &Paths, registry: &Registry, id: &str) -> Result<()> {
    install_target(paths, registry, id, crate::update::target_triple()?)
}
pub(super) fn install_target(
    paths: &Paths,
    registry: &Registry,
    id: &str,
    target: &str,
) -> Result<()> {
    let root = registry.root_for(paths, id, target)?;
    util::reject_symlink_components(&root)?;
    if root.exists() {
        verify_target(paths, registry, id, target)?;
        return Ok(());
    }
    let parent = root.parent().context("package store parent")?;
    util::ensure_directory(parent)?;
    let staging = tempfile::Builder::new()
        .prefix(".prepare-")
        .tempdir_in(parent)?;
    let package = registry.get(id)?;
    let platform = package
        .platforms
        .get(target)
        .context("package unavailable on this target")?;
    eprintln!(
        "mirket package: preparing {id} {} ({} pinned archives)",
        package.version,
        package.common.len() + platform.artifacts.len()
    );
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(240))
        .redirect(reqwest::redirect::Policy::limited(8))
        .user_agent(concat!("mirket/", env!("CARGO_PKG_VERSION")))
        .build()?;
    for artifact in package.common.iter().chain(&platform.artifacts) {
        let download = download(paths, artifact, &client)?;
        archive::extract(artifact, &download, staging.path())
            .with_context(|| format!("extract pinned artifact for {id}"))?;
    }
    super::embedded::stage(staging.path(), &package.embedded)?;
    for name in &package.notices {
        let path = staging.path().join("licenses").join(name);
        if path.exists() {
            bail!("notice collides with package payload");
        }
        util::atomic_write(&path, notice(name)?)?;
    }
    if !platform.executable.starts_with('{') && !staging.path().join(&platform.executable).is_file()
    {
        bail!("package {id} executable is absent from staged payload");
    }
    let receipt = Receipt {
        schema_version: 1,
        id: id.into(),
        version: package.version.clone(),
        target: target.into(),
        definition: registry.identity(id, target)?,
        files: inventory(
            staging.path(),
            &application_roots(staging.path(), package, target)?,
        )?,
        container: if package.container.is_some() && target.contains("linux") {
            Some(super::container::build(
                paths,
                registry,
                id,
                target,
                staging.path(),
            )?)
        } else {
            None
        },
    };
    util::atomic_write(
        &staging.path().join(archive::RECEIPT),
        &serde_json::to_vec_pretty(&receipt)?,
    )?;
    // Setup holds its operation lock throughout preparation and activation.
    // No published generation is overwritten or modified in place.
    if root.exists() {
        bail!("package generation appeared during preparation");
    }
    fs::rename(staging.path(), &root).context("publish prepared package generation")?;
    verify_target(paths, registry, id, target)?;
    Ok(())
}

pub(super) fn download(
    paths: &Paths,
    artifact: &super::Artifact,
    client: &reqwest::blocking::Client,
) -> Result<PathBuf> {
    let cache = paths.root.join("packages").join("cache");
    util::ensure_directory(&cache)?;
    let path = cache.join(&artifact.sha256);
    util::reject_symlink_components(&path)?;
    if path.exists() {
        if file_identity(&path, artifact.max_download_bytes)?.sha256 != artifact.sha256 {
            bail!("cached package archive differs from its pin");
        }
        return Ok(path);
    }
    let mut response = client.get(&artifact.url).send()?.error_for_status()?;
    if response
        .content_length()
        .is_some_and(|n| n > artifact.max_download_bytes)
    {
        bail!("package download exceeds byte limit");
    }
    let mut temporary = tempfile::NamedTempFile::new_in(&cache)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut count = 0u64;
    loop {
        let n = response.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > artifact.max_download_bytes {
            bail!("package download exceeds byte limit");
        }
        hash.update(&buffer[..n]);
        temporary.write_all(&buffer[..n])?;
    }
    if format!("{:x}", hash.finalize()) != artifact.sha256 {
        bail!("package archive SHA-256 does not match its curated pin");
    }
    temporary.as_file().sync_all()?;
    temporary
        .persist_noclobber(&path)
        .context("publish verified archive cache")?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_payload_is_receipted_and_drift_blocks_reinstallation() {
        let home = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let mut registry = Registry::embedded().unwrap();
        let mut package = registry.get("playwright").unwrap().clone();
        package.dependencies.clear();
        package.notices.clear();
        package.common.clear();
        package.embedded = vec![super::super::Member {
            path: "README.md".into(),
            destination: "mirket/payload.txt".into(),
            executable: false,
        }];
        package.platforms = BTreeMap::from([(
            crate::update::target_triple().unwrap().into(),
            super::super::Platform {
                artifacts: vec![],
                executable: "mirket/payload.txt".into(),
            },
        )]);
        registry.packages.insert("playwright".into(), package);
        install(&paths, &registry, "playwright").unwrap();
        let root = registry.root(&paths, "playwright").unwrap();
        let payload = root.join("mirket/payload.txt");
        let receipt = verify(&paths, &registry, "playwright").unwrap();
        assert_eq!(receipt.files.len(), 1);
        assert_eq!(
            fs::read(&payload).unwrap(),
            include_bytes!("../../runtime/packages/README.md")
        );
        fs::write(&payload, b"changed owned runner").unwrap();
        assert!(
            install(&paths, &registry, "playwright")
                .unwrap_err()
                .to_string()
                .contains("drifted")
        );
        assert_eq!(fs::read(payload).unwrap(), b"changed owned runner");
    }

    #[test]
    fn corrupted_cached_archive_prevents_publication_and_cleans_staging() {
        let home = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let cache = paths.root.join("packages/cache");
        fs::create_dir_all(&cache).unwrap();
        let good = b"reviewed executable fixture";
        let other = b"reviewed second file";
        let artifacts = [good.as_slice(), other.as_slice()]
            .into_iter()
            .enumerate()
            .map(|(index, bytes)| super::super::Artifact {
                url: "https://example.invalid/pinned-file".into(),
                sha256: util::sha256(bytes),
                format: "file".into(),
                max_download_bytes: 1024,
                max_unpacked_bytes: 1024,
                strip_prefix: None,
                destination: String::new(),
                files: vec![super::super::Member {
                    path: format!("file-{index}"),
                    destination: format!("file-{index}"),
                    executable: index == 0,
                }],
            })
            .collect::<Vec<_>>();
        fs::write(cache.join(&artifacts[0].sha256), good).unwrap();
        fs::write(
            cache.join(&artifacts[1].sha256),
            b"corrupted cache contents",
        )
        .unwrap();
        let mut registry = Registry::embedded().unwrap();
        let mut package = registry.get("playwright").unwrap().clone();
        package.dependencies.clear();
        package.notices.clear();
        package.common = artifacts.clone();
        package.platforms = BTreeMap::from([(
            crate::update::target_triple().unwrap().into(),
            super::super::Platform {
                artifacts: vec![],
                executable: "file-0".into(),
            },
        )]);
        registry.packages.insert("playwright".into(), package);
        let generation = registry.root(&paths, "playwright").unwrap();
        let error = install(&paths, &registry, "playwright").unwrap_err();
        assert!(error.to_string().contains("differs from its pin"));
        assert!(!generation.exists());
        assert_eq!(
            fs::read_dir(generation.parent().unwrap()).unwrap().count(),
            0
        );
        fs::write(cache.join(&artifacts[1].sha256), other).unwrap();
        install(&paths, &registry, "playwright").unwrap();
        let receipt = verify(&paths, &registry, "playwright").unwrap();
        assert_eq!(receipt.files.len(), 2);
        assert_eq!(fs::read(generation.join("file-0")).unwrap(), good);
        fs::write(generation.join("file-0"), b"user-modified content").unwrap();
        assert!(
            install(&paths, &registry, "playwright")
                .unwrap_err()
                .to_string()
                .contains("drifted")
        );
        assert_eq!(
            fs::read(generation.join("file-0")).unwrap(),
            b"user-modified content"
        );
    }
    #[test]
    fn inventory_detects_extra_missing_and_changed_files() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        fs::write(temp.path().join("one"), b"one").unwrap();
        let expected = inventory(temp.path(), &[]).unwrap();
        fs::write(temp.path().join("two"), b"two").unwrap();
        assert_ne!(expected, inventory(temp.path(), &[]).unwrap());
        fs::remove_file(temp.path().join("two")).unwrap();
        fs::write(temp.path().join("one"), b"two").unwrap();
        assert_ne!(expected, inventory(temp.path(), &[]).unwrap());
        fs::remove_file(temp.path().join("one")).unwrap();
        assert!(inventory(temp.path(), &[]).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn inventory_rejects_links_and_checks_executable_bit() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let file = temp.path().join("one");
        fs::write(&file, b"one").unwrap();
        let expected = inventory(temp.path(), &[]).unwrap();
        util::set_executable_mode(&file, true).unwrap();
        assert_ne!(expected, inventory(temp.path(), &[]).unwrap());
        std::os::unix::fs::symlink(&file, temp.path().join("link")).unwrap();
        assert!(inventory(temp.path(), &[]).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn dmg_inventory_records_scoped_link_targets_and_detects_drift() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let app = temp.path().join("Fixture.app");
        for version in ["A", "B"] {
            fs::create_dir_all(app.join("Versions").join(version)).unwrap();
            fs::write(
                app.join("Versions").join(version).join("library"),
                b"identical bytes",
            )
            .unwrap();
        }
        let link = app.join("Versions/Current");
        std::os::unix::fs::symlink("A", &link).unwrap();
        let scope = [app.clone()];
        let expected = inventory(temp.path(), &scope).unwrap();
        assert!(matches!(
            expected.get("Fixture.app/Versions/Current"),
            Some(EntryIdentity::Link(_))
        ));
        assert!(inventory(temp.path(), &[]).is_err());
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink("B", &link).unwrap();
        assert_ne!(expected, inventory(temp.path(), &scope).unwrap());
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink("../../../outside", &link).unwrap();
        assert!(inventory(temp.path(), &scope).is_err());
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(
            "Fixture.app/Versions/A/library",
            temp.path().join("unscoped-link"),
        )
        .unwrap();
        assert!(inventory(temp.path(), &scope).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn cache_hash_rejects_fifo_before_open() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let fifo = temp.path().join("fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo)
                .status()
                .unwrap()
                .success()
        );
        assert!(file_identity(&fifo, 1024).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn literal_backslash_cannot_alias_an_installed_path() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        fs::create_dir(temp.path().join("dir")).unwrap();
        fs::write(temp.path().join("dir/file"), b"data").unwrap();
        inventory(temp.path(), &[]).unwrap();
        fs::rename(temp.path().join("dir/file"), temp.path().join("dir\\file")).unwrap();
        assert!(inventory(temp.path(), &[]).is_err());
    }
}
