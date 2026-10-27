# M3 Astro WASI source and notice closure follow-up

Observed **2026-10-25** on macOS ARM64. This follows the
[WASI rights review](m3-vue-astro-wasi-rights.md) for the unchanged
`@astrojs/astro2tsx` 0.1.3 WASM, SHA-256
`1804c946bc37959ed893a42f1acc50d586d0eeb3ed89a8dc4a6bb6922eb0abc4`.
Only this document and private source evidence were added. No upstream build,
helper, installer, dependency admission, client activation or release occurred.

## Result

The Cargo and Rust/WASI source-and-notice gap is now substantially narrower and
has a concrete **34,114,752-byte private candidate set**. It contains 162 source
archives, the preserved license/attribution files, a checksum inventory and a
local MPL source-availability notice. Five napi crate notice omissions were
repaired from each crate's exact VCS commit. The official Rust standard-library
notice aggregate was recovered and verified. No non-free Rust dependency was
identified in the publication build records.

**This does not admit the complete published Astro runtime.** The exact release
job records compilation, but does not expose a linker input list or the complete
effective build environment. The optional external WASI SDK/setjmp condition is
still unobserved. The final shipped JavaScript files, their embedded source/patch
mapping and recipient-visible package integration also remain open. A linked
input report is a correspondence gate for this project; it is not asserted to
be a universal requirement of the inspected permissive licenses.

## Publication evidence is stronger than the lock walk

