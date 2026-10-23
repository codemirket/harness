# Agent runtime

This document describes the Mirket 0.0.1 native agent-runtime package. It is
the current contract for a clean installation; earlier runtime reports are
historical evidence and do not describe supported storage or recovery formats.

The package has three native owners:

| Owner | Responsibility | Durable data |
| --- | --- | --- |
| Rust fact store | Explicit project-scoped facts, lexical/semantic retrieval, expiry, supersession and forgetting | `runtime/projects/<scope>/facts.sqlite` |
| Rust execution journal | Run transitions, immutable assignments, deltas, snapshots, deadlines and uncertain effects | `runtime/projects/<scope>/execution.sqlite` |
| Contained embedding helper | Bounded MiniLM vectors for fact writes and recall | No persistent state |

The fact and journal databases are created only through their versioned native
initializers. They are scoped by the registered project and model identity,
use SQLite WAL with bounded pages, and reject unsupported application IDs or
versions without conversion. Empty stores are valid. The helper is selected
only when the package is enabled and receives fixed model/native assets; it has
no credentials, host project mounts or network access.

## Memory operations

`memory_put` accepts one explicit text, provenance, kind, optional expiry and an
idempotency key. The store writes the text, normalized 384-dimensional vector,
FTS5 row and request fingerprint in one transaction. Repeating the same key and
payload returns the same receipt. A supersession names the observed prior ID
and digest; the compare-and-swap fails if the prior fact changed. `memory_forget`
requires the observed digest, creates a tombstone and removes the active fact;
run traces and backups remain separate evidence.

`memory_read` supports `hybrid` and explicit `lexical` mode. Hybrid retrieval
selects eligible, non-expired, non-superseded facts before ranking lexical FTS5
and exact cosine candidates with deterministic reciprocal-rank fusion. Results
are bounded and retain source IDs, digests and provenance. Retrieval is
context, never instructions, permissions or verification. A query can report
incomplete coverage when objective chunks or result aggregation are omitted; empty results are not a claim of universal relevance.

Worker recall preserves explicit queries exactly. Without them, it samples the
beginning and end of the objective and task-need reasons, reports omitted
chunks, and caps results at twenty. `memory_recall` is `off`, `best_effort` or
`required`; required failures stop before model inference, while best-effort
failures return an explicit unavailable status.

## Recovery operations

`workflow_start` and `workflow_resume` operate on the journal's version-1
envelope. Assignments are immutable; observations are bounded and may be
recalled by identity. Every transition uses a run revision and idempotency key.
Snapshots are written at the initial transition and periodically; intervening
events are bounded JSON deltas with digest and gap checks. State, replay bytes,
event count, pause allowance and cumulative deadline are bounded.

An effect intent is recorded before any external dispatch. After a process
failure it is reported as `effect_uncertain` and cannot be replayed blindly.
File writes can be reconciled from their before/after receipts; other effects
remain blocked until a host verifies them. Host verification is separate from
the journal transition and can revoke a stale completion. A same-key retry
returns the recorded result; a changed payload or checkpoint revision fails.

Diagnostics retain queue/startup, model load/inference, retrieval/lock, checkpoint,
serialization and tool elapsed measurements in the existing trace. Checkpoints
report appended bytes; helper transport reports framed request/response bytes.
Helper resident memory is sampled from Linux `/proc/self/status` after inference,
not attributed to individual queries. Unobservable measurements remain null;
these diagnostics do not claim complete provider or whole-host memory accounting.
Private prompt/fact bodies are excluded from diagnostic summaries.

## Export, backup and support boundaries

`data_export` is a paged logical export with a versioned cursor and source
revision. `data_plan` and `data_apply` provide a bounded, hash-bound deletion
plan for selected facts and completed runs. Fact deletion, workflow deletion,
run traces and backup retention have separate owners.

Backups are same-format native snapshots. Creation holds the maintenance lease,
uses SQLite `VACUUM INTO` for the coordination and native databases, quiesces stores,
validates scope/model identities and publishes a manifest atomically. Restore
rejects unsupported or mismatched native databases before publication and
preserves unrelated opaque evidence. There is no pre-0.0.1 database migration,
dual-write path or compatibility adapter.

## Embedding asset contract

The helper speaks a narrow JSON-lines protocol with bounded requests and no
unknown fields. `identity` returns the package, model and manifest identities;
`embed` accepts up to eight texts, rejects tokenizer overflow instead of
silently truncating, and returns finite 384-dimensional vectors and phase
timings. The model is the pinned multilingual MiniLM revision in
[`runtime/agent/model.json`](../runtime/agent/model.json). Native ONNX files,
sizes and digests are pinned in [`native.json`](../runtime/agent/native.json).

## Installation and diagnostics

Use `mirket setup --target all --yes` to select the package, `mirket doctor` to
check the managed installation, and the package owner to prepare/start its
private engine. A package receipt, executable identity or database file does
not prove that a live host invoked the runtime. Verify a real `memory_read`,
workflow status, contained helper smoke test and same-format backup/restore
before release acceptance. Runtime diagnostics expose phase timings and
redacted failure classes without provider errors, credentials or fact contents.

The native package is currently supported only where the managed Podman
controller and its pinned Linux container are verified. Core CLI portability
does not imply contained-runtime portability.

## Capacity and diagnostics

The initial supported envelope is 10,000 active facts per project. Exact dense
scoring scans every eligible vector even above that envelope and reports the
stress-envelope flag; it does not truncate the store to meet a latency target.
Native database and WAL sizes are accounted directly from the fixed owned files
under the project writer lease. Normal writes reserve bounded capacity below
the 512 MiB ceiling; 64 MiB remains reserved for recovery. Reads and exports
remain available when normal write admission is exhausted. Explicit doctor and
backup/restore perform full structural and semantic validation.

Embedding helpers remain warm for at most 60 idle seconds, with two active
helpers and a bounded waiting room per home. Waiting clients cause idle helpers
to yield capacity early. Cancelled or unhealthy transports are discarded rather
than reused with unread responses. Metadata, lexical reads, lifecycle and journal
operations do not consume an embedding slot. Automatic objective queries may be
fitted to the tokenizer window with an explicit incomplete signal; explicit
queries and fact writes retain the 128-token rejection contract. Timings report
queue, startup, model load/inference, lexical lookup and dense scoring without
recording private text in a diagnostic log.
