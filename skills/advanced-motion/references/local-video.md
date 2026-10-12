# Editable source to local video

Use an engine for encoded video or exact frames. Interactive website motion
stays with the project's real DOM/CSS/SVG/Canvas and browser QA.

Choose `hyperframes` for finite web compositions with local media and a GSAP
score. Choose `motion-canvas` for drawn scenes with paths, gradients, text,
clipping and generator-driven choreography. Both are optional macOS ARM64 runtime
profiles; inspect their installed schemas before promising a format or feature:

```sh
mirket package show hyperframes
mirket package install hyperframes
mirket package probe hyperframes --schemas --project /absolute/project
mirket package probe motion-canvas --schemas --project /absolute/project
```

Use the existing model and host tools. Serving runs no npm/install hook, hosted
service or additional model. Package selection, readiness, actual invocation,
usable artifacts and visual acceptance remain separate facts.

## Web compositions

Copy [the six-second editable specimen](../assets/hyperframes/composition.json)
and its complete local assets into a visible project directory; do not edit a
managed skill copy. Its editorial direction is an example, not a default style.
Change the concept, copy, hierarchy, geometry and rhythm to match the brief.

A version-1 manifest declares HTML `entry`, explicit `assets`, `width`, `height`,
`fps` and `frames`. Entry/assets are relative to the manifest directory. Use even
64..1920 dimensions with at most 2,073,600 pixels, 24/30/60 fps and at most 900
frames/30 seconds. Inputs total at most 32 MiB, each asset at most 8 MiB.

The root has `data-composition-id="main"`, `data-start="0"`, matching
`data-width`, `data-height` and `data-duration`. Register the finite GSAP timeline
as `window.__timelines.main`. The pinned local GSAP helper is available at
`/__mirket/gsap.min.js` during Mirket capture. An independent HTTP preview must
map that route to the same installed package asset; the example does not duplicate
third-party runtime files. Do not use wall-clock timers, unseeded randomness,
scroll history or accumulated frame deltas for an export score.

Keep text in DOM, editable vector geometry in inline SVG and dense drawing in
Canvas. Declare all image/font/media files. Mapped SVG images are static; animate
inline SVG through the owned score. Declare media timing using the actual profile
contract rather than assuming a playing audio/video element is captured. Unsupported
or incomplete inputs are errors to resolve before rendering.

```sh
mirket package call hyperframes composition_inspect --project /absolute/project --arguments '{"source":"film/composition.json"}'
mirket package call hyperframes composition_frame --project /absolute/project --arguments '{"source":"film/composition.json","frame":45,"output":"film/beat.png"}'
mirket package call hyperframes composition_verify --project /absolute/project --arguments '{"source":"film/composition.json"}'
mirket package call hyperframes composition_render --project /absolute/project --arguments '{"source":"film/composition.json","output":"film/film.mp4"}'
```

Frame indices are zero-based. The real producer/player performs capture; matching
FFprobe independently checks the resulting encoding. MP4 and WebM profiles are
bounded and do not imply support for every upstream format. Read the tool's current
schema and error rather than supplying arbitrary codec flags.

## Drawn scenes

Read the selected `motion_canvas_*` schemas. The editable JSON describes nodes,
geometry, palette/gradients, text, clips and explicit timed changes. Mirket compiles
only its fixed generated source, without importing project build configuration or
hooks. The actual Motion Canvas scene graph and playback managers own drawing and
timing. Its source is an editable drawn scene, not a standalone animated SVG.

Use `motion_canvas_inspect`, `motion_canvas_frame`, `motion_canvas_verify` and
`motion_canvas_render` to inspect saved source, important beats and the resulting
video. Preserve SVG artwork separately when a vector handoff is also needed.

## Judge the saved result

Inspect initial, overlap, focal, settled and loop-seam frames where applicable.
Forward/reverse sample checks establish bounded seeking consistency, not every
frame's quality or runtime smoothness. Watch the saved video at normal speed;
repair pacing, copy fit, clipping, seams and hierarchy in editable source, then
rerender the affected evidence. A hash or correct frame count cannot judge taste.

Jobs have a 90-second deadline and 32-MiB output cap. Shorten/simplify work that
exceeds it. Existing outputs need explicit overwrite intent; use a known revision
hash when replacing a reviewed artifact. Offline composition code runs in Mirket’s
constrained Podman profile. Native browser code uses host permissions; scoped paths
and blocked requests alone do not create an OS sandbox.

For page embedding, provide suitable controls, description, poster and a usable
static/reduced-motion path. Keep diagnostic capture machinery outside production
UI. Deliver editable source, requested export and concise inspected evidence.
