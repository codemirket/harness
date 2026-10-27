# M3 finite Astro CommonJS runtime qualification

Observed **2026-10-25**, macOS **26.5.2 build 25F84 ARM64**, using the explicit
private **Node 24.22.1** executable, SHA-256
`6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329`.
This follows the [source and notice closure](m3-astro-wasi-source-closure.md)
and exercises the existing [Vue/Astro semantic fixtures](m3-vue-astro-navigation-qualification.md).

## Result and admission state

A private **41-file / 2,387,061-byte** Astro WASI leaf works with the published
CommonJS runtime and excludes the unused napi `dist/fs.js` bundle and its 47
embedded package-name regions. All **23 shipped JavaScript implementation files**
were actually loaded. Six transform/extraction/diagnostic/mapping cases match
the original complete runtime exactly, under both default and production
`NODE_ENV`, with the macOS network-denial profile active. Existing Vue/Astro
semantic cases also pass with the selected leaf overlaid into their trusted
fixture runtime.

**The leaf remains private and unadmitted.** It is not wired into shared
Rust/TypeScript, MCP, distribution assembly or an installed home. The published
WASM's exact consumed linker inputs and effective environment remain a release
gate, described below. No upstream installer, lifecycle hook, build helper,
user installation, commit, push or publication ran in this slice.

The implementation files are
[`scripts/assemble-astro-cjs-runtime.py`](../scripts/assemble-astro-cjs-runtime.py),
[`tests/test_astro_cjs_runtime.py`](../tests/test_astro_cjs_runtime.py) and
[`tests/astro-cjs-runtime.cjs`](../tests/astro-cjs-runtime.cjs). The assembler
accepts only the frozen archive hashes and exact license supplement, validates
all inputs before staging, refuses an existing destination, and executes no
package code. The new tests require explicit real archive and Node paths.

## Exact file selection and metadata change

Astro's published `index.js` is ESM. Its implementation bytes are retained.
It imports `@napi-rs/wasm-runtime`, whose original import condition selects
`runtime.js`. Merely deleting that file would break Astro; a real negative
control confirms `ERR_MODULE_NOT_FOUND` with the original condition restored.

The authored package metadata explicitly maps that dependency to the published
`runtime.cjs`. It also selects the unminified published CommonJS entries of the
emnapi peers, wasm-util and tslib, eliminating conditional wrappers and their
unused alternatives. Each package exposes only its selected entry and its
package metadata. Scripts are omitted. This changes package resolution and
supported exports; it changes **no JS implementation or WASM bytes**. This is
an internal finite payload, not a replacement distribution of every upstream
package API.

| Package | Files retained, in addition to authored `package.json` | Total |
| --- | --- | ---: |
| `@astrojs/astro2tsx` 0.1.3 | `index.js`, `astro2tsx.wasm32-wasi.wasm`, `LICENSE` | 4 |
| `@napi-rs/wasm-runtime` 1.2.4 | `runtime.cjs`, `dist/emnapi-plugins.cjs`, `dist/fs-proxy.cjs`, exact source `LICENSE` supplement | 5 |
| `@emnapi/core` 1.11.3 | `dist/emnapi-core.cjs.js`, `LICENSE` | 3 |
| `@emnapi/runtime` 1.11.3 | `dist/emnapi.cjs.js`, `LICENSE` | 3 |
| `@emnapi/wasi-threads` 1.2.3 | `dist/wasi-threads.cjs.js`, `LICENSE` | 3 |
| `@tybys/wasm-util` 0.10.4 | 15 files under `lib/cjs/`, `LICENSE` | 17 |
| `tslib` 2.8.1 | `tslib.js`, `LICENSE.txt`, `CopyrightNotice.txt` | 4 |

The remaining two files are the private root `package.json` and
`THIRD_PARTY_NOTICES/embedded-emnapi-core-2.0.0-alpha.5/LICENSE`.
The wasm-util selection is exactly `lib/cjs/{asyncify,index,jspi,load,memory,webassembly}.js`
and `lib/cjs/wasi/{error,fd,fs,index,path,preview1,rights,types,util}.js`.
Its own WASI filesystem implementation remains necessary; the separately
exported napi/memfs bundle is absent.

