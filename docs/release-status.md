# Mirket 0.0.1 release status

Mirket 0.0.1 is the new fresh-install baseline. The published GitHub release,
its `mirket-release.json` manifest and `SHA256SUMS` are authoritative for
available downloads and checksums. A source commit or local build is not a
published binary.

| Distribution | Target and limit |
| --- | --- |
| Native CLI and standalone installers | macOS ARM64, Linux x86-64 and Windows x86-64; each requires its own native build and installer checks |
| Optional design and agent-runtime packages | Verified managed Podman controller on macOS ARM64 with Linux ARM64 containers; CLI portability does not imply container portability |
| Application data | One per-user `~/.mirket` root (`%USERPROFILE%\.mirket` on Windows) for executable, packages, databases, dependencies and container runtime |

0.0.1 does not migrate Mirket task databases, project identities, design state
or installation receipts from prior releases. Start with a fresh Mirket home
and register each project explicitly. Host-owned Codex/Claude configuration and
project-local selection files remain at their host/project boundaries. Do not
delete project source files to reset Mirket.

The current product keeps OpenPencil editable design, optional Kroki and
Excalidraw, Mem0 fact retrieval, LangGraph checkpoints, scoped worker tools,
and explicit recovery and verification contracts. Bundled encoded-video
production is absent. A package receipt or installed executable alone does not
prove a live desktop connection or useful task outcome.

## Published 0.0.1 baseline

[Mirket 0.0.1](https://github.com/codemirket/harness/releases/tag/mirket-v0.0.1)
was built from commit `943084d4cc1bca42c6d7e5b70d45d4d7a1120ed8` with
compiled-source fingerprint
`9fafb9569f18111047ac704301513aca67ae85ab04a0f588f95382425e0f133e`.
All six jobs in [CI run 39615033279](https://github.com/codemirket/harness/actions/runs/39615033279)
passed. Native logs report 429 macOS, 427 Linux and 394 Windows passing tests,
with zero failures and eight explicit opt-in tests ignored on each target.
[Website checks](https://github.com/codemirket/harness/actions/runs/39615033486)
passed.

Contained acceptance passed against the exact CI macOS executable, including
real Mem0/LangGraph package behavior, desktop protocol contracts, parallel
calls, design output and durable recovery. The separate 70-test design suite
passed without skips. The source audit inventoried 732 dependency coordinates;
the complete local image audit covered the four pinned images and both built
runtimes. Advisory findings remain visible under report policy, including nine
unresolved Pillow findings. These checks do not certify security, natural tool
adoption, design quality or human acceptance.

All 23 public assets were downloaded without authentication and matched the
verified bundle, GitHub SHA-256 digests, published sizes and `SHA256SUMS`. The
public source archive matched all 707 tracked files at the release commit and
the compiled-source fingerprint. The eight earlier GitHub releases and their
tags were removed; `mirket-v0.0.1` is the only published Mirket release tag.

On macOS ARM64, the verified release executable was installed at
`~/.mirket/bin/mirket`; `doctor`, the design and agent-runtime package probes,
and public `update --check` passed. The old local project registrations and
Mirket-owned task, memory and checkpoint stores were intentionally removed
without a backup for a fresh start. Project-local source and selections were
left in their repositories. Codex Desktop's prior MCP processes were stopped
for replacement; the desktop session still needs to reconnect before live
activation can be claimed. Claude live activation was not checked.

Future release acceptance requires native formatting, Clippy, tests,
standalone installer/setup/doctor checks on all three targets, contained
package and recovery acceptance on macOS ARM64, audited images, exact artifact
hashes and public download verification. Advisory-only dependency findings
remain visible; incomplete inventories or failed identities do not pass.
