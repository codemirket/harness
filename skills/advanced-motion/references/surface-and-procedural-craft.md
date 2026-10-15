# Animated surfaces and procedural graphics

Read the part needed by the chosen medium. The [scene mechanics](scene-mechanics.md)
reference owns shared clocks, scroll geometry and interruption; do not create a
second controller for the same property.

## Color and gradients are part of the composition

Give color a job: indicate state, direct attention, suggest changing light or connect
two scene chapters. Choose deliberate endpoints and inspect the path between them.
A full hue cycle is appropriate only when that changing spectrum fits the subject.
Keep required text readable throughout the change, not only in endpoint captures.
Inspect small captions, focus indicators and transparent overlaps at the brightest
and lowest-contrast beats. A stable reading surface can be more useful than trying
to animate text and background in opposite directions.

Choose the mechanism from what actually changes:

- For a simple state change, transition the relevant color property. Avoid
  `transition: all`, which can accidentally animate future layout changes.
- For a wash moving over art, move a bounded gradient layer or change its opacity.
  Keep its coverage adequate at extrema and clip the art layer deliberately;
  do not clip focused content to conceal an edge seam.
- For a transition between two gradient compositions, overlapping resolved layers
  with an opacity blend can be easier to control than interpolating entire
  `background-image` values. Test memory/paint cost before calling it cheaper.
- For changing stops or angles, use supported typed custom properties or an
  explicit draw/update path. Do not assume unregistered custom properties or
  arbitrary gradient strings interpolate continuously in the target renderer.

Color space and hue path affect intermediate colors. Where supported, explicit
gradient interpolation such as `in oklab` or a chosen polar hue path can avoid an
unwanted muddy detour; compare the actual result rather than declaring one space
universally better. Retain an ordinary color/gradient fallback when needed. Wide
gamut, transparency and blending can change the visible result; inspect the actual
background. Never use color change alone for essential state.

Keep large blur/filter areas proportionate to the effect. A transformed enormous
layer can still cost memory and painting. For a loop, compare both color and rate
of change at the seam. Prefer a finite atmosphere change when continuous motion
adds no meaning; prolonged autoplay needs an appropriate pause/stop treatment.

## Expressive materials behind semantic controls

When the brief calls for a liquid or lit surface, keep the artwork in a separate
noninteractive layer behind actual DOM buttons, links and status text. Hide a
decorative SVG from assistive technology, give it `pointer-events: none`, and
keep readable labels and focus rings outside any blur, alpha threshold or
displacement filter. The ordinary control must remain visible and usable before
the effect initializes and when it fails. Do not hide an essential action until
the first rendered frame or a reveal completes.

A small liquid silhouette can be constructed from overlapping SVG shapes, a
bounded Gaussian blur and an alpha contrast threshold; composite color into that
silhouette separately from the content. Inspect the merged edge, clipping and
contrast in the actual integration. A filter's intermediate raster area can be
large even when its paths are simple; measure before claiming acceleration or
low cost. An ordinary solid background is a valid fallback.

Scope definition IDs and material values per instance. A shared renderer may
share its clock or cached immutable geometry, but that does not establish
independent theme/preset state. Render two instances with different settings,
change one while the other remains mounted, and check both. If the implementation
has global material state, respect that limitation or choose an instance-safe
technique; silently letting the last update change every surface is not a theme
system. Recompute affected surface geometry on layout changes rather than polling
unchanged bounds indefinitely. Preserve the lifecycle rules below.

For character construction and optical-size status families, use `graphic-design`;
`svg-creation` owns editable vector geometry.
Choose an expressive material because it serves that composition. The existence
of metal, glow, goo or avatar effects does not justify adding them to a control.

