---
name: database-systems
description: Design, debug, and operate database schemas, queries, transactions, migrations, and caches. Use for PostgreSQL, Redis, MongoDB, SQLite, or other persistence work where data integrity, concurrency, query cost, or recovery matters.
---

# Database systems

Work from the application's invariants and observed workload. Choose the relevant techniques below; a query fix does not require a database redesign. Use the existing engine, driver, migration tool, and deployment conventions unless a change is justified and authorized.

## Establish the actual contract

Identify the engine and version, driver behavior, topology, connection pooling, and affected environment. Find the schema, migrations, callers, and existing operational runbooks. Determine what must remain true when requests overlap, retries occur, or a process fails between steps. Distinguish the authoritative store from derived indexes, replicas, caches, and external systems.

State a concrete success condition: an invariant enforced, a query made correct, measured latency reduced, a migration safely resumable, or recovery demonstrated. Use representative fixtures and redacted diagnostics; do not expose connection strings or customer data. Access and implementation authorization do not imply permission to mutate production or perform destructive operations.

Read only the engine reference needed:

- [PostgreSQL](references/postgresql.md): isolation, query plans, index builds, and contention.
- [Redis](references/redis.md): cache correctness, transactions, leases, and bounded key operations.
- [MongoDB and document databases](references/document-databases.md): document boundaries, atomic updates, indexes, and consistency.
- [SQLite](references/sqlite.md): writer concurrency, connection settings, WAL, and backups.

For another engine, verify its guarantees in the documentation for the deployed version; familiar SQL syntax does not imply identical locking, DDL, NULL, or isolation semantics.

## Put invariants where concurrent writers cannot bypass them

Use appropriate unique, foreign-key, check, and non-null constraints. An application-level existence check alone cannot enforce uniqueness. Define identity, tenant scope, deletion behavior, units, time-zone semantics, precision, and nullability explicitly. Prefer exact representations for money; keep business identifiers distinct from incidental row position.

Map each operation to its atomic boundary. Prefer an atomic conditional update or increment over a read-modify-write round trip. Check affected-row counts and distinguish a failed precondition from success. For multi-record invariants, choose the required isolation, locking, or version check and explain the anomaly it prevents.

Keep transactions short; acquire multiple locks in a consistent order where practical. Retry only documented transient failures, with a bound and appropriate delay. Retry the whole transaction's decision logic when its snapshot is invalid. A lost response after commit is an ambiguous outcome: use a durable idempotency key or reconcile state before repeating non-idempotent work.

Do not call an external payment, send a message, or publish an event inside a transaction retry callback unless repetition is safe. When a database change and external event must agree, consider a transactional outbox with an idempotent consumer; a local transaction alone cannot make two systems atomic.

## Change queries using evidence

Read the real query and its parameters, including ORM-generated SQL and extra round trips. Check semantics first: joins multiplying rows, missing tenant filters, unstable pagination, NULL comparisons, and timezone boundaries can resemble performance bugs.

Use a representative data distribution and the engine's plan tools. Compare estimated versus actual cardinality, rows examined versus returned, repeated loops, sorts or spills, lock waits, and end-to-end latency. A sequential scan can be the right plan. Match indexes to predicates, ordering, and workload; account for write amplification, space, and redundant indexes. Do not prescribe indexes from column names alone.

Execution-based explain tools run work; mutations, functions, triggers, and large reads can have effects or consume production capacity. Start with non-executing plans and use an appropriate isolated dataset for execution measurements. Record the data scale, concurrency, and cache state behind a performance claim.

Use a deterministic unique tie-breaker for pagination. For changing datasets, decide whether users need a stable snapshot or a live feed before choosing offset or keyset pagination. Bound batches and result sizes without silently truncating required results.

## Make changes recoverable

For rolling deployments, use compatible stages: add the new representation, deploy compatible readers/writers, backfill, verify, switch consumers, then retire the old representation when safe. Explicitly handle writes occurring during the backfill. Use bounded, resumable, idempotent batches with progress markers and a reconciliation query.

Estimate locks, table rewrites, log growth, replication lag, and disk headroom for the engine and version. Set appropriate statement or lock limits through the project's operational conventions. Rehearse significant migrations on representative data, including interruption and restart. A destructive down migration is not a recovery plan; distinguish code rollback, data repair, and restore.

Validate recovery through a supported consistent backup and an isolated restore when recovery is in scope. Check both structural integrity and application invariants; a completed backup command alone does not demonstrate restorability.

## Treat caches as a consistency decision

Specify the cache key's tenant/user scope, version, expiry, negative-result behavior, and acceptable staleness. Describe how writes invalidate or update cached values and what happens during cache failure. Prevent a delayed stale reader from repopulating a value after invalidation when that race matters. Use version checks or a proven project pattern rather than claiming that a TTL eliminates the race.

Bound stampedes and retries, keep TTL jitter proportional to the freshness requirement, and measure memory and eviction behavior. Do not allow loss of a derived cache to destroy authoritative state. Sessions, queues, counters, and leases may require different durability and eviction policies from disposable cached results.

## Verify the failure that matters

Use integration checks on the actual engine for constraints and transaction behavior. Exercise competing connections with deliberate synchronization for concurrency bugs; timing sleeps alone are weak evidence. Test duplicate requests, stale versions, partial backfills, timeout ambiguity, cache misses, and restart behavior as relevant. Report measured results, remaining assumptions, and any production-only condition that was not reproduced.
