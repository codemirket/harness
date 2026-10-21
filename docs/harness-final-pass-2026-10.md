# Mirket final pass: decisions and implementation order

The later [implementation report](harness-final-pass-implementation-2026-10.md) supersedes this review-time plan, including the explicitly authorized advisory-only policy. Earlier gate failures below are historical evidence, not current release rules.

21 October 2026. Decision baseline. The user subsequently authorized video retirement, explicit Excalidraw selection, compatibility preservation and checks. See the [implementation record](harness-final-pass-implementation-2026-10.md) for current results; the evidence and proposed choices below retain their review-time meaning.

**Recommendation: keep the small host-integration core, preserve proven specialist outcomes, and reduce what optional installations carry.** The next useful reduction is removing encoded video from the normal design generation and separating Excalidraw from ordinary diagram preparation. The next reliability work is the failing dependency admission gate and a small set of natural task opportunities. Another broad rewrite, catalog purge or recovery-system merger is not justified.

This decision record uses the [system map](harness-map.md), [complete inventory](harness-map/inventory.md), [methods](harness-map/methodologies.md), [dependencies](harness-map/dependencies.md), [prior decision trials](harness-simplification-2026-10.md) and their [evidence](harness-simplification/evidence.json). It supersedes the prior record's proposed next steps, not its dated results. All proposed removals below require implementation authorization; none happened in this pass.

## Basis and limits

- Reviewed `main` at `cc8a258655ca15edf5f584df5052184bcc596b92`, including existing local package-service and dependency-audit fixes. Working-source fingerprint: `4b97d97fa892384da0dc321c1bee5148da601e7804182d8dca1266d4f70e5c8d`. The map's generated-output check passed again.
- The local candidate and installed executable differ. The current working tree is not established as deployed. Prior local native verification covers this same runtime-source fingerprint: 418 passed, nine explicit opt-in tests ignored. It was not rerun for this review.
- Current declarations: 347 catalog entries, 63 profiles, 41 roles, 37 facets, 17 workflows, 27 main MCP tools, 14 design tools and two runtime packages. These counts are not adoption or installed-resource measurements.
- Earlier design and agent-runtime trials establish specific artifacts and recovery cases. They do not establish current natural tool selection, general design quality or current cold-start/RSS savings.
- This review used passive repository inspection and existing evidence. It did not start services, contact inference providers, scan upstream advisories again, install dependencies or run model trials. No provider/model choice is proposed implicitly.
- One map defect was corrected: a skill/source ID collision falsely marked `planning-with-files` unresolved. Relation-aware extraction now yields 881 nodes, 1,456 edges and no unresolved declarations. This was an inventory error, not a missing runtime capability.

## Retained product contract

Core Mirket safely installs and maintains host guidance/configuration, makes relevant expertise discoverable, and supplies trustworthy identity, inventory, compatibility and recovery information. The host owns ordinary coding, review, browser/app interaction, permissions and session lifecycle; providers own billing and usage panels.

Durable task coordination, routed workers, specialist design production, explicit memory and interrupted-worker recovery are optional. Optional must mean avoidable acquisition and execution where a large dependency group is involved, not merely an unused tool in a monolithic image. Keep the existing package and service owners; do not build a new plugin manager for this reduction.

OpenPencil remains the editable-design engine. Keep new-document and edit-existing journeys. Preserve source identity, inventory, licenses, permissions, project isolation, immutable receipts, rollback, publication recovery and original execution/repair budgets. Serving must not secretly start the VM. Unsupported platforms must be explicit.

Completion means the retained journeys produce useful artifacts and recover honestly. It does not require implementation of every catalog ambition. This final pass introduces no new capability promises.

## Prioritized decisions

Confidence concerns the disposition, not universal output quality. KEEP OPTIONAL is a positive decision to retain a contract; it is not a request for endless experiments. REMOVE CANDIDATE names a recommended product reduction with a bounded compatibility gate. Priority denotes the next work, not component importance.

