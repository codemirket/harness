# GSAP in an existing browser project

Read only when GSAP is installed or its addition is authorized for an actual
need. This is a focused integration reference, not an installer or a replacement
for the installed version's official documentation. CSS/WAAPI or an existing
Motion integration may already cover the work. Video export retains HyperFrames
lint → snapshot → render and inspection of the delivered media.

## Compose a score with owned state

Register only the plugins used. Establish the component/scene scope and create
related animations inside its `gsap.context()` or existing `useGSAP()` lifecycle.
Use timeline labels for meaningful beats and relative offsets for their
relationships. Keep application state and control availability independent of
timeline completion. Seek and reverse the scene to inspect overlapping property
ownership, not merely its final pose.

Inspect `from()`/`fromTo()` initial rendering when several tweens share a target.
An immediately applied start state can overwrite an earlier beat before its turn.
Choose explicit start ownership or `immediateRender` behavior for the actual
sequence; do not stack arbitrary delayed tweens until the flash disappears.
Use one owner per animated property. Separate layout placement from expressive
transforms where the existing DOM topology requires it.

## Measure a scroll relationship

Separate an entry trigger from scroll-linked progress. Choose the actual scroller
and measure geometry after the relevant layout is ready. For a horizontal track,
pixel travel is `Math.max(0, track.scrollWidth - viewport.clientWidth)`. To reveal
the right-hand content by moving the track left, target `x: -travel`. `xPercent`
expresses a percentage of the animated element,
not CSS pixels. An overflow-free track needs no artificial pinned journey.

When the design calls for continuous scroll control, declare `scrub` deliberately
and a measured `end` range. Use function-based dimensions and appropriate
`invalidateOnRefresh` behavior for a changing layout. Reconsider the scene at
breakpoints; do not preserve a desktop pin on narrow content by habit. Recompute
after relevant font/media/layout changes without calling refresh every frame.
Create triggers in document order when possible. If refresh ordering really
requires `refreshPriority`, higher values refresh earlier, not later.

Reduced motion should expose the readable composition without gratuitous pinned
space. A matchMedia branch can own the moving version and its teardown. Confirm
that switching preference or breakpoint restores the styles and geometry that
the now-active composition needs.

## Own callbacks as well as tweens

`contextSafe()` associates GSAP objects created by a later callback with the
context. It does not cancel that callback's timer, remove its event listener or
make arbitrary asynchronous work safe after unmount. Cancel scheduled work,
remove listeners and invalidate stale completions explicitly. Revert the local
context on disposal. A component must not kill every ScrollTrigger on the page.

In React, test mount → cleanup → mount and two simultaneous instances. Choose
dependency/rebuild behavior deliberately so re-rendering neither duplicates
timelines nor preserves stale measurements. When using `@gsap/react`, inspect
the actual installed hook contract, including `revertOnUpdate` where applicable.
Do not introduce the hook package merely because the reference mentions it.

## Inspect the result

Exercise normal playback, overlap frames, reversal, resize, late content, live
motion-preference changes and removal during pending callbacks. Verify that an
unrelated scene survives teardown. For scroll, test before/within/after the range
in both directions, including zero overflow. A scrub option, transform tween or
successful registration proves neither smoothness nor visual appeal; profile
actual jank and judge the subject's movement against the brief.

## Sources and corrections

Selective methods adapted from greensock/gsap-skills at
`aed9cfd3277740755f6bfc1155c7aa645403b760` (MIT © 2026 GreenSock).
Mirket corrects the reviewed scroll example's `Max.max` typo, pixel/percentage
mix-up and missing scrub relationship, and narrows callback-cleanup claims.
No upstream code sample or runtime dependency is bundled here. Full notice
accompanies the skill. API corrections were checked against the official
[ScrollTrigger](https://gsap.com/docs/v3/Plugins/ScrollTrigger/) and
[React lifecycle](https://gsap.com/resources/React/) documentation; check the
installed API again when implementing version-sensitive behavior.
