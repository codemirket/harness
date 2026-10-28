# M2 canonical and lexical memory verification

`tests/memory-smoke.mjs` exercises the actual Rust stdio MCP server with Node
built-ins and an explicitly supplied private runtime. It creates disposable
private state and project directories, changes no desktop configuration, and
removes its own fixtures and processes on completion.

## Reproduce

Build the Rust binary through the admitted SQLite build guard, then run with
absolute paths:

```sh
/absolute/private/node tests/memory-smoke.mjs \
  --mirket /absolute/mirket \
  --node /absolute/private/node
```

The fixture uses `mirket dev serve` with explicit state, release and Node paths.
It negotiates MCP `2025-11-25`, checks the current public memory surface including
`memory_index_refresh`, and verifies saved-source revisions without a generic
file-range MCP tool. It bounds frames to 1 MiB and RPC waits to five seconds, and prefers structured
results over JSON text fallback. No package installation or implicit executable
lookup occurs in the fixture.

## Observed passing run

The following is historical 2026-10-24 evidence for the former adapter-bearing
surface. Both `tests/memory-smoke.mjs` and the existing `tests/mcp-smoke.mjs` passed on
2026-10-24 at approximately 18:07 UTC, on macOS 26.5.2, ARM64, using the assembled
private Node **24.22.1**. The Rust debug binary linked the admitted SQLite 3.54.1
source; each canonical-store open verifies its exact source identity, FTS5 and
omitted extension loader.

| Tested input | SHA-256 |
| --- | --- |
| Rust debug binary | `62c50cb0762b8c22c202ae0dd35a6554a95966bc651b46b06feaeeae148c4149` |
| Private Node binary | `6f32069480ec14d52b6d7b1f3c7124cfbbc46203abf5ddf5c65b127435f3f329` |
| Compiled text adapter | `3ea3e1a548b5be6b70ece6487c7ce34f05ead1b8ecf1e824eabf8a4637f8be9e` |

These hashes identify that run; later builds require affected checks again.

The expanded prepared-target receipt regression in `tests/memory-smoke.mjs` also
passed on 2026-10-24 at 18:17 UTC with the same private runtime and adapter, using
Rust binary SHA-256
`593b14b608d25c2beff5eea40b6e79a3d2d611c345a17cbcf02310e01a7dca85`.
All eleven memory journey groups passed. The text fixture was not rerun against
this later binary.

The memory journey established:

- Unregistered read-only searches/reads and stale-release preparations leave
  identity and memory storage absent and the state snapshot unchanged.
- Preparation reserves an intent without creating the canonical database.
  Another live connection cannot commit that intent or change its receipt.
- Concurrent and repeated same-ID commits create one record. The fixture permits
  an explicit transient busy error, never a second effect, and inspects the
  resulting canonical receipt.
- Competing corrections prepared against revision one cannot both succeed.
  The accepted revision is current, its original source-backed history survives,
  and old content disappears from current lexical search.
- A deliberately discarded commit response is reconciled by another connection,
  followed by abrupt owner exit and reconnect. The saved identity survives;
  the new connection cannot replay the old owner's operation.
- Current/unresolved and temporal filters remain explicit. Applicable critical
  constraints bypass lexical rank; omitted applicability context and count/byte
  overflow are reported. Serialized recall stays within its requested budget.
- Unknown source URI schemes remain literal provenance data.
- Shared preferences require their own explicit scope and preference record
  type. Independent project records remain isolated.
- A minimal, inspected Git worktree metadata fixture shares repository-wide
  knowledge while keeping task progress, exact reads, history and corrections
  confined to the selected checkout. No Git command, hook or repository helper
  executes during this fixture.
- Forgetting requires its own commit tool and owning connection. Reconciliation
  cannot execute a merely prepared forgetting intent, including when a different
  operation has already forgotten the same target. A correction after forgetting
  preparation invalidates that stale intent.
- Successful forgetting hides all three retained target revisions from exact
  reads, history and search, removes its three consumed save-operation receipts
  and three prepared target-mutation receipts (including pending revision and
  supersession). Later commits using removed IDs fail unavailable. Unrelated
  prepared mutations, uncommitted forgetting decisions, records and scopes
  survive. Exact forgetting-operation status
  remains inspectable after reconnect, and committed cleanup can be reconciled
  explicitly without creating a new forgetting decision.
- A newer server release rejects cached release IDs for searches/preparations
  without filesystem effects, while scoped save/forget status inspection remains
  available. Stale-release forgetting preparation and reconciliation are also
  refused before filesystem effects.

The text-artifact fixture also passed against the new Engine cleanup implementation,
including its original prepare/start/cancel,
source-drift, destination-conflict, cross-owner, revision-inspection and
response-loss checks with the expanded nineteen-tool inventory.

Before integration, `cargo test --locked --lib memory` passed sixteen actual
SQLite tests covering canonical isolation, read-only opens, revision CAS,
history/supersession, literal FTS syntax, critical coverage, privacy checks,
integer bounds and atomic mutation receipts, including unavailable targets.
The memory guide's tool names and scope syntax were inspected against the wired
server schema. The optional skill-creator validator could not run with the system
Python because PyYAML is absent; no dependency was added for that check.

## Qualification limits

This establishes the canonical/lexical foundation through real MCP stdio, not
full M2 completion. The configured backup/restore caller journey passed as
recorded below. Semantic quality qualification, automatic source freshness and
native desktop client activation still require separate evidence. The frozen retrieval runs recorded in
`docs/memory-evaluation-contract.md` failed their accepted quality gates. This run does
not qualify Windows, power-loss durability, interrupted native SQLite recovery
or arbitrary hostile filesystem replacement. Stored source revisions retain
provenance; they do not automatically validate facts after a branch change.

