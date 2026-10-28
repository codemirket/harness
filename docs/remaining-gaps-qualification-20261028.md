# Remaining-gap qualification — 2026-10-28

Continuation from `b5c248930a404c0ff3d53e8e4fe865d44209940c` under the
[approved research plan](../planning/research-improvements.md). This pass closes
specific local evidence gaps. Native quality comparisons remain unqualified.

## Finding, correction and actual outcome

| Finding | Correction / source | Observed outcome and limit |
| --- | --- | --- |
| A mutable cohort deadline could extend a frozen budget; setup was outside the per-cell clock. | `tests/research-quality-trials.py`, `tests/test_research_quality_trials.py` seal and validate finite clocks, original duration, model/effort and zero retries; include setup in elapsed time. | Mutation tests reject extended deadlines, changed settings and invalid clocks. Hashes guard trusted local drift, not an adversary rewriting every record. |
| A fresh disposable native home stalled before thread creation. | Optional `--model-cache` snapshots the current host catalog into frozen inputs and copies identical bytes into each condition. | No-model app-server creation completed in 0.096 seconds with the cache. This fixes that observed startup prerequisite only; the later actual turn still failed on network/routing requests. |
| Interrupting a native cell discarded its captured streams; the cohort continued after a timeout. | Runner drains and records interrupted output, cleans auth/processes, and stops after incomplete execution. | A real disposable child emits an event, interrupts its parent, is terminated, and leaves retained event/result records with no copied auth and no second cell. This is a runner test, not a successful Codex model trial. |
| Wrong-project and unsupported-tool rejection did not prove absence of project effects. | `tests/mcp-smoke.mjs` submits valid-revision cross-project source and destination requests and snapshots both projects. | Actual stdio MCP requests were refused and both project trees remained unchanged. Unsupported `file_read` also left project and state/receipt trees unchanged. Existing absent-runtime, stale-release and stale-source refusals passed. |
| Interactive focus/mobile/reduced-motion observations were missing. | `tests/interactive-design-qa.test.mjs`; [browser record](interactive-design-qualification-20261028.md). | Actual host-browser clean/defect cases were observed, and the lead viewed saved clean-mobile, missing-focus and overflow pixels. Retained validation checks source/capture integrity and scoped paths. This does not establish aesthetic uplift or authenticate event/capture records. |
| An unqualified optional headless path raced on partial stdout. | Removed it; retain an exported fixture for Codex host-browser use and a bounded evidence validator. | Actual qualification uses connected host Chrome. No second browser service or production dependency was added. |

## Native attempt ledger and current prerequisite

