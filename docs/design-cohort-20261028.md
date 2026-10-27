# Eight-cell design cohort and demonstrated corrections

The D4 cohort ran on macOS 26.5.2 (25F84), ARM64, against
`a1971dfa82ae204a8df6bd8716b99fd752575498`. All eight initial author turns ran;
none was retried. Four fresh native reviewers inspected neutral artifacts.
The study closed before implementation changes. This is a small diagnostic
comparison with important harness confounds, not evidence of general skill
benefit, a Codex-versus-Claude comparison, or completion of the catalog's
`planned-not-run` acceptance.

## Frozen protocol and actual evidence

Local evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-matched-design-20261027/`.
The durable `cohort-evidence/` archive retains frozen inputs, source snapshots,
model traces/results, neutral packs, original reviews, the final budget and
lead adjudication. `cohort-oracles/` retains independent UI captures, decoded
frames, playback observations and the disputed-finding measurements. Session
homes and credential copies were excluded; no trial auth files remained.

The external seal SHA-256 is
`b6e7c3579be51db1a17f7c6acd28474ff30b42953eb42fae7c7df8eb1ab54fca`.
Eight native Codex CLI turns were configured as `gpt-6-astra` / `high`, with
`--approve-for-me`, equal tool access, identical paired briefs/producer contracts,
and isolated specialist guidance. The configured CLI/model settings are
observed; backend model identity is not independently authenticated. Reviewers
inherited the capable lead configuration without a lower-capability override;
their exact provider identity is likewise not separately authenticated.

The fixed ceilings were 240 seconds per initial, 180 per repair, four shared
repairs, 2,700 model seconds and 2,700 wall seconds. Actual author time was
1,471.635 seconds. Reviewer time was conservatively charged by elapsed wall time,
411.397 seconds, from one pre-reserved pool. Total: **1,883.032 model seconds**,
**2,115.241 wall seconds**. All reservations settled to zero. No repair was
eligible after the lead rejected unsupported major findings; minor findings
were retained under the frozen allocation rule. Initial artifacts are therefore
also final artifacts. No budget was reset through another specimen or reviewer.

| Neutral artifact | Task / condition | Author seconds | Retained outcome and independent review |
| --- | --- | ---: | --- |
| `22cbadb5` | Responsive UI / skill | 240.026 | Timed out without `turn.completed`; complete partial page retained. Reviewer 2/2/2; independent Enter-based filtering, empty/clear and error/retry checks worked. |
| `6785555b` | Responsive UI / no skill | 230.873 | Native turn completed, but frozen classifier marked it failed because Chrome updater diagnostics said `Permission denied`. Reviewer 2/2/2; independent interactions worked. Original failure record remains unchanged. |
| `4eba8d19` | Illustration / no skill | 177.916 | Saved editable SVG and actual Mirket PNGs. Reviewer 2/2/2; minor gold thread/fingertip merge at pressure size. |
| `0fcaecb8` | Illustration / skill | 226.654 | Saved editable SVG and PNGs; optimizer refused accessibility attribute. Reviewer 1/1/2; minor faint half-size label and needle-eye clarity. Source was unchanged between initial and final renders, not secretly repaired. |
| `6540ee78` | Captioned motion / skill | 155.485 | Real HyperFrames MP4, 36 independently decoded frames; reviewer 2/2/2, no concrete findings. |
| `cf9a94fa` | Captioned motion / no skill | 132.516 | Real MP4; editable source delivered under `.evidence` following an ambiguous prompt, while checker expected root files. Reviewer 1/1/2; one-frame blank hierarchy at 12 and leaf disappearance at 23. |
| `c4008b09` | Vector transition / no skill | 118.146 | Real MP4 and editable named groups. Original reviewer 1/1/2 alleged two major defects; both rejected by lead pixel measurements and isolated-frame inspection. Those scores are not treated as accepted quality results. |
| `bc1fa570` | Vector transition / skill | 190.017 | Real MP4 and editable named groups; reviewer 2/2/2, no findings. |

UI was exercised in Codex's browser at 1280×800 and 390×844, with saved desktop,
mobile, empty and error captures plus keyboard state records. Both authors'
CDP keyboard probes were inconclusive; direct host Enter interaction worked.
Neither viewport overflowed horizontally. The second neutral pack omitted its
referenced SVG, so independent captures used the unchanged complete source;
the reviewer was told this limitation without a condition label.

Illustrations were inspected at 640×480, 320×240 and hand/needle detail. One SVG
was also opened in the host browser as a separate renderer check. All four
videos independently probed as 640×360, 12/1 fps, 36 frames, one video stream and
no audio stream. All 144 frames were decoded with the explicit private FFmpeg;
reviewers inspected saved pixels and lead checks isolated transition/contact
neighbours. All four exports were observed playing at rate 1 and reaching their
three-second end in the host browser. This establishes playback completion,
not continuous perceptual pacing or audio quality.

Masking was limited: source/comments/style could reveal guidance. Reviewer
claims and self-reported scores were never treated as authenticated truth.

## Finding-to-fix matrix across the six review areas

| Area | Reproducible finding / consequence | Correction and source owner | Evidence / remaining limit |
| --- | --- | --- | --- |
| 1. Skill authoring/references | This cohort does not demonstrate overall benefit from the selected specialist bodies. Illustration scores varied in the opposite direction from captioned motion. | No speculative skill rewrite or reference replacement. Existing Hallmark/Taste adoption remains in [source-design-adoption](../planning/reviews/2026-10-27/source-design-adoption.md). | All eight artifacts and original reviews retained. Catalog refresh reports no changed hashes; source-level provenance remains unchanged. |
| 2. Skill/tool pairing | Actual SVG renders succeeded, but optimization rejected only `aria-labelledby`; removing that attribute made the old optimizer pass. | [design-operations.ts](../adapters/src/design-operations.ts), `validateSvg` / `optimizeSvg`: accept local ARIA ID tokens; preserve IDs, descriptions and referenced empty containers through SVGO when ARIA references exist. | Real MCP replay of the exact failed source now optimizes and rasterizes. External/malformed references still refuse. No new dependency or tool route. |
| 3. Project memory | No new memory defect demonstrated by this design cohort. | No memory edits; retain [memory review](deep-review-memory.md). | Fresh ordinary Rust memory tests passed. Heavy model/scale fixtures remain ignored unless explicitly supplied; no new semantic benchmark claimed. |
| 4. Capability matrix | Common prompt placed editable files under `.evidence`, while the checker expected project-root files; neutral pack omitted a referenced SVG and the common contract. | [matched_design_study.py](../tests/matched_design_study.py), `PROMPT`, `production`, `review_pack`: agree on root deliverables and retain SVG visual assets plus `PRODUCTION.md`. | Discriminating file-placement/pack tests pass. The old source-placement result remains a confound; no post-hoc model rerun. See [capability matrix](deep-review-capability-matrix.md) for broader coverage. |
| 5. Agent coordination | Broad substring matching classified a Chrome updater OS warning as a host approval denial, despite a completed native turn. | [motion-feedback-pilot.py](../tests/motion-feedback-pilot.py), `failure_signals`: inspect host failure payloads, exclude command echoes/author prose and generic updater warnings; retain native failed-tool permission signals. | Actual retained UI trace no longer produces an approval signal. Genuine approval/startup/tool-error negative cases pass. D5 agent-choice/override/interruption scenarios remain planned; Mirket cannot enforce native spawning. |
| 6. Feedback/output quality | Reviewer alleged plane reversal at 22–24 and missing headline glyphs at 16/19. Acting on those claims would spend a repair turn on unsupported defects. | Lead retained original findings and wrote `lead-adjudication.json`; no artifact modification. Evidence pack and classifier corrections above improve the review inputs. | Red-plane bounds move downward: y182–197 →190–205 →200–213. Headline crop 19–25 is byte-identical; 16/19 differ from legible frame 0 by at most 27/255 per channel, inconsistent with loss of multiple dark glyphs. Saved enlarged crops and measurements retained. |

The captioned-film leaf disappearance at 23 is real, but the brief did not
require contact through that final middle-frame boundary: a visible pause is
present earlier. The review's stronger interpretation was qualified in the
lead adjudication. Both transition polish observations and three illustration
pressure-size findings remain recorded; no mandatory criterion was invented to
justify a repair. No claim is made that either condition is universally better.

## Implementation verification

The exact cohort SVG source revision was
`sha256:4229e61f08e377acddcb97806b92977d71d3585133f84f024f082d0dceeffcb8`.
The fixed actual MCP optimizer produced
`sha256:7961f5c84bae7075b1078aafebb8eb57795a8d22d961e14cdbbefb77a5e7f13c`.
Its named accessibility targets survived; a fresh PNG was saved, reopened and
visually inspected. Original-versus-optimized raster comparison changed 54 of
921,600 RGB channel values, mean absolute delta 0.000951, maximum 42. This is
not a repair of the faint authored label. An external ARIA URL failed without
creating its destination.

This replay used disposable state at `svg-fix-qualification-v2/`. Its first
preflight attempt (`svg-fix-qualification/`) was refused because the test harness
created state directories with overly broad permissions; no operation ran.
The corrected attempt used mode 0700. Both records remain retained. This is a
runtime regression replay, not another creative/model trial.

Fresh gates on macOS ARM64:

- Build-input guard passed before Rust compilation.
- `cargo test --locked --offline`: 505 passed, five explicitly ignored across
  library/CLI/integration suites. Ignored: private Sharp payload, real embedding
  package, real uninstall fixture, synthetic semantic retrieval and scale run.
- `cargo clippy --locked --offline --all-targets -- -D warnings` and
  `cargo fmt --check`: passed.
- Locked existing TypeScript closure compiled using the explicit private Node;
  full adapter tests: 67 passed, one private Raster-module fixture skipped.
- Affected Python gates: 116 passed initially; after the code reviewer found
  two additional classifier edge cases, all 48 affected runner tests passed
  again (117 distinct tests across the gates). Coverage includes frozen
  scopes/budgets, neutral pack integrity, actual-denial preservation and
  updater/echo false positives, including failed-command echoes and top-level
  MCP permission errors.
- `tests/mcp-smoke.mjs` with explicit Mirket/private Node paths: passed on
  darwin/arm64. Actual design effect was separately verified above.
- Repository-owned catalog refresh: no changes, 60 retained reference records;
  audit SHA-256 `1c3814383b1eb0603e53a15f2ddf2dd2c8c76c7a34600a8aa3dff887f80287f0`.

No dependencies, admission batch, live installation, model router, alternate
video engine or agent orchestrator changed. HyperFrames remains the sole engine.
Windows native fidelity, the proposed Mac OS floor, waived Desktop GUI,
continuous human perceptual judgment and public distribution source/notices
remain unqualified. D5 is the next remaining design-plan step. The earlier
[six-area review](deep-review-20261027.md) remains the broader audit; this follow-up
does not silently close its unresolved paths.

Changed files: `adapters/src/design-operations.ts`,
`adapters/tests/design-operations.test.mjs`, `tests/matched_design_study.py`,
`tests/motion-feedback-pilot.py`, `tests/test_matched_design_study.py`,
`tests/test_motion_feedback_pilot.py`, `planning/remaining-design-work.md`,
`README.md`, and this report. The local commit ID is recorded in the task's
delivery message; no push or publication is authorized by these checks.
