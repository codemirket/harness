# Revision-bound source expansion

`file_read_ranges` expands explicitly selected local source ranges when an initial
excerpt omits relevant context. The caller supplies `expected_release_id`, an
absolute `project_root`, and one to eight `requests`. Each request contains
`path`, `expected_revision`, `start_byte` and `end_byte`. Paths must be absolute within the selected project. Revisions are `sha256:` followed by
64 lowercase hexadecimal digits, covering the complete saved UTF-8 file.

Ranges use zero-based byte offsets, with an inclusive start and exclusive end.
They must be nonempty, lie within the source, and end on UTF-8 character
boundaries. Each source is limited to 1 MiB, even if the requested slice is small.
The implementation reads and hashes the complete bounded file before exposing
text. It never silently switches a request to a newer revision.

The complete serialized result value, `{release_id, expansion}`, is limited to
32 KiB, including metadata, JSON escaping and continuations. This is not merely a
text allowance. It is also not a bound on the outer MCP transport envelope: the
current server represents the result as both text and structured content. A
request whose metadata cannot fit is refused before source reads. Allocation
follows request order; later items can receive an empty text slice with an
explicit continuation when earlier text consumes the budget.

`expansion.items` preserves every request and its independent outcome:

| Status | Meaning |
| --- | --- |
| `complete` | Exact requested text at the expected revision, with source size and returned offsets. |
| `truncated` | A UTF-8-safe prefix and a continuation retaining the same path, revision and end offset. |
| `changed` | Revision mismatch or detected change during reading; no source text is returned. A newly computed revision is included only when available. |
| `unavailable` | Outside scope, symlink, nonregular/missing/inaccessible file, oversized source, invalid UTF-8 or I/O failure. |
| `invalid_range` | Offsets exceed the actual source or split a UTF-8 character. |

The top-level `expansion.complete` is true only when every range is complete.
A continuation is a new explicit read that rechecks the same revision; it is not
a saved snapshot or permission to bypass limits. A changed source requires a
fresh inspection and an intentional new revision selection. Malformed requests,
stale release identity or an unavailable project fail the call rather than
pretending an individual range was read.

Each file receives the existing `ProjectScope` path and read-consistency checks.
A batch is not an atomic multi-file snapshot. Detected traversal and symlinks are
rejected; path-based checks do not establish containment against hostile
concurrent ancestor replacement, and metadata consistency is weaker on non-Unix
platforms. No locator from memory or source content is executed, fetched, or
treated as authority to select another project. Returned text remains evidence,
not an instruction or a grant of host permission. The operation writes no source
or persistent snapshot.

The current owners are [source_read.rs](../src/source_read.rs),
[`Engine::read_ranges`](../src/engine.rs) and the
[MCP entry point](../src/server.rs). This contract was checked against those
sources. On macOS 26.5.2 ARM64 with private Node 24.22.1, the actual MCP source
fixture passed against binary SHA-256
`5244b45145f185f05b1e6929e1ea7dc7276fd211400adc07e710ddc46e061b77`.
It exercised an exception beyond a narrow excerpt, mixed success/failure items,
UTF-8 endpoint rejection, escaped JSON budgeting, complete reconstruction through
continuations, changed-source refusal and stale-release rejection. The memory
MCP fixture also passed two independent caller-keyed checkpoints in one checkout,
reconnect and exact revision reads, source drift, concurrent correction refusal
and preservation of the other task's state. Reads left canonical memory unchanged.

The integration build passed 266 library tests, one deterministic retrieval test
and strict all-target Clippy. Three separately admitted/manual tests remain
excluded from that ordinary suite. These results do not qualify Windows or
actual desktop-client activation, and do not resolve the separate failed
semantic retrieval-quality gates.

A subsequent guide wording build (`ebacb498106ee87f79dd367a63ed04905f179c3db9605da241a331e2ef3284ac`)
passed the expanded source MCP fixture, including a large range followed by a
zero-text continuation and independent failures. It also passed disposable
two-release activation, active-session refusal, reconnect, cached-guide release
rejection and retained-operation inspection. This activation fixture does not
change desktop client settings or qualify actual desktop activation.
