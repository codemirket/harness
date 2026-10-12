//! Bounded editor jobs, each with a private working directory and process group.
use super::files::MAX_OUTPUT_BYTES;
use crate::package_mcp::LaunchSpec;
use anyhow::{Context, Result, bail};
use rmcp::{RoleServer, service::RequestContext};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::Child,
    sync::watch,
};

pub const JOB_TIMEOUT: Duration = Duration::from_secs(30);
const LOG_LIMIT: u64 = 128 * 1024;

pub struct Control<'a> {
    pub request: Option<&'a RequestContext<RoleServer>>,
    pub shutdown: watch::Receiver<bool>,
    pub timeout: Duration,
}
impl Control<'_> {
    pub fn check(&self) -> Result<()> {
        if *self.shutdown.borrow() || self.request.is_some_and(|r| r.ct.is_cancelled()) {
            bail!("editor job cancelled; staged output was not published");
        }
        Ok(())
    }
    async fn cancelled(&self) {
        let mut shutdown = self.shutdown.clone();
        let host = async {
            if let Some(request) = self.request {
                request.ct.cancelled().await;
            } else {
                std::future::pending::<()>().await;
            }
        };
        tokio::select! { _=host=>{}, _=async { let _=shutdown.wait_for(|s|*s).await; }=>{} }
    }
}

