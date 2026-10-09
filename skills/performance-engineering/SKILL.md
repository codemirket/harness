---
name: performance-engineering
description: Diagnose and improve latency, throughput, memory or resource cost using a measured baseline and controlled comparison. Use for profiling, query/service optimization and capacity experiments; avoid speculative tuning and speed claims from a single timing.
---

# Performance engineering

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
