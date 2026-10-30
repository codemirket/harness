---
name: ui-motion
description: "Implement interactive animation with tested lifecycle, interruption and reduced-motion behavior in existing UI code. Skip static styling or exported-video production."
---

# UI motion

Bind movement to real UI state and a purpose such as feedback or continuity. Decide how often the user will encounter it before adding it; do not animate a frequent keyboard-initiated action merely for polish. Define initial/final state, trigger, repeated trigger, interruption and unmount/cleanup before implementation. Keep keyboard focus, semantics and layout usable independently of animation.

Prefer the project's existing CSS, Web Animations or animation library for the required lifecycle. Verify the actual installed library/version before using framework-specific APIs. A catalog reference does not authorize a dependency or another animation engine. Avoid multiple owners competing for the same property.

Choose mechanics from the behavior: CSS may cover a simple state transition; Web Animations can supply explicit playback/cancel control; an existing library may be appropriate for shared layout, gestures, springs or a coordinated timeline. Use the installed API rather than combining examples from unrelated versions. Preserve semantic visibility separately from opacity and transforms so exiting content cannot remain a hidden keyboard target.

Define how the current state responds while motion is running. Rapid toggles should settle at the last requested state; list insertion/removal should preserve identity; route changes should cancel obsolete work. Restore owned inline styles, observers, listeners and animation handles on cleanup. Keep focus on a stable logical control rather than moving it to follow decorative motion. When shared layout is involved, verify identity, geometry and interruption in the real component tree.

Inspect actual playback with the project's installed browser tests/tools. The optional Pi peer can supplement local `127.0.0.1` screenshots and navigation-key checks, but fixes reduced motion on and cannot prove normal-speed playback, pointer/form behavior or scripted interruption. Report a missing route when those checks are required. Test first load, rapid reversal/retrigger, navigation/unmount, resize and reduced-motion mode. Confirm that an interrupted animation reaches the correct logical state and leaves no listeners, stale transforms or hidden interactive controls. Profile when there is a demonstrated frame-budget problem rather than claiming smoothness from code shape.

Inspect normal speed and representative intermediate frames. Include content extremes and delayed fonts/media when they affect geometry. A compositor-friendly property can still produce poor pacing, excessive motion or dropped frames under real workload. Retain the reduced-motion journey's information and access; opacity or color feedback may remain when it aids comprehension, while travel and decorative movement can be removed. Check that disabling a decorative sequence does not prevent completion callbacks or leave content hidden.

For framework-specific questions, select only the approved source matching the existing project: pinned [Motion](https://github.com/motiondivision/motion/tree/c5fab238ab4f8d8132d1cb05d6607439b3904058), [Anime](https://github.com/juliangarnier/anime/tree/01b81be1df6843ccfe0a71c0699a746bf740dd77) or [GSAP guidance](https://github.com/greensock/gsap-skills/tree/aed9cfd3277740755f6bfc1155c7aa645403b760). These are optional comparison locators; inspect compatibility and exact reuse rights before adopting material.

Return source, tested lifecycle states and playback evidence. Unless the task sets another budget, share 45 minutes and four final repairs across motion production and review, with one target repair. Stop after the requested behavior/access checks pass or report missing runtime/assets and unresolved defects at the limit. Do not claim exported codec support from a browser preview.
