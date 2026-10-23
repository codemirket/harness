//! Explicitly registered local executables. Execution is a CLI-only operation.
use crate::{paths::Paths, util};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub executable: PathBuf,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub helpers: BTreeMap<String, String>,
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        bail!("tool names use 1-64 lowercase letters, digits or hyphens");
    }
    Ok(())
}

fn load(paths: &Paths) -> Result<BTreeMap<String, Tool>> {
    let path = paths.root.join("tools.json");
    if !path.try_exists()? {
        return Ok(BTreeMap::new());
    }
    Ok(serde_json::from_slice(&util::read_bounded(
        &path,
        1024 * 1024,
    )?)?)
}

fn save(paths: &Paths, data: &BTreeMap<String, Tool>) -> Result<()> {
    util::atomic_write(
        &paths.root.join("tools.json"),
        &serde_json::to_vec_pretty(data)?,
    )
}

fn lock(paths: &Paths) -> Result<util::FileLock> {
    paths.ensure()?;
    util::FileLock::acquire(&paths.root.join("tools.lock"))
        .context("another tool registry operation is running")
}

fn executable_digest(path: &Path) -> Result<String> {
    let data = util::read_bounded(path, 256 * 1024 * 1024)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::metadata(path)?.permissions().mode() & 0o111 == 0 {
            bail!("tool is not executable");
        }
    }
    Ok(util::sha256(&data))
}

pub fn register(paths: &Paths, name: &str, executable: &Path) -> Result<Value> {
    validate_name(name)?;
    if !executable.is_absolute() {
        bail!("executable must be an absolute path");
    }
    // Resolve a user-selected launcher once; pin the resulting regular file.
    let executable = executable.canonicalize()?;
    let sha256 = executable_digest(&executable)?;
    let _lock = lock(paths)?;
    let mut tools = load(paths)?;
    if tools.contains_key(name) {
        bail!(
            "tool is already registered; remove it explicitly before registering different bytes"
        );
    }
    tools.insert(
        name.to_string(),
        Tool {
            executable,
            sha256,
            helpers: BTreeMap::new(),
        },
    );
    save(paths, &tools)?;
    Ok(
        json!({"registered": name, "tool":tools[name], "scope":"CLI execution under the host's permissions"}),
    )
}

pub fn remove(paths: &Paths, name: &str) -> Result<Value> {
    let _lock = lock(paths)?;
    let mut tools = load(paths)?;
    if tools.remove(name).is_none() {
        bail!("unknown tool: {name}");
    }
    save(paths, &tools)?;
    Ok(json!({"removed":name}))
}

pub fn list(paths: &Paths) -> Result<Value> {
    Ok(serde_json::to_value(load(paths)?)?)
}

fn localized_podman(
    paths: &Paths,
    source: &Path,
    client_sha: &str,
    helpers: &[PathBuf; 2],
) -> Result<Tool> {
    let gvproxy_sha = executable_digest(&helpers[0])?;
    let vfkit_sha = executable_digest(&helpers[1])?;
    let generation =
        crate::util::sha256(format!("{client_sha}:{gvproxy_sha}:{vfkit_sha}").as_bytes());
    let parent = paths.root.join("dependencies/podman");
    crate::util::ensure_directory(&parent)?;
    let destination = parent.join(generation);
    let client_name = source
        .file_name()
        .context("Podman client filename missing")?;
    let client = destination.join("bin").join(client_name);
    let gvproxy = destination.join("libexec/podman/gvproxy");
    let vfkit = destination.join("libexec/podman/vfkit");
    let expected = [
        (client.as_path(), client_sha),
        (gvproxy.as_path(), gvproxy_sha.as_str()),
        (vfkit.as_path(), vfkit_sha.as_str()),
    ];
    if !destination.try_exists()? {
        let stage = tempfile::Builder::new()
            .prefix("podman-stage-")
            .tempdir_in(&parent)?;
        let bin = stage.path().join("bin");
        let libexec = stage.path().join("libexec/podman");
        crate::util::ensure_directory(&bin)?;
        crate::util::ensure_directory(&libexec)?;
        for (from, to) in [
            (source, bin.join(client_name)),
            (helpers[0].as_path(), libexec.join("gvproxy")),
            (helpers[1].as_path(), libexec.join("vfkit")),
        ] {
            crate::util::reject_symlink_components(from)?;
            fs::copy(from, &to)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&to, fs::Permissions::from_mode(0o700))?;
            }
        }
        fs::rename(stage.path(), &destination)?;
    }
    for (path, sha) in expected {
        crate::util::reject_symlink_components(path)?;
        if executable_digest(path)? != sha {
            bail!(
                "localized Podman dependency has changed: {}",
                path.display()
            );
        }
    }
    Ok(Tool {
        executable: client,
        sha256: client_sha.to_owned(),
        helpers: BTreeMap::from([("gvproxy".into(), gvproxy_sha), ("vfkit".into(), vfkit_sha)]),
    })
}

