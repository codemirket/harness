use super::Artifact;
use crate::util;
use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub(super) const MAX_FILES: usize = 40_000;
pub(super) const RECEIPT: &str = ".mirket-package.json";

/// A portable package path, including when validated on a different host OS.
pub(super) fn normal_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 2048
        || path.contains(['\\', ':', '\0'])
        || path.split('/').count() > 64
    {
        bail!("invalid package path");
    }
    for part in path.split('/') {
        let base = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.ends_with([' ', '.'])
            || part.len() > 255
            || part.chars().any(|c| c.is_control())
            || matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (base.len() == 4
                && (base.starts_with("COM") || base.starts_with("LPT"))
                && matches!(base.as_bytes()[3], b'1'..=b'9'))
        {
            bail!("nonportable package path: {path}");
        }
    }
    Ok(())
}

pub(super) fn validate_artifact(a: &Artifact) -> Result<()> {
    let url = reqwest::Url::parse(&a.url)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || a.sha256.len() != 64
        || !a.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        || !matches!(a.format.as_str(), "tar-gz" | "zip" | "file" | "dmg-app")
        || a.max_download_bytes == 0
        || a.max_download_bytes > 512 * 1024 * 1024
        || a.max_unpacked_bytes == 0
        || a.max_unpacked_bytes > 2 * 1024 * 1024 * 1024
    {
        bail!("invalid pinned package artifact");
    }
    if !a.destination.is_empty() {
        normal_path(&a.destination)?;
    }
    if let Some(prefix) = &a.strip_prefix {
        normal_path(prefix)?;
    }
    if a.format == "file" && a.files.len() != 1 {
        bail!("raw artifact must have exactly one output mapping");
    }
    if a.format == "dmg-app" {
        let app = a
            .strip_prefix
            .as_deref()
            .context("DMG requires an application name")?;
        if app.contains('/')
            || !app.ends_with(".app")
            || a.destination.is_empty()
            || !a.destination.ends_with(".app")
            || !a.files.is_empty()
        {
            bail!(
                "DMG artifacts require one top-level .app and an explicit application destination"
            );
        }
    }
    let mut sources = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    for member in &a.files {
        normal_path(&member.path)?;
        normal_path(&member.destination)?;
        if !sources.insert(&member.path)
            || !destinations.insert(member.destination.to_lowercase())
            || member.destination == RECEIPT
        {
            bail!("duplicate or reserved artifact output");
        }
    }
    Ok(())
}

struct Extractor<'a> {
    root: &'a Path,
    artifact: &'a Artifact,
    found: BTreeSet<String>,
    outputs: BTreeSet<String>,
    bytes: u64,
    entries: usize,
}
impl<'a> Extractor<'a> {
    fn member(
        &mut self,
        path: &str,
        regular: bool,
        directory: bool,
        size: u64,
        executable: bool,
        reader: &mut impl Read,
    ) -> Result<()> {
        self.entries += 1;
        if self.entries > MAX_FILES {
            bail!("archive exceeds entry limit");
        }
        let path = path
            .strip_prefix("./")
            .unwrap_or(path)
            .trim_end_matches('/');
        if directory && path.is_empty() {
            return Ok(());
        }
        normal_path(path)?;
        self.bytes = self
            .bytes
            .checked_add(size)
            .context("archive size overflow")?;
        if self.bytes > self.artifact.max_unpacked_bytes {
            bail!("archive exceeds unpacked byte limit");
        }
        let selected = if !self.artifact.files.is_empty() {
            self.artifact
                .files
                .iter()
                .find(|m| m.path == path)
                .map(|m| (m.destination.clone(), m.executable))
        } else {
            let relative = match &self.artifact.strip_prefix {
                Some(prefix) if path == prefix => return Ok(()),
                Some(prefix) => match path.strip_prefix(&format!("{prefix}/")) {
                    Some(p) => p,
                    None => return Ok(()),
                },
                None => path,
            };
            let dest = if self.artifact.destination.is_empty() {
                relative.to_string()
            } else {
                format!("{}/{relative}", self.artifact.destination)
            };
            Some((dest, executable))
        };
        let Some((relative, executable)) = selected else {
            return Ok(());
        };
        if directory {
            return Ok(());
        }
        if !regular {
            bail!("selected archive member is a link or special file: {path}");
        }
        normal_path(&relative)?;
        if relative == RECEIPT || !self.outputs.insert(relative.to_lowercase()) {
            bail!("duplicate or reserved archive output: {relative}");
        }
        let output = self.root.join(&relative);
        util::ensure_directory(output.parent().context("archive output parent")?)?;
        util::reject_symlink_components(&output)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&output)
            .with_context(|| format!("create package member {relative}"))?;
        let count = io::copy(&mut reader.take(size + 1), &mut file)?;
        if count != size {
            bail!("archive member size differs from header");
        }
        file.flush()?;
        set_mode(&output, executable)?;
        self.found.insert(path.into());
        Ok(())
    }
}

