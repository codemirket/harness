# Independent architecture review — pass 2

2026-10-24. Architecture Reviewer; planning only. Reviewed the six files named in this directory's `baseline.json`. All six SHA-256 values matched before assessment. Formed the current architecture assessment before reading the previous architecture report and synthesis; did not read current peer reports. Applied `skill-catalog`, `engineering-judgment` and `work-planning` proportionally. No runtime, installer, dependency, client or old-harness operations were performed.

## Verdict

**No architecture planning blocker remains. Keep the revised architecture and milestone order.** The first-pass ownership, forgetting, effect knowledge, artifact finalization, resource admission and sequencing findings are closed at the planning-contract level. Their runtime acceptance tests remain necessary; closure here does not claim they have passed.

The added contracts belong to the existing Rust owners and address actual cross-client effects. They do not justify a broker, agent journal, universal transaction framework, additional memory service or plugin system. Host-owned agents/tools, one Rust crate and TypeScript adapter package, complete private `.mirket` payloads for all three targets, embedded memory, two Python owners and the simple release strategy remain coherent. Runtime/component breadth follows the selected capability scope; removing platforms or capabilities would change that scope rather than merely simplify implementation.

One narrow refinement below should be settled when defining M0a/M1 operation schemas. It does not require another product decision or block complete-plan approval.

## Prioritized refinement

### A2.1 — Preserve the fresh-versus-expired distinction after receipt eviction

**Priority:** Medium, nonblocking. **Classification:** improvable. **Confidence:** high on the edge case; this is not an observed implementation defect.

**Evidence:** `planning/proposal.md:221` makes repeated starts with the same operation ID return the known operation; `planning/proposal.md:223` adds seven-day retention under a 50 MiB budget and correctly prohibits automatic replay of expired/unknown IDs. `planning/implementation-plan.md:112` and `planning/implementation-plan.md:128` assign response-loss reconciliation and operation identity to M0a/M1. The no-replay invariant is sufficient as an architecture requirement, but receipt eviction is not yet a named discriminating fixture, and the text does not state whether seven days is an upper retention horizon when the byte cap is reached sooner.

**Failure example:** a completed operation loses its response. Enough subsequent completions evict its ordinary receipt before seven days. After reconnect, the host repeats the original start ID. An implementation using “receipt absent means first start” can repeat the mutation; rejecting every absent ID instead can prevent new operations. The existing no-replay rule already forbids the former, so this is a refinement of how that rule is proved, not a newly missing safety promise.

**Smallest complete change:** clarify that ordinary receipts are retained **up to** seven days subject to the byte cap; absence of a receipt never establishes that a submitted identity is fresh. Require the operation schema to distinguish a valid fresh start from an expired/replayed identity independently of ordinary receipt presence, and keep retries requiring reconciliation/new intent explicit. Choose the concrete issuance/expiry mechanism during M0a/M1; do not prescribe a general idempotency framework in the plan. Add one fixture: force byte-cap eviction before the age horizon, reconnect, retry the old ID and prove no effect repeats, then prove a genuinely fresh operation can still start. Also exercise age expiry.

**Tradeoff:** a caller may have to create a new intent explicitly after expiry; bounded storage cannot promise indefinite result replay. This preserves current scope, bounded receipts and the no-automatic-replay guarantee. It does not require permanent per-operation tombstones or retaining artifact bodies. Unresolved commit/ownership receipts remain protected under the existing rule.

## First-pass architecture closures

