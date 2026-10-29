# Video export route

The pi candidate retains **HyperFrames 0.8.197**, exact private render-only
Chrome and verified FFmpeg/ffprobe on **macOS 26.5.2 ARM64**. The
[qualification record](qualification.md) identifies the fresh installed
export, changed-source, decoded audio, cancellation and saved-pixel evidence.
Pi owns source authoring, models and host interaction. No Remotion,
personal-browser fallback or first-use download is selected.

## Author and prepare

Read the [video skill](../skills/video-production/SKILL.md) and its
[pinned authoring contract](../skills/video-production/references/hyperframes-0.8.197-authoring.md)
before writing a composition, including source-only handoff. Keep editable
HTML/CSS/JS, fonts, local media, supplied audio and caption sources in the project.
The literal `data-no-timeline` root marker, dimensions/duration, registered
animation instances and caption wire format are part of the contract.

`video_prepare` accepts one saved project-owned HTML source and an explicit list
of selected relative local assets, each bound to its SHA-256 revision. It does
not scan the project, fetch assets or start a render. Supply the format,
dimensions, rational fps, frame count and new project-scoped output destination.
Check `runtime_status` and actual operation schemas; a tool listing is not proof
that its payload is configured or the operation succeeds.

## Execute and reconcile

`video_start` launches a prepared intent once. `video_status`, `video_cancel`
and `video_result` inspect/control the same operation. After response loss or
uncertain finalization, inspect the original operation and destination before
retrying. Cancellation does not roll back already published output. A stale
source, changed runtime, missing asset, remote dependency or destination race
must refuse instead of producing a misleading success.

The five-minute preparation lifetime, installation-wide heavy-job admission,
owned scratch, process cleanup and artifact transaction bound Mirket's effects.
They are not an OS sandbox for arbitrary hostile code. The active pi workflow must obtain any required user authorization;
a denied render stays unperformed. Continue only independent authorized work.

## Formats and bounds

| Contract | Bound / behavior |
| --- | --- |
| Formats | `mp4`, `webm`, `gif`, `png_sequence_tar`; GIF is silent. |
| Inputs | Explicit selected sources total at most 16 MiB across 128 files. |
| Output | One streamed new artifact, hard 2 GiB cap; refuse oversize rather than truncate. |
| Capture | At most 18,000 frames, 1–60 rational fps, 600 seconds, 1920 pixels per axis and 1920×1080 total pixels; even dimensions. These validation limits are not performance or quality guarantees. |
| Time / scratch | Default 30-minute work deadline includes queue time; 5 GiB work-tree budget sampled every 250 ms. This is an abort policy, not reserved disk or an atomic quota. |
| PNG archive | Ordered `frame_000001.png` onward, optional audio/caption sidecars and manifest; entry identity/order/traversal checks in adapter and Rust. |
| Captions | Ordered non-overlapping cue array with exactly `start_frame`, `end_frame`, `text`; end frame exclusive. Burned captions; PNG archive also carries derived SRT/WebVTT. Supplied captions, not transcription. |

The source remains intact; output publication uses a new destination and
rechecks source/destination identity. Uncertain cleanup remains explicit.
Source validation and bounded diagnostics can explain a refusal; they do not
establish intended motion or visual quality.

## Verify the deliverable

Inspect the static composition before adjusting motion. Preview a bounded
sample, then verify the actual export with explicit FFmpeg/ffprobe: format,
frame count/order/timing, changed-source/changed-pixels, transition and final
states, supplied audio and caption content/timing. Decode and display saved
frames with an image-capable tool. Record normal-speed playback observations
and their limits. Check transparency only for the actual codec/player tested.

Use the [preview/beat procedure](../skills/video-production/references/preview-and-beats.md)
when a seekable preview is needed. Project preview controls stay outside the
exported source. A moving browser preview may freeze in export; demonstrate
corresponding named states. Numeric [motion checks](../scripts/check-motion-observations.mjs)
can expose geometry/hold/overlap defects but do not judge aesthetics.

Keep one task-level review/repair budget across production and independent
review. New specimens/delegates do not reset it. Deliver editable sources,
checked exports, actual evidence and remaining defects; a renderer exit or
receipt alone is insufficient.

## Implementation and distribution

The [Rust video contract](../src/video.rs), [session owner](../src/video_session.rs),
[export adapter](../pi/src/runtime/video-export.ts) and
[policy](../pi/src/runtime/video-policy.ts) define the executable boundary.
[Installed video tests](../tests/video-mcp-smoke.mjs) exercise actual exports,
reediting, cancellation and refusal; the release record states which ran.

Exact payload inventories and admission decisions remain under
`packaging/admission` because the runtime/assembler consumes their hashes. A
`.proposed` inventory does not itself grant admission. Current pi qualification
uses a complete disposable offline payload; it does not authorize redistribution
or changes to the user's installation. First use never substitutes a PATH
executable or acquires another runtime. See [qualification](qualification.md)
for current evidence and limits, and [admission records](../packaging/admission/README.md)
for exact payload decisions.