pub(super) fn extract(artifact: &Artifact, input: &Path, root: &Path) -> Result<()> {
    validate_artifact(artifact)?;
    util::ensure_directory(root)?;
    if artifact.format == "dmg-app" {
        return super::dmg::extract(artifact, input, root);
    }
    let mut extraction = Extractor {
        root,
        artifact,
        found: BTreeSet::new(),
        outputs: BTreeSet::new(),
        bytes: 0,
        entries: 0,
    };
    match artifact.format.as_str() {
        "tar-gz" => {
            // Count the entire decoded stream, including skipped members and headers.
            let decoder = flate2::read::GzDecoder::new(fs::File::open(input)?);
            let limit = artifact.max_unpacked_bytes + 16 * 1024 * 1024;
            let mut bounded = decoder.take(limit + 1);
            {
                let mut archive = tar::Archive::new(&mut bounded);
                for entry in archive.entries()? {
                    let mut entry = entry?;
                    let path = entry
                        .path()?
                        .to_str()
                        .context("non-UTF8 archive member")?
                        .to_string();
                    let kind = entry.header().entry_type();
                    let size = entry.size();
                    let executable = entry.header().mode()? & 0o111 != 0;
                    extraction.member(
                        &path,
                        kind.is_file(),
                        kind.is_dir(),
                        size,
                        executable,
                        &mut entry,
                    )?;
                }
            }
            io::copy(&mut bounded, &mut io::sink())?;
            if bounded.limit() == 0 {
                bail!("archive exceeds decoded stream limit");
            }
        }
        "zip" => {
            let mut zip = zip::ZipArchive::new(fs::File::open(input)?)?;
            if zip.len() > MAX_FILES {
                bail!("archive exceeds entry limit");
            }
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i)?;
                let path = entry.name().to_string();
                let mode = entry.unix_mode().unwrap_or(0o100644);
                let dir = entry.is_dir();
                let regular = entry.is_file() && matches!(mode & 0o170000, 0 | 0o100000);
                let size = entry.size();
                extraction.member(&path, regular, dir, size, mode & 0o111 != 0, &mut entry)?;
            }
        }
        "file" => {
            let mut file = fs::File::open(input)?;
            let mapping = &artifact.files[0];
            extraction.member(
                &mapping.path,
                true,
                false,
                file.metadata()?.len(),
                mapping.executable,
                &mut file,
            )?;
        }
        _ => bail!("unsupported package archive"),
    }
    for member in &artifact.files {
        if !extraction.found.contains(&member.path) {
            bail!("archive is missing selected member {}", member.path);
        }
    }
    if extraction.outputs.is_empty() {
        bail!("archive produced no files");
    }
    Ok(())
}

pub(super) fn set_mode(path: &Path, executable: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            path,
            fs::Permissions::from_mode(if executable { 0o755 } else { 0o644 }),
        )?;
    }
    #[cfg(not(unix))]
    {
        let _ = (path, executable);
    }
    Ok(())
}

