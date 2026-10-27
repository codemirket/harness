---
name: workbook-authoring
description: "Create or revise an editable workbook with formulas, tables or charts requiring recalculation and saved-file verification. Skip one-off arithmetic outside a workbook."
---

# Workbook authoring

Use the available Codex host spreadsheet tools and inspect the existing workbook before editing. Preserve sheets, formulas, names, formats, hidden/linked content and source identity relevant to the task. Mirket does not bundle an Office runtime.

Separate input assumptions, calculated cells and presentation. Keep units, date/number formats, ranges and error behavior explicit. Prefer formulas for values users should be able to change; do not replace live formulas with cached numbers merely to make an export look correct.

Recalculate, save and reopen the actual deliverable. Inspect every affected sheet/render for truncation, number display, table/chart ranges, print layout and error values. Verify material totals and a changed-input case against an independent calculation. State external-link, macro or unsupported-function limits.

Return the editable workbook, source/assumption links and recalculation/visual evidence. Stop when requested behavior and saved outputs are verified, or report the exact unavailable formula/host path. A formatted grid or cached result alone does not establish a working workbook.
