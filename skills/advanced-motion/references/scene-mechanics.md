# Scene mechanics and craft

Read the sections matching the scene. These worked mechanics are native examples;
adapt them to the existing API and component lifecycle. Parameters come from the
storyboard and rendered tuning, not a universal duration table.

## A shared clock, with intentional overlap

Describe a sequence as beats and property owners before writing effects. For a
product assembly, a body can settle while a lid begins to hinge; a label may appear
once the silhouette is legible. Starting every part together erases that relationship.
Large travel usually needs a different curve from a subtle secondary response.
Use delay/overlap to keep the focal action readable, not to make users wait.

This native example coordinates individual effects; do not assume a group-effect
timeline API is available. On a mounted active document, a common start time aligns tracks.
Each plan row owns its node/property and contains `frames`, `at`, `duration`, `easing`:
`reportAnimationError` represents the host's existing error reporter.

```js
const origin = document.timeline.currentTime;
const tracks = plan.map(({ node, frames, at, duration, easing }) => {
  const animation = node.animate(frames, {
    delay: at, duration, easing, fill: 'both'
  });
  animation.startTime = origin;
  animation.finished.catch(error => {
    if (error.name !== 'AbortError') reportAnimationError(error);
  });
  return animation;
});
const seek = time => tracks.forEach(a => { a.pause(); a.currentTime = time; });
const dispose = () => tracks.forEach(a => a.cancel());
```

