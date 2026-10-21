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
Source fingerprint of the completed recall optimization (before the follow-up below):
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


## Pre-release follow-up

This implementation pass starts from clean `c5ecde3`. The subsequent authorized
follow-up activates the matching design stack and commits the changes. Publication,
push, main-CLI replacement and provider inference remain outside this pass. No library is exempt from replacement;
retention below means the current consumer needs its behavior, not a permanent
architectural commitment.

### Adoption and owned authority

Host entry guidance now directs focused Mem0 recall for recurring decisions,
verified stable-fact retention with provenance and supersession, and LangGraph
resume after interruptions. The canonical method is `context-management`, also
searchable by Mem0, LangGraph and interrupted-worker terms. Self-contained work
needs no memory ceremony. Worker results and recovery status disclose recall
availability/count/coverage without duplicating private facts or queries.

Mirket owns its managed configuration and routed dispatch. Passive runtime status
now gives actionable reconnect/doctor diagnostics for mismatched or unreadable
identity/selection. Proactive repair stays within existing authorization through
the owning commands. It does not confer device-wide permissions, install packages,
start a VM, schedule background work or intercept host-native tools.

One reproduced compliance loophole was repaired: `openpencil_inspect` schema lookup
could count as required document inspection. Saved-document inspection now returns
a consumed-source digest, checked against the scoped current file by Rust; only
that receipt satisfies the existing worker requirement. Discovery remains useful
and successful, without pretending it inspected a document. Existing artifact,
visual, recovery and host-verification checks remain in place.

### Runtime dependency decisions

A fresh, network-disabled Linux ARM64 trial read installed distribution metadata
and traversed active requirement markers/extras from all six agent roots. All 97
distributions are reachable; there is no orphan distribution to safely uninstall
from this exact supported closure. This is dependency evidence, not proof that
every feature of each library is useful. Recorded file sizes total 427.38 MiB,
excluding models, base OS, image compression/layers, private data and RSS.

| Component or group | Actual consumer / unique value | Decision and removal consequence |
| --- | --- | --- |
| Mem0, local Qdrant, SQLAlchemy/history | Explicit scoped facts, hybrid retrieval and mutation history | Keep. A replacement must preserve provenance, expiry, supersession, idempotency, pagination and export/migration; no current drop-in demonstrated. |
| LangGraph, SQLite checkpoint adapter and required LangChain/SDK closure | StateGraph, durable interrupts, checkpoint CAS and budget-preserving effect recovery | Keep. Do not delete SDK/prebuilt distributions merely because Mirket calls StateGraph; the installed package declares them required. A narrower supported upstream distribution is a valid future replacement. |
| FastEmbed, ONNX Runtime, tokenizer, NumPy, multilingual model and BM25 assets | Offline dense semantics and lexical retrieval | Keep. Lexical-only removes semantic capability; replacing embeddings requires reindex/migration and representative retrieval evidence. |
| spaCy, English model and exclusively reachable NLP closure | Lemmatization/entity preprocessing in Mem0 | Best reduction candidate: 27 distributions, 171.42 MiB of recorded files exclusive to these roots. Fresh four-arm trial still does not establish broad noninferiority; do not strip this closure in this release. |
| OpenAI/PostHog/LangSmith clients | Required by Mem0 / LangChain metadata; extraction/tracing/network disabled in owned runtime | No standalone uninstall. Removing them requires a supported lean upstream boundary or maintained fork and import/metadata tests; a disabled service is not an orphan package. |
| FastEmbed → Pillow | Required pinned closure despite text-only Mirket embedding use | Keep compatible constraints. The fresh 732-coordinate audit still reports nine advisory findings; Pillow 13 conflicts with the current FastEmbed constraint. Do not break the lock or fork merely to clear advisories. |
| MCP SDK, AJV | Design transport and bounded argument validation | Keep; actual imports in server/OpenPencil adapter. Removing them changes protocol/schema enforcement. |
| OpenPencil and its CLI/core/CanvasKit/Yoga closure | Saved editable documents and lazy operation schemas | Keep, with stronger inspection evidence. Any replacement must prove creation, existing-document edit, save/reopen, IDs, tokens and exports. |
| Satori + resvg | Static Flexbox/text layout plus native vector previews | Keep now. Browser replacement requires matched font/wrapping/layout and delivery-contract trials; resvg alone does not supply layout. |
| Sharp | Intentional raster crop/resize/format conversion | Keep; input staging is not an equivalent transformation. |
| Odiff | Independent comparison of saved pixels and dimension failures | Keep; renderer-local comparison does not replace its standalone contract. |
| SVGO + saxes + css-tree | Optimization and SVG/XML/CSS validation across several retained renderers | Keep libraries. Standalone optimizer consolidation remains possible only preserving its larger input contract and saved consumers. |
| Playwright/Chromium, Typst, fonts | Working DOM inspection; paged output; reproducible baseline glyphs | Keep. Editable files do not replace working UI or paged typesetting. Additional licensed fonts remain task inputs. |
| Kroki / Excalidraw | Actual graph layout / optional specialized syntax | Keep outside core defaults; Excalidraw remains explicit opt-in with old selections preserved. No service was started by this review. |
| Video, Python pip and Node package managers in runtime layers | Already removed from shipped runtime by prior implementation | Keep excluded. No new removal or image-size saving claimed in this pass. |