/// Import the exact registered native Podman client and helpers into Mirket's
/// owned dependency tree.
pub fn localize_podman(paths: &Paths) -> Result<Value> {
    let _lock = lock(paths)?;
    let mut tools = load(paths)?;
    let old = tools
        .get("podman")
        .context("register Podman before localization")?;
    ensure_registered_podman(paths, old)?;
    if old.executable.starts_with(&paths.root) && old.helpers.len() == 2 {
        return Ok(json!({"localized":old.executable,"changed":false}));
    }
    let helper_files = crate::podman::machine_helper_files(&old.executable)?;
    let replacement = localized_podman(paths, &old.executable, &old.sha256, &helper_files)?;
    let config = paths.root.join("execution/podman/containers.conf");
    let new_config = crate::podman::helper_config(&replacement.executable)?;
    let previous = if config.try_exists()? {
        let saved = crate::util::read_bounded(&config, 4096)?;
        let exact = crate::podman::helper_config(&old.executable)?;
        ensure!(
            saved == exact.as_bytes() || saved == new_config.as_bytes(),
            "Mirket Podman helper configuration has drifted; no registry change"
        );
        Some(saved)
    } else {
        None
    };
    let changed_config = previous.as_deref() != Some(new_config.as_bytes());
    if changed_config && previous.is_some() {
        crate::util::atomic_write(&config, new_config.as_bytes())?;
    }
    tools.insert("podman".into(), replacement);
    if let Err(error) = save(paths, &tools) {
        if changed_config && let Some(previous) = previous {
            crate::util::atomic_write(&config, &previous)
                .context("restore Podman helper configuration after registry failure")?;
        }
        return Err(error).context("save localized Podman registry");
    }
    Ok(json!({"localized":tools["podman"].executable,"changed":true,"source_preserved":true}))
}

fn ensure_registered_podman(paths: &Paths, tool: &Tool) -> Result<()> {
    if executable_digest(&tool.executable)? != tool.sha256 {
        bail!("registered podman executable changed; inspect the approved binary");
    }
    if !tool.helpers.is_empty() {
        ensure!(
            tool.executable.starts_with(&paths.root) && tool.helpers.len() == 2,
            "localized Podman helper receipt is incomplete or outside ~/.mirket"
        );
        let files = crate::podman::machine_helper_files(&tool.executable)?;
        for (name, file) in [("gvproxy", &files[0]), ("vfkit", &files[1])] {
            ensure!(
                tool.helpers
                    .get(name)
                    .is_some_and(|sha| executable_digest(file).ok().as_ref() == Some(sha)),
                "localized Podman helper {name} changed"
            );
        }
    }
    Ok(())
}

/// Resolve only an explicitly selected executable whose bytes still match.
pub(crate) fn verified(paths: &Paths, name: &str) -> Result<PathBuf> {
    let tools = load(paths)?;
    let tool = tools.get(name).with_context(|| format!("register the prerequisite: mirket tool register {name} --executable /absolute/path/to/{name}"))?;
    if executable_digest(&tool.executable)? != tool.sha256 {
        bail!("registered {name} executable changed; inspect and register the approved binary");
    }
    if name == "podman" {
        ensure_registered_podman(paths, tool)?;
    }
    Ok(tool.executable.clone())
}

pub fn doctor(paths: &Paths) -> Result<Value> {
    let checks: Vec<_> = load(paths)?
        .into_keys()
        .map(|name| match verified(paths, &name) {
            Ok(_) => json!({"name":name,"ok":true}),
            Err(error) => json!({"name":name,"ok":false,"issue":error.to_string()}),
        })
        .collect();
    Ok(json!({"ok":checks.iter().all(|c| c["ok"] == true),"checks":checks}))
}

pub async fn run(
    paths: &Paths,
    name: &str,
    cwd: &Path,
    args: &[String],
    timeout: u64,
) -> Result<i32> {
    if timeout == 0 || timeout > 86400 {
        bail!("timeout must be between 1 and 86400 seconds");
    }
    if args.len() > 256 || args.iter().map(String::len).sum::<usize>() > 65536 {
        bail!("too many tool arguments");
    }
    let executable = verified(paths, name)?;
    let cwd = crate::paths::canonical_project(cwd)?;
    let mut command = tokio::process::Command::new(executable);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let shutdown = crate::process::shutdown_signal()?;
    run_owned(&mut command, Duration::from_secs(timeout), shutdown, true).await
}

