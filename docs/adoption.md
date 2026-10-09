# Adopt the harness

This harness gives Codex Desktop and Claude Desktop Code a shared set of working
instructions, reusable skills and tools for checking their work. It helps you keep
those choices consistent across projects and machines while leaving each project
in charge of its architecture, commands and release process.

Start with one client and one small project. Rehearse the installation, inspect
what it prepares, then try a real task whose result you can judge. You can add
specialists and optional tools as your work needs them.

**In this guide:** [concepts](#understand-the-three-layers) ·
[client choice](#1-choose-your-client-and-review-the-defaults) ·
[safe rehearsal](#2-rehearse-without-changing-your-client) ·
[installation](#3-install-into-your-real-client) ·
[project adoption](#4-adopt-an-existing-project) ·
[first task](#5-try-one-real-task-and-judge-the-result) ·
[team use and updates](#maintain-your-installation-and-share-it-with-a-team) ·
[recovery](#troubleshooting-and-recovery)

## Understand the three layers

| Layer | What it contains | How you manage it |
| --- | --- | --- |
| Shared installation | Working principles, global skills, selected client settings and MCP definitions | `install`, then `check` |
| Project selection | A small foundation plus skills chosen for the project's actual work | `project init` / `project add`, then `plan`, `sync`, `doctor` |
| Runtime tools | Clients, compilers, browsers, renderers, credentials and connected services | Install and configure separately; inspect with the readiness tools and the client's own UI |

A **skill** is a directory of instructions, sometimes with references or scripts.
A **profile** selects a group of skills; it is not another agent or an invocation.
The **catalog** records available selections, source pins, dependencies and
caveats. An **MCP server** exposes tools to a client and may need its own runtime
or account.

Installing a skill makes its files available. The agent still needs to discover
it, select it for the task and follow it. A successful check of installed files
does not prove any of those later steps—or that the finished work is good.

## 1. Choose your client and review the defaults

You need Git and Python **3.9 or newer**. The installer uses Python's standard
library; there is no package installation step. To use the result, you also need
the selected client installed, signed in and able to run a task.

```sh
git clone https://github.com/codemirket/harness.git
cd harness
python3 --version
python3 ai.py --help
```

Keep the checkout in a stable location for either installation mode: catalog and
CLI helpers use this source checkout even when skill files are installed as copies.
On Windows, use `py -3` wherever this guide shows `python3`.

| Select | Intended surface | Files prepared under your home directory |
| --- | --- | --- |
| `--target codex-desktop` | Codex Desktop and Codex CLI | `.codex/AGENTS.md`, `.agents/skills/`, `.codex/config.toml` |
| `--target claude-desktop` | Claude Desktop **Code** sessions and Claude Code CLI | `.claude/CLAUDE.md`, `.claude/skills/`, `.claude/settings.json`, `.claude.json` |
| `--target all` | Both of the above | Both sets of destinations |

The default is both clients, so use an explicit target throughout your first
installation. Claude Chat and Cowork use a separate manual account handoff; the
local Code installation does not configure them. Custom `CODEX_HOME` or
`CLAUDE_CONFIG_DIR` locations are not supported when they differ from the standard
layout. See [target coverage](targets.md) for these boundaries.

Read [shared instructions](../instructions/AGENTS.md),
[target settings](../registry/targets.json) and [MCP definitions](../registry/mcp.json)
before installing. The defaults include Codex's `on-request` approval policy and
`workspace-write` sandbox, Claude's `default` permission mode, and OpenAI's
documentation MCP server. Selected settings are applied to your client; unrelated
configuration is preserved. Instructions guide behavior, while native client
settings provide the controls supported by that client.

Optional model, appearance and plugin preferences are managed separately and
reflect the maintainer's choices. They are not required for adoption.

## 2. Rehearse without changing your client

From the harness checkout, create a disposable home and project. On macOS or a
POSIX shell:

```sh
harness_trial="$(mktemp -d)"
mkdir -p "$harness_trial/home" "$harness_trial/project"
```

In Windows PowerShell, use this preparation instead:

```powershell
$harness_trial = Join-Path ([System.IO.Path]::GetTempPath()) ("harness-trial-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path "$harness_trial/home", "$harness_trial/project"
```

The following commands use the same variable in either shell. Substitute `py -3`
on Windows. Change the target if you selected Claude or both clients.

```sh
python3 ai.py install --target codex-desktop --home "$harness_trial/home" --mode copy --dry-run
python3 ai.py install --target codex-desktop --home "$harness_trial/home" --mode copy
python3 ai.py check --target codex-desktop --home "$harness_trial/home" --mode copy
```

Read the report and inspect the files in the temporary home. The dry run previews
the proposed changes; the next command writes only into that home, and `check`
verifies the resulting installation. An alternate `--home` does not switch the
home used by a running client or authenticate an account.

Now rehearse project selection using a locally authored SVG workflow:

```sh
python3 ai.py project init --project "$harness_trial/project" --target codex-desktop
python3 ai.py catalog show svg-creation
python3 ai.py project add --project "$harness_trial/project" --skill svg-creation --dry-run
python3 ai.py project add --project "$harness_trial/project" --skill svg-creation
python3 ai.py project plan --project "$harness_trial/project"
python3 ai.py project sync --project "$harness_trial/project"
python3 ai.py project doctor --project "$harness_trial/project"
```

Inspect `.ai/project.json`, `.ai/project.lock.json` and the installed project
skills under `.agents/skills/` (or `.claude/skills/` for Claude). Init supplies four
foundation skills: architecture review, debugging, test design and release
operations. The example adds one specialist. This path uses local harness
content; some other catalog selections fetch pinned upstream sources.

`add` changes the declaration; `sync` installs the selected payloads and writes
the lock; `doctor` checks the result. Repeat sync and doctor to confirm the
installation is stable. Keep the rehearsal directory until you have inspected it,
then remove that disposable directory when you no longer need it.

## 3. Install into your real client

Choose between source-backed links and managed copies. Links reflect changes in
the checkout immediately. Copies provide a snapshot of the skill files and need
another installation run to receive those updates. In both modes, catalog helpers
still use the source checkout: moving or deleting it breaks those helpers, and
updating it changes the CLI they invoke before any copied skills are refreshed.
The `auto` mode chooses the platform default; an explicit mode makes your choice
clear. This guide uses copies for a first installation:

```sh
python3 ai.py install --target codex-desktop --mode copy --dry-run
```

Review the proposed paths and settings. Close the selected client before applying
changed settings, then run:

```sh
python3 ai.py install --target codex-desktop --mode copy
python3 ai.py check --target codex-desktop --mode copy
```

Repeat the explicit mode when checking. Existing settings receive a local backup
before replacement. If the installer reports an ownership conflict, follow the
recovery guidance below before retrying.

Reopen the client and start a fresh session. Confirm the shared instructions and
`skill-catalog` are discoverable, inspect MCP status, and invoke one relevant
skill or tool. Check the actual paths if another plugin or earlier installation
supplies the same skill name. Avoid enabling duplicate direct-install and plugin
copies without understanding the client's resolution behavior.

## 4. Adopt an existing project

First read the project's `AGENTS.md`, `CLAUDE.md` and other contribution or setup
instructions. Inspect its current Git status and `.ai/project.json`, if present.
Retain the project's architecture, required checks and authorization boundaries.
The harness is shared guidance, not a replacement project template.

For a project that has no harness manifest, run from the harness checkout,
replacing `/absolute/project` with its existing directory:

```sh
python3 ai.py project init --project /absolute/project --target codex-desktop
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
python3 ai.py runtime doctor --project /absolute/project --json
```

Project init creates the selection manifest; it does not replace the project's
instruction files. If a manifest already exists, inspect its selections and go
straight to plan, sync and doctor. Init refuses a differing existing manifest.
Use `project add` for additions and review target changes deliberately.

The default `project-foundation` complements the shared global installation. For
collaborators who do not have those globals, `portable-foundation` is an explicit
alternative providing the broader portable foundation. Review overlap before
adding it: same-name global and project skills can coexist, and client precedence
can select a different copy than you intended.

Add specialties when there is a concrete need. For example, a project producing
editable vector artwork can use the `svg-creation` sequence from the rehearsal.
For another task, discover the relevant contract and inspect the proposed skills:

```sh
python3 ai.py capabilities list
python3 ai.py capabilities show backend-engineering
python3 ai.py catalog search "database"
```

Read dependencies, compatibility and caveats before selection. A catalog search
also includes indexed discovery records; finding one does not make it a reviewed
installable skill. [The catalog](catalog.md) and [project target guide](project-targets.md)
explain profiles, provider-specific selections and source verification.

`runtime doctor` reports prerequisites without installing tools or running project
scripts. Missing optional tools may make a diagnostic unsuccessful even when the
skills are installed correctly. Its CLI client probes do not establish desktop
activation. Also run the project's own setup and required checks.

## 5. Try one real task and judge the result

Use a small task with a result you can inspect. Here is a starting prompt to adapt
after replacing the bracketed sentence:

> Read this project's instructions and `.ai/project.json`, inspect the relevant
> files and current changes, and use skill-catalog to select the workflow for this
> task: [describe one small change and the behavior you expect]. Preserve unrelated
> changes. Use the existing project tools, identify any missing prerequisite, and
> complete the relevant checks. Report what changed, the evidence you inspected,
> and any remaining limits. Do not commit, publish, deploy or install dependencies.

For an SVG project, a concrete task could be an editable illustration with light
and dark previews and an inspection at small sizes. For an application, it could
be one reproducible bug with a regression check based on the expected behavior.

Judge adoption at three levels:

1. **Installed:** global check and project doctor pass, with the expected target
   paths and selections.
2. **Used:** the fresh client session discovers and applies the relevant workflow,
   and the required tools actually run.
3. **Delivered:** the change meets your task's acceptance criteria. Inspect the
   artifact or behavior yourself; a skill name in a response is not evidence of
   quality.

Optional workbench tools can help collect evidence:

```sh
python3 ai.py workbench doctor
```

Markdown checks use the Python standard library. Browser, vector and office
rendering have additional prerequisites. Configure existing tools as described
in [the workbench guide](workbench.md); the harness does not download them for you.
Use [Fieldwork Studio](../examples/craft-lab/) as an optional browser/vector
exercise once the appropriate tools are available.

## Maintain your installation and share it with a team

Fork the repository if you want to own the shared defaults. Change working
principles in `instructions/`, global selections in `registry/harness.json`,
client policy in `registry/targets.json`, and server definitions in
`registry/mcp.json`. Preserve upstream attribution and review the
[third-party terms](third-party-notices.md) when redistributing selected skills.
Keep credentials and machine-specific runtime configuration outside Git.

For a team, agree on a harness revision and record it in the team's setup guide.
Share the project's `.ai/project.json` and generated `.ai/project.lock.json` through
your normal code review process. Decide whether project skill copies are tracked
or reconstructed during onboarding; preserve their receipts in either case.
Keep installed copies generated from the reviewed source instead of editing them
in place. Each developer still needs to install the selected client, authenticate
and verify their runtime locally. A checked-in lock does not install tools.

Review upstream changes before updating your checkout, particularly with linked
global skills. Preserve your fork's local changes and use your normal Git update
workflow. After accepting an update, preview and reconcile global installation
with your chosen target and mode, then run project plan, sync and doctor for each
affected project. Run the project's required checks as well. Updates are not
scheduled automatically by default.

If you explicitly enable the optional maintenance workflow on your own fork,
update `maintenance.expected_repository` in `registry/harness.json` to your fork's
GitHub identity first. See [scheduling](scheduling.md) for its scope. Existing
project copies can retain receipts naming an older source location when their
content still matches; review the provenance report described in
[project targets](project-targets.md) instead of rewriting those receipts.

## Troubleshooting and recovery

| Symptom | Next step |
| --- | --- |
| Install reports an unmanaged file or edited copy | Preserve it, compare it with the proposed source and decide how to merge or relocate it. Do not delete it just to clear the check. |
| Project doctor reports stale lock or selected-copy drift | Run project plan. Review local edits and source changes before sync; do not fabricate receipts or hand-edit hashes. |
| Old skills remain after changing the selection | Sync preserves unselected copies. Inspect their paths and ownership, then deliberately remove only copies you no longer need. |
| Check passes but a skill is missing in the client | Open a fresh session, inspect client discovery and exact paths, and look for duplicate names or the wrong client surface. |
| A tool or renderer is unavailable | Inspect runtime/workbench diagnostics and the skill's declared dependencies. Configure the required existing tool or choose an available supported workflow. |
| MCP configuration is present but calls fail | Check live client MCP status, the server endpoint, authentication and any local executable requirements. File checks cannot establish service health. |

There is no general uninstall command. Use the installation report to identify
confirmed harness-owned files or links, preserve your later edits and review the
relevant configuration backup before restoring it. Removed source entries are
not automatically pruned. Keep the checkout available until source-backed links
have been deliberately reconciled.

See [installation and recovery](../setup/README.md) for operational details,
[target coverage](targets.md) for client limitations and
[verification](verification.md) for the repository's checks.
