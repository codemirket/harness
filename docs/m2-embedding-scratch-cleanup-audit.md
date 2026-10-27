# Embedding scratch cleanup audit

Audited on 2026-10-26 on macOS 26.5.2 ARM64. The current implementation removes
session scratch during awaited server shutdown. Terminating the server before
that cleanup owner finishes can retain an empty directory even when the separate
supervisor successfully reaps the model and releases admission. A controlled
SIGTERM probe reproduced that boundary. No production code was changed.

The historical Codex run did not capture its Mirket server's exit status or
signal. Its residue is consistent with this boundary; the exact historical
termination mechanism remains unknown. No graceful cleanup defect was reproduced.

## Original observation

The [read-only Codex memory trial](m2-codex-model-memory-qualification.md) left
`state/jobs/op-embed-18e1e7cc63fd18a0-9ae0-1` in its disposable Mirket home.
The preserved directory has device **16777230**, inode **308450859**, UID **501**,
mode **0700**, and no entries. Its creation time was
**2026-10-25T22:56:31.708616Z**, during the model turn. The name encodes server
PID **39648**. The Codex process exited **0** after **21.38 s** without the
evaluator's timeout/termination path being invoked. That client exit does not
establish how its child server exited.

The audit preserved the original directory and its identity. It did not rerun the
Codex model trial, change the user's installation, or adopt any retained path for
cleanup.

## Ownership and cleanup

The implementation assigns distinct responsibilities:

1. [MemoryRuntime](../src/memory_runtime.rs) reserves one session directory with
   `OwnedScratch::create` when a cold session starts. Request completion leaves
   the warm session resident. Its `Drop` implementation signals shutdown;
   awaiting `shutdown` is required to wait for cleanup.
2. [OwnedScratch](../src/owned_scratch.rs) records the creator's state, jobs and
   scratch directory identities. The value moves into the server's
   `embedding_session::parent_owner` task. Dropping the value never deletes it.
3. [The supervisor](../src/embedding_session.rs) receives the scratch path in
   `ConfigWire`. It owns the native leaf and durable heavy-slot marker. Parent
   pipe closure starts leaf termination/reaping and marker cleanup. The
   supervisor has no `OwnedScratch` value and performs no scratch deletion.
4. The parent owner requires a valid terminal cleanup report, successful
   supervisor reap and the expected terminal stream EOF. It then calls `remove_empty`, which
   rechecks the recorded directory identities, refuses nonempty directories,
   removes only that directory and flushes its parent.
5. Both the [development server](../src/main.rs) and
   [installed deferred server](../src/deferred_server.rs) await server shutdown
   after MCP transport completion. Killing the process or ending its Tokio
   runtime can prevent the parent cleanup task from reaching step 4.

This follows the existing [embedding lifecycle contract](m2-embedding-lifecycle.md).
The explicit source comment in `memory_runtime.rs` requires awaited shutdown and
describes the separate supervisor lifeline for abrupt process/runtime loss.
Process absence and a path name do not reconstruct the lost creator stamps.
Removing retained paths on that evidence would expand the cleanup contract.
Moving stamped cleanup ownership to the supervisor would require a separately
designed handoff; no such change is made by this audit.

## Controlled process checks

Two fresh private state roots copied the trial's synthetic identity, canonical
memory and owning lifecycle state. Each exercised one actual MiniLM-backed
`memory_search` through MCP using `dev serve`, then observed one server, its
supervisor, one model leaf, one resident marker and an empty stamped session
directory. Both searches reported semantic availability with no missing eligible
records. No generative model turn was involved.

The probes used the trial's exact installed core executable, SHA-256
`75e557a48ee52559b9811bd6796e1133d5821ed6f0a119e000bc1b6a7f3d4239`,
its explicit private Node/adapter/MiniLM paths, and macOS network-denial profile
`(version 1) (allow default) (deny network*)`. The profile was applied to each
server subtree; this was not a packet-capture test.

| Deliberate stop | Server exit | Scratch after stop | Supervisor and leaf | Admission after stop |
| --- | --- | --- | --- | --- |
| Close MCP stdin and await exit | 0 | Removed | Absent at inspection | No unresolved markers; both heavy-slot locks acquired independently |
| SIGTERM to the exact owned server | Signal 15 | Same empty directory, inode 308488792 | Absent at inspection | No unresolved markers; both heavy-slot locks acquired independently |

The respective post-stop observation intervals were **0.239 s** and **0.210 s**.
These are single observations, not latency acceptance measurements. Canonical
memory and identity bytes in each copied state remained unchanged. The
SIGTERM residue remains preserved as a second controlled specimen.

These checks establish the current graceful-versus-terminated parent boundary
on this core and platform. They do not prove which signal, runtime event,
filesystem error or shutdown path occurred in the historical Codex process.
Capturing a future child's actual exit/signal and shutdown report would be
needed to distinguish those possibilities. No Windows or desktop GUI activation
claim follows from these probes.

## Verification and retained evidence

`python3 scripts/check-build-inputs.py` passed before compilation.
`cargo test --locked --offline --lib memory_runtime::tests -- --nocapture`
passed **9/9**. With an explicit `MIRKET_TEST_NODE`, the session group passed
**11/11** using `--test-threads=1`. The initial concurrent session run passed
**10/11**: `dropped_start_future_still_reaps_and_removes_scratch` exceeded its
three-second wait. The same test passed in isolation and in the serial group.
That initial failure and its empty fixture are retained; its exact timing cause
was not established and the later runs do not erase it. No test deadline or
production cleanup behavior was changed.

An initial process fixture omitted the memory lifecycle directory and received a
search error. Its report is retained. The corrected fixtures copy all three
owning state directories and passed the checks above.

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/embedding-scratch-cleanup-audit-20261026/`:
`inputs.json`, `probe.py`, `v2-graceful-eof.json`, `v2-sigterm-parent.json`,
`original-preserved.json`, the initial failed fixture/report, focused test logs,
verification metadata and evidence hashes. The original trial's evidence remains
in its separate `memory-codex-model-20261026/` directory. This audit changes only
this document in shipped source.
