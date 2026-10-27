# Daybreak production record — 2026-10-28

Subsequent user-reported clipping and spacing defects were corrected in the
[follow-up video review](daybreak-video-review-20261028.md). The original
production evidence below remains historical; the specimen contains the newer
export.

Built an original responsive landing page and an editable HyperFrames reel
against the [user-selected Granger target](../planning/granger-design-target.md).
The [source and exported MP4](../specimens/daybreak/README.md) share artwork,
font, page sections and identity. No new production dependency was admitted.
Codex handled authored assets/code/browser review; Mirket's actual finite video
route rendered two disposable-project exports with explicit private Node,
Chrome, FFmpeg and FFprobe paths. The user's live `.mirket` installation was
not used or modified.

## Frozen scope and budget

Private evidence root:
`/Users/nazmiyilmaz/.codex/task-state/harness-daybreak-production-20261028/`.
`protocol.json` froze a 45-minute wall ceiling, two generated image assets,
one page and reel, one independent review and one targeted repair batch.
Initial source and pixels are under `initial/`, the actual initial video under
`export-initial/`. One comparable-capability independent reviewer inspected the
saved page/film/reference pixels, with no production-file ownership. Findings
are in `independent-review.md`; review-frame copies are in `review-frames/`.
The shared budget was not restarted for the reviewer or the repaired export.

## Finding → correction → outcome

| Finding and source | Correction | Actual recheck |
| --- | --- | --- |
| Reviewer: exposed plum stage between hero and club at 3.75–4.25s; `film.js` panel transforms. | Full-size opaque club slides over the hero; image/card enter before the shot settles. | Decoded final frames 112/120/127/135 and native frame 120 viewed. An independent pixel check detects a 156px full-height stage gutter initially and 0px finally at 4s. |
| Reviewer: two translucent pages compete at 10–10.5s; `film.js` return. | Sessions and the hidden club leave together as opaque panels above the stationary hero. | Native final frame 307 and neighbouring 300/315 viewed: clean edge, no double-exposed typography. |
| Lead: desktop wordmark descender clipped; mobile wordmark exceeds its intended inset; `site.css`, `film.css`. | Give the wordmark its full line box; reduce mobile size to fit. | Final desktop1440, mobile390 and narrow320 captures viewed; full wordmark visible, document width equals viewport width. |
| Lead: mobile group crop cuts away the fourth runner; `site.css`. | Preserve the image's 3:2 ratio on mobile. | Final390/320 screenshots show all four people. |

The reviewer found the identity coherent and the hero strongest, but judged the
supporting compositions and typography less distinctive than Granger. Those
remaining craft limits were retained rather than silently converting this into
an unlimited redesign. The uniform first decoded frame remains a deliberate
brief reveal, with a nonblank player poster. No soundtrack was requested.

## Observed verification

Platform: macOS ARM64 (`darwin-arm64`). This task used the existing development
binary at source HEAD `480489db199daf46b7fd82fda5fe93c8a562e912`; runtime identity
and executable paths are in each export's `runtime-receipt.json`. It did not
qualify an installed release or use a model-driven approval trial.

- Actual mobile keyboard journey: filter to the steady run, open its dialog,
  reject an empty name, submit a sample name, display the correct run in a
  truthful local-only confirmation, Escape and return focus to its trigger.
  Whitespace-only input is also rejected. Rapid filter changes leave the
  correct visible rows/count. See `initial/journey.json`.
- Actual reduced-motion emulation: two easy runs remain visible, count is two,
  active animations are zero and scroll behavior is auto. Emulation was reset.
  See `initial/reduced-motion.json`.
- Saved final responsive layouts at1440×1000,390×844 and320×740; no document
  overflow. The explicit width checks guard against inspecting the wrong tab
  after changing a browser-wide viewport override.
- Browser-normalized keyframes retain all16 named tracks and required authored
  transforms/masks/stroke properties. The master clock remains linear. Raw
  native readback is under `initial/accepted-keyframes.json` and
  `final/accepted-keyframes.json`.
- Both actual exports report1280×720,30fps,360frames,12seconds. Seven baseline
  frames plus sixteen final transition samples were decoded and viewed as a
  contact sheet, with defect frames also viewed at native size. All360 final
  frames decode to995,328,000RGB bytes without errors. Decoding every frame does
  not mean every frame was visually inspected.
- The final encoded MP4 plays at rate1 to its12-second end without a media
  error; chapter seeking pauses at the requested time. Browser observations
  were discrete snapshots, not continuous human perceptual playback review.
  The independent reviewer did not watch continuous playback either.
- Syntax checks pass for `site.js`, `film.js`, `reel-player.js`. Final hashes,
  decoder result and syntax output are in `final/checks.json`; initial and
  final source manifests bind each export to its inputs. These checks are not
  substitutes for the inspected pixels.

No Rust, adapter, catalog or shipped skill source changed in this production
step. Consequently their previous locked/offline build, Clippy, format, adapter
and MCP gates were not represented as new runs; the prior results remain in
[the diagnostic record](art-motion-diagnostic-20261028.md). No catalog acceptance
cell was promoted. Windows support, installed distribution fidelity and the
waived Codex Desktop GUI qualification remain untested. The broader six-area
review and design-cohort limits remain as recorded in the existing plan.
