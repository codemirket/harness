# Mirket

**Expertise and tools for the agents you already use.**

Mirket is an open-source, local harness for **Codex Desktop** and **Claude Desktop
Code**. It adds focused professional guidance, task-scoped tools and inspectable
work records to your existing agent environment.

Your agent still reasons, writes code and uses its host's permissions. Mirket
helps it choose relevant expertise, carry out bounded work and verify what it
produced. It is a Rust CLI and local MCP server—not a replacement chat app or
model subscription.

[Website](https://harness.mirket.dev) ·
[Releases](https://github.com/codemirket/harness/releases/latest) ·
[Contributing](CONTRIBUTING.md) · [MIT license](LICENSE)

## Get started with your agent

Paste this into Codex or Claude with local file and terminal access:

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

The agent follows the [installation guide for agents](docs/agent-installation.md):
it inspects your device, verifies the official release, applies setup and checks
the result. Start a fresh host session if needed to load the new tools.

**Prefer a manual install?** Download the matching executable from
[Releases](https://github.com/codemirket/harness/releases/latest), verify its
checksum and follow [manual installation](docs/installation.md). The 1.0.4 release
provides macOS ARM64, Linux x86-64 and Windows x86-64 binaries. Use the selected
release's manifest for available targets.

The core binary needs no source checkout, Node, Python or background service.
You need a supported host and its existing authentication. Optional packages have
separate runtime requirements; a fresh unattended setup installs none of them.

## What Mirket contributes

| Need | What the harness provides | What the agent still does |
| --- | --- | --- |
| Relevant expertise | Discoverable roles, focused skills and source references for engineering, design, research, writing and other work | Select and read useful guidance for the actual task |
| Bounded execution | A routed spawner for Mirket-owned workers, with discovered models and tools scoped to declared task needs | Decide whether delegation helps and integrate the result |
| Continuity | Optional project-bound task records, decisions, revisions and evidence | Keep useful state for long-running work without duplicating an existing workflow |
| Verification | Completion checks for required delivery and current acceptance/failure-probe artifacts | Run real tests, inspect outputs and judge whether they satisfy the request |
| Local production tools | An optional design package with a shared workspace and purpose-built renderers | Produce and inspect the actual artifact |

For example, on a checkout-screen task the agent can read interface and engineering
guidance, implement the flow, exercise keyboard and error states, and inspect the
rendered screen. Mirket supplies the method and tools; useful results depend on
the agent actually applying them. A task receipt or file hash does not prove
correctness, human approval or a measured improvement in model quality.

## Use it in a project

After installation, ask your agent:

> Inspect this project and its instructions. Use Mirket to select and install
> the expertise relevant to its stack and current work. Preserve existing
> selections and edits, verify project health, and explain what you selected.
> Do not change application behavior as part of setup.

The underlying workflow remains explicit and scriptable:

```sh
mirket project init --project /absolute/project --target all
mirket project add --project /absolute/project --profile project-foundation
mirket project add --project /absolute/project --capability "Backend Engineer"
mirket project plan --project /absolute/project
mirket project sync --project /absolute/project
mirket project doctor --project /absolute/project
```

Replace the path, target and capability with the project's actual choices.
`project init` records targets; it does not select skills. Add `project-foundation`
for engineering or release work and only the specialties the project needs.
Use `portable-foundation` explicitly when the hosts lack shared global guidance.
Selections live in `.mirket/project.json`; managed copies have integrity receipts.
Your project's instructions and explicit choices govern the work.

## How it fits together

1. **Setup connects the host.** `mirket setup` installs the managed executable,
   shared instructions, skills and local MCP configuration. It preserves unrelated
   settings and stops on conflicting managed edits.
2. **The agent retrieves expertise.** Catalog and capability tools expose focused
   guidance, source provenance and verification criteria. The catalog is a
   discovery aid; agents can use other relevant sources or no reference.
3. **Work uses the appropriate tools.** Native host tools remain available. When
   Mirket-owned workers are used, the router discovers authenticated targets and
   enforces the assignment's capability and tool requirements. Native host
   subagents are outside that routing boundary.
4. **The result gets checked.** Real application checks and artifact inspection
   establish behavior. Optional task records retain evidence and detect stale
   bindings, conflicting revisions and repeated mutations.

Read [role selection](docs/roles.md), [workflow and MCP](docs/workflows.md),
[worker routing](skills/agent-coordination/references/model-routing.md) and
[task evidence](docs/task-evidence.md) for the detailed contracts.

## Optional design workspace

The single `design` package adds tools for producing and checking visual work:

| Work | Tools |
| --- | --- |
| Design tokens | Style Dictionary |
| Live interfaces and browser inspection | Playwright |
| Editable design files | Official Penpot MCP |
| Diagrams, video and paged documents | Kroki, HyperFrames, Typst |
| Static graphics, vectors and images | Satori, resvg, SVGO, Sharp |
| Pixel comparison | Odiff |

It uses a private Podman environment and a shared local workspace. Install it only
when needed, after checking its prerequisites. Services and tools have distinct
readiness and invocation checks; a successful package probe is not a verified
render. See [the package guide](docs/packages.md) and [Podman setup](docs/podman.md).
The core expertise and coordination features work without this stack.

## Maintain the installation

```sh
mirket doctor         # Check installation, configuration and selected packages
mirket update --check # Check the official release without changing anything
mirket update         # Update and reapply your saved setup
```

Updates preserve saved choices. Project skill updates remain explicit: run
`mirket project sync` for the project you intend to reconcile. Inspect managed
file conflicts before retrying; do not delete receipts to force an overwrite.
See [installation and recovery](docs/installation.md).

## Boundaries worth understanding

- **Local harness, configured inference.** The executable and task state run
  locally. Model calls use your configured targets, which may be remote. Provider
  data policies, authentication, usage limits and billing still apply.
- **Host permissions remain authoritative.** Guidance, role selection and tool
  registration do not grant access or create an operating-system sandbox.
- **Installation and activation are different.** Doctor checks installation and
  protocol behavior. A native host session must discover and invoke a tool to
  establish live activation.
- **Checks are evidence, not guarantees.** Mirket validates declared records and
  artifacts. Semantic correctness, real user outcomes and human acceptance need
  their own verification.

## Documentation

| I want to… | Read |
| --- | --- |
| Install through an agent | [Agent installation](docs/agent-installation.md) |
| Install manually, update or recover | [Installation](docs/installation.md) |
| Choose expertise | [Roles and skills](docs/roles.md) |
| Configure projects and coordinate work | [Workflows and MCP](docs/workflows.md) |
| Understand task evidence | [Evidence contracts](docs/task-evidence.md) |
| Use local production tools | [Packages](docs/packages.md) |
| Understand the implementation | [Architecture](docs/architecture.md) |
| Contribute code or guidance | [Contributing](CONTRIBUTING.md) · [Source review](docs/source-review.md) |

## Development

From a source checkout with the pinned Rust toolchain:

```sh
cargo install --locked --debug --path . --bin mirket
mirket dev check --project .
mirket catalog check
```

Keep the bootstrap executable outside Cargo's build output. Use disposable homes
and projects for installer tests. The [contribution guide](CONTRIBUTING.md)
covers platform checks, package tests and release packaging. The public website
is maintained in [website/](website/README.md).

## License

Original material is [MIT licensed](LICENSE). Imported guidance and package
components retain their own licenses, source pins and
[third-party notices](docs/third-party-notices.md).