| Prior finding | Current evidence | Assessment |
| --- | --- | --- |
| A1 — Connection ownership versus shared leases | `proposal.md:225`, `proposal.md:227`; `implementation-plan.md:130` | Closed. Live children remain connection-owned; one installation record governs admission, profiles, cleanup and activation. Descendants must be proven stopped, PID reuse is considered, lock order is explicit, and unknown ownership stays recoverable/busy. Platform APIs and targeted recovery mechanics belong to M1. |
| A2 — Forgetting survives recovery | `proposal.md:241`, `proposal.md:511`, `proposal.md:513`; `implementation-plan.md:136` | Closed. Content-free lineage exclusions live outside ordinary restored/rolled-back state, become authoritative before acknowledgement, govern concurrent backup completion, and preserve the stated external-copy boundary. Physical cleanup has an explicit incomplete state. |
| A3 — Uncertain effects | `proposal.md:221`, `proposal.md:223`; `implementation-plan.md:128` | Closed. RPC and operation identities differ; execution status and effect knowledge differ; cancellation is not rollback. Reconnect inspection and bounded receipts support local reconciliation, while unknown external submissions are not replayed. A2.1 strengthens the expiry test rather than reopening this design. |
| A4 — Artifact publication unit | `proposal.md:229`, `proposal.md:231`; `implementation-plan.md:140` | Closed. Single-file or new owned directory/archive publication includes all members and linked/source correspondence. Host edits retain their owner. Explicit in-place multi-file work requires a supported recoverable boundary; a material capability loss returns for a decision. No arbitrary filesystem transaction is promised. |
| A5 — Resource classification and reclamation | `implementation-plan.md:63`, `implementation-plan.md:64`, `implementation-plan.md:68`, `implementation-plan.md:69`; `proposal.md:225` | Closed. Initialization/rebuild work receives heavy admission; combined slots are reserved atomically; idle analyzers are immediately reclaimable; queue and execution deadlines differ; embeddings unload. Numeric defaults remain measured starting points, not correctness guarantees. |
| A6 — M0 critical path and early activation | `implementation-plan.md:110`, `implementation-plan.md:112`, `implementation-plan.md:114`, `implementation-plan.md:118`; `platform-qualification.md:100` | Closed. A small real-client/guide/child/inspection slice includes synthetic update/crash/reconnect. Bounded risk slices follow, passive analyzers and Office/extraction lead, and successful slices can advance independently. Target access remains a release gate without blocking unrelated work. |

## Remaining implementation work is appropriately assigned

Exact locks/process identity, receipt schemas, filesystem finalization primitives, scoped restore/import mechanics and per-backend operation schemas are implementation responsibilities under explicit invariants, not reasons for another architecture questionnaire. M0a should freeze only the shared contract demonstrated by its useful journeys; M0b should not become another all-tools-before-foundation gate.

Private Office relocation, passive Ruby/PHP and other analyzers, Windows ARM64 helpers, direct PDF fidelity, client delivery and native release trust remain explicitly qualified work. The current documents neither claim them verified nor silently reduce the accepted scope. The revised native PHP candidate and one bounded extraction comparison preserve one selected implementation per capability (`toolchain-proposal.md:149`, `toolchain-proposal.md:305`, `toolchain-proposal.md:307`). No additional backend research or runtime subsystem is warranted solely because those tests have not run.

This review makes no new external technical claim: its conclusions concern the frozen documents and their internal contracts. Upstream evidence was not revalidated, and no performance, fidelity, platform support or actual client activation was measured. Complete-plan approval, exact dependency admission, required target access and eventual publication authorization remain separate from this review.

## Amendment closure — 2026-10-24

**A2.1 is closed at the planning-contract level; no residual architecture issue or planning blocker remains from this refinement.** Checked the amended `proposal.md:223` and `implementation-plan.md:112` against the frozen reviewed inputs. Retention is now explicitly up to seven days subject to earlier byte-cap eviction; receipt absence cannot establish freshness; a bounded issuance/expiry rule must distinguish fresh from expired/replayed identities independently of receipt presence. The required test covers early eviction, age expiry, reconnect/retry without another effect and a genuinely fresh start. Protected-receipt pressure now defers capacity-requiring mutations while preserving inspection, status and reconciliation, avoiding a recovery deadlock. No permanent per-operation tombstone store or indefinite replay guarantee was introduced.

Read the adjacent release-binding/receipt-migration amendments and the changed implementation-plan passages for interaction with this closure; no conflict with A2.1 was found. This was a targeted amendment check, not a fresh audit. Verified all six current SHA-256 values against `baseline.json`'s `final_files`, and all six preserved reviewed inputs against its original `files` values. Actual issuance/expiry behavior and the named failure fixtures remain M0a/M1 implementation evidence to produce; no runtime checks were performed.
