# Statistical restraint with uncertainty intact

Instance `ons-statistical` · data explanation · owner `design-inspiration`.

## Reference and transfer

Read ONS guidance on [small multiples](https://service-manual.ons.gov.uk/data-visualisation/chart-types/small-multiple-charts),
[annotations](https://service-manual.ons.gov.uk/data-visualisation/guidance/annotations)
and [uncertainty](https://service-manual.ons.gov.uk/data-visualisation/guidance/showing-uncertainty-in-charts).
The transferable language is stable encodings, comparable panels, restrained
annotation and a conclusion that retains material uncertainty. These selected
chart guides do not establish a complete ONS interface language or universal
rules for all statistical methods.

## Worked example: district journey estimates

Brief: communicate three close survey estimates without implying a confident
ranking. Constructed values are North 32% (26–38), East 30% (24–36), West 28%
(22–34), with illustrative 95% intervals. No real survey is implied.
Concept: compare estimates **and the limits of comparison** in the same view.
Grammar: quiet statistical bulletin, direct title, one blue encoding, common
0–50% axis, three comparable panels, visible interval marks and exact-value table.

Located decisions in `assets/design-languages/ons-statistical/index.html`:

- The headline and `.standfirst.good-only` carry uncertainty into the main claim.
- All three SVGs use the same viewBox and scale: 5 drawing units per percentage
  point. The point and range endpoints reconcile to the stated values.
- `.range` is an interval, not a decorative divider. Labels and SVG descriptions
  identify its meaning; the table keeps the numbers available without the chart.
- At narrow sizes panels stack with axes, units and interpretation intact.
- `.footnotes.good-only` avoids the common fake certainty that interval overlap
  alone is a formal significance test. Survey design and covariance matter.

Open `?variant=misleading`: interval marks vanish and the prominent narrative
asserts a league table. It still looks like a sober bulletin. Probe whether that
assertion follows from the available evidence. The retained table intentionally
lets a reviewer expose the misleading hierarchy rather than hiding all clues.

## Choose and adapt

Use for public statistics, research reports and estimates where uncertainty can
change a conclusion. Avoid chart ceremony for an exact operational count, or
using a statistical publication's visual authority to legitimize fabricated data.

Transfer exercise: compare four uncertain energy-use estimates. Decide whether
panels actually aid comparison; select intervals from the real analysis, preserve
units and denominator, and write a conclusion the intervals support. A beautiful
range mark does not validate the underlying statistical model.

## Source and rights

Selected service-manual pages inspected 2026-10-15. Their OGL v3 notice includes
exceptions; that notice is not a license to ONS identity, third-party material or
every linked dataset. Sources remain link-only. This lesson, charts and fictional
values are authored by Mirket under MIT, not copied ONS assets. The manifest keeps
the exact source links, capture identities and scope.

## Read, render, verify

Read `mirket catalog read design-inspiration --path references/design-languages/ons-statistical.md`.
MCP `skill_read` uses `skill_id: design-inspiration` and the same path; resource
`mirket://skills/design-inspiration/references/design-languages/ons-statistical.md`.
Open `assets/design-languages/ons-statistical/index.html` under the installed skill.
The data/research owner verifies inference and units; `graphic-design` verifies
encoding and intended-size readability; `interface-design` verifies reflow and
table/disclosure access. Compare interval endpoints, displayed values, narrow
panels and prominent claim in both variants. Rendering alone proves no inference.
