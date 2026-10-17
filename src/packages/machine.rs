//! Digest-verified boot artifact for the existing private Podman controller.
use crate::{paths::Paths, util};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

const SHA: &str = "633971788fcfe7df04f49da53f06c98b28933c5df2f684d57c2f9831411cac77";
const BYTES: u64 = 937_980_979;

pub(super) fn verify(path: &Path) -> Result<bool> {
    util::reject_symlink_components(path)?;
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() != BYTES {
        return Ok(false);
    }
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 128 * 1024];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        hasher.update(&buffer[..length]);
    }
    Ok(format!("{:x}", hasher.finalize()) == SHA)
}

pub(super) fn download(paths: &Paths) -> Result<PathBuf> {
    let cache = paths.root.join("cache/podman");
    util::ensure_directory(&cache)?;
    let _lock = util::FileLock::acquire(&cache.join("download.lock"))?;
    let destination = cache.join(format!("{SHA}.raw.zst"));
    if destination.exists() {
        ensure!(
            verify(&destination)?,
            "cached Podman image differs from its source pin"
        );
        return Ok(destination);
    }
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .timeout(Duration::from_secs(600))
        .build()?;
    let mut response = client
        .get(format!(
            "https://quay.io/v2/podman/machine-os/blobs/sha256:{SHA}"
        ))
        .send()?
        .error_for_status()?;
    if let Some(length) = response.content_length() {
        ensure!(
            length == BYTES,
            "Podman image response size differs from its pin"
        );
    }
    let mut temporary = tempfile::NamedTempFile::new_in(&cache)?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; 128 * 1024];
    loop {
        let length = response.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        size += length as u64;
        ensure!(size <= BYTES, "Podman image exceeds its pinned size");
        hasher.update(&buffer[..length]);
        temporary.write_all(&buffer[..length])?;
    }
    ensure!(
        size == BYTES && format!("{:x}", hasher.finalize()) == SHA,
        "downloaded Podman image differs from its source pin"
    );
    temporary.flush()?;
    temporary.as_file().sync_all()?;
    temporary
        .persist_noclobber(&destination)
        .context("publish verified Podman boot image")?;
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boot_pin_rejects_truncated_and_non_regular_artifacts() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().canonicalize().unwrap();
        let file = directory.join("boot.raw.zst");
        std::fs::write(&file, b"truncated").unwrap();
        assert!(!verify(&file).unwrap());
        assert!(!verify(&directory).unwrap());
        #[cfg(unix)]
        {
            let link = directory.join("link");
            std::os::unix::fs::symlink(&file, &link).unwrap();
            assert!(verify(&link).is_err());
        }
    }
}
