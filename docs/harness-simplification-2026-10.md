# Mirket simplification and completion decisions — 21 October 2026

This is the implementation and decision record for the authorized follow-up to the
simplification review. The changes are in the review checkout, based on
`1da51f6c8c46f873304892a9962b98ec407f3eb2`; they are **unreleased**. The original
checkout and installed services were not updated. No production dependencies,
provider/model substitutions, paid model trials, commits or publication were added.

The useful reduction is smaller discovery output and genuinely selective diagram
preparation. Deleting tool names is not the objective. The SVG, raster, comparison
and recovery contracts have distinct consumers and failure semantics. Encoded
video is now explicitly experimental instead of implying established production
quality. Existing workflow IDs and saved selections remain available.

## A. Retained product contract

Core Mirket safely installs and maintains host guidance/configuration; makes
relevant expertise discoverable; and provides trustworthy identity, inventory,
compatibility and recovery information. Host permissions and session lifecycle
remain host-owned. Source inspection, editing, review and browser capabilities
already supplied by the host remain host-owned too.

Durable task coordination, routed workers, container-backed design, memory recall
and interrupted-worker recovery are explicit optional layers. A fresh unattended
setup selects no runtime package. Selecting `design` now prepares only its core
image and build base unless saved or explicit diagram selection requires more.
The managed VM controller remains macOS ARM64; native CLI support on other
platforms does not establish container-runtime support there. Serving does not
implicitly activate the VM or services.

OpenPencil remains the editable-design engine. New-document creation and editing
an existing document are separate retained journeys. Usage and billing remain in
provider panels. Advisory findings alone do not trigger expensive remediation.
Identity, permissions, path isolation, immutable receipts, rollback, publication
recovery and execution budgets remain requirements.

Completeness means useful retained journeys, including failure and recovery, work
with observable artifacts. It does not mean every historical ambition becomes a
supported feature. A tool invocation, registered skill or compliance with a required
tool is insufficient evidence of useful adoption or semantic correctness.

## B. Prioritized component decisions

Confidence below concerns the stated disposition, not universal task quality.
`KEEP OPTIONAL` means retained under explicit selection, not individually installable.
Sources: [architecture](architecture.md), [catalog](../src/catalog.rs),
[workflow definitions](../registry/capabilities.json),
[design owner](../src/packages/services.rs), [design operations](../runtime/design/tools/operations.mjs),
[OpenPencil owner](../runtime/design/tools/openpencil.mjs),
[agent runtime](../runtime/agent/), [router](../src/router/).

