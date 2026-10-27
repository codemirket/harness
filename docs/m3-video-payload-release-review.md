# Video payload source and release review — 2026-10-26

**Source-only review; release admission remains false.** Keep the selected
HyperFrames 0.8.197 adapter, one private Chromium and controlled FFmpeg/ffprobe.
The next release-safe step is an isolated, file-preserving package fixture with
complete notices, followed by installed tests against the qualified Chromium.
This review does not select another dependency, prune an unproven file, admit a
runtime, or qualify either Windows target.

The [approved plan](../planning/implementation-plan.md),
[video adapter contract](video-export-route.md),
[original candidate observations](m3-motion-video-qualification.md) and
[native image source review](video-native-source-closure.md) remain the owning
product and behavior records. The older
[proposed video inventory](../packaging/admission/m3-video-candidate.json.proposed)
was inspected without changing it. In particular, its 260-entry lock identity
does not describe the current lock.

## Exact inputs and conservative payload

The observed `adapters/package-lock.json` is **132,863 bytes**, SHA-256
`c4c72361ef9a83d2e030e6e2338caa66e50cdd755571db794eca9587808dad8b`,
with **261 resolved package entries**. Relative to the older proposed inventory,
the additional entry is `@mozilla/readability` 0.6.0; no previously recorded
package version, URL or integrity changed. Readability belongs to browser
extraction and is outside the producer closure below. The old proposal's
260-entry count is therefore not evidence of an additional video dependency.

Recursive lock dependencies, installed optionals and locked installed peers
from `@hyperframes/producer` select **88 package instances** on this Mac.
Nested dependency instances are counted separately and their files counted
once. Forty-eight absent platform-optional entries are recorded separately;
they were not treated as missing required Mac files. This is a conservative
package closure, not a runtime load trace or proof that all its code is needed.

| Review inventory | Regular files | Bytes |
| --- | ---: | ---: |
| Complete selected published package contents | 6,012 | 85,114,036 |
| Current installed contents of those packages | 6,012 | 85,115,683 |
| Three compiled video adapter files, root `package.json`, policy receipt | 5 | 42,732 |
| Conservative next-fixture runtime skeleton | **6,017** | **85,158,415** |

The 88 instances use **86 distinct published archives**, totaling **24,361,214
compressed bytes**; summing archives per package instance gives 24,477,790 bytes.
All were acquired as inert bytes and matched their current locked SHA-512 integrity.
Every installed regular file was compared to its archive member: no missing or
extra file was found, and the only differences are the **eight existing policy
patches**, matching their recorded original and patched hashes. Their stable
`Modified by Mirket` comments remain present. The six HyperFrames packages
account for 834 files and 10,408,222 current installed bytes.

The concrete fixture layout preserves the current relative layout of
`node_modules/`, `dist/video-main.js`, `dist/video-export.js`,
`dist/video-policy.js`, root `package.json`, and
`node_modules/.mirket-video-policy.json`. The latter is 2,170 bytes, SHA-256
`61808bc4acd188a446bd92fe358b98d4b2d974f7214550c8a5b036878c7a6f37`.
Patched upstream imports resolve back to that exact neighboring
`dist/video-policy.js`; arbitrary relocation of individual files is unsafe.

This selects the video package boundary out of the larger combined adapter
lock, without rebundling or rewriting upstream code. It is the smallest
**defensible package-level candidate from this review**, not a demonstrated
minimum file set. All selected package files, including declarations, maps and
currently unused branches, remain in the inventory. An eventual smaller
selection must earn its own installed evidence. Node, Chromium, FFmpeg/ffprobe,
recipient notices, source downloads and project-owned composition/assets are
additional boundaries; these numbers are not total release size.

## Eager imports, dynamic imports and assets

`adapters/src/video-export.ts` dynamically imports the producer after selecting
the explicit private executables. Within that producer import, ESM reexports
and import barrels resolve more code than Mirket's three producer API calls.
A TypeScript 5.9.3 AST parser inspected source without importing or executing
any target module. The six HyperFrames package boundaries are:

| Package, all 0.8.197 | Source-reachable boundary from producer |
| --- | --- |
| `producer` | Eager `dist/index.js`, containing the selected orchestration APIs and other bundled branches. |
| `engine` | Eager root/reexport graph, including browser manager and frame/audio/encoding services; the parser reaches 71 eager files. Do not remove services merely because one fixture does not exercise their exports. |
| `core` | Eager root and explicit compiler/font/audio subpaths, reaching 112 eager files. Additional font helpers are dynamically imported. |
| `parsers` | Eager root and explicit parser subpaths, reaching 24 eager files plus one literal dynamic target. |
| `lint` | Eager root, imported by the producer and core compiler guard. |
| `studio-server` | Eager **helper exports** `manual-edits-render-script` and `encoder-options`, resolving their two helper files and two shared chunks. The Studio root server entry is not reached by this inspected graph. |

