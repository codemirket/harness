use crate::{
    install::{OperationLock, Target, read_state, read_state_snapshot},
    paths::Paths,
    util::{atomic_write, read_bounded, reject_symlink_components, sha256},
};
use anyhow::{Context, Result, bail};
use reqwest::{Url, blocking::Client, redirect::Policy};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const MAX_BINARY: u64 = 256 * 1024 * 1024;
const MANIFEST_URL: &str =
    "https://github.com/codemirket/harness/releases/latest/download/mirket-release.json";

#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    pub from: Option<PathBuf>,
    pub sha256: Option<String>,
    pub check: bool,
    pub entrypoint: Option<PathBuf>,
    pub expected_entrypoint_sha256: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct UpdateReport {
    pub current_version: String,
    pub available_version: String,
    pub updated: bool,
    pub reconfigured: bool,
    pub binary: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<EntrypointReport>,
}
#[derive(Debug, Clone, Serialize)]
pub struct EntrypointReport {
    pub path: PathBuf,
    pub sha256: String,
    pub updated: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    pub version: String,
    pub assets: Vec<ReleaseAsset>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseAsset {
    pub target: String,
    pub url: String,
    pub sha256: String,
}

pub fn target_triple() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-gnu"),
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-gnu"),
        ("windows", "aarch64") => Ok("aarch64-pc-windows-msvc"),
        ("windows", "x86_64") => Ok("x86_64-pc-windows-msvc"),
        pair => bail!("no release target is configured for {} {}", pair.0, pair.1),
    }
}

