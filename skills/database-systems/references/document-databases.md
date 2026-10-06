# MongoDB and document boundaries

Use these MongoDB-specific guarantees only for MongoDB; other document databases differ. Confirm server, driver, replica-set or sharded topology, and configured read/write concerns. Official sources reviewed on 2026-10-06.

## Model the atomic boundary

Choose embedded data when it belongs to one aggregate, is read or updated together, and has bounded growth. Use references when ownership, independent access, or unbounded collections make embedding costly. Specify validation and indexes even when the database permits flexible documents; flexibility does not remove the application's schema.

MongoDB writes are atomic at the single-document level. For a read-modify-write operation, include the expected version or previous value in the update filter, then check the match result. Prefer operators such as `$inc` for increments. An `updateMany` operation is not an atomic transaction across all affected documents. Multi-document invariants may require a transaction or a different boundary. [Atomicity and transactions](https://www.mongodb.com/docs/manual/core/write-operations-atomicity/)

A unique index can enforce a suitable identity invariant, subject to topology and index restrictions. Check missing fields, nulls, arrays, and tenant scope against the intended semantics before depending on that index.

## Read and write consistency

Choose read preference, read concern, write concern, and session behavior from the required guarantee. Reading from a replica does not by itself guarantee a fresh view after a write. Do not equate an acknowledged operation with every desired failover or durability guarantee; inspect the actual configuration.

Transactions require a supported topology; do not assume a standalone development server reproduces a replica set. Keep transactions short and verify driver retry semantics. A callback passed to a retrying transaction API may run more than once: keep external effects outside it or make them safely idempotent. Account for transient transaction errors separately from unknown commit outcomes. [Production transaction considerations](https://www.mongodb.com/docs/manual/core/transactions-production-consideration/), [transaction API behavior](https://www.mongodb.com/docs/manual/core/transactions/)

## Query and lifecycle behavior

Inspect the real filter, projection, sort, and aggregation pipeline with representative data. Use the server's explain results to compare documents and keys examined with results returned. Build compound indexes from that workload and its sort requirements; validate array/multikey and sharding restrictions. Do not assume a covered query or good selectivity from the index's existence alone.

Bound embedded arrays and batch work. Include a deterministic tie-breaker in sorted pagination. When document shapes evolve, make mixed-version readers and writers explicit, backfill in resumable batches, and validate both legacy and new forms during the compatibility window.

TTL cleanup is asynchronous. An expired record can remain beyond its nominal expiry, so enforce an expiration predicate in authorization, eligibility, or other time-sensitive reads. Building a TTL index over a backlog can trigger substantial deletion work; estimate that workload before deployment. [TTL index behavior](https://www.mongodb.com/docs/manual/core/index-ttl/)
