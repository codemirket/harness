# Portable office production

Use one authoring route for each artifact. Prefer the host's native format skill
when its editor, renderer and storage tools are actually available. Otherwise use
independent libraries and the portable workbench. Do not copy proprietary host
helpers into the project or assume a helper mentioned in another host exists.

## Resolve the tools

The portable entry point is `scripts/workbench/documents.py` in the harness
checkout. Resolve the checkout before running it from an installed skill. All
commands require a new output directory with an existing parent.

| Surface | Requirement | Evidence |
| --- | --- | --- |
| `inspect` | Python 3.9+ standard library | ZIP/XML summaries, bounded package reads, source hash and reported findings. PDF inspection is only a header/raw-marker scan. |
| `render` | LibreOffice for Office inputs; Poppler `pdfinfo` and `pdftoppm` for all inputs | Actual PDF, a known page count, one PNG per page, input/output hashes and tool logs. |
| `demo` | python-docx, python-pptx, openpyxl plus render tools | Editable fictional brief, deck and workbook; two-page brief; deterministic budget checks; six rendered pages on the tested runtime. |

Set `HARNESS_SOFFICE`, `HARNESS_PDFINFO` and `HARNESS_PDFTOPPM` to executable paths
when PATH does not resolve them. The parent workbench can select `HARNESS_PYTHON`;
when invoking this script directly, launch it with that interpreter explicitly.
The helper does not switch interpreters or install packages. If a tool is
missing, state exactly which capability cannot run and use an available native
route or obtain the user's dependency-installation direction.

Codex Desktop can expose its workspace dependency loader and native artifact
tools. Resolve those at runtime. Claude Desktop Code needs its own available
interpreter and renderers, or deliberately configured existing paths. Shared
configuration must not hardcode a developer's Codex cache directory.

## Inspect, author and render

```sh
python3 scripts/workbench/documents.py inspect report.docx --output report-inspection
python3 scripts/workbench/documents.py render report.docx --output report-render
python3 scripts/workbench/documents.py render slides.pptx --output slides-render
python3 scripts/workbench/documents.py render model.xlsx --output model-render
python3 scripts/workbench/documents.py render report.pdf --output pdf-render
```

`inspect` never executes macros, follows external resource URLs, extracts ZIP
paths, or opens the file in Office. It flags active package parts and external
relationships without printing their target URLs. It can detect some direct,
unrotated slide shapes outside the slide, spreadsheet error cells and absent or
empty formula caches. It does not validate the complete OOXML schema, chart
semantics, grouped shapes, reading order, text overflow or calculation freshness.

`render` copies the input into temporary staging and uses an isolated LibreOffice
profile. Active/external package content reported as an error blocks office
conversion. Ordinary hyperlinks are reported separately and are not fetched by
the helper. Conversion is a local subprocess, not a security sandbox for arbitrary
malicious documents; use an appropriately isolated environment when that threat
is relevant.

Rendering has a 128 MiB source limit, bounded ZIP expansion, a maximum of 80 PDF
pages, subprocess timeouts and PNGs bounded to 1800 pixels on the longest side.
The report preserves diagnostics and verifies expected output files. An error
may leave a partial output directory for diagnosis; it never claims successful
review or overwrites a previous directory. Temporary conversion profiles are
cleaned up. Use a higher-resolution native preview for tiny details when needed.

For XLSX, `render` additionally writes a recalculated copy under `recalculated/`
and inspects it before PDF conversion. Keep the original editable file. Formula
errors in the calculated copy are failures; missing/empty caches remain warnings
because they can also represent empty-string results. No generic recalculation
check proves a model is correct. Independently assert expected totals, inspect
formula ranges and verify one representative input change.

## Authoring choices

**Word:** python-docx covers common paragraphs, styles, tables, headers and
footers. Use the supplied template where available. Check page breaks, table
headers/splits, orphan headings and captions. Deliberate OOXML edits or a native
editor may be necessary for unsupported revisions/fields. Diff meaningful text
and relationships after editing an existing file; do not recreate it from a
plain-text dump.

**Slides:** python-pptx is the initial Python path; PptxGenJS is an optional Node
alternative when its features justify another dependency. Keep claims supported,
headings readable and data editable. Render charts and dense slides at actual
size. A contact sheet helps assess consistency but cannot replace individual
slide inspection. Verify speaker notes and citations if requested.

**Sheets:** openpyxl writes formulas but does not evaluate them. Separate inputs,
formulas and outputs; name units and periods. Check zero/negative values,
boundary rows, dates and cross-sheet references. LibreOffice may differ from Excel
for modern functions, dynamic arrays, linked workbooks and unsupported objects.
Do not silently drop such features or claim Excel compatibility without a test.

## Visual review and handoff

Open every rendered page or slide with the host's image viewer. Review hierarchy,
spacing, contrast, page rhythm, chart labels, crop and font substitution. Inspect
full-size detail where the contact sheet is insufficient. Repair the source and
rerender after changes. Keep the final source hash associated with the reviewed
images so a later edit cannot inherit stale visual evidence.

`report.json` separates `machine_checks` from `visual_review`. Its visual status
is intentionally `not_performed`; the CLI cannot know whether an agent or person
looked at its images. In the task's final report, identify the inspected coverage
and any remaining concern. A successful machine report is never a substitute for
this review.

For a reproducible capability exercise:

```sh
python3 scripts/workbench/documents.py demo --output fieldwork-demo
```

The independently authored example uses fictional Fieldwork Studio figures:
20 discovery hours at $100, 30 design hours at $120 and 40 build hours at $150.
The base is $11,600; 10% contingency is $1,160; the total is $12,760. The script
asserts the recalculated cells and a two-page Word render. Review the actual
brief, three slides and workbook preview before using this as visual evidence.
It is a capability demonstration, not a benchmark proving improvement over a
previous workflow.