pub fn update(paths: &Paths, options: &UpdateOptions) -> Result<UpdateReport> {
    if options.from.is_some() != options.sha256.is_some() {
        bail!("offline update requires both --from and --sha256");
    }
    if options.expected_entrypoint_sha256.is_some() && options.entrypoint.is_none() {
        bail!("--expected-entrypoint-sha256 requires --entrypoint");
    }
    if options.check && options.entrypoint.is_some() {
        bail!("--entrypoint cannot be combined with --check");
    }
    if let Some(digest) = &options.expected_entrypoint_sha256 {
        validate_digest(digest)?;
    }
    let _lock = OperationLock::acquire(paths, "update")?;
    let (state, expected_state) = read_state_snapshot(paths)?
        .context("run mirket setup before updating the configured environment")?;
    let current_version = state.version.clone();
    let current_number = parse_version(&current_version)?;
    let (candidate_bytes, expected_digest, expected_version) = if let Some(source) = &options.from {
        let digest = options
            .sha256
            .as_deref()
            .context("missing artifact digest")?;
        validate_digest(digest)?;
        (
            read_bounded(source, MAX_BINARY).context("read update artifact")?,
            digest.to_ascii_lowercase(),
            None,
        )
    } else {
        let client = release_client()?;
        let manifest_bytes = download(&client, MANIFEST_URL, 1024 * 1024).context("read official release manifest; a published mirket release with mirket-release.json is required")?;
        let manifest: ReleaseManifest =
            serde_json::from_slice(&manifest_bytes).context("invalid release manifest")?;
        validate_manifest(&manifest)?;
        let available_number = parse_version(&manifest.version)?;
        if available_number < current_number {
            bail!(
                "release version {} is older than installed {}",
                manifest.version,
                current_version
            );
        }
        let target = target_triple()?;
        let asset = manifest
            .assets
            .iter()
            .find(|asset| asset.target == target)
            .with_context(|| {
                format!("release {} has no artifact for {target}", manifest.version)
            })?;
        if options.check {
            return Ok(UpdateReport {
                current_version,
                available_version: manifest.version,
                updated: false,
                reconfigured: false,
                binary: paths.binary(),
                entrypoint: None,
            });
        }
        (
            download(&client, &asset.url, MAX_BINARY)?,
            asset.sha256.to_ascii_lowercase(),
            Some(manifest.version),
        )
    };
    if sha256(&candidate_bytes) != expected_digest {
        bail!("update artifact SHA-256 does not match; installation was not changed");
    }
    if candidate_bytes.is_empty() {
        bail!("update artifact is empty");
    }
    let entrypoint = options
        .entrypoint
        .as_deref()
        .map(|path| {
            EntrypointPlan::prepare(
                paths,
                path,
                options.expected_entrypoint_sha256.as_deref(),
                &candidate_bytes,
                &expected_digest,
            )
        })
        .transpose()?;
    // A staged executable owns its embedded payload and performs the same setup
    // transaction as an ordinary installation. No updater-specific payload path exists.
    let staging = tempfile::Builder::new()
        .prefix(".update-")
        .tempdir_in(&paths.root)?;
    let staged = staging.path().join(if cfg!(windows) {
        "mirket.exe"
    } else {
        "mirket"
    });
    atomic_write(&staged, &candidate_bytes)?;
    make_executable(&staged)?;
    let version_output = run_bounded(
        Command::new(&staged)
            .env("MIRKET_DIRECT", "1")
            .arg("--version"),
        Duration::from_secs(10),
    )?;
    let available_version = std::str::from_utf8(&version_output)
        .context("update artifact version is not UTF-8")?
        .trim()
        .strip_prefix("mirket ")
        .context("update artifact did not identify itself as mirket")?
        .to_string();
    let available_number = parse_version(&available_version)?;
    if available_number < current_number {
        bail!("artifact version {available_version} is older than installed {current_version}");
    }
    if expected_version
        .as_deref()
        .is_some_and(|expected| expected != available_version)
    {
        bail!("artifact version differs from its release manifest");
    }
    if options.check {
        return Ok(UpdateReport {
            current_version,
            available_version,
            updated: false,
            reconfigured: false,
            binary: paths.binary(),
            entrypoint: None,
        });
    }
    let before = read_bounded(&paths.binary(), MAX_BINARY)
        .context("installed mirket binary is unavailable")?;
    let old_digest = sha256(&before);
    let target = if state.options.targets.len() == 2 {
        "all"
    } else {
        match state.options.targets[0] {
            Target::Codex => "codex",
            Target::Claude => "claude",
        }
    };
    let mut command = Command::new(&staged);
    command.env("MIRKET_DIRECT", "1");
    command.arg("--home").arg(&paths.home).args([
        "setup",
        "--yes",
        "--target",
        target,
        "--expected-state",
        &expected_state,
    ]);
    if state.options.microsoft_learn {
        command.arg("--microsoft-learn");
    }
    if let Some(plan) = &entrypoint {
        plan.recheck()
            .context("entrypoint changed before setup; installation was not changed")?;
    }
    run_bounded(&mut command, Duration::from_secs(1800)).context(
        "new runtime setup failed; run mirket setup to recover any interrupted transaction",
    )?;
    // Verify the committed generation instead of trusting a successful child exit.
    let receipt = read_state(paths)?.context("updated runtime did not write a setup receipt")?;
    if receipt.version != available_version
        || receipt.options != state.options
        || sha256(&read_bounded(&paths.binary(), MAX_BINARY)?) != expected_digest
    {
        bail!(
            "updated runtime did not produce the expected binary and saved setup; run mirket doctor to inspect the installation"
        );
    }
    let entrypoint = entrypoint
        .map(|plan| plan.apply(&available_version, &paths.binary()))
        .transpose()?;
    Ok(UpdateReport {
        current_version,
        available_version,
        updated: old_digest != expected_digest,
        reconfigured: true,
        binary: paths.binary(),
        entrypoint,
    })
}

struct EntrypointPlan {
    path: PathBuf,
    before_digest: String,
    before_permissions: std::fs::Permissions,
    digest: String,
    staged: tempfile::NamedTempFile,
}

fn same_permissions(left: &std::fs::Permissions, right: &std::fs::Permissions) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        left.mode() == right.mode()
    }
    #[cfg(not(unix))]
    {
        // Temporary Windows attributes change when a staged file is persisted.
        left.readonly() == right.readonly()
    }
}