| Priority / surface | Outcome and execution owner | Discovery, prerequisite and invocation | Evidence, overlap, cost and removal consequence | Disposition / confidence |
| --- | --- | --- | --- | --- |
| P0 installation, update, identity, project publication | Safe host configuration and recoverable changes; Rust install/update/project owners | CLI setup, doctor, update; explicit home/project and reviewed destinations | Native rollback, drift, symlink and recovery tests; different boundaries from worker checkpoints. Removal loses safe installation/release recovery | KEEP CORE / high |
| P0 catalog and capability metadata | Relevant expertise with canonical workflow descriptions; catalog owner | `catalog_search`, `catalog_show`, `skill_read`, `capability_plan`; no optional runtime | Five targeted outcome queries now reach relevant guidance; role workflow references retain eligibility but stop repeating full descriptions. Metadata retrieval is not natural adoption | CONSOLIDATE + REPAIR ADOPTION / high for structure, unverified for selection |
| P0 design preparation and receipts | Explicit optional image preparation; existing package owner | `package prepare design --group diagrams`; package must already be selected, engine explicitly available | Core never requests diagram images in fake-engine test; failed pull/retry, immutable receipts, image restoration, old full receipts and saved groups covered. Extra optional receipt is owned by the existing generation mechanism | CONSOLIDATE / high for tested transitions |
| P1 `tokens_compile` / Style Dictionary | Shared canonical design tokens | `design-tokens`; design core; direct MCP or scoped routed call | Real five-format compiler/cache/concurrency tests. Avoids divergent formats; renderer consumers still need it | KEEP OPTIONAL / high |
| P1 `openpencil_inspect` | File inspection and lazy schema discovery | `editable-interface`; design core; inspect without source for schema | 87 operations remain behind four wrappers. Schema-only call succeeds while another connection holds render slot and leaves empty workspace unchanged; document reads still lock | KEEP OPTIONAL / high for wrapper; not all upstream semantics |
| P1 `openpencil_document` | New or imported editable document | Same workflow; canonical tokens and authorized source/output | Real new/import/save/reopen/token tests; an allowed creation tool is still a distinct journey | KEEP OPTIONAL / high |
| P1 `openpencil_edit` | Validated operation and atomic saved revision | Inspect saved IDs/schema, then edit with revision preconditions | Real edits, IDs, failure and isolation cases; no evidence that every one of 58 write operations is semantically correct | KEEP OPTIONAL / high for retained contract |
| P1 `openpencil_export` | Inspectable exported pixels from saved source | Saved document, design core, explicit export | Real export tests; preserves editable source separately. Browser screenshot is not a file-editor replacement | KEEP OPTIONAL / high |
| P1 `raster_transform` / Sharp | Intentional crop, resize and codec conversion | `raster-processing`; design core; explicit transformation | Real codec/orientation/transparency/bounds tests. Internal staging validates and preserves bytes; it does not replace transformation | KEEP OPTIONAL / high |
| P1 `odiff_compare` / Odiff | Independent comparison of two saved images | `pixel-comparison`; two bounded PNGs | Independent mismatch/dimension and diff contract. Renderer-specific baseline comparisons overlap only partially | KEEP OPTIONAL / high |
| P1 `vector_render` / resvg + SVGO | Editable semantic SVG plus preview | `vector-art`; design core; authored SVG and bounded dimensions | Real text, hidden-state, IDs and preview tests. No graph layout or automatic Flexbox text layout | KEEP OPTIONAL / high |
| P1 `svg_optimize` / SVGO | Optimization including inputs beyond vector preview limit | `vector-optimization`; standalone SVG input/output | Actual >2 MiB input rejected by direct vector preview but optimized successfully, then previewed; hidden ID retained. Removing wrapper saves no SVGO dependency | KEEP OPTIONAL; consolidation rejected this phase / high |
| P1 `static_render` / Satori + resvg | Flexbox and text layout | `static-graphic`; tokens, fonts, design core | Three real inspected cards: wrapping, multilingual text and data. Browser may cover outcomes but requires different authoring; equivalent repair/quality cost unmeasured | KEEP OPTIONAL; replacement not established / medium |
| P1 `browser_screenshot` / Playwright + Chromium | Local UI states, interaction and visual evidence | `web-interface` / `interactive-motion`; local source, bounded actions | Real Chromium 2x, actions, timing, scope and cancellation tests. Host browser remains preferred when sufficient; removing Mirket wrapper removes bounded worker access | KEEP OPTIONAL / high |
| P1 `typst_compile` / Typst | Paged PDF and every-page images | `paged-media`; local typeset source and tokens | Actual two-page PDF with distinct inspected pages; absolute asset rejection. Tool deadline corrected to 25 minutes; overall work budget stays 30 | KEEP OPTIONAL / high for fixture |
| P1 `kroki_render` / Kroki | Actual graph layout | `diagram`; explicit prepared/started diagrams group | Live installed Graphviz trial preserved four nodes, three arrows and both branch labels. SVG renderer alone does not supply layout. Nominal historical image size about 2.86 GB, not measured reclaimable bytes | KEEP OPTIONAL; excluded from fresh core preparation / high |
| P2 Excalidraw companion | Excalidraw source-format rendering behind Kroki | Same tool, `diagram_type=excalidraw`; explicit diagrams group | Unique input-format contract, but current user adoption/quality trial absent. About 1.63 GB historical nominal size; shared layers prevent additive savings claim | REMOVE CANDIDATE for separate future opt-in; default exclusion completed / medium |
| P2 `hyperframes_render` / HyperFrames + FFmpeg | Experimental finite encoded export | `motion`; authored local scene and design core; lint → snapshots → encode | One named adapter opportunity recovered after fixture corrections; 1 s H.264, 320×180, 24 decoded advancing frames. No natural selection or ordinary-speed playback evaluation. Chromium shared; video is still bundled | UNVERIFIED for production promise; promise narrowed / high |
| P1 memory / Mem0, FastEmbed, Qdrant, BM25, spaCy | Explicit facts, provenance, retrieval, expiry, supersession and forgetting | `memory_read/put/forget`; selected agent-runtime and project scope | Four fresh-index arms, 23 facts/15 queries: recall@3 1.0 for all; three semantic arms falsely return facts for tax query. NLP has no demonstrated marginal gain here; this is a ceiling fixture, not a migration test. 97 locked distributions and ~252 MB assets are baseline inventory | KEEP OPTIONAL; no NLP expansion or removal this phase / medium |
| P0 LangGraph and SQLite checkpoint backend | Budget-preserving interrupted execution, effect intent/CAS and recovery | `workflow_status`, `resume_subagent`, `workflow_forget`; optional runtime | Real runtime restart/stale checkpoint/effect/host-handoff suite. Coordination DB is not a replacement for checkpoint semantics | KEEP OPTIONAL / high |
| P1 coordination SQLite | Task/evidence/delivery state | Durable task tools only when continuity is needed | Native evidence/hash/history/terminal-state tests. Memory facts are not task state; no duplicate tracker added | KEEP OPTIONAL / high |
| P2 router policy | Eligible provider/model and bounded dispatch | `model_registry`, `route_task`, optional `spawn_subagent` | Local fake provider reproduces strongest-first ranking for T1 request. Lowest supported effort meeting floor is separate; workflow floors can exceed tool floors. No authorized real-model comparison performed | UNVERIFIED cost/quality tradeoff; policy preserved / high |

