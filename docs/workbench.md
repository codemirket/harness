# Make, inspect, repair

The workbench supplies repeatable artifact operations. The agent supplies design,
implementation and judgment. Use the project's real test suite and native host
capabilities when they are stronger; these commands fill missing local tooling.

## One-time runtime selection

```sh
python3 ai.py workbench doctor
python3 ai.py workbench configure --node /absolute/node --node-modules /absolute/node_modules --python /absolute/python3 --browser /absolute/chromium --soffice /absolute/soffice --pdftoppm /absolute/pdftoppm --pdfinfo /absolute/pdfinfo
```

Configure only tools you have. Nothing is downloaded. Paths stay in
`~/.agent-harness/toolchain.json`, outside Git; `HARNESS_TOOLCHAIN` changes that
location. `HARNESS_NODE`, `HARNESS_NODE_MODULES`, `HARNESS_PYTHON`,
`HARNESS_BROWSER`, `HARNESS_SOFFICE`, `HARNESS_PDFTOPPM` and `HARNESS_PDFINFO` override saved paths.
Node needs Playwright for browser work and sharp for vectors. Document demos need
python-docx, python-pptx and openpyxl; rendering needs LibreOffice and Poppler.
These are optional authoring tools, not dependencies of installation.

A successful doctor means imports/version probes worked. Run a scenario or render
on the actual host before claiming it can produce an artifact. Managed runtime
paths can change after a host upgrade. Reconfigure when they do.

## UI and animation

Run an existing project server, then define the actual user journey in a JSON
scenario. `browser --help` documents the bounded schema. The included example:

```sh
python3 -m http.server 4173 --bind 127.0.0.1 --directory examples/craft-lab
# In another terminal:
python3 ai.py workbench browser --scenario examples/craft-lab/scenario.json --output build/browser-review
```

Create `build` first. Every output directory must be new. The browser uses fresh
contexts, explicit viewport sizes and both normal/reduced motion. It exercises
controls, checks stated outcomes, saves PNGs, hashes and a report, and records
console/page/HTTP/transport failures separately. Initial URLs must be loopback;
external origins require an explicit scenario allowlist. This guard is not an OS
network sandbox. Use only a development app and actions within the task's scope.

Capture motion frames with `{"op":"capture","label":"opening","times_ms":[0,100,250]}`
immediately after the triggering action. Offsets start at capture, not at the
original input event; screenshot overhead is recorded and frames are samples,
not precise frame timing or a performance profile. For frame-perfect video use a
video renderer. Exercise rapid reversal and keyboard input in the actual UI.

Open desktop, mobile and transition PNGs. Check hierarchy, copy, alignment,
contrast and state continuity; use a screen reader and keyboard where relevant.
Automated assertions do not certify accessibility or visual quality. The runner
requires visual review even when its checks pass. No assertions means limited
evidence, never a completed QA pass. Overflow candidates need contextual review.

## Editable vector and illustration

```sh
python3 ai.py workbench vector examples/craft-lab/fieldwork.svg --output build/vector-review
```

This preserves the SVG source, inspects basic structure and renders 24/64/256/1024px
exports on light/dark backgrounds plus a contact sheet. Inspect individual large
exports as well: the sheet reduces large assets to fit. It rejects active content,
raster embeds and CSS imports; use presentation attributes for this narrow path.
It is not a general SVG sanitizer. Design for the actual size: the specimen is a
64px-and-larger illustration, not a legible 24px icon. Create a simpler silhouette
for icon usage instead of relying on resolution.

For raster illustration use the host's available image-generation tool or an
explicitly connected provider. Specify composition, subject, medium, lighting,
palette, intended placement and crop; inspect the result in that placement.
This harness does not supply image-generation credentials or a cross-client
image model. Keep the editable/vector and raster delivery contracts distinct.

## Office documents

```sh
python3 ai.py workbench documents demo --output build/office-demo
python3 ai.py workbench documents inspect /absolute/brief.docx --output build/doc-structure
python3 ai.py workbench documents render /absolute/brief.docx --output build/doc-render
```

The demo creates a fictional brief, deck and formula workbook, then renders and
recalculates with existing tools. Inspect actual page PNGs and reopened sources.
The inspector checks selected OOXML structure, formula caches and external/macro
flags. Bounds checks cannot see every text overflow. Rendering proves an export
path, not Microsoft Office or Google import fidelity. Use the native document,
spreadsheet or presentation skill when its available tools provide stronger QA.
See [office production](../skills/office-authoring/references/production.md).

## Markdown and programming

```sh
python3 ai.py workbench markdown /absolute/README.md --output build/markdown-review
```

Checks a limited ATX-heading/link/fence subset. External links and renderer
extensions require separate checking. Read the rendered document and run relevant
commands safely in the documented environment. Never run untrusted examples just
because they appear in a code block.

For programming, keep the actual project build, type checks, behavior tests and
release gates authoritative. Reproduce the failure, derive expected values from
requirements, fix the cause, and prove the known defect is caught. Browser checks
supplement those tests. Do not auto-skip failing tests or replace them with source
text assertions. A harness cannot invent project invariants from a generic list.

## Review that can reject the result

Give a fresh reviewer the brief, constraints, artifacts and relevant executable
checks. Ask them to perform the journey and inspect the output before reading the
author's quality claims. See the [review recipe](../skills/work-planning/references/artifact-review.md).
Classify findings as requirement defects, craft weaknesses, preferences or
unverified paths. Fix concrete defects, regenerate affected artifacts and review
again. Stop when requirements and named defects are resolved; more iterations
can make work worse. Keep user acceptance separate from automated checks.

## Use from an installed skill or plugin

The installed `skill-catalog/scripts/harness.py` forwards these commands to its
resolved checkout or the plugin's bundled snapshot:

```sh
python3 /absolute/installed/skill-catalog/scripts/harness.py workbench doctor
```

The [Codex exporter](plugins.md) carries the workbench with its foundation bundle.
Claude Desktop Code uses the same shared sources through direct installation.
Chat/Cowork handoff packages do not automatically gain a local shell/runtime.
