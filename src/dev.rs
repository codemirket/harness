use crate::{catalog::Catalog, update, util};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const BINARIES: [&str; 4] = ["mirket", "install-codex", "install-claude", "install-all"];
const MAX_CARGO_MESSAGE: u64 = 4 * 1024 * 1024;

pub fn cargo_program() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO") {
        return path.into();
    }
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let path = PathBuf::from(home)
            .join(".cargo/bin")
            .join(if cfg!(windows) { "cargo.exe" } else { "cargo" });
        if path.is_file() {
            return path;
        }
    }
    "cargo".into()
}

fn checkout(project: &Path) -> Result<PathBuf> {
    let project = project.canonicalize()?;
    let manifest = String::from_utf8(util::read_bounded(&project.join("Cargo.toml"), 128 * 1024)?)?;
    let manifest = manifest.parse::<toml_edit::DocumentMut>()?;
    if manifest
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(toml_edit::Item::as_str)
        != Some("mirket")
    {
        bail!("development commands require a Mirket source checkout");
    }
    Ok(project)
}

fn cargo(project: &Path, args: &[&str]) -> Result<()> {
    eprintln!("mirket dev: cargo {}", args.join(" "));
    if !Command::new(cargo_program())
        .args(args)
        .current_dir(project)
        .status()?
        .success()
    {
        bail!("cargo {} failed", args.join(" "));
    }
    Ok(())
}

