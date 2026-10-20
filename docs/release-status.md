# Versions and availability

Checked on **2026-10-20**. Documentation can describe a verified desktop build
before its binaries are published. Installing the latest public release
does not install unpublished features.

| Distribution | State | How to use its documentation |
| --- | --- | --- |
| [Public 1.0.5](https://github.com/codemirket/harness/releases/tag/mirket-v1.0.5) | Published binaries for macOS ARM64, Linux x86-64 and Windows x86-64 | Use the release manifest and [versioned source documentation](https://github.com/codemirket/harness/tree/mirket-v1.0.5/docs); use this tag when you need the 1.0.5 implementation |
| Desktop build 1.0.11 | Built, tested and deployed to an existing macOS ARM64 installation; agent/design containers verified on Linux ARM64 | Newer behavior is documented here for that installation. The 1.0.11 source is on `main`; native release verification and downloadable assets are pending. Do not treat a source build as a published binary |

Always resolve [the release list](https://github.com/codemirket/harness/releases)
again before installing. A release manifest determines available targets; source
portability does not establish native platform verification.

## What the newer desktop build adds

| Capability | Contract and limit |
| --- | --- |
| Medium-aware production | Supplied targets or inspected native specimens; shared 15/30/45-minute budgets and two/three/four final repair rounds. [Visual workflow](visual-quality.md) |
| Live identity | Main process executable/source/catalog fingerprints and installed-byte comparison; direct design responses include readable module identity. Each connection needs its own call. [Activation](agent-installation.md#4-verify-installation-and-activation-separately) |
| Host verification | Bind declared inputs before running checks; retain immutable attempts and mark changed evidence stale. [Evidence](task-evidence.md) |
| Project memory | Mem0 OSS 2.2.1 with local embeddings, explicit provenance, expiry and supersession. Candidate expansion stops at 256 and reports incomplete recall. [Runtime](agent-runtime.md) |
| Durable workers | LangGraph 1.2.16 and SQLite checkpointer 3.1.2, with scoped dispatch and bounded resumption. Uncertain effects are not blindly replayed. [Runtime](agent-runtime.md) |
| Recovery | Restorable durable-state archives and relocation of an original renamed directory. Assets/settings need separate backups; cross-filesystem copies are unsupported. [Recovery](agent-runtime.md#restorable-backups-and-project-relocation) |
| Typed lifecycle | Object cursors and retention plans through MCP; previews delete nothing and apply rejects stale plans. [Desktop contracts](agent-runtime.md#desktop-contracts-and-bounded-recall) |
| Release acceptance | Explicit local gate for contained runtime, desktop contracts and backup recovery. It supplements standard checks and is not a hosted CI result. [Contributing](../CONTRIBUTING.md#contained-release-acceptance) |

## Verification scope

The 1.0.11 deployment passed the standard Rust gate (392 tests passed, seven
opt-in tests excluded), contained runtime regressions, explicit desktop acceptance
and backup recovery. Post-restart calls in Codex verified matching runtime hashes,
four parallel memory/lifecycle/identity calls, typed cursors, a saved design-token
output and three source/installed/embedded skill matches. Claude installation
files matched; its live conversation connection was not independently observed.

Those are bounded local checks, not a claim that every tool, platform, real-project
retrieval task or visual output has passed. New public binaries need their own
release artifacts, platform checks and publication. Source publication does not itself publish a binary or change an installed desktop runtime.
