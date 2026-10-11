# Replay and late-event probes

Use a small fixture whose business result can be calculated independently. This
example specifies its assumptions; a real source must supply equivalent identity
and ordering evidence before the same strategy is appropriate.

## Current-state projection

The source supplies immutable event IDs and a strictly increasing version per order.
All amounts are integer minor units of one currency. A delete makes the order
inactive. The sink retains the highest observed version, including a deletion
tombstone, for at least the supported replay horizon.

| Arrival | Event ID | Order | Version | Operation | Amount |
| --- | --- | --- | --- | --- | --- |
| 1 | e1 | A | 1 | upsert | 100 |
| 2 | e2 | B | 1 | upsert | 50 |
| 3 | e3 | A | 3 | delete | — |
| 4 | e2 | B | 1 | duplicate of arrival 2 | 50 |
| 5 | e4 | A | 2 | late upsert | 80 |

There are five deliveries, four distinct events and two entity identities. The
current projection has A deleted at version 3 and B active at version 1: one active
order, amount 50. An event-history consumer may retain e4; it must not overwrite
A's newer current state. A matching event ID with different contents is a source
contract conflict, not an ordinary duplicate.

Inject failure after sink commit but before the corresponding checkpoint. Resume
with the persisted checkpoint and inspect the actual resulting state. Redelivery
counters can increase; distinct-event and current-state results must remain stable.
Also backfill A version 1 after the delete and show that it does not resurrect A.
A clean, ordered import can pass while an unconditional upsert fails these
recovery cases.

## Vary the missing assumption

- Remove the reliable version or event identity. The previous expected result is
  no longer justified by arrival time alone. Identify what the real source can
  guarantee and whether an authoritative snapshot or conflict handling is needed.
- Let two records share a timestamp at a page boundary. A timestamp-only `>` cursor
  can skip a record. Test the actual connector's stable cursor/pagination contract;
  a composite cursor helps only when the source supports its ordering semantics.
- Supply an invalid amount or incompatible schema. Check both the rejection record
  and the completeness status. Counting it as zero hides the data defect.
- Age a tombstone or deduplication key beyond retention, then replay older data.
  Demonstrate the supported replay horizon or require a safe full reconciliation.
- For windowed aggregates, deliver an event after the watermark. Check whether the
  configured engine drops, revises or separately emits it, and whether the consumer
  applies revisions correctly. Do not substitute a current-state upsert test for
  aggregate/window correctness.

Keep the test's raw deliveries, expected business result, source/sink checkpoints
and observed recovery result together. A source-specific transaction guarantee,
watermark policy or retry default must be checked against the deployed version's
official documentation; the fixture does not establish those guarantees.
