# Worked example: API plus asynchronous worker

Assume an API persists an accepted job and a worker calls a slow provider.
Draw: caller → API → durable jobs → worker → provider. Add result reads and
status ownership. An accepted response means durable acceptance only; the UI
must distinguish pending, failed and completed work.

| Failure | Required decision | Useful isolated check |
| --- | --- | --- |
| Provider exceeds deadline | Who retries; total age/attempt limit | Delay response past budget; inspect job state and bounded workers |
| Worker dies after provider success | Reconcile using provider key/result | Replay same job; verify one effect or explicit unknown outcome |
| Arrival exceeds completion | Admission/backpressure and oldest-job limit | Bounded burst; verify rejection/degradation and recovery |
| DB unavailable | Acceptance fails without false success | Fail write; verify caller and queue agree |
| Poison job repeats | Quarantine with diagnosable state | Repeat deterministic failure; healthy jobs still progress |

Start capacity reasoning with units. In a stable illustrative workload,
20 jobs/second × 0.4 seconds of mean worker occupancy is about 8 concurrently
occupied slots. It does not establish that 8 slots meet a tail-latency target:
arrival bursts, service-time variance, utilization and provider limits matter.
If admitted arrivals are 30/s while completions sustain 20/s, backlog grows by
about 600 jobs/minute until rejection, capacity or load changes. Record observed
arrival/completion/age curves to test this model.

Budget example (invented for design): a 1,000 ms request allows 100 ms admission,
600 ms dependency work and 300 ms serialization/network margin. Nested retries
must fit the remaining deadline. Three attempts in each of three nested layers
can cause 27 leaf attempts; choose a retry owner and measure actual attempts.
Use jitter only with a bounded policy; do not retry invalid or forbidden work.

Recovery is part of the state machine. Define when leases expire, whether a
second worker can run, how stale results are rejected and who reconciles
ambiguous external effects. Verify this against actual storage/queue guarantees;
do not label a workflow “exactly once” from an in-process test.

Deliver a small topology, a failure table with concrete expected behavior,
capacity assumptions and the observed experiment. Monitor user outcomes plus
queue age, saturation and dependency timing. Thresholds must correspond to an
action; collecting every metric is not an operational plan.