The npm provenance's invocation identifies
[release job 115818580181](https://github.com/withastro/compiler-rs/actions/runs/38554023819/job/115818580181).
The GitHub connector retrieved its actual logs. The unauthenticated REST log
endpoint returned HTTP 403; this was an access limitation, not missing logs.
The saved decoded log is 108,769 bytes, SHA-256
`28ee24cb7fe28dd9fabd6448a396840990aa1c478272f2c6e9ee44f1373caa6e`.
Its build and npm publication steps succeeded. The run failed afterward while
dispatching the playground update, so the run's overall failure does not imply
the published compiler was never built.

The log shows the source checkout
`342e26603bf6416545a85b506b434d25226e1435`, the package build command
`napi build --platform --no-js --target wasm32-wasip1-threads --release`, and
the workspace toolchain override to Rust **1.96.1**. The workflow also installed
stable 1.99.0; treating that installation message as the compiler used would be
incorrect. The WASM's `/rustc/31fca3adb.../` source marker agrees with the
workspace override and official 1.96.1 release identity.

| Evidence level | Observed set | What it establishes |
| --- | --- | --- |
| Entire compiler lock | 297 packages; 249 registry | Recoverable lock candidates across the whole workspace. |
| Earlier conservative Astro lock walk | 196 packages; 168 registry + 27 Biome + Astro | Dependency-edge upper bound including development, platform and feature edges. |
| Exact publication build progress | **144 packages; 116 registry + 27 Biome + Astro** | Packages compiled in this release job, including host build scripts and proc macros. |
| WASM source-path markers | **8 registry + 11 Biome** | These specific source identities occur in the selected binary's data. |

The eight registry identities evidenced by binary paths are `hashbrown` 0.15.5
and 0.17.1, `htmlize` 1.1.0, `indexmap` 2.14.2, `napi` 3.8.3, `smallvec` 1.16.3,
`unicode-segmentation` 1.13.3 and `unicode-width` 0.2.2. The eleven Biome identities
are `biome_console`, `biome_diagnostics`, `biome_html_factory`,
`biome_html_parser`, `biome_html_syntax`, `biome_js_factory`, `biome_js_parser`,
`biome_js_syntax`, `biome_parser`, `biome_rowan` and `biome_text_size`, at
`4eadd575d6890a785998279193ceeff9d3098850`.

Neither compilation nor an absent source marker proves whether a crate's code
survived target selection, optimization, linker GC or LTO. In particular,
`option-ext` 0.2.0 is **observed compiled**, but is not binary-path evidenced.
The candidate conservatively supplies its complete 7,345-byte source archive,
SHA-256 `04744f49eae99ab78e0d5c0b603ab218f515ea8cfe5a456d7629ad883a3b6e7d`,
and its unmodified MPL-2.0 body, SHA-256
`66a3107d5ad6a058aab753eaac2047ccb2ed0e39465dd0fe5844da3e300d5172`.
This avoids relying on unproved dead-code removal to discharge
[MPL-2.0 §§3.1–3.4](https://www.mozilla.org/en-US/MPL/2.0/).

## Exact Rust and WASI artifacts recovered

The official [1.96.1 distribution manifest](https://static.rust-lang.org/dist/channel-rust-1.96.1.toml)
is dated 2026-06-30. Its bytes matched the independently fetched SHA-256 sidecar.
The following archives matched that manifest or the official release asset
digest, as applicable. They were opened as data; their installation scripts and
executables were never run.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `rust-std-1.96.1-wasm32-wasip1-threads.tar.xz` | 23,058,524 | `77ed5d64d661ecfc15b65774804a3abb217730a43354e6fc562aacbe01432f33` |
| `rust-src-1.96.1.tar.xz` | 3,868,228 | `b343b6553bc772225f6a2b5be5055017f29794dcf4e08020366b27a0a40fc1c2` |
| `rustc-1.96.1-x86_64-unknown-linux-gnu.tar.xz` | 80,218,896 | `3545a0efad2355ecb0a3b9ac02efee96e27f1f9d24b7ce2fc3f279b2efb0d923` |
| `wasi-sysroot-30.0.tar.gz` | 70,466,982 | `0c8f3e0acab6d3061834395a16550c359766909fda046af3d9b8cf3bfdb1c2cc` |
| wasi-libc source `ec0effd769df5f05b647216578bcf5d3b5449725` | 1,631,604 | `8620290bc982bdfbd4686f383ef1f35395b37d7ad611062838aa9ad2f3b12c63` |
| emnapi source `b2011ea96d28fe60a6b8eabb82c2d6264daf7460` | 416,423 | `83e8952bab1f13b8196701d50d93a6edbfc327376751f7224dbc8bc089c3512d` |

The target archive embeds commit
`31fca3adb283cc9dfd56b49cdee9a96eb9c96ffd`, the release version, its builder
configuration, the Rust rlibs and three self-contained WASI objects. Its
`libc.a` and `crt1-reactor.o` are **byte-identical** to the official
[WASI SDK 30 sysroot](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-30):

| Object | Bytes | SHA-256 |
| --- | ---: | --- |
| `libc.a` for `wasm32-wasip1-threads` | 3,343,790 | `f6ba701780d6a1960f4a6e774aaf0caf08d566e270bd7cd61a4c238abe4af85e` |
| `crt1-reactor.o` | 1,844 | `7d8c5bf97d1f2b08419717ca97bb8cdd3274c8995a75c192d30961187db01b6d` |

This is actual object correspondence, beyond inference from a generic Rust
license policy. The exact Rust CI recipe selects SDK 30, and its SDK tree pins
wasi-libc to the source commit above. The separate SDK LLVM commit is
`222fc11f2b8f25f6a0f4976272ef1bb7bf49521d`; it must not be confused with
Rust's LLVM submodule.

The target builder configuration enables optimized compiler builtins. Therefore
`rust-src` alone is not the whole preferred-source set: the candidate also
captures the **211 generic/wasm compiler-rt source and license files** from
Rust's exact LLVM submodule
`1cb4e3833c1919c2e6fb579a23ac0e2b22587b7e`. All 209 source-file Git blob hashes
were checked; the two root license bodies were also captured. The generated
97,829-byte subset archive has SHA-256
`ae02dc11b14db8d7457a31caeb3bfd4d8ab57ec19989c338864dfc2b79a8a386`.
This is a source superset for the relevant builtin family, not a claim that
every file was linked. Forty registry archives from Rust's own `library/Cargo.lock`
were independently fetched and verified against its checksums, totaling
12,052,722 bytes; this separate conservative source set includes other-target
and development entries.

The checksum-verified rustc archive supplies
`rustc/share/doc/rust/COPYRIGHT-library.html`: **425,651 bytes**, SHA-256
`78c163fcec50e64bfd85fedb850c273595602fafa2b41f30f75d4e410b80ee83`.
It is included with Rust's root notices, nested source licenses, compiler-rt
terms, wasi-libc's cloudlibc/musl/musl-fts notices and conservatively retained
inline C-source attribution. Calling all these components simply MIT would
lose required identities and exceptions.

## Prepared source and notice candidate

Private evidence lives under
`~/.codex/task-state/harness-rewrite-implementation/vue-astro-qualification-20261025/wasi-source-closure-20261025/`.
The `candidate-source-notices/` directory is an inspectable handoff, containing:

- `SOURCE-MANIFEST.json`: source and notice paths, archive hashes, declared
  expressions, candidate license choices and evidence level for the 116 registry
  packages compiled in the publication job; Astro, Biome and system source records.
- `SOURCES/cargo/`: those 116 exact checksum-verified archives, including the
  complete unmodified `option-ext` source. `SOURCE-AVAILABILITY.md` points to the
  local archive and MPL body.
- `SOURCES/astro/`: the exact compiler-rs source archive. `SOURCES/biome/` contains
  a 333-file preferred-build-source subset for the 27 compiled crates, keeping
  generated Rust, build scripts, ARIA input data, manifests and notices while
  excluding the separately attributed repository benchmark/test corpora.
- `SOURCES/system/` and `SOURCES/rust-library-registry/`: the Rust source archive,
  compiler-rt subset, wasi-libc and emnapi sources, and forty checksum-verified
  Rust-library registry archives.
- `THIRD_PARTY_NOTICES/`: all captured Cargo bodies, exact missing napi bodies,
  nested Biome notices, Rust's standard-library aggregate, system notices and
  inline wasi-libc notice material.

The five `.crate` archives lacking their standalone notice body were
`napi-build` 2.3.1, `napi-derive-backend` 5.0.2, `napi` 3.8.3, `napi-sys` 3.2.1
and `napi-derive` 3.5.2. Their `.cargo_vcs_info.json` files identify napi-rs
commits `67e711d69b7e681807d157ea85cf8173fd923537`,
`0b80be8d9e64ba6c1733fcf31533e4f58bfc46b4` and
`be4b16ca00aa2cecd19be6ffe6de59495b471b14`. The root notice captured from each
is the same 2,138-byte two-author MIT body, SHA-256
`3f1ce66533302df3a32edbfdfc0b78f0dd34659e4c1f5817162e5ea3c2297215`.
The candidate records and retains it rather than borrowing an unpinned adjacent
release's body.

The Biome source subset includes W3C-derived ARIA JSON build data. It retains
the data's exact specification permalinks plus a W3C attribution supplement,
document status and the applicable 2015/2023 document-license bodies. The
[W3C permission](https://www.w3.org/copyright/document-license-2023/) expressly
addresses derivative material in software and supporting materials; the subset
is not presented as a new technical specification. These source-only data inputs
must not be silently relabeled by Biome's top-level SPDX field. Their sampled
description strings were absent from this WASM; that sampling does not prove
all related generated code absent.

## Remaining release boundary and least disruptive path

The recovered job log does not contain `cargo -vv` output, a linker map, final
effective feature/target configuration, sysroot file hashes as consumed by the
Astro build, or a full environment dump. The published binary remains stripped
apart from `build_id`. Source and default build behavior support the identified
Rust/SDK family; they do not prove that the helper's optional `WASI_SDK_PATH`
branch and `libsetjmp.a` linkage were absent. The lock and helper source identify
emnapi 1.11.3 and its candidate static library, but the log does not hash the
archive actually linked. This remaining uncertainty is stated, not filled in
with an invented linker record.

The complete npm runtime also remains a separate distribution boundary. Its
CommonJS `runtime.cjs` requires only `@emnapi/core`, `@emnapi/runtime`,
`@tybys/wasm-util`, `dist/emnapi-plugins.cjs` and `dist/fs-proxy.cjs`. The large
47-package `dist/fs.js` bundle belongs to the separate `./fs` ESM export.
A final finite CommonJS package can potentially omit that file and unused
browser/ESM files, but exclusion must be real in the package inventory and
validated by fresh module loading and Astro semantic tests. This review did not
modify the payload or claim those files were already omitted. The retained
plugin bundle still needs its exact `@emnapi/core` 2.0.0-alpha.5 source/notice
mapping, distinct from the external 1.11.3 peer.

The least disruptive completion is to retain the selected Astro engine, finalize
that minimal shipped file set, integrate the prepared source/notices, and obtain
an exact linked-input record. If upstream records cannot supply it, a controlled
build of the same pinned source and pinned Rust/SDK/emnapi inputs can produce a
new truthfully identified WASM, followed by the existing semantic/runtime
qualification. No byte reproduction of the published hash is promised. A
controlled build or final payload pruning is a next implementation/assembly
step; neither was executed during this inert dependency review. Replacing the
engine or deferring Astro is not necessary to explain the current evidence gap.

## Verification and limits

All 249 earlier Cargo archive SHA-256 values and all 66 earlier npm candidate
SHA-256/SHA-512 values were rechecked. The new official release archives, forty
Rust-library archives, compiler-rt source blobs, object comparisons and every
candidate source/notice file were checked against the recorded hashes. The
notice preparation initially failed on the five missing napi bodies, then
passed with exact-commit supplements. The candidate contains **729 files** and
162 source archives; it is a conservative source/notice handoff, not the claimed
minimum linked-component bill of materials.

The minimized Biome subset and the complete candidate have not been rebuilt as
standalone projects. No Sigstore trust/signature verification, final bundle
admission, security audit, Windows execution, installed packaging, desktop
activation, release publication or user acceptance is established here.