The fresh 23-fact/15-query comparison again obtained answerable recall@3 of 1.0
in hybrid+NLP, hybrid-no-NLP, dense and lexical arms. The first three returned
nothing for only one of two unanswerable cases; lexical for both. It exercises
identifiers, inflections, multilingual facts, supersession and expiry, but remains
a small synthetic fixture, not proof that NLP or semantics can be removed. Runtime
metadata and raw results are retained outside shipped source in the task evidence
directory. Temporary metadata instrumentation was restored after the trial.

### Designer capability and remaining release claims

The existing guidance covers art direction, typography proof, references and asset
rights, semantic diagrams, editable masters, source/export handoff and located
critique. All twelve direct design npm libraries have active consumers. Adding a
new design framework or a larger reference catalog would not resolve the missing
quality evidence. The useful retained outcome boundaries are:

| Outcome | Proof required from the actual task | Remaining limit |
| --- | --- | --- |
| New and existing editable design | Saved .fig, reopened layers/IDs, final export and inspected pixels | A schema manifest or simple shape test cannot certify all operations or complex imports. |
| Web interface | Real states, keyboard/reflow, narrow/wide rendering, actual assets and text | Screenshot availability does not establish usability or visual craft. |
| Static/vector/raster assets | Editable master where requested, typography/alpha/use-size inspection and coherent family | Rich raster creation remains a host capability, not a worker renderer. |
| Diagram | Source semantics checked against every rendered relationship | Graph layout success is not semantic accuracy; optional services must be explicitly available. |
| Paged output | Every final page inspected, fonts/assets and destination requirements verified | Generic RGB/PDF output is not press certification; specialized print/prepress is host-owned. |
| Video or specialized native/3D work | Explicitly selected capable host tool and actual final artifact | No bundled encoder or general native/3D production promise. |

The earlier negative design-quality evaluation remains valid historical evidence.
This pass improves deterministic inspection fidelity and host guidance, not a
measured aesthetic or natural-selection gain. No new paid model campaign was run.

### Next decisions in order

1. Exercise the changed candidate through native, runtime and disposable-host
   checks before release. Reconnect each actual desktop only after an authorized
   installation; candidate acceptance is not installed-client activation.
2. Run bounded natural task opportunities on an authorized host: recurring project
   decision/retention, interrupted worker, new/editable design, live interface,
   static/vector, diagram and paged artifact, plus self-contained and unavailable
   cases. Distinguish first attempt from repair, correct invocation from useful
   contribution, host fallback from harness output, and report opportunities as
   denominators. Use existing evaluation contracts and preserve negative results.
3. Decide the NLP reduction with a separate held-out corpus of realistic project
   facts, difficult identifiers/inflections/multilingual queries and negatives.
   Freeze relevance labels before comparison; require no lost critical facts,
   unchanged expiry/supersession/isolation, and a tested reindex/rollback path.
   If it fails, retain NLP and record the failing cases rather than expanding an
   open-ended optimization project. This is the largest evidenced reduction lead.
