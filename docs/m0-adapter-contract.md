# Historical M0 private text adapter contract

This 2026-10-24 qualification describes a removed adapter. The current release
has no generic text replacement job, public file read or `adapters/main.js`.
Codex owns general file read/edit; this record is retained as dated evidence.

This finite fixture qualifies the private Rust → Node → staged artifact path.
It is not an Office engine, PDF editor, shell, public MCP server or sandbox.
The Rust owner supplies an explicit private Node executable and compiled adapter
path; the adapter never locates another executable, starts children or downloads
dependencies. Host permissions remain authoritative.

## Process and framing

Launch the compiled `adapters/dist/main.js` with `--release-id FULL_ID`.
The identity is an opaque 1–256 character ASCII token (`A-Z a-z 0-9 . _ : -`).
The adapter emits one ready message, accepts one UTF-8 JSON request terminated by
LF followed by stdin EOF, emits one result or error, and exits. Stdout is only
newline-delimited JSON; stderr never includes document text. Each input/output
frame is at most 65,536 UTF-8 bytes, including LF. Input must finish within five
seconds. One invocation performs at most one operation; this is no persistent
service or general language evaluator.

```json
{"type":"ready","protocol_version":1,"release_id":"m0-fixture","operations":["text.replace_once"],"max_frame_bytes":65536,"max_input_bytes":1048576}
```

```json
{"type":"run","protocol_version":1,"release_id":"m0-fixture","job_id":"job-1","input_path":"/absolute/input.txt","output_path":"/absolute/stage/output.txt","operation":"text.replace_once","find":"old text","replace":"new text"}
```

Every key is required and additional keys are rejected. Job IDs follow the same
token grammar as release IDs. Paths are absolute, at most 4,096 characters and
contain no NUL; the source and output must differ. The output's existing parent
directory must be a real directory. Rust validates scope and assigns a private,
owned staging directory; this adapter is not a filesystem containment boundary.
The source must be a regular, non-symlink UTF-8 text file up to 1,048,576 bytes.
An optional UTF-8 BOM and all untouched text, including line endings, survive.
Malformed UTF-8 and NUL-containing input are rejected. `find` is nonempty and
`find`/`replace` are at most 16,384 UTF-8 bytes each. Literal matches, including
overlapping matches, must occur exactly once. Zero or multiple occurrences fail
before creating an output. Output must also fit the one MiB limit.

```json
{"type":"result","protocol_version":1,"release_id":"m0-fixture","job_id":"job-1","status":"staged","output_path":"/absolute/stage/output.txt","input_bytes":18,"output_bytes":18,"replacements":1}
```

Success means the exclusive new staging file was written, synced and closed;
it does not mean publication, source preservation under concurrent edits, user
acceptance or semantic correctness. Rust rechecks source/destination identity,
validates the saved result, records its commit boundary and finalizes the file.
It owns receipts, response-loss reconciliation, cancellation state and cleanup.
Existing destinations (including symlinks) are never overwritten by this adapter.

```json
{"type":"error","protocol_version":1,"release_id":"m0-fixture","job_id":null,"code":"invalid_request","message":"Request fields are invalid.","output_may_exist":false}
```

Errors have exit status 1; success exits 0. Error codes distinguish
`invalid_startup`, `invalid_frame`, `request_timeout`, `invalid_request`,
`release_mismatch`, `invalid_source`, `invalid_text`, `match_count`,
`output_too_large`, `output_exists`, `io_error`, and `cancelled`. Messages are
bounded fixed text, not backend error strings. `output_may_exist` becomes true
after exclusive creation and remains true on write, sync, close or subsequent
signal failure; inspect the stage rather than replaying an uncertain operation.

SIGINT/SIGTERM request best-effort cancellation, stop stdin and prevent subsequent
write steps where observable. They cannot roll back a completed write. No child
processes are created; Rust must supervise this process, bound stdout/stderr and
terminate/reap its owned process on deadline, disconnect or cancellation. Abrupt
kill may leave a partial stage without any terminal message. Platform process
ownership and native-client lifecycle remain Rust/integration qualification gates.

## Development checks

From `adapters`, compile using the admitted local development compiler with
`./node_modules/.bin/tsc -p tsconfig.json`, then run
`node --test tests/*.test.mjs`. The package's runtime has no third-party
dependencies. Compilation uses the locked TypeScript compiler and official
`@types/node` development declarations.
The tests launch the compiled adapter and inspect real disposable files.
Compilation and these tests do not qualify a private distributed runtime,
Windows behavior, native host activation, Office fidelity or PDF rendering.
