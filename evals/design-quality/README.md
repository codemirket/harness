# Fresh design-quality experiment

This local experiment compares the current twenty-seven-reference context with
an improved context on delivered visual and interaction quality. It is distinct
from the ceiling pilot in `evals/design-coverage/`. The intentional intervention
is supplied reference context and its selection guidance. Ordinary interface
guidance, host tools, inherited model/settings, raw task, budget, stopping and
collection are held fixed. No provider launcher, dependency or results database
is introduced. The existing role-eval version 1 report remains the authoritative
functional/access record. Continuous expert craft ratings are a separately bound
evidence artifact, not a redefinition of that schema's binary score.

## Fixed design

There are two development briefs and four fresh sealed holdouts. Each has two
fresh host trials per arm: eight development plus sixteen held-out executions,
24 total. Development results may inform the improved guidance; held-out inputs,
rubrics, probes and outputs remain hidden from guidance authors until the
improved source is frozen. The experiment designer authors original MIT-licensed
cases independently of guidance changes. Frozen inputs are not edited to make
workers succeed. No failed or partial run is replaced or silently repaired.

Each worker is a fresh no-fork host subagent, with no model/provider override or
delegation. It receives the exact common prompt in `worker-prompt.md`, filled
only with owned output path, assigned reference root, common ordinary root and
raw brief bytes. The same twelve-minute/28-tool-call budget applies to every cell.
Actual backend model/version, effort, temperature, seed, token use, cost and model
latency are unknown unless the host exposes them; unknown values remain null.
Fresh workers reduce conversation carryover but do not authenticate model identity.

The root preserves context 0 before edits and freezes context 1 before any
holdout dispatch. All reference files, ordinary guidance, checker source,
protocol, prompts and rubrics receive project-relative SHA-256 bindings. File
identity is not evidence of execution authenticity or a security boundary.
Host worker IDs, actual returns, saved artifacts and failures are retained.
Observed source reads require host evidence. A prepared packet or a worker's
statement alone proves availability/self-report, not actual retrieval.

Development may run baseline before tuning and improved afterward; these cells
are excluded from the causal quality estimate. All held-out arms run after the
improved freeze. The held-out schedule counterbalances which arm starts each
brief/repeat, with a fixed permutation committed before dispatch. Concurrent
workers have disjoint write directories. Paired repeats do not share outputs.
No label is a condition name in worker or judge packets.

## Primary outcome and interpretation

The primary outcome is continuous craft quality (0–100) under the fixed
`craft-rubric.json`. Each of two independent blinded judges first sees only the
raw brief, neutral-label screenshots and observer renders. It records all craft
scores and located visual reasons before accessing source, decision, reference
selection, arm identity or guardrail results. Artifacts in the render-only packet
contain no source-read receipts or context titles. The second stage inspects
actual source/behavior and functional/access guardrails; it does not revise the
sealed first-stage score. A discovered rendering fault is retained and reported.

The craft composite is the prespecified weighted mean of five dimensions.
Average judges within trial, repeats within arm/brief, then compute improved minus
baseline for each brief. The primary estimate is the unweighted mean of the four
brief differences. The independent comparison count is **four briefs**, not 16
runs or 32 ratings. Report all per-brief differences, judge-specific differences,
repeat variability, range, and leave-one-brief-out means. Do not calculate a
p-value or a confidence interval by treating judges/repeats as independent briefs.
An optional descriptive cluster bootstrap must explicitly label n=4 instability;
it is not required and is not a significance decision.

A practical win requires all of these prespecified conditions:

- Mean held-out craft gain at least 7 points on the 0–100 scale.
- At least three of four brief-level gains at least 5 points.
- Neither judge's overall gain below 3 points; no brief mean loss worse than 5.
- No new consequential functional or access failure relative to the matched
  baseline, and no reduction in full guardrail-pass count across eight runs.
- All planned outputs retained, both calibrated judges complete, identities
  consistent, and no material leak or treatment crossover.

These are decision thresholds for a bounded exploratory study, not validated
minimal perceptual differences. A guardrail defect is consequential when it
changes the requested state/data, prevents an essential keyboard path, hides
required content at the target size, or presents an untrue result. Report all
other defects too. Binary version-1 guardrail scores cannot substitute for
continuous craft. Style novelty, a famous reference, screenshots, tool counts,
source hashes or workflow badges do not count as quality gains. Human acceptance
remains `not_collected` regardless of agent scores.

