# macOS ARM64 Sharp/libvips source closure

> **Historical custom-build evidence.** Active Raster qualification now uses
> the [upstream prebuilt candidate](m4-raster-prebuilt-qualification.md), while
> Video has a separate Sharp-free worker. The custom-build recipient path below
> is no longer in active code.

The later [upstream prebuilt provenance check](m4-raster-prebuilt-provenance-20261027.md)
verified the two selected Mac native npm attestations and exact source commits.
The statement below that signature verification had not been performed records
the earlier review state; the full upstream native source/notice gate remains.

The [video candidate](video-export-route.md) remains excluded from release
admission. The selected upstream native pair has verified package identity.
A separate, reduced raster source build now passes private image and functional
modified-library tests. A subsequent disposable Mirket raster MCP package now
passes source/notice reconciliation, meaningful replacement and rollback. Actual
release installation/import, durable recipient delivery and release admission
remain open. Neither native artifact is admitted by this evidence.

## Verified selection

The actual development selection is `@img/sharp-darwin-arm64` 0.35.5 and
`@img/sharp-libvips-darwin-arm64` 1.3.4, containing libvips 8.18.7. Both exact npm
tarballs match the adapter lock's SHA-512 integrity and every installed package
file. All six Darwin ARM64 members of the official sharp-libvips workspace asset
also match the installed package. The asset is 86,852,556 bytes, SHA-256
`a40fc59e3aa3635177613fa63f29ceecb28b5ad99e7919def47a61f6ca06f0ee`.
It contains platform binaries and developer headers, rather than complete native
dependency sources. [Upstream release](https://github.com/lovell/sharp-libvips/releases/tag/v1.3.4).

| Selected file | Bytes | SHA-256 |
| --- | ---: | --- |
| `sharp-darwin-arm64-0.35.5.node` | 280,256 | `791921f975371be33aed727e06283282cc001c960792eb52505b579fdaaf1b55` |
| `libvips-cpp.8.18.7.dylib` | 18,231,688 | `c1a5b5633b21aeb8a229abf7d062e07e4d778c0b44ed43cd1f93362bda58c322` |

All 14 installed Sharp `src/*` files match its v0.35.5 source archive. The
annotated tags resolve to Sharp commit
`51a990faa26ade5586a4934ac9673c98d8893326` and sharp-libvips commit
`ebb95f8add54eee8bed840e3fb587e4cbec857d7`. The npm provenance payloads identify
these commits and their release workflow runs. Payloads were decoded and their
subject digests checked; certificate, signature and transparency-log verification
was not performed. Package/file identity is stronger evidence here than a claim
that the native source build was independently reproduced.
[Sharp tag](https://github.com/lovell/sharp/tree/51a990faa26ade5586a4934ac9673c98d8893326),
[native tag](https://github.com/lovell/sharp-libvips/tree/ebb95f8add54eee8bed840e3fb587e4cbec857d7).

## Retained source and notice inputs

A private, explicitly incomplete staging candidate preserves the two repository
archives, pristine libvips 8.18.7, librsvg 2.63.2 and Cairo 1.18.6 archives, all
five external recipe patch inputs, GPLv3/LGPLv3 texts and a partial notice corpus.
It preserves the selected libvips README's upstream license table. The README
does contain that table; the missing material is the complete license,
copyright, disclaimer, patent-grant and source-delivery corpus.

The selected Cairo archive's `COPYING` permits MPL-1.1 or LGPL-2.1 for its
implementation. Keep the upstream table's MPL-1.1 election unless a deliberate,
verified alternative is chosen; do not silently turn its LGPL alternative into
an `-or-later` grant. The libvips source archive includes its vendored libnsgif,
including its MIT copyright/permission/disclaimer notice, which is also retained
separately. Source archives preserve per-file attributions. These observations
improve the inventory; three pristine native archives did not close the other 25
named components or embedded subprojects and Rust crates. The bounded follow-up
below advances acquisition without admitting the binaries.
[Cairo source](https://cairographics.org/releases/cairo-1.18.6.tar.xz),
[libvips source](https://github.com/libvips/libvips/releases/download/v8.18.7/vips-8.18.7.tar.xz).

The native recipe changes librsvg features, runs `cargo update --workspace`, and
applies a memory-limit patch. The pristine librsvg lock has 366 package entries,
357 from a registry, and no Git-source entries. That is a workspace-lock count,
not the compiled crate inventory. Cargo's `--workspace` option preserves already
locked non-workspace packages unless they are absent; this is not an unrestricted
registry update. The edited final lock can potentially be reconstructed from the
retained inputs, but reconstruction and verification were not performed. The
release workspace does not supply that final lock or crate sources.
[Librsvg source](https://download.gnome.org/sources/librsvg/2.63/librsvg-2.63.2.tar.xz),
[Cargo semantics](https://doc.rust-lang.org/cargo/commands/cargo-update.html#option-cargo-update---workspace).

The retained scripts also select a moving Rust nightly and cargo-c and install
build prerequisites. They were inspected without running installers, hooks or
native builds. Exact tool versions and a reproducible build environment are
provenance/qualification questions; the project must separately establish
complete source and compilation/installation scripts for recipients.
[Pinned native recipe](https://github.com/lovell/sharp-libvips/blob/ebb95f8add54eee8bed840e3fb587e4cbec857d7/build/posix.sh).

## Bounded source follow-up — 2026-10-26

All 25 remaining named recipe archives were acquired inertly, totaling
**98,313,956 bytes**, with exact URLs, sizes, HTTP identity headers and SHA-256
digests retained outside shipped source in task-state
`sharp-source-followup-20261026/native-source-acquisition.json`. These are
acquisition hashes, not independently reproduced native-build evidence.
Inspection of those archives and the three earlier archives counted 533,099,083
regular source bytes. It preserved 78 candidate notice files, including AOM patent
material, FreeType's FTL.TXT and mozjpeg's IJG notice, plus 50 subproject controls.
The notice corpus still needs semantic and linked-component review.

All 357 registry archives in librsvg's pristine lock are now preserved with
matching lock checksums: 210 reused verified existing cache bytes and 147 were
downloaded, totaling 38,675,419 bytes. This source superset is not the final
post-edit lock or compiled crate inventory. No Cargo, upstream code, installer,
hook or native build ran during acquisition. The five earlier patch inputs are
unchanged. The reduced build below materializes the three applicable Mac patches;
the excluded librsvg/UltraHDR states remain separate original-stack work.
GLib's forced gvdb fallback is specifically pinned by its acquired wrap to
`2b42fc75f09dbe1cd1057580b5782b08f2dcb400`. The subsequent source build acquired
that exact export: 24,716 bytes, SHA-256
`069a00aa1fc893f18423602f4e095583be5a220429f6e8a58d70511490b4b019`;
its Git pax comment identifies the same full commit.

A controlled raster-focused Sharp/libvips build could materially reduce this
closure: disable `rsvg`, `pangocairo` and `fontconfig`, use the already selected
resvg/Chromium paths for SVG and authored text, and retain the HEIF/AOM path for
required AVIF alongside WebP/PNG/JPEG. Optional archive/TIFF/UltraHDR/GIF-saving
features could also be excluded if their removal preserves the agreed input and
output contract. The external review recorded a 14-named-input hypothesis.
Actual configuration below corrected its component inventory. The resulting
library is a new native artifact; this review makes no application or
release-package substitution.

Separate immutable same-release source/relink downloads can be evaluated with
verified hashes and recipient instructions. The source archives need not be
installed under `.mirket` or included in runtime startup hashing. Durable access,
complete notices, final source/build mapping and the modified-library installation
journey remain release gates.

## Reduced raster source build — 2026-10-26

A controlled local build of libvips 8.18.7 succeeded on macOS 26.5.2 (25F84)
ARM64. Its **14 named inputs plus gvdb** are libvips, GLib, libffi,
proxy-libintl, zlib-ng, **Expat**, libexif, lcms, AOM, libheif, mozjpeg, libpng,
libimagequant and libwebp. Expat is unconditional in this exact libvips version;
the first configure disproved its omission from the earlier candidate. Highway
is disabled. All input archives, the three applicable retained Mac patches,
exact source edits and dated per-file modification notices are frozen outside
shipped source under task-state `sharp-source-followup-20261026/raster-qualification`.

Meson uses `auto_features=disabled`, explicit PNG/JPEG/WebP/HEIF, EXIF, lcms,
imagequant and zlib features, nofallback and the exact local gvdb subproject.
Text/SVG/font rendering, TIFF/archive/UltraHDR/GIF, Highway, other optional
loaders and dynamic modules are disabled. libheif's actual saved configuration
has AOM AV1 encode/decode enabled and every other codec backend disabled,
including its otherwise default-enabled x264/OpenH264 paths. AOM's optional
libyuv, embedded Highway and WebM I/O are disabled. This is an inspected graph,
rather than an exclusion inferred solely from a feature list.

The recipe uses Xcode 26.6/Apple clang 21.0.0, CMake 4.4.4, pkgconf 3.0.9,
Python 3.9.6, Meson 1.9.2 and Ninja 1.12.1 at recorded explicit paths. It runs
at most two jobs, denies network and writes outside private task-state, and
enforces a 25 GiB free-space floor and 1 GiB private-file cap. Direct-IP network
and outside-write failure probes were observed. Saved failures include `strip`
using a forbidden temporary directory, an over-restrictive static iconv lookup,
Meson's private module path, missing Expat and a recipient patch temporary path.
The final recipe corrects them without widening the write boundary. It invokes
no upstream installer, `build.sh`/`posix.sh`, Git hook, Rust or Cargo.

The saved baseline link map identifies 1,259 selected static archive members,
five direct C++ API objects and Apple system imports. Every selected archive
member maps to supplied source, including generated source and two same-named
cache objects resolved by linker symbols. The notice corpus retains 61 upstream
license/copyright/patent/author files, 1,220 unique linked-source preambles,
applicable npm notices and GPLv3/LGPLv3 texts; complete archives preserve all
header and other file attributions. The intended LGPLv3 election uses applicable
later-version grants; original source grants remain authoritative. Disabled
LCMS GPL plugins are absent from the link graph. Required mozjpeg/IJG wording,
AOM/WebP patent texts and incorporated fastfeat/vector notices are retained.

The frozen private source kit is **105,315,672 bytes**, SHA-256
`fb1f835e96de79e2a0b15f35df8aa051181b2b09de0d976c612873914025f284`.
It contains the 15 native archives, applicable patches, build/relink controls,
notices, source manifests, tool inputs and exact Sharp/build-recipe source
archives. A fresh no-network materialization reconstructed all **8,693** source
files/symlinks with **zero manifest differences**. A second complete fresh
compile was not run; the complete source build and subsequent functional relink
were run. This local archive has not been delivered or published to recipients.

The unchanged Sharp 0.35.5 addon loaded the new dylib in a disposable sibling
package layout, identified as `1.3.4-mirket-raster-qualification`. The installed
baseline dylib is 14,631,104 bytes, SHA-256
`1801c6f1cb12363a8d1815d224840001031ed96f6f986e9e91e03dce35ee453a`.
Under Node v24.22.1, it passed 16 checks: lossless PNG/WebP/AVIF with alpha,
bounded lossy AVIF/WebP color/alpha error,
bounded JPEG error, palette PNG, EXIF pixel rotation, ICC P3/sRGB conversion and
profile retention, alpha compositing, resizing, the exact HyperFrames 0.8.197
JPEG thumbnail transform, corrupt-input rejection and expected SVG/text rejection.
All 15 shared publisher/candidate check records, including encoded hashes, match.
This does not exercise the full HyperFrames export or separate resvg/Chromium
font and composition journey.

A separate interface-compatible modification changes unsigned-byte negation
from `255-input` to `254-input`. The modified library was rebuilt, installed and
ad-hoc signed only in a second disposable layout. Its 16 checks pass, including
the changed result `[10,20,30]` → `[244,234,224]` versus the baseline
`[245,235,225]`; process reports identify the distinct loaded private dylib.
This proves meaningful library modification through Sharp's sibling-package
mechanism. It does not exercise the eventual Mirket recipient import, integrity
verification or rollback journey. Neither the existing installation nor release
verification rules were changed.

## Disposable Mirket recipient operation — 2026-10-26

External task state `sharp-recipient-journey-20261026/package/` now contains a
disposable raster component, local source delivery, prominent library/IJG
wording, full copied notices and an explicit fixture-only import/rollback
helper. The component directory is `runtime/design/raster/`; the selected module
is `runtime/design/raster/node_modules/sharp/dist/index.mjs`. After rollback,
the exact minimum runtime is **124 regular files, 16,087,960 bytes** across
Sharp, `@img/colour`, detect-libc, semver, the unchanged ARM64 addon and the
derivative native package. Fixture backup/test dylibs and controls are additional
test material. The exact minimum inventory is retained externally and in the
[raster source/recipient proposed record](historical-admission/m4-raster-source-recipient.json.proposed).
The restored fixture is also frozen as
`sharp-raster-recipient-fixture-20261026.tar.gz`, 123,568,262 bytes, SHA-256
`6ae5a0202786cc9230d62d10d77b7dec07eb3bb928d4f9c2d428c15e86912bbe`.
All 209 archive files match the restored fixture, including its 207-file
manifest inventory, manifest and explicit baseline selection record.

All 119 wrapper/addon/dependency files match the earlier frozen application lock;
the remaining five native-package files identify this derivative. The unchanged
280,256-byte addon retains SHA-256
`791921f975371be33aed727e06283282cc001c960792eb52505b579fdaaf1b55`, and all
14 shipped Sharp source files match the supplied exact Sharp archive. The local
kit retains its 105,315,672-byte identity above; all 364 embedded manifest inputs
were reverified (365 archive files including the manifest). The copied notice
tree contains **71 files, 2,151,327 bytes**, including the 61 upstream notice
files and linked-source/npm/license indexes. All are byte identical to the
earlier corpus. Recipient instructions name the source kit, checksum, existing
tool prerequisites, preparation, build, modification and rollback controls.

The finite Mirket operation resizes/re-encodes, so the earlier negate change
would not demonstrate changed behavior through that operation. A separate
dated patch changes only `libvips/resample/reduceh.cpp`: three-band uchar
horizontal reduction subtracts 16 from the first channel, clipped at zero.
It preserves the interface. The separate libvips-only build reused hash-verified
static dependencies, ran one job for 60.683 seconds, denied network/outside
writes and retained memory-pressure level 1. It left the prior source/build
tree and baseline untouched. The 843 supplied regular libvips source files
differ only in that explicitly supplied patch. Its link map selects the same
1,259 archive-member multiset as the baseline; the source/notice map remains
bound with the modified source identified separately.

The resulting locally signed replacement is 14,631,104 bytes, SHA-256
`40ca6321841f21b5ac4e44b5ca03ce999d16cd613fca36ea44e50bf5a3ee5060`.
The fixture verifies the complete 207-file package, explicit replacement hash,
ARM64 architecture, code signature and all **11,331 defined exported symbols**
before atomically replacing only the dylib. A deliberately corrupt replacement
failed before any write. No addon, wrapper or application code was changed.

The unchanged Rust server/worker and Node design adapter completed
`design_prepare` → `design_start` → terminal receipt inspection against the
same canonical package module for **baseline, replacement and rollback**.
Each phase passed five jobs: RGB PNG, JPEG, WebP, AVIF and an RGBA PNG control.
The lossless RGB PNG changed from SHA-256
`05bbfe652cfe488d3872860b69f91b18b1f9e3a1fb2435c2257c5279e1b45488` to
`328ddac36d45ef7e345a4efa75c9a214de6f986826c7bca5a521b34a5c78505e`.
Every decoded red value is exactly 16 lower; green and blue are unchanged.
The RGBA control's encoded and decoded bytes are unchanged. JPEG/WebP/AVIF
remain decodable at the requested dimensions and reflect the intentional color
change; their lossy differences are retained without a lossless claim.

Rollback restored dylib SHA-256
`1801c6f1cb12363a8d1815d224840001031ed96f6f986e9e91e03dce35ee453a`.
All five saved encoded outputs and decoded pixel hashes then match the initial
baseline exactly. Receipts for all 15 selected jobs confirm success and complete
scratch cleanup; no owned server/adapter remained. OS observations identify both
the unchanged addon and the package's current dylib in actual raster children.
MCP operations retained network/outside-write denial. Two successful image runs
initially lacked loader observations because macOS blocked `ps` in that sandbox;
the failures are preserved. An outer host observer supplied the actual loader
records without changing the raster invocation or its restrictions.

This establishes the local technical replacement journey through the intended
MCP operation. It uses `mirket dev serve` and a task-owned component verifier;
it does **not** exercise an admitted installed-release updater or change normal
release integrity rules. Installed distribution loading still supplies
`raster_module: None`. Development installed-release wiring needs an owning
component/schema/manifest change and a disposable installed test. Public
recipient admission additionally requires actual durable source access, final
rights/Installation Information terms, authentication/privacy and required
client/platform qualification. Local source delivery alone does not settle
those facts. Detailed commands, outputs, loaded paths, retained failures and
the restored fixture remain under external task state
`sharp-recipient-journey-20261026/`.

## Required recipient journey

The addon dynamically references `@rpath/libvips-cpp.8.18.7.dylib`, and its first
rpath resolves to the neighboring `sharp-libvips-darwin-arm64/lib` directory.
The selected dylib has an ad-hoc linker signature, with no team identifier or
internal requirement. Those static observations do not demonstrate successful
use of a substantively modified library. Sharp's source loader also tries a
local rebuilt addon before its prebuilt package. Rebuilding that addon itself
was not exercised. The reduced dylib swap and Mirket MCP replacement/rollback
were exercised privately above; the admitted-release installation path remains
unqualified.
[Sharp binding recipe](https://github.com/lovell/sharp/blob/51a990faa26ade5586a4934ac9673c98d8893326/src/binding.gyp),
[Sharp loader](https://github.com/lovell/sharp/blob/51a990faa26ade5586a4934ac9673c98d8893326/lib/sharp.mjs),
[source-build instructions](https://sharp.pixelplumbing.com/install/#building-from-source).

Before admission, complete these concrete gates for the artifact actually chosen:

1. Preserve the demonstrated reduced candidate's source/build/notice binding
   when selecting the final shipped library/addon and component manifest. If
   choosing the original full-feature publisher dylib,
   its final librsvg lock/crate closure and complete original stack remain open;
   the narrower source build does not qualify that different artifact.
2. Supply prominent recipient library/use notices, applicable copyrights,
   disclaimers and patent grants alongside a complete same-release source kit
   through the intended durable verified download mechanism. Verify actual
   recipient access. Cairo's MPL source-availability notice remains applicable
   if choosing the original Cairo-containing stack.
3. Carry the demonstrated shared-library modification and MCP rollback through
   the intended installed-release import/integrity path, retaining saved output,
   loader identity and cleanup evidence. Account for applicable combined-library
   obligations and Installation Information. The disposable fixture does not
   implement that admitted application path.
4. Preserve recipients' modification and debugging rights in the delivery terms.
   The current release verifier checks payload hashes and has no admitted video
   payload. A direct development Node load or re-signing an unchanged dylib does
   not establish that the eventual recipient can install and execute a modified
   combination.

The LGPL alternatives and required notices follow
[LGPLv3 §§3–5](https://www.gnu.org/licenses/lgpl-3.0.html); Cairo source-availability
and executable notices follow [MPL-1.1 §§3.2–3.6](https://www.mozilla.org/en-US/MPL/1.1/).
Neither an upstream URL nor a private staging archive is a completed recipient
delivery. Byte-identical rebuilds are useful evidence, not asserted here as a
standalone license requirement. The reduced private source build and functional
modified-library/MCP runs above establish their stated technical outcomes.
Release admission, publication and the admitted recipient installation/rights
journey remain uncompleted; the existing installation was not changed.
