# Personal Codex harness

This repository is my source of truth for **Codex desktop**: personal guidance, global and project skills, portable app preferences, and Codex plugin bundles. Codex CLI supports installation, automation and diagnostics. ChatGPT/Codex subagents are the normal delegation path; Claude Code CLI provides bounded second opinions when useful. Claude desktop is outside this harness.

[Browse the catalog](docs/catalog.md) for 14 default global skills, 195 project selections and 61 composable profiles. Each project starts with a rich 12-skill foundation and adds specialists for its current work and credible later stages. Coverage includes web, desktop, mobile, APIs, integrations, databases, research, AI systems, documents, design, media, marketing, testing, security and operations. [Source reviews](docs/source-review.md) distinguish reviewed payloads from integrations that need additional work.

## Install and check

Keep the Git checkout in a stable directory. The harness needs Python 3.9+ and uses only its standard library.

```sh
python3 ai.py install --dry-run
python3 ai.py install
python3 ai.py check
```

On Windows use `py -3`. Without arguments, `sh setup/macos.sh` and `& .\setup\windows.ps1` run the complete installer. It verifies Codex desktop, Codex CLI and Claude Code CLI; checks CLI sign-in status; reconciles global instructions and skills; merges the portable settings source; and registers a native daily **12:00 AM (00:00), local time** maintenance job. Missing applications produce official setup links. It does not download applications or sign in for you.

Changes to app preferences require closing Codex desktop and other active Codex clients, running the installer from a terminal, then reopening the app. A matching configuration is a no-op and can be checked while the app is open. macOS uses live skill links; Windows defaults to managed copies. Conflicting user content stops preflight. See [setup, diagnostics and recovery](setup/README.md).

Use `--no-schedule` with `install` or `check` when deliberately preparing an isolated home or an exported bundle without a Git checkout. This skips schedule registration and verification; it does not remove an existing job.

## Daily maintenance

The OS job uses user cron on macOS/Linux and Task Scheduler on Windows. It fetches the expected repository, fast-forwards a clean `main` checkout from `origin/main`, then launches fresh code to synchronize global guidance, skills and portable preferences. Local edits, untracked files, unfinished Git operations and ahead/diverged branches are preserved and reported for review. It never stashes, resets or commits them automatically.

```sh
python3 ai.py schedule plan
python3 ai.py schedule install
python3 ai.py schedule check
python3 ai.py maintenance run
python3 ai.py maintenance status
```

Changed preferences wait while Codex is open; global guidance can still refresh. Close Codex and run `python3 ai.py maintenance sync`, or let the next daily run retry. Schedule registration does not prove a successful run. Cron misses runs while the Mac is asleep; Windows catch-up requires the user session and scheduler conditions. See [scheduling, logs and recovery](docs/scheduling.md).

The global [AGENTS.md](instructions/AGENTS.md) requires relevant skill pickup and evidence-based work while remaining under 100 lines. [CLAUDE.md](instructions/CLAUDE.md) defines the supporting Claude Code role. Global availability does not load every skill into every task.

[Task routing](skills/skill-catalog/references/task-routing.md) maps UI design, engineering, technical documentation, artifacts, motion and search outcomes to reviewed capabilities and their actual invocation names. [Routing audit](docs/reviews/skill-routing.md) records the Trixpo reference check and the limits of installation and selection evidence. UI refinement requires inspected, comparable renders and iteration against the requested visual goal.

## Deliver and evaluate quality

[Delivery standards](skills/skill-catalog/references/delivery-standards.md) connect task selection to the evidence needed for frontend, motion, engineering, documentation, integrations and research. Selected skills now carry concrete composition examples, asynchronous state patterns, boundary decisions, integration failure handling, source-checked documentation and claim verification. Project conventions and actual tools determine the implementation.

Five synthetic development exercises check delivered outputs separately from registration and qualitative review:

```sh
python3 ai.py eval list
mkdir -p build/evaluation-runs
python3 ai.py eval prepare --case engineering-ledger-total --output build/evaluation-runs/ledger \
  --model 'exact model/version or unavailable' --condition candidate
# Assign the prepared task; edit only its workspace, then:
python3 ai.py eval check --run build/evaluation-runs/ledger
python3 ai.py eval report --run build/evaluation-runs/ledger
```

The CLI runs local candidate code with normal host permissions and bounded duration/output. It launches no model. Frontend, documentation and research cases also require recorded artifact review through `eval review`. Passing these development cases does not establish production readiness or a quality gain. See [the harness design, pilot evidence and next evaluation steps](docs/quality-harness.md).

## Share app preferences

[registry/codex-settings.json](registry/codex-settings.json) contains reviewed portable values captured from this Mac: model/reasoning/response preferences, desktop themes and interaction preferences, and public plugin enablement flags. It excludes credentials, permission policies, project paths, remote devices, history, private plugins and account connections.

```sh
python3 ai.py settings plan
python3 ai.py settings apply
python3 ai.py settings doctor
# Deliberately refresh the source after changing your preferred settings:
python3 ai.py settings capture --output registry/codex-settings.json
```

