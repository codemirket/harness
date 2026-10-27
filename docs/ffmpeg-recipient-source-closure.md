# FFmpeg 9.0.2 recipient-source qualification

Observed 2026-10-26 on macOS 26.5.2 ARM64. The selected private FFmpeg/ffprobe
pair has exact source, notices, a working recipient rebuild and a checked local
binary/source delivery fixture. **Release admission remains blocked.** Actual
recipient delivery, product terms, codec patent review and platform qualification
are separate from this local proof.

This document owns the FFmpeg source/recipient follow-up to
[the motion qualification](m3-motion-video-qualification.md). The earlier pair's
23-check staged media tests remain historical evidence for that pair. They do
not establish the final pair's subtitle support.

## Selected executable and source identities

The selected files are under external task state
`ffmpeg-qualification-20261025/video-followup-20261025/private/bin/`.
Both were independently rehashed and inspected with `-version`, `-L`,
`-buildconf`, `-codecs`, `-protocols` and `otool -L`.

| File | Regular bytes | SHA-256 |
| --- | ---: | --- |
| ffmpeg | 9,647,448 | `8374f88f171b233c072da4acbf9058e9b563ce048d9300edabbe6036f616505e` |
| ffprobe | 9,454,392 | `8c6f1213081e3805be3e2d69b9db85ff291235771b76b4f35bd64bf5ad280a1c` |

The program is **GPL-2.0-or-later**, with statically incorporated x264, libvpx
and Opus; no nonfree/version3 option was enabled. Network protocols and
autodetection are disabled; the selected protocols are local file and pipe.
Mach-O declares macOS 15.0 and links Apple system facilities, without Homebrew
dylib dependencies. Only macOS 26.5.2 ARM64 execution was exercised.

| Complete source archive | Bytes | SHA-256 |
| --- | ---: | --- |
| FFmpeg 9.0.2 | 12,040,788 | `8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e` |
| x264 `b35605ace3ddf7c1a5d67a2eb553f034aef41d55` | 1,040,327 | `cd71a7515b0e9a012e1ac9b1f8415bebcaf6fc97d4db32286642ac4c0fbe24f9` |
| libvpx 1.17.0 | 5,670,546 | `1020f184046187baa2985dbde38e0691f49c44088bca7a1842b0236c6081dc0a` |
| Opus 1.6.1 | 10,472,813 | `6ffcb593207be92584df15b32466ed64bbec99109f007c82205f0194572411a1` |

