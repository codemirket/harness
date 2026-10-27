# Video authoring pilot: independent export oracle

This is the external export and saved-pixel review for the frozen 24-cell
motion-authoring pilot. It is not a model score and it does not claim visual
uplift. The run made no model calls, repairs, retries, or writes to the pilot
projects. It copied only the supplemental manifest's expected final files and
named font into fresh private export sources.

## Frozen inputs and runtime

- Supplemental manifest: `/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-qualified/final-24-cell-export-manifest.json`
  - SHA-256: `1294728b5cba16759b8e7848908ad514948bd87c36c4adec4e3902780c2e9ed5`
- Independent rubric frozen before output inspection:
  `/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-exports/frozen-external-review-rubric.json`
  - SHA-256: `ac60a24ef71f252e7b4da405c8bc910c528f8f03e8ad2371aa7ecea3e094ea7f`
- Oracle script and private output index:
  `/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-exports/run-independent-exports.mjs` and `export-index.json`
- Private Mirket runtime: release `motion-authoring-20261027`, Mirket `0.0.1`,
  Node `v24.22.1`, Chrome for Testing `156.0.8078.47`, FFmpeg `9.0.2`, on
  `darwin-arm64` (macOS 26.5.2 arm64 pilot platform).
- Explicit binaries: repository `target/debug/mirket`, the private Node,
  `adapters/dist/video-main.js`, private Chrome for Testing, FFmpeg, and FFprobe
  recorded in each runtime receipt. The user `~/.mirket` installation was not
  used or changed.

The production profile was 640×360, 12 fps, 36 frames, 3 seconds, MP4. Each
successful export was decoded at frames 0, 6, 12, 24, and 35. Those 75 PNGs
were opened with the image-capable `view_image` tool before the observations
below were recorded. The private review record is
`pilot-exports/visual-review.json`.
The 75 decoded-frame SHA-256 values are retained in the adjacent private
`pilot-exports/decoded-frame-sha256.txt`; for example, the reopened Pebble
frame-35 PNG has SHA-256
`cc23c6cee05181e117b4d49b63968ca3ab669fc9819f3a12b93ba29eddb2e35d`.
An initial, superseded misread is retained as
`pilot-exports/visual-review-initial-superseded.json` with no observation-time
claim.

## Outcome matrix

| Outcome | Count | Cells |
| --- | ---: | --- |
| Exported, authoring phase passed | 13 | all four `malformed-root-cues`, all four `pebble-motion`, all four `valid-control`, and `glasshouse-product-baseline-r2` |
| Exported with authoring timeout preserved | 2 | `glasshouse-product-baseline-r1`, `glasshouse-product-diagnostics-seed-r1` |
| Missing final deliverables; no export attempted | 5 | all four `cinder-vector`; `glasshouse-product-diagnostics-seed-r2` |
| Truthful refusal recorded | 4 | all four `refusal-local-asset` |

The split is therefore 15 rendered videos, 4 refusals, and 5 completed
non-exportable cells. The two complete Glasshouse sources whose authoring
turns timed out were rendered for independent inspection, but remain labelled
as authoring failures; this does not turn a timeout into a pass. The five
missing-deliverable cells were not treated as pending. No result directory was
reused from the aborted run; its partial outputs are quarantined under
`pilot-exports-aborted-20261027` and are not evidence. The aborted pass ran
approximately 21:10–21:13 TRT on 2026-10-27, using the older
`final-24-cell-manifest.json` (the authoring-only manifest). While it was
rendering, the supplemental `final-24-cell-export-manifest.json` arrived at
21:13:56 TRT with explicit `rows`, expected deliverables, caption selections,
and export flags. The pass was interrupted and its 16 partial cell
directories (including `source-selection.json`, runtime stdout/stderr, and
some decoded outputs) were moved to the quarantine directory; they were not
merged into the final index. The corrected pass was then run once against the
supplemental manifest. This restart changed the manifest/input mapping only:
it made no model call, authoring repair, authoring retry, or pilot-project write.
It was an export-verification rerun; the extra rendering is not hidden in the
24-cell authoring budget.

## Saved-pixel findings

### Glasshouse

The three exported cells (`baseline-r1`, `baseline-r2`, and
`diagnostics-seed-r1`) show the required `Glasshouse`, `See every seedling`,
and `Bench 4` copy at every sampled frame. The active state advances across
the three facts, and captions stay in the dark bottom band without a visible
collision in the inspected frames. The three cells use distinct stacked,
card-row, and list compositions and reach a visible `Bench 4` state at frame
35. Five samples do not prove smoothness, unsampled-frame behavior, audio,
alpha, or player playback.

### Pebble

All four exported cells show `Pebble` and `Keep one good note.` and a clay
marker moving from an early absent/entering state to a middle label-crossing
state and a late settled state. I reopened the exact saved
`/Users/nazmiyilmaz/.codex/task-state/harness-video-diagnostics-20261027/pilot-exports/pebble-motion-baseline-r2/runtime/decoded/frame-35.png`
at original resolution after an initial misread: its Pebble title is fully
visible and centered. No reproducible visual defect was confirmed in these
five-frame samples. The sample still does not establish the exact pause
duration or unsampled reversal behavior.

### Contract-only fixtures

The four `malformed-root-cues` and four `valid-control` exports are a white
body with their supplied cue text in the bottom caption band. This is
consistent with their frozen briefs: they test source-contract repair or
preservation and supply no visual scene. Both cue texts are readable in the
sampled frames. No design quality or uplift is claimed for these fixtures.

The lead's source/pixel cross-check identified a font-coverage limit: these
minimal sources declare `LocalNoto` but do not assign it to the body or
captions. Their decoded captions appear serif. Thus these fixtures establish
caption content and timing, not use of the selected Noto face; the actual
resolved face was not measured. The frozen specimens remain unchanged.
The authoring reference now explicitly applies its chosen family and explains
caption inheritance. That post-pilot guidance clarification was not part of
the measured treatment bundle.

### Cinder and refusal cells

No Cinder pixels were reviewed because all four cells lacked the required
editable source files (`index.html`, `film.css`, `film.js`, and
`illustration.svg`). The four refusal cells produced `decision.json` with the
refusal path and were not sent to video export. Their decision receipts are
evidence of the refusal contract only, not of a video outcome.

## What this evidence does and does not establish

The runtime receipt confirms saved H.264 artifacts, complete 36-frame decode,
dimensions, duration, source revisions, and selected local inputs. It
explicitly leaves alpha qualification, audio playback, player playback, and
semantic motion to independent review; this report does not promote those
fields to verified claims. The runtime's legacy optional red-rectangle probe
was not used for these fixtures. Windows, mobile, other output formats, audio,
alpha, and full-frame temporal smoothness remain unqualified. This small,
unblinded sample cannot establish aesthetic uplift from the diagnostic skill
delta.

After the initial review disagreement, the lead separately opened all 41
byte-distinct PNGs covering the 75 sampled files; `lead-pixel-groups.json`
records identical-byte groups. This confirmed the withdrawn clipping claim
was a review error, and supported the limited observations above. Identical
hashes only identify repeated copies of pixels that were actually viewed;
they do not establish unobserved frames or typography provenance.
