# Install and reconcile

Run from a stable checkout with Python 3.9+; no Python packages are required.
Zed is the default target. The installer prepares configuration for an existing
client; installing the client and authenticating remain separate steps.

## Target installation

```sh
python3 ai.py install --target zed --dry-run
python3 ai.py install --target zed
python3 ai.py check --target zed
```

Supported public targets are `zed`, `codex-desktop`, `claude-desktop`, and `all`.
Use `py -3` instead of `python3` on Windows. The platform wrappers in this folder
provide convenient entry points; the Python CLI is the complete interface.

The installer preflights shared guidance, skills, target settings and MCP entries,
then reconciles the selected destinations. It preserves unrelated configuration
and comments. Selected settings are replaced by the declared source values;
conflicting MCP endpoint definitions fail rather than silently replacing them.

Close selected clients before applying changed settings, then reopen them. The
installer does not close applications. A matching installation is a no-op;
`check` is read-only and can run while clients are open.

```sh
# Prepare an existing isolated home without changing your own installation:
python3 ai.py install --target all --home /absolute/test-home --mode copy
python3 ai.py check --target all --home /absolute/test-home --mode copy
```

`--mode auto` selects the platform default; `link` keeps source-backed files and
`copy` creates managed copies. Repeat an explicit mode when checking it. Keep the
checkout available when using links.
An alternate `--home` prepares files there; it does not switch accounts, launch a
client with that home or verify that a live client reads those files.

Installation does **not** install apps or dependencies, check authentication,
invoke a model, start MCP servers, register jobs, upload account skills, or alter
project registrations. Consult [target coverage](../docs/targets.md) before treating
prepared configuration as an active capability.

## Verify activation

After `check` passes, open a new session in the selected client. Verify shared
instructions and a representative skill are available, inspect MCP server status,
and test one relevant tool. In Zed, also inspect the Extensions page for declared
extensions. These checks establish client discovery and invocation separately
from file correctness. Model access, service availability and tool behavior need
their own evidence.

For Zed external agents, install the engine's target too: Codex configuration for
Codex, Claude configuration for Claude. Zed's native instruction and skill loader
does not govern those agents.

## Ownership and recovery

Missing global files are created, matching files retained, and intact managed
files updated. Unmanaged content, edited managed copies and unsafe links stop
preflight. Unrelated skills are preserved. Inspect conflicts and retain local work;
do not delete content merely to make an installation pass.

Existing settings receive a local backup before replacement. File updates use
staging and replacement, but an operating-system failure can leave earlier writes
completed. Read the result, preserve backups, fix the reported cause and rerun.
Installation does not prune removed skills or uninstall applications.

## Project capabilities and plugins

Project setup is separate from global installation:

```sh
python3 ai.py project init --project /absolute/project --target zed
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Use `--target all` for all clients. Init adds the configured `project-foundation`;
`project add --skill ID` or `--profile ID` registers later requirements. Add does
not install: follow it with sync and doctor. Existing manifests keep their choices.
Use `portable-foundation` explicitly when the shared globals are absent.

Codex and Zed share project `.agents/skills` and require identical selections.
Claude uses `.claude/skills`. Source pins, required companions and payload hashes
remain checked. Sync preserves modified and unselected copies. See [project
targets](../docs/project-targets.md), [catalog](../docs/catalog.md), and the optional
[Codex plugin export](../docs/plugins.md) workflow.

## Project readiness

`runtime doctor` remains an optional diagnostic, separate from install/check.
Its legacy client probes cover Codex and Claude Code CLI, not Zed or Claude
Desktop activation. It probes project prerequisites without installing them or
running project scripts. For a running local app:

```sh
python3 ai.py runtime doctor --project /absolute/project --json
python3 ai.py runtime doctor --project /absolute/project --url http://127.0.0.1:3000 --screenshot /absolute/project/build/readiness.png --json
```

Start the app through its own workflow first. The screenshot destination must be
new. Capture uses an isolated browser profile without your authenticated session;
page resources may access the network. Inspect the image and actual interactions.
A successful HTTP probe or screenshot does not establish visual acceptance,
dependency integrity or production readiness. Run the project's required checks.

## Optional legacy workflows

`ai.py settings plan/apply/doctor` manages optional portable Codex preferences.
It is not part of target installation. See [settings scope](../docs/settings.md).

Scheduling is opt-in and separate. Existing jobs are not removed by installing the
new harness. Review [scheduling](../docs/scheduling.md) before using
`ai.py schedule install` or `ai.py maintenance run`; these commands retain their
legacy maintenance workflow and are not a general all-target scheduler.

For Claude Chat/Cowork account customization, run
`python3 ai.py handoff --output /absolute/new-directory`, then follow the generated
manual instructions. Files on disk alone do not enable account skills or connectors.
