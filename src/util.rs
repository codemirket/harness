use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Component, Path},
};

/// Owns an advisory lock for one operation. It is deliberately not cloneable.
pub struct FileLock {
    _file: fs::File,
}

impl FileLock {
    pub fn acquire(path: &Path) -> Result<Self> {
        reject_symlink_components(path)?;
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        file.try_lock()?;
        Ok(Self { _file: file })
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Closing this descriptor alone can leave the lock held by a descriptor
        // inherited during a concurrent process launch. Release ownership now.
        let _ = self._file.unlock();
    }
}

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn safe_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("path must contain only normal relative components");
    }
    Ok(())
}

pub fn reject_symlink_components(path: &Path) -> Result<()> {
    let mut current = std::path::PathBuf::new();
    for part in path.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                bail!("refusing symlink: {}", current.display())
            }
            Ok(_) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
    }
    Ok(())
}

pub fn ensure_directory(path: &Path) -> Result<()> {
    reject_symlink_components(path)?;
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))?;
    Ok(())
}

pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    reject_symlink_components(path)?;
    let parent = path.parent().context("file must have a parent")?;
    ensure_directory(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    if let Ok(metadata) = fs::metadata(path) {
        file.as_file().set_permissions(metadata.permissions())?;
    }
    file.write_all(data)?;
    file.as_file().sync_all()?;
    file.persist(path)
        .with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

pub fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    reject_symlink_components(path)?;
    let before = fs::symlink_metadata(path)?;
    if !before.is_file() || before.len() > max_bytes {
        bail!("file is not regular or exceeds {max_bytes} bytes");
    }
    let file = fs::File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > max_bytes {
        bail!("file is not regular or exceeds {max_bytes} bytes");
    }
    let mut bytes = Vec::new();
    file.take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        bail!("file exceeds {max_bytes} bytes");
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn dropping_lock_owner_releases_while_an_inherited_descriptor_remains() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary
            .path()
            .canonicalize()
            .unwrap()
            .join("operation.lock");
        let owner = FileLock::acquire(&path).unwrap();
        // A duplicate keeps the same open file description alive, as does the
        // descriptor inherited during another thread's fork-before-exec window.
        let inherited = owner._file.try_clone().unwrap();
        assert!(FileLock::acquire(&path).is_err());
        drop(owner);
        let next = FileLock::acquire(&path).expect(
            "completed operation must release the lock before an inherited descriptor closes",
        );
        assert!(FileLock::acquire(&path).is_err());
        drop(inherited);
        assert!(FileLock::acquire(&path).is_err());
        drop(next);
        assert!(FileLock::acquire(&path).is_ok());
    }

    #[test]
    fn reads_reject_special_files_and_excess_bytes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        assert!(read_bounded(&root, 1024).is_err());
        let file = root.join("large");
        fs::write(&file, b"abcd").unwrap();
        assert!(read_bounded(&file, 3).is_err());
        #[cfg(unix)]
        {
            let fifo = root.join("fifo");
            assert!(
                std::process::Command::new("mkfifo")
                    .arg(&fifo)
                    .status()
                    .unwrap()
                    .success()
            );
            assert!(read_bounded(&fifo, 1024).is_err());
        }
    }
}
