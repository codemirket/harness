# Independent architecture review

Date: 2026-10-24. Scope: the proposed fresh rewrite, not the existing harness implementation. Reviewer role: Architecture Reviewer. This report does not approve implementation, dependencies, publication, or a change to an accepted product decision.

## Executive verdict

**Keep the main architecture. Tighten the shared lifecycle and recovery contracts, and reorder feasibility work before treating the plan as implementation-ready.** Host-owned agents and ordinary commands, one Rust application, one compiled TypeScript adapter package, private on-demand tools, canonical SQLite memory, and a complete per-target distribution are a coherent design. There is no demonstrated reason to replace them with a service fleet, graph database, agent runtime, plugin framework, VM, or a generic workflow engine.

The real complexity is the number of privately maintained tool/runtime closures and the interaction of multiple stdio servers with installation-wide resources and persistent state. One crate and one npm package do not remove those obligations. Conversely, the presence of many binaries is not by itself overengineering: much of that inventory follows directly from the accepted capabilities, private-packaging requirement, and three release targets. Reducing languages, removing Windows ARM64, dropping saved profiles, making tools optional downloads, or using the user's installations would change the product contract. I do not recommend silently making any of those changes.

Six findings below identify consequential contract omissions or execution-order problems. None is a measured implementation failure. The first three deserve resolution before consumers of those contracts are built; the remaining three should shape the foundation and qualification work. These changes do not require another broad architecture questionnaire or another round of general tool research.

## Evidence and limits

- Read the implementation plan, detailed proposal, decisions, platform matrix, and relevant toolchain sections. Applied `skill-catalog` and `engineering-judgment`, including its boundary/failure reference. No other reviewer findings were used.
- Verified all six planning-file SHA-256 values against `baseline.json`; all matched. The repository reports untracked planning content. No application implementation was reviewed or inferred.
- No builds, installers, tool launches, dependency operations, runtime probes, client interactions, or network requests were performed. Only this report was written.
- **Measured facts:** no runtime, performance, package-size, fidelity, or client-activation measurements are available. Hash equality is file-identity evidence only.
- **Source-review facts:** the cited documents contain the contracts and sequencing described below. **Inference:** the failure scenarios show what a conforming implementation still needs to disambiguate; they do not establish that an implementation has already failed.
- No new external technical claim is needed for these findings, so no external source was fetched or relied on as independent verification. Upstream URLs in the planning documents remain their authors' research evidence, not newly verified evidence from this review.

Severity means expected consequence if the ambiguity survives into implementation: **High** risks incorrect effects, data recovery, or shared-state ownership; **Medium** risks starvation, avoidable redesign, or unreliable completion. Status uses the requested labels: `overengineered`, `overcomplicated`, `insufficient`, or `improvable`.

## Findings

### A1 — Specify connection-owned processes and installation-wide leases as separate concepts

**Severity:** High. **Status:** insufficient. **Confidence:** High that the ownership contract is incomplete; medium on which failure an eventual implementation would exhibit.

**Locations:** [implementation-plan.md:62](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:62), [implementation-plan.md:126](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:126), [proposal.md:46](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:46), [proposal.md:128](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:128), [proposal.md:378](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:378).

**Evidence and concrete failure:** Each client can launch its own stdio server; browser sessions belong to their initiating connection; disconnect cleans up owned work; heavy-job/analyzer limits and activation exclusion span the installation. Analyzer reuse is described by project/release validity, without saying whether live processes can be reused across connections. A server can disappear while a child is still finishing or failing to terminate. If its admission/profile lease becomes available immediately, another client can reuse that profile or an update can migrate state while the old child still uses it. Conversely, retaining a stale record indefinitely makes the installation permanently busy. Sharing an analyzer between connections also makes "owner disconnect" ambiguous.

This is more than selecting a lock API: it determines which process is allowed to use or release each resource. The plan correctly calls for PID-reuse and disconnect tests, but leaves the invariant they must establish implicit.