Merging preserves unselected configuration and comments, and backs up an existing configuration locally. Missing source values do not remove target values. The harness does not install plugins or connect accounts; Codex may fetch configured plugins during marketplace refresh. Models, fonts, themes and plugins depend on each device and account. Desktop appearance keys are version-dependent. See [portable preferences](docs/settings.md).

## Delegate selectively

Use available Codex subagents for independent work. For a material uncertainty that benefits from Claude's perspective, give it a compact assignment and keep Codex responsible for verifying and integrating the findings:

```sh
python3 ai.py delegate claude --project /absolute/project --prompt-file /absolute/review.txt --plan
python3 ai.py delegate claude --project /absolute/project --prompt-file /absolute/review.txt --timeout 120 --output /absolute/new-result.json
```

The runner uses the official installed Claude CLI and its existing authentication. It limits the session to reading project files, bounds execution and output, and blocks silent API/provider billing fallback. It cannot edit, run shell commands or start more agents. See the [delegation contract](skills/agent-coordination/references/claude-code.md) for limitations and how to supply a useful brief.

## Declare project capabilities

Inspect the project's instructions, stack and roadmap. Initialize its `.ai/project.json` with the default `project-foundation` profile, then add reviewed specialists where the project has a concrete current or foreseeable need. Do not install the whole catalog or every profile.

The foundation contains **work-planning, context-management, agent-coordination, research-and-synthesis, security-judgment, architecture-review, debugging, test-design, ci-maintenance, release-operations, document-parsing and office-authoring**. It complements the global skills. Skills remain registered for reuse; agents load only the guidance relevant to each task.

```sh
python3 ai.py catalog profiles
python3 ai.py catalog list --scope project --query database
python3 ai.py catalog search redis
python3 ai.py catalog show database-systems
python3 ai.py project init --project /absolute/project
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Codex is the default target. At initialization, add justified profiles with `--profile`, individual entries with `--skill <id>`, or project exceptions with `--skip <id>`. Required companions resolve automatically and cannot be skipped. Use `--target both` only when Claude Code also needs project registrations; `claude-code` is a CLI-only target, and the legacy `claude` spelling remains compatible.

When an existing project needs a new skill, persist it in the project catalog and install it through the same workflow. For example, when a database becomes part of the project:

```sh
python3 ai.py project add --project /absolute/project --skill database-systems --dry-run
python3 ai.py project add --project /absolute/project --skill database-systems
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

`project add` preserves existing choices and metadata, adds the configured foundation and requested selections, and validates the candidate before updating the manifest. It does not install payloads or update the lock; follow it with sync and doctor. Existing manifests and installations gain no new selections merely by running plan/sync. For details, see [project selection and incremental registration](setup/README.md#project-capabilities-and-plugins).

For provider-specific additions, use `--target-skill claude:matt-git-guardrails-claude-code` with `project init --target both`, or `project add` when Claude is already a declared target. Scoped additions upgrade the manifest to schema v2 with `target_skills`. Shared selections still apply to every declared target; unsupported combinations fail explicitly. See [provider-specific selections and sidecar migration](docs/project-targets.md).

Project copies go to `.agents/skills` for Codex and optionally `.claude/skills` for Claude Code. Synchronization checks pinned source and adapted hashes, retains companions and licenses, and records `.ai/project.lock.json`. It preserves modified and unselected copies. One-off catalog installation remains available for deliberate standalone use; ongoing project skills belong in the manifest and lock.

## Export Codex plugins

Six bundles cover foundation, web engineering, design, research/documents, marketing and engineering operations:

```sh
mkdir -p build
python3 ai.py export --bundle personal-foundation --output build/personal-marketplace-v2
```

Repeat `--bundle` to combine selections. The destination must be absent. Export produces a Codex marketplace, portable and Codex plugin manifests, verified payloads and a build lock. The foundation includes a movable registry engine. Export does not activate plugins; avoid installing the same skills globally and through a plugin. See [plugin usage](docs/plugins.md).

## Repository layout

```text
ai.py                       Command-line entry point
instructions/               Codex global and Claude Code delegate guidance
registry/harness.json       Client, delegation, global/project defaults and maintenance policy
registry/codex-settings.json Portable Codex app and model preferences
registry/catalog.json       Curated entries, profiles, hashes and source pins
registry/source-index.json  Broader discovery inventory and review status
skills/                     Authored skill sources
lib/                        Installation, scheduling, settings, delegation and catalog code
evaluations/                Development tasks, seed workspaces, verifiers and review rubrics
setup/                      Stable macOS and Windows wrappers
docs/                       Catalog, reviews, integration and verification evidence
tests/                      Isolated behavior and regression checks
build/                      Generated marketplace exports (ignored)
```

## Maintain the source of truth

Review changed instructions, companions, licenses and executable behavior before updating pins and payload hashes. Global links follow this checkout; managed copies follow deliberate synchronization. Runtime packages and accounts remain separate integration work; see [runtime integrations](docs/runtime-integrations.md).

```sh
python3 scripts/render_registry.py
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
```

The renderer updates catalog/source tables; `--check` detects drift. Codex and Claude guidance have distinct roles and are maintained separately. [Verification evidence](docs/verification.md) records completed checks and remaining limits.
