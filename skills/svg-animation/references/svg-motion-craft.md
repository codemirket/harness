# SVG motion craft and recipes

Read the recipes relevant to the requested animation. Reuse project timing,
geometry, accessibility and lifecycle conventions. Examples demonstrate mechanics,
not a universal visual style or permission to install an animation library.

## Storyboard, rhythm and restraint

Choose one focal event. For an illustrative reveal, an initial pose, anticipation,
main action, overlapping secondary response and settled pose can describe a
sequence. A quick product confirmation may need only a short reveal and settle.
Do not apply cartoon overshoot or a long intro where the subject or user task
calls for precision. State a few important beats in milliseconds or normalized
progress; implement curves and phase offsets, then tune during normal playback.

Motion should describe form: a sail flexes from its mast, an orbiting point follows
its track, a folding object hinges from its crease. Related elements can overlap
with a small lag rather than appearing as unrelated simultaneous pulses. Give
the eye a rest; moving every detail makes the focal point harder to perceive.
Use a simple continuous loop with matched start/end position and velocity when
looping is requested. A duplicated endpoint alone does not remove a velocity jump.

For fidelity work compare the reference at matching beats and normal speed.
Inspect the actual intermediate shapes, not only endpoint screenshots. Label an
approximation when the source geometry or timing is unavailable.

## Select the delivery mechanism

| Context | Useful starting point | Boundary to verify |
| --- | --- | --- |
| Self-contained animated `.svg` shown through `<img>` or CSS image | CSS keyframes; declarative SMIL for a supported attribute/path requirement | Scripts and interaction inside the image are disabled; host CSS cannot select internal nodes. Test the asset's own styles, media preferences and fallback in this embedding. |
| Inline SVG decorative sequence | Scoped CSS keyframes for simple motion; Web Animations for replay/seek/playback | Unique IDs, SVG transform coordinates, no invisible base state, reduced-motion and controller cleanup. |
| Interactive SVG in an existing application | Existing project animation API or native Web Animations | State ownership, rapid repeated actions, focus/controls, SSR/hydration and disposal. |
| Complex timeline, morph or exported video | Already installed compatible library/exporter when it solves the requirement | Actual plugin/version/license support, contour preparation, reduced-motion and export fidelity. Ask before a new dependency. |

Standalone SVG JavaScript may run when opened as a document but will not run in
image mode. Do not deliver a script-dependent logo as an `<img>` and claim it
animates. SMIL has its own timeline; `getAnimations()` controls CSS/Web Animations,
not a general SMIL pause solution. Choose and test the corresponding API rather
than assuming one controller owns all mechanisms.

For an image with a static alternative, the host can select the unanimated asset
explicitly. This also makes the host's preference contract independently testable:

```html
<picture>
  <source media="(prefers-reduced-motion: reduce)" srcset="art-static.svg">
  <img src="art-animated.svg" width="800" height="600" alt="A sailboat beside a lighthouse">
</picture>
```

Keep the animated asset's own reduced-motion styles for direct use. Check the
host's selected `currentSrc` at load and live changes, and separately render the
raw image with an effective browser/device preference. Main-page emulation and
inline `getAnimations()` do not prove the preference reached an SVG image resource.

## Coordinate systems and transform ownership

For a local bounding-box rotation, set `transform-box: fill-box` and the intended
`transform-origin` explicitly. For motion around a shared canvas point, use the
viewBox coordinate system or a group translated to that point. Validate origin
by sampling at multiple rotations, including the narrow placement.

Use outer groups for static positioning and inner groups for animation:

```xml
<g transform="translate(120 80)">
  <g class="asset-rotor"><path d="M-16 0H16M0-16V16"/></g>
</g>
```

Animating the inner group avoids replacing placement. Avoid CSS, SVG attributes
and a JS library all competing for `transform` on the same element. Scope classes
to the asset/component. Reused inline `defs` and ARIA references need per-instance
IDs, including in SSR; verify two copies together.

## Stroke drawing and filled reveals

Normalize simple line art with `pathLength="1"`, numeric `stroke-dasharray="1"`
and `stroke-dashoffset` from `1` to `0`. Percentage dash values refer to viewport
geometry and are not a substitute for this normalization. Check round caps for
an initial visible dot and `vector-effect`/scaling behavior at the target size.
Do not apply drawing to a filled silhouette and expect its interior to reveal.
Use a clip/mask reveal, a deliberate moving construction or opacity for that form.

