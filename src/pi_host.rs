//! Read-only verification of the exact external pi host closure.
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    path::{Component, Path},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Inventory {
    schema: u32,
    host_version: String,
    entry: String,
    lock_sha256: String,
    files: Vec<Entry>,
}
#[derive(Deserialize)]
struct Entry {
    path: String,
    bytes: u64,
    sha256: String,
}
fn open_regular(path: &Path, limit: u64) -> Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err("invalid host file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.nlink() != 1 {
            return Err("aliased host file".into());
        }
    }
    Ok(file)
}
fn digest(path: &Path, limit: u64) -> Result<(u64, String)> {
    let mut file = open_regular(path, limit)?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    let mut bytes = 0;
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        if bytes > limit {
            return Err("host file grew beyond bound".into());
        }
        hash.update(&buffer[..count]);
    }
    Ok((
        bytes,
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    ))
}
fn verify_files(root: &Path, files: &[Entry]) -> Result<()> {
    if !root.is_absolute() || root.canonicalize()? != root {
        return Err("host root must be canonical".into());
    }
    let mut expected = BTreeMap::new();
    for entry in files {
        let relative = Path::new(&entry.path);
        if entry.path.is_empty()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || expected.insert(entry.path.as_str(), entry).is_some()
        {
            return Err("invalid host inventory path".into());
        }
    }
    let mut pending = vec![root.to_path_buf()];
    let mut observed = Vec::new();
    while let Some(directory) = pending.pop() {
        for item in fs::read_dir(directory)? {
            let item = item?;
            let path = item.path();
            let kind = item.file_type()?;
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            let name = path
                .strip_prefix(root)?
                .to_str()
                .ok_or("host path is not UTF-8")?;
            if kind.is_symlink()
                && path
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|name| name == ".bin")
            {
                let target = path.canonicalize()?;
                if target
                    .strip_prefix(root)
                    .ok()
                    .and_then(Path::to_str)
                    .is_some_and(|name| expected.contains_key(name))
                {
                    continue;
                }
            }
            if kind.is_file() && name == ".package-lock.json" {
                continue;
            }
            if !kind.is_file() {
                return Err(format!("unexpected external host file: {name}").into());
            }
            let entry = expected
                .get(name)
                .ok_or_else(|| format!("unexpected external host file: {name}"))?;
            observed.push((path, *entry));
        }
    }
    if observed.len() != expected.len() {
        return Err("external host files are missing".into());
    }
    // Bounded parallel reads retain full byte verification without serializing
    // thousands of small filesystem operations in the pi process.
    std::thread::scope(|scope| -> Result<()> {
        let size = observed.len().div_ceil(8).max(1);
        let workers: Vec<_> = observed
            .chunks(size)
            .map(|chunk| {
                scope.spawn(move || -> std::result::Result<(), String> {
                    for (path, entry) in chunk {
                        let (bytes, hash) =
                            digest(path, 256 * 1024 * 1024).map_err(|error| error.to_string())?;
                        if bytes != entry.bytes || hash != entry.sha256 {
                            return Err(format!("external pi host differs: {}", entry.path));
                        }
                    }
                    Ok(())
                })
            })
            .collect();
        let results: Vec<_> = workers.into_iter().map(|worker| worker.join()).collect();
        for result in results {
            result
                .map_err(|_| "host verification worker failed")?
                .map_err(std::io::Error::other)?;
        }
        Ok(())
    })
}
pub fn verify(root: &Path, inventory: &Path, expected_sha256: &str) -> Result<serde_json::Value> {
    let mut bytes = Vec::new();
    open_regular(inventory, 8 * 1024 * 1024)?
        .take(8 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    let hash: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if bytes.len() > 8 * 1024 * 1024 || hash != expected_sha256 {
        return Err("host inventory differs from binding".into());
    }
    let inventory: Inventory = serde_json::from_slice(&bytes)?;
    if inventory.schema != 1
        || inventory.host_version != "1.4.0"
        || inventory.entry != "@earendil-works/pi-coding-agent/dist/cli.js"
        || inventory.files.len() != 15249
    {
        return Err("unsupported host closure".into());
    }
    verify_files(root, &inventory.files)?;
    Ok(
        serde_json::json!({"root":root,"entry":root.join(inventory.entry),"lockSha256":inventory.lock_sha256,"verifiedFiles":inventory.files.len()}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_missing_extra_and_linked_host_files_are_refused() {
        let root = std::env::temp_dir().join(format!(
            "mirket-host-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let path = root.join("provider.js");
        fs::write(&path, b"qualified").unwrap();
        let (bytes, sha256) = digest(&path, 1024).unwrap();
        let files = vec![Entry {
            path: "provider.js".into(),
            bytes,
            sha256,
        }];
        verify_files(&root, &files).unwrap();
        fs::write(&path, b"changed").unwrap();
        assert!(verify_files(&root, &files).is_err());
        fs::write(&path, b"qualified").unwrap();
        fs::write(root.join("extra.js"), b"code").unwrap();
        assert!(verify_files(&root, &files).is_err());
        fs::remove_file(root.join("extra.js")).unwrap();
        fs::remove_file(&path).unwrap();
        assert!(verify_files(&root, &files).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("missing", &path).unwrap();
            assert!(verify_files(&root, &files).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }
}
