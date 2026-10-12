# Browser motion examples

These are editable, fictional compositions for studying decisions and testing
integration. They are not client brands, product claims or required templates.
Adapt content, hierarchy, palette, geometry and timing to the actual brief; copying
one example's aesthetic is not evidence that a capability fits different brands.

| Example | Inspect when the task needs | Deliberate direction |
| --- | --- | --- |
| [Solstice](../assets/solstice/index.html) | Type-led landing sequence, layered color, scroll relationships and editable SVG integrated into a page | Warm editorial space and a composed vector focal point |
| [Signal](../assets/signal/index.html) | Procedural Canvas artwork coordinated with semantic DOM, input and scroll | Dark graphic contrast and a more geometric, rhythmic composition |

The production pages use native browser media. They require no separate model,
paid server, design service or framework installation. Runtime/browser support and
rendered quality still need verification in the consuming project.
Each example is authored for Mirket under the included MIT `LICENSE.txt`; retain
that notice when copying or redistributing it. Fonts are system stacks, not bundled
font files, so inspect line breaks on the actual delivery platform.

## Locate and run the installed payload

Select `advanced-motion` explicitly through the project's existing Mirket flow:

```sh
mirket project add --project /absolute/project --skill advanced-motion
mirket project sync --project /absolute/project
mirket project doctor --project /absolute/project
```

After sync, the examples are in `.agents/skills/advanced-motion/assets/` for Codex
or `.claude/skills/advanced-motion/assets/` for Claude. Use the actual selected
target path. These are managed files: copy the chosen example into an ordinary
project-owned example directory before adapting it; do not edit installed payloads.

Serve that copy using the project's already available local development/static
server and open its `index.html` through the host browser. For a server rooted at
the copied `assets` directory, the routes are `/solstice/` and `/signal/`. Use the
project's established command and bind locally for a local preview. Do not run a
package download, upstream installer or `npx` helper merely to open an example.
If no suitable server/browser is available, name that missing prerequisite rather
than treating `doctor` as a render. A `file:` opening does not prove the intended
HTTP/module/resource behavior.

When Python 3 is already available, its standard-library server is an optional
local preview command, not a Mirket runtime requirement:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 --directory /absolute/project/examples/motion
```

With both copied example folders there, open `http://127.0.0.1:8765/solstice/` or
`http://127.0.0.1:8765/signal/`. Stop the local server after inspection. Do not install
Python merely to follow this example when a project server is already available.

## Inspect the design and its failure paths

Use the production entry to judge the page without diagnostic UI. Observe the
actual entrance, reading sequence and interaction at normal speed before seeking
or pausing for frame inspection. The initial frame must not strand text or actions
invisible. Keep the editable SVG source separate from its page controller and
exports; inspect it both as an asset and at its integrated size.

Capture desktop and narrow states with normal and reduced motion. Include the
focal overlap/midpoint, final rest, hover/focus and a meaningful scroll position.
Inspect the no-script/static fallback. For temporal evidence use the browser's
available recording or scoped animation inspection; deterministic captures do not
measure real-time smoothness. Any development-only controls must remain outside
the production entry and be omitted when adapting the example into a product.

Exercise a discriminating failure probe: quick repeated input, scroll forward
then backward, or interrupt a sequence with reduced motion. Check that the latest
semantic state wins, content stays usable, no stale completion changes it later,
and the resulting artifact still renders. Check console errors and teardown when
the page is embedded into a component lifecycle.

Record one located visual observation, repair its source and reinspect the same
scenario. Useful criticism describes a relationship: a luminous layer washing out
a caption, SVG motion clipping at narrow width, or an overlap obscuring the focal
action. Do not use a universal taste score or claim a screenshot is owner approval.
Retain the changed source, relevant frames/recording and the tested environment;
report actual invocation and visual inspection separately from installation.
