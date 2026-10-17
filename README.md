# Mirket

Mirket is a local agent harness for **Codex Desktop** and **Claude Desktop Code**.
One Rust CLI installs reviewed expertise and local MCP packages, coordinates
project work and checks the evidence attached to a task.

```text
mirket setup       Choose clients and configure your environment
mirket doctor      Check installation, configuration and selected packages
mirket update      Update the CLI and reapply your saved setup
```

The runtime contains shared instructions, authored specialist skills and outcome
contracts discoverable through `mirket capabilities list`. The core starts without a source checkout, Python, Node or a background
service. Optional MCP packages bring pinned runtimes when required. Project SDKs
and native applications remain explicit prerequisites.

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

Setup presents client, documentation and optional package choices before applying
them. It installs a managed runtime under
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
mirket capabilities facets
mirket capabilities team "Frontend Engineer" --objective "Build checkout" \
  --facet "web-interface=Checkout screen and browser state change" \
  --facet "service-contract=Order endpoint reconciles retries"
mirket catalog search database --limit 10
mirket catalog read database-systems
```

Capability contracts cover engineering, design, infrastructure, quality,
executive decisions, finance, product, marketing, market analysis, research,
translation, project management, spreadsheets and documentation. Specialized roles
also cover security, accessibility, data pipelines, AI applications, MCP, desktop
apps, user research and experimentation. See [role selection](docs/roles.md). A plan selects
one lead per requested outcome and supplies prerequisites, acceptance criteria
and a failure probe. Each role also has a distinctive professional perspective,
method, decision rule and handoff. For work spanning disciplines, use a team only
when separate independent work would materially help or the user/project
explicitly requests collaboration; cross-discipline scope alone does not require
one. The host interprets the task and supplies applicable facets, then Mirket
assigns related professionals with separate contributions and checks. The host
dispatches actual independent agents, reconciles their findings and verifies the
integrated result. Read the relevant guidance instead of loading the catalog.

The reviewed catalog also contains pinned upstream choices, adaptations and
license information. `mirket catalog show ID` explains an entry's delivery and
requirements. Fetching or installing guidance never executes its upstream helpers.

Design work can also use built-in language and interaction lessons with source/rights
records, original runnable examples, misleading contrasts and transfer exercises.
Search by source or purpose, for example `mirket catalog search "Porsche"` or
`mirket catalog search "statistical uncertainty"`. Results give exact lesson and
example routes under the optional `design-inspiration` skill. See the
[language index](skills/design-inspiration/references/design-languages/index.md).
The existing craft owners implement and verify the result; external brand assets
remain link-only. The library supports inspiration, understanding, comparison and
review. It does not prescribe an aesthetic, reference count or production recipe.
Use project and outside references, combine useful perspectives or proceed without
a reference when existing evidence suffices; explicit user constraints still govern.

Optional structured search describes the library by family, platform,
interactions and access constraints without combining unrelated examples:
`mirket catalog search --family commerce --platform web --interaction cart-checkout --constraint keyboard`.
The result exposes partial and unrepresented combinations. A worked web example,
source-only native guidance and validation of the actual task remain distinct.
See the [coverage guide](skills/design-inspiration/references/design-languages/coverage.md)
and [transfer evaluation](evals/design-coverage/README.md).

## Adopt a project

```sh
mirket project init --project /absolute/project --target all
mirket project add --project /absolute/project --profile project-foundation
mirket project add --project /absolute/project --capability "Backend Engineer"
mirket project plan --project /absolute/project
mirket project sync --project /absolute/project
mirket project doctor --project /absolute/project
```

Use `mirket project remove` to remove an explicit selection, `mirket project
configure --target codex|claude|all` to choose clients, and `mirket project sync
--prune` to remove unselected managed copies after integrity checks.

Project selection lives in `.mirket/project.json`. `project init` records target
hosts without selecting project skills. Add `project-foundation` explicitly for
an engineering or release boundary, then add specialties justified by the
project. When the selected hosts have no shared installation, explicitly choose
the `portable-foundation` profile to copy shared guidance into the project.
Target membership is explicit and installed copies have integrity
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
semantic correctness, reviewer identity or human approval. The coordination
server cannot select a host model, expand host permissions, spawn agents or
execute shell commands. The host retains execution, tools and session lifecycle.
See [workflow and MCP](docs/workflows.md) for the CLI equivalents and boundaries.

## Add local MCP capabilities

Select reviewed packages through the same CLI:

For an existing installation whose saved package selection needs replacement,
run the new binary's `mirket setup --target all --package design --yes` after
starting the [private Podman environment](docs/podman.md). This replaces the
selection and reconciles owned client bindings. The commands below add the
package to a fresh installation or reuse the current design selection.

```sh
mirket package list
mirket package install design
mirket package start design
mirket package status
mirket package probe design --schemas
```

The single `design` package covers Style Dictionary tokens, Satori plus resvg,
Browserless, official Penpot MCP, Kroki, HyperFrames, Typst, imgproxy, SVGO,
ComfyUI and Odiff. The services share a fixed writable workspace reported by
`package start` and `package status`. Direct design tools use paths relative to
its `/workspace` mount; the official Penpot `export_shape` and `import_image`
`filePath` arguments use absolute `/workspace/...` paths as required by their
upstream schemas. Start and status also report assigned loopback host ports;
the service port numbers in the design topology are internal to its private
network. On a fresh home,
`setup --yes` selects no optional packages; saved choices are reused by setup and
update.

The selected profiles require no paid API. Browser navigation can use the network.
Native applications and a running Podman environment remain prerequisites
where needed. `package status` checks files, `package probe` checks MCP discovery,
and a real task establishes useful output. See [local MCP packages](docs/packages.md)
for output policies, platform limits and runtime prerequisites.

See [packages](docs/packages.md) for routing, ports, lifecycle and runtime
requirements. Compile design tokens before rendering; inspect actual artifacts,
Browserless DOM and 2x screenshots, and comparable Odiff output.

## Register other executables

Approved local executables can be registered and invoked through the CLI:

```sh
mirket tool register renderer --executable /absolute/renderer
mirket tool doctor
mirket tool run renderer -- --help
```

Registration pins the executable's bytes. A changed binary requires inspection
and explicit registration again. Execution inherits the CLI's host permissions;
there is no implicit package download, shell evaluation or sandbox. This registry
is for explicit local executables; curated MCP packages use `mirket package`.

## Develop and verify

After the first CLI build, use its development commands:

```sh
mirket dev check --project .
mirket dev build --project . --release
mirket benchmark --iterations 200
mirket dev dist --project . --output /absolute/new-dist
```

[Task evidence](docs/task-evidence.md) documents checkpoint recovery and structured
verification/contribution receipts. The [role workflow suite](evals/role-workflow/README.md)
provides eight scenarios and validates saved host trial reports.

The [architecture guide](docs/architecture.md) explains responsibilities,
[contribution guide](CONTRIBUTING.md) covers isolated checks, and
[source review](docs/source-review.md) describes catalog admission.
After an approved production dependency change, `mirket dev lock --project .`
reconciles the workspace lockfile. Real package tests are opt-in and require an
explicitly marked disposable installation.
Measurements cover named workloads and the machine that ran them; they do not
claim universal model-quality or latency improvements.

Original material is [MIT licensed](LICENSE). Imported guidance and package
components retain their [source notices](docs/third-party-notices.md), licenses
and provenance.
