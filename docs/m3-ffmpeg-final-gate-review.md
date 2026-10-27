# FFmpeg final recipient and release gate review

Reviewed 2026-10-26 on macOS 26.5.2 ARM64. **The exact macOS source kit is
technically sufficient to rebuild and relink the selected FFmpeg/ffprobe pair
with the recorded existing toolchain. Release admission remains false.** A
fresh unmodified rebuild has passed; a deliberate modified-source rebuild,
modified executable use through Mirket, final distribution and recipient access
have not passed. This is an engineering review, not legal certification.

This review supplements [the source qualification](ffmpeg-recipient-source-closure.md)
and its [proposed record](../packaging/admission/m3-ffmpeg-source-closure.json.proposed).
It does not change their selection or admit the combined video payload.

The later [neutral successor qualification](m3-ffmpeg-neutral-successor-qualification.md)
closes the successor's local source build/relink, private-path remediation,
explicit IJG acknowledgment and finite relocated media/MCP checks. The exact
selection table below identifies the preserved predecessor. The successor has
its own frozen source/binary identities and remains unadmitted; actual recipient
access, modified installed use, rights and platform gates stay open.

## Exact selection and bounded evidence

| Artifact | SHA-256 |
| --- | --- |
| `ffmpeg`, 9,647,448 bytes | `8374f88f171b233c072da4acbf9058e9b563ce048d9300edabbe6036f616505e` |
| `ffprobe`, 9,454,392 bytes | `8c6f1213081e3805be3e2d69b9db85ff291235771b76b4f35bd64bf5ad280a1c` |
| `ffmpeg-9.0.2-darwin-arm64-recipient-source-20261026.tar.gz`, 29,434,536 bytes | `f420e0ea8e047fa01a3ae0bcc0d4df29f97a87712ee6f72e338c49ae83c616ba` |

The kit contains the complete FFmpeg 9.0.2, exact x264 commit, libvpx 1.17.0
and Opus 1.6.1 archives; editable build helpers; selected configuration;
executable identities; and retained notices. This review independently rehashed
all 65 manifest inputs and the frozen archive: no mismatch. The archive contains
66 regular files including its manifest, with no other member types except
directories. The prior fresh rebuild compared all 12,409 upstream source files,
matched the selected component configuration and matched small decoded/media
fixtures. Executable hashes differ after rebuilding at another root; executable
byte reproducibility was not established.

`build.py` compiles the three codec static libraries, then builds and links both
executables from source. No missing application object set was identified for
rebuilding these two executables. Existing Apple Xcode/Clang/macOS SDK, make,
Python 3.14 and an explicitly selected pkgconf remain prerequisites; they are
not supplied by the kit. This conclusion covers this pair and does not establish
source or relinking sufficiency for Chromium's separate FFmpeg library.

