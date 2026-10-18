//! Narrow process ownership mechanics; callers retain their own execution policies.
#[cfg(unix)]
use anyhow::bail;
use anyhow::{Context, Result};
#[cfg(unix)]
use std::process::Stdio;
use std::{pin::Pin, time::Duration};

/// Normal runtime drop waits for blocking work; cancellation must not block
/// an async worker or abandon the owned child in a detached async task.
pub(crate) fn reap_on_drop(cleanup: impl FnOnce() + Send + 'static) {
    // A runtime may cancel a queued blocking task before it starts during
    // shutdown. The closure's guard still runs cleanup in that case.
    struct Reap<F: FnOnce()>(Option<F>);
    impl<F: FnOnce()> Drop for Reap<F> {
        fn drop(&mut self) {
            if let Some(cleanup) = self.0.take() {
                cleanup();
            }
        }
    }
    let cleanup = Reap(Some(cleanup));
    if let Ok(runtime) = tokio::runtime::Handle::try_current() {
        runtime.spawn_blocking(move || drop(cleanup));
    } else {
        drop(cleanup);
    }
}

#[cfg(unix)]
pub(crate) fn signal_group(group: u32, signal: &'static str) -> Result<()> {
    group_signal_status(group, signal).map(|_| ())
}

#[cfg(unix)]
fn group_signal_status(group: u32, signal: &'static str) -> Result<bool> {
    // Only the process group created by Command::process_group(0) is targeted.
    // Both signal choices and the system utility are fixed by this module.
    let mut utility = std::process::Command::new("/bin/kill")
        .env_clear()
        .args([signal, "--", &format!("-{group}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("cannot signal the owned process group")?;
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    loop {
        if let Some(status) = utility.try_wait()? {
            // A nonzero status also occurs when the complete group already exited.
            return Ok(status.success());
        }
        if std::time::Instant::now() >= deadline {
            let _ = utility.kill();
            let _ = utility.wait();
            bail!("owned process-group signal utility exceeded its deadline");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

pub(crate) type ShutdownSignal = Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;

pub(crate) fn shutdown_signal() -> Result<ShutdownSignal> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .context("cannot listen for process termination")?;
        Ok(Box::pin(async move {
            tokio::select! {
                _ = terminate.recv() => Ok(()),
                result = tokio::signal::ctrl_c() => result.context("cannot observe process interrupt"),
            }
        }))
    }
    #[cfg(not(unix))]
    {
        Ok(Box::pin(async {
            tokio::signal::ctrl_c()
                .await
                .context("cannot observe process interrupt")
        }))
    }
}

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// Lifecycle ownership only. Environment, permissions and I/O belong to callers.
/// Unix groups are cooperative ownership: descendants can deliberately escape by
/// creating a different session/group. This is not a sandbox or effect rollback.
pub(crate) struct OwnedChild {
    child: Option<tokio::process::Child>,
    finished: bool,
    #[cfg(unix)]
    group: Option<u32>,
    #[cfg(unix)]
    terminal: Option<unix::Terminal>,
    #[cfg(windows)]
    job: Option<windows::Job>,
}
impl OwnedChild {
    pub(crate) fn child(&mut self) -> &mut tokio::process::Child {
        self.child.as_mut().expect("live owned child")
    }
    pub(crate) fn spawn(
        command: &mut tokio::process::Command,
        inherit_terminal: bool,
    ) -> Result<Self> {
        command.kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        #[cfg(windows)]
        let job = {
            command.creation_flags(0x00000004); // CREATE_SUSPENDED
            windows::Job::new()?
        };
        let child = command.spawn().context("start owned tool process")?;
        #[allow(unused_mut)]
        let mut owned = Self {
            #[cfg(unix)]
            group: child.id(),
            #[cfg(unix)]
            terminal: None,
            child: Some(child),
            finished: false,
            #[cfg(windows)]
            job: Some(job),
        };
        #[cfg(windows)]
        {
            let _ = inherit_terminal;
            owned
                .job
                .as_ref()
                .expect("owned job")
                .attach_and_resume(owned.child.as_ref().expect("owned child"))?;
        }
        #[cfg(unix)]
        if inherit_terminal {
            owned.terminal =
                unix::Terminal::handoff(owned.group.context("tool process group unavailable")?)?;
        }
        Ok(owned)
    }
    fn force(&mut self) -> Result<()> {
        #[cfg(unix)]
        if let Some(group) = self.group {
            signal_group(group, "-KILL")?;
        }
        #[cfg(windows)]
        self.job.as_ref().expect("owned job").terminate()?;
        let _ = self.child().start_kill();
        Ok(())
    }
    pub(crate) async fn finish(&mut self) -> Result<()> {
        // Kill remaining owned descendants even after a successful launcher exit.
        // Registered CLI jobs are foreground operations, not daemon launchers.
        self.force()?;
        tokio::time::timeout(Duration::from_secs(3), self.child().wait())
            .await
            .context("tool launcher did not exit during cleanup")??;
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                #[cfg(unix)]
                let active = if let Some(group) = self.group {
                    unix::group_active(group)?
                } else {
                    false
                };
                #[cfg(windows)]
                let active = self.job.as_ref().expect("owned job").active()?;
                if !active {
                    return Ok::<(), anyhow::Error>(());
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .context("owned tool descendants did not exit during cleanup")??;
        self.finished = true;
        #[cfg(unix)]
        {
            self.group = None;
        }
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let Some(mut child) = self.child.take() else {
            return;
        };
        #[cfg(unix)]
        let group = self.group.take();
        #[cfg(unix)]
        let terminal = self.terminal.take();
        #[cfg(windows)]
        let job = self.job.take().expect("owned job");
        let _ = child.start_kill();
        reap_on_drop(move || {
            #[cfg(unix)]
            if let Some(group) = group {
                let _ = signal_group(group, "-KILL");
            }
            #[cfg(windows)]
            let _ = job.terminate();
            let _ = child.start_kill();
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            while std::time::Instant::now() < deadline {
                let reaped = matches!(child.try_wait(), Ok(Some(_)));
                #[cfg(unix)]
                let active = group.map_or(Ok(false), unix::group_active);
                #[cfg(windows)]
                let active = job.active();
                if reaped && matches!(active, Ok(false)) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            #[cfg(unix)]
            drop(terminal);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn dropping_native_child_defers_group_reaping_off_runtime_worker() {
        use tokio::io::AsyncBufReadExt;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .max_blocking_threads(1)
            .build()
            .unwrap();
        let (started, ready) = std::sync::mpsc::channel();
        let (release, blocked) = std::sync::mpsc::channel();
        runtime.spawn_blocking(move || {
            let _ = started.send(());
            let _ = blocked.recv();
        });
        ready.recv_timeout(Duration::from_secs(3)).unwrap();
        runtime.block_on(async {
            let mut command = tokio::process::Command::new("/bin/sh");
            command
                .args(["-c", "sleep 30 & echo ready; wait"])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            let mut child = OwnedChild::spawn(&mut command, false).unwrap();
            let group = child.group.unwrap();
            let mut lines = tokio::io::BufReader::new(child.child().stdout.take().unwrap()).lines();
            assert_eq!(lines.next_line().await.unwrap().as_deref(), Some("ready"));
            drop(child);
            tokio::task::yield_now().await;
            // The only blocking worker is occupied: the descendant still exists,
            // proving Drop returned without performing the group cleanup itself.
            assert!(unix::group_active(group).unwrap());
            release.send(()).unwrap();
            tokio::time::timeout(Duration::from_secs(5), async {
                while unix::group_active(group).unwrap() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .unwrap();
        });
    }

    #[test]
    fn queued_reaper_runs_even_when_runtime_cancels_its_blocking_task() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .max_blocking_threads(1)
            .build()
            .unwrap();
        let (started, ready) = std::sync::mpsc::channel();
        let (release, blocked) = std::sync::mpsc::channel();
        runtime.spawn_blocking(move || {
            started.send(()).unwrap();
            blocked.recv().unwrap();
        });
        ready.recv_timeout(Duration::from_secs(3)).unwrap();
        let (cleaned, observed) = std::sync::mpsc::channel();
        runtime.block_on(async {
            reap_on_drop(move || {
                let _ = cleaned.send(());
            });
        });
        // The only blocking worker is occupied, so this reaper cannot start
        // before shutdown marks queued blocking tasks for cancellation.
        runtime.shutdown_background();
        release.send(()).unwrap();
        observed.recv_timeout(Duration::from_secs(3)).unwrap();
    }
}
