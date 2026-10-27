# Sharp-free development video export — 2026-10-26

**Saved development MP4, WebM, GIF and PNG-sequence tar artifacts passed
independent checks on macOS 26.5.2 ARM64.** This verifies the reduced HyperFrames worker with the existing private
Node 24.22.1, development Chromium snapshot and FFmpeg/ffprobe 9.0.2. It does
not admit an installed Video component, prove network isolation or repair the
Chromium snapshot's free-license/source gate.

## Failure isolation before the successful export

Two previous launch refusals remain unchanged in the [retry report](m3-video-sharp-free-export-retry.md).
The fresh resource guard passed at **Normal=1, 78% free memory**. The first
actual worker attempt failed before readiness because its disposable Node
permission wrapper omitted file-read grants for the explicitly selected
Chromium, FFmpeg and ffprobe paths. A separate diagnostic reproduced the denied
`realpath` operation. With exactly those file reads granted, all three path
checks, eight private policy file hashes and private directory creation passed.

The next attempt reached Chromium but its page closed. A matched Puppeteer/CDP
probe showed repeated `sandbox initialization failed: Operation not permitted`
messages from Chromium's child processes under the test's outer macOS
`sandbox-exec` profile. Without that outer profile, the same private Node
wrapper, browser flags and Chromium binary passed `newPage`, service-worker
bypass and request-interception setup. Mirket's product process path has no
outer `sandbox-exec` layer; the successful development export used that path.
These probes identify the test envelope failure. They do not establish offline
network containment of the export; the worker's patched application policy and
browser flags still need their release verification.

## Saved output and independent checks

A fresh disposable project produced a **53,654-byte, 2.000-second MP4**, SHA-256
`a8c74e92a3f51e3f8fe3dd23bd0fec66c945f6534c0b92d5224fcafe1101c0f8`.
The Mirket job reached `succeeded`, returned the matching artifact revision and
reported `removed_verified_owned_files` scratch cleanup. Its stale-revision
negative control refused before the real job. All seven source asset hashes
remained unchanged.

Private ffprobe reported **24 H.264 frames at 640×360** and two seconds of
48 kHz AAC audio. Decoded RGB checks found the red element move from x=32 to
x=492, captions visible during their cue and absent after it, and both colors
of the local checker asset. I inspected frames 1, 4, 13 and 24; their composition,
Turkish text, moving elements and timed caption were visually consistent with
the checks. Private FFmpeg decoded **96,000 mono audio samples**; independent
440 Hz voice and 660 Hz music amplitudes were 0.1373 and 0.0686, a **2.0003**
ratio.

The first verifier requested raw `s16le` output, which this deliberately
limited FFmpeg build does not mux. Its failure remains in the original
`run-result.json`; it happened **after** the MP4 had succeeded. A separate
saved-artifact verifier used the build's `wav` muxer and passed without
replaying the export. The worker trace contains six expected Sharp-denial
controls, no Sharp or libvips loaded object, and only the selected Chromium,
FFmpeg and ffprobe child executables launched. The package also attempted two
diagnostic subprocess probes (`nvidia-smi` and `ps`); the private wrapper denied
both, and the export continued. There was no remaining owned process after
completion. Guard samples remained Normal with at least **75% free memory**.

The disposable output and evidence are under
`~/.codex/task-state/harness-rewrite-implementation/video-sharp-free-export-no-outer-sandbox-20261026/`.
The frozen preparation, original run result and independent saved-output review
have SHA-256 values
`e870ff34da5d9ff9e0adcefdcf7a358302e4b65d863ff407f3b7f7654c4a8a8e`,
`46c6f0759bd6bcfcf4df398acdd06ed6a1c6ba21f2d4132096ac7163c3c47821`
and `6c9d981dc5fdc31a548c2825a0a16503cc03949fa7413a3dbf1657d87de52188`.
The trace, resource samples and process-cleanup receipt are also retained.

This first result was **one development MP4**. The later exact-component
four-format continuation is recorded below. Normal installed loading still
refuses Video. Private Chromium source, notice and rights closure, admitted
browser/video payloads, native Windows, Codex Desktop GUI activation and
offline behavior remain open.

## Startup diagnostic follow-up

The first worker attempt exposed a Rust diagnostic gap: a structured, early
`type=error` frame was reported only as `invalid video readiness`. The video
process owner now distinguishes a correctly bound startup refusal from a
malformed or mismatched frame and reports only its bounded ASCII error code.
The initial failure would therefore identify `video_export_failed` while still
rejecting an unexpected release, job identity, stage claim or injected code.
The focused regression test passed. After the build-input guard, the full
locked offline suite passed **552 library tests, 11 CLI tests and one active
integration test** (seven existing test gates ignored), strict all-target
Clippy and formatting passed, the pinned private Node adapter build passed,
and its process suite passed **94 tests with three existing skips**. The new
development CLI passed the real stdio MCP smoke. This code change was made
after the MP4 export; it does not alter that saved artifact or claim installed
Video admission.

