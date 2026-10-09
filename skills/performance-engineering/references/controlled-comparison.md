# Worked recipe: optimize tenant transaction history

Hypothesis: fetching the latest 20 rows for one tenant spends time scanning and
sorting a much larger table. First inspect the real query, parameters, row
counts and query plan. A composite index might help; it also consumes storage
and imposes write maintenance. This is a hypothesis, not an automatic change.

1. Build identical disposable baseline/candidate databases from a fixed seed
   with realistic tenant skew and history sizes. Choose a stable ordering with
   a tie-breaker. Verify both return exactly the independently expected rows,
   including empty tenants and timestamp ties.
2. Record query plans before and after; investigate scans, sorts, estimated or
   observed row counts with the engine's native diagnostics. SQLite explicitly
   treats EXPLAIN QUERY PLAN output as unstable: do not make tests depend on its
   display format. Validate behavior and measure time instead.
3. Warm both cases equally for a warm-read experiment. Run interleaved or
   seeded-random order repetitions with identical parameter sequences. Retain
   per-query milliseconds, sample count, median and a clearly defined percentile.
   A percentile over small batches is not a production request percentile.
4. Measure candidate file/index size and a representative write batch. Keep
   transactions, durability settings and connection lifetime the same. If write
   cost or cold-cache behavior cannot be measured, mark the tradeoff unresolved.
5. Recheck results after writes and rerun the failure/invariant tests affected by
   the change. Compare against the same baseline, not the previous favorable
   sample. Stop if the improvement is within noise or shifts the bottleneck
   without helping the user operation.

Example report structure (fill with observations, not invented figures):
`workload; runtime; baseline/candidate change; raw-sample artifact; result
identity; median/p95 method; storage/write tradeoff; failure checks; limits`.
Keep the benchmark executable in the project when it protects a material
performance contract. Avoid adding timing assertions to ordinary CI when
uncontrolled hosts would make them flaky.

[SQLite query-plan documentation](https://www.sqlite.org/eqp.html) explains the
diagnostic limits. Python's [timeit documentation](https://docs.python.org/3/library/timeit.html)
discusses timer/setup and repetition effects; select measurement semantics for
the workload rather than treating a single fast run as evidence.