struct Job {
    child: Child,
    #[cfg(unix)]
    group: Option<u32>,
}
impl Job {
    // A fixed platform utility targets only the process group we just created.
    // No shell or caller-selected command/arguments enter this operation.
    fn kill_group(&mut self) {
        #[cfg(unix)]
        if let Some(pid) = self.group.take() {
            let _ = std::process::Command::new("/bin/kill")
                .env_clear()
                .args(["-KILL", "--", &format!("-{pid}")])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = self.child.start_kill();
    }
    async fn finish(&mut self) -> Result<()> {
        #[cfg(unix)]
        let group = self.group;
        // Native browser drivers own detached browser groups. Give their
        // registered TERM/exit handlers a bounded opportunity to reap those
        // groups before forcibly terminating the launcher. SIGKILL alone
        // bypasses that cleanup and can leave a browser writing into staging.
        #[cfg(unix)]
        if self.child.try_wait()?.is_none() {
            if let Some(group) = group {
                let _ = std::process::Command::new("/bin/kill")
                    .env_clear()
                    .args(["-TERM", "--", &format!("-{group}")])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            if tokio::time::timeout(Duration::from_secs(1), self.child.wait())
                .await
                .is_err()
            {
                // The pinned Playwright driver forces its captured browser
                // group on a repeated TERM while graceful close is pending.
                // Preserve the total two-second grace before launcher KILL.
                if let Some(group) = group {
                    let _ = std::process::Command::new("/bin/kill")
                        .env_clear()
                        .args(["-TERM", "--", &format!("-{group}")])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                }
                let _ = tokio::time::timeout(Duration::from_secs(1), self.child.wait()).await;
            }
        }
        self.kill_group();
        tokio::time::timeout(Duration::from_secs(3), self.child.wait())
            .await
            .context("editor process did not exit during cleanup")??;
        // A reaped launcher does not mean its descendants have finished exiting.
        // Wait before the caller removes their shared workspace: a late native
        // filesystem operation can otherwise race recursive directory removal.
        #[cfg(unix)]
        if let Some(group) = group {
            tokio::time::timeout(Duration::from_secs(3), async {
                loop {
                    let exists = tokio::process::Command::new("/bin/kill")
                        .env_clear()
                        .args(["-0", "--", &format!("-{group}")])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .kill_on_drop(true)
                        .status()
                        .await?;
                    if !exists.success() {
                        return Ok::<(), std::io::Error>(());
                    }
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            })
            .await
            .context("editor descendants did not exit during cleanup")??;
        }
        Ok(())
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.kill_group();
    }
}

async fn bounded_read(mut stream: impl AsyncRead + Unpin) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    (&mut stream)
        .take(LOG_LIMIT + 1)
        .read_to_end(&mut bytes)
        .await?;
    if bytes.len() as u64 > LOG_LIMIT {
        bail!("editor diagnostic output exceeds 128 KiB");
    }
    Ok(bytes)
}

async fn output_limit(path: Option<&Path>) -> Result<()> {
    loop {
        if let Some(path) = path {
            match std::fs::symlink_metadata(path) {
                Ok(metadata) if !metadata.is_file() || metadata.len() > MAX_OUTPUT_BYTES => {
                    bail!("editor output is not a regular file or exceeds 32 MiB")
                }
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

pub async fn run(
    spec: &LaunchSpec,
    args: &[String],
    workspace: &Path,
    output: Option<&Path>,
    control: &Control<'_>,
) -> Result<Vec<u8>> {
    if let Some(runtime) = &spec.container {
        let deadline = std::time::Instant::now() + control.timeout;
        let check = || {
            control.check()?;
            if std::time::Instant::now() >= deadline {
                bail!("container job deadline exceeded; no output published");
            }
            Ok(())
        };
        // One budget covers queueing, startup, input staging, execution and output
        // retrieval. Dropping a job also evicts its remote worker, not just a client.
        let operation = async {
            let _permit = runtime
                .gate
                .acquire()
                .await
                .context("container session closed")?;
            check()?;
            let mut job = runtime.job(workspace, args, &check).await?;
            let result = async {
                check()?;
                let stdout = job.execute().await?;
                check()?;
                job.collect(workspace, output).await?;
                check()?;
                Ok(stdout)
            }
            .await;
            if result.is_ok() {
                job.complete();
            } else {
                job.failed().await?;
            }
            result
        };
        return tokio::select! {
            result=operation=>result,
            _=control.cancelled()=>Err(anyhow::anyhow!("container job cancelled; no output published")),
            _=tokio::time::sleep(control.timeout)=>Err(anyhow::anyhow!("container exceeded its {} second deadline; no output published",control.timeout.as_secs())),
        };
    }
    run_process(spec, args, workspace, output, control, true).await
}

async fn run_process(
    spec: &LaunchSpec,
    args: &[String],
    workspace: &Path,
    output: Option<&Path>,
    control: &Control<'_>,
    private_home: bool,
) -> Result<Vec<u8>> {
    control.check()?;
    let home = workspace.join("home");
    std::fs::create_dir_all(&home)?;
    let mut command = tokio::process::Command::new(&spec.command);
    command
        .args(&spec.args)
        .args(args)
        .env_clear()
        .envs(&spec.env);
    if private_home {
        command
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("XDG_CONFIG_HOME", home.join("config"))
            .env("XDG_CACHE_HOME", home.join("cache"))
            .env("XDG_DATA_HOME", home.join("data"))
            .env("INKSCAPE_PROFILE_DIR", home.join("inkscape"))
            .env("PYTHONNOUSERSITE", "1")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .env("PYTHONSAFEPATH", "1")
            .env(
                "PATH",
                if cfg!(windows) {
                    ""
                } else {
                    "/usr/bin:/bin:/usr/sbin:/sbin"
                },
            )
            .env("TMPDIR", workspace)
            .env("TMP", workspace)
            .env("TEMP", workspace);
    }
    command
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let child = command
        .spawn()
        .context("start verified editor executable")?;
    let mut job = Job {
        #[cfg(unix)]
        group: child.id(),
        child,
    };
    let mut stdout = job.child.stdout.take().context("editor stdout pipe")?;
    let mut stderr = job.child.stderr.take().context("editor stderr pipe")?;
    let operation = async {
        let (status, stdout, stderr) = tokio::try_join!(
            async { job.child.wait().await.map_err(anyhow::Error::from) },
            bounded_read(&mut stdout),
            bounded_read(&mut stderr)
        )?;
        if !status.success() {
            let diagnostic = String::from_utf8_lossy(&stderr);
            bail!(
                "editor failed ({status}): {}",
                diagnostic.chars().take(2000).collect::<String>()
            );
        }
        // stdout is editor data, never copied into the MCP transport directly.
        Ok(stdout)
    };
    let result = tokio::select! {
        result=operation=>result,
        _=control.cancelled()=>Err(anyhow::anyhow!("editor job cancelled; no output published")),
        _=tokio::time::sleep(control.timeout)=>Err(anyhow::anyhow!("editor exceeded its {} second deadline; no output published",control.timeout.as_secs())),
        result=output_limit(output)=>result.and_then(|_|Err(anyhow::anyhow!("editor output monitor stopped"))),
    };
    // Keep the pipes open when cancellation drops the operation future, and
    // drain bounded diagnostics while TERM handlers run. Closing them first
    // can deliver SIGPIPE and bypass the launcher's browser cleanup entirely.
    let (cleanup, _) = tokio::join!(
        job.finish(),
        tokio::time::timeout(Duration::from_secs(3), async {
            let _ = tokio::try_join!(bounded_read(&mut stdout), bounded_read(&mut stderr));
        })
    );
    cleanup?;
    control.check()?;
    result
}

pub fn output(path: &Path) -> Result<Vec<u8>> {
    crate::util::read_bounded(path,MAX_OUTPUT_BYTES)
        .context("editor did not create a regular bounded output; a successful process exit alone is insufficient")
}

/// Validate framing, dimensions and complete zlib scanlines, not just a PNG suffix.
pub fn png(bytes: &[u8], expected_width: Option<u32>) -> Result<(u32, u32)> {
    use std::io::Read;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        bail!("editor output is not a PNG");
    }
    let mut index = 8usize;
    let mut header = None;
    let mut compressed = Vec::new();
    let mut ended = false;
    while index + 12 <= bytes.len() {
        let length = u32::from_be_bytes(bytes[index..index + 4].try_into()?) as usize;
        let end = index
            .checked_add(12)
            .and_then(|v| v.checked_add(length))
            .context("PNG size overflow")?;
        if end > bytes.len() {
            bail!("truncated PNG chunk");
        }
        let kind = &bytes[index + 4..index + 8];
        let data = &bytes[index + 8..index + 8 + length];
        let mut crc = flate2::Crc::new();
        crc.update(&bytes[index + 4..index + 8 + length]);
        if crc.sum() != u32::from_be_bytes(bytes[index + 8 + length..end].try_into()?) {
            bail!("PNG chunk checksum is invalid");
        }
        if header.is_none() && kind != b"IHDR" {
            bail!("PNG lacks its initial header");
        }
        match kind {
            b"IHDR" => {
                if header.is_some() || length != 13 {
                    bail!("invalid PNG header");
                }
                let w = u32::from_be_bytes(data[0..4].try_into()?);
                let h = u32::from_be_bytes(data[4..8].try_into()?);
                if w == 0
                    || h == 0
                    || w > 8192
                    || h > 8192
                    || u64::from(w) * u64::from(h) > 16_777_216
                    || data[8] != 8
                    || !matches!(data[9], 2 | 6)
                    || data[10..13] != [0, 0, 0]
                {
                    bail!(
                        "unsupported PNG dimensions or encoding; expected bounded 8-bit RGB/RGBA without interlacing"
                    );
                }
                header = Some((w, h, if data[9] == 6 { 4usize } else { 3 }));
            }
            b"IDAT" => compressed.extend_from_slice(data),
            b"IEND" => {
                if length != 0 || end != bytes.len() {
                    bail!("invalid PNG end");
                }
                ended = true;
                break;
            }
            _ => {}
        }
        index = end;
    }
    let (w, h, channels) = header.context("missing PNG header")?;
    if !ended || compressed.is_empty() || expected_width.is_some_and(|width| width != w) {
        bail!("PNG is incomplete or has unexpected dimensions");
    }
    let stride = w as usize * channels + 1;
    let expected = stride * h as usize;
    let mut decoded = Vec::new();
    flate2::read::ZlibDecoder::new(compressed.as_slice())
        .take(expected as u64 + 1)
        .read_to_end(&mut decoded)?;
    if decoded.len() != expected || decoded.chunks_exact(stride).any(|row| row[0] > 4) {
        bail!("PNG has invalid or truncated pixel data");
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_runs_launcher_cleanup_before_forcing_its_group() {
        let workspace = tempfile::tempdir().unwrap();
        let spec = LaunchSpec {
            command: "/bin/sh".into(),
            args: vec![
                "-c".into(),
                "trap 'printf cleaned > cleanup.txt; exit 0' TERM; printf ready > ready.txt; while :; do sleep 0.03; done".into(),
            ],
            env: Default::default(),
            cwd: workspace.path().to_owned(),
            allowed_tools: None,
            denied_tools: Default::default(),
            input_policy: "none".into(),
            container: None,
            startup_timeout: Duration::from_secs(3),
        };
        let (stop, shutdown) = watch::channel(false);
        let cancel = async {
            tokio::time::timeout(Duration::from_secs(3), async {
                while !workspace.path().join("ready.txt").exists() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap();
            stop.send(true).unwrap();
        };
        let control = Control {
            request: None,
            shutdown,
            timeout: Duration::from_secs(5),
        };
        let operation = run(&spec, &[], workspace.path(), None, &control);
        let (result, ()) = tokio::join!(operation, cancel);
        assert!(result.unwrap_err().to_string().contains("cancelled"));
        assert_eq!(
            std::fs::read(workspace.path().join("cleanup.txt")).unwrap(),
            b"cleaned"
        );
        workspace.close().unwrap();
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn completed_launcher_reaps_descendants_before_workspace_cleanup() {
        let workspace = tempfile::tempdir().unwrap();
        let spec = LaunchSpec {
            command: "/bin/sh".into(),
            args: vec![
                "-c".into(),
                "sleep 60 >/dev/null 2>&1 & echo $! > descendant.pid; printf done".into(),
            ],
            env: Default::default(),
            cwd: workspace.path().to_owned(),
            allowed_tools: None,
            denied_tools: Default::default(),
            input_policy: "none".into(),
            container: None,
            startup_timeout: Duration::from_secs(3),
        };
        let (_stop, shutdown) = watch::channel(false);
        let result = run(
            &spec,
            &[],
            workspace.path(),
            None,
            &Control {
                request: None,
                shutdown,
                timeout: Duration::from_secs(3),
            },
        )
        .await
        .unwrap();
        assert_eq!(result, b"done");
        let pid = std::fs::read_to_string(workspace.path().join("descendant.pid")).unwrap();
        // No polling here: cleanup must be complete when run() returns, even
        // when the launcher exits normally while its background child is alive.
        assert!(
            !tokio::process::Command::new("/bin/kill")
                .args(["-0", pid.trim()])
                .output()
                .await
                .unwrap()
                .status
                .success()
        );
        workspace.close().unwrap();
    }
    #[test]
    fn success_without_a_usable_artifact_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        assert!(output(&temp.path().join("missing.png")).is_err());
        assert!(png(b"\x89PNG\r\n\x1a\n", None).is_err());
        assert!(png(b"success", None).is_err());
    }
}
