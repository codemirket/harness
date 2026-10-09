# Engineering capability review — 2026-10-09

This review adds five focused workflows and one worked reference per workflow.
It does not establish expert-level output or measure the effect of activating a
skill in a fresh agent session. Registry membership and routing are maintained
separately from this review.

## Boundaries and additions

Existing sources were inspected before writing: `engineering-judgment` and its
service-integration reference; `architecture-review`; `test-design`;
`database-systems` and its SQLite reference; `release-operations`; and the task
routing reference. They already cover general correctness, retries and outboxes,
transaction semantics, structural review, testing and deployments.

| Addition | Distinct responsibility | Worked reference |
| --- | --- | --- |
| [backend-engineering](../../skills/backend-engineering/SKILL.md) | Request-to-domain construction, contract, lifecycle and admission | Debit command, durable replay and original receipt semantics |
| [systems-engineering](../../skills/systems-engineering/SKILL.md) | Topology, capacity, overload and recovery | API/worker/provider failure table and explicit capacity assumptions |
| [infrastructure-engineering](../../skills/infrastructure-engineering/SKILL.md) | Desired/observed resources, network, storage and safe plan review | Add a worker to an existing deployment |
| [project-scaffolding](../../skills/project-scaffolding/SKILL.md) | Selected native generator, staged merge and existing-repo preservation | Add one service without reinitializing a monorepo |
| [performance-engineering](../../skills/performance-engineering/SKILL.md) | Attributable baseline and controlled result-preserving comparison | Tenant history read with index/write/storage tradeoffs |

Each main skill is 236–297 words. References hold the detailed decision examples.
The skills route to existing database/integration/release workflows rather than
replicating their algorithms. Generator selection does not authorize downloads,
dependencies, lifecycle hooks or Git initialization. Infrastructure guidance does
not authorize production operations.

## Local executable exercise

Executed on macOS 26.6.2 ARM64, Python 3.9.6 and SQLite 3.51.0:

```sh
python3 examples/assurance/engineering.py --output build/capability-engineering/new-report.json
```

The retained source is [engineering.py](../../examples/assurance/engineering.py).
It is a synthetic assurance fixture, not a production service or runtime dependency.
Ignored `build/capability-engineering/shipped-report.json` retains raw timing
samples, query plans, environment and results. SQLite files were temporary and
removed when the experiment ended. Exact shipped source SHA-256:
`67b1950eafb4bade9130460cbd7ec42921dee981b7fb68685f77716da498f4b8`.

A callable service performs tenant-scoped positive-integer credit debits and
stores receipts atomically. Twelve checks passed:

- Exact debit result and replay after the original response is discarded.
- Same key with changed payload is rejected without additional debit.
- Cross-tenant access is denied before receipt lookup; boolean amounts are rejected.
- Insufficient credit cannot mutate the account.
- An injected exception between debit and receipt rolls back both effects.
- Two concurrent distinct commands cannot overdraw using separate connections.
- Concurrent identical commands have one durable effect and identical results.
- A held writer lock produces a bounded busy failure (204.791 ms observed).
- Replay returns the original receipt after subsequent commands.
- Reopened connections preserve each tenant's balance-plus-successful-debits invariant.

Four in-memory defect mutations were also detected by these checks: omitted
tenant authorization, omitted payload conflict handling, a corrupted replay
receipt, and committing instead of rolling back on failure. Mutation outcomes
are retained in the ignored `mutations.json`; this bounded experiment checks
that the tests reject those defects, not that every possible defect is covered.

Final independent invariant: tenant A balance 150 + debits 850 = opening 1,000;
tenant B balance 1,000 + debits 0 = opening 1,000. Both remain nonnegative.
These tests exercise a trusted callable service context, not authentication,
HTTP serialization, a process crash, power loss or a distributed deployment.
The injected exception is a rollback check, not proof of crash recovery. No
infrastructure stack or native scaffold generator was executed.

## Controlled optimization experiment

Two identical databases initially contained 60,000 history rows. Half belonged
to one hot tenant; the remaining rows were spread among tenant identifiers.
The candidate added `(tenant, created DESC, id DESC)`. Both queried latest 20
rows for a hot tenant, two smaller tenants and an absent tenant; timestamp ties
used the ID as a tie-breaker. Expected results were independently computed in
Python and matched every timed query. Plans showed baseline scan/sort versus
candidate index search; no test relies on plan display formatting.

