# Runtime integration review — 21 October 2026

Keep Mem0 and LangGraph in the optional agent runtime. Their integration reaches
actual worker prompts and recoverable execution; neither is merely installed.
Keep Python and Node where the retained engines require them. No low-effort Rust
port identified here removes either runtime or preserves the complete contract
without a substantial replacement. Graphiti is not a better default for the
current explicit-fact, offline memory contract.

This review follows implementation commit `33f4cd2e61935a216cab820b23705acbcbb634b5`
on `main`. The cleanup and measured recall optimization are recorded with this
document in local Git history; nothing was pushed or deployed.
Current source fingerprint:
`c0a8aa3ba4ded447888bd44b3e555cad419c47e255af7f6eb7a0b8b991a1bf7e`.
The [system map](harness-map.md), [runtime guide](agent-runtime.md) and
[implementation record](harness-final-pass-implementation-2026-10.md) retain their
distinct inventory, operating and historical roles.

## Actual integration and its limits

| Outcome | Path into the implementation | Evidence and disposition |
| --- | --- | --- |
| Remember an explicit project fact | Host MCP/CLI → Rust scope/validation → private IPC → `memory.put` → real `Memory.add(infer=False)` | Mem0 owns vector records and history. Keyed retries, provenance, expiry, explicit supersession and digest-bound forgetting are exercised. KEEP OPTIONAL. |
| Recall useful context for a worker | `Driver::start` → LangGraph recall node → Mem0 search → rank interleaving → `project_memories` in the inference prompt | Up to eight explicit or objective/task-derived queries; results bounded to 20 and labeled untrusted. Scope comes from Rust. KEEP OPTIONAL. |
| Survive an interruption | Actual Rust worker loop → `Driver::advance` → LangGraph `StateGraph`, `SqliteSaver`, interrupts and synchronous persistence | Intent is saved before dispatch and state after dispatch. Resume checks checkpoint identity and original bindings. KEEP OPTIONAL. |
| Preserve execution budgets | Saved envelope/snapshot plus graph deadline, turn, pause and verification state | Known inactive pauses receive bounded cumulative credit; turns and repair budgets never reset. An uncertain effect is not blindly replayed. KEEP. |
| Host verification | Rust host-check record → graph verification transition | A failed or stale check can revoke prior verified status. The graph cannot confer host permission or semantic acceptance. KEEP. |
| Coordinate tasks | Existing task SQLite | Coordination/evidence remains separate from Mem0 facts and LangGraph execution checkpoints. Do not merge merely because all persist state. KEEP OPTIONAL. |

Source owners: [memory.py](../runtime/agent/memory.py),
[workflow.py](../runtime/agent/workflow.py),
[durable.rs](../src/router/durable.rs),
[execution.rs](../src/router/execution.rs),
[IPC](../src/agent_runtime.rs), [bridge](../runtime/agent/bridge.py), and
[lifecycle](../runtime/agent/lifecycle.py).

The package is optional. Once selected, a runtime failure does not silently turn
a durable worker into an ephemeral one. Without it, workers are explicitly
ephemeral; required recall fails before inference. Recall defaults to best effort
within an enabled runtime. An empty store is a successful empty result, including
in required mode; required does not mean a matching fact must exist.

Only the host writes memory. Automatic transcript extraction is deliberately
disabled, and the fixed LLM adapter rejects inference. Local embeddings and
hybrid retrieval remain active. Memory is recalled at the initial graph boundary
and retained in the checkpoint; it is not continually refreshed during a run.
Native Codex/Claude sessions are not automatically checkpointed by Mirket.
These are explicit scope limits, not missing library wiring. Selecting the package
does not prove that hosts naturally store useful facts or choose durable workers.

## Cleanup completed

