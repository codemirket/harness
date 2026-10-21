# Mirket 1.1.0: release candidate and supported targets

This checkout prepares **1.1.0**, starting on 2026-10-21. It is **not published**.
The [1.0.11 release page](https://github.com/codemirket/harness/releases/tag/mirket-v1.0.11)
remains the published download reference. Its artifacts and verification notes
belong to 1.0.11, not this candidate. Use the selected release's
`mirket-release.json` or `SHA256SUMS` to verify downloads. A source commit,
local candidate or running CI job is not a published binary.

| Distribution | Targets and requirements |
| --- | --- |
| Core CLI and standalone installers | macOS ARM64 (`aarch64-apple-darwin`), Linux x86-64 (`x86_64-unknown-linux-gnu`), Windows x86-64 (`x86_64-pc-windows-msvc`) |
| Optional design and agent-runtime packages | Supported private Podman environment with Linux ARM64 containers. An x86-64 CLI does not imply x86-64 container support |
| Source | This 1.1.0 checkout, with the locked Rust toolchain and dependency graph; no 1.1.0 tag has been published |

Resolve [the release list](https://github.com/codemirket/harness/releases) again
before installing. For an older installation, use its versioned documentation;
[1.0.5 documentation](https://github.com/codemirket/harness/tree/mirket-v1.0.5/docs)
remains available. The retained memory/recovery commands require at least 1.0.11;
the changes below describe the 1.1.0 candidate.

## Changes prepared for 1.1.0

| Change | Outcome and compatibility |
| --- | --- |
| Smaller default design stack | Bundled encoded-video production, HyperFrames and FFmpeg are removed. Browser interaction/motion remains; encoded video requires an explicitly chosen host tool. Legacy video calls receive a diagnostic rather than a silent substitution. |
| Explicit diagram dependencies | Fresh core preparation excludes diagram services. Kroki and Excalidraw have separate selections; migration preserves old selections that implicitly included Excalidraw. Existing assets and recovery state are retained. |
| Clearer expertise and tools | Discovery exposes owned workflows with canonical metadata. Existing profile IDs and memberships remain; worker production and verification contracts reject schema-only inspection as artifact evidence. |
| Stronger Mem0 adoption | Guidance calls for relevant prior decisions and verified reusable facts. Workers report bounded recall coverage; batch recall reuses one Mem0 handle without adding a resident service. Warm synthetic four/eight-query batches saved approximately 59–73%; this is not an end-to-end task benchmark. |
| Preserved LangGraph recovery | Selected workers retain checkpoint, intent/CAS, original budget and uncertain-effect semantics. Host sessions retain their own lifecycle. |
| Saved design inspection | OpenPencil file inspection reports the consumed source digest; schema discovery remains separate. The four wrappers and editable file engine are retained. |
| Dependency reporting | Advisory-only findings remain visible without blocking ordinary changes. Incomplete scans, missing required inventories, identity failures and owned-boundary defects still require action. |

The [runtime integration review](runtime-integration-review-2026-10.md) records
the recall measurements, dependency decisions and quality limits. Natural tool
adoption and a general design-quality gain have not been demonstrated. No NLP,
Satori or core recovery capability was removed without replacement evidence.

## Retained product contracts

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

Before publication, build optimized native distributions for the release targets,
validate installers and updates, and run contained acceptance with an exact
packaged executable. The release manifest requires the accepted binary to be one
of its candidates and every candidate to share the final source/version identity.
Earlier debug or 1.0.11 acceptance receipts cannot approve these new binaries.
Retain source-lock and full image audit evidence, checksums and license notices.
Claude live activation is deferred; configuration checks are not proof of a
running Claude session. Final native-platform and publication status must be
reported from actual receipts rather than inferred from this document.

### Local candidate verification — 21 October 2026

The optimized macOS ARM64 distribution and its Codex, Claude and combined
standalone installers were built and validated locally. Candidate identity:

- Source: `2c2b37058a2a61abb38d3994ea346ba6af420b9857efb4822eacabf8fbbf27d9`.
- Packaged CLI: `cf947b8eb21f5a9a971ade51abef9a7a4147da183da64122c418834810568efe`.

| Check | Observed result |
| --- | --- |
| Native gate | Formatting, Clippy and 426 tests passed; eight opt-in tests explicitly ignored |
| Packaged installation | All four executable variants passed disposable setup, doctor, update and MCP checks |
| Design tools | 70 tests passed, no skips, in the isolated Linux ARM64 test container |
| Contained acceptance | All five checks passed against the exact packaged CLI: contained runtime, desktop contracts, parallel runtime, design output and backup recovery |
| Artifact assembly | Local macOS manifest validated against packaged bytes and the new acceptance receipt; checksums and license notices retained |
| Source dependency audit | 732 coordinates checked; nine unresolved Pillow findings, three existing ZIP exceptions, no stale reviews |
| Full image audit | All four pinned base/service images and both candidate runtime exports passed identity and required inventory checks; no missing targets |
| Documentation and evaluation tooling | Generated map current; local candidate-document links and diff checks passed; 15 audit tests, five remediation tests and synthetic evaluation preflight passed |

Trivy 0.76.0 reported 5,265 findings in Kroki, 129 in Excalidraw, 262 in the
Node base, 262 in the Python base, 572 in the built design image and 264 in the
built agent image. These are finding records, not unique vulnerabilities across
images. Findings include high and critical severities and remain unresolved;
passing report policy establishes inventory/identity checks, not safety. The
built design image had no reported npm advisories; the agent image's language
findings concerned Pillow 12.3.0, whose reported 13.0.0 remedy conflicts with the
current FastEmbed constraint. OS and upstream service remediation remains
separate from this version preparation.

The audit retained raw reports, scanner/database provenance and verified export
receipts. Runtime exports matched the already prepared design and agent images;
no installed main CLI replacement or service restart was needed. No model
inference or natural-adoption trial was run.

Windows x86-64 and Linux x86-64 native builds/validation and final-revision remote
CI remain pending. The single-target local manifest is not the complete
three-target release. Claude live activation remains deferred. Nothing has been
pushed or published, and the installed desktop CLI remains the prior candidate.

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
