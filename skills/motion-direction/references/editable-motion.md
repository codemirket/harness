# Editable motion contract

Use this reference when a motion artifact must remain editable, be reviewed at deterministic times, or pass between a direction pass and a project-owned source adapter. It is a small contract, not a preset library. Keep supplied artwork, fonts and assets under their existing owners; these examples use only local DOM/SVG and native browser animation APIs (WAAPI and CSS `@keyframes`).

## Contract fields

Record one row per beat or property owner:

| Field | Record |
|---|---|
| Target | Stable element/group ID and its semantic role; never “the second shape.” |
| Owner | The one source rule responsible for each animated property; note any parent transform. |
| Base state | The resting DOM/SVG attribute or CSS value before the animation. |
| Keyframes | Absolute values that intentionally override the base for this beat; do not confuse a changed base with a changed keyframe. |
| Pivot | `transform-origin` or the SVG anchor in the coordinate system where the motion should pivot. |
| Timing | Duration and delay with units, easing, stagger, loop/iteration and the expected final/resting state. |
| Control | Trigger, replay/reverse/cancel behavior, interruption rule, and whether an external controller may seek. |
| Target context | Intended viewport/player/export and any reduced-motion or mobile behavior that actually exists. |

Use one owner for a property. If a keyframe already controls a fill, color, transform or opacity, changing the base value may have no visible effect; edit the owning keyframe or remove that ownership deliberately. Preserve the source geometry and names while exporting or adapting it. A stable ID, valid render or file hash does not prove that the intended pixels or playback are correct.

## Deterministic handoff

For live DOM UI, create WAAPI animations paused at `currentTime = 0`. For finite CSS export candidates, use named `@keyframes` with `animation-play-state: paused`; discover the resulting `CSSAnimation` through the target's `getAnimations()` and pause/seek its `currentTime`. In either case expose a small task-owned seek function that clamps to the known timeline duration; do not invent a renderer or hide playback behind a receipt. A project adapter may drive the same animation from its own clock. HyperFrames remains the source adapter for authored video work; these snippets are browser-native examples, not a replacement.

At minimum inspect the first, an informative intermediate, and the final/resting time. For interaction, inspect reversal or cancellation from a mid-progress state. For type or line work, check the actual path/text geometry at the target size. If the runtime or image-capable inspection is unavailable, report that evidence as untested.

## Original copyable examples

The snippets are intentionally small and use no package, remote asset, redistributed font or SVGator/Remotion source. The live-UI example uses a paused WAAPI animation; the vector and type examples use finite CSS `@keyframes` tracks so a CSS-based export adapter can discover them through `getAnimations()`. Each exposes external seek through `Animation.currentTime`.

### 1. Product interaction: save acknowledgement

The button owns the state; the SVG group owns the acknowledgement transform/opacity. The base SVG values are a fallback, while the keyframes own the short feedback beat.

```html
<button id="save-action" type="button">
  Save
  <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true">
    <g id="save-mark" opacity=".7" transform="scale(.96)"
       style="transform-box: fill-box; transform-origin: center;">
      <path d="M5 12.5 10 17l9-10" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
    </g>
  </svg>
</button>
<span id="save-status" role="status" aria-live="polite"></span>
<script>
const saveButton = document.querySelector('#save-action');
const saveMark = document.querySelector('#save-mark');
const saveStatus = document.querySelector('#save-status');
const saveDuration = 220;
const saveAnimation = saveMark.animate(
  [
    { transform: 'scale(.96)', opacity: .7 },
    { transform: 'scale(1.10)', opacity: 1, offset: .55 },
    { transform: 'scale(1)', opacity: 1 }
  ],
  { duration: saveDuration, easing: 'cubic-bezier(.2,.8,.2,1)', fill: 'both' }
);
saveAnimation.pause();
saveAnimation.currentTime = 0;

saveButton.addEventListener('click', () => {
  saveStatus.textContent = 'Saved.';
  saveAnimation.currentTime = 0;
  saveAnimation.play();
});

window.productMotion = {
  seek(ms) {
    saveAnimation.pause();
    saveAnimation.currentTime = Math.max(0, Math.min(saveDuration, ms));
  }
};
</script>
```

