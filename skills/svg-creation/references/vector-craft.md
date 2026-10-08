# Vector craft and production

Use the sections matching the asset. Values below explain construction choices;
they are not a universal style, fixed palette or mandatory effect stack.

## Compose before detailing

Write a short art direction from the actual brief: subject, focal silhouette,
shape language, density, palette and where the asset sits. For example, a travel
illustration might use broad coastal planes and one architectural focal point;
an infrastructure diagram needs readable relationships rather than painterly
depth. Do not turn every brief into a generic gradient orb, floating card or gear.

Block three useful scales: the dominant mass, supporting forms, and small accents.
Check whether negative spaces remain intentional and the eye reaches the focal
point. Inspect a silhouette or temporary grayscale preview to expose confused
hierarchy. Use detail where it describes form or meaning; equal detail everywhere
competes with the subject. Align the composition optically within its frame.

For reference reconstruction, map overall bounds and major landmarks first.
Compare an overlay or side-by-side render at matching dimensions where available.
Fix proportions before color and effects. A missing source or unreadable detail
is a limit to state, not permission to invent an approved logo.

## Geometry and Bézier discipline

- Use circles, ellipses and rounded rectangles for truly geometric forms. For
  organic contours, place anchors at extrema or changes in curvature, then tune
  handles. Adding many anchors often introduces bumps and makes editing harder.
- In a cubic segment `C x1 y1 x2 y2 x y`, handles set the tangent at each endpoint.
  At a smooth join, align incoming and outgoing handle directions; adjust lengths
  for the intended curvature. A cusp or corner intentionally breaks continuity.
  `S` reflects the previous cubic handle, so it is unsuitable for an unrelated
  next curve. Check joins enlarged and at display size.
- Close filled silhouettes intentionally. `fill-rule="evenodd"` can express
  holes without relying on contour direction; `nonzero` depends on winding.
  Inspect counters, cutouts and self-intersections rather than toggling fill rules
  until an accidental contour happens to look right.
- Keep one perspective. An isometric construction can use shared basis vectors
  such as `(0.866, 0.5)`, `(-0.866, 0.5)`, `(0, -1)`; draw from a common grid.
  An organic illustration need not use that projection. Avoid mixing arbitrary
  face angles and incompatible vanishing points within one object.
- Round only after inspecting error at final scale. A small coordinate change can
  destroy a narrow counter or a motion path. Path count and byte size alone do
  not measure craft.

A smooth two-segment contour, with explicit handles for later editing:

```xml
<path d="M 24 92 C 48 28 96 28 120 76 C 144 124 188 124 216 60"
      fill="none" stroke="currentColor" stroke-width="4"
      stroke-linecap="round" stroke-linejoin="round"/>
```

The handles adjacent to `(120,76)` share a direction. Adapt coordinates to the
subject instead of copying this wave into every illustration.

## Icons, marks and optical alignment

For icons, reuse the established viewBox, stroke, cap/join, corner family and
padding. Design the minimum supported size explicitly. A 16px icon and a 1600px
hero do not need the same construction. Simplify tiny details or supply an optical
small-size variant when justified; do not shrink a complex master blindly.

Pixel alignment depends on the actual scale and stroke width. At 1:1, an odd
device-pixel stroke may benefit from a half-pixel center; this is not a universal
rule across sizes or DPR. Compare real rasterization. Circular/diagonal shapes
often need small optical corrections relative to rectangles. Keep recognizable
negative space and balanced apparent weight. Use `vector-effect="non-scaling-stroke"`
only when fixed screen stroke is intended; it changes the relationship between
form and line when resized.

For a mark, inspect one-color reproduction and the required small size. Color
must not be the only thing separating essential parts. Do not invent trademark
ownership or claim originality/licensing review beyond work actually performed.

## Color, material and effects

Use a compact set of color roles: main mass, supporting plane, accent and outline
where appropriate. Flat, engraving, cut-paper and dimensional work each need
their own coherent material rules. Gradients should explain surface or lighting;
choose explicit stops and direction. `gradientUnits="userSpaceOnUse"` provides a
shared coordinate model across forms; object-bounding-box gradients fit each
object and can produce different highlights.

Separate foreground, subject and background by value/contrast and edges. Draw
cast/contact shadows from the same light as the highlights. Inspect transparency
over the actual light/dark background. Avoid adding blurs, glow or noisy textures
to every surface. If a texture is justified, keep it vector/filter-native for a
pure-vector requirement and verify its export support.

Define filter and mask regions deliberately. Blurs and displaced pixels can be
clipped by their regions or the root viewport; expanding only the viewBox may
not fix the filter. Masks have different coordinate and luminance/alpha behavior
from clip paths. Check the real browser and requested editor/exporter. A visible
browser result does not prove every vector editor supports the effect.

## Editable structure and integration

Use semantic groups such as `hull`, `sail`, `reflection` or `orbit`, shared `defs`
and meaningful IDs. Avoid generated path soup for simple shapes. A component's
public size/color/label API should follow project conventions; do not create a
configuration framework for one asset.

Definitions are document-wide when SVG is inline. Generate an instance prefix
with the project's SSR-safe ID mechanism, propagate it to `url(#...)`, `href` and
ARIA references, and keep IDs stable across hydration. Scope CSS to the asset.
Render two different-colored instances together to reveal collisions. A sprite
needs its own tested `<symbol>`/`<use>` sizing and labeling contract.

Keep a meaningful `viewBox`, aspect ratio and margins. Use `preserveAspectRatio`
intentionally; `none` distorts artwork. For web usage, preserve intrinsic ratio
when sizing, and check clipping in the parent container. Avoid bounding-box
cropping that removes stroke, shadow or motion travel. Separate canvas padding
from host page layout.

Informative SVG may use `role="img"` and `aria-labelledby` referencing a unique
`title` and optional `desc`. Decorative inline artwork can use `aria-hidden="true"`
and `focusable="false"`. Labels explain the content rather than listing every
path. Live text requires font availability and correct shaping; check accented,
long or non-Latin content when affected. Outlines lose searchable/editable text.

## Critique and handoff

Inspect thumbnail, intended size and enlarged detail on the actual background.
For icons include the minimum size; for print include the available target
renderer. Name and repair concrete defects: lumpy curves, hairline gaps, uneven
apparent weight, ambiguous silhouette, mismatched perspective, crowded negative
space, abrupt gradient seams, clipped effects or illegible text.

Use a contact sheet or animation storyboard only when it helps comparison. An
automated structural pass cannot judge these visual problems. Keep a readable
master and compare optimized output in the real context. Deliver final geometry,
optional size variants and exports, source provenance and the tested integration.
Do not claim all-browser, print or production certification from one render.

## Technical sources

These specifications describe syntax/behavior, not an aesthetic recipe. Check
installed browser/editor support when using features beyond the existing project.

- [SVG coordinates, viewBox and units](https://www.w3.org/TR/SVG2/coords.html)
- [SVG paths and fill geometry](https://www.w3.org/TR/SVG2/paths.html)
- [SVG painting, strokes and paint servers](https://www.w3.org/TR/SVG2/painting.html)
- [SVG accessibility](https://www.w3.org/TR/SVG2/struct.html#Accessibility)
- [SVG embedding/processing modes](https://www.w3.org/TR/SVG2/conform.html)
