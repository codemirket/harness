# Installation and updates

Download the Mirket executable for the machine from the
[latest release](https://github.com/codemirket/harness/releases/latest). Asset
names include the OS and architecture: `aarch64` is ARM64 and `x86_64` is Intel/AMD
64-bit. Rename the selected `mirket-TARGET` file to `mirket` (`mirket.exe` on
Windows), make it executable on macOS or Linux with `chmod +x mirket`, add its
directory to PATH and run `mirket setup`. Each release includes `SHA256SUMS` for
the downloadable binaries, plus `LICENSE`, `NOTICE.md` and Rust dependency
notices accompanying the native distributions. Source installation uses
`cargo install --locked --path . --bin mirket`; normal use needs only the compiled
binary.

Setup prompts for Codex, Claude or both, optional Microsoft Learn docs, local MCP
and editor packages, and confirmation. `--yes` uses explicit flags or saved
choices without prompting. A fresh unattended setup selects no optional packages:

```sh
mirket setup --target all --yes
mirket setup --target codex --microsoft-learn --yes
mirket setup --target all --package doc-scraper --package jscpd --yes
mirket setup --dry-run --target all
mirket doctor
```

Repeated `--package` flags replace the saved package selection. With no package
flags, setup reuses saved choices. `--no-packages` clears the selection. Package
installation retrieves pinned artifacts and required runtimes without executing
package managers or lifecycle hooks. See [local MCP packages](packages.md) for
configuration, platform support and invocation checks.

Node and CPython are private package dependencies; installing them does not
replace system runtimes. MarkItDown's text/Office/PDF profile is available on
macOS 14+ and Linux glibc 2.28+ for ARM64 and x64, with Windows unavailable.
DBHub's local SQLite profile is also macOS/Linux only. Playwright has no native
Windows ARM64 browser package. Device SDKs, simulators and applications used for
document rendering remain separate prerequisites.

Inkscape and Glaxnimate are selectable native editor packages with Mirket's Rust
MCP adapters. Their complete application bundles supply the required native
libraries and private runtimes; no separate editor or Python installation is
needed. Both profiles are macOS ARM64 only: Inkscape requires macOS 11.3 or later,
and Glaxnimate requires macOS 13.0 or later. Native checks cover a logged-in
macOS 26.6.2 ARM64 session; other macOS versions and native host activation
require separate verification. The workflow uses command-line rendering and
Qt offscreen animation without opening a GUI editing session. No external X11
server was used; operation without a logged-in session or WindowServer is
unverified. Live SVG text still needs the intended fonts available to the editor.
Inspect the tool schemas and follow the
[editable source walkthrough](packages.md#create-vector-graphics-and-animation):

```sh
mirket package install inkscape glaxnimate
mirket package probe inkscape --schemas --project /absolute/project
mirket package probe glaxnimate --schemas --project /absolute/project
```

Use an existing writable project directory. Editor artifact arguments are
project-relative paths; sources and exports are distinct, and overwriting an
existing output requires explicit intent. Each editor process uses a private
profile and a 30-second deadline. These application controls do not replace host
permissions or create an operating-system sandbox.

Editor installation downloads the official signed DMG, mounts it read-only,
copies the selected app and verifies the copied signature before publishing the
package generation. Valid relative links inside the bundle are preserved. A
file-provider-managed location, including some Documents folders, can attach
metadata that invalidates the copied signature. Mirket refuses that installation
without stripping metadata or re-signing the app. For a disposable `--home`, use a
normal local directory outside those managed folders. The default home normally
stores the packages under `~/.mirket`; `--home` redirects client configuration as
well as package storage, so it is not merely a cache-location option.

OpenAI Docs is configured by setup. Microsoft Learn is optional. These servers
are documentation integrations; setup does not log in or invoke their tools.

| Target | Instructions | Skills | MCP configuration |
| --- | --- | --- | --- |
| Codex Desktop and CLI | `~/.codex/AGENTS.md` | `~/.agents/skills/` | `~/.codex/config.toml` |
| Claude Desktop Code and CLI | `~/.claude/CLAUDE.md` | `~/.claude/skills/` | `~/.claude.json` |

The coordination server entry launches the absolute installed binary with
`mcp serve`. Each selected package has a separate `mirket-package-ID` entry using
that same executable with `package serve ID`.
Mirket preserves unrelated settings, credentials, servers and permission choices.
Start a new host session after setup. A successful doctor checks installation;
actual host discovery and useful tool invocation need a native session.
The targets use the hosts' shared configuration: [Codex MCP](https://developers.openai.com/codex/mcp)
and [Claude Desktop Code configuration](https://code.claude.com/docs/en/desktop#shared-configuration).

### Editor connection budgets

For the two editor server entries, setup writes Codex
`startup_timeout_sec = 90` and `tool_timeout_sec = 120`. These are per-server
seconds fields in the [official Codex MCP configuration](https://developers.openai.com/codex/mcp#other-configuration-options)
and [configuration schema](https://developers.openai.com/codex/config-schema.json).
The startup budget includes package integrity checks and application startup;
the tool budget permits operations with several native steps, such as PDF export
and validation. Each individual native editor process still has a 30-second
deadline. Mirket's CLI editor connection also allows 90 seconds for startup.

Claude editor entries use `timeout: 120000`, the per-server tool deadline in
milliseconds documented by [Claude Code MCP](https://code.claude.com/docs/en/mcp).
Its separate host startup timer is `MCP_TIMEOUT`, documented with a 30000 ms
default in the [environment reference](https://code.claude.com/docs/en/env-vars).
The per-server tool field does not change that startup timer. Mirket preserves
the host's global environment; a variable supplied only to the server child
cannot configure the parent host's timer. Desktop's
[environment configuration](https://code.claude.com/docs/en/desktop#environment-configuration)
also differs from arbitrary shell exports. These settings describe supported
connection budgets, not evidence of native-client activation.

## Standalone target installers

`setup/codex.rs`, `setup/claude.rs` and `setup/all.rs` compile to standalone
`install-codex`, `install-claude` and `install-all` executables. They contain the
same complete Rust runtime and embedded content. Their default operation is
setup for the named target; the installed executable is `mirket`.

They accept setup options such as `--yes`, repeated `--package`, `--no-packages`
and global `--home`. The same sources
build for macOS, Linux and Windows. Use an executable compiled for the device's
OS and architecture. Native testing coverage is recorded by CI and the release
checks; a cross-platform source file is not proof of native behavior.

## Update

`mirket update` retrieves the official GitHub release manifest, selects this
binary's target, checks the SHA-256 digest and version, then launches the verified
candidate to reapply saved setup. `mirket update --check` reports availability
without installing. The repository must have a published release with
`mirket-release.json` and its listed executables; an absent release is an explicit
error and does not change the installation.

For a separately obtained, trusted local release executable:

```sh
mirket update --from /absolute/mirket --sha256 EXPECTED_SHA256
```

A checksum downloaded from the same release establishes transfer integrity; it
is not an independent release signature. Downloads use the official release
origin and HTTPS. Never replace the expected digest just to accept a mismatch.

A bootstrap CLI on PATH forwards to a verified newer managed release, including
new commands and version output. Published versions must be immutable.

Updates reapply selected global setup, package choices and package settings using
the new executable's embedded definitions. They do not follow upstream package
releases independently. Project skill selections remain explicit;
use `mirket project sync` on the project whose payloads you intend to reconcile.
A conflicting managed edit stops setup. Inspect and preserve it before retrying.
Do not grant overwrite authority by deleting a receipt.

## Diagnosis and isolation

`mirket doctor --json` returns structured installation, configuration, package
integrity and registered-tool checks. `--project PATH` includes that project's
managed-copy verification.
`mirket tool doctor` checks registered executables for missing or changed bytes.
`mirket package status` checks selected package files; `mirket package probe ID`
starts a package and checks its permitted MCP tool discovery. Add `--schemas`
to inspect the filtered arguments before invoking tools. Verify meaningful
tool output and native-client invocation separately. Doctor does not start package
servers, launch a model, authenticate accounts or claim activation.

For experiments, pass `--home` with an existing disposable absolute directory.
It redirects the managed runtime and both clients' installation files. Use a
separate project root for task evidence. Remove the disposable directories when
finished; do not run test setup against your actual home by accident.
The [contribution guide](../CONTRIBUTING.md#exercise-installed-packages) describes
the explicit disposable-home marker and opt-in live package tests.
