# M3 motion/video candidate qualification

**Current candidate evidence, not release admission.** [Decision 93](../planning/decisions.md#decision-93--restore-private-chromium-and-full-designvideo-route) restores private Chromium and editable HyperFrames HTML/CSS/JS video. The Mac fixture, closure and rights findings below are again relevant to the [revised plan](../planning/implementation-plan.md). They do not qualify a production video package, private browser reuse or either Windows target.

Observed **2026-10-25**, macOS 26.5.2 (25F84), ARM64. This qualifies a small
local authored fixture and proposes the next integration boundary. It does
**not** admit a production video package or establish the three-platform release
contract. No runtime, adapter, Cargo dependency, installed Mirket release or
client configuration was changed by this qualification.

## Decision supported by the evidence

Continue with **HyperFrames producer 0.8.197, local Anime.js 4.5.0/CSS/WAAPI,
the single selected OSS Chromium, and controlled FFmpeg/ffprobe**. This preserves
decisions 41–42, 48–49 and the Chromium-only release scope. Use the narrow local
library API; omit the upstream CLI/plugin, Studio application, cloud rendering,
media generation and downloaded templates. The dependency closure still includes
Studio server code: the library boundary does not erase that closure.

Chromium, browser animation APIs, Anime.js and FFmpeg provide established
foundations. **The assembled HyperFrames path is not yet a demonstrated mature
release runtime.** A short fixture passes on one Mac, and the current producer
still needs explicit authoring rules, controlled launch/network behavior and
packaging work. There is no measured evidence here that it is faster than the
alternatives. Reusing one Chromium payload and keeping one render worker avoids
a second browser payload and unbounded fan-out; it does not establish a
performance advantage.

| Candidate, current source snapshot | Rights and integration evidence | Result for Mirket |
| --- | --- | --- |
| HyperFrames **0.8.197**, published 2026-10-25; tag commit `f51650d8a72ed472effff89da087c7e1cf6b2c98` | Six published HyperFrames packages contain Apache-2.0 LICENSE files despite lacking npm `license` fields. Node >=22; local producer API; actual four-format fixture below. [Release](https://github.com/heygen-com/hyperframes/releases/tag/v0.8.197), [source license](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/LICENSE), [producer API](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/packages/producer/README.md). | Closest fit to the agreed editable HTML/DOM source and formats; retain as sole integration candidate, with release gates below. |
| Remotion **4.0.538**, published 2026-10-22 | Released core license restricts free eligibility and derivative resale/relicensing; current FAQ requires a Company License outside the free categories. This does not meet the selected unrestricted free/OSS runtime baseline for Mirket's recipients. [Exact license](https://github.com/remotion-dev/remotion/blob/v4.0.538/LICENSE.md), [FAQ](https://www.remotion.dev/docs/license/faq). | Keep as an engineering reference. Do not bundle it or treat a small-team free allowance as distribution clearance. Individual Remotion utilities may have separate terms. |
| Motion Canvas **3.17.2**, published 2024-12-14 | MIT; documented editor-driven frame rendering and external FFmpeg conversion. The open headless issue concerns rendering **without a browser**; its existence does not prove headless browser automation impossible. No equivalent local API fixture was run here. [License](https://github.com/motion-canvas/motion-canvas/blob/v3.17.2/LICENSE), [rendering](https://motioncanvas.io/docs/rendering/), [issue](https://github.com/motion-canvas/motion-canvas/issues/1218). | Established animation authoring option, but replacing the selected HTML pipeline would add editor/project integration and a different source model. No demonstrated improvement justifies that change. |
| Revideo renderer **0.11.0**, published 2026-07-10; npm `gitHead` `73479d620d151792edfee45cca32395fd1b60b94` | MIT, documented Node headless render API and browser preview. Renderer depends on Vite/Puppeteer; FFmpeg package adds installer packages, fluent-ffmpeg and telemetry. The repository documents PostHog counting and an opt-out. [Pinned license](https://github.com/midrender/revideo/blob/73479d620d151792edfee45cca32395fd1b60b94/LICENSE), [API](https://docs.re.video/api/renderer/renderVideo/), [repository](https://github.com/midrender/revideo). | A credible API-oriented OSS alternative if HyperFrames fails its required gates. It uses a different scene model and has its own provisioning/telemetry boundary; no executable comparison or speed claim was made. Do not package it as a fallback. |

CSS/WAAPI are the default for interface motion. Anime.js 4.5.0 is MIT and has no
npm runtime dependencies. Motion 14.6.0 is MIT but brings framer-motion/tslib;
it was inspected as an interface library, not installed or qualified in this
video fixture. Preserve the host application's existing animation conventions.
[Anime.js release source](https://github.com/juliangarnier/anime/tree/2c9cf8ea00329f6768c7d7902252ed977d75ce42),
[Motion package metadata](https://registry.npmjs.org/motion/14.6.0).

## Exact candidate closure and rights boundary

All qualification files are retained under
`~/.codex/task-state/harness-rewrite-implementation/motion-video-qualification-20261025/`
(referred to below as `<evidence-root>`). They are private development evidence,
not release payloads. `package-lock.json` is 71,085 bytes, SHA-256
`973257cb202158e74948e49e05d07418ce59408587b1eefe1b7baa17be82aa9c`.
It records **138 resolved package entries**, including platform optionals;
**88 packages** installed on this Mac with lifecycle scripts disabled.
Installed regular npm bytes total **86,891,937**, excluding runtime, browser,
media executables, source archives and fixture outputs. This is not an installed
release-size estimate.

The exact entry-by-entry versions, integrity, licenses and platform selectors
are in that lock and `evidence/closure.json`. Principal runtime edges are:

| Boundary | Resolved identity and implications |
| --- | --- |
| Local renderer | `@hyperframes/{producer,engine,core,parsers,lint,studio-server}` **0.8.197**. All six embedded LICENSE files are 10,763 bytes, SHA-256 `4259155fb06f127687ee7b0a8a3682d45132db0f2da26cbc0b7a2d1e796436b8`. Root source tag is source evidence; npm omits `gitHead`, so this is not an independently attested source-to-package build. |
| Browser integration | `puppeteer` / `puppeteer-core` **25.16.0**, `@puppeteer/browsers` **3.2.6**. The full Puppeteer package has a browser-downloading postinstall; it was skipped. The selected producer cannot simply be assumed to work with only puppeteer-core. |
| Compiler/server/image closure | Hono **4.13.14**, `@hono/node-server` **2.1.5**, esbuild **0.28.2**, Sharp **0.35.5**, plus the exact parser/font/image dependencies in the lock. esbuild's postinstall was skipped; its matching Mac ARM64 optional executable is present. No Bun runtime or upstream build script was run. |
| Native image closure on this Mac | `@img/sharp-darwin-arm64` **0.35.5** and `@img/sharp-libvips-darwin-arm64` **1.3.4**, containing libvips **8.18.7**. The latter package declares **LGPL-3.0-or-later**; do not substitute the general libvips LGPL-2.1 description for this binary. `versions.json` lists 28 incorporated libraries, including image/font/XML/archive codecs. `evidence/sharp-native.json` and `native-wasm-files.json` bind actual native bytes and dylib links. |
| Native image targets in lock | Sharp **0.35.5** has Windows x64 and ARM64 optional packages, each declaring `Apache-2.0 AND LGPL-3.0-or-later`; esbuild **0.28.2** also has both targets. Metadata availability is not native execution or complete notices/source evidence. |
| Animation/font assets | Local `animejs` **4.5.0** UMD bundle; existing qualified Noto Sans TTF, SHA-256 `bfb7bb691513f12e734dc346c03a03f784912432d7e3fa8e56efcf906fe86b3d`, OFL-1.1. No CDN, GSAP, proprietary font, model or generation service is included in the fixture. |
| Node | Existing explicit private **24.22.1** executable. No PATH lookup for Node. Its own admitted notices/runtime remain separate owning evidence. |
| Chromium | Historical development OSS snapshot **Mac_Arm/1706546, 156.0.8078.0**, source `0295c61ffd3fd108222e972cddf11d048524c39b`. The selected route is now [setup-acquired Chrome for Testing](m3-browser-setup-acquisition.md); this earlier test did not qualify it. |
| Media tools used here | Explicit **FFmpeg/ffprobe 9.0.2**, Homebrew revision `9.0.2_1`. This is a development selection, not installed-tool fallback or a package proposal to copy Homebrew. Full transitive `otool -L` inspection found **21 non-system files, 39,562,768 regular bytes, 20 Apple system dependencies, no unresolved links**. Absolute Homebrew links prevent treating this as a private redistributable closure. `evidence/ffmpeg-native-closure.json` records each path/hash/edge. |

HyperFrames' published artifacts still carry code for remote script/font/media
localization and GSAP CDN repair. Their presence is not a GSAP license admission;
the inspected Anime regression fixture itself loads GSAP and a CDN Anime build.
The Mirket fixture deliberately uses neither. Do not reuse upstream examples or
generators without checking their imported code, assets and rights.
[Anime adapter](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/packages/core/src/runtime/adapters/animejs.ts),
[upstream regression source](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/packages/producer/tests/animejs-adapter/src/index.html).

`npm audit --omit=dev --ignore-scripts --json` returned **zero advisories** for
this lock on the observation date. This covers the npm advisory service's
reported matches, not the native Chromium/libvips/FFmpeg codec closure or the
absence of vulnerabilities. The FFmpeg security page lists branch fix commits;
a release number or the npm result does not establish their presence in a
particular binary. Admission needs an exact native source/configuration/patch
and advisory assessment. [FFmpeg security](https://ffmpeg.org/security.html).

### Controlled media package to qualify next

Use one private source-built FFmpeg/ffprobe pair with software **libx264 + native
AAC** for H.264/AAC MP4, **libvpx-vp9 + libopus** for WebM, and built-in PNG/GIF
paths. The observed host dependencies include x264 **r3222**, libvpx **1.17.0**
and Opus **1.6.1**; these are development identities, not approved source hashes.
The exact source pins, build tools, patches and platform artifacts must be frozen
before production admission. FFmpeg's official download page identifies
**9.0.2**, released 2026-09-18; upstream distributes source rather than official
Windows/macOS executables. [Downloads](https://ffmpeg.org/download.html).

Start from `--disable-autodetect --disable-network`, enable only the agreed
local input/output codec/container/filter set, and exclude `--enable-nonfree`.
Keep `ffmpeg` and `ffprobe`; do not bring ffplay, OpenSSL, x265, SVT-AV1, VMAF,
SDL or hardware-encoder stacks into a minimum build merely because the host
binary has them. PNG needs its compression dependency; mixing/resampling,
scaling, image sequences, palette GIF, concat and the filters actually emitted
by the pinned producer must all be exercised before calling that build complete.
This is a build boundary, not a tested configure command.

libx264 makes the selected FFmpeg executable GPL; the broad host build reports
GPL-3.0-or-later because it also enables version3. Keep corresponding exact
source, configuration, changes, license texts and recipient delivery materials
for the chosen build and each incorporated library. A permissive Rust wrapper
does not relicense the executable. Codec patent rights are a separate question
from source license rights. Do not use a binary labelled “nonfree” or assume
unreviewed third-party Windows builds solve these obligations.
[FFmpeg n9.0.2 license](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/LICENSE.md),
[FFmpeg legal guidance](https://ffmpeg.org/legal.html).

## Authored fixture and actual observations

The saved editable source is `fixture/index.html`, SHA-256
`ea954e74953b92e19b25214e9229610c5a70a6830fb50622a94ddcfc0700a7d6`.
It contains a 640×360, two-second composition at 12 fps: three colored blocks
move 480 px using Anime.js, CSS and WAAPI; local Noto text includes Turkish
characters; the supplied text caption is visible over `[0.25, 1.75)`; two local
one-second PCM WAV tones start at 0.25 and 0.75 seconds with gains 0.5 and 0.25.
They are authored test audio, not generated speech. A white inset panel preserves
readability with transparent outer pixels. The rendered target and actual WebM
playback screenshot were visually inspected.

All renders selected one worker, software browser mode, screenshot capture,
SDR, no browser pool and no experimental drawElement or streaming capture.
The browser was the same existing Chromium executable used for browser
qualification; no second browser was downloaded. Instrumented render calls show
only the explicit Chromium, FFmpeg and ffprobe spawn paths, zero Node fetches,
and loopback-only page asset requests. Page requests and Node fetch were guarded
in the qualification wrapper. This is observed authored-fixture behavior, not
an OS network sandbox or proof against arbitrary hostile scripts.

| Check | Observed result |
| --- | --- |
| PNG sequence | Exactly `frame_000001.png` through `frame_000024.png`, plus `audio.m4a`. All three block left edges are 32, 92, 152, 272, 392, 452 and 492 px at sampled times 0, 0.25, 0.5, 1, 1.5, 1.75 and 23/12 s. RGBA transparency is present. |
| Burned caption | Pixel checks find no caption at frame 1, visible text at frames 4 and 13, and no caption at frames 22 and 24, matching `[0.25, 1.75)`. Supplied SRT/WebVTT text and cue timing are retained as editable sidecars; HyperFrames did not create those sidecars automatically. |
| MP4 | **55,326 bytes**, H.264/yuv420p 640×360 at 12/1, AAC stereo 48 kHz; both streams and container report 2.000 s. Complete independent FFmpeg decode succeeds; sampled decoded frames were extracted. This Chromium reports no H.264/AAC playback support and rejects it with `DEMUXER_ERROR_NO_SUPPORTED_STREAMS`. No native-player compatibility claim is made. |
| WebM | **48,429 bytes**, VP9, Opus stereo 48 kHz; video duration 2.000 s, container 2.008 s. Metadata carries `ALPHA_MODE=1`; independent **libvpx-vp9** decoding proves four channels and 31,116 fully transparent pixels in frame zero. Metadata alone was not used as alpha proof. |
| Owned Chromium WebM playback | Loaded, sought to 1.000 s and advanced during play. Canvas corner pixel is `[0,0,0,0]`; screenshot shows the page's pink background through the video margin. Preview serving required a same-origin loopback page and HTTP byte-range responses for reliable seeking. |
| GIF | **174,408 bytes**, exactly 24 frames, no audio stream, 2.000 s. Frame delays alternate 80/90 ms because GIF timing is quantized to hundredths of a second; a 12 fps label does not imply identical 83.333 ms GIF delays. Complete decode succeeds. |
| Audio timing/gain | Decoded MP4, WebM and PNG audio sidecar each yield 96,000 mono analysis samples at 48 kHz, or 2.000 s. Spectral windows show silence at 0.1 s, only 440 Hz at 0.4 s, both at 0.9 s, only 660 Hz at 1.4 s and silence at 1.9 s. Overlap amplitudes are approximately 0.194/0.097 (2:1). Peaks remain below 0.283 full scale. No listening test with real speech/music or loudness standard was performed. |
| Missing image | A local missing PNG produces strict `media_load_failed`; the final destination remains absent and no output transaction remains. The producer discovers this after capture/encode work, so Mirket should preflight prepared assets before launch. |
| Cancellation | Abort at entry to rendering yields `RenderCancelledError`/cancelled and no final destination or transaction. This is a **before-capture** cancellation check; active capture/encode, disconnect and process-death cancellation remain open. |

Two discriminating failures matter for authoring:

1. Without **`data-no-timeline`** on this non-GSAP composition root, strict
   rendering waits for `window.__timelines[id]` and fails with
   `sub_timeline_readiness_timeout`. The supported marker fixes this without
   adding GSAP or suppressing strict warnings.
2. Anime v4 instances **must be registered in `window.__hfAnime`**. A deliberately
   unregistered instance produces a successful upstream render whose Anime
   block remains at x=32 throughout, while CSS/WAAPI and captions advance.
   Status, different whole-frame hashes and a moving CSS neighbor would all be
   convincing but insufficient checks. Inspect the intended animated element.

The first upstream launch also demonstrated private-state gaps: HyperFrames
passes `--no-sandbox`/`--disable-setuid-sandbox`, and the browser's detached
crashpad handlers used the normal macOS Chromium crash directory despite a
private HOME. The two exact task-created handler PIDs were inspected and
terminated; no unrelated directory was removed. Subsequent launches explicitly
set `--crash-dumps-dir` inside task state; the final process check found no
remaining handlers from these runs. A production launch/ownership contract must
resolve these behaviors; setting HOME is insufficient.

## Smallest integration contract

Use a video worker entry in the **existing compiled TypeScript package**, owned
by the Rust job machinery. Do not add another public browsing interface,
scheduler, media daemon or custom animation renderer.

1. **Prepare:** bind the selected project, source tree/asset hashes, explicit
   dimensions, finite rational fps/duration, allowed output profile, supplied
   audio/timed captions and a new destination. Preserve editable HTML/CSS/JS,
   assets, font rights and provenance. Preflight missing files, dynamic/external
   asset references, actual input codecs and available encoders before spending
   a render slot. Asset acquisition belongs to a separately authorized
   preparation operation.
2. **Configure:** pass the release-bound absolute Node/Chromium/FFmpeg/ffprobe
   paths. `producerConfig.chromePath` is required;
   `HYPERFRAMES_FFMPEG_PATH` and `HYPERFRAMES_FFPROBE_PATH` are set by the owning
   worker, checked before import/launch and never caller-supplied executable
   discovery. Scrub inherited producer/browser/media/provider settings and use a
   private HOME, temp, profiles, cache and crash directory. Refuse missing or
   changed executables. [Browser resolution](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/packages/engine/src/services/browserManager.ts), [media resolution](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/packages/parsers/src/ffBinaries.ts).
3. **Launch:** enforce the normal heavy-job and owned-browser admission limits,
   one internal worker and no pool initially. Use
   `createRenderJob`/`executeRenderJob` with explicit config, strict warnings,
   screenshot capture and `hdrMode: 'force-sdr'`. Disable automatic GPU
   calibration/fast-capture/streaming routes for this first qualified profile.
   Unsupported requested effects remain qualification work; this profile does
   not silently satisfy their gate. No hidden browser, codec, model or asset
   download is allowed.
4. **Control effects:** the local library still fetches remote assets and
   searches shared/system tools when paths are omitted. Before production,
   qualify a supported hook or minimal source patch that enforces local-only
   asset loading and browser request policy **before navigation**, private
   launch arguments/crash state and retained process birth identities.
   Qualification monkey-patching is not the installed implementation. Clear
   or reject source/plugin mechanisms that could defeat the prepared-asset
   contract; this process is not advertised as a sandbox.
5. **Stage and finish:** render only inside the owned staging tree; independently
   check frame count/order, intended movement, alpha, complete decodability,
   stream timing, audio gains/peaks and caption Unicode/cues before finalizing a
   new destination. Use WebM for Chromium's owned audiovisual preview; retain
   MP4 as the agreed external delivery format with explicit player evidence.
   Preserve the normal source/destination race checks and uncertain-effect
   receipt rules. Upstream transactional output is not a replacement for
   Mirket's artifact commit record.
6. **Cancel/report:** connect the upstream AbortSignal to the existing bounded
   start/status/cancel/result jobs and owned-descendant cleanup. Do not hold a
   long MCP RPC or replay an unknown effect. Report exact output codecs,
   silent formats, frame/time/color/alpha profile, source identities, warnings,
   evidence and unverified paths. Serialize SRT/WebVTT from supplied cue records
   with deterministic Unicode/cue-order checks; no transcription is implied.

Keep the plan's 30-minute video deadline unless a longer bounded request is
selected beforehand. The 640×360/12 fps/two-second fixture is **evidence scope**,
not a proposed first-release maximum. 1080p typography, larger durations,
resolution/fps limits and disk reservations must be measured and frozen before
release. No universal render-time, memory or package-size claim is supported.

## Open production gates and evidence index

Production remains blocked on the exact private media build, full npm/native
notices and corresponding-source materials; the owning Chromium rights gate;
local-only pre-navigation enforcement; sandbox/launch/state/descendant behavior;
active cancellation/disconnect/race recovery; actual supplied video extraction
and seeking; nested compositions; real narration/music, fade/trim/mix controls;
seven-language text/layout and caption coverage; color management; required
1080p and long compositions; two-job resource admission and memory/disk
measurements. These are retained capabilities, not scope removed by the small
fixture.

Native **Windows 11 x64 and ARM64** execution was not available. The npm native
metadata is not qualification of Chromium/Puppeteer, codecs, DLL closure,
architecture-compatible FFmpeg/Sharp workers, paths with spaces/non-ASCII,
process ownership, cancellation or executable-in-use updates. Do not silently
make FFmpeg an x64 exception on Windows ARM64; choose and qualify an explicit
per-process architecture under the accepted policy. macOS 15, signing and
notarization were also not exercised.

Within `<evidence-root>`:

- `package.json`, `package-lock.json`, `evidence/*-registry.json`,
  `package-summary.json`, `closure.json`, `identity-summary.json`,
  `native-wasm-files.json`, `sharp-native.json`, `npm-audit.json`: exact package,
  rights, architecture and advisory observations.
- `source/`: pinned selected upstream source/licenses and retrieved FFmpeg
  legal/security pages; `evidence/source-fetch.json` records URL/hash evidence.
- `fixture/`, `fixture-unregistered/`, `fixture-missing/`, `prepare_fixture.py`,
  `render.mjs`: editable test sources and the qualification-only guarded runner.
  The final `fixture/` and its manifest own the tested revision. The current
  preparation script contains that source; `prepare_fixture-initial.py` preserves
  the initial specimen before the retained authoring repairs.
- `outputs/`, `evidence/*-job.json`, `*-events.json`, `render-*.log`,
  `export-inspection.json`, `frame-inspection.json`, `content-checks.json`,
  `browser-playback.json` and PNG captures: actual outputs, failed hypotheses,
  timing/content and visual/playback observations.
- `evidence/environment.json`, `ffmpeg-native-closure.json`, `ffmpeg-buildconf.txt`,
  `ffmpeg-license.txt`, `execution-summary.json`,
  `first-run-orphan-cleanup.json`: selected development runtime, native closure
  and exact owned cleanup evidence.

No benchmark comparison, production admission, publication, commit or native
Codex activation is asserted by this document.

## Historical controlled FFmpeg source-build follow-up — 2026-10-25

The approved-plan follow-up built one private **FFmpeg/ffprobe 9.0.2** pair on
macOS 26.5.2 ARM64 from the exact FFmpeg release archive, x264 revision
`b35605ace3ddf7c1a5d67a2eb553f034aef41d55`, libvpx **1.17.0** and Opus
**1.6.1**. This supersedes the Homebrew selection only for the new native media
experiment; the earlier HyperFrames fixture still used its recorded host pair.
No Chromium was launched and no video release was admitted.

The controlled build disables network protocols and automatic dependency
detection, statically incorporates the three selected codec libraries, and
keeps only the selected local codec/container/filter set. It ran explicit
configure and make targets, with manual task-owned staging; no upstream install
target, autogen, package-manager action or hook ran. The actual pair reports
**GPL-2.0-or-later**, with no nonfree flag. Its ARM64 Mach-O files declare macOS
15.0 minimum and link only Apple system libraries/frameworks (libSystem, zlib,
CoreFoundation, CoreVideo and CoreMedia), with no Homebrew dylib dependency.
That minimum declaration does not establish macOS 15 execution.

The executables selected for this initial source-build stage were:

| File | Regular bytes | SHA-256 |
| --- | ---: | --- |
| ffmpeg | 9,579,144 | `46cad2230b71986ab4abf7a53e5d49effdc0e7f7a79a9fbfa6ecde6ccf35311e` |
| ffprobe | 9,369,576 | `22ae4461bd19c3625e2700df7b166e8813cf4255c03cdc5c14d2defabbcfcaad` |

Two complete runs of an explicit **staged** native path passed 23 checks each:
mix supplied PCM audio, encode tracks separately, then mux H.264/AAC MP4 or
VP9/Opus WebM with timed Unicode subtitles. The 320×180, two-second, 12-fps
fixture preserved all 24 frame identities/moving-square positions; MP4 and GIF
duration was 2.000 seconds, WebM 2.008 seconds. Caption text and cue times
round-tripped; decoded audio onset was 0.25004 seconds and measured gains
matched the fixture. PNG frames decoded losslessly. GIF delays alternated
0.08/0.09 seconds and summed to 2.00 seconds. VP9 alpha decoded through the
explicit libvpx decoder as 0/129/255 from input 0/128/255; this is a tested lossy
alpha path, not browser/player transparency qualification. A source/MP4/WebM/GIF
contact sheet was inspected. Missing/network input refusal, preservation of an
existing destination and owned process cancellation/reaping were exercised.

The tests exposed material limits. A one-pass command combining two audio
inputs, H.264 and subtitles intermittently stalled and was killed/reaped at its
deadline; its cause remains unresolved. The staged path is the passing native
specimen, not proof that the intended HyperFrames/Mirket API passes. Also,
`-n` preserved an existing destination while returning exit code zero: a future
adapter must verify the expected fresh stage and contents. Early minimal-build
tests caught and corrected the internal `movtext` component name and required
GIF parser; generated-output checks were necessary beyond build success.

Exact archives, source hashes, the verified FFmpeg release signature, 22 named
notice/license/patent files, build recipe, generated configuration, native
commands/failures and local recipient source export remain under
`~/.codex/task-state/harness-rewrite-implementation/ffmpeg-qualification-20261025/`.
The 29,413,732-byte source bundle is SHA-256
`25caedf6f80ae18f206322ef7ea3b1196e1cfc94df0b3f356251528cb432f28f`.
Its four source archives verified and extracted offline from a fresh local
recipient directory. At this stage a fresh recipient rebuild and public
source-download delivery had not been tested. This initial pair and its subtitle
checks are historical; the final pair has different hashes and MOVTEXT disabled.

The [2026-10-26 FFmpeg recipient-source qualification](ffmpeg-recipient-source-closure.md)
records the final selected pair, the successful later fresh recipient rebuild,
independent source-signature/extraction/advisory checks, refreshed full
source/notice kit and checked local binary/source delivery fixture. No release
source offer or actual recipient delivery was published. Product rights/terms,
patent review, intended API/finalization, larger compositions, macOS 15,
signing, Windows 11 x64/ARM64 and combined runtime qualification remain open.
