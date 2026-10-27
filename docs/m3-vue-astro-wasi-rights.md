# M3 Astro WASI rights and source-closure review

Observed **2026-10-25** from the exact private payload used by the
[Vue/Astro qualification](m3-vue-astro-navigation-qualification.md). This review
changed only this document and private evidence. It did not build upstream code,
run upstream helpers, install dependencies, activate a client, or admit a release.

## Decision

The inspected source and package licenses support a finite **free/open-source
redistribution path**. There is no identified non-free SPDX expression in the
reviewed Astro dependency closure. A sole MPL-2.0 dependency requires retaining
its source and recipient notice if its covered code is distributed. Optional GPL
or LGPL branches in other expressions do not force selecting those branches.

**The exact published WASM is not yet a closed release input.** Its Cargo and
npm source pins are recoverable, but neither the published archive nor the
captured provenance identifies all linked Rust/WASI system-library inputs. The
recipient notice/source set has not been assembled into the release. This is a
bounded evidence and packaging gap, not a finding that the software is non-free.
Reproducible compilation is not itself a condition of the inspected permissive
licenses; the separate project gate requires enough correspondence to identify
what is being distributed. Do not label the whole binary simply “MIT” or treat
npm SPDX fields as a compiled-component bill of materials.

## Exact artifacts and source correspondence

