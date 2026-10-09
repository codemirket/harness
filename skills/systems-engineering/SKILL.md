---
name: systems-engineering
description: Design or diagnose service topology, reliability, capacity and cross-service failure behavior. Use for dependency chains, queues, overload, recovery and consistency across components; use architecture-review for code structure and release-operations for rollout.
---

# Systems engineering

Model the system that exists before proposing components. Trace a real request
and one asynchronous job. Mark trust boundaries, durable state owners, network
hops, queues and the operations that can happen more than once. Record source
paths/configuration for topology facts and mark unknowns. Read
[reliability and capacity](references/reliability-capacity.md) when dependencies,
load or recovery determine the design.

Name the user-visible reliability target and workload envelope: latency window,
acceptable errors, data-loss tolerance, recovery time and expected bursts. Use
existing service objectives when available; proposed numbers are assumptions,
not commitments. Distinguish availability from freshness and correctness.

For each critical dependency, decide timeout, retry ownership, concurrency
limit, admission behavior and degradation. Carry an end-to-end budget through
the chain. Make queues bounded or give their storage, age and overload policy a
real owner. More replicas do not fix a shared saturated database.

Compare the smallest viable change with doing nothing: which observed failure
or measured bottleneck improves, what new failure mode appears, and how it is
recovered. Use `database-systems` for data consistency and
`engineering-judgment` for external effect semantics. Avoid adding distributed
coordination where a single owner meets the requirement.

Verify a representative failure and recovery path in an isolated environment:
slow/down dependency, duplicate work, worker death or bounded overload. Check
user outcome, resource growth and recovery, not only process uptime. Report
measured capacity separately from estimates; a local simulation does not prove
regional failover, production load capacity or a recovery objective.
