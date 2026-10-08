# SVG creation and animation

The harness now provides two authored, project-selectable workflows:

- [`svg-creation`](../../skills/svg-creation/SKILL.md): vector illustration, icons,
  marks, silhouette, Bézier construction, optical balance, perspective, controlled
  shading, editable geometry and real-size critique.
- [`svg-animation`](../../skills/svg-animation/SKILL.md): vector choreography,
  storyboard beats, path drawing, reveals, compatible morph geometry, transform
  origins, embedding, playback, lifecycle and reduced motion.

The `svg-studio` profile contains only these two skills. The `design-studio` plugin
bundle includes it. Global defaults remain at 14; no library, renderer, editor,
MCP server, hook, account or paid service is added. Native geometry and existing
browser/animation tooling supply the implementation path. A project's already
installed library remains usable when it solves a concrete requirement.

## Apply to a project

For an existing registered project, run from the harness checkout:

```sh
python3 ai.py project add --project /absolute/project --profile svg-studio
python3 ai.py project sync --project /absolute/project
python3 ai.py project doctor --project /absolute/project
```

Use `project init --project /absolute/project --profile svg-studio` for an
unregistered project, adding `--target both` only if Claude Code also needs the
skills. Preserve existing project choices. A new registration requires a fresh
session/client skill refresh where the client discovers skills at session start.
Global links follow this checkout; managed copies and exported plugins need sync
or regeneration. Updating the harness does not add specialists to every project.

The [global routing reference](../../skills/skill-catalog/references/task-routing.md)
selects vector creation separately from SVG animation, ordinary UI transitions
and page composition. `interface-design` and `motion-design` point to the relevant
specialists. Agents should read the selected body and craft reference, construct,
render, critique and refine the actual artifact. XML validity, file count and
declared skill selection do not establish quality.

## Scope and technical basis

These are original craft instructions with short authored mechanical examples;
no upstream skill package, artwork, font or runtime is copied. Technical behavior
was checked on 2026-10-08 against primary specifications linked in the references:
[SVG coordinates](https://www.w3.org/TR/SVG2/coords.html),
[paths](https://www.w3.org/TR/SVG2/paths.html),
[embedding](https://www.w3.org/TR/SVG2/conform.html),
[CSS transforms](https://www.w3.org/TR/css-transforms-1/#transform-box),
[Web Animations](https://www.w3.org/TR/web-animations-1/),
[reduced motion](https://www.w3.org/TR/mediaqueries-5/#prefers-reduced-motion)
and [pause/stop requirements](https://www.w3.org/WAI/WCAG22/Understanding/pause-stop-hide.html).
Specification descriptions do not imply implemented support in every browser or
vector editor; the workflow requires testing the target context.

The optional `svg-creation/scripts/audit_svg.py` uses only Python's standard
library. It checks bounded UTF-8 XML, a useful viewBox, unique IDs and common local
references; it reports active content, image elements, foreign content, live text
and nonlocal dependencies for inspection. It does not execute SVG, validate all
path/CSS syntax, sanitize uploads or judge visual quality. Use the actual project
trust boundary for externally supplied markup.

## Development verification

The retained [interactive coastal study](../examples/svg-studio/preview.html)
includes [editable static artwork](../examples/svg-studio/static.svg) and a
[finite animated SVG](../examples/svg-studio/animated.svg). Open the HTML locally
or through an existing local server to try replay and pause/resume. It is an
original development example, not a required visual style for project assets.

Browser checks caught a meaningful embedding distinction: main-frame media
emulation stopped inline motion but did not freeze the raw SVG image. A fresh
Chromium launch with the reduced-motion flag produced unchanged, settled image
captures, while Playwright's main-frame media query still reported no preference.
These observations do not establish an OS preference toggle. The retained preview
therefore also uses a host `<picture>` static alternative; its selected source
and captures were checked at load and after a live emulated preference change.
The skill reference records this explicit integration path and verification limit.

Registration and browser evidence are recorded separately in
[the verification record](../evidence/svg-studio-2026-10-08.json). It includes
both-provider installation, source payload hashes, structural regressions and a
bounded agent exercise producing editable vector art and motion. Actual rendered
output and interactions are inspected before accepting the exercise.

The exercise is a development sample, not a held-out comparison, measured quality
gain or human acceptance. Browser evidence applies to the exercised renderer;
Safari, Firefox, real mobile hardware, vector editors and print exporters require
their own relevant checks. The skills raise the craft and verification standard;
they cannot guarantee every agent will produce professional artwork without
reference fidelity, iteration and judgment.
