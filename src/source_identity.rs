//! Shared build/runtime fingerprint contract. No Git state or user files.
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
pub const INPUTS: &[&str] = &[
    "src",
    "setup",
    "skills",
    "registry",
    "instructions",
    "docs/third-party-notices.md",
    "docs/licenses",
    "runtime/design",
    "runtime/agent",
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "rust-toolchain.toml",
    "tests",
];
fn files(root: &Path, path: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.file_name().is_some_and(|n| n == ".DS_Store") {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::other(
            "source identity does not follow symbolic links",
        ));
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            files(root, &entry?.path(), out)?;
        }
    } else {
        out.push(
            path.strip_prefix(root)
                .map_err(io::Error::other)?
                .to_owned(),
        );
    }
    Ok(())
}
pub fn fingerprint(root: &Path) -> io::Result<String> {
    let mut inputs = Vec::new();
    for name in INPUTS {
        files(root, &root.join(name), &mut inputs)?;
    }
    inputs.sort();
    let mut hash = Sha256::new();
    for path in inputs {
        let name = path.to_string_lossy().replace('\\', "/");
        let bytes = fs::read(root.join(path))?;
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(format!("{:x}", hash.finalize()))
}
