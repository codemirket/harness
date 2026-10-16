# Select notation from the relationship

Instance `ft-relationship-notation` · analytical graphics · owner `design-inspiration`.

## Reference and transfer

Read the [FT Visual Vocabulary](https://github.com/Financial-Times/chart-doctor/blob/main/visual-vocabulary/README.md).
The useful instance is a relationship-to-notation teaching grammar. Deviation,
distribution and flow ask different questions before a chart type is chosen.
This is not a complete FT brand system; inaccessible current Origami guidance is
not treated as freshly inspected component documentation.

## Worked example: three questions, three encodings

Brief: teach an analyst to preserve the question and grain when choosing a chart.
Concept: shared editorial voice, different notation for different relationships.
Grammar: three numbered question/diagram pairs, common type and restrained color,
direct labels, explicit measures and an exact-value alternative.

Located decisions in `assets/design-languages/ft-relationship-notation/index.html`:

- `#deviation` uses a meaningful zero and signed lengths. A is −8, B +3 and C +11
  percentage points from target; these are not shares summing to a whole.
- `#distribution` uses equal 5-minute bins and counts of completed visits. The
  bars total 20; the axis is waiting duration, not calendar time.
- `#flow` preserves one 100 kg batch through two destinations, 70 kg and 30 kg.
  The outgoing amounts reconcile to the input. The simplified band geometry is
  an original construction, not an FT poster extraction.
- The common `.row` structure expresses editorial consistency without forcing
  all three questions into one chart silhouette. Text, units and table survive
  narrow reflow and no-script use.

Open `?variant=misleading`: deviation gets a share-of-whole silhouette; waiting
time bins get a connected line; transfer becomes isolated bars. The surface
remains coherent. Ask what the visual implies about sign, chronology and
conservation, and whether a reader can recover the intended relationship without
repairing the chart from its caption. A possible chart in one context can be a
poor encoding for this question; no type is universally banned.

## Choose and adapt

Use for choosing analytical forms, data explainers and chart review. Avoid treating
the vocabulary as an exhaustive recipe, or choosing a sophisticated diagram
before checking the observations, grain, units and audience's actual question.

Transfer exercise: one dataset contains monthly totals, transaction-level wait
times and journeys between stations. State the observation grain for each view;
choose encodings from the question. Test one attractive wrong alternative and
explain which inference it invites. Reconcile totals before styling flow widths.

## Source and rights

Selected repository README inspected 2026-10-15. The root README limits MIT to
software and excludes FT content. A GitHub MIT badge does not license the Visual
Vocabulary poster, article images, fonts or brand. Those remain link-only. This
lesson and all diagrams, typography and teaching values are original Mirket MIT
material. The manifest records actual source capture identity and rights limits.

## Read, render, verify

Read `mirket catalog read design-inspiration --path references/design-languages/ft-relationship-notation.md`.
MCP `skill_read`: `skill_id: design-inspiration`, that path; resource
`mirket://skills/design-inspiration/references/design-languages/ft-relationship-notation.md`.
Open `assets/design-languages/ft-relationship-notation/index.html` from the installed
skill. Data/research ownership checks grain and reconciliation; `graphic-design`
checks encodings and intended-size labels; `interface-design` checks access and
reflow. Compare actual arrows/bands, zero, bins and table in both variants.
