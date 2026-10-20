//! Owned IPC boundary: third-party code gets private state, never host tools/keys.
use crate::{paths::Paths, process::OwnedChild, util};
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const MAX: u64 = 4 * 1024 * 1024;

// Rootless OCI namespace setup can race across concurrently launched containers.
// Keep this cross-process gate only through the identity handshake: desktop
// clients share an engine, but established sessions must remain independent.
async fn startup_lock(path: &Path, wait: Duration) -> Result<util::FileLock> {
    tokio::time::timeout(wait, async {
        loop {
            match util::FileLock::acquire(path) {
                Ok(lock) => return Ok(lock),
                Err(error)
                    if matches!(
                        error.downcast_ref::<std::fs::TryLockError>(),
                        Some(std::fs::TryLockError::WouldBlock)
                    ) => {}
                Err(error) => return Err(error.context("agent runtime startup lock")),
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .context("agent runtime startup queue deadline exceeded; no request dispatched")?
}

// Separate short operations from long workers. Limits apply across desktop processes.
async fn admission(paths: &Paths, worker: bool, wait: Duration) -> Result<util::FileLock> {
    let directory = paths.root.join("execution/podman/agent-admission");
    util::ensure_directory(&directory)?;
    let count = if worker { 2 } else { 1 };
    let lane = if worker { "worker" } else { "operation" };
    tokio::time::timeout(wait, async {
        loop {
            for slot in 0..count {
                match util::FileLock::acquire(&directory.join(format!("{lane}-{slot}.lock"))) {
                    Ok(permit) => return Ok(permit),
                    Err(error)
                        if matches!(
                            error.downcast_ref::<std::fs::TryLockError>(),
                            Some(std::fs::TryLockError::WouldBlock)
                        ) => {}
                    Err(error) => return Err(error),
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .context("agent runtime capacity queue expired; no request dispatched")?
}

pub fn enabled(paths: &Paths) -> Result<bool> {
    Ok(crate::install::saved_options(paths)?
        .is_some_and(|options| options.packages.iter().any(|id| id == "agent-runtime")))
}
pub fn scope(paths: &Paths, project: &Path) -> Result<String> {
    crate::state::State::open(paths)?.runtime_scope(project)
}

pub struct Session {
    admission: Option<util::FileLock>,
    maintenance: Option<util::FileLock>,
    project_scope: Option<String>,
    project_root: Option<std::path::PathBuf>,
    child: OwnedChild,
    input: Option<tokio::process::ChildStdin>,
    closed: bool,
    output: BufReader<tokio::process::ChildStdout>,
    engine: crate::podman::Engine,
    container: String,
    stderr: Option<tokio::task::JoinHandle<()>>,
}
impl Session {
    pub async fn open(paths: &Paths, project: Option<&Path>) -> Result<Self> {
        Self::open_lane(paths, project, false).await
    }
    pub async fn open_worker(paths: &Paths, project: &Path) -> Result<Self> {
        Self::open_lane(paths, Some(project), true).await
    }
    async fn open_lane(paths: &Paths, project: Option<&Path>, worker: bool) -> Result<Self> {
        ensure!(
            enabled(paths)?,
            "agent-runtime is not selected; use mirket package install design agent-runtime"
        );
        let maintenance = crate::maintenance::shared(paths)?;
        let permit = admission(paths, worker, Duration::from_secs(120)).await?;
        let owned = paths.clone();
        let project_scope = project.map(|p| scope(paths, p)).transpose()?;
        let command_scope = project_scope.clone();
        let (engine, args, container) = tokio::task::spawn_blocking(move || {
            crate::packages::agent::command(&owned, command_scope.as_deref())
        })
        .await??;
        let startup = startup_lock(
            &paths.root.join("execution/podman/agent-start.lock"),
            Duration::from_secs(120),
        )
        .await?;
        let mut command = tokio::process::Command::from(engine.command(&args));
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command, false)?;
        let input = child.child().stdin.take().context("runtime stdin")?;
        let output = BufReader::new(child.child().stdout.take().context("runtime stdout")?);
        let mut stderr = child.child().stderr.take().context("runtime stderr")?;
        // Drain upstream diagnostics without exposing private memories or keeping
        // an unbounded transcript. Structured bounded errors reach the caller.
        let stderr = tokio::spawn(async move {
            let mut bytes = [0; 8192];
            while stderr.read(&mut bytes).await.is_ok_and(|n| n != 0) {}
        });
        let mut session = Self {
            admission: Some(permit),
            maintenance: Some(maintenance),
            project_scope,
            project_root: project.map(crate::paths::canonical_project).transpose()?,
            child,
            input: Some(input),
            closed: false,
            output,
            engine,
            container,
            stderr: Some(stderr),
        };
        // Prove loaded identity before any memory or workflow mutation.
        session.call(None, "identity", json!({})).await?;
        drop(startup);
        Ok(session)
    }
    pub async fn call(
        &mut self,
        project: Option<&Path>,
        operation: &str,
        arguments: Value,
    ) -> Result<Value> {
        if let Some(project) = project {
            ensure!(
                Some(crate::paths::canonical_project(project)?) == self.project_root,
                "runtime session project changed"
            );
        }
        let request = json!({"operation":operation,"scope":project.and(self.project_scope.as_ref()),"arguments":arguments});
        let mut bytes = serde_json::to_vec(&request)?;
        ensure!(bytes.len() < MAX as usize, "agent request exceeds 4 MiB");
        bytes.push(b'\n');
        let response = tokio::time::timeout(Duration::from_secs(120), async {
            let input = self.input.as_mut().context("agent runtime is closed")?;
            input.write_all(&bytes).await?;
            input.flush().await?;
            let mut bytes = Vec::new();
            (&mut self.output)
                .take(MAX + 1)
                .read_until(b'\n', &mut bytes)
                .await?;
            ensure!(bytes.len() <= MAX as usize, "agent runtime response exceeds 4 MiB");
            if bytes.last() != Some(&b'\n') {
                let status = tokio::time::timeout(
                    Duration::from_secs(1),
                    self.child.child().wait(),
                ).await;
                let status = match status {
                    Ok(Ok(status)) => status.to_string(),
                    _ => "unavailable".into(),
                };
                anyhow::bail!("agent runtime ended before a complete response (process status: {status}); request was not replayed");
            }
            let response: Value = serde_json::from_slice(&bytes)?;
            ensure!(
                response.get("error").is_none(),
                "agent runtime: {}",
                response["error"].as_str().unwrap_or("invalid result")
            );
            ensure!(
                response["runtime"]["versions"]
                    == crate::packages::agent::definition()?["versions"],
                "agent runtime version drift"
            );
            for (name, contents) in crate::packages::agent::files() {
                if (name.ends_with(".py") && !name.contains('/'))
                    || ["requirements.lock", "model.json", "bm25.json"].contains(&name)
                {
                    ensure!(
                        response["runtime"]["files"][name] == util::sha256(contents),
                        "agent runtime source drift: {name}"
                    );
                }
            }
            ensure!(
                response.get("result").is_some(),
                "agent runtime omitted result"
            );
            Ok::<_, anyhow::Error>(response)
        })
        .await
        .context("agent runtime deadline exceeded; inspect before replaying a mutation")??;
        Ok(response)
    }
    pub async fn close(mut self) -> Result<()> {
        // Tokio's process pipe poll_shutdown is a no-op on Unix; dropping the
        // writer is what delivers EOF to the bounded bridge loop.
        drop(self.input.take());
        let status = tokio::time::timeout(Duration::from_secs(10), self.child.child().wait())
            .await
            .context("agent runtime did not close")??;
        ensure!(status.success(), "agent runtime exited unsuccessfully");
        self.child.finish().await?;
        self.closed = true;
        self.admission.take();
        if let Some(stderr) = self.stderr.take() {
            let _ = stderr.await;
        }
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        if let Some(stderr) = self.stderr.take() {
            stderr.abort();
        }
        let admission = self.admission.take();
        let maintenance = self.maintenance.take();
        let engine = self.engine.clone();
        let name = self.container.clone();
        crate::process::reap_on_drop(move || {
            let _admission = admission;
            let _maintenance = maintenance;
            // The generated name is confined to this owned invocation. Check the
            // recipe label before cleanup so an unrelated replacement is retained.
            if crate::packages::agent::owned_session(&engine, &name).unwrap_or(false) {
                let _ = engine.run(
                    &["rm".into(), "--force".into(), name],
                    Duration::from_secs(10),
                );
            }
        });
    }
}

pub async fn call(
    paths: &Paths,
    project: Option<&Path>,
    operation: &str,
    args: Value,
) -> Result<Value> {
    let mut session = Session::open(paths, project).await?;
    let result = session.call(project, operation, args).await;
    let closed = session.close().await;
    let result = result?;
    closed?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn long_workers_preserve_operation_capacity_and_queue_is_bounded() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().to_path_buf()).unwrap();
        let first = admission(&paths, true, Duration::from_secs(1))
            .await
            .unwrap();
        let _second = admission(&paths, true, Duration::from_secs(1))
            .await
            .unwrap();
        let operation = admission(&paths, false, Duration::from_secs(1))
            .await
            .unwrap();
        assert!(
            admission(&paths, true, Duration::from_millis(60))
                .await
                .is_err()
        );
        assert!(
            admission(&paths, false, Duration::from_millis(60))
                .await
                .is_err()
        );
        drop(first);
        drop(operation);
        assert!(
            admission(&paths, true, Duration::from_secs(1))
                .await
                .is_ok()
        );
        assert!(
            admission(&paths, false, Duration::from_secs(1))
                .await
                .is_ok()
        );
    }

    #[test]
    fn startup_holder_fixture() {
        let Some(path) = std::env::var_os("MIRKET_TEST_STARTUP_LOCK") else {
            return;
        };
        let path = std::path::PathBuf::from(path);
        let _lock = util::FileLock::acquire(&path).unwrap();
        std::fs::write(path.with_extension("ready"), b"locked").unwrap();
        // EOF releases the lock, including when the supervising test fails.
        let _ = std::io::stdin().read_line(&mut String::new());
    }

    #[tokio::test]
    async fn startup_queue_coordinates_processes_and_releases_after_handshake() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let path = root.join("startup.lock");
        let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "agent_runtime::tests::startup_holder_fixture"])
            .env("MIRKET_TEST_STARTUP_LOCK", &path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null());
        let mut holder = OwnedChild::spawn(&mut command, false).unwrap();
        tokio::time::timeout(Duration::from_secs(5), async {
            while !path.with_extension("ready").exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        let error = startup_lock(&path, Duration::from_millis(50))
            .await
            .err()
            .expect("another process owns startup");
        assert!(error.to_string().contains("no request dispatched"));
        // Different installations do not share this queue.
        let other = startup_lock(&root.join("other.lock"), Duration::from_millis(50))
            .await
            .unwrap();
        drop(other);
        let waiting_path = path.clone();
        let waiting =
            tokio::spawn(async move { startup_lock(&waiting_path, Duration::from_secs(5)).await });
        tokio::task::yield_now().await;
        assert!(!waiting.is_finished());
        drop(holder.child().stdin.take());
        assert!(holder.child().wait().await.unwrap().success());
        holder.finish().await.unwrap();
        let handshake = waiting.await.unwrap().unwrap();
        assert!(util::FileLock::acquire(&path).is_err());
        drop(handshake);
        // A completed handshake does not keep startup locked for session life.
        assert!(startup_lock(&path, Duration::from_millis(50)).await.is_ok());
    }

    #[tokio::test]
    async fn cancelling_queued_start_does_not_claim_or_leak_the_lock() {
        let temporary = tempfile::tempdir().unwrap();
        let path = temporary
            .path()
            .canonicalize()
            .unwrap()
            .join("startup.lock");
        let held = util::FileLock::acquire(&path).unwrap();
        let waiting_path = path.clone();
        let waiting =
            tokio::spawn(async move { startup_lock(&waiting_path, Duration::from_secs(5)).await });
        tokio::task::yield_now().await;
        waiting.abort();
        assert!(matches!(waiting.await, Err(error) if error.is_cancelled()));
        drop(held);
        assert!(startup_lock(&path, Duration::from_millis(50)).await.is_ok());
    }

    #[tokio::test]
    async fn startup_queue_does_not_retry_path_errors() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        let error = startup_lock(&root, Duration::from_secs(1))
            .await
            .err()
            .unwrap();
        assert!(error.to_string().contains("startup lock"));
        assert!(!error.to_string().contains("deadline"));
        #[cfg(unix)]
        {
            let link = root.join("linked.lock");
            std::os::unix::fs::symlink(root.join("target"), &link).unwrap();
            assert!(startup_lock(&link, Duration::from_secs(1)).await.is_err());
            assert!(!root.join("target").exists());
        }
    }
}
