# PostgreSQL decisions

Confirm the server version and pooler mode before choosing SQL or relying on session state. This reference uses official PostgreSQL 18 documentation reviewed on 2026-10-06; consult the matching manual for another deployment.

## Concurrency and invariants

At Read Committed, successive statements can observe different committed states. Repeatable Read provides a stable snapshot but does not prevent every serialization anomaly. Serializable can reject a transaction to preserve serial equivalence: handle SQLSTATE `40001` by retrying the entire transaction, including decisions made from reads. Do not blindly retry constraint violations. For a narrowly scoped invariant, an atomic conditional update or correctly chosen row lock can be simpler than increasing isolation globally. [Isolation documentation](https://www.postgresql.org/docs/18/transaction-iso.html)

Example decision: reserving stock can use `UPDATE ... SET available = available - :n WHERE id = :id AND available >= :n RETURNING ...`, with a validated positive quantity and a check of whether a row was returned. A preliminary read alone does not reserve anything. Use a transaction if reservation and related records must commit together. Cross-row rules need their own concurrency argument.

Investigate blocked work through the blocking transaction and its lock lifetime. Increasing pool size can increase contention. Avoid holding a transaction open during user interaction or external network calls. Decide whether session settings survive the project's pooler behavior rather than assuming connection affinity.

## Query plans

Start with `EXPLAIN`; use `EXPLAIN (ANALYZE, BUFFERS)` on a suitable dataset when execution is authorized. It executes the statement, so a write, volatile function, or trigger can have effects. Wrapping a measurement in a rollback is not a universal sandbox for external or nontransactional side effects. Compare actual row counts and loop counts with estimates; inspect buffer activity and spills, not only the headline cost. Planner cost units are not milliseconds. [Using EXPLAIN](https://www.postgresql.org/docs/18/using-explain.html)

Inspect actual predicates, ordering, row distribution, statistics freshness, and existing indexes before adding one. Validate nullable unique keys, collation, casts, and expression predicates against the intended semantics. Check that the same plan is useful for representative parameter values; a single convenient sample can hide skew.

## Schema changes

An ordinary index build blocks writes. `CREATE INDEX CONCURRENTLY` allows concurrent writes but takes additional work and waits, cannot run inside a transaction block, and can leave an invalid index after failure. Check the index state and documented recovery path before rerunning; do not infer success from the object merely existing. Partitioned tables and unique builds have additional restrictions. Check that the migration runner permits the required transaction mode. [CREATE INDEX](https://www.postgresql.org/docs/18/sql-createindex.html)

For large backfills, track durable progress with stable keys and recheck rows changed concurrently. Avoid a single unbounded update unless measured impact and maintenance conditions justify it. Track dead tuples, autovacuum progress, storage growth, replication lag, and long-lived transactions through the project's existing operational tools; do not disable maintenance to make a benchmark look faster.
