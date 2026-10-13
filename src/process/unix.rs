//! Preserve terminal job-control ownership while a foreground tool uses its own
//! process group. The CLI is the sole owner of this handoff; no editor uses it.
use anyhow::{Context, Result};
use std::io::IsTerminal;
unsafe extern "C" {
    fn tcgetpgrp(fd: i32) -> i32;
    fn tcsetpgrp(fd: i32, group: i32) -> i32;
    fn getpgrp() -> i32;
    fn signal(number: i32, handler: usize) -> usize;
}
const SIGTTOU: i32 = 22; // macOS and Linux, the supported Unix targets
pub(super) struct Terminal {
    group: i32,
    handler: usize,
}
impl Terminal {
    pub(super) fn handoff(group: u32) -> Result<Option<Self>> {
        if !std::io::stdin().is_terminal() {
            return Ok(None);
        }
        // Only hand off a terminal actually owned by this foreground CLI.
        let previous = unsafe { tcgetpgrp(0) };
        if previous < 0 || previous != unsafe { getpgrp() } {
            return Ok(None);
        }
        // The parent becomes a background group during the handoff. Ignore
        // SIGTTOU until restoration, so it can safely reclaim its terminal.
        let handler = unsafe { signal(SIGTTOU, 1) }; // SIG_IGN
        if handler == usize::MAX {
            return Err(std::io::Error::last_os_error())
                .context("preserve terminal signal handler");
        }
        let guard = Self {
            group: previous,
            handler,
        };
        if unsafe { tcsetpgrp(0, group as i32) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("give tool foreground terminal ownership");
        }
        // A very fast read may have received SIGTTIN before the handoff.
        super::signal_group(group, "-CONT")?;
        Ok(Some(guard))
    }
}
impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            tcsetpgrp(0, self.group);
            signal(SIGTTOU, self.handler);
        }
    }
}

pub(super) fn group_active(group: u32) -> Result<bool> {
    if !super::group_signal_status(group, "-0")? {
        return Ok(false);
    }
    #[cfg(target_os = "linux")]
    {
        // Container PID 1 may leave killed orphan zombies indefinitely. Zombies
        // cannot write or retain pipe FDs; don't mistake them for live workers.
        let mut saw_member = false;
        for entry in std::fs::read_dir("/proc")? {
            let entry = entry?;
            if !entry
                .file_name()
                .to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_digit())
            {
                continue;
            }
            let stat = match std::fs::read_to_string(entry.path().join("stat")) {
                Ok(stat) => stat,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            let Some((_, fields)) = stat.rsplit_once(") ") else {
                anyhow::bail!("invalid process status during tool cleanup");
            };
            let mut fields = fields.split_whitespace();
            let state = fields.next();
            let _parent = fields.next();
            if fields.next().and_then(|v| v.parse::<u32>().ok()) == Some(group) {
                saw_member = true;
                if state != Some("Z") && state != Some("X") {
                    return Ok(true);
                }
            }
        }
        // If processes disappeared during the snapshot, a fresh signal probe
        // confirms disappearance on the next poll rather than guessing success.
        if saw_member {
            return Ok(false);
        }
    }
    Ok(true)
}
