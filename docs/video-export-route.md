# Finite authored-video integration candidate

This is a Codex-only development route, not release admission. The installed
release verifier deliberately supplies no video payload. The current Chromium
rights blocker, combined npm/native notices and corresponding source, exact
media build, platform and capability gates still govern release packaging.
The earlier [candidate qualification](m3-motion-video-qualification.md) remains
the evidence for its recorded runtime; it does not establish this new adapter.

`video_prepare` selects an existing project-owned HTML composition, its SHA-256
revision, and an explicit list of relative local asset paths and revisions.
There is no implicit project scan or acquisition. HTML/CSS/JS, local font/media
assets, supplied audio and caption JSON stay editable in the project. The
author supplies the root's `data-composition-id`, `data-no-timeline`, dimensions
and duration; they must match the requested rational fps and frame count.
Anime.js instances must still be registered in `window.__hfAnime`.

`video_start` starts that issued intent once. `video_status`, `video_cancel` and
`video_result` inspect/control the same operation. After a lost response, inspect
its receipt. A different/disconnected owner cannot turn a receipt into a new
render. The five-minute preparation expiry, installation-wide two heavy slots
and eight waiting permits, existing JobState, OperationStore issuance, private
scratch ownership and ArtifactPlan transaction remain the owners. Cancellation,
deadline and disconnect close the finite owned process group; an uncertain
cleanup retains the admission marker. A source or destination race refuses
publication. The process boundary is not an OS sandbox.

The prelaunch receipt is persisted before the heavy-slot marker is created, so
ordinary receipt-lock contention cannot strand a marker for work that never
launched. A terminal status remains protected while publication effect or child
cleanup is uncertain. It becomes retirable only after a known effect and
verified scratch disposal, or a confirmed prelaunch cancellation.

## Explicit candidate limits

The video-specific artifact path streams one output file with a **hard 2 GiB
cap**; the general artifact path retains its 20 MiB cap. Oversized output is
refused; the source remains intact and a new destination is not published.
The maximum dimension/frame/fps/duration controls are validation bounds, not a
promise that a 1080p or ten-minute composition will fit or has been qualified.
Publication rechecks the source and destination, streams into an exclusively
owned sibling stage, and links without replacement. A known-owned partial
stage is removed after cancellation or a write failure; uncertain ownership
or deletion retains the path for inspection. There is no directory-rename
fallback or unbounded artifact publisher.

Selected sources total at most 16 MiB across 128 files. Capture is bounded to
18,000 frames, 1–60 rational fps, 600 seconds, 1920 pixels per axis and 1920×1080
total pixels with even dimensions. The default work deadline, including queue
time, is 30 minutes. A 5 GiB private work-tree budget is sampled every 250 ms;
it is an abort policy, not an atomic filesystem quota. A short-lived excess
between samples remains possible. Chromium singleton links/sockets are counted
as entries and never followed by that disk scan. Source/artifact reads separately
require canonical regular files.

Output profiles are `mp4`, `webm`, `gif` and explicitly `png_sequence_tar`.
The last publishes one uncompressed `.tar` artifact. It contains ordered
`frame_000001.png` onward, optional `audio.m4a`, supplied caption sidecars and a
deterministic manifest binding every entry's name, size, SHA-256, dimensions,
frame count and fps. Both the adapter and Rust validate the archive; traversal,
duplicate, nested, unexpected and out-of-order entries are refused. GIF is silent.

Caption JSON is a selected UTF-8 asset containing ordered non-overlapping
`{ "start_frame": 3, "end_frame": 21, "text": "Hazır · yerel kaynaklar" }`
cue records in an array. The worker derives burned captions in its private copy;
PNG tar also includes deterministic SRT and WebVTT. These are supplied captions,
not transcription. Semantic caption placement, complex typography, narration/
music quality, audio gain/synchronization, transparency and player compatibility
require content-specific checks beyond a successful decode.

## Exact private runtime and source patch