These counts describe this static graph, not observed executed functions.
The parser reached 961 file nodes across the literal import/reexport/require
graph. It does not prove control flow, computed imports, filesystem-loaded
assets or availability of a smaller payload.

The browser manager dynamically imports **`puppeteer` 25.16.0 first**, with
`puppeteer-core` 25.16.0 as its upstream catch fallback; frame capture also
imports `puppeteer-core`. Retain both at the current source boundary.
Puppeteer reexports core, and core's Node/browser entry points retain
`@puppeteer/browsers` 3.2.6, including its launch, install and CLI modules.
That package in turn has dynamic CLI/downloader dependencies. A CDP-only
workflow does not by itself justify removing BiDi, Firefox or installer code
from those eager barrels. Retaining code grants no authorization to invoke it;
the installed route must still use only its explicit private Chromium and
perform no download or upstream installation.

Other concrete boundaries are:

- **HyperFrames browser runtime:** producer reads its sibling
  `dist/hyperframe.manifest.json` and the manifest's
  `hyperframe.runtime.iife.js`, whose recorded SHA-256 is
  `1010f54ffc72ff6dd0d697239903eaac0494d831922023edbfbbf234f86d0a1f`.
  These filesystem-loaded assets are not ordinary module-import edges.
- **esbuild 0.28.2:** a literal dynamic import in core's HTML bundler; the
  matching `@esbuild/darwin-arm64` executable remains selected. Its presence
  is separate from proving which authored inputs use it.
- **wawoff2 2.0.1:** eagerly imported by core font compression. Its root loads
  both compression/decompression JavaScript bindings containing inline WASM;
  a missing `.wasm` filename does not imply absence of compiled native code.
- **Puppeteer configuration:** full Puppeteer uses lilconfig's configuration
  search, including possible JavaScript configuration files. Rust launches the
  adapter with its owned job root as cwd and clears inherited environment.
  Installed tests must verify that the retained configuration loader cannot
  execute an unrelated project/home configuration in that actual layout.

The static parser's 13 unresolved literal edges are optional or alternate
integrations: `pnpapi`, `canvas`, seven `cssstyle` references, `yauzl`,
`proxy-agent`, `utf-8-validate` and `bufferutil`. Its two computed calls are
lilconfig's file-URL import and yargs-parser's `require(path)`. These observations
do not select those packages as new production dependencies, establish that
their branches execute, or prove every optional fallback works.

## Sharp overlap with Design/Raster

The manifest closure includes Sharp because Studio declares it optional.
However, the inspected producer graph reaches only Studio helper subpaths;
the Studio root contains the `import("sharp")` thumbnail path. Neither the
current video adapter nor its inspected literal module graph imports Sharp.
The original qualification's separate `inspect.mjs` **does import Sharp** for
pixel inspection. Thus installed/native inspection evidence should not be
read as proof that this current producer worker needs the addon.

The conservative Sharp subtree is **six instances, 125 files and 19,698,667
bytes**: `sharp`, `@img/colour`, `detect-libc`, `semver`,
`@img/sharp-darwin-arm64` and `@img/sharp-libvips-darwin-arm64`.
The last two retain the original published addon and full libvips artifact
identified in the [native source review](video-native-source-closure.md).

The separate Design/Raster candidate uses the same Sharp 0.35.5 wrapper/addon
boundary but a **different, reduced source-built libvips**, with its own
source/notice and modified-library qualification. The current original video
libvips hash is not that derivative's hash. Do not silently copy one component's
rights/source verdict to the other, choose the original full stack merely to
satisfy an unused optional dependency, or ship two native stacks by accident.
The existing design JS lock shares only eight package instances with this
producer closure; their pins match. Raster is a separately owned component.

The reduction test is concrete: assemble a disposable copy with the entire
six-instance Sharp subtree absent, forbid fallback resolution outside that
copy, and run actual installed MP4/WebM/GIF/PNG exports, local fonts, raster
input assets, supplied audio/captions and cleanup. Record module/file and native
load evidence. Only after that result should the final inventory omit those
files. This review performs no pruning or replacement.

## Exact sources and notice findings

Pinned primary source archives were retained without running their build
scripts, installers, containers or hooks:

