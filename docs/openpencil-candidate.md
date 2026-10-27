# OpenPencil editable-file component

The macOS ARM64 component creates, reopens, patches and exports a bounded
editable native `.fig` subset through pinned OpenPencil 0.16.0 modules. It
supports real components (native `SYMBOL` records), fixed single-line text
boxes and fixed-size horizontal/vertical layouts. The existing design job
owner controls source revisions, cancellation, staging, publication and recovery.
The mode is `editable_file`.

Admission is recorded in
`packaging/admission/m4-openpencil-macos.runtime.json`; the exact retained
source selection is `packaging/admission/openpencil-editable.candidate.json`.
See [qualification](openpencil-editable-qualification.md) for evidence and limits.
The component adds no runtime package. CanvasKit, Yoga, WASM, font fetching and
font parsers remain physically excluded.

## Operations and editable fields

| Operation | Private input | Private stage | Operation fields |
| --- | --- | --- | --- |
| `openpencil.create` | `source.json` | `result.fig` | none |
| `openpencil.patch` | `source.fig` | `result.fig` | `edits` |
| `openpencil.export_svg` | `source.fig` | `result.svg` | `target_path` |

Create accepts UTF-8 JSON. All nodes need unique bounded sibling names and
explicit width/height. Frames and components can contain children; rectangles,
ellipses and text are leaves. This example produces an editable button:

```json
{
  "page": "Components",
  "nodes": [{
    "kind": "component", "name": "Button", "width": 200, "height": 40,
    "fill": "#1463ff", "layout": "horizontal", "gap": 8,
    "padding_top": 8, "padding_right": 12, "padding_bottom": 8, "padding_left": 12,
    "counter_align": "center",
    "children": [
      {"kind": "ellipse", "name": "Icon", "width": 20, "height": 20, "fill": "#ffffff"},
      {"kind": "text", "name": "Label", "width": 120, "height": 24,
       "text": "Send İ", "font_size": 16, "line_height": 24, "fill": "#ffffff"}
    ]
  }]
}
```

Patch edits select an existing unique path and change typed properties:

```json
{"edits":[
  {"path":"Button","properties":{"width":240,"gap":16}},
  {"path":"Button/Label","properties":{"text":"Sent İ","font_size":18}}
]}
```

| Properties | Supported contract |
| --- | --- |
| `x`, `y`, `width`, `height`, `fill`, `opacity`, `corner_radius` | Fixed geometry/solid fill; ellipses and text refuse corner radius. |
| `text`, `font_size`, `line_height` | Text nodes only. Nonempty single-line text, fixed box, regular Noto Sans family, left/top alignment. Defaults: 16px font, 24px line height. |
| `layout` | Frames/components only: `none`, `horizontal`, `vertical`; fixed dimensions on both axes. |
| `gap`, `padding_top`, `padding_right`, `padding_bottom`, `padding_left` | Nonnegative spacing, defaults zero; require horizontal or vertical layout. |
| `primary_align` | `start` (default), `center`, `end`, `space_between`. |
| `counter_align` | `start` (default), `center`, `end`. |

Layout is an exact fixed-size, no-wrap subset calculated by the adapter and
stored as native stack fields. Children keep their supplied dimensions; the
adapter does no wrapping, shrinking, stretching, text measurement or shaping.
Children, gaps and padding must fit their container. Layout children omit
`x` and `y`; patching their position is refused. Reopened native geometry must
agree with supported layout metadata. Patches preserve native record GUIDs,
including component descendants whose imported graph lacks a direct source ID.

SVG export selects an explicit path such as `Button`. It uses OpenPencil's
pinned SVG serializer and preserves editable text as SVG text. `Noto Sans` is
a referenced font family: the SVG itself embeds no font or glyph outlines and
this route makes no glyph-coverage or pixel-equivalence claim. A checked raster
preview can use the separately admitted private design font/renderer. The
native archive carries a placeholder thumbnail.

## Bounds and refusals

Input is limited to 1 MiB, output to 16 MiB, scene nodes to 256, hierarchy to
16 levels and patches to 64 edits. Dimensions are 1–2048 pixels, position is
−2048–2048, opacity is 0–1, corner radius is 0–512, spacing is 0–512, font size
is 1–256 and line height is 1–512. All numbers must be finite. Colors use six-
or eight-digit hexadecimal values. Text is 1–4096 well-formed UTF-16 units without controls
or explicit bidi control characters.