async fn run_owned(
    command: &mut tokio::process::Command,
    timeout: Duration,
    cancelled: impl std::future::Future<Output = Result<()>>,
    inherit_terminal: bool,
) -> Result<i32> {
    let mut owned = crate::process::OwnedChild::spawn(command, inherit_terminal)?;
    let result = tokio::select! {
        status = owned.child().wait() => status.map(|s| s.code().unwrap_or(1)).map_err(anyhow::Error::from),
        _ = tokio::time::sleep(timeout) => Err(anyhow::anyhow!("tool exceeded {} seconds; owned processes terminated", timeout.as_secs_f64())),
        signal = cancelled => signal.and_then(|_| Err(anyhow::anyhow!("tool cancelled; owned processes terminated"))),
    };
    owned.finish().await?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    // Native test executable avoids shell/platform quoting and exercises the same
    // owned child mechanics on Windows, macOS and Linux without dependencies.
    #[test]
    fn process_fixture() {
        let Ok(mode) = std::env::var("MIRKET_TEST_TOOL_MODE") else {
            return;
        };
        let root = PathBuf::from(std::env::var_os("MIRKET_TEST_TOOL_ROOT").unwrap());
        if mode == "leaf" {
            std::fs::write(root.join("ready"), b"ready").unwrap();
            std::thread::sleep(Duration::from_secs(3));
            std::fs::write(root.join("late"), b"late side effect").unwrap();
            return;
        }
        if mode == "normal" {
            std::process::exit(7);
        }
        let mut leaf = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tools::tests::process_fixture", "--nocapture"])
            .env("MIRKET_TEST_TOOL_MODE", "leaf")
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !root.join("ready").exists() {
            assert!(std::time::Instant::now() < deadline, "leaf did not start");
            std::thread::sleep(Duration::from_millis(10));
        }
        if mode == "success" {
            std::process::exit(7);
        }
        let _ = leaf.wait().unwrap();
    }

    fn fixture(root: &Path, mode: &str) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "tools::tests::process_fixture", "--nocapture"])
            .env("MIRKET_TEST_TOOL_MODE", mode)
            .env("MIRKET_TEST_TOOL_ROOT", root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command
    }
    async fn ready(root: &Path) {
        tokio::time::timeout(Duration::from_secs(5), async {
            while !root.join("ready").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("owned descendant must start before cancellation");
    }
    async fn no_late_effect(root: &Path) {
        assert!(
            root.join("ready").exists(),
            "probe must actually launch the descendant"
        );
        tokio::time::sleep(Duration::from_millis(3300)).await;
        assert!(
            !root.join("late").exists(),
            "descendant wrote after ownership ended"
        );
    }

    #[tokio::test]
    async fn owned_tool_preserves_normal_exit_status() {
        let root = tempfile::tempdir().unwrap();
        let result = run_owned(
            &mut fixture(root.path(), "normal"),
            Duration::from_secs(10),
            std::future::pending(),
            false,
        )
        .await
        .unwrap();
        assert_eq!(result, 7);
    }
    #[tokio::test]
    async fn owned_tool_cleans_descendants_after_launcher_success() {
        let root = tempfile::tempdir().unwrap();
        let result = run_owned(
            &mut fixture(root.path(), "success"),
            Duration::from_secs(10),
            std::future::pending(),
            false,
        )
        .await
        .unwrap();
        assert_eq!(result, 7);
        no_late_effect(root.path()).await;
    }
    #[tokio::test]
    async fn owned_tool_timeout_stops_late_descendant_writes() {
        let root = tempfile::tempdir().unwrap();
        let error = run_owned(
            &mut fixture(root.path(), "wait"),
            Duration::from_secs(2),
            std::future::pending(),
            false,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("exceeded"));
        no_late_effect(root.path()).await;
    }
    #[tokio::test]
    async fn owned_tool_cancellation_stops_late_descendant_writes() {
        let root = tempfile::tempdir().unwrap();
        let cancel = async {
            ready(root.path()).await;
            Ok(())
        };
        let error = run_owned(
            &mut fixture(root.path(), "wait"),
            Duration::from_secs(10),
            cancel,
            false,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("cancelled"));
        no_late_effect(root.path()).await;
    }
    #[tokio::test]
    async fn owned_tool_dropped_future_stops_late_descendant_writes() {
        let root = tempfile::tempdir().unwrap();
        let mut command = fixture(root.path(), "wait");
        let task = tokio::spawn(async move {
            run_owned(
                &mut command,
                Duration::from_secs(10),
                std::future::pending(),
                false,
            )
            .await
        });
        ready(root.path()).await;
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        no_late_effect(root.path()).await;
    }

    #[test]
    fn registration_never_executes_and_changed_bytes_fail() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().to_owned()).unwrap();
        let executable = dir.path().join("tool");
        std::fs::write(&executable, b"fixture-a").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        register(&paths, "renderer", &executable).unwrap();
        assert_eq!(doctor(&paths).unwrap()["ok"], true);
        std::fs::write(&executable, b"fixture-b").unwrap();
        assert_eq!(doctor(&paths).unwrap()["ok"], false);
        assert!(register(&paths, "renderer", &executable).is_err());
        remove(&paths, "renderer").unwrap();
        assert_eq!(list(&paths).unwrap(), json!({}));
        assert!(register(&paths, "../escape", &executable).is_err());
    }

    #[test]
    fn podman_localization_preserves_the_registration_and_pins_helpers() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_owned()).unwrap();
        let source = paths.home.join("approved-podman");
        let bin = source.join("bin");
        let libexec = source.join("libexec/podman");
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(&libexec).unwrap();
        let client = bin.join("podman-remote");
        let gvproxy = libexec.join("gvproxy");
        let vfkit = libexec.join("vfkit");
        for (path, bytes) in [
            (&client, b"approved-client".as_slice()),
            (&gvproxy, b"approved-gvproxy".as_slice()),
            (&vfkit, b"approved-vfkit".as_slice()),
        ] {
            fs::write(path, bytes).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
            }
        }
        register(&paths, "podman", &client).unwrap();
        let config = paths.root.join("execution/podman/containers.conf");
        crate::util::ensure_directory(config.parent().unwrap()).unwrap();
        fs::write(&config, crate::podman::helper_config(&client).unwrap()).unwrap();
        let old = fs::read(&config).unwrap();
        let changed = localize_podman(&paths).unwrap();
        assert_eq!(changed["changed"], true);
        let selected = verified(&paths, "podman").unwrap();
        assert!(selected.starts_with(paths.root.join("dependencies/podman")));
        assert_eq!(fs::read(&client).unwrap(), b"approved-client");
        assert_ne!(fs::read(&config).unwrap(), old);
        assert_eq!(
            fs::read_to_string(&config).unwrap(),
            crate::podman::helper_config(&selected).unwrap()
        );
        assert_eq!(localize_podman(&paths).unwrap()["changed"], false);
        let helper = selected
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("libexec/podman/vfkit");
        fs::write(&helper, b"tampered-vfkit").unwrap();
        assert!(
            verified(&paths, "podman")
                .unwrap_err()
                .to_string()
                .contains("helper vfkit changed")
        );
    }

    #[test]
    fn podman_localization_refuses_unowned_helper_configuration() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_owned()).unwrap();
        let source = paths.home.join("approved-podman");
        let bin = source.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let client = bin.join("podman-remote");
        for name in ["podman-remote", "gvproxy", "vfkit"] {
            let path = bin.join(name);
            fs::write(&path, name).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            }
        }
        register(&paths, "podman", &client).unwrap();
        let config = paths.root.join("execution/podman/containers.conf");
        crate::util::ensure_directory(config.parent().unwrap()).unwrap();
        fs::write(&config, b"[engine]\nhelper_binaries_dir = [\"/foreign\"]\n").unwrap();
        assert!(localize_podman(&paths).is_err());
        assert_eq!(verified(&paths, "podman").unwrap(), client);
        assert_eq!(
            fs::read(&config).unwrap(),
            b"[engine]\nhelper_binaries_dir = [\"/foreign\"]\n"
        );
    }

    #[tokio::test]
    async fn run_rechecks_registration_before_spawning() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir(&home).unwrap();
        let paths = Paths::new(home).unwrap();
        // The full debug test executable can exceed the production registry's
        // size limit. Compile a small native fixture with the test toolchain
        // instead of coupling this behavioral probe to debug-symbol size.
        let executable = dir
            .path()
            .join(format!("tool{}", std::env::consts::EXE_SUFFIX));
        let source = dir.path().join("fixture.rs");
        std::fs::write(
            &source,
            r#"fn main() { std::fs::write("invoked", b"fixture ran").unwrap(); }"#,
        )
        .unwrap();
        let compiled = std::process::Command::new("rustc")
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        register(&paths, "fixture", &executable).unwrap();

        let args = vec![];
        assert_eq!(
            run(&paths, "fixture", dir.path(), &args, 10).await.unwrap(),
            0
        );
        assert_eq!(
            std::fs::read(dir.path().join("invoked")).unwrap(),
            b"fixture ran"
        );
        std::fs::remove_file(dir.path().join("invoked")).unwrap();

        std::fs::write(&executable, b"changed after registration").unwrap();
        let error = run(&paths, "fixture", dir.path(), &args, 10)
            .await
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("registered fixture executable changed")
        );
        assert!(!dir.path().join("invoked").exists());

        let error = run(&paths, "missing", dir.path(), &args, 10)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("register the prerequisite"));
    }
}
