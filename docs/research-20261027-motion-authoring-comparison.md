# What Remotion and SVGator get right

Research: 2026-10-27, macOS ARM64, repository HEAD
`8d4f3923822ee67317863a97cd60d69f737d06f8`. Dated source window:
2025-04-27–2026-10-27. Current undated documentation is background, not evidence
of recent adoption. This is research and a proposed implementation sequence;
it does not qualify a new renderer or change installed skills.

Detailed primary-source inventories, exact skill pins, licensing boundaries,
and limitations: [Remotion](research-20261027-remotion.md) and
[SVGator](research-20261027-svgator.md).

## The useful distinction

Remotion makes the relationship between source, composition, time and preview
explicit. SVGator makes the relationship between editable geometry, motion
properties and interaction explicit. My inference is that these reduce the
cost of a precise revision. Neither source establishes that its engine alone
produces good art direction.

For our harness, retain Hallmark/Taste for interface direction and HyperFrames
for the qualified video route. Build on the existing visual review rules with
better authoring feedback and reusable, original examples. Another renderer
is not supported by this research as the necessary solution.

## Craft and workflow lessons

| Lesson | Reference evidence | Application to our harness |
| --- | --- | --- |
| A motion design system should be usable, not just described | Remotion's [design-system documentation](https://www.remotion.dev/docs/design-systems) and public brand Studio organize parameterized animated assets | Project-owned examples with named controls, editable sources, preview and export evidence; start with a few real needs rather than a large effect catalog |
| Review a precise moment | Remotion's frame/sequence model and the live probe below expose finite time and source context | A named beat and frame should identify the visible defect and its source owner; seconds/frame units must be explicit |
| Preserve product identity | The [Presscut prompt](https://www.remotion.dev/prompts/product-demo-for-presscut), displayed as eight months old, asks for the actual application's interface and a customer-demo narrative | Reuse permitted project components/assets and compare against the real product; reduce invented dashboards and unmotivated transitions |
| Give illustration a strong still composition | SVGator's pinned September 2026 craft skill separates artwork, motion and interaction | Inspect hierarchy, silhouette, negative space and lettering before animation; keep supplied artwork intact when the request is only to animate it |
| Make motion explain the subject | SVGator's skill distinguishes motion channels and output contexts | Choose a draw-on, transformation, emphasis or reveal for a stated communication purpose; avoid accumulating unrelated effects |
| Make small revisions identifiable | SVGator's [August 2026 MCP guidance](https://www.svgator.com/blog/best-practices-for-svgator-mcp/) uses named elements, exact property shapes and read-before-write | Address a stable element ID, property and beat; distinguish a base value from the animation that overrides it; retain a reversible change |
| Author for the destination | SVGator's current export help describes feature subsets and embedding constraints | Keep existing `svg-lottie` target/player checks; do not infer browser capabilities or export parity from one vendor's settings |

Do not import SVGator's aesthetic prohibitions as universal rules. Its pinned
skill includes categorical preferences about colors, easing and channel counts.
Those require context and a visual judgment. Likewise, Remotion-specific React
or CSS restrictions are not the HyperFrames authoring contract.

## Direct public-product observations

The lead used Codex's in-app browser on 2026-10-27. These observations are
separate from the two source-research passes.

### Remotion public brand Studio

URL: <https://www.remotion.dev/brand>.

1. Discovered the page's WebMCP surface, then actually called composition and
   sequence inspection, composition selection, frame seeking and playback-state
   inspection. Tool discovery alone was not counted as success.
2. Selected `AnimatedLogo`: the returned composition was 1080×1080, 30 fps,
   450 frames. The sequence covered frames 0–449.
3. Viewed frame 0: an empty transparent checkerboard. Sought frame 45 and
   viewed the blue layered triangular mark and wordmark. Playback state
   reported frame 45, paused, with looping enabled.
4. The UI exposed a theme form, timeline and source locations including
   `Root.tsx:348` and `AnimatedLogo.tsx:18`. This is a concrete example of
   connecting the visible result to editable source and composition settings.

This verified public preview navigation and inspection. It did not verify
render/export parity, playback smoothness, a local Remotion installation,
accessibility, or improved aesthetic quality. Screenshots were viewed live
through the browser tool; no saved frame artifacts were retained. Do not cite
this as saved-pixel qualification of the harness.

### SVGator designer example

URL: [William Benjamin Powell case study](https://www.svgator.com/blog/animation-in-half-the-time-a-freelance-designers-story/),
20 May 2026.

Viewed two live states of the embedded DON’T STOP typography. Bold red and gold
lettering, dark extruded outlines/shadows and a white ground established the
graphic. The relative word heights changed between observations, directing
attention through coordinated deformation. My visual interpretation is that
the lettering and composition already carry the piece; motion develops that
idea instead of supplying all its interest.

These were live observations, not controlled time samples or saved exports.
No SVGator editor, authenticated MCP, export operation or local asset was
exercised. The vendor's faster-turnaround claims remain a designer's reported
experience, not measured performance or causal evidence.

## Counterevidence and popularity limits

A [recent positive Remotion discussion](https://www.reddit.com/r/SaaS/comments/1vdha3r/codex_remotion_is_crazyy_made_this_launch_video/)
describes supplying application code, a reference video, assets and voiceover.
It is useful workflow testimony, not proof that a short generic prompt gives
the same result. The thread also contains dissatisfied users and inconsistent
one-shot/two-shot wording. Do not inherit claims that using the real components
eliminates screenshot review or that screenshots always cause hallucination.

A [contrasting recent discussion](https://www.reddit.com/r/SaaS/comments/1rl0dml/anyone_actually_using_remotion_for_saas_feature/)
reports runnable compositions with excessive movement and poor transitions
despite codebase access and visual references. These threads were displayed
as three and seven months old at review. Their anecdotes support testing
direction and feedback separately from engine success; they are not an
independent benchmark or a representative popularity sample. Gallery counters
are not verified social likes. No new claim about X popularity is made here.

## Proposed implementation sequence and falsifiable checks

The present video MCP surface in `src/server.rs` exposes finite prepare, start,
status, cancel and result jobs. Existing skills already require visual samples,
decoded output, property ownership, local assets and shared repair budgets.
The following are additions to authoring usability and fixtures, not a claim
that those safeguards are absent.

| Priority | Smallest useful addition | Owners and boundaries | Required evidence before adoption |
| --- | --- | --- | --- |
| 1 | A project-owned seekable specimen with named beats and source/element context | Codex opens and inspects its ordinary browser preview; HyperFrames remains the finite export producer. First prove the pinned producer can share the relevant timing state; do not create a second clock by assumption | Seek forward/backward/repeat, change one control, and compare matching preview/export states. Include a deliberately unregistered animation that must fail parity |
| 2 | Focused original motion examples: a product interaction, logo/illustration gesture, and typographic beat | Existing `motion-direction`, `vector-art`, `ui-motion` and `video-production` resources; project-owned sources and rights-cleared assets | Still composition plus normal-speed playback, pressure sizes, loop seam or resting state, and a held-out brief requiring adaptation rather than copying |
| 3 | A compact editable motion contract | Stable element/group ID, pivot where relevant, property owner, explicit timing units, expected rest and target format; no new general scene DSL | Reordered layers still target the same object; keyframed/base-value conflict is exposed; overlapping property owners and seconds/frame mistakes produce distinguishable failures |
| 4 | Beat-based review evidence | Extend the existing evidence packet with named scene boundaries and expected states, sharing the original task budget | Inspect saved previews and decoded exports at the same states; include captions near boundaries, final frame, loop/reversal and post-repair regressions |

Run one held-out product demo and one vector illustration loop with frozen
inputs, equal budgets and a matched current-workflow baseline. Retain first
artifacts, independent findings, repairs, final artifacts and remaining defects.
Use a capable independent visual reviewer. Compare communication, product
fidelity, composition and motion readability as well as successful rendering.
Do not declare an improvement from hashes, receipts or author self-report.

## Reuse and qualification limits

Remotion core uses a custom license; the inspected skills repository does not
declare a repository license. SVGator's service is proprietary, while its
pinned public motion-design skill declares MIT. Exact pins and rights findings
are in the companion reports. Learn methods without copying unlicensed prose,
brand assets or gallery artwork. Any later licensed reuse still needs accurate
catalog provenance and repository-owned refresh tooling.

This turn changes research documents only. No engine, dependency, catalog,
installation, production skill or live `~/.mirket` state was changed. No Rust
compilation was needed. Windows qualification and the waived Codex Desktop GUI
check remain unverified; these public browser observations do not satisfy them.
