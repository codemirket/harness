# M3 Rust navigation admission decision

Observed **2026-10-25**, macOS **26.5.2 (25F84), ARM64**. **Hold the official
rust-analyzer 2026-10-19 binary.** The 21 missing license-body captures are
resolved for the conservative source inventory. Inert inspection of the actual
analyzer supplies stronger evidence that the affected `anyhow 1.0.102` and
`crossbeam-epoch 0.9.18` inputs are present. It does not establish vulnerable
function reachability or a complete binary closure. One bounded corrected-build
route is specified below; it has not been executed or admitted.

This supplements the [qualification](m3-rust-navigation-qualification.md) and
[independent recheck](m3-rust-navigation-qualification-recheck.md). Only this note
and private evidence changed. No dependency, source implementation, distribution,
installation, toolchain, client configuration, commit or publication changed. The
scope is the Codex-only rewrite and its approved M3 navigation boundary.

## Clearance matrix

| Boundary | New evidence | Decision |
| --- | --- | --- |
| `anyhow 1.0.102` | The exact analyzer contains `anyhow-1.0.102/src/error.rs` and `src/fmt.rs` paths and anyhow native symbols. [RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html) applies to this source version. The checksum-verified 1.0.103 archive changes the context-downcast reference construction. | **Blocked for this artifact.** No navigation trigger or affected-function reachability is claimed. Use the corrected version in the proposed build rather than promote an absence of textual calls to proof of safety. |
| `crossbeam-epoch 0.9.18` | The exact analyzer contains its `src/atomic.rs`, `internal.rs` and synchronization source paths, plus crossbeam native symbols. Its exact crate source dereferences pointers in `fmt::Pointer` for both `Atomic` and `Shared`. The 0.9.20 archive formats raw addresses instead. | **Blocked for this artifact.** This resolves the advisory's title/body trait inconsistency in favor of the actual `fmt::Pointer` implementation and [upstream correction](https://github.com/crossbeam-rs/crossbeam/pull/1276). Exploitable navigation reachability remains unproven. |
| Optional `paste 1.0.15` | Its sole incoming registry edge in this conservative closure is `tikv-jemalloc-ctl`; the local enabling path is `rust-analyzer/jemalloc → profile/jemalloc`. The pinned macOS workflow selects the system allocator, and `Malloc::System` supplies no allocator feature. `paste` is a procedural macro build input, not a runtime dylib. | **Source-selection disposition available:** excluded by the documented macOS system-allocator recipe. Preserve the [unmaintained notice](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) if a future build selects it. The workflow is evidence of intent, not exact compiled-feature provenance. |
| 21 archives without captured notice files | All 21 now map to actual license bodies, immutable source pins and retained manifest/source-notice evidence; 18 distinct body files supply 40 package/body associations. | **Source-inventory gap cleared.** This does not identify which complete source/license set corresponds to the downloaded binary. |
| Exact analyzer closure | The conservative root traversal has 281 packages / 245 registry packages. Native embedded paths positively match 85 of those registry name/version pairs. Absence of a path or symbol cannot exclude a package after optimization. The release assets contain no SBOM/link manifest; a bounded GitHub attestation lookup for the exact compressed digest returned HTTP 404. | **Still blocked.** Neither a Cargo.lock traversal nor the 85 positive matches is an exact binary SBOM. The failed lookup does not prove that no provenance exists elsewhere. |
| Private compiler / supplied sysroot | Official Rust 1.99.1 compiler component and channel manifest checksums match. Three ARM64 files form the observed non-system dynamic-link graph: rustc, its driver and LLVM. Complete component copyright/license files are retained. | **Statically feasible; not admitted.** No private compiler was executed, relocated, or qualified in this review. Its source inventory also needs a target-selected advisory disposition. |

The analyzer identity remains uncompressed SHA-256
`62c40a961c93cb4bb60b2a95f2ebc8825ff37323705a4d499f02cbb8f701b04e`,
source commit `5ec20d044dbf04895877a55921ddc0ae6e73e127`. Native paths and symbols
are positive binary evidence of particular inputs, not a call graph or proof
that every crate function survives linking.

