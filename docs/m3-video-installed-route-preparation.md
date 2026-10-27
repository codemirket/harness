# Video installed-route preparation — 2026-10-26

This is historical preparation. The exact Mac Video payload later passed local
first-party installation and installed export checks; see the
[current qualification](simplification-qualification-2026-10-27.md#final-local-mac-integration).
Public source and notice delivery remains a separate gate. Statements below
that normal installation refuses Video describe the earlier build.

## 2026-10-27 direct CLI comparison

The selected `@hyperframes/producer` 0.8.197 package declares import exports
and no `bin` entry. The current adapter uses its public `resolveConfig`,
`createRenderJob` and `executeRenderJob` APIs while binding the private Chrome,
source root, frame/format limits and explicit FFmpeg verification. Invoking an
undocumented internal CLI would add a second behavior contract and discard
several of those checks without a demonstrated reduction in package closure.
Keep the narrow API adapter and its installed release gate; no alternate
renderer or CLI path is admitted by this comparison.

The frozen proposal below was superseded by the four-format development
successor recorded in the final section. Its original test receipts remain
historical evidence; normal installed Video is still refused.

**Video remains unavailable in normal installed releases.** The source now
recognizes a finite Video component and its prospective private payload paths,
but a compiled admission gate rejects it before manifest creation, staging or
setup can accept it. No development qualification API or manifest escape hatch
was added. This prepares an owning route; it does not qualify an installed
offline export.

The [Sharp dependency review](m3-video-sharp-dependency-review.md) identified a
concrete reduced candidate. The [operation contract](video-export-route.md)
still owns rendering, effects, cancellation and output requirements. The earlier
development export evidence and the two stopped Sharp-free export attempts
retain their original scopes.

## Frozen candidate

The new [proposed inventory](../packaging/admission/m3-video-sharp-free-macos.runtime.json.proposed)
records the existing **82-package, 5,892-file, 65,459,748-byte** private copy under
the prospective `runtime/video/` prefix. Every file was rehashed against the
retained copy; the transformation changes inventory paths only. Its source
manifest remains SHA-256
`589fba4abecfd7d80966e22330cb914ddf01c2abc6025c58ed3467f64de6c012`.
The proposal itself is **1,260,404 bytes**, SHA-256
`fd000b985835028faaa4716dcf2c57ef3c3cded0cb457d437904198052f7e6b0`.

The compiled selection verifies that proposal identity, the current locked
package identity, counts, totals, distinct paths and fixed native media pins.
All six Sharp-subtree packages remain absent. The three compiled Video adapter
files, root package metadata and eight-patch policy receipt are included with
their exact existing hashes. The only selected native esbuild helper would
receive executable mode; package CLI and install scripts remain non-executable
data. No npm command, lifecycle script, downloader or upstream helper ran.

Video reuses the existing private Node selection rather than selecting a
second Node. Its media paths are `runtime/video/bin/ffmpeg` and
`runtime/video/bin/ffprobe`, bound to the existing final development pair:

| Input | Bytes | SHA-256 |
| --- | ---: | --- |
| Node 24.22.1 | 122,545,856 | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |
| FFmpeg 9.0.2 | 9,647,448 | `8374f88f171b233c072da4acbf9058e9b563ce048d9300edabbe6036f616505e` |
| ffprobe 9.0.2 | 9,454,392 | `8c6f1213081e3805be3e2d69b9db85ff291235771b76b4f35bd64bf5ad280a1c` |

Those actual existing private files were rehashed. None of those runtime
binaries was copied, changed, rebuilt or run by this work. Their existing
admission, source and recipient limitations remain in force.

## Loader and refusal boundary

[`distribution`](../src/distribution.rs) recognizes `video` in the strict
component schema and infers it from `runtime/video`. On macOS ARM64 its normal
layout checks the compiled candidate and then returns the specific
`video release component remains unadmitted` error. Windows layouts refuse
the component; no Windows execution or admission follows from these source
changes. The Raster gate retains its previous refusal text.

The internal [candidate loader](../src/distribution/video_candidate.rs) can
derive the private Node, adapter and media paths only alongside the
`BrowserPayload` supplied by the same verified release. It accepts the shared
`runtime/browser/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing` convention and
refuses a missing browser or a path belonging to another release. It introduces
no Video-specific browser pin or fallback. The browser is now admitted through
the separate setup-acquired selection; presenting Video candidate bytes or
declaring `video` still cannot enable an installed Video payload.

The existing installed recipient resolver, release store, host registration,
user installation and client settings were unchanged. No additional production
dependency, publication, commit or push occurred.

## Verification and remaining gates

The build-input guard passed on macOS 26.5.2 ARM64. The first focused run
exposed a test-fixture error: its placeholder executable lacked the source
owner-execute mode, so `write_manifest` correctly refused it. The fixture now
sets the selected file modes; the rerun passed **29 distribution tests** with
one existing Raster fixture test ignored. Strict all-target Clippy with
warnings denied and `cargo fmt --all -- --check` passed. The full locked
offline suite then passed **551 library tests, 11 CLI tests and one active
integration test**; seven tests were ignored by their existing gates. The
rebuilt CLI has SHA-256
`99990dc27ea9518df0b3bb3af485d93175b554367880be8074521a2263d94b3e`.
The real stdio MCP smoke passed its installed-unavailable Video refusal and
the existing source/job/reconciliation checks with that executable, the exact
private Node 24.22.1 and compiled adapter. The
focused tests cover the frozen finite inventory, missing Sharp packages, shared Node
and absence of an independent browser selection; loader binding to the same
private release; and refusal before a new manifest or installation home exists.

Normal release admission now requires replacing this frozen proposal with the
tested Sharp-free worker and neutral FFmpeg pair, complete combined npm/native
source and notice/recipient delivery, actual disposable installed offline exports across
all four profiles with persistent Sharp denial and meaningful assets/audio/
caption/output checks, cancellation/source preservation/owned cleanup, and
native platform/Codex Desktop activation evidence. Hashes and these source tests
do not establish those outcomes.

Private verification evidence is under
`~/.codex/task-state/harness-rewrite-implementation/video-installed-route-20261026/`.

## Four-format and neutral-media successor

The earlier proposal selected the first Sharp-free worker and original media
pair. The [four-format continuation](m3-video-sharp-free-development-export.md)
changed exactly one compiled worker file: `dist/video-export.js` grew from
29,787 to 30,006 bytes and now has SHA-256
`efa4b6861878e3a7a1edac631f8769bb9810873c680820cfa3df8a1778190474`.
An inventory scan found no other changed, missing or extra selected runtime
file in the 5,892-file tree. The corrected total is 65,459,967 bytes.

The same proposal now selects the separately qualified
[neutral private FFmpeg successor](m3-ffmpeg-neutral-successor-qualification.md):
`ffmpeg` is 9,630,936 bytes, SHA-256
`9415ba4acee04dadb45b06878a528bb529819ce009b621e702a847fa78a4c7d8`;
`ffprobe` is 9,437,880 bytes, SHA-256
`6554f4a4223e41acdcd83ba92c2139d678068e2e953d9220cd9631a7c034b392`.
The updated proposal itself is 1,260,433 bytes, SHA-256
`d88f58b1048287196fe08d92cb2eb3eea888b781a1c9d064b9c030629f930483`.
Rust's compiled candidate checks use these exact identities. The existing
closed admission gate is unchanged. A host-installed Homebrew FFmpeg pair also
passed [four development outputs](m3-host-ffmpeg-development-qualification.md),
but its mutable shared-library closure makes the already qualified private
pair the simpler Mac release candidate.

## Disposable installed qualification fork

On macOS 26.5.2 ARM64, an isolated source copy opened Video admission only
when `MIRKET_VIDEO_QUALIFICATION_FORK=1` was explicitly set. Its sole Rust
source difference wrapped the existing Video refusal in that flag. The normal
source and binary still reject Video before manifest creation. The fork binary
SHA-256 was `1718b063ca2c9b27d0da9d2f487b93a6370ce6f4f77d334f61fe3aed32c3173a`;
the unchanged shipping development binary was
`8943cf3d654221637f7476165b19ea7bbb75a9239d8922c3eddaf4d144bf004a`.

A disposable Browser+Skills release was combined with the exact 5,892-file
Sharp-free video candidate and neutral FFmpeg pair. The fork wrote and staged
release `sha256:8cfff5afa71561ce7dc6a96d18239c559759e359c5ede1cab9cf6c210cae38b0`.
`setup-core` activated it with a stable launcher; `doctor-core` reported a
healthy manifest, launcher and `browser,video,skills` components. The selected
installed `video-export.js`, `ffmpeg` and `ffprobe` matched their candidate
hashes. The installed MCP server then produced these saved outputs:

| Format | Saved bytes | SHA-256 |
| --- | ---: | --- |
| MP4 | 53,373 | `766763043ac71f7be6a15002872e6e1066237c4ee1144e0f0165087d31e74db4` |
| WebM | 46,261 | `18d8f3c90a3319dd072defce97ea7ac7640be24c770256115d81ccb463aef7de` |
| GIF | 159,011 | `f2226c9f1f14eef9b01798592b9488d407fce5753bc85abce177c0a0cd8eb3ad` |
| PNG sequence tar | 375,808 | `acdae75c9d31dbdb30ef386189bfce4e2db54625dba69efbe944ed9cce8315b5` |

The installed test also passed stale-source refusal and saved-artifact receipt
checks. A source edit changed 3,693 decoded heading pixels and produced a
different MP4 revision. Independent decoded PCM showed the intended onset/end
windows and a voice-to-music gain ratio of 2.0003. Unicode caption sidecars
and burned cues changed 11,993 pixels during the cue and zero after it.
In-flight cancellation left no artifact and cleaned owned scratch. A 48-frame
PNG sequence completed at 33,396,736 bytes. The original development mode of
`tests/video-mcp-smoke.mjs` still passed against the same assets and binary
pair, with the same MP4 revision. An installed failure probe with an external
script reference was refused by source preflight before rendering; it saved no
artifact, preserved the source and cleaned owned scratch.

All six Sharp-related package subtrees were absent from the installed Video
module tree, and module resolution from the installed adapter could not find
Sharp. The installed export succeeded under that condition. Without the
explicit fork flag, the fork's installed doctor refused the release; setting
the flag again restored its healthy report without changing release files.

The first installed test failed because its fixture HTML referenced an
unselected `checker.png`; the test now includes that image when present. The
caption scenario also now handles a composition without a preexisting caption
element. These were test-fixture omissions, not changes to the renderer or
release payload. The resulting qualification remains local: no admitted
Video release, source/notice or recipient-rights clearance, Windows execution
or Codex Desktop GUI activation follows from it. The complete disposable
source, home and failure receipts are retained under
`~/.codex/task-state/harness-rewrite-implementation/video-installed-qualification-20261026/`.