pub(super) fn files(root: &Path, application_roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    fn visit(
        base: &Path,
        path: &Path,
        out: &mut Vec<PathBuf>,
        names: &mut BTreeSet<String>,
        application_roots: &[PathBuf],
    ) -> Result<()> {
        util::reject_symlink_components(path)?;
        for item in fs::read_dir(path)? {
            let item = item?;
            let file = item.path();
            let relative = relative_name(base, &file)?;
            normal_path(&relative)?;
            if !names.insert(relative.to_lowercase()) || names.len() > MAX_FILES {
                bail!("duplicate paths or excessive package entries");
            }
            if item.file_type()?.is_symlink() {
                let app = application_roots
                    .iter()
                    .find(|app| file.starts_with(app) && file != **app)
                    .context(
                        "package content includes a symlink outside a reviewed DMG application",
                    )?;
                super::dmg::link_target(app, &file)?;
                out.push(file);
            } else if item.file_type()?.is_dir() {
                visit(base, &file, out, names, application_roots)?;
            } else if item.file_type()?.is_file() {
                if relative != RECEIPT {
                    out.push(file);
                }
            } else {
                bail!("package content includes special file");
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    visit(
        root,
        root,
        &mut out,
        &mut BTreeSet::new(),
        application_roots,
    )?;
    out.sort();
    Ok(out)
}

pub(super) fn relative_name(root: &Path, file: &Path) -> Result<String> {
    let relative = file
        .strip_prefix(root)?
        .to_str()
        .context("non-UTF8 package path")?;
    #[cfg(windows)]
    let relative = relative.replace('\\', "/");
    normal_path(relative)?;
    Ok(relative.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::{ZipWriter, write::SimpleFileOptions};
    fn artifact() -> Artifact {
        Artifact {
            url: "https://example.org/package.zip".into(),
            sha256: "a".repeat(64),
            format: "zip".into(),
            max_download_bytes: 1024,
            max_unpacked_bytes: 1024,
            strip_prefix: None,
            destination: String::new(),
            files: vec![],
        }
    }
    #[test]
    fn rejects_portable_traversal_and_device_paths() {
        for name in [
            "../x", "/x", "C:/x", "a\\b", "a//b", "a/./b", "CON.txt", "x/NUL", "LPT9", "x. ",
            "foo/..",
        ] {
            assert!(normal_path(name).is_err(), "{name}");
        }
        normal_path("node_modules/@scope/name/file.js").unwrap();
    }
    #[test]
    fn dmg_requires_one_explicit_application_without_member_mappings() {
        let mut a = artifact();
        a.format = "dmg-app".into();
        assert!(validate_artifact(&a).is_err());
        a.strip_prefix = Some("Fixture.app".into());
        a.destination = "Fixture.app".into();
        validate_artifact(&a).unwrap();
        a.strip_prefix = Some("nested/Fixture.app".into());
        assert!(validate_artifact(&a).is_err());
        a.strip_prefix = Some("Fixture.app".into());
        a.files.push(super::super::Member {
            path: "one".into(),
            destination: "one".into(),
            executable: false,
        });
        assert!(validate_artifact(&a).is_err());
    }
    #[test]
    fn rejects_zip_traversal_duplicates_links_and_bombs() {
        for (name, mode, cap) in [
            ("../outside", 0o644, 1024),
            ("link", 0o120777, 1024),
            ("big", 0o644, 1),
        ] {
            let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
            let input = temp.path().join("input.zip");
            let mut zip = ZipWriter::new(fs::File::create(&input).unwrap());
            if mode == 0o120777 {
                zip.add_symlink(name, "/etc/passwd", SimpleFileOptions::default())
                    .unwrap();
            } else {
                zip.start_file(name, SimpleFileOptions::default().unix_permissions(mode))
                    .unwrap();
                zip.write_all(b"data").unwrap();
            }
            zip.finish().unwrap();
            let mut a = artifact();
            a.max_unpacked_bytes = cap;
            assert!(extract(&a, &input, &temp.path().join("out")).is_err());
            assert!(!temp.path().join("outside").exists());
        }
    }
    #[test]
    fn selected_member_must_exist_and_cannot_overwrite() {
        let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let input = temp.path().join("input.zip");
        let mut zip = ZipWriter::new(fs::File::create(&input).unwrap());
        zip.start_file("one", SimpleFileOptions::default()).unwrap();
        zip.write_all(b"data").unwrap();
        zip.finish().unwrap();
        let a = artifact();
        let out = temp.path().join("out");
        extract(&a, &input, &out).unwrap();
        assert!(extract(&a, &input, &out).is_err());
        let mut missing = a;
        missing.files.push(super::super::Member {
            path: "absent".into(),
            destination: "bin/tool".into(),
            executable: true,
        });
        assert!(extract(&missing, &input, &temp.path().join("other")).is_err());
    }

    fn tar_bytes(entries: &[(&str, tar::EntryType, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (name, kind, data) in entries {
            let mut header = tar::Header::new_gnu();
            // Write a raw header so the traversal fixtures reach the extractor;
            // the builder's path setter would reject them before the test.
            assert!(name.len() < 100);
            header.as_mut_bytes()[..name.len()].copy_from_slice(name.as_bytes());
            header.set_entry_type(*kind);
            header.set_mode(0o6777);
            header.set_size(data.len() as u64);
            if kind.is_symlink() || kind.is_hard_link() {
                header.set_link_name("../outside").unwrap();
            }
            header.set_cksum();
            builder.append(&header, *data).unwrap();
        }
        builder.finish().unwrap();
        builder.into_inner().unwrap()
    }

    fn write_gzip(path: &Path, decoded: &[u8]) {
        let mut encoder = flate2::write::GzEncoder::new(
            fs::File::create(path).unwrap(),
            flate2::Compression::fast(),
        );
        encoder.write_all(decoded).unwrap();
        encoder.finish().unwrap();
    }

    fn tar_artifact() -> Artifact {
        Artifact {
            format: "tar-gz".into(),
            ..artifact()
        }
    }

    fn tar_test_directory() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("mirket-tar-test-")
            .tempdir_in(std::env::temp_dir().canonicalize().unwrap())
            .unwrap()
    }

    #[test]
    fn tar_selected_files_are_regular_and_do_not_preserve_special_modes() {
        let temp = tar_test_directory();
        let input = temp.path().join("input.tar.gz");
        write_gzip(
            &input,
            &tar_bytes(&[
                ("tool", tar::EntryType::Regular, b"tool"),
                ("README", tar::EntryType::Regular, b"unused"),
                ("unselected-link", tar::EntryType::Symlink, b""),
            ]),
        );
        let mut artifact = tar_artifact();
        artifact.files.push(super::super::Member {
            path: "tool".into(),
            destination: "bin/tool".into(),
            executable: true,
        });
        let out = temp.path().join("out");
        extract(&artifact, &input, &out).unwrap();
        assert_eq!(fs::read(out.join("bin/tool")).unwrap(), b"tool");
        assert!(!out.join("README").exists());
        assert!(!out.join("unselected-link").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(out.join("bin/tool"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o7777,
                0o755
            );
        }
    }

    #[test]
    fn tar_rejects_selected_symlinks_and_hardlinks() {
        for kind in [tar::EntryType::Symlink, tar::EntryType::Link] {
            let temp = tar_test_directory();
            let input = temp.path().join("input.tar.gz");
            write_gzip(&input, &tar_bytes(&[("tool", kind, b"")]));
            let mut artifact = tar_artifact();
            artifact.files.push(super::super::Member {
                path: "tool".into(),
                destination: "bin/tool".into(),
                executable: true,
            });
            let error = extract(&artifact, &input, &temp.path().join("out")).unwrap_err();
            assert!(
                error.to_string().contains("link or special file"),
                "{error}"
            );
            assert!(!temp.path().join("outside").exists());
        }
    }

    #[test]
    fn tar_rejects_traversal_and_case_colliding_members() {
        for names in [
            vec!["../outside"],
            vec!["tool", "tool"],
            vec!["Tool", "tool"],
        ] {
            let temp = tar_test_directory();
            let input = temp.path().join("input.tar.gz");
            let entries = names
                .iter()
                .map(|name| (*name, tar::EntryType::Regular, &b"data"[..]))
                .collect::<Vec<_>>();
            write_gzip(&input, &tar_bytes(&entries));
            assert!(extract(&tar_artifact(), &input, &temp.path().join("out")).is_err());
            assert!(!temp.path().join("outside").exists());
        }
    }

    #[test]
    fn tar_limits_include_unselected_members_and_decoded_trailing_data() {
        let temp = tar_test_directory();
        let input = temp.path().join("input.tar.gz");
        let mut artifact = tar_artifact();
        artifact.max_unpacked_bytes = 1;
        artifact.files.push(super::super::Member {
            path: "tool".into(),
            destination: "bin/tool".into(),
            executable: true,
        });
        write_gzip(
            &input,
            &tar_bytes(&[
                ("tool", tar::EntryType::Regular, b"t"),
                ("unselected", tar::EntryType::Regular, b"too large"),
            ]),
        );
        let error = extract(&artifact, &input, &temp.path().join("out")).unwrap_err();
        assert!(error.to_string().contains("unpacked byte limit"), "{error}");

        let mut decoded = tar_bytes(&[("tool", tar::EntryType::Regular, b"t")]);
        decoded.resize(decoded.len() + 16 * 1024 * 1024 + 1, 0);
        write_gzip(&input, &decoded);
        let error = extract(&artifact, &input, &temp.path().join("trailing")).unwrap_err();
        assert!(
            error.to_string().contains("decoded stream limit"),
            "{error}"
        );
    }
}
