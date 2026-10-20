# Installation and updates

> **Version scope:** This guide describes Mirket 1.0.11. Use the matching release manifest and [version guide](release-status.md); older binaries may not support every command.

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
before execution, make it executable on macOS or Linux with `chmod +x mirket`,
add its
directory to PATH and run `mirket setup`. Each release includes `SHA256SUMS` for
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
mount. The official Penpot `export_shape` and `import_image` `filePath` arguments
require absolute `/workspace/...` paths. Design calls do not use `--project` or
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

When replacing a saved package selection, first run the new release binary's
`mirket setup --target all --package design --yes` with its private Podman
environment running. Setup reconciles the owned bindings and saves the new
selection. An update reuses saved selections; it does not choose replacements.

`mirket update` retrieves the official GitHub release manifest, selects this
binary's target, checks the SHA-256 digest and version, then launches the verified
candidate to reapply saved setup. `mirket update --check` reports availability
without installing. The repository must have a published release with
`mirket-release.json` and its listed executables; an absent release is an explicit
error and does not change the installation.

### Updating older task databases

Before updating an installation with existing tasks, quiesce Mirket task-state
writers and retain a consistent SQLite backup of `~/.mirket/state.db`. SQLite's
backup API includes committed WAL data; copying only an active database file is
not a complete backup. Rehearse recovery in a disposable home.

The current runtime's first task-state or MCP open migrates valid schema-1,
schema-2 or schema-3 databases transactionally to schema 4. Projects, tasks,
deliveries, evidence, checkpoints, contracts and recorded idempotency replies
retain their values. Earlier migrations add evidence contracts and the
creation-time/ID pagination index. Schema 4 adds `project_scopes`, retaining each
project's original path-derived runtime namespace and directory identity.
Current schema-4 opens validate without writing a new schema. Verify the migrated
records and database integrity before resuming writers.

New task mutations return versioned compact receipts. Use task status/history
for full state and recovery. Exact retries of pre-upgrade operations return their
original recorded reply, which can be a full snapshot. See [task evidence](task-evidence.md).

Embedded skill notices are now scoped to each owner's declared sources. This
one-time change invalidates older aggregate delivery hashes: resume an active
task with status, then reread its selected guidance with the current revision
and a fresh key before completion. Task/evidence/history records are retained.
Future unrelated notice updates no longer invalidate that owner's payload.

Restart existing Mirket MCP connections during cutover. Replacing the executable
does not replace an already-running process. Runtimes predating schema 4 refuse the new database;
a binary-only downgrade cannot read the new database. Preserve newer state and
reconcile post-backup writes before restoring an old backup. New host sessions
use the updated connection.

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
The [contribution guide](../CONTRIBUTING.md#exercise-the-design-stack) describes
disposable live acceptance and the separate JavaScript tool tests.
