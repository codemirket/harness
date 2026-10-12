# From a visual brief to motion

Use this when the requested result needs a visual concept or several coordinated
effects. Reuse the current brief/task record; this is not another approval stage.

## Make the direction specific enough to build

Extract audience, message, next action, actual copy, brand constraints and reference
material. Distinguish supplied facts from fictional demonstration content. Inspect
references for useful relationships: type against image, density against rest,
material against lighting, or movement against a stable anchor. Do not infer motion
from a still image or reuse reference assets without rights.

Name a concrete organizing idea, then a few decisions that make it visible:

| Decision | Record what changes the composition |
| --- | --- |
| First read | The subject or message that should win attention; what stays secondary |
| Type | Roles, scale contrast, line breaks and reading rhythm in the available font |
| Color and material | Surface/text/accent roles; flat ink, paper, light, glass or another justified treatment |
| Space and depth | Alignment, negative space, overlap and the real reason for dimensionality |
| Motion language | Focal action, direction, weight, pace, supporting response and resting pose |
| Restraint | The elements that remain still and the information never delayed by motion |

“Premium and dynamic” predicts little. “Quiet specimen captions surround an
oversized botanical silhouette; a sun-colored field warms once as the stem opens,
then the page rests” predicts choices. Another brief may justify hard graphic
cuts, compact grotesk type and a measured procedural pulse. These are illustrative
directions, not default aesthetics. Change hierarchy, material and rhythm when a
second direction is requested; a palette swap alone is not a new concept.

Build a static composition and one representative animated beat first. Inspect
them in the page before extending a long sequence. Preserve an established brand
unless changing it is part of the task. A complex scene is not required for an
ordinary usability or visual-spacing repair.

## Select a medium for each responsibility

| Need | Preferred starting point | Switch only for a concrete reason |
| --- | --- | --- |
| Readable text, layout, controls, simple surfaces | Semantic HTML + CSS; Web Animations for imperative playback | Avoid redrawing text or controls in a canvas merely to synchronize them |
| Editable illustration, crisp paths, named moving parts | SVG with static groups; CSS/WAAPI or supported declarative animation | Canvas can suit dense repeated drawing when per-object DOM/editability is unnecessary |
| Procedural lines, dense particles, generated textures | Canvas 2D with an explicit draw function | A handful of vector shapes rarely needs a perpetual raster loop |
| Real 3D geometry, camera/light relationships, shader field effects | Existing WebGL tooling after a small target-device probe | CSS transforms may cover layered planes; neither medium guarantees lower cost |
| Encoded video or a fixed frame sequence | Existing frame-based renderer/exporter with explicit frame rate and duration | A live page or screen recording does not establish deterministic video delivery |

Combine media when their roles are clear: semantic DOM copy over a Canvas field,
an editable SVG focal object inside a CSS color scene, or DOM controls around an
existing WebGL viewport. Give each animated property one owner and share scene
time/progress where coordination matters. Do not add a framework, model, service
or library to satisfy a medium label. Inspect actual target support and installed
tools before promising the delivery path.

For vector work, read `svg-animation` and `svg-creation` for geometry/embedding.
For a real route change, read `page-transitions`; scroll narration does not require
a router. For video, select the applicable catalog/native video workflow and verify
the actual exporter. An editor adapter's supported subset is not the browser's SVG
feature set; inspect its schemas before sending an asset.

## Write a short motion score

Describe meaningful beats rather than animating every element on entry. Record
trigger or progress range, focal property, support, rest, and interruption policy.
A type-led opening may be immediately readable while one background layer settles;
the next section can respond to scroll without replaying the hero. A diagram may
need a path to lead a label; an action button must not wait for that narrative.

Choose which relationship drives each effect: elapsed time, input state, scroll
progress or data. Keep unrelated clocks separate. A finite entrance need not scrub
backward; a progress scene must respond sensibly to reverse scrolling. State what
happens on replay, resize, rapid input and a live reduced-motion change.

Define the static/reduced composition alongside the score. Remove distracting
travel or looping while preserving content, control semantics and the focal idea.
Use ordinary reading order on narrow screens if the wide choreography loses its
spatial relationship. Do not stretch page length or hide essential text to make a
timeline easier to demonstrate.

## Critique the relationship, then revise

Watch ordinary playback before seeking. Does attention move in the intended order?
Does the motion feel like the chosen material? Are pauses long enough to read but
short enough that actions feel available? Inspect initial, overlap, peak and settled
frames for type contrast, clipping, tangencies, seams and unintended focal points.

Record an observed defect and its consequence, then change its owner. For example:
“At the palette handoff, the luminous field obscures the small caption; keep its
surface stable while the art changes.” Reinspect the same beat, normal playback
and affected narrow/reduced states. A favorable screenshot or self-awarded score
does not establish timing, accessibility, performance or human visual approval.