The main MCP surface remains 27 tools. Grouping below makes every retained owner
explicit; it is not an instruction to invoke all of them.

| Main MCP tools | Owner, outcome and path | Prerequisites, evidence and removal consequence | Decision |
| --- | --- | --- | --- |
| `catalog_search`, `catalog_show`, `skill_read`, `capability_plan` | Catalog; find/read expertise and workflow choices | Core metadata; outcome search and compact-plan tests. Removing them breaks bounded expertise access | KEEP CORE; consolidate descriptions |
| `team_facets`, `team_plan` | Catalog; optional composition | No worker starts; existing role/facet fixtures. No proof of natural use, no exact duplicate role bodies found | KEEP OPTIONAL; no bulk pruning |
| `runtime_status`, `project_list` | MCP identity/state owners; inspect process and registered roots | Local identity/inventory; protocol/identity tests. Do not conflate metadata with live package readiness | KEEP CORE |
| `verification_handoff` | Verification owner; scoped host check handoff | Declared artifacts/checks; native tests; removing loses structured host-owned verification path | KEEP OPTIONAL |
| `model_registry`, `route_task`, `spawn_subagent` | Router; discover, plan, dispatch scoped work | Configured authorized providers; discovery/plan can refresh state, spawner can incur inference. Fake provider tests are not real-model evidence | KEEP OPTIONAL; no policy change |
| `task_start`, `task_status`, `task_history`, `task_validate`, `task_checkpoint`, `task_evidence`, `task_finish` | Coordination state; durable delivery record | Registered project and explicit need for continuity; existing native lifecycle tests | KEEP OPTIONAL; no second tracker |
| `memory_read`, `memory_put`, `memory_forget` | Memory; scoped explicit facts | Optional runtime; real retrieval and lifecycle tests. Forgetting requires exact revision | KEEP OPTIONAL |
| `workflow_status`, `resume_subagent`, `workflow_forget` | Checkpoint owner; inspect/recover/forget execution | Optional runtime, scope and preserved budgets; restart/failure tests | KEEP OPTIONAL |
| `runtime_data`, `runtime_data_apply` | Runtime lifecycle; typed export/retention | Exact plans and digests; native schemas and runtime lifecycle tests. No replacement by generic DB deletion | KEEP OPTIONAL |

## C. Removal and consolidation ledger

