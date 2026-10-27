# Read-only Python source structure

`python_inspect` exposes one explicitly selected saved Python source through the
existing finite owned worker. Its public language enum is exactly `python`.
`code_inspect` retains `js`, `ts`, `jsx`, `tsx`; `web_inspect` retains `html`,
`css`. These routes share project scope, immutable input, response validation,
saved-source recheck, admission and process ownership. This is AST structure,
not Python symbol search, definitions, references, type checking or project
navigation.

## Request and source evidence

Supply `expected_release_id`, an explicit absolute `project_root`, and `request`
with absolute `path`, `language: "python"`, `pointer`, nullable
`expected_revision`, `start_child` and `limit`. Empty pointer selects the source
root. Every nonempty pointer and nonzero child offset requires the observed
full `sha256:` revision. Ordinal pointers identify occurrences within that
revision; they are opaque to callers and have no semantic identity across
edits.

The response remains `{ release_id, inspection: { project_root, path, language,
pointer, outcome } }`. Each page reports the fixed backend
`{ name: "cpython_ast", version: "3.13.17" }`, exact original-source revision
and byte count, selected AST occurrence, bounded immediate children, and a
revision-bound continuation. Each occurrence has `kind`, `pointer`,
`child_count`, and either original `start_byte`/`end_byte` bounds or a null
location with `location_reason`. Expand proven spans with `file_read_ranges`
and the returned source revision.

`complete` describes immediate-child pagination. `structure_complete` is false
when any retained AST occurrence lacks a provable source range; it is not an
execution or semantic-validity claim. The source root covers all supplied
bytes, including an initial UTF-8 BOM. Other ranges use CPython's original
UTF-8 byte columns, translated through physical line offsets while retaining
CRLF, lone CR, LF, tabs and Unicode bytes. No source reserialization or line
ending normalization is used. Syntax-error columns use their separate
one-based Unicode-column convention and yield a point only when proven.

`name` is bounded parser-decoded display metadata, including Python's Unicode
identifier normalization. An attribute's name does not narrow its full AST
expression range to that name, and no inferred name span is exposed. AST
positions for decorated definitions begin at `def`/`class`; decorator
expressions remain separate children in source order. Comments, punctuation,
and operator/context helper objects are not separate occurrences. Containers
such as `arguments` and `match_case` that lack their own parser positions retain
their children and explicitly unavailable locations. No bounding box around
their descendants is presented as an exact source range.

`changed`, `missing_pointer`, `invalid_start`, `budget_blocked` and `refused`
remain explicit outcomes. Syntax errors refuse the entire tree; CPython's AST
parser offers no recovery tree, so no repaired source or partial preceding
statements are returned. NULs, incompatible source-encoding declarations,
parser failures and traversal quotas also refuse. A metadata block retains
its occurrence pointer/kind/proven location; names are never silently cut.
An output too small even for that block returns a compact `output_budget`
refusal. A changed source observed after confirmed child cleanup discards the
whole page and every location. This is a recheck, not an atomic filesystem
snapshot.

## Fixed private runtime

`src/python_inspect.rs` launches only the explicit configured private
CPython executable and file-backed `adapters/python_source_inspect.py`, using
`-I -S -B`, an empty inherited environment and the existing private worker
directory. The script is a sibling of the configured SQL script in the
documents component; it does not load SQLGlot or any wheel. Installed release
assembly owns script/runtime inventory and pinning. Source development keeps
the script at `adapters/python/python_source_inspect.py`. There is no PATH,
system Python, project interpreter or package-manager fallback.

Only standard-library AST parsing and UTF-8 encoding detection receive source
bytes. `ast.parse` uses a constant logical filename, `exec` grammar mode,
Python 3.13 grammar, type-comment parsing and `optimize=0`; this requests an AST,
not executable compilation or evaluation. The leaf receives no project path.
There is no project import, module/configuration discovery, plugin, build,
package acquisition, file output or project-source execution. Trusted private
Python and standard-library initialization are still host processes, not an OS
sandbox. The leaf refuses an interpreter other than CPython 3.13.17.

## Bounds and observed verification

Bounds are 1 MiB original source, depth 64, 32,768 retained occurrences, 100
immediate children per page, 4 KiB per occurrence's encoded metadata, 16 sampled
location issues, and 32 KiB for the complete structured result after reserving
the parent envelope. The shared transport retains its 2 MiB input frame,
64 KiB leaf response frame and 16 KiB counted stderr. The Python child has a
30-second maximum execution deadline inside the existing finite worker/job
deadline, cancellation and cleanup owner. Parser-internal recursion or memory
failure can precede the bounded iterative walk and results in refusal or owned
child failure, never a partial tree.

On macOS 26.5.2 build 25F84 ARM64, the build-input guard and locked offline
binary build passed. Eleven Python test groups passed under the explicitly
selected admitted private CPython 3.13.17. They exercise original
BOM/CRLF/lone-CR/Unicode spans, decorators and unavailable containers, Python
3.13 type aliases and match syntax, syntax/encoding/NUL refusals, canonical
pointers, revision-bound paging, source/depth/node/metadata/output quotas,
duplicate/noisy/trailing protocol input, quiet diagnostics, and preserved
startup/configuration sentinels. Throwing import/file/process sentinels after
trusted initialization observed no calls while parsing the exercised source;
that proves the finite exercised path, not arbitrary Python APIs.

Six focused Rust tests passed with explicit `MIRKET_TEST_PYTHON` and
`MIRKET_TEST_BIN` selections, so their native paths ran. They cover separate
public language deserialization, fixed backend and original-byte validation,
saved-source change/symlink refusal, real private-leaf Unicode and malformed
source, deadline termination/reaping, and the actual same-binary finite worker.
Without these explicit environment selections, the three native tests return
early. The disposable four-component macOS ARM64 release
`sha256:254db338a1b0bb03dae043e67c4b163d9060bcdd01cecfa90cb01a9bd4a499c6`
passed `doctor` and `tests/python-inspect-installed-smoke.mjs` through actual
MCP, private CPython and same-binary worker. The installed fixture covers
original BOM/Unicode/CRLF byte spans, unavailable syntax locations, syntax
refusal, pagination, stale revision, scope rejection and no project writes.
The same release passed combined five-dialect SQL/TOML/XML/YAML/PDF and
HTML/CSS/JS/TS/JSX/TSX installed smokes, plus Codex CLI app-server read,
policy and managed-registration journeys without a model turn or user
configuration change. A separate [Desktop-bundled Codex model turn](m3-codex-model-turn.md)
read the same installed release's status and one saved file; it did not call
`python_inspect`. Native Windows ownership, actual Codex GUI activation and
cross-file semantic Python navigation remain unqualified.
