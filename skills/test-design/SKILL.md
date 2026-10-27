---
name: test-design
description: "Design tests when adequacy or an oracle is uncertain, using behavioral failures, invariants, properties or independent comparisons. Skip mechanical tests that merely mirror implementation."
---

# Test design

Identify the externally meaningful contract and the plausible wrong implementation a test should reject. Choose inputs at boundaries, state transitions and interactions where that implementation diverges. A larger assertion count is not evidence of a stronger oracle.

Use example tests for concrete regressions, properties for meaningful invariants and differential comparisons when an independent implementation/source is trustworthy. Account for shared bugs and unsupported domains in the comparator. Control seeds, clocks, concurrency and environment only as needed for reproducibility; preserve a minimal counterexample.

Exercise the intended failure before accepting the test, through the original bug, a focused mutation or a deliberately wrong fixture. Avoid snapshots and mocks that reproduce the same assumptions as the code under test. Match integration depth to the consumer risk.

Deliver the test cases, oracle rationale, discriminating failure and actual results. Stop when the important failure modes and required gates are covered. Do not add low-impact mirror tests or lower thresholds to make a suite pass.