## Calibration, blinding and disagreement

QA supplies style-neutral positive/negative synthetic controls covering more
than one visual language. Controls establish that judges detect a meaningful
hierarchy/content/readability failure without rewarding one fashionable style.
They do not enter the treatment estimate. Each judge sees an identical-artifact
pair under different labels and counterbalanced presentation order, plus the
controls, before heldout scores. Calibration is recorded before unblinding. A
judge is calibrated only if it ranks the positive above its paired negative in
both control families by at least 10 points and scores the identical pair within
5 points. If calibration fails, retain the failure, clarify only rubric
application, repeat calibration once on another fixed synthetic family, and
retain both attempts. A second failure blocks a quality-gain claim; do not choose
a more favorable judge after seeing heldout scores.

Judges use independently permuted packet orders with case/arm placement
counterbalanced; neither sees the other judge's scores before locking its own.
Record first/second placement and check order-related differences descriptively.
A more-than-20-point trial disagreement triggers located third-person inspection
by the coordinating lead after initial scores are frozen. Report the dispute;
do not overwrite or average away the original disagreement. Raters are same-host
agents, not independent human design experts or user participants. Condition
blinding may be imperfect from the work itself and is explicitly assessed by a
post-score arm guess plus confidence, before arm labels are revealed.

## Functional/access observation

Workers deliver `index.html` and `decision.md` only in owned output directories;
optional worker proof is self-report. QA's observer runs outside that directory,
blocks external network requests, preserves runtime errors, and captures fresh
1280×900, 390×844 and 320×800 layouts, focal states, keyboard focus, reduced-motion
and reflow approximations. Frozen per-case action plans assert literal independent
expectations and persisted state, rather than trusting a success message. Generic
layout candidates and an automated scan are not automatic craft or access grades.
No native, real assistive technology, real user or device acceptance is claimed.

For a fresh visual comparison, freeze an actual-viewport capture method and its
state coverage before dispatch. [Viewport evidence](viewport-proof.md) provides
native-size current frames and uniform vertical scans, with regression controls
for false and real obstructions. The original `observe.cjs` and packet builder
retain full-page views for their original reports. A Chrome 155 counterexample
showed that a scrolled full-page image could paint an unfocused, offscreen skip
link inside the document image. Use actual viewport pixels and focus evidence
to adjudicate that condition; keep the original captures and scores unchanged.
The capture regression establishes measurement behavior, not generated quality.

Every case has explicit boolean functional/access guardrails in its frozen rubric;
reviewers cite actual state, observable effects and source ownership where needed.
Version-1 `score` is passed guardrail criteria divided by criterion count. Completed
but wrong work is `task_failed`; inability to launch/collect due to infrastructure
is `infrastructure_failed`, remains unscored, and still occupies its planned cell.
A blank output caused by budget exhaustion is a task failure, not infrastructure.
Uncertain infrastructure writes are reconciled before retry; output is never
reimplemented in the original cell. Evidence beyond the checker's size limits
stays in the frozen directory and is hash-manifested through bounded evidence.

## Stopping and contingency

Stop the first wave after all 24 planned executions are sealed and both judges
are locked, irrespective of interim scores. A real host outage can pause work;
resume the same pending cells, recording delay. Do not peek at heldout arm
aggregates to allocate more trials, retune guidance, drop a brief or change a
threshold. Failures remain in the matrix. If a worker never starts, record that
cell as infrastructure failure; a diagnostic replacement, if needed, is a
separate labeled artifact excluded from the primary matrix.

If thresholds are unmet, calibration fails, there is material leakage, or the
brief-level results conflict, the result is inconclusive or a measured loss.
The user has authorized local failure-driven iteration. A subsequent wave requires
a recorded next-wave decision before generating at least four new independently
authored sealed briefs and a new frozen protocol. Same-arm replication keeps
the two contexts and thresholds fixed. Changed guidance is a new intervention.
Report waves separately, including this failed or inconclusive first wave; never
pool results to hide it, retune/regrade old holdouts as fresh, select best runs or
manufacture a winner. Publication and production changes remain separate effects.

This small hand-authored task sample estimates transfer only to these briefs
under this host and budget. It cannot establish general design improvement,
user preference, production fitness, efficiency gains or model superiority.