| Priority / component | Decision | Useful outcome, evidence and cost | Next action and confidence |
| --- | --- | --- | --- |
| P0 installation, update, identity, project publication | KEEP CORE | Safe host changes, exact source/binary identity, managed drift and rollback. Existing native failure/recovery checks. These safeguards prevent expensive corruption. | Preserve owners and gates. High. |
| P0 source dependency admission | REPAIR | The prior 21 October audit still fails on nine Pillow 12.3.0 advisories; its reported fix conflicts with the pinned FastEmbed constraint. This is a release blocker under the existing gate, not proof of a reachable exploit. | One bounded reachability/compatibility disposition; no blanket waiver, forced incompatible bump or speculative runtime rewrite. High for blocker; exploit reachability unknown. |
| P1 catalog search/show/read and capability planning | KEEP CORE + REPAIR ADOPTION | Canonical expertise access and targeted query tests. Graphic plan was measured at 10,224 vs 12,280 serialized characters. Useful natural selection remains untested. | Test real outcome prompts before more ranking/prompt edits. Do not shrink by count. High for contract, limited adoption evidence. |
| P1 14 global skills and host instructions | SIMPLIFY at the point of an observed conflict | Global text is paid in attention even without optional runtime. No demonstrated duplicate bodies; repeated boundaries across instructions are not automatically redundant. | On a failed opportunity, shorten the specific competing instruction and retain one owning reference. No arbitrary word quota or global rewrite. Medium. |
| P2 63 profiles / 41 roles / 37 facets | KEEP; simplify presentation only | Installation bundles, professional contracts and composition solve different problems. Eight starter profiles already reduce default exposure. No demonstrated semantic duplicate warrants deletion. | Keep saved IDs and memberships, selective role loading and explicit alternatives. No profile/role merge project. High. |
| P2 149 upstream / 139 reference / five package-delivery entries | KEEP as metadata | Pinned discovery/provenance can be useful without installing or loading all bodies. External prerequisites and rights remain relevant. | Remove an entry only for a concrete stale, unsafe, misleading or replaced contract; not lack of usage logs. Do not fetch everything to review it. High. |
| P1 OpenPencil four wrappers / 87 lazy operations | KEEP OPTIONAL | Real save/reopen/edit/export, identity and failure cases; lazy schemas contain exposure. Engine semantics and new/edit journeys still need task-level checks. | No wrapper merger or restoration of Penpot. High. |
| P1 tokens_compile / Style Dictionary | KEEP OPTIONAL | Canonical token outputs shared across production tools; real format/cache/concurrency tests. | Keep one compiler and consumer contract. High. |
| P1 vector_render / resvg and SVGO | KEEP OPTIONAL | Editable SVG plus pixels; actual rendering and semantic source tests. | Retain as vector owner; do not claim it replaces graph or Flexbox layout. High. |
| P2 static_render / Satori | KEEP OPTIONAL; close removal proposal | Three inspected layout cards; Flexbox and text layout are real functionality. Browser replacement would shift authoring and repair work, not simply eliminate a duplicate. | No replacement trial campaign absent a concrete maintenance problem. Medium. |
| P2 svg_optimize / SVGO | KEEP OPTIONAL; reject wrapper deletion | A real larger-input case cannot go directly through vector_render. SVGO remains shared even if wrapper disappears. | Keep its input/output contract. Revisit only if a unified API demonstrably preserves it. High. |
| P1 raster_transform / Sharp | KEEP OPTIONAL | Intentional resize/crop/conversion, orientation/transparency/bounds checks. Staging preserves inputs and cannot replace it. | Preserve transform tests and format behavior. High. |
| P1 odiff_compare / Odiff | KEEP OPTIONAL | Independent saved-image comparison, dimension/layout failure and diff artifact. | Preserve independent comparison even where renderers also compare pixels. High. |
| P1 browser_screenshot / Playwright / Chromium | KEEP OPTIONAL | Scoped worker UI interaction and rendered states. Host browser often suffices, but workers need an owned bounded path. | Prefer host for ordinary host tasks; keep worker scope, cancellation and actual UI evidence. High. |
| P1 typst_compile / Typst | KEEP OPTIONAL | Real paged PDF and every-page images; distinct from screenshots and editable office files. | Retain 25-minute tool ceiling within 30-minute work budget. High for contract. |
| P1 Kroki | KEEP OPTIONAL | Actual graph layout; inspected Graphviz nodes/edges/labels. Already excluded from fresh core acquisition. | Keep explicit diagram preparation; verify labels and relationships. High. |
| P1 Excalidraw companion | REMOVE CANDIDATE from fresh ordinary diagram selection | Provides a distinct Excalidraw input renderer, but no current useful user outcome is demonstrated. Historical nominal image size ~1.63 GB; no measured reclaimable-byte claim. | Make companion an explicit selection through existing groups; migrate old full-diagram selections faithfully. Do not delete saved support or images silently. Medium. |
| P1 HyperFrames / encoded video / FFmpeg | REMOVE CANDIDATE from normal supported design generation | One named offline opportunity failed first and encoded on fourth invocation; no natural selection or production playback result. Acquisition is still bundled. “Experimental” alone does not remove burden. | Recommend retiring the bundled encoded-video path rather than building a second motion subsystem. Explicit capability-loss/migration gate below. Medium, a scope decision rather than proof of uselessness. |
| P1 coordination SQLite and seven task tools | KEEP OPTIONAL | Durable task/evidence/delivery history and CAS; useful when continuity is needed. | Do not require a durable task for ordinary single-session work or create a second tracker. High. |
| P1 scoped worker dispatcher / verification_handoff | KEEP OPTIONAL | Restricts file/tools and hands host checks back to the host; useful beyond merely starting a model. | Keep authorization, digest and scope boundaries. Adoption trial must distinguish contribution from invocation. High for boundary. |
| P1 memory CRUD / Mem0 / hybrid + NLP | KEEP OPTIONAL; repair relevance evidence, freeze architecture | Explicit facts, provenance, expiry, supersession, forgetting and export are useful. Small four-arm comparison has ceiling recall and semantic false positives. 97-wheel closure is substantial, but count is not a replacement design. | Keep current runtime and thresholds. Investigate the recorded negative before adding features; no wholesale Mem0, dense or NLP removal now. Medium. |
| P1 LangGraph / checkpoint SQLite | KEEP OPTIONAL | Original intent, pending effects, CAS and budget-preserving resume have real restart/failure tests. | Do not replace with task DB or memory. No framework-removal project without a complete replacement state machine. High. |
| P1 runtime_data / runtime_data_apply and backup/retention owners | KEEP OPTIONAL | Typed export/plan/apply, revision binding and safe recovery across different persistence boundaries. | Retain explicit plans and restores; no generic DB deletion replacement. High. |
| P2 model_registry / route_task / router policy | KEEP OPTIONAL; evaluate policy separately | Strongest-eligible ranking is demonstrated; cheapest-adequate benefit is not. Tool/workflow/task floors and failover interact. | Keep policy until authorized matched-model evidence. Do not lower floors as a simplification shortcut. High for observation; savings unknown. |
| P2 repeated design declarations | CONSOLIDATE narrowly | Schemas, dispatch and some trusted-tool lists repeat names; tool floors and budgets already share JSON owners. Independent authorization checks are not redundant. | Remove only mechanically duplicated inventories with fail-closed parity; do not create a universal policy framework or combine distinct timeout/floor semantics. Medium. |
| P2 VM/resource guard, path isolation and owned services | KEEP OPTIONAL infrastructure | Necessary for contained tool execution and predictable admission. Managed controller supports macOS ARM64; native CLI matrix is broader. | Keep explicit activation, limits and unsupported-case behavior. No portability expansion in this pass. High. |

