# Scroll media and scene verification

Use when scroll position drives a substantial image/video scene or when reviewing
its rendered behavior. Ordinary controls remain in `motion-design`; route/history
semantics remain in `page-transitions`. Reuse the project's implementation and
capture tools. These techniques do not require a scroll engine, generated footage,
pinning or an additional verification framework.

## Distinguish progress, seeking and visible media

Keep evidence for these different facts separate:

| Fact | Useful observation | What it cannot establish |
| --- | --- | --- |
| Requested position | Latest scene progress and target media timestamp | A seek occurred or pixels changed |
| Seek completion | Media `seeking` becomes false / `seeked` fires | The intended frame is visibly composited |
| Decoded frame submitted | Supported `requestVideoFrameCallback` metadata, including `mediaTime` | Visibility through the actual crop, opacity, overlays or poster |
| Visible scene | Inspect the composed output at that target | Smooth normal-speed playback or untested device behavior |

Register a supported video-frame callback before requesting a new frame, and
compare its `mediaTime` with the current target. Derive tolerance from the encoded
frame timestamps and required accuracy; do not demand equality with an arbitrary
continuous scroll time. The callback is a compositor-submission signal, not proof
of a pixel reaching the user's eyes. It can also arrive late. An already displayed
matching frame may be reused; do not wait forever for an event when no new frame
is needed. If this API is unavailable, record the weaker signal and verify the
actual output in that target browser; a timer cannot upgrade evidence strength.

Use latest-intent ownership for asynchronous work: a scene/media-source generation
and the current desired timestamp. Coalesce rapid input toward the newest target
instead of building a queue of obsolete seeks. On reversal or source replacement,
ignore obsolete readiness callbacks and validate the returned frame against the
current target. A generic `seeked` event has no application request ID. Cancellation
and callback ownership must follow the existing scene lifecycle.

For example, a request for 0.8 progress followed by 0.2 invalidates permission for
the old result to activate initial media or mark the scene settled. A callback for
0.8 is evidence of the old frame only. Track desired and displayed timestamps
separately; a pending target is not settled merely because an older frame is visible.

