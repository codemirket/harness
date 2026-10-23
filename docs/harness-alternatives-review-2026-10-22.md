# Harness efficiency and alternatives review — 22 October 2026

Mirket's strongest architectural choice is its explicit Rust ownership of permissions, installation, dispatch and evidence. Its largest remaining efficiency opportunities are narrower: retrieval filtering, serialized memory operations, repeated worker state transfer, document edit granularity, and browser compatibility. A framework replacement is not the first remedy for most of these.

This is a source review and internet research report, not an implementation or comparative benchmark. Baseline: checkout `0293dce`, Mirket 0.0.2. The working tree was clean at entry. No services, VM, model inference, installers or third-party helpers were started. No production dependencies were added. Recommendations below require separate implementation decisions. Dates on historical reports are preserved; their resolved defects are not presented as current defects.

**Evidence labels:** observed = inspected current code; historical measurement = existing repository experiment, not rerun; hypothesis = plausible cost requiring measurement; candidate = researched alternative without demonstrated Mirket superiority.

## Decision priorities

| Priority | Decision | Evidence | Expected benefit / remaining uncertainty |
| --- | --- | --- | --- |
| 1 | Filter inactive facts before retrieval ranking | Observed correctness/coverage limitation | Avoid candidate starvation; migration must preserve interrupted supersession semantics |
| 1 | Treat Playwright and Chromium as a tested version pair | Observed packaging choice + upstream warning | Reduce compatibility and rendering variance; no current failure reproduced |
| 1 | Separate artifact appearance checks from interoperability and accessibility checks | Observed tool boundary + upstream limitations | Catch failures screenshots cannot establish |
| 2 | Batch editable-document mutations within one bounded transaction | Observed per-call process/save/reopen | Fewer process and serialization cycles; measure actual latency |
| 2 | Profile memory startup, lock wait and tree scanning independently | Observed serialized path | Locate dominant cost before adding a daemon |
| 2 | Measure checkpoint bytes and remove redundant full-snapshot transfer | Observed repeated snapshots | Lower IPC/storage overhead while retaining effect boundaries |
| 2 | Make reservation units and context budgeting explicit | Observed bytes-as-reservation proxy | Avoid misleading token terminology and premature admission failure |
| 2 | Compare explicit-fact memory against FTS5 and FTS5 + vectors | Candidate architectural simplification | Determine whether Mem0's additional machinery earns its cost |
| 3 | Trial D2 for architecture diagrams | Candidate narrower producer | Potentially avoid a service for a common output family |
| 3 | Evaluate optional extraction/accessibility capabilities | Candidate additions | Fill concrete deliverable gaps without broadening every worker |
| 3 | Measure guidance selection and marginal usefulness | Current inventory is not outcome evidence | Reduce maintenance/context burden where demonstrated |

## 1. Memory is the most defensible place to challenge the current stack

