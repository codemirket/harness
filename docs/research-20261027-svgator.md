# SVGator research for vector and motion workflows

Date window: 2025-04-27 through 2026-10-27. This is a primary-source product/method review, not a qualification of SVGator or an endorsement of adding it to Mirket. Raw HTML retrievals and hashes are private under `/Users/nazmiyilmaz/.codex/task-state/harness-remotion-svgator-20261027/svgator/`. I did not open SVGator in browser UI, create an account, run its MCP server, or inspect live examples. The method evidence below comes from SVGator's dated blog posts plus current help/tutorial pages. The latter are explicitly treated as undated background because their pages do not expose a publication date.

The SVGator editor/MCP is a closed SaaS service, but the public `SVGator/skills` repository is different: its README and the pinned skill frontmatter state MIT. MIT applies to that repository's skill files under its license terms; it does not license SVGator client artwork, gallery examples, fonts, templates, or third-party assets. The recommendations below abstract workflow decisions only; no source prose or code was copied, and they do not add a dependency, an MCP route, a remote renderer, or a shared asset.

## Dated primary source ledger

Each summary is intentionally short and states what the source can and cannot prove.

| ID | Primary source and verified date | What it documents | Claim strength |
|---|---|---|---|
| S1 | [Best practices for SVGator MCP](https://www.svgator.com/blog/best-practices-for-svgator-mcp/), 5 Aug 2026 | SVGator says its MCP can list/read/create/edit/export projects; its suggested loop is inspect/read, make one validated edit, request a rendered preview, then export. It recommends named elements, explicit start/end/duration/easing/loop instructions, duplication before risky experiments, and saving expiring download links. It also lists format constraints. | Strong for the vendor's documented contract and limits; unverified for this host or any claimed visual quality. |
| S2 | [What's new in SVGator: Five editor updates](https://www.svgator.com/blog/whats-new-in-svgator/), 21 Aug 2026 | The editor added per-group links, animated-text alignment, image replacement without rebuilding layer order, and project/animation library saving. The post distinguishes genuinely new capabilities from workflow shortcuts. | Strong for the dated feature description; not evidence that a project is editable or that the update is available under every plan/host. |
| S3 | [How to create SVG text animations](https://www.svgator.com/blog/how-to-create-svg-text-animations/), 16 Apr 2026 | The article converts text to individual paths for stroke/reveal/stagger work, names CSS/JavaScript/no-code tradeoffs, and recommends its JavaScript path for richer animation and triggers; this is not a general browser limitation. It calls out accessibility/limitations sections but does not establish a screen-reader result. | Strong for the documented authoring/export distinction; vendor guidance, not independent performance or accessibility evidence. |
| S4 | [140 SVG Animation Examples](https://www.svgator.com/blog/cool-svg-animation-examples-to-inspire/), 20 Apr 2026 | SVGator's categorized gallery distinguishes simple feedback, line/path work, morphs, UI states and larger storytelling. Its key decision is CSS for simple one-shot motion and JavaScript for pause/reverse/replay/scroll control, with a tradeoff of runtime/file weight. | Useful taxonomy and vendor recommendation; inspirational gallery is not independent quality proof and should not be mined for assets. |
| S5 | [SVGator skills repository](https://github.com/SVGator/skills/tree/190dced40487286de1336c2200f6c16bcdf5c9b8/svgator-motion-design), [pinned README license](https://github.com/SVGator/skills/blob/190dced40487286de1336c2200f6c16bcdf5c9b8/README.md), commit `190dced40487286de1336c2200f6c16bcdf5c9b8`, 29 Sep 2026 (`svgator-motion-design/SKILL.md` declares MIT) | The pinned public skill separates still-frame design, motion direction, and interaction for standalone SVG assets. Its references cover composition, form/type, choreography, context adaptation, property meaning, timing, output types, interactivity and quality review. | Strong primary evidence of the published craft method and license at that revision; not evidence that the companion SaaS/MCP is connected or that its numeric heuristics are universal. |

For reproducibility, the inspected GitHub blob SHAs (40-character Git object IDs returned by the GitHub tree API) at that commit were: `SKILL.md` `0ecf1c2b4f021803602490f684be9b8712a53604`; `design/composition.md` `dc9899ac09095cd3a9611909313eba53e7e7505c`; `design/form-and-type.md` `20fd8438afffdf6fe31b744cfe5a260879ef377f`; `director/choreography.md` `cc7c24a47f7e4e56a37f5c8f844dd5fd3e4f71bd`; `director/context-adaptation.md` `ca1afd929b4e548752f9eb81d27dc29adcd89a96`; `director/interactivity.md` `3a0d93229d77ee6a9cc71ecf64d67d2913b6082a`; `reference/interactivity.md` `55274ebf8cbacd0be31e5fff3c7f4d80a56fc18b`; `reference/output-types.md` `c980d43697fc0f642e00cb70c738edd175401052`; `reference/property-meaning.md` `9618bf0320c57b6992dea50f8d20fa1ba12a8384`; `reference/quality-checklist.md` `9e64ccc8720b8456813034f8c0728a169ee95415`; and `reference/timing-easing.md` `33991ba428023cd2360d29fcc6e38f6d9bc68dac`. Each can be opened under the same full commit URL. The private retrieval SHA-256 content hashes are recorded separately in the raw directory and are not presented as Git blob IDs.

## Current undated background sources

These pages were read for concrete limits, but not used as dated evidence: [supported file formats](https://www.svgator.com/help/getting-started/what-file-format-does-svgator-support), [Lottie support](https://www.svgator.com/help/export-and-file-formats/lottie-support-in-svgator), [interactive SVG](https://www.svgator.com/help/animation-and-interactivity/how-to-create-an-interactive-svg-animation), [SVG export settings](https://www.svgator.com/help/export-and-file-formats/svg-export-settings), [origin/anchor point](https://www.svgator.com/help/animation-and-interactivity/how-to-change-an-elements-origin-point), and [quick interface tour](https://www.svgator.com/tutorials/quick-tour-of-the-svgator-interface). They document useful constraints: SVG imports must be static; animated SVG re-import loses animation; Lottie import is beta; images have fewer animators and increase project/export size; its click trigger requires its JavaScript export; scroll-into-view requires inline SVG and is per-animation rather than per-element; CSS has a smaller trigger/animator subset; origin placement changes transform behavior; and the editor represents elements in a named tree with timeline/keyframes. Recheck these pages before relying on a format-specific detail because no revision/date is exposed.

## Transferable methods (and limits)

### 1. Treat the editable scene as a named, inspectable hierarchy

S1 says to inspect canvas/timeline/export settings, read a property before writing, address elements by stable name/ID rather than layer position, and preview after each single edit. S2 shows why this matters: groups can own links and image replacement can preserve layer order. The transferable method is a project-owned layer contract: meaningful group names, stable IDs, explicit owner/property, and one reversible change before the next. For vector art this protects editable geometry and hierarchy; for motion it prevents an animation edit from silently targeting the wrong layer.

Counterexample: stable IDs do not establish semantic accessibility, and a vendor validation receipt does not show the intended route or pixels. Preserve a duplicate before a destructive experiment, but keep the repair budget shared.

### 2. Put pivots/anchors at the physical joint or center of the intended motion

Current SVGator help/tutorials explain that origin placement changes Position, Scale, Rotate and Skew results; examples place a bounce origin at the base, a character joint at the joint, and a watch hand at the dial center. The useful abstraction is to decide the physical pivot before keyframing and record it with the layer/geometry owner. This is a precise addition to the existing `vector-art` and `motion-direction` guidance when a figure, mechanism or logo needs believable rotation—not a universal center-point rule.

Limit: the sources are undated vendor tutorials and do not prove that a converted/exported target preserves the same pivot. Verify the decoded/player result in the actual consuming context.

### 3. Make timing, easing, stagger and loop state named controls, not adjectives

S1's dated MCP guidance asks for concrete start/end/duration/order and explicit loop count; S3 describes per-segment easing and staggered letter paths. The current tutorials show that a custom curve or step function changes the meaning of a transition, while the dated articles do not claim that a bounce or “polished” default is correct. Transfer the method as a small motion contract: element/beat, property, start/end, duration, delay/stagger, easing, loop/reverse/hold behavior, and expected resting state. This directly complements `motion-direction`'s named timing and `ui-motion`'s interruption contract.

Contraindication: numeric examples in tutorials are demonstrations, not product-wide thresholds. Do not turn a 300 ms or 50 ms stagger example into a rule.

### 4. Choose CSS/JS/export target from the required behavior

S3/S4 and current SVGator export help converge on a practical *SVGator export* matrix: within that exporter, CSS is presented as suitable for simple one-shot/load/hover effects, while its JavaScript export is recommended for richer animators, path-length work, click/scroll/programmatic control, pause/reverse/replay and per-player behavior. This is not a general claim that browser CSS, Web Animations or project JavaScript cannot implement those behaviors. Current SVGator help adds that its scroll-into-view option requires inline SVG (not `<img>`/`<object>`) and its built-in events apply to the animation rather than individual elements. This is a useful preflight for `svg-lottie`, `ui-motion`, `scroll-motion`, and `interface-composition`: choose the consuming target and trigger contract first, then test the actual embed/player.

Limit: the source is a vendor compatibility description, not a browser matrix or Mirket capability receipt. Mirket should not add an SVGator runtime merely to satisfy a feature that Codex/project code already handles.

### 5. Treat export conversion as a feature subset, not a promise of parity

The current Lottie help labels support beta and enumerates concrete loss: animated skew cannot export, Step Start has no native equivalent, some polystar/polygon rotations/points, masks/mattes/effects and modifiers are unsupported, and auto-orient is unavailable. The import help separately warns that animated SVG re-import becomes the first keyframe and image assets reduce available animators/increase size. Transferable method: inventory geometry, masks, anchors, interpolation, effects, text, images and triggers against the target subset before editing; retain the source; compare representative start/action/rest frames in the intended player; and record any flattening or fallback.

This confirms rather than expands the existing `svg-lottie` skill. The vendor table may change, is undated, and is not a guarantee for other Lottie players.

### 6. Use an AI/MCP context contract around an editable project

S1's dated article says the assistant can read project structure, make one validated edit, return a rendered preview, duplicate/organize projects and export. The practical method is narrower than “AI makes animation”: provide the intended route/project, named layers, visual purpose, timing/easing, trigger, target format and acceptance state; read the current value; make one change; inspect the preview; then continue or revert. Keep the editable project and downloaded artifact, because the article says links expire. Treat server-side credential/OAuth, validation, and “clean editable output” as vendor claims until an authorized host trial proves them.

Mirket has no SVGator MCP binding in `skills/catalog.json`, `src/native_skills.rs`, `src/skill_sync.rs`, or `src/server.rs`. Therefore this method is guidance for a future explicitly qualified external integration, not a promise that Mirket or Codex can invoke SVGator today. It must not override Codex ownership of agents, permissions, browsing, shell or Git.

### 7. Start from a still frame and a semantic motion channel, not a generic bounce

S5's `design/composition.md` asks for a frozen-frame hierarchy, clear hero, grouped negative space, asymmetry and optical correction before motion; `design/form-and-type.md` asks for one shape language and deliberate material/lettering choices. Its `reference/property-meaning.md` then maps channels to meaning: a stroke draw-on can communicate construction, a path morph transformation, scale importance, and opacity presence—but its strongest claim is the decision question, not the prescribed aesthetic. S4's dated gallery broadens the subject set to line/path drawings, data visualization, UI states, characters, text, morphs and hero storytelling. Transfer: inspect the still at target size, name the subject/material/type rationale, and choose the channel that explains the subject. A generic bounce is justified only when it carries that meaning.

Reject S5's “AI-slop” label, “never linear,” “two channels is the sweet spot,” and other style laws as universal gates. They are authored taste heuristics. Mirket's existing `vector-art` and `art-direction` evidence/rights/inspection rules are the safer owner.

Rights limit: MIT covers the public skill repository, not the gallery/tutorial images or any client artwork. No image, sample SVG, font, character, or recognizable visual treatment was copied.

### 8. Make choreography and interaction answer the context

S5's `director/choreography.md` gives a useful structure: a hero leads, supporting elements answer with counter-motion/follow-through, and setup → action → resolution should remain readable. `director/context-adaptation.md` asks how size, frequency and export target change density and amplitude; `director/interactivity.md` adds an especially concrete test: the resting state should hint at the response, the input should be acknowledged immediately, and a forward timeline should have a coherent reverse/freeze/restart behavior. Its `reference/interactivity.md` distinguishes whole-animation export triggers from a Player API and explicitly limits per-element event binding in the MCP data model.

Transfer: decide whether motion is authored narrative or user feedback, expose a resting affordance, define interruption/return behavior, and inspect the consuming context. Preserve `scroll-motion`'s requirement that scrubbing remains readable at arbitrary progress and `ui-motion`'s lifecycle/focus rules. Treat S5's durations, 1/3 rules, 20–30% counter-motion and 50–150ms offsets as examples to test—not hard product thresholds. Offline video has no hover/scroll state; do not invent one.

## Capability comparison with Mirket

| Need | Existing Mirket owner | SVGator method that is useful | What remains unqualified / not added |
|---|---|---|---|
| Editable vector geometry, groups, IDs, rights and inspected SVG | `vector-art` + qualified resvg/SVGO | Named hierarchy and pivot-first authoring | No SVGator editor/runtime; SVGator assets remain external and rights-bound. |
| Interactive UI motion and lifecycle | `ui-motion` + project CSS/WA/API | CSS-versus-JS trigger preflight; one property owner; inspect actual state | No vendor player; do not infer browser support from SVGator help. |
| Scroll-linked scenes | `scroll-motion` | Inline-SVG/trigger limitation and event ownership are a useful integration check | Existing project scroll tools remain owner; no SVGator scroll route. |
| SVG/Lottie asset conversion | `svg-lottie` | Feature-subset inventory and representative frame parity | SVGator Lottie is explicitly beta in current help; no parity claim. |
| Authored/exported video | `video-production` via private HyperFrames/FFmpeg | Separate animated-source edit from cloud export settings; verify decoded output | SVGator cloud render, expiring links and codecs are not a qualified Mirket route. |
| AI-directed editable animation | Codex owns model/permissions; no Mirket route | Context contract: named elements, one edit, preview, preserve source | No SVGator MCP tool appears in the repository capability map; OAuth/validation claims untested. |

## Concrete gap map

1. **Keep:** `vector-art` already requires editable geometry, meaningful grouping, rights, render-before/after inspection, and a consuming-context check. Add no SVGator dependency. The named-layer/pivot contract is a useful conditional technique for future vector/motion tasks.
2. **Keep:** `motion-direction` already requires explicit trigger/timing/easing/interrupt/reduced-motion decisions and a playable specimen. S1/S3 support its named-control wording; tutorial numbers should remain examples only.
3. **Keep:** `svg-lottie` already requires target-player inventory, tested feature subset, representative frame parity and source preservation. SVGator's beta table is evidence for why that gate matters, not a new promise.
4. **Keep:** `video-production` already requires qualified private rendering and decoded frame/audio/caption QA. SVGator's cloud video export is a comparison/rights concern, not a replacement route.
5. **No new tool pairing:** the repository contains no SVGator MCP/server entry, and this research did not authorize credentials, SaaS connection, dependency admission, or a browser-based qualification. If a future product decision requires SVGator, it needs an explicit capability contract covering project identity, permissions, read-before-write, rendered preview, export/download retention, format limits, and actual consuming-player verification.

## Evidence and unresolved limits

- All dated claims above are vendor-authored primary material. They establish documented behavior and constraints, not independent quality, accessibility, performance, licensing, or support-level guarantees.
- Current help/tutorial pages are useful background but undated. Recheck before pinning an exact compatibility matrix.
- No live SVGator editor/MCP connection, export, or downloaded asset was exercised in this SVGator source research. Saved HTML retrievals are evidence of page content only; they do not prove an operation works. Separate live public-example inspection is recorded in [the root-owned motion-authoring comparison](research-20261027-motion-authoring-comparison.md).
- No upstream text, code, vector, image, font, or template was copied. SVGator's closed SaaS rights and any imported third-party asset rights remain unresolved for a future integration.
