---
name: scroll-motion
description: "Implement requested scroll-linked scenes or page transitions with trigger/state maps, responsive behavior and interruption checks. Skip ordinary document scrolling."
---

# Scroll motion

Map the requested narrative to scroll ranges, scenes, entry/exit states and the normal document reading order. Decide what remains understandable with motion reduced or unavailable. Inspect real content height and assets before choosing pinned regions or trigger positions.

Use the project's established scrolling/animation tools. Derive geometry from current layout and refresh it when content, viewport, fonts or media change. Avoid stealing keyboard/page navigation or trapping the user in a scene. Keep one owner for each pinned element/property and clean up triggers on navigation.

Specify each scene's start/end condition, content visible before/within/after it, sticky or pinned behavior and the handoff to the next scene. Choose a scrubbed relationship only when scroll position should express progress; event-triggered entrances need different replay rules. A page transition additionally needs an owner for navigation, old/new view lifetime, history restoration and focus. Keep that contract distinct from a decorative reveal.

Keep a useful document flow before effects initialize and when motion is reduced. Avoid blank spacer regions, permanent hidden content and sections whose only access is a precisely timed gesture. Scale or remove a pinning treatment when its content cannot fit the available viewport. Treat horizontal scenes, nested scroll containers and late-loading media as geometry decisions, not a fixed desktop pixel sequence.

Inspect forward/reverse scrolling, large jumps, reload at an interior position, resize/orientation change, slow/late assets and reduced motion through Codex's available browser. Check focus, anchor navigation, sticky overlap, content visibility and the final resting state. A smooth short recording does not prove the scene survives these conditions.

Exercise an anchor or keyboard focus target inside a scene and confirm it remains reachable and visible. Check browser back/forward behavior when routing is part of the task. Retain evidence at scene boundaries and after interruption, as well as normal-speed playback. Measure a demonstrated performance issue against the real page workload before claiming a frame-rate improvement.

The approved pinned [Scroll Craft](https://github.com/nateherkai/scroll-craft/tree/75d81f74e83692add18cd7a8a8e078b8a887a579) and [GSAP guidance](https://github.com/greensock/gsap-skills/tree/aed9cfd3277740755f6bfc1155c7aa645403b760) sources are optional comparison locators for a specific scene problem. They do not authorize a new engine, workflow or reuse of upstream examples.

Return the trigger/state map, source and observed responsive/interruption evidence. Unless the task sets another budget, share 45 minutes and four final repairs across motion production and review, with one target repair. Stop at the requested checks or that limit and report remaining defects. Do not add scroll theater to an unrequested ordinary reading surface.
