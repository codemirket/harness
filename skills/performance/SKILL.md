---
name: performance
description: "Investigate a measured bottleneck or performance-budget regression with controlled baseline, profiling and comparable results. Skip speculative optimization."
---

# Performance

Define the user-visible metric, workload, resource budget and correctness constraints. Record hardware/platform, build, warm/cold state, input distribution and background load. Reproduce the regression or establish a baseline before optimizing.

Profile the actual expensive path and distinguish queue/wait time, compute, I/O and memory. Change the cause supported by measurements. Preserve behavior and avoid trading an unmeasured cache, fan-out or retained memory cost for a faster toy case.

Compare baseline and change under matched conditions with repeated observations appropriate to variability. Include tail behavior, failure/large-input cases and post-use idle/resource cleanup when relevant. Keep instrument overhead and excluded work explicit.

Return the measured cause, intervention, comparable results and correctness checks. Stop when the stated budget is met or evidence rejects the hypothesis; report remaining uncertainty and measurement limits. Do not claim speed, smoothness or low memory from code shape or one uncontrolled run.