The existing adapter npm package adds exact `@hyperframes/producer` 0.8.197.
The combined lock records 260 resolved package entries including development
and platform optionals. It changes no existing direct version pin. All 6,128
files in the prior admitted design runtime were compared and remained byte
identical; this does not admit the additional video closure. The proposed
[video inventory](../packaging/admission/m3-video-candidate.json.proposed) is
explicitly non-admitted. All six HyperFrames packages have null npm license
metadata but identical 10,763-byte embedded Apache-2.0 LICENSE files, SHA-256
`4259155fb06f127687ee7b0a8a3682d45132db0f2da26cbc0b7a2d1e796436b8`.
This is positive permissive-license evidence with notice/modification conditions;
it is not completed recipient delivery or attested source-to-package provenance.
Apache-2.0 does not itself require a corresponding-source offer. Its license,
applicable upstream attributions and any applicable NOTICE text must reach the
recipient, and each modified file must identify the modification.
The actual Mac process loads `@img/sharp-darwin-arm64` 0.35.5 (Apache-2.0) and
`@img/sharp-libvips-darwin-arm64` 1.3.4 (declared LGPL-3.0-or-later), containing
libvips 8.18.7. Its `versions.json` names 28 components including libvips; it
does not enumerate the complete incorporated closure. These are installed native
dependencies, not merely entries for other platforms in the lock. esbuild's
Darwin ARM64 0.28.2 executable is also present (MIT). Upstream sharp-libvips
1.3.4's third-party notice table includes LGPLv3 components, Cairo under MPL-1.1
and permissive components. Its recipe patches native sources and regenerates
librsvg's Rust lock after changing features. The release gate therefore needs
the complete elected license/attribution corpus and exact corresponding native
sources, patches, final Rust dependency closure and build scripts; pristine
release links and the 28-name list are insufficient. The LGPL modified-library
or relink/rebuild/install path must also be demonstrated. The existing release
verifier's rejection of changed payload bytes does not establish that right.
The selected x264-enabled FFmpeg executable is GPL-2.0-or-later; its separate
recipient source and notice obligations remain required.
The detailed [Sharp/libvips source-closure review](video-native-source-closure.md)
records the verified inputs and incomplete recipient rights/install path.

There is no npm postinstall patch or runtime monkey patch. After a locked,
scripts-disabled development dependency extraction and TypeScript compilation,
the explicit `adapters/scripts/prepare-video-policy.mjs` step validates every
original hash in `video-policy-inputs.json` before writing any change. It patches
eight exact HyperFrames/Puppeteer files and records original/patched hashes.
The video leaf refuses a missing or changed patch receipt. The patches add
explicit source hooks for private launch arguments/profile/crash state,
non-detached Puppeteer children, request interception before navigation,
canonical scoped file serving and guarded Node/font fetches. Only an origin
registered by the owned server constructor is eligible for Node fetch; another
loopback port and redirects are refused. The browser permits that same literal
127.0.0.1 origin. Dynamic network/worker source mechanisms and missing/local
asset escapes are refused. These are application controls for authored sources,
not a claim of containment for arbitrary hostile code.

Each of the eight modified Apache-2.0 files carries a prominent, stable
`Modified by Mirket` notice. A receipt-backed unchanged assembly rerun preserves
the same bytes. A patched file without its matching receipt, or any other input
drift, refuses assembly.

Node, Chromium, FFmpeg and ffprobe must be explicit canonical payload files;
there is no PATH/system/browser-cache discovery. Inherited provider, producer,
browser and media settings are removed. Executable identities are captured on
the first video preparation and checked before every launch, so MCP startup
does not hash Chromium/media payloads. Strict warnings, one worker, SDR,
screenshot capture, no browser pool, no live browser streaming and no GPU calibration form
the candidate profile.

The leaf independently probes frame count/dimensions/codecs/duration and
completely decodes the output through the explicit private FFmpeg. PNGs are
decoded before archiving; Rust rechecks archive identity and frame dimensions.
The final frame plus exact artifact/decode evidence and process-group cleanup
decide success. Direct-child exit status is not asserted: the unreaped child
retains the group identity until cleanup, and a group kill after its final frame
could legitimately change that status. No hash/exit/metadata claim substitutes
for intended-motion, caption or visual inspection.

After a persisted successful/failed outcome and proven child cleanup, the owner
freezes and rechecks an exact private file/directory inventory before removing
scratch. Unexpected entries, links, special files, changed bytes or ownership
refuse cleanup. Uncertain publication/cleanup keeps recovery evidence. There is
no path-only recursive deletion and no automatic replay.

## Observed checks and remaining gates

Focused source tests exercise rational caption timing/Unicode, deterministic
archives, asset staleness/missing files, remote/dynamic references, scoped paths,
unowned loopback origins and pre-navigation interception. Rust process tests
use an explicit private Node and refuse a malformed final frame and corrupt
stage after confirmed owned-group cleanup. An opt-in native fixture selects
all runtime paths explicitly; without those environment variables it does not
claim native execution.

