//! Fixed, read-only macOS disk-image extraction into a prepared package generation.
//! Only one reviewed application is copied. Mounted contents are never executed.
use super::{Artifact, archive};
use crate::util;
use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
#[cfg(any(target_os = "macos", all(test, unix)))]
use std::io::Write;
use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub(super) fn extract(artifact: &Artifact, input: &Path, root: &Path) -> Result<()> {
    verify_image(artifact, input)?;
    #[cfg(target_os = "macos")]
    {
        platform::extract(artifact, input, root)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = root;
        bail!("DMG application extraction is available only on macOS");
    }
}

fn verify_image(artifact: &Artifact, input: &Path) -> Result<()> {
    util::reject_symlink_components(input)?;
    let metadata = fs::symlink_metadata(input)?;
    if !metadata.is_file() || metadata.len() < 512 || metadata.len() > artifact.max_download_bytes {
        bail!("DMG input must be a bounded regular UDIF image");
    }
    let mut file = fs::File::open(input)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut count = 0u64;
    loop {
        let size = file.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        count += size as u64;
        if count > artifact.max_download_bytes {
            bail!("DMG input exceeds its byte limit");
        }
        hasher.update(&buffer[..size]);
    }
    if count != metadata.len() || format!("{:x}", hasher.finalize()) != artifact.sha256 {
        bail!("DMG image SHA-256 does not match its curated pin");
    }
    file.seek(SeekFrom::End(-512))?;
    let mut signature = [0u8; 4];
    file.read_exact(&mut signature)?;
    if &signature != b"koly" {
        bail!("DMG image has no UDIF trailer");
    }
    Ok(())
}

fn link_parts(target: &Path) -> Result<VecDeque<String>> {
    let text = target.to_str().context("non-UTF8 application symlink")?;
    if text.is_empty() || text.len() > 2048 || target.is_absolute() || text.contains('\\') {
        bail!("application symlinks must have bounded relative targets");
    }
    let mut parts = VecDeque::new();
    for part in text.split('/') {
        if !matches!(part, "." | "..") {
            archive::normal_path(part)?;
        }
        parts.push_back(part.to_owned());
    }
    if parts.len() > 64 {
        bail!("application symlink target is too deep");
    }
    Ok(parts)
}

/// Preserve the original relative target only after resolving every link without
/// following an absolute/escaping target, including links encountered in parents.
pub(super) fn link_target(application: &Path, link: &Path) -> Result<String> {
    util::reject_symlink_components(application)?;
    if !fs::symlink_metadata(application)?.is_dir() {
        bail!("application root must be a directory");
    }
    let relative = archive::relative_name(application, link)?;
    if !fs::symlink_metadata(link)?.file_type().is_symlink() {
        bail!("application entry is not a symlink");
    }
    let target = fs::read_link(link)?;
    link_parts(&target)?;
    let mut pending: VecDeque<String> = relative.split('/').map(str::to_owned).collect();
    let mut resolved = PathBuf::new();
    let mut visited = BTreeSet::new();
    while let Some(part) = pending.pop_front() {
        match part.as_str() {
            "." => continue,
            ".." => {
                if !resolved.pop() {
                    bail!("application symlink escapes its application");
                }
                continue;
            }
            _ => {}
        }
        let candidate = application.join(&resolved).join(&part);
        let metadata =
            fs::symlink_metadata(&candidate).context("application symlink has a missing target")?;
        if metadata.file_type().is_symlink() {
            if !visited.insert(candidate.clone()) || visited.len() > 64 {
                bail!("application symlink cycle or chain limit exceeded");
            }
            let mut parts = link_parts(&fs::read_link(&candidate)?)?;
            parts.append(&mut pending);
            if parts.len() > 4096 {
                bail!("application symlink resolution is too deep");
            }
            pending = parts;
        } else if metadata.is_file() || metadata.is_dir() {
            if metadata.is_file() && !pending.is_empty() {
                bail!("application symlink traverses a regular file");
            }
            resolved.push(part);
        } else {
            bail!("application symlink targets a special file");
        }
    }
    let destination = application.join(resolved);
    if fs::symlink_metadata(&destination)?.is_dir() && link.starts_with(&destination) {
        bail!("application symlink points to an ancestor directory");
    }
    target
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow::anyhow!("non-UTF8 application symlink"))
}

