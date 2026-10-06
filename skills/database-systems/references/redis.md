# Redis decisions

Confirm server and client versions, standalone versus clustered topology, persistence, replication, and eviction configuration. Identify whether each key is disposable cache data or durable application state. Reviewed against official documentation on 2026-10-06; newer command options need a deployed-version check.

## Atomic operations and retry behavior

Prefer an atomic command for counters or conditional updates. Pipelining reduces network round trips; it does not supply a transaction boundary. `MULTI`/`EXEC` prevents interleaving during execution, but Redis does not roll back successful commands when another queued command fails at execution time. Inspect every result. `WATCH` can detect changed keys and abort an optimistic transaction; repeat the read and decision on retry. Preserve connection affinity for the transaction and clear watch state through the client API. [Transactions](https://redis.io/docs/latest/develop/using-commands/transactions/)

Check cluster key-slot requirements before choosing a multi-key operation or script. Keep server-side scripts bounded. A network timeout can hide a completed increment or enqueue; retrying blindly can duplicate work. Use application-level idempotency where the result cannot safely be repeated.

## Cache boundaries

Use namespaced keys with explicit tenant scope and a schema version when representation changes. Set value and expiration together when creating an expiring entry. Decide how invalidation survives process failure after the authoritative commit. For strict freshness, an expiring cache alone is insufficient; version-check the cached representation or use the source of truth for the critical decision.

Avoid an unbounded `KEYS` scan in a busy service. Iterate with `SCAN`, process bounded batches, and allow duplicate keys. An empty batch does not finish the scan; the returned zero cursor does. `COUNT` is a work hint, not a guaranteed page size, and a scan is not a consistent snapshot of changing data. [SCAN guarantees](https://redis.io/docs/latest/commands/scan/)

## Leases and external resources

A single-instance lease can be acquired with `SET key unique-token NX PX duration`; release only if the stored token still matches, using an atomic conditional deletion supported by the deployed version or a reviewed script. Separate `GET` then `DEL` is unsafe. Never unconditionally delete a lease in a finally block.

Expiration does not stop an old holder from continuing work. Where stale holders could corrupt an external resource, that resource must reject stale fencing tokens or enforce the invariant itself. Account for process pauses, network partitions, replication failover, and clock assumptions before treating a distributed lease as a correctness guarantee. Redis's own lock documentation calls out these limits; adding replicas alone does not prove mutual exclusion. [Distributed lock guidance](https://redis.io/docs/latest/develop/clients/patterns/distributed-locks/)

Test cache-miss surges, eviction, expiry, reconnects, and ambiguous responses separately from the healthy path. Observe memory growth, rejected writes, hit ratio, latency, and hot keys with the project's tools. Do not adopt a generic memory limit or eviction policy without checking which keys may safely disappear.