The current path is Rust → contained Python → Mem0 → local Qdrant/history, with FastEmbed, ONNX assets, BM25 and spaCy. Automatic extraction is deliberately disabled: `ExplicitFacts.generate_response` rejects inference and writes use `Memory.add(infer=False)`. See [memory.py](https://github.com/codemirket/harness/blob/0293dce091771398520e6151a86906d0bb48f663/runtime/agent/memory.py), [requirements](https://github.com/codemirket/harness/blob/0293dce091771398520e6151a86906d0bb48f663/runtime/agent/requirements.in) and [model manifest](../runtime/agent/model.json).

This configuration uses real Mem0 functionality: vector storage, history and hybrid retrieval. It is not a decorative dependency. I fetched the exact `mem0ai==2.2.1` wheel named in the lock, verified its SHA-256, and inspected its Qdrant adapter without importing it. It includes a BM25 sparse slot, lazy sparse encoder and a legacy-collection fallback. Therefore “BM25 is unused” would be an unsupported criticism. Upstream also documents explicit raw insertion through `infer=False`. [Mem0 source documentation](https://github.com/mem0ai/mem0/blob/main/docs/core-concepts/memory-operations/add.mdx).

The architectural concern is fit: Mirket already owns provenance, scopes, expiry, supersession, idempotency, export and recovery, while disabling extraction. It is reasonable to ask whether a narrower fact repository could preserve these semantics with fewer adapters. The 97-wheel closure belongs to the whole agent runtime, not Mem0 alone; removing Mem0 does not imply removing 97 packages or Python.

### Concrete remaining inefficiency: post-filtered inactive facts

`_search` retrieves candidates, checks expiry/supersession afterward, and retries with increasingly large candidate sets up to 256. Active matches can remain behind inactive records. The existing `test_search_expands_past_inactive_candidates_and_reports_ceiling` explicitly covers 300 stale candidates preceding a current fact. Reporting `incomplete` is honest but does not restore the missing fact. Each candidate also incurs active-state checks against history metadata. [Implementation](https://github.com/codemirket/harness/blob/0293dce091771398520e6151a86906d0bb48f663/runtime/agent/memory.py), [regression fixture](https://github.com/codemirket/harness/blob/0293dce091771398520e6151a86906d0bb48f663/runtime/agent/test_runtime.py).

**Recommended first change:** maintain a queryable active-state projection or an active-only retrieval index, leaving immutable history/export separate. Expiry needs a query-time condition or reliable expiry maintenance. Supersession needs reconciliation across storage boundaries: a crash between vector mutation and publication must never expose a half-published replacement. Do not merely increase the ceiling.

**Acceptance:** a relevant current fact remains retrievable behind thousands of expired/superseded facts; a crash during replacement preserves the prior publication contract; forgetting and restored backups produce the same visible set. Compare recall and abstention, not only throughput.

### Alternatives worth comparing

| Option | Why it fits | Cost / reason not to select blindly | Recommendation |
| --- | --- | --- | --- |
| Existing Mem0/Qdrant, with active filtering | Lowest migration risk; existing history and restore paths | Retains adapter/dependency complexity | First production candidate |
| SQLite FTS5 alone | Explicit short facts, SQL lifecycle filtering, lexical/BM25 baseline | Paraphrases and cross-language queries may regress | Required experimental control |
| SQLite FTS5 + sqlite-vec | Facts, metadata and retrieval can share a transactional database | Extension is pre-v1; filters have specific supported operators; model runtime remains | Best simplification challenger |
| Direct Qdrant adapter | Preserve current vector engine while removing memory abstraction | Mirket assumes more history/lifecycle responsibilities; may save little dependency weight | Intermediate comparator |
| LanceDB | Hybrid vector/full-text retrieval and reranking for richer corpora | Another storage/indexing architecture; excessive for a small fact set | Reconsider for document-scale retrieval |
| Graphiti | Temporal relationships and multi-hop knowledge | Changes ingestion/storage semantics and adds graph complexity | Reject as the default fact-store replacement |

FTS5 documents BM25 and Unicode/trigram tokenizers; none is a promise of Turkish morphology or multilingual semantic equivalence. sqlite-vec supports metadata and partition columns but explicitly warns about pre-v1 changes. LanceDB documents combined full-text/vector retrieval. Graphiti's value is relationship-oriented retrieval, which the current concise-fact contract does not establish a need for. [FTS5](https://sqlite.org/fts5.html), [sqlite-vec filtering](https://alexgarcia.xyz/sqlite-vec/features/vec0.html), [stability warning](https://alexgarcia.xyz/sqlite-vec/api-reference.html), [LanceDB](https://docs.lancedb.com/search/hybrid-search), [Graphiti](https://github.com/getzep/graphiti).

### Embeddings: compare retrieval suitability, not leaderboard position

The current manifest pins a 384-dimensional multilingual MiniLM model with a 128-token window and a 235,052,644-byte ONNX asset. The adapter rejects overflow rather than silently truncating. This is safe but forces short facts/queries and complicates recall query construction.

**First model challenger:** `intfloat/multilingual-e5-small`, using its required query/passage prefixes and 512-token window. Its model card identifies multilingual retrieval behavior and low-resource-language limitations. It is a plausible better fit, not a measured winner. Changing embeddings requires reindexing, recalibrating thresholds and testing score/order behavior; equal dimensionality does not make vectors compatible. [Model card](https://huggingface.co/intfloat/multilingual-e5-small/raw/main/README.md).

**Second challenger:** EmbeddingGemma 300M when longer inputs demonstrably matter. Its model card describes multilingual embeddings and a larger input window, but its Gemma license and model/runtime footprint require a deliberate review. Do not select it solely because it is newer. [Model card](https://huggingface.co/google/embeddinggemma-300m).

`fastembed-rs` is a credible local Rust embedding implementation to investigate, including text/sparse embeddings and reranking. It is not a Rust replacement for Mem0 or LangGraph, and ONNX/model packaging remains. Test tokenizer, normalization, offline loading, quantization and ARM64 behavior before claiming parity. [Project](https://github.com/Anush008/fastembed-rs).

## 2. Memory operations serialize more work than their names suggest

`bridge.py` obtains an exclusive project lock, enumerates the private tree, rejects symlinks and sums file sizes before yielding the project directory. Standalone calls start contained processes; a worker already retains its session. This gives clear ownership but couples unrelated state operations to scans and serialized storage access. [Bridge](https://github.com/codemirket/harness/blob/0293dce091771398520e6151a86906d0bb48f663/runtime/agent/bridge.py), [runtime review](runtime-integration-review-2026-10.md).

The existing shared-handle optimization should stay. Historical warm synthetic measurements show four-query batches falling from roughly 416–450 ms to 163–183 ms and eight-query batches from 799–860 ms to 217–241 ms. Those numbers exclude container startup and inference. They do not justify a new resident service by themselves.

**Next experiment:** separate queue time, container startup, imports, embedding initialization, tree scan, store opening and search. Use empty, 100-, 1,000- and 10,000-fact stores and checkpoint-heavy trees. Observe median and tail latency, RSS and idle cost.

If startup dominates frequent calls, a project-scoped idle-expiring session pool is a candidate. If scans dominate, evaluate incremental accounting plus periodic full verification. Preserve symlink rejection, storage bounds, cancellation and isolation. A permanently resident multi-project daemon is a larger security/lifecycle change and should not be the first optimization.

## 3. Durable execution has a useful framework but overlapping responsibilities

The Python graph has four nodes: recall, worker, effect and verification. Rust owns the actual worker loop, tools, model calls, artifacts and much of the resumability policy. Both languages interpret budget/pause state. Full worker snapshots include transcript, attempts, artifacts, image records, tool completion and quality-gate state. The Rust loop advances a full snapshot at turn entry and again before inference, in addition to later effect transitions. [Workflow](https://github.com/codemirket/harness/blob/0293dce091771398520e6151a86906d0bb48f663/runtime/agent/workflow.py), [durable adapter](../src/router/durable.rs), [execution loop](../src/router/execution.rs).

This is an observed maintenance and serialization surface. It is not proof that LangGraph is slow or that checkpoints are redundant. Intent-before-effect and result-after-effect must remain distinct. LangGraph's documented persistence provides real checkpoint/recovery value. [Persistence documentation](https://docs.langchain.com/oss/python/langgraph/persistence).

**Near-term recommendation:** profile snapshot bytes and checkpoint latency; use shared transition conformance fixtures for Rust/Python; investigate delta records plus periodic snapshots. Retain compare-and-set, pending-effect reconciliation, budget conservation, input identities and host verification revocation.

| Recovery alternative | Assessment |
| --- | --- |
| Keep LangGraph, reduce redundant transfer | Best immediate option; smallest recovery migration |
| Owned Rust transition journal in SQLite | Strong long-term simplification hypothesis because Rust already owns effects; substantial correctness burden |
| DBOS | Credible lightweight comparator: Python workflows/steps with default SQLite and automatic recovery; still Python and a new replay/migration contract |
| Restate | Rust SDK and durable service execution are relevant, but a separate server/service topology is a poor immediate fit for this local harness |

DBOS is more relevant here than reflexively adding a distributed workflow platform. Its documentation requires non-deterministic operations to be handled through the appropriate durable steps. Neither DBOS nor a homegrown journal makes arbitrary external writes exactly-once. Restate's repository documents its server and multiple SDKs. [DBOS integration](https://docs.dbos.dev/python/integrating-dbos), [workflow semantics](https://docs.dbos.dev/python/tutorials/workflow-tutorial), [Restate](https://github.com/restatedev/restate).

**Migration acceptance:** crash before intent, after intent, after effect but before result, during host verification and during restore; prove no unsafe replay and no reset of turns/repair/time budgets. Retain old-run readability or an explicitly supported drain/migration path.

## 4. Reservation accounting uses bytes under a token label

The current worker computes reservation as `prompt.len() + images.len() * 16384 + max_completion_tokens`, then accumulates reservations across attempts. In Rust, string length is bytes. This is a conservative admission proxy mixing units, not actual token consumption; it can overreserve long or multilingual prompts and unused output capacity. Provider tokenization and image pricing differ, so replacing it with a universal bytes/4 formula would also be wrong. [Execution](../src/router/execution.rs), [limits](../src/router/policy.rs).

**Recommendation:** document/rename the conservative unit, or implement provider-specific token estimates with explicit unknown cases. Separate maximum exposure reservation from observed completion consumption. Keep bounded execution and explicit model choice. Compare stop reasons on representative tasks before relaxing limits.

This is not a recommendation to restore a billing dashboard. The repository records a user decision to remove that feature. Current source still retains per-attempt numeric usage diagnostics; historical removal text should be reconciled with current behavior. [Recorded product decision](harness-accounting-removal-2026-10.md).

## 5. Browser and automation versions are independently selected

The design image installs Chromium through a dated Debian snapshot, while npm separately pins `playwright-core`. The launcher invokes `/usr/bin/chromium` through the owned guard. Reproducible package bytes do not establish a supported browser/protocol pair. Playwright explicitly cautions that it works best with its bundled Chromium and does not guarantee other versions. [Dockerfile](../runtime/design/tools/Dockerfile), [launcher](../runtime/design/tools/chromium-launcher), [upstream contract](https://playwright.dev/docs/api/class-browsertype).

**Recommendation:** prefer a pinned Playwright browser revision prepared at image-build time, provided it meets the image's security/update requirements. Alternatively, deliberately retain Debian Chromium and record a tested pair with a compatibility fixture. Never download browsers during serving. Preserve the guard, sandbox constraints, fonts, 2x capture and network restrictions.

**Acceptance:** navigation, fonts, keyboard/focus, reduced motion, rapid reversal, cancellation, renderer cleanup and pixel baselines on the exact pair. This review did not reproduce a mismatch failure.

## 6. OpenPencil needs coarser edits and consumer-level verification

Each edit stages the source, starts a CLI mutation, serializes a `.fig`, starts another CLI to read the saved tree, then publishes the file. IDs may remap during serialization. Many tiny edits therefore cause repeated process launches, document parsing and tree readback. Existing batch operations may already cover some cases; prefer those before adding a new API. [Adapter](../runtime/design/tools/openpencil.mjs), [operation inventory](../runtime/design/tools/openpencil-tools.json).

**Recommendation:** make one bounded batch the normal authoring unit. If current batch operations are insufficient, add a validated transactional edit batch: one open, bounded operations, one save, one reopened-tree result and one atomic publication. Preserve operation schemas, limits and failure rollback; do not expose arbitrary evaluation as a shortcut.

A separate risk is interoperability. Upstream explicitly warns that complex Figma files can lose component, color or text fidelity when reopened in Figma, even without edits. Mirket's successful OpenPencil export does not establish round-trip correctness. [OpenPencil file-format documentation](https://openpencil.dev/reference/file-format).

**Choice by deliverable:** keep OpenPencil for local editable documents; use native Figma tooling when Figma round-trip fidelity is the acceptance target; prefer HTML/CSS + Playwright for a browser implementation, and SVG + resvg for standalone vectors. A collaborative design-server replacement would introduce accounts/services and does not follow from this finding.

**Acceptance:** 20 representative edits, timing/process counts and output equivalence; component/variable/text/font fixtures reopened in the intended consumer; saved IDs remain usable; interrupted batch never partially publishes.

## 7. Keep distinct renderers where their outputs justify them

Satori, Chromium, OpenPencil, resvg and Typst overlap visually but serve different output contracts. Combining them into a single universal renderer would lose capabilities. The inefficiency to test is repeated translation between them, especially designing in one layout engine and rebuilding in another.

**Recommended routing:** browser-native work stays in DOM/CSS; simple generated cards use Satori when SVG output is useful; vector art stays SVG; editable design stays OpenPencil; paged PDF stays Typst. Avoid a Satori → screenshot → manual DOM recreation path for a website.

Current upstream Satori documentation now describes Taffy, Grid and block layouts. The attempted version-specific `v0.62.0` README URL was unavailable, so I do not apply current-main capabilities to the pinned package. Claims such as “Satori never supports Grid” would be an unsafe remembered generalization. It remains a partial CSS implementation. [Current Satori documentation](https://github.com/vercel/satori).

**Keep Style Dictionary:** `tokens.mjs` already caches by token bytes plus recipe/lock identity and verifies cached outputs. Recompilation on every edit is not an established problem. Upstream v5 also targets token-resolution performance. [Local cache](../runtime/design/tools/tokens.mjs), [v5 changes](https://styledictionary.com/versions/v5/migration/).

**Keep Sharp, resvg and SVGO** unless codec/font/output fixtures expose a concrete limitation. A language rewrite alone does not improve their contract. Keep Odiff as pixel evidence, paired with semantic/behavioral checks; a pixel match is not usability or correctness.

## 8. D2 is a worthwhile diagram alternative; replace only the applicable path

Kroki supports a broad format surface, but requires a service for diagram production. A local D2 executable is a narrower candidate for architecture diagrams. Current D2 0.9.0 release notes state that TALA is open source and bundled, including its JS/WASM package. The old assumption that TALA necessarily requires a separate paid plugin is therefore outdated for that release. [D2 release](https://d2lang.com/releases/0.9.0/), [layout engines](https://d2lang.com/tour/layouts/).

**Recommendation:** trial D2 for architecture diagrams through the existing design package. Keep Kroki for formats D2 does not replace; keep explicit Excalidraw when an editable sketch is requested. Do not add a fourth permanent service. Pin the exact candidate and verify license/notices and actual binary closure before adoption.

**Acceptance:** nested systems, long labels, cycles, legends, multilingual text and deterministic repeated output. Compare layout quality, startup, image bytes and operational steps. SVG-only production and optional bitmap exports may have different dependency costs; measure the selected path.

## 9. Podman is defensible; Apple Container is a conditional experiment

Podman provides a private VM and established OCI lifecycle. Its upstream supports macOS providers and Windows/Linux options, but Mirket's own managed-package contract is narrower. Native CLI portability must remain separate from package portability. [Podman machine](https://docs.podman.io/en/stable/markdown/podman-machine.1.html).

Apple Container is a real alternative for Apple silicon/macOS 26, consumes OCI images and uses lightweight virtual machines. This could simplify or improve some Mac workloads, but no comparative result here establishes lower startup time, disk use or maintenance. Its default installation/service/data ownership differs from Mirket's private-root contract. [Apple Container](https://github.com/apple/container).

**Recommendation:** retain Podman. Test an optional backend only if macOS-only support and the operational cost justify it. Reuse the existing package owner and acceptance suite; do not build a parallel installer. Check explicit startup, private mounts, network-off execution, child termination, image identity, backup and multi-service diagrams. Avoid Apple container-machine's broad home mapping for this containment purpose. [Container-machine tradeoff](https://github.com/apple/container/blob/main/docs/container-machine.md).

## 10. Add verification capabilities before adding more producer frameworks

The design toolset has browser capture, pixel comparison and model inspection, but no dedicated axe dependency in the inspected manifest. Typst output is visually inspected, while accessibility requires semantic structure and consumer behavior.

**First addition:** optional `axe-core`/Playwright checks for browser output, with structured violations and incomplete checks. Playwright's official guide uses axe integration and explicitly says automated checks cover only some accessibility problems. Retain keyboard and screen-reader checks where required. [Guide](https://playwright.dev/docs/accessibility-testing).

**Second addition:** expose requested PDF accessibility standards in the Typst workflow and preserve semantic text/headings/alt text. Typst documents tagged PDF and PDF/UA-1 checks; passing them is not universal reader compatibility. Test reading order and intended consumers. [Typst accessibility](https://typst.app/docs/guides/accessibility/).

**Conditional addition:** a bounded document-extraction adapter for research tasks. The researched Kreuzberg repository now redirects to Xberg, a Rust-core document extraction project with CLI/MCP and language bindings. This is a candidate for text/metadata/table extraction, not editable Office authoring. Compare it on actual PDFs/scans/DOCX before adopting its broad format claims; inspect OCR/native dependencies, licenses, resource bounds and malformed-file handling. [Current repository](https://github.com/xberg-io/xberg).

These additions should be explicit workflow capabilities, not tools injected into every assignment. Native host document/browser tools remain preferable when they already satisfy the task. All new production dependencies still need approval before implementation.

## 11. Catalog breadth needs outcome evidence and simpler decision presentation

The verified catalog has 347 entries, 41 roles, 63 profiles, 37 facets and 18 workflows. This is a large maintenance/selection surface, but counts alone prove neither waste nor usefulness. The 14 global skill bodies total 82,353 bytes on disk; that is not evidence that every byte enters every prompt. No identical profile memberships were found when comparing sorted skills plus `include_globals`.

**Recommendation:** retain IDs and saved selections. Present the task outcome first, then eligible workflow and actual runtime prerequisites; keep professional roles as optional lenses. Measure selection mistakes, irrelevant guidance read, host fallback, completion and useful artifact quality. Compare host baseline, targeted guidance and current selection on representative tasks with a held-out set.

Do not add a graph database to explore the 882-node/1,336-edge catalog export. Existing JSON plus generated views is adequate until a concrete query fails. Oxigraph was considered as an embedded graph candidate, but SPARQL storage solves no demonstrated current problem. [Oxigraph](https://github.com/oxigraph/oxigraph).

## 12. Documentation drift can make an optimization target the wrong system

The earlier critique contains fixed findings; the integration review records important improvements. The system-map overview still says 234 Cargo records while the generated inventory has 247. Historical accounting-removal wording differs from current per-attempt diagnostic fields. Version labels in historical implementation records also describe their own candidate, not necessarily this checkout.

**Recommendation:** keep dated records immutable as history and generate current counts/feature status from owners. Add explicit supersession links where current behavior changed. Do not add another manually maintained inventory. This report is a dated decision review, not a replacement product specification.

## Proposed experiment order and stopping rules

1. **Contract and verification fixes:** browser pairing, active-fact filtering design, OpenPencil consumer fixtures, explicit reservation units. Preserve failing/discriminating fixtures.
2. **No-new-service optimizations:** batch edits, snapshot/IPC profiling, memory startup/scan attribution. Keep known caches and shared-handle reuse.
3. **Memory bake-off:** current stack, lexical-only, lexical + vector; same facts and queries. Include English/Turkish, exact IDs, paraphrases, contradictions, unanswerable queries, supersession and long input. Measure recall@k, false positives, startup, p50/p95, RSS, disk and restore. Proposed adoption gate: no loss of lifecycle correctness or required retrieval cases, plus a material measured operational improvement agreed before the run.
4. **Narrow producer trials:** D2 and accessibility checks. Use saved artifacts and actual consumers, not renderer exit status.
5. **Only if justified:** Rust recovery journal or alternate container backend. These are the highest migration-risk changes and should not be bundled with a memory-engine swap.

A low-level diagnostic span export could help across these experiments: queue, startup, lock, retrieval, inference, tool and checkpoint phases, with redacted identities. Existing traces should be extended first. OpenTelemetry is a possible export format, but its semantic conventions have explicit stability levels; do not turn evolving GenAI conventions into a new mandatory hosted collector or billing subsystem. [Stability guidance](https://opentelemetry.io/docs/specs/otel/versioning-and-stability/).

## Research method, confidence and limits

Research used actual web searches and opened primary project documentation/repositories for storage, retrieval, embeddings, durable workflows, container backends, rendering, document extraction and accessibility. Local inspection covered current adapters, locks, workflow definitions, tool manifests, tests and previous remediation records. The exact Mem0 wheel was inspected as archive text after checksum verification; no upstream code was executed. Current-main documentation is treated as candidate evidence rather than proof of pinned-version behavior.

High-confidence structural findings: post-filter starvation, exclusive scan/lock path, full snapshot transfer, bytes-based reservation proxy, independent browser selection, per-edit OpenPencil launch/readback and existing token cache. Their latency/quality impact remains unmeasured here. Migration benefits, alternative-engine superiority and adoption rates are hypotheses.

This pass did not rerun native/contained acceptance, start a VM, benchmark models, evaluate every one of the 347 skills, audit all transitive dependencies, recheck every historical advisory, or establish Figma round-trip behavior experimentally. The prior source-map check passed in the inventory turn; this report's local references and current inventory are checked separately. An existing historical negative result is retained as evidence, never recast as a new measurement.

**Recommended disposition:** keep the Rust control plane, SQLite coordination, scoped MCP boundary, current isolation and rollback semantics; optimize current memory and design paths first. Make SQLite-based explicit memory, D2 and accessibility verification the first focused challengers/additions. Keep LangGraph and Podman until comparative evidence justifies their substantially riskier replacements.