The grouped decisions cover every main MCP tool: catalog_search/show, skill_read, capability_plan; team_facets/plan; project_list/runtime_status; model_registry/route_task/spawn_subagent; verification_handoff; task_start/status/history/validate/checkpoint/evidence/finish; memory_read/put/forget; workflow_status/resume_subagent/workflow_forget; runtime_data/runtime_data_apply. Exact prerequisites and invocation owners remain in the map rather than duplicated into another configuration table.

## What is already done

Do not schedule these as new simplification work: compact canonical workflow responses; eight starter profile defaults; targeted outcome tags/search regressions; fresh core preparation without diagram images; experimental video wording; OpenPencil schema-only lock exemption; corrected workspace-write and paged-timeout guidance; removal of active Penpot expectations from current probes. These have implementation or bounded test evidence, not proof of natural adoption.

The consolidated review's optional-image addition fix, missing-image diagnostic and source-exception hash regression are present locally. They need inclusion in the eventual approved candidate and release gates, not a second implementation. Prior history documents must remain dated; obsolete deployment prose must not be copied as current status.

## Removal and consolidation contracts

### 1. Retire bundled encoded video — recommended product cut

Retained replacement: host-owned video tooling when available and explicitly authorized. Mirket browser interaction remains for live interface motion. **Neither is an equivalent replacement for a self-contained MP4/WebM encoder.** Lost capability: Mirket-owned finite encoded-video production and its worker workflow. Preserve existing scene/output files and old evidence.