`runtime.manifest.json` pins every final relative file path and SHA-256, with
manifest SHA-256
`cf1ef1fe8724290cf175cec9ecb0eb3928b3f7267784dc32a3bba610a9b790c4`.
`astro-cjs-inventory.json` additionally records every archive member or authored
metadata origin, size and source commit; its SHA-256 is
`8fecf9cb3b0b8f3f072d1663dceed126c8e37f90c0fda3fab51e803b0a46a93b`.
Every implementation file matches its checksum-verified npm archive member.
The published WASM remains
`1804c946bc37959ed893a42f1acc50d586d0eeb3ed89a8dc4a6bb6922eb0abc4`.

## JavaScript source and notice closure

The recipient-shaped private source/notice companion reuses the earlier
34,114,752-byte Rust/WASI candidate and adds the selected JavaScript sources,
original npm archives and complete notices. It contains **751 files /
36,138,511 bytes**. `ASTRO-CJS-SOURCE-MANIFEST.json` binds the runtime inventory
to these component sources and notice paths, SHA-256
`053adbaa8718a9502ac90e5dfb3175ec0bf671a000ea4d896f10696021a631b5`.
The whole companion has a separate per-file checksum inventory.

| Distributed component | Exact source association | Terms / notice |
| --- | --- | --- |
| Astro ESM loader and WASM | compiler-rs `342e26603bf6416545a85b506b434d25226e1435`; previously captured exact source archive | Astro MIT plus the separately enumerated Rust/WASI source/notices; the whole WASM is not labeled simply MIT. |
| napi CommonJS runtime and fs proxy | napi-rs `7e3f293e2d6a3032eabfe51ff38bcaa82d342a2f`, paths under `wasm-runtime/` | Two-author MIT body, SHA-256 `3f1ce66533302df3a32edbfdfc0b78f0dd34659e4c1f5817162e5ea3c2297215`; published `runtime.cjs` exactly matches the source file. |
| External core/runtime/wasi-threads | emnapi `b2011ea96d28fe60a6b8eabb82c2d6264daf7460`; exact 1.11.3/1.11.3/1.2.3 npm archives | MIT body `2088f55c6731498410b20c95ebd345ed4a4a800bf74c0fba4d282fd2a021daaa`. |
| Embedded async-work and threadsafe-function plugins | `@emnapi/core` **2.0.0-alpha.5**, emnapi `ca282f418104016b17b9661af98c9791301f819c` | Exact plugin archive and preferred source captured; the same MIT body is separately retained beside the runtime. |
| wasm-util CommonJS modules | `51836d7bf26fb7f43b05dd6477b3ca4260c945cf` | Exact source/archive MIT body `09e436100bf926e78df875ec80cf3d0c643dfec779b52cfe2aa96afa0de714cb`. |
| tslib helpers | `d72d6f70b36286bc3f94a3dda1e64dcb568b1370` | 0BSD body `210b19e543130388c68654b7497e967119ce17145f66ab7d85688fbd70f08751` and original Microsoft copyright notice. |

The 59,988-byte shipped plugin bundle has SHA-256
`52ab7f50c419e212cedcfead479f51b8ad9b5c3c4a532ff8912ab9f5ea25822e`.
It contains exactly the two source regions
`@emnapi/core/dist/plugins/async-work.js` and
`@emnapi/core/dist/plugins/threadsafe-function.js`. The exact napi-rs source lock
resolves this input to unpatched **2.0.0-alpha.5**; it is not the external
1.11.3 core. Its original archive SHA-256 is
`f01629823357b6ddd7daaccb86f64f1bbbbc976368ae267b3f5c94b8b625f449`.
The package's declared source commit, complete source regions and lock association
are preserved. The bundler output was not reproduced; a direct normalized-AST
experiment retains differences from constant folding and temporary-variable
inlining and is not presented as equivalence proof. The previously captured
publication statements' Sigstore trust/signatures also remain unverified.

New preferred-source captures have these exact archive hashes:

- emnapi plugin source, 487,557 bytes:
  `aacc06247b7aa392a060dbaff583cde5e6ce07202293dd1ea4785da0fea176a2`.
- wasm-util source, 47,074 bytes:
  `d20054bc0ec13b17bd27d03625e176d9d1b4caa455133c85f6e00c50586ad413`.
- tslib source, 28,801 bytes:
  `a22b3db9f9bfd489b3015bb12d7c3c03a3a469a15d13e9149a76f4b42781d3f7`.
