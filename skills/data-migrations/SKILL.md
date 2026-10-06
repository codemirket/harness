---
name: data-migrations
description: Plan and verify database or persisted-data migrations with concurrency, backfills, compatibility, integrity checks, and recovery.
---

# Data migrations

Identify the engine/version, schema, volume, traffic pattern, writers/readers,
replicas, availability budget, and failure impact. Read current engine-specific
DDL/locking behavior; a familiar statement can take different locks by version.

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

Choose conflict resolution for records changed during the backfill. Consider
change capture, version predicates, reconciliation passes, or temporary write
coordination based on the required invariant and actual engine capabilities.
A row count alone does not prove semantic equality.

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