### 2. Vector gesture: draw a local route

The path geometry is editable SVG. Its dash array is the base setup; only `stroke-dashoffset` belongs to the draw-on animation. The path length is measured from the saved geometry rather than guessed.

```html
<style>
#route-stroke {
  --route-length: 0px;
  stroke-dasharray: var(--route-length) var(--route-length);
  stroke-dashoffset: var(--route-length);
  animation-name: route-draw;
  animation-duration: 900ms;
  animation-timing-function: cubic-bezier(.2,.7,.2,1);
  animation-fill-mode: both;
  animation-play-state: paused;
}
@keyframes route-draw {
  from { stroke-dashoffset: var(--route-length); }
  to { stroke-dashoffset: 0; }
}
</style>
<svg id="route-art" viewBox="0 0 240 80" role="img" aria-label="A route drawing itself">
  <path id="route-stroke" d="M12 62 C50 12 92 72 124 34 S190 12 228 46"
        fill="none" stroke="#263238" stroke-width="4"
        stroke-linecap="round"/>
</svg>
<script>
const route = document.querySelector('#route-stroke');
const routeLength = route.getTotalLength();
route.style.setProperty('--route-length', `${routeLength}px`);
const routeDuration = 900;
const routeAnimation = route.getAnimations()
  .find(animation => animation.animationName === 'route-draw');
if (!routeAnimation) throw new Error('route-draw CSSAnimation was not created');
routeAnimation.pause();
routeAnimation.currentTime = 0;

window.vectorGesture = {
  seek(ms) {
    routeAnimation.pause();
    routeAnimation.currentTime = Math.max(0, Math.min(routeDuration, ms));
  }
};
</script>
```

### 3. Typography beat: staggered words without a redistributed font

The content and font remain project-owned DOM. Each named word owns its own animation; the seek function drives all tracks from one external clock.

```html
<style>
#type-beat { display: inline-flex; flex-wrap: wrap; gap: .25em; }
#type-beat [data-type-beat] {
  display: inline-block;
  animation-name: type-beat;
  animation-duration: 520ms;
  animation-timing-function: cubic-bezier(.2,.8,.2,1);
  animation-fill-mode: both;
  animation-play-state: paused;
}
#type-ready { animation-delay: 0ms; }
#type-to { animation-delay: 70ms; }
#type-ship { animation-delay: 140ms; }
@keyframes type-beat {
  from { transform: translateY(.45em); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}
@media (prefers-reduced-motion: reduce) {
  #type-beat [data-type-beat] { animation: none; opacity: 1; }
}
</style>
<h2 id="type-beat" aria-label="Ready to ship">
  <span id="type-ready" data-type-beat style="display: inline-block;">Ready</span>
  <span id="type-to" data-type-beat style="display: inline-block;">to</span>
  <span id="type-ship" data-type-beat style="display: inline-block;">ship</span>
</h2>
<script>
const typeParts = [...document.querySelectorAll('[data-type-beat]')];
const typeDuration = 520;
const typeDelay = 70;
const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
const typeAnimations = reduced ? [] : typeParts.flatMap(part =>
  part.getAnimations().filter(animation => animation.animationName === 'type-beat')
);
typeAnimations.forEach(animation => {
  animation.pause();
  animation.currentTime = 0;
});
if (reduced) typeParts.forEach(part => {
  part.style.animation = 'none';
  part.style.opacity = '1';
});

window.typeMotion = {
  seek(ms) {
    typeAnimations.forEach(animation => {
      animation.pause();
      animation.currentTime = Math.max(0, Math.min(typeDuration + 2 * typeDelay, ms));
    });
  }
};
</script>
```

Validate syntax and the contract statically before handing the snippet to a project adapter. Then use the actual consuming context for pixel and playback inspection; `currentTime` control is deterministic control, not visual proof.