**Smallest complete recommendation:** Make every live analyzer, adapter, browser session, and job connection-owned initially. Reuse a live analyzer only within that owner's valid checkout/release context. Share durable memory and qualified persistent caches, not live process handles. Maintain a small installation-wide admission/ownership record protected by native locking; a resource is not reusable until the owning work is confirmed stopped or recovery has established that it cannot still use the resource. Unknown ownership stays busy/recoverable. The same ownership source must drive job limits, saved-profile exclusion, cleanup, and update admission. Specify a lock order for maintenance, resource admission, and scope mutations; do not hold a global lock throughout a ten-minute job. Platform-specific cleanup can differ without changing these invariants.

**Verification:** Kill one stdio server during a child operation, immediately start a competing client and updater, and verify no profile reuse, over-admission, or activation occurs until old ownership is resolved. Disconnect one client while the other continues a same-project lookup.

**Tradeoff / product contract:** This can duplicate analyzer startup across clients and reduce cache warmth. It preserves all accepted capabilities and the no-broker topology. If cross-client live analyzer reuse is desired, that is a distinct owner/IPC design, not an incidental interpretation of "reused"; it should not slip into 0.0.1 unnoticed.

### A2 — Make forgetting metadata survive restore and interrupted recovery

**Severity:** High. **Status:** insufficient. **Confidence:** High on the missing recovery invariant; no claim that the planned implementation already violates it.

**Locations:** [proposal.md:477](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:477), [proposal.md:479](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:479), [proposal.md:142](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:142), [decisions.md:148](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/decisions.md:148), [implementation-plan.md:132](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:132).

**Evidence and concrete failure:** The plan promises that known forgotten records cannot return through managed snapshots or older imports. It mentions content-free deletion identifiers and requires applying known forgetting records during restore, but does not say whether that knowledge lives inside the state being replaced. If a scope restore also replaces its deletion metadata, an old export can become importable again. If forgetting removes live records, then crashes before marking an in-flight backup ineligible, a snapshot can capture the pre-forget content and later be accepted. Index revision checks do not solve either case.

The documents already acknowledge backup, cap, and interruption work. The additional issue is that deletion knowledge has a different rollback policy from ordinary memory; it must be an explicit architecture invariant rather than an optional metadata convenience.

**Smallest complete recommendation:** Give forgetting a durable content-free exclusion record, keyed by stable scope/record lineage, that ordinary scope restore and update rollback cannot erase. Before acknowledging forgetting, make all relevant read/import/restore paths consult that record and make affected snapshots ineligible. Physical removal/invalidation can then follow a recoverable bounded operation, with incomplete cleanup visible. Backups finishing concurrently must validate against the current forgetting state before admission. Do not invent content fingerprints or semantic matching to recognize arbitrary rewritten copies of forgotten text: retain the documented boundary around known identities and external exports.

**Verification:** Forget after creating an old export; restore an earlier scope snapshot; import the old export; interrupt each transition and retry. The deleted lineage must stay unavailable while unrelated scopes and valid recovery points remain usable.

**Tradeoff / product contract:** Adds a small explicit piece of non-rollback control state and a completion rule. It does not add a memory engine or change the agreed forgetting outcome. If the team instead chooses best-effort deletion only, that would materially weaken decisions 21/23/63 and needs an explicit product decision.

### A3 — Represent uncertain effects in the adapter/result contract

**Severity:** High. **Status:** insufficient. **Confidence:** High that the stated result categories do not define this distinction.

**Locations:** [implementation-plan.md:124](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:124), [implementation-plan.md:47](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:47), [implementation-plan.md:65](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:65), [proposal.md:204](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:204).

**Evidence and concrete failure:** The proposed envelope has complete/partial/unsupported/error states, and the plan correctly prohibits replay after uncertain effects. It does not define how uncertainty reaches the host. A browser submission can occur before timeout; an output can finalize before the adapter's response is lost; a memory mutation can commit before disconnect. Returning an ordinary error or treating cancellation acknowledgement as proof of rollback would encourage a duplicate external action or an incorrect "nothing changed" answer. "Partial" also needs to distinguish known partial output from unknown effect occurrence.

