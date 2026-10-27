# Retiring interrupted preparations

An abrupt client exit can leave a protected receipt for an operation that was
prepared but never started. The explicit `dev intent-retire` maintenance command
handles one such expired receipt. It does not scan jobs or replay work.

For an artifact preparation:

```sh
/absolute/mirket dev intent-retire \
  --state-dir /absolute/disposable-state \
  --release-id the-issued-release \
  --kind artifact \
  --project-root /absolute/project \
  --operation-id the-issued-operation
```

For a project-memory preparation use `--kind memory` with the same explicit
project selection. Shared preferences use `--kind memory --shared-preferences`
instead of `--project-root`. This development command does not select the user's
existing installation implicitly.

All clients and workers using that state must have stopped. The maintenance
guard rejects live sessions and unresolved heavy-job ownership. Neither an
absent PID nor an unlocked per-owner file is sufficient to authorize retirement.

The typed owner checks the selected release, project or memory scope, issued ID
and original expiry. It compares the reconstructed prepared receipt with the
store's immutable fingerprint of the payload issued originally. An ordinary
receipt update cannot silently change the original binding. This fingerprint
detects drift through the store API; it is not authentication against an attacker
who can replace private state.

Text retirement requires a never-started, unattempted receipt with no execution
or artifact evidence and no job scratch at that ID. Memory retirement also
checks canonical mutation and lifecycle authority. The memory receipt records
whether canonical and lifecycle authority existed when it was prepared, so a
first create with no database is distinct from a missing previously existing
database. Corrupt, missing or contradictory authority remains protected.

Successful retirement marks text receipts `cancelled` and memory receipts
`expired_not_started`, making them eligible for normal completion retention.
Repeating the same retirement confirms that result
without renewing its completion time. The old ID cannot become a new operation.
No document, canonical memory, backup, index, scratch directory, ownership file
or unresolved marker is deleted by this command.

Started jobs, uncertain effects, incomplete recovery and abandoned scratch need
their own reconciliation. This command does not establish their cleanup.

## Verification status

On macOS 26.5.2 ARM64, core
`89aa48a2d4f1d4b97b7fa8d7b62ec00f284fd93ade6e740d4771b781ff0588e1`
passed the build-input guard, formatting, strict all-target Clippy, 351 library
tests and one deterministic retrieval test. Three manual tests remained ignored.
The real text MCP, configured memory/restore and two-release activation journeys
also passed. This verifies functional behavior; the separate semantic retrieval
quality failures remain unresolved.

Focused cases distinguish initial absent memory from missing/corrupt authority;
refuse drift, started work, scratch and contradictory canonical/forget/restore
records; preserve unrelated memory; and keep identical finalized bytes on retry.
Forgetting also removes retired preparation receipts bound to the forgotten
target.

The real MCP fixture also passed under OS network denial. It prepared text,
project-memory and shared-preference intents, verified live-maintenance refusal,
killed and reaped the owning server, and verified unexpired refusal with unchanged
receipt bytes. After the actual five-minute expiry, all three retired successfully.
An identical repeat across a wall-clock second preserved completion metadata;
reconnecting could not execute the old IDs. Source bytes stayed unchanged and no
destination, canonical memory, job scratch or heavy-job marker was created.
The fixture completed in approximately 304 seconds and confirmed child cleanup.
It changed neither clocks nor receipt timestamps.

Native Windows and power-loss qualification remain separate gates.

Artifact receipts bind a required text/PDF operation kind under schema 2. Both
share unused-intent retirement; each public start tool accepts only its own kind.
Prepared snapshots share a 64 MiB aggregate bound in addition to the 64-intent cap.
