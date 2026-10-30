# Focused design/video source adoption review (2026-10-27; updated 2026-10-30)

This is the explicit source-to-skill mapping for the nine project skills whose
catalog classification is `method-adapted`. It records reviewed techniques, not
copied source text or executable dependencies. The local wording, output
contracts and evidence rules remain authored in each `SKILL.md`; the catalog
refresh script does not infer fidelity from a URL, hash, receipt or family name.
All nine catalog acceptance sets remain `planned-not-run`; this document is not
model or visual acceptance evidence.

## Mapping ledger

| Skill trigger | Exact reviewed source and section | Technique retained locally | Local decision/output/evidence rule | Removed assumptions and rights boundary |
| --- | --- | --- | --- | --- |
| `art-direction` when a brief needs a distinct visual concept and inspected specimen | Hallmark `13ac0ec7e148655948100b6396439e481361d690`, `skills/hallmark/SKILL.md` sections “Design flow (default)” and “2. Pick a macrostructure FIRST”; Taste `f624f37a8c650b9cc550253104925efe4c8de246`, `skills/taste-skill/SKILL.md` sections “1. Read the brief and the project”, “3. Draw the direction” and “7. Gate”; Impeccable `4e8504f10106a1cd7a99e37a401aa368d1d55576`, `README.md` “The Skill: impeccable” and “Anti-Patterns”. | Start from a brief and reference intent, choose a constrained structural idea, test one real-content specimen, then check the rendered result for generic treatment that weakens the product idea. | Return the brief-linked direction, exclusions and inspected specimen; record source/asset rights and remaining visual defects. A familiar style alone is not a defect. | No brand, font, artwork, generator, installer, detector or upstream prose is redistributed. Method is independently worded; source-level terms remain in receipts and plan. |
| `design-critique` when a visual artifact needs ranked, actionable critique | Hallmark `13ac0ec7e148655948100b6396439e481361d690`, `skills/hallmark/SKILL.md` “Review format”/design-flow checks; Taste `f624f37a8c650b9cc550253104925efe4c8de246`, `skills/taste-skill/SKILL.md` “The inspector” and “7. Gate”; Impeccable `4e8504f10106a1cd7a99e37a401aa368d1d55576`, `README.md` “The Skill: impeccable”, “Anti-Patterns” and “audit”. | Check the composition against structure, brief/real content and observable anti-patterns; rank defects by consequence and pair each with a concrete repair and recheck target. | Return findings with location, consequence, repair and remaining defect; do not call an artifact verified from a checklist or detector alone. | No external rubric text, browser/CLI detector or hosted design tool is adopted; the three pinned sources are method references, not installed tools. |
| `interface-composition` when a product surface needs hierarchy and state-aware composition | Hallmark `13ac0ec7e148655948100b6396439e481361d690`, `skills/hallmark/SKILL.md` “2. Pick a macrostructure FIRST” and component-scope output; Taste `f624f37a8c650b9cc550253104925efe4c8de246`, `skills/taste-skill/SKILL.md` “1. Read the brief and the project” and “6. Build it, still”; Impeccable `4e8504f10106a1cd7a99e37a401aa368d1d55576`, `README.md` “Anti-Patterns” and “audit”. | Use content hierarchy, real loading/empty/error states and responsive pressure as composition inputs, then inspect rendered treatment and claim truth. | Return a project-scoped composition with affected states and saved-pixel inspection evidence; preserve explicit exclusions. | No design system, component package, browser automation, detector or brand asset is imported. |
| `motion-direction` when sequence, timing, interruption or reduced motion needs a coherent specification | Huashu commit `4fc7bfdbd5ec3fa2d43ce7a6433de60976a5e4b1`, `references/storyboard-basics.md` sections “4 · 能量骨架” and “5 · 轻量分镜卡”, plus `references/hyperframes-backend.md` “校验与渲染”. | Use shot cards/beats, purpose per transition, interruption behavior and a reduced-motion alternative before implementation. | Return a choreography/spec with timing, lifecycle, interruption and reduced-motion outcomes; inspect the saved result when a renderer is available. | Huashu is MIT at the pinned revision; no renderer, backend, remote font, quota, approval model or source text is redistributed. |
| `scroll-motion` when scroll geometry, refresh or interruption requires a plan | Emil `e8a175de22ae1e49370fc144c1f3bb9aeedf988d`, `skills/emil-design-eng/SKILL.md` sections “The Animation Decision Framework” and “Perceived performance”, applied only to scroll context. | Specify scene geometry and refresh/re-entry behavior, and treat interruption/reduced motion as first-class states. | Return scene boundaries, ownership and evidence for refresh, resize, interruption and reduced-motion behavior; do not infer success from a receipt. | No GSAP runtime, browser or animation package is installed by this skill. |
| `ui-motion` when an isolated interaction needs lifecycle-safe motion | Emil `e8a175de22ae1e49370fc144c1f3bb9aeedf988d`, `skills/emil-design-eng/SKILL.md` sections “Review Format (Required)” and “The Animation Decision Framework”. | Check the installed API/version, define lifecycle cleanup and reduced-motion behavior, and keep isolated implementation separate from direction. | Return changed source plus behavior checks for mount/update/unmount, cancellation and reduced motion. | No upstream API examples or package installation are adopted; host/project toolchain owns dependencies. |
| `vector-art` when editable SVG geometry or a sprite needs authoring/refinement | MengTo commit `a45892d2fdadab75b31fb16abecc02f101b31c43`, `agent-skills/illustration/illustration-framed-panel/SKILL.md` and `illustration-flat/SKILL.md` sections “Palette”, “Line and fill”, “Composition” and “Techniques”; local resource `skills/vector-art/references/illustration-craft.md` is independently worded. | Prefer editable geometry, coherent construction constraints and inspected exports at target sizes; retain an asset-rights check. | Return editable SVG/source, optimized export and saved-pixel checks at target sizes; report clipping or unsupported effects. | No upstream SVG, demo, script or font is copied. Local reference is bounded and hashed by catalog tooling. |
| `openpencil` when editable native design source is requested | OpenPencil `24461adec7073312d460d76f08f069de18e4fb78`, `skills/open-pencil/SKILL.md` “MCP workflow”, “Tool discovery”, “Tips” and `references/design-authoring.md` “Composition and layout”, “Verification”; local resource `skills/openpencil/references/file-authoring.md`. | Select one file/revision/target, author container-owned layout through bounded exact-path edits, then reopen and inspect structure and saved pixels under real text pressure. | Return changed editable source, native identity/reopen evidence and inspected exports; refuse unsupported editor/asset features and stale revisions. | Skill-local MIT; no upstream prose, CLI/editor control, eval, artwork, variables or rich layout API copied. |
| `video-production` when an editable composition must export with decoded QA | HyperFrames authoring commit `f51650d8a72ed472effff89da087c7e1cf6b2c98` producer/troubleshooting guides; HyperFrames skill commit `10a4ed56cb09159afec0dba3ea784e3162ee2e71`, `skills/hyperframes/references/motion.md` “Motion craft”/“Seeing motion” and `review.md` “Measure, then look”/“Notes and fixes”; FFmpeg skill commit `eeff097f86654830fa8aa409185ea477524c4cb8`, `SKILL.md` “Workflow” and `docs/design-decisions.md` §2.21.1 / `scripts/_common/emit.py` stream-end checks. | Keep editable composition/source and asset readiness; choose absolute frame indices for beats, cuts, cue boundaries and subject motion; verify saved video and audio stream endpoints independently. | Return source/export, exact sampled frames, decoded frame/audio/caption results and remaining visual/audible limits; refuse early-ending audio and recheck only repaired states. | HyperFrames Apache-2.0 and FFmpeg skill MIT at their pins. No upstream text/scripts, installer, 42-tool MCP suite, private Chrome, cloud/account workflow or unqualified API is adopted. |

