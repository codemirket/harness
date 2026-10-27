# Video recipient source and notice proposal — 2026-10-26

**Source-only proposal; release admission remains false.** Signed publisher
provenance now binds all six HyperFrames 0.8.197 packages to the selected source
commit. All 42 embedded font faces have exact package/license mappings. A
separate local notice packet and scoped source export are frozen below. The
embedded WOFF2/Brotli build remains a historically corroborated source candidate,
without independent reproduction or build attestation.

This advances the [video payload review](m3-video-payload-release-review.md).
It supersedes that review's unresolved HyperFrames source-to-package mapping
and adds the previously unenumerated embedded font boundary. It does not
change its unpruned runtime inventory, the
[proposed admission](../packaging/admission/m3-video-candidate.json.proposed),
the [approved plan](../planning/implementation-plan.md), or the separate
[browser acquisition](m3-browser-setup-acquisition.md),
[FFmpeg](m3-ffmpeg-neutral-successor-qualification.md) and
[Sharp/libvips](video-native-source-closure.md) gates.

## HyperFrames source-to-package mapping is verified

All six exact registry records expose both SLSA v1 provenance and npm publish
attestation bundles despite omitting `gitHead`. The retained original tarballs
match the current adapter lock's SHA-512 integrity. Each attestation's sole
subject matches that same SHA-512 digest; the following SHA-256 values identify
the retained compressed archives:

| HyperFrames 0.8.197 package | Archive SHA-256 |
| --- | --- |
| `core` | `e1fdf1905b8a43176b65ada13d6cc981d25a8f19d92e95514884efb57b085d27` |
| `engine` | `45660d2003e2dbb519323de87acf68bb3c11601addaf3832803b4d85668063ac` |
| `lint` | `52f462779abf610827e4b6475268128f1c164e93f460d1797fff0c5987e8271f` |
| `parsers` | `0d486510a3e031eaee1a9c5871187f10ace76b83f979d9ed5271f43ea20498d3` |
| `producer` | `17a7047871ac9fa2671d90cbba7dd8925ef995a64a79a082f7e6bbcb44c1be8c` |
| `studio-server` | `803c91c04aab5534eed1d014d5c54a8cf4e700549933f4e5a4475d63cca4147d` |

