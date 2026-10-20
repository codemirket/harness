# Harness critique remediation — 20 October 2026

This implementation addresses the release/advisory priorities and the reproduced
worker and feedback defects from the [critique](harness-critique-2026-10.md).
The dated critique/map remain historical observations; this document records the
changes and their verification. No commit, publication or desktop installation
is part of this work.

## Changes and discriminating evidence

| Finding | Implementation | Evidence / remaining scope |
| --- | --- | --- |
| F01: release gates | Native packaging depends on an owned Linux/Node 22 JS job and advisory admission. The JS job installs the exact npm lock without lifecycle scripts and compiles the Linux runtime guard. `dev dist` writes an adjacent build-identity record. `dev acceptance --output` emits a new receipt only after contained checks succeed; `dev manifest --acceptance` requires that receipt, a matching actual candidate executable and matching source identity across every native input. | Unit regressions reject stale source, binary, target, version and missing checks; receipt output cannot overwrite earlier evidence. Existing release tests verify local binary bytes. A real acceptance attempt against an unprepared generation emitted no receipt. Actual release acceptance and remote GitHub jobs remain separate checks. |
| F02: dependency triage | Rust 1.99.1, rustls 0.23.46, AJV 8.18.0 and compatible fflate 0.7.5 override; only rustls changed in the Cargo closure. Notices updated. Targeted `dev lock --package --precise` supports future transitive fixes. | [Live audit](harness-remediation/dependency-audit.json): 679 coordinates, zero untriaged findings. Three ZIP 6 advisories remain explicitly excepted until 2026-11-20 because the owner uses bounded regular-member reads rather than the affected extraction APIs. The exception binds the reviewed source bytes. Actual Satori TTF/WOFF2 render tests pass with patched fflate. |
| F03: required recall | Shared preflight rejects absent required recall before discovery/inference, with a defensive check at durable startup. No implicit installation. | CLI regression asserts error, zero provider POSTs and no created run. Existing empty-store/runtime-failure semantics remain. |
| F04: impossible worker route | Workflow metadata declares `execution_owner`, exposes worker availability and marks editable-interface host-owned. Assignment rejects it before tool discovery. Existing Penpot object guards remain. | Every workflow is iterated through preparation: eligible worker plans retain their scoped tools; the host workflow reports its boundary explicitly. |
| F05: lost failure cause | Saved failure receipts retain stage, bounded failure code and numeric JSON parse location. Inference attempts retain a digest of the rejected response. Arbitrary provider exception text is not copied into receipts. | Synthetic malformed provider action fails and retains `invalid_json`, line/column and response hash. A private sentinel and an arbitrary provider usage field are absent from the saved trace. Other contract failures still use a conservative generic contract code. |
| F06: lost usage | HTTP adapters preserve allowlisted provider token counters and per-attempt elapsed milliseconds, including malformed-action failures. Missing/native counters remain null, not zero. | Synthetic provider counters 1234/56 and response identity survive the CLI trace. This supplies measurement inputs; it does not establish routing efficiency or change the existing quality/tier policy. |
| F07: bounded read dead ends | Source files up to 16 MiB can be read in UTF-8-safe pages of at most 128 KiB. Directory listings use sorted pages of at most 1000 entries, with a bounded 100000-entry scan. Continuations require the previous digest. | Regressions cover multibyte boundaries, drift, missing continuation digest, 1001 entries and partial-read rejection. Whole-file verification requires contiguous coverage of the current digest after the latest mutation. A synthetic provider attempts completion after only one page, is rejected, reads the remaining page and then completes. Existing model-context and output limits still apply. |
| F09: prefix-only recall | Requests may supply up to eight focused `memory_queries`. Automatic recall uses both ends of the objective, task-need reasons and bounded samples of intermediate chunks. Actual queries and incomplete coverage are retained; explicit over-window queries fail rather than truncate. Results are interleaved by query rank, not compared by uncalibrated scores. | Dependency-free regression distinguishes suffix-only Turkish/German requirements and preserves explicit queries. This is a bounded retrieval strategy, not a claim that eight queries cover every long objective. |
| F10: ineffective retrieval assertion | Each frozen answerable multilingual case must retrieve its expected label in the top three. Unanswerable behavior is observed separately. | The empty-retriever fake now fails. [Actual contained retrieval](harness-remediation/agent-tests.json) finds the expected fact for all six answerable cases and returns no result for the unanswerable case. It is retrieval evidence, not answer quality or prompt-injection resistance. |

## Added maintenance paths

