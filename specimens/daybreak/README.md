# Daybreak

Original fictional city running-club landing page and a 12-second HyperFrames
presentation, made against the user's Granger visual target. This is a design
specimen, not an actual club or booking service.

## View and edit

Serve this directory with a local HTTP server, then open `index.html` for the
working page or `reel.html` for the actual exported film. The current task-local
preview is at <http://127.0.0.1:8768/> while its server remains running.
The standalone `daybreak.mp4` can also be opened directly.

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

The reel intentionally has different staging from the usable page. Its three
beats are hero (0–3.45s), club (3.45–6.65s), sessions (6.65–9.75s), then a hero
return. Presentation wipes are not imposed on normal page navigation. Its
opening starts on plum; the player uses a nonblank hero poster. The silent
film has no speech to caption; this is not a captioning qualification.

## Assets and rights

The two raster images were generated for this original concept using Codex
imagegen with no Granger images as inputs. The people and events are fictional.
`assets/PROVENANCE.json` records origins and hashes. Noto Sans is an unmodified,
already admitted font; its SIL Open Font License is retained in `assets/OFL.txt`.
No Granger branding, imagery, source code or soundtrack is redistributed.
SVG is used for small editable symbols and the route drawing. No new runtime
dependency or renderer was added.

## Quality evidence and limits

See [the production record](../../docs/daybreak-production-20261028.md) and
[subsequent video defect review](../../docs/daybreak-video-review-20261028.md).
The current export includes the repaired title mask and hero spacing;
`qa/check-export.mjs` retains discriminating pixel regressions.
Initial artifacts, one independent critique, one targeted repair batch and
final checks were retained. The result is still less compositionally varied
and typographically distinctive than the selected reference. It is not a
model comparison, a premium-design certification or a release qualification.
