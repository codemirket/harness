---
name: performance-engineering
description: Diagnose and improve latency, throughput, memory or resource cost using a measured baseline and controlled comparison. Use for profiling, query/service optimization and capacity experiments; avoid speculative tuning and speed claims from a single timing.
---

# Performance engineering

## Professional stance

Treat an optimization as a causal claim about a user operation under a stated
workload. Ask what consumes the time or resources and which cost moves elsewhere
when the proposed change wins.

Optimize only after attributing the bottleneck. Reject a faster candidate that
violates correctness or another required budget; change the recommendation when
representative load, cold behavior or variance reverses the result.

Ask systems engineering whether the generator represents arrivals and domain
owners which outputs must stay identical. Challenge a favorable average that hides
tail latency, errors, memory growth or reduced offered load.

Provide baseline/candidate revisions, workload and raw samples, observed effect
and uncertainty, correctness evidence and the conditions under which the result
should be reevaluated.

## Working method

Define the affected user operation, metric and workload before changing code.
Locate the actual entry path and instrument enough to attribute cost: CPU,
allocation/GC, I/O, query count, contention or dependency delay. A slow wall-clock
measurement alone does not identify the bottleneck. Read
[the comparison recipe](references/controlled-comparison.md).

Capture baseline revision, runtime/configuration, machine, data distribution,
concurrency and cache conditions. Use realistic sizes and include a pathological
case when it matters. Separate setup from steady-state measurement; record
startup/cold behavior separately if users experience it. Fix correctness before
optimizing a broken path.

Confirm the profiler or load generator can observe the target process and build;
record sampling overhead and timer resolution when they affect the comparison.
For service load, record offered arrivals, completed requests, errors and queued
time. A client that waits for each response before sending more reduces offered
load during a stall and can conceal overload. Choose that closed-loop workload
only when it represents the callers; include intended arrival timing when
evaluating a fixed-rate traffic or capacity claim.

Change one cause, preserve observable results and repeat the same workload with
baseline and candidate. Alternate or randomize order; keep raw samples and
report variability. Measure the relevant tradeoff: read vs write cost, latency
vs memory, throughput vs tail latency, or freshness vs cache hit rate. Make
budgets explicit; reject a candidate that wins one metric by violating them.

Use `database-systems` for query/index mechanics and `systems-engineering` for
capacity/overload design. Run representative behavior and failure regressions,
not merely a benchmark. State whether results are microbenchmark, integration
or production evidence. Report magnitude and limits; do not extrapolate a local
warm read into production throughput or advertise an unmeasured speedup.