**Smallest complete recommendation:** Keep the existing typed envelope, but define transport/execution status separately from effect knowledge: for example, `not_attempted`, `confirmed`, and `unknown`, plus the job/operation identity and available evidence. Cancellation acknowledgement means the cancellation request was accepted, not that no effect occurred. After a finalization boundary, report committed output where known. For local jobs, retain a bounded receipt sufficient to inspect the committed path/revision after a lost response; avoid retaining document content or creating a durable agent journal. For web actions with no reliable reconciliation mechanism, return unknown and let the host inspect the actual page/service before a new action. Do not add a universal exactly-once or automatic-retry subsystem.

**Verification:** Drop the adapter response immediately after local finalization; time out a browser action after dispatch; cancel concurrently with memory commit. The result must neither claim rollback nor automatically replay, and the host must be able to distinguish a definite failure from an uncertain outcome.

**Tradeoff / product contract:** Adds one narrow semantic distinction and bounded receipts where useful. No capability or permission change. This implements the already accepted no-uncertain-replay contract rather than introducing durable session resumption.

### A4 — Define the atomic artifact unit before promising safe finalization for all outputs

**Severity:** Medium. **Status:** insufficient. **Confidence:** High on the multi-file mismatch; medium on how often users would encounter it.

**Locations:** [implementation-plan.md:47](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:47), [implementation-plan.md:126](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:126), [proposal.md:204](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:204), [proposal.md:399](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:399), [proposal.md:403](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:403).

**Evidence and concrete failure:** Staging and source/destination conflict detection are specified generically, but deliverables include source plus PDF, HTML plus local assets, PNG frame sequences, token outputs, and handoff directories. Replacing the principal file successfully does not establish that its companion files were committed. Cancellation between asset and source replacement can leave a usable-looking but mixed-version artifact. In-place edits to user-owned multi-file trees also encounter writers outside Mirket's locks.

**Smallest complete recommendation:** Define three explicit finalization cases: a single file; a new owned artifact directory/archive containing a manifest of its members; and host-owned project edits. Stage and validate every member of a new multi-file artifact together, then publish its containing artifact root using a qualified operation. Treat source/PDF correspondence and linked assets as part of verification. Keep ordinary source-code edits under the host's established edit workflow rather than forcing them through an all-purpose artifact transaction. For explicitly requested in-place multi-file changes, declare the supported conflict/commit boundary and preserve recoverable originals; return unsupported if the adapter cannot meet it. Do not imply arbitrary external editors participate in Mirket's locking.

**Verification:** Interrupt between every member write of a source/PDF and HTML/assets export; concurrently alter a destination asset. The visible result must be the previous complete artifact, the new complete artifact, or an explicit non-final recovery state—not a successful mixed set.

**Tradeoff / product contract:** New-copy behavior remains unchanged. Restricting any previously implied arbitrary multi-file in-place atomicity is a contract clarification and potentially a narrowing; record it explicitly rather than silently treating the principal file as the whole artifact. Avoid solving this with a general filesystem transaction framework.

### A5 — Resource limits need reclaim-on-demand and workload classification

**Severity:** Medium. **Status:** improvable. **Confidence:** High on the idle-slot timing conflict; resource-size consequences remain unmeasured.

**Locations:** [implementation-plan.md:61](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:61), [implementation-plan.md:62](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:62), [implementation-plan.md:63](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:63), [implementation-plan.md:66](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:66), [implementation-plan.md:76](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:76), [proposal.md:518](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:518).

**Evidence and concrete failure:** Two analyzer processes may remain idle for five minutes while additional demand queues; an ordinary navigation call has a 60-second deadline. A third-language request can therefore time out while both occupied slots are doing no work. It is unspecified whether queue time consumes the execution deadline. Separately, the heavy-job category lists render/export/extraction, while analyzer initialization and semantic index rebuilding can also consume significant resources; only heavy admission is explicitly deferred by memory pressure. The embedding runtime has no stated idle-unload owner, despite two resident stdio servers and the aggregate 250 MiB idle target.