impl EntrypointPlan {
    fn prepare(
        paths: &Paths,
        path: &Path,
        expected: Option<&str>,
        bytes: &[u8],
        digest: &str,
    ) -> Result<Self> {
        if !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
            || !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("mirket" | "mirket.exe")
            )
        {
            bail!("entrypoint must be an absolute mirket or mirket.exe file without traversal");
        }
        reject_symlink_components(path)?;
        let before_digest =
            sha256(&read_bounded(path, MAX_BINARY).context("read existing entrypoint")?);
        let path = path.canonicalize()?;
        if path == paths.binary().canonicalize()? {
            bail!("entrypoint must be separate from the managed runtime");
        }
        #[cfg(windows)]
        if path == std::env::current_exe()?.canonicalize()? {
            bail!(
                "cannot replace the executing Windows entrypoint; run update from the distinct managed or published executable"
            );
        }
        if before_digest != digest
            && expected.is_none_or(|value| !value.eq_ignore_ascii_case(&before_digest))
        {
            bail!(
                "entrypoint SHA-256 does not match; supply --expected-entrypoint-sha256 for the existing file; installation was not changed"
            );
        }
        let before_permissions = std::fs::metadata(&path)?.permissions();
        let mut staged = tempfile::Builder::new()
            .prefix(".mirket-entrypoint-")
            .tempfile_in(path.parent().context("entrypoint has no parent")?)?;
        staged.write_all(bytes)?;
        make_executable(staged.path())?;
        staged.as_file().sync_all()?;
        Ok(Self {
            path,
            before_digest,
            before_permissions,
            digest: digest.into(),
            staged,
        })
    }

    fn recheck(&self) -> Result<()> {
        if sha256(&read_bounded(&self.path, MAX_BINARY)?) != self.before_digest
            || !same_permissions(
                &std::fs::metadata(&self.path)?.permissions(),
                &self.before_permissions,
            )
        {
            bail!("entrypoint changed since preflight; refusing to overwrite it");
        }
        Ok(())
    }

    fn apply(self, installed_version: &str, managed_binary: &Path) -> Result<EntrypointReport> {
        (|| {
        self.recheck()?;
        let permissions = self.staged.as_file().metadata()?.permissions();
        let updated = self.before_digest != self.digest || !same_permissions(&self.before_permissions, &permissions);
        if updated {
            self.staged.persist(&self.path)
                .with_context(|| format!("replace entrypoint {}; close processes using that file and retry from the managed executable", self.path.display()))?;
        }
        if sha256(&read_bounded(&self.path, MAX_BINARY)?) != self.digest
            || !same_permissions(&std::fs::metadata(&self.path)?.permissions(), &permissions)
        {
            bail!(
                "entrypoint changed after replacement; inspect {}",
                self.path.display()
            );
        }
        Ok(EntrypointReport {
            path: self.path,
            sha256: self.digest,
            updated,
        })
        })().with_context(|| {
            format!("managed runtime updated to {installed_version}, but entrypoint refresh failed; retry from {} with the same explicit entrypoint options", managed_binary.display())
        })
    }
}

