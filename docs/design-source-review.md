# Visual and motion source review

Reviewed 2026-10-18. This review addresses output that works and animates but
remains below the desired visual quality. A larger effect library alone does
not resolve that. The current production contract is
[image-to-code with bounded quality gates](visual-quality.md).

All ten repositories were fetched at the revisions below and their tracked
inventories inspected. Review covered licenses, entry points, relevant referenced
methods and selected implementations or validators. Large component/plugin
repositories were examined by relevant owner, not every demo or business-domain
workflow. This is an admission review, not a full security audit or execution
benchmark. No upstream installer, hook, helper, hosted MCP or dependency
installation was executed.

## Admission decisions

| Source | Decision | Existing owner |
| --- | --- | --- |
| GSAP Skills | Corrected mechanics; original source replaces mirrors | `advanced-motion` |
| Diagram Design | Semantic construction and architecture comparison | `graphic-design` |
| Motion Primitives | Reference-only component analysis; no copied code | `motion-design` |
| LottieFiles | Retain vocabulary; strengthen motion diagnosis | `advanced-motion` |
| Canvas UI | Reference-only platform/lifecycle analysis | `advanced-motion` |
| Better Design | Public token/state review methods; no remote engine | `interface-design` |
| Taste Skill | Updated pin; subject specificity and focal proof | Interface/graphic art direction |
| Hallmark | Structural variety, without theme quotas | Interface art direction |
| Karpathy Skills | Exclude as redundant | Existing engineering/planning already cover it |
| Knowledge Work Plugins | Selected design critique and handoff | `interface-design` |

Exact admitted-source paths, pins, boundaries and notice selection are in
`registry/catalog.json`. Applicable full MIT/Apache texts accompany affected
embedded skills through `.upstream-licenses/NOTICE.md`. Reference-only analysis
of Canvas UI and Motion Primitives uses explicit notice exemptions: no upstream
source, prose or assets are redistributed. No new production dependency is added.

## 1. GreenSock GSAP Skills