| Change | Replacement; preserved/lost behavior | Consumers and transitive cleanup | Compatibility and evidence |
| --- | --- | --- | --- |
| Remove repeated full `role_workflows` descriptions | Canonical full `worker_workflows`; role list keeps ID and eligibility | Catalog serializer, skill-catalog guidance, capability tests. No dependency/image/schema-tool removal | Repository consumers only read retained fields. Saved workflow/role IDs unchanged. External JSON consumers must resolve descriptions by ID; response shape change is documented |
| Remove Penpot-specific current extractor exception and stale current probe expectations | Current scoped OpenPencil and unavailable-runtime behavior | `evals/harness-map/extract.py`, critique probe/readme; no deletion of historical evidence, licenses or retained volumes | Historical reports retain their revision. Generator refreshed in disposable output only; prior inventory is not relabeled current |
| Remove default diagram image preparation | Explicit `prepare --group diagrams`; startup remains explicit | Existing services/CLI owner, immutable receipts, recovery hints, package guide and tests; no new registry or package | Saved diagram groups and old full receipts retained. Fresh core has no diagram download requests. No installed images deleted |
| Narrow general encoded-video promise | Experimental `motion` workflow and tool; inspected named export only | Workflow metadata, MCP description, local-video guidance and package docs | Tool/workflow IDs retained; HyperFrames/FFmpeg still bundled. No dependency or image saving claimed |
| Do not remove standalone SVG optimizer | Existing larger-input optimization contract | A future merge would need input-limit equivalence, standalone output semantics, assigned-tool/floor mappings, schema/instruction/test updates; SVGO stays for retained tools | Current trial demonstrates a counterexample to direct replacement |
| Do not remove Satori now | Retain existing Flexbox contract | A future replacement needs authored browser/vector fixtures, fonts/wrapping parity, artifact/timeout behavior, package-lock/image/notice cleanup, workflow/tool aliases and CI coverage | No new users should be promised equivalent authoring cost without a comparison |
| Exclude Excalidraw from core; do not delete selected support | Existing optional diagrams group | A later separate opt-in/removal must change group/env/receipt/container aliases, accepted diagram types, registry image/license/audit entries, docs/skills, startup/rollback tests and CI inventories | Existing `diagram` selections include Excalidraw. Silently changing that selection would remove a saved capability; explicit migration is required |
| Do not remove NLP/Mem0/checkpoints on this fixture | Existing runtime, no storage migration | NLP removal would need requirements/lock/model/BM25 assets, recipe/licenses/audits/CI cleanup and preserved retrieval/index rebuild semantics. Mem0 replacement additionally needs data migration, expiry/supersession/forget/export compatibility | Equal small-fixture recall is not migration safety. Checkpoint recovery has a separate owner and cannot be folded into memory or task SQLite |

No dead runtime helper was proven disposable beyond the current Penpot exception.
No new framework, tracker or telemetry was introduced. Conditional deletions are
closed as **not justified in this phase**, rather than left as endless background
experiments. Reopen only on an actual task opportunity or explicit migration goal.

## D. Gap register

| Class | Finding / status | Consequence and bounded next condition |
| --- | --- | --- |
| Reproduced defect, repaired | Fresh core preparation requested both diagram images; now only node build base/design-tools | Fake-engine no-request assertion, optional pull failure/retry and saved-group tests pass; real clean-machine download-byte saving unmeasured |
| Reproduced overhead, repaired | Full workflow metadata repeated under role selection | Same canonical serialization: 12,280 → 10,224 characters (16.7% less), while retaining all 17 canonical choices and 10 role references |
| Documentation mismatch, repaired | Assigned design workspace writes described too restrictively | Source and assigned workspace remain separate authorized roots; native cross-project/symlink tests and scoped browser tests retained |
| Documentation mismatch, repaired | Pending Codex design reconnect, current Penpot expectations, paged deadline | Reconnect is dated evidence, Claude remains untested; historical Penpot evidence preserved; 25-minute Typst tool ceiling matches implementation |
| Reproduced recovery issue, repaired | Disabled diagrams hint omitted preparation; new optional preparation needed safe retry | Explicit stop/prepare/start hint; image restoration does not rewrite immutable receipt; no implicit service activation |
| Reproduced retrieval limitation | Unanswerable annual-tax query returned unrelated facts in hybrid-NLP, no-NLP and dense arms | Preserve threshold 0.25 and failure. Lexical abstained on 2/2 negatives, but score calibration differs; no claim it is a generally better replacement |
| Unsupported promise, narrowed | Encoded video advertised as a general production route | Experimental in workflow/tool/guidance; one encoded fixture verified, no ordinary-speed playback or autonomous quality claim; no separate installation switch |
| Suspected adoption failure, repaired entry point only | Outcome searches previously missed/directly obscured static/editable/diagram/video/memory guidance | Five deterministic query regressions pass. This is discovery access, not agent selection or useful contribution |
| Unknown | Natural selection across retained journeys | Zero current autonomous selection trials. No paid workers launched; next evidence must use real relevant opportunities and an authorized model, including negative/no-specialist case |
| Unknown | Current cold startup, full RSS, end-to-end latency/cost saving | No before/after benchmark. Disposable doctor includes one debug MCP sample, not a release comparison; old 56–75 ms warm calls remain historical |
| Unknown/platform limit | Fresh Claude activation; managed containers outside macOS ARM64 | Do not infer from CLI tests or configured executables. Requires explicit client/platform journey |
| Historical result retained | Prior design trials 0/3 first attempt, 1/3 recovery; later artifacts 3/3 | Older explicitly routed revision; not current natural adoption. Earlier quality evaluation did not demonstrate gain |