4. Trial a browser-based replacement for Satori only on actual static-layout
   opportunities. Preserve editability, fonts, input limits and saved selections;
   if equivalent, clean the dependency/lock/image/schema/registry/guidance/tests
   together. Do not remove shared SVGO or introduce a Rust port merely for counts.

### Follow-up verification and scorecard

Candidate: macOS ARM64, source
`f6ea4b36cb91572ba8e380f0aa5f9bd839ae270be31a3cef168b8d3981911616`, binary
`b67a3dd931734e1bb6e883d5ecc0291b3a77917aa4e6e9d0addde59153f5aec9`.
Evidence: `~/.mirket/work/pre-release-_x90jv2f` (logs, dependency graph/trial,
matching candidate, disposable setup and rendered OpenPencil specimen).

| Check or outcome | Before | This pass |
| --- | --- | --- |
| Native gates | 424 passed, 8 opt-in ignored | Formatting/Clippy and 426 passed, 8 opt-in ignored; includes schema-discovery false-positive and recall-disclosure regressions |
| Design runtime | 70 passed | 70 passed, no skips; real saved OpenPencil inspection digest and schema-only absence checked. Saved 480×240 solid-color specimen inspected; not a design-quality benchmark |
| Agent runtime | Real Mem0/LangGraph integration already exercised | Contained runtime and fresh four-arm retrieval comparison passed; subsequent complete acceptance passed against the activated design stack |
| Recovery | Prior checkpoint/backup tests | Explicit backup-runtime test passed: history, pending effect, relocation and recovery with disposable state |
| Host delivery and inventory | Prior source | Catalog valid; both-target disposable setup/doctor passed; generated map matches source |
| Evaluation tooling | Prior check results | 15 dependency-audit tests, 5 remediation tests, synthetic quality-packet preflight passed; zero model executions |
| Dependency audit | Nine untriaged Pillow findings | 732 coordinates queried; same nine findings, three existing zip exceptions, no stale reviews. No fresh OS/image scan claimed |
| Installed-stack acceptance | Initial attempt failed: design stack absent/drifted | Matching stack activated through the supported stop/prepare/start path; all five full-acceptance checks passed and a candidate-bound receipt was created |
| Dependency/image savings | Prior video/package-manager removal | No new removals or claimed savings; measured exclusive NLP metadata footprint is a candidate, not reclaimed space |
| Natural adoption / designer quality | Historical weak/negative evidence | Still unmeasured; guidance and artifact enforcement improved, no inferred quality gain |

An initial design-check invocation rejected the older bootstrap fingerprint after
final guidance changed. Rebuilding the matching executable resolved that specific
failure; its isolated design suite then passed. Initial full acceptance reached
fresh MCP/skills and real memory/pagination/concurrency checks, then failed at
installed design launch. That failure is retained in the evidence rather than
presented as a code defect or a passed gate.

The subsequent user-authorized activation used the installed CLI to stop the old
design services without purging data, then the matching candidate to prepare and
start them. Schema-3 `core`/`diagrams` selection migrated to schema-4
`core`/`diagrams`/`excalidraw`, preserving the old companion behavior. The workspace
and Kroki port were retained. The active tools image is
`sha256:46395b3f3ba702a37b37bbceaf3f0fa335f33e116c195c08dc570e42c00b05cb`.
A direct OpenPencil response matched all 14 reported runtime files to source.

Full acceptance then passed: contained runtime, desktop contracts, parallel
runtime, design output and backup recovery. `acceptance-activated.json` in the
same evidence directory binds the source and executable digests above. This
resolves the installed-stack acceptance blocker without lowering a check or
changing source to bypass the prerequisite. The unchanged source also retains
the passing 426 native/70 design checks. No further code repair was needed.

Saved setup choices, client configuration and the installed main CLI were not
replaced. These checks used fresh candidate processes; they do not establish
reconnection of existing desktop sessions or aesthetic acceptance. Source changes
and this verification record are committed together locally. Nothing was pushed
or published.