Unknown fields, duplicate JSON keys, images, embedded glyphs/resources,
variables, instances/component overrides, published/library components,
rich text, arbitrary fonts, auto text sizing, grid, wrap and hug/stretch sizing
are refused before staging. Imports must use the exact pinned canonical Kiwi
schema in bounded version-101 DEFLATE archives, one sRGB design page,
translations and only supported native records. Matching ZIP local/central
records, CRCs and a 4 MiB expansion cap are checked before upstream parsing.
This is a qualified subset, with no general Figma/editor interchange claim.

## Runtime and process contract

Source and stage are distinct direct siblings inside a canonical private job
directory. The leaf never publishes a user destination: it creates a private
stage exclusively with mode 0600 and reports its hash. The design
ArtifactPlan/JobState/OperationStore owner performs publication and reconciliation.

Launch the admitted private Node with empty `NODE_OPTIONS`/`NODE_PATH`:

```text
PRIVATE_NODE --max-old-space-size=192 /absolute/openpencil-main.js --release-id RELEASE --runtime-root /absolute/private-runtime --runtime-sha256 RUNTIME_JSON_SHA256
```

The verified leaf emits a `ready` frame with protocol version 1, exact release
identity, the three operation names, max frame/input/output sizes
65536/1048576/16777216, deadline range 100–60000ms, engine version `0.16.0`
and mode `editable_file`. Send one UTF-8 JSON line and EOF within five seconds:

```json
{"type":"run","protocol_version":1,"release_id":"RELEASE","job_id":"JOB","job_root":"/absolute/private-job","input_path":"/absolute/private-job/source.json","source_revision":"sha256:SOURCE_SHA256","output_path":"/absolute/private-job/result.fig","operation":"openpencil.create","deadline_ms":30000}
```

Patch additionally requires `edits`; export requires `target_path`. Other
request fields are refused. Result frames report identity, operation,
`status=staged`, paths, byte counts, source/output hashes, format, node count,
engine version and mode. Error frames report identity, code/message and
`output_may_exist`; errors exit nonzero. The latter becomes true as soon as
exclusive stage creation succeeds, including an interrupted write. Rust owns
wall deadlines, bounded stdout/stderr, exact frame validation, kill/reap,
source/destination revalidation and uncertain-stage reconciliation. These
controls do not create an OS sandbox.

## Exact source and notice closure

The 596 retained files (559 modules, 16 package metadata files, 21 notices;
2,433,673 bytes) remain identical to the prior qualified pure-JavaScript
closure. Compiled adapter pins independently bind every file; runtime.json
cannot authorize drift, additions, links or unlisted directories. The exact
SVG exporter entry point avoids the broader SVG import/font barrel.

The selection records official npm URLs, tarball hashes and registry-integrity
matches. All 794 npm-shipped TypeScript sources from the four OpenPencil
packages matched upstream Git blobs at
`ffe2fc5798468988a5943c5f5737905c3c7a28b5`. JavaScript distributions were not
independently rebuilt, and full DSSE/Sigstore verification was not performed.
The expanded typed operations change Mirket source only; they do not admit the
previously unqualified Yoga or CanvasKit native payloads.

Retained packages are @iconify/utils 3.1.7, @noble/hashes 2.4.0, the four
OpenPencil packages 0.16.0, @xmldom/xmldom 0.9.12, culori 4.0.2, es-toolkit
1.53.0, fflate 0.8.3, fzstd 0.1.1, js-base64 3.9.4, nanoevents 9.1.0,
postcss-value-parser 4.2.0, svgpath 2.6.0 and valibot 1.5.0. All are MIT
except js-base64 (BSD-3-Clause); retained supplemental notices include Kiwi's
adapted runtime attribution, culori's additional license and es-toolkit's
Lodash/Underscore notice. In-source notices remain byte-identical.

## Reproduction

Assembly reads reviewed existing source files; it installs nothing and runs no
upstream helper:

```text
python3 scripts/openpencil-assemble.py --source-root /absolute/reviewed-source-root --output /absolute/absent-private-runtime
PRIVATE_NODE adapters/node_modules/typescript/bin/tsc -p adapters/tsconfig.json
PRIVATE_NODE tests/openpencil-candidate-smoke.mjs --node PRIVATE_NODE --adapter /absolute/adapters/dist/openpencil-main.js --runtime /absolute/private-runtime --runtime-sha256 RUNTIME_JSON_SHA256 --evidence /absolute/absent-evidence-directory
```

The installed smoke additionally assembles a disposable core release, runs
setup-core/doctor-core and performs real MCP create/patch/export with the
representative component and text edit. Native editor fidelity, Windows and
actual Codex Desktop GUI activation remain separate gates.
