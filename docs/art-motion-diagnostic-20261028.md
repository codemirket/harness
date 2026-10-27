# Artwork versus motion diagnostic — 2026-10-28

The user asked whether disappointing creations originate in SVG authoring or
animation, then authorized a controlled comparison. Baseline: `f8741be`.
This is a distinct diagnostic question; the closed D4 cohort and its failed
trials remain unchanged. It does not complete the six-area audit or D5/D6.

## Finding and consequence

The strongest reproducible failures were in authored animation and layer order,
not the SVG format or HyperFrames capture. Static art direction is a separate
quality issue: the lead found the prior character illustrations conventional
and schematic; the independent reviewer found them coherent and serviceable,
not intrinsically broken. Neither judgment proves that a different renderer or
model would produce excellent illustration.

| Finding | Observable consequence | Correction / owner | Evidence |
| --- | --- | --- | --- |
| CSS keyframes used SVG-style `translate(0 10px)` syntax | Chromium discarded heart transforms but retained opacity. The export succeeded while the intended subjects did not rise. | Corrected diagnostic A to CSS comma-separated transforms. `scripts/create-motion-preview.mjs` now accepts optional `expected_properties` and refuses missing accepted target/property pairs. | Initial `CSS.supports` false, browser `getKeyframes()` lacks transforms; repaired keyframes include them and frame 18 shows lifted hearts. Positive/negative browser probes below. |
| Whole-effect easing was treated as segment easing | A's intended late closure occurred by the film's midpoint. Authored offset times disagreed with decoded output. | Repaired A uses a linear effect clock and outgoing-keyframe easing. `skills/motion-direction/references/editable-motion.md` explains the distinction without prohibiting intentional global easing. | Initial frames 18–35 are closed; repaired frames 18–24 hold the intended open reveal and frame 35 rests closed. |
| The adapted SVG rig painted a foreground border over moving hearts | B frames 15/18/24 have a blue horizontal stroke through red hearts. A correct closed still did not establish moving-layer correctness. | Shared final rig keeps the top mouth edge behind hearts; front perimeter retains the sides and bottom. Motion reference guidance now calls out travel-path/draw-order checks. | Initial/final decoded pixels and retained rig sources. Both final arms use identical geometry, HTML, CSS and font. |
| Reference-inspired choreography was crowded | Independent neutral reviewer preferred initial A to initial B despite B's reference. Five arcs competed for focus. | One shared repair batch reduces B to three smaller, staggered arcs. The motion skill separates static direction, staging and implementation diagnosis. | Neutral review `cell-82` = A, `cell-17` = B; original review preserved. Final B has separated heart arcs without the slicing edge. No fresh independent final ranking. |
| Presence of animation handles can be mistaken for coverage | A moving neighbour or surviving opacity conceals a dropped transform. A valid constant transform still does not imply movement. | Preview reports its intended-property coverage limit. Checks establish accepted property presence only, not movement, timing or quality. | Invalid-transform and wrong-target fixtures refuse; valid fixture moves to y=50 at 1500ms; constant-transform control passes presence but stays y=0. |
| Local evidence delivery initially prevented seeking | Python's basic HTTP server exposed video seekable range `[0,0]`; slider text changed while the video stayed at time 0. | Task-local range-capable review server, outside shipped source. Actual media time rechecked. | Both final videos seek to 1.500s; saved screenshot shows frame 18. This was a viewer-delivery issue, not a HyperFrames export failure. |

The final A retains a translucent seal/flap handoff and a long still hold.
Final B remains simple and its open-flap apex is close to the headline, although
no actual headline clipping was demonstrated. These are disclosed instead of
starting another creative retry. The diagnostic establishes concrete repairs,
not premium art direction or aesthetic superiority.

## Frozen comparison and retained artifacts

Evidence root on the qualification machine:

```text
/Users/nazmiyilmaz/.codex/task-state/harness-art-motion-diagnostic-20261028
```

`comparison.html` is the user-facing local page with both final MP4s, both
initial MP4s, the prior still and Google's motion reference. Open it with its
relative asset tree intact. `serve-review.py` optionally serves that tree on
127.0.0.1:8767 with byte-range support; this is a disposable evidence utility,
not an installed Mirket tool. `comparison-screenshot.jpg` records the matched
frame view.

Protocol SHA-256:
`3d28f8b240f3dbb1305ac9be1b291ce9d39ed909a6a5caf750b9c81d27e64751`.
It freezes two initial treatments, one independent review, one shared repair,
and a 45-minute creative window. A fresh native author received the fixed rig
and brief without the motion reference. The lead authored B using the frozen
reference. Authors therefore differ: this is not a randomized or matched-model
experiment, and cannot isolate the effect of the reference or establish a model
ranking. Native agents inherited the lead configuration; backend model identity
was not independently authenticated. No new CLI author trial was run.
The creative diagnostic closed after 1011.663 seconds (16.86 minutes); the one
repair is exhausted. Documentation and gate reporting do not reopen it.