Before implementation, inspect saved motion selections and pending checkpoints through the existing read/preview paths with their known effects. If an active required consumer exists, report it and stage an explicit migration; do not silently remove its tools or route to another provider. Without an active consumer requiring preservation, remove the bundled path in the next approved generation. If the user actually needs this outcome, retain it only under a separately approved acquisition design using the existing package mechanism. Do not build that branch merely to avoid making a product choice.

Cleanup must cover `hyperframes_render` definition/dispatch and operation helpers; motion workflow/tool assignments, policy and budgets; required-tool checks and design gating; HyperFrames direct npm dependency and only unshared transitives; FFmpeg/ffprobe only after all consumers are checked; Dockerfile help checks/build steps, license notices, image/SBOM/audit inventory; catalog/role/profile descriptions, installed guidance, schema/tests/CI and saved motion selection diagnostics. Chromium/Playwright remain for browser work. Remove the Go/esbuild patch-build machinery only if the surviving lock graph no longer needs esbuild; do not assume the entire build stage is video-only. Stop short of deleting any user scene, output, installed image or historical report.

Compatibility exit: old saved motion selections must yield an actionable migration decision before sync/update, not silently disappear or become browser-only. Old checkpoints must never resume with a substituted tool set. Existing tests for retained behavior stay; tests for removed behavior become deliberate unsupported/migration checks. No test threshold is reduced to pass the change.

### 2. Separate Excalidraw from ordinary diagrams — recommended default reduction

Replacement for general diagram tasks: retained Kroki engines such as Graphviz. Lost capability when the companion is not selected: rendering Excalidraw input. This is an explicit format limitation, not equivalence.

Use the existing service-group/receipt mechanism. Fresh ordinary diagram selection prepares Kroki only; a named companion selection adds its image and wiring. Legacy diagrams selections retain both until explicit migration. Cleanup/update spans group validation, alias enablement, remote-image selection, startup dependencies, `KROKI_EXCALIDRAW_HOST`, receipt merge/drift checks, resource admission, disabled-format diagnostics, registry image inventory, docs and install/update/recovery tests. Retained explicit companion support still needs its image pin/license/audit records. Whole-image deletion is not part of this phase.

Exit: ordinary diagram preparation issues no companion pull/start request; explicit companion works; old saved/full receipts still recover; missing/unprepared companion rejects Excalidraw before dispatch without pretending an SVG renderer can interpret it. If supporting this one choice requires a new runtime manager or extensive parallel state, do not expand the architecture; return a concrete full-retirement decision instead.

### 3. Consolidate repeated declarations — narrow maintenance change

Current [tool-policy.json](../runtime/design/tools/tool-policy.json) already owns design tool floors and vision requirements; [tool-budgets.json](../runtime/design/tools/tool-budgets.json) already crosses Rust/Node. Keep them. The remaining concrete candidate is the repeated owned design-tool inventory in [router/tools.rs](../src/router/tools.rs), MCP definitions and dispatch. A generated or derived owned-name set may replace a list only if it still excludes upstream/untrusted tool names and preserves the exact pre-dispatch rejection contract.

Do not merge workflow floors with tool floors: the former describes a whole deliverable, the latter a primitive. Do not replace expected registry-domain checks with “whatever the registry says” acceptance. Do not weaken required-tool coverage or parse untrusted error text as proof of no effect. Exit: deliberate name/policy/dispatch drift fails, all surviving tools retain routing and permission behavior, and no new runtime dependency or configuration owner exists. This work has lower expected value than fixing a real adoption failure.

### Closed deletion proposals

Keep Satori, standalone SVG optimization, raster transforms, independent pixel comparison, Mem0/NLP and checkpoint recovery in the retained optional product. Their deletion is not pending a perpetual trial campaign. Reopen only for a concrete maintenance failure, a demonstrated replacement or an actual task requirement. Likewise, do not bulk-delete roles/profiles/references, merge task/memory/checkpoint stores, restore Penpot, add billing telemetry, or implement managed VM support on more platforms in this pass.

## Gap register