The first intended private-media run exposed float PCM audio intermediates and
the `.m4a` (`ipod`) muxer absent from the earlier native candidate. A subsequent
run reached browser capture and exposed JPEG frame decoding and the `setparams`
filter used by the producer's MP4 path. The original source archives and private
pair are preserved. The bounded follow-up adds only those exact components to
the same source-built candidate; no second media engine, Homebrew dylib or
network feature is substituted. Follow-up build and intended-API results live
in external task state under `ffmpeg-qualification-20261025/video-followup-20261025/`
and `video-integration-20261025/`.

The bounded MP4 follow-up passed on macOS 26.5.2 ARM64: strict HyperFrames
capture with supplied audio, H.264/AAC metadata, all 24 expected frames at
640×360/12 fps over two seconds, complete independent decode, exact artifact
identity and process-group cleanup. The native test took 5.59 seconds; this is
one fixture observation, not a performance claim. That run deliberately selected
only MP4. It did not exercise this assembled route's WebM/GIF/PNG tar,
intended-motion pixels, source-change output, caption burn, audio gain/sync,
alpha or player playback checks.

The later disposable MCP path passed preparation, start, receipt, private MP4
render and saved-file publication with verified scratch cleanup. Repetition
initially exposed intermittent `video_export_failed` results. The sampled
work-tree scan could observe a transient Chromium directory, then fail with
`ENOENT` when that directory vanished before traversal; unlike disappearing
files, disappearing child directories were not tolerated. The scanner now
ignores only that race below its existing root and continues to fail on a
missing root or other errors. Fifteen consecutive complete MCP runs then
passed with byte-identical 55,329-byte MP4 output and the same source-preserving
receipt. This supports the repair for the observed fixture but does not by
itself qualify other formats, large workloads or release reliability.

The same saved-source fixture also passed one complete real MCP operation for
each selected profile on macOS ARM64: MP4 (55,329 bytes), WebM (48,443 bytes),
GIF (174,408 bytes) and PNG-sequence tar (431,104 bytes). Each operation passed
source-revision refusal, saved artifact identity and verified scratch cleanup.
The direct native adapter qualification separately decoded all four formats and
checked the first/last PNG frames for the intended moving element. These are
small controlled fixtures, not general content or player-compatibility claims.
An additional real MCP cancellation run reached active private scratch,
cancelled the operation, verified that no artifact was published, preserved the
project source and reported confirmed owned cleanup. This covers the sampled
in-flight path, not every process-failure or lost-owner interleaving.

The video-specific streamed publisher then passed a real MCP
`png_sequence_tar` result of **33,396,736 bytes**, beyond the general 20 MiB
cap. The saved archive was reopened with the expected SHA-256 and decoded
frame evidence, and its private scratch was cleaned. A separate MP4 publication
and an active MP4 cancellation also passed after the partial-stage cleanup
repair; cancellation published no artifact and verified cleanup. Source and
process evidence is retained in external task state under
`video-file-backed-20261026/`. Focused tests cover a partial-copy cancellation,
an injected disk-write failure, externally changed partial data that must be
retained, and disposal of a complete stage without rereading a potentially
2 GiB body on the async executor. These fixtures establish the bounded path,
not a disk reservation or general long-video fidelity.

| Final development pair | Bytes | SHA-256 |
| --- | ---: | --- |
| FFmpeg 9.0.2 | 9,647,448 | `8374f88f171b233c072da4acbf9058e9b563ce048d9300edabbe6036f616505e` |
| ffprobe 9.0.2 | 9,454,392 | `8c6f1213081e3805be3e2d69b9db85ff291235771b76b4f35bd64bf5ad280a1c` |

Those exact files are development selections, not admitted release payloads.
The refreshed final FFmpeg source/notice/recipe bundle is 29,422,938 bytes,
SHA-256 `90a1aaf8f7b9295cc39a0eff1d558aabc8e23279c635cc7b0b29acb95d03f3c6`,
retained under external task state `ffmpeg-recipient-final-20261025/`.
Its manifest verification, offline extraction and corruption refusals passed.
A fresh macOS ARM64 recipient command completed extraction/rebuild in 96.729
seconds, with about 54.6 GiB minimum free under the sampled 25 GiB floor. All 12,409
regular source files stayed unchanged, the complete component configuration
matched, and the remaining configuration differences were recipient paths.
A small H.264/decode and PCM-float/AAC-M4A fixture produced byte-identical media
with selected and rebuilt tools. Executable hashes differ; no byte-for-byte
binary reproducibility or general media fidelity claim follows.

