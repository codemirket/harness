# Medium-aware targets and production budgets

Use for new visual work, substantial redesign and expressive scenes. An existing
approved screen or component system is a usable target for a focused repair.
Backend changes and mechanical asset processing need no new design exercise.
The historical reference path and `image_to_code` request mode remain supported.

## Choose the target for the deliverable

A supplied target governs. Otherwise choose the representation that exposes the
important design decisions before expanding the work:

| Deliverable | Useful target |
| --- | --- |
| Photography, painterly artwork, rich raster concept | Supplied/licensed imagery or an available host image model; inspect the full composition at intended crop |
| Product UI and existing component systems | Actual DOM specimen with real content, fonts, tokens and representative states; inspect its browser capture |
| Editable vector illustration or icon | Native SVG specimen with meaningful groups, attachments and an inspected raster preview |
| Pixel art | Native pixel specimen at intended grid and displayed scale, with palette and sampling checked |
| Diagram or data explanation | Native nodes/edges or chart data and a rendered semantic proof; verify relationships before decoration |
| Typography, paged layout or editable design document | Typeset or native editor specimen with actual text and fonts; inspect the exported page/frame |

Choose image generation when it improves the requested medium. Its availability
does not force UI, diagrams or vector assets through a raster concept. Conversely,
a native specimen cannot stand in for requested photographic or painterly work.
An accepted component system is useful only when its actual rendered composition
and relevant states have been inspected; a DESIGN.md or token list alone is not
visual evidence. Several references may resolve distinct decisions.

For a native specimen, state why the medium fits, construct one minimal proof,
render and inspect it before expanding sections, variants or choreography. The
legacy `code_fallback` path also permits this when the host has no usable image
model; record the missing capability without installing a provider or changing
models. Target selection needs no extra approval ceremony.

Freeze the target with its path, source/rights, viewport and digest. Use separate
output paths so later captures cannot overwrite it. Exact copy, controls and
logos stay in editable source; generated lettering is only a placement guide.

## Target gate

Inspect the complete composition: first read, typography, image/text relationship,
crop and subject construction. Permit one targeted target repair, then report the
specific gap if it remains unsuitable. A native proof must be rendered and
inspected before it becomes the frozen target. Do not expand while this gate fails.

Translate the target into semantic components, responsive rules and usable states.
Retain artwork as an asset when appropriate; keep text, controls and layout
editable. Compile tokens before dependent rendering. Preserve hierarchy,
proportions, palette and focal construction; explain responsive/behavior adaptations.

## Final gate

Render actual output and inspect it alongside the frozen target at matching
content, viewport and state. Inspect narrow layouts for responsive deliverables.

| Check | Required evidence |
| --- | --- |
| Composition | First read, hierarchy, spacing, proportions and crop; explain target deviations |
| Typography | Actual fonts, readable sizes, wrapping, language shaping and contrast |
| Assets | Source/rights, sharpness/transparency and coherent focal construction |
| Responsive | No clipping/overflow; readable order and reachable controls at narrow/wide sizes |
| Behavior | Real primary action, keyboard/focus and relevant state transitions |
| Motion | Requested playback, intermediate states, interruption and reduced motion; subject movement when promised |

Record pass, fail or not applicable with located observations. Responsive and
behavior cannot be waived for working UI; motion cannot be waived for animation.
Pixel differences locate drift; similarity percentages, builds and model opinions
do not establish aesthetic quality, semantic accuracy or user acceptance.

## Budget the actual production

Declare only actual deliverables in `task_needs`. These are finite scheduling
limits, not estimates of typical speed or promises of completion:

| Workflows | Shared wall time | Final correction rounds after initial review |
| --- | --- | --- |
| `static-graphic`, `diagram`, `vector-art` | 15 minutes | 2 |
| `web-interface`, `editable-interface`, `paged-media` | 30 minutes | 3 |
| `motion`, `interactive-motion` | 45 minutes | 4 |

The largest applicable row governs a combined composition, never the sum. The
production clock includes inference, rendering and verification in the execution
loop; dependency setup and discovery precede it. Reserve time for final export
and inspection based on actual renderer cost. Agree the output scope before
starting: one responsive flow, one bounded scene, or a defined set of pages.
Large campaigns need explicit independently useful deliverables, not repeated
attempts disguised as new tasks. Adding an unneeded workflow to buy time is invalid.

Fix located material defects, rerender and repeat affected checks. Stop when
required checks pass; unused rounds are not a quota. At a time or review limit,
preserve output and report failed gates and remaining defects. Do not relaunch,
change the target or create variants to reset limits. More work needs new user
direction. Native host work uses the same budgets as guidance; Mirket only
enforces time and evidence inside its owned worker loop.

## Owned worker contract

Design-production requests require one of:

```json
{"design_input":{"mode":"image_to_code","image":"reference/target.png"}}
```

```json
{"design_input":{"mode":"native_specimen","medium":"dom","reason":"The existing component system and real text determine this responsive UI."}}
```

```json
{"design_input":{"mode":"code_fallback","reason":"The host exposes no usable image model; compose and inspect a minimal coded proof."}}
```

Native media are `dom`, `svg`, `pixel`, `diagram`, `typeset` and `design_document`.
This declaration selects no new tool or permission. Use assigned tools that can
produce the requested medium; report gaps. For an already inspected native
specimen supplied by the host, `image_to_code` can reference its frozen capture.

The router supplies the `design_review` action schema, resolved
`execution_seconds` and `max_correction_rounds`. Image-to-code requires a passing
inspected target review before mutations. Native/fallback targets must be
produced and inspected output. Implement, render, inspect and submit a `final`
review of current output. The runtime enforces sequence, image identity, current
renders, applicable checks and bounds. Medium suitability, minimal-proof scope,
source-to-render correspondence and review notes still require host judgment.
