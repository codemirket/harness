# Personal Agentic Harness

A shared toolkit for **Codex Desktop** and **Claude Desktop Code**: reusable skills,
MCP configuration, working instructions and executable checks for finished work.
Keep capability sources in one repository and install them through small,
client-specific adapters.

The workflow is **make → render or exercise → inspect → repair**. It covers UI,
animation, illustration, office documents, Markdown and software engineering.
It is a personal setup you can fork and adapt; review the defaults before installing.

![Fieldwork Studio, the included interactive UI specimen](docs/assets/fieldwork-studio.png)

*The included [Fieldwork Studio example](examples/craft-lab/) has working search,
filters, saved items, keyboard-operable previews and original SVG illustrations.*

## What you get

| Capability | Concrete output |
| --- | --- |
| UI and motion | Browser journeys, desktop/mobile screenshots and normal/reduced-motion frames |
| Vector assets | Editable SVG, multi-size light/dark renders and a contact sheet |
| Office documents | Editable DOCX/PPTX/XLSX, page renders and formula recalculation |
| Markdown | Local link, heading and fence checks, plus technical/editorial review workflows |
| Engineering and QA | Project-specific behavior tests, failure investigation and independent artifact review |
| Shared setup | 14 global skills, project skill selection, MCP definitions and native target settings |

The installer needs only Python's standard library. Artifact tools use optional,
already-installed runtimes. Neither installation nor a passing check guarantees
visual quality; inspect the actual result. See [research and design decisions](docs/research/decisions-2026-10-09.md).

## Quick start

You need Git, Python **3.9+**, and at least one supported client installed and signed
in. Keep the clone in a stable location: linked installations depend on it.

```sh
git clone https://github.com/nazmirket/.ai.git personal-ai
cd personal-ai

# Preview the exact changes first. Choose one client or use --target all.
python3 ai.py install --target codex-desktop --dry-run

# Close the selected client before applying changed settings.
python3 ai.py install --target codex-desktop
python3 ai.py check --target codex-desktop
```

For Claude, substitute `--target claude-desktop`. `--target all` installs both.
On Windows, use `py -3` in place of `python3`. The installer supports managed
copies as well as links; add `--mode copy` consistently to install/check to keep
copies. See [setup and recovery](setup/README.md).

After installation, reopen the client and start a fresh session. Confirm that
shared instructions and `skill-catalog` are available, inspect MCP status, and
exercise a representative skill. `check` verifies files and configuration;
it cannot establish that a running client has loaded them.

### What installation changes

| Target | Guidance | Skills | Configuration |
| --- | --- | --- | --- |
| Codex Desktop / CLI | `~/.codex/AGENTS.md` | `~/.agents/skills/` | `~/.codex/config.toml` |
| Claude Desktop Code / CLI | `~/.claude/CLAUDE.md` | `~/.claude/skills/` | `~/.claude/settings.json`, `~/.claude.json` for MCP |

The defaults include OpenAI's documentation MCP server and selected native
permission settings. Review [target defaults](registry/targets.json),
[MCP definitions](registry/mcp.json) and [shared instructions](instructions/AGENTS.md)
before applying them. Unrelated configuration is preserved; conflicting unmanaged
files stop installation. Changed configuration receives a local backup.

Installation does not install clients or packages, log into accounts, start MCP
servers, register a schedule, change projects or enable every skill in the catalog.
Optional model, appearance and plugin preferences are a separate command and
contain the maintainer's choices; they are not universal defaults.

**Claude Chat and Cowork:** local Code configuration does not configure those
surfaces. Use the manual [account handoff](docs/targets.md) for supported skill
uploads; local shell tools and account connections do not transfer automatically.
Zed is not an installation target.

## Add capabilities to a project

Run these commands from the harness checkout, with the absolute path to your
project. Inspect that project's instructions and current Git changes first.