Check an active timeline before creating tracks; `currentTime` can be unresolved.
Use visible settled base styles matching the final pose. Filled effects retain
property ownership; cancel them after the intended base state is installed or at
disposal. `commitStyles()` is not a universal cleanup step: persisted inline values
can override responsive styles. Seeking all tracks uses scene time, including delays.
Cancellation may reject `finished`; expected cancellation must not leak as an error.
See [timing/playback](https://www.w3.org/TR/web-animations-1/) and
[startTime](https://developer.mozilla.org/en-US/docs/Web/API/Animation/startTime).

For interactive retargeting, sample current values before replacing an effect,
then animate toward the latest destination. Use an existing spring/gesture API
when velocity matters. Track a generation or current-handle identity for completion
work. A stale exit completion cannot remove an object after a newer entrance.

## Interrupted FLIP: commit layout, compensate presentation

This translation-only recipe handles surviving, stable DOM shells during a
synchronous reorder. Shells have no competing transform, no rotated/scaled ancestor,
and a consistent scroll position between reads. It preserves text size; size changes
still happen immediately. `timing` is the project's chosen duration/easing.

```js
function attachFlip(getShells, timing) {
  const reduce = matchMedia('(prefers-reduced-motion: reduce)');
  const active = new Map();
  let disposed = false;
  const rects = () => new Map(Array.from(getShells(), n => [n, n.getBoundingClientRect()]));
  const stop = () => {
    for (const animation of active.values()) animation.cancel();
    active.clear();
  };
  function update(commitLayout) {
    if (disposed) return;
    const first = rects(); // Includes the current animated presentation.
    stop();
    commitLayout(); // Must synchronously commit the actual DOM layout here.
    const last = rects(); // All geometry reads precede new animation writes.
    if (reduce.matches) return;
    for (const [node, to] of last) {
      const from = first.get(node);
      if (!from) continue; // New/removed identities need their own presence policy.
      const dx = from.left - to.left, dy = from.top - to.top;
      if (!dx && !dy) continue;
      const animation = node.animate([
        { transform: `translate(${dx}px, ${dy}px)` }, { transform: 'none' }
      ], { ...timing, fill: 'none' });
      active.set(node, animation);
      animation.finished.then(() => {
        if (active.get(node) === animation) active.delete(node);
      }, error => {
        if (error.name !== 'AbortError') reportAnimationError(error);
      });
    }
  }
  const changed = () => { if (reduce.matches) stop(); };
  reduce.addEventListener('change', changed);
  return { update, dispose() {
    disposed = true;
    stop();
    reduce.removeEventListener('change', changed);
  } };
}
```

For framework rendering, take First before the update and Last in the framework's
post-commit/layout phase, before paint. Guard overlapping asynchronous updates by
generation. Do not insert an arbitrary timeout and assume it observes the right DOM.
For transformed ancestors use a consistent local/matrix coordinate space or the
installed library's tested layout implementation. Add actual scroll deltas when
scroll can change between snapshots; document-relative and viewport-relative reads
cannot be mixed. Identity follows the item, not its changing list index.

If size continuity matters, inverse scale uses First/Last width and height ratios
with an explicit origin, commonly the top-left of a dedicated shell. Scaling text,
strokes and corners distorts them; a position-only move, clipped resize or separate
content layer may look better. Account for zero/hidden bounds, virtualized rows,
enter/exit copies and focus semantics through the project's existing behavior.

## Transform topology and CSS 3D

Allocate roles before tuning numbers: layout shell → compensation shell → expressive
object → local face/hinge. Do not add every layer when one owner suffices. An inner
rotation should not overwrite a parent's travel. `translate(...) rotate(...)` and
`rotate(...) translate(...)` describe different paths; nested shells make the local
coordinates explicit. Individual `translate`/`rotate`/`scale` have a defined composition
order with `transform`; they are not an arbitrary reordering mechanism.

For a hinged panel, place its rotation origin on the actual edge, then compare
closed, partly open and open poses. A center-origin rotation describes a spin.
Keep transform-list structure compatible when possible; matrix decomposition can
choose a surprising intermediate rotation. When dimensional motion represents a
solid object, show consistent thickness/occlusion rather than unrelated card tilts.

A parent's `perspective` gives descendants a common camera; `perspective()` in an
individual transform participates in that element's local transform order. Choose
camera distance from scene scale and depth so near faces do not explode or cross
the camera. `transform-style: preserve-3d` is not inherited: relevant intermediate
nodes need to preserve the chain. Backface visibility controls visual painting,
not keyboard/semantic availability of controls on the reverse face.

Grouping values can force a preserved node to flatten: opacity below 1, filters,
clipping/masking, `overflow: hidden` and paint containment are common causes.
`overflow: clip` and `visible` are exceptions to that overflow grouping rule.
Keep a scene's fade/clip wrapper outside its preserved chain or treat individual
leaf faces; inspect actual rendering. Perspective/transforms also create stacking
or containing-block effects that may change overlays and fixed descendants.
See the current draft's [transform model](https://drafts.csswg.org/css-transforms-2/),
[flattening rules](https://drafts.csswg.org/css-transforms-2/#grouping-property-values)
and [perspective](https://drafts.csswg.org/css-transforms-2/#perspective-property).

Navigation capture names also affect topology: a non-`none` `view-transition-name`
flattens that node's 3D rendering and creates a stacking context/backdrop root even
without an active transition. Keep capture identity off a required preserved-depth
node; inspect a flat leaf or an outer wrapper around a self-contained 3D scene.
See [rendering consolidation](https://drafts.csswg.org/css-view-transitions-1/#rendering-consolidation).

## Scroll: trigger, progress and range are different decisions

Use an intersection trigger for a finite reveal when position need not continuously
drive the effect. For a scrubbed scene, progress should map reversibly from the
actual scroll range; a scroll handler that restarts a time-based animation will lag
and drift. Native scroll progress uses the scroller's scrollable range; view progress
uses a subject's passage through its scrollport. Choose the correct subject/scroller,
logical or physical axis, inset and animation range. A named timeline may need
appropriate scope when the animated target and subject are separate branches.

Use a visible base composition. This CSS excerpt assumes `--scene-travel` has a
deliberate value; browser support for the actual syntax still needs target review:

```css
.scene-art { opacity: 1; transform: none; }
@supports (animation-timeline: view()) and (animation-range: entry 0% cover 35%) {
  @media (prefers-reduced-motion: no-preference) {
    .scene-art {
      animation: scene-reveal auto linear both;
      animation-timeline: view(block);
      animation-range: entry 0% cover 35%;
    }
  }
}
@keyframes scene-reveal {
  from { opacity: 0; transform: translateY(var(--scene-travel)); }
  to { opacity: 1; transform: none; }
}
```

Declare timeline/range after the `animation` shorthand so a reset cannot detach
the intended driver. Check no-scroll/inactive ranges and content already visible
on load; a scroll-linked entrance must not strand essential information hidden.
If using a JS fallback, batch reads, clamp progress with a nonzero range and share
one scheduled update per scene. Preserve native scroll input; do not add scroll
hijacking, artificial page length or pinning merely to demonstrate the technique.
See [scroll/view timelines and ranges](https://www.w3.org/TR/scroll-animations-1/)
and [Chrome's implementation guide](https://developer.chrome.com/docs/css-ui/scroll-driven-animations).

## Recompute deliberately; review in real time

Cache geometry until its inputs change. Observe the relevant container size, not
only window resize; also handle fonts/media, data and breakpoint changes when they
affect layout. ResizeObserver reports size changes, not transform movement. Avoid
feedback loops that resize the observed box on every notification; coalesce work
and act only when the geometry inputs actually differ. Keep reads together before
writes. See [Resize Observer](https://drafts.csswg.org/resize-observer/).

On rebuild, preserve meaningful state and explicit user pause. Either retain
normalized progress against the new geometry or settle/retarget from the current
pose according to the interaction. Cancel superseded effects and pending frames,
disconnect observers and remove listeners at disposal. Do not globally clear styles.
Live [reduced-motion changes](https://www.w3.org/TR/mediaqueries-5/#prefers-reduced-motion)
must reach the chosen readable composition without triggering a fresh spatial intro.

Review normal-speed playback plus important sampled beats; inspect acceleration,
handoffs, overlaps, bounds and loop velocity. A matched first/last position can
still have a visible velocity seam. For measured jank, record the actual trigger
and device/browser context; inspect scripting, layout, paint, layer costs and frame
timing. Restrict `will-change`/large filters to an observed need and release owned
hints. Transform/opacity can still be expensive with large surfaces or effects.
See [runtime profiling](https://developer.chrome.com/docs/devtools/performance).
Sources checked 2026-10-08. These primary references describe evolving APIs; they
do not certify target support, aesthetic quality or performance of an implementation.
