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

## Translate a brief into motion traits

Choose a small set of traits from the actual message and references, then describe
observable choices. “Curious and light” might mean a head leads a turn, a flexible
antenna follows, and the figure returns to a quiet pose. “Measured and substantial”
might mean deliberate acceleration, a firm arrival and a restrained supporting
response. Treat these as hypotheses to render, not personality defaults for an
industry or illustration type. Color and path associations depend on audience,
brand and context; do not promise a universal emotional response.

Connect the movement to the static composition. The focal silhouette, contrast and
negative space should already tell the story; movement can then direct attention
to a relationship. Supporting motion may remain absent when the action reads well
alone. Do not require primary, secondary and ambient layers for every scene.

| Craft idea | Concrete choice to inspect |
| --- | --- |
| Apparent weight | Relate acceleration, braking, deformation and recovery to the implied material. A rigid object can move decisively without wobbling; a soft appendage may lag. Long duration alone does not establish weight. |
| Anticipation | A small preparation can make an illustrative action legible: a figure gathers before a leap. Omit it when it delays feedback; essential controls respond immediately. |
| Follow-through | Let a flexible part continue after its driver settles, then damp toward rest. Keep the attachment and mass relationships plausible across frames. |
| Secondary response | A supporting eye turn, cloth bend or shadow change can clarify the focal action. Reduce it if it becomes the first read or invents a conflicting light source. |
| Staging and rest | Preserve silhouette and readable labels through overlap; end in a composed pose. Stillness can carry the message and give a finite scene its ending. |

Anticipation, action, response and resolution form one possible short narrative.
Use only the beats the message needs; no universal phase percentages, mandatory
counter-motion, one-third-screen limits or one-third-element concurrency rule.
Opacity-only changes and linear movement are valid when they express the intended
relationship. A routine action does not need a miniature film.

### Contrasting brief examples

- **A curious field-guide companion.** Keep an asymmetrical silhouette beside the
  unchanged question text. The eyes lead toward a discovered specimen; the head
  follows on a small arc, one flexible antenna catches up, and the bot returns to
  a readable attentive pose. Traits: exploratory, light, contained. Start/replay is
  optional; status text and the next action are immediately available. Under reduced
  motion show the attentive pose without the turn. Review whether the antenna
  supports the gaze or competes with it at the smallest placement.
- **A precision instrument introduction.** Keep the scale, captions and primary
  action still. A rigid ring rotates into alignment at a measured pace, a fine
  marker confirms alignment, then the assembly rests. Traits: deliberate, exact,
  grounded. No bounce or ambient glow is required. A straight or constant-speed
  portion may communicate mechanism; inspect braking and alignment rather than
  selecting a curve by industry. Reduced motion shows the aligned instrument.

These are contrasting creative hypotheses, not asset or timing presets. Existing
Solstice and Signal examples show other compositions; inspect their actual normal
playback before treating any effect as a recommendation for a new brief.

### Account for time and spring parameters accurately

For `n` equal animations of duration `d` whose starts are spaced by `s`, the
**stagger span** is `(n - 1) × s`; **total completion** from the first start is
`d + (n - 1) × s`. Eight 200ms animations spaced by 40–60ms have a 280–420ms start
span and complete after **480–620ms**. Include any initial delay separately. For
unequal or overlapping schedules calculate `max(start_i + duration_i)`. Label what
a budget measures; keep controls and semantic state independent of that budget.
There is no universal acceptable stagger threshold.

A physics spring's damping coefficient is not its damping ratio. For a linear
mass–spring–damper model, `ζ = c / (2 × sqrt(k × m))`, where `c` is damping, `k`
stiffness and `m` mass. Ratios `0 < ζ < 1`, `ζ = 1` and `ζ > 1` describe underdamped,
critically damped and overdamped responses. A library's `damping: 20` is not a
ratio of 20 unless its API explicitly defines it that way. Inspect actual parameter
units, initial velocity, rest thresholds and duration/bounce alternatives in the
installed API; do not copy a cross-library table. Tune and inspect the resulting
motion rather than claiming a parameter value guarantees a particular emotion.

## Select a medium for each responsibility