**Smallest complete recommendation:** Treat five minutes as a maximum idle retention period, not a minimum reservation. Reclaim an idle analyzer immediately when a valid waiter needs its slot; never preempt an active operation just to fit the default. Distinguish a bounded queue deadline from the execution deadline and report both. Define a small fixed workload classification for the finite inventory, including analyzer initialization, embedding rebuilds, and ML extraction; apply pressure admission to new memory-heavy work across categories. Explicitly decide when each server's embedding runtime loads and unloads, and measure the two-client case. Start with lazy loading and bounded idle retention; do not introduce a model service solely to pursue the idle number. Use one consistent installation-wide accounting view as in A1.

**Verification:** Two clients hold idle analyzers; a third language must progress without waiting for the full TTL. Run two heavy jobs plus a semantic rebuild and analyzer cold start; confirm bounded admission, useful cancellation, and honest pressure reporting. Measure idle memory after prior semantic use, not just before the model was ever loaded.

**Tradeoff / product contract:** Cold analyzer/model starts may increase while idle retention becomes predictable. This preserves throughput-first concurrency and all language/browser coverage. The 8 GiB and 250 MiB figures remain hypotheses to measure, not proven capacity or hard safety limits.

### A6 — Narrow the M0 critical path and bring update/reconnect feasibility forward

**Severity:** Medium. **Status:** overcomplicated. **Confidence:** High on the sequencing mismatch; no elapsed-time estimate is claimed.

**Locations:** [implementation-plan.md:109](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:109), [implementation-plan.md:113](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:113), [implementation-plan.md:117](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:117), [implementation-plan.md:120](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:120), [implementation-plan.md:126](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:126), [implementation-plan.md:165](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/implementation-plan.md:165), [proposal.md:116](/Users/nazmiyilmaz/Projects/harness-rewrite/planning/proposal.md:116).

**Evidence and concrete cost:** M0 simultaneously spans three browsers, design, authored video/audio, native Office, extraction, direct PDF editing, SQL binding, several of the hardest language loaders, and native semantic memory on three targets. Yet its stated output is a "small working packaged path." Actual two-client invocation/ownership sits in M1, while update/reconnect and interrupted activation are concentrated in M5. This makes the first milestone almost an entire dependency-admission campaign, while the shared no-broker installation lifecycle—the architecture all those adapters depend on—can remain unproven until substantial integration exists.

The existence of qualification work is not the finding; the plan already acknowledges it. The avoidable problem is its dependency order and overly broad M0 exit boundary.

**Smallest complete recommendation:** Keep the same deliverables and release gates, but split M0 internally. First prove one minimal packaged Rust-to-child path through both clients on all three targets, with two-server ownership, cancellation, private paths, and a synthetic two-release activation/reconnect/crash fixture. A synthetic state migration is enough to test the architectural boundary; this is not public release preparation. Then sequence bounded risk experiments: passive analyzer loading and private Office/Windows ARM64 extraction first because they can force genuine product/dependency choices; follow with the remaining design/video/PDF/SQL/memory slices. Admit exact dependency batches per slice. Carry successful slices into M1/M2 instead of holding all foundation work behind every M0 experiment. M5 still verifies the complete assembled, signed distribution and full recovery matrix.

**Tradeoff / product contract:** Adds an early executable installation skeleton and focuses the first integration loop. It does not cut a capability, delay Windows to a later release, bypass approval, or qualify a real tool through a fake. Moving signing/provider selection to M5 remains reasonable; moving all activation mechanics there does not.

## What should stay

