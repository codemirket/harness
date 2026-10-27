# Selected-source Python navigation leaf

Implemented 2026-10-25 on macOS 26.5.2 ARM64 using the already selected private
CPython 3.13.17. This is an isolated implementation and handoff, not an installed
release, public MCP route, desktop activation result or Windows qualification.
It replaces none of the existing syntax inspection operations. No commit, push,
publication, upstream installer, package manager, hook or project code ran.

## Ownership and integration

`src/python_navigation.rs` owns typed requests/results, exact payload checks,
selected source capture/rechecks and the finite child lifecycle.
`adapters/python_navigation.py` owns one immutable source projection, private
backend extraction and cache, passive Jedi calls and original-byte locations.
The existing `process::OwnedChild` supplies process groups, cancellation and
confirmed reaping. No new daemon, runtime, production Cargo dependency or
interchangeable analyzer fallback was introduced.

The integration owner must:

1. Select original files explicitly with `CapturedSources::capture(root, files)`.
   Each `SourceFile` has an ID, original path, source/stub role and expected
   SHA-256 revision. `ProjectScope` rejects detected links and escaping paths.
   Never send original absolute project paths to the Python process.
2. Construct `PythonRequest`: protocol version 1, release/job IDs, ordered
   captured sources, `model_revision(&sources)`, query and limits. A source has
   `id`, relative portable `path`, `role` (`source`/`stub`), original UTF-8 `text`
   and `revision`. The query has `operation` (`definition`/`references`),
   `source_id` and an original UTF-8 `byte_offset` inside an identifier.
3. Supply `PythonPayload` with explicit canonical private paths to `python`,
   `script`, `jedi_wheel` and `parso_wheel`. Acquire ordinary worker admission,
   a fresh empty owned job directory, a cancellation token and deadline, then
   call `run_leaf`. Never abort its future and treat that as confirmed cleanup.
4. Recheck `CapturedSources::recheck()` before delivering a result, and bind
   each result location to the corresponding original file. Revalidate wire
   data with `validate_response` across worker boundaries.
5. Keep admission/ownership on an error with `cleanup_confirmed == false` and
   reconcile explicitly. `cleanup_after_reap` can remove this leaf's single
   bounded no-link temporary tree only after the process is known reaped.
   The ordinary runner already calls it on success, cancellation and timeout.
6. Add source, adapter, exact wheels, notices and runtime identities to the
   release inventory before route exposure, then exercise the actual installed
   worker/MCP path and Codex client. This isolated work changes no shared route,
   release manifest, adapter package manifest or installation.

The model revision binds ordered source IDs, projection paths, roles and byte
revisions, both exact wheel hashes, adapter bytes and the fixed
`cpython-3.13.17-macos-arm64` model. **This identity is macOS ARM64 only.** A
future Windows implementation requires a separately qualified, target-bound
model identity and native process/runtime ownership; it must not reuse this one.

## Semantic contract and bounds

The fixed `coverage: selected_parser_names` states the finite scope. Every
selected `.py`/`.pyi` is parsed; `get_names(all_scopes=True, definitions=True,
references=True)` enumerates names, and each binding is resolved by
`Name.goto(follow_imports=True, follow_builtin_imports=False)`. The adapter does
not use the silently capped ordinary `get_references` API and does not implement
a lexical reference matcher or alternate semantic resolver.

`complete` means the bounded selected-name pass finished with a unique selected
target and no observed unresolved/ambiguous binding or unsupported construct.
This is not a claim about unselected sources, runtime reachability, dataflow or
all Python semantics. Assignment aliases, conditional/control-flow constructs,
star imports, decorated definitions, inherited classes and dynamic reflection
are conservatively incomplete. Resolved builtin/backend-stub identities are
not fabricated into original-source spans. Unknown/external targets are
incomplete with no invented target location. Ambiguous queried targets,
syntax failures, unsupported query positions and boundary violations are
refused; no locations accompany a refusal.

Name, location, deadline and output cutoffs return `incomplete` with an explicit
reason and an empty location set. Other incomplete results may retain verified
locations plus reasons. Every location includes selected source ID/revision,
identifier bytes, and original UTF-8 start/end offsets. BOM, CRLF and Unicode
code-point columns are translated using original bytes, then checked again by
Rust. Duplicate, outside-source, stale, malformed or mismatching spans fail.

Limits are 128 sources, 256 KiB per file, 2 MiB total source, 4 MiB request,
60 KiB response, 32 path components, 20,000 names, 256 locations and 10 seconds
analysis per request; callers may narrow name/location/time bounds. Rust caps
the whole child lifecycle at 30 seconds and independently kills/reaps on its
deadline or cancellation. There is no cross-model in-memory or disk cache.

## Before-read and passive execution policy