The native reviewer received neutral IDs, saved stills and sampled frames,
without author source or rationale. Its initial report's broad playback wording
was narrowed after clarification: it used QuickTime through the native computer
surface and observed several times/endpoints, without retained continuous
playback capture. Those observations are not treated as proof of perceptual
smoothness. The lead's saved 1× browser records separately establish start/end
progress and actual seek time. No lack of inspection was inferred from absent
CLI image events.

Key retained files:

- `protocol.json`, `brief.md`, `repair-ledger.json`, `shared-artwork-identity.json`;
- `source-{a,b}-{initial,final}/`, `export-{a,b}-{initial,final}/`;
- `decoded-{a,b}-{initial,final}/`: 36 independently decoded PNGs each;
- `sequence-{a,b}-{initial,final}.png`: sampled frame sheets, inspected alongside
  selected full-size transition frames; all 144 frames were decoded, not all
  individually visually inspected;
- `review-pack/review.md`: independent initial findings and playback correction;
- `browser-invalid-keyframes.json`, `browser-a-final-keyframes.json`,
  `browser-preview-checks.json`, `browser-final-playback.json`,
  `browser-final-seek.json`;
- `red-subject-oracle.json`: independent colour-region movement discriminator,
  not an aesthetic score or complete object tracker;
- `static-render/result.json`: actual disposable Mirket source/rig PNG renders;
- `gates/summary.json` and logs, `reference/manifest.json`, `NOTICE.md`.

## References and rights

The unchanged static asset is [Google Noto love letter at commit
99701e07d3d1fe1bdbcdb14964547f0fd592966f](https://github.com/googlefonts/noto-emoji/blob/99701e07d3d1fe1bdbcdb14964547f0fd592966f/2D/svg/emoji_u1f48c.svg)
(commit dated 2026-10-26), with its directory's Apache-2.0 notice and the full
license retained. The original SVG SHA-256 is
`166acacbe38c229aa9d1cdd6abe8acdd5c52ed53dca4d6f3d8a77ba79378315d`.
The derived rig preserves the heart paths and reconstructs separable envelope
layers; it is not the exact original SVG.

The [official Noto animation collection](https://googlefonts.github.io/noto-emoji-animation/)
serves the GIF and Lottie reference. The GIF is frozen by SHA-256
`f02181e3c422de830c4e3985d6d4ded226aaec2d30c08ccaaef3cee0e672bcda`;
its 84 variable-duration frames total 2760ms. The Lottie has 184 frames at
60fps (3066.7ms), so the two timebases are not conflated. The precise directional
reference uses the GIF's observed opening, burst and closing phases, adapted
to the fixed three-second profile.

[Google's publisher statement](https://developers.googleblog.com/en/updates-to-emoji-new-characters-new-animation-new-color-customization-and-more/)
licenses the animations under CC BY 4.0. Its 2022 date is disclosed: this is
rights evidence, not a claimed recent trend or recent creation date. The
moving download URLs have retained hashes, not invented immutable version IDs.
Credit, source/license links and modification notes accompany the adaptations.
The direct CC legal-text download returned HTTP403; the CC license URI and
publisher statement remain retained. The unchanged admitted NotoSans font
matches SHA-256 `bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d`
and its OFL notice is retained. No Noto artwork, animation runtime or new
production dependency is added to the Mirket release.

## Verification and current limits

Actual platform: macOS 26.5.2, build 25F84, ARM64. Rendering used existing
explicit private Node, Chrome for Testing, FFmpeg/ffprobe and the compiled
HyperFrames adapter through real Mirket MCP jobs with disposable project,
state, home and temporary directories. Four exports each contain one video
stream, 640×360, 12fps, 36 frames; no audio was requested. The private renderer
worked for this scope. Live `~/.mirket` and client settings were not changed.

After `python3 scripts/check-build-inputs.py` passed:

- `cargo test --locked --offline`: 505 passed, 5 ignored across targets;
- `cargo clippy --locked --offline --all-targets -- -D warnings` and
  `cargo fmt --check`: passed;
- locked adapter closure compiled using explicit private Node and local
  TypeScript; adapter process tests: 67 passed, 1 skipped (optional raster
  runtime not supplied to that test);
- preview tests: 10 passed, including moving-neighbour/dropped-property and
  wrong-target discriminators; browser probes validate actual parsing/seek;
- catalog/provenance refresh tests: 6 passed;
- explicit-binary/private-Node MCP smoke: passed;
- repository-owned `refresh-skill-catalog.py --write` updated body/resource
  hashes and generated reference evidence; subsequent check reports no drift.
  Existing provenance classifications remain unchanged; this diagnostic's
  external assets are not newly adopted shipped skill resources.

The optional system skill-creator `quick_validate.py` could not start because
PyYAML is absent from both available Python environments. No package was added
for that convenience check; the repository's actual native skill/catalog tests
and refresh consistency checks passed. This validator is not reported as passed.

Windows, macOS floor qualification, the waived Codex Desktop GUI check,
production-frame-rate motion, audio, sophisticated illustration, and model
comparisons remain unqualified. Catalog planned-not-run acceptance is unchanged.
