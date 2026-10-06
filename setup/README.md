# Install and reconcile the personal harness

Codex desktop is the primary client. Codex CLI supports diagnostics and automation; Claude Code CLI supplies selective second opinions. Claude desktop is unsupported. Keep this repository in a stable directory with Python 3.9+; no Python packages are required.

## Complete installation

```sh
python3 ai.py install --dry-run
python3 ai.py install
python3 ai.py check
```

Use `py -3` on Windows. The complete installer:

1. Discovers Codex desktop, Codex CLI and Claude Code CLI, checks executable accessibility and bounded CLI version probes, and requests sanitized CLI authentication status.
2. Preflights global file ownership, the portable settings merge and native schedule registration before changing harness files.
3. Installs the declared global guidance/skills for Codex and the supporting Claude Code CLI, then applies portable Codex preferences.
4. Registers and reads back the current user’s daily 00:00 local-time maintenance job.
5. Reports readiness, changes, warnings and any remaining authentication work. `check` repeats read-only verification, including the schedule, and exits nonzero for drift or unavailable prerequisites.

The installer does not install third-party applications, log in, invoke models, install plugin packages, or change project registrations. Missing runtimes include official installation guidance. Follow it, authenticate through each CLI's normal flow, and rerun. A known runtime failure blocks writes; authentication attention allows harness files to be prepared but returns a nonzero status and never claims readiness. Sign-in status is not a model entitlement or quota test.

Use explicit paths for custom installations:

```sh
python3 ai.py runtime doctor --check-auth --json
python3 ai.py install --codex /absolute/codex --claude /absolute/claude --desktop /Applications/ChatGPT.app
```

Desktop verification checks identity metadata and executable accessibility without launching the GUI. PATH visibility is reported separately; discovered absolute CLI paths remain usable. Windows uses bounded package/known-location discovery; native Windows verification must run on that device. `--home /existing/home --no-schedule` supports isolated file preparation, but does not redirect account authentication; it reports auth as unchecked for another home. Native scheduling always belongs to the current OS user. Use `--no-schedule` with `install` or `check` for deliberate file-only setup, including exported bundles without a Git checkout. It skips registration/verification and leaves existing jobs in place.

## App settings

Review [the shared source](../registry/codex-settings.json) and [its scope](../docs/settings.md). Changed preferences require quitting Codex desktop and other active Codex clients, applying from a terminal, and reopening. The CLI conservatively defers if desktop process state is unknown. It never closes the app automatically. An unchanged configuration is safe to check while the app is open.

```sh
python3 ai.py settings plan
python3 ai.py settings apply
python3 ai.py settings doctor
```

The settings command merges into the active `CODEX_HOME/config.toml`, or `~/.codex/config.toml` by default. Complete installation requires the standard Codex home and refuses a different `CODEX_HOME`, because global instruction destinations use the standard user home. For a custom home, use the individual commands and arrange its guidance deliberately. Full setup verifies configured values, not rendered themes or live plugin discovery.

## Platform wrappers and existing jobs

On macOS, `sh setup/macos.sh` runs the complete installer. On Windows, `& .\setup\windows.ps1` does the same, choosing `py -3` or `python` and defaulting to managed copies. Windows requires no link privileges in copy mode; `-Mode link` needs normal OS support. `-SharedDir 'C:\path\to\.ai'` selects another checkout. On macOS use `AI_SHARED_DIR=/absolute/checkout`.

Explicit wrapper targets retain their previous guidance-and-skills-only behavior:

```sh
sh setup/macos.sh codex
sh setup/macos.sh claude-code
sh setup/macos.sh both
```

The legacy `claude` spelling means Claude Code CLI. Explicit targets keep their guidance-only behavior. Complete installation now registers daily maintenance and migrates the exact known legacy midnight installer job; unrelated jobs remain unchanged. A modified or ambiguous legacy job needs review. For advanced guidance-only options use `ai.py` directly:

