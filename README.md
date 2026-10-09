# Personal agentic harness

One source for my instructions, skills and MCP servers, installed through small
adapters for **Zed**, **Codex Desktop** and **Claude Desktop Code**. Zed is the
default. Each target also receives the settings it can enforce; Zed can manage
extension installation. Shared capabilities stay independent of client layouts.

## Install

Keep this checkout in a stable directory. Python 3.9+ and its standard library
are sufficient. Run from this repository; on Windows use `py -3`.

```sh
python3 ai.py install --target zed --dry-run
python3 ai.py install --target zed
python3 ai.py check --target zed
```

Use `--target codex-desktop`, `--target claude-desktop`, or `--target all` for the
other installations. Close selected clients before applying changed settings.
The installer preserves unrelated settings and comments, backs up existing
configuration, and stops on conflicting unmanaged content.

Installation supplies shared instructions, 14 global skills, selected MCP
configuration and target settings. It does not install applications, dependencies,
log in, start MCP servers or register a schedule. `check` verifies installed files
and configuration drift; client activation is a separate check.

Claude Desktop installation covers local **Code** sessions and Claude Code CLI.
Chat and Cowork use different customization surfaces. For account upload assets:

```sh
mkdir -p build
python3 ai.py handoff --output build/claude-account-handoff
```

The destination must be new. Upload and enable the generated skills manually;
the handoff does not connect accounts or activate tools. See [target coverage and
limits](docs/targets.md) and [setup and recovery](setup/README.md).

## Sources

| Source | Owns |
| --- | --- |
| [instructions/AGENTS.md](instructions/AGENTS.md) | Shared working principles |
| [skills/](skills/) | Authored capability content |
| [registry/harness.json](registry/harness.json) | Shared selections and defaults |
| [registry/targets.json](registry/targets.json) | Target paths, settings and extensions |
| [registry/mcp.json](registry/mcp.json) | Client-neutral HTTP and stdio MCP definitions |
| [registry/catalog.json](registry/catalog.json) | Reviewed project skills, profiles and source pins |

OpenAI documentation MCP is enabled by default; Microsoft Learn is available but
disabled. Change the neutral registry to change selection. Credentials and account
connections belong outside this repository.

Instructions guide agent behavior. Permission settings constrain supported client
actions but remain user-editable. Existing per-tool approvals are preserved, so
installing a default confirmation policy is not a complete lockdown.

## Add project capabilities

Inspect the project's instructions, stack and scope first. Start with
`project-foundation`, then add relevant specialists from the [catalog](docs/catalog.md).
The foundation supplies architecture, debugging, testing and release guidance;
global skills supply the shared workflows.

```sh
python3 ai.py project init --project /absolute/project --target all
python3 ai.py project add --project /absolute/project --skill database-systems --dry-run
python3 ai.py project add --project /absolute/project --skill database-systems
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

`add` updates the declaration; `sync` installs verified payloads and records the
lock. Existing selections and modified or unselected copies are preserved.
Zed and Codex share `.agents/skills`, so their selections must agree when both
are enabled. Claude uses `.claude/skills`. See [project target selection](docs/project-targets.md).

## Optional tools

The core install has no scheduled side effects. Existing scheduling commands are
an explicit opt-in; review [legacy maintenance scope](docs/scheduling.md) before
using them. They are separate from target installation.

- [Portable Codex preferences](docs/settings.md): optional themes and app settings
  through `ai.py settings`; not applied by the target installer.
- [Runtime integrations](docs/runtime-integrations.md): prerequisites and manual setup.
- [Project readiness](setup/README.md#project-readiness): inspect installed tools
  and optionally a running local app.
- [Quality evaluation](docs/quality-harness.md): bounded development exercises.
- [Codex plugin exports](docs/plugins.md): alternative packaging for selected skills.
- [Bounded Claude delegation](skills/agent-coordination/references/claude-code.md):
  optional independent review using an installed CLI.

## Maintain

Review capability content and executable behavior before updating source pins.
Links follow this checkout; managed copies refresh on installation or sync.
Installation and registration do not prove an agent used a skill successfully.

```sh
python3 scripts/render_registry.py --check
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
```

See [migration decisions and verification](docs/work/target-migration.md).
