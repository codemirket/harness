# Deliver and review the intended outcome

Use for substantial implementation, consequential work across boundaries or
subjective deliverables where a fresh inspection can expose a meaningful gap.
Keep the task's existing plan or record. This reference does not require a new
planner agent, fixed sprints, a review server or another document.

## Agree on observable completion

Capture the user's intended journey or decision, scope and non-goals, important
constraints, and evidence that will establish completion. Separate requirements
from assumptions and optional improvements. A concise list in the existing task
record is enough; do not turn a short request into an expanded product roadmap.

Make each consequential condition observable at the affected consumer. For a
bulk edit, the result might be the selected records retaining their new values
after reload. A toast alone cannot establish that condition. For a product brief,
it might be a supported choice between options, with uncertain assumptions visible
and no invented customer evidence. Name the source or owner of disputed criteria.

Check the most uncertain path before scaling implementation. For software, use a
small working slice through real entry points, data and effects; for a migration,
exercise a representative fixture and recovery path; for research or a plan,
carry one important claim through its sources to the resulting decision. Use
substitutes for unavailable external systems explicitly. A stub or local exercise
can establish a boundary contract without proving the live system works.

Retain accepted conditions while implementing. When new evidence changes the
scope, update the record with the reason and source of the change. Do not lower
the bar, remove a required behavior or rewrite the verifier to approve the output.

## Inspect the result independently

Use a separate reviewer when the task's uncertainty, impact or visual judgment
warrants it and an appropriate reviewer is available. Give the reviewer the
original request, accepted conditions, relevant constraints, exact artifacts or
revision, runtime entry points and allowed actions. The builder's summary helps
locate work; it is not the review's evidence. Use the host's existing delegation
tools and coordination guidance, without switching provider or model implicitly.

Have the reviewer inspect or exercise the delivered result against the user's
journey. Include a consequential failure or edge case when it can distinguish a
working result from a convincing demonstration. A live browser is useful for UI
behavior; database observations for durable effects; original sources for research;
and actual claims, audience and decision criteria for product or marketing work.

Keep these findings distinct:

- **Requirement defect:** state the condition, reproducible trigger or observation,
  location and consequence. Identify whether it blocks the requested outcome.
- **Preference or improvement:** explain how the suggestion serves the intended
  audience or direction. A reviewer's taste does not become a new requirement.
- **Unverified:** name the path, missing evidence and the next check that would
  resolve it. Missing access and an inconclusive check are not passing results.

For visual work, assess coherence, hierarchy, craft and usability against the
project's direction and comparable renders. Novelty is useful only when it serves
that direction. Scores without inspected artifacts and reasons add little; an
independent agent can still be biased or wrong. Resolve disagreement against
requirements and evidence. If only self-review is available, describe it as such.

## Repair from fresh evidence

Prioritize defects that prevent the user's outcome. Diagnose the responsible
boundary before changing it; ask what observation would distinguish the proposed
cause from another explanation. Fix, then rerun the affected checks and inspect
the changed artifact. Preserve useful passing evidence for unchanged inputs.

Bound review effort by the task's scope, remaining risk and available budget.
Revisit the approach when repeated feedback or repairs produce no new evidence;
do not continue generating cosmetic variants or replaying the same failing check.
Identify a different experiment, missing capability or unresolved user decision.
An exhausted review budget does not turn remaining defects into acceptance.

Keep the best-supported version: later iterations may weaken clarity, behavior
or maintainability. Before a handoff, record the tested revision or input hashes,
observed results, outstanding uncertainty, unfinished work and next concrete check.
Keep source inspection, local execution, live verification and owner acceptance
distinct. Do not imply that a reviewer approved paths they could not inspect.

When the same observed failure recurs, prefer a focused regression check, better
fixture, clearer tool boundary or a corrected specialist reference. Tie the change
to the actual trigger; avoid accumulating broad permanent rules from one incident.

## Worked example: an operational bulk edit

The request is to let an operator change the owner of selected records. Existing
product rules require unselected records to remain unchanged and current filter
state to survive. The accepted condition covers selection, submission, persisted
values after reload and visible failure recovery. No new approval workflow or
batch scheduler is implied.

Build one small selection-to-write-to-reload path first. A reviewer then discovers
that the success toast appears while a stale filter key sends the wrong record
IDs. This is a blocking requirement defect with a reproducible observation. Their
suggestion to make the toolbar float is a preference. Concurrent edits remain
unverified until the project's conflict contract is exercised.

Repair the ID boundary and verify the selected and unselected rows through the
actual consumer path. Recheck the affected interaction at narrow width because
selection controls changed. A fixture-backed check can prove the repaired mapping;
it cannot establish live permissions or production persistence. Report each result
at its actual level of evidence, and retain the regression for this failure.