The retained FFmpeg release signature was independently verified with the
current official key, fingerprint
`FCF986EA15E6E293A5644F10B4322F04D67658D8`. The current key bytes match the
retained key. Opus's official download checksum matches. Archive PAX identities
match the official x264 commit and libvpx tag commit; the x264 commit is also
the observed official stable-branch head. These latter checks are source/endpoint
identity evidence, without a new independent signature claim.
[FFmpeg downloads](https://ffmpeg.org/download.html),
[x264 stable](https://code.videolan.org/videolan/x264/-/tree/stable),
[libvpx tag](https://github.com/webmproject/libvpx/tree/v1.17.0),
[Opus downloads](https://opus-codec.org/downloads/).

## Frozen source kit and notices

External review root:
`~/.codex/task-state/harness-rewrite-implementation/ffmpeg-source-review-20261026/`.
Its source-only archive is
`ffmpeg-9.0.2-darwin-arm64-recipient-source-20261026.tar.gz`, **29,434,536 bytes**,
SHA-256 `f420e0ea8e047fa01a3ae0bcc0d4df29f97a87712ee6f72e338c49ae83c616ba`.
It has 65 manifest-listed inputs, plus the manifest. The four full source
archives, helpers, selected configuration, binary identities and prior receipts
are unchanged from the prior final recipient bundle. Only README/index wording
and two added Opus patent-term/provenance files differ. Both older source kits
remain preserved.

All 22 original named notice files remain byte identical. They include FFmpeg's
license/COPYING files and retained authors/test notices, x264 COPYING/AUTHORS,
libvpx LICENSE/AUTHORS/PATENTS and retained third-party notices, and Opus
COPYING/AUTHORS. Full archives also preserve source-file notices and terms;
the named corpus is not a claim that every retained test/tool was linked.

The exact Opus archive's COPYING provides URLs to Xiph.Org, Broadcom and
Microsoft IETF patent declarations. The added `OPUS-PATENT-LICENSES.txt` retains
the complete grants, definitions and conditions from the official license
page's patent section, with source identity, attribution and its CC-BY-3.0
website license. The implementation's unchanged COPYING remains authoritative
for its BSD notice. The examined declarations did not establish a missing-copy
breach; this supplement improves offline access to the terms. Patent conditions
and unrelated codec patents still require product-specific review.
[Opus license page](https://opus-codec.org/license/),
[Xiph declaration](https://datatracker.ietf.org/ipr/1524/),
[Broadcom declaration](https://datatracker.ietf.org/ipr/1526/),
[Microsoft declaration](https://datatracker.ietf.org/ipr/1914/).

## Recipient rebuild and local delivery evidence

The retained `ffmpeg-recipient-final-20261025/` evidence contains a successful
fresh recipient rebuild, completed in 96.729 seconds using reviewed configure
and static-library/ffmpeg/ffprobe make targets. No install target, autogen,
downloader, package-manager operation or hook was invoked. That full build was
an invocation-policy proof; network traffic was not independently monitored.

All **12,409 regular upstream source files** remained byte identical after that
build, with no extra/missing regular files. Generated component configuration
matched the selected build exactly; the remaining generated-header differences
were the known recipient-root strings. Selected and rebuilt programs produced
identical small H.264/setparams MP4, decoded RGB, float WAV and AAC/ipod M4A
fixtures. Whole executable hashes differ; byte-reproducible executables are not
claimed.

This independent review repeated manifest verification and fresh extraction
with OS network access and writes outside the owned review root denied. All
12,409 extracted regular files matched the prior recipient build's source.
A deliberately changed build recipe failed before extraction. The refreshed
frozen archive was separately unpacked, its exact file inventory checked, and
all 65 manifest inputs verified under the same restrictions. Native compilation
was not repeated for this notice-only refresh. Disk observations remained above
the 25 GiB floor.

The frozen kit README supplies commands for an existing Python 3.14, Apple
Xcode/Clang/macOS SDK, make and explicit existing pkgconf path. Its verifier
supports `--verify-only` and `--extract-only`; a fresh directory is required
for a full rebuild. Build tools/SDK are prerequisites and are not redistributed
or installed by these helpers. The selected configuration intentionally remains
unchanged: final `mov_text` spelling leaves **MOVTEXT encoder and decoder
disabled**. No timed mov_text subtitle capability follows from the older tests.

`local-recipient-fixture/` accompanies both exact binaries with the frozen
source archive, full notice corpus and readable license/source directions.
`local-recipient-verification.json` verifies both binary identities, source-kit
identity, all 65 kit inputs, original notices and the exact delivery file set.
The fixture demonstrates a local accompanied-source layout for consideration
under GPLv2 section 3(a). No source offer, actual recipient transmission, public
download or final product terms were created. GPLv2's source route and the
selected GPL build govern this step; FFmpeg's LGPL-oriented checklist alone
does not settle it. [GPLv2 supplied by FFmpeg 9.0.2](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/COPYING.GPLv2),
[FFmpeg legal guidance](https://ffmpeg.org/legal.html).

## Current advisory scope

The captured current FFmpeg security index's master and 9.0 sections identify
11 distinct CVEs through 17 fix commit IDs. Fourteen IDs are ancestors of release
commit `946fcce07b6dcd0331c8cc609192aeff5e1924f8`; the three remaining mainline
IDs have explicitly identified release cherry-picks among those ancestors.
All ten affected source files match the exact n9.0.2 tag bytes. The selected
swscale implementation includes its listed fix; the listed EXR/SANM/MagicYUV/
AV1/RASC/LCL/RSCC/TIFF decoders are disabled and the zmqsend tool is not built.
The complete per-CVE/commit/define evidence is retained in
`ffmpeg-advisory-ancestry.json` and `ffmpeg-tag-source-equivalence.json`.
[FFmpeg security index](https://ffmpeg.org/security.html).

The exact libvpx 1.17.0 changelog includes security fixes; Opus 1.6.1 is the
observed current official download and x264 is the observed stable head.
This bounded primary-source assessment establishes the listed fix mapping; it
does not establish absence of vulnerabilities in every retained library or
untrusted-media path. Refresh advisory/component assessment against the final
payload at release. [libvpx changelog](https://github.com/webmproject/libvpx/blob/v1.17.0/CHANGELOG).

## Remaining release gates

1. Select and exercise the actual GPLv2 section 3 recipient route. Bind the
   exact delivered executables, complete source kit, applicable notices and
   accessible instructions in the final payload and distribution channel.
   The local fixture does not establish real recipient access or continuing
   availability. No three-year written offer has been promised here.
2. Review final product aggregation and recipient terms, preserving the relevant
   modification/redistribution rights. Complete codec-patent and grant-condition
   review for the actual distributor/use/jurisdictions. This source exercise
   cannot certify those organizational/legal facts.
3. Qualify final package authentication and review embedded build paths for
   privacy; preserve original evidence while preparing distributable records.
   Recheck source/notice identities and current advisories after any payload or
   configuration change. A subtitle repair would create a new selection.
4. Exercise the intended Mirket/HyperFrames operation, full required formats,
   media fidelity, failure/cancellation/finalization and resource limits.
   The earlier stalled one-pass experiment remains unresolved historical
   evidence. macOS 15 execution, signing/notarization and native Windows 11
   x64/ARM64 execution remain unqualified.
5. Complete the separate Chromium, Sharp/libvips and combined product recipient
   obligations. This FFmpeg-only record neither admits the media payload nor
   changes any application operation or release gate.

Machine-readable FFmpeg-only status is in
[`m3-ffmpeg-source-closure.json.proposed`](../packaging/admission/m3-ffmpeg-source-closure.json.proposed).
The private `review.md` indexes exact captures, commands and retained failures.
