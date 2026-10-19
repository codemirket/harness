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
them in the page before extending a long sequence. Judge the focal art at rest,
the subject's isolated movement and the complete composition separately. A good
page entrance cannot compensate for a weak drawing or an inert requested subject.
Preserve an established brand
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
| Interactive shader effects and WebGL surfaces | Inspect the existing web implementation through Playwright after a small target-device probe | CSS transforms may cover layered planes; neither medium guarantees lower cost |
| Encoded video or a fixed frame sequence | HyperFrames lint, snapshot and render with explicit frame rate and duration | A live page or screen recording does not establish deterministic video delivery |

Combine media when their roles are clear: semantic DOM copy over a Canvas field,
an editable SVG focal object inside a CSS color scene, or DOM controls around an
existing WebGL viewport. Give each animated property one owner and share scene
time/progress where coordination matters. Do not add a framework, model, service
or library to satisfy a medium label. Inspect actual target support and installed
tools before promising the delivery path.

For vector work, read `svg-animation` and `svg-creation` for geometry/embedding.
For a real route change, read `page-transitions`; scroll narration does not require
a router. For video in Mirket, use HyperFrames lint, snapshot and render, then
inspect the encoded output. Verify its supported SVG features against the actual
tool schema and rendered frames.

For independently moving image planes, read [connected layers](connected-layers.md)
for registration, contact, clean plates, readiness and responsive recomposition.

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

Diagnose the cause before changing every parameter:

| Observed weakness | Useful first experiment |
| --- | --- |
| The object feels weightless | Keep the camera and decoration still; inspect contact, acceleration and braking of the subject alone |
| The action is hard to read | Inspect the preparation, peak and settled silhouettes; change pose, occlusion or staging before adding effects |
| Everything feels equally important | Remove a competing motion temporarily and compare attention at normal speed |
| Motion feels mechanical despite elaborate easing | Check path, pivot, part timing and follow-through; changing duration alone may preserve the defect |
| The artwork is weak even when paused | Repair its construction or choose the appropriate asset medium before tuning choreography |

Treat these as isolated hypotheses, then restore the scene and check the total
effect. For an unsettled tempo, compare a restrained and a more energetic reading
of the same action at equivalent scale. Do not change artwork, layout and timing
simultaneously and attribute the result to easing. Preserve useful rest; movement
counts and continuous activity are not measures of expressive quality.

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
industry/color defaults. The attribution supplies motion vocabulary. Mirket video
production uses HyperFrames lint, snapshot and render with output inspection.

The spring distinction is checked against the linear model in
[MIT's Quantifying Uncertainty lectures](https://www.ocw.mit.edu/courses/12-s990-quantifying-uncertainty-fall-2012/8bf391b4f96b87e03cf8328d669ef5e2_MIT12_S990F12_lec2-3.pdf)
and the separate physics/duration options in [Motion's spring API](https://motion.dev/docs/spring).
These explain terminology; they do not require Motion or claim every engine uses
that model. Retained upstream notices accompany the skill payload.