Three warm-up rounds preceded 40 repetitions of four parameter values per
variant (160 read observations each), with variant order shuffled using seed
42. Read measurements included execution and fetch; setup was excluded.
Ten 1,000-row committed write batches then tested write cost, with matching
results after each batch. Both ended at 70,000 rows.

| Observation | Baseline | Indexed candidate |
| --- | ---: | ---: |
| Warm read median | 1.771792 ms | 0.013396 ms |
| Warm read p95, nearest-rank | 4.934791 ms | 0.031958 ms |
| 1,000-row write median | 0.713417 ms | 1.201917 ms |
| Final database bytes | 1,245,184 | 2,711,552 |

The candidate improved this local warm-read workload while increasing median
write time and file size. These are one-process observations, not production
latency or throughput estimates. The mixed read percentile combines four query
shapes and is not a service SLO percentile. Ten write batches give limited tail
evidence; host scheduling and cache state remain uncontrolled. No cold-cache,
concurrent benchmark, production distribution or sustained load was measured.
This exercise demonstrates a measurement workflow; it does not justify adding
an index to an unrelated application.

## Scaffold, bounded overload and plan-input exercises

Retained [systems.py](../../examples/assurance/systems.py) runs with only stdlib:

```sh
python3 examples/assurance/systems.py --output build/capability-engineering/new-systems-final.json
```

The executed report is `build/capability-engineering/systems-final.json`; source
SHA-256 is `bda1269318be2974c5c0615e0b7b4654137319fa0b6bcea48fb8f3fa4d980c68`.
Five scaffold checks preserve two original files byte-for-byte, execute a staged
Python command (7 → 14), reject negative input with exit 2, reject a destination
conflict before copying any new file, and reject a linked destination ancestor.
This exercises a staged merge procedure using a hand-written fixture; no native
framework generator was invoked and no cross-process atomic merge is claimed.

A real stdlib bounded queue is driven through deterministic outage/recovery ticks.
Of 12 offered jobs, six were admitted and six rejected immediately. The backlog
never exceeded three; all six admitted jobs completed once after recovery and
the queue drained by tick eight. Conservation, no duplicate completion and no
rejected job being accepted were checked throughout. These are six local model
invariants, not measurements of broker durability or production service capacity.

Four synthetic plan-input checks accept unchanged configuration/state/target
identity and reject a changed desired exposure, changed observed generation or
changed environment. They demonstrate why prior evidence must be invalidated;
they do not implement or exercise Terraform plan freshness, provider refresh,
apply safety or resource health. No cloud or deployment credentials were used.

## Validation and evidence limits

The skill-creator structural validator could not run because PyYAML is absent
from both system and configured Python runtimes. No dependency was installed.
A scoped stdlib check verified the exact two-field frontmatter used here, name
matching, bounded description and local Markdown link targets. This is not a
replacement for a general YAML parser. The repository Markdown checker and
whitespace checks were run on the authored files; neither proves instructional
quality. No blind comparison against baseline agent output has been performed.

Further representative evaluation should run the same tasks with and without
each selected workflow: an endpoint with replay/conflict bugs, a dependency
outage and overload case, an infrastructure diff with accidental exposure, a
scaffold into an already dirty repository, and a slow operation whose apparent
optimization changes results. Use independently specified outcome assertions,
resource limits and a reviewer unaware of the condition. Retain failures and
required interventions; do not substitute skill counts or self-ratings for
measured results.

## Primary technical sources

Checked 2026-10-09:

- [SQLite transactions](https://www.sqlite.org/lang_transaction.html): explicit write transaction and busy behavior.
- [SQLite isolation](https://www.sqlite.org/isolation.html): connection isolation and serialized writers.
- [Python sqlite3](https://docs.python.org/3/library/sqlite3.html): connection/transaction API; the exercise uses Python 3.9-compatible explicit transactions.
- [SQLite query plans](https://www.sqlite.org/eqp.html): diagnostic output stability limits.
- [Python timeit](https://docs.python.org/3/library/timeit.html): measurement setup and repetition caveats.
- [Docker Compose config](https://docs.docker.com/reference/cli/docker/compose/config/): resolved model and quiet validation.
- [Terraform plan](https://developer.hashicorp.com/terraform/cli/commands/plan): refresh, speculative planning and sensitive plan artifacts.
