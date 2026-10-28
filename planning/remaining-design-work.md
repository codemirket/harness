# Remaining design work

Revised 2026-10-28 at the user's request to stop the repeated timeout-investigation
cycle. Reviewed baseline: `a75dea2`. This replaces the active sequence in the
previous version of this file; historical evidence and frozen trials remain
unchanged. It is an execution detail of `implementation-plan.md`, not release
approval or permission to modify the live installation.

## Decision and practical outcome

Finish one usable, repeatable local design-production path. Stop making a
four-minute isolated author turn the prerequisite for using that path. The
240-second limit belongs to closed experiments, not the product's definition
of good design. Timeouts remain failed executions; independently reviewed
artifacts retain their separate findings. Do not reclassify a failed cohort.

The [research and source comparison](../docs/design-workflow-reset-20261028.md)
supports incremental work from saved artifacts, deterministic checks between
creative steps, and a finite independent review/repair cycle. It does **not**
establish the root cause of our creative latency or promise a particular speedup.

Keep HyperFrames 0.8.197, Hallmark/Taste, the accepted Daybreak direction and
Codex Desktop ownership. Codex owns authoring, models, native agents, permissions,
Git and live browser review. Mirket owns its scoped design/video operations.
Use the existing private renderer; no evidence currently warrants changing it.
Keep a capable creative lead and comparable reviewer. Preserve explicit user
model/effort choices; do not add model routing or a second agent orchestrator.

## What stops now

- No new Tidepool/Lantern-style paired cohort, timeout calibration, model ranking
  or runner instrumentation work without a new demonstrated production blocker.
- No full redesign, new aesthetic specimen, replacement renderer, framework,
  new schema, generic QA engine or duplicated starter generator.
- No repeated full exports while static art or a specific transition is still
  unresolved. Reuse accepted art; inspect the smallest relevant preview first.
- No recursive reviewer debate or cosmetic repairs after the agreed criteria
  pass. Existing failures and catalog `planned-not-run` rows remain visible.

These are prioritization decisions. They do not waive required release gates or
convert incomplete high-risk skill qualification into a pass.

## Production sequence using existing owners

| Stage | Concrete input and owner | Saved result and decision |
| --- | --- | --- |
| 1. Direction and static composition | Codex lead reads the actual brief, selected UI/art skill and existing [motion shot card](../skills/motion-direction/references/storyboard-and-transitions.md). Reuse accepted Daybreak for this closure; no new mood-board search. | Exact copy, asset/font identities, dimensions, relevant UI states, beat times and at most five task-specific acceptance criteria. Inspect the still before moving it. Existing user acceptance need not be requested again. |
| 2. Editable implementation | Codex lead edits one stage at a time in the same project. For new basic source, use [create-video-seed.mjs](../scripts/create-video-seed.mjs); for the accepted visual treatment, use a disposable Daybreak copy. Read the [pinned producer contract](../skills/video-production/references/hyperframes-0.8.197-authoring.md). | Saved source plus a short handoff in the existing project README: completed step, source revision, next operation, known defects and budget used. No new persistence service. A partial stage is not completed merely because a file exists. |
| 3. Preflight and preview | Project tools own [source checking](../scripts/check-video-source.mjs); Codex's browser owns the existing [native-animation preview](../scripts/create-motion-preview.mjs) and live UI. | Fix concrete root/font/asset/timing/property failures before a full render. Preview covers CSS/WAAPI only; do not invent a universal seek API or assert export parity. UI journeys stay in the Desktop browser, not an unconfigured CLI author. |
| 4. Export and independent QA | Mirket's existing finite video job; lead retains its operation ID and reconciles status instead of resubmitting. Project FFmpeg/FFprobe checks and one independent capable reviewer follow. | Actual decoded output, requested-profile check, saved-frame inspection, transition neighbours where relevant and playback evidence with honest limits. Use [Daybreak's checks](../specimens/daybreak/qa/README.md) and existing evidence/lineage tools; do not build another report format. |
| 5. Repair or delivery | Lead integrates only findings tied to the brief and actual viewport/frame/interaction. | At most one repair batch across all stages/reviewers, then rerun the checks affected by it. Deliver source, checked output and remaining defects, or a specific blocked result. A changed film cannot inherit old human acceptance. |

The stages are work boundaries, not five new subagents or five tiny timeout
budgets. Use the current Desktop task; a phase handoff does not need a new CLI
session. If an authorized task continues across turns, preserve the same source,
remaining work and total budget. A timed-out frozen trial is never resumed under
this production procedure.

## Remaining implementation: one 60-minute closure window

This is a **work cap**, not a prediction that all research/release questions can
be solved in an hour. Start its clock with the next implementation action after
this plan revision. All stages, tool waits, review and repair share it; a new
turn, artifact or reviewer does not reset it. End with a result or an explicit
remaining blocker. No automatic follow-up cohort.

