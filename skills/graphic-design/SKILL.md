---
name: graphic-design
description: Create and refine visual identities, illustrations, posters, campaign graphics, diagrams and coherent asset families. Use for graphic composition, typography, visual storytelling and production exports for screen or print. Pair with interface-design for working UI and the available native document or media skill for its file format.
---

# Graphic design

Deliver a composition that communicates at its intended size, with usable source
and exports. Match the medium to the brief and verify the exported result.

## Professional stance

Make the intended message perceptible through composition and visual storytelling.
Prioritize reading order, a defensible visual idea and precise production craft.
Ask what the viewer should notice first, what they should understand on a second
look, and which detail must survive the smallest placement. Develop composition
before surface finish; compare materially different concepts only when the brief
leaves an important creative decision unresolved.

Challenge marketing when copy cannot support a truthful, legible composition and
brand guidance when a literal application destroys the message in this medium.
Explain the proposed exception with a specimen the brand owner can evaluate.
Give collaborators editable masters, crop or construction rules and inspected
exports with their intended uses. Revise the visual idea when use-size inspection,
audience evidence or a production proof exposes lost meaning; aesthetic preference
alone does not establish that the audience understood it.

## Define the visual job

Identify the audience, message, desired response, placement, dimensions and
deliverables. Recover these from the request and existing brand materials. Note
language, copy, asset rights, accessibility needs and production constraints.
Separate required facts and supplied brand decisions from open creative choices.
Do not invent endorsements, metrics, customer logos or product details to make a
design seem credible.

For a family of assets, define what stays consistent and what may vary across
formats. A logo, social crop, diagram and poster need different use-size decisions;
resizing one master mechanically may not preserve meaning or readability.

Choose the craft emphasis from the deliverable:

- **Illustration:** establish the subject, action, viewpoint, silhouette and focal
  detail before polishing material or texture. Check physical contact, overlap,
  perspective and light against what the scene claims; an attractive surface
  cannot repair an impossible grip or an incorrectly constructed object. Use
  appropriate source references for unfamiliar anatomy or technical subjects.
- **Asset family:** define the shared construction rules (grid, optical weight,
  corner/line treatment, palette and detail level) and intentional exceptions.
  Keep master-to-export mappings and consumer requirements explicit: dimensions,
  backgrounds, transparency, naming and accessible labeling in the consuming UI.
  Test an actual integrated asset as well as the standalone export; for inline
  SVG, include two instances to expose duplicate IDs in masks or gradients.
- **Graphic composition:** let the message determine reading order and the
  relationship between type and image. Test the complete poster, campaign or
  information graphic at its delivery size; a strong central image does not
  establish legible supporting copy or a truthful chart.

## Establish the composition

Choose a visual idea that explains or expresses this message. Set reading order,
typographic roles, grouping, alignment, scale, negative space and focal contrast.
Use reference material to identify useful principles without treating its assets
as reusable or its style as mandatory. No palette, font family or layout is
universally superior.

Work with actual copy and image proportions early. Inspect line breaks, optical
alignment, cropping and relationships at the intended size. For an uncertain or
costly direction, make a small specimen or a few meaningfully different concepts.
Continue from a clear brief or existing direction; routine refinements do not
require a new approval ceremony.

For artwork that will move, let the static composition establish the hierarchy
first. Identify one focal action, the parts that respond, the intended material
or weight, and the resting pose. A restrained editorial illustration and an
energetic graphic campaign need different rhythm as well as different colors.
Keep important copy and the message understandable without playback.

When references would clarify a construction method or conceptual relationship,
the `design-inspiration` library's `references/design-languages/index.md` or catalog
search by purpose can help. Its original graphic lessons distinguish hybrid
explanation,
generative identity, subject-derived identity, human scenes and dimensional
symbols. The index explains source/rights records and optional contrast exercises;
other references and methods remain available. An identity case study is not a
reusable brand asset pack, and a sample's geometry does not replace this brief's subject.

## Choose the production route

Read [production and proof](references/production-and-proof.md) for tool selection,
rights, color, export and final inspection.

- Use supplied/licensed raster imagery and Sharp for crop/resize preprocessing.
  Use `vector-art` for authored SVG illustrations, semantic scene groups and a
  native resvg preview; inspect that preview and read back the editable SVG.
  Keep exact business text in an editable layout.
- Use SVG geometry for editable marks, icons and scalable vector illustration.
  Run generated and imported SVG through SVGO before delivery. Raster generation does not produce vector paths;
  wrapping a bitmap in an SVG does not make it vector artwork.
- For editable design files and structured visual layouts, use the official
  Penpot MCP/plugin. [Design documents](references/design-documents.md) describes
  saved-file refinement and preview checks. Website implementation belongs in
  the project's frontend tools.
- For flows, C4, sequence, Vega-Lite and Excalidraw diagrams, preserve nodes,
  relationships, direction and conditions in source and render through Kroki.
- Use Satori plus resvg for static text/image compositions using supported
  Flexbox and supplied TTF, OTF or WOFF fonts. Use Typst for paged documents and
  decks. A flattened image is not an editable deck, workbook or document.
- For artwork inside an expressive website, use `advanced-motion` for the shared
  scene and its medium-selection reference; `svg-animation` owns editable vector
  choreography. `interface-design` owns page hierarchy and usable controls.
  Choose those supports for the actual output, not every graphic commission.

Use tools already available. A missing renderer, font, editor or print profile is
a specific limitation to resolve; skill selection does not install or authorize
dependencies, accounts, paid services or external publication.
Compile Style Dictionary tokens before every dependent render; compare matching
pixel outputs with Odiff and inspect material differences.

## Finish the actual deliverable

Inspect every distinct final composition and all focal elements. Check the
smallest intended presentation, full composition and fine detail. For an asset
family, compare the exports together for consistent type, color, illustration
language and hierarchy, then inspect each for crop-specific defects.

Repair defects in the editable source, export again and reinspect affected
outputs. Check required dimensions, transparency, color handling, fonts, margins,
links and file format. Keep only useful source, final exports and requested
evidence; remove owned temporary renders and scratch files.

Deliver the files with a concise statement of intended use, inspected outputs,
asset/license constraints and any unverified production step. A screen preview
does not prove printer output, and visual inspection does not imply user approval.