| Need | Preferred starting point | Switch only for a concrete reason |
| --- | --- | --- |
| Readable text, layout, controls, simple surfaces | Semantic HTML + CSS; Web Animations for imperative playback | Avoid redrawing text or controls in a canvas merely to synchronize them |
| Editable illustration, crisp paths, named moving parts | SVG with static groups; CSS/WAAPI or supported declarative animation | Canvas can suit dense repeated drawing when per-object DOM/editability is unnecessary |
| Procedural lines, dense particles, generated textures | Canvas 2D with an explicit draw function | A handful of vector shapes rarely needs a perpetual raster loop |
| Interactive shader effects and WebGL surfaces | Inspect the existing web implementation through Browserless after a small target-device probe | CSS transforms may cover layered planes; neither medium guarantees lower cost |
| Encoded video or a fixed frame sequence | HyperFrames lint, snapshot and render with explicit frame rate and duration | A live page or screen recording does not establish deterministic video delivery |

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

## Plan connected layers before preparing assets

Use this when a scene needs independently moving image planes. A static image,
simple DOM composition or existing footage may already satisfy the brief; depth
does not require a fixed number of layers or a more elaborate renderer.

Record only the relationships the scene needs:

| Part | Motion responsibility | Contact or occlusion invariant |
| --- | --- | --- |
| Environment | Establish distance with restrained movement or none | A clean plate fills every area uncovered by moving foreground parts |
| Subject and support | Carry the focal action | Feet, wheels or a product base stay attached to their intended support unless separation is the action |
| Foreground | Reveal or frame the subject | Its travel never accidentally hides essential copy or exposes unfinished edges |
| Copy and controls | Preserve meaning and usable actions | Semantic DOM remains readable and operable independently of decorative progress |

For a simple grounded cutout, name its contact point in source coordinates and
the corresponding point on its support. A common contact pivot and shared
translation can keep that point aligned while the planes scale differently.
Separate layout placement from expressive transforms and verify the **rendered**
contact after crop, transform order and responsive changes. A shared CSS percentage
origin is insufficient when assets have different bounds. Perspective, rotation
or changing geometry may require another construction; do not force a flat pivot
model onto a real 3D relationship.

For example, a lamp on a plinth can grow in apparent scale as the environment
recedes while its base remains on the plinth. The opening already shows the product
and description; the midpoint reveals the material; the exit settles into the
details. Inspect whether this relationship helps the brief before adding movement.
This is an illustrative construction, not a prescribed hero or brand direction.

Have `graphic-design` prepare registered layers, real alpha and the clean plate
using its production-and-proof reference. Retain the editable/master assets and
their contact/crop coordinates with the delivery exports. A generated checkerboard
is not transparency, and separately generated layers may disagree in light or
perspective even when their prompts match.

Keep a complete poster or static composition until all **required** layers are
decoded and aligned. Optional atmosphere must not block readiness. If a required
layer fails, retain the complete fallback rather than showing half a scene or a
duplicate subject. Reuse the lifecycle ownership in scene mechanics so late loads
cannot reveal a replaced scene. Essential text and actions stay available throughout.

At narrow sizes reconsider crop, contact coordinates, typography, stacking order
and travel together. Inspect opening, an intermediate beat and resolved exit in
both directions, including delayed/failed layers and geometry changes. Look for
floating contacts, duplicated subjects, newly exposed holes, halos and occlusion
of copy. Reduced motion uses the readable resting composition without gratuitous
pinned space. These observations establish the inspected states, not human approval.

Technique adapted from Nate Herk's [scroll-craft hero-depth guidance](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/references/hero-depth.md)
and [layer contract](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/references/approved-collection.md),
MIT; no mandatory depth, layer count, provider or author example assets retained.

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

## Sources and scope

Expressive vocabulary selectively adapts LottieFiles
[motion-design-skill](https://github.com/LottieFiles/motion-design-skill/tree/f9a8a041b85185ee4881b3471d3415e939aac772),
MIT © 2025 LottieFiles: motion personality, emotion mapping, animation principles,
choreography and narrative structure. Mirket's examples and conditional choices
replace universal tables, required layers, unsupported GPU capacity claims and
industry/color defaults. This source does not supply Lottie authoring, export or
renderer verification; those require their actual separate tools and output checks.

The spring distinction is checked against the linear model in
[MIT's Quantifying Uncertainty lectures](https://www.ocw.mit.edu/courses/12-s990-quantifying-uncertainty-fall-2012/8bf391b4f96b87e03cf8328d669ef5e2_MIT12_S990F12_lec2-3.pdf)
and the separate physics/duration options in [Motion's spring API](https://motion.dev/docs/spring).
These explain terminology; they do not require Motion or claim every engine uses
that model. Retained upstream notices accompany the skill payload.
