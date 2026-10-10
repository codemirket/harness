# Installation and updates

Download the Mirket executable for the machine from the
[latest release](https://github.com/codemirket/harness/releases/latest). Asset
names include the OS and architecture: `aarch64` is ARM64 and `x86_64` is Intel/AMD
64-bit. Rename the selected `mirket-TARGET` file to `mirket` (`mirket.exe` on
Windows), make it executable on macOS or Linux with `chmod +x mirket`, add its
directory to PATH and run `mirket setup`. Each release includes `SHA256SUMS` for
the downloadable binaries. Source installation uses
`cargo install --locked --path . --bin mirket`; normal use needs only the compiled
binary.

Setup prompts for Codex, Claude or both, then optional Microsoft Learn docs and
confirmation. `--yes` uses explicit flags or saved choices without prompting:

```sh
mirket setup --target all --yes
mirket setup --target codex --microsoft-learn --yes
mirket setup --dry-run --target all
mirket doctor
```

OpenAI Docs is configured by setup. Microsoft Learn is optional. These servers
are documentation integrations; setup does not log in or invoke their tools.

| Target | Instructions | Skills | MCP configuration |
| --- | --- | --- | --- |
| Codex Desktop and CLI | `~/.codex/AGENTS.md` | `~/.agents/skills/` | `~/.codex/config.toml` |
| Claude Desktop Code and CLI | `~/.claude/CLAUDE.md` | `~/.claude/skills/` | `~/.claude.json` |

The local server entry launches the absolute installed binary with `mcp serve`.
Mirket preserves unrelated settings, credentials, servers and permission choices.
Start a new host session after setup. A successful doctor checks installation;
actual host discovery and useful tool invocation need a native session.
The targets use the hosts' shared configuration: [Codex MCP](https://developers.openai.com/codex/mcp)
and [Claude Desktop Code configuration](https://code.claude.com/docs/en/desktop#shared-configuration).

## Standalone target installers

`setup/codex.rs`, `setup/claude.rs` and `setup/all.rs` compile to standalone
`install-codex`, `install-claude` and `install-all` executables. They contain the
same complete Rust runtime and embedded content. Their default operation is
setup for the named target; the installed executable is `mirket`.

They accept setup options such as `--yes` and global `--home`. The same sources
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

Updates reapply selected global setup. Project skill selections remain explicit;
use `mirket project sync` on the project whose payloads you intend to reconcile.
A conflicting managed edit stops setup. Inspect and preserve it before retrying.
Do not grant overwrite authority by deleting a receipt.

## Diagnosis and isolation

`mirket doctor --json` returns structured installation, configuration and tool
checks. `--project PATH` includes that project's managed-copy verification.
`mirket tool doctor` checks registered executables for missing or changed bytes.
Doctor does not launch a model, authenticate accounts or claim activation.

For experiments, pass `--home` with an existing disposable absolute directory.
It redirects the managed runtime and both clients' installation files. Use a
separate project root for task evidence. Remove the disposable directories when
finished; do not run test setup against your actual home by accident.
