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

/// Keep exports separate from every editable input. Canonicalization alone does
/// not resolve case or normalization aliases on every filesystem. Callers still
/// apply their extension/profile checks through `read` and `Destination`.
pub fn distinct_output<'a>(
    root: &Path,
    name: &str,
    inputs: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    util::safe_relative(Path::new(name))?;
    let output = root.join(name);
    util::reject_symlink_components(&output)?;
    let target = match fs::metadata(&output) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    #[cfg(windows)]
    let target_identity = target
        .as_ref()
        .map(|_| windows_identity::read(&output))
        .transpose()?;
    for input in inputs {
        if name.eq_ignore_ascii_case(input) {
            bail!("export must be distinct from every editable input");
        }
        util::safe_relative(Path::new(input))?;
        let source = root.join(input);
        util::reject_symlink_components(&source)?;
        let source_metadata = fs::metadata(&source)?;
        if target.is_some() && output.canonicalize()? == source.canonicalize()? {
            bail!("export resolves to an editable input");
        }
        #[cfg(unix)]
        if let Some(target) = &target {
            use std::os::unix::fs::MetadataExt;
            if source_metadata.dev() == target.dev() && source_metadata.ino() == target.ino() {
                bail!("export aliases an editable input file");
            }
        }
        #[cfg(windows)]
        if let Some((_, target)) = &target_identity {
            let (_source_handle, identity) = windows_identity::read(&source)?;
            if *target == identity {
                bail!("export aliases an editable input file");
            }
        }
        #[cfg(not(unix))]
        let _ = source_metadata;
    }
    Ok(())
}

#[cfg(windows)]
mod windows_identity {
    use super::*;
    use std::{ffi::c_void, os::windows::io::AsRawHandle};

    // FILE_ID_INFO uses the volume and full 128-bit ID, including on ReFS.
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info
    #[repr(C)]
    #[derive(PartialEq, Eq)]
    pub(super) struct Identity {
        volume: u64,
        id: [u8; 16],
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        #[link_name = "GetFileInformationByHandleEx"]
        fn file_information(handle: *mut c_void, class: i32, buffer: *mut c_void, size: u32)
        -> i32;
    }

    pub(super) fn read(path: &Path) -> Result<(fs::File, Identity)> {
        let file = fs::File::open(path).context("open artifact for Windows file identity")?;
        let mut identity = Identity {
            volume: 0,
            id: [0; 16],
        };
        // SAFETY: the read-only File keeps a valid handle alive; repr(C)
        // Identity is the documented FILE_ID_INFO layout and the exact buffer
        // size is passed. Class 0x12 selects only that fixed structure.
        // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfileinformationbyhandleex
        let result = unsafe {
            file_information(
                file.as_raw_handle(),
                0x12,
                (&mut identity as *mut Identity).cast(),
                std::mem::size_of::<Identity>() as u32,
            )
        };
        if result == 0 {
            return Err(std::io::Error::last_os_error())
                .context("filesystem could not establish Windows artifact identity");
        }
        if identity.id == [0; 16] {
            bail!("filesystem did not supply a usable Windows artifact identity");
        }
        // Keep both compared handles alive so IDs cannot be reused mid-check.
        Ok((file, identity))
    }
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

    #[test]
    fn export_alias_guard_preserves_source_and_allows_distinct_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let original = b"editable source must survive";
        fs::write(root.join("source.svg"), original).unwrap();
        fs::write(root.join("other.svg"), b"another input").unwrap();
        for name in ["source.svg", "Source.svg", "SOURCE.SVG"] {
            assert!(
                distinct_output(&root, name, ["source.svg"]).is_err(),
                "{name}"
            );
        }
        #[cfg(any(unix, windows))]
        {
            fs::hard_link(root.join("source.svg"), root.join("alias.svg")).unwrap();
            assert!(distinct_output(&root, "alias.svg", ["source.svg"]).is_err());
        }
        assert!(distinct_output(&root, "other.svg", ["source.svg", "other.svg"]).is_err());
        assert!(distinct_output(&root, "../source.svg", ["source.svg"]).is_err());
        assert!(distinct_output(&root, "out.svg", ["../source.svg"]).is_err());
        distinct_output(&root, "out.svg", ["source.svg", "other.svg"]).unwrap();
        Destination::new(&root, "out.svg", &["svg"], false, None)
            .unwrap()
            .publish(b"exported output", "image/svg+xml")
            .unwrap();
        distinct_output(&root, "out.svg", ["source.svg"]).unwrap();
        assert_eq!(fs::read(root.join("source.svg")).unwrap(), original);
        #[cfg(any(unix, windows))]
        assert_eq!(fs::read(root.join("alias.svg")).unwrap(), original);
    }

    #[cfg(unix)]
    #[test]
    fn export_normalization_and_symlink_aliases_are_refused() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let composed = "caf\u{e9}.svg";
        let decomposed = "cafe\u{301}.svg";
        fs::write(root.join(composed), b"editable").unwrap();
        // Normalization equivalence depends on the filesystem. A hard link also
        // exercises the identity guard on filesystems with distinct spellings.
        if !root.join(decomposed).exists() {
            fs::hard_link(root.join(composed), root.join(decomposed)).unwrap();
        }
        assert!(distinct_output(&root, decomposed, [composed]).is_err());
        std::os::unix::fs::symlink(composed, root.join("link.svg")).unwrap();
        assert!(distinct_output(&root, "link.svg", [composed]).is_err());
        assert!(distinct_output(&root, "out.svg", ["link.svg"]).is_err());
        assert_eq!(fs::read(root.join(composed)).unwrap(), b"editable");
    }
}