```sh
python3 ai.py project init --project /absolute/project --target all
python3 ai.py project add --project /absolute/project --skill svg-creation --skill motion-design --dry-run
python3 ai.py project add --project /absolute/project --skill svg-creation --skill motion-design
python3 ai.py project plan --project /absolute/project
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Use only the client targets and specialists that fit the project. Init adds the
small `project-foundation`; add records selections; sync installs verified
payloads; doctor checks them. Existing manifests retain their selections.
Codex project skills go under `.agents/skills`, Claude skills under `.claude/skills`.
Modified and unselected copies are preserved rather than silently deleted.

Browse the [skill catalog](docs/catalog.md) and [target selection guide](docs/project-targets.md).
Some optional skills are fetched from pinned upstream sources and require network
access. Review their dependencies and licenses before selecting them.

## Run the workbench

Start by discovering which tools are available:

```sh
python3 ai.py workbench doctor
```

| Operation | Optional prerequisites |
| --- | --- |
| Browser scenarios | Node.js, Playwright and a compatible Chromium browser |
| Vector rendering | Node.js, sharp and Python |
| Office inspection | Python standard library |
| Office rendering | LibreOffice, Poppler (`pdfinfo`, `pdftoppm`) |
| Office demo authoring | python-docx, python-pptx and openpyxl, plus renderers |
| Markdown checks | Python standard library |

Configure existing executable and module paths with `workbench configure`; they
stay in `~/.agent-harness/toolchain.json`, outside the repository. Nothing is
downloaded automatically. The [workbench guide](docs/workbench.md) documents exact
configuration, environment overrides, scenario schema and evidence limits.

To try the included UI, run this in one terminal:

```sh
python3 -m http.server 4173 --bind 127.0.0.1 --directory examples/craft-lab
```

Open `http://127.0.0.1:4173`. With the optional tools configured, run in another
terminal from the same checkout:

```sh
mkdir -p build
python3 ai.py workbench browser --scenario examples/craft-lab/scenario.json --output build/browser-review
python3 ai.py workbench vector examples/craft-lab/fieldwork.svg --output build/vector-review
python3 ai.py workbench documents demo --output build/office-demo
python3 ai.py workbench markdown README.md --output build/readme-review
```

Use a new output directory each time; existing artifacts are never overwritten.
On Windows create `build` with `New-Item -ItemType Directory -Force build`.
Open the generated PNGs and editable originals, inspect the reports, and fix
observed problems. Stop the local server when finished. Generated artifacts are
ignored by Git. Raster image generation uses an available host tool or your
separately connected provider; this repository does not supply an image model.

## Optional Codex plugin

The same source can be packaged as a single `personal-workbench` plugin:

```sh
mkdir -p build
python3 ai.py export --bundle personal-workbench --output build/personal-marketplace
codex plugin marketplace add ./build/personal-marketplace
```

Use the supported desktop plugin interface to install and verify it. This bundle
contains the global workflows, SVG/motion skills, executable helpers and examples.
Avoid enabling duplicate plugin and direct-install copies of the same skills.
Plugin packaging does not install global instruction files, provision runtimes or
make quality checks automatic. See [plugin setup and updates](docs/plugins.md).

## Customize and update

| File or directory | Purpose |
| --- | --- |
| [instructions/AGENTS.md](instructions/AGENTS.md) | Shared working principles |
| [skills/](skills/) | Capability instructions, references and helpers |
| [registry/harness.json](registry/harness.json) | Global selections, project defaults and plugin bundles |
| [registry/targets.json](registry/targets.json) | Client destinations and native settings |
| [registry/mcp.json](registry/mcp.json) | Shared MCP definitions |
| [registry/catalog.json](registry/catalog.json) | Reviewed project skills, profiles and source pins |

Fork this repository to maintain your own choices. Keep credentials, personal
configurations and generated reports outside Git. Review upstream changes before
pulling: linked skills immediately reflect edits to the checkout. For managed
copies, rerun installation; for project skills, run plan, sync and doctor.
There is no automatic update schedule by default.

If installation reports a conflict, preserve the existing content and review the
proposed change. Do not delete files to force a pass. Recovery, backup behavior
and isolated-home rehearsals are documented in [setup](setup/README.md#ownership-and-recovery).
There is no general uninstall command; remove only confirmed harness-owned links
or files and restore the relevant configuration backup after reviewing it.

## Development and verification

```sh
python3 scripts/render_registry.py --check
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
```

Run the shell syntax check on a POSIX shell. Browser/vector integration tests need
their optional runtimes and explicitly skip when unavailable. The latest recorded
macOS run passed **552 tests** with no skips; see the [verification record](docs/research/verification-2026-10-09.md)
for scope and limits. Windows client installation is supported, but the recorded
artifact-rendering evidence is from macOS, not a Windows runtime certification.

For changes, keep target adapters separate from shared capabilities, add behavior
checks for consequential changes, update affected documentation and preserve
upstream attribution. Never make a failing check pass by weakening it.

## License

Original harness code, documentation and examples are available under the
[MIT License](LICENSE). Third-party and adapted material retains its own terms;
see [third-party notices](docs/third-party-notices.md), per-file attribution and the
[source review](docs/source-review.md). Catalog entries do not relicense upstream
skills, models, services or optional tools.

Historical Windmill skill sources retained in Git history remain under AGPLv3,
as documented in the third-party notices; they are not part of the current install.
