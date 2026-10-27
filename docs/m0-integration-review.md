# Independent M0 integration review

Reviewed the first integrated `engine.rs`, `server.rs`, `operation_store.rs` and
`job_state.rs`, following publication into `artifacts.rs` and supervision into
`process.rs`. This is a review of the finite text slice, not a demand to implement
the remaining toolchain. The initial findings below were sent immediately to
their owners; files were being integrated concurrently. Line numbers identify
the inspected version, and symbols identify the affected boundary after edits.

## Material findings

### P1 — Unconfirmed child cleanup released admission

The initial `process.rs:195–198` returned an ordinary error when termination or
reaping was unconfirmed, with a message saying to retain admission. The initial
`Engine::start` worker (`engine.rs:108–114`, later 296–303) dropped its `_lease`
on every error. A reap timeout or termination failure therefore reopened a heavy
slot while its previous child could still run. This is a concrete ownership
failure on an existing error path, separate from the explicitly unqualified
abrupt-parent-death scenario.

Small complete fix: return typed cleanup knowledge and keep the slot blocked
when stopped ownership cannot be established. Preserve an inspectable recovery
record before the owning process can exit; retaining only an in-memory lease
does not survive that exit. Inject cleanup failure and prove another owner
cannot acquire the affected capacity. The process owner has introduced typed
`AdapterError.cleanup_confirmed`. Engine now marks its slot unresolved before
launch and clears that durable marker only after confirmed cleanup
(`engine.rs:350–367`); the updated admission code retains unresolved markers
across lease drop. This addresses the identified release path by inspection.
The process/admission owner's cleanup-failure probes and integrated verification
remain the evidence needed to close it.

### P1 — Another connection could cancel a prepared operation and overwrite its transition

The initial `Engine::cancel` (`engine.rs:89–95`) accepted any same-release,
same-project receipt. It ignored whether removal from this connection's pending
map actually found an operation. A second connection could therefore cancel the
first connection's prepared intent. Because `load` and `save` took separate store
locks, its stale prepared snapshot could also overwrite a concurrent start's
running receipt. The published `job_cancel` description promised an owned
operation, so this was a defect in the current surface.

Small complete fix: bind mutable operation handles to a connection identity and
serialize permitted transitions. Keep receipt inspection available across
connections/releases without granting mutation. The root has now added
`Receipt.owner_id`, an owner lock, owner checks in `cancel`/`start`, and read-only
interrupted-state reporting when an owner is gone (`engine.rs:223–274` in the
reformatted revision). This closes the identified cross-process mutation path
by inspection. **Verified in the actual stdio smoke:** two simultaneous servers
share the same state; owner B reads the prepared receipt but its cancel/start
calls fail without changing receipt or fixture bytes, and owner A then completes
the original operation successfully. The current-thread
runtime and non-awaiting start/cancel handlers serialize their local transition
steps; a later multithreaded Engine entry point would need an explicit
per-operation transition guard rather than relying on that runtime constraint.

### P1 — Receipt capacity could prevent confirmations of already admitted work

The initial `reserve_bytes` counted existing files and rejected only when their
count was already greater than `MAX_RECORDS`. `issue` could add one beyond the
limit, after which every update failed. Independently, new small prepared
receipts could consume space needed to grow or atomically stage a running
operation's confirmation. `Engine::execute` saves unknown effect before
publication and confirmation afterward, so this can strand a published result's
durable evidence. It is not fixed by refusing the next new job after exhaustion.

**Fixed in this review:** `operation_store.rs:156–224` now reserves the full
64 KiB allowed receipt size per issued record plus one 64 KiB replacement stage
under the 50 MiB aggregate bound. This permits at most 799 records at the current
byte defaults; the 4,096 record limit remains an independent ceiling. New issues
consume reservation; existing updates reuse it. Peak actual bytes are checked
separately. Unknown files, leftover stages and oversized/aliased records block
mutations while existing direct receipt reads remain available.

Seven targeted operation-store tests pass on macOS ARM64, including four new
small-limit cases: the byte reservation boundary with full-sized confirmations,
the exact record boundary with continued updates, unknown/temporary entries
preserved for inspection, and an oversized existing record. Command:
`cargo test --locked operation_store::tests -- --nocapture`. Physical disk failure,
unexpected state corruption and failed syncs remain possible; reservation does
not turn persistence into an infallible operation.

## Checked boundaries without a new defect finding

- `ArtifactPlan` checks the selected source and destination before staging and
  again before publication. `StagedArtifact::publish` uses no-clobber hard-link
  creation, then verifies the actual saved bytes. No additional ordinary stale-
  source or overwrite defect was found. Its documented hostile ancestor-swap
  and filesystem durability limits remain qualification gaps.
  The expanded actual stdio smoke also passed source-change-after-prepare and
  destination-created-after-prepare cases, preserving newer bytes, plus saved
  artifact revision rejection after an independent edit.
- A missing/evicted receipt cannot authorize a fresh start: Engine loads the
  issued record before execution, and the store has no caller-ID creation API.
  The subsequent [receipt-retention implementation](m1-receipt-retention.md)
  adds age and capacity eviction after explicit typed-owner finalization.
- A duplicate started/terminal ID returns the known operation; prepared intent
  cannot migrate to a new connection merely because its receipt is readable.
- The selected local rmcp 3.7.0 source already bounds stdio ingress. Tuple stdio
  transport uses `AsyncRwTransport::new`, whose `DEFAULT_MAX_LINE_LENGTH` is
  16 MiB (`rmcp-3.7.0/src/transport/async_rw.rs:58,97`). Its public
  `with_max_line_length` method permits a smaller bound. The lower-level codec's
  unbounded default is not the receive route used by the current `main`.
- The child protocol has a five-second ready deadline, a 60-second overall
  child deadline and bounded stdout/stderr. Those timers do not establish a
  deadline for synchronous source reads, staging/fsync, publication or receipt
  writes. The current single-thread runtime can also delay tool responses while
  doing those synchronous operations. End-to-end operation/RPC timeout and
  cancellation latency remain unmeasured; do not advertise the child deadline as
  a bound on the full artifact operation.

Actual desktop activation, full crash-point reconciliation, parent-death child
cleanup and Windows ownership remain declared M0 qualification gates. They are
not presented here as newly discovered support regressions. The production
claim must remain narrower than those unexecuted paths.

The expanded stdio smoke passed on macOS 26.5.2 ARM64 with Node 24.18.1 and MCP
2025-11-25. See [the verification record](m0-verification.md) for run identities;
later integration changes require affected checks again.
