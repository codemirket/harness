---
name: advanced-motion
description: Art-direct, implement and review expressive website motion, coordinated sequences, animated color and gradients, scroll scenes, procedural Canvas graphics and local web-composition video exports. Use for a substantial visual scene or choreography beyond ordinary controls; SVG art and navigation transitions retain their own specialist workflows.
---

# Advanced motion

Lead substantial browser compositions: sequences across elements, animated surfaces,
changing layout, layered transforms and motion tied to scroll or reactive data.
Choose one lead from the requested result. Ordinary controls, menus and feedback
use `motion-design`; navigation/shared page-transition semantics use
`page-transitions`; vector artwork and path morphs use `svg-creation` and
`svg-animation`. Support those workflows only at an affected DOM boundary.
Use `interface-design` for the page's content and hierarchy, `graphic-design` for
an asset family, and the project's frontend workflow for state/lifecycle integration.
A new visual direction, framework, model or external design service is not implied.

Read only the reference needed for the current decision:

- [Art direction and medium](references/art-direction-and-medium.md) turns a brief
  into a visual concept, motion language and DOM/SVG/Canvas/WebGL/video choice.
- [Scene mechanics](references/scene-mechanics.md) covers coordinated clocks,
  interrupted FLIP, transform/3D topology, scroll ranges and responsive rebuilding.
- [Animated surfaces and procedural graphics](references/surface-and-procedural-craft.md)
  covers gradients, color, Canvas lifecycle and frame-based export boundaries.
- [Rendered examples](references/rendered-examples.md) provides two contrasting,
  editable browser compositions and a reproduction/critique path.
- [Local video export](references/local-video.md) selects HyperFrames for web
  compositions or Motion Canvas for drawn scenes, with inspected local exports.

Reuse the project's tokens and APIs. Examples demonstrate choices, not a required
palette, timing system or architecture. A form-spacing repair or ordinary button
hover does not need this scene workflow.

## Establish the scene contract

Inspect the actual DOM/component tree, current effects, installed versions,
embedding, target browsers and renderer. Identify each animated property's owner,
measured geometry and inputs that can invalidate it. A catalog entry is not runtime
availability; the existing GSAP catalog entries are manual review candidates.

Connect the scene to the brief: who should notice what, which visual relationship
carries the brand, and what should remain still. Choose a small motion vocabulary
that reinforces composition, typography and material; do not start with an effect
inventory. Build and render one representative beat before extending a direction.

Define the trigger, meaningful initial/settled states, focal beats, replay/loop
policy and response to interruption. A brief storyboard in the working task can
show anticipation, action, secondary response and rest when those suit the subject.
Keep application state, focus and essential information independent of decorative
completion. A sequence must not delay an available action to finish its performance.

Choose the reduced-motion composition before building: remove unnecessary spatial
travel, parallax and looping while keeping meaning, controls and content visible.
Account for the preference at load and during the mounted session. Prolonged
autoplay needs the host's appropriate playback controls, not only a media query.

## Choreograph relationships

Establish a readable focal action. Related motion should explain causality,
weight, depth or spatial continuity; supporting elements can overlap without all
moving together. Tune intervals and travel from the subject, distance, density,
task frequency and existing brand. Do not apply overshoot, stagger or parallax to
every element. Keep pauses and a clear settled pose where the brief needs them.

Express beats relative to a shared clock or existing timeline labels. Prefer an
explicit compact schedule over chains of timeouts or completion callbacks; the
sequence should remain intelligible when sought, reversed or shortened. Reversing
a decorative entrance is not automatically the right exit choreography.

Separate static placement, layout compensation, expressive movement and direct
input transforms when they need independent ownership. Nested shells are useful
when an effect would otherwise overwrite another transform. Set transform order,
origin and reference coordinates deliberately; matrix interpolation can take an
unexpected path when endpoints use unrelated transform lists.

## Implement the topology that the effect requires

Prefer native CSS/Web Animations or the project's installed library. Introduce a
library only for a concrete unmet need, following the project's dependency rules.
Do not require GSAP, Motion, WebGL, React or an additional setup/controller layer.
Check current official documentation and actual target support for the technique;
feature detection and a meaningful static fallback are useful where support differs.

For FLIP, capture the current rendered position before cancelling the old effect,
commit the actual layout change, read final geometry together, then apply inverse
compensation and animate it away. Coordinate with the framework's real DOM commit.
Preserve stable item identity. Treat changing size, scroll and transformed ancestors
explicitly; viewport rectangle subtraction is not a general matrix solution.

For CSS 3D, choose a common camera, preserved transform chain and local hinges.
Inspect flattening from opacity, clipping, filters and containment before changing
perspective or adding translation. Leaf faces, layout shells and fade wrappers may
need different roles. Verify backfaces, overlap and bounds at intermediate rotations.

For scroll motion, distinguish a trigger from a continuous progress relationship.
Choose the actual scroller, axis, subject and range; use measured progress or supported
scroll/view timelines. Keep natural scrolling and readable static content. A resize,
font/media arrival, data change or breakpoint can invalidate geometry and ranges.

## Own interruption and lifecycle

Retarget from the current visual state toward the latest requested state. Preserve
velocity only when the chosen API supports it; positional continuity alone does
not prove physical continuity. Invalidate old completion work before it can hide,
remove or reset a newly active scene. Handle cancellation rejections intentionally.

Scope animation handles, observers, listeners and scheduled frames to the scene.
Dispose them on removal; restore only styles the scene owns. Rebuild invalidated
geometry deliberately, preserving semantic state, user pause and sensible progress.
Background/offscreen suspension and resumption should follow the scene's purpose.
Do not cancel unrelated page animations or restart everything on every render.

## Verify the actual result

Review normal playback and sampled important beats, initial/settled poses and loop
seams where applicable. Refine collisions, origin drift, clipping, unreadable overlap,
awkward pacing and velocity jumps. Seeking helps frame inspection; it cannot certify
real-time smoothness. Compare a supplied reference at equivalent beats and placement.
Write a correction against an observed relationship, such as a bright midpoint
competing with the heading or a delayed response breaking causality, then inspect
the same scenario after the change. Keep capture/seek controls outside production UI.

Exercise relevant rapid triggers, retarget/reversal, pause/replay, scroll reversal,
resize/content arrival, unmount/remount, reduced motion and two instances. Review the
actual embedding at intended and narrow sizes, including its readable no-script
and reduced-motion composition. Profile observed jank or a performance
claim with the project's/browser's tools; property names and library choice are not
measurements. Report editable implementation, rendered evidence and untested targets.