## Evidence and limits

`packaging/provenance/skills/reference-fetches.json` and
`reference-audit.json` retain URL, status, final URL, byte count and SHA-256
receipts. A successful fetch is source evidence only. It does not establish
operation success, visual quality, video decoding, tool availability, model
selection or user acceptance. The focused adoption run retained raw bodies
outside the repository; only bounded local reference notes are shipped.

The nine skills continue to require independent held-out trials. In
particular, the catalog's broad planned acceptance is not complete until a
capable lead/reviewer or equivalent independent oracle inspects the resulting
artifact. Windows qualification and the waived Codex Desktop GUI check remain
unverified.

The 2026-10-29 design selection extends the already reviewed pinned Impeccable
anti-pattern method to art direction and interface composition and the pinned
Taste inspector method to critique. It follows the user's explicit choice of
Hallmark, Taste and Impeccable as primary design guidance. The 2026-10-27 fetch
receipts and the earlier qualification limits retain their original dates;
this edit adds neither a runtime dependency nor a new model outcome trial.

## 2026-10-30 bounded adoption trials

These are focused implementation checks, separate from the catalog's planned
model-to-useful-output cases and the older 2026-10-27 qualification. The local
candidate was macOS 26.5.2 (25F84) ARM64 with private Node 24.22.1, private
project and Pi state. Source snapshots, rights and complete rejected-scope
notes are in the task-private reviews below; the pinned URL/hash receipts are
retained in `reference-fetches.json`.

| Slice | Focused evidence | Scope limit |
| --- | --- | --- |
| OpenPencil file authoring | `/private/tmp/mirket-openpencil-skill.37iTbq/review.json` and `trial3/verification.json`: 16 typed operations completed; saved native source reopened and patched; changed exports were inspected. A long label clipped despite passing geometry, then a wider container and text box rendered the whole label. | Deterministic typed trial; native editor fidelity, held-out Pi skill selection and user acceptance not established. The initial failed invocation remains in the private trial. |
| HyperFrames frame review | `/private/tmp/mirket-hyperframes-frame-trial/results.json` and `identity.json`: two 13-frame MP4s with identical endpoints were decoded; the midpoint check detected an overshoot at frame 6. Beat-index arithmetic found frame 450 where cumulative rounded steps would reach frame 448. | Synthetic composition and local frame inspection; no broad motion-quality or playback claim. No newer producer API admitted. |
| FFmpeg stream end | `/private/tmp/mirket-ffmpeg-stream-end-e_qidtw5/consumer-trial.json`, `tests.json` and `final-focused.json`: private FFmpeg/ffprobe MP4 and WebM controls showed a 2-second container/full decode could conceal a 0.5-second audio tail. The new verifier rejected those short tracks and accepted full and delayed-end controls; locked adapter build and tests passed (143 pass, one existing skip), including 15 focused video tests. | Checks encoded stream extent. Audible content, gain, sync and player behavior still require content review. The initial unavailable-lavfi fixture attempt is retained privately. |