| Repository and exact pin | Archive bytes | SHA-256 |
| --- | ---: | --- |
| [HyperFrames `f51650d8a72ed472effff89da087c7e1cf6b2c98`](https://github.com/heygen-com/hyperframes/tree/f51650d8a72ed472effff89da087c7e1cf6b2c98) | 146,536,987 | `f1ed14e824da461091df47ef69b896fca669dad599a8a93acc67ecda01504ae3` |
| [Puppeteer `a207faf527ce4305b33a6c0f9706b090c21c115e`](https://github.com/puppeteer/puppeteer/tree/a207faf527ce4305b33a6c0f9706b090c21c115e) | 2,151,672 | `91f0563bee18cca4a69cace959fd1cc8bdfa1bfc04940436202226e2689f9861` |
| [esbuild `609683d892977362a0f99026cb74b96263d728a9`](https://github.com/evanw/esbuild/tree/609683d892977362a0f99026cb74b96263d728a9) | 2,011,018 | `04cb2d38fb6af093577aa8bf174add7286099b65ef7854480cbbb8ff061cef1f` |
| [wawoff2 `2ba75100c9ec6031a5a1d32ddd1120409aa68573`](https://github.com/fontello/wawoff2/tree/2ba75100c9ec6031a5a1d32ddd1120409aa68573) | 956,894 | `f92bf439e499e92bc66ba54f562f564ac1b44088e8259e5c4a97fa3d9e8ce1ee` |

The registry identifies the Puppeteer/esbuild/wawoff2 commits through `gitHead`;
the six HyperFrames records lack `gitHead`. HyperFrames' tag/source pin remains
unattested source-to-package evidence. All 209 comparable packaged Puppeteer
source files match this source archive; both wawoff2 generated binding files
also match theirs. These comparisons establish those file identities, not an
independently reproduced compiler/native build.

The retained **173 obvious package notice/README files** total 686,203 bytes.
That acquisition is not a semantic conclusion that every required notice has
been supplied. Specific remaining assembly work is:

1. **HyperFrames:** all six embedded Apache-2.0 licenses match the pinned
   10,763-byte source LICENSE, SHA-256
   `4259155fb06f127687ee7b0a8a3682d45132db0f2da26cbc0b7a2d1e796436b8`.
   The pinned tree has no root NOTICE; its two producer test-media NOTICE files
   do not admit those media assets. Preserve the eight modification notices,
   applicable attributions and package licenses in actual recipient delivery.
2. **Puppeteer:** the three selected published packages have no full top-level
   LICENSE file. The pinned root Apache-2.0 license is now retained separately
   (11,344 bytes). `puppeteer-core/lib/third_party/{mitt,parsel-js,rxjs,urlpattern-polyfill}`
   embeds its license/attribution text in generated JS; retain those full
   headers and index them in recipient notices. The pinned build controls
   explicitly copy these licenses into the generated code.
3. **Embedded BiDi dependencies:** preserve Chromium-BiDi's own LICENSE and
   `mapper_bundle_THIRD_PARTY_NOTICES` (2,343 bytes). That file identifies
   bundled mitt 3.0.1 and **Zod 4.6.5**; the separately locked Zod 4.6.7 notice
   does not describe that distinct bundled version. Chromium-BiDi's npm record
   lacks `gitHead`; exact upstream source-to-bundle mapping remains unresolved.
4. **wawoff2/WOFF2/Brotli:** the published package ships the wrapper MIT LICENSE
   but no separate WOFF2/Brotli notices, and its generated bindings contain no
   visible copyright/license text. Its [pinned Dockerfile](https://github.com/fontello/wawoff2/blob/2ba75100c9ec6031a5a1d32ddd1120409aa68573/src/Dockerfile) selects WOFF2
   `a0d0ed7da27b708c0a4e96ad7a998bddc933c06e` and Emscripten 2.0.34;
   the Makefile statically links WOFF2 and Brotli into `SINGLE_FILE` bindings.
   The WOFF2 parent's Brotli gitlink is
   `533843e3546cd24c8344eaa899c6b0b681c8d222`, but the Dockerfile clones
   recursively **before** checking out WOFF2 and does not update its submodule
   afterward. That gitlink is candidate source evidence, not proof of the
   embedded Brotli revision. Exact upstream WOFF2/Brotli and Emscripten notices
   were acquired; source/build mapping and elected runtime notice coverage
   still require review. Do not silently rebuild or replace this dependency to
   conceal the gap.
5. **esbuild/native contents:** the platform package has only a short README
   beside binary/package metadata. Retain esbuild's MIT text, applicable
   incorporated xxhash/Go runtime notices and exact binary/source provenance.
   The pinned esbuild source preserves its separate xxhash notice. A package
   SPDX field alone does not supply recipient notices. Inert binary inspection
   found a `go1.26.5` string; that tag's Go LICENSE/PATENTS were retained, without
   treating the string as independently attested compiler provenance.
6. **Native media/image/source delivery:** use the exact separately qualified
   [neutral FFmpeg successor](m3-ffmpeg-neutral-successor-qualification.md) only
   through a new explicit package selection; the old proposed media hashes are
   unchanged. Both FFmpeg candidates remain unadmitted. If Sharp is retained,
   resolve which exact native artifact is selected and carry its matching
   source/notices and modified-library installation route. Private source
   archives are not actual recipient delivery.

Apache/MIT/BSD package terms do not create a general corresponding-source offer
requirement. Complete source/build mapping is also a project provenance gate;
the independently applicable GPL/LGPL native delivery and modification rights
remain governed by the owning media/raster reviews. Their successful local
fixtures do not certify the aggregate product's terms or rights.

## Concrete next assembly and installed gates

Keep the release flag closed and create one disposable candidate from the
unpruned manifest. Bind the **current** lock, all package archive integrity,
eight patch inputs/outputs, three adapter files, policy receipt, exact private
Node/Chromium/media identities and separate notice/source delivery manifests.
Preserve the runtime layout and manifest-loaded IIFE. Do not execute npm
lifecycle scripts, browser installers or first-use downloads. A final receipt
must distinguish source identity, assembled-file identity, installed operation
and recipient delivery; no one receipt establishes all four.

Once the exact private Chromium is available and the native lane is clear:

1. Run the unpruned fixture through installed Mirket MCP, using an empty private
   home/cache and a path containing spaces/non-ASCII. Refuse resolution outside
   the fixture; observe exact Node/browser/media/native-module paths, no
   installer/download, offline local assets, private profiles and cleanup.
2. Repeat the same installed operation without Sharp's subtree before making
   any reduction. Keep Studio helper/chunk files and both Puppeteer packages.
   Test missing IIFE/changed policy receipt and changed executable refusals as
   discriminating failures; preserve composition and destination on refusal.
3. Exercise Puppeteer configuration with unrelated project/home configuration
   sentinels and confirm they are neither imported nor run. Inspect actual
   cwd/home/search behavior rather than assuming environment scrubbing closes
   a file-based loader. Missing optional integrations should either remain
   unused or fail explicitly on their unsupported paths.
4. Recheck all four actual formats with independent decode/content evidence,
   supplied audio gain/sync, caption timing/Unicode, changed-source pixels,
   active cancellation and process cleanup. Reuse the existing small fixtures
   as regression evidence, then complete representative 1080p/long composition,
   imported composition, local video extraction, typography/player/alpha and
   combined resource gates from the approved plan.
5. Carry the selected notices and hash-bound same-release sources through the
   intended recipient download/install/modification path. Final signing,
   release-time advisory assessment, actual macOS 15 execution, **native
   Windows 11 x64 and ARM64**, and Codex Desktop GUI activation remain separate
   gates. Metadata availability is not native qualification.

No native build, target-module import, browser/video/media launch or installed
journey ran for this review. Its source tooling ran on macOS 26.5.2 ARM64; the
AST parser used an explicitly selected development Node 24.18.1 and the current
TypeScript 5.9.3 parser, without executing reviewed target modules. The existing
user installation and client settings were untouched.

## Private evidence

The evidence root is
`~/.codex/task-state/harness-rewrite-implementation/video-payload-release-review-20261026/`.
It preserves:

- `closure-review.json`, all 86 distinct `tarballs/` and exact `registry/` records:
  package pins, dependencies, file/archive comparisons, notice candidates and
  absent optionals;
- `installed-file-manifest.json`, `candidate-package-receipt.json` and
  `input-snapshot/`: the unpruned candidate inventory and exact adapter/policy
  inputs; the candidate receipt is SHA-256
  `2aa58df36990f37862dc3a81a18cbfc450dffcf0dcec9aa2f63eaa52619a79dc`;
- `module-graph.json` and its source-only parser: eager/literal dynamic edges,
  unresolved optional integrations and computed-import limits;
- `source-archives/`, `source-acquisition.json`, `source-member-comparison.json`,
  `package-notices/`, `source-notices/`, `source-metadata/` and
  `embedded-notice-acquisition.json`: primary pinned inputs and known gaps;
- `input-stability.json`: all eleven captured live source/build/receipt inputs
  remained byte identical at the recorded final comparison.

The repository change is this new review only. Publication, admission,
source-offer issuance and modifications to other owners' files were not made.