Private evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-gap-closure-20261028/`.
The task budget was frozen before calls: 2,400 seconds, at most 21 native cells,
Astra/high, no repairs or retries. Earlier failed cohorts remain historical costs;
this continuation does not relabel them as valid matched acceptance.

- A single 60.011-second availability turn timed out with no JSONL output.
  Account usage reported ordinary usage available; this is not attributed to
  the previous provider rate limit.
- Five bounded app-server probes started no model turns: initialize succeeded;
  thread creation stalled with an empty disposable home, also with optional
  plugins disabled and from a neutral temporary directory. Copying the current
  host model catalog resolved thread creation. All copied auth files were removed.
- A new frozen six-question/two-condition corpus protocol then started. The
  first no-skill cell timed out after 90.018 seconds without an answer. It emitted
  thread/turn-start events and logged authenticated HTTP and workspace-routing
  discovery timeouts. A start event is not completion or output quality.
- The already-running with-skill cell was interrupted after approximately
  60.002 seconds rather than spending the rest of the cohort on that failure.
  Its pre-fix runner did not retain the communicate buffers; the interruption
  ledger explicitly records that evidence loss. No result is reconstructed.
- Thus this pass used one availability attempt and two outcome-cell attempts;
  no matched outcome pair completed. No native reviewer, near-miss, live-web or
  product/marketing cells ran. Their acceptance remains open. The prepared
  product protocol is retained privately and is not marked executed.

Automatic approval review was selected as authorized. No denial was observed
that explains these network timeouts, and Mirket cannot repair Codex's native
routing or grant host permissions. A functioning authenticated disposable CLI
turn is the external prerequisite for further model comparisons. No model
switch, quota reset, unsafe fallback, retry or fresh budget was used to bypass it.
The runner fixes made after this failed attempt are tested locally, not claimed
as native model qualification. The oracle is withheld from the prompt/project;
workspace-write does not establish read isolation from sibling evidence.

## Gates and limits

Actual platform: macOS 26.5.2 ARM64; private Node 24.22.1; native CLI
0.164.0-alpha.11.1. Requested Astra/high is a host setting, not independent
provider identity attestation. Gate logs are under the private evidence root.

- Build-input guard passed before Rust compilation.
- Locked/offline Rust: 509 passed, 5 ignored; all-target Clippy with warnings
  denied and formatting passed.
- Adapter: all 209 installed entries matched locked versions, URLs and
  integrities; 50 absent entries are optional. The first ad-hoc comparison
  incorrectly required optional entries and failed; the corrected comparison
  did not install or alter dependencies. Explicit-private-Node TypeScript build
  passed; process tests: 68 passed, 1 skipped.
- Explicit-binary/private-Node MCP smoke passed on disposable state/projects.
- Retained browser evidence validator: 2 passed, with the actual evidence
  directory supplied explicitly. Without it, live evidence validation is skipped.
- Final runner tests: 18 passed, including real child interruption. Full Python
  discovery: 224 tests, 1 existing skip (`python-tests-final.log`).
- Repository catalog refresh reported no changed hashes or provenance inputs.
  No skill body/resource changed in this pass.

R1 native research/live-web quality, R2 native reviewer calibration, R4 unnamed
selection/near misses and R7/R8 native useful-output comparison remain open.
The local R4 boundary and R6 interactive fixture outcomes above are narrower
claims. No universal uplift, additional creative acceptance, Windows, macOS 15
floor or waived Desktop GUI qualification is asserted. No live `.mirket`
installation, user configuration, production dependency or publication changed.

## Continuation after deferring native CLI work

The user deferred CLI connectivity/authentication and dependent native model
comparisons. Reauthentication is not a demonstrated requirement: observed
failures were connection/routing timeouts, not an authentication rejection.
No further login, model or provider calls were made in this continuation. The
failed cohorts remain closed and their costs remain recorded; deferral is not
acceptance or a budget reset.

The existing `tests/reinstall-installed-smoke.py` now exercises actual installed
project references in addition to its canonical-memory reinstall journey. A
fresh source-built binary and repository skill assembler produced a disposable
core/Node/skills release. The source Node bytes matched the admitted base-release
manifest before copying. Actual installed CLI and MCP outcomes on macOS 26.5.2
ARM64 were:

- Nine opted-in project skills delivered all 11 catalogued Markdown references;
  all nine global skill copies remained lean and matched their source bodies.
- Repeated project sync returned unchanged and preserved the project files.
- An edited research reference was reported drifted. Both sync and removal
  refused it, preserving the edited bytes, other skills and receipt files.
- Selecting video production against this deliberately core-only release failed
  without creating any files or directories in that project.
- Uninstall/reinstall preserved project skill resources, selection and receipts.
  Project doctor reported managed, and subsequent sync was unchanged.
- MCP recalled the retained canonical fact after reinstall, while the forgotten
  fact remained unavailable; 17 canonical state/backup files were preserved.
- Managed project removal subsequently removed its owned skills while retaining
  an unrelated project skill. Personal skills, instructions and model settings
  in the disposable user home also survived.

This closes the installed resource-delivery/preservation check for these skills;
it does not establish native skill discovery, selection or output quality. No
production source or dependency correction was needed. The vector/video
resources require their qualified payloads and are outside this core-only
fixture; prior installed runtime evidence remains separate.

Evidence:
`/Users/nazmiyilmaz/.codex/task-state/harness-gap-closure-20261028/same-home-reinstall-21iw1_fn/`.
The summary, CLI responses and actual MCP requests/results are retained there.
Release ID: `sha256:143d87d0465230c72cfdc54b25a0282496565e0df31a91f84fe2d65cd486f806`.
Core SHA-256: `5a5a98f5fd48e9a21672ee3b10449f8052ee74562608f10995c2e015e0bbe6a7`.

The build-input guard and locked/offline build passed before this journey. The
explicit-binary/private-Node MCP smoke passed again with that built binary;
catalog tooling reported no changes. Rust and adapter source were unchanged,
so their passing full gates above were retained rather than repeated. The
extended integration fixture passed end to end. Native model comparisons remain
deferred; Windows, macOS 15 floor, Desktop GUI and broad quality uplift remain
unqualified. No live installation, push or publication occurred.
