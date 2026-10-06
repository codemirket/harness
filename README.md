# Personal AI harness

This repository is the source of truth for my shared agent instructions, global skills, project capability selections and Codex/Claude plugin bundles. It supports macOS and Windows using Python 3.9+ and the standard library.

[Browse the catalog](docs/catalog.md) for the current counts, profiles and selections. [Source reviews](docs/source-review.md) distinguish reviewed installations from useful integrations that still need runtime or licensing work. Coverage includes web, desktop, mobile, APIs, integrations, databases, research, AI systems, documents, design, media, marketing, testing, security and operations.

## Install the declared global setup

```sh
python3 ai.py plan --target both
python3 ai.py sync --target both
python3 ai.py doctor --target both
```

On Windows use `py -3` instead of `python3`. Choose `codex`, `claude` or `both`. The platform wrappers remain available for existing scheduled jobs:

```sh
sh setup/macos.sh both
```

```powershell
& .\setup\windows.ps1 both
```

The declared **14 global skills** cover selection, engineering, interface design, marketing, document workflows, research, parsing, office authoring, collaboration, context management, CI maintenance, AI evaluation, security and durable work planning. They are available broadly and loaded when relevant. Their exact IDs and app destinations live in [registry/harness.json](registry/harness.json).

[AGENTS.md](instructions/AGENTS.md) and [CLAUDE.md](instructions/CLAUDE.md) require skill pickup at project entry and when needs materially change. They reuse unchanged selections and preserve task-based judgment. Both remain below 100 lines. These are agent instructions; they are not a host-enforced execution policy.

macOS uses live links; Windows defaults to managed copies and requires no symbolic-link privileges. Setup preserves unrelated files and refuses to overwrite edited or unrecognized content. Rerun synchronization to refresh copies. See [setup and recovery](setup/README.md).

## Declare project capabilities

Ask the agent to inspect the stack and compose a complete setup from the registry. For repeatable installation, save the selection in the target project's `.ai/project.json`:

```sh
python3 ai.py catalog profiles
python3 ai.py catalog list --scope project --query database
python3 ai.py catalog search redis
python3 ai.py catalog show database-systems
python3 ai.py project init --project /absolute/project --target both --profile full-stack --profile backend-node --profile collaboration
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Repeat `--profile`, add `--skill <id>`, or omit optional selections with `--skip <id>` during initialization. Edit an existing manifest deliberately when needs change. Dependencies resolve automatically; required companions cannot be skipped. Compose lifecycle and platform profiles rather than limiting a project to a handful of skills.

Project copies go to `.agents/skills` for Codex and `.claude/skills` for Claude. Synchronization verifies pinned source and adapted hashes, retains references/assets/licenses, and records `.ai/project.lock.json`. It updates intact managed copies and preserves local edits. Unselected copies remain in place for deliberate cleanup. Predictable preparation failures occur before skill writes; later OS failures can leave earlier completed items.

For a one-off copy, `python3 ai.py catalog install <id> --agent both --project /absolute/project` remains supported. The global `skill-catalog` includes locator-aware wrappers for running these commands from either linked or copied installations.

## Export personal plugin marketplaces

The named bundles in `registry/harness.json` package foundation, web engineering, design, research/documents, marketing and engineering operations:

```sh
mkdir -p build
python3 ai.py export --bundle personal-foundation --output build/personal-marketplace
```

Repeat `--bundle` to include several bundles. The output must not already exist. Export writes both marketplace formats, portable and host-specific plugin manifests, pinned skill payloads and a reproducible build lock. The foundation bundle includes its own registry engine so skill selection works after the bundle is moved to another device.

Export does not activate a plugin or provision its runtime. Use the generated marketplace with a supporting client and verify discovery in a fresh session. Avoid enabling an exported bundle alongside another installation of the same skills. See [plugin usage](docs/plugins.md) for client-specific steps and compatibility limits.

## Repository layout

```text
ai.py                   Single command-line entry point
instructions/           Shared AGENTS.md and matching CLAUDE.md
registry/harness.json   Global selection, target adapters and named bundles
registry/catalog.json   Curated entries, profiles, hashes and source pins
registry/source-index.json  Broader discovery inventory and review status
skills/                 All authored skill sources
lib/                    Registration, reconciliation and export implementation
setup/                  Stable macOS and Windows wrappers
docs/                   Generated catalog, reviews, integration and verification evidence
tests/                  Isolated behavior and regression checks
build/                  Generated marketplace exports (ignored)
```

Source metadata is discovery data, not an instruction to execute code. Registration and export do not run upstream installers, hooks, helpers, dependencies or external services. Accounts, host permissions and runtime packages remain explicit integration work; their contracts are recorded in [runtime integrations](docs/runtime-integrations.md). Other clients need a reviewed target adapter rather than a guessed configuration path.

## Maintain the source of truth

Review changed instructions, companions, licenses and executable behavior before updating source pins and payload hashes. Global links follow this checkout; project copies follow deliberate synchronization. Keep credentials and machine-specific preferences outside the repository.

```sh
python3 scripts/render_registry.py
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
```

The renderer updates the human catalog/source tables and the CLAUDE mirror from their canonical inputs; `--check` detects drift. See [verification evidence](docs/verification.md) for completed checks and what remains unverified. No catalog can guarantee coverage of every future tool or task; indexed candidates make additional review possible without starting discovery from scratch.
