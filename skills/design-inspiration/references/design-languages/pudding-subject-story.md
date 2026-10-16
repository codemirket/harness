# Subject generates the explanation

Instance `pudding-subject-story` · narrative explanation · owner `design-inspiration`.

## Reference and transfer

Study [The Pudding's 2025 onion essay](https://pudding.cool/2025/08/onions/):
the subject shapes headline lettering, cross-section diagrams, local controls and
the reading sequence. The inspected opening and vertical-cut section show this
relationship; source markup exposes explode switches and parameter controls.
That evidence does not establish the article's mathematics, keyboard behavior or
responsive quality. This is one editorial instance, not a universal Pudding system.

Transfer **one subject generating several communication forms**, with each form
explaining the same mechanism. Do not transplant onion lettering, photographs,
prose or the entire scrollytelling structure into an operational interface.

## Worked example: one sheet, many layers

Brief: teach how repeated idealized folds change footprint and layer count.
Concept: the whole sheet remains; one quantity grows while the other shrinks.
Grammar: paper-colored field, crease crossing the title, original square outline,
green folded area, local fold control, then an invariant explained in prose.

Located decisions in `assets/design-languages/pudding-subject-story/index.html`:

- `.title-fold` introduces the material relationship before the first paragraph.
- `.experiment` pairs `#folded-sheet` with `#fold-count`. `draw()` calculates
  `layers = 2 ** folds` and the corresponding footprint from the same input.
- The original square remains in `.subject-form`, so shrinking area has a visible
  denominator. Layer count alone cannot explain the tradeoff.
- `#relationship`, SVG title/description and the native value list preserve the
  model beyond the drawing. No scroll gesture is required to read the explanation.
- The model is explicitly idealized: it does not predict paper thickness limits.

Open `?variant=misleading`: the diagram becomes an attractive abstract emblem.
The same numbers still change, but the subject-to-form relationship has disappeared.
Ask a reader to locate 25% of the original footprint at two folds. A polished
surface and working slider must not pass that probe by themselves.

## Choose and adapt

Use for mechanism explanations, learning and visual essays where readers benefit
from exploration. Avoid imposing an essay on frequent, time-sensitive operations.
Start from the current subject and content; choose only the media that help it.

Transfer exercise: explain why a river splits around an island. Derive diagram,
reading sequence and one local control from flow conservation. Keep quantities
and limitations truthful; a decorative wavy headline alone is insufficient.

## Source and rights

Reference edition: 2025 essay, inspected 2026-10-15. The manifest records source
capture identity and its limits. External article, lettering, images and data
remain link-only; no general reuse license was established. This lesson and its
paper specimen are original Mirket material under the bundled MIT license.
Visibility is not permission to copy assets or evidence of human acceptance.

## Read, render, verify

Read `mirket catalog read design-inspiration --path references/design-languages/pudding-subject-story.md`.
MCP: `skill_read` with `skill_id: design-inspiration` and that path, or resource
`mirket://skills/design-inspiration/references/design-languages/pudding-subject-story.md`.
Read/open the example at `assets/design-languages/pudding-subject-story/index.html`
under the same installed skill. `interface-design` owns usable controls;
`graphic-design` owns the diagram and concept. Inspect wide and narrow sizes,
keyboard range/reset, zero and four folds, no-script content and the misleading
variant. New subjects need fresh arithmetic and comprehension checks.