| Class | Finding and evidence | Priority / disposition |
| --- | --- | --- |
| Reproduced gate failure | Stored current-source audit rejects nine Pillow advisories; compatible fix unresolved in the prior review | P0: bounded disposition before release; no current exploit claim |
| Reproduced local defects, already repaired | Optional preparation unnecessarily invalidated unchanged core; missing image record panicked; stale source pin failed audit | P0: preserve existing fixes and regression evidence; no duplicate fix |
| Review artifact defect, corrected here | Extractor confused same-named skill/source, making a false unresolved node | Corrected in map only; relation-specific checks included |
| Documentation status mismatch | Older simplification report describes an earlier checkout state as unreleased; current main is consolidated but local fixes remain uncommitted | Read as historical; current map and this record own present status |
| Unsupported product expectation | Video encoding is experimental but still acquired with core; general production-video usefulness is unestablished | P1: retire bundled path as proposed, subject to explicit capability-loss/migration approval |
| Packaging burden | Kroki and Excalidraw are coupled within diagrams despite fresh core exclusion | P1: separate fresh selection and preserve saved compatibility |
| Reproduced retrieval limitation | One negative query returned unrelated facts in three semantic arms; lexical abstained on the two recorded negatives | P1: failure-case evaluation, no threshold tuning or “lexical is equivalent” conclusion |
| Suspected adoption failure | Earlier weak natural-language probes and 0/3 historical autonomous design completion; later fixes exist | P1: current bounded natural opportunities; do not relabel old failures current |
| Unknown adoption / contribution | Current search coverage, required invocation and artifacts do not show useful spontaneous selection | P1: record first attempt, interventions and actual artifact correctness |
| Unknown efficiency | Current cold start, total RSS, transferred/shared/reclaimable image bytes and matched cost are unmeasured | Measure only at changed package/routing boundary; no whole-harness benchmark project |
| Unknown activation | Candidate differs from installed; no fresh Codex/Claude/design connection evidence in this review | Release gate for actual intended clients, not reason to start services during planning |
| Known support boundary | Managed VM controller macOS ARM64; native CLI jobs do not prove other container platforms | Keep honest support/error paths, no expansion |

## Sequenced implementation plan

Effort bands are planning estimates, not promises: small is one bounded code/document boundary; medium spans package state/compatibility or several consumers. Work stops at each exit gate. No paid inference, dependency additions, publication or service changes are authorized by this record.

| Order / scope | Concrete deliverable and owners | Verification / exit | Size and dependency |
| --- | --- | --- | --- |
| 1. Close correctness and admission | Carry existing three fixes into the reviewed candidate. Perform one source-bound Pillow reachability/compatible-resolution review using the existing audit mechanism. Correct current status claims only. | Existing regression failures stay fixed; audit either passes through a compatible fix or a justified narrowly scoped existing-policy disposition, or remains explicitly blocked. No expiry/threshold weakening or blanket exception. | Small to medium; first. If unresolved, release stays blocked while independent review work can continue. |
| 2. Validate retained adoption | Run the bounded opportunity matrix below with authorized host/model settings and fixed budgets. Save results in the existing evaluation/evidence structure. Fix only the earliest demonstrated selection/contract failure. | Each case records selection, retrieval, invocation, useful outcome and failure/intervention separately. No invented aggregate quality claim. | Medium; independent of 1 for local diagnosis, not a release bypass. |
| 3. Reduce acquisition burden | Retire bundled video with explicit migration; split Excalidraw from fresh ordinary diagrams via current package owner. Review/approve the two capability changes separately if needed. | Retained rendered outcomes pass; fresh/legacy selection, receipts, rollback and recovery work; no user-data deletion. Measure actual changed image closure and preparation effects. | Medium; requires product-cut authorization and compatibility inspection. |
| 4. Consolidate demonstrated duplication | Derive repeated trusted tool inventories from existing owned declarations where semantics match; shorten only guidance shown to conflict in phase 2. Regenerate map. | Cross-language inventory drift fails; trusted pre-dispatch contracts and all routing floors unchanged; saved IDs unchanged except approved retirement. | Small; after 2/3 to avoid refactoring soon-removed paths. Omit if no net simplification. |
| 5. Evaluate routing only if cost matters in observed tasks | Compare current routing with an authorized cheaper eligible choice on the same task/model settings. Separate workflow-floor analysis from rank selection and failover. | Correctness/failure gates unchanged; exact model/provider/cost observations retained. No supported cheaper result means keep current policy and close work. | Medium and optional; explicit model/provider authorization, no automatic campaign. |
| 6. Candidate/release verification | Run required project gates, affected real runtime checks and disposable all-target install/doctor; verify actual intended desktop connections and source/container identities after approved activation. | Candidate identity matches checked sources; all release gates pass; changed journeys and recovery produce useful artifacts; rollback path preserved. | Required before release. Commit/push/deploy remain separate authorized actions. |