Keep semantic content and controls visible in base HTML/CSS, including no-script
and failed-media states. Choose a complete static poster/composition for reduced
motion at load and when the preference changes while mounted; stop decorative
seeking and invalidate pending completions. Avoid fetching optional decorative
clips when the static treatment is selected. Restore deliberate state when motion
is allowed again, respecting user pause. A CSS change alone does not stop a JS
media controller. See [scene lifecycle](scene-mechanics.md#recompute-deliberately-review-in-real-time)
and [surface lifecycle](surface-and-procedural-craft.md#canvas-2d-a-drawn-layer-not-the-whole-interface)
for listener/frame ownership, suspension and disposal.

For first activation or replacement of a media source, keep a complete poster or
compatible prior composition until the replacement is ready. During ordinary
same-source seeks, keep the last valid displayed frame through a bounded wait;
do not reset to the poster on every scroll input. Repeated poster swaps can flash
even when endpoint captures pass. If readiness fails or the wait expires, apply
the defined recovery policy: retain a valid static frame or restore a complete
fallback, and report that degraded state separately from successful moving-media
verification. Handle denied playback, download/decode failures and incompatible
source changes without hiding content. Do not silently play audio or repeatedly
retry playback to bypass browser/user policy.

## Encode for the actual viewing relationship

Keep the editable source separate from delivery encodes. Inspect the final file,
including its frame timestamps, dimensions, crop, color and first/last visible
frames. A poster should match the intended resting frame and responsive crop; it
need not always be frame zero. Mobile may need a distinct composition or a static
poster rather than a smaller version of the desktop clip.

Inter-frame codecs can require decoding dependencies to reach a requested frame.
A shorter keyframe interval can reduce that work at a cost in compression; it is
not a universal cure for seeking latency. Compare representative encodes with
the actual browser, device, network/cache state, crop and scroll/reversal pattern.
Choose codec, resolution, frame rate and keyframe spacing from that evidence.
Do not adopt a fixed GOP, file-size or frame-rate rule from one reference site.
MP4 `faststart` changes metadata placement for startup; it does not prove responsive
scrubbing. Check the required encoder/filter capabilities directly, not by counting
how many unrelated filters a tool exposes. Use the existing approved toolchain.

When two clips must join, compare the outgoing and incoming **delivery** frames in
the actual scene. Re-encoding, color handling and responsive crop can introduce a
seam even when source anchors match. Inspect normal playback and reversal through
the join for position, brightness, motion direction and velocity continuity.
Frame similarity is one diagnostic, not aesthetic acceptance. A deliberate cut,
separate still or simpler scene may serve the brief better than an invisible join.

## Sample the scene's geometry, including its boundaries

Identify each scene's entry, active range, meaningful beats and exit from the real
scroller and layout. Keep the observed scroll offset alongside scene progress and
media timestamp. Sample both sides of scene/clip joins, pinned entry and release,
and interesting interior states. Evenly spaced whole-document captures can miss a
short scene or shift every comparison when unrelated content changes.

For a simple vertical document with no inset, let `T` be the scene's document top,
`H` its height, `V` viewport height and `y` document scroll offset:

- A natural-flow scene travels through the viewport from `T - V` to `T + H`.
  Its visible-interval progress is `clamp((y - (T - V)) / (H + V), 0, 1)`.
- A viewport-height sticky stage inside a taller wrapper is pinned from `T` to
  `T + H - V`, under those assumptions. Its pinned progress uses `H - V` only
  when that value is positive. Also inspect the visible entry/exit outside it.
- If `H = V`, pinned travel is zero. Do not disguise this by dividing by one pixel;
  choose the natural-flow mapping, a finite trigger or a static composition.

For example, `H = V = 800` gives 1600px of natural-flow passage and no pinned
travel. That distinction prevents a reveal from collapsing to an instantaneous
jump. These formulas are illustrative: nested scrollports, sticky offsets,
horizontal/logical axes and transformed ancestors require their own coordinates.
Recompute when geometry changes as described in [scene mechanics](scene-mechanics.md).

Use reproducible scene-relative positions for comparisons, then add unscheduled
interior stops and live forward/reverse scrolling. Include states during fades,
source loading, seeking and responsive rebuilds. A user can stop between planned
capture points; never tune a defect to fall between samples. Keep natural rest or
reading intervals when intentional—unchanged pixels alone do not prove dead scroll.

For an endpoint comparison, await the intended target with a bounded timeout and
record which readiness signals were observed. Inspect transient states separately;
waiting for settling must not erase evidence of blank flashes or stale frames.
Use a short diagnostic clip with distinguishable frames when a nearly static
source would hide a frozen decoder, then verify the actual delivery footage too.
Normal-speed interaction establishes timing behavior; stills cannot establish
smoothness. Emulated viewport dimensions do not establish a physical phone's
decoder, touch, power-policy or playback behavior.

## Check contrast in the actual composition

Inspect text, controls and focus indicators against the changing media beneath
them, including the weakest intermediate states and responsive crop. Keep intended
scrims, parent opacity, translucent fills, filters and blends in the measurement.
Hiding an entire text container can remove its protective background and measure
a different scene. If isolating glyphs, preserve the rest of the compositing stack
and verify that the capture method does so. Keep layout, media frame, color space
and pixel scaling consistent; a lossy/downsampled image can miss a small bright
region or distort a result near a threshold.

A background alpha above 0.5 is still translucent. As a simple sRGB example, white
text on 51% black over white has a resulting gray background near `#7d7d7d`, around
4.1:1 contrast—not the 21:1 obtained by treating the fill as opaque black. This
illustrates the error; actual filters, color spaces and stacked transparency need
the real composition. Apply the appropriate text size/weight and non-text criteria
instead of one universal threshold. Do not exclude a readable-state requirement
merely because an element is below a chosen opacity cutoff. Partially transparent
text needs its effective foreground considered too.

Record source element/state, frame or scroll position, method and limitations for
findings. Fix the observed legibility issue, potentially with a stable reading
surface or different composition; do not move a fade between screenshot samples.
Automated ratios, successful builds and a contact sheet do not replace inspection
of live behavior or human visual acceptance.

## Sources and adaptation boundary

Adapted techniques from MIT-licensed `nateherkai/scroll-craft`, revision
`75d81f74e83692add18cd7a8a8e078b8a887a579`: scene-relative inspection and media-aware
diagnostics in [shoot.mjs](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/scripts/shoot.mjs),
the device/verification concerns in [verify.md](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/references/verify.md),
and encoding, poster/crop and seam choices in [assets.md](https://github.com/nateherkai/scroll-craft/blob/75d81f74e83692add18cd7a8a8e078b8a887a579/plugins/nateherk-design/skills/scroll-craft/references/assets.md).
No upstream runtime or helper is required. Its seek/timer-based painted flag,
opacity-based contrast exclusions, sampler-driven fade advice, fixed encode rules
and provider/workspace integration are not retained. The geometry example and
corrected evidence distinctions here are Mirket guidance. Source licensing does
not grant rights to unrelated media or external services.

Primary references checked 2026-10-13; verify target implementation support:

- [HTML media seeking](https://html.spec.whatwg.org/multipage/media.html#seeking)
  defines seeking events; [video-frame callbacks](https://wicg.github.io/video-rvfc/)
  defines submitted-frame metadata and its timing limitations.
- [FFmpeg libx264 options](https://ffmpeg.org/ffmpeg-codecs.html#libx264_002c-libx264rgb)
  and [MP4 format options](https://ffmpeg.org/ffmpeg-formats.html#mov_002c-mp4_002c-ismv)
  describe encoding controls; neither establishes a target device's performance.
- [Alpha compositing](https://www.w3.org/TR/compositing-1/#simplealphacompositing)
  and [WCAG contrast](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html)
  supply the composition/contrast basis. Measured pixels and applicable criteria
  still need to correspond to the actual rendered state.
