---
name: svg-animation
description: Design, implement, or critique SVG animation, animated vector illustrations, logo reveals, path drawing, morphs, and SVG motion sequences. Use for vector choreography with editable geometry, reduced motion, playback and integration checks; ordinary UI transitions use motion-design.
---

# SVG animation

Build motion from a strong static vector composition and a clear communication
goal. Let this workflow lead vector choreography; use `svg-creation` for artwork
and the project's frontend workflow for application lifecycle. Ordinary menus,
dialogs and control transitions stay with `motion-design`. Use `page-transitions`
for navigation and `advanced-motion` for a larger DOM timeline or scroll sequence;
this workflow still owns vector geometry, path effects and morph craft.

## Establish the motion contract

Inspect the asset, semantic groups, placement, support targets and existing
animation APIs. Identify what moves, why, trigger, duration, sequence, final
state, repetition, interruption and reduced-motion treatment. For substantial
work, read [SVG motion craft](references/svg-motion-craft.md) and sketch a short
storyboard with important beats. Match the brand; do not animate every path or
add an indefinite decorative loop by default.

Choose delivery context before API:

- Standalone SVG or an HTML image: use supported declarative CSS/SMIL where
  appropriate; image embedding does not run SVG JavaScript or expose inner nodes
  to the host page. Verify the actual embedding and reduced-motion treatment.
- Inline SVG/component: CSS or Web Animations can handle many sequences and
  interactive playback. Reuse the project's installed motion library when it
  resolves an actual requirement. No new library is implied by this skill.
- Video/export: retain editable vector source and use the requested existing
  exporter. A browser animation does not establish video or editor compatibility.

Verify target support for morphing, motion paths and exporter behavior against
current official documentation and a small render. Use a workable transform,
crossfade or path construction when a requested technique is unavailable.

## Choreograph and build

1. Make the unanimated/reduced-motion composition meaningful and visible. Keep
   essential labels, state and controls available without animation or script.
2. Plan a focal action with supporting motion and intentional rests. Set origins,
   phase offsets and curves from the subject's weight and purpose. Continuity and
   restraint matter more than the number of effects.
3. Separate static placement from animated transforms with nested groups. Set
   coordinate origins deliberately; `transform-box`/`transform-origin` defaults
   can produce unexpected SVG rotation. Keep reusable instance IDs scoped.
4. Use normalized dash drawing, masks, transforms, opacity or compatible morph
   contours according to the motion. Read the reference's matching recipe;
   do not substitute a stroke reveal for a filled illustration.
5. Integrate playback and lifecycle. Prevent stacked timelines/listeners on
   replay, reverse from a sensible current state where interactive, and clean up
   on removal. Handle preference changes during the mounted session.

Keep application state independent of decorative completion callbacks. Prefer a
finite entrance or user-triggered replay when it meets the brief. Provide
appropriate pause/stop for prolonged autoplay and honor the project's accessibility
requirements. Avoid flashes; essential information cannot exist only mid-animation.
Offscreen/background motion should not consume avoidable work; resume according
to the semantic task rather than blindly restarting everything.

## Verify temporal and visual quality

Inspect playback at normal speed and sampled frames: initial, important beats,
settled state and loop seam when relevant. Refine jerks, origin drift, clipped
travel, noisy overlap and poorly paced rests. A beautiful final frame does not
prove the sequence. Deterministic seeking is useful for capture, but it does not
measure smoothness or real-time performance.

Exercise the actual trigger, replay, pause/resume and relevant rapid repeat,
reversal, cancellation/unmount. Check reduced motion at load and after a live
preference change; content must remain visible and motion must cease as intended.
Inspect minimum/intended sizes, narrow placement and two instances together.
Measure frame performance only when needed for a reported performance claim.
Include real device/browser checks when the task requires their behavior; report
unsupported or untested targets plainly.

Deliver editable SVG plus the real CSS/component/controller, a static fallback
where required, concise playback/integration instructions and rendered evidence.
State what was exercised and what remains unverified. Registration, a timeline
definition and screenshots alone do not certify high-quality motion.
