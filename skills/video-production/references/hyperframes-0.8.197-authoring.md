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
```

`DriftSans.woff2` must be included in the selected revision-bound assets passed
to `video_prepare`; the declaration's relative URL must resolve to that exact
selected file. `src: local("Helvetica")` is a system-font lookup, not a
bundled project font, and does not satisfy this contract. A source can therefore
be previewable on the authoring host yet be refused by Mirket with
`explicit_local_font_required`. Keep any fictional/font fixture local and
record its provenance rather than silently substituting a host font.

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
the HTML and emits editable SRT/VTT sidecars. Include an explicit closing
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

CSS/WAAPI-only motion may use `data-no-timeline`, but still needs a normal-speed
playback or decoded-frame check. Preserve interruption, reduced-motion and
reading-hold decisions from `motion-direction`; this reference does not impose
a universal timing curve, scene count or transition.

## Local assets and refusal cases

Keep HTML/CSS/JS, fonts, images, audio and captions in the selected project.
Use relative local references without query strings/fragments. Select every
referenced file and record its revision before `video_prepare`. The Mirket
preflight refuses missing or unselected assets, absolute/remote URLs, dynamic
fetch/import mechanisms, stale revisions and unsupported effects before a
render slot is consumed. It also requires an explicit local font declaration
for the selected profile. Do not use a CDN or assume that setting `HOME` makes
arbitrary page code private.

An asset refusal is evidence of the Mirket source policy, not an upstream
HyperFrames defect. Conversely, a prepared receipt or successful export does
not prove rights, visual intent, player compatibility or complete process
cleanup. Save independent decoded frame/audio/caption evidence and disclose
unexercised paths.

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