| Finding | Change | Preserved behavior |
| --- | --- | --- |
| Every commit could stale all four generated inventory outputs even with unchanged input bytes | Schema 3 uses content/source hashes, not HEAD, branch or dirty-file lists; CI checks generation | Exact input provenance and actual drift detection; source archives need no Git metadata |
| Runtime guide still described a single objective excerpt and fixed overfetch | Documented focused queries and adaptive bounded candidate search | Existing limits and incomplete-coverage reporting |
| System map quoted intermediate 420/69 test counts and 808 audit coordinates as current | Reconciled with the committed 424/70 results and 732-coordinate audit; dated both | Prior failures and historical evidence remain visible |
| Architecture implied every operation was implemented only in Rust | Named the owned Python/Node adapters and Rust authority | One orchestration and permission boundary |
| Shared design child launcher retained unused HyperFrames environment overrides | Removed the unused override surface and obsolete telemetry assertion | Fixed clean child environment, private scratch, output limits, cancellation and process-tree cleanup |
| Two critique helpers were obsolete | Removed the duplicate audit script and broken historical feedback generator; linked maintained checks | Original dated JSON and Git history, including negative results |
| Implementation report still said uncommitted after its authorized commit | Corrected status and linked this follow-up | No claim of push, deployment or live-client activation |

Remaining retired-video references in the server are an explicit diagnostic for
old callers. Tests assert the encoder is absent. Historical Penpot/video reports
remain historical; browser motion guidance still has a retained outcome. These
are not active bundled-video dependencies. Saved selections, recovery formats,
path isolation and release checks were not relaxed.

## Measured recall reuse

The initial review identified a repeated Mem0/Qdrant/history open for each of up
to eight queries. A subsequent authorized experiment compared the unchanged path
with a shared context, then repeated the comparison using the implemented
`recall_many`. Keep the optimization: it exceeds the predeclared useful-benefit
criterion of at least 15% and 50 ms saved for a four- or eight-query batch, without
a meaningful one-query regression.

The fixture used 22 and 128 synthetic facts, real pinned Mem0/FastEmbed retrieval,
one/four/eight queries, and five alternating pairs per case in a network-disabled
Linux ARM64 container. Embeddings were warm; ingestion, container startup, IPC,
query construction and worker inference are outside these timings. Both arms ran
against the same store. Final implementation medians:

| Facts | Queries | Reopen per query | Shared handle | Saved |
| --- | --- | --- | --- | --- |
| 22 | 1 | 99.70 ms | 98.86 ms | 0.84 ms (0.8%) |
| 22 | 4 | 415.94 ms | 163.06 ms | 252.88 ms (60.8%) |
| 22 | 8 | 798.52 ms | 217.40 ms | 581.12 ms (72.8%) |
| 128 | 1 | 105.22 ms | 104.64 ms | 0.58 ms (0.6%) |
| 128 | 4 | 449.66 ms | 182.62 ms | 267.04 ms (59.4%) |
| 128 | 8 | 860.41 ms | 240.79 ms | 619.63 ms (72.0%) |

Every measured multi-query pair improved: four-query savings ranged from
246.35–279.06 ms and eight-query savings from 579.17–703.53 ms. Single-query paired
differences ranged from a 1.34 ms slowdown to a 3.41 ms improvement, consistent
with no material single-query change. The earlier experimental shared context
independently showed 59.5–60.9% and 72.2–73.3% median reductions respectively.

Opening accounts for only part of the gain: eight-query opening time fell from
58.76 to 7.32 ms for 22 facts and 89.95 to 11.54 ms for 128 facts. Most of the
savings occur during search with the instance reused; these measurements do not
separately attribute that remainder to individual upstream caches.

Records, order, metadata and coverage disclosures matched. The first experimental
run rejected an exact-bit score comparison. The shared path differed by at most
`1.1102230246251565e-16`; repeated-read controls showed zero difference in these
samples. Subsequent comparisons retained exact payload/order checks and recorded
score deltas separately. Production thresholds and existing retrieval expectations
were not changed. The new regression compares scores to 12 decimal places.

