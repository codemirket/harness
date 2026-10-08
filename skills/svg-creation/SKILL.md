---
name: svg-creation
description: Create, refine, or review editable SVG vector illustrations, icons, logos, pictograms, and decorative artwork. Use for vector art, precise paths, scalable brand assets, or SVG cleanup; SVG choreography uses svg-animation, and page layout uses interface-design.
---

# SVG creation

Produce an intentional composition and editable vector source. A file with an
SVG extension, valid XML, or many paths does not establish visual quality.

## Define the artwork

Inspect supplied references, brand assets, existing icons, intended background,
placement and minimum displayed size. Establish the subject, mood, visual style,
focal point and delivery context: standalone asset, inline component, sprite,
print or a starting frame for animation. Use the project's design system when
present. Preserve approved marks; do not replace them with an approximation.

For an open brief, choose a coherent direction and explain it briefly. For
fidelity work, inspect the reference and compare proportions, silhouette,
negative space, curves and palette in equivalent renders. Ask only when the
missing direction materially changes the requested result. Do not add a variant
picker or an asset pipeline by default.

Read [vector craft](references/vector-craft.md) before substantial illustration,
logo or icon work. Use its relevant construction and critique sections. For
animated artwork, let `svg-animation` lead motion and use this skill for geometry.
For a whole screen, `interface-design` leads placement and hierarchy.

## Construct deliberately

1. Block out the silhouette, major masses and negative space. Render at the
   target size before investing in detail. The subject must read without effects.
2. Refine contours with primitives and intentional Bézier paths. Use few useful
   anchors, deliberate tangent continuity, a consistent perspective and optical
   alignment. Keep symmetry only where the subject calls for it.
3. Establish a small hierarchy of color, stroke and detail. Design light and
   shadow from one lighting model when depth is appropriate. Texture, gradients
   and filters support form; they cannot rescue weak composition.
4. Organize editable semantic groups, shared definitions and named motion parts.
   Set a usable `viewBox`, explicit namespace for standalone files, and intentional
   sizing. Preserve enough coordinate precision for the minimum displayed size.
5. Inspect on the intended background and at small, intended and enlarged sizes.
   Correct collisions, uneven strokes, accidental tangencies, clipped effects
   and competing detail. Repeat until the named visual problems are resolved.

Use code-native geometry or an available vector editor. A raster image wrapped
in SVG is not a vector-art deliverable. If image generation supplies a mood or
composition reference, construct the final editable geometry separately and
disclose the reference; do not call automatic tracing professional refinement.
Use existing tools first. Additional editors, libraries, fonts and optimizers
need the user's authorization under the project's rules.

## Make the asset usable

- Match the integration contract. An inline component needs instance-unique
  definition and accessibility IDs; static files can use a stable asset prefix.
  Check two copies together, not only one isolated preview.
- For informative inline artwork, use a useful accessible name and description
  where needed; for an HTML image use the host's appropriate `alt`. Decorative
  artwork should be hidden from assistive technology. Interactive SVG needs the
  project's semantic controls and keyboard behavior.
- Keep source groups and a readable master when an optimized output is requested.
  Use an already available optimizer only after rendering a before/after comparison;
  preserve IDs, text, precision and groups needed by consumers or motion.
- Decide editable text versus outlined lettering from the delivery requirements
  and available licensed fonts. Preserve editable source when outlining; never
  promise font-independent fidelity for an asset that relies on an external font.
- Treat imported SVG as untrusted markup. Inspect active content and dependencies;
  use the project's real sanitization boundary if accepting uploads. This workflow
  and its structural audit are not sanitizers.

## Verify and deliver

Optionally run the bundled standard-library structural check:

```sh
python3 scripts/audit_svg.py /absolute/path/art.svg --json
```

Run it relative to this skill directory or use the installed script's absolute
path. It checks XML, `viewBox`, IDs and common local references, and reports active
content, raster elements and dependencies for inspection. It does not validate
path grammar, all CSS, accessibility, clipping, security or aesthetics.

Render the real integration and inspect it. Deliver the editable SVG/component,
requested exports and concise integration details: dimensions, palette hooks,
label/decorative treatment, dependencies and animation handoff groups. Report
observed quality, the inspected sizes/context and material gaps. A browser preview
does not establish print/editor fidelity or compatibility on untested devices.