An image-compatible finite CSS drawing treatment with a visible default:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 120">
  <style>
    .draw-route { stroke-dasharray: 1; stroke-dashoffset: 0; }
    @media (prefers-reduced-motion: no-preference) {
      .draw-route { animation: route-draw 900ms ease-out both; }
    }
    @keyframes route-draw { from { stroke-dashoffset: 1; }
                          to { stroke-dashoffset: 0; } }
  </style>
  <path class="draw-route" pathLength="1" d="M24 88Q120 8 216 72"
        fill="none" stroke="currentColor" stroke-width="4"
        stroke-linecap="round"/>
</svg>
```

For a mask, choose coordinate units and region deliberately. Alpha/luminance and
mask/clip semantics differ; inspect edges, stroke/filter padding and moving bounds.
The mask's own movement must also obey reduced motion. Preserve a fully visible
static composition outside the motion media query.

## Morphing and motion paths

Simple path interpolation requires compatible topology: subpath count, matching
command structure, point order and winding. Keep correspondence around landmarks
so a contour does not fold or twist through itself. Precompute compatible paths
when practical. A library may normalize paths but does not choose good landmarks
or guarantee clean intermediate silhouettes. Crossfading separate shapes is a
valid alternative when literal morphing adds little value.

For route/orbit motion, use supported CSS offset-path, SMIL `animateMotion`, an
existing library or sampled path geometry according to delivery context. Check
orientation at cusps, tangent continuity and path travel bounds. Avoid reading
length/bounds for every object on every frame; reuse geometry until inputs change.
Neither a declared transform nor a library name proves smooth rendering.

## Controller lifecycle and reduced motion

Use a visible settled base style and opt into motion only when allowed. This
native finite-replay controller illustrates cancellation and live preference
handling for an inline group's opacity. Adapt to project lifecycle and motion;
do not copy it onto nodes owned by a competing animation system.

```js
function attachReveal(group, replayButton) {
  const preference = matchMedia('(prefers-reduced-motion: reduce)');
  let animation;
  const stop = () => { animation?.cancel(); animation = undefined; };
  const replay = () => {
    stop();
    if (preference.matches) return; // Base style already exposes final artwork.
    animation = group.animate([{ opacity: 0 }, { opacity: 1 }], {
      duration: 600, easing: 'ease-out', fill: 'none'
    });
  };
  const changed = () => { if (preference.matches) stop(); };
  preference.addEventListener('change', changed);
  replayButton.addEventListener('click', replay);
  replay();
  return () => {
    stop();
    preference.removeEventListener('change', changed);
    replayButton.removeEventListener('click', replay);
  };
}
```

For interactive reversal, retarget from the current visual state using the
project's supported API; do not enqueue complete old sequences. Invalidate stale
completion callbacks so they cannot hide a new state. A sequence paused manually
should not silently resume because another preference/visibility event occurs.
Scope cancellation to this asset; never cancel all page animations.

Reduced motion needs a chosen final composition, not only `duration: 0`. Remove
unnecessary spatial travel and looping while retaining meaning. Check preference
at load and when it changes. Prolonged autoplay may require pause/stop even when
reduced motion is supported; use the host's accessible controls. For an image
embed where external playback cannot be controlled reliably, prefer a finite
sequence or provide a static alternative rather than claiming pause support.

## Verify motion and costs

Inspect ordinary playback and deterministic samples at meaningful beats. Check
initial visibility, path/mask edges, origin drift, intermediate morphs, overlaps,
settled pose and loop seam. Exercise replay, pause/resume, rapid triggers,
unmount/disposal, preference changes and two instances. Confirm the actual embed
works, not just a direct-open document.

Keep filters and animated detail proportionate to the displayed size. Profile
actual frame times when jank or a performance claim matters. Geometry, filters,
masking and repaint area can be expensive even when some motion uses transforms.
Deterministic seeking or virtual time is capture evidence, not an FPS measurement.
State untested browser/device/exporter targets and retain editable source.

## Technical sources

- [SVG embedding/processing modes](https://www.w3.org/TR/SVG2/conform.html)
- [CSS SVG transform reference boxes](https://www.w3.org/TR/css-transforms-1/#transform-box)
- [Web Animations timeline and playback](https://www.w3.org/TR/web-animations-1/)
- [SVG declarative animation](https://svgwg.org/specs/animations/)
- [Reduced-motion media feature](https://www.w3.org/TR/mediaqueries-5/#prefers-reduced-motion)
- [WCAG pause, stop, hide](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html)
