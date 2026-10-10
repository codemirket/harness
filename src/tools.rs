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
    let tools = load(paths)?;
    let tool = tools
        .get(name)
        .context("tool is not registered; use mirket tool register")?;
    if executable_digest(&tool.executable)? != tool.sha256 {
        bail!("tool executable changed; inspect it before registering it again");
    }
    let cwd = crate::paths::canonical_project(cwd)?;
    let mut child = tokio::process::Command::new(&tool.executable)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let result = tokio::time::timeout(Duration::from_secs(timeout), child.wait()).await;
    match result {
        Ok(status) => Ok(status?.code().unwrap_or(1)),
        Err(_) => {
            child.kill().await?;
            let _ = child.wait().await;
            bail!("tool exceeded {timeout} seconds; direct child terminated");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
