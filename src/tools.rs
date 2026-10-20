//! Explicitly registered local executables. Execution is a CLI-only operation.
use crate::{paths::Paths, util};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tool {
    pub executable: PathBuf,
    pub sha256: String,
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
    tools.insert(name.to_string(), Tool { executable, sha256 });
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

/// Resolve only an explicitly selected executable whose bytes still match.
pub(crate) fn verified(paths: &Paths, name: &str) -> Result<PathBuf> {
    let tools = load(paths)?;
    let tool = tools.get(name).with_context(|| format!("register the prerequisite: mirket tool register {name} --executable /absolute/path/to/{name}"))?;
    if executable_digest(&tool.executable)? != tool.sha256 {
        bail!("registered {name} executable changed; inspect and register the approved binary");
    }
    Ok(tool.executable.clone())
}

pub fn doctor(paths: &Paths) -> Result<Value> {
    let checks: Vec<_> = load(paths)?.into_iter().map(|(name,tool)| {
        match executable_digest(&tool.executable) {
            Ok(hash) if hash == tool.sha256 => json!({"name":name,"ok":true}),
            Ok(_) => json!({"name":name,"ok":false,"issue":"executable bytes changed; inspect and register the approved binary"}),
            Err(error) => json!({"name":name,"ok":false,"issue":error.to_string()}),
        }
    }).collect();
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
