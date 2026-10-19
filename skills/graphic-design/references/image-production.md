# Image production capability and handoff

Separate art direction, image creation, and final layout. A general reasoning
model plus a renderer is not automatically an image-generation service. Rich
raster artwork and editable geometric vectors have different producers; neither
medium is inherently better. Choose for the brief and actual consumer.

## Check the actual capability

Inspect the tools exposed in the current host session and their documented
contracts. For generation/editing, establish supported inputs, reference images,
output delivery, transparency, edit constraints, and applicable permissions or
cost limits. Read the host's image skill when available. Do not infer support
from a model name, installed plugin, package receipt or tool description alone;
availability, successful invocation and useful output are separate observations.

Mirket's owned worker tools currently do not include image generation or image
editing. `raster-processing` only transforms existing images. Do not add invented
workflow IDs, enable ambient worker tools, change providers or route through a
shell to evade the assigned scope. A host with an authorized image tool can
produce assets directly, then supply the resulting files for Mirket composition,
vector work, export and inspection. No new account or dependency is implied.

If the host has no usable image model, direct composition in code is permitted.
Record the gap, build one minimal HTML/CSS/SVG/Canvas composition, render and
inspect it as the frozen target before expanding. This fallback needs neither
additional approval nor a new provider. Keep requested-medium limitations explicit;
do not claim photographic or painterly equivalence from a geometric sketch.
A prompt alone is an unfinished
asset, and a generated bitmap inside SVG is not an editable vector illustration.

## Carry a usable brief across the boundary

Use the task's existing notes rather than another mandatory state format. Carry:

- Intended message, subject/action, audience and placement dimensions/crops.
- Visual direction: viewpoint, focal hierarchy, shape language, palette,
  lighting/material, detail level and exclusions that matter to this brief.
- Relevant reference images and the separate lesson each provides; preserve
  source identity, rights and which supplied brand assets must remain exact.
- Required output format, resolution and real transparency, plus editing needs.
- Exact text/marks to keep in the layout layer and the space the image must leave.
- Available producer, authorized limits and how the final pixels will be inspected.

For image-to-code, establish a full composition target rather than only a focal
asset. Keep its identity and use different paths for implementation captures.
Generate or edit through the actual tool, inspect returned pixels, and refine
visible defects rather than repeatedly adding adjectives to the prompt. Keep
iteration feedback concrete: subject contact, malformed forms, hierarchy,
lighting continuity, crop or unwanted detail. Follow the tool's edit/reference
contract to preserve identity across revisions. Do not invent a seed or a
reproducibility guarantee the producer does not expose. Permit one target repair,
then at most two implementation correction rounds within ten minutes of active
production/verification for the composition. At the limit retain current output
and report unresolved gates; do not start another sample or worker.

## Integrate and verify

Use generated imagery as an image layer when that fits the deliverable. Keep
business text and supplied logos exact in the editable composition. Check crop,
contrast, edge quality and actual alpha against the intended backgrounds.
Inspect the composed asset at its real placement size, not only the image in
isolation. Preserve useful source assets and final outputs with provenance;
do not ship every rejected variant. Report missing editability or generation
limitations rather than calling a flattened result an editable master.
