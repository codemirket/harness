# Editable illustration craft

This is an on-demand method for artful SVG cards and character scenes. It is a
method adaptation from the pinned MengTo illustration skills, not a copied
style kit. Use it with the main `vector-art` skill; keep the user’s brief and
existing Mirket ownership authoritative.

## Choose a style and combinations deliberately

Start with the output context: card or page size, crop, background, audience,
and the one idea the viewer should understand first. Select one coherent
family by its observable relationships:

- outline versus no-outline shapes;
- flat fills versus drawn texture or limited shading;
- character proportion, viewing angle, and gesture;
- prop/detail density and amount of surrounding space;
- palette roles and line hierarchy; and
- whether the composition is staged in a frame, on an implicit floor, or in
  open space.

Write the contract before authoring: subject and relationships, focal object or
action when relevant, palette roles, line/fill rule, composition, short text,
and motifs to avoid. A supplied style reference is a fidelity anchor, not a
license to trace it. If the brief does not identify a family, make the smallest
style decision that supports the subject and state the assumption. Prefer one
coherent family for a spot illustration; combine a framed scene’s panel/
breakout language with a no-outline floating-character family only when the
brief explicitly calls for that hybrid and the contract says which rules win.

## Block action, pose, and contact first

For figurative spot art, prefer one visible verb and, when useful, one hero
prop. For abstract, decorative, or multi-object compositions, choose the
smallest block-in that represents the brief instead. When a figure is present,
place the torso/hip, shoulder line, knees, feet, prop, and hand contacts as
simple geometry before faces, clothing, texture, or decorative marks. At the
intended display size, ask:

- Does the silhouette read without labels?
- Is the action legible from the lean, twist, weight-bearing limb, and prop?
- Does the figure separate from the background by value or outline?
- Do hands actually reach the object, and do feet/seat/prop/shadow meet their
  support?
- Is there enough negative space for the focal action and the reading order?

When figurative hands are in scope, write down left/right, palm/back, finger
direction, and which side the thumb belongs on. Build cuff or wrist, forearm,
palm, fingers, and thumb as a connected construction. Mirror a permitted
template only after checking handedness; a mirrored hand is not automatically
anatomically right. Draw an object being held before the hand so fingers and
thumb overlap it.

For figurative scenes, keep far limbs/objects behind near ones and preserve a
deliberate draw order: background/card, environment, floor or support, far
figure/prop shapes, torso, near limbs and hands, held-object overlap, face/hair,
accents, and labels. Abstract and non-figurative compositions may use another
order. In every case, each overlap should explain depth or contact rather than
hide a gap.

## Add style-specific detail

Once the block-in passes, add only details that reinforce the selected family:
face direction and expression, hair masses, clothing separation/folds, prop
controls, texture marks, breakout elements, and short labels. Keep labels short
enough to remain shapes in the composition; use real text only when the output
needs editable/searchable text and verify its font/embedding rights.

Use measured values from an approved style reference as local anchors, not
universal laws. Re-measure when the canvas, crop, medium, or subject changes.
Avoid detail that disappears at 1x, creates accidental tangencies, or competes
with the action. Do not use gradients, filters, raster images, or shadows just
to rescue a weak silhouette; use them only when the chosen family and output
contract require them and the qualified route preserves them.

## Inspect the saved result

Render the actual saved SVG before and after qualified optimization. Inspect the
pixels, not merely a command result:

1. View the intended size for silhouette, focal action, reading order, palette,
   contrast, and label legibility.
2. View a pressure size or crop for clipping, thin strokes, merged fills,
   disappearing gaps, and accidental visual readings.
3. Zoom each hand, face, foot, held-object overlap, and support/contact region
   from the saved vector. Check thumb side, finger separation, joints, contact
   shadows, and whether the object is in front/behind as intended.
4. Compare beside supplied or approved style references at matching scale. Judge
   relationships—line hierarchy, density, silhouette, palette roles, and
   breakout/contact behavior—not whether the new art copies a particular mark.
5. Have a capable independent reviewer identify contract failures and concrete
   pixel defects when the work is high-risk or subjective. Apply the smallest
   repair supported by that finding and rerender the affected views.

Keep the original source when optimization changes or loses geometry, IDs,
masks, gradients, clip paths, symbols, accessibility metadata, or external
references. Report unsupported features rather than silently flattening them.
Visual inspection remains untested when no image-capable view of the saved
render occurred.

## Adaptation and rights note

Authoring reference: [MengTo/Skills illustration guide](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/illustration/README.md),
[framed-panel skill](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/illustration/illustration-framed-panel/SKILL.md),
[flat skill](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/illustration/illustration-flat/SKILL.md),
and [iterate-until-verified](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/agent-skills/codex/iterate-until-verified/SKILL.md),
all inspected at commit `a45892d2fdadab75b31fb16abecc02f101b31c43` on
2026-10-27. The repository’s [MIT license](https://github.com/MengTo/Skills/blob/a45892d2fdadab75b31fb16abecc02f101b31c43/LICENSE)
covers its own source under that license; this skill is a concise, independently
written method adaptation. It does not copy the examples, palettes, scripts,
assets, private paths, or upstream benchmark claims. Third-party fonts/assets
still require their own rights review.

For the substantial method adaptation, retain the applicable upstream notice:

```text
MIT License

Copyright (c) 2026 Meng To

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