## Sharp-free four-format continuation

The next disposable MCP session ran WebM and GIF successfully, then refused
PNG-sequence tar with `video_export_failed`. That failed operation published no
artifact and its receipt records `effect=not_attempted` with verified scratch
removal. A fresh direct-worker diagnostic retained `failure.json`; after its
launcher matched Rust's private `TMPDIR` and kept stdin open, it identified
Node 24's permission model refusing `FileHandle.sync()` in the tar writer.
The earlier diagnostic attempts, including the stdin-EOF cancellation and
unscoped temp-directory refusal, are preserved as separate fixtures.

The tar is disposable worker scratch. [Rust's video publisher](../src/artifacts.rs)
independently verifies its complete bytes, copies them into a separate sibling
stage and calls `sync_all()` there before publication. The adapter therefore
stopped calling Node `fsync` on the private tar. A focused regression simulates
the permission-model denial and confirms a complete, block-aligned archive can
still be read. A fresh development component copied and rehashed all **5,892**
files from the earlier Sharp-absent control; only compiled
`dist/video-export.js` changed, from SHA-256
`8644a2e812643e35ef48d5f1ddda9568c5dab7a6f606d1eb38e3b75e577dc4e5`
to `efa4b6861878e3a7a1edac631f8769bb9810873c680820cfa3df8a1778190474`.
The reduced closure still physically omits Sharp and the five dependent
packages identified in the dependency review.

A fresh MCP session using that exact repaired component completed WebM, GIF
and PNG-sequence tar sequentially. A second fresh fixture selected the **same
component** for MP4. All four saved outputs matched their receipts, preserved
the seven selected source assets, and reported confirmed publication and
`removed_verified_owned_files` scratch cleanup:

| Format | Saved bytes | SHA-256 | Independent check |
| --- | ---: | --- | --- |
| MP4 | 53,654 | `a8c74e92a3f51e3f8fe3dd23bd0fec66c945f6534c0b92d5224fcafe1101c0f8` | 24 H.264 frames, AAC, 96,000 decoded mono analysis samples |
| WebM | 48,281 | `d126d279fc0e8b73c8b30bf9a0e1ecf7dabb1767740d8d1c7f0cbacd18851da0` | 24 VP9 frames, Opus, 96,000 decoded mono analysis samples |
| GIF | 180,359 | `38f6256e5291965ba3cad6e3ecb7b3a08ad6376790f9723a3c87d248045747c5` | 24 decoded frames, no audio stream |
| PNG-sequence tar | 414,720 | `cdfe5d2511c624790db8b496cad57307436b82638249dfc4f0b81cebad7dcf6b` | 24 ordered, decoded PNGs; verified manifest, AAC sidecar and caption sidecars |

Independent RGB decoding put the moving element at x=32 in frame 1 and x=492
in frame 24 in every format. The caption region changed in at least 11,892
pixels while the cue was active and in zero pixels after it. The supplied
checker retained both colors. I inspected saved PNG frames 4 and 24; the
composition, motion and caption state agree with those checks. WebM and tar
audio amplitude ratios for the supplied 440/660 Hz tones were **1.9943** and
**2.0003** respectively. These are synthetic content checks, not a listening
or general player-compatibility test.

The three-format trace records **18 expected Sharp-denial controls**, no
Sharp/libvips loaded object, and six refused incidental `nvidia-smi`/`ps`
diagnostic attempts across its three jobs. Its sampled guard stayed Normal=1,
with at least **74% free memory** and **44,943,904,768 bytes free disk**. The
MP4 fixture independently passed the same Sharp and content checks. The
retained verification records are under private task state:

- `video-sharp-free-all-formats-fixed-20261026/saved-format-verification.json`, SHA-256 `a6f3970b1b9b95686dcbecfb29ced8a506ca18ba3c64331626117d49de558e19`
- `video-sharp-free-fixed-mp4-20261026/saved-export-verification.json`, SHA-256 `37ed92c67a0694413ce476e5c5ae5c65e4d9764914876747278d84724ca8c3fd`

These tests support selecting a Sharp-free **development** Video component.
They do not admit its installed payload or prove offline containment. The
private Chromium source/rights closure, video recipient notices, installed
loader/offline exports, native Windows, Codex Desktop GUI activation and
broader player and content fidelity remain separate gates.