The bundle preserves the exact final PCM-float, M4A, MJPEG and `setparams`
selection and the full FFmpeg/x264/libvpx/Opus sources/notices. It also preserves
a limitation: the final configure receipt spells `mov_text`, leaving MOVTEXT
encoder/decoder disabled. The older pair's timed mov_text qualification does
not establish that feature for this pair. No selected binary was changed here.
The bundle remains local: actual complete-source delivery or a valid
offer/download route still needs recipient verification before admission.
Retain the libvpx/Opus notices and patent-grant texts alongside FFmpeg/x264
licensing; this evidence does not certify codec patent rights.
No other format/platform is inferred from the MP4 test. The failed concurrent
Rust compile during another module's integration is retained separately from
the video runtime results; it did not run the native fixture.

An intended-API retest on 2026-10-26 used this exact final private media pair,
the retained development Chromium snapshot and the existing two-second source.
One disposable MCP `video_prepare`/`video_start`/`video_result` operation passed
stale-source refusal, source preservation, saved-file publication, receipt
identity and verified scratch cleanup without an implementation repair. The
55,329-byte MP4 is SHA-256
`d406f6917b441a6862179325887318316062f5d9bed6c8dbd3935012e5440f31`.
Independent probing and complete decode confirmed H.264/AAC, 24 frames at
640×360/12 fps and 2.0 seconds. Decoded pixels placed the intended Anime.js
element at x=32 in the first frame and x=492 in the last; inspected first,
middle and last frames showed the three motion tracks and timed fixture caption.
The build-input guard, locked adapter compilation, ten video/policy tests and
focused locked Rust checks passed, including malformed-result/corrupt-stage
refusal with the explicit private Node. Complete RPC frames, editable sources,
output, runtime/source hashes and cleanup evidence remain in private task state
`video-intended-api-retest-20261026/`. This was one small MP4 observation on
macOS 26.5.2 ARM64; it adds no format, audio-gain/sync, playback or release claim.

A separate 2026-10-26 disposable MCP `png_sequence_tar` run selected a
project-owned `captions.json` cue over frames 3–20 at 12 fps. Its editable HTML
source removed the fixture's old inline caption so the supplied cue was the
only caption. The saved 414,720-byte archive has SHA-256
`78e72109866db8065df95de682390e7b60f9d5692b5a530595cc89d22a93b509`;
the sidecars preserved `Hazır · yerel kaynaklar` and the exact 0.250–1.750 s
cue. The private FFmpeg decoded frames 1, 4 and 22. A fixed lower-frame strip
changed in 11,993 RGB pixels while the cue was active and in zero pixels after
it ended. Preparation also refused a stale source revision; publication and
scratch cleanup were confirmed. The first verification command tried an
`image2pipe` demuxer absent from this deliberately reduced FFmpeg, so the
checked script now reads saved PNGs through its selected `image2` demuxer.
This is evidence for supplied caption timing and pixels in the development
route, not a general typography, accessibility or player-compatibility result.

The same disposable MCP fixture then imported the existing HTML composition,
rendered it as MP4, edited its visible heading in the project source and
rendered a second new MP4. The original output retained SHA-256
`d406f6917b441a6862179325887318316062f5d9bed6c8dbd3935012e5440f31`;
the edited output has SHA-256
`f99505b3db53f6468ac56ad45c85f5173fd02346b1ee3fbfdd1b88b24122cc47`.
Independent FFmpeg decoding found 3,434 changed RGB pixels in the first
frame's heading region. Both operations had separate saved-file receipts,
preserved their selected source revision and confirmed private scratch cleanup.
This verifies a visible source edit through the integrated development route;
it does not establish arbitrary imported-composition compatibility.

The same disposable MCP path passed an MP4 audio timing/gain fixture. The
project source specifies a 440 Hz voice from 0.25–1.25 seconds at volume 0.5
and a 660 Hz music tone from 0.75–1.75 seconds at volume 0.25. Independent
decoding through the selected private FFmpeg yielded exactly 96,000 mono PCM
samples at 48 kHz. 0.1-second windows before, during and after the
cues found only the intended frequencies; their overlapping amplitudes were
0.137307 and 0.068643, a 2.0003 voice/music ratio. The real MCP operation
again passed stale-source refusal, source preservation, receipt identity and
verified scratch cleanup. This bounds one supplied-audio case on macOS ARM64;
it does not qualify arbitrary assets, subjective mix quality or player playback.

Release remains blocked on the retained Chromium and media rights/source gates,
combined dependency recipient delivery, complete format/audio/caption/content
qualification, representative long renders and resource measurements, native
macOS 15 and Windows 11 x64/ARM64 ownership/execution, and actual Codex Desktop
activation. The existing user installation and client settings were untouched.
