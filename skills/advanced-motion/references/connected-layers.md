# Plan connected layers before preparing assets

Use this when a scene needs independently moving image planes. A static image,
simple DOM composition or existing footage may already satisfy the brief; depth
does not require a fixed number of layers or a more elaborate renderer.

Record only the relationships the scene needs:

| Part | Motion responsibility | Contact or occlusion invariant |
| --- | --- | --- |
| Environment | Establish distance with restrained movement or none | A clean plate fills every area uncovered by moving foreground parts |
| Subject and support | Carry the focal action | Feet, wheels or a product base stay attached to their intended support unless separation is the action |
| Foreground | Reveal or frame the subject | Its travel never accidentally hides essential copy or exposes unfinished edges |
| Copy and controls | Preserve meaning and usable actions | Semantic DOM remains readable and operable independently of decorative progress |

For a simple grounded cutout, name its contact point in source coordinates and
the corresponding point on its support. A common contact pivot and shared
translation can keep that point aligned while the planes scale differently.
Separate layout placement from expressive transforms and verify the **rendered**
contact after crop, transform order and responsive changes. A shared CSS percentage
origin is insufficient when assets have different bounds. Perspective, rotation
or changing geometry may require another construction; do not force a flat pivot
model onto a real 3D relationship.

For example, a lamp on a plinth can grow in apparent scale as the environment
recedes while its base remains on the plinth. The opening already shows the product
and description; the midpoint reveals the material; the exit settles into the
details. Inspect whether this relationship helps the brief before adding movement.
This is an illustrative construction, not a prescribed hero or brand direction.

Have `graphic-design` prepare registered layers, real alpha and the clean plate
using its production-and-proof reference. Retain the editable/master assets and
their contact/crop coordinates with the delivery exports. A generated checkerboard
is not transparency, and separately generated layers may disagree in light or
perspective even when their prompts match.

Keep a complete poster or static composition until all **required** layers are
decoded and aligned. Optional atmosphere must not block readiness. If a required
layer fails, retain the complete fallback rather than showing half a scene or a
duplicate subject. Reuse the lifecycle ownership in scene mechanics so late loads
cannot reveal a replaced scene. Essential text and actions stay available throughout.

At narrow sizes reconsider crop, contact coordinates, typography, stacking order
and travel together. Inspect opening, an intermediate beat and resolved exit in
both directions, including delayed/failed layers and geometry changes. Look for
floating contacts, duplicated subjects, newly exposed holes, halos and occlusion
of copy. Reduced motion uses the readable resting composition without gratuitous
pinned space. These observations establish the inspected states, not human approval.

Technique adapted from Nate Herk's [scroll-craft hero-depth guidance](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/references/hero-depth.md)
and [layer contract](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/references/approved-collection.md),
MIT; no mandatory depth, layer count, provider or author example assets retained.
