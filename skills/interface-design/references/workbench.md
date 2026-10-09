# Browser evidence you can act on

Start the project's app using its existing workflow. Use the actual page and real
components. Prefer an installed browser automation capability; the harness also
provides a repeatable local scenario runner. It needs its documented runtime and
browser already available; missing dependencies are reported, not installed here.

## Find the harness

Use the installed `skill-catalog` skill's `scripts/harness.py` wrapper; it resolves
its source checkout, including copied installations. Do not assume a personal
absolute path. For a standard Codex installation:

```sh
python3 ~/.agents/skills/skill-catalog/scripts/harness.py workbench browser --scenario /absolute/scenario.json --output /absolute/new-review
```

For Claude Code/Claude Desktop Code use the same relative wrapper under
`~/.claude/skills/skill-catalog/`. If working inside the harness checkout, the
identical command is `python3 ai.py workbench browser ...`. The output directory
must be new. Use `--help` for current runtime requirements and allowed URLs.

## Describe a real interaction

This scenario targets the dependency-free `examples/craft-lab/index.html`. Serve
that directory with an existing local static server. The example below assumes
`python3 -m http.server 8765 --bind 127.0.0.1 --directory examples/craft-lab` was
started from the harness checkout. Keep the server process bounded to the task.

```json
{
  "schema_version": 1,
  "url": "http://127.0.0.1:8765/",
  "viewports": [
    {"name": "desktop", "width": 1440, "height": 1000},
    {"name": "mobile", "width": 390, "height": 844}
  ],
  "motion": ["no-preference", "reduce"],
  "actions": [
    {"op": "click", "selector": "[data-testid=filter-paper]"},
    {"op": "fill", "selector": "[data-testid=search]", "value": "Cotton"},
    {"op": "click", "selector": "[data-testid=preview-cotton]"},
    {"op": "capture", "label": "preview", "times_ms": [0, 90, 240]}
  ],
  "assertions": [
    {"op": "visible", "selector": "[data-testid=preview-dialog]"},
    {"op": "count", "selector": "[data-testid=material-card]", "expected": 1}
  ]
}
```

Make a second scenario for closing the dialog, restoring focus, toggling a saved
item and viewing Saved. A third should produce zero search results and recover
through Clear filters. Scope assertions to actual behavior; do not count a
screenshot as evidence that an unexercised interaction works.

## Read the evidence

Inspect the page and captures, not just the result JSON. Check hierarchy and
content at normal size, keyboard focus, long labels, empty states and the narrow
layout. Watch playback to judge interruption and timing; frame captures alone do
not establish either. An animation starting midway through capture is not a
precise zero-time sample. For exact timeline comparisons, use a development-only
seek handle or pause/set time on the relevant animation engine.

Keep automated checks, agent visual review and human acceptance separate. A
review should say, for example, “The selected filter disappears below the search
on mobile; move the summary next to the result count,” not “make it more premium.”
Never update screenshot baselines solely to silence failures.

## Make an illustration that fits

Write an asset brief with the subject, composition, palette, lighting or material
treatment, intended pixel size and placement. Specify where text needs negative
space and which edges may be cropped. Use the project's supplied/licensed assets,
an available native image-generation tool, or an explicitly approved provider.
This harness supplies the workflow, not a portable image-generation service;
provider access, account connection and runtime must already be available or be
separately authorized.

Inspect the generated source at full size, then place it in the real component.
Check desktop/mobile crops, contrast behind text, seams, transparency edges and
unwanted marks or lettering. Keep the source and its prompt/provenance alongside
the delivered export when the project permits. For a vector deliverable, retain
editable paths, shapes and text; embedding a bitmap inside an SVG wrapper does
not make it editable vector artwork. Use the workbench's SVG renderer on an
actual vector master and inspect its contact sheet before delivery.

## Reference choices, not mixed branding

The user's reference shelf includes GitHub, Airbnb, Apple, WhatsApp, OpenAI and
Slack, alongside HIG, Carbon, Base, Polaris and Primer. Treat these as reference
preferences, not a prescribed visual recipe. Inspect the relevant live flow when
using one: repository navigation, booking, a product story, conversation, or a
workspace are different jobs. Identify one useful behavior or composition choice
and explain how it fits the current task. Do not copy brand assets or assume a
familiar product's surface appearance makes a new interaction usable.

- [Primer dialog](https://primer.style/product/components/dialog/) provides a
  useful action/focus-return model for previews.
- [Carbon filtering](https://carbondesignsystem.com/patterns/filtering/) informs
  filter visibility and understandable result changes.
- [Apple motion](https://developer.apple.com/design/human-interface-guidelines/motion)
  is relevant platform guidance; verify the actual interaction and accessibility
  behavior rather than borrowing a decorative animation style.
- [Polaris](https://shopify.dev/docs/api/polaris) and
  [Uber Base](https://base.uber.com/) are additional user-selected references.
  Use their relevant workflow/component documentation when the task fits.

Fieldwork Studio is an original demonstration with a restrained work-product
layout, warm surfaces and editable SVG material studies. Its search, filters,
favorites and preview are functional; material records are fictional. It is not
an imitation of these systems or evidence of measured agent-quality improvement.
