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

Setup prompts for Codex, Claude or both, local MCP
and editor packages, and confirmation. `--yes` uses explicit flags or saved
choices without prompting. A fresh unattended setup selects no optional packages:

```sh
mirket setup --target all --yes
mirket setup --target all --package playwright --yes
mirket setup --dry-run --target all
mirket doctor
```

Repeated `--package` flags replace the saved package selection. With no package
flags, setup reuses saved choices. `--no-packages` clears the selection. Package
installation retrieves pinned artifacts and required runtimes without executing
package managers or lifecycle hooks. See [local MCP packages](packages.md) for
configuration, platform support and invocation checks.

Package runtimes are private dependencies and do not replace system runtimes.
Native applications and operating-system prerequisites remain explicit.

Inkscape's complete signed application bundle supplies its native libraries.
The SVG profile covers macOS ARM64, with a declared minimum macOS 11.3; live text
needs available fonts. Headless browser design/QA and media profiles reuse private
Node/Chromium and shared FFmpeg/FFprobe, initially verified on macOS ARM64.
No package manager, upstream installer or first-render download runs while serving.

```sh
mirket package install inkscape axe-core hyperframes motion-canvas
mirket package probe inkscape --schemas --project /absolute/project
```

Container renderers and Penpot require the approved native Podman client selected
through `mirket tool register podman --executable /absolute/path/to/podman`.
Initialize and start the dedicated environment with `mirket environment init` and
`mirket environment start`. These are explicit operations; MCP calls never start
a VM or install host software. See [execution environment](podman.md).
Install `penpot`, then `mirket package start penpot` before probing its tools.
The service has a loopback frontend, durable volumes and private credentials;
its design operations run in an active headless Chromium session.
Use `mirket package stop penpot` to stop owned containers while retaining designs.

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

OpenAI Docs is configured by setup. This documentation integration does not
log in or invoke tools during setup.

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

For a 2.0.0 installation, download and verify the 3.0.0 executable first, then
run that executable's `update` against the existing home. Close the selected
desktop clients before this reconfiguration. The 2.0.0 updater and bootstrap
require receipt fields and saved choices removed in 3.0.0; they cannot verify
the new receipt. Use the resulting managed 3.0.0 executable for further commands.
If PATH contains a separate 2.0.0 executable, replace it during this update with
the entrypoint options below, invoking the downloaded 3.0.0 executable. Preserve
the setup receipt: it identifies owned settings and files for reconciliation.
If the old updater already reported an error after installation, run 3.0.0
`doctor`, then 3.0.0 `update` to complete and verify the saved setup.

To replace a separate CLI file on PATH with the same verified release, run the
managed CLI and identify both the destination and its current SHA-256 digest:

```sh
/absolute/home/.mirket/bin/mirket update \
  --entrypoint /absolute/bin/mirket \
  --expected-entrypoint-sha256 CURRENT_ENTRYPOINT_SHA256
```

The entrypoint must be an existing regular file named `mirket` (`mirket.exe` on
Windows), with an absolute path and no symlinks. Mirket checks its current bytes
before updating the environment and again before replacing the file. A changed
digest refuses replacement; an entrypoint already matching the candidate is a
safe retry. This option cannot be combined with `--check`. On Windows, invoke a
different Mirket executable because a running entrypoint cannot be replaced.
If the environment update succeeds but entrypoint replacement fails, the error
identifies that partial outcome so the replacement can be retried.

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
