---
name: data-pipelines
description: "Implement import, ETL or replay behavior when lineage, duplicates, late events and partial failure affect data integrity. Skip one-off descriptive analysis."
---

# Data pipelines

Define source identity, schema/version, event/time semantics, destination grain and lineage. Identify commit/checkpoint ownership and which effects can occur independently. Keep originals or recoverable inputs according to the project's retention/privacy contract.

Design idempotency, deduplication, ordering and late-event policy from actual source guarantees. Separate rejected records from silent loss; bound retries/backpressure and make partial progress inspectable. A repeated batch must not duplicate effects, and a checkpoint must not get ahead of committed output.

Test duplicate and late input, schema drift, partial failure, restart/replay and reconciliation of source-to-destination counts/values. Use representative disposable data and inspect the actual saved output, not only a runner success.

Return pipeline source/configuration, lineage/checkpoint contract and recovery evidence. Stop at unknown source identity, unsupported schema or an unapproved destructive replay. Do not erase anomalous records or silently change time semantics to make a pipeline appear complete.
