# Character construction and status families

Use this when a brief benefits from illustrated identity or a meaningful state
family. A quiet text label or existing icon can be the better solution. Preserve
the established brand and actual application status; animation cannot manufacture
progress, attention, confidence or completion.

## Build a character that can hold a pose

1. **Silhouette:** block one recognizable mass in a common coordinate system.
   Check it filled with one color at the smallest intended size. Choose supporting
   parts for identity or action; avoid thin appendages that disappear in placement.
2. **Face landmarks:** mark eye baseline, eye spacing, gaze center and mouth anchor
   relative to that mass. Adjust these landmarks for each silhouette. A shared
   coordinate system does not make one face placement fit every shape. Keep the
   face attached to the head through rotation, tilt and squash.
3. **Apparent volume and material:** decide whether the subject is flat paper,
   ceramic, soft fabric or another material justified by the brief. A darker side
   plane, consistent highlight and contact shadow can explain a small rounded
   object without a full renderer. For projected depth, keep front-face landmarks
   on the same projected plane; moving eyes independently across the silhouette
   can look like sliding decals. Material detail must survive at use size.
4. **Pose rig:** name the moving groups and their pivots. Separate placement from
   body tilt/lift, face direction and secondary parts. Express a small pose as
   data, then let one controller own each property. Avoid unrelated random
   movement that breaks the character's apparent weight or gaze.
5. **Coherent states:** define a readable resting pose for each real state. Keep
   identity, lighting and structural proportions stable while varying posture or
   emphasis. A ready state might face the viewer; working might lean toward the
   work; rest might lower the gaze. These are hypotheses for this character, not
   universal emotional mappings. Explain why each distinction serves its audience.

Motion choreography belongs to `svg-animation` or the surrounding
`advanced-motion` scene. Hand over editable groups, pivots, state poses, margins
and which detail carries the message. Do not bake application success into a
celebration's completion callback. A resting pose must still communicate with
motion disabled; visible status text supplies the precise meaning.

## Design a family at optical sizes

Define shared construction rules and purposeful differences before adding
effects. Use shape, grouping, posture and text as well as color. Never depend on
recognizing a subtle loop to distinguish waiting from failure.

| Example placement | Construction decision | Check |
| --- | --- | --- |
| Compact status beside text | One strong glyph, generous gaps, no texture or tiny pupils | Recognizable at its actual CSS size and background |
| Mid-size agent identity | Stable silhouette and larger face landmarks; minimal side plane | Face and state remain legible without movement |
| Editorial illustration | More articulated pose, intentional lighting and supporting detail | Focal action remains stronger than the material treatment |

These are roles, not fixed breakpoints. Derive sizes from the real consumers.
Compare the family at matched optical weight; reducing a large master's dimensions
alone may collapse eyes, counters or line gaps. For a busy/search/connected family,
one compact construction could use separated marks, a scanning frame and joined
marks, respectively. Test whether viewers understand those choices; keep literal
status text when the distinction matters. A public package's named states are
examples, not a standardized visual language.

## Small editable specimen

Open [the character study](../assets/procedural-character/index.html) through the
project's existing local preview server. The inline SVG groups in that HTML are
the editable vector source; `styles.css` sets two independent materials and
`motion.js` maps three example states to finite poses. This is an original small
construction example, not a library engine or a reusable production component.

- The two silhouettes use different shapes/materials and unique gradient IDs.
  Face landmarks remain inside their moving parent group.
- Real buttons change visible status immediately. With JavaScript unavailable,
  the two ready poses and text are visible; disabled demonstration controls are
  explained. Decorative completion never owns status.
- Repeated input starts from the current displayed transform and cancels the
  prior owner's animation. State changes settle immediately under reduced motion,
  including live preference changes. There are no ambient loops.
- Offscreen/hidden/removal handling settles or cancels finite movement. A
  disconnected custom element releases its listeners/observer; reconnection
  restores its latest state. SVG viewBox coordinates preserve geometry on resize.
- The compact static glyphs illustrate ready/working/rest with simplified shapes
  and labels. They are deliberately reconstructed for small use rather than
  scaled copies of the larger character.

Inspect both at normal playback speed, an intermediate pose, rest and narrow
width. Use keyboard input, rapidly change state, toggle reduced motion during a
transition, remove/reinsert one instance and change its material independently.
Check labels, focus, clipping and sibling independence. The particular easing and
duration are specimen choices, not timing rules or measured performance claims.

The specimen's SVG is editable vector geometry; a Canvas rendering of a
procedural character would be a bitmap. Neither proves delivery of an editable
3D master. A loading/reveal shader consumes existing images; it does not generate
an illustration. Keep original geometry/code and exported pixels distinct.

## Source and rights boundary

Method synthesis informed by public MIT sources in
[Libraries.dev at a275665](https://github.com/Jakubantalik/Libraries.dev/tree/a27566557b4def79bcdcedc71858c0fb28020b6d):
`packages/bot-avatars/src/{shapes,draw,engine}.ts` for shape/landmark/pose separation
and `packages/thinking-orbs/src/{presets.ts,ThinkingOrb.tsx,theme.ts}` for optical
tuning and lifecycle concerns. This example does not copy those renderers, shapes,
presets or artwork. Applicable notices are retained by Mirket's source records.
The repository distinguishes public MIT package code from plan-licensed Studio
exports, Pro presets and recipes. API availability grants no paid entitlement;
do not infer asset rights from a demo or package interface.
