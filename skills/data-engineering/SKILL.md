---
name: data-engineering
description: Build, debug or review ingestion and transformation pipelines, incremental loads, CDC, event processing and data backfills. Use when source-to-consumer correctness, replay, freshness or recovery is the deliverable; use data-analysis for interpreting results and database-systems for database administration or query tuning.
---

# Data engineering

Trace one representative record from its source through extraction, transformation,
checkpoint and sink to the consuming table or application. Inspect the actual
engine, connector versions, schedule, credentials scope and existing operating
commands. A scheduler's successful run does not establish complete or correct data.

## Establish the data contract

- Identify the source grain, stable entity and event identities, units, null rules,
  schema, deletes and authoritative ordering. A per-entity version is not a global
  cursor; an event timestamp is not necessarily a unique or monotonic change token.
- Separate event time, ingestion time and processing time, with explicit timezones.
  Define late-data correction, duplicate handling and retention against the actual
  replay and backfill horizon. In streaming systems, inspect the source's watermark
  and window policy; a watermark must not be presented as proof of source completeness.
  [Beam's model](https://beam.apache.org/documentation/programming-guide/#watermarks-and-late-data)
  illustrates why arrival order and event time can differ.
- Locate the snapshot/change-stream boundary and pagination or partition coverage.
  Establish how missing partitions, expired source history and schema changes become
  visible failures. Preserve lineage to the source batch, cursor and transform version.

## Make recovery preserve meaning

Find the exact commit boundary between durable output and source progress. A crash
after writing output but before saving progress can replay it; advancing progress
first can lose it. Use the existing system's coordinated commit or stable idempotency
mechanism, then test that boundary. Limit an exactly-once claim to the participating
systems and configuration that provide it. A broker transaction does not by itself
make an external API effect atomic. See the scoped guarantees in
[Kafka's delivery semantics](https://kafka.apache.org/41/design/design/#message-delivery-semantics).

Prevent stale updates and backfills from overwriting newer state or resurrecting
deletes. If ordering information is insufficient, expose the ambiguity and use an
authoritative reconciliation strategy instead of inventing a timestamp tie-breaker.
Bound retries, batches and concurrency to the actual source and sink capacity.
Treat a timed-out write as an uncertain outcome until reconciled.

Validate schema and business invariants before publishing. Keep rejected or
quarantined records traceable and counted; do not silently coerce invalid values or
drop records to obtain a green run. A quarantine is useful only with a visible owner,
reprocessing path and explicit effect on completeness. Apply access and retention
limits to raw data, debug logs and quarantine stores as well as the published table.

For schema or storage migrations, add `data-migrations` at that boundary. For a
backfill, define the covered range, interaction with live writes, restart point and
cutover reconciliation before running it. Reuse the project's engine and toolchain;
this skill does not provision infrastructure or grant production write access.

## Verify the consumer outcome

Reconcile deliveries, distinct events, accepted/rejected records and final entities
at their respective grains. Check semantic totals, key uniqueness, null coverage,
partition completeness and freshness against the source contract. Inspect bounded
throughput, lag and recovery evidence when operating performance is in scope.

Exercise the relevant convincing failure: duplicate delivery, an older update after
a delete, a crash between sink commit and checkpoint, or a schema-invalid record.
Read [replay and late-event probes](references/replay-and-late-events.md) when those
boundaries are involved. Verify the sink and consumer after recovery, not just logs.
Report the exact source range, transformation, checks, rejected data and remaining
coverage limits. Do not claim source completeness when only a sample was available.
