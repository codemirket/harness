# Bounded Bash and PowerShell source inspection

Observed 2026-10-25 on macOS 26.5.2 ARM64. This slice adds an isolated finite
Python leaf, Rust process owner and their tests. The initiating agent separately
admitted the exact nine-file macOS payload in
`packaging/admission/m3-shell-source-macos.json` after reviewing these bytes,
sources, native imports, rights and advisory checks. It is now wired through the
finite worker and installed in a separate disposable release. Windows execution
remains open; neither the user's client settings nor the old harness changed.

## Result and semantic limit

The smallest qualified local candidate is the existing private CPython 3.13.17
plus three native extensions: py-tree-sitter 0.26.0, Tree-sitter Bash 0.25.1 and
Airbus CERT Tree-sitter PowerShell 0.26.4. It exposes concrete syntax over supplied
original UTF-8 bytes. It does not start Bash, PowerShell, PSES or a language server.

**Definitions and references remain unsupported.** Four same-spelled variables
in different scopes remain four distinct syntax occurrences, without an invented
binding relation. An unresolved command remains a syntax occurrence. No unique
name, declaration node or complete parse upgrades that result to a resolved
symbol. The [earlier server qualification](m3-shell-navigation-qualification.md)
still owns the Bash/PSES binding counterexamples; this slice does not repair or
conceal them.

