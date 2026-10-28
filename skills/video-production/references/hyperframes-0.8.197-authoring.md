# HyperFrames 0.8.197 authoring contract

This reference is for the Mirket video route pinned to HyperFrames producer
0.8.197 (source commit
`f51650d8a72ed472effff89da087c7e1cf6b2c98`). It describes source authoring;
the host still owns model choice, agents, permissions and ordinary browsing.
The Mirket adapter's source preflight is stricter than the upstream package and
its refusal is not an OS sandbox for arbitrary HTML.

## Minimal local composition

The current Mirket route requires the selected entry file to contain a
composition root whose dimensions and duration match the requested profile:

```html
<!doctype html>
<html><head></head><body>
<div
  data-composition-id="demo"
  data-no-timeline
  data-width="640"
  data-height="360"
  data-duration="2"
></div>
</body></html>
```

For a CSS/WAAPI or other non-GSAP composition, `data-no-timeline` is the
supported declaration that there is no `window.__timelines["demo"]` to wait for.
Without it, this pinned producer can spend its readiness timeout waiting for a
timeline and fail with `sub_timeline_readiness_timeout`. Do not add the marker
to a composition that actually requires a registered GSAP timeline and then
claim that strict timeline synchronization was tested; use the applicable
timeline contract and a separate held-out output check. The current Mirket
preflight accepts only the explicit no-timeline profile, so a timeline-based
source must be reported as unsupported by this route until its contract is
qualified.

The root's `data-width`, `data-height` and `data-duration` are authored values,
not a post-load variable. Use a finite rational FPS and frame count whose
duration equals the root duration. The output format, dimensions and frame
count are independently checked after rendering.

The root is only one part of the Mirket source contract. A minimal selected
project also needs an explicit bundled font asset and a matching declaration,
for example:

```css
@font-face {
  font-family: "DriftSans";
  src: url("./DriftSans.woff2") format("woff2");
  font-display: block;
}
body { font-family: "DriftSans"; }
```

`DriftSans.woff2` must be included in the selected revision-bound assets passed
to `video_prepare`; the declaration's relative URL must resolve to that exact
selected file. `src: local("Helvetica")` is a system-font lookup, not a
bundled project font, and does not satisfy this contract. A source can therefore
be previewable on the authoring host yet be refused by Mirket with
`explicit_local_font_required`. Keep any fictional/font fixture local and
record its provenance rather than silently substituting a host font.

Declaring a face does not apply it. Assign the selected family to the visible
text; the injected caption overlay inherits the body's family unless authored
CSS overrides it. The static preflight checks literal local font declarations,
not computed font use or glyph fallback. Inspect the captioned export and
verify the intended face/glyph coverage before claiming font fidelity.

## Caption source and safe area

When `profile.captions_path` is selected, that JSON file is an array of cues,
not a document containing FPS, dimensions or scene metadata. Each cue has
exactly `start_frame`, `end_frame` and `text`. Frame ranges are zero-based,
end-exclusive, ordered, non-overlapping and inside the requested frame count:

```json
[
  {"start_frame": 0, "end_frame": 12, "text": "Opening message"},
  {"start_frame": 12, "end_frame": 24, "text": "Closing message"}
]
```

This example matches 24 frames; retain the real scene text and timing. Select
the caption file as a revision-bound asset. The adapter burns the captions into
the HTML; PNG-sequence tar exports also retain editable SRT/VTT sidecars.
Include an explicit closing
`</body>` so insertion succeeds; browser repair of a fragment is insufficient.
Reserve the lower caption region in the composition, then inspect the decoded
captioned export for collision with footer copy or controls. A source preview
without the burned captions cannot establish that safe area. These are Mirket
adapter requirements, not defects or universal rules of HyperFrames.

## Anime and other authored motion

If the composition uses the admitted local Anime.js **4.5.0** UMD bundle, keep
the created v4 animation instance in
`window.__hfAnime` so HyperFrames can discover and seek it at each frame:

```html
<script src="./anime.umd.min.js"></script>
<script>
  window.__hfAnime = [anime.animate("#subject", {
    x: [0, 480],
    duration: 2000,
    ease: "linear",
    autoplay: false
  })];
</script>
```

The exact local Anime bundle and the source's other script/font/media assets
must be selected and revision-bound. A successful render with an unregistered
Anime instance can leave the intended object frozen while unrelated CSS/WAAPI
content still changes. Inspect the intended object's first, middle and final
decoded frames; whole-frame hashes, status and a moving neighbor are not enough.

CSS/WAAPI motion may use `data-no-timeline`, but still needs a normal-speed
playback and decoded-frame check. This pinned runtime has separate CSS and
WAAPI adapters; native `element.animate()` handles are sought through the
WAAPI adapter. Do not confuse them with unregistered Anime instances. A
finite native-WAAPI translation was independently checked at five decoded
frames; arbitrary timers/canvas or another animation library do not inherit
that result. Preserve interruption, reduced-motion and
reading-hold decisions from `motion-direction`; this reference does not impose
a universal timing curve, scene count or transition.

## Decode with the qualified build

The currently qualified private FFmpeg build decodes saved exports with
`-fps_mode passthrough`; it does not provide the old `-vsync` option or the
`select`/`tile` filters. Use the exact runtime executable and a fresh output
directory, for example:

