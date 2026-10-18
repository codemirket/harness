# Install Mirket with an agent

This is the execution guide for an agent acting on a user's request to install
Mirket. The public [installation prompt](../website/public/install-prompt.txt)
links here. Use the user's selected host and existing permissions. Installation
configures the harness; it does not authorize rewriting application code,
changing providers, logging into accounts or installing optional runtimes.

## 1. Inspect before changing anything

Identify the operating system, native architecture, active host (Codex Desktop,
Claude Desktop Code, or their CLI), available terminal tools and existing Mirket
executable. Inspect existing setup through Mirket, without printing credentials
or unrelated private configuration. Preserve saved host and package choices.

Infer the target from the user's request and active environment. Ask only if the
choice remains ambiguous; do not silently configure both hosts. Use the existing
project's instructions and preserve its uncommitted work. A project directory is
not required for device installation.

If Mirket is already installed, run `mirket --version`, `mirket doctor --json` and
`mirket update --check --json`. A healthy installation at the requested version
needs no replacement. For an authorized update, use `mirket update`, then rerun
health checks. If the user requested a specific release, keep that choice;
`update` normally follows the latest release, not an arbitrary version flag.
Read [updates and recovery](installation.md#update) before repairing drift or
upgrading installations with existing task state.

## 2. Obtain and verify the official release

Use only the [codemirket/harness releases](https://github.com/codemirket/harness/releases)
and their attached artifacts. Resolve the requested release (latest stable when
unspecified), record its tag, and use assets from that same release throughout.
Read its `mirket-release.json` manifest and select the exact OS/architecture
match. Validate that the binary URL belongs to that repository's GitHub release,
uses HTTPS, and refers to the selected release. Never execute a downloaded file
before verification.

For the current 1.0.4 release, published binaries cover:

| Device | Manifest target |
| --- | --- |
| macOS, Apple Silicon | `aarch64-apple-darwin` |
| Linux, x86-64 | `x86_64-unknown-linux-gnu` |
| Windows, x86-64 | `x86_64-pc-windows-msvc` |

The manifest for the selected release is authoritative. Do not substitute a
binary for another architecture. If no artifact matches, explain the gap and
consult the source-build option in [manual installation](installation.md).
Installing a missing compiler or other prerequisites needs the user's applicable
authorization; source installation is not a silent fallback.

Download the selected binary to a fresh temporary directory. Compute its SHA-256
with the host's available tool (`shasum -a 256`, `sha256sum`, or PowerShell
`Get-FileHash -Algorithm SHA256`) and compare it exactly with the manifest.
Stop on a mismatch or missing checksum. Same-release `SHA256SUMS` is also
published. These digests establish artifact integrity against the release;
they are not an independent publisher signature.

Rename the verified executable to `mirket` (`mirket.exe` on Windows), set its
executable bit on Unix, and invoke `--version` by its absolute path. Check that
it matches the selected release. Keep applicable release notices with any
retained downloaded distribution. Do not disable OS security checks to force
execution. If the platform requires user action, explain the exact action.

## 3. Apply the selected setup through Mirket

Use the verified executable's `setup --help` if necessary. Select `codex`,
`claude`, or `all` according to the user's intent. The following example is for
Codex only; use the absolute executable path until PATH has been established:

```sh
mirket setup --dry-run --target codex
mirket setup --target codex --yes
```

Inspect the dry-run outcome before applying it. A conflict needs inspection and
preservation of unique edits, not deletion of receipts or blind overwriting.
Setup owns the managed executable, guidance and MCP configuration. Do not create
an alternate installer or manually reconstruct those files.

A fresh unattended setup selects no optional packages. Existing saved package
choices are reused when package flags are omitted. Do not pass `--no-packages`
unless clearing the saved selection is intended. Do not add the `design`
package, Podman or any other prerequisite without an explicit selection. See
[packages](packages.md) when the user requests that capability.

Use the resulting managed executable at `~/.mirket/bin/mirket` (the corresponding
user-profile path and `.exe` on Windows) for subsequent checks. If needed, add
that directory to the user's PATH using the appropriate shell or user-scoped
platform mechanism, preserving existing entries and following host permissions.
Verify resolution in a new shell. If PATH cannot be changed, provide the working
absolute path and state the remaining step. Never leave the only entrypoint in
a temporary download directory.

## 4. Verify installation and activation separately

Run the managed executable's `--version`, `doctor --json` and `catalog check`.
Resolve actionable failures within the installation scope. Do not claim that a
warning disappeared merely because the command succeeded. For selected packages,
check readiness and discovery separately from actual artifact production.

A running host session may not reload new MCP configuration. Ask the user to open
a fresh session when necessary. In a host where Mirket is available, invoke
`catalog_search` for a relevant topic and `skill_read` for an embedded result.
Verify that the tool returned useful content. This establishes activation in
that host; it does not establish activation in every configured host.

If this chat is attached to a project, inspect its instructions, registration and
`.mirket/project.json`. Recommend focused expertise based on its actual stack
and work. Device installation alone does not authorize application changes.
When project setup is also authorized, use `mirket project init`, explicit
selection, `project plan`, `project sync` and `project doctor` as described in
[the project workflow](workflows.md). Preserve current selections and managed
edits; do not bulk-install the catalog.

## 5. Report the observed outcome

Report the installed version, executable path, configured hosts, preserved or
selected optional packages, checks and their results, and whether live activation
was verified. Identify any remaining fresh-session step or prerequisite. Remove
only task-owned temporary downloads after the managed installation is verified.
Do not describe an incomplete or unavailable check as passed.
