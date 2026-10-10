---
name: spreadsheet-analysis
description: Analyze, audit and build spreadsheet models with traceable inputs, preserved workbook features, verified formulas and reconciled business results. Use for Excel expert work, spreadsheet QA and analytical XLSX or CSV deliverables; choose the host's live Excel capability for an active workbook session.
---

# Spreadsheet analysis

Resolve the actual workbook, business question and requested deliverable before
changing cells. Preserve the distinction between formula source, cached values
and values recalculated by a compatible engine.

## Choose the capable route

Use an available native spreadsheet skill and its tools first. An active Excel
session or Excel app attachment requires the host's live Excel capability;
editing a similarly named file is not verification of that session. For file
work, use the native file workflow or an already available portable library and
calculation engine. A registered skill does not prove either runtime exists.

Respect the requested location and format. Inspect the existing workbook before
rebuilding it, and retain an original or recoverable version for consequential
edits. Use the available office workflow for authoring/rendering mechanics; this
skill owns the analytical model and acceptance checks. Do not install a runtime
or connect an account merely to activate this workflow.

## Inspect the model and its dependencies

- Identify sheets, tables, named ranges, formulas, types, hidden/filtered data and
  the grain of each input. Check join cardinality and duplicates before summing.
  Record units, currency, period, date convention and missing-value treatment.
- Inspect macros, external links/connections, pivots, charts, protections and
  other features touched by the edit. Select a route that preserves required
  features; a library flag or a successful save is not preservation evidence.
- Keep formulas as formulas and dates/numbers as their intended types. Reading
  cached values can support inspection, but saving that values-only view can
  destroy formulas. CSV cannot preserve workbook formulas, styling or features.
- Do not run macros, refresh external data or follow workbook links incidentally.
  Those effects require the user's relevant authorization and a capable route.

## Build and reconcile

Separate inputs, assumptions and derived outputs in a way that fits the workbook.
Use formulas for relationships that should respond to changed inputs. Define
signs, units and period boundaries; check copied ranges and absolute/relative
references. Do not replace missing inputs or errors with zero for cosmetic
cleanliness. Preserve an intentional blank and distinguish it from a true zero.

Check the business identities independently of the formulas: component totals,
opening-plus-movement balances, cohort counts, revenue/cash differences and
scenario assumptions as applicable. A workbook that calculates without errors
can still answer the wrong question. Test a material input change and a relevant
boundary such as zero volume, a refund or a period cutoff.

For stale results or a macro-enabled workbook, read the
[worked formula and preservation case](references/recalculation-evidence.md).

## Verify the saved deliverable

Recalculate with a compatible available engine when formula results are part of
the deliverable. Reopen the saved result and inspect formulas, calculated values,
errors and the features affected by the edit. Report the actual engine and any
unsupported formulas or unrefreshed links. Manual arithmetic or merely opening
an XLSX package is not proof that the workbook recalculated.

Inspect relevant rendered sheets/charts at usable size for labels, units,
clipping, misleading scales and number/date formatting. Check actual live state
when the task targeted Excel. Report analytical reconciliation, calculation,
file preservation and visual evidence separately. If an engine or feature
cannot be verified, deliver the useful work with that exact limitation rather
than claiming current cached totals or cross-application fidelity.