- A seven-file napi runtime/source-lock subset, 144,782 bytes:
  `b1ec58d1defcaa359a37bd4abcf4c9c857f4d466db8493f09b10220b0a0fbf2e`.
  Its member bytes are checked against the previously captured exact napi-rs
  source archive; it excludes the unused bundled fs implementation.

The source map records no applied code patch. Authored resolution metadata is
recorded separately. The source subset is not claimed to be a standalone
reproducible build project. Original npm archives, source files and notices are
available locally in the companion, including the existing complete MPL
`option-ext` source and local source-availability notice.

## Runtime, semantic and failure checks

`tests/astro-cjs-runtime.cjs` verifies the whole final file set and every hash,
then loads Astro from the selected leaf. Its observed module cache equals the
23-file shipped JS set; it reads the selected WASM once. Removed fs/ESM exports
refuse. Six cases compare complete output objects against the original runtime:
Unicode with CRLF, component props, styles and processed scripts, open
frontmatter, syntax errors and a source-side global assignment that must not
execute. Code, mappings, ranges, diagnostics and extracted content agree in
default and production modes. Each mode ran in a fresh Node process under the
existing macOS network-denial profile.

Four assembler tests pass against actual frozen inputs. They check genuine
assembly, rejection of a changed archive before staging, rejection of an
incorrect notice, preservation of an existing destination, and real loader
failure controls. Removing the embedded plugin or external runtime peer yields
`MODULE_NOT_FOUND`; corrupting the selected WASM yields `CompileError`; restoring
the original ESM condition while leaving the ESM file excluded yields
`ERR_MODULE_NOT_FOUND`. These controls prevent a hidden original-payload fallback
from looking like successful pruning.

For the existing native suite, a private copy of the full Vue/Astro fixture
runtime replaced only these seven package trees. The 25 Vue declaration files
remained byte-identical; the test overlay has a newly serialized manifest. The resulting semantic fixture has 4,260 files
and 66,308,120 runtime bytes; it is a test overlay, not a qualified minimum for
all Vue/TypeScript dependencies. The existing integration-test executable was
reused, SHA-256
`21ff3d66094c5b5efdfedee2b60af54a7a8eaf7032d422d0eac4364aaf528ad7`;
its source and fixture hashes exactly match the prior qualification. No Rust
compilation was needed.

The first full native run passed **18/19 tests in 90.17 seconds**, including the
real Vue/Astro definitions, references, property bindings, Unicode/CRLF spans,
structure, refusal and stale-data cases. Its only failure was the ps-based
Unicode child-observation assertion. The deliberately sanitized test-host
environment omitted the locale: macOS `ps` displayed `İ` as `M-DM-0` under the
C locale, so the exact marker lookup failed. A separate live-child control
demonstrates that escaping and that `en_US.UTF-8` preserves the marker. With
that locale selected, the **unchanged** cancellation/source-drift test passes
in **9.85 seconds**, including actual PID observation, reaping and empty scratch.
The failure and focused rerun are retained; this is not reported as a single
clean 19-test run. A narrow future harness fix could select an explicit UTF-8
locale for `ps` or observe the owned PID/process group directly. No shared
fixture/assertion was edited here.

Generic adapter cases in that executable still return early without their
separate adapter selection; their pass labels do not establish real adapter or
desktop activation. The Vue/Astro fixture selections were present and executed.

## Advisory check and exact remaining gate