Every invocation extracts only verified regular wheel members, creates selected
source files exclusively, freezes source/backend files and directories and uses
a new owned cache. Case-insensitive module collisions and `.py`/`.pyi` or
module/package identity collisions are refused. No project configuration,
`.pth`, startup module, site package, environment discovery or project executable
extension is copied or selected.

The private interpreter uses `-I -S -B`, cleared environment, unusable PATH,
private HOME/TMP and an empty job cwd. Jedi receives an explicitly constructed
`Project`, fixed source sys.path, `smart_sys_path=False`,
`load_unsafe_extensions=False`, empty `auto_import_modules`, unsafe interpreter
execution disabled and its exact-pin same-process `InterpreterEnvironment`.
It calls `Script`, never the executing `jedi.Interpreter` API.

A CPython audit hook checks backend opens/enumeration before filesystem access.
It permits selected projection bytes, the verified backend, private stdlib
(excluding site/dist packages) and owned cache, and rejects outside reads,
process/network operations and source execution. Static source-inferred
`sys.path` additions cannot authorize an outside read. Existing symlink source
selection is refused by Rust; the extracted backend/source trees contain no
links. The tests install the actual hook and attempt to open an outside FIFO
and an in-projection link to it: refusal occurs before a blocking OS open.

This is a policy around a trusted, pinned passive backend, **not an OS sandbox**.
Private runtime/stdlib integrity and parent lifeline remain the release/worker
owner's responsibilities. Path-based checks do not contain a hostile same-user
ancestor replacement; restrictive modes are not same-user isolation. Full
immutable-release and installed-parent-death behavior need their own gates.

## Exact closure, rights and source

The selected unchanged wheels are:

| Input | SHA-256 |
| --- | --- |
| Jedi 0.20.1 wheel, 4,885,980 bytes | `0fb16d86c4a4c73c37ba518c77419975e30fcc620658a8d14fbb5720cdd34142` |
| Parso 0.8.7 wheel, 107,025 bytes | `a8926eb2a1b915486941fdbd31e86a4baf88fe8c210f25f2f35ecec5b574ca1c` |
| Jedi source distribution | `2f71208c3f9c1bca057c0e90d3f272aba44ace88fc4067d7587e9e069331b7e5` |
| Parso source distribution | `eaaac4c9fdd5e9e8852dc778d2d7405897ec510f2a298071453e5e3a07914bb1` |

The existing remediation acquisitions were reused read-only. An independent
inert recheck verified all 5,661 hashed wheel RECORD entries and all 5,651
non-dist-info files against the corresponding source distributions. The full
wheels contain 5,663 regular files and 14,939,387 logical bytes. No installer or
package import was needed for this check.

Retained unchanged notices include Jedi MIT LICENSE/AUTHORS, Parso MIT/PSF
LICENSE/AUTHORS, bundled typeshed Apache-2.0/additional MIT text and Django-stubs
MIT text. Jedi's unsigned source tag resolves to
`ab89beb06cc4ad51ec3b99d7f2023401a8d11815`; Parso's to
`bfe30584415cde2e5a686512c2b109e1d06da3b6`. The upstream typeshed/Django pins
and original per-file rights inventory remain linked by hash in the private
payload qualification. Wheel/sdist equality and unsigned tags establish those
observations, not signed or reproducible builds.

## Verification and evidence

`tests/test_python_navigation.py` runs the actual exact private interpreter and
wheels. Its 13 finite groups exercise alias/re-export/attribute definitions and
the exact 10-location reference set, a 3-location two-character name, 71
locations across 35 consumers, explicit stubs, BOM/Unicode/CRLF byte slices,
outside static paths, actual audit/FIFO denial, passive sentinels, changed source
models, syntax and ambiguity, unsupported assignment aliases, malformed frames,
path/revision collisions, and name/location/deadline limits.

`tests/python_navigation.rs` independently exercises request/result boundaries,
malformed/forged spans and statuses, content revisions with restored original
mtime, symlink/ancestor/outside selection, exact real payload launch,
cancellation, deadlines, reaping and scratch cleanup. The two real-payload tests
are explicitly ignored unless selected with `--include-ignored` and
`MIRKET_PYTHON_NAVIGATION_PAYLOAD` pointing to a private payload JSON; ordinary
test counts must not be reported as real backend coverage. Included legacy
process tests retain their own explicit Node-runtime opt-in.

Final hashes, notices, source verification, commands and outcomes are retained
under
`~/.codex/task-state/harness-rewrite-implementation/python-navigation-implementation-20261025/`.
Read `validation.json`, `python-tests.json`, `payload-qualification.json` and
`evidence-manifest.json` for the exact tested source identity. Timing is local
fixture wall time, not a performance claim. Installed MCP/Codex behavior,
universal Python references, native Windows, oldest-platform compatibility and
release admission are not established by these isolated tests.