pub fn check(project: &Path) -> Result<Value> {
    let project = checkout(project)?;
    cargo(&project, &["fmt", "--all", "--", "--check"])?;
    cargo(
        &project,
        &[
            "clippy",
            "--locked",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    cargo(&project, &["test", "--locked", "--all-targets"])?;
    Ok(json!({"ok":true,"checks":["format","clippy","tests"],"source":project}))
}

pub fn format(project: &Path) -> Result<Value> {
    let project = checkout(project)?;
    cargo(&project, &["fmt", "--all"])?;
    Ok(json!({"ok":true,"source":project}))
}

pub fn test(project: &Path, filter: Option<&str>, ignored: bool) -> Result<Value> {
    let project = checkout(project)?;
    let mut args = vec!["test", "--locked", "--all-targets"];
    if let Some(filter) = filter {
        if filter.starts_with('-') {
            bail!("test filter cannot begin with '-'");
        }
        args.push(filter);
    }
    if ignored {
        args.extend(["--", "--ignored"]);
    }
    cargo(&project, &args)?;
    Ok(json!({"ok":true,"filter":filter,"ignored":ignored,"source":project}))
}

pub fn build(project: &Path, release: bool) -> Result<Value> {
    let project = checkout(project)?;
    let mut args = vec!["build", "--locked", "--bins"];
    if release {
        args.push("--release");
    }
    cargo(&project, &args)?;
    Ok(json!({"ok":true,"profile":if release{"release"}else{"debug"},"source":project}))
}

pub fn dist(project: &Path, output: &Path) -> Result<Value> {
    let project = checkout(project)?;
    match std::fs::symlink_metadata(output) {
        Ok(_) => bail!("distribution destination must be new"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let source = source_package(&project)?;
    let artifacts = build_artifacts(&project, &source)?;
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let parent = parent.canonicalize()?;
    let output = parent.join(
        output
            .file_name()
            .context("distribution path must have a name")?,
    );
    let stage = tempfile::tempdir_in(&parent)?;
    let binaries = stage_binaries(&artifacts, stage.path())?;
    let info = executable_info(&binaries["mirket"].path, &source.version)?;
    let manifest = package_binaries(stage.path(), &binaries, &info)?;
    util::atomic_write(
        &stage.path().join("mirket-release.json"),
        &serde_json::to_vec_pretty(&manifest)?,
    )?;
    util::atomic_write(
        &stage.path().join("LICENSE"),
        &util::read_bounded(&project.join("LICENSE"), 65536)?,
    )?;
    util::atomic_write(
        &stage.path().join("NOTICE.md"),
        &util::read_bounded(&project.join("docs/third-party-notices.md"), 256 * 1024)?,
    )?;
    std::fs::rename(stage.path(), &output)?;
    Ok(
        json!({"ok":true,"target":info.target,"version":info.version,"output":output,"published":false}),
    )
}

#[derive(Debug)]
struct SourcePackage {
    manifest: PathBuf,
    id: String,
    version: String,
}

fn source_package(project: &Path) -> Result<SourcePackage> {
    let manifest = project.join("Cargo.toml").canonicalize()?;
    let document = String::from_utf8(util::read_bounded(&manifest, 128 * 1024)?)?
        .parse::<toml_edit::DocumentMut>()?;
    let version = document
        .get("package")
        .and_then(|package| package.get("version"))
        .and_then(toml_edit::Item::as_str)
        .context("Mirket Cargo.toml must declare its package version")?;
    update::parse_version(version)?;
    let bytes = command_output(
        Command::new(cargo_program()).current_dir(project).args([
            "metadata",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ]),
        "read Cargo source metadata",
        Duration::from_secs(120),
        MAX_CARGO_MESSAGE,
    )?;
    let metadata: Value = serde_json::from_slice(&bytes).context("invalid Cargo metadata")?;
    package_identity(&metadata, &manifest, version)
}

fn package_identity(metadata: &Value, manifest: &Path, version: &str) -> Result<SourcePackage> {
    let matches = metadata["packages"]
        .as_array()
        .context("Cargo metadata did not list source packages")?
        .iter()
        .filter(|package| {
            package["manifest_path"].as_str().is_some_and(|path| {
                Path::new(path)
                    .canonicalize()
                    .is_ok_and(|path| path == manifest)
            })
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        bail!("Cargo metadata must identify exactly one package for the source Cargo.toml");
    }
    let package = matches[0];
    if package["name"].as_str() != Some("mirket") || package["version"].as_str() != Some(version) {
        bail!("Cargo source package identity differs from Cargo.toml");
    }
    let id = package["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .context("Cargo metadata omitted the source package ID")?;
    Ok(SourcePackage {
        manifest: manifest.to_path_buf(),
        id: id.to_string(),
        version: version.to_string(),
    })
}

struct CargoArtifacts<'a> {
    source: &'a SourcePackage,
    executables: BTreeMap<String, PathBuf>,
}

impl<'a> CargoArtifacts<'a> {
    fn new(source: &'a SourcePackage) -> Self {
        Self {
            source,
            executables: BTreeMap::new(),
        }
    }

    fn record(&mut self, message: &Value) -> Result<()> {
        if message["reason"] != "compiler-artifact"
            || !message["target"]["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
        {
            return Ok(());
        }
        let Some(name) = message["target"]["name"]
            .as_str()
            .filter(|name| BINARIES.contains(name))
        else {
            return Ok(());
        };
        let manifest_matches = message["manifest_path"].as_str().is_some_and(|path| {
            Path::new(path)
                .canonicalize()
                .is_ok_and(|path| path == self.source.manifest)
        });
        if !manifest_matches {
            return Ok(());
        }
        if message["package_id"].as_str() != Some(&self.source.id) {
            bail!("Cargo artifact package identity changed for {name}");
        }
        let executable = PathBuf::from(
            message["executable"]
                .as_str()
                .with_context(|| format!("Cargo did not report an executable for {name}"))?,
        );
        if !executable.is_absolute() {
            bail!("Cargo reported a relative executable path for {name}");
        }
        if let Some(previous) = self
            .executables
            .insert(name.to_string(), executable.clone())
            && previous != executable
        {
            bail!(
                "Cargo produced multiple executable paths for {name}; build one target per distribution"
            );
        }
        Ok(())
    }

    fn finish(self) -> Result<BTreeMap<String, PathBuf>> {
        for name in BINARIES {
            if !self.executables.contains_key(name) {
                bail!("Cargo did not produce the {name} executable for the source package");
            }
        }
        if self.executables.values().collect::<BTreeSet<_>>().len() != BINARIES.len() {
            bail!("Cargo reported the same executable path for multiple binaries");
        }
        Ok(self.executables)
    }
}

fn cargo_messages(
    reader: impl BufRead,
    source: &SourcePackage,
    diagnostics: &mut impl Write,
) -> Result<BTreeMap<String, PathBuf>> {
    let mut reader = reader;
    let mut artifacts = CargoArtifacts::new(source);
    let mut line = Vec::new();
    loop {
        line.clear();
        let bytes = (&mut reader)
            .take(MAX_CARGO_MESSAGE + 1)
            .read_until(b'\n', &mut line)?;
        if bytes == 0 {
            break;
        }
        if bytes as u64 > MAX_CARGO_MESSAGE {
            bail!("Cargo message exceeded {MAX_CARGO_MESSAGE} bytes");
        }
        let Ok(message) = serde_json::from_slice::<Value>(&line) else {
            diagnostics.write_all(&line)?;
            continue;
        };
        if message["reason"] == "compiler-message"
            && let Some(rendered) = message["message"]["rendered"].as_str()
        {
            diagnostics.write_all(rendered.as_bytes())?;
        }
        artifacts.record(&message)?;
    }
    artifacts.finish()
}

fn build_artifacts(project: &Path, source: &SourcePackage) -> Result<BTreeMap<String, PathBuf>> {
    let args = [
        "build",
        "--locked",
        "--bins",
        "--release",
        "--message-format=json-render-diagnostics",
    ];
    eprintln!("mirket dev: cargo {}", args.join(" "));
    let mut child = Command::new(cargo_program())
        .args(args)
        .current_dir(project)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("start Cargo release build")?;
    let result = cargo_messages(
        BufReader::new(child.stdout.take().context("read Cargo build output")?),
        source,
        &mut std::io::stderr().lock(),
    );
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait().context("wait for Cargo release build")?;
    let executables =
        result.with_context(|| format!("collect Cargo release artifacts ({status})"))?;
    if !status.success() {
        bail!("Cargo release build failed ({status}); inspect the build diagnostics above");
    }
    Ok(executables)
}

fn command_output(
    command: &mut Command,
    operation: &str,
    timeout: Duration,
    limit: u64,
) -> Result<Vec<u8>> {
    let mut output = tempfile::tempfile()?;
    let mut error = tempfile::tempfile()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(output.try_clone()?)
        .stderr(error.try_clone()?)
        .spawn()
        .with_context(|| format!("{operation}: failed to start executable"))?;
    let result = (|| {
        let started = Instant::now();
        let status = loop {
            if output.metadata()?.len() > limit || error.metadata()?.len() > limit {
                bail!("{operation}: output exceeded {limit} bytes");
            }
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if started.elapsed() >= timeout {
                bail!("{operation}: exceeded {} seconds", timeout.as_secs());
            }
            thread::sleep(Duration::from_millis(25));
        };
        if output.metadata()?.len() > limit || error.metadata()?.len() > limit {
            bail!("{operation}: output exceeded {limit} bytes");
        }
        if !status.success() {
            error.seek(SeekFrom::Start(0))?;
            let mut bytes = Vec::new();
            error.take(limit).read_to_end(&mut bytes)?;
            bail!(
                "{operation}: {status}: {}",
                String::from_utf8_lossy(&bytes).trim()
            );
        }
        output.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        output.take(limit).read_to_end(&mut bytes)?;
        Ok(bytes)
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutableInfo {
    name: String,
    version: String,
    target: String,
}

fn validate_info(bytes: &[u8], source_version: &str) -> Result<ExecutableInfo> {
    let info: ExecutableInfo =
        serde_json::from_slice(bytes).context("built Mirket info is not valid JSON")?;
    if info.name != "mirket" || info.version != source_version {
        bail!("built executable identity must match Mirket source version {source_version}");
    }
    update::validate_manifest(&release_manifest(&info, "0".repeat(64)))?;
    Ok(info)
}

fn executable_info(binary: &Path, source_version: &str) -> Result<ExecutableInfo> {
    let output = command_output(
        Command::new(binary)
            .env("MIRKET_DIRECT", "1")
            .args(["info", "--json"]),
        "execute built Mirket identity probe",
        Duration::from_secs(10),
        64 * 1024,
    )
    .context("distribution requires a host capable of executing the selected Cargo target; check CARGO_BUILD_TARGET or Cargo build.target")?;
    validate_info(&output, source_version)
}

struct StagedBinary {
    path: PathBuf,
    sha256: String,
}

fn stage_binaries(
    artifacts: &BTreeMap<String, PathBuf>,
    stage: &Path,
) -> Result<BTreeMap<String, StagedBinary>> {
    let directory = stage.join("binaries");
    std::fs::create_dir(&directory)?;
    let mut binaries = BTreeMap::new();
    for name in BINARIES {
        let source = artifacts
            .get(name)
            .context("missing built executable")?
            .canonicalize()
            .with_context(|| format!("resolve Cargo executable for {name}"))?;
        let bytes = util::read_bounded(&source, 256 * 1024 * 1024)?;
        let mut path = directory.join(name);
        if let Some(extension) = source.extension() {
            path.set_extension(extension);
        }
        util::atomic_write(&path, &bytes)?;
        std::fs::set_permissions(&path, std::fs::metadata(&source)?.permissions())?;
        binaries.insert(
            name.to_string(),
            StagedBinary {
                path,
                sha256: util::sha256(&bytes),
            },
        );
    }
    Ok(binaries)
}

fn asset_name(binary: &str, info: &ExecutableInfo) -> String {
    let extension = if info.target.ends_with("-windows-msvc") {
        ".exe"
    } else {
        ""
    };
    format!("{binary}-{}{extension}", info.target)
}

fn release_manifest(info: &ExecutableInfo, sha256: String) -> update::ReleaseManifest {
    update::ReleaseManifest {
        version: info.version.clone(),
        assets: vec![update::ReleaseAsset {
            target: info.target.clone(),
            url: format!(
                "https://github.com/codemirket/harness/releases/download/mirket-v{}/{}",
                info.version,
                asset_name("mirket", info)
            ),
            sha256,
        }],
    }
}

fn package_binaries(
    stage: &Path,
    binaries: &BTreeMap<String, StagedBinary>,
    info: &ExecutableInfo,
) -> Result<update::ReleaseManifest> {
    let mut hashes = String::new();
    let manifest = release_manifest(info, binaries["mirket"].sha256.clone());
    update::validate_manifest(&manifest)?;
    for name in BINARIES {
        let binary = &binaries[name];
        let name = asset_name(name, info);
        std::fs::rename(&binary.path, stage.join(&name))?;
        hashes.push_str(&format!("{}  {name}\n", binary.sha256));
    }
    std::fs::remove_dir(stage.join("binaries"))?;
    util::atomic_write(&stage.join("SHA256SUMS"), hashes.as_bytes())?;
    Ok(manifest)
}

fn stats(mut samples: Vec<f64>) -> Value {
    samples.sort_by(f64::total_cmp);
    json!({"samples":samples.len(),"median_us":samples[samples.len()/2],"p95_us":samples[((samples.len()-1)*95)/100],"min_us":samples[0]})
}

pub fn benchmark(catalog: &Catalog, iterations: usize) -> Result<Value> {
    if !(10..=10000).contains(&iterations) {
        bail!("iterations must be between 10 and 10000");
    }
    let exe = std::env::current_exe()?;
    let mut startup = Vec::new();
    for _ in 0..iterations.min(30) {
        let begin = Instant::now();
        let output = Command::new(&exe).args(["--version"]).output()?;
        if !output.status.success() {
            bail!("startup probe failed");
        }
        startup.push(begin.elapsed().as_secs_f64() * 1e6);
    }
    let mut search = Vec::new();
    let mut plan = Vec::new();
    let mut read = Vec::new();
    for _ in 0..iterations {
        let begin = Instant::now();
        std::hint::black_box(catalog.search("database", 0, 20)?);
        search.push(begin.elapsed().as_secs_f64() * 1e6);
        let begin = Instant::now();
        std::hint::black_box(catalog.capability_plan("CFO", &["Excel Expert".to_string()])?);
        plan.push(begin.elapsed().as_secs_f64() * 1e6);
        let begin = Instant::now();
        std::hint::black_box(catalog.read("engineering-judgment", "SKILL.md", 65536)?);
        read.push(begin.elapsed().as_secs_f64() * 1e6);
    }
    let bytes = util::read_bounded(&exe, 256 * 1024 * 1024).context("read benchmark binary")?;
    Ok(
        json!({"version":env!("CARGO_PKG_VERSION"),"target":env!("MIRKET_TARGET"),"binary_sha256":util::sha256(&bytes),"binary_bytes":bytes.len(),"process_version_startup":stats(startup),"catalog_search":stats(search),"capability_plan":stats(plan),"skill_read":stats(read),"scope":"Local warmed filesystem, sequential calls; startup includes process scheduling. No model, network or application-quality measurement."}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn source(directory: &Path) -> SourcePackage {
        let path = directory.join("Cargo.toml");
        std::fs::write(&path, "[package]\nname='mirket'\nversion='9.8.7'\n").unwrap();
        SourcePackage {
            manifest: path.canonicalize().unwrap(),
            id: "path+file:///source#mirket@9.8.7".to_string(),
            version: "9.8.7".to_string(),
        }
    }

    fn artifact(source: &SourcePackage, name: &str, output: &Path) -> Value {
        json!({
            "reason":"compiler-artifact", "package_id":source.id,
            "manifest_path":source.manifest, "target":{"name":name,"kind":["bin"]},
            "executable":output.join(name)
        })
    }

    fn messages(source: &SourcePackage, output: &Path) -> Vec<u8> {
        BINARIES
            .iter()
            .map(|name| format!("{}\n", artifact(source, name, output)))
            .collect::<String>()
            .into_bytes()
    }

    #[test]
    fn cargo_identity_requires_current_manifest_and_version() {
        let root = tempfile::tempdir().unwrap();
        let source = source(root.path());
        let metadata = json!({"packages":[
            {"name":"mirket","version":"9.8.7","id":"other-package","manifest_path":root.path().join("other/Cargo.toml")},
            {"name":"mirket","version":"9.8.7","id":source.id,"manifest_path":source.manifest}
        ]});
        let identity = package_identity(&metadata, &source.manifest, "9.8.7").unwrap();
        assert_eq!(identity.id, source.id);
        assert!(package_identity(&metadata, &source.manifest, "9.8.6").is_err());
        assert!(package_identity(&json!({"packages":[]}), &source.manifest, "9.8.7").is_err());
    }

    #[test]
    fn cargo_artifacts_use_reported_paths_and_retain_human_diagnostics() {
        let root = tempfile::tempdir().unwrap();
        let source = source(root.path());
        let output = root
            .path()
            .join("custom-target/x86_64-pc-windows-msvc/release");
        let mut data = b"cargo wrapper diagnostic\n".to_vec();
        data.extend_from_slice(
            format!("{}\n", json!({"reason":"compiler-message","message":{"rendered":"error: useful compiler diagnostic\n"}})).as_bytes(),
        );
        let mut unrelated = artifact(&source, "mirket", &root.path().join("wrong"));
        unrelated["manifest_path"] = json!(root.path().join("dependency/Cargo.toml"));
        data.extend_from_slice(format!("{unrelated}\n").as_bytes());
        data.extend_from_slice(&messages(&source, &output));
        let mut diagnostics = Vec::new();
        let artifacts = cargo_messages(Cursor::new(data), &source, &mut diagnostics).unwrap();
        for name in BINARIES {
            assert_eq!(artifacts[name], output.join(name));
        }
        assert_eq!(
            String::from_utf8(diagnostics).unwrap(),
            "cargo wrapper diagnostic\nerror: useful compiler diagnostic\n"
        );
    }

    #[test]
    fn cargo_artifacts_reject_changed_package_or_mixed_targets() {
        let root = tempfile::tempdir().unwrap();
        let source = source(root.path());
        let mut artifacts = CargoArtifacts::new(&source);
        artifacts
            .record(&artifact(&source, "mirket", &root.path().join("target-a")))
            .unwrap();
        assert!(
            artifacts
                .record(&artifact(&source, "mirket", &root.path().join("target-b")))
                .is_err()
        );
        let mut changed = artifact(&source, "install-all", root.path());
        changed["package_id"] = json!("different-source-package");
        assert!(CargoArtifacts::new(&source).record(&changed).is_err());
        let mut relative = artifact(&source, "install-all", Path::new("target/release"));
        assert!(CargoArtifacts::new(&source).record(&relative).is_err());
        relative["executable"] = Value::Null;
        assert!(CargoArtifacts::new(&source).record(&relative).is_err());
        assert!(CargoArtifacts::new(&source).finish().is_err());
    }

    #[test]
    fn cargo_messages_are_bounded() {
        let root = tempfile::tempdir().unwrap();
        let source = source(root.path());
        let input = vec![b'x'; MAX_CARGO_MESSAGE as usize + 1];
        let error = cargo_messages(Cursor::new(input), &source, &mut Vec::new()).unwrap_err();
        assert!(error.to_string().contains("exceeded"));
    }

    #[test]
    fn executable_identity_rejects_version_and_target_mismatch() {
        for info in [
            json!({"name":"other","version":"9.8.7","target":"x86_64-apple-darwin"}),
            json!({"name":"mirket","version":"1.0.0","target":"x86_64-apple-darwin"}),
            json!({"name":"mirket","version":"9.8.7","target":"unknown-target"}),
        ] {
            assert!(validate_info(&serde_json::to_vec(&info).unwrap(), "9.8.7").is_err());
        }
    }

    #[test]
    fn unavailable_target_execution_reports_a_clear_packaging_limit() {
        let root = tempfile::tempdir().unwrap();
        let binary = root.path().join("foreign-mirket.exe");
        std::fs::write(&binary, b"not an executable for this host").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let error = executable_info(&binary, "9.8.7").unwrap_err();
        assert!(error.to_string().contains("host capable of executing"));
        assert!(error.to_string().contains("Cargo build.target"));
    }

    #[test]
    fn packaging_uses_built_identity_and_immutable_artifact_copies() {
        let root = tempfile::tempdir().unwrap();
        let output = root
            .path()
            .canonicalize()
            .unwrap()
            .join("configured-cargo-target");
        std::fs::create_dir(&output).unwrap();
        let mut artifacts = BTreeMap::new();
        for name in BINARIES {
            let path = output.join(format!("{name}.exe"));
            std::fs::write(&path, name.as_bytes()).unwrap();
            artifacts.insert(name.to_string(), path);
        }
        let stage = tempfile::tempdir().unwrap();
        let stage_path = stage.path().canonicalize().unwrap();
        let binaries = stage_binaries(&artifacts, &stage_path).unwrap();
        std::fs::write(&artifacts["mirket"], "concurrent build output").unwrap();
        let info = validate_info(
            br#"{"name":"mirket","version":"9.8.7","target":"x86_64-pc-windows-msvc"}"#,
            "9.8.7",
        )
        .unwrap();
        let manifest = package_binaries(&stage_path, &binaries, &info).unwrap();
        assert_eq!(manifest.version, "9.8.7");
        assert_eq!(manifest.assets[0].target, "x86_64-pc-windows-msvc");
        assert_eq!(
            manifest.assets[0].url,
            "https://github.com/codemirket/harness/releases/download/mirket-v9.8.7/mirket-x86_64-pc-windows-msvc.exe"
        );
        assert_eq!(manifest.assets[0].sha256, util::sha256(b"mirket"));
        let checksums = std::fs::read_to_string(stage.path().join("SHA256SUMS")).unwrap();
        for name in BINARIES {
            let filename = format!("{name}-x86_64-pc-windows-msvc.exe");
            assert_eq!(
                std::fs::read(stage.path().join(&filename)).unwrap(),
                name.as_bytes()
            );
            assert!(
                checksums.contains(&format!("{}  {filename}\n", util::sha256(name.as_bytes())))
            );
        }
        assert!(!stage.path().join("binaries").exists());
    }
}
