---
name: office-authoring
description: Create or revise Word documents, spreadsheets, and slide decks with structured content, editable source, calculation checks, and rendered verification. Use an available native format skill when it provides a stronger implementation workflow.
---

# Office authoring

Identify the deliverable, audience, format, source material, template, and required
editable elements. Inspect existing files before replacing their structure. Use
the host's native document tools when available; otherwise inspect installed
libraries and renderers before choosing an implementation path. Do not install a
new production dependency merely to follow an example.

## Word and structured documents

Use semantic headings, paragraphs, lists, tables, captions, and section breaks.
Preserve styles, comments, tracked revisions, headers/footers, fields, bookmarks,
and relationships that the requested edit does not change. Specify the revision
state when comparing or extracting text. A plain-text round trip loses structure.

For generated DOCX, a library such as python-docx can create common elements;
unsupported revision/field features need a capable native editor or deliberate
OOXML editing. Reopen the result and verify relationships and document contents.
Render using an available office application or compatible renderer. Check page
breaks, orphan headings, table splits, numbering, fonts, and clipping at actual
page size. Rendering compatibility is not guaranteed across office applications.

## Spreadsheets and models

Separate inputs, calculations, assumptions, and outputs. Preserve numeric types,
units, currencies, dates/timezones, locale, missing values, and identifier strings.
Use formulas for derived values when the workbook must remain editable. Make
references and ranges deliberate; check formula propagation and boundary rows.

A file-writing library such as openpyxl does not calculate formulas. Recalculate
with an available spreadsheet engine before asserting computed results; if none
is available, independently check consequential calculations and state that the
workbook's cached formula values remain unverified. Check error cells, totals,
cross-sheet references, named ranges, filters, charts, and print areas. Preserve
macros only with a format/tool that supports them; never execute embedded macros
merely to inspect a workbook.

## Presentations

Outline the argument and evidence before styling. Give each slide a clear claim
or purpose, with content density suited to the audience and delivery context.
Use the existing theme, grid, typography and master layouts. Prefer editable text,
shapes, tables and charts where future editing matters; source every material fact
and preserve attribution for external assets.

A library such as python-pptx or PptxGenJS can create editable decks; inspect local
availability and version before relying on its APIs. Render every slide through
an available compatible engine. Check overlap, text overflow, contrast, cropping,
axis labels, reading order and font substitution. Check speaker notes and links
when part of the deliverable. Never claim visual verification from source alone.

## Verify and hand off

Finish source edits before final export. Reopen the delivered file, compare
content and consequential figures against sources, and inspect the rendered
result. Export a PDF preview when useful without replacing the editable artifact.
Report which formats/renderers were checked and any unsupported feature. A valid
ZIP/XML package establishes file structure, not correct layout or calculations.