The separate integration review records the response-loss/concurrency findings
and their source fixes in `docs/m2-memory-integration-review.md`.


## Configured runtime journey (passed after cleanup repair)

The optional `--embedding-package /absolute/verified/package` argument reuses the
existing bounded MCP client and adds a separate disposable configured state,
project and managed backup directory. Calls that may initialize the native model
have an explicit 90-second fixture timeout; ordinary canonical calls retain the
five-second timeout. This mode must run only during coordinated model admission.

The authored journey checks: two bounded index pages from previously unindexed
canonical records; eligible real SQLite snapshot contents via read-only Node
builtin SQLite; committed and corrected revision indexing; multilingual recall
using configured resources; obsolete and forgotten revision exclusion; removal
of affected managed snapshots; observable lexical fallback with an unavailable
payload; preservation of canonical save and backup when indexing is pending;
and explicit rebuild after reconnect. It then stops configured MCP sessions and
uses the actual `mirket dev memory` backups/restore-prepare/status/commit/reconcile
CLI, verifies a prior canonical revision returns while forgotten content remains
absent, and rebuilds invalidated derived index coverage explicitly. The CLI is
given canonical existing private roots as its contract requires.

Three actual runs used the explicit private Node 24.22.1 and approved small-model
package on macOS ARM64, with `ORT_DISABLE_TELEMETRY=1` and the OS network-deny
wrapper. The tested core SHA-256 was
`72c91ee1be35de7360650dd95b6ecdf4a5f4e3018e85231091709528c8a5872b`.
The initial run stopped at an unobserved discarded-response commit in the canonical
baseline. It did not establish that the dropped invocation had committed. The
two diagnostic runs passed that baseline and reached the same configured-runtime
failure; diagnostics were expanded without relaxing assertions or timeouts.

Both diagnostic runs verified bounded index pagination, real eligible SQLite
snapshot contents, indexed save/correction, multilingual candidate recall,
currentness, forgetting and affected snapshot removal. They also verified lexical
fallback and a durable canonical save plus eligible backup while indexing was
pending. Reconnecting with the original valid model package could not rebuild
the missing revision: the response reported `status:"pending"`, one missing head,
and `embedding_unavailable` with `both heavy-job slots are busy`.

Independent reproduction with an unavailable package found the earlier cause:
`embedding cleanup unconfirmed: owned process-group termination failed:
Operation not permitted (os error 1)`. A valid leaf failure reply was followed by
a failed termination attempt, leaving both unresolved ownership markers. The
owner corrected that finite error-reply/reap path and passed a targeted regression
with two unavailable-package requests and a subsequent real text job. The later
complete journey below verifies the integrated repair. The final diagnostic run took 62.25
seconds. All fixture processes stopped and the disposable state was removed.

Those failed diagnostic runs did not reach the restore CLI or post-restore
index rebuild section. The private evidence directories are
`memory-configured-mcp-20261024T185715Z`, its `-diagnostic` sibling, and its
`-diagnostic2` sibling under the implementation task-state root; they retain input
hashes, stdout and stderr. This is an integration/currentness fixture, not a
replacement for the failed frozen multilingual quality gates recorded in
`docs/memory-evaluation-contract.md`.


The subsequent complete run passed against core SHA-256
`6b69e460c08cbb1267d453ad935d07b003c582bce2a95293d8908cecccea426d`,
with the same explicit private Node/model package, telemetry disabled before
load, and OS network denied. Its fixture SHA-256 was
`550606149b29eb93c76141ad0530296f690ef3f6cf999dbe66ff82d914f72363`.
The canonical baseline and every configured assertion passed. Three unavailable
payload calls returned the bounded `embedding_payload_invalid` failure; a later
connection using the original verified package rebuilt the missing revision
without replaying canonical saves. Actual maintenance CLI backup inspection,
restore preparation, exact-operation status, commit and reconciliation all
passed. The previous canonical revision returned, forgotten content stayed
unavailable, recall reported invalidated derived coverage, and explicit index
refresh rebuilt that coverage from current canonical records. The final private
job directory was empty. All fixture sessions stopped and disposable state was
removed; no fixture/MCP/native model processes remained.

This full journey exited zero in 113.40 seconds (99.55 user, 8.50 system), with
reported maximum resident set size 1,275,953,152 bytes. This is a multi-process
fixture measurement, not a per-request latency or retained-server memory claim.
Evidence is retained in the private implementation task-state directory
`memory-configured-mcp-20261024T190724Z-cleanup-fixed`, including exact input
hashes and complete stdout/stderr. The core hash was checked again afterward
and was unchanged. This passing integration run does not supersede either failed
retrieval quality report or qualify native desktop client activation.

## Source expansion and task continuation

The revision-bound [source expansion integration](retired-evidence.md#retired-docs-m2-source-expansion-md) passed
on macOS ARM64 with the existing memory fixture, including its configured native
model/backup/restore journey. Two caller-selected task keys in the same checkout
resume independently after reconnect. Exact checkpoints preserve source revisions,
plan pointers, distinctions between observation/inference/proposal, and unknown
effects. A changed source returns no text; read-only resume changes no canonical
state; competing progress corrections cannot overwrite a newer revision.

The configured journey took 24.735 seconds on the unoptimized integration build;
this is a fixture duration, not a retrieval-performance or quality benchmark.
