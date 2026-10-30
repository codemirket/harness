# Mirket editable-file authoring

Use this reference for the JSON scene and typed patch route. It is independently
worded guidance for Mirket's admitted OpenPencil 0.16.0 subset, not the upstream
CLI, JSX renderer, Figma Plugin API or a connection to the running editor.
Check `runtime_status` and the active tool schema before use.

## Scope and operations

Use `design_prepare` with the explicit `project_root`, `source_path`,
`expected_revision`, new `destination_path`, and active `expected_release_id`.
The revision is `sha256:` followed by the source file's SHA-256. Start the returned
operation with `design_start`; inspect its terminal result before consuming the
destination. Keep prepare/start/terminal evidence with the artifact. A revision
mismatch requires rereading the source and reviewing the intended edit.

The `operation` envelope selects `kind: "open_pencil"`; its nested `operation`
supports only:

| Nested operation | Input → output | Target |
| --- | --- | --- |
| `{"kind":"create"}` | JSON scene → native `.fig` | Scene nodes |
| `{"kind":"patch","edits":[…]}` | `.fig` → new `.fig` | Exact slash-delimited node paths |
| `{"kind":"export_svg","target_path":"Button"}` | `.fig` → `.svg` | One named node and its subtree |

Paths are relative to the sole page: `Button/Label`, not a page name or a native
node ID. Names must be nonempty, unique among siblings, and contain no `/` or
control characters. Avoid renaming/reparenting assumptions: this patch operation
changes properties, not structure. Keep original files and source identities;
do not treat a changed file hash as proof of an intended visual change.

## Construct the supported structure

A scene has exactly `page` and `nodes`. Every node needs `kind`, `name`, `width`
and `height`; only `frame` and `component` accept `children`. The other kinds are
`rectangle`, `ellipse` and `text`. Required text is one nonempty line. Current
bounds are 256 nodes, 16 node levels, 1 MiB input, 16 MiB output, dimensions
1–2048 px and 1–64 edits per patch. Keep requests small enough to inspect.

Use `fill` as `#RRGGBB` or `#RRGGBBAA`, `opacity` in 0–1 and supported uniform
`corner_radius` in 0–512. Text exposes `text`, `font_size` (1–256) and
`line_height` (1–512). The font is fixed regular Noto Sans; there is no font-family
or weight property. Text boxes are fixed, single-line, left/top aligned and do
not measure, shrink or wrap their contents.

Containers accept `layout: "horizontal"` or `"vertical"`, `gap`, four explicit
`padding_*` values, `primary_align` (`start`, `center`, `end`, `space_between`)
and `counter_align` (`start`, `center`, `end`). Gap and padding are 0–512 px.
Omit child `x` and `y` inside these layouts. With `layout: "none"`, place children
explicitly and omit spacing/alignment properties.

For each layout container, verify that main-axis child sizes plus gaps and both
paddings fit its size; every cross-axis child must fit between its paddings.
`space_between` distributes spare room after the minimum gap. Geometry validation
checks box sizes, not painted text bounds. Fix overflowing content at its source;
clipping is unavailable and cannot serve as a repair.

For example, this native component has two editable children and a container
that owns their positions:

```json
{
  "page": "Components",
  "nodes": [{
    "kind": "component", "name": "Button",
    "width": 240, "height": 48, "fill": "#155EEF",
    "layout": "horizontal", "gap": 12,
    "padding_top": 12, "padding_right": 16,
    "padding_bottom": 12, "padding_left": 16,
    "counter_align": "center",
    "children": [
      {"kind":"ellipse","name":"Mark","width":20,"height":20,"fill":"#FFFFFF"},
      {"kind":"text","name":"Label","width":144,"height":24,
       "text":"Send report","font_size":16,"line_height":24,"fill":"#FFFFFF"}
    ]
  }]
}
```

The occupied width is `16 + 20 + 12 + 144 + 16 = 208`; vertical text occupancy
is `12 + 24 + 12 = 48`. A width of 216 still fits these boxes; 200 must refuse.
The label still needs a rendered glyph/fit check. `component` means a native
editable component, not a promise of linked instances, variants or behaviors.

## Change and prove the result

For the saved example, the nested patch operation can change related properties
together while retaining the native nodes:

```json
{
  "kind": "patch",
  "edits": [
    {"path":"Button","properties":{"width":264,"gap":16}},
    {"path":"Button/Label","properties":{"text":"Sent İ","font_size":18}}
  ]
}
```

Reopen the actual saved destination, verify the component/text/layout structure
and stable native identities where an admitted inspection route is available,
then export `target_path: "Button"` from that changed file. Confirm the SVG's
dimensions and changed content and inspect its rendered pixels. Keep the
original and changed exports when assessing whether unintended regions changed.
For repeated elements, list exact intended paths once and group related edits
within the 64-edit bound; avoid broad searches or unrelated restyling.

Run requested longer-label and narrower-container cases on copies. A text-box
overflow may pass file validation, so use rendered output as its oracle. A
container-fit refusal should leave the source intact and publish no destination.
Report a missing font, stale source, absent path or unsupported feature instead
of bypassing validation or switching tools silently.

Upstream app/CLI installation, arbitrary `eval` or JSX `render`, XPath/lint,
PNG/PDF/code export, variables/formulas, linked instances, Hug/Fill, wrapping,
grid, rich text, embedded assets, vectors and live controls are outside this
route. Use separately admitted capabilities only when the task needs them;
no upstream tool name grants their availability.

## Method source

Reviewed upstream [skill](https://github.com/open-pencil/open-pencil/blob/24461adec7073312d460d76f08f069de18e4fb78/skills/open-pencil/SKILL.md)
sections “MCP workflow”, “Tool discovery”, “Key tools for agents” and “Tips”, and
[design-authoring](https://github.com/open-pencil/open-pencil/blob/24461adec7073312d460d76f08f069de18e4fb78/skills/open-pencil/references/design-authoring.md)
sections “Composition and layout”, “Variables and components” and “Verification”.
The useful methods are explicit target scope, container-owned placement, grouped
targeted edits and structure-plus-render checks under content/size pressure.
The examples and rules above are authored for Mirket's different typed contract.
The reviewed skill-local [MIT license](https://github.com/open-pencil/open-pencil/blob/24461adec7073312d460d76f08f069de18e4fb78/skills/open-pencil/LICENSE.txt)
names Danila Poyarkov (2026); no upstream prose, examples, fonts or runtime bytes
are reproduced here.
