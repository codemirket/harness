# Python navigation remediation: Jedi/parso

Observed **2026-10-25**, macOS **26.5.2 ARM64**, with the already selected private
**CPython 3.13.17**. **Jedi 0.20.1 + parso 0.8.7 is a promising smaller backend
for explicitly selected source snapshots, but is not admitted.** Its ordinary
reference API fails the same alias-call requirement as ty/Pyright and has
additional silent limits. A task-only prototype recovers the tested references
by asking Jedi to resolve every parser-provided name. File-access policy,
immutable-cache ownership, integration, exact release admission and native
Windows verification remain required.

This work adds no dependency, release payload, public route or registered
adapter. All executable experiments and downloaded bytes are in
`python-navigation-remediation-20261025/` under the implementation task-state
root. The installed runtime and prior qualification evidence were used read-only.
No upstream installer, hook, build script, pip/npm invocation, Cargo build,
commit, push or user-configuration change was performed.

## Exact candidate and rights evidence

The current [Jedi metadata](https://pypi.org/pypi/jedi/0.20.1/json) requires Python
3.10+ and exactly one non-extra dependency, `parso >=0.8.7,<0.9.0`.
[Parso 0.8.7](https://pypi.org/pypi/parso/0.8.7/json) has no non-extra dependency.
Development/documentation extras were neither selected nor executed.

| Inert input | Bytes | SHA-256 |
| --- | ---: | --- |
| `jedi-0.20.1-py2.py3-none-any.whl` | 4,885,980 | `0fb16d86c4a4c73c37ba518c77419975e30fcc620658a8d14fbb5720cdd34142` |
| `parso-0.8.7-py2.py3-none-any.whl` | 107,025 | `a8926eb2a1b915486941fdbd31e86a4baf88fe8c210f25f2f35ecec5b574ca1c` |
| Jedi source distribution | 3,121,738 | `2f71208c3f9c1bca057c0e90d3f272aba44ace88fc4067d7587e9e069331b7e5` |
| Parso source distribution | 401,824 | `eaaac4c9fdd5e9e8852dc778d2d7405897ec510f2a298071453e5e3a07914bb1` |

Archive hashes matched PyPI metadata. Safe extraction accepted regular files and
directories only; neither archive supplied an extracted link. All 5,661 hashed
wheel RECORD entries matched their bytes and sizes. All **5,651 non-dist-info
files** matched the corresponding source-distribution files exactly. The two
extracted wheels total **14,939,387 logical bytes, 5,663 files**, excluding the
already selected Python runtime, evidence copies and a future adapter. They
contain no separate executable, shared library, extension module or WASM file.

The annotated, unsigned source tags resolve to Jedi commit
[`ab89beb06cc4ad51ec3b99d7f2023401a8d11815`](https://github.com/davidhalter/jedi/tree/ab89beb06cc4ad51ec3b99d7f2023401a8d11815)
and Parso commit
[`bfe30584415cde2e5a686512c2b109e1d06da3b6`](https://github.com/davidhalter/parso/tree/bfe30584415cde2e5a686512c2b109e1d06da3b6).
The tag, PyPI package and source-distribution observations are distinct; matching
wheel/source-distribution bytes do not prove a signed or reproducible build.

The actual wheels retain original notices, rather than license labels alone:

- Jedi MIT `LICENSE.txt` and its referenced `AUTHORS.txt`.
- Parso MIT/PSF `LICENSE.txt` and `AUTHORS.txt`. Its license identifies files
  derived from CPython and includes the PSF terms and copyright text.
- Bundled typeshed `LICENSE`, containing Apache-2.0 and additional MIT terms;
  the pinned Jedi tree records typeshed gitlink
  `4bb9d8351d0795fbc7dee3da069013a48641689d`.
- Bundled Django-stubs MIT notice with its original author; gitlink
  `fd057010f6cbf176f57d1099e82be46d39b99cb9`.

Unchanged notice copies and a per-file source/rights inventory are retained in
task-state. This is a substantially smaller and more inspectable permissive
source closure than the qualified ty binary build closure. It does not authorize
shipping: select the exact retained tree, preserve notices/source attributions,
approve the dependency batch and verify assembly. An exact-version OSV batch
query returned `{}` for each package on this date; that is a dated absence of
reported matches, not a security guarantee.

## Passive API and observed failures

The probe calls `jedi.Script`, explicit `jedi.Project`, and Jedi's internal
`InterpreterEnvironment` to reuse the current private process. It does **not**
call `jedi.Interpreter`, import project modules, create a project environment or
use default interpreter discovery. This internal environment class needs an
exact-pin compatibility test; its name must not be confused with the unsafe
REPL API. The [upstream security contract](https://jedi.readthedocs.io/en/latest/docs/features.html#security)
distinguishes static `Script` from executing `Interpreter` behavior.

The private interpreter was launched with `-I -S -B`, private HOME/cache/temp,
an unusable PATH, and explicit verified library roots. The Project had fixed
`sys_path`, `smart_sys_path=False`, `load_unsafe_extensions=False` and no
environment path. Adapter-owned settings cleared `auto_import_modules` and
disabled unsafe Interpreter behavior. All calls were sequential. No packages
were imported from the original project or an ambient environment.

A logging-only CPython audit hook recorded no process/network API attempt.
macOS denied child execution and network; an independent control returned EPERM
for both a `touch` child and localhost connection. Top-level project code,
startup/sitecustomize/usercustomize/setup/backend code and executable `.pth`
sentinels produced no marker. The explicitly constructed Project did not read
the supplied `.jedi/project.json`, `pyproject.toml` or `.pth` configuration.
Reference enumeration did read sentinel `.py` files as source, without executing
them. These observations establish the exercised passive path, not an OS
sandbox or all possible Jedi behavior.

| Hand-authored requirement | Actual observation |
| --- | --- |
| Same-file, direct import and import-alias definitions | Pass; original function declaration returned |
| Module alias/attribute and re-export definitions | Pass |
| Explicit dependency `.pyi` | Pass through a supplied stub directory; no environment discovery |
| Unknown definition and document names | Empty unknown result; expected declarations present |
| Original Unicode/BOM/CRLF positions | Pass for Turkish paths/identifiers and a non-BMP emoji before calls |
| Stock references at the original declaration | **Fail:** omits alias declarations/calls and re-exported alias uses |
| Stock references for two-character `go` | **Fail:** returns only its declaration, omitting both cross-file occurrences |
| Stock references, 35 direct consumers | **Fail:** returns **61/71** expected locations, without a completeness flag |
| Explicit sys_path as a source-read boundary | **Fail:** literal source `sys.path.append(outside)` resolves and reads an external `.py` file even with smart paths disabled |
| A symlinked source inside the projection | **Fail:** follows the link; textual in-tree output path conceals external canonical storage |
| Saved import changed with its former mtime restored | **Fail:** cached old `selam` name points at the now-`vedar` bytes |

The [pinned reference implementation](https://github.com/davidhalter/jedi/blob/ab89beb06cc4ad51ec3b99d7f2023401a8d11815/jedi/inference/references.py)
skips project search for names of length two or less, limits searches to 2,000
opened/30 parsed files, and searches the original spelling. The failures are
discriminating counterexamples to treating `get_references()` as complete.
The [import owner](https://github.com/davidhalter/jedi/blob/ab89beb06cc4ad51ec3b99d7f2023401a8d11815/jedi/inference/imports.py)
adds statically inferred source path modifications after the explicit Project
paths. A projection plus fixed sys_path therefore does not by itself constrain
all backend reads. Rejecting an external result after analysis cannot undo that
read.

An initial `fast_parser=False` probe failed with a Parso-cache KeyError, before
navigation. The passing paths retain Jedi's default `True` and sequential use.
The [API warning](https://jedi.readthedocs.io/en/latest/docs/api.html#script)
about mutable parser objects still applies. Imported-file disk caches use
pickle and mtime; private fresh caches and immutable model identities are
necessary. A caller-controlled or cross-model cache cannot be trusted.

## Selected-source prototype and measurements

The isolated prototype enumerates **parser-provided** names with
`get_names(all_scopes=True, definitions=True, references=True)` in every
explicit source snapshot. For each name it calls
`Name.goto(follow_imports=True, follow_builtin_imports=False)` and compares
resolved declaration locations. Jedi owns binding/import resolution; this is
neither lexical reference matching nor a second semantic resolver.

It returned the exact **10/10** declaration/import/alias/call/attribute/re-export
locations, **3/3** short-name locations and **71/71** locations for the 35-consumer
case. Each pass used the predeclared finite source set. The first scan of 176
names took **492.689 ms**; later scans took **73.058 ms** and **77.835 ms**. The
same process imported Jedi/parso in **299.196 ms**, its first ordinary definition
took **14.560 ms**, and maximum RSS for the whole experiment was **76,791,808
bytes (73.23 MiB)**. The initial ordinary-API-only run observed 62.72 MiB maximum
RSS. Filesystem caches and concurrent desktop load were uncharacterized. These
are local observations, not p95, cold-install, large-project or idle-lifetime
claims; the ty/Pyright reports used different measurement paths.

The prototype result contains **101 passing and seven failing assertions**;
the failures above remain preserved, including a separate stale-identifier-slice
failure. The three enumeration passes do not turn the whole experiment into a
pass. Enumerating every token does not prove complete Python semantics: an
unresolved or multiply resolved name, dynamic attribute, assignment alias,
conditional import, star import or inference cutoff needs its own contract and
corpus. The prototype makes no generic completeness claim.

## Recommended integration boundary

Qualify a single **Jedi selected-snapshot leaf on the existing general Python
worker**, instead of adding a Python LSP stack or another runtime. Keep ty and
Pyright unadmitted; do not retain an interchangeable fallback chain. This path
has a smaller source/notice closure and a demonstrated bounded repair for the
held alias example. It is not a drop-in production replacement yet.

Before integrating it:

1. Prepare a finite, immutable, no-link source/stub projection with a
   one-to-one original-byte map, explicit interpreter/language version and
   platform model. Bind every source, stub, setting and backend hash to the
   model revision. Do not copy project configuration or executable extensions.
2. Enforce the approved source-read boundary **before** backend file access.
   Prove that static sys.path changes, relative imports, namespace paths,
   archive imports and links cannot introduce unselected sources. A pinned
   backend import-policy adaptation may be smaller than a new resolver, but
   it still needs source inspection and independent refusal probes. Current
   stock behavior has failed this gate.
3. Use fresh private cache/projection identities per model; never mutate a
   projection or rely on mtimes. Serialize the backend and discard all live
   Name/Script objects at the invocation/model boundary. Keep startup flags,
   settings and same-process environment adapter-owned.
4. For bounded references, enumerate selected parser names and delegate every
   binding to Jedi. Validate unique target identities, original identifier
   bytes and all returned paths. Refuse ambiguity; classify unresolved or
   unsupported forms as incomplete. Bound source bytes, names, work, output
   and deadlines; a budget cutoff must produce explicit truncation.
5. Reuse existing Rust source/revision/process/admission ownership, recheck
   originals before response, assemble the exact approved dependency tree and
   notices, and exercise actual installed MCP/Codex operations and failures.
   No new daemon, project interpreter execution or package manager is needed.

[Rope 1.15.0](https://pypi.org/pypi/rope/1.15.0/json) declares LGPL-3.0-or-later
and `pytoolconfig[global]`; [Astroid 4.3.4](https://pypi.org/pypi/astroid/4.3.4/json)
declares LGPL-2.1-or-later and would still need a references owner.
[Zuban 0.10.1](https://pypi.org/pypi/zuban/0.10.1/json) declares AGPL-3.0-only.
These labels do not mean their use is forbidden, but they do not offer a simpler
permissive admission boundary than the two tested wheels. None was executed or
admitted. Parso alone supplies parsing rather than semantic resolution.

## Native Windows and evidence limits

The same two `none-any` wheels have no architecture-specific native payload.
Jedi's Requires-Python field, rather than its legacy `py2.py3` wheel tag, governs
the interpreter floor. The official
[python-build-standalone 20261021 release](https://github.com/astral-sh/python-build-standalone/releases/tag/20261021)
lists normal-GIL CPython 3.13.17 install-only assets for Windows x64 and ARM64;
their exact metadata is retained. **Neither was downloaded or run in this
experiment.** The existing [Python admission](../packaging/admission/m3-python-macos.md)
is macOS ARM64 only and explicitly leaves Microsoft's runtime redistribution
terms open. Pure wheels remove an additional analyzer CRT closure; they do not
close the Python runtime's Windows rights, DLL loading, ownership, Unicode-path,
case-insensitive import, cache or actual-native-execution gates.

Retained evidence includes `probe.py`, `prototype-result.json`,
`fast-parser-false-stderr.txt`, the first results/expectation corrections,
`summary.json`, `denial-control.json`, `source-rights-inventory.json`, unchanged
archives/notices, registry/tag metadata and `windows-feasibility.json`.
`evidence-manifest.json` records their hashes. No installed Python navigation,
complete general-Python references, native Windows support or release admission
is established by this document.
