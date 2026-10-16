# Carbon: visible technical operations

**Language:** Dense technical operations. This is a curator interpretation of a coherent reference instance, not a claim that the upstream uses this exact category name. Study this as an alternative direction; do not blend whole systems indiscriminately.

**Use when:** An operator must find, compare and act on many structured records while preserving filter and selection scope.

**Poor fit:** A single simple decision or a brand narrative needs more explanation than simultaneous record comparison.

## Brief → concept

A fictional workshop operator needs to find two units requiring review among five equipment records and add diagnostic notes to a local queue. A note does not change readiness, schedule maintenance or contact a team.

**Concept:** The table is the work surface. Make the scope of a filter and the scope of a batch action visible before an operator acts. Density comes from alignment and stable roles, not from miniature controls.

## Source evidence → transferable grammar

**Observed source and coverage:** Official data-table guidance names search/filter utilities, sorting, expansion and batch operations; filtering distinguishes selection models. Fresh 1280×900 inspection showed the current guideline shell and surrounding table guidance. Its embedded live-demo region was blank in the captured state, so that capture is not a fresh demo-interaction result. Earlier official-source reading supports the toolbar/scope interpretation.

**Interpretation to transfer:**

- Aligned fields support comparisons across records.
- A stable toolbar keeps filter controls before their results.
- Applied filter text and result count describe the current scope.
- Selection count belongs beside the action that uses it.
- Urgency receives a redundant textual signal; color is supplementary.
- Long identifiers and names wrap; true tables may scroll within a labelled region.

These relationships inform the original specimen below. They do not establish task success, accessibility conformance or a right to copy the reference’s identity.

## Located decisions in the original specimen

| Location in `index.html` | Decision | Why it serves the brief |
| --- | --- | --- |
| `.toolbar`, `#applied`, `#result-count` | Explicit Apply commits the draft search/status values; text records the applied query and count. | Typed input is not silently presented as an already-applied filter. |
| `.status.review`, `#rows` | Two “Needs review” rows lead the default view; text and a square marker distinguish their status. | The work needing attention is visible without interpreting a color alone. |
| `#selection`, `#selection-count`, `#queue` | Only visible checked rows are acted on. Any filter change clears selection, with a status message. | An operation cannot silently include hidden selected records. |
| `.table-wrap`, `.unit-name` | A labelled focusable table region confines horizontal scroll, while equipment names wrap within cells. | A long label cannot widen the entire document or detach the unit ID from its record. |

The identity, all copy, CSS, JavaScript and inline artwork are original teaching material in this repository. The content is visibly fictional. The page loads no network assets, fonts, libraries or data. Native disclosures preserve their meaning with JavaScript disabled; enhanced controls are hidden without JavaScript and the complete default facts remain readable.

## Attractive wrong variant

Open the same HTML with `?variant=misleading`. The misleading query gives the first screen to reassuring true totals, places the three ready units first, reduces review emphasis and moves the toolbar and applied state below the table. The executive overview is attractive. It fails an operator brief by subordinating urgent records and the operation scope. The numbers remain true; their hierarchy serves a different job.

Judge the variant against the brief at the same width and with the same facts. Do not score it by polish alone. The explanatory teaching aside is outside `main`; the actual specimen lives in `main` in both variants.

## Failure probe

Filter to Needs review, check one unit, then apply a search that returns no records. Selection must clear and an explicit empty state must appear. Clear must restore five records. Queueing two visible selected units must add exactly two local notes and leave their readiness unchanged. Inspect the long particulate-monitoring label at 390px.

At narrow width also inspect the visible focus ring, long labels, enlarged text and the meaningful order of content. No critical interaction is hover-only; no animation is needed to communicate state.

## Materially different transfer task

Design a volunteer shift-assignment view with eligibility, time-window and selected-person constraints. Keep filter scope and action scope visible, but derive whether hidden selections may persist from the actual assignment contract. Supply the normal, no-match and mixed-eligibility cases. Do not transplant maintenance statuses or assume every domain should clear selections.

**Reject the transfer when:** Reject a transfer that says “Assign selected” without naming its actual scope, or that reports successful assignments for people excluded by eligibility.

## Exact owner, read and render route

Craft owner: `interface-design`. Embedded package owner: `design-inspiration`. Exact instance ID: `carbon-operations`.

```sh
mirket catalog read design-inspiration --path references/design-languages/carbon-operations.md
mirket catalog read design-inspiration --path assets/design-languages/carbon-operations/index.html
```

MCP reads use `skill_read` with `skill_id: "design-inspiration"` and the exact `path` above. The card resource is `mirket://skills/design-inspiration/references/design-languages/carbon-operations.md`; the HTML resource is `mirket://skills/design-inspiration/assets/design-languages/carbon-operations/index.html`. A source read does not render the page.

Render the saved `assets/design-languages/carbon-operations/index.html` from the installed `design-inspiration` directory in a browser. Project installations ordinarily place it under `.agents/skills/design-inspiration/` for Codex or `.claude/skills/design-inspiration/` for Claude; use the actual installation route returned by Mirket. A loopback static server or a direct local-file browser open is enough: no build step or dependency install is required. Compare the base URL and the same URL with `?variant=misleading`.

## Source edition, rights and exclusions

- [Carbon Design System: Data table guidelines](https://www.carbondesignsystem.com/building-blocks/core/components/data-table/guidelines) — Current redirected documentation; page marked updated 2026-09-30, inspected 2026-10-15. Link-only documentation/reference. Repository software is Apache-2.0; this does not clear all website media, IBM branding or fonts. No Carbon code is copied. [Rights evidence](https://raw.githubusercontent.com/carbon-design-system/carbon/main/LICENSE).
- [Carbon Design System: Filtering](https://www.carbondesignsystem.com/building-blocks/core/patterns/filtering) — Current redirected documentation; source evidence marked updated 2026-08-11. Link-only reference. Software license and website/media rights remain separate. [Rights evidence](https://raw.githubusercontent.com/carbon-design-system/carbon/main/LICENSE).

All upstream sources are **link-only** here. Their software or font license scopes remain separate from branding and website/media rights. No upstream file is redistributed by this lesson, no protected mark is imitated, and no upstream package is installed or executed. Source edition and checked date describe the inspected evidence, not a promise that a live URL will remain unchanged.

## Verification scope

Inspected on 2026-10-15 in local headless Chrome 155 on macOS at 1280×900 and 390×900: purposeful and misleading variants, visible keyboard focus and document geometry. The 390px checks also covered JavaScript-off reading and actual 200% text sizing. All seven interface specimens made zero HTTP(S) requests in this local run. The located local interactions were exercised separately: correction and review recovery; filter, empty and batch scope; native navigation/disclosures; reconciled cost and invalid input; full article reading; and the clipboard-denied fallback as applicable.

The table and code block intentionally use labelled local horizontal scroll regions at narrow widths. Source-reference capture limits are stated above. No screen-reader session, physical-device test, successful system clipboard write, human visual acceptance, native-client activation or measured usability gain is claimed.
