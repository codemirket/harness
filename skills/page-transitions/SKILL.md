---
name: page-transitions
description: Design, implement, or review page and route transitions, shared-element navigation, and SPA or multi-page View Transitions. Use for continuity between screens while preserving routing, history, focus, scroll, async ownership and reduced motion; complex in-page choreography uses advanced-motion.
---

# Page transitions

Let this workflow lead navigation motion. Start with the project's actual router,
browser targets, rendering lifecycle and accessibility conventions. Use
`engineering-judgment` or the project's frontend workflow for navigation/data
correctness, `interface-design` for composition, and `advanced-motion` for complex
in-page transformations. Ordinary controls stay with `motion-design`; vector
illustrations and morphs use `svg-creation` and `svg-animation`.

Read [navigation craft](references/navigation-craft.md) before substantial work.
Apply its matching SPA, multi-page, shared-element and interruption guidance.
Registration does not install a router or animation runtime.

## Establish continuity

1. Inspect real source/destination screens and their loading, error and back
   states. Identify what users need to recognize across navigation: an item,
   reading position, hierarchy or a change of context. Avoid animating every
   region or delaying a frequent action for a cinematic entrance.
2. Choose a focal transition and short supporting beats from the existing brand
   and motion tokens. Keep stable navigation stable. Direction must follow the
   actual relationship and history action, not always push content the same way.
3. Make the settled views and immediate-navigation path usable first. Preserve
   links, deep links, refresh, modified clicks, browser back/forward, title,
   announcements, focus and the project's scroll restoration policy.
4. Name initial, pending, committed, interrupted and settled states. Application
   state commits independently of decorative completion; no essential content or
   working control can depend on a finish event that may never occur.

## Choose the implementation boundary

- Prefer supported browser APIs or the project's already installed animation
  integration. Check actual versions and support against official documentation;
  do not adopt experimental releases or introduce a library just for this skill.
- For SPA navigation, connect the animation to the router's real DOM commit.
  A state setter or navigation promise may not mean that the destination has
  rendered. Use the supported framework integration rather than wrapping a
  speculative timeout around navigation.
- For multi-page navigation, consider same-origin cross-document View Transitions
  with both documents opting in. Retain normal links and browser history; check
  target support, lifecycle and back/forward cache behavior. Unsupported browsers
  should retain normal navigation.
- Use catalog `vercel-react-view-transitions` only for a matching React task with
  compatible installed versions. Read its adaptation; a general transition brief
  does not justify changing frameworks, replacing history or rewriting routes.
- If native snapshots or an existing library cannot represent the needed effect,
  use a scoped overlay/layout technique from `advanced-motion`. Keep clones
  noninteractive and hidden from assistive technology; clean up every exit path.

## Integrate without races

Use stable, unique identities for shared elements. Inspect actual old/new bounds,
clipping, aspect ratios, typography and image readiness; a stretched card label
rarely improves continuity. Scope temporary names to the selected entity and
remove them with lifecycle-aware ownership.

Prepare needed data outside a rendering-blocking snapshot transaction. Pending
and error states remain honest. Guard obsolete asynchronous work before commit;
skipping an animation does not cancel a fetch or navigation callback. Handle
rapid A-to-B-to-C requests through the router's existing ownership policy.

Distinguish a visual transition failure from a failed application update. A
skipped capture must not rerun a successful mutation or push another history
entry. Surface actual update errors through the normal error path. Clean up owned
names, snapshots, animations, listeners and controllers on interruption/removal;
an older completion must not remove a newer transition's state.

For reduced motion, omit spatial travel and use immediate navigation or the
project's restrained alternative. Handle the preference at load and live changes.
Focus, title, announcements and scroll behavior belong to navigation, independent
of whether the visual transition runs or finishes.

## Verify the navigation and the motion

Exercise real links, rapid repeats, late data, failure, back/forward, direct entry,
refresh and relevant same-route or hash navigation. Check focus placement,
keyboard access and scroll restoration on both normal and fallback paths. Force
an unsupported/skipped visual path when practical and confirm one application
commit with no duplicate history or stale destination.

Inspect normal-speed playback and important intermediate frames at affected
widths, content lengths and themes. Refine discontinuous identity, stretched text,
double fades, blank intervals, clipped travel and excessive motion. Test reduced
motion at load/live change and disposal. Captured frames do not prove smoothness;
profile the actual renderer when making a performance claim.

Deliver the integrated implementation and concise evidence: what continuity
improved, which routing/interaction cases and browsers were exercised, and any
material untested device or framework boundary. Native navigation, an attractive
settled frame and a declared workflow are separate evidence.