The selected `--enable-gpl`/libx264 configuration is GPL-2.0-or-later. FFmpeg's
LGPL linking checklist does not qualify this GPL selection.
[Exact FFmpeg license](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/LICENSE.md),
[FFmpeg legal guidance](https://ffmpeg.org/legal.html).

## Minimal local closeout before publication

| Action and owner | Observable completion | Present status |
| --- | --- | --- |
| Packaging: freeze one final binary/source/notice set | Final package binds both executable hashes, source-kit hash, full notice corpus and readable source/build directions; independently verify the saved package and refuse a deliberately changed input. Include the source installation/staging instructions used by the final product. | Closed for the successor's local source/notice fixture: 64 source-kit inputs and 29 delivery files verify, and a changed recipe is refused. Final release packaging and actual recipient access remain open. |
| Source qualification: document and exercise a modified-source build | Starting from a verified pristine kit in a new directory, extract, make a harmless observable source change, rebuild/relink, and show the resulting executable executes that change while retaining the required media behavior. Preserve original evidence separately and label the changed sources/output. | The successor's fresh neutral source build/relink and source preservation passed. A deliberate modified-source build remains untested. |
| Recipient workflow: qualify explicit modified replacement | Exercise the documented private installation/import path in disposable Mirket state using the recipient-built pair; run a media operation and show which binaries were used. If the release verifier correctly rejects changed official hashes, provide an explicit separately identified user-build route and test it. | The successor passed actual disposable development MCP operations with explicit relocated binaries. Recipient-installed modified replacement remains untested. This is a product check, not a universal GPLv2 installer-feature requirement. |
| Notices: finish the index against final linked code | Retain all source-file terms and add an explicit Independent JPEG Group acknowledgment to recipient documentation. Read-only symbol inspection of the exact final `ffmpeg` found `_ff_j_rev_dct`, `_ff_j_rev_dct1`, `_ff_j_rev_dct2` and `_ff_j_rev_dct4`; the relevant exact source header and FFmpeg license explain the conditions. | Closed locally for the successor: the notice index explicitly acknowledges IJG and all 24 prior notice files/full source remain unchanged. No predecessor breach is asserted. [Exact notice](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/libavcodec/jrevdct.c), [license explanation](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/LICENSE.md). |
| Packaging/security: authenticate and review final bytes | Inspect final Mach-O architecture/load commands, signing state, package verification and embedded paths; remove or explicitly disposition private build-path information in distributable records. Refresh component/advisory evidence at final freeze. Any rebuilt/changed executable requires a new exact source/binary/notice binding. | Successor private-path removal, system-only load inspection and local source/binary binding passed. Neutral fallback/tool metadata is documented. Final signing/package authentication and release-time advisory refresh remain open. |
| Media owner: qualify the exact final pair in the intended operation | Run the required Mirket/HyperFrames formats and fidelity checks, failure/cancellation/descendant cleanup, staging/finalization races and bounded memory/disk admission; preserve outputs and actual executable identities. | The exact successor passed relocated direct MP4/WebM/GIF/PNG and AAC/Opus checks, all four real MCP formats and active cancellation with its build root absent. Broader fidelity/resource/failure coverage remains open; MOVTEXT stays disabled. See [successor evidence](m3-ffmpeg-neutral-successor-qualification.md) and [retained production gates](m3-motion-video-qualification.md#open-production-gates-and-evidence-index). |

The modification workflow needs no new source acquisition or production
dependency. `rebuild.py` verifies the frozen supplied files before extraction,
so editing a manifest-listed recipe and rerunning that verifier is deliberately
rejected. In a separate working copy, verify and extract first; edit the
extracted source; then invoke the supplied editable `build.py` with
`MIRKET_BUILD_PKG_CONFIG` set to the explicit existing pkgconf path. Inspection
shows that `build.py` reads those extracted sources directly and creates local
build/dependency/output trees. These modification steps have not been executed
by this review. Document changes and preserve upstream notices if a modified
copy is redistributed.

## Distributor, recipient and platform gates

| Required decision or evidence | Why local source proof does not close it |
| --- | --- |
| Distributor chooses and implements the final source route | GPLv2 section 3(a) covers accompanied source; section 3 also recognizes equivalent source access at the same designated download place. A section 3(b) written offer is a separate route with a minimum three-year term. Do not promise an offer accidentally or treat private staging as actual access. [GPLv2 section 3](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/COPYING.GPLv2). |
| Final delivery is tested from the recipient's position | Fetch or receive the final signed/authenticated binary and matching source/notices using the actual channel, without maintainer-only paths or credentials. Verify all identities, readability and reconstruction directions; record the channel's retention/access arrangements for the selected route. A local archive and hash cannot establish these facts. Actual publication/transmission requires its own authorization. |
| Distributor reviews final product boundaries and terms | Preserve GPL notices, modification/redistribution rights and the prohibition on added restrictions. Mere aggregation has a separate rule; a process boundary alone does not decide the final product's legal classification. Review the actual Mirket/HyperFrames package and terms. [GPLv2 sections 1–2 and 6](https://github.com/FFmpeg/FFmpeg/blob/n9.0.2/COPYING.GPLv2). |
| Distributor obtains a product-specific codec/grant disposition | Exact source licenses and the retained Opus grants do not settle H.264/AAC/other codec patents for the actual distributor, uses and jurisdictions. Opus grants contain defined coverage and conditions, including reciprocity/termination provisions. Have the responsible rights/legal reviewer assess those organizational facts; this review makes no royalty-free or patent-clearance certification. [FFmpeg patent guidance](https://ffmpeg.org/legal.html#Patent-Mini-FAQ), [official Opus terms](https://opus-codec.org/license/). |
| Release owner qualifies supported macOS behavior | Mach-O declares macOS 15.0; only macOS 26.5.2 ARM64 execution was observed. Test macOS 15, the final signed/notarized delivery, clean recipient installation and operation. Account credentials/signing authority and native test access are distinct prerequisites. |
| Windows owner qualifies both native targets | Native Windows 11 x64 and ARM64 require their own exact binaries/source/configuration/notices, native DLL/architecture closure, process ownership, cancellation, resource and recipient journeys. The macOS kit and cross-checks cannot establish them. Both remain explicit release gates under the user's decision. See [Windows status](windows-qualification-status.md). |
| Combined release owner closes Chromium/Sharp/product gates | This FFmpeg pair is separate from Chromium's LGPL FFmpeg and Sharp/libvips recipient obligations. Each exact closure and the combined runtime must pass its own final evidence. |

## Review provenance and limits

Private evidence lives under
`~/.codex/task-state/harness-rewrite-implementation/`: this review read
`ffmpeg-source-review-20261026/{review.md,recipient-candidate/,local-recipient-fixture/,local-recipient-verification.json}`
and `ffmpeg-recipient-final-20261025/rebuild-verification.json`. It rechecked the
frozen source-kit identity and every manifest input, rehashed all 29 local
delivery files without a mismatch, inspected build-helper control flow and the
selected binary's IJG symbols, and consulted the primary
license/codec sources linked above. It did not run native builds, media tools,
Mirket integration, installers, signing/notarization or a distribution channel.

The original source-only review did not run native checks; the linked successor
record owns the later native results and frozen source identities. No upstream
installer, release-package mutation, commit, push, external publication,
recipient offer or admission was performed. Completion of this document does
not close any untested row above.
