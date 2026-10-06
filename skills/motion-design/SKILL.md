---
name: motion-design
description: Implement or review requested interface motion, including interruption, lifecycle, reduced motion, and measured rendering behavior. Use for animation work or observed motion problems.
---

# Motion design

Use for a requested transition or an observed motion problem.

Use `skill-catalog`'s task routing for a platform-specific addition. General
interface animation stays in this workflow. React View Transitions require a
matching task and installed-version support; effect naming uses
`animation-vocabulary`; video production and native app animation need different
capabilities. Do not follow unavailable upstream `animate` or
`review-animations` sibling names as if they were installed.

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
