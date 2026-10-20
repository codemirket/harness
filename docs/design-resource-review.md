# Design capability review and adoption proposals

Reviewed 19 October 2026. This extends the [earlier source review](design-source-review.md).

## Decision

Broaden Mirket around **better subject construction, semantic components,
explicit motion-engine selection, typography proof and reusable video scenes**.
Do not install an effect collection as the solution to all five problems.

The strongest candidates are Anatomy for constructed explanatory illustrations,
Radix for React interaction semantics, Motion for React layout/presence work,
Anime.js for framework-independent SVG/DOM work, and Remotion with selected
Remocn components for reusable React video production. GSAP remains a strong
choice for elaborate scroll/timeline work. These are task-dependent alternatives,
not a proposed bundle of competing engines.

Keep the current rendering tools where they fit. Replace their role when an
actual deliverable demonstrates a better route. In particular, evaluate Remotion
against HyperFrames for recurring, data-driven video; retain HyperFrames for
existing HTML compositions unless the comparison warrants migration.

## Review boundary

The review inspected the inventories, README/license evidence and selected
implementation or guidance files from all 17 supplied GitHub repositories.
Refero's resource page was read as text. Exact revisions, selected files and
content digests are in [the source ledger](design-resource-sources.json).
Large repositories were sampled at relevant owners, not audited line by line.
The prompt collection was sampled; its full advertised external gallery was not
reviewed. Collection licenses do not establish rights to every linked asset.

No candidate installer, hook, demo or helper was executed. Recommendations below
are engineering judgments from source review, not comparative benchmarks or
proof that generated designs improve. Runtime verification here covers Mirket's
own changed adapters. The existing ten-tool design stack was inspected through
its registry, locked packages, adapters and tests; this is not an audit of every
transitive npm/Rust dependency or every marketing skill in the catalog.

## What to borrow from each resource