## E. Implementation sequence and completion

1. **Contract and recovery repairs completed:** workspace documentation, reconnect
   status, current probes, Typst deadline and explicit diagram recovery path.
2. **Bounded decision trials completed:** real Satori/SVG/Typst outputs, four-arm
   retrieval fixture, graph layout and encoded-video opportunity. Failures and
   interventions remain in evidence; no model inference was used.
3. **Proven redundant surfaces removed:** repeated workflow descriptions and the
   current Penpot extractor exception. Distinct transformation/recovery tools kept.
4. **Default burden reduced:** diagrams excluded from fresh core preparation through
   the existing package mechanism. Video promise narrowed; no unsupported NLP deletion.
5. **Canonical discovery consolidated:** workflow summary comes from canonical
   metadata; role references preserve decisions. No parallel policy registry.
6. **Valuable adoption entry points repaired:** targeted outcome tags and workflow
   summaries. Natural adoption remains a separate unmeasured outcome.
7. **Router evaluated without changing authorized models:** fake-provider ranking
   confirms the policy concern. No real-model quality/cost comparison or policy
   substitution was justified by the current evidence.
8. **Remaining capability gaps bounded:** no speculative feature expansion. OpenPencil,
   paged output, intentional transformations and independent QA have useful retained
   contracts. Production video and unsupported-platform promises are explicitly limited.

Release/deployment is a separate operation. This checkout is reviewable, not an
accepted published release. Full candidate-bound release acceptance against
matching prepared packages, host reconnection and required native platform CI
must precede publication; do not reuse the old release receipt.

## F. Regression and adoption verification

The existing native, JavaScript, dependency and release gates remain unchanged.
The selective-preparation fixture tests no diagram request on fresh core, failed
optional pull without partial receipt, retry, missing pinned image restoration,
immutable/conflicting/unknown receipts, saved groups and core compatibility after
preparation. It does not claim a measured bandwidth saving or test a real empty VM.

OpenPencil schema discovery is tested through a separate MCP connection while
another connection owns the render slot; saved-file operations keep their lock.
Existing file scopes, sibling-project rejection, semantic IDs, overwrite revisions,
cancellation and native child containment remain checked.

| Opportunity | Current verification | Natural-adoption acceptance for a future authorized task |
| --- | --- | --- |
| Ordinary source change/review | Host-owned; native harness regression suite | Complete useful source change without unnecessary runtime/worker selection |
| Browser interface | Real Chromium action/timing/2x/scope tests | Choose suitable host or scoped runtime; primary action, keyboard and rendered states correct |
| New OpenPencil document | Real create/token/save/reopen/export tests | Discover editable route without a named tool, produce useful reopened source and inspected export |
| Edit existing document | Real validated edit/ID/revision/failure tests | Preserve unrelated content and requested edit; no replacement by a newly created imitation |
| Static/vector graphic | Three inspected Satori cards; larger SVG optimization counterexample; semantic vector tests | Correct content/layout/fonts and useful editable artifact; compare authoring/repair effort only on matched tasks |
| Diagram | Named live Graphviz graph, inspected labels/directions | Correct relationships and readable layout; attribute graph layout to Kroki, not merely renderer invocation |
| Paged document | Real two-page PDF and inspected page images | Every page and content checked; missing page must fail even when PDF header is valid |
| Video, experimental | Named offline export with failed attempts retained; decoded motion | Keep experimental until relevant natural opportunity, playback/content/audio where requested, and a justified installation decision |
| Memory recall | Four ranking arms with identifiers, inflection, multilingual, expiry and supersession | Correct facts used as evidence, injection treated as data, irrelevant/unanswerable cases handled, migration/reindex preserved |
| Interrupted worker | Real runtime checkpoint/effect/CAS/budget tests | Resume original intent without duplicate uncertain effect, reset budget or silent model switch |
| No-specialist task | Host-owned contract; no model trial | Finish a simple task without unnecessary specialist/tool selection |
| Unavailable runtime/platform | Fake-provider refusal before inference; native validation | Explain explicit activation/support limits, preserve task state, use only authorized host fallback and label it |

