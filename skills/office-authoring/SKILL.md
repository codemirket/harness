---
name: office-authoring
description: Create and revise editable Word documents, spreadsheets and slide decks; render the saved artifacts, inspect every page and verify calculations. Use a capable native format skill when available, with the portable document workbench as a cross-client fallback.
---

# Office authoring

Deliver the requested editable file with evidence that its content, calculations
and rendered layout were checked. Inspect supplied files and templates before
changing their structure. Determine the audience, decision, source facts and
required editable elements from the request; ask only for consequential missing
information.

Use the host's native document, presentation or spreadsheet workflow when its
actual tools are available. Otherwise use the independent portable workbench.
A registered skill is not an installed authoring library, renderer or calculation
engine. Read [production workflow](references/production.md) for executable
commands, tool configuration, design review and compatibility limits.

## Make the content work

Lead a brief with its decision and evidence; give a slide one clear purpose.
Use semantic headings and a coherent type, spacing and color system. Prefer
editable text, tables and charts. Preserve source meaning, units, uncertainty,
source attribution and the user's template. Label fictional examples explicitly.

For Word edits, preserve unrelated styles, comments, revisions, fields and
relationships. Plain-text extraction is not a faithful round trip. For slides,
review the argument as a sequence before decorating individual pages. For
spreadsheets, distinguish input cells from derived formulas, preserve numeric
and date types, and test changed-input behavior as well as known totals.

## Use the executable checks

From the harness checkout, with a verified Python interpreter:

```sh
python3 scripts/workbench/documents.py inspect input.docx --output new-inspection
python3 scripts/workbench/documents.py render input.docx --output new-render
python3 scripts/workbench/documents.py demo --output new-demo
```

Replace `python3` with the configured interpreter when libraries live in another
environment. `inspect` uses the standard library. `render` requires configured
LibreOffice/Poppler tools; `demo` also requires python-docx, python-pptx and
openpyxl. The helper never installs dependencies. Use a new output directory;
it refuses replacement. Inspect before rendering untrusted office files.

## Review the actual deliverable

Finish source edits, render the saved file and inspect every page or slide.
Check hierarchy, legibility, table splits, clipping, overlap, glyphs, chart
labels, crop and contrast. Automated shape bounds do not detect text overflow.
Repair the generator or editable source and rerender affected artifacts.

An XLSX writer does not calculate formulas. Use a calculation engine, retain the
original, then compare formulas and cached results in the recalculated copy.
Check independently known totals and boundary cases. A green calculation status
does not prove the formulas implement the intended business logic.

Report machine checks, visual review and unresolved compatibility separately.
The workbench deliberately reports visual review as not performed: producing
PNGs cannot establish that someone inspected them. Never claim a LibreOffice
render proves identical Microsoft Office behavior. Deliver the requested files;
keep inspection outputs as supporting evidence unless the user wants previews.