| Step / owner | Work and files | Acceptance | Allocation |
| --- | --- | --- | --- |
| R1 / lead | Make one concise production entry in the existing video authoring reference and Daybreak README, linking the above tools in execution order. Reuse current shot-card/preview instructions; edit only a demonstrated missing or conflicting handoff. | A future author has a single route from saved source to checked output, explicit private-path inputs, failure handling and an example handoff. No extra global skill, dependency, runner or duplicated guide. | 15 min |
| R2 / lead | Exercise one maintenance journey on a disposable copy of accepted Daybreak: change one short supplied heading/CTA, rebuild its composition and render through the already-qualified route. Freeze exact before/after text, paths and affected beats before editing. | Requested text is visibly present and unclipped in the actual changed export; unrelated composition is preserved. Original accepted specimen/export stay untouched. This tests editability and delivery, not new art direction or model superiority. | 20 min |
| R3 / reviewer and lead | Inspect changed and neighbouring decoded frames, affected UI viewport/interaction if the shared page changed, and actual saved playback. Run the applicable existing positive/negative checks and verify stale acceptance is not transferred. | Separate execution status, scope/required-file validity, export integrity, visual findings and user acceptance in the existing report fields/prose. No single success flag substitutes for these observations. | 15 min |
| R4 / lead | One evidence-based repair batch if needed, affected rechecks and final handoff. If none is needed, finish early. | Source/export links, measured time, exact defects resolved/remaining, tests and limitations. No speculative fix or fresh specimen. | 10 min reserve |

If R2 exposes a runtime defect rather than a workflow gap, stop the maintenance
journey, preserve the reproduction and name its owner. Do not expand this window
into another engine investigation. A required external approval/asset/runtime
prerequisite is reported immediately; continue independent documentation/checks
within the remaining window. Automatic approval review remains a host choice,
not permission bypass. No publication or live `~/.mirket` changes.

## Verification

Documentation/utility changes use affected repository checks, real artifact
positive/negative journeys and diff/catalog integrity checks. If skill resources
change, refresh hashes/provenance with repository tooling; no new third-party
source adoption is implied by this research. Do not claim unchanged runtime gates
were rerun. Before any Rust compilation run `python3 scripts/check-build-inputs.py`;
changes to runtime/catalog inputs require the applicable locked/offline Rust,
all-target Clippy with warnings denied, formatting, locked adapter build/tests
and explicit-private-Node/current-binary MCP smoke. Required checks are not
waived by the time cap: unfinished checks remain unverified.

## Closed work and retained limits

| Work | Evidence and current disposition |
| --- | --- |
| D1–D2 scope and repair lineage | [Scope](../docs/design-evidence-scope-20261027.md) and [lineage](../docs/design-evidence-lineage-20261027.md) implemented and tested. Reuse them. |
| D3–D4 matched design runner/cohort | [Runner](../docs/matched-design-study-20261027.md) and [eight-cell cohort](../docs/design-cohort-20261028.md): completed investigation with confounds/failures retained; no general skill-benefit claim. No new cohort in this closure. |
| D5 coordination | [Initial qualification](../docs/native-coordination-qualification-20261028.md) retained failures; [follow-up](../docs/coordination-followup-20261028.md) verified scoped single/parallel integration and exact host-command termination. Original stall cause unknown; no hard spawn enforcement. |
| Artwork/motion diagnosis and Granger target | [Controlled diagnosis](../docs/art-motion-diagnostic-20261028.md), [target](granger-design-target.md), [Daybreak production](../docs/daybreak-production-20261028.md) and [defect repair](../docs/daybreak-video-review-20261028.md) retained. Do not reopen accepted art direction. |
| D6 | [Review pack](../docs/design-review-pack-20261028.md) delivered; user accepted the current candidate. Acceptance is bound to its exact export. |
| Tidepool/Lantern and timing capture | [Acceptance sample and follow-ups](../docs/design-acceptance-sample-20261028.md): both cohorts closed; native timeouts remain failures. `a75dea2` timing capture qualified by tests and one readiness probe. No need for further timing infrastructure to complete R1–R4. |

## Separate backlog, not hidden completion claims

- Broad skill benefit, catalog model acceptance and the remaining original
  six-area audit are not established by this production closure. Keep their
  unqualified cases; resume only for a concrete release decision or new defect.
- Public recipient/source/notice delivery remains governed by the [D6 release
  handoff](../docs/design-review-pack-20261028.md#current-gate-handoff).
- macOS 15 floor and Windows x64/ARM64 require actual native target evidence.
  Current local evidence is macOS 26.5.2 ARM64. Desktop GUI remains waived and
  untested. These cannot be solved by more local design cohorts.
- Generic glyph clipping, aesthetic excellence, full-frame continuity and
  universal preview/export parity are not proven by bounding boxes or hashes.
  Use targeted checks plus saved-pixel judgment and report sampling limits.

Closure means the documented local production route and maintenance journey
were completed and honestly checked, or the cap produced a concrete blocker.
It does not mean the entire harness or a public release is qualified.
