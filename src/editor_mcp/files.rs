//! Project-local, staged editor artifacts. No caller supplies an absolute path.
use crate::util;
use anyhow::{Context, Result, bail};
use rmcp::schemars;
use serde::Serialize;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub const MAX_SVG_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_OUTPUT_BYTES: u64 = 32 * 1024 * 1024;

pub fn scoped(root: &Path, name: &str, extensions: &[&str]) -> Result<PathBuf> {
    if name.is_empty()
        || name.len() > 1024
        || name.contains(['\\', ':', '\0'])
        || name.chars().any(char::is_control)
        || name
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == ".." || p.starts_with('.'))
    {
        bail!(
            "use a project-relative artifact path with normal, visible components; absolute paths, hidden directories and traversal are refused"
        );
    }
    let relative = Path::new(name);
    util::safe_relative(relative)?;
    if !extensions
        .iter()
        .any(|extension| relative.extension().is_some_and(|e| e == *extension))
    {
        bail!(
            "artifact path must end in one of: {}",
            extensions.join(", ")
        );
    }
    let path = root.join(relative);
    util::reject_symlink_components(&path)?;
    let parent = path.parent().context("artifact has no parent")?;
    if !parent.is_dir() || !parent.canonicalize()?.starts_with(root) {
        bail!("artifact parent must be an existing directory inside the selected project");
    }
    Ok(path)
}

pub fn read(root: &Path, name: &str, extensions: &[&str], limit: u64) -> Result<Vec<u8>> {
    util::read_bounded(&scoped(root, name, extensions)?, limit)
        .with_context(|| format!("read project artifact {name}"))
}

pub struct Destination {
    root: PathBuf,
    name: String,
    path: PathBuf,
    before: Option<Vec<u8>>,
}

impl Destination {
    pub fn new(
        root: &Path,
        name: &str,
        extensions: &[&str],
        overwrite: bool,
        expected_sha256: Option<&str>,
    ) -> Result<Self> {
        let path = scoped(root, name, extensions)?;
        let before = match fs::symlink_metadata(&path) {
            Ok(_) => Some(util::read_bounded(&path, MAX_OUTPUT_BYTES)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if before.is_some() && !overwrite {
            bail!("output {name} exists; choose another path or explicitly set overwrite=true");
        }
        if let Some(expected) = expected_sha256
            && before
                .as_ref()
                .is_none_or(|bytes| util::sha256(bytes) != expected)
        {
            bail!("output differs from expected_sha256; inspect it before overwriting");
        }
        Ok(Self {
            root: root.into(),
            name: name.into(),
            path,
            before,
        })
    }

    pub fn publish(self, bytes: &[u8], media_type: &str) -> Result<Artifact> {
        if bytes.is_empty() || bytes.len() as u64 > MAX_OUTPUT_BYTES {
            bail!("editor produced an empty or oversized artifact");
        }
        // Recheck after the editor has finished. No-clobber publication is atomic;
        // replacing an explicitly selected existing file also checks its snapshot.
        scoped(
            &self.root,
            &self.name,
            &[self.path.extension().unwrap().to_str().unwrap()],
        )?;
        let current = match fs::symlink_metadata(&self.path) {
            Ok(_) => Some(util::read_bounded(&self.path, MAX_OUTPUT_BYTES)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        if current != self.before {
            bail!("output changed during the operation; no artifact was published");
        }
        let mut temporary = tempfile::NamedTempFile::new_in(self.path.parent().unwrap())?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;
        if self.before.is_some() {
            temporary
                .persist(&self.path)
                .context("replace selected artifact")?;
        } else {
            temporary
                .persist_noclobber(&self.path)
                .context("publish artifact without overwriting")?;
        }
        Ok(Artifact {
            path: self.name,
            media_type: media_type.into(),
            bytes: bytes.len(),
            sha256: util::sha256(bytes),
        })
    }
}

#[derive(Debug, Serialize, rmcp::schemars::JsonSchema)]
pub struct Artifact {
    /// Path relative to the selected project. Read or open this saved file to inspect it.
    pub path: String,
    pub media_type: String,
    pub bytes: usize,
    pub sha256: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_overwrite_and_late_changes_are_refused() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        for path in [
            "../out.svg",
            "/tmp/out.svg",
            "x/../out.svg",
            ".git/config.svg",
            "x\\out.svg",
            "x//out.svg",
            "out.svg/",
            "out.png",
            "missing/out.svg",
        ] {
            assert!(scoped(&root, path, &["svg"]).is_err(), "{path}");
        }
        let dest = Destination::new(&root, "art.svg", &["svg"], false, None).unwrap();
        fs::write(root.join("art.svg"), b"concurrent content").unwrap();
        assert!(dest.publish(b"new art", "image/svg+xml").is_err());
        assert!(Destination::new(&root, "art.svg", &["svg"], false, None).is_err());
        assert!(Destination::new(&root, "art.svg", &["svg"], true, Some("wrong")).is_err());
        let expected = util::sha256(b"concurrent content");
        Destination::new(&root, "art.svg", &["svg"], true, Some(&expected))
            .unwrap()
            .publish(b"revised art", "image/svg+xml")
            .unwrap();
        assert_eq!(fs::read(root.join("art.svg")).unwrap(), b"revised art");
    }
    #[cfg(unix)]
    #[test]
    fn symlink_files_and_parents_are_refused() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::write(root.join("real.svg"), b"art").unwrap();
        std::os::unix::fs::symlink("real.svg", root.join("link.svg")).unwrap();
        std::os::unix::fs::symlink(&root, root.join("alias")).unwrap();
        for name in ["link.svg", "alias/out.svg"] {
            assert!(scoped(&root, name, &["svg"]).is_err());
        }
    }
}