The pinned [release workflow](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/.github/workflows/release.yaml)
uses macOS system allocation, PGO training from a separate pinned clap checkout,
and a moving stable Rust toolchain. Its
[build command](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/xtask/src/dist.rs#L140-L150)
does not enforce `--locked`. Cargo normally reuses a valid existing lockfile;
the missing flag does **not** itself demonstrate changed dependency versions.
The unresolved fact is the absence of a receipt binding the release's actual
resolved graph, build inputs/features and output bytes. Build-only, PGO-training,
runtime-linked and shipped-file closures must remain distinct.

## Recovered source rights

`missing-rights-resolution.json` retains each package's original declaration,
archive identity, source pin, provenance method and license-file hashes. The
already verified crate archives remain the source of original bytes. Any source
file containing a copyright or SPDX notice is retained alongside its manifest.

| Previously omitted package(s) | Immutable origin / recovery |
| --- | --- |
| `la-arena 0.3.1` | [Release commit `a0467c5`](https://github.com/rust-lang/rust-analyzer/tree/a0467c546c1e412cd1b35257f8d5ae08f79072d6). Both source files and `Cargo.toml.orig` match the crate byte for byte; retain root MIT and Apache bodies. |
| `line-index 0.1.2` | Crate VCS pin [`3bda0cc`](https://github.com/rust-lang/rust-analyzer/tree/3bda0cc2c07f6330b543be2c12c5e156847ba94e); root MIT and Apache bodies. |
| Eleven `ra-ap-rustc_* 0.166.0` crates | Every published manifest description names [`ce89c898`](https://github.com/rust-lang/rust/tree/ce89c898570852a1bb441d77570596e50bf362c2). Retain root MIT/Apache bodies and the pinned `license-metadata.json`; its Rust-root dual-license scope covers these crates. The explicit `rustc_llvm` wrapper exception is outside this eleven-crate set. Keep the published legacy `MIT / Apache-2.0` spelling in provenance. |
| `perf-event 0.4.8`, `perf-event-open-sys 4.0.0` | VCS pins [`fdfcd6e`](https://github.com/jimblandy/perf-event/tree/fdfcd6eab9de3e40b2ecbcc01a1be3b1bd15405b) and [`3aecf09`](https://github.com/jimblandy/perf-event/tree/3aecf09156daa67b5579cc7742168cc95dcbe063); root MIT/Apache bodies. The latter historical repository URL returns 404; the exact commit is available in the current monorepo. |
| `pulldown-cmark-to-cmark 10.0.4` | VCS pin [`5f6ec45`](https://github.com/Byron/pulldown-cmark-to-cmark/tree/5f6ec453d5c44d45d09100749d1c1e2891e05bf7); exact upstream manifest declares Apache-2.0. The package and repository root contain no separate license body. Supplement the declaration/source with the full [canonical Apache 2.0 text](https://www.apache.org/licenses/LICENSE-2.0.txt), recording its hash and origin explicitly. This text is not represented as an upstream-packaged file. |
| `tenthash 1.1.0` | VCS pin [`d420ce8`](https://github.com/cessen/tenthash/tree/d420ce80cfbf9a9cbbc41a55efb620ca9ed72efa); actual `licenses/MIT.txt` and `APACHE-2.0.txt`. |
| `valuable 0.1.1` | VCS pin [`9efc29b`](https://github.com/tokio-rs/valuable/tree/9efc29b6e58cef28f6566a47aa7e142a55fead77); actual root MIT `LICENSE`. |
| `xflags 0.3.2`, `xflags-macros 0.3.2` | VCS pin [`bc37dc0`](https://github.com/matklad/xflags/tree/bc37dc0b02dd3853aff7cafff1c4f0dd4293af45); root MIT/Apache bodies. |
| `xshell-macros 0.2.7` | VCS pin [`9a40594`](https://github.com/matklad/xshell/tree/9a405946ecbbfe50fe4a7229bd764194154e4ce6); root MIT/Apache bodies. |

This closes a capture omission, not a general legal opinion or permission to
relabel every dependency MIT. Preserve the existing full conservative notices,
the recovered texts, source identity and the actual per-package license choices
when assembling the corrected artifact. Compiler, LLVM, Rust source, runtime and
any copied standard-library inputs require their own inventory.

## Private compiler feasibility

The official [Rust 1.99.1 channel manifest](https://static.rust-lang.org/dist/channel-rust-1.99.1.toml)
names the dated component
[`rustc-1.99.1-aarch64-apple-darwin.tar.xz`](https://static.rust-lang.org/dist/2026-10-15/rustc-1.99.1-aarch64-apple-darwin.tar.xz).
Its **50,728,492 bytes** hash to
`23948a0a9c162fd2952defb3fb0462c44e8fcfd1a4ab8a274a1e1bdb2cf810a3`,
matching the manifest. The component contains **279,896,323 regular-file bytes**,
including full copyright inventories, license bodies, debugging/linking tools,
rustdoc and the proc-macro server. Those extra executables are not required merely
because they share the component.

| Proposed no-input compiler-query native file | Bytes | SHA-256 |
| --- | ---: | --- |
| `bin/rustc` | 413,480 | `1ba7bb8e273bd64125f4048d2ecce530e0a6470e565ab1b3aecf0b82b532f7b1` |
| `lib/librustc_driver-451a19fa7a19fc36.dylib` | 83,390,408 | `ebf983c7473eac96e90a4201ca969801ac53a3cf4abc8f27b0021d53fb148395` |
| `lib/libLLVM.dylib` | 143,609,472 | `2a8a4612d921ac0bd6be3b7b601f9a71d93b8cdf0a23f36d4c2df15cb616c981` |

These total **227,413,360 bytes**, before rights files and source/model inputs.
All are ARM64. `otool` reports only the mutually supplied `@rpath` libraries and
system library/framework paths; `LC_RPATH` is `@loader_path/../lib`. The observed
load-command floor is macOS 11.0, which does not qualify execution or native trust
on any supported OS floor. This is a static candidate set, not a measured working
minimal distribution. The existing development compiler is byte-identical to the
official component's rustc, but no user's toolchain is selected for production.

The official compiler `COPYRIGHT.html` conservatively lists old and patched
anyhow/crossbeam versions plus paste. It even marks the older versions as
`In libstd: Yes`; that label is source inventory metadata, not a link map.
The actual driver embeds **crossbeam-epoch 0.9.20** paths. No anyhow or paste
version path was found in these three native files; that absence cannot establish
exclusion. Do not transfer the analyzer's confirmed older versions to the
compiler or declare the separate compiler advisory review clean from this scan.
Retain the full official `COPYRIGHT.html`, `COPYRIGHT-library.html`, root rights
files and all bundled license texts, including LLVM's distinct terms.

The supplied Rust 1.99.1 source archive remains separately pinned at SHA-256
`e485a88c45db546b73571df7bebfcb4454ba4fb552c492e25fc5b00eeb5bf22d`.
The previously tested hand-authored core/alloc/std graph is only a bounded model.
It does not prove a complete standard-library graph. A prepared graph must bind
the selected target, cfg, source roots, transitive edges and revisions to this
source/runtime pair and explicitly classify unavailable macro/generated semantics.

## One bounded corrected-build route

Keep the selected analyzer source commit; create one explicit downstream build
with only these two locked dependency corrections, the system allocator, and no
PGO-training checkout. Do not run `xtask`, a release installer, project build,
package helper or an evolving upstream release workflow as incidental review.

| Corrected crate | Verified crates.io archive SHA-256 |
| --- | --- |
| `anyhow 1.0.103` | `2a4385e2e34eb35d6b3efe798b9eb88096925d87726c0798709bf56d9ed84af3` |
| `crossbeam-epoch 0.9.20` | `2d6914041f254d6e9176c01941b21115dcfb7089e55135a35411081bd106ef3f` |

The fixed archives and their notices are retained as data. They are not added to
the rewrite's dependencies or admitted by this proposal. The next bounded build
must:

1. Freeze the source archive, exact two-entry lock diff, selected Cargo/Rust
   toolchain, target/SDK/linker, feature selection and offline crate directory.
   Resolve the selected normal/build dependency graphs separately from dev,
   other-platform, optional and PGO inputs. Review every selected build script
   before execution. In particular, upstream analyzer `build.rs` invokes `git
   log`; the proc-macro-server build script invokes the selected compiler. Bind
   those commands and their environment explicitly rather than inherit PATH.
2. Build directly with frozen/offline Cargo inputs and the system allocator;
   retain the actual compiler invocations, selected features, source checksums,
   dependency/link evidence, output digest and per-file/package notices. The
   corrected binary's provenance must cover Rust standard-library/runtime code
   incorporated by its exact toolchain too. Do not present an offline lockfile
   alone as the complete linked closure.
3. Assemble the real private compiler-query candidate and its rights files by
   explicit file copy from verified official components, never an upstream
   installation script. Qualify only the three already observed fixed no-input
   query forms with a controlled environment, fixed built-in target, empty
   fallback PATH, no source input, bounded output/time and the supplied sysroot
   model. Record actual opens/children sufficiently to validate this candidate
   file set, relocation, missing-library refusal and no Cargo/rustup fallback.
   The target-layout query uses upstream's explicit `RUSTC_BOOTSTRAP=1`; it is
   not permission to accept arbitrary `-Z`, target JSON, project arguments or
   compiler execution from callers. Disposition the separate compiler advisory
   inventory against its actual selected inputs/query surface.
4. Re-run the existing exact Unicode/CRLF, local/module/dependency/core,
   stale-source/model, missing-input, configuration poisoning, cancellation,
   crash/recovery and no-project-execution controls against the new bytes. Bind
   all immutable source/model/config inputs and both executable closures to the
   backend policy. Native Windows targets, installed MCP/Codex operation and OS
   floors retain their separate gates.

Stop if that single corrected-artifact attempt cannot establish its selected
closure or passive boundary; report the concrete failed gate. This review did
not start that build, silently select a newer analyzer, or attempt alternatives.
The smallest next implementation step is this one private macOS build/admission
receipt. Production Rust backend integration follows its passing receipt and the
fixed-policy snapshot/source-model contract from the recheck.

## Evidence and limits

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/rust-navigation-admission-20261025/`.
`retrievals*.json` record URLs, retrieval time, hashes and the two bounded 404
observations. `missing-rights-resolution.json`, `missing-rights-provenance/`,
`analyzer-embedded-source-paths.json`, `analyzer-symbols-selected.txt`, corrected
crate diffs, `private-compiler-feasibility.json`, native load-command captures and
`compiler-rights/` preserve the substantive findings. `verification.json` records
**43 passing identity/content checks**; `evidence-manifest.json` binds saved
bytes. No upstream program, installer, hook or compiler was executed during this
review; native `file`, `strings`, `nm` and `otool` inspected data.

The new evidence clears the notice-capture omission and the crossbeam trait
ambiguity. It does not clear the official analyzer, establish vulnerable-input
reachability, exact linker closure, a working private compiler distribution,
complete Rust semantics, installed Rust navigation, Windows behavior or Codex
activation. Whole-crate compilation gates were not rerun for this docs-only
change.
