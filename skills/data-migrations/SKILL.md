---
name: data-migrations
description: Plan and verify database or persisted-data migrations with concurrency, backfills, compatibility, integrity checks, and recovery.
---

# Data migrations

## Professional stance

Own preservation of meaning while representations and consumers change. Ask which
writers remain active during every phase, which records fall behind a cursor and
exactly when recovery stops being a simple rollback.

Prefer compatible staged change when online readers and writers must coexist.
Change the plan when source coverage, conflict policy or recovery cannot preserve
required values; delay destructive contraction until consumers are verified.

Ask database engineering for lock and transaction guarantees and release
engineering for actual version overlap. Challenge equal counts or sums as proof of
correct transformed entities and cursors as proof of complete source coverage.

Give release and application owners ordered scripts, phase prerequisites,
transformation version, reconciliation queries/results, restart state and the
point where rollback requires data repair.

## Working method

Identify the engine/version, schema, volume, traffic pattern, writers/readers,
replicas, availability budget, and failure impact. Read current engine-specific
DDL/locking behavior; a familiar statement can take different locks by version.

Locate the project's migration runner, migration history and checksum/locking
rules. Verify the actual driver, transaction mode and isolated target before
running scripts; an installed database shell may differ from the application's
driver. Use the existing runner and rehearse with the same transaction boundaries
as deployment. Changing an already applied migration can invalidate history;
prefer a new corrective migration under the project's convention.

## Design for mixed versions

Prefer expand, migrate, verify, switch, and contract when online compatibility is
needed. Define how old/new application versions read and write during every phase.
A backfill before enabling dual writes can miss concurrent updates; establish the
write path or reconciliation mechanism before relying on copied data.

Use constraints and ownership boundaries to preserve invariants. Handle existing
nulls, duplicates, invalid values, timezone/encoding differences, and foreign keys
before enabling stricter constraints. Do not silently discard nonconforming rows.

## Make the data movement resumable

Use bounded batches with a stable unique cursor or explicit migration state.
Nullable source values must not trap the backfill in an endless eligibility loop.
Make retries idempotent and record progress only when the corresponding writes
commit. Bound transaction size and monitor lock time, replication lag, storage,
write amplification, and user-visible latency.

Define the source coverage boundary separately from the resume cursor. Concurrent
inserts or key changes behind an advancing cursor can be missed even when every
batch succeeds. Use the source's supported snapshot/change-capture contract or a
final reconciliation that covers those writes. Persist cursor and migration
version together so a changed transformation cannot silently resume old progress.

Choose conflict resolution for records changed during the backfill. Consider
change capture, version predicates, reconciliation passes, or temporary write
coordination based on the required invariant and actual engine capabilities.
A row count alone does not prove semantic equality.

For example, replacing balances 10 and 20 with 15 and 15 preserves both count and
sum while corrupting both entities. Reconcile the transformed values by stable
identity or another independently justified invariant. Sampled checks establish
only sampled coverage; record any excluded rows and the reason.

## Verify and recover

Dry-run on representative data. Compare counts, sampled values, aggregates,
constraints, and application behavior across old/new paths. Exercise partial
failure, restart, duplicates, and concurrent updates. Check query plans/indexes
when the new layout changes access patterns.

Define backups and restoration evidence, cutover checks, rollback/forward-fix
conditions, and the point after which rollback loses data or requires translation.
Destructive cleanup waits until consumers have migrated and retention requirements
are met. Production execution and destructive changes need explicit authorization.

Deliver migration scripts, ordering and application prerequisites, verification
queries, resumability behavior, and recovery procedure. Report actual checks and
unverified production assumptions rather than calling a migration universally safe.
