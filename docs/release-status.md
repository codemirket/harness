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

Release acceptance requires native formatting, Clippy, tests, standalone
installer/setup/doctor checks on all three targets, contained package and
recovery acceptance on macOS ARM64, audited images, exact artifact hashes,
and public download verification. Advisory-only dependency findings remain
visible; incomplete inventories or failed identities do not pass. The final
release record must identify the exact source commit, CI run, artifact hashes,
remaining limitations and live verification separately from historical tests.
