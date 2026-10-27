//! Shared finite-child ownership and bounded diagnostic streams.
//! Mirket owns only the selected child and confirms reaping before releasing admission.

use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt};
use tokio::process::Child;

const MAX_FRAME_BYTES: usize = 65_536;
const MAX_STDERR_BYTES: usize = 16_384;
const REAP_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct AdapterError {
    pub message: String,
    /// False requires retained admission/ownership for reconciliation. True does
    /// not mean that the private stage is absent or that an effect rolled back.
    pub cleanup_confirmed: bool,
}
impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for AdapterError {}
impl From<String> for AdapterError {
    fn from(message: String) -> Self {
        Self {
            message,
            cleanup_confirmed: true,
        }
    }
}
impl From<&str> for AdapterError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
pub(crate) fn validate_job_dir(path: &Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| "private adapter job directory is unavailable")?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || fs::canonicalize(path).map_err(|_| "job directory cannot be resolved")? != path
    {
        return Err("adapter requires a canonical private job directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("adapter job directory must exclude group/other access".into());
        }
    }
    Ok(())
}

pub(crate) async fn read_frame<R: AsyncBufRead + Unpin>(reader: &mut R) -> Result<Vec<u8>, String> {
    let mut frame = Vec::new();
    loop {
        let buffer = reader
            .fill_buf()
            .await
            .map_err(|_| "adapter stdout read failed")?;
        if buffer.is_empty() {
            return Err("adapter stdout ended before a complete frame".into());
        }
        let end = buffer.iter().position(|byte| *byte == b'\n');
        let consumed = end.map_or(buffer.len(), |index| index + 1);
        if frame.len() + consumed > MAX_FRAME_BYTES {
            return Err("adapter stdout frame exceeds its byte limit".into());
        }
        frame.extend_from_slice(&buffer[..consumed]);
        reader.consume(consumed);
        if end.is_some() {
            return Ok(frame);
        }
    }
}

pub(crate) async fn drain_stderr<R: AsyncRead + Unpin>(mut reader: R) -> Result<usize, String> {
    let mut buffer = [0u8; 2048];
    let mut total = 0;
    loop {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(|_| "adapter stderr read failed")?;
        if count == 0 {
            return Ok(total);
        }
        total += count;
        if total > MAX_STDERR_BYTES {
            return Err("adapter stderr exceeds its byte limit".into());
        }
    }
}

pub(crate) async fn cancelled(cancel: Arc<AtomicBool>) {
    while !cancel.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

pub(crate) struct OwnedChild {
    pub(crate) child: Child,
    pid: u32,
    reaped: bool,
    identity_held: bool,
}

impl OwnedChild {
    pub(crate) fn is_reaped(&self) -> bool {
        self.reaped
    }

    pub(crate) fn new(child: Child) -> Result<Self, String> {
        let pid = child.id().ok_or("adapter child identity is unavailable")?;
        Ok(Self {
            child,
            pid,
            reaped: false,
            identity_held: true,
        })
    }

    pub(crate) async fn wait(&mut self) -> Result<std::process::ExitStatus, String> {
        match self.child.wait().await {
            Ok(status) => {
                self.reaped = true;
                self.identity_held = false;
                Ok(status)
            }
            Err(_) => {
                self.identity_held = false;
                Err("child exit identity could not be established".into())
            }
        }
    }

    fn kill_owned_group(&mut self) -> Result<(), String> {
        if self.reaped {
            return Ok(());
        }
        if !self.identity_held {
            return Err("child identity is no longer proven; do not signal a numeric PID".into());
        }
        #[cfg(unix)]
        {
            let pid: i32 = self
                .pid
                .try_into()
                .map_err(|_| "adapter child identity is invalid")?;
            if pid <= 0 {
                return Err("adapter child identity is invalid".into());
            }
            // SAFETY: the child was launched into a fresh group with PGID=PID.
            // We retain its unreaped Child, so its PID cannot be recycled. The
            // negative identifier targets only that owned process group.
            if unsafe { libc::kill(-pid, libc::SIGKILL) } == -1 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::EPERM) {
                    // Darwin reports EPERM for a group containing only its
                    // exited, unreaped leader. Reap that direct child first,
                    // then confirm the group disappeared. A live descendant
                    // or an uncertain group still retains the admission.
                    if self.child.try_wait().ok().flatten().is_some() {
                        self.reaped = true;
                        self.identity_held = false;
                        if unsafe { libc::kill(-pid, 0) } == -1
                            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                        {
                            return Ok(());
                        }
                        return Err("owned process group remains after direct-child reap".into());
                    }
                }
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(format!("owned process-group termination failed: {error}"));
                }
            }
        }
        match self.child.start_kill() {
            Ok(()) => Ok(()),
            Err(error) => {
                // A finite leaf can exit between the group signal and the
                // direct-child signal. Reap proof, rather than the signal's
                // return value, decides whether ownership is complete.
                match self.child.try_wait() {
                    Ok(Some(_)) => {
                        self.reaped = true;
                        self.identity_held = false;
                        Ok(())
                    }
                    _ => Err(format!("adapter termination failed: {error}")),
                }
            }
        }
    }

    pub(crate) async fn terminate_and_reap(&mut self) -> Result<(), String> {
        self.kill_owned_group()?;
        if self.reaped {
            return Ok(());
        }
        match tokio::time::timeout(REAP_TIMEOUT, self.child.wait()).await {
            Ok(Ok(_)) => (),
            Ok(Err(_)) => {
                self.identity_held = false;
                return Err("adapter reap failed".into());
            }
            Err(_) => return Err("adapter reap deadline exceeded".into()),
        }
        self.reaped = true;
        self.identity_held = false;
        Ok(())
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.reaped && self.identity_held {
            let _ = self.kill_owned_group();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn shared_streams_enforce_complete_bounded_frames_and_stderr() {
        assert_eq!(
            read_frame(&mut BufReader::new(&b"{}\n"[..])).await.unwrap(),
            b"{}\n"
        );
        assert!(read_frame(&mut BufReader::new(&b"{}"[..])).await.is_err());
        let too_large = vec![b'x'; MAX_FRAME_BYTES + 1];
        assert!(
            read_frame(&mut BufReader::new(&too_large[..]))
                .await
                .is_err()
        );
        assert_eq!(drain_stderr(&b"safe"[..]).await.unwrap(), 4);
        assert!(
            drain_stderr(&vec![b'x'; MAX_STDERR_BYTES + 1][..])
                .await
                .is_err()
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn already_exited_owned_child_can_be_reaped_after_group_cleanup() {
        use std::os::unix::process::CommandExt;
        let mut command = tokio::process::Command::new("/usr/bin/true");
        command.as_std_mut().process_group(0);
        let mut child = OwnedChild::new(command.spawn().unwrap()).unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        child.terminate_and_reap().await.unwrap();
        assert!(child.is_reaped());
    }
}