pub fn validate_manifest(manifest: &ReleaseManifest) -> Result<()> {
    parse_version(&manifest.version)?;
    if manifest.assets.is_empty() || manifest.assets.len() > 16 {
        bail!("release manifest must list between 1 and 16 artifacts");
    }
    let mut targets = std::collections::BTreeSet::new();
    for asset in &manifest.assets {
        if !matches!(
            asset.target.as_str(),
            "aarch64-apple-darwin"
                | "x86_64-apple-darwin"
                | "aarch64-unknown-linux-gnu"
                | "x86_64-unknown-linux-gnu"
                | "aarch64-pc-windows-msvc"
                | "x86_64-pc-windows-msvc"
        ) {
            bail!("unsupported release target: {}", asset.target);
        }
        if !targets.insert(&asset.target) {
            bail!("release manifest repeats target {}", asset.target);
        }
        validate_digest(&asset.sha256)?;
        let url = Url::parse(&asset.url).context("invalid release asset URL")?;
        let prefix = format!(
            "/codemirket/harness/releases/download/mirket-v{}/",
            manifest.version
        );
        if url.scheme() != "https"
            || url.host_str() != Some("github.com")
            || !url.path().starts_with(&prefix)
            || url
                .path()
                .strip_prefix(&prefix)
                .is_none_or(|name| name.is_empty() || name.contains('/'))
            || url.query().is_some()
            || url.fragment().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
        {
            bail!("release asset URL must identify the matching official GitHub release");
        }
    }
    Ok(())
}
fn validate_digest(digest: &str) -> Result<()> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("SHA-256 must contain exactly 64 hexadecimal characters");
    }
    Ok(())
}
pub(crate) fn parse_version(version: &str) -> Result<(u64, u64, u64)> {
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty()
                || !part.bytes().all(|byte| byte.is_ascii_digit())
                || part.len() > 1 && part.starts_with('0')
        })
    {
        bail!("release version must be a stable MAJOR.MINOR.PATCH version");
    }
    Ok((parts[0].parse()?, parts[1].parse()?, parts[2].parse()?))
}
fn release_client() -> Result<Client> {
    let policy = Policy::custom(|attempt| {
        let url = attempt.url();
        if attempt.previous().len() >= 5 {
            return attempt.error("too many release redirects");
        }
        if url.scheme() != "https"
            || !matches!(
                url.host_str(),
                Some(
                    "github.com"
                        | "release-assets.githubusercontent.com"
                        | "objects.githubusercontent.com"
                )
            )
        {
            return attempt.error("release redirect left the official HTTPS delivery hosts");
        }
        attempt.follow()
    });
    Ok(Client::builder()
        .user_agent(concat!("mirket/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(90))
        .redirect(policy)
        .build()?)
}
fn download(client: &Client, url: &str, max: u64) -> Result<Vec<u8>> {
    let response = client.get(url).send()?.error_for_status()?;
    if response.content_length().is_some_and(|length| length > max) {
        bail!("release response exceeds {max} bytes");
    }
    let mut bytes = Vec::new();
    response.take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        bail!("release response exceeds {max} bytes");
    }
    Ok(bytes)
}
fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::{fs, os::unix::fs::PermissionsExt};
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
fn run_bounded(command: &mut Command, timeout: Duration) -> Result<Vec<u8>> {
    let mut output = tempfile::tempfile()?;
    let mut error = tempfile::tempfile()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(output.try_clone()?)
        .stderr(error.try_clone()?)
        .spawn()
        .context("start verified mirket executable")?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= timeout {
            child.kill()?;
            child.wait()?;
            bail!(
                "mirket child operation exceeded {} seconds",
                timeout.as_secs()
            );
        }
        if output.metadata()?.len() > 1024 * 1024 || error.metadata()?.len() > 1024 * 1024 {
            child.kill()?;
            child.wait()?;
            bail!("mirket child operation exceeded its output limit");
        }
        thread::sleep(Duration::from_millis(25));
    };
    if output.metadata()?.len() > 1024 * 1024 || error.metadata()?.len() > 1024 * 1024 {
        bail!("mirket child operation exceeded its output limit");
    }
    output.seek(SeekFrom::Start(0))?;
    error.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    output.take(1024 * 1024).read_to_end(&mut bytes)?;
    if !status.success() {
        let mut message = String::new();
        error.take(8192).read_to_string(&mut message)?;
        bail!("mirket child exited with {status}: {}", message.trim());
    }
    Ok(bytes)
}

#[cfg(test)]
mod entrypoint_tests {
    use super::*;

    #[test]
    fn entrypoint_update_late_failure_reports_partial_success_without_overwrite() {
        let temporary = tempfile::tempdir().unwrap();
        let paths = Paths::new(temporary.path().canonicalize().unwrap()).unwrap();
        std::fs::create_dir_all(paths.binary().parent().unwrap()).unwrap();
        std::fs::write(paths.binary(), b"previous runtime").unwrap();
        let path = paths.home.join("mirket");
        std::fs::write(&path, b"previous entrypoint").unwrap();
        let candidate = b"verified candidate";
        let version = env!("CARGO_PKG_VERSION");
        let old_digest = sha256(b"previous entrypoint");
        let plan = EntrypointPlan::prepare(
            &paths,
            &path,
            Some(&old_digest),
            candidate,
            &sha256(candidate),
        )
        .unwrap();
        // Exercise the commit boundary after the managed setup has completed.
        std::fs::write(paths.binary(), candidate).unwrap();
        std::fs::write(&path, b"concurrent edit").unwrap();
        let error = format!("{:#}", plan.apply(version, &paths.binary()).unwrap_err());
        assert!(
            error.contains(&format!(
                "managed runtime updated to {version}, but entrypoint refresh failed"
            )),
            "{error}"
        );
        assert!(
            error.contains("entrypoint changed since preflight"),
            "{error}"
        );
        assert_eq!(std::fs::read(paths.binary()).unwrap(), candidate);
        assert_eq!(std::fs::read(&path).unwrap(), b"concurrent edit");
        assert!(
            EntrypointPlan::prepare(
                &paths,
                &path,
                Some(&old_digest),
                candidate,
                &sha256(candidate)
            )
            .is_err()
        );
        std::fs::write(&path, b"previous entrypoint").unwrap();
        let retry = EntrypointPlan::prepare(
            &paths,
            &path,
            Some(&old_digest),
            candidate,
            &sha256(candidate),
        )
        .unwrap();
        assert!(retry.apply(version, &paths.binary()).unwrap().updated);
        assert_eq!(std::fs::read(path).unwrap(), candidate);
    }
}