#[cfg(any(target_os = "macos", all(test, unix)))]
fn copy_application(source: &Path, destination: &Path, maximum_bytes: u64) -> Result<()> {
    struct CopyState {
        source: PathBuf,
        names: BTreeSet<String>,
        bytes: u64,
        files: usize,
        maximum: u64,
    }
    impl CopyState {
        fn visit(&mut self, source: &Path, destination: &Path) -> Result<()> {
            for entry in fs::read_dir(source)? {
                let entry = entry?;
                let path = entry.path();
                let relative = archive::relative_name(&self.source, &path)?;
                if !self.names.insert(relative.to_lowercase())
                    || self.names.len() > archive::MAX_FILES
                {
                    bail!("application contains duplicate paths or too many entries");
                }
                let output = destination.join(entry.file_name());
                let metadata = fs::symlink_metadata(&path)?;
                if metadata.is_dir() {
                    fs::create_dir(&output)?;
                    self.visit(&path, &output)?;
                } else if metadata.file_type().is_symlink() {
                    let target = link_target(&self.source, &path)?;
                    self.add_bytes(target.len() as u64)?;
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(&target, &output)?;
                    #[cfg(not(unix))]
                    bail!("application symlink extraction requires a Unix target");
                } else if metadata.is_file() {
                    self.add_bytes(metadata.len())?;
                    let mut input = fs::File::open(&path)?;
                    let mut file = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&output)?;
                    let copied =
                        std::io::copy(&mut (&mut input).take(metadata.len() + 1), &mut file)?;
                    if copied != metadata.len() {
                        bail!("application file changed during extraction");
                    }
                    file.flush()?;
                    #[cfg(unix)]
                    let executable = {
                        use std::os::unix::fs::PermissionsExt;
                        metadata.permissions().mode() & 0o111 != 0
                    };
                    #[cfg(not(unix))]
                    let executable = false;
                    util::set_executable_mode(&output, executable)?;
                    self.files += 1;
                } else {
                    bail!("application contains a special file");
                }
            }
            Ok(())
        }
        fn add_bytes(&mut self, count: u64) -> Result<()> {
            self.bytes = self
                .bytes
                .checked_add(count)
                .context("application size overflow")?;
            if self.bytes > self.maximum {
                bail!("application exceeds unpacked byte limit");
            }
            Ok(())
        }
    }
    util::reject_symlink_components(source)?;
    if !fs::symlink_metadata(source)?.is_dir() {
        bail!("selected DMG application is not a directory");
    }
    util::reject_symlink_components(destination)?;
    util::ensure_directory(
        destination
            .parent()
            .context("application destination has no parent")?,
    )?;
    fs::create_dir(destination)
        .context("application destination already exists or cannot be created")?;
    let mut state = CopyState {
        source: source.to_owned(),
        names: BTreeSet::new(),
        bytes: 0,
        files: 0,
        maximum: maximum_bytes,
    };
    state.visit(source, destination)?;
    if state.files == 0 {
        bail!("selected DMG application contains no regular files");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::{
        ffi::OsString,
        process::{ExitStatus, Stdio},
        time::Duration,
    };
    use tokio::io::AsyncReadExt;

    const OUTPUT_LIMIT: usize = 2 * 1024 * 1024;
    struct Output {
        status: ExitStatus,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    }

    enum SystemUtility {
        DiskImage,
        CodeSignature,
    }
    impl SystemUtility {
        fn path(&self) -> &'static str {
            match self {
                Self::DiskImage => "/usr/bin/hdiutil",
                Self::CodeSignature => "/usr/bin/codesign",
            }
        }
    }

    fn run(utility: SystemUtility, args: Vec<OsString>, timeout: Duration) -> Result<Output> {
        // The installation API is synchronous and may be called from a runtime.
        // A private worker owns async pipes, their exact byte cap and child cleanup.
        std::thread::spawn(move || -> Result<Output> {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(async move {
                    let mut child = tokio::process::Command::new(utility.path())
                        .args(args)
                        .env_clear()
                        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                        .stdin(Stdio::null())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .kill_on_drop(true)
                        .spawn()
                        .with_context(|| format!("start {}", utility.path()))?;
                    let stdout = child
                        .stdout
                        .take()
                        .context("macOS utility stdout unavailable")?;
                    let stderr = child
                        .stderr
                        .take()
                        .context("macOS utility stderr unavailable")?;
                    async fn read(
                        mut stream: impl tokio::io::AsyncRead + Unpin,
                    ) -> Result<Vec<u8>> {
                        let mut bytes = Vec::new();
                        (&mut stream)
                            .take((OUTPUT_LIMIT + 1) as u64)
                            .read_to_end(&mut bytes)
                            .await?;
                        if bytes.len() > OUTPUT_LIMIT {
                            bail!("macOS utility exceeded its output limit");
                        }
                        Ok(bytes)
                    }
                    let operation = async {
                        let (stdout, stderr, status) =
                            tokio::try_join!(read(stdout), read(stderr), async {
                                child.wait().await.context("wait for macOS utility")
                            })?;
                        Ok(Output {
                            status,
                            stdout,
                            stderr,
                        })
                    };
                    match tokio::time::timeout(timeout, operation).await {
                        Ok(Ok(output)) => Ok(output),
                        failure => {
                            let _ = child.start_kill();
                            let _ =
                                tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
                            match failure {
                                Ok(Err(error)) => Err(error),
                                Err(_) => {
                                    Err(anyhow::anyhow!("{} exceeded its deadline", utility.path()))
                                }
                                Ok(Ok(_)) => unreachable!(),
                            }
                        }
                    }
                })
        })
        .join()
        .map_err(|_| anyhow::anyhow!("macOS utility worker failed"))?
    }

    fn hdiutil(args: Vec<OsString>, timeout: Duration) -> Result<Output> {
        run(SystemUtility::DiskImage, args, timeout)
    }

    fn verify_signature(application: &Path) -> Result<()> {
        successful(run(SystemUtility::CodeSignature,
            vec!["--verify".into(), "--deep".into(), "--strict".into(), application.as_os_str().into()],
            Duration::from_secs(120))?, "copied application signature verification")
            .context("the copied application must retain its official signature; Mirket does not alter signing metadata (use a local home outside a file-provider folder)")?;
        Ok(())
    }

    fn successful(output: Output, operation: &str) -> Result<Vec<u8>> {
        if !output.status.success() {
            let end = output.stderr.len().min(4096);
            bail!(
                "{operation} failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr[..end]).trim()
            );
        }
        Ok(output.stdout)
    }

    fn mounted_at(bytes: &[u8], mount: &Path) -> Result<bool> {
        let text =
            std::str::from_utf8(bytes).context("disk-image utility returned non-UTF8 XML")?;
        let document = roxmltree::Document::parse_with_options(
            text,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                nodes_limit: 32_768,
                ..Default::default()
            },
        )
        .context("disk-image utility returned invalid property-list XML")?;
        if !document.root_element().has_tag_name("plist") {
            bail!("disk-image utility returned no property list");
        }
        let mount = mount
            .to_str()
            .context("disk-image mount path must be UTF-8")?;
        for key in document
            .descendants()
            .filter(|n| n.has_tag_name("key") && n.text() == Some("mount-point"))
        {
            let value = key.next_sibling_element();
            if value.is_some_and(|n| n.has_tag_name("string") && n.text() == Some(mount)) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    struct Mount {
        directory: Option<tempfile::TempDir>,
        path: PathBuf,
        active: bool,
        confirmed: bool,
    }
    impl Mount {
        fn new() -> Result<Self> {
            let parent = std::env::temp_dir().canonicalize()?;
            let directory = tempfile::Builder::new()
                .prefix("mirket-dmg-")
                .tempdir_in(parent)?;
            let path = directory.path().join("mount");
            fs::create_dir(&path)?;
            Ok(Self {
                directory: Some(directory),
                path,
                active: false,
                confirmed: false,
            })
        }
        fn attach(&mut self, input: &Path) -> Result<()> {
            self.active = true;
            let bytes = successful(
                hdiutil(
                    vec![
                        "attach".into(),
                        "-readonly".into(),
                        "-nobrowse".into(),
                        "-noautoopen".into(),
                        "-mountpoint".into(),
                        self.path.as_os_str().into(),
                        "-plist".into(),
                        input.as_os_str().into(),
                    ],
                    Duration::from_secs(120),
                )?,
                "read-only DMG attach",
            )?;
            self.confirmed = mounted_at(&bytes, &self.path)?;
            if !self.confirmed {
                bail!("disk image did not mount at Mirket's private mountpoint");
            }
            Ok(())
        }
        fn close(&mut self) -> Result<()> {
            if !self.active {
                return Ok(());
            }
            let result = (|| -> Result<()> {
                if !self.confirmed
                    && let Ok(bytes) = hdiutil(
                        vec!["info".into(), "-plist".into()],
                        Duration::from_secs(15),
                    )
                    .and_then(|o| successful(o, "DMG mount inspection"))
                    && matches!(mounted_at(&bytes, &self.path), Ok(false))
                {
                    return Ok(());
                }
                for force in [false, true] {
                    let mut args = vec!["detach".into(), self.path.as_os_str().into()];
                    if force {
                        args.push("-force".into());
                    }
                    if hdiutil(args, Duration::from_secs(20)).is_ok_and(|o| o.status.success()) {
                        return Ok(());
                    }
                }
                let bytes = successful(
                    hdiutil(
                        vec!["info".into(), "-plist".into()],
                        Duration::from_secs(15),
                    )?,
                    "DMG mount inspection",
                )?;
                if mounted_at(&bytes, &self.path)? {
                    bail!("disk image remains mounted");
                }
                Ok(())
            })();
            self.active = false;
            if let Err(error) = result {
                // Never recursively delete a directory that might still be mounted.
                if let Some(directory) = self.directory.take() {
                    let _ = directory.keep();
                }
                return Err(error).with_context(|| {
                    format!(
                        "could not detach the private DMG mount {}; it was retained for recovery",
                        self.path.display()
                    )
                });
            }
            self.directory.take();
            Ok(())
        }
    }
    impl Drop for Mount {
        fn drop(&mut self) {
            if let Err(error) = self.close() {
                eprintln!("mirket package: {error:#}");
            }
        }
    }

    pub(super) fn extract(artifact: &Artifact, input: &Path, root: &Path) -> Result<()> {
        let mut mount = Mount::new()?;
        let result = (|| {
            mount.attach(input)?;
            let source = mount.path.join(
                artifact
                    .strip_prefix
                    .as_deref()
                    .context("DMG application name missing")?,
            );
            let application = root.join(&artifact.destination);
            copy_application(&source, &application, artifact.max_unpacked_bytes)?;
            verify_signature(&application)
        })();
        let cleanup = mount.close();
        match (result, cleanup) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(error), Err(cleanup)) => {
                Err(error.context(format!("DMG cleanup also failed: {cleanup:#}")))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn dmg_mount_identity_uses_the_exact_private_path() {
            let xml = b"<?xml version=\"1.0\"?><!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\"><plist version=\"1.0\"><dict><key>system-entities</key><array><dict><key>mount-point</key><string>/private/mirket/mount</string></dict></array></dict></plist>";
            assert!(mounted_at(xml, Path::new("/private/mirket/mount")).unwrap());
            assert!(!mounted_at(xml, Path::new("/private/another/mount")).unwrap());
            assert!(mounted_at(b"bad output", Path::new("/private/mirket/mount")).is_err());
        }
        #[test]
        fn dmg_failed_attachment_removes_its_private_mount_directory() {
            let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
            let image = temp.path().join("invalid.dmg");
            fs::write(&image, b"not a disk image").unwrap();
            let mut mount = Mount::new().unwrap();
            let private = mount.path.parent().unwrap().to_owned();
            assert!(mount.attach(&image).is_err());
            mount.close().unwrap();
            assert!(!private.exists());
        }
        #[test]
        fn dmg_unsigned_application_fails_signature_verification() {
            let temp = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
            let application = temp.path().join("Unsigned.app");
            fs::create_dir_all(application.join("Contents/MacOS")).unwrap();
            fs::write(
                application.join("Contents/MacOS/unsigned"),
                b"not signed application code",
            )
            .unwrap();
            let error = verify_signature(&application).unwrap_err();
            assert!(
                format!("{error:#}").contains("copied application signature verification failed")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> tempfile::TempDir {
        tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap()
    }
    fn artifact(bytes: &[u8]) -> Artifact {
        Artifact {
            url: "https://example.invalid/application.dmg".into(),
            sha256: util::sha256(bytes),
            format: "dmg-app".into(),
            max_download_bytes: 1024,
            max_unpacked_bytes: 1024,
            strip_prefix: Some("Fixture.app".into()),
            destination: "Fixture.app".into(),
            files: vec![],
        }
    }
    #[test]
    fn dmg_digest_and_udif_trailer_are_checked_before_mounting() {
        let temp = temp();
        let image = temp.path().join("fixture.dmg");
        let bytes = vec![0u8; 512];
        fs::write(&image, &bytes).unwrap();
        let a = artifact(&bytes);
        assert!(
            verify_image(&a, &image)
                .unwrap_err()
                .to_string()
                .contains("UDIF trailer")
        );
        let mut changed = bytes;
        changed[..4].copy_from_slice(b"koly");
        fs::write(&image, &changed).unwrap();
        assert!(
            verify_image(&a, &image)
                .unwrap_err()
                .to_string()
                .contains("SHA-256")
        );
        verify_image(&artifact(&changed), &image).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn dmg_application_copy_preserves_internal_framework_links_only() {
        let temp = temp();
        let app = temp.path().join("Fixture.app");
        fs::create_dir_all(app.join("Contents/Framework.framework/Versions/A")).unwrap();
        fs::write(
            app.join("Contents/Framework.framework/Versions/A/library"),
            b"framework bytes",
        )
        .unwrap();
        std::os::unix::fs::symlink(
            "A",
            app.join("Contents/Framework.framework/Versions/Current"),
        )
        .unwrap();
        std::os::unix::fs::symlink(
            "Versions/Current/library",
            app.join("Contents/Framework.framework/library"),
        )
        .unwrap();
        std::os::unix::fs::symlink("/Applications", temp.path().join("Applications")).unwrap();
        let output = temp.path().join("staged/Fixture.app");
        copy_application(&app, &output, 1024).unwrap();
        assert_eq!(
            fs::read_link(output.join("Contents/Framework.framework/library")).unwrap(),
            Path::new("Versions/Current/library")
        );
        assert_eq!(
            fs::read(output.join("Contents/Framework.framework/library")).unwrap(),
            b"framework bytes"
        );
        assert!(!output.parent().unwrap().join("Applications").exists());
        assert!(copy_application(&app, &output, 1024).is_err());
        assert!(copy_application(&app, &temp.path().join("small.app"), 1).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn dmg_links_reject_absolute_escape_dangling_cycle_and_ancestor_targets() {
        let temp = temp();
        let app = temp.path().join("Fixture.app");
        fs::create_dir_all(app.join("Contents")).unwrap();
        fs::write(app.join("Contents/regular"), b"ordinary file").unwrap();
        fs::write(temp.path().join("outside"), b"outside").unwrap();
        for (name, target) in [
            ("absolute", "/etc/passwd"),
            ("escape", "../../outside"),
            ("missing", "absent"),
            ("cycle", "cycle"),
            ("ancestor", ".."),
            ("non-directory", "regular/../regular"),
        ] {
            let path = app.join("Contents").join(name);
            std::os::unix::fs::symlink(target, &path).unwrap();
            assert!(link_target(&app, &path).is_err(), "{name}");
        }
        std::os::unix::fs::symlink("absolute", app.join("Contents/indirect")).unwrap();
        assert!(link_target(&app, &app.join("Contents/indirect")).is_err());
    }
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "requires MIRKET_DMG_FIXTURES, MIRKET_DMG_TEST_CLI and MIRKET_DMG_TOOL_HOME with codesign registered; no downloads or editor execution"]
    fn dmg_live_official_applications_preserve_the_verified_bundle_inventory() {
        let fixtures = PathBuf::from(
            std::env::var_os("MIRKET_DMG_FIXTURES").expect("set MIRKET_DMG_FIXTURES"),
        );
        let fixtures = fixtures.canonicalize().unwrap();
        let cli = PathBuf::from(
            std::env::var_os("MIRKET_DMG_TEST_CLI").expect("set MIRKET_DMG_TEST_CLI"),
        );
        let tool_home = PathBuf::from(
            std::env::var_os("MIRKET_DMG_TOOL_HOME").expect("set MIRKET_DMG_TOOL_HOME"),
        );
        let fixture_home = tempfile::Builder::new()
            .prefix("mirket-dmg-live-")
            .tempdir_in(std::env::temp_dir().canonicalize().unwrap())
            .unwrap();
        let (image, app, executable, digest, files, links) = (
            "inkscape.dmg",
            "Inkscape.app",
            "inkscape",
            "eacca94ab01e59467cdd5452a2867c3734450f9a4cfb3346956cc9b07f34f3f1",
            12_446,
            41,
        );
        {
            let stage = tempfile::Builder::new()
                .prefix("dmg-extraction-")
                .tempdir_in(fixture_home.path())
                .unwrap();
            let a = Artifact {
                url: "https://example.invalid/already-downloaded-official-fixture.dmg".into(),
                sha256: digest.into(),
                format: "dmg-app".into(),
                max_download_bytes: 512 * 1024 * 1024,
                max_unpacked_bytes: 2 * 1024 * 1024 * 1024,
                strip_prefix: Some(app.into()),
                destination: app.into(),
                files: vec![],
            };
            archive::extract(&a, &fixtures.join(image), stage.path()).unwrap();
            let app = stage.path().join(app);
            let entries = archive::files(stage.path(), std::slice::from_ref(&app)).unwrap();
            let actual_links = entries
                .iter()
                .filter(|path| fs::symlink_metadata(path).unwrap().file_type().is_symlink())
                .count();
            assert_eq!(actual_links, links, "{image}");
            assert_eq!(entries.len() - actual_links, files, "{image}");
            assert!(app.join("Contents/MacOS").join(executable).is_file());
            assert!(!stage.path().join("Applications").exists());
            let status = std::process::Command::new(&cli)
                .arg("--home")
                .arg(&tool_home)
                .args(["tool", "run", "codesign", "--cwd"])
                .arg(stage.path())
                .args(["--timeout", "120", "--", "--verify", "--deep", "--strict"])
                .arg(&app)
                .status()
                .unwrap();
            assert!(
                status.success(),
                "copied signature verification failed for {image}"
            );
        }
    }
}