The six SLSA statements identify the same source commit,
`f51650d8a72ed472effff89da087c7e1cf6b2c98`, repository
`https://github.com/heygen-com/hyperframes`, workflow
`.github/workflows/publish.yml` at `refs/heads/main`, and
[run 40181125929, attempt 1](https://github.com/heygen-com/hyperframes/actions/runs/40181125929/attempts/1).
The separately fetched `v0.8.197` tag resolves directly to that commit.

Verification used the existing development Sigstore 4.1.1 client,
`@sigstore/verify` 3.1.1 and `@sigstore/tuf` 4.0.2, with an explicitly selected
Node 24.18.1 and a new private TUF cache. No dependency was installed. The
verification policy required the GitHub Actions OIDC issuer,
`https://token.actions.githubusercontent.com`, and the exact certificate
identity
`https://github.com/heygen-com/hyperframes/.github/workflows/publish.yml@refs/heads/main`.
Certificate-transparency and transparency-log thresholds were each one.

All **six SLSA bundles and six npm publish bundles passed** the verifier. The
registry's twelve recorded package signatures also verified against its fetched
public signing keys. A changed DSSE payload failed with `DSSE payload hash
mismatch`; the genuine bundle failed under an unrelated expected workflow
identity. Original bundles, verified TUF metadata/targets, bootstrap trust
material and a readable verification summary are retained in the proposal.

This verifies the publisher's signed package/source identity. It does not prove
that the upstream workflow was hermetic, that every dependency is licensed,
or that Mirket independently reproduced the packages. The pinned
[publish workflow](https://github.com/heygen-com/hyperframes/blob/f51650d8a72ed472effff89da087c7e1cf6b2c98/.github/workflows/publish.yml)
was read without execution; it checks the immutable checkout and builds from
the frozen Bun lock before publishing. Its broader release/cloud jobs are not
part of Mirket's selected runtime.

## Embedded fonts are a separate licensed boundary

`@hyperframes/core/dist/fonts/fontData.generated.js` contains **42 WOFF2
faces, 735,172 decoded font bytes**. Its SHA-256 is
`bee52875611f736d6fef7aa7971ab4f0a3f7b744ed91d8af7431e9c463b31a3f`.
These are bundled runtime bytes independent of the project's supplied Noto
fixture; the core Apache license is not their font license.

The pinned source's `bun.lock`, SHA-256
`d9a17a1631b18b32aa8d066d394ffb7da5c71be8152e2529c752411d1685b499`,
selects these 18 Fontsource packages. Each exact published archive matched its
locked SHA-512 integrity. Every decoded embedded face matched an archive font
member **byte for byte**, without decoding a font or executing the generator.
All matching files are the generator's Latin, normal-style WOFF2 inputs.

| Fontsource package suffix | Version | Embedded normal weights |
| --- | --- | --- |
| `archivo-black` | 5.2.8 | 400 |
| `eb-garamond` | 5.2.7 | 400, 700 |
| `ibm-plex-mono` | 5.2.7 | 400, 700 |
| `inter` | 5.2.8 | 400, 700, 900 |
| `jetbrains-mono` | 5.2.8 | 400, 700 |
| `lato` | 5.2.7 | 400, 700, 900 |
| `league-gothic` | 5.2.8 | 400 |
| `montserrat` | 5.2.8 | 400, 700, 900 |
| `noto-sans-jp` | 5.2.9 | 400, 700 |
| `nunito` | 5.2.7 | 400, 700, 900 |
| `open-sans` | 5.2.7 | 400, 700 |
| `oswald` | 5.2.8 | 400, 700 |
| `outfit` | 5.2.8 | 400, 700, 900 |
| `playfair-display` | 5.2.8 | 400, 700, 900 |
| `poppins` | 5.2.7 | 400, 700, 900 |
| `roboto` | 5.2.10 | 400, 700, 900 |
| `source-code-pro` | 5.2.7 | 400, 700 |
| `space-mono` | 5.2.9 | 400, 700 |

**Unresolved faces: none.** Every package contains a full OFL-1.1 LICENSE,
copied unchanged, including its supplied copyright and any Reserved Font Name
wording. The embedded font bytes were not modified or renamed. Their per-face
archive paths, sizes and hashes, all 18 package integrity values, and all
license hashes are in `embedded-font-mapping.json` and the source/notice index.
These are existing bundled components, not newly selected runtime dependencies.

Two Fontsource notices have only a generic `Google Inc.` attribution. Their
original texts remain intact, with additional pinned upstream family notices:

- [Source Code Pro OFL](https://github.com/google/fonts/blob/49fbebd3dc75d42fe72c4a3eef6524f8fcc335fd/ofl/sourcecodepro/OFL.txt)
  at `49fbebd3dc75d42fe72c4a3eef6524f8fcc335fd`, SHA-256
  `cb30d3086a8b3ce0b9e3690bf48d6620402b61160bc658076f95180ccd9e9dae`,
  retains Adobe's copyright and Reserved Font Name `Source`;
- [Noto Sans JP OFL](https://github.com/google/fonts/blob/295d98a7a0c17c68f1341eaeea354e7960ea70d3/ofl/notosansjp/OFL.txt)
  at `295d98a7a0c17c68f1341eaeea354e7960ea70d3`, SHA-256
  `1c05c68c34f9708415aada51f17e1b0092d2cea709bf4a94cd38114f9e73d7d9`,
  retains Adobe's copyright and the same Reserved Font Name.

Those are supplemental notice evidence, not a claim that the fonts were
independently rebuilt from those upstream repositories. Latin file identity
also does not establish Japanese glyph coverage or the plan's multilingual
typography acceptance.

## wawoff2 mapping and remaining native provenance

All **eleven** published wawoff2 2.0.1 files match the source snapshot at its
registry `gitHead`,
[`2ba75100c9ec6031a5a1d32ddd1120409aa68573`](https://github.com/fontello/wawoff2/tree/2ba75100c9ec6031a5a1d32ddd1120409aa68573),
including both generated JavaScript bindings. Static base64 extraction found
these modules; neither was instantiated:

| Embedded module | WASM bytes | SHA-256 |
| --- | ---: | --- |
| Compression | 682,166 | `0f9cbc41a9e16c777c4ce8175859d5753d7a81c1614ce7d51653e5cb381dbd58` |
| Decompression | 212,978 | `517f31132df89609b695b46d07c6ee1a9deb9d6d388e33d1706127e3c4543fc1` |

The pinned Dockerfile selects WOFF2
`a0d0ed7da27b708c0a4e96ad7a998bddc933c06e` and Emscripten image tag 2.0.34.
Its Makefile links WOFF2 and Brotli into single-file bindings. The WOFF2
parent's Brotli gitlink is
`533843e3546cd24c8344eaa899c6b0b681c8d222`.

The earlier recipe-order concern is now narrower: the script clones recursively
before checking out WOFF2 and does not subsequently update its child, but
GitHub's `master` history at the generated binding commit's timestamp
(`95801116c65bc247b1e36cd27212b7602ba7ffb2`, 2022-01-19 16:02:47 UTC)
already points to that same WOFF2 parent and Brotli gitlink. The current
repository metadata still names `master` as default. This is positive
historical source evidence; it is not proof of what a particular container
actually cloned or compiled. Neither WASM module carries custom build metadata.

The exact complete source candidates are retained:

| Source candidate | Archive bytes | SHA-256 |
| --- | ---: | --- |
| WOFF2 `a0d0ed7da27b708c0a4e96ad7a998bddc933c06e` | 39,047 | `0be399be51907a9deb40b0b5b680df8151e99ee3aa9eb7d2505896ead13675d9` |
| Brotli parent gitlink `533843e3546cd24c8344eaa899c6b0b681c8d222` | 23,816,886 | `60209cc58dfe13d260199b149c3585ddc89fd02f5ad4d3b47cebfef51a1c7ec7` |
| Emscripten 2.0.34 | 28,243,170 | `a182b0c9d96b3a326f37b8126efe213ad78cfcc25e2eb246ee476ca199590fd6` |

The notice proposal supplements wawoff2's wrapper MIT license with full
WOFF2/Brotli MIT notices, Emscripten LICENSE/AUTHORS, and separate musl,
libc++, libc++abi, compiler-rt and libunwind texts. It proposes Emscripten's MIT
alternative while preserving its entire original dual-license/Node notice.
The separately copied sublibrary texts retain their own terms. No original
notice is edited or replaced by a generic SPDX label.

Still missing are an independently verified native build/linked-component map
or publisher attestation for the generated modules, and the immutable digest
of the historical compiler container. A matching wrapper snapshot and plausible
recipe do not establish those facts. These permissive licenses do not impose
a general corresponding-source offer; exact native provenance remains a project
gate. No replacement binary or new dependency was chosen to bypass it.

## Scoped source and recipient packet

The full 146,536,987-byte HyperFrames repository archive remains **private
reference evidence only**. Its producer HDR fixture NOTICE describes a
publisher-owned YouTube excerpt without a redistribution grant. The core
Apache license does not admit that media. Neither that clip nor the full
archive is in the proposed recipient source mirror.

The separate scoped export contains **706 unchanged upstream production-source
and build-control files**, the 18 original font licenses, two supplemental font
notices, scope directions and a source manifest: **728 regular files** total.
It excludes test/fixture directories, conventional test/spec files, docs,
compiled dist, unselected workspace packages and binary/media files. It is
source evidence for the selected six components, not a claim of a complete,
independently rebuilt monorepo workspace. No runtime file was pruned.

| Frozen local proposal | Bytes | SHA-256 |
| --- | ---: | --- |
| `hyperframes-0.8.197-scoped-source-20261026.tar.gz` | 2,954,796 | `e6906d075e16ab22e4c0b505bec3fed5e3511c010a186d6e2613b8819ea674f1` |
| `video-recipient-source-notice-proposal-20261026.tar.gz` | 381,751 | `98ee5cf46e79194a3812b6075dd9de609677c10355f55f610216a437ed5f7a1d` |

The recipient packet contains **238 files, 1,614,895 regular uncompressed
bytes**. Its 232 copied inputs retain exact original bytes. It includes the
unpruned package notice candidates, missing Puppeteer root license, four
unchanged Puppeteer third-party modules with their complete embedded notices,
the BiDi mapper notice, supplemental esbuild/xxhash/Go material, embedded
font/WASM notices, Mirket source/patch/lock captures, attestations, trust evidence
and readable indexes. It contains no executable runtime selection.

`SOURCE-INDEX.json`, SHA-256
`fd49fdb3b8bb04554eb0af0db8e2fe334b7cacf9cab29e12a86dbb1bcc1407cb`,
binds **111 retained archives, 193,165,366 bytes**, through exact size/hash and
proposed relative same-release asset paths. These are 86 distinct current npm
archives, 18 font archives, six other upstream source archives, and the scoped
HyperFrames source export. The indexes distinguish published compiled packages
from native source candidates. Sources stay separate from installed runtime
files and startup hashing. These private local references are not a published
source offer or verified recipient download.

Two independent serializations of each generated archive matched. A fresh
recipient unpack verified the exact packet manifest; all 111 referenced archive
identities were rechecked. Changed notice bytes, an unexpected file and a
corrupt source archive were refused. The scoped source manifest verified and
contains no binary/media file. The packet's manifest is SHA-256
`c0b2e9a5a24292b22a0b47f0263da71d0a623c55b527f3d54c7f7f95c951c409`.
No named builder-home marker was found in the exported packet; this bounded
check is not a general privacy certification.

## Remaining release-safe work

Keep this as a separate proposal and the release flag closed. The next assembly
must bind the current unpruned runtime manifest, eight patch hashes, policy
receipt, exact private executables and these notice/source identities without
runtime npm installs or downloads. Then:

1. Resolve the wawoff2 native provenance gate through a bounded reviewed rebuild
   or adequate publisher evidence; retain its original bytes until a separate
   artifact selection is justified. Finish esbuild's native Go/xxhash mapping
   and Chromium-BiDi's bundled-source reconciliation. BiDi's mapper retains
   bundled Zod **4.6.5**, distinct from the separately locked Zod 4.6.7.
2. Either prove the entire six-instance Sharp subtree absent through the
   installed video negative control, or select and qualify one exact shared
   native image artifact with matching rights/source/replacement materials.
   The original full libvips binary is not cleared by the separate reduced
   raster candidate's evidence. No pruning or substitution occurred here.
3. Carry all selected notices and required native source/modification materials
   through the actual durable same-release recipient download/install path.
   Scope any mirrored source and assets to verified reuse rights; do not mirror
   the excluded HyperFrames test media. Finish aggregate terms/rights review,
   release authentication and current advisory checks.
4. After private Chromium qualifies and the native lane is clear, run installed
   offline exports, configuration-loader and module-resolution negative
   controls, actual format/content/audio/caption checks, cancellation and owned
   cleanup. macOS 15, **native Windows 11 x64 and ARM64**, representative load
   and Codex Desktop GUI activation remain mandatory separate gates.

Source identity, copied notices, signed provenance and manifest verification do
not establish runtime fidelity, recipient access or release acceptance.

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/video-recipient-closure-20261026/`:
`provenance-verification.json`, original `attestations/`, private `trust/`,
`wawoff2-source-mapping.json`, `embedded-font-mapping.json`, font/source
acquisition and reconciliation records, `hyperframes-scoped-source-result.json`,
`recipient-proposal/`, `assembly-result.json` and
`recipient-verification.json`. The prior payload review retains the original
86 npm archives and full upstream source references. No reviewed target module,
upstream build/generator/installer, native media/browser process or publication
ran. The repository change for this follow-up is this new document only.

The later [Sharp-free installed qualification fork](m3-video-installed-route-preparation.md#disposable-installed-qualification-fork)
resolves the technical Sharp-absence control in item 2 for the reduced
candidate: the six subtrees were absent, module resolution failed, and the
installed export succeeded. This does not retroactively qualify the earlier
unpruned candidate or close source, notice and release rights for the reduced
one.
