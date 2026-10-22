# Mirket release status

Mirket 0.0.2 contains the post-0.0.1 workflow and runtime fixes.
The 0.0.1 release established the fresh-install baseline. The published GitHub release,
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

## 0.0.2 release scope

This release includes the changes in `ee17264` and `38f99b5`: bounded source
search, explicit worker budgets and usage, simpler capability routing, no-change
source outcomes, native text operations, advisory expertise, applicable design
review, independent persistence and recall, immutable observation references,
resumable activation, and separate execution identity. It also separates
observational capture/export from design production, supports explicit image
equality requirements, and retains inspection evidence for large images.

The [implementation report](simplification-implementation-2026-10.md) records
conditional decisions: LangGraph and the NLP dependency closure remain; comparative
model-quality improvements are not claimed. Existing 0.0.1 installation and project
data are preserved. Old checkpoint execution bindings remain enforced.

Publication requires the native and contained gates below. The selected release's
manifest, build receipts and acceptance artifact identify the exact published
source and binaries; this source document does not predeclare passing CI.

## Published 0.0.2

[Mirket 0.0.2](https://github.com/codemirket/harness/releases/tag/mirket-v0.0.2)
was published on October 22, 2026 from commit
`e7b875cf147260b7a012f38ab257883d6736c71f`, with compiled-source fingerprint
`d0482b152dd40aea9a1329090bfa81ab3f9fc7eed83397fa8f59ef3913e9f14d`.
All six jobs in [native CI run 39753753026](https://github.com/codemirket/harness/actions/runs/39753753026)
passed: 444 macOS, 442 Linux and 409 Windows tests, zero failures and nine
explicit opt-in tests excluded per platform. Contained acceptance passed
against the exact CI macOS binary; all 72 design tests passed without skips.

All 30 public assets were downloaded and checked against their published sizes,
digests and verified bundle. The source archive matched all 740 tracked files
and the compiled-source fingerprint. Source and image advisory findings remain
visible under report policy; these checks are not a security certification.

After a Codex Desktop restart, the actual MCP connection reported 0.0.2 with
matching executable, source, catalog and execution identities. A live design
browser capture matched the expected release modules and inspected output.
Claude activation was not checked. These are observations of that macOS ARM64
installation, not a guarantee for other devices or tasks.

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
The documentation follow-up also passed all six jobs in
[CI run 39618860794](https://github.com/codemirket/harness/actions/runs/39618860794).

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
tags were removed; `mirket-v0.0.1` was the only published Mirket release tag at that verification.

On macOS ARM64, the verified release executable was installed at
`~/.mirket/bin/mirket`; `doctor`, the design and agent-runtime package probes,
and public `update --check` passed. The old local project registrations and
Mirket-owned task, memory and checkpoint stores were intentionally removed
without a backup for a fresh start. Project-local source and selections were
left in their repositories. Codex Desktop's prior MCP processes were stopped
for replacement. After the Codex Desktop restart on 2026-10-21, a live main
`runtime_status` call reported Mirket 0.0.1, the installed executable digest
`20221fee28ff8658c6dd59d2e119f1b8bd42ff6ba88f93a93549df4583e6a208`
and a matching process. A live design `openpencil_inspect` call completed in
headless-file mode with OpenPencil 0.16.0; its reported runtime file hashes
matched the release source. These calls establish activation and runtime
identity, not a new design-quality acceptance run. Claude live activation was
not checked.

Future release acceptance requires native formatting, Clippy, tests,
standalone installer/setup/doctor checks on all three targets, contained
package and recovery acceptance on macOS ARM64, audited images, exact artifact
hashes and public download verification. Advisory-only dependency findings
remain visible; incomplete inventories or failed identities do not pass.