```sh
python3 ai.py plan --target both
python3 ai.py sync --target both --mode copy
python3 ai.py doctor --target both
```

| Target | Guidance | Global skills |
| --- | --- | --- |
| Codex desktop and CLI | `~/.codex/AGENTS.md` | `~/.agents/skills/<name>` |
| Supporting Claude Code CLI | `~/.claude/CLAUDE.md` | `~/.claude/skills/<name>` |

Both receive the 14 authored global skills. The restricted Claude delegation runner deliberately suppresses ordinary instruction/skill loading; supply relevant project constraints in its brief. The Claude registrations support direct CLI use without making it the primary orchestrator.

## Daily 12:00 AM maintenance

The full installer registers a native OS job at **00:00 every day in the device’s local timezone**. macOS/Linux use user cron (`0 0 * * *`); Windows uses a least-privilege current-user Task Scheduler task with catch-up enabled while the user is logged in. No scheduler password or elevated task is created.

```sh
python3 ai.py schedule plan
python3 ai.py schedule install
python3 ai.py schedule check
python3 ai.py maintenance run
python3 ai.py maintenance status
```

Direct `schedule install` can register the job while Codex is open or this checkout has uncommitted work. Complete installation still requires all its preflight checks. Registration validates the checkout and maintenance policy; it does not fetch or clean the repository.

Each run requires a clean `main` checkout tracking the expected `origin/main`, fetches noninteractively, permits only a fast-forward, and starts the updated synchronization code in a fresh process. Dirty/untracked work, local-ahead or divergent history, unexpected origins and unfinished Git operations stop that run without automatic stash, reset or commit. A currently dirty checkout becomes eligible after its changes are deliberately resolved; job registration need not be repeated.

Guidance/skills can refresh while Codex is open. Changed app preferences are deferred without closing the app and retried on the next eligible daily run. To finish sooner, close Codex and run `python3 ai.py maintenance sync`. Inspect the report for partial or deferred work; a registered job or updated checkout alone is not completed synchronization.

Run status is stored in `~/.agent-harness/maintenance-status.json`; bounded, rotated logs are in `~/.agent-harness/logs/maintenance.log`. Cron output is suppressed to avoid external mail. If Python cannot start, the saved status may remain absent or stale. Check the timestamp and perform a manual run when validating installation. Cron does not wake a sleeping Mac or catch up missed midnight runs. Windows catch-up depends on the logged-in session and OS conditions; native behavior must be checked on Windows. See [scheduling and recovery](../docs/scheduling.md).

## Ownership and recovery

Global ownership is recorded in `~/.agent-harness/state.json`. Missing items are created, matching items retained, and intact managed items updated. Unmanaged files, edited copies and unsafe links stop preflight. Unrelated skills remain untouched. Inspect and preserve conflicting content rather than deleting it to force installation. Copies of skill-catalog retain a checkout locator; refresh them if the checkout moves. macOS links require the checkout to stay available.

Predictable failures are checked before writes. Each global item is staged with recovery, but a later OS failure may leave earlier completed items. Receipts track completed items; failed restoration retains a `.previous-...` backup. Settings keep a private local backup of an existing config before atomic replacement. If settings fail after global installation, the installer reports partial completion. Fix the cause and rerun; do not discard backups blindly.

Synchronization does not prune removed selections or uninstall any app. No permissions, hooks, credentials, MCP servers or account connections are synchronized.

## Project capabilities and plugins

Project initialization defaults to Codex:

```sh
python3 ai.py project init --project /absolute/project --profile full-stack --profile backend-node
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Use `--target both` if direct Claude Code sessions also need those skills. Review the project's manifest and lock with its other configuration. Sync downloads only reviewed pinned payloads and never executes upstream helpers. Alternatively, [export Codex plugins](../docs/plugins.md), avoiding duplicate installation of the same skill set.