The implementation extracts the existing search loop once for standalone reads
and batches. It opens one handle per batch under the unchanged project lock,
checks expiry/supersession per query, and closes on every exit. No resident cache,
new dependency, IPC operation or checkpoint format was added. A permanent test
compares the worker's actual interleaving with the prior repeated-read control,
checks closure after a failure on the second query, preserves best-effort versus
required behavior, and proves a later batch observes a host mutation. Existing
inactive-result ceiling checks now exercise both entry points.

Raw samples, initial failure, source snapshots and the temporary benchmark method
are retained in `~/.mirket/work/mem0-recall-osb20me5`. Temporary instrumentation was
removed from the production suite. These are warm synthetic batch results, not a
claim about end-to-end task speed, natural adoption or general answer quality.

## Remaining friction: prioritize by measured opportunity

| Priority | Observed structure | Smallest next investigation or change | Acceptance boundary |
| --- | --- | --- | --- |
| P2 | Standalone memory/lifecycle calls start a contained process; workers already retain a session | Separate process startup, embedding initialization and search timing on representative calls | Do not add a resident service or replace Python without demonstrated benefit; retain admission, cancellation and project isolation |
| P2 | Each project operation takes an exclusive storage lock and scans the private tree for size/symlinks | Measure lock wait and scan cost against store size | Keep the bounded storage and path contract; do not replace it with an unchecked cache |
| P2 | Synchronous checkpoints capture substantial worker snapshots before and after effects | Measure bytes and latency over a representative long run | Preserve intent/result boundaries, restart, CAS, verification and original budgets; do not merge away crash-recovery states |
| P2 | Broad source-generation binding rejects resume after unrelated compiled-input changes | Record how often this prevents useful recovery before narrowing identity | Any narrower contract must still bind executable behavior, selected guidance, tools and model execution capabilities |
| P3 | Natural memory/durable-worker adoption remains unmeasured | Use relevant task opportunities: explicit stable fact, later recall, interrupted worker, and task needing neither | Separate first attempt from intervention, invocation from useful contribution, and harness from host fallback |

These are bounded candidates, not measured regressions or reasons to retain
everything indefinitely. No hot-path change was made from speculation alone.
The targeted recall optimization above addresses the measured opportunity;
further runtime changes need their own evidence, not a language migration goal.

## Graphiti assessment

