# Image-to-code and bounded quality gates

Use for new visual work, substantial redesign and expressive scenes. An existing
approved screen is a usable target for a focused repair. Backend changes and
mechanical asset processing do not need a new design exercise.

## Establish one visual target

Inspect the host's available image tools. Use a supplied target when it governs
the task. Otherwise use the available authorized image model to create one full
composition with the intended content, hierarchy, focal asset, palette and use
size before implementing the interface. An isolated decorative asset is not a
page-composition target. Keep exact copy and logos in editable code; generated
lettering is only a placement guide. Several references can resolve distinct
decisions, but synthesize one target instead of starting a sample campaign.

If the host has no usable image model, direct HTML/CSS, SVG or Canvas composition
is permitted. Record the missing capability, construct one minimal composition,
render it and inspect it before expansion. Do not install a provider, change
models or ask for permission merely to use this fallback. State any requested
medium that the available tools cannot achieve; do not claim equivalence.

Freeze the target with its path, source/rights, viewport and digest. Use a separate
output path so later captures cannot overwrite it. Target selection is an agent
decision within the brief, not user approval or an extra approval ceremony.

## Target gate

Inspect the complete composition: first read, typography, image/text relationship,
crop and subject construction. A known material defect means the target is not
ready. Permit one targeted target repair, then stop with the specific gap if it
remains unsuitable. For code fallback, the first rendered composition is this
gate. Do not expand sections, variants or animation while it remains unresolved.

Translate the target into semantic components, responsive rules and usable
states. A screenshot used as the whole interface is not image-to-code. Retain
artwork as an asset when appropriate; keep text, controls and layout editable.
Compile tokens before rendering. Preserve hierarchy, proportions, typography,
palette and focal construction; explain necessary responsive/behavior adaptations.

## Final gate

Render the actual implementation and inspect it alongside the frozen target at
matching content, viewport and state. Inspect a narrow layout separately.

| Check | Required evidence |
| --- | --- |
| Composition | First read, hierarchy, spacing, proportions and crop; explain target deviations |
| Typography | Actual fonts, readable sizes, wrapping and contrast |
| Assets | Source/rights, sharpness/transparency and coherent focal construction |
| Responsive | No clipping/overflow; readable order and reachable controls at narrow/wide sizes |
| Behavior | Real primary action, keyboard/focus and relevant state transitions |
| Motion | Requested real-time playback, intermediate states, interruption and reduced motion; subject movement when promised |

Record pass, fail or not applicable with located observations. Responsive and
behavior cannot be waived for working UI; motion cannot be waived for animation.
Pixel differences locate drift; similarity percentages, screenshots, model
opinions and passing builds do not establish aesthetic quality or user acceptance.

Allow the initial final review plus at most two correction rounds. Fix located
material defects, rerender and repeat affected checks. Budget ten minutes of
active production/verification per bounded composition. Do not reset it by
launching another worker, splitting retries into new tasks or generating extra
variants. At the limit preserve current output and report the failed gate and
remaining defects. Additional work needs new user direction. An independent
reviewer or video export is only needed when the deliverable or uncertainty calls
for it; neither is a default extra production stage.

## Owned worker contract

Design-production requests require one of:

```json
{"design_input":{"mode":"image_to_code","image":"reference/target.png"}}
```

```json
{"design_input":{"mode":"code_fallback","reason":"The host exposes no image model; compose and inspect a minimal coded proof."}}
```

The router supplies the `design_review` action schema. Inspect the target and
submit a passing `target` review before image-to-code mutations. A fallback target
must be produced and inspected output. Implement, render, inspect and submit a
`final` review comparing current output with the frozen target. The runtime
enforces ordering, image identity, current renders, applicable checks, review
count and a 600-second execution deadline. Check notes and fallback availability
are caller/model testimony, not authenticated truth. Native host tools follow the
same instructions under host permissions; Mirket cannot intercept their calls.
