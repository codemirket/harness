# M3 Rust navigation qualification recheck

Observed **2026-10-25**, **macOS 26.5.2 (25F84), ARM64**. The pinned
**rust-analyzer 2026-10-19** remains a useful candidate for prepared, offline
definitions and references. Independent repetitions passed **44 evidence checks**,
including same-file, cross-file, supplied-dependency and supplied-core targets,
exact original Unicode/CRLF locations, missing-input cases and stale analysis.
**It is not ready for production admission or an installed Rust-navigation claim.**
The unresolved gates are the compiler/probe and source-model boundary, exact
redistribution closure and advisory dispositions, and the owned Mirket backend.

This recheck changes only this note and external disposable evidence. No
production source, lockfile, inventory, installation, client setting, dependency,
commit or publication was changed. The scope is Codex-only. It supplements the
[original experiment](m3-rust-navigation-qualification.md) and the
[test-only LSP transport](m3-lsp-client.md), not their activation status.

## Identity and method

The reused native binary is byte-identical to the original experiment:

| Input | Verified identity |
| --- | --- |
| Official [release](https://github.com/rust-lang/rust-analyzer/releases/tag/2026-10-19) | Source commit `5ec20d044dbf04895877a55921ddc0ae6e73e127`; reports `0.3.3089-standalone (5ec20d044d 2026-10-18)` |
| Analyzer, 38,205,216 bytes | SHA-256 `62c40a961c93cb4bb60b2a95f2ebc8825ff37323705a4d499f02cbb8f701b04e` |
| Compressed analyzer | SHA-256 `eedad08a96f7bf92afd0890463787337d17591ea2e3df216033c6bc3ccc9d802` |
| Tag source archive | SHA-256 `16bc0e82e7313f4b6ba7f4b74a6a50da4a95247e56351ee433a89750f8cf33cc` |
| Explicit existing development compiler | Rust 1.99.1, `/Users/nazmiyilmaz/.rustup/toolchains/1.99.1-aarch64-apple-darwin/bin/rustc`; SHA-256 `1ba7bb8e273bd64125f4048d2ecce530e0a6470e565ab1b3aecf0b82b532f7b1` |

The copied client ran against a new disposable fixture, reusing candidate,
captured query metadata and supplied Rust 1.99.1 source through read-only links.
Every selected source file inspected for this review was compared to its bytes
inside the pinned archive: eleven files, including configuration precedence,
inline loading, VFS walking, release features, licenses and Cargo.lock.
All 245 conservative registry archives and 403 previously captured rights files
were rehashed successfully. No upstream build, installer or hook was executed.

All analyzer repetitions used an explicit executable, private home/config/cache
paths and a macOS `sandbox-exec` profile denying network. A fresh socket-connect
control failed with `EPERM`. This qualifies the test's network denial; the
production process owner is not a sandbox. Named execution sentinels and process
observations do not establish a complete kernel-level execution trace.

## Repeated behavior

| Case | Observed result and implication |
| --- | --- |
| Same-file definition after Turkish text and emoji | UTF-16 origin `4:52–4:57` maps to original UTF-8 `[163,168)`; declaration `3:7–3:12` maps to `[62,67)`, exact `yerel`. |
| Cross-file definition | `yardımcı::çift` resolves to original `src/yardımcı.rs`, bytes `[7,12)`, exact `çift`. |
| Prepared dependency | `aktar` resolves to supplied `dep/lib.rs`, bytes `[19,24)`. |
| Supplied core | `Option` resolves to supplied official `core/src/option.rs`, UTF-16 `597:9–597:15`, bytes `[24100,24106)`. The hand-authored core/alloc/std graph does not establish a complete standard-library or target model. |
| References | Exactly the declared `yerel` plus its app/module uses: app `[62,67)`, `[163,168)` and module `[41,46)`. |
| Missing dependency / sysroot / model | The corresponding target is empty; missing model yields five empty results and no sentinel invocation. Other declared targets survive where applicable. No dependency, sysroot or project preparation occurs. An empty result cannot classify metadata completeness. |
| Proc-macro metadata present, globally disabled | A graph naming an invalid proc-macro dylib still resolves the ordinary targets; only trusted compiler-query sentinels ran. No proc-macro server sentinel ran. This is not complete proof of all file opens or macro-dependent semantics. |
| Notified source edit | Target moves after `didChange`; participating source hashes differ and the host contract refuses `stale`. |
| Unnotified saved source edit | Adding a CRLF before the target without a notification leaves the raw analyzer result at old line 0. That position is invalid against the new empty first line. Freshness must be checked against original bytes outside the analyzer. |
| Inline project-model edit | Removing the dependency edge on disk leaves the old raw dependency target available. A model revision change must refuse the operation and replace the session. |
| Shutdown | All six navigation/source-change runs completed standard shutdown/exit and reaped the direct analyzer with exit 0. Actual candidate cancellation, abrupt owner death and descendant escape were not requalified here. |

The final positive run again made **nine compiler probes**, three graph-loading
passes of the same three no-input queries:

```text
--version
--print cfg -O --target aarch64-apple-darwin
-Z unstable-options --print target-spec-json --target aarch64-apple-darwin
```

The qualification wrapper forwards only the exact captured query arguments to
the selected real compiler. There were zero Cargo, rustup, formatter,
build-script, proc-macro-server or project-command sentinel invocations in that
run. The wrapper is instrumentation, not a proposed production compiler or a
portable execution policy. The missing-input and disabled-proc-macro repetitions
also use the real selected compiler for permitted probes.

## Trust boundary that the owner must implement

Upstream's [security statement](https://rust-analyzer.github.io/book/security.html)
assumes trusted projects. Disabling build scripts and proc macros is necessary
but insufficient for a passive owner. The pinned
[inline loader](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/project-model/src/workspace.rs#L477)
still queries the compiler for version, target cfg and layout. Its fallback
loads the sysroot through Cargo metadata unless a `sysroot_project` is supplied.
The documented [JSON model](https://rust-analyzer.github.io/book/non_cargo_based_projects.html)
also permits runnable commands, target-specific queries and proc-macro paths.

Project `rust-analyzer.toml` workspace settings precede the client's settings in
the pinned configuration getter. The initial VFS directory walk follows links
and filters excluded directories, so putting a config filename in `files.exclude`
does not suppress it. The repeated positive control sent a formatting request
to a deliberately poisoned disposable workspace and **executed the harmless
project-command sentinel**. The formatting response was `null`: a result without
edits does not mean no command ran. Navigation must never expose that request.

The smallest honest adapter contract is:

1. Bind one admitted private analyzer and its immutable release inputs by
   version/hash. Admit a real private compiler/probe runtime and sysroot closure;
   this experiment's existing global compiler path is development evidence only.
   Select fixed target/cfg values, controlled environment and explicit paths.
   Bound repeated probe loading. No PATH, rustup, wrapper or project-command
   fallback may supply executable identity.
2. Accept only a validated, prepared inline graph with explicit transitive
   dependencies, source roots, cfg/env and matching sysroot source/model.
   Preserve graph provenance and revision. Remove runnable/build execution
   metadata and reject unqualified target, discovery, plugin and command fields.
   Missing metadata is an explicit incomplete result; a navigation request does
   not authorize Cargo metadata, downloads, builds, generators or macro execution.
3. Enforce configuration isolation against the analyzer's actual reads. A
   private HOME and client settings do not neutralize workspace TOML. Use an
   owned immutable, configuration-free source snapshot with exact original-file
   mappings, or demonstrate an equally effective fixed-policy implementation.
   Copying a root without validating links and every source/include root is
   insufficient. Freeze all analyzed inputs, not just files returned as targets.
4. Expose finite definition/reference operations through the common owner, with
   only the validated document-state messages required by that policy. No raw
   LSP method, execute-command, formatter, check, runnable, custom discovery or
   configuration tunnel. Server-initiated requests cannot authorize host effects.
   Startup, document updates, ready state, request, cancellation and shutdown
   share one deadline and bounded transport/output budgets.
5. Convert negotiated positions against exact original UTF-8 bytes. Validate
   origin, target and selection ranges, UTF-16 surrogate boundaries, CRLF,
   canonical scope and URI identity. Map snapshot locations only through a
   proven original-source mapping. Recheck all participating source/model/input
   revisions before returning results; discard stale results and invalidate the
   session on model, branch, release or configuration changes. Mutable
   before/after hashes are not an atomic snapshot or a cure for an old AST.
6. Report backend/release, source/model revisions and declared-corpus coverage.
   Distinguish missing metadata, disabled generated output, unsupported operation,
   loading, stale, cancelled, deadline and truncation from a complete empty
   result. Quiescence and initialization alone do not establish completeness.
   Preserve the existing connection ownership, two-live-analyzer limit, atomic
   heavy-job admission, queue/deadline/idle release and confirmed cleanup before
   releasing a slot. No daemon or new analyzer framework is needed.

`src/lsp_client.rs` remains `cfg(test)` only, with unchanged source hash
`3f720c8a931aa371533e0a351d676feb70e154cd47c5e3120c111e67774b5545` at inspection.
Its finite methods, hostile-server checks, revision conversions and cancellation
tests are useful transport groundwork. Existing fake-server tests do not prove
this backend policy, complete analyzer inputs, production admission, installed
MCP behavior or Codex activation. This recheck did not rerun the whole-crate
build/test gates because it changes no compiled source.

## Rights and advisory admission gates

The conservative source closure has 281 packages, including 245 registry packages.
Its locked optional, development and other-platform edges are not an exact
macOS-linked SBOM. The analyzer's MIT OR Apache-2.0 choice does not cover the
entire closure; Unicode, LLVM-exception, MPL and other package notices remain
distinct. Twenty-one archives have no captured notice file under the existing
filename scan, including the `ra-ap-rustc_*` family and `line-index`. Their SPDX
declarations are evidence, not replacement license bodies. Resolve original
source rights/notices for the actual payload. The Rust source and compiler
runtime, including `librustc_driver` and LLVM, need their own private closure and
rights. None is admitted by this note.

A new source-only review used the primary
[RustSec database](https://github.com/RustSec/advisory-db/tree/21d909e4c8bd4e14c7aea58bf980c881a0053ab1),
commit `21d909e4c8bd4e14c7aea58bf980c881a0053ab1`, retrieved 2026-10-25.
It parsed 1,332 advisory records with zero parse failures, found 45 package-name
matches, then reviewed all 47 matching closure-version rows against patched,
unaffected and withdrawn ranges. Three require disposition:

| Candidate closure entry | Record and disposition |
| --- | --- |
| `anyhow 1.0.102` | [RUSTSEC-2026-0190](https://rustsec.org/advisories/RUSTSEC-2026-0190.html), unsound `Error::downcast_mut` after context; patched `>=1.0.103`. This is a direct analyzer dependency. No matching call was found in its own `crates/` sources; that is not proof about the linked transitive binary. Resolve affected-function reachability or select a corrected, separately qualified artifact. |
| `crossbeam-epoch 0.9.18` | [RUSTSEC-2026-0204](https://rustsec.org/advisories/RUSTSEC-2026-0204.html), invalid-pointer formatting; patched `>=0.9.20`, unaffected `<0.9.0`. Locked path: analyzer → rayon → rayon-core → crossbeam-deque → crossbeam-epoch. The advisory title and description differ on the formatting trait; retain that uncertainty and inspect the implementation before making an exposure claim. Version match is confirmed; exploitable navigation reachability is unproven. |
| `paste 1.0.15` | [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html), unmaintained, no patched version. The conservative path is optional jemalloc profiling. The pinned macOS release workflow selects the system allocator, so this is expected to be excluded; confirm actual release features/provenance rather than label it a demonstrated shipped runtime vulnerability. |

The remaining version rows were patched, unaffected or withdrawn under those
records. This is a bounded source-closure review, not a clean-binary audit,
proof of exploitability or an audit of the separate compiler runtime. Do not
claim that no advisories apply. The unresolved versions, provenance and missing
rights are release-admission blockers until corrected or explicitly dispositioned
under the existing admission process; successful navigation cannot waive them.

## Evidence and completion limits

New evidence is retained outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/rust-navigation-recheck-20261025/`.
`evidence-manifest.json` binds saved files. `verification-recheck.json` records
44 passing checks and original spans. `recheck-identity.json`,
`inspected-source-identity.json`, `passive-real-compiler.*`, the three
`missing-*` cases, `disabled-proc-macro.*`, `unnotified-source-change.*`,
`execution-route-controls.*` and `network-control.json` preserve the runtime
evidence. `rustsec-head.json`, `rustsec-db.tar.gz`, `rustsec-matches/`,
`rustsec-version-disposition.json` and `source-advisory-paths.json` preserve the
dated advisory review. Scripts are task-owned instrumentation. Shared immutable
candidate/source metadata remains in the original experiment's evidence folder.

No general Rust feature/reference completeness, performance, atomic mutable-file
snapshot, parser memory safety, production sandbox, native Windows execution,
macOS 15 behavior, fresh-install trust, production compiler closure, installed
Rust MCP tool or Codex GUI/model-turn activation is established. The immediate
implementation work is one Rust backend over the existing finite owner, after
resolving the fixed-policy source/probe boundary and admission blockers above.