| Input | Bytes / SHA-256 | Source evidence |
| --- | --- | --- |
| `@astrojs/astro2tsx` 0.1.3 npm archive | 504,488 / `ecba34fe481da75b92a838df1631863010ebcf68391edd58f48b0db75d5e62bc` | MIT body in archive; compiler-rs source `342e26603bf6416545a85b506b434d25226e1435`. |
| `astro2tsx.wasm32-wasi.wasm` | 1,677,030 / `1804c946bc37959ed893a42f1acc50d586d0eeb3ed89a8dc4a6bb6922eb0abc4` | The published executable being reviewed; target requested by source package is `wasm32-wasip1-threads`. |
| compiler-rs source archive | 389,855 / `2455866699fddb245c400010fd0b5341151ee14d6cf3b076e5dd17f74a336ebf` | [Immutable source archive](https://codeload.github.com/withastro/compiler-rs/tar.gz/342e26603bf6416545a85b506b434d25226e1435). |
| Compiler `Cargo.lock` | SHA-256 `7c62ffd73f811f1e13cd78e6ca3aa80fdf13ecdd904f4dce50cb8b427a81025e` | [Exact lockfile](https://github.com/withastro/compiler-rs/blob/342e26603bf6416545a85b506b434d25226e1435/Cargo.lock). |
| `@napi-rs/wasm-runtime` 1.2.4 npm archive | 157,157 / `e64c2013eba51ad6624ccf94f06c8a570c0d80488b7eabf17af2934690ccb887` | Registry `gitHead` and captured provenance identify napi-rs `7e3f293e2d6a3032eabfe51ff38bcaa82d342a2f`; upstream MIT body contains LongYinan and GitHub notices. |
| napi-rs source archive | 2,755,240 / `a5124a33770eef064fafc98414c53ea5c598285ca20d69dea99d8ca44c39f384` | [Immutable source archive](https://codeload.github.com/napi-rs/napi-rs/tar.gz/7e3f293e2d6a3032eabfe51ff38bcaa82d342a2f). |
| Biome source archive | 16,385,399 / `fb476d040049f77ee7d14370d77cb3e04c57d4702577e794a9e09e2eb7bde74c` | [Exact git dependency](https://codeload.github.com/biomejs/biome/tar.gz/4eadd575d6890a785998279193ceeff9d3098850). |

The compiler lock contains **297 packages**: 249 registry crates, 27 Biome git
crates, 18 OXC git crates, and three workspace crates. A conservative graph walk
from `astro2tsx` selects **196 packages**: 168 registry crates, the 27 Biome crates,
and `astro2tsx`. The walk follows every lock dependency edge, including development,
platform and feature edges. It is deliberately an upper bound, **not a claim that
196 packages contributed code to the WASM**. The OXC/lightningcss compiler branch
is outside that graph. The npm package version is 0.1.3 while its workspace Rust
crate version is 0.1.0; both identities must be retained. The
[Astro crate manifest](https://github.com/withastro/compiler-rs/blob/342e26603bf6416545a85b506b434d25226e1435/crates/astro2tsx/Cargo.toml)
provides that distinction.

Every one of the 249 registry crate archives was downloaded inertly and verified
against its **Cargo.lock SHA-256 checksum**, totaling 31,184,972 archive bytes.
Manifests, VCS hints and notice files were retained. The exact SPDX expressions
for each name/version and the conservative graph membership are recorded in
private `cargo-registry-rights.json`; no manifest was executed.

The captured npm provenance statements bind each selected npm archive's SHA-512
to its package/version and identify the source commits above. Their payload
subject digests match the selected archives. **The Sigstore trust/signatures were
not independently verified.** These captured statements provide a source lead
and declared publication association; they do not supply a linker input list,
prove every source was used, or establish byte-reproducible output. The
[Astro statement endpoint](https://registry.npmjs.org/-/npm/v1/attestations/@astrojs%2fastro2tsx@0.1.3)
and [napi-rs statement endpoint](https://registry.npmjs.org/-/npm/v1/attestations/@napi-rs%2fwasm-runtime@1.2.4)
are retained as original bytes.

## Rust and bundled JavaScript license findings

The 168 registry crates in the conservative Astro graph declare these exact
SPDX-expression families (legacy `/` separators are preserved from manifests):

- `MIT`, `MIT OR Apache-2.0`, `Apache-2.0 OR MIT`, `MIT/Apache-2.0`, `Apache-2.0`.
- `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`.
- `Unlicense OR MIT`, `Unlicense/MIT`, `0BSD OR MIT OR Apache-2.0`, `ISC`.
- `Zlib`, `MIT OR Zlib OR Apache-2.0`, `Zlib OR Apache-2.0 OR MIT`.
- `(MIT OR Apache-2.0) AND Unicode-3.0`.
- `MIT OR Apache-2.0 OR LGPL-2.1-or-later`.
- `MPL-2.0`.

`option-ext` **0.2.0** is the sole solely reciprocal expression in this graph:
`MPL-2.0`, reached through `directories` → `dirs-sys`. Its exact crate archive
contains `LICENSE.txt` and the preferred Rust source. Conservative packaging can
include that source regardless of whether a future target/feature/link report
proves its code absent. If covered executable code is distributed, recipients
must be informed how to obtain its source under MPL-2.0, with notices preserved;
the MPL permits a larger work under other terms subject to those requirements.
This is a free-software source duty, not a commercial-use prohibition.
[Mozilla MPL-2.0 §§3.1–3.4](https://www.mozilla.org/en-US/MPL/2.0/).

`r-efi` 6.0.0 offers MIT or Apache-2.0 as alternatives to LGPL; select and record
a permissive branch. The full 249-registry lock superset also contains
`self_cell`'s `Apache-2.0 OR GPL-2.0-only` and nine further MPL crates outside the
Astro graph. They are not evidence that GPL or CSS processing was linked into this
binary. If the whole workspace source is repackaged, its broader source rights
still need preservation. SPDX `OR` permits a choice; `AND Unicode-3.0` requires
both the selected code license and Unicode notice.

All 27 Biome graph crates inherit `MIT OR Apache-2.0`. Their own notice files
include additional Rome/RSLint authors and Apache-2.0 codespan material, including
`crates/biome_console/CODESPAN_LICENSE` and
`crates/biome_js_syntax/RSLINT_LICENSE`; the top-level license alone loses these
identities. Preserve those bodies and any file-level notices from the source
actually included. The original Biome archive also contains benchmark/test
corpora with separate attribution, including Wikipedia CC-BY-SA notices. It is
private review evidence, **not an automatically cleared whole-repository source
redistribution artifact**. A source release needs an enumerated build-source set
and retained notices, or a separate review of the extra corpora.

The npm runtime archive omits its standalone napi-rs MIT body. The selected source
body was already captured by the qualification, and must accompany distribution.
More substantially, its `dist/fs.js` is a **639,375-byte bundle containing 47
upstream npm package names**, including memfs's `@jsonjoy.com/*` modules, Node
polyfills, `qs`, `punycode`, `ieee754`, `readable-stream` and `tslib`.
`dist/emnapi-plugins.js` and `.cjs` separately embed `@emnapi/core` plugin code.
These are distributed code even if the finite Astro path does not import the
`./fs` export. The
[exact bundler configuration](https://github.com/napi-rs/napi-rs/blob/7e3f293e2d6a3032eabfe51ff38bcaa82d342a2f/wasm-runtime/rolldown.config.js)
confirms those inputs.

The napi-rs source lock and embedded module-region names were used to capture
**66 exact candidate npm archives**, including alternate lock versions and the
build inputs `emnapi` 1.11.3 and `@napi-rs/cli` 3.7.4. All archive integrity
values matched registry SHA-512 metadata. Their metadata licenses are MIT,
BSD-3-Clause, Apache-2.0 or 0BSD. This is a conservative candidate set, not an
exact source map of every transformed region: duplicate lock versions and source
patches remain distinguishable in `embedded-npm-rights.json` and the captured
`.yarn/patches`. The source lock resolves plugin `@emnapi/core` to
**2.0.0-alpha.5**, distinct from the separately installed **1.11.3** core peer.
The small inline BSD/MIT banners in `dist/fs.js` are not a replacement for all
required copyright/license bodies. Resolve bundle versions and retained notices
before copying the complete npm runtime into a release; do not silently inherit
the qualification's smaller npm dependency inventory as a bundle inventory.
The candidate `glob-to-regex.js` 1.2.0 archive has no standalone notice body;
its declared MIT terms still need an exact source-body supplement. Candidate
`@tybys/wasm-util` 0.9.0/0.10.3 archives have the same omission; those are source-lock
candidates, distinct from the qualification's separately installed 0.10.4.
Do not copy a neighboring release's notice while claiming exact provenance.

## Exact linked inputs still missing

Inert source inspection establishes the following link boundary:

1. Astro's `build.rs` calls `napi_build::setup()`; the exact `napi-build` **2.3.1**
   WASI code links static **`emnapi-basic-mt`**, and can add the Rust target
   sysroot's **`crt1-reactor.o`**. If `WASI_SDK_PATH` is set and contains it, the
   code also links **`libsetjmp.a`** from that SDK.
2. `@napi-rs/cli` **3.7.4** sets `EMNAPI_LINK_DIR` to the selected
   `emnapi/lib/wasm32-wasip1-threads` directory. The exact **emnapi 1.11.3** npm
   archive is captured. Its `libemnapi-basic-mt.a` is **26,210 bytes**, SHA-256
   **`2d1a49de33c9940a5907a6068b0ab0b64d0c47a2a7f0926b7f00e57aa2f91c07`**;
   metadata identifies source `b2011ea96d28fe60a6b8eabb82c2d6264daf7460`.
   That is a qualified candidate link input, not proof that this exact archive
   supplied the published binary.
3. The WASM contains Rust `/rustc/31fca3adb283cc9dfd56b49cdee9a96eb9c96ffd/`
   and Biome `4eadd57` source-path strings, consistent with the captured source
   family. Its only custom section is `build_id`; it has no producers or linked
   dependency/source manifest. Source-path strings cannot establish complete
   input correspondence or the exact standard-library/sysroot archive bytes.
   The official Rust commit API identifies that commit as the **1.96.1 stable
   release**; its root MIT/Apache bodies and COPYRIGHT were captured. This narrows
   the Rust source identity, but does not enumerate or hash the target sysroot
   and C library objects.
   [Exact Rust commit](https://github.com/rust-lang/rust/commit/31fca3adb283cc9dfd56b49cdee9a96eb9c96ffd).
4. The captured Astro release workflow selects stable Rust, latest Node and a
   napi build script; the repository also records Rust 1.96.1. No captured
   statement fixes the final Rust target package, C library/compiler-builtins
   contents, external WASI SDK presence or linked object list. Those system
   sources/notices cannot be inferred from Cargo.lock.

These points are directly inspectable in the captured `napi-build` source,
[napi CLI source](https://github.com/napi-rs/napi-rs/blob/762a0e389a0196d7446666ee5ef8468994dcac4f/cli/src/api/build.ts),
[Astro release workflow](https://github.com/withastro/compiler-rs/blob/342e26603bf6416545a85b506b434d25226e1435/.github/workflows/release.yml)
and WASM section/string inventory. Rust's general dual-license policy is useful
context, but its own policy directs readers to component repositories; it does
not identify the third-party system objects in this selected binary.
[Rust licensing policy](https://rust-lang.org/policies/licenses/).

## Finite recipient-visible packaging contract

A qualified Astro release should contain the following concrete artifacts,
verified against its final selected binary hash:

| Artifact | Required content |
| --- | --- |
| `THIRD_PARTY_NOTICES/astro-wasi/` | Astro MIT; chosen Cargo license branches and all required bodies/attributions; Biome nested notices; napi-rs's two MIT notices; embedded JS notices; Rust/system-library notices and applicable Unicode/LLVM exceptions. Preserve upstream NOTICE files. |
| `SOURCE-MANIFEST.json` | Every shipped binary/bundle hash → exact source package/commit, archive hash, license choice, patches, scope and relative notice/source paths. Distinguish Cargo build/dev/platform candidates from actual linked inputs and bundled JS from npm external dependencies. |
| `SOURCES/astro-wasi/` | Exact Astro and required Biome build source; checksum-verified registry sources; corresponding emnapi C/support source; Rust target/system sources once identified; napi-rs runtime source and the resolved embedded JS source. Preserve preferred source forms and file notices. Avoid unreviewed benchmark/doc assets in a minimized build-source archive. |
| `SOURCE-AVAILABILITY.md` | Recipient-visible MPL-2.0 information and a local source path for `option-ext` 0.2.0 (and any other covered code actually shipped), plus license location. A private home path or an unpinned moving repository link is insufficient. |
| `BUILD-CORRESPONDENCE.json` | Actual target/features, compiler/linker and sysroot archive identities, environment affecting linkage, emnapi archive identity, source/lock/patch hashes, command record and final output hash. Include a link-input report or equivalent artifact record that resolves the system-library gap. |

The lowest-risk closure path is a controlled build from the admitted free source
and exact system inputs, shipping that newly hashed output and rerunning the
existing semantic/runtime qualification. It must be separately authorized and
reviewed: this review did **not** run the upstream napi helper or a build. A new
build need not claim to reproduce the old hash; it must truthfully identify its
own inputs and output. Alternatively, sufficiently complete upstream build
records can close the selected published artifact's missing correspondence.

If that work is deferred, the smallest qualified product alternative is to defer
Astro's compiled transform while retaining independently qualified Vue/TypeScript
work. A Vue-only release must actually omit this WASM/runtime bundle and have its
own manifest/notice closure; changing a capability flag while copying the whole
private payload does not remove the distribution boundary. An older Go/WASM
Astro compiler is a new candidate with its own source, notice and behavior gates,
not an automatic license or compatibility substitute.

## Evidence and verification limits

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/vue-astro-qualification-20261025/wasi-rights-20261025/`:
`cargo-registry-rights.json`, `astro2tsx-lock-closure.json`,
`embedded-npm-rights.json`, `embedded-fs-modules.json`,
`source-captures.json`, decoded provenance payloads, `wasm-inspection.json`,
source/build-script captures and original archives. Capture scripts contain no
installation/build commands. Registry checksum/integrity checks were exercised;
WASM section parsing, source-marker inspection and notice-body inspection were
inert. This review establishes the stated source/rights findings and remaining
conditions, not a legal opinion, cryptographic provenance verification,
reproducible build, complete file-by-file copyright audit, release admission,
security containment or desktop activation.