On this date, the
[npm bulk advisory endpoint](https://registry.npmjs.org/-/npm/v1/security/advisories/bulk)
returned no matches for the seven selected package names and eight versions,
including embedded core 2.0.0-alpha.5. The
[OSV batch endpoint](https://api.osv.dev/v1/querybatch) returned no matches for
the 116 registry name/version pairs observed compiled in the Astro publication
job. Exact public-package requests and responses are retained. These lookups
do not establish binary reachability, exhaustive security coverage of Biome/git
sources, Rust/WASI system objects, compiler behavior or absence of vulnerabilities.

The remaining release gate is concrete: obtain the **actual consumed linker
input list and effective build environment for the selected WASM**, including
the compiler/linker identity and arguments, target/features, hashes of the
sysroot objects/rlibs and emnapi static archive as consumed, and whether
`WASI_SDK_PATH` caused an external SDK and `libsetjmp.a` to participate. The
official Rust 1.96.1 target's libc/reactor objects now have exact SDK 30 source
correspondence, but the stripped publication artifact and ordinary progress log
do not prove those defaults were the only consumed inputs. This task does not
invent that record or infer that the optional branch was absent.

Sufficient upstream build records could close that gate. Otherwise a controlled
same-engine build with explicitly pinned Rust/SDK/emnapi inputs and a captured
link report can produce a newly identified WASM, followed by the semantic/runtime
checks. Until then the 41-file leaf and its source/notices remain private and
unadmitted. Windows x64/ARM64, the oldest macOS, installed assembly and actual
Codex activation remain untested.

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/astro-cjs-runtime-qualification-20261025/`:
`candidate/`, source captures and correspondence files, raw/production runtime
reports, native logs, locale controls, advisory requests/responses and checksum
inventories. This review changed only the new assembler/tests, this document
and task-owned private evidence.

## Addendum: exact linker provenance and bounded replacement attempt

The **published WASM gate remains open**. The publication job's ordinary Cargo
log and npm statements do not supply the consumed linker input list or effective
environment. A fresh read of the
[workflow run's artifact listing](https://github.com/withastro/compiler-rs/actions/runs/38554023819)
returned an empty list. This establishes the result of that lookup, not that no
record could exist elsewhere upstream. No exact linker/environment record was
recovered from the retained public evidence.

A controlled **Rust 1.99.1** replacement was attempted locally, using the same
Astro and Biome source commits and registry package checksums. It was stopped
before final linking by the task's disk guard. **No replacement WASM was
produced, no final WASM linker recorder ran, and no admission is proposed.**
The original 41-file leaf remains private, byte-identical and unadmitted.

### Inspected build paths and input identities

The exact published `@napi-rs/cli` 3.7.4 source and `napi-build` 2.3.1 source were
read as inert source. The CLI inherits `RUSTFLAGS` or `CARGO_BUILD_RUSTFLAGS`,
selects `EMNAPI_LINK_DIR` through the resolved emnapi package, and checks its
version against core/runtime. If an existing `WASI_SDK_PATH` is present it also
selects that SDK's linker and C tools, sysroot and thread/SJLJ flags. The Astro
package also declares a `prepack` build; the explicit build-step log alone does
not establish the effective environment of every build during publication.

Astro's `build.rs` calls `napi_build::setup()`. For WASI that implementation
requests `emnapi-basic-mt`, its explicit exports/imported memory, a 64,000,000-byte
stack, 4 GiB maximum memory and `--no-check-features`. It derives the reactor
object location from `RUSTC`, `TARGET` and the compiler's neighboring sysroot.
The independent optional SDK branch adds its library directory and requests
`setjmp` only when `libsetjmp.a` exists. These branches explain why the public
progress log cannot establish the final input selection.

The following archives/objects were inspected without running an installer or
their package helpers:

| Input | Identity and observation |
| --- | --- |
| Published emnapi 1.11.3 `lib/wasm32-wasip1-threads/libemnapi-basic-mt.a` | 26,210 bytes; SHA-256 `2d1a49de33c9940a5907a6068b0ab0b64d0c47a2a7f0926b7f00e57aa2f91c07`. Its 14 WASM object members and archive metadata were separately hashed. The exact emnapi source commit's CMake source lists match those member names, including the uv files and the two async-worker source files. This is an exact selected archive identity and source association; that archive was not rebuilt here. |
| SDK 30 `lib/wasm32-wasip1-threads/libsetjmp.a` | 2,904 bytes; SHA-256 `5e5a1c4d2ce75d3ebb1caed370dab91283c835cc37ff505b2ddb2936c5e54072`. Its sole code member is `rt.c.obj`; the corresponding WASI-libc `libc-top-half/musl/src/setjmp/wasm32/rt.c` was inspected. This is optional-branch evidence, not proof that publication used it. It is kept outside the replacement's library search directory. |
| Official Rust 1.99.1 WASI target archive | 24,585,048 bytes; SHA-256 `5d44556bbae559fd029f207c8c2140b63ed70f0aa29f08be1bf42bed1bf7f716`. Explicitly fetched from the dated official release URL and checked against both the installed channel manifest and freshly captured official manifest/sidecar. Its embedded compiler commit is `58cf8f9c35b355199727627d07ae34a4e2846d67`. No rustup target installation occurred. |
| Rust 1.99.1 target `self-contained/libc.a` | 3,364,598 bytes; SHA-256 `31f94db527af4f9c7c7078d697d98b3ef4c8b3d36ddc71c27751243ec2d31079`. |
| Rust 1.99.1 target `self-contained/crt1-reactor.o` | 1,850 bytes; SHA-256 `19f897d74e508d0d2021fa4515b7ab5b561a930d65aa84e3d7d030704f091301`. |

The new target's libc/reactor bytes differ from the earlier Rust 1.96.1/SDK 30
objects. Its
[official Rust build Dockerfile](https://github.com/rust-lang/rust/blob/58cf8f9c35b355199727627d07ae34a4e2846d67/src/ci/docker/host-x86_64/dist-various-2/Dockerfile)
selects SDK **33**. This is a new source/notice closure to finish; the old SDK 30
byte correspondence cannot be reused for these new objects. SDK 33's pinned
source tree identifies WASI-libc
`161b3195fc2558d2b1ba3eb9ffae3b2b47407623` and LLVM
`4434dabb69916856b824f68a64b029c67175e532`; the SDK's sysroot CMake logic was
captured and inspected. A byte comparison against the SDK 33 release sysroot
was not attempted within the remaining budget.

### Controlled build and stopping evidence

The existing native compiler is Rust **1.99.1 / LLVM 23.1.1**, on macOS ARM64
with 16 GiB memory and 10 logical CPUs. Existing toolchain files were only read
or referenced. The private sysroot contains **58 target files / 109,702,428
bytes**, each hashed against the verified target archive; its host paths refer
to the existing toolchain. Host rustc, Cargo, rust-lld and compiler dylib hashes
are saved in `host-tools.json`.

All 168 registry archives in the earlier Astro lock closure were checksum
verified before private vendor staging. Existing cached source files were
hardlinked only after byte-hash equality with the archive members. The private
Cargo metadata narrows the workspace to Astro and the 27 selected Biome crates,
uses local Biome source paths, and removes unused development, test, benchmark
and example declarations. The resulting lock has 155 packages, including 127
registry entries. **Every registry name/version/checksum equals the original
lock.** These are resolved packages, not a claim that all were compiled or
retained in a binary. Nineteen metadata files/lock changes are recorded with
their original and final hashes. Rust implementation source is unchanged.

Thirty-three staged `build.rs` source files, included build helpers, napi's
WASI setup, and emnapi's build source lists were inspected. Their relevant
operations are compiler-version/feature probes, cfg/link declarations and
generation from already captured local data into `OUT_DIR`. The controlled
Cargo build intentionally compiled and ran its applicable reviewed build
scripts and proc macros. No npm lifecycle script, napi CLI helper, upstream
installer or automatic package/toolchain download ran.

The build used the original release profile: optimization 3, fat LTO, one
codegen unit, stripped symbols, no debug info and aborting panics. The saved
command is explicit `cargo build --locked --offline --release --lib --target
wasm32-wasip1-threads --message-format=json-render-diagnostics -vv`, run under
the existing network-denial profile. `HOME`, Cargo home, scratch and output
were private. The complete initial environment is saved; inherited toolchain
and Cargo settings are absent. In particular `WASI_SDK_PATH` is unset, so the
reviewed napi build script does not request an SDK or `-lsetjmp`; its requested
external archive is the selected emnapi library. No final link occurred to
establish an actual consumed-library list.
Rustc invocations record arguments, environment and file-argument hashes. A
WASM linker recorder was prepared to hash input objects/libraries and obtain
the link map, trace and archive-extraction reasons; final linking was never
reached.

Two preliminary failures are retained: omitted Biome examples still declared
in Cargo metadata, then the task-owned argument recorder treating a long cfg
expression as a filename. The private metadata exclusion and recorder were
corrected. They are not upstream compiler or semantic failures.

The repository's admitted-input guard was rerun successfully **after** the
budget stop, not immediately before this private attempt. Its SQLite/vector/rmcp
scope does not qualify the private Astro graph. That timing is retained as a
process limitation; any resumed compilation must run the project guard first
and separately verify this private graph.

The resumed compilation produced completed Cargo artifact messages for 81
package identities; six also had WASI-target artifacts. Main parser/final
Astro output remained unfinished. The monitor terminated the owned process
group after observing **435,085,312 newly allocated bytes**, with **435,601,408
bytes** at process exit. Its conservative 430 MB stop point leaves headroom
below the authorized 500 MB ceiling for concurrent compiler writes; this does
not establish the total space a completed build would require. Minimum sampled
free space was **3,694,592,000 bytes**, above the 2 GiB floor. A subsequent process
check found no remaining owned build command.

The 927 partial build-output files were hashed and removed. Retained staging
and evidence used **208,953,344 newly allocated bytes** immediately after that
cleanup, excluding pre-existing byte-verified hardlinks; later small metadata
captures are separately inventoried. No partial output is offered as a usable
WASM. Without a replacement artifact, no new original-versus-replacement
runtime or semantic comparison can be run. The earlier published-runtime
results above remain the observed behavior evidence.

### Exact inputs and host needed to finish

Continue this same engine on a disposable **aarch64-apple-darwin** build host,
or an explicitly qualified equivalent host, with the recorded Rust/Cargo/linker
toolchain, a native C linker for host build scripts/proc macros, Python 3.13 for
the recorders, the private Node 24.22.1 and existing semantic test executable.
Provide a larger task workspace allowance while retaining 2 GiB free. A 2 GiB
task allowance is a conservative proposed next budget, not a measured completed
build requirement. Keep automatic downloads disabled and capture the final
effective environment and all consumed target/link inputs before testing.

The input list is pinned, even where a capture is still required:

- Existing Astro, Biome, 168 registry archives, emnapi 1.11.3 archive/source and
  selected runtime/source-notice companion are the exact inputs already listed
  above and in the preceding closure document. Retain the private Cargo metadata
  changes and regenerated lock, or review any different metadata explicitly.
- Rust 1.99.1 aarch64-apple-darwin compiler release archive SHA-256
  `23948a0a9c162fd2952defb3fb0462c44e8fcfd1a4ab8a274a1e1bdb2cf810a3`,
  matching host standard library and Cargo component pins from the captured
  **2026-10-15** channel manifest, and the already captured matching WASI target
  archive `5d44556bbae559fd029f207c8c2140b63ed70f0aa29f08be1bf42bed1bf7f716`.
- Preferred Rust source: official `rust-src-1.99.1.tar.xz`, SHA-256
  `e485a88c45db546b73571df7bebfcb4454ba4fb552c492e25fc5b00eeb5bf22d`,
  and the Rust commit's LLVM submodule
  `1b9c0d5ff9bbe7634aead059efe6b11a7eeba145`, including the compiler-rt
  builtins selected by the captured `library/compiler-builtins/compiler-builtins/build.rs`.
  The captured Rust library lock enumerates 30 registry source archives with
  checksums; 15 already exist in prior verified evidence and 15 still need
  capture. This is a conservative library-lock source set, not target reachability.
- SDK 33 source pins above, its notices and the source relationship for every
  actually consumed C object. The official release offers
  `wasi-sysroot-33.0+m.tar.gz`, 124,253,494 bytes, SHA-256
  `063bc1b56582b9923e08ac9b89e58789618d851763f01530b3ff20b9e5df0ca3`;
  capture it inertly for the target-object byte comparison. The captured
  release metadata and source trees identify the exact expected input.

Completion requires a final identified WASM, its consumed-input/environment
record and completed source/notice companion, followed by the full existing
runtime/mapping corpus and Vue/Astro semantic fixtures against that artifact.
Check advisory coverage for any newly compiled or shipped versions. These are
remaining gates, not waived requirements. A 1.99.1 result would be a new artifact
identity and would not retroactively prove the published 1.96.1 build inputs.

The addendum evidence is the private
`astro-cjs-runtime-qualification-20261025/controlled-build-20261025/` directory
under the task-state root cited above: official channel/source captures,
`source-inputs.json`, `metadata-{originals,final}/`, `metadata-patches.json`,
`static-input-inspection.json`, `rustc-invocations.jsonl`, all three attempt
logs/results, `removed-partial-build-outputs.json`, `final-verification.json`
and the per-file evidence inventory. Only this document and private evidence
changed in this continuation; shared runtime/distribution and the 41-file
candidate were not edited.
