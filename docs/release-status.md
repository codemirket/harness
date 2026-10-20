# Mirket 1.0.11: versions and supported targets

This is the **1.0.11** source and documentation reference, prepared on 2026-10-20.
The [1.0.11 release page](https://github.com/codemirket/harness/releases/tag/mirket-v1.0.11)
is authoritative for publication status, native artifacts and verification notes.
Use its `mirket-release.json` or `SHA256SUMS` to verify downloads. A source commit
or a running CI job is not a published binary.

| Distribution | Targets and requirements |
| --- | --- |
| Core CLI and standalone installers | macOS ARM64 (`aarch64-apple-darwin`), Linux x86-64 (`x86_64-unknown-linux-gnu`), Windows x86-64 (`x86_64-pc-windows-msvc`) |
| Optional design and agent-runtime packages | Supported private Podman environment with Linux ARM64 containers. An x86-64 CLI does not imply x86-64 container support |
| Source | [Versioned source](https://github.com/codemirket/harness/tree/mirket-v1.0.11), with the locked Rust toolchain and dependency graph |

Resolve [the release list](https://github.com/codemirket/harness/releases) again
before installing. For an older installation, use its versioned documentation;
[1.0.5 documentation](https://github.com/codemirket/harness/tree/mirket-v1.0.5/docs)
remains available. New commands below require 1.0.11 unless otherwise documented.

## What 1.0.11 adds

| Capability | Contract and limit |
| --- | --- |
| Medium-aware production | Supplied targets or inspected native specimens; shared 15/30/45-minute budgets and two/three/four final repair rounds. [Visual workflow](visual-quality.md) |
| Live identity | Main process executable/source/catalog fingerprints and installed-byte comparison; direct design responses include readable module identity. Each connection needs its own call. [Activation](agent-installation.md#4-verify-installation-and-activation-separately) |
| Host verification | Bind declared inputs before running checks; retain immutable attempts and mark changed evidence stale. [Evidence](task-evidence.md) |
| Project memory | Mem0 OSS 2.2.1 with local embeddings, explicit provenance, expiry and supersession. Candidate expansion stops at 256 and reports incomplete recall. [Runtime](agent-runtime.md) |
| Durable workers | LangGraph 1.2.16 and SQLite checkpointer 3.1.2, with scoped dispatch and bounded resumption. Uncertain effects are not blindly replayed. [Runtime](agent-runtime.md) |
| Recovery | Restorable durable-state archives and relocation of an original renamed directory. Assets/settings need separate backups; cross-filesystem copies are unsupported. [Recovery](agent-runtime.md#restorable-backups-and-project-relocation) |
| Typed lifecycle | Object cursors and retention plans through MCP; previews delete nothing and apply rejects stale plans. [Desktop contracts](agent-runtime.md#desktop-contracts-and-bounded-recall) |
| Release acceptance | Explicit local gate for contained runtime, desktop contracts and backup recovery. It supplements standard checks. [Contributing](../CONTRIBUTING.md#contained-release-acceptance) |

## Verification and upgrade boundaries

The native [CI workflow](https://github.com/codemirket/harness/actions/workflows/check.yml)
runs formatting, Clippy, standard tests, packaging, each standalone installer,
disposable setup/doctor and managed updates on all three release targets. Release
notes identify the successful artifact-producing run. The separate ARM64 contained
gate exercises real memory/workflow behavior, typed MCP calls, parallel operations,
design output and backup recovery. Core native CI does not run those optional
containers on every platform.

Task databases migrate to schema 4. Preserve consistent state backups before
upgrading older installations; a binary-only downgrade cannot read the newer
schema. Read [migration and recovery](installation.md#updating-older-task-databases).
Reconnect desktop MCP sessions after an update and verify each actual connection.
Neither native CI nor a package receipt proves that a user's app reconnected,
that retrieval works on every project, or that a visual result satisfies its brief.
