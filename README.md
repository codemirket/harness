<a href="https://harness.mirket.dev"><img src="website/public/logo.png" alt="Mirket organization logo" width="80" height="80" /></a>

# Mirket

**Expertise and tools for Codex and Claude.**

Mirket is an open-source, local harness for **Codex Desktop** and **Claude Desktop
Code**. It adds focused professional guidance, task-scoped tools and inspectable
work records to your existing agent environment.

Your agent still reasons, writes code and uses its host's permissions. Mirket
helps it choose relevant expertise, carry out bounded work and verify what it
produced. It is a Rust CLI and local MCP server—not a replacement chat app or
model subscription.

For visual work, Mirket uses **medium-aware targets**: inspect a supplied image,
a host-generated composition or a native specimen, build the editable result,
then compare actual renders. Quality gates and deliverable-based budgets bound
production and correction.

[Website](https://harness.mirket.dev) ·
[Releases](https://github.com/codemirket/harness/releases/latest) ·
[Contributing](CONTRIBUTING.md) · [MIT license](LICENSE)

**Version availability:** the public download is **1.0.5** as of 2026-10-20.
The verified desktop build **1.0.11** adds the memory, recovery and verification
features documented below; it has not been published as a release or source
update. See [versions and availability](docs/release-status.md) and use the
selected release’s commands and manifest.

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
checksum and follow [manual installation](docs/installation.md). The 1.0.5 release
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
| Visual production | Medium-aware target/final review gates for Mirket-owned design workers | Establish the target, inspect actual renders and record specific defects |
| Continuity | Optional project-bound task records, decisions, revisions and evidence | Keep useful state for long-running work without duplicating an existing workflow |
| Verification | Completion checks for required delivery and current acceptance/failure-probe artifacts | Run real tests, inspect outputs and judge whether they satisfy the request |
| Local production tools | An optional design package with a shared workspace and purpose-built renderers | Produce and inspect the actual artifact |

For example, on a checkout-screen task the agent inspects a visual target, reads
interface and engineering guidance, implements the flow, and compares actual
renders while checking keyboard and error states. Mirket supplies the method and tools; useful results depend on
the agent actually applying them. A task receipt or file hash does not prove
correctness, human approval or a measured improvement in model quality.

## Visual work: target to working artifact

1. **Establish and inspect a target.** Use a supplied reference, an inspected
   native specimen, or a host-generated raster composition suited to the medium. For a focused repair, the
   existing approved screen can be the target. Check the hierarchy, typography,
   assets and intended use before implementation. Freeze the inspected target
   so later output cannot silently replace it.
2. **Build the real artifact.** Translate the target into editable components,
   text, layout and behavior. Keep exact logos and copy in the source. An image
   displayed as the entire interface does not satisfy image-to-code.
3. **Render, compare and review.** Inspect current output against the frozen
   target at matching content, viewport and state. For UI, inspect narrow and
   wide layouts and exercise the real interactions.

The final review records an outcome and specific observations for all six checks:

| Quality gate | What gets inspected |
| --- | --- |
| Composition | Hierarchy, spacing, proportions and crop |
| Typography | Actual fonts, size, wrapping, legibility and contrast |
| Assets | Provenance, rights, sharpness and fit within the composition |
| Responsive layout | Narrow/wide reflow, clipping and reachable controls |
| Behavior | Primary actions, keyboard/focus and relevant states |
| Motion | Requested playback, intermediate states, interruption and reduced motion |

Responsive layout and behavior cannot be waived for working UI; motion cannot
be waived for animation. Allow **one target repair**, then **an initial final
review with deliverable-based limits**: static/vector/diagram work gets
**15 minutes and two repairs**, interfaces/paged work **30 minutes and three**,
motion **45 minutes and four**. Combined work shares the largest limit. At the limit, preserve the output and report unresolved defects.
Starting another worker or generating more variants does not reset the budget.

### Native specimens and image fallback

The agent chooses a native DOM, SVG, pixel, diagram, typeset or editor specimen
when it fits the medium, then renders and inspects it before expansion.
The legacy code fallback also remains available when no image model is usable. That
first inspected render becomes the frozen target; the same final quality gates
and limits apply. This fallback requires no new image provider or model change.
Mirket's design package supplies renderers and image processing, not an image
generation model. Image generation, when available, belongs to the host.

After setup, you can ask:

> Build this interface using Mirket's medium-aware workflow. Inspect my supplied
> target, or render one minimal native specimen with actual components and text.
> Use image generation when the intended artwork benefits from it.
> Keep the result editable, compare actual renders with the frozen target, and
> report the six quality checks. Respect the correction and time limits; report
> unresolved defects instead of starting a sample campaign.

**What is enforced?** Mirket-owned design workers require a declared input and
ordered target/final reviews. The runtime checks evidence identity and freshness,
required check outcomes, review counts and a shared 15/30/45-minute execution deadline.
Native host tools receive the same workflow as guidance; Mirket does not
intercept their calls. Visual observations and fallback reasons remain agent
judgments, so passing the gates is not an aesthetic guarantee or human approval.
See [visual production](docs/visual-quality.md) for the full contract and scope.

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

## Memory, recovery and verification in the newer desktop build

The optional `agent-runtime` uses **Mem0** for explicit project facts and
**LangGraph** for durable worker checkpoints. It shares the main MCP connection;
local recall needs no hosted memory account or inference credentials. Retrieval
filters inactive facts with a bounded candidate search and reports incomplete
recall. Retrieved text never grants permission.

Host verification binds inputs before checks run and marks changed evidence stale.
Durable-state archives preserve coordination records, memory history, checkpoints
and traces; project source, design assets, credentials and desktop settings need
separate backups. Relocation rebinds a renamed original directory while retaining
its namespace. Cross-filesystem copies are not supported by that command.

These features require the [verified 1.0.11 desktop build](docs/release-status.md),
not the current public 1.0.5 download. Read [memory and durable workers](docs/agent-runtime.md),
[verification handoffs](docs/task-evidence.md) and
[recovery](docs/agent-runtime.md#restorable-backups-and-project-relocation).

For an existing matching installation, `mirket package install design agent-runtime`
retains both optional packages. Create a durable archive with
`mirket backup create --output /absolute/new-backup.tar`. Restore requires the
reported SHA-256, original home/runtime and empty durable destinations. Follow
the recovery guide before applying restore or relocation.

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
| Check feature/version availability | [Versions and availability](docs/release-status.md) |
| Browse all reader guides | [Documentation index](docs/README.md) |
| Store project memory, resume workers or recover state | [Agent runtime](docs/agent-runtime.md) |
| Install through an agent | [Agent installation](docs/agent-installation.md) |
| Install manually, update or recover | [Installation](docs/installation.md) |
| Choose expertise | [Roles and skills](docs/roles.md) |
| Configure projects and coordinate work | [Workflows and MCP](docs/workflows.md) |
| Understand task evidence | [Evidence contracts](docs/task-evidence.md) |
| Build visual work from a target | [Image-to-code and quality gates](docs/visual-quality.md) |
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
