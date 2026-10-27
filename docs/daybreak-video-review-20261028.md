# Daybreak video defect review — 2026-10-28

The user reported small defects in the delivered reel, then specifically named
the opening's clipped `y` and positioning on another page. This explicitly
authorized a follow-up to the closed initial production pass. Two demonstrated
defects were repaired without changing the page, imagery, renderer or timing.
The current [film player](../specimens/daybreak/reel.html) has an Opening button
for inspecting the corrected title, alongside the existing chapter controls.

## Findings and corrections

| Defect | Cause and correction | Evidence |
| --- | --- | --- |
| Opening `y` descender and right edge of `k` were cut off even after the reveal settled. | The horizontal `clipPath` in `film.js` also clipped vertical glyph overhang and the right overhang caused by tight tracking. Expand the mask vertically and laterally while retaining the horizontal reveal. | At 0.7s the line box ended at 443.09px while the glyph ink extended below it. Disabling only the mask restored both letters. Same-browser masked/unmasked comparison changed 1084pixels before repair, zero after repair (RGB difference threshold 12). |
| Hero CTA and large wordmark nearly touch in the settled shot. | The earlier line-height repair brought the wordmark's painted top to 421px, about 3px below the button. Reduce only the film's wordmark from 255px to 235px. | Final painted top is about 439px, giving roughly 21px separation. The independently reserved 18px strip below the button contained 431 cream pixels initially and zero finally. |

The independent reviewer inspected all 360initial saved frames and native
transition boundaries. No additional demonstrated defects were found; the
moving panels' temporary edge occlusion was distinguished from resting text
clipping. The lead inspected all 360final frames at contact-sheet scale and
native frames at the defect times. Contact sheets are useful for flashes,
seams and ordering, not for certifying every tiny glyph. The unchanged opening
reveal begins on a plum first frame; the player uses a nonblank poster.

## Controls and regression checks

[check-export.mjs](../specimens/daybreak/qa/check-export.mjs) is a frozen-artifact
pixel check with explicit private executable/video paths. It checks y/k ink,
CTA separation and the earlier full-height stage-gutter defect. The initial
video fails the first three checks; the repaired video passes all four.

An IAB JPEG screenshot initially proved an unsuitable exact raster oracle for
the private video renderer: the candidate's k region had 80 classified lime
pixels versus 95 in that browser screenshot, narrowly failing the unchanged 85%
coverage threshold. This was investigated rather than lowering the threshold.
A separate counterfactual was rendered through the same Mirket route with only
the opening mask disabled. Its y/k regions have 285/83 pixels versus 283/80 in the
repaired video. The original video has zero in both regions. The browser pair's
zero-difference result separately establishes that the repaired mask does not
remove visible glyph ink in that browser. The control export is not the
candidate being used as its own oracle; its independent source manifest and
single intervention are retained.

One original-code repair batch was used. There was one capable independent
reviewer, with disjoint ownership of findings only, followed by targeted
verification of the repaired output. The new task's protocol allowed 30 minutes
and at most two repair batches; the prior production budget stayed closed.

## Actual outcome and limits

- Actual Mirket exports ran with fresh disposable projects/homes/state and
  explicit private Node, Chrome, FFmpeg and FFprobe paths. No live installation
  writes. This pass produced a repaired export and an unmasked control export.
- The delivered MP4 remains 1280×720, 30fps, 360 frames and 12 seconds, intentionally
  silent. All 360frames decode without errors (995,328,000 RGB bytes).
- All 360initial and final frames were inspected at contact-sheet scale; the
  defect frames and representative settled pages were also viewed at native
  size. This is broader inspection than the earlier sampled-frame pass.
- The refreshed player seeks to the opening at 0.7s and plays at rate 1 through
  the 12-second end without a media error. Progression was observed at discrete
  times; this is not a claim of continuous human perceptual playback review.
- Syntax checks pass for changed film code, player code and the new pixel
  checks. The live page and its behavior were unchanged. Rust, adapter and
  catalog source were unchanged, so their previous gates were not rerun or
  represented as fresh qualification. No catalog hashes required updating.
- No other defect was identified in these checks. This is a bounded review,
  not a guarantee of universal defect freedom or a new platform qualification.
  Windows and the waived Desktop GUI check remain untested.

Private evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-daybreak-video-review-20261028/`.
It retains `protocol.json`, `initial/source/`, all initial/final decoded frames
and sheets, masked/unmasked browser controls, `review.md`, the final review,
`repair.json`, both exports' source manifests and runtime receipts, the first
failed cross-renderer comparison, unchanged-threshold same-renderer regression
results, `final/checks.json`, `final/playback.json` and closeout.
