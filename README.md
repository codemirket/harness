# Mirket

Mirket is a local agent harness for **Codex Desktop** and **Claude Desktop Code**.
One Rust CLI installs reviewed expertise, serves a local MCP server, coordinates
project work and checks the evidence attached to a task.

```text
mirket setup       Choose clients and configure your environment
mirket doctor      Check installation, configuration and registered tools
mirket update      Update the CLI and reapply your saved setup
```

The runtime contains its shared instructions, 52 authored skills and 33 capability
contracts. It starts without a source checkout, Python, Node or a background
service. External project tools remain separate, explicitly selected dependencies.

## Install and set up

Download the executable for your operating system and architecture from the
[Mirket releases](https://github.com/codemirket/harness/releases/latest). Rename
it to `mirket` (`mirket.exe` on Windows), make it executable on macOS or Linux,
and put it in a directory on PATH. Then run `mirket setup` and `mirket doctor`.

From a source checkout, install the CLI using the pinned Rust toolchain:

```sh
cargo install --locked --path . --bin mirket
mirket setup
mirket doctor
```

Setup presents client and documentation choices before applying them. It installs a managed runtime under
`~/.mirket/bin`, skill copies and instructions in the selected client locations,
and a local `mirket` MCP entry. A new host session loads the configured tools.

For automation or an isolated test home:

```sh
mirket --home /absolute/test-home setup --target all --yes
mirket --home /absolute/test-home doctor --json
```

The supplied home must exist. `--dry-run` validates setup without installing it.
Setup preserves unrelated client configuration and stops on unmanaged or modified
file conflicts. It does not change model, authentication or permission settings.
See [installation](docs/installation.md) for targets, standalone installers,
updates and recovery.

## Use focused expertise

```sh
mirket capabilities plan "Frontend Designer" --with "QA Engineer for Design"
mirket capabilities plan "CFO" --with "Excel Expert"
mirket catalog search database --limit 10
mirket catalog read database-systems
```

Capability contracts cover engineering, design, infrastructure, quality,
executive decisions, finance, product, marketing, market analysis, research,
translation, project management, spreadsheets and documentation. A plan selects
one lead per requested outcome and supplies prerequisites, acceptance criteria
and a failure probe. Read the relevant guidance instead of loading the catalog.

The reviewed catalog also contains pinned upstream choices, adaptations and
license information. `mirket catalog show ID` explains an entry's delivery and
requirements. Fetching or installing guidance never executes its upstream helpers.

## Adopt a project

```sh
mirket project init --project /absolute/project --target all
mirket project add --project /absolute/project --capability "Backend Engineer"
mirket project plan --project /absolute/project
mirket project sync --project /absolute/project
mirket project doctor --project /absolute/project
```

Use `mirket project remove` to remove an explicit selection, `mirket project
configure --target codex|claude|all` to choose clients, and `mirket project sync
--prune` to remove unselected managed copies after integrity checks.

Project selection lives in `.mirket/project.json`. The foundation supplies
architecture, debugging, testing and release workflows; add specialties justified
by the project. Target membership is explicit and installed copies have integrity
receipts. Project instructions and the user's choices remain authoritative.

For coordination without project skill copies, use `mirket project register`.
Only CLI-registered roots are available to the MCP task tools.
`mirket project unregister ID --yes` removes that registration and its task
history. Project files and installed skill selections are preserved.

## Coordinate work with MCP

`mirket mcp serve` is a local stdio server launched by the configured host. It
provides bounded expertise discovery, skill delivery, durable task state and
artifact evidence. Tasks use optimistic revisions and idempotency keys to detect
conflicting or repeated writes. Completion checks required skill delivery and
current acceptance/failure-probe artifacts.

These checks establish workflow and artifact consistency. They cannot prove
semantic correctness, reviewer identity or human approval. Mirket cannot select
a host model, expand host permissions, spawn agents or execute shell commands
through MCP. The host retains execution, tools and session lifecycle.
See [workflow and MCP](docs/workflows.md) for the CLI equivalents and boundaries.

## Extend the runtime

Approved local executables can be registered and invoked through the CLI:

```sh
mirket tool register renderer --executable /absolute/renderer
mirket tool doctor
mirket tool run renderer -- --help
```

Registration pins the executable's bytes. A changed binary requires inspection
and explicit registration again. Execution inherits the CLI's host permissions;
there is no implicit package download, shell evaluation or sandbox. This registry
provides a single boundary for additional binaries and tool packages.

## Develop and verify

After the first CLI build, use its development commands:

```sh
mirket dev check --project .
mirket dev build --project . --release
mirket benchmark --iterations 200
mirket dev dist --project . --output /absolute/new-dist
```

The [architecture guide](docs/architecture.md) explains responsibilities,
[contribution guide](CONTRIBUTING.md) covers isolated checks, and
[source review](docs/source-review.md) describes catalog admission.
Measurements cover named workloads and the machine that ran them; they do not
claim universal model-quality or latency improvements.

Original material is [MIT licensed](LICENSE). Imported and adapted guidance keeps
its [source notices](docs/third-party-notices.md) and catalog provenance.
