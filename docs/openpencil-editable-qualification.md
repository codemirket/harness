# Editable OpenPencil qualification — 2026-10-26

Status: the bounded editable-file subset passed local macOS ARM64 source,
private process and disposable installed-core MCP checks. This extends the
previous shape-only route; it does not establish the entire expanded design
milestone, native-editor interchange fidelity, Windows operation, Codex Desktop
GUI activation or the proposed minimum macOS version.

## Exact admitted closure

| Identity | Value |
| --- | --- |
| Platform observed | macOS 26.5.2 (25F84), ARM64 |
| Private Node | 24.22.1, explicitly selected absolute executable |
| Rust | 1.99.1 (58cf8f9c3) |
| OpenPencil | 0.16.0; upstream source commit `ffe2fc5798468988a5943c5f5737905c3c7a28b5` |
| File mode | `editable_file` |
| Selection | `packaging/admission/openpencil-editable.candidate.json` |
| Selection SHA-256 | `fd966b3140dfe08a47fbaa4481cebaecd4c1a79856b821edb2be2cf7b238fb0f` |
| Runtime JSON SHA-256 | `a86929a1380cd40bcfdf9a477d79b5a4f9f5fe489ad3d4a2f7e168b76328f0f9` |
| Runtime JSON size | 98,210 bytes |
| Payload | 596 files; 2,433,673 bytes; 16 packages; 21 notices |

All runtime files remain byte-identical to the existing qualified
pure-JavaScript closure. Only Mirket's typed adapter and source-selection
metadata changed. The new adapter writes native `SYMBOL`, `TEXT` and stack
layout fields using the pinned OpenPencil serialization modules. A bounded
fixed layout calculation handles the selected subset without Yoga. The
previous Yoga compiled-source/build gate remains unresolved, and Yoga,
CanvasKit and all WASM remain excluded. No upstream installer/helper, npm
resolution, user editor or user installation was invoked.

The [component contract](openpencil-candidate.md) records the exact supported
fields, imported-record allowlist, process protocol, bounds and rights closure.
Prior official npm tarball/source correspondence and installed notices are
retained; no broader binary rebuild or provenance-verification claim is added.

## Verified artifact and effects

The representative source is a genuine component with an ellipse icon and
plain text label, fixed width 200px/height 40px, horizontal layout, padding
12px horizontally/8px vertically and gap 8px. It is saved as `.fig`, parsed and
materialized through OpenPencil, patched to width 240px/gap 16px/text `Sent İ`
and font size 18px, saved again, reopened and exported to SVG.

Checks inspect native `SYMBOL` identity, `TEXT` content/font size, `HORIZONTAL`
stack fields and fixed sizing. The icon reopens at x=12/y=10 and the label
moves from x=40 to x=48/y=8. Native GUIDs of the component and both descendants
survive the patch. Original source bytes remain unchanged. A second native
edit to vertical layout reopens at icon x=90/y=8 and label x=40/y=40; start,
center, end and space-between primary alignment have independently calculated
position expectations. The SVG retains 240×40 bounds, x=48 translation,
editable `Sent İ` text and the Noto Sans family/18px font-size attributes.

A saved SVG was separately rendered at 960×160 through Mirket's existing
private resvg leaf and hash-verified admitted NotoSans.ttf. The PNG was
visually inspected: blue button, white circle, visible white `Sent İ` label.
This confirms that specimen under that selected renderer/font. OpenPencil's
SVG remains a family reference without embedded fonts, measured text or glyph
outlines; no general text shaping, glyph-coverage or pixel-equivalence claim
is made.

Refusal probes cover malformed/duplicate JSON, unpaired UTF-16 surrogate text,
unsupported fields/fonts/styles, grid/hug/baseline layout, text metrics,
nonfinite native values, mismatched saved layout positions, overflow,
position edits of layout children, cyclic hierarchy, missing targets,
CRC corruption and a DEFLATE expansion bomb. SVG text escaping is checked.
Stale revisions, existing stages and source links refuse before write. Added
CanvasKit/Yoga files, unlisted empty directories and notice tampering remain
refused even when the caller rewrites runtime.json and its claimed hash.

## Checks and evidence

- The mandatory build-input guard passed before Rust compilation.
- Locked TypeScript compilation passed; the adapter suite passed 89 tests,
  with its explicit real-browser test skipped because no browser was supplied.
- The OpenPencil assembly tests passed both drift/rollback checks.
- Four focused Rust tests passed, including the real private-process test
  selected with exact `MIRKET_TEST_OPENPENCIL_*` paths and runtime hash.
- `tests/openpencil-candidate-smoke.mjs` passed the real native saved-file and
  refusal checks above.
- `tests/openpencil-installed-smoke.mjs` assembled a disposable release and
  passed setup-core, doctor-core before/after, and actual installed MCP
  prepare/start/receipt/publication for component/text/layout create,
  patch and SVG export, plus stale-source refusal and source preservation.
  Final disposable release identity: `sha256:981da8d9ff3f11a2c3df4bee7bf74abdfec011f68e556904f005808bd9d35070`.

Private evidence is retained outside shipped source under
`~/.codex/task-state/harness-rewrite-implementation/`:

| Evidence | Location |
| --- | --- |
| Final leaf records and saved artifacts | `openpencil-editable-final-v3-20261026/verification.json` and sibling `.fig`/`.svg` files |
| Preview inspected | `openpencil-editable-final-v2-20261026/preview/result.png` |
| Final installed receipt | `openpencil-editable-installed-final-20261026.json` |
| Source/compiled adapter identity | `openpencil-editable-source-pins-20261026.json` |
| Assembled private runtime | `openpencil-editable-runtime-20261026/runtime.json` |

The v2 preview uses the same representative exported SVG as the final leaf
run; the final source change additionally refuses malformed surrogate text.
Temporary failed fixtures remain separate from successful evidence and are
not acceptance receipts. The installed fixture cleans its disposable home
and leaves the JSON receipt. Global setup/skill activation, actual Desktop
journeys, target OS floors and native Windows testing remain independent
release gates. Root integration owns the full Rust/Clippy and release checks.
