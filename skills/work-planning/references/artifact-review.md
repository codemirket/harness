# Review the artifact, not the author's confidence

Use for visual delivery in any format and substantial cross-boundary delivery.
Apply the checks relevant to the artifact and changed scope. For a small change,
inspect directly. An independent host reviewer with read-only scope can expose
missed defects in substantial or repeatedly unsuccessful work; a second agent is
not a ritual.

## Establish the visual direction before scaling

Define what the viewer should understand, notice or do, the intended display size,
and the visual character appropriate to the brief. Use supplied references or the
project's design system to ground decisions. When anatomy, gesture, machinery or
spatial relationships matter, use suitable visual references to construct them.
Preserve intentional stylization while checking that the subject and action read.

Choose the production medium from the deliverable before constructing it. Use an
available image-generation/editing workflow for complex bitmap artwork, native
geometry for exact diagrams or required editable vectors, and an actual animation
workflow for motion. A website placement alone does not require SVG. Check tool
availability; do not silently replace a missing production capability with crude
code-drawn art, or substitute a still image for an animation requirement.

Resolve the most demanding representative element before building a whole set:
for example, the central illustration, a dense slide or the critical motion pose.
Inspect it at its actual use size and in context. If the chosen method repeatedly
produces weak results, change the construction or permitted medium. Do not keep
polishing a defective foundation or animate it before its static poses work.

## Inspect the actual artifact

Give the reviewer:

- The complete applicable brief, constraints, reference intent and audience;
  preserve exact requirements rather than an abbreviated gallery caption. Identify
  producer-added choices separately so they are not mistaken for user requirements.
- Exact source and artifact paths, how to launch/open them, and the changed scope.
- Required user journeys and known compatibility limits.
- Permission to run the relevant safe local checks; no edits or external actions.

Withhold the author's aesthetic self-rating. Ask the reviewer to open the actual
pages/screens/files and exercise the journey. A report JSON or source diff alone
cannot establish presentation quality. If rendering is unavailable, label the
review structural and identify the uninspected output.
Reconcile the review's scope against the supplied brief: an omitted requirement
remains unreviewed even when the reviewer approves the rest.

Review in this order: can the user complete the task; are content and calculations
correct; can they perceive and operate it; does hierarchy direct attention; are
spacing, typography, illustration, motion and language coherent with the brief?
Inspect the whole composition at the intended viewing size, then zoom into focal
and error-prone details. A thumbnail can hide malformed drawing; an isolated crop
can hide a weak composition. Compare the relevant reference properties explicitly,
without imposing a single aesthetic on every project.

Use the relevant checks:

| Output | Visual questions to resolve |
| --- | --- |
| Pages and websites | Does the hierarchy lead to the intended action? Do typography, spacing, imagery and section rhythm form a coherent composition with real content? Inspect affected widths, states and themes; responsive changes must preserve essential meaning and controls. |
| PDFs and presentations | Inspect every page or slide in the final saved artifact, including full-page composition and readable detail at its intended viewing size. Check narrative sequence, density, alignment, typography, tables, chart scales and labels, image quality and crops. Inspect embedded artwork and diagrams themselves. For revisions, inspect affected pages and any repagination. |
| Images and illustrations | Does the subject, expression or action read without explanation? Check silhouette, anatomy or deliberate abstraction, joints, gestures, grip and contact, object proportions, perspective, occlusion, lighting and material consistency. Look for fused or extra parts, accidental tangencies, malformed text, broken contours and unsuitable image resolution. |
| Diagrams | Trace each essential flow against the facts. Check direction, ownership, containment, sequence, labels and legend; confirm that spatial grouping and color communicate the intended relationships. Inspect crossings, arrow endpoints and legibility at delivery size. |
| Animations | Watch normal playback through a complete sequence or loop, then inspect intermediate poses and transitions. Check timing, continuity, deformation, attachment and contact, depth and occlusion, and loop seams. Exercise interruption, reversal, controls and reduced motion where applicable. Attractive endpoints or sampled frames alone cannot establish coherent motion. |

For office calculations, inspect formula inputs, totals and caches separately.
For interactive output, exercise the journey and keyboard/focus behavior as well.

## Make an acceptance decision

Each finding needs the observable problem, reproduction/location, consequence,
and smallest useful correction. Classify requirement defect, craft weakness,
preference, or unverified path. Do not manufacture findings or turn preferences
into blockers. Do not award numerical beauty scores without calibrated examples.

Reject the current version when a focal element is visibly malformed, the visual
communicates the wrong relationship, essential content is unreadable, or motion
breaks the intended action. Clean layout, a pleasing palette or passing tests
cannot compensate for that defect. Do not describe a redraw-level problem as
optional refinement. Intentional abstraction should remain intelligible and
internally coherent; photorealism is not a universal acceptance criterion.

Record a concise decision in the existing task record or review: ready for the
stated use, needs revision with located findings, or visually unverified with the
missing inspection named. Identify the actual artifact, page/region/state/frame
and relevant viewing conditions. A checklist tick or generated screenshot is not
an observation. User rejection reopens the acceptance decision; do not repeat an
earlier endorsement without addressing the cited defect.

The delivering agent resolves findings against the brief, repairs accepted defects,
reruns affected checks and inspects regenerated output both in detail and in context.
After a repair, recheck the original defect and any affected composition or motion.
Stop when the requested result meets the brief. If a material defect cannot be
resolved within the available tools or scope, deliver it explicitly as unfinished
and explain the remaining work; do not recommend it as ready.

## Keep improvement claims proportionate to evidence

Reviewer agreement is advisory evidence, not a quality certificate. When a review
method approves a known rejected example, retain the missed defect and withhold
claims of reliable acceptance; adding votes or stronger wording does not validate
the method. The delivering agent still owns inspection and the final decision.

For claimed harness improvement, run matched briefs with/without the workflow,
hold model/tools/budget constant, randomize labels for independent judgment, and
retain outcomes and costs. One attractive demo is not comparative evidence.