Reviewed upstream commit `8e320f880e7d169a907dbf2a7ac4188586c03b55`.
Its manifest reports `graphiti-core` 0.31.2 and Python 3.10–3.x. The base manifest
includes Neo4j, OpenAI and PostHog clients; optional extras add other stores and
models. This is a manifest inspection, not an installed transitive-size or
vulnerability comparison. Switching to it would not remove Python.
[Pinned manifest](https://github.com/getzep/graphiti/blob/8e320f880e7d169a907dbf2a7ac4188586c03b55/pyproject.toml).

Graphiti's distinctive value is temporal entities/relationships with episode
provenance and hybrid semantic, lexical and graph retrieval. This could help
questions spanning changing relationships and historical validity. Supported
storage includes embedded FalkorDBLite, so an external database server is not
always required. Kuzu is deprecated in the reviewed source and is not a sound new
default. The reference server is not an authorization boundary; a Mirket
integration would still require its existing scope and permission owner.
[Pinned overview](https://github.com/getzep/graphiti/blob/8e320f880e7d169a907dbf2a7ac4188586c03b55/README.md).

The normal episode path uses model-driven extraction and resolution. Direct
triplet ingestion still supplies embeddings and resolves entities/edges; it is
not established as equivalent to Mirket's `infer=False` contract. Defaults use
OpenAI clients, while configured alternatives can be local. Do not infer that
every search needs a paid generation call.
[Ingestion and client construction](https://github.com/getzep/graphiti/blob/8e320f880e7d169a907dbf2a7ac4188586c03b55/graphiti_core/graphiti.py).

The optional GLiNER2 client handles local entity extraction, but delegates relation
extraction, deduplication and summarization to its required wrapped LLM client.
It therefore does not make the full graph ingestion path inference-free.
[GLiNER2 implementation](https://github.com/getzep/graphiti/blob/8e320f880e7d169a907dbf2a7ac4188586c03b55/graphiti_core/llm_client/gliner2_client.py).

| Mirket need | Current Mem0 integration | Graphiti implication | Decision |
| --- | --- | --- | --- |
| Explicit concise project facts without extraction inference | Implemented and tested | Different ingestion semantics require adaptation and evidence | Keep Mem0 |
| Local offline retrieval | Pinned embeddings, local store, network-disabled runtime | Configure and package a graph store plus approved model stack | No demonstrated simplification |
| Historical relationship and multi-hop queries | Not the retained simple-fact contract | A plausible unique benefit | Reconsider only for real temporal/relationship task opportunities |
| Scope, expiry, supersession, forgetting, export and backup | Mirket adapters enforce these contracts | Migration and equivalent failure semantics must be built and verified | No drop-in replacement |
| Worker execution recovery | LangGraph owns it | A knowledge graph is not an execution checkpointer | Keep LangGraph independently |

Recommendation: do not install or migrate to Graphiti now. If actual graph tasks
appear, compare the same explicit facts plus a small relationship/history fixture.
Require useful answers and appropriate abstention; measure ingestion failures,
repair, calls, latency, memory and disk. Preserve IDs/provenance, supersession,
expiry, forgetting, export/import and restore. Bind approved providers/models
explicitly. Upstream performance claims are not Mirket benchmarks. No Graphiti
runtime or model was executed in this review.

## Opportunistic Rust assessment

| Boundary | Why it is not an easy useful migration now | Disposition |
| --- | --- | --- |
| CLI, MCP, routing, installation, isolation, backup | Already Rust | Keep |
| Mem0 adapter and store | Native rewrite must replace retrieval, history, local-store semantics and migration; moving IPC glue leaves the dependency closure | Keep Python |
| LangGraph graph/checkpointer | Reimplementing its journal, interrupts and replay semantics replaces LangGraph rather than integrating it more deeply | Keep Python |
| OpenPencil and design JS engines | Chosen engine and retained tools use the Node ecosystem; a Rust wrapper still needs them | Keep Node |
| resvg and Typst | Native Rust engines already supply the rendering/typesetting work; wrappers preserve the shared tool contract | Keep current owners |
| Raster, browser and Flexbox operations | Alternative libraries require codec, color, layout, font and interaction equivalence; no mechanical drop-in established | No rewrite for language count |
| Evaluation and inventory scripts | Development tooling, not a Python requirement for the core installed CLI | Fix concrete drift, not language purity |

The reviewed official [Mem0 documentation](https://docs.mem0.ai/open-source/overview)
describes Python/TypeScript interfaces; no verified drop-in Rust implementation
was identified. [LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence)
explains the checkpoint/thread ownership used here. The chosen
[OpenPencil](https://github.com/open-pencil/open-pencil) must not be confused with
other similarly named repositories.
[Playwright's supported languages](https://playwright.dev/docs/languages) do not
make a Rust CDP client behaviorally equivalent. Existing native
[resvg](https://github.com/linebender/resvg) and
[Typst](https://github.com/typst/typst) already capture much of the relevant Rust
benefit without rewriting the harness.

## Verification and scorecard

The initial integration baseline used the already prepared Linux ARM64 image,
synthetic disposable state and no network, user memory or paid inference.
Recipe: `93d935f6ac439688fe7df76be0af48a9ff9520d349331a8cf19223703fb5dbc8`.
Versions: Mem0 2.2.1, LangGraph 1.2.16, SQLite checkpointer 3.1.2, FastEmbed 0.9.0.
All seven reported runtime source/lock hashes matched the pre-optimization files.
The later recall optimization changes memory.py, workflow.py and their test suite;
the baseline below retains its original scope.

The package test returned `ok: true`. Its 13-test suite covers memory persistence,
scope, retries, expiry, supersession/interrupted publication, pagination,
inactive-result overfetch, checkpoint restart, stale-checkpoint rejection, effect
and host handoffs, bounded pauses, lifecycle recovery and containment.

| Measurement | Result | Limit |
| --- | --- | --- |
| Synthetic retrieval | 22 facts, seven queries; six answerable recall@3 values of 1; one unanswerable returns no results | Small retrieval fixture; not answer quality, adoption or NLP equivalence |
| Query timings | 98.10–304.45 ms | Within one contained fixture; not matched cold-start or before/after performance |
| Fixture duration | 91.89 seconds | Includes tests and fixture work |
| Peak child RSS | 942,708 KiB in Linux fixture | Not VM RSS or steady-state production usage |
| Contradiction query | New and old color facts both retrieved | Explicit supersession has a separate test; this result is not automatic contradiction resolution |
| Injection fixture | Adversarial text was retrieved as expected | No prompt-injection resistance or safe answer-generation claim |

| Final check, including recall optimization | Result |
| --- | --- |
| Native gate on macOS ARM64 | Formatting and Clippy passed; 424 tests passed, zero failed, eight opt-in tests ignored |
| Contained agent runtime on Linux ARM64 | All 14 tests passed, including the new batch equivalence, closure, mutation and failure regression; all seven reported runtime source/lock hashes match |
| Rebuilt design image on Linux ARM64 | 70 tests passed, zero skipped; includes real browser/rendering, private child environment and process cleanup |
| Catalog and installation | Catalog valid; setup and doctor passed for Codex and Claude in a disposable home; MCP protocol/identity checks passed |
| Evaluation integrity | 15 dependency-audit tests and five remediation tests passed; synthetic quality-packet preflight passed with zero model executions |
| Generated map | `--check` passes; source archive without `.git` and changed Git metadata pass; changed source bytes fail as expected |
| Repository consistency | Local-link gate and whitespace check passed; no references to the deleted helper paths remain |

The final candidate executable SHA-256 is
`35f464853f0581ef5ea0e9f0460401a69d661acd3aa150ba15d2881b871e3531`.
Tested design image:
`sha256:19ce5efbd9d52e34c09d146b4f8661eb2211fbaa9dc792f4d50a183ee6866ae0`.
Final agent recipe:
`80c46911da5126fc80d67abe4debe89831f4f668f386ddbdc20b8ae5db306671`.
Final checks are recorded in `~/.mirket/work/mem0-recall-osb20me5/final-verification.json`.
The first new batch regression tried to inspect Mem0's connection attribute after
close had cleared it. The corrected test retains the original connection and
verifies that queries raise after closure; the final full suite passed.

The installed user binary remains unchanged. The first native run caught links
to this document before it had been written; the completed files passed the full
rerun. Both logs are retained. No check or threshold was relaxed.

Before cleanup, unchanged source became a stale inventory after a commit, five
retired video environment keys remained, and two obsolete generators could
mislead a new review. Those concrete issues are removed. Runtime dependency
counts are unchanged. The later recall measurements establish a bounded warm
batch latency improvement; no cost, memory, startup or answer-quality gain is claimed.

Evidence and pinned upstream source snapshots
are outside shipped source in `~/.mirket/work/runtime-review-ria6v6ir`.
The previous committed implementation's checks remain in
`~/.mirket/work/harness-hardening-8n431owk`.

No new production dependency, provider change, persistent worker, desktop
reconfiguration or data migration occurred. Windows/Linux native activation,
live desktop reconnection, natural selection and a full candidate image advisory
scan are not established by these local checks. Repository inspection is not a
proof that no future drift or inefficiency exists.
