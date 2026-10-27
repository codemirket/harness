# M3 Rust semantic-navigation candidate qualification

Observed **2026-10-25**, macOS **26.5.2 (25F84), ARM64**. This is an independent,
bounded qualification of **rust-analyzer 2026-10-19**, for the Codex-only rewrite.
**Disposition: retain this exact pin as a semantic candidate; hold release
admission.** Same-file, cross-file, prepared-dependency and supplied-core
definitions, references and document symbols pass the bounded stdio fixtures
under network denial. The source lock review has two unresolved affected-version
findings, the official binary lacks a verified exact linked closure, and the
required private compiler closure is not admitted. Native Windows and installed
Codex operation remain unqualified. This task changed only this note and private
evidence; it did not change implementation, release or host configuration.
The original qualification verifier passes **27/27 checks**. The additional
admission experiment below retains actual protocol, process and cost records.

## Exact candidate and sources

The official [release](https://github.com/rust-lang/rust-analyzer/releases/tag/2026-10-19)
was downloaded as data; no upstream installer or helper was run. The API tag
resolves to source commit `5ec20d044dbf04895877a55921ddc0ae6e73e127`.
The binary reports `0.3.3089-standalone (5ec20d044d 2026-10-18)`.

| Item | Bytes | SHA-256 |
| --- | ---: | --- |
| `rust-analyzer-aarch64-apple-darwin.gz` | 14,071,674 | `eedad08a96f7bf92afd0890463787337d17591ea2e3df216033c6bc3ccc9d802` |
| Uncompressed analyzer | 38,205,216 | `62c40a961c93cb4bb60b2a95f2ebc8825ff37323705a4d499f02cbb8f701b04e` |
| Official tag source archive | 4,082,124 | `16bc0e82e7313f4b6ba7f4b74a6a50da4a95247e56351ee433a89750f8cf33cc` |
| [Rust source 1.99.1 archive](https://static.rust-lang.org/dist/rust-src-1.99.1.tar.xz) | 5,863,216 | `e485a88c45db546b73571df7bebfcb4454ba4fb552c492e25fc5b00eeb5bf22d` |

The analyzer archive digest matches the official GitHub asset `digest`; the Rust
source archive matches its official `.sha256` file. Mach-O inspection reports
native ARM64, macOS load-command floor 14.0, and these dynamic dependencies:
CoreFoundation, CoreServices, `libiconv.2.dylib`, and `libSystem.B.dylib`, all system
paths. Its signature is ad hoc, with no signing team. Neither that load-command
floor nor successful execution on macOS 26 qualifies macOS 15 or native trust
behavior on a fresh installation.

The analyzer [source licenses](https://github.com/rust-lang/rust-analyzer/tree/5ec20d044dbf04895877a55921ddc0ae6e73e127)
are MIT OR Apache-2.0. Evidence preserves both texts and their hashes. A
conservative traversal from the `rust-analyzer` root in its exact `Cargo.lock`
contains 281 packages, including 245 registry packages and 36 local packages.
All 245 archives were downloaded as data, checked against lockfile SHA-256
values, and inspected for manifests/notices without building anything. Their
compressed total is **37,404,542 bytes**. The inventory includes optional,
development and other-platform edges; it is deliberately **not an exact
macOS-linked binary SBOM**. Its license expressions include MIT/Apache choices,
Apache with LLVM exception, Unicode-3.0, BSD/ISC/Zlib, CC0, Unlicense and MPL-2.0.
Do not label the entire closure MIT. Retain the actual source/notices and resolve
linked-package provenance before redistribution. The Rust source archive also
contains its own copyright/license and vendored-source notices; it is a separate
input, not covered solely by rust-analyzer's top-level license.

### Advisory and binary-provenance disposition

The data-only review used RustSec snapshot
[`21d909e4c8bd4e14c7aea58bf980c881a0053ab1`](https://github.com/RustSec/advisory-db/tree/21d909e4c8bd4e14c7aea58bf980c881a0053ab1),
archive SHA-256
`e3ca3600f749e9234738584caad6a115278997f6c6528053faebc108465d8774`.
It matched **45 advisory records** against the 245 registry package names:
41 records were patched or outside affected ranges, one was withdrawn, one was
unmaintained, and two had affected locked versions. Every matched advisory body,
its hash, version ranges and per-version classification is retained.

| Exact source-lock input | Finding | Admission disposition |
| --- | --- | --- |
| `anyhow 1.0.102`, direct analyzer dependency | [RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html): unsound mutable error downcast after context; patched at 1.0.103 | No `downcast_mut` token appears in local analyzer crates. Registry production-source scan found the affected method definition, but no established caller of that anyhow API. This is textual triage, not proof of unreachability. |
| `crossbeam-epoch 0.9.18`, through `rayon → rayon-core → crossbeam-deque` | [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204.html): invalid pointer dereference in pointer formatting; patched at 0.9.20 | The package is on the runtime dependency path. No passive LSP input triggering the affected formatting path was established; reachability remains unresolved. |
| `paste 1.0.15`, through optional allocator instrumentation | [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html): unmaintained | Record maintenance risk and determine selected binary features before including it in the linked closure. This is not a demonstrated runtime vulnerability. |

The analyzer's [security page](https://github.com/rust-lang/rust-analyzer/security)
listed no published repository advisories when inspected. That does not clear its
dependencies. Neither affected-version match is presented as demonstrated native
binary exploitability. The pinned upstream
[`xtask/src/dist.rs`](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/xtask/src/dist.rs)
build command does not specify `--locked`; the tag's lockfile is therefore not an
attestation of the downloaded binary's selected dependency versions/features.

**Do not admit this binary from a top-level MIT/Apache label or a clean fixture
alone.** Admission needs an exact binary/source closure and a recorded disposition
of these findings, or a separately selected fixed pin/build with fresh hashes,
rights and behavior checks. This experiment stopped at that firm boundary and
did not silently upgrade or rebuild the candidate.

### Windows artifacts inspected as data

Both official Windows ZIP digests match the release metadata. Their PE machine
types match the named architectures; each ZIP contains the executable and PDB.

| Target | ZIP bytes / SHA-256 | Executable bytes / SHA-256 |
| --- | --- | --- |
| Windows ARM64, PE `0xaa64` | 15,884,787 / `dc2b6c35e2be0bf13f4ae2a6c69adca5730c91d011bfac31371279a4e55bcb2b` | 35,612,160 / `e1f35c6e1e19f05cc8a0946187692e6691d2d26ce09d102a831869315a6b5841` |
| Windows x64, PE `0x8664` | 17,750,891 / `7df6e1a2595d1906e8441cb83fd58170b47d32eb09bc52c1b2184538cfd49a71` | 39,097,856 / `6f76533ca854bc2708bfa1caab68240ed5b89e530eafc1ac9865f5a569f5ec1d` |

The import tables name `KERNEL32`, `api-ms-win-core-synch-l1-2-0`, `ntdll`,
`OLEAUT32`, `ADVAPI32`, `combase`, `SHELL32`, `PSAPI`, `USERENV` and
`bcryptprimitives`. No VC runtime DLL appears in these import tables, consistent
with the upstream distribution command's `+crt-static` flag. Import inspection
does not establish late-loaded dependencies, native execution, process-tree
cleanup, trust dialogs or the private Windows compiler closure. Both Windows
targets remain **unqualified**.

## Actual fixture and results

The hand-authored `rust-project.json` describes an application, a supplied local
source dependency, and a supplied sysroot graph containing real `core`, `alloc`
and `std` source roots from Rust 1.99.1. This graph establishes the tested `core`
path; it does not establish a complete standard-library/build-target model.
Application/dependency files use CRLF, Turkish identifiers/path components,
non-ASCII argument names and an emoji before a queried use. The client negotiated
**UTF-16**. Each returned range was independently converted against original UTF-8
bytes and checked for the exact declaration/reference text.

| Operation | Observed original result |
| --- | --- |
| Same-file definition after `"İ 😀"` | `yerel` use LSP `4:52–4:57` → declaration `3:7–3:12`; original UTF-8 use `[163,168)`, declaration `[62,67)` |
| Cross-file module definition | `yardımcı::çift` → `src/yardımcı.rs`, `0:7–0:11`, exact text `çift` |
| Supplied dependency definition | `aktar` → supplied `dep/lib.rs`, `1:7–1:12` |
| Supplied sysroot definition | `core::option::Option` → official `core/src/option.rs`, `597:9–597:15` |
| References, declaration included | Exactly three `yerel` ranges: app declaration `[62,67)`, app use `[163,168)`, module use `[41,46)` |
| Missing dependency edge/source metadata | Dependency target empty; local/module targets still resolve; no package preparation |
| Missing sysroot source/model | `Option` target empty; local/module/dependency targets still resolve; no sysroot provisioning |
| Explicit missing project file | All five semantic requests return empty results; no compiler/Cargo/project command |
| Target source changed during an operation | Raw result follows the edited line; pre/post source hashes disagree, so the fixture's host contract reports `stale` |
| Saved inline model changed | Analyzer can still return the old dependency target; changed model hash requires refusal and session replacement |

An empty LSP result does not distinguish absent symbols from incomplete project
metadata. Mirket must perform that classification from verified inputs. The
revision checks above are exercised by the qualification client, **not yet by a
production Rust navigation owner**. A separately tested
[common finite LSP client](m3-lsp-client.md) now exists under `cfg(test)` only;
neither that dormant client nor this direct analyzer fixture establishes
production Rust support.

The first sysroot attempt with `cargo.autoreload=false` loaded only the already
opened application/dependency graph. Library roots arrived through VFS later;
the queued graph refresh remained disabled and `Option` returned empty. A fixed
inline JSON project with workspace refresh enabled completed the supplied graph
and resolved the target without Cargo. The final run retained this configuration
and waited for a quiescent server. Do not promote `initialize` or a quiescent
status alone to a graph-completeness pass.

## Execution boundary and observed traps

The upstream [security statement](https://rust-analyzer.github.io/book/security.html)
assumes trusted code and lists several execution risks. Inspection used the
pinned source, especially `project-model/src/workspace.rs`, `sysroot.rs`,
`toolchain_info/*`, `rust-analyzer/src/config.rs`, `reload.rs`, `flycheck.rs`,
`handlers/{notification,request}.rs`, `discover.rs`, and `toolchain/src/lib.rs`.
The relevant routes are broader than build-script and proc-macro switches:

| Route | Qualified boundary / required integration control |
| --- | --- |
| Cargo discovery/metadata, Cargo scripts, build scripts | Fixed supplied JSON graph only; no Cargo/detached-file discovery and no automatic preparation. Both build scripts and proc macros disabled. JSON does not itself execute `build.rs`. |
| Compiler/toolchain/sysroot queries | `load_inline` still probes compiler version, cfg and target layout. A supplied graph does **not** mean zero child execution. Supply a verified private sysroot with its exact trusted `bin/rustc`; never let a project supply that executable or fall through to PATH/rustup proxies. |
| Sysroot Cargo metadata | Supply `sysroot_project` together with `sysroot_src`. Supplying only source can take the Cargo-metadata route. |
| Per-crate target queries | Explicit `target` fields can add compiler probes. Validate target/cfg metadata and bound total probes. |
| Proc-macro server/dylibs | Global proc-macro enable remains false. The adversarial graph named an invalid dylib and sentinel server; neither ran/loaded in the passive run. Generated macro-dependent semantics remain incomplete. |
| Workspace discovery command | Fixed `workspace.discoverConfig=null`; never accept a project-selected discovery executable. |
| Flycheck/check/runnables/test runner | Strip JSON `runnables`/build execution metadata; disable checking. Do not forward save/check/test/run commands to a navigation-only session. |
| Formatter and configurable command overrides | Reject formatting and command-bearing extension requests at the finite client boundary. A separate formatting control **did execute the harmless project-command sentinel** from workspace configuration, proving that these routes are live. |
| Project/user analyzer configuration | Project `rust-analyzer.toml` can override workspace settings before client settings. `files.exclude` names do **not** suppress such a file in the initial directory walk: the pinned walker filters excluded directories. Do not rely on that setting as a configuration policy. Use verified configuration-free input snapshots or an equally demonstrated fixed-policy boundary. |
| `.cargo/config.toml`, `rust-toolchain.toml`, editor plugin settings | Adversarial files were present. The selected direct compiler path and fixed inline graph bypassed Cargo/rustup; the VS Code extension was not loaded. No editor plugin is part of this candidate. |
| Environment and executable fallback | Fresh private HOME/CARGO_HOME/RUSTUP_HOME/config/temp roots; controlled env; explicit analyzer and sysroot paths. Never inherit wrappers, command overrides or user Cargo proxies. |

All navigation runs used a macOS `sandbox-exec` test profile denying network.
An independent socket-connect control failed with `EPERM`. This establishes the
test's network denial, not a production Mirket sandbox or evidence of zero network
attempts. Process sampling and named sentinels are observations, not a complete
kernel execution trace or protection against memory-safety flaws.

The final `passive-real-compiler` run forwarded only an allowlisted exact set of
no-input compiler query arguments to explicitly selected Rust **1.99.1** at
`/Users/nazmiyilmaz/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin/rustc`
(SHA-256 `1ba7bb8e273bd64125f4048d2ecce530e0a6470e565ab1b3aecf0b82b532f7b1`).
It logged **nine probes**, three graph-loading passes of:

```text
--version
--print cfg -O --target aarch64-apple-darwin
-Z unstable-options --print target-spec-json --target aarch64-apple-darwin
```

There were **zero Cargo, rustup, formatter, build-script, proc-macro-server
or project-command sentinel invocations in that final passive run**.
The analyzer exited 0. The instrumentation is task-owned qualification code and
is not proposed as a shipped compiler substitute. The compiler was an explicitly
selected existing development prerequisite; this does not admit its private
release closure. Production must qualify a real approved compiler/probe input
path rather than silently ship the replay sentinel or use a user's global tool.

## Additional refusal, recovery and cost experiment

`admission-extension-20261025/probe.py` uses the same analyzer digest and real
compiler. It invokes the compiler directly through the explicitly supplied
sysroot; the earlier allowlisted compiler wrapper remains the exact probe-argument
evidence. The new runs use an empty PATH, fixed environment, private state roots,
no project configuration, the network-denial profile and the same finite semantic
methods. No Cargo compilation, package installation or project command is used.
The only analyzer stderr was a warning that its private user configuration file
did not exist. That path was kept private and empty.

Two exact prepared corpora are retained with original-byte manifests. The small
corpus has three Rust source files, **404 bytes**, including the supplied source
dependency. The larger synthetic corpus has 53 files, **268,754 bytes**, with
5,000 additional simple function declarations and 50 additional cross-file calls.
Every run resolved the four same/module/dependency/core targets to their exact
original text; references numbered exactly three and 53 respectively. Document
symbols returned four and 54 top-level entries; declaration selections for
`yerel`, `çağır` and `maybe` matched original bytes. The module entries explain the
larger symbol count. This is a generated bounded corpus, not a representative
large workspace or a complete Rust language corpus.

A task-owned preflight experiment rejected **18 cases before launch**, recording
zero launches for every refusal: root/nested analyzer configuration, Cargo
configuration, toolchain files, formatter/editor configuration, `runnables`,
plugins, proc-macro flags/dylib metadata, build metadata, sysroot discovery,
outside source roots, a source symlink, stale source and a stale model. Accepted
inputs were rehashed after each run. This Python experiment demonstrates the
selected refusal cases; it is **not production preflight code** or a complete
filesystem race defense. Installed integration must enforce canonical scoped
paths, no-follow opens, complete source/model manifests and immutable policy
inputs using the owning Rust scope/revision mechanisms.

After readiness, a held analyzer was deliberately killed with `SIGKILL` and
reaped. A fresh process over the unchanged model then repeated all four
definitions, three references and declaration-symbol checks successfully. An
immediately cancelled reference request returned LSP **`-32800`**, and the host
outcome remained `cancelled`. All seven sessions, including normal shutdown,
forced stop, restart and cancellation, had an empty observed process group after
reaping. These are macOS qualification-client observations; the separate common
client tests own malformed/hostile-server transport behavior, and native Windows
ownership remains open.

The timing experiment used fresh processes, three small runs and one larger
run, with OS/filesystem caches **not purged**. Timing includes protocol handling
and a concurrent `ps` observer. Readiness is launch-to-quiescent status, not proof
that all semantic work is finished. First method timings below are sequential
requests after that status. Each run then repeated one definition, references
and symbols ten times.

| Corpus/run | Ready ms | First definition ms | First references ms | Sampled analyzer peak MiB |
| --- | ---: | ---: | ---: | ---: |
| 3 files / 1 | 651.1 | 685.4 | 270.6 | 341.5 |
| 3 files / 2 | 342.5 | 671.1 | 261.5 | 339.8 |
| 3 files / 3 | 330.0 | 670.7 | 265.9 | 349.9 |
| 53 files / 1 | 368.4 | 695.2 | 274.3 | 353.4 |

Within the small runs, repeated-request medians were **0.092–0.105 ms** for the
already-loaded dependency definition, **0.124–0.129 ms** for references and
**0.121–0.140 ms** for symbols. The larger run medians were **0.103 ms**, **1.299
ms** and **0.330 ms**, respectively. These tiny warm requests do not establish
end-to-end Mirket latency: a fresh-process definition in this fixture took about
**1.0–1.34 seconds** from launch. A fresh process per individual request therefore
needs a separate product latency decision; a persistent bounded owner needs its
own invalidation/idle-release measurements.

RSS was sampled approximately every 20 ms, plus the duration of each `ps` query.
It is a lower bound on peak memory, not a kernel maximum or a resource cap.
Transient compiler children were observed in some samples. The old sentinel
trace supplies the exact permitted child-query vectors; sampling cannot prove
the absence of short-lived children. These results support further bounded
integration work, but do not establish the two-analyzer budget on large projects,
long-session stability or Windows performance.

## Minimal integration contract and remaining gates

Use the existing connection-owned admission/process lifecycle and a finite Rust
navigation adapter. Bind analyzer/version/hash, trusted compiler/sysroot identity,
prepared project-model revision and all participating source revisions. Admit a
verified inline graph with explicit source/dependency scopes and known cfg/env;
remove executable fields and prevent project/user analyzer configuration from
changing policy. Permit only needed read requests plus validated document state
updates; never expose a transparent LSP command tunnel. Reject locations outside
project and explicitly approved dependency/sysroot roots, malformed positions,
surrogate splits and stale sources/model inputs. Convert against exact original
bytes and recheck revisions before returning results. On missing metadata,
loading, disabled macro output or truncation, return explicit incomplete coverage.
Replace the session when its model changes; inline model edits are not automatic
invalidation.

The analyzer path for reproduction is exactly
`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/rust-navigation-qualification-20261025/candidate/bin/rust-analyzer`,
with the uncompressed digest above. It is private experiment state, not a user
installation or stable installed-release path. A production preflight must bind
the analyzer, compiler and complete sysroot payload identities; use only a
host-produced static graph with validated dependency indices, roots, cfg/target
and fixed empty/allowlisted crate environment; refuse unknown/executable fields
and conflicting configuration before starting; and retain exact source/model
revisions for the whole operation. Missing dependency/sysroot/generated-macro
metadata must produce `incomplete`/`unsupported`, not a falsely complete empty
result. No user-selected argv, environment or implicit PATH selection is allowed.

The common client's byte/URI/revision/cancellation/output-budget and
hostile-server tests remain test-only. Installed integration must exercise those
properties against the admitted real analyzer and preserve the two-live-analyzer
limit and existing queue/deadline/idle-release behavior. This qualification does
not validate all Rust features, execute a real proc macro/build script, or
establish complete reference coverage in arbitrary projects. Native Windows
x64/ARM64 execution, oldest-macOS behavior, production compiler/static license
closure, installed Mirket MCP, Codex Desktop GUI/model activation and
packaging/trust are **unqualified**. No simpler replacement backend was qualified;
a syntax-only parser would not satisfy these semantic operations. Keep Rust
navigation unexposed until admission and integration are completed.

## Retained evidence

Task-owned evidence is under:
`~/.codex/task-state/harness-rewrite-implementation/rust-navigation-qualification-20261025/`.
The candidate has an explicit private `candidate/bin/rust-analyzer` path there.
Key records are `release.json`, `tag-ref.json`, `identity.json`, source archives,
`conservative-closure.json`, checked `crate-archives/`, extracted `crate-rights/`,
`metadata/responses.json`, `passive-real-compiler.{inputs,result.json,protocol.json,
process-samples.json}`, the three `missing-*.{inputs,result.json,protocol.json}`
cases, `execution-route-controls.*`, `network-denial-control.json`,
`verification.json`, and the qualification scripts. Earlier failed/synthetic
runs remain labeled so they cannot replace the final real-compiler result.

The added `admission-extension-20261025/` directory contains `probe.py`,
`probe-results.json`, `input-manifests.json`, per-session `*.protocol.json`,
`*.process.json` and stderr, `measurement-summary.json`, both prepared corpora,
the exact RustSec snapshot/archive and advisory bodies, `advisory-matches.json`,
`advisory-disposition.json`, `affected-use-scan.json`, `summarize-review.py`,
both checked Windows archives and `windows-inspection.json`. The small fixture
model is at `admission-extension-20261025/small prepared İ/rust-project.json`.
No Cargo command was needed for this documentation/data/protocol extension.