1. **Host ownership:** Agents, model choices, normal commands, Git, builds, and source editing stay in the host. Mirket tool supervision is not agent orchestration. Do not reintroduce routing, worker journals, provider adapters, or a second permission system.
2. **Small source architecture:** One crate with focused modules, one adapter package, one packaging inventory. Split process boundaries for incompatible runtimes and expensive/crash-prone work; do not equate process count with a need for multiple application services or crates.
3. **Embedded memory:** Canonical records, revision checks, exact retrieval, semantic candidates, scope/currentness rechecks, explicit supersession, and host curation fit this product. Keep relational links; no demonstrated need for a general graph ontology, automatic consolidation agent, learned reranker, or multi-backend fallback framework. Define those only if acceptance evidence later establishes a gap.
4. **Private runtime ownership:** The two Python owners are justified by the stated ABI/architecture boundary; merging them merely to make the diagram smaller is not a reliable simplification. Likewise, reuse browser/runtime files only after compatibility is demonstrated. A single coordinated release need not mean one interchangeable runtime for unrelated consumers.
5. **One active release and explicit updates:** Immutable release contents, separate user state, stage/busy behavior, maintenance admission, and no automatic post-write rollback are good simplifications. Keep the removed custom signing/channel machinery removed. Publisher and native-signing details remain operational release work.
6. **Bounded capability claims:** Source-first PDF pagination, qualified direct edits, structural Kotlin, offline SQL with proven bindings, explicit SDK prerequisites, and per-engine browser reporting are sound boundaries. Rejecting unsupported input honestly is preferable to silently changing the requested operation.
7. **Actual artifact/client evidence:** Reopen/edit/render checks, independent formula expectations, no-project-execution sentinel fixtures, and real native Windows tests are appropriate. Neither source availability nor a Rust cross-build establishes the private tool fleet's behavior.

## Implement-before / implement-after ordering

| Before building dependent behavior | Then implement and qualify |
| --- | --- |
| Define A1 ownership/lease invariants and A3 effect states; choose minimal stable launch/activation mechanism | Thin real two-client, three-target slice; disconnect/crash/admission/update tests with disposable state |
| Define supported artifact units and finalization boundaries (A4) | Single-file and directory artifact paths; expand tool adapters without duplicating commit logic |
| Define canonical record identity, restore boundaries, and non-rollback forgetting authority (A2) | Exact memory, corrections/forgetting/restore first; semantic indexing and context assembly on top of those invariants |
| Define idle reclamation and resource categories (A5) | Measure analyzer/model retention and combined workloads; tune proposed numbers using actual evidence |
| Bound each M0 risk slice and its concrete dependency batch (A6) | Passive analyzers and private Office/ARM64 extraction, then remaining risky adapters; keep full M3 coverage as a release requirement |
| Keep distribution mechanics continuously exercised on small payloads | Complete payload assembly, native signing, full installation/upgrade matrix, rights review, and resource measurements in M5 |

Do not postpone the shared workflow's minimal invocation/scope/effect rules entirely to M4: use enough guidance during the first client slice to exercise them. The twelve complete guides and their source-rights review can remain M4 work.

## Acknowledged gaps that are not additional findings

Windows ARM64 x64 helpers, private Office/UNO relocation, native FFmpeg closure, seven-language OCR fidelity, OpenPencil file-mode behavior, HyperFrames compatibility, passive analyzer loading, native signing, client workspace metadata, exact model quality, and resource sizes are already explicitly unqualified. They are real gates, but repeating them as newly discovered architecture defects would overstate this review.

The complete all-target package will require substantial release maintenance even if every recommendation here is adopted. A smaller footprint through fewer capabilities or optional packs is a possible different product, not a free architectural optimization under the present choices. There is no evidence here warranting that scope change before M0.

## Material decisions remaining

No new provider, platform, framework, or broad product choice is needed from the user on this source review alone. The implementing owner can propose the concrete A1–A3 invariants and A5–A6 ordering within the approved direction. A4 needs an explicit supported boundary if “in-place editing” was intended to promise atomic updates of arbitrary multi-file projects. Any demonstrated M0 inability to satisfy passive analysis, private distribution, or an accepted artifact operation must return as a concrete capability tradeoff; a failed experiment is not permission to add a fallback framework or silently reduce support.
