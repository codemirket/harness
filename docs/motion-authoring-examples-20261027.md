# Motion authoring examples

This comparison records the bounded original examples in [`skills/motion-direction/references/editable-motion.md`](../skills/motion-direction/references/editable-motion.md). It is an implementation aid for Mirket's existing motion/vector owners, not a new runtime or a catalog entry.

| Example | Editable source | Primary owner | Contract exercised | External control |
|---|---|---|---|---|
| Product save acknowledgement | Local button + inline SVG group | Button state plus `#save-mark` transform/opacity | Stable ID, base fallback, keyframe ownership, 220 ms feedback and reset/replay | `window.productMotion.seek(ms)` → `Animation.currentTime` |
| Vector route gesture | Local SVG path + finite CSS `@keyframes` | `#route-stroke` dash geometry/offset | Measured path length, explicit base dash setup, 900 ms draw-on and resting end | `window.vectorGesture.seek(ms)` → discovered `CSSAnimation.currentTime` |
| Typography beat | Local DOM spans + finite CSS `@keyframes` | One animation per named word | Project-owned text/font, 70 ms stagger, reduced-motion static fallback | `window.typeMotion.seek(ms)` → discovered `CSSAnimation.currentTime` |

## Why these three

They cover one user response, one editable vector gesture, and one typographic sequence without copying SVGator/Remotion source, fonts or assets. They are deliberately small enough to compare in a project preview: inspect the start, intermediate and final times, then test interruption or reduced motion where the target supports it.

The Save example uses native WAAPI for live UI only. Vector and type use finite CSS animations because the current HyperFrames CSS adapter discovers CSS animations through `getAnimations()`; the runtime also has a separate WAAPI adapter. The lead independently verified a finite native-WAAPI translation in five exported frames; the Save example remains a live UI interaction, not that export fixture. These snippets do not define a project-preview helper API or custom timeline bridge. HyperFrames remains the source adapter for authored video, and a consuming adapter may drive the exposed seek functions without changing the source contract.

## Bounded validation

On 27 Oct 2026 I ran the three extracted examples in a real Chromium browser page in addition to the static extraction and in-memory script-parser checks. The product example's click handler produced the visible `Saved.` status and its SVG mark used the declared `fill-box`/center pivot; the vector example discovered its named CSS animation and `seek(450)` changed the computed `strokeDashoffset`; and the typography example discovered three named CSS animations, retained explicit `inline-block` declarations, and produced non-identity transforms at an intermediate seek. This is browser behavior evidence on the local Chromium runtime, not saved-pixel inspection, export parity, accessibility certification or smooth-playback evidence.

Reproduction identity: Codex CUA opened a new tab in the normal Google Chrome app (`mcp__cua_repl`: `getApp("Google Chrome")`, New Tab, address-bar navigation) to a transient `data:text/html;charset=utf-8,...` route containing the three extracted examples plus a small in-page results `<pre>`. The selected executable was `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome`, reporting `Google Chrome 156.0.8078.48` on this macOS host. The transient harness was not retained as a repository script or log (`test script path: none; log path: none`); the observed result was the browser's rendered/accessibility output, recorded above. This used Codex's ordinary Chrome only, not Mirket's private Chrome, which remains reserved for video rendering.

The relevant runtime checks remain project-owned: use a capable reviewer for creative judgment, inspect saved pixels with an image-capable tool, and verify the actual consuming context. A valid snippet, hash or seek function does not establish visual quality, accessibility, export parity or smooth playback.