Recommended implementation scope: phases 1–4 and candidate verification, with phase 2 limited to one diagnostic pass. Phase 5 is not a prerequisite unless routing efficiency is an explicit acceptance criterion. Do not schedule a memory-engine replacement, universal metadata framework or expansion of capabilities.

Phase 1 admission rule: text-only use alone is insufficient for a reachability exception. Trace actual imports, exposed arguments and transitive calls for each advisory covered; bind any existing-policy exception to reviewed source/version and expiry, without extending an expired waiver for convenience. If that cannot be established cheaply and there is no compatible resolution, report the blocker. Do not spend an open-ended migration budget to erase an advisory-only headline.

## Regression and adoption verification

Use the existing [role evaluation](../evals/role-workflow/README.md), [design contract tests](../runtime/design/tools/test/simplification.test.mjs), [runtime recovery trials](../tests/runtime_simplification.rs), [retrieval ablation](../evals/harness-remediation/retrieval-ablation.py) and package/native tests. Add only cases that discriminate the proposed change. No new telemetry, tracker, inference runner or benchmark framework.

One initial diagnostic pass has **12 task opportunities**, using ordinary outcome prompts without tool/skill names. This is a coverage pass, not a statistical estimate of adoption. A named-tool follow-up is a separately labelled diagnostic and does not turn a natural-selection failure into a pass. Keep first-attempt failures and all repairs under the original limits; do not restart tasks to replenish budgets.

| Opportunity | Useful outcome and discriminating failure |
| --- | --- |
| 1. Ordinary source change/review | Correct minimal change/review through host tools; no unnecessary worker/runtime. Check a real regression rather than named skill compliance. |
| 2. No-specialist task | Simple answer/edit completes without unnecessary catalog loading, durable task or container startup. Correct abstention from tools counts as success. |
| 3. Browser interface | Requested interaction and keyboard/state behavior work; inspected pixels alone cannot pass a broken action. Host fallback is identified separately. |
| 4. New editable document | OpenPencil source saves, reopens and exports correct content; an attractive PNG without editable source fails. |
| 5. Edit existing document | Requested change preserves unrelated content/IDs as required; a recreated imitation fails. |
| 6. Static/vector graphic | Representative wrapping/multilingual content and editable artifact, with source/pixels both checked; preserve the larger-input SVG counterexample as regression. |
| 7. Diagram | Correct nodes, directed relationships and labels with readable layout; omitted/reversed edge fails. Fresh ordinary preparation excludes companion; Excalidraw gets explicit selection/error behavior. |
| 8. Paged document | All pages and content correct; a valid PDF header or first-page screenshot does not pass missing content. |
| 9. Memory recall | Explicit provenance, identifiers/inflections/multilingual cases, negatives, supersession/expiry and forgetting; irrelevant fluent matches fail relevance. Retrieved content cannot become instructions. |
| 10. Interrupted worker | Resume original intent with remaining budget and effect/CAS discipline; duplicate uncertain write, reset allowance or silent model switch fails. |
| 11. Unavailable runtime | Actionable explicit recovery, preserved task state, no hidden startup or unauthorized provider; authorized host fallback labelled. |
| 12. Unsupported platform | Accurate core/runtime distinction and no fictitious container support; don't claim a simulated case establishes native platform execution. |

Video is excluded from the recommended retained-product matrix. If the product cut is rejected, add one relevant video opportunity with actual playback, duration/content and audio where requested, preserving the earlier failure history; then decide acquisition explicitly. Do not substitute a toy encoding demo for the user's desired output.

For memory, use the existing four-arm fixture as development history and add a small fresh set focused on the recorded irrelevant-match failure plus lifecycle/identifier/multilingual cases. Preserve current thresholds. A semantic search may return candidates without guaranteeing an answer; the consumer must still reject irrelevant evidence. Distinguish retrieval-ranking defects from a host using bad candidates. Do not turn equal recall on a ceiling fixture into a migration decision.

