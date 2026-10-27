# Read-only JavaScript/TypeScript source structure

`code_inspect` exposes one saved JavaScript, TypeScript, JSX or TSX source through
the existing finite original-source worker. Its public language enum is exactly
`js`, `ts`, `jsx`, `tsx`; `web_inspect` remains exactly `html`, `css`. The two
public schemas share the same internal scope, snapshot, byte-only transport,
owned-process cleanup, response validation and saved-source recheck. They do
not create a second execution or installation path.

## Request and observed result

The request supplies `expected_release_id`, an explicit absolute `project_root`,
and `request` containing an absolute `path`, explicit `language`, ordinal
`pointer`, nullable `expected_revision`, `start_child` and `limit`. Empty pointer
selects the source root. Every nonempty pointer or nonzero child offset requires
the observed full `sha256:` revision. Pointers are occurrence identities within
that source revision; they are not semantic symbol identities.

The response remains `{ release_id, inspection: { project_root, path, language,
pointer, outcome } }`. A page contains its fixed backend identity
`{ name: "typescript", version: "5.9.3" }`, exact original-source revision and
byte count, a selected syntax occurrence and bounded immediate children. Each
child has a syntax `kind`, ordinal `pointer`, `child_count` and original UTF-8
`location`, or a null location with a reason. Identifier `name` values are
parser-decoded display metadata, including Unicode escapes; they do not imply
binding, type or cross-file resolution.

`complete` describes immediate-child pagination only. `structure_complete`
describes whether this bounded syntax tree has parser diagnostics, missing or
recovery nodes, unknown syntax, or unproved offsets. It does not claim valid
types, valid JS-only grammar, a complete project or executable correctness.

`changed`, `missing_pointer`, `invalid_start`, `budget_blocked` and `refused`
remain explicit outcomes. A blocked metadata item retains its pointer, syntax
kind and proven source span for expansion through `file_read_ranges`. A source
change observed after confirmed child cleanup discards the entire page and all
its locations. This is a revision recheck, not an atomic filesystem snapshot.

## Fixed passive parser boundary

The compiled `code-source-inspect.js` imports the fixed private
`typescript` package. The admitted selection is TypeScript 5.9.3's unchanged
`lib/typescript.js`, package metadata, `LICENSE.txt` and complete
`ThirdPartyNoticeText.txt`. Runtime packaging and inventory are owned by the
release assembly; this module never discovers project or host TypeScript.

Only `createSourceFile` and `forEachChild` are called. The language freezes
`ScriptKind.JS`, `TS`, `JSX` or `TSX`, with the pinned parser's `ESNext` target
and parent-link construction disabled. A constant logical source filename
selects no source file or project model. There is no compiler host, Program,
language service, configuration discovery, import/module resolution, default
library loading, type acquisition, plugin, emitter, package manager or source
execution. Neither project paths nor project configuration reach the Node leaf.

The structure is the parser's deterministic child occurrence order. It omits
comments and most punctuation as separate nodes, includes the end-of-file
occurrence, and retains original `node.pos..end` ranges including leading
trivia. A source root covers all supplied bytes. Empty JSX attributes and array
holes can be legitimate zero-width syntax occurrences. Missing identifiers and
declarations and nodes carrying the parser's direct recovery flag have null
locations and explicit reasons; parser diagnostics are sampled separately.
This does not claim that every descendant or ancestor of recovery is invalid.

Offsets are mapped from the original decoded UTF-16 string to original UTF-8
bytes. BOMs, CRLF, non-ASCII names and astral code points are preserved without
normalization. Surrogate-interior, negative, reversed or out-of-source spans
cannot be exposed. Malformed UTF-8 and NUL-containing source are refused.

## Bounds and verification

Bounds are 1 MiB original input, depth 64, 32,768 syntax occurrences, 100 immediate
children per requested page, 4 KiB per node's encoded metadata, 16 sampled parser
diagnostics, and 32 KiB for the complete structured tool result after the parent
reserves envelope bytes. The shared transport retains its 2 MiB request frame,
64 KiB response frame, 16 KiB counted stderr, finite admission and owned-child
deadlines. Deep input can fail inside the parser before the iterative walk;
parser failure returns a refusal and the process owner retains its deadline.
No partial tree is returned after a parser or traversal quota refusal.

On macOS 26.5.2 ARM64, the build-input guard and locked TypeScript compilation
passed. The focused Node 24.18.1 run passed 33 tests across the code leaf,
existing web leaf and finite transport. It covers all four script kinds,
BOM/CRLF/Unicode byte spans, decoded identifier spelling, valid zero-width
occurrences, malformed recovery, revision-bound selection/paging, metadata,
input/node/depth quotas, fixed language dispatch and protocol refusals. Throwing
filesystem/process sentinels after trusted library import observed zero calls
for the four finite parser cases; this establishes the exercised passive path,
not arbitrary TypeScript APIs or an operating-system sandbox.

Five focused Rust source-contract tests passed, including separate public
language deserialization and exact TypeScript backend/span validation. Three
additional native-worker tests require explicit `MIRKET_TEST_NODE`,
`MIRKET_TEST_BIN` and `MIRKET_TEST_WEB_ADAPTER` selection to exercise their native
paths; without those variables they return early. The real worker corpus now
includes HTML and all four code script kinds. The full Rust source gate passed
493 library tests with three optional native tests ignored; strict all-target
Clippy passed, and the adapter gate passed 74 tests with one unadmitted browser
test skipped.

The disposable four-component macOS ARM64 installed release
`sha256:0380914756adee40e05371ab7e7c901ebeceb281da4c3b01172dd7ec6873ccc3`
passed `doctor`, the actual MCP HTML/CSS/JS/TS/JSX/TSX source smoke using private
Node 24.22.1 and same-binary worker, five-dialect SQL/TOML/XML/YAML/PDF smoke,
and Codex CLI app-server read, policy and managed-registration journeys without
a model turn or user configuration change. The installed source smoke verified
all four script kinds, original Unicode/CRLF byte spans, bounded
selection, revision drift, invalid language, scope refusal and source preservation.
Windows process ownership, actual Codex Desktop model activation and cross-file
semantic navigation remain unqualified.
