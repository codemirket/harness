---
name: motion-design
description: Implement or review ordinary UI animation, including hover feedback, expand/collapse, enter/exit, interruption and reduced motion. Use for control motion and observed interface animation problems; navigation uses page-transitions and complex choreography uses advanced-motion.
---

# Motion design

Use for a requested transition or an observed motion problem.

Use `skill-catalog`'s task routing for a platform-specific addition. General
control animation stays in this workflow. Route page/route transitions and shared
navigation identity to `page-transitions`; complex timelines, layout transformations,
scroll sequences and CSS 3D choreography use `advanced-motion`. Read only the
matching craft reference and keep one lead. Effect naming uses
`animation-vocabulary`; video production and native app animation need different
capabilities. Do not follow unavailable upstream `animate` or
`review-animations` sibling names as if they were installed.

Vector illustration motion, logo reveals, path drawing and morphs use catalog
`svg-animation` and its SVG motion-craft reference; `svg-creation` supports geometry.
This workflow still owns the surrounding interface transitions. Use a vector
specialist for the actual need rather than routing every animated icon to a new
library or video renderer.

For implementation or critique, read the matching sections of
[interaction recipes](references/interaction-recipes.md): controls and feedback,
menus/dialogs, disclosure and tab changes, list/data updates, or gestures. The
worked examples show how to choose motion from actual state transitions. Reuse
the project's tokens and components; the suggested values are tuning starting
points, not required defaults.

1. Identify what changes, what the motion communicates, how often users encounter
   it, and whether it delays an action. Sometimes the best change removes motion.
2. Inspect the current tokens, animation APIs, and component lifecycle. Select
   the smallest implementation that supports interruption, exit, and cleanup.
3. Coordinate origin, direction, duration, and easing with the surrounding UI.
   Begin from existing conventions; verify new values by rendering the result.
4. Exercise rapid repeats, reversal, cancellation, unmounting, and reduced motion.
   Test gesture velocity and scrolling conflicts on real hardware if applicable.
5. Use frame inspection or profiling for jank; do not infer performance solely
   from CSS property names. Provide actionable findings with observed evidence.

Source techniques: Emil Kowalski `skills/animate/SKILL.md`, `skills/apple-design/SKILL.md`,
and `skills/review-animations/SKILL.md`, MIT at commit
e8a175de22ae1e49370fc144c1f3bb9aeedf988d. These recipes intentionally do not carry
upstream fixed timing tables, blanket easing bans, or mandatory sibling invocation.

For repeatable evidence use available browser tools or an approved registered
renderer through `mirket tool run`. Capture timed frames after input, include
normal/reduced motion and an explicit resulting-state assertion.
The frame offsets start at capture; inspect reversal live and profile when needed.
A static screenshot comparison that disables animations cannot verify motion.
