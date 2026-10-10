# Produce editable office artifacts

Use the host's actual document, presentation or spreadsheet workflow when
available. Identify the authoring library, renderer and calculation engine before
choosing an implementation. Native tools may supply these together. Otherwise
use explicitly approved local executables registered through `mirket tool`.
A skill does not provide a renderer, fonts, account access or permission to install
packages. Inspect untrusted input before opening macros or active content.

## Preserve the source model

Inspect file structure, template, styles, relationships, comments, formulas and
required editability before changing the document. Extracting text is not a safe
round trip for all Word or PowerPoint features. Preserve unsupported objects or
report their limitation; do not silently flatten them.

Write semantic headings and actual tables. Use coherent typography, spacing,
color and alignment. Keep slides focused on one idea, charts tied to source
numbers, and input cells separate from formulas. Label hypothetical figures.

## Render the saved artifact

Render the final editable file using the available native engine or approved
renderer. Review every page or slide at presentation size and inspect detailed
areas. Check clipped text, table splits, glyphs, chart labels, crop, contrast,
legibility and overlaps. Shape bounds do not detect every text overflow.
Repair the generator or source, regenerate the file and reinspect changed pages.

A LibreOffice rendering is evidence for that engine; it does not prove identical
Microsoft Office behavior. Rendering a file proves visibility, not visual quality
or human approval. Deliver the editable original and the requested preview,
keeping diagnostic files out of the user's artifact folder.

## Recalculate spreadsheets

A workbook writer may preserve formulas without evaluating them. Use an actual
calculation engine when calculated results matter, and compare both formulas
and cached values in the recalculated copy. Check known totals independently and
change a representative input to observe dependency behavior.

Verify numeric/date types, date systems, units, signs, row grain and error cells.
Assess external links, volatile functions, pivots, macros and advanced formulas
against the engine that actually ran. If recalculation or native behavior cannot
be tested, identify the specific uncovered result rather than asserting success.

## Retain meaningful evidence

For substantial tracked work, attach the editable artifact, rendered review
output and calculation check to the Mirket task using project-relative paths.
Acceptance and the failure probe should describe actual observed outcomes.
Hashes detect stale artifacts; they do not validate the argument, calculations
or reviewer identity. Apply the audience's editorial and visual criteria too.
