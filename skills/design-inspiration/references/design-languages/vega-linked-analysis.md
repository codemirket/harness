# Vega-Lite: one selection, linked analytical views

Instance `vega-linked-analysis` · data · owner `design-inspiration`.
The language name describes an interaction grammar interpreted by Mirket; this
original study does not bundle or execute Vega-Lite.

## Brief → concept → grammar

**Brief.** A field researcher must explore which constructed station visits fall
inside a travel-time range, compare observations by setting and tell the selected
population from the whole dataset. Twelve fictional records are supplied inline.

**Concept.** A visible selection is a shared data query. The points, linked sums,
selected records and population statement must describe the same records.

**Grammar.** Use a fixed-scale scatterplot for travel minutes versus observations,
a translucent interval for the selected time range and muted unselected context.
The companion bars show selected observation sums over faint all-setting totals
on one fixed 0–50 scale. Native fields and a setting filter expose the same query
without requiring a brush gesture. A clear action restores the complete dataset.

## Located choices in the original study

| Location | Choice and purpose |
| --- | --- |
| `#scatterplot`, `#brush`, `#points` | Original SVG retains every point’s position on fixed scales. The horizontal brush selects time only; points outside the complete query become muted, not removed from context. |
| `#range-from`, `#range-to`, `#habitat`, `#filters` | Labeled numeric fields and native select supply equivalent keyboard input. Reversed bounds produce an actionable native validation error. No hidden interval or setting remains after reset. |
| `#selection-summary`, `#station-total`, `#observation-total` | `data-selected-count` is the selected-record count. `data-denominator="12"` always identifies the whole constructed dataset. Observation sums use count units; they are not station percentages. |
| `#river-bar`, `#park-bar`, `#river-value`, `#park-value` | Green lengths and `data-value` reconcile to selected rows. Background totals remain River 40 and Park 43; the common scale never renormalizes to the current selection. |
| `#table-help`, `#station-table` and its stable `tr[data-id]` rows | A continuation cue stays outside the scrolling boundary. Exact selected records expose identity, setting, travel minutes and observation counts; the complete static table survives without script. |
| `#linked-empty`, `#reset` | A valid empty interval produces zero selected records, zero green sums and a real way to widen or clear. Empty selection is distinct from missing data. Pointer cancellation restores the pre-drag query. |

**Fit:** coordinated exploration of a bounded dataset, analytical comparison,
brush/filter relationships and explicit selected-versus-total populations.
**Poor fit:** inferring causation from these constructed points, aggregate queries
with unknown denominators, very large datasets needing a real rendering/data
engine, or an ornamental dashboard whose panels do not answer one question.

## Contrast and failure probe

Open `?variant=misleading`. The scatterplot and selected table update normally,
but both green bars retain all-setting sums under the selected-stations title.
The polished chart now tells a different population’s story.

Set 0–20 minutes with all settings. The actual selection is `a`, `b`, `c`: **3 of
12 stations**, **12 observations**, River **7 / 40**, Park **5 / 43**. Reconcile the
three table rows, point states, summary, numeric values and `data-value` on each
bar. The misleading route still shows 40 and 43 in green, failing that invariant.
Then set 0–5: the purposeful route shows zero bars and the visible empty state.
Clear selections: 12 records and total 83 observations return. Apply the River
filter within 0–20 and verify 2 of 12 records, River 7 and Park 0. The denominator
must not quietly become 2 just because a filter is active.

## Comparison at use size

This lesson addresses shared-query identity and selected-versus-total populations. Choose the focused original [comparison-at-use-size practice](comparison-at-use-size.md) instead when the open problem is separately wrapping headings that displace intended partner graphics, or required columns continuing beyond a clipping boundary without a visible cue. It can be read directly without this card. Long labels or paired graphics alone do not require another lesson; shared-heading/stacked layouts and vertical records may already preserve the relationship. Add the practice only if its distinct placement problem remains unresolved alongside the query problem and the host/project read limits permit it.

## Materially different transfer task

Build a maintenance backlog explorer using age in days versus estimated work
hours. A brush selects an age interval; a separate priority filter and a linked
team-hours view consume the same query. Use different data, layout and encodings,
state whether cancelled jobs belong in the denominator and preserve an explicit
empty result. Accept only when someone can reconstruct every displayed total
from the selected job records. Changing these station names is insufficient.

## Source, edition, inspected relationship and rights

Read current [selection parameters](https://vega.github.io/vega-lite/docs/selection.html),
[rectangular brush](https://vega.github.io/vega-lite/examples/interactive_brush.html)
and [interactive dashboard with cross highlight](https://vega.github.io/vega-lite/examples/interactive_concat_layer.html),
checked **2026-10-15**. The examples declare the **Vega-Lite v6** schema; this is a
live documentation edition, not an immutable source pin. The rendered dashboard
was visually inspected in Chrome: category bars coordinate with selected marks
on a contextual matrix, with separately labeled total and selected counts. The
transfer is shared-query identity and explicit populations, not its palette,
movie data, binned matrix or upstream specification.

Sources remain link-only (`license_url: null` for the inspected documentation
media; no blanket reuse right was established). A software license must not be
assumed to cover every example dataset or brand asset. Exclude Vega marks,
upstream data, specification code, fonts and screenshots. This page uses original
inline SVG, CSS, JavaScript and constructed data under the library’s MIT license,
system fonts and no runtime dependency or network assets.

## Read, render and verify

- Card resource: `mirket://skills/design-inspiration/references/design-languages/vega-linked-analysis.md`.
- CLI: `mirket catalog read design-inspiration --path references/design-languages/vega-linked-analysis.md`.
- MCP: `skill_read` with `{"skill_id":"design-inspiration","path":"references/design-languages/vega-linked-analysis.md"}`.
- Executable source resource: `mirket://skills/design-inspiration/assets/design-languages/vega-linked-analysis/index.html`. Read with `skill_read` using `{"skill_id":"design-inspiration","path":"assets/design-languages/vega-linked-analysis/index.html"}`; text delivery does not render charts.
- Render the installed skill’s `assets/design-languages` locally. In this checkout: `python3 -m http.server 8847 --bind 127.0.0.1 --directory skills/design-inspiration/assets/design-languages`, then open `http://127.0.0.1:8847/vega-linked-analysis/` and its contrast route.
- Build owner: `frontend-engineering` for shared query state and input; the project’s data/research owner validates units and inference, and `interface-design` inspects comparison and reflow.

Chrome on macOS exercised pointer brushing, equivalent field selection, setting
filtering, reset, empty results, exact linked sums and the contrast failure.
Narrow view and doubled text retain page reflow with deliberately scrollable
fixed-scale chart/table regions. Static content includes all 12 records. No
Vega runtime, touch device, screen reader, production query service or statistical
inference was verified. Rendered evidence and automated checks are not human
acceptance or a claim of universal analytical coverage.
