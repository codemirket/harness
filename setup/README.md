# Install and reconcile the personal harness

Keep this entire repository in a stable local directory. Python 3.9+ is required; the installer uses only its standard library. Global installation is defined by `registry/harness.json`, not a second hard-coded list in the platform scripts.

| Target | Instructions | Global skills |
| --- | --- | --- |
| Codex | `~/.codex/AGENTS.md` | `~/.agents/skills/<name>` |
| Claude Code | `~/.claude/CLAUDE.md` | `~/.claude/skills/<name>` |

The current desired set contains 14 authored skills. Codex and Claude need not be installed to prepare these files. File registration does not prove discovery by a running app; start a fresh session and verify the expected names.

## macOS

```sh
python3 ai.py plan --target both
sh setup/macos.sh both
python3 ai.py doctor --target both
```

The wrapper accepts `codex`, `claude` or `both`. It finds the repository relative to itself, so it works from another directory. `AI_SHARED_DIR=/absolute/checkout sh setup/macos.sh both` selects another copy. Default installation creates live links; moving the checkout breaks those links. Use `--mode copy` with `ai.py sync` when copies are preferable.

Existing scheduled calls to `setup/macos.sh codex` and `setup/macos.sh claude` continue to work. Setup does not create or change scheduled jobs. A job running this checkout picks up the declared set on its next execution.

## Windows 11

```powershell
py -3 .\ai.py plan --target both
& .\setup\windows.ps1 both
py -3 .\ai.py doctor --target both
```

The wrapper uses `py -3` or `python`, verifies Python 3.9+, and defaults to managed copies. Symbolic-link privileges are unnecessary in copy mode. Optional `-Mode link` requires the device's normal link privileges. `-SharedDir 'C:\path\to\.ai'` selects a different repository. Resolve PowerShell execution policy according to device policy if required.

Copy mode stores a repository locator with `skill-catalog`. Keep the checkout available and rerun setup after moving or updating it. Native PowerShell execution and client discovery must be checked on the Windows device; macOS test results do not establish them.

## Ownership, updates and recovery

The installer records ownership in `~/.agent-harness/state.json`. It creates missing destinations, leaves matching content intact, and updates previous managed content only when its fingerprint still matches. It recognizes old links from this checkout to `components/AGENTS.md` and migrates them to the new per-host instruction files. Unknown links, modified copies and colliding user files stop preflight. Unrelated skills remain untouched.

Do not delete a conflicting directory to force installation. Inspect the reported path and preserve or merge its edits. A subsequent `plan` shows the remaining work. Copy ownership depends on the receipt; keep it with the installation.

All predictable source/destination errors are checked before writes. Each changed item is staged and has local recovery. This is not a multi-file transaction: a later OS failure may leave earlier completed items. Rerun after addressing the cause. If restoration itself fails, the `.previous-...` backup is retained beside the destination for recovery. Never remove it before inspecting its content.

The installer does not prune skills removed from the global selection or a project manifest. Review obsolete registrations and their ownership before deliberate removal. It also does not change models, app preferences, hooks, MCP servers, permissions, credentials or account connections.

## Project manifests and plugins

From the repository, choose profiles and declare both targets:

```sh
python3 ai.py project init --project /absolute/project --target both --profile full-stack --profile backend-node
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Review and commit the target project's manifest and lock when appropriate for that project. Source copies include provenance and license notices. `sync` may download reviewed bytes from immutable upstream revisions, but never executes the downloaded code. Profiles and hashes remain controlled by this repository.

For clients using marketplaces, [export a plugin bundle](../docs/plugins.md). Choose direct global registration or a plugin for the same skill set to avoid duplicate discovery. Both routes read the same registry.
