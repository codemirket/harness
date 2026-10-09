# Worked example: Fieldwork Studio

This is a fictional materials-library brand extracted from the harness's
`examples/craft-lab/index.html`, then turned into a proposed working specification.
It demonstrates the method; it is not a template identity for unrelated products.
The source specimen is optional context when available, not a runtime dependency.

## Position and visual language

Audience: designers comparing a small set of material references. Promise: find,
inspect and retain a useful reference without losing the collection context.
Personality: quiet and practical. In use, that means descriptive material names,
short metadata, restrained green actions, warm neutral surfaces and original
geometric material studies. It does not mean tiny text or hidden controls.

## Observed primitives and proposed role map

| Primitive | Hex | Role in the specimen |
| --- | --- | --- |
| Paper | `#f5f4ef` | Workspace background |
| Surface | `#fffefa` | Cards, inputs and dialog |
| Ink | `#252e29` | Primary text |
| Muted ink | `#647066` | Supporting text |
| Rule | `#dcdfd6` | Decorative separators |
| Forest | `#345947` | Primary action and mark |
| Pale green | `#e8eee6` | Selected navigation background |

Keep these observed primitives. A proposed token handoff can add role aliases:

```json
{
  "color": {
    "surface": {"canvas": "#f5f4ef", "raised": "#fffefa"},
    "text": {"primary": "#252e29", "secondary": "#647066"},
    "action": {"primary": "#345947", "onPrimary": "#ffffff"},
    "border": {"decorative": "#dcdfd6"}
  },
  "font": {"body": "system-ui, sans-serif"},
  "radius": {"control": "6px", "card": "10px"}
}
```

The actual source's platform font stack starts with `-apple-system`, then
`BlinkMacSystemFont`, `Segoe UI`, `sans-serif`. The simplified token above is a
proposal; do not describe it as an exact extraction. Its card radius is 10px,
control radius usually 6px, dialog 14px. Document exceptions by component role
instead of flattening them into one radius.

Measure Ink/Paper, Muted/Surface, white/Forest and interactive outlines against
adjacent surfaces. Decorative Rule is intentionally subtle; do not reuse it as
the only necessary indicator of an input or selected state without checking
that boundary. Contrast is a property of a rendered pair, not a color name.

## Typography and mark

Observed desktop heading: 30px, line height 1.2, weight 570. Mobile heading:
27px. Material names: 14px desktop/15px mobile. Supporting copy: 11–13px.
These are observed sizes, not an approved legibility floor. In a real handoff,
test actual font availability, user scaling and the smallest supporting text;
promote weak labels before proclaiming a type scale complete.

The original mark is a green line-built F/grid motif, `viewBox="0 0 30 34"`.
Preserve its aspect ratio and editable strokes. The specimen renders it at
29×32px desktop and 25×27px mobile. Inspect both before choosing a minimum.
Proposed clear space: at least one main stem width around the artwork; compare
that rule to the real lockup rather than deriving a sacred ratio after the fact.
Do not stretch the mark, use it as a tiny status icon, or merge its details into
a texture. A smaller favicon is a separate design task, not a downscaled promise.

## Voice through useful examples

| Situation | Observed or proposed copy | Reason |
| --- | --- | --- |
| Product title | Observed: “Materials library” | Names the job plainly. |
| Product introduction | Observed: “Good things begin with a closer look.” | One expressive line; controls remain concrete. |
| Save action | Proposed standard: “Save material” / “Saved” | Same object and state in card, preview and assistive labels. |
| Empty collection | Observed: “Your collection starts here” + “Explore all materials” | Explains the next useful action. |
| Save failure | Proposed: “Couldn't save this material. Try again.” | Acknowledges failed persistence, keeps the recovery action explicit. |
| Material claim | Observed: “Warm white · 300 gsm” | Fictional record, clearly labeled demo; not a supplier certification. |

Current card accessible labels use “Save Cotton paper”; the dialog uses “Save to
collection” / “Saved to your collection”. This is a vocabulary variation to
resolve if the brief requires a consistent action system, not evidence that the
user journey is broken. Distinguish a consistency proposal from a requirement.

## Asset and delivery contract

Use names such as `fieldwork-mark-forest.svg`, `material-cotton-master.svg` and
`material-cotton-640.png`. Inline vector instances need unique paint-definition
IDs; retain semantic groups in the master. The specimen's material studies use
framed geometry, subtle texture and controlled warm/green color families. Avoid
claiming photographed realism: these are illustrative studies.

Inspect the lockup at its displayed size, all six material studies on their
cards, and a preview crop. Export raster at a requested size from the actual
vector master. Include source dimensions, color space and dependencies. Do not
label an untested CMYK conversion or editor import as print-ready.

## Current primary references

- [W3C contrast minimum](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html):
  normal text uses 4.5:1, qualifying large text 3:1; thresholds are not rounded up.
- [W3C non-text contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html):
  inspect required component/state information, not every decorative separator.
- [Primer color usage](https://primer.style/product/getting-started/foundations/color-usage/):
  a relevant reference for semantic color roles, not a license to copy a brand.

Sources reviewed for the workflow on 2026-10-09. Brand approval and rights to
third-party assets are separate from technical validity and contrast measurements.