This layering pattern is informed by public MIT
[Libraries.dev Gooey](https://github.com/Jakubantalik/Libraries.dev/blob/a27566557b4def79bcdcedc71858c0fb28020b6d/packages/liquid-gooey/src/Gooey.tsx)
and its `filter.tsx`; the shared-state failure probe follows
[`metal-fx`'s renderer](https://github.com/Jakubantalik/Libraries.dev/blob/a27566557b4def79bcdcedc71858c0fb28020b6d/packages/metal-fx/src/engine/renderer/loop.ts).
No package or paid Studio content is required. Package-specific reduced-motion
behavior varies; inspect initial and live changes in the implementation instead
of assuming one package's handling applies to another.

## Canvas 2D: a drawn layer, not the whole interface

Use Canvas when procedural or dense drawing is its real advantage. Keep primary
text, links, controls and semantic state in the DOM. Give informative drawing an
equivalent description; hide purely decorative drawing from assistive technology.
Provide a visible static poster/vector composition for disabled script, failed
context creation and the chosen reduced-motion treatment. A blank canvas with a
successful JS load is not a usable fallback.

Keep drawing deterministic where the design permits it: `draw(time, geometry,
state)` with stable seeds for procedural variation. Derive time from elapsed
timestamps, not frame counts; refresh rates differ. For scroll-driven drawing,
draw from clamped progress instead of starting another elapsed-time loop. A redraw
on input/resize may suffice when nothing is changing; continuous rendering must
have an actual visual purpose.

Separate logical CSS dimensions from backing bitmap dimensions. Size the latter
for the effective device pixel ratio and a deliberate resolution budget, then set
the drawing transform once for that bitmap. Resizing canvas dimensions resets
context state: restore drawing configuration and repaint. Check narrow dimensions,
zero-size hidden containers and a changed pixel ratio. Avoid accumulating scale
transforms on every resize or growing the bitmap without a measured need.

Own one frame request per scene. Keep the user's pause separate from temporary
offscreen/document suspension, and preserve it on return. Recompute invalidated
geometry and cancel frames/listeners/observers on teardown. Under reduced motion,
draw the chosen readable pose without leaving a perpetual loop running. A live
preference change must reach that pose without restarting an intro.

## WebGL and frame export boundaries

Use WebGL when genuine geometry/depth or a shader effect earns the extra runtime
and GPU work. Probe the actual target before committing to a large scene. Reuse an
installed renderer; a skill is not permission to add a dependency. Bound render
resolution and resource counts from inspection/profiling. Handle context creation
failure, loss/restoration and disposal with a usable static composition; retain DOM
content and controls. Do not require the scene to load for a link to work.

When the deliverable is video, choose exact size, duration, frame rate, alpha/audio
requirements and an available exporter before authoring. Make each output frame
depend on explicit frame time and seeded inputs; wait for required fonts/assets.
Retain editable source separately from the encoded file. Inspect first/last frames,
representative intermediate frames, loop join if requested, and the saved video's
real playback. Check export dimensions, frame count/duration and audio sync where
applicable. Browser seeking proves neither encoding nor editor compatibility.

## Review and evidence

Run desktop and narrow layouts at normal speed, then inspect important sampled
beats. Exercise rapid pointer/focus input, scroll reversal or interrupted playback
according to the scene. Check no-script, reduced motion at load/live change,
offscreen return and cleanup. Diagnostic timelines, frame labels and capture
controls belong in a separate development surface or browser automation, not the
production interface.

When making a performance claim, record the actual browser/device, viewport,
pixel ratio, scene and measurement method. Inspect scripting, layout, painting,
memory and frame behavior relevant to the claim. A deterministic frame set or an
average FPS alone can conceal hitches; do not generalize one machine's observation
to untested mobile hardware.

## Primary references

Checked 2026-10-11; verify implemented support in the target browser.

- [CSS color interpolation](https://www.w3.org/TR/css-color-4/#interpolation) and
  [CSS gradient interpolation](https://www.w3.org/TR/css-images-4/) define color
  paths and gradient behavior; specifications alone do not prove implementation.
- [Registered property animation](https://drafts.css-houdini.org/css-properties-values-api-1/#animation-behavior)
  defines typed interpolation; verify the syntax and actual browser result.
- [Canvas semantics and bitmap sizing](https://html.spec.whatwg.org/multipage/canvas.html#the-canvas-element)
  define fallback, accessible equivalents and context/bitmap behavior.
- [Canvas rendering considerations](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Optimizing_canvas)
  discuss repeated drawing, resolution and measured implementation tradeoffs.
- [Pause, stop, hide](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html)
  explains when moving content needs controls; a media query alone is not a pause control.
- [WebGL context events](https://registry.khronos.org/webgl/specs/latest/1.0/#5.15)
  define creation, loss and restoration boundaries for a GPU renderer.