```text
<ffmpeg> -v error -i <saved.mp4> -fps_mode passthrough -start_number 0 <frames>/frame-%04d.png
```

Create `<frames>` first. Decode the full export and inspect the saved frames
with the host image tool, including adjacent frames around suspected defects.
Do not spend the review budget rebuilding unavailable montage filters or
installing another media stack. A contact sheet is optional; individual decoded
frames are sufficient for frame-specific judgment. Check the selected build
again if its identity changes. Decoding success is not visual inspection or
normal-speed playback evidence.

## Local assets and refusal cases

Keep HTML/CSS/JS, fonts, images, audio and captions in the selected project.
Use relative local references without query strings/fragments. Select every
referenced file and record its revision before `video_prepare`. The Mirket
preflight refuses missing or unselected assets, absolute/remote URLs, dynamic
fetch/import mechanisms, stale revisions and unsupported effects before media
processes launch. It also requires an explicit local font declaration
for the selected profile. Do not use a CDN or assume that setting `HOME` makes
arbitrary page code private.

Static refusals can include bounded `diagnostics` in `video_status` and
`video_result`: selected relative path, phase, field, reason, expected bound,
and zero-based cue index where applicable. Use those findings together; a
missing root marker and invalid captions can coexist. The list is capped and
is not an exhaustive review of the composition. Recheck after editing and
refresh the selected source revisions explicitly; do not replay an old intent.

An asset refusal is evidence of the Mirket source policy, not an upstream
HyperFrames defect. Conversely, a prepared receipt or successful export does
not prove rights, visual intent, player compatibility or complete process
cleanup. Save independent decoded frame/audio/caption evidence and disclose
unexercised paths.

## Saved source to checked export

For an existing accepted composition, edit the saved project rather than
reconstructing its artwork. Keep exact requested copy, affected beats and allowed
files explicit. Finish a static composition before changing motion; use the
existing `motion-direction` shot card when a new sequence needs direction.

1. Check the selected runtime/release and freeze the source/profile. In this
   harness checkout, `scripts/create-video-seed.mjs` can create new minimal
   source with a caller-selected local font/notice; it is unnecessary for an
   existing composition. Repository utilities are not installed MCP tools or
   files promised in unrelated projects.
2. Run `scripts/check-video-source.mjs` using an explicit private Node path,
   `--project ABS_PROJECT --manifest ABS_MANIFEST`. Its manifest contains
   `schema_version: 1`, a relative `entry_file`, the complete profile (including
   `captions_path: null` when absent), and `sources` entries with relative
   `path`, `sha256:`-prefixed `revision` and byte count `bytes`. Select the entry
   and every referenced asset; regenerate revisions after an edit. This uses
   the checkout's compiled adapter and reports static validity only.
3. Inspect the affected source state in Codex's browser before a full render.
   For CSS/WAAPI frame inspection in this checkout, the existing
   `scripts/create-motion-preview.mjs --project ABS_PROJECT --entry ABS_HTML
   --motion ABS_MOTION_JSON --output ABS_NEW_PREVIEW` uses the profile described
   in [preview and beat correspondence](preview-and-beats.md). The HTML, motion
   JSON and new preview must be inside that canonical project. Keep its controls
   outside the selected render assets; other animation types need their own
   qualified preview. A source preview is not saved-export inspection.
4. Call `video_prepare` with those revisions, an explicit project root and a
   fresh destination. Retain the returned operation ID before `video_start`.
   Inspect `video_status`/`video_result` for that same job after a lost response;
   do not submit a replacement render while the original effect is uncertain.
   Report unavailable routes or host approval denial rather than bypassing them.
5. Probe and decode the actual output using the selected private FFprobe/FFmpeg
   paths and the recipe above. Inspect changed beats and their neighbours,
   exercise affected UI separately in Codex's browser, and record playback
   observations. Repair only a demonstrated defect within the shared task budget,
   then repeat the affected checks. Preserve the previous artifact and findings.

A short handoff in the existing project README is enough: `completed: CTA edit
and static preflight; source: <manifest/revisions>; next: inspect frame 90 and
start the prepared export; operation: <ID or not prepared>; defects: <known or
none observed>; budget: <used/remaining and repairs used>`. Record native turn
completion, scope validity, export checks, visual findings and human acceptance
separately. A saved file after a timeout does not complete that author trial;
a new export does not inherit the previous export's acceptance.

## Exact upstream boundary

The pinned upstream [producer README](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/packages/producer/README.md)
documents Chrome BeginFrame capture, FFmpeg encoding/audio mixing, Node >=22,
Chrome/Chromium and FFmpeg requirements, and `createRenderJob` /
`executeRenderJob`. The pinned [troubleshooting guide](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/docs/guides/troubleshooting.mdx)
documents composition IDs/dimensions, local/remote media issues, animation
registration, preview/render differences and the no-timeline wait. These are
source references, not permission to install upstream tooling or switch
renderers. No upstream source code or assets are copied into this reference;
the HyperFrames packages are Apache-2.0 at the pinned source and any future
redistributed upstream-derived files must retain their LICENSE/NOTICE and
modification records.
