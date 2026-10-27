# Neutral FFmpeg successor qualification

On 2026-10-26, a separate macOS ARM64 development successor passed its source
build, configuration/private-path checks, relocated media comparison and finite
Mirket/HyperFrames MCP smokes. **Release admission remains false.** The original
pair and source kit remain byte identical; this record does not replace their
existing proposed inventory or establish actual recipient delivery.

This follows [the private-path review](m3-ffmpeg-private-path-review.md) and
[the final recipient gate review](m3-ffmpeg-final-gate-review.md).

| Successor artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `ffmpeg` | 9,630,936 | `9415ba4acee04dadb45b06878a528bb529819ce009b621e702a847fa78a4c7d8` |
| `ffprobe` | 9,437,880 | `6554f4a4223e41acdcd83ba92c2139d678068e2e953d9220cd9631a7c034b392` |
| `ffmpeg-9.0.2-darwin-arm64-neutral-successor-source-20261026.tar.gz` | 29,939,250 | `f40269afa8a45026f7051fce0b78ef5e314122d436ecea7eed9f101d084f61c3` |

## Exact changes and build evidence

The source selection remains FFmpeg 9.0.2, x264 commit
`b35605ace3ddf7c1a5d67a2eb553f034aef41d55`, libvpx 1.17.0 and Opus 1.6.1.
No upstream source was edited. The recipient-relative build ran from the private
neutral root `/private/tmp/mirket-ffmpeg-build-20261026`, adding only
`-ffile-prefix-map=<root>=.` to global CFLAGS/CXXFLAGS and FFmpeg extra-cflags.
Explicit developer lane guards precede native execution.

The guarded build completed in **87.948 seconds**, with eight successful
configure/static-library/ffmpeg/ffprobe steps and a minimum observed free space
of **51,560,153,088 bytes**. This is one build observation, not a performance
claim. It invoked no upstream install, autogen, downloader, package-manager or
hook target. The reviewed invocation policy is not an OS sandbox or independent
network-traffic measurement.

All **12,409 regular upstream source files** remained byte identical. FFmpeg's
component configuration and x264/libvpx/Opus configuration headers match the
predecessor byte for byte. The only differing FFmpeg `config.h` defines are
`FFMPEG_CONFIGURATION`, `FFMPEG_DATADIR` and `AVCONV_DATADIR`; normalizing the
root strings and added mapping flag gives an exact match. Opus hardening remains
enabled. MOVTEXT encoder/decoder remain disabled, so no timed MP4-subtitle
capability is inherited or repaired by this successor.

## Privacy and relocation

Neither successor binary matches the original builder login, captured host,
account-root or task-state markers in UTF-8, UTF-16LE or UTF-16BE. All **62 Opus
source-filename strings** are now component-relative. Three ordinary strings
still contain the neutral root: compiled data directory, FFmpeg configuration
and libvpx configuration. Generic pkgconf/Apple toolchain paths remain build
provenance. These are documented residuals, not hidden private paths.

Both binaries declare macOS 15.0, contain no DWARF/debug section or RPATH, and
load only libSystem, Apple system zlib, CoreFoundation, CoreVideo and CoreMedia.
Only macOS 26.5.2 ARM64 execution was observed; minimum-version execution and
final signing/notarization remain open.

The exact successor bytes were copied into a private path containing spaces and
non-ASCII characters. While the neutral build root was temporarily absent,
they encoded/decoded MP4, WebM, GIF and PNG fixtures and decoded AAC/Opus audio.
Decoded video/audio match the predecessor; small H.264/setparams MP4,
float-WAV and AAC/ipod-M4A files match byte for byte. Corrupt input was refused.
The neutral root was restored after each check.

Two verifier assumptions were corrected with their failures preserved: GIF
rawvideo comparison uses frame passthrough to avoid default output duplication,
and PCM audio comparison uses the elected WAV muxer rather than an unelected
raw-s16le muxer. Neither correction changed sources, binaries or features.

