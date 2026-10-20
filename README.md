<a href="https://harness.mirket.dev"><img src="website/public/logo.png" alt="Mirket organization logo" width="64" height="64" /></a>

# Mirket

**A local harness for Codex Desktop and Claude Desktop Code.**

Mirket gives your coding agent focused expertise, tools for specific tasks, and
records of what was checked. It helps structure development, design and research
inside your existing agent environment.

**You set the goal and permissions. Your agent does the work. Mirket supplies
methods, scoped worker tools and verification records.** It runs as a Rust CLI
and local MCP server. Your existing provider, authentication and billing still
apply.

[Website & usage guide](https://harness.mirket.dev) ·
[Documentation](docs/README.md) ·
[Release 1.0.11](https://github.com/codemirket/harness/releases/tag/mirket-v1.0.11) ·
[Contributing](CONTRIBUTING.md)

## What it adds

| Capability | Use it when you need to… |
| --- | --- |
| **Focused expertise** | Find and read relevant engineering, design, research or writing guidance without loading the whole catalog. |
| **Scoped workers** | Delegate independent work to Mirket-owned workers with discovered models and tools limited to the assignment. |
| **Task records** | Retain decisions, progress and verification evidence for work that needs coordination or resumption. |
| **Verification contracts** | Detect missing or changed evidence at completion, alongside the agent’s real tests and artifact inspection. |
| **Optional production tools** | Render and inspect visual work, or add explicit project memory and durable worker checkpoints. |

Routine work can stay direct. Use delegation and durable tracking when they help;
your project’s instructions and explicit choices govern the work. Mirket checks
records and artifact identity; correctness and human acceptance still need real
verification.

## Get started

In Codex or Claude with local file and terminal access, paste the installation
prompt below. It directs the agent to verify the official release, preserve your
settings and run setup checks.

<details>
<summary><strong>Show installation prompt</strong></summary>

<!-- installation-prompt:start -->
```text
Install Mirket for the agent environment I am using on this device. Read
https://raw.githubusercontent.com/codemirket/harness/main/docs/agent-installation.md
and carry out the installation, rather than only explaining it.

Inspect my OS, architecture, available host and any existing Mirket setup.
Use the official codemirket/harness release, verify its checksum before
execution, and preserve my existing settings, credentials and project files.
Use Mirket's own setup and update commands. Ask only for choices or
permissions you cannot infer safely. Keep optional packages opt-in.

Run the health checks, report the installed version and actual results, and
explain any fresh-session step needed to activate the tools. Do not claim
activation until a Mirket tool has been invoked in the host. If this chat
has a project, inspect it and propose the relevant expertise without
changing application code as part of installation.
```
<!-- installation-prompt:end -->

</details>

For manual setup, follow [installation](docs/installation.md). The core binary
needs no source checkout, Node, Python or background service. Check the selected
release’s [supported targets](docs/release-status.md) and verify its checksum
before execution. Optional packages have separate prerequisites.

After setup, start a fresh host session if needed and ask the agent to invoke
Mirket’s `runtime_status` tool. A successful `mirket doctor` checks installation;
a tool call in your host establishes that the connection is active.

## Use it in a project

Open your project in the agent and ask:

> Inspect this project and its instructions. Use Mirket to select and install the
> expertise relevant to its stack and current work. Preserve existing selections
> and edits, verify project health, and explain what you selected. Do not change
> application behavior as part of setup.

Then describe the work as usual. For example:

> Use Mirket’s relevant engineering guidance to fix this failing checkout. Find
> the cause, make the smallest complete change, and run the project checks.
> Explain what changed and what remains untested.

For explicit CLI setup, replace the path, target and capability as appropriate:

```sh
mirket project init --project /absolute/project --target all
mirket project add --project /absolute/project --profile project-foundation
mirket project add --project /absolute/project --capability "Backend Engineer"
mirket project plan --project /absolute/project
mirket project sync --project /absolute/project
mirket project doctor --project /absolute/project
```

`init` records target hosts, `add` selects guidance, and `sync` installs it.
Selections live in `.mirket/project.json`. Use `portable-foundation` when the
hosts have no shared global guidance. See [roles](docs/roles.md) and
[project workflows](docs/workflows.md) for more options.

## Architecture at a glance

The CLI and MCP server share one Rust implementation. The CLI handles setup and
project management; MCP lets your agent discover guidance and use task tools.

| Layer | Responsibility | Source |
| --- | --- | --- |
| **CLI & MCP** | Command dispatch and the local agent-tool connection | `src/cli.rs`, `src/mcp.rs` |
| **Catalog & projects** | Embedded roles, skills, references and project selections | `src/catalog/`, `src/catalog.rs`, `src/project.rs` |
| **Setup & updates** | Managed installation, integrity checks and preserved configuration | `src/install.rs`, `src/update.rs` |
| **State & routing** | SQLite task records, evidence and bounded worker execution | `src/state.rs`, `src/state/`, `src/router/` |
| **Optional packages** | Design tools, memory and durable worker runtime | `src/packages/`, `runtime/` |

Local installation and task state live under `~/.mirket`; project selections and
sync receipts live in the project’s `.mirket` directory. Model calls use configured
targets, which may be remote. The host retains permission authority; Mirket does
not create an operating-system sandbox or intercept native host subagents.

Read the [architecture guide](docs/architecture.md) for the complete module map,
state layout and execution boundaries.

## Optional packages

- **`design`** adds Playwright, OpenPencil, Style Dictionary and tools for diagrams,
  video, documents, vectors and image processing. Its visual workflow starts with
  an inspected target and checks the actual output. See [packages](docs/packages.md)
  and [visual verification](docs/visual-quality.md).
- **`agent-runtime`** adds explicit project memory through Mem0 and durable worker
  checkpoints through LangGraph. See [memory and recovery](docs/agent-runtime.md).

Both require a supported private Podman environment with Linux ARM64 containers.
The core harness works without them. See [runtime prerequisites](docs/podman.md).

## Maintain and contribute

```sh
mirket doctor         # Check installation and configuration
mirket update --check # Check for a release
mirket update         # Update using saved setup choices
```

Update project guidance explicitly with `mirket project sync --project
/absolute/project`. Inspect managed-file conflicts before retrying.

For source development, use the pinned Rust toolchain:

```sh
cargo install --locked --debug --path . --bin mirket
mirket dev check --project .
mirket catalog check
```

[Contributing](CONTRIBUTING.md) covers disposable setup tests and platform checks.
[All guides](docs/README.md) cover installation, evidence, packages and recovery.
The website source lives in [website/](website/README.md).

Original material is [MIT licensed](LICENSE). Imported guidance and runtime
components retain their [source pins and licenses](docs/third-party-notices.md).
