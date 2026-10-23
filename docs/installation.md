# Installation and updates

> **Version scope:** Mirket 0.0.1 clean baseline. Confirm published artifacts in the [release guide](release-status.md).

For guided setup, paste the [installation prompt](../website/public/install-prompt.txt)
into your agent. The [agent execution guide](agent-installation.md) covers device
inspection, release verification and activation. This page is the manual and
recovery reference.

## Manual installation

Download the Mirket executable for the machine from the
[latest release](https://github.com/codemirket/harness/releases/latest). Asset
names include the OS and architecture: `aarch64` is ARM64 and `x86_64` is Intel/AMD
64-bit. Rename the selected `mirket-TARGET` file to `mirket` (`mirket.exe` on
Windows), verify its SHA-256 against the same release's manifest or `SHA256SUMS`
before execution. Place the verified executable in `~/.mirket/bin/mirket`
(`%USERPROFILE%\.mirket\bin\mirket.exe` on Windows), make it executable on
macOS or Linux with `chmod +x`, and run setup from that absolute path. Keep
that directory on PATH if desired. This is the installed executable location
on all supported CLI platforms; no separate release-binary location is needed.
Each release includes `SHA256SUMS` for
the downloadable binaries, plus `LICENSE`, `NOTICE.md` and Rust dependency
notices accompanying the native distributions. Source installation uses
`cargo install --locked --path . --bin mirket`; normal use needs only the compiled
binary.

Setup prompts for Codex, Claude or both, documentation integration, optional
packages, and confirmation. `--yes` uses explicit flags or saved
choices without prompting. A fresh unattended setup selects no optional packages:

```sh
mirket setup --target all --yes
mirket setup --dry-run --target all
mirket doctor
```

Repeated `--package` flags replace the saved package selection. With no package
flags, setup reuses saved choices. `--no-packages` clears the selection. Package
preparation uses pinned sources and declared build steps. Serving does not
resolve dependencies or run package installers. See [local MCP packages](packages.md) for
configuration, platform support and invocation checks.

Package runtimes are private dependencies and do not replace system runtimes.
Mirket's per-user data root is `~/.mirket` on macOS and Linux and
`%USERPROFILE%\.mirket` on Windows. Runtime downloads, package generations,
databases, VM disk/images/volumes, the imported Podman client and host-side
Podman runtime paths live under it. Host client configuration, projected skills
and project-local `.mirket` selection files remain in their owning host or
project directories. The managed container controller has been verified on
macOS ARM64; this location contract does not claim VM readiness on other hosts.
The design stack is one `design` package. After setting up the selected Podman
environment, install, start and inspect it:

```sh
mirket package install design
mirket package start design
mirket package status
mirket package probe design --schemas
```

`package start` and `package status` report a fixed host workspace under the
selected Mirket home and the assigned loopback host ports. Service port numbers
in the design topology are internal; use the reported host ports for host access.
Direct design tool paths are relative to its `/workspace`
mount, including OpenPencil document sources and outputs. Design calls do not use `--project` or
a configurable port. See
[packages](packages.md) and the [execution environment](podman.md) for service
topology. Tool calls retain the host's permissions.

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
For builds with `runtime_status`, compare that actual-client response with
`mirket info --json`: executable/source/catalog fingerprints must match and
`installed_binary.matches_process` must be true. Check each desktop connection
separately. Invoke the design or memory tool as well when that package is selected;
main-server identity does not identify a separate package process.
The targets use the hosts' shared configuration: [Codex MCP](https://developers.openai.com/codex/mcp)
and [Claude Desktop Code configuration](https://code.claude.com/docs/en/desktop#shared-configuration).

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

Mirket 0.0.1 is a fresh baseline. It creates the current schema-4 task database
and versioned project identities. It does not migrate databases, project
registrations, package state or managed host receipts from deleted pre-0.0.1
releases. Keep an older installation separate until its owner has explicitly
chosen what data to retain or remove; do not silently bind its projects to a
new installation.

Future releases must publish an explicit migration contract; 0.0.1 does not preserve or reinterpret older runtime state.
`mirket update --check` can inspect a newer published release.
`mirket update` verifies that release's manifest and executable hash before
reapplying saved setup. The installed binary remains under `~/.mirket/bin` on
all CLI platforms. Update keeps saved package selections and refuses modified
managed copies. Restart desktop MCP sessions after an update and compare each
live `runtime_status` with the installed executable. Project selections need
an explicit `mirket project sync` in each project.

For a separately obtained trusted release executable, use:

```sh
mirket update --from /absolute/mirket --sha256 EXPECTED_SHA256
```

A checksum from the release establishes transfer integrity, not an independent
publisher signature. Future releases must retain their own state migration or
explicit fresh-start contract; a lower version number alone cannot safely read
newer data.

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
The [contribution guide](../CONTRIBUTING.md#exercise-the-design-stack) describes
disposable live acceptance and the separate JavaScript tool tests.

An intentional release-number reset uses a local artifact with a verified digest:

```sh
MIRKET_DIRECT=1 /absolute/candidate/mirket update --plan \
  --from /absolute/candidate/mirket --sha256 ARTIFACT_SHA256 \
  --reset-version-from INSTALLED_VERSION
```

Inspect the plan, then replace `--plan` with `--activate`. The reset must name the
exact installed version. Ordinary updates still reject older versions. The shared
setup transaction retains managed-file drift checks, saved client choices and
rollback; it does not translate or erase durable databases. An interrupted
activation must resume with the identical artifact and reset options. After the
candidate is installed, the same request remains resumable only while its exact
bytes are still installed. Reconnect clients and verify their live identities.