Use relevant opportunities as denominators. Record natural versus named selection,
first attempt versus intervention, invocation versus correctness, compliance versus
contribution, and harness versus host fallback. Keep failed/abandoned tasks in the
denominator. A fixed small fixture cannot establish general quality gain.

## G. Before/after scorecard and evidence

| Measure | Before | After / status | Interpretation |
| --- | --- | --- | --- |
| Graphic Designer plan, compact JSON characters | 12,280, installed 1.0.11 | 10,224, candidate; −2,056 (16.7%) | Direct same-query measurement; not token, latency or billing saving |
| Canonical choices / role references | 17 / 10, duplicated descriptions | 17 / 10, canonical + compact | Information/eligibility retained; external consumers resolve description by ID |
| Fresh design preparation images | Build base + design-tools + two diagram images | Build base + design-tools | Behavior tested with fake engine; no actual cold transfer/RSS saving claimed |
| Targeted query access | Historical misses/indirect results | Five exact query regressions pass | Deterministic discovery; zero current autonomous selection trials |
| Catalog / roles / profiles / facets / workflows | 347 / 41 / 63 / 37 / 17 | Unchanged | Catalog is not installed tools; 5 package-delivery entries still mean 2 runtime packages |
| Main / design MCP tools | 27 / 14 | Unchanged | Useful simplification does not require tool-count reduction |
| Memory | Historical 22 facts / 7 queries | 23 facts / 15 queries / four fresh-index arms | All answerable recall@3=1; semantic arms abstain 1/2, lexical 2/2; no NLP benefit or resource saving established |
| Encoded-video opportunity | No current useful-output evidence in handoff | 1 named opportunity: first attempt failed; recovered on fourth invocation | First failure was fixture PID1, second timeline lint, third insufficient fixture scratch disk; fourth encoded. No new campaign/budget reset |
| Design quality | Historical evaluation found no demonstrated gain | No new claim | Inspected fixtures establish their own outputs only |
| Cold startup / RSS / cost | Not current measured baseline | Not compared | Debug doctor numbers and old warm transport samples are different measurements |
| Source/platform/release | Review base and deployed identity distinct | Candidate native macOS ARM64 checks; not deployed | Original working tree and installed binary preserved; no fresh Claude/other-platform activation |

Final native gate: **417 tests passed, 9 explicit opt-in tests ignored, 0 failed**,
with formatting and Clippy warnings denied. The final contained design gate passed **77/77 with no skips**. The real
agent-runtime suite and explicit backup/history/pending-effect/relocated-memory
recovery test also passed. Closing identity details are recorded in
[evidence](harness-simplification/evidence.json). Disposable `setup --target all`
and `doctor --json`, catalog validation, six current local-provider probes,
5 remediation and 12 dependency-audit Python tests, and synthetic quality-packet
preflight passed. Python audit checks used installed Python 3.13 after system
Python 3.9 lacked `tomllib`; no dependency was installed to conduct the review.

The compact [evidence bundle](harness-simplification/evidence.json) records identities,
checks, output hashes, failure limits and measurements. Full logs, synthetic output
images/video, the failed attempts and disposable trial reports are retained outside
shipped source in the task's evidence directory. Reproduction owners are
[capability tests](../tests/capability_workflows.rs),
[selective preparation tests](../src/packages/services.rs),
[design contract trials](../runtime/design/tools/test/simplification.test.mjs),
[explicit offline runtime trials](../tests/runtime_simplification.rs),
[retrieval fixture](../evals/harness-remediation/retrieval-ablation.py),
and
[current routing probes](../evals/harness-critique/probe.py). The pre-retirement
video fixture was removed with the final cleanup; its original revision and
outcome remain recorded in the evidence bundle.
