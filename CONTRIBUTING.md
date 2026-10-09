# Contributing to Harness

Contributions that make a real workflow easier to adopt, use or verify are welcome.
Useful starting points include an unclear setup step, a reproducible installer
failure, a missing failure case, or a specialist workflow with a concrete task
and evidence that it helps.

For usage, begin with the [adoption guide](docs/adoption.md). Report reproducible
problems in [GitHub Issues](https://github.com/codemirket/harness/issues), and
propose changes through a pull request from your fork. Remove credentials,
private prompts and identifying project data from examples and logs.

## Find the owning source

| Location | Responsibility |
| --- | --- |
| `ai.py`, `lib/` | CLI, installation, catalog resolution and verification |
| `instructions/AGENTS.md` | Shared working principles |
| `skills/` | Authored workflows, references and helpers |
| `registry/harness.json` | Global selection, project defaults and bundles |
| `registry/targets.json`, `registry/mcp.json` | Client configuration and MCP definitions |
| `registry/catalog.json` | Reviewed entries, profiles, source pins and payload contracts |
| `scripts/render_registry.py` | Generated catalog documentation |
| `tests/` | Automated behavior checks |
| `examples/`, `evaluations/` | Inspectable specimens and task exercises |
| `docs/`, `setup/` | Guides, reference material and dated evidence |

Keep client-specific adapters separate from shared guidance. Existing projects
retain authority over their architecture, commands and release rules. Avoid
expanding the global skill set when a project-selected specialist solves the need.

## Develop without changing your installed clients

Use Git and Python 3.9+ in a separate checkout. The core installer needs no pip
packages. Be aware that a checkout used for a linked installation is live: editing
its shared skills also changes what the installed links expose.

For installer work, create an empty temporary directory and pass its absolute
path explicitly:

```sh
python3 ai.py install --target all --home /absolute/empty-test-home --mode copy --dry-run
python3 ai.py install --target all --home /absolute/empty-test-home --mode copy
python3 ai.py check --target all --home /absolute/empty-test-home --mode copy
```

The home directory must exist. This prepares files there; it does not launch
either client or demonstrate that a live client discovers them. Use a separate
temporary project for `project init`, `add`, `sync` and `doctor` tests.

## Verify the changed behavior

Run these checks from the harness checkout:

```sh
python3 scripts/render_registry.py --check
python3 -m unittest discover -s tests
sh -n setup/macos.sh
git diff --check
```

Run the shell check in a POSIX shell. When changing catalog data, regenerate its
documentation with `python3 scripts/render_registry.py` before the `--check` step.
Optional browser/vector checks may skip if their runtimes are unavailable; report
those skips. Do not install dependencies or connect services as an incidental part
of verification. See the [workbench guide](docs/workbench.md) for tool configuration.

For documentation, execute affected examples in an appropriate isolated environment
and check local links and structure:

```sh
python3 ai.py workbench markdown README.md --output /absolute/new-readme-report
```

The output directory must not already exist. Inspect the rendered Markdown too;
the checker does not execute examples, fetch external links or judge prose.

For installer changes, preserve user modifications, unrelated configuration and
failure diagnostics. Test the affected conflict or recovery path. For workflow
changes, show a representative task and its resulting artifact; installing a
skill or having an agent name it is not proof of a useful outcome.

## Add or update a skill

Start with the [source review process](docs/source-review.md). Review the actual
body, companions, helpers, activation behavior, dependencies and license before
registering an upstream entry. A catalog search result is only discovery data.

Keep the skill focused on a recognizable task. State prerequisites and what a
finished result should demonstrate. Preserve attribution, source pins and notices;
refresh applicable hashes after changing payloads. Test registration in a temporary
project and exercise the intended workflow with the required tools available.

## Make the change easy to review

Describe the problem, the resulting behavior and the checks you actually ran.
Include a small reproduction or before/after example when it clarifies the change.
Identify unverified clients, operating systems and optional runtimes explicitly.
Do not rewrite dated evidence to imply that an older run tested new source bytes.

Original harness material uses the [MIT License](LICENSE). Imported and adapted
material retains its own terms; preserve the [third-party notices](docs/third-party-notices.md)
and any more specific per-file notices.
