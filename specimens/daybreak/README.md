# Daybreak

Original fictional city running-club landing page and a 12-second HyperFrames
presentation, made against the user's Granger visual target. This is a design
specimen, not an actual club or booking service.

## View and edit

Serve this directory with a local HTTP server, then open `index.html` for the
working page or `reel.html` for the actual exported film.
The standalone `daybreak.mp4` can also be opened directly.

Open `review.html` for the D6 review pack: the same corrected film, chapter
controls, before/current opening frames, representative decoded frames and
three art-direction questions. Images link to their full-size PNGs. The project
owner accepted this design direction on 2026-10-28 in the chat. That decision is
recorded against this export in `qa/review-pack.json`; it is not a release
approval or a general defect-freedom claim. The page submits no feedback itself.

- `index.html`, `site.css`, `site.js`: responsive page, keyboard-operable pace
  filters and local-only booking dialog. No data is sent or retained.
- `film.html`, `film.css`, `film.js`: editable 1280×720 composition, 12 seconds,
  30fps, 360 frames, silent. Native WAAPI tracks with absolute millisecond
  offsets and a linear master clock; curves belong to individual segments.
- `build-reel.py`: copies the page's three current sections into `film.html`.
  Run `python3 specimens/daybreak/build-reel.py` from the repository root after
  changing the page sections. This does not render a new MP4. Export `film.html`
  through the qualified Mirket video route with the stated profile and matching
  asset revisions. Do not use the live user installation for qualification.
- `reel.html`, `reel-player.js`: non-autoplay MP4 player with chapter inspection
  buttons, download link and link back to the working page.
- `review.html`, `review.css`, `qa/review/`: review presentation and six saved
  export frames. `qa/review-pack.json` binds the film, composition inputs and
  frame snapshots; run `PRIVATE_NODE qa/check-review-pack.mjs` after changes.
  A mismatch means the pack needs refresh, not that the new artwork is bad.
  This check establishes identity only; continue actual pixel/playback review.
  To also verify the saved video's profile and each current frame's timestamp,
  run `PRIVATE_NODE qa/check-review-pack.mjs ABS_DAYBREAK_DIR ABS_FFMPEG ABS_FFPROBE`.
  The earlier defective frame remains historical evidence; it is not compared
  against the current film. A changed candidate cannot inherit the recorded
  human acceptance merely by replacing its source-video hash.

The reel intentionally has different staging from the usable page. Its three
beats are hero (0–3.45s), club (3.45–6.65s), sessions (6.65–9.75s), then a hero
return. Presentation wipes are not imposed on normal page navigation. Its
opening starts on plum; the player uses a nonblank hero poster. The silent
film has no speech to caption; this is not a captioning qualification.

## Focused maintenance

This directory preserves the accepted film baseline. The subsequent
[maintenance journey](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/docs/design-production-closure-20261028.md) found
that the live page's absolute wordmark could cover the CTA at 1280×720. The
verified repair is now integrated into the page head: the wordmark occupies
normal flow below the CTA. The film builder supplies its own head, so the
accepted film source/export and its review record remain unchanged. Film
acceptance does not qualify the live page at every viewport.

Use a disposable copy for qualification so the accepted candidate stays intact.
For a copy/layout edit, change `index.html`, then run `python3 ABS_COPY/build-reel.py`;
the script resolves its own directory and rebuilds that copy's `film.html`.
Keep `site.css`, `film.css`, `film.js` and the local assets unchanged unless the
requested change needs them. Freeze the exact change and affected beats first.

Follow [saved source to checked export](../../skills/video-production/references/hyperframes-0.8.197-authoring.md#saved-source-to-checked-export)
from the harness checkout. Select `film.html`, `site.css`, `film.css`, `film.js`,
`assets/track.png`, `assets/club.png` and `assets/NotoSans.ttf` for this silent
1280×720, 30fps, 360-frame film. The generated preview, page controls and old
exports are not render inputs. Supply explicit private executable paths and
disposable state; do not use the live user installation for qualification.

Inspect the modified page/preview before export. After export, run the existing
`qa/check-export.mjs` against the new MP4, inspect the changed decoded frames and
neighbours, and exercise any affected page interaction at desktop/mobile sizes.
The frozen pixel checks do not verify arbitrary replacement copy; read its
actual saved pixels. Run `qa/check-review-pack.mjs` on the original as a control.
It should reject a copy whose film/source changed but whose old review manifest
was retained. Do not rewrite that manifest to transfer the original acceptance.

Keep the new export under its own name and record source revisions, the Mirket
operation ID, checks, remaining defects and budget used in this copy's handoff.
One targeted repair is the limit for the approved closure task. Its final report
can establish a maintenance result without claiming new user art-direction
acceptance or a general skill/model advantage.

## Assets and rights

The two raster images were generated for this original concept using Codex
imagegen with no Granger images as inputs. The people and events are fictional.
`assets/PROVENANCE.json` records origins and hashes. Noto Sans is an unmodified,
already admitted font recorded there.
No Granger branding, imagery, source code or soundtrack is redistributed.
SVG is used for small editable symbols and the route drawing. No new runtime
dependency or renderer was added.

## Quality evidence and limits

See [the production record](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/docs/daybreak-production-20261028.md) and
[subsequent video defect review](https://github.com/codemirket/harness/blob/b3068a181a674f2aff751de4ff254f8120e7f96e/docs/daybreak-video-review-20261028.md).
The current export includes the repaired title mask and hero spacing;
`qa/check-export.mjs` retains discriminating pixel regressions.
Initial artifacts, one independent critique, one targeted repair batch and
final checks were retained. The result is still less compositionally varied
and typographically distinctive than the selected reference. It is not a
model comparison, a premium-design certification or a release qualification.
