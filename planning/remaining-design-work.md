# Remaining design work

Created 2026-10-27 after user authorization to commit the current work, plan the
remaining design work and start implementation. Baseline commit: `dba8873`.
This is an execution detail of `implementation-plan.md`, not a replacement
product plan or authorization to publish or change the live installation.

## Outcome and boundaries

Make Codex's UI, editable vector/illustration and HyperFrames motion work
reliably reviewable against the actual brief, then measure whether the selected
skills improve representative results. Keep Hallmark/Taste reference adoption
and HyperFrames. No new engine, model router, agent orchestrator, general Mirket
browser, dependency or native editor attachment is planned.

Codex owns authors, model/reasoning choice, reviewers, browser interaction and
permissions. Mirket owns its scoped design/video effects. The user permits a
host-browser render route if private Chrome is demonstrably impractical; this
is a conditional alternative to qualify, not a reason to change a working
renderer during a frozen comparison.

The recent follow-up demonstrated two authors avoiding the observed contrast
failure and one actual model repair, with independent decoded-frame review and
1× host-browser playback. It did not establish broad aesthetic superiority,
statistical skill benefit, continuous perceptual pacing, Windows or Desktop GUI
coverage. Prior failed trials and exhausted budgets remain historical evidence.

## Work order and acceptance

| ID / priority | Deliverable and owner | Acceptance / failure discriminator | Status |
| --- | --- | --- | --- |
| D1 / implemented | Root: strengthen the existing repository evidence checker with frozen, condition-specific file/directory scope and capture/review chronology. | A repair packet can legitimately use `evidence/`; a fresh-author packet limited to `.feedback/` refuses it. Reject extra files, prefix lookalikes, symlinks, changed scope and observations before their captures or outside the shared window. Return unrecorded scope explicitly for historical packets. Validate a packet containing actual saved repair images; neither a hash nor a reviewer field certifies visual truth. | Passed bounded checks; [actual artifact evidence](../docs/design-evidence-scope-20261027.md) |
| D2 / implemented | Root: add explicit finding disposition and repair lineage to the same record format; independent reviewer challenges negative cases. | Preserve initial source/captures; bind a repair to its parent, finding, budget event and revised captures. Rejected reviewer claims retain the original claim and a reason tied to the brief. A status word alone cannot prove resolution; unknown parent, stale source, changed criterion or uncharged repair refuses. | Passed bounded checks; [before/after artifact replay](../docs/design-evidence-lineage-20261027.md) |
| D3 / implemented | Root: extend the existing disposable trial runner for a matched specialist-skill/no-specialist-skill comparison; independent capable reviewers score neutral artifact IDs in D4. | Freeze brief, common production contract, allowed outputs, model/effort, runtime, assets, rubric and shared budget before authoring. Both arms get equal tool access. No-skill omits the specialist guidance, not the format/runtime prerequisites. Prove both positive outputs and denied/missing/stale-route handling without treating a receipt as task success. | [Runner and two native calibration turns qualified](../docs/matched-design-study-20261027.md); current MCP missing/stale-route probes passed. Denial retention is regression-tested; native host-denial evidence remains historical and policy-specific. Creative scoring not run. |
| D4 / next | Root and comparable creative reviewer: run one finite cohort covering responsive UI, editable illustration, multi-beat captioned motion and an editable vector transition. | Four briefs × two conditions, one initial turn per cell; inspect real UI interactions at desktop/mobile, exported SVG/full/detail pixels, and decoded video transition boundaries. Retain initial findings, bounded repairs, final artifacts and remaining defects. Expand only for a shared failure within the same budget; never silently retry a failed cell. | Planned, not run |
| D5 | Root: qualify native host model/agent choices on the available Codex surface. | Record configured and observed lead/reviewer model and effort, weaker-creative-delegate attempt, explicit user override, interruption and approval denial. Distinguish CLI evidence from Desktop GUI; no claim of Mirket hard enforcement or provider identity from configuration alone. | Planned; GUI stays waived/unverified |
| D6 | Root: assemble a reviewable playback/art-direction pack and carry distribution/platform gates forward. | Actual saved videos, timed browser playback and decoded evidence remain separate from perceptual judgments. Ask for human timing/art-direction judgment only where the available observation surface cannot supply it. Release requires applicable source/notices; Windows and OS-floor support require actual native targets. | External/perceptual and release gates remain open |

D1 does not solve D2 or D3 by adding more metadata. Declared reviewer identity,
capability and independence are attestations unless host evidence establishes
otherwise; the checker must not turn them into authenticated claims. A complete
file inventory describes the selected evidence root at check time, not writes
elsewhere on the machine or an OS sandbox.

## Cohort controls

For D4, freeze at most eight author turns (240 seconds each), one shared
45-minute wall window and 2700 aggregate model seconds, and at most four
180-second repair turns across all conditions/specimens/delegates. These are
ceilings, not an instruction to spend the entire budget. If reservations or
review time leave insufficient room, mark remaining cells not run. Do not reset
these limits by opening a new specimen or delegate. Start this distinct cohort
only after D1–D3 acceptance passes and the frozen protocol is reviewable.

A fresh reviewer receives the actual brief and neutral saved artifacts, without
condition labels or the author's self-assessment where feasible. Review claims
must name a concrete artifact/state/frame and user consequence. The lead checks
suspect frames individually and against neighbours before repair; invented
requirements such as changing a button's fill are not defects. Disagreement is
resolved from saved behavior and the brief, not majority vote. Report masking
limitations and do not present the small cohort as a statistical benchmark.

## Verification and completion

Use only disposable homes/projects and explicit private executable paths.
Before any Rust compilation run `python3 scripts/check-build-inputs.py`.
Changes to Rust/runtime/catalog inputs require the locked/offline Rust tests,
all-target Clippy with warnings denied, formatting, locked adapter build/tests
and explicit-binary/private-Node MCP smoke as affected. Python/JS-only tooling
changes require their discriminating regression tests, real positive/negative
artifact journeys and diff/catalog integrity checks; prior unchanged runtime
gates are retained rather than silently represented as fresh runs.

D1 ends with tested code, an actual artifact packet, a rejection probe and a
short implementation report. Later rows stay planned until their own evidence
exists. Do not promote catalog `planned-not-run` acceptance, claim broad design
quality or mark the original six-area audit complete from this plan.
