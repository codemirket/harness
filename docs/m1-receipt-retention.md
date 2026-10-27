# Operation receipt retention

The operation store keeps one private JSON envelope per server-issued ID. The
envelope contains the exact UTF-8 receipt payload and store-owned retention
metadata. Schema 2 also retains a SHA-256 fingerprint of the original issued
payload through updates and finalization. Typed owners use this to detect
changed preparation bindings; it does not establish effect completion or
authenticate hostile replacements of private state. The store does not infer
recovery state from the payload's status strings. There is no reader for earlier
qualification-only receipt formats.

New receipts are protected. Their typed owner can explicitly finalize them only
after its effect and recovery obligations are resolved. Finalization records the
completion time once. Repeated identical writes are idempotent; a finalized
payload cannot be changed, reopened or have its retention clock renewed.

The current owner boundaries are:

- Text: a terminal, known effect and confirmed owned scratch disposal, or an
  atomic cancellation of a prepared job that never started.
- Memory save: the canonical SQLite mutation committed and its result is known.
  An optional backup or derived-index failure does not undo that commit.
  An explicit repeated commit can repair failed receipt finalization using the
  same available canonical mutation, including after reconnect. It does not
  rerun the mutation, backup or indexing; ordinary status remains read-only.
- Forgetting: the durable exclusion and all managed cleanup completed. Missing
  backup access keeps the receipt protected; successful explicit reconciliation
  can finalize it.
- Restore: canonical completion is confirmed and no matching pending recovery
  binding remains. A `complete` phase alone is insufficient.

Text and memory owners also retire their own unused preparations at expiry or
graceful shutdown. The still-held in-memory intent proves that no effect was
started; durable identity must match that original intent. Retirement records
`cancelled` and permits only an identical confirmation retry. Unknown or changed
evidence stays protected. A missing memory receipt can discard an unused local
intent, but cannot recreate evidence or establish external cleanup.

An explicit [maintenance retirement](m1-interrupted-preparations.md) also handles
expired text and memory preparations after abrupt owner death. It validates the
original issuance and absence of contradictory execution or memory authority;
it does not reclaim started work or unresolved ownership.

Simultaneous graceful shutdowns may contend for the receipt lock. Shutdown
retirement waits at most one second for native lock contention before any receipt
inspection or finalization. Ordinary tool calls still return busy immediately;
other OS errors propagate without retry. A timeout leaves protected evidence and
does not authorize replay. Empty text retirement does not acquire the store lock.

Memory shutdown closes mutation admission before retiring preparations. Already
admitted canonical writes finish first; derived publication checks admission
again after inference. No mutation guard is held across a model await. Forgetting
also removes cancelled correction/supersession receipts for the exact forgotten
target, including their content-derived intent digests.

Completed receipts remain inspectable for up to seven days, subject to earlier
capacity eviction. An expired read reports unavailable evidence without writing
or deleting anything. Protected receipts do not expire. New issuance validates
the entire bounded inventory under the permanent exclusive lock, removes expired
completions, then selects the oldest remaining completions if capacity requires
it. Corrupt, unsupported, unexpected or interrupted replacement entries prevent
mutation; they are not guessed to be garbage.

The 50 MiB budget reserves a full 64 KiB on-disk envelope per issued receipt plus
one full replacement stage. Escaping and completion metadata count toward that
64 KiB limit. Capacity required for an already-issued protected operation is
reserved before later preparations can consume it. If protected records exhaust
capacity, new issuance refuses; existing inspection and recovery remain
available. There is no permanent per-operation tombstone store or cleanup daemon.

Only issuance allocates a new ID. Start, update, finalize, status and
reconciliation require existing issued evidence. An expired or evicted ID never
becomes a fresh invocation. Saved user artifacts are outside receipt retention.

## Verification

After unused-intent retirement and shutdown admission were integrated, the
native-enabled suite passed 244 library tests and one deterministic retrieval
fixture, with strict all-target Clippy. Three manual tests remained ignored;
the model package was not configured in Cargo. Binary SHA-256
`f148907a760305ee8620c1d51b3ddc89812d2bb07410216e0f11eb1b1a097404`
then passed the real text MCP journey, two-release activation, and configured
memory MCP/restore journey on macOS ARM64 with private Node 24.22.1. The memory
journey denied network access and disabled telemetry before model initialization.
It covers functional behavior, not the failed retrieval-quality gates or scale
performance. The actual 60-second queue deadline also passed separately.

Retirement regressions cover 64 unused memory intents, exact original text and
memory identity drift, lost finalization acknowledgement, unknown effects,
forgotten cancelled targets, no registration after shutdown and refusal of
late derived publication. Earlier store qualification below retains its inputs.

On macOS ARM64, the 15 store tests cover explicit immutable finalization, exact
UTF-8 round trips, encoded-byte accounting, age expiry without read-side writes,
early oldest-completion eviction, protected-record pressure, full inventory
validation before any deletion, and rejection of reused missing IDs. The
combined library run passed 226 tests, with two explicit native/model tests
ignored. Runtime-dependent tests in that first combined run were not enabled;
separate actual private-worker and MCP runs are recorded in the integration
verification documents.

Typed-owner tests verify that finished text/save/restore receipts cannot be
reopened, retained text recovery remains protected, and missing managed backup
access prevents forgetting-receipt retirement until reconciliation succeeds.
Text cleanup flushes the checked jobs parent after removing the private directory
before allowing receipt retirement; an unconfirmed flush preserves protection.
The canonical save test also restores its pre-confirmation receipt bytes to
simulate lost convenience confirmation: SQLite authority still reports exactly
one committed mutation, without replay.

Unused restore and transfer preparations have explicit typed maintenance
retirement commands. Those commands check their corresponding recovery
authority; process absence alone is not cleanup evidence. Unresolved recovery
and stale owner-file sweeping remain unfinished. Native Windows filesystem and
power-loss behavior remain unqualified.

The later macOS integration on core
`d115785773dde1c23eac0d00b7a6bc0bd7880746c507ad6811b9fa1e9ef03b49`
passed 331 library tests, strict Clippy and the actual configured memory journey.
Three new native store cases cover transient contention, the one-second refusal
and immediate non-contention errors. Three actual MCP clients with unused
preparations closed concurrently, exited successfully and left cancelled
receipts without canonical writes or replay.