| Resource | Useful improvement and receiving owner | Decision and limits |
| --- | --- | --- |
| [Ian Xiaohei illustrations](https://github.com/helloianneo/ian-xiaohei-illustrations) | Give editorial illustration a paragraph-level shot list: the claim, explanatory relationship, subject's action and placement. Test whether removing the character destroys the explanation. Owner: `graphic-design`, with `marketing-writing` supplying the claim. | Adopt the planning method. Do not make the Xiaohei character, white background, 16:9 format or fixed image count a Mirket default. MIT source; preserve attribution for an eventual adaptation and review asset rights separately. |
| [Anatomy](https://github.com/wheresryan22/anatomy) | Build explanatory objects from attached parts, shared camera coordinates, coherent depth and one model driving geometry/readouts. Test collisions and occlusion during reversal, not only assembled/exploded endpoints. Owners: `graphic-design`, `svg-creation`, `advanced-motion`. | Highest-value craft expansion. Pilot a focused constructed-illustration reference before importing its kit. The audited craft guide's minimum part/path counts and fixed visual grammar are house choices, not quality gates. Numerical audit success does not prove the technical analogy is true. MIT. |
| [Awesome GPT Image 2](https://github.com/YouMind-OpenLab/awesome-gpt-image-2) | Index prompts by purpose, subject, material, camera, layout and required exact text. Convert a selected prompt into a brief with editable copy and inspectable output. Owner: image-production handoff. | Reference/search source only. Save a small selected prompt with its creator/source and observed result; do not embed thousands of examples or accept the collection's capability claims as measured model behavior. Collection declares CC BY 4.0 and acknowledges community rights concerns. |
| [Animate.css](https://github.com/animate-css/animate.css) | Reuse a small vocabulary of class-driven entrances/exits where CSS is sufficient. Distinguish animation time, delay and iteration count. Owner: `motion-design`. | Optional in projects already using it; custom CSS may be smaller for a few effects. Its reduced-motion CSS shortens duration/iterations but retains delay, so it does not guarantee immediate useful content. Retain independent state completion and cancellation. MIT. |
| [Anime.js](https://github.com/juliangarnier/anime) | Use scoped cleanup, media-query rebuilds, timelines and SVG drawing/morphing without a React dependency. Owners: `advanced-motion`, `svg-animation`. | Preferred candidate for a new framework-independent animation need when native APIs are insufficient. Review v4 APIs, not old v3 snippets. Sampled `morphTo` resamples geometry: inspect correspondence and degenerate/very short paths; a successful morph is not guaranteed to preserve shape identity. MIT. |
| [Aseprite](https://github.com/aseprite/aseprite) | Add a genuine pixel-art production lane: indexed palettes, deliberate frame timing, tags, slices, sprite sheets and frame metadata. Owner: asset production. | Optional user-provided tool, only for pixel-art deliverables. Do not bundle its executable into the design image as though it were permissively licensed; the reviewed EULA restricts redistribution and source use. Batch export is useful but does not create the artwork or rig. |
| [Motion](https://github.com/motiondivision/motion) | Reuse React presence, layout projection and gesture integration rather than rebuilding them in every component. Owners: frontend integration, `motion-design`, `page-transitions`. | Strong candidate for React products. Keep stable keys and semantic state separate from exit presence. At the reviewed revision, `useReducedMotion` initializes state from a shared value without subscribing the component to changes; its comment promises more. Add a live preference test or an application-owned reactive subscription. MIT core. |
| [GSAP](https://github.com/greensock/GSAP) | Use labelled timelines, measured ScrollTrigger ranges, scoped context/revert and appropriate SVG/text plugins for complex scenes. Owner: `advanced-motion`. | Keep as an option for requirements that justify it. One engine owns each property/scene. The standard license permits ordinary commercial and AI-generated usage, but is not MIT and includes a visual-builder restriction. Evaluate the actual distribution/use rather than rejecting all AI use. |
| [ScrollReveal](https://github.com/jlmakes/scrollreveal) | Retain its distinction between entry triggers, reset behavior and cleanup. Owner: simple reveal integration. | Do not add as a default. IntersectionObserver plus CSS often covers this narrow task, while a selected existing engine avoids another lifecycle owner. The repository declares GPL-3.0 and offers commercial licensing; resolve the receiving application's terms before adoption. |
| [mo.js](https://github.com/mojs/mojs) | Study short bursts, coordinated radial shapes and expressive finite accents. Owner: procedural motion. | Niche reference; prototype against CSS/SVG or the selected engine before adding another runtime. Verify stop/dispose, background suspension and reduced motion in the consumer. No benchmark here establishes a size or speed advantage. MIT. |
| [GSAP Skills](https://github.com/greensock/gsap-skills) | Keep focused API guidance and examples, with independently verified corrections. Owner: existing `gsap-integration.md`. | Already admitted at the same revision. Do not duplicate it. Reconfirmed the `Max.max` typo, pixel value assigned to `xPercent`, reversed refresh-priority advice and overclaim about `contextSafe`. Mirket's existing reference already corrects these. Skills are MIT; the GSAP runtime has separate terms. |
| [Awesome Persona Distill Skills](https://github.com/xixu-me/awesome-persona-distill-skills) | Extract an evidence-backed brand voice: characteristic phrasing, reasoning habits, examples, exclusions and counterexamples. Owner: `marketing-writing`. | Use as discovery for methodology, not a global persona router. A simulated expert voice is not expertise or customer research. Keep private examples out of shipped guidance; use authorized brand/source material. CC0 applies to the list, not automatically to linked skills. |
| [Remocn](https://github.com/Remocn/remocn) | Borrow scene vocabulary, caption emphasis, explicit visual states and frame-driven transition composition. Owner: video production. | Selected components are candidates only after choosing Remotion. Registry items have transitive local helpers: copying one visible component is not a complete installation. Sampled canvas effects depend on newer HTML-in-canvas APIs and fallbacks. MIT components do not remove Remotion's separate license obligations. |
| [Remotion](https://github.com/remotion-dev/remotion) | Make reusable video scenes functions of frame, configuration and assets; use a scrubbable player, explicit durations and overlap accounting. Owner: encoded video. | Serious alternative to HyperFrames for React-based, parameterized campaigns and recurring videos. Compare identical briefs before replacing the package route. Rendering/React/browser closure and licensing add adoption cost. The reviewed license distinguishes eligible free users from organizations requiring a company license. |
| [Awesome DESIGN.md](https://github.com/voltagent/awesome-design-md) | Study how a design language is expressed through semantic color roles, typography, density, surfaces and component states. Owner: `interface-design`. | Reference material, not brand authority. Sampled Linear/Stripe files are third-party analyses; custom font names and inferred values do not confer font rights or prove a canonical system. Keep Mirket's stronger existing source-to-consumer evidence chain. MIT repository scope is separate from brand assets. |
| [Refero DESIGN.md resources](https://styles.refero.design/design-md/design-md-resources) | Choose references serving comparable users; adapt roles and repeated decisions to the product brief. Owner: design-system reconstruction. | Keep as a live reference source. The page itself distinguishes product requirements from visual rules and captured systems from finished product screens. No bulk gallery/template import or claimed visual review; reuse permission for linked content remains unresolved. |
| [React Bits](https://github.com/DavidHDev/react-bits) | Use as a visual vocabulary and selective implementation study for text, backgrounds and expressive sections. Owner: `advanced-motion`. | Do not bundle the component collection. Its reviewed license restricts redistribution of components/tools themselves. Sampled `AnimatedContent` checks reduced motion once and emits hidden initial content; `SplitText` also starts hidden. Adaptation needs no-script, keyboard, live preference and lifecycle verification. Attractive demos do not supply these guarantees. |
| [Radix Primitives](https://github.com/radix-ui/primitives) | Use established dialog, focus, dismissal, presence and composition behavior under an original visual design. Owners: frontend integration and `motion-design`. | Highest-value React foundation candidate. Animate the primitive's actual state; for JS exits, deliberately coordinate `forceMount`, presence and focus. CSS animation suspension does not automatically cover arbitrary JS timelines. Test rapid close/reopen, nested portals, Escape and focus return. MIT. |

## Improvements to implement next, in order

The concrete source findings can be checked in the pinned
[Motion preference hook](https://github.com/motiondivision/motion/blob/c5fab238ab4f8d8132d1cb05d6607439b3904058/packages/framer-motion/src/utils/reduced-motion/use-reduced-motion.ts),
[React Bits entrance component](https://github.com/DavidHDev/react-bits/blob/e12c22192ac7b4b452c5e2feb9b586d74765d696/src/content/Animations/AnimatedContent/AnimatedContent.jsx),
[Animate.css base rules](https://github.com/animate-css/animate.css/blob/3f8ab233dbbd9d2fe577528d2296382954be3d1a/source/_base.css),
and the [earlier GSAP corrections](design-source-review.md#1-greensock-gsap-skills).

1. **Add an outcome-based engine decision reference.** For an existing product,
   keep its working stack. For a new need: CSS/WAAPI for ordinary finite effects;
   Motion for React layout/presence/gestures; Anime.js for framework-independent
   SVG/DOM work; GSAP for elaborate timelines and scroll relationships. Radix
   owns component semantics rather than competing for animation ownership.
   Acceptance: the same interrupted dialog/list task works with one state owner,
   two instances, live reduced motion and teardown. No engine is a style choice.

2. **Add constructed illustration craft to existing owners.** Use an Anatomy-led
   pilot and an editorial-metaphor pilot: one real mechanism and one conceptual
   process. Carry attachments, occlusion, material and source-to-visual meaning.
   Acceptance: understandable at placement size, technically reviewed, and still
   coherent during reversal. Do not require the same isometric look everywhere.

3. **Treat typography as tested output.** Maintain a small script/format corpus:
   Turkish diacritics, Arabic mixed with numbers, combining characters, emoji,
   multiline display text and condensed narrow layouts. Record font bytes,
   weight/style, fallback and embedding rights. Compare actual exported glyphs,
   line breaks and crop safety in each renderer. Split animated text by grapheme
   and preserve a complete accessible reading. The pinned Satori has HarfBuzz and
   WOFF2 support; avoid obsolete blanket claims that it cannot shape Arabic, while
   retaining its documented bidi limitations.

4. **Pilot reusable video scenes before changing the renderer.** Build the same
   captioned product film in the existing HyperFrames route and a separately
   authorized Remotion project. Include a data change, a late font, portrait
   reframing, audio, exact last frame and a nonsequential seek. Compare revision
   effort, render resource use, reproducibility and exported quality. Remocn is
   an accelerator after this decision, not a reason to make it in advance.

5. **Connect marketing claims to visual proof.** For each campaign beat, bind
   audience, claim, supporting product evidence, visual action and next step.
   Hold that meaning across poster, landing page, short film and narrow crop.
   Derive voice from authorized examples with counterexamples. Test persuasion
   with audience evidence; stylistic resemblance and generated personas do not
   establish conversion.

6. **Strengthen DESIGN.md rather than multiplying theme skills.** Extend the
   existing evidence document with relevant typography, motion and asset rules,
   including state and responsive exceptions. Pair exact token declarations with
   actual consuming components. Use brand-analysis collections to discover
   alternatives, then write the receiving product's own rules.

7. **Use medium-aware targets and deliverable budgets. Implemented.** The router
   accepts a native DOM/SVG/pixel/diagram/typeset/editor specimen with a suitability
   reason, retaining image targets and legacy fallback. Targets still require
   inspected rendered evidence. Static/vector/diagram work gets 15 minutes and
   two final repairs, interfaces/paged work 30 minutes and three, motion 45 minutes
   and four. Combined deliverables share the largest limit. These are scheduling
   bounds, not measured performance promises. See the [current contract](visual-quality.md).

## Current third-party stack audit

| Existing tool | Assessment of current usage | Action or remaining gap |
| --- | --- | --- |
| Style Dictionary 5.7.0 | Useful canonical token compiler with output hashes and drift checking. | **Fixed:** cache generations bind input bytes, the compiler/format module and locked dependencies. A changed-recipe regression verifies regenerated output for unchanged tokens, while unchanged recipes reuse verified output. Penpot accepts only a subset of the emitted token types; document/test that target capability explicitly. |
| Satori 0.62.0 | Appropriate for static cards with supported layout, not a browser-equivalence promise. Mirket's adapter intentionally narrows layout to Flexbox. | **Fixed:** reject-only WOFF2 gate despite supported pinned renderer. Keep real-browser routing for unsupported layout and text requirements. Inspect bidi mixed inline cases and outlined-text editability. |
| resvg-js 2.6.2 | Native previews avoid a browser round trip and retain bounded execution. | Keep. DejaVu fallback makes text visible but does not prove the requested typeface rendered. Require exact local fonts for typography-sensitive exports; WOFF2 support added to Satori is not a claim about resvg's font-file API. |
| SVGO 4.1.0 | `cleanupIds:false` and `collapseGroups:false` were insufficient to preserve editable states. | **Fixed:** prevent removal of hidden elements, empty containers and unused definitions. Test hidden/empty groups plus unchanged resting pixels. Continue sanitizing separately. Optimization still changes geometry representation; it is not a lossless editor round-trip. |
| Sharp 0.35.5 | Good isolated decoder, pixel bounds, orientation handling and explicit transform interface. | **Fixed:** video/Typst staging no longer resizes non-PNG artwork to 2048 or re-encodes all supported rasters. Validate exact snapshots with full decoding and retain original bytes. Explicit raster transformations remain available. |
| Playwright 1.64.1 | Bounded DOM evidence, actions, reduced-motion selection and 2x captures are useful. | Keep. Separate captures with two initial preferences do not prove a preference change during one mounted session. Extend live scenario testing where needed; a sought frame does not prove real-time playback. No renderer replacement needed. |
| HyperFrames 0.8.149 | Existing lint → snapshot → render route is coherent for HTML video. | Retain pending the comparative pilot. Staging fix applies. Snapshot success and encoded metadata do not independently prove audio sync or temporal craft. Preserve unrelated media-evidence work already present in the checkout. |
| Typst 0.15.1 | Appropriate for paged output; staged roots and per-page inspection are useful. | Keep. Staging fix applies. Need use-size type/crop checks and explicit source-asset preparation; retaining bytes does not certify color management by every consuming renderer. |
| Penpot 2.17.0, pinned official MCP | Correct boundary for editable design objects; token synchronization and export-font embedding have focused checks. | Keep as optional editable-design capability. Current token bridge rejects unsupported composite types. No claim of full DTCG parity. Native editable-file journey was not exercised in this review. |
| Kroki 0.32.0 | Good route for declarative graph layouts. | Keep for graph semantics. Do not send crafted machinery, subject illustration or brand artwork to a graph-layout engine merely because it can output SVG. No full service-format matrix was exercised. |
| Odiff 4.5.0 | Useful for locating changes in matched images; native PNG inspection avoids duplicate screenshots. | Keep. Use equal content, viewport, font, time and state. Pixel equality can coexist with deleted invisible SVG state; the new regression explicitly tests both source structure and pixels. |

The earlier guidance sources (Taste, Hallmark, LottieFiles, Motion Primitives,
Canvas UI, Better Design and GSAP Skills) retain the decisions in the prior
review. Most already teach subject-led composition, interruption, tokens and
critical selection. Adding another general style checklist would duplicate them;
the proposed illustration, typography and concrete integration tests address
more specific gaps.

## Implemented from the review

Recommendations 1, 2, 3, 5 and 6 now have focused guidance in existing owners:
[engine selection](../skills/advanced-motion/references/engine-selection.md),
[constructed illustration](../skills/graphic-design/references/constructed-illustration.md),
[typography proof](../skills/interface-design/references/typography-proof.md),
[marketing claim-to-visual handoff](../skills/marketing-writing/SKILL.md) and
[maintained design rules](../skills/interface-design/references/art-direction-and-critique.md).
Recommendation 7 updates the actual router, MCP schema, installed instructions,
skills and public documentation. Pinned provenance and applicable notices follow
the guidance. Recommendation 4 remains a comparative pilot, not an adopted video
runtime or a claim of measured quality gains.

## Runtime repairs

- Shared raster staging validates the bounded source snapshot without reopening
  the original path, then preserves encoded bytes, dimensions, alpha/profile and
  metadata. It rejects corrupt/oversized input before writing the staged asset.
  Multi-page animated rasters in the handled formats now require explicit frame
  preparation/video instead of being silently flattened. Resizing/re-encoding is
  an explicit asset-preparation step.
- SVG hygiene retains hidden states, empty attachment groups and unused definitions.
- Static Satori rendering accepts WOFF2 with matching signature validation.
- Token generations invalidate on compiler/configuration or lock changes while
  preserving the separate token-source digest.
- The documentation site brand lockup wraps at narrow widths with doubled text.
- Regression tests exercise native decoding/rendering and real WOFF2 glyph output.
  No candidate production library, service or asset pack was installed or bundled.

## Verification and limits

Rust 1.99.0, rustfmt and Clippy are installed on macOS ARM64; the standard Cargo
environment is available in new zsh sessions. The CLI was rebuilt outside Cargo's
build output. Required `mirket dev check --project .` passed formatting, Clippy
with warnings denied and all standard Rust targets. Four explicitly ignored test
entries remain opt-in (including a subprocess helper, upstream fetch, published
update and the 125-second SDK regression); the applicable normal deadline and
child-cleanup tests passed. New gate tests cover native proof requirements,
changed targets/stale renders, supported media, rejected overrides and shared
15/30/45-minute budgets with their repair limits. MCP tests exercise exposed
schemas and invalid native input without dispatching paid inference.

The final design-tool suite ran against the changed source in the existing Linux
ARM64 container with Node 22.23.3: **78 passed, 0 failed, 0 skipped**, including both
opt-in live Chromium tests. Token-cache tests preserve unchanged-source identity
while a modified compiler actually regenerates different output; dependency-lock
changes also invalidate reuse. Raster snapshots, corrupt/animated input, hidden
SVG state and WOFF2 glyph rendering have regressions. Earlier before/after probes
and saved WOFF2 PNG, revealed SVG preview and one-page Typst PDF were inspected.
Full HyperFrames video export was not repeated; its staging path was exercised.

The rebuilt CLI's catalog check, disposable setup for both hosts and doctor passed.
Doctor exercised actual child-process MCP discovery, resource/skill delivery,
concurrent requests and malformed-input rejection. This establishes installation
and protocol behavior, not activation in an already-running desktop session.
Revised skill validation, local reference links and pinned source links passed.

The website build, script checks and deployment test passed. Chromium checks
passed at 1440, 768, 390 and 320 CSS pixels, with 2x captures, doubled text,
keyboard access, clipboard success/denial and JavaScript disabled. The enlarged
brand lockup and updated workflow/budget captures were visually inspected.

This review records the tested design-resource changes. Later runtime and public
release verification is tracked in [version availability](release-status.md).
No measured model-quality gain, candidate-library benchmark, live Penpot editing,
full video production or human acceptance follows from these checks.
