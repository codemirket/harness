# Visual production

Mirket supplies visual guidance, scoped production tools and artifact checks.
It does not certify aesthetic quality. A valid render, an AI review and user
acceptance are different observations.

For the current source decisions and exclusions, see the
[ten-source design review](design-source-review.md).

## Required image-to-code workflow

For design production, inspect a supplied target or create a full composition
with the host's available image model before implementation. When the host has
no usable image model, direct composition in code is permitted: render and
inspect one minimal proof before expansion. Keep text, layout and controls
editable; a flattened screenshot is not a working interface.

This applies to new visual work, substantial redesign and expressive scenes.
An existing approved screen can be the target for a focused repair. Backend-only
changes, mechanical asset processing and reviews of existing work do not require
a new visual composition.

### 1. Inspect and freeze the target

Check the full composition before extending it: hierarchy, typography, focal
asset, crop and intended use. Retain its path, digest, source/rights and viewport.
Keep subsequent captures at separate paths. A material target defect needs a
repair before implementation continues. Target selection is an agent decision
within the brief; it does not add a user-approval ceremony.

### 2. Implement editable structure

Translate the target into semantic components, responsive rules and working
states. Keep exact copy and logos in editable source and explain necessary
adaptations. Compile design tokens before rendering.

### 3. Compare current output and record all six checks

| Check | Evidence |
| --- | --- |
| Composition | Hierarchy, spacing, proportions, crop and explained target deviations |
| Typography | Actual fonts, readable sizes, wrapping and contrast |
| Assets | Source/rights, sharpness, transparency and coherent construction |
| Responsive | Narrow/wide reflow, no clipping and reachable controls |
| Behavior | Real primary action, keyboard/focus and relevant transitions |
| Motion | Requested playback, intermediate states, interruption and reduced motion |

Record pass, fail or not applicable with located observations for each check.
Responsive and behavior cannot be waived for working UI. Motion cannot be waived
for an animated deliverable. A failed applicable check prevents completion.

The [image-to-code contract](../skills/interface-design/references/image-to-code.md)
defines target and final gates for composition, typography, assets, responsive
layout, behavior and requested motion. Freeze the target and compare current
rendered output at matching content, viewport and state. Use several references
when they resolve distinct questions; one reference is not a universal quality
standard. Do not add a mandatory reviewer or video export to ordinary UI work.

### Code fallback when the host has no usable image model

Record the missing capability, then compose one minimal HTML/CSS, SVG or Canvas
proof. Render and inspect it before adding sections, variants or animation.
The first passing render becomes the frozen target and the same final review
applies. Do not install a provider or change models simply to avoid this fallback.
Be explicit about media the available tools cannot produce.

### Runtime contract and limits

All eight design-production workflows require `design_input`: `web-interface`,
`editable-interface`, `static-graphic`, `diagram`, `motion`, `interactive-motion`,
`paged-media` and `vector-art`. The host supplies one of these shapes on the
`route_task` or `spawn_subagent` request (these are fields, not complete requests):

```json
{"design_input":{"mode":"image_to_code","image":"reference/target.png"}}
```

```json
{"design_input":{"mode":"code_fallback","reason":"This host has no usable image model; compose and inspect one minimal coded proof."}}
```

The target image path is relative to the assigned project. The image must be
available inside that workspace and inspected before the target review.

An owned design worker requires `design_input` with `mode: image_to_code` and
`image`, or `mode: code_fallback` and the host's availability `reason`. Before
image-to-code mutations, the runtime requires target inspection and a passing
`design_review` target action. Fallback must bind its target to produced, inspected
pixels. Final review binds current rendered pixels, compares the frozen target,
and records six explicit check outcomes. Missing, stale, failed or waived required
checks prevent completion. Target selection allows one correction; final review
allows two. The execution loop has a 600-second deadline. Do not relaunch workers
to reset these limits; retain incomplete output and report remaining defects.

These runtime gates enforce sequence, identity and bounds for declared design
workflows. They do not authenticate a host's capability declaration, prove that a
capture represents the intended source, measure aesthetic quality, or intercept
native host tools. Native hosts receive the same workflow in installed guidance.
No sample campaign is required to use or validate these structural gates.

### Verify activation without producing samples

After updating, start a fresh host session and inspect its live Mirket tools.
Both routing and worker request schemas must expose `design_input`.
`capability_plan` must mark all eight design-production workflows with
`design_input_required: true`. Read this reference through the live connection:

```json
{"skill_id":"interface-design","path":"references/image-to-code.md"}
```

Use `route_task` for a routing-only rejection check: a design request missing
`design_input`, or containing an empty fallback reason, must fail. A valid route
preview returns `execution: not_dispatched` with the review contract. Do not call
`spawn_subagent` to verify activation. An installed version number or a separate
CLI process does not establish that the current host connection has reloaded.

## Choose the actual production route

Start with the requested subject, medium, placement and editability. The
graphic-design skill covers art direction and critique; its
[image-production reference](../skills/graphic-design/references/image-production.md)
describes capability discovery and the handoff to a host image-generation/editing
tool. Mirket's isolated workers have no image generator. They can compose
supplied assets, author SVG, render diagrams and prepare/export existing pixels.
Use an available authorized host producer for rich raster illustration, or
report the missing capability before promising that result.

Keep exact logos and business text in the editable composition. Inspect final
pixels at actual use sizes, full composition and focal detail. A generated image
that looks attractive alone can still fail its intended layout or crop.

## Preserve an explicit model choice

The host's selected model is not automatically the model used by a routed worker.
Read `mirket models list`, then set `model_selection` on the worker request with
the exact discovered `target`, `id` and supported `effort` the user chose. Preview
with `mirket models route --request /absolute/request.json` before execution.
The choice narrows eligibility and prevents substitution. Unsupported effort,
missing model or an unmet capability floor yields no eligible candidate; it
does not authorize another provider or weaker execution. Omit the selection
only when ordinary policy-based routing fits the user's instructions.

## Keep useful visual context

Owned `inspect_image` calls can label pixels as `reference`, `comparison` or
`artifact`. Up to two references receive retention priority within four images
and eight MiB total; the latest inspection always receives room. Changed or
missing images are removed. Unchanged references and earlier output remain
context across revisions, but every mutation still requires fresh verification.
Use legible contact sheets or reread evicted images when a larger set matters.

## Critique against more than one example

The brief and brand requirements govern. Several relevant examples may teach
different principles; similarity to one example is not the success criterion.
Use [art direction and critique](../skills/graphic-design/references/art-direction-and-critique.md)
to examine concept, composition, construction, coherence and placement fit.
Locate defects, distinguish preference from correctness, preserve strengths,
and revise the highest-impact issues. Independent review can help consequential
work but is neither automatic nor proof of user acceptance.

The [illustration evaluation suite](../evals/illustration-quality/README.md)
contains eight original varied briefs and a separate craft-review protocol.
Use fixed comparable configurations, preserve failed trials and collect fresh
transfer evidence before claiming a general improvement. Its shipped templates
are not observed results or a quality certification.