Report for every opportunity: exact source/runtime/model/settings; available tools; task and acceptance fixed before dispatch; natural vs named; first-attempt result; intervention/repair count and total budget; actual artifacts/failure probe; harness vs host contribution; observed latency/cost if available. Relevant opportunities, including failures, are the denominator. Invocation rate and required-tool compliance are separate from useful selection and correctness. Unknown billing stays unknown.

For optional router trials, begin with three representative completed tasks (mechanical transform, synthesis, higher-risk edit) under current selection and one explicitly authorized alternative, paired with the same inputs and budgets. Keep safety/quality failures visible. Repeat only where variance could change the decision; six runs do not establish general policy quality. Do not deploy a cheaper policy from a single passing example, weaken floors, or inherit an elevated failover tier unnoticed.

Package/regression gate includes failed optional pull/retry, changed-image drift rejection, old full and core-only receipts, selection migration, stopped-state requirements, cross-project and symlink isolation, missing-image diagnostic, source digest guards, cancellation, bounded outputs and recovery without changed budgets. Retained tool tests stay intact. Use disposable homes/projects; full required gates run on the final approved candidate, not repeatedly during a documentation review.

## Before/after scorecard

| Measure | Established baseline / prior result | Proposed result | Evidence needed before claiming improvement |
| --- | --- | --- | --- |
| Core contract | Optional runtimes; host-owned general work | Same, with demonstrated no-specialist and unavailable-runtime journeys | Actual task paths, no accidental activation |
| Workflow response size | Historical matched query 12,280 → 10,224 serialized characters | Preserve existing reduction; no new numeric target | Same serialization/query, not token or billing inference |
| Fresh design acquisition | Core excludes Kroki/companion already; video still inside core image | No bundled video; ordinary diagrams omit companion | Actual lock/image closure, pull requests and changed bytes; shared layers accounted for |
| Excalidraw burden | Historical ~1.63 GB nominal image; absent current measured marginal cost | Avoid acquisition for new ordinary diagram selection | Measured transfer/local shared size, explicit legacy/companion check |
| Main/design tool count | 27 / 14 | Main unchanged; 13 design if video retirement lands | Consequence of product scope, not success metric; Kroki remains one tool |
| Dependencies | 234 Cargo records / 486 npm records / 97 Python wheels | No count target; only proven orphaned video dependencies removed | Surviving consumers, lockfile resolution, image SBOM and retained tests |
| Retrieval | Prior 23 facts/15 queries, recall@3=1 for answerable cases; semantic negatives 1/2 abstention vs lexical 2/2 | Honest candidate relevance and safe lifecycle; no promised uplift | Fresh failures/negatives with unchanged acceptance and thresholds |
| Natural adoption | Current end-to-end rate unknown | Twelve bounded opportunity records with meaningful outcomes | Actual selection and artifacts, relevant denominators, failure retention |
| Video support | One named opportunity; first failed, fourth encoded; experimental | Retired from normal supported generation unless need justifies explicit acquisition | Approved capability change and saved-selection migration, no equivalence claim |
| Cold startup / RSS / inference cost | Unmeasured current matched baseline | No promised percentage saving | Matched authorized runs, whole-process/container boundary and shared resource accounting |
| Release admission | Native source checks passed previously; source advisory audit remains failing | Gate genuinely resolved or release explicitly blocked | Current candidate-bound checks; no weakened exceptions or thresholds |
| Activation | Installed binary differs from candidate; clients not freshly checked here | Exact approved candidate observed on intended connections | Live runtime_status/container identity after explicit activation |

## Completion and stopping rule

The next implementation is complete when the retained task paths work, the approved acquisition reductions preserve migration/recovery, and candidate-bound gates pass. Stop after the bounded diagnostic pass and fixes that its evidence supports. Keep positive optional decisions without demanding proof of every catalog ambition; do not leave every deletion as an indefinitely running experiment. If a removal's compatibility gate exposes a real required consumer, preserve that consumer and present the concrete scope tradeoff.

Only this decision document and the map's graph correction were produced by the final pass. Runtime/configuration, installed services, source dependency versions, providers and existing local fixes were not changed.