`structure_complete` means only that the selected grammar reported no error or
missing nodes within the fully bounded model. It does not establish language
conformance, type validity or binding completeness. The PowerShell grammar itself
references PowerShell 7.3; it is not the official PowerShell 7.6 parser.
[Grammar owner and scope](https://github.com/airbus-cert/tree-sitter-powershell/tree/4ffe56d66aa926c28442befbb5b89d1437d7199c).

The isolated files are `adapters/python/shell_source_inspect.py`,
`tests/test_shell_source_inspect.py`, `src/shell_inspect.rs` and
`tests/shell_leaf.rs`. The current macOS payload admission is owned by the
manifest above. The leaf refuses every runtime/platform except the qualified
CPython 3.13.17 on macOS ARM64, and the Rust owner refuses other targets before
launch.

## Why a direct PowerShell parser was excluded

The public `Parser.ParseInput` calls `ParseMode.Default`, whose post-parse checks
invoke `SymbolResolver`. A `using module` statement reaches
`Get-Module -ListAvailable`, so parsing can discover modules outside the supplied
source bytes. DSC configuration parsing also contains PowerShell/default-keyword
loading paths. This is enough to disqualify an unrestricted source-only contract;
it is not a claim that every parse executes project code. The ARM64 DSC refusal
does not qualify Windows x64 behavior.
[Parser](https://github.com/PowerShell/PowerShell/blob/v7.6.8/src/System.Management.Automation/engine/parser/Parser.cs),
[post-parse checks](https://github.com/PowerShell/PowerShell/blob/v7.6.8/src/System.Management.Automation/engine/parser/ast.cs),
[module discovery](https://github.com/PowerShell/PowerShell/blob/v7.6.8/src/System.Management.Automation/engine/parser/SymbolResolver.cs).

The exact retrieved files match the earlier captured source byte for byte. Their
SHA-256 values are respectively
`4b828414e20f48094bc161234de3109fda7a2ac63eda7bf93f03cf15498aa992`,
`8ae1d4a25b930109c23f86d2fd2464acbab767fae5471b3e01eb241d7d3a3f7c`
and `29af629206e552d05317ce2f72ba27ccbbc0dd475d5e1dd940bc5e205b9ffe2d`.

shfmt 3.14.1 was also inspected and exercised through explicit
`--language-dialect bash --to-json` over stdin. Its explicit dialect disables
EditorConfig lookup, and its parser is useful. The official release has no
Windows ARM64 binary, and a separate Go executable/source closure would not
simplify the shared two-grammar route. Its rights are BSD-3-Clause, not MIT.
[Fixed CLI implementation](https://github.com/mvdan/sh/blob/a3f0c75d21d918756fa38de8b5d3429efde7948b/cmd/shfmt/main.go),
[release](https://github.com/mvdan/sh/releases/tag/v3.14.1),
[license](https://github.com/mvdan/sh/blob/v3.14.1/LICENSE).
It is not an automatic fallback.

## Exact candidate bytes and source correspondence

The primary PyPI JSON endpoints and original wheel URLs were retrieved directly.
Archives were hash-verified and manually extracted; no installer, build script,
package initializer, upstream helper or hook ran. No extras are selected. All
three distributions declare no mandatory Python runtime dependencies.
[py-tree-sitter metadata](https://pypi.org/pypi/tree-sitter/0.26.0/json),
[Bash metadata](https://pypi.org/pypi/tree-sitter-bash/0.25.1/json),
[PowerShell metadata](https://pypi.org/pypi/tree-sitter-powershell/0.26.4/json).

| macOS ARM64 artifact | Archive bytes | SHA-256 |
| --- | ---: | --- |
| `tree_sitter-0.26.0-cp313-cp313-macosx_11_0_arm64.whl` | 140,829 | `6189c6c340c7384357711e3d92645e96bfb79f7a502f86de1ebdb23eb43f7dab` |
| `tree_sitter_bash-0.25.1-cp310-abi3-macosx_11_0_arm64.whl` | 202,659 | `f4a34a6504c7c5b2a9b8c5c4065531dea19ca2c35026e706cf2eeeebe2c92512` |
| `tree_sitter_powershell-0.26.4-cp310-abi3-macosx_11_0_arm64.whl` | 119,092 | `b5dde429c9de55b75906e240d6db1cf85417e2fc0a56d7b321810c2cd4cf3f98` |

The captured source-distribution hashes are
`b40c219edccc4564530c96f8f1556f6202b37cda964d1cbd7bd2b7e68b40a245`,
`bfc0bdaa77bc1e86e3c6652e5a6e140c40c0a16b84185c2b63ad7cd809b88f14`
and `ffc7f7526420fe335cb78823b38bc8b0c27453eb974ca6056779e4cfefffa605`
in the same order.

The py-tree-sitter release tag identifies commit
`a9e753ef67875c55267ebe02e786eac193a53bc1`; its Tree-sitter core submodule is
**`7f534862c3ec939c3a6ee147f7600ef5c1bf900f`**, not the separate Tree-sitter
0.26.0 release commit. All 13 binding-source files and all 45 bundled core-library
files in the source distribution match those pinned repositories byte for byte.
All five Bash `src` files match
`a06c2e4415e9bc0346c6b86d401879ffb44058f7`.
[Binding source](https://github.com/tree-sitter/py-tree-sitter/tree/a9e753ef67875c55267ebe02e786eac193a53bc1),
[actual core source](https://github.com/tree-sitter/tree-sitter/tree/7f534862c3ec939c3a6ee147f7600ef5c1bf900f),
[Bash source](https://github.com/tree-sitter/tree-sitter-bash/tree/a06c2e4415e9bc0346c6b86d401879ffb44058f7).

PowerShell's tag identifies `4ffe56d66aa926c28442befbb5b89d1437d7199c`.
Its source distribution's generated `parser.c` differs only in the first comment
(the generator version is omitted); parser code/tables match. The bundled
`array.h` also differs, but no compiled file includes it: this grammar has no
external scanner, and its extension builds only `binding.c` and `parser.c`,
whose only project include is `tree_sitter/parser.h`. The upstream publishing
workflow regenerates the parser with a floating Tree-sitter setup action.
[Build source](https://github.com/airbus-cert/tree-sitter-powershell/blob/4ffe56d66aa926c28442befbb5b89d1437d7199c/setup.py),
[publishing workflow](https://github.com/airbus-cert/tree-sitter-powershell/blob/4ffe56d66aa926c28442befbb5b89d1437d7199c/.github/workflows/publish_pypi.yml).

PyPI supplies a publish attestation for the inspected PowerShell wheel. Its
statement names the exact wheel digest and its certificate names that repository,
tag and commit. This was captured and decoded, not independently verified against
a Sigstore trust root; it is not a reproducible-build claim. The corresponding
PyPI provenance endpoints for the inspected binding/Bash wheels returned 404.
Exact artifacts/source remain retained; source inspection does not independently
prove binary reproducibility.

## Native and rights closure

The directly loaded runtime subset has only these three files. Package
`__init__.py`, queries, stubs and metadata are not needed or loaded.

| Relative to the explicit private vendor | Bytes | SHA-256 |
| --- | ---: | --- |
| `tree_sitter/_binding.cpython-313-darwin.so` | 339,632 | `6e3e957aee2f969c40b0f97003e862468ede91f1dc28afb310591ad942e4772c` |
| `tree_sitter_bash/_binding.abi3.so` | 1,433,552 | `1d3f561134d6d6a98c7ad890dcc00f019b1a660dab94dcd0bbfd513cb5bece19` |
| `tree_sitter_powershell/_binding.abi3.so` | 1,050,576 | `54ea53de27d65ea3f2b562eb0c46ad6b1c7c99272b01b10204a93f9d62bc530b` |

`otool -L` reports only `/usr/lib/libSystem.B.dylib` for all three; Python API
symbols resolve from the selected private interpreter. The grammar extensions'
undefined symbols contain no filesystem, subprocess or network API. The shared
Tree-sitter binding also exposes logging/graph/debug APIs, including descriptor
writes, which this leaf never calls. Only `Language`, `Parser.parse(bytes)` and
node inspection are used; no query, logging, input/progress callback, included
range, editor provider or debug-graph interface is admitted.

Retain all five captured notice files under `notices/shell-source/`:

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `py-tree-sitter-MIT.txt` | 1,088 | `1af0790543fffac99dd3bd290bf83ca963f4f8029065eb6f35638f4df863ae8e` |
| `tree-sitter-core-MIT.txt` | 1,080 | `c5cfb43042b6b72045f4ba997834d0a7786d2793d91680868b5815b39f14fc78` |
| `tree-sitter-core-unicode.txt` | 21,001 | `6a18c5fac70d7860b57f5b72b4e2c9a1ba6b3d2741eef7ff9767c5379364f10d` |
| `tree-sitter-bash-MIT.txt` | 1,080 | `49bf33cf78ef5897e4e161ce1517df7de1ae5042a65b6bcfd44401e0fc606559` |
| `tree-sitter-powershell-MIT.txt` | 1,068 | `1fca8454c0d77a73f9922a52d39e1f7c339f5da5d03f2766eff66b859fa19477` |

The native runtime embeds Unicode support. Its supplied Unicode/ICU notice is
absent from the wheel's single top-level MIT notice, so the candidate explicitly
supplements it with the entire pinned upstream notice rather than assuming the
package's MIT label covers every incorporated file.
[Core license](https://github.com/tree-sitter/tree-sitter/blob/7f534862c3ec939c3a6ee147f7600ef5c1bf900f/LICENSE),
[incorporated Unicode notice](https://github.com/tree-sitter/tree-sitter/blob/7f534862c3ec939c3a6ee147f7600ef5c1bf900f/lib/src/unicode/LICENSE),
[Bash license](https://github.com/tree-sitter/tree-sitter-bash/blob/a06c2e4415e9bc0346c6b86d401879ffb44058f7/LICENSE),
[PowerShell grammar license](https://github.com/airbus-cert/tree-sitter-powershell/blob/4ffe56d66aa926c28442befbb5b89d1437d7199c/LICENSE).

The admitted selection is **2,862,014 regular bytes**, including the finite leaf
and 25,317 notice bytes; its native subset is 2,823,760 bytes. The three downloaded
wheels total 462,580 archive bytes; their complete extractions total 2,865,158
bytes. The existing admitted general CPython is reused, not counted as free or
replaced. No separate PowerShell/.NET runtime is required by this syntax route.
This is a packaging-size observation, not a memory/latency benchmark.

On 2026-10-25, exact-version OSV queries for all three PyPI distributions and the
actual core commit returned no records; all four owners' public GitHub repository
advisory endpoints returned empty lists. These bounded public checks are retained
and do not establish absence of undisclosed or unindexed vulnerabilities.
[OSV query API](https://google.github.io/osv.dev/api/#tag/v1/POST/v1/query),
[binding advisories](https://github.com/tree-sitter/py-tree-sitter/security/advisories),
[core advisories](https://github.com/tree-sitter/tree-sitter/security/advisories),
[Bash advisories](https://github.com/tree-sitter/tree-sitter-bash/security/advisories),
[PowerShell grammar advisories](https://github.com/airbus-cert/tree-sitter-powershell/security/advisories).

## Verification and integration contract

Ten tests pass against both the original extraction and the three-file private
subset, using exact private CPython 3.13.17, `-I -S -B`, no usable host PATH and
macOS `sandbox-exec` network denial. They cover independently checked original
Unicode/astral/CRLF spans, local/global name collisions, unresolved names, error
and missing-node evidence, stale revisions, continuation/missing pointers,
NUL/source/node/depth/metadata/output limits, semantic/unknown-field refusal,
tampered payloads, malicious unused package initializers, single-frame IPC and
missing private payloads.

The passive fixture contains Bash command substitution, `source`, `eval` and a
network command, plus PowerShell `using module`, `using assembly`, DSC
configuration, module import, file write and dynamic evaluation. No source marker
or Python audited external operation occurred. Native grammar source and linked
imports were inspected as well. This qualifies the finite tested route; it is not
an OS sandbox or an unrestricted claim about every native parser API.

The exact shared-core contract is:

1. Launch the existing private Python with `-I -S -B`, the private leaf path,
   `--release-id RELEASE_ID --vendor RELEASE/runtime/shell-source`, a cleared
   environment and the existing owned job directory/process/deadline machinery.
   There is no global-runtime, package, module, settings or PATH fallback. The leaf
   hashes each canonical regular native file before loading and checks its stamp
   afterward. The owner must continue enforcing immutable release identity and
   actual process cleanup.
2. Use the same finite envelope as Python inspection: `protocol_version`,
   `release_id`, `job_id`, `source_revision`, `source_utf8`, `request`. The exact
   request fields are `language` (`bash` or `powershell`), `pointer`,
   `expected_revision`, `start_child`, `limit`, `max_response_bytes`. The owner
   must supply the exact expected revision, including on the root request.
3. Reuse the `WebSourceInspection` result shape and pagination semantics.
   Source is capped at 1 MiB, frame at 2 MiB, nodes at 32,768, retained depth at
   64, page children at 100 and result at 32 KiB before the transport wrapper.
   Every pointer is an occurrence path bound to the supplied source revision.
4. Preserve `ERROR` and missing-node issues and set `structure_complete=false`.
   A missing node has no source location; its issue may carry the parser's
   zero-width insertion position. A real error node keeps its original range.
   Pagination completion is distinct from structural completeness. Every valid
   span is checked against original UTF-8 character boundaries; no newline,
   BOM or identifier normalization occurs.
5. Reject definition/reference/symbol operations at the owning navigation
   dispatcher as unsupported for this route. The leaf accepts no such methods,
   project paths, configuration, plugin controls or import graph. Do not convert
   a declaration/name field into semantic success. Rehash the actual saved input
   before the owner returns current-source results.
6. Use the exact admitted dependency/native/notice bytes. The finite worker,
   disposable installed MCP and Codex app-server route are verified below;
   relocation and native Windows remain separate acceptance gates.

## Isolated Rust owner

The isolated Rust owner now implements the process boundary above. Its public
`ShellPayload { python, script, vendor }`, `validate_shell_request` and `run_leaf`
reuse the existing shared request/result types and real process helpers. It
accepts only Bash/PowerShell requests and macOS ARM64. The vendor directory must
contain exactly the three declared subdirectories, each with exactly its pinned
native extension; extra files, symlinks, additional hard links, size/hash drift or
noncanonical paths refuse before launch. File/directory identity, modification
time and Unix change time are checked before and after execution, including a
same-byte file replacement. Release inventory continues to own the Python/script
hashes; these paths are checked as canonical regular files and their identities
must remain unchanged during the operation.

The owner launches private `-I -S -B` Python with a cleared environment containing
only the private temporary-directory variables and a private job cwd. It caps
execution at the earlier caller deadline or 30 seconds, sends one bounded frame,
requires exactly one validated response followed by EOF, caps stderr through the
common helper, and confirms child reap on cancellation, timeout and malformed
output. Callers must cancel and await; aborting the future is not reap proof. A
valid response is withheld if final payload verification changes.

Before compilation, `python3 scripts/check-build-inputs.py` passed. With the
disposable `r-af5c872947b8a57c903f8283bc12f25a9cd5797c5bde5afa04f55970304b59e7`
release's explicit private Python and the frozen script/native subset,
`cargo test --locked --offline --test shell_leaf` passed 16 tests, comprising
eight shell tests and eight included common-helper tests. The shell tests
executed the real Bash/PowerShell leaf and exercised malformed/incomplete/trailing
frames, oversized stdout/stderr/result budgets, a wrong backend, nonzero exit,
prelaunch cancellation/deadline/non-shell refusal, owned cancellation/deadline
reap, invalid native trees, and native/script replacement after launch. The
included helper's optional Node journeys retain their own explicit-runtime
guards; this run does not add Node qualification. An initial incomplete-output
fixture closed Python's wrapper without forcing descriptor EOF; it was corrected
to close descriptor 1, then the full target passed.
`cargo clippy --locked --offline --test shell_leaf -- -D warnings` also passed.

The exact explicit inputs are `MIRKET_TEST_PYTHON`,
`MIRKET_TEST_SHELL_SCRIPT` and `MIRKET_TEST_SHELL_VENDOR`; no installed/global
runtime is discovered if those test inputs are absent. The test target compiles
`src/shell_inspect.rs` by path, independently of the public registration. Its
isolated checks complement the installed evidence below.

## Windows and remaining gates

All six Windows wheels were downloaded and hash-verified. Their native PE headers
identify x64 (`0x8664`) and ARM64 (`0xaa64`) correctly. Native imports require the
selected private `python313.dll` for Tree-sitter, `python3.dll` for the two abi3
grammars, `VCRUNTIME140.dll`, kernel32 and the Windows CRT API-set libraries.
The private Windows CPython/VC runtime closure, source/notices and DLL resolution
must be reconciled and executed on each target; no host/global DLL fallback is
qualified here. Their exact archive/native hashes and imports are retained in
`windows-native-evidence.json`. Wheel existence is not native execution evidence.

## Integrated macOS ARM64 checkpoint

The admitted nine-file shell selection is pinned in
`packaging/admission/m3-shell-source-macos.json`. The release inventory requires
its authored script, three native modules and five original notices. The exact
assembler `scripts/assemble-shell-source-macos.py` reads the approved private
native subset and repository script/notices without running upstream code. Five
synthetic assembly tests cover exact output, byte tampering, symlinks, existing
destinations and rollback after a forced copy failure. A real assembly reproduced
all nine files and 2,862,014 bytes.

The same-binary finite worker and `shell_inspect` MCP route are now integrated.
The new disposable macOS ARM64 four-component release source is
`~/.codex/task-state/harness-rewrite-implementation/releases/codex-shell-source-clean-20261025`,
identity `sha256:0fdf6a45d9d0ec0b51289ec16b7dd8cb23e51cffeaec0bba6dc3f51512693aea`.
`mirket setup` installed it in the separate
`homes/codex-shell-source-clean-20261025` home, and `doctor --json` reported a healthy
manifest, launcher and active payload. The installed MCP smoke passed Bash and
PowerShell structure, original Unicode/CRLF spans, parser recovery, stale
revision and scope/language refusal, and no source execution. The Desktop-bundled
Codex CLI app-server then registered Mirket into a disposable Codex home and
called `shell_inspect` successfully through its native MCP route; no model turn
or user's live Codex configuration was changed.

After integration, the build guard, `cargo test --locked --offline` (510 library
passes, three ignored; CLI and isolated integration targets passed), strict
all-target Clippy, `cargo fmt --all -- --check`, the 90 passing adapter tests
(one optional browser test skipped), and the complete development MCP inventory
smoke passed. These checks establish the tested macOS ARM64 syntax route.
Binding-correct definitions/references, native Windows behavior, macOS 15
qualification and actual Codex GUI activation remain open.

All archived inputs, source comparisons, native import evidence, public advisory
responses, tests and the assembled candidate are outside shipped source at
`~/.codex/task-state/harness-rewrite-implementation/shell-source-next-20261025/`.
`private-subset.manifest.json` records every candidate file, hash and size; its
SHA-256 is `f9a60b261558f31857267682c837660b802d5a50a4b3678c480ecbb9861eee38`.
The admitted release uses the same native modules and notices, with the authored
script's final status comment pinned separately in the release admission manifest.
