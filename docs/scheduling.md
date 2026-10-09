# Optional legacy maintenance

Modern `ai.py install` and `ai.py check` neither register nor verify schedules.
Maintenance is disabled by default in `registry/harness.json`; deliberately enable
`maintenance.enabled` before opting into registration or execution. A disabled
policy blocks registration and maintenance runs. It does not unregister an
existing OS job; inspect and remove an unwanted job deliberately.

The retained legacy job runs at **12:00 AM (00:00) every day in the device's local
timezone**. It refreshes this repository and reconciles Codex/Claude guidance,
skills and portable Codex preferences. It does **not** manage the new target
settings or neutral MCP registry. Use target `install` and `check`
for those. The OS job does not open an AI session.

| Device | Scheduler | Timing and conditions |
| --- | --- | --- |
| macOS/Linux | Current user’s crontab, `0 0 * * *` | Runs at local midnight when the machine and cron service are available. Missed runs are not caught up. |
| Windows 11 | Current-user Task Scheduler task | Daily local midnight, least privilege, interactive user session, `StartWhenAvailable` catch-up. No stored password or elevation. |

The policy is declared in [harness.json](../registry/harness.json): `maintenance.enabled` (false by default), time `00:00`, timezone `system`, remote `origin`, branch `main`, expected repository identity `github.com/codemirket/harness`. The supported contract is daily local midnight; do not treat these fields as a general scheduling language.

## Register and inspect

After reviewing and explicitly enabling the maintenance policy:

```sh
python3 ai.py schedule plan
python3 ai.py schedule install
python3 ai.py schedule check
```

Use `py -3` on Windows. These commands manage only the current OS user's owned
job. Registration requires a valid Git checkout and enabled policy. It can be
performed while Codex is open or the repository has local edits. Registration
checks the stored scheduler contract, not whether a run succeeds.

The historical complete installer remains available as `ai.py legacy-install`
for compatibility. Its broader runtime, preferences and scheduling behavior is
separate from modern target installation.

An existing exact legacy midnight installer line is replaced by the owned maintenance entry. Other cron jobs and tasks remain untouched. Modified legacy lines, malformed ownership markers, a conflicting Windows task, or an existing `CRON_TZ` override require review. Scheduler changes are backed up locally before replacement and checked afterward; concurrent edits are not silently overwritten.

Keep both the checkout and Python executable at stable paths. Re-run schedule registration after moving either. Registration confirms the stored scheduler contract, not that the OS has successfully launched it.

## What a run does

```sh
python3 ai.py maintenance run
python3 ai.py maintenance status
```

The updater checks repository identity, the `main` branch and its `origin/main` upstream, a clean index/worktree and unfinished Git operations. It fetches noninteractively with bounded execution, permits only a fast-forward from the expected origin, checks the resulting checkout, and launches the updated synchronization entry point in a fresh process. This avoids continuing with Python modules imported before the update.

Local changes, untracked files, local-ahead or divergent history, an unexpected remote, and active Git operations stop the run. The updater does not stash, reset, commit, push or resolve conflicts automatically. Resolve the reported condition deliberately, then run maintenance again; the daily job remains registered. Uncommitted work in this checkout is a temporary reason to skip an update, not a reason to discard the work.

The refreshed process reconciles global registrations for Codex and the Claude Code CLI, preserving unmanaged or modified content. It also applies the allowlisted [portable preferences](settings.md). It does not synchronize every project, update catalog pins independently, install runtime dependencies, connect plugin accounts or invoke a model.

A process lock prevents overlapping maintenance runs. Git and child-process output is bounded, and durable reports omit raw remote URLs, credentials and Git error output. A failed fetch or synchronization remains a failed/partial run; it is not reported as ready merely because the scheduler launched.

## Preferences while Codex is open

Global instructions and skills can refresh while Codex is open. If portable preferences already match, no settings write is needed. If changes are needed, maintenance leaves the live configuration alone and reports deferred work. It never forces Codex to close.

The next eligible daily run retries. To finish sooner, close Codex desktop and other active Codex clients, then run:

```sh
python3 ai.py maintenance sync
```

This applies the current checkout without another fetch; review the resulting report and reopen Codex after settings change. A deferred settings report is not completed preference synchronization. App theme rendering, model access and plugin execution still need confirmation on the destination device.

## Last-run evidence and recovery

- `~/.agent-harness/maintenance-status.json` records the last reported run.
- `~/.agent-harness/logs/maintenance.log` contains bounded, rotated maintenance logs.
- `python3 ai.py maintenance status` reads the saved status; inspect its timestamp and outcome.
- `python3 ai.py schedule check` verifies registration, not the outcome of the last run.

Cron stdout/stderr is suppressed to avoid sending external mail. If Python or the entry point cannot start, the updater cannot write a new status; an absent or stale timestamp is not success. Perform a manual `maintenance run` when validating installation, and inspect the native scheduler if expected runs stop appearing.

macOS privacy controls may prevent cron from accessing a checkout in Documents. If a run reports access errors, inspect that device’s privacy permissions or choose an accessible stable checkout location. Registration does not grant Full Disk Access. A sleeping/offline Mac can miss midnight; cron does not wake it or catch up automatically. Run maintenance manually after a missed run when needed.

Windows requests catch-up when available and runs only in the current user’s logged-in session. Sleep, login state, executable availability and scheduler policy affect delivery. Native Windows scheduling has not been verified by macOS fixture tests; verify registration and one real run on the Windows device.

For failed or partial synchronization, preserve local changes and retained backups, resolve the stated cause, and retry. Do not delete user files, settings backups or locks merely to force a successful status. See [setup ownership and recovery](../setup/README.md#ownership-and-recovery).