The compiled data directories remain fallback paths. Success with the root
absent establishes that the exercised fixed-argument route does not require
their contents; it is not a general-purpose relocated CLI or executed
no-file-open guarantee.

## Intended Mirket/HyperFrames operation

The current development Mirket, private Node, compiled adapters, Chromium,
fixture files and successor pair were hashed before use and rechecked afterward.
The eight required policy-patch hashes were verified. With the neutral build
root absent throughout, the actual disposable Rust/MCP/Node/HyperFrames path
passed:

| Scenario | Output bytes | Observed seconds |
| --- | ---: | ---: |
| MP4 | 55,329 | 10.598 |
| WebM | 48,443 | 7.266 |
| GIF | 174,408 | 7.039 |
| PNG-sequence tar | 431,104 | 7.062 |
| Active MP4 cancellation | No saved artifact | 4.179 |

These smokes exercise source-revision refusal, output identity, source
preservation and owned scratch/process cleanup. The source uses the existing
640×360, 12 fps, two-second composition. They are bounded regression evidence;
they do not establish general player compatibility, long-render fidelity,
complex typography, all cancellation interleavings or other platforms. All
projects/state were disposable; installed user state was unchanged. The native
lane was explicitly released after these checks and subsequent work was source-only.

## Frozen source and local recipient fixture

The deterministic source-only archive has **64 manifest inputs plus the
manifest**, ten actual generated configuration files, all four complete source
archives, retained signature/key and all **24 original notices unchanged**.
The notice index now explicitly acknowledges the Independent JPEG Group; full
source-file notices remain intact. The actual build and rebuild helpers are
byte identical to those used by the observed build. Post-build changes are
limited to README/index, evidence and verifier support for the frozen-candidate
status.

Historical and new host-bearing configure logs/config.status files stay in
private evidence. The export omits them and explicitly curates private paths
out of comparison/MCP records; it does not relabel edited records as unchanged
raw receipts. Exported nonarchive material has no named builder/root/host marker
match under the same bounded scan. This is not a general credential audit.

Two independent archive serializations match. A fresh unpack verified all 64
inputs; a changed recipe failed before source extraction. Fresh extraction then
verified all 12,409 upstream files against the observed native-build baseline,
with an exact 12,474-file kit/source inventory. Native compilation was not
repeated from this final documentation/evidence-only export. Executable byte
reproducibility is not claimed.

`local-successor-recipient-fixture/` binds both exact executables, the source
archive, notices and readable directions through 29 verified delivery files.
Its delivery manifest SHA-256 is
`02d9e269612d8710e2edc194eafb942a6eaac0dda049ab85a33ee783313a2277`.
It is a local accompanied-source fixture, with no source offer or actual
recipient transmission.

## Evidence and remaining gates

External task state is
`~/.codex/task-state/harness-rewrite-implementation/ffmpeg-private-path-successor-20261026/`:

- `native-successor-qualification-20261026/`: exact private commands/inputs,
  configuration/source comparison, runtime-string/load observations, retained
  verifier failures, relocated media and five MCP smoke receipts.
- `native-successor-source-export-20261026/`: frozen source archive,
  `freeze-receipt.json`, `frozen-kit-verification.json`, expanded source kit and
  local delivery fixture. Temporary extraction checks were verified and removed.
- `/private/tmp/mirket-ffmpeg-build-20261026/native-evidence/`: guarded native
  build receipt and logs; compilation outputs remain private and preserved.

The kit provides complete technical inputs for the observed local build/relink.
A deliberate modified-source rebuild and recipient-installed modified use are
still untested. Actual final recipient access and terms, product aggregation,
codec-patent/grant disposition, final package authentication/signing, release-time
advisory refresh, macOS 15 execution, native Windows 11 x64/ARM64 and separate
Chromium/Sharp/combined-product qualification remain release gates. This
engineering evidence is not legal certification or admission.