[Reviewed revision](https://github.com/greensock/gsap-skills/tree/aed9cfd3277740755f6bfc1155c7aa645403b760):
`aed9cfd3277740755f6bfc1155c7aa645403b760`.

Examined the eight skill areas (core, timeline, ScrollTrigger, plugins, utilities,
React, performance and frameworks), README/license and vanilla/React examples.
The strongest methods are labelled timelines, scoped context/revert ownership,
explicit initial rendering, measured scroll geometry and framework lifecycle.
They implement a chosen scene; they do not choose a strong visual concept.

The source contains concrete defects that rule out a blind import:

- Its horizontal-scroll example uses undefined `Max.max`, passes a pixel
  expression to `xPercent`, and omits `scrub` despite describing scroll-linked
  movement. Mirket specifies measured pixel travel, zero-overflow handling and
  explicit progress ownership.
- Its `refreshPriority` explanation reverses the order. Higher values refresh
  earlier according to the [official ScrollTrigger documentation](https://gsap.com/docs/v3/Plugins/ScrollTrigger/).
- Its React advice says `contextSafe` callbacks become no-ops after unmount.
  The [official lifecycle documentation](https://gsap.com/resources/React/)
  describes recording GSAP objects for context cleanup and separately removing
  listeners. Timers and stale asynchronous work still need their own owner.

Eight `open-design-gsap-*` mirror records become one `gsap-reference` record,
with related original-source paths. Corrected local guidance ships under
`advanced-motion/references/gsap-integration.md`. No default GSAP recommendation,
global teardown, runtime dependency or performance guarantee is adopted.
MIT notice: GreenSock, 2026.

## 2. Cathryn Lavery Diagram Design

[Reviewed revision](https://github.com/cathrynlavery/diagram-design/tree/550fd66edff058bc73056c3416b76d69cfe865ba):
`550fd66edff058bc73056c3416b76d69cfe865ba`.

Examined the main skill, semantic-patterns, architecture-delta and animation
references, geometry/self-check source and third-party notices. Its most useful
method is semantic pattern before visual grammar: queues communicate capacity
behavior, policy views distinguish failed/skipped/not reached, transformations
retain source identity, and architecture deltas preserve entity identity.

The new graphic-design diagram reference carries those methods, source-to-element
correspondence when simplifying, connector meaning and a misleading failure-case
trace. Added, removed, changed, moved and rewired remain distinct. Kroki still
owns supported declarative graph layouts. Animation expresses actual events and
retains a complete static interpretation.

Fixed palette/type choices, node/arrow/animation budgets, diagonal connector bans,
required questionnaires and controller conventions are excluded. These can be a
house style, but are not general semantic requirements. Scripts and icon/font/
template bundles are not imported. Selected methods retain Cathryn Lavery's 2025
MIT notice; third-party asset permissions remain separate.

## 3. ibelick Motion Primitives

[Reviewed revision](https://github.com/ibelick/motion-primitives/tree/120f64f6ca60348e251f929e9c81f11ccbe45eda):
`120f64f6ca60348e251f929e9c81f11ccbe45eda`.

Examined package/README, component inventory and transition-panel, text-effect
and morphing-dialog implementations. Useful techniques include presence
transitions, shared visual identity and text staging. A beautiful demo does not
establish the receiving application's semantic control contract.

The default morphing-dialog trigger accepts an optional ref prop while focus
restoration reads the provider's separately created ref; the default trigger
does not attach that provider ref. Character splitting uses UTF-16 code units,
which can damage emoji/combining sequences. The reviewed components do not
establish a complete reduced-motion policy. The existing interaction reference
now addresses trigger identity, rapid reopen, language-aware segmentation,
accessible complete text and stable panel identity.

The README claims MIT and links `LICENSE.md`, absent from the tracked revision.
No component code is copied under an assumed license. Future component admission
must resolve permission and verify actual behavior. This review does not claim
browser execution of those demos.

## 4. LottieFiles Motion Design Skill

[Reviewed revision](https://github.com/LottieFiles/motion-design-skill/tree/f9a8a041b85185ee4881b3471d3415e939aac772):
`f9a8a041b85185ee4881b3471d3415e939aac772`.

Examined skill/director structure, choreography and narrative references alongside
previously selected personality, animation-principle and timing material. This is
already Mirket's pinned revision. Useful vocabulary includes apparent weight,
preparation, follow-through, focal staging and rest.

The revision makes diagnosis more discriminating: isolate the subject from
camera/decorative movement, inspect meaningful poses, vary one timing or
construction hypothesis, then restore the composition. A weak resting drawing
needs an art correction; elaborate easing cannot supply it.

Industry personality defaults, universal emotional/color claims, required motion
layers and fixed phase/concurrency percentages remain excluded. Essential
controls cannot wait for narrative completion. This does not add Lottie export;
Mirket's actual video route remains HyperFrames. MIT notice: LottieFiles, 2025.

## 5. DavidHDev Canvas UI

[Reviewed revision](https://github.com/DavidHDev/canvas-ui/tree/a04a99b235299ee79a123a16ccb2e6cf9da74b6b):
`a04a99b235299ee79a123a16ccb2e6cf9da74b6b`.

Examined README/license/package, effect inventory and Liquid's React wrapper and
renderer: initialization, pointer input, visibility, motion preference, resize
and disposal. The interesting architecture retains real HTML semantics while a
GPU effect supplies an optional visual treatment.

Its custom MIT-plus-Commons-Clause license restricts redistribution of the
components themselves, including bundles. Mirket imports no components, shaders,
assets or source excerpts. This admission decision does not claim that ordinary
application use is banned.

Original integration guidance distinguishes HTML-in-canvas capability from
canvas/GPU availability and requires a usable fallback. A concrete probe retains
entered text and the focus contract across failure and recovery: remounting
semantic content can fail even when screenshots match. Resolution, render targets,
suspension and cleanup require inspection. Experimental demos do not establish
ordinary cross-browser support; flags are not a deployment strategy for essential
content.

## 6. marvkr Better Design

[Reviewed revision](https://github.com/marvkr/better-design/tree/e146625f3041ef8089a2013587435fc763dc3b15):
`e146625f3041ef8089a2013587435fc763dc3b15`.

Examined README, skill, registry/component structure, Linear CSS/motion helpers
and modal-transition implementation/tests. Public material includes semantic
theme tokens, reusable UI source and focused behavior tests. The modal tests
address focus entry/return, nested Escape, hidden controls and closing semantics.

The advertised hosted review engine is outside this public repository. The
skill's demand to connect that MCP is not adopted, and reading this repo does
not establish review or installation of that engine. Brand-inspired themes also
do not establish official brand-system equivalence.

The useful adaptation pairs foreground/background roles through real variants
and states, measures actual compositing, and verifies the semantic component
consuming the tokens. No theme pack, brand assets, hosted server or new dependency
is bundled. Public-method adaptation retains Marvin Kaunda's 2026 MIT notice;
linked assets retain their own rights.

## 7. Leonxlnx Taste Skill

[Reviewed revision](https://github.com/Leonxlnx/taste-skill/tree/f624f37a8c650b9cc550253104925efe4c8de246):
`f624f37a8c650b9cc550253104925efe4c8de246`.

Inventoried skill variants and examined the main skill, critique/imagery material,
redesign, minimalist, brutalist, soft and GPT variants, and the inspection helper's actual checks. The useful
advance is subject specificity: objects, materials, rituals, places and outputs
generate the visual idea. Removing generic decoration without supplying visual
substance merely produces an empty wireframe.

Mirket updates its older Taste pin and adapts subject-to-composition reasoning,
claim-to-proof matching and early focal-asset inspection. A conditional noun
substitution test can reveal an absent subject; it does not demand novelty when
an established brand or faithful reproduction governs.

Image/moving-element quotas and helper minimum-motion thresholds are rejected.
So are random direction mandates, universal layout/type bans, compulsory moving
imagery, fake tool execution suggested by the GPT variant, and fabricated realism
such as randomized dates or unsourced data. Four previously installable Taste
preset/redesign entries and their three style-only profiles are removed rather
than retaining contradictory mandates behind an adaptation disclaimer. The
`design-redesign` profile now uses `interface-design`; retired identifiers have
no aliases. Existing projects that selected them must revise their selection
explicitly; this review does not rewrite unrelated project manifests.
Gallery/jury claims remain upstream
reports, not independent Mirket evidence. No helper is run or installed. Selected
adaptation retains the MIT notice.

## 8. Nutlope Hallmark

[Reviewed revision](https://github.com/Nutlope/hallmark/tree/13ac0ec7e148655948100b6396439e481361d690):
`13ac0ec7e148655948100b6396439e481361d690`.

Examined the skill, structural catalog and hero-enrichment material. This is
already Mirket's pinned revision. Useful visual variety includes reading
structure: specimen, index, comparison, editorial argument, workbench or sequence.
A palette swap on one hero shell does not explore those alternatives.

Mirket retains that breadth as options derived from the reading task, alongside
truthful claims and visible subject evidence. It does not rotate themes, demand
a fixed number of changed axes, mandate an image provider or accept a self-awarded
score. Exact hero-height and entrance-duration prescriptions remain examples.
Effect recommendations also vary within the source, so an effect name cannot
serve as a consistent quality standard. No templates/assets/provider integration
are imported. MIT notice retained.

## 9. Multica Karpathy Skills

[Reviewed revision](https://github.com/multica-ai/andrej-karpathy-skills/tree/2c606141936f1eeef17fa3043a72095b4765b9c2):
`2c606141936f1eeef17fa3043a72095b4765b9c2`.

Read README, CLAUDE.md and skill body; inventoried nine tracked files. Its four
principles—surface assumptions, keep solutions simple, control scope and define
verifiable outcomes—are already covered by Mirket's engineering/planning owners.
They do not supply a missing illustration or motion method.

No duplicate skill or global layer is admitted. Blanket stopping on uncertainty
and narrow-edit prescriptions also need user scope to govern; this task explicitly
authorizes substantive revision. The README installation target names another
fork, so it is not a command to execute during review. This is Multica's adaptation
of observations, not a claim of Karpathy authorship or endorsement. MIT is declared
in prose/frontmatter but no standalone LICENSE file was present. No source content
is redistributed.

## 10. Anthropic Knowledge Work Plugins

[Reviewed revision](https://github.com/anthropics/knowledge-work-plugins/tree/33eeda1e8fd856455b05479d13a0f85b5f803744):
`33eeda1e8fd856455b05479d13a0f85b5f803744`.

Inventoried the marketplace, plugin manifests/licenses and connector boundaries.
Examined all seven design skill bodies and selected adjacent data-visualization/
engineering review guidance. Other business-domain plugins are not needed to
repair the visual production boundary and are not imported wholesale.

The useful methods are critique calibrated to exploration/refinement/delivery,
state-specific handoff and separating observations from interpretation. Mirket
adapts these with real states, source owners, measurements and explicit unknowns.
A screenshot cannot establish hidden behavior or precise token values. Connector
templates and implied ticket/publication actions do not grant authority.

The accessibility skill groups a 44-pixel target rule under a WCAG 2.1 AA review,
but [WCAG 2.1 SC 2.5.5 is Level AAA](https://www.w3.org/WAI/WCAG21/Understanding/target-size.html).
This compliance table, unsupported scanner-coverage percentages and unqualified
chart-selection shortcuts are excluded. Selected design methods use the root
Apache-2.0 license, with full text and explicit modification notices. Separate
plugin licenses do not become one blanket permission. No upstream NOTICE file
was found in the reviewed inventory.

## What would establish improvement

The earlier harness already separated render validity from aesthetic acceptance,
handled lifecycle and supported multiple media. This revision adds decisions
before implementation expands: derive an idea from subject evidence, prove the
real focal asset at use size, inspect a subject action separately, and diagnose
concept/construction/composition/timing failures before adding polish. It also
consolidates discovery and retains concrete component/diagram failure probes.

Catalog validation and disposable installation establish that references/notices
reach intended owners. Behavioral forward-review can expose instruction conflicts.
Neither proves better visual output. A future quality study would need varied
illustration briefs, the then-current harness, comparable conditions, preserved
failed trials and separate craft, functional and human judgments. Such a study
requires an explicit evaluation task; routine production and gate verification
do not launch it. Source admission alone is never reported as high visual quality.

## Verification of this revision

On macOS arm64, the Mirket 1.0.4 development build passed formatting, Clippy and
all 344 standard tests across 24 suites; four opt-in tests remained ignored.
Catalog validation passed with 347 entries and 63 profiles. Five affected skill
entry points passed the skill validator.

A disposable both-host setup and selected project sync reported healthy through
both doctors. All 74 source files in the four selected design skill payloads
matched the working tree across Codex and Claude destinations; applicable source
pins were present in their notice payloads. Six changed references were retrieved
exactly through the installed MCP server. An independent reasoning review covered
five contrasting cases and led to explicit static-proof and diagram-routing
clarifications. It did not create or grade visual artifacts.

These results establish source/payload consistency and tested harness behavior.
Native desktop-client activation, upstream demo execution and improved visual
quality were not measured. The changes are in the development working tree;
the installation checks used disposable homes/projects, not a published release.