- [CI](../.github/workflows/check.yml) runs advisory admission on changes and weekly.
  [Dependabot](../.github/dependabot.yml) proposes Cargo, npm and action updates.
  New findings, partial/error responses, expired exceptions and changed reviewed
  source fail admission. See the [audit workflow](../evals/dependency-audit/README.md).
- `mirket dev design-check --project . --engine-home /absolute/engine-home`
  builds/reuses the reviewed design image and runs the owned JS suite in an isolated
  Linux container. Tests use a fresh workspace and loopback HTTP server, no network,
  and no user-project mount. It does not replace service receipts or running services.
- Shared source fingerprints now include the pinned toolchain and Rust test fixtures,
  along with runtime recipes and locks. Git checkout policy keeps authored text LF
  across native platforms while preserving pinned license bytes. Acceptance compares checkout and executable
  identity before and after the checks. Receipts establish local test/byte consistency;
  they are not signatures, user approval or proof of other desktop connections.

## Verification

- Linux ARM64, Node 22, patched design image: **81 passed, zero failed/skipped**,
  including real Chromium, cancellation/cleanup, raster/vector output, TTF and WOFF2.
  [Preserved result](harness-remediation/design-tests.json).
- Real offline agent-runtime package suite passed against the changed recall/test
  files. It exercised Mem0/FastEmbed, LangGraph, scope, retention, stale checkpoints,
  retry and containment behavior. [Preserved result](harness-remediation/agent-tests.json).
- Four standalone Python discrimination tests passed: advisory rejection/expiry/drift,
  suffix/explicit-query recall and empty-retriever baseline rejection. These supplement
  the actual runtime suite rather than substituting for it.
- Final macOS ARM64 Rust gate: **401 passed, zero failed, seven opt-in tests ignored**;
  formatting and Clippy passed. The previous motion/MCP timeout did not recur.
  [Gate counts and source identity](harness-remediation/checks.json),
  [full sanitized log](harness-remediation/rust-check.log).
- Disposable setup for both hosts, doctor and catalog validation passed. Release
  assembly rejects missing acceptance and a real packaged source mismatch without
  writing output. [Installation checks](harness-remediation/installation-checks.json),
  [source-drift rejection](harness-remediation/release-drift-rejection.json).
- Final optimized macOS package matches the final source fingerprint. All four
  executable checksums and required notices verified; the CLI and all three standalone
  installers passed setup and doctor in separate disposable homes.
  [Native package evidence](harness-remediation/native-package-checks.json).
- The patched native TLS client successfully fetched and verified a pinned source
  archive in a disposable cache, without installing or executing upstream code.
  [HTTPS/archive probe](harness-remediation/tls-archive-probe.json).

Negative evidence is retained. The first local macOS JS attempt could not supply
the Linux-only runtime guard. The first isolated Linux attempt passed 78/81 because
the runner omitted the live-browser fixtures' writable workspace/HTTP prerequisites;
those fixtures were then provisioned and all 81 passed. The large-file CLI test first
used a synthetic model with too little declared context; the final fixture declares
sufficient context while the insufficient-headroom protection remains unchanged.
No thresholds or timeouts were raised to obtain these passes.

## Remaining findings and release limits

These were the limits at the end of the first batch. See the
[recovery and evaluation follow-up](harness-recovery-2026-10.md) for subsequent
F08/F11/F12 changes and the F13 investigation.

- **F08 remains open:** narrow resume compatibility to the selected execution identity,
  validate fallbacks when needed, and correctly rebind native vision-conformance
  evidence. This is a durable-recovery change beyond the reproduced fixes in this pass.
- **F11 remains open:** run a valid, blinded quality campaign after validating the complete
  collector-to-analyzer path. The old invalid campaign has not been relabeled as a gain.
- **F12 remains open:** an active-time/pause-budget design needs its own recovery contract;
  existing absolute deadlines and anti-reset protections remain.
- **F13 remains under investigation:** a passing full gate does not by itself explain the
  previous 15-second MCP startup failure. No deadline was raised and no production
  performance conclusion is claimed.
- Native adapter usage, calibrated routing/cost comparisons, repository search, OS/image
  advisories and the full upstream Penpot dependency closure still need dedicated work.
- The installed desktop executable and live design services retain their previous
  generation. New dependency images were prepared for isolated tests. [Current package status](harness-remediation/package-generation-status.json) reports
  that the new design generation is not installed. A full candidate
  contained-acceptance receipt and native-client activation must be obtained before
  release/deployment; unit tests or the separate component suites cannot substitute.
- macOS ARM64 is the locally exercised native platform. Windows/Linux native packaging
  and the new remote CI graph have not run here. No PR, release or external message was sent.
