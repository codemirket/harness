# Review fixes and second-pass outcome

2026-10-24. The user authorized applying the independent reviewers’ recommendations and repeating the review. All six planning documents were revised. Four fresh independent reviewers ran with `gpt-6-astra`, High reasoning; each formed an initial assessment before consulting its own preceding report for closure. Three ran concurrently; Toolchain started when a slot opened. No peer second-pass reports were supplied to the reviewers.

**Result: the revised plan is ready for complete-plan agreement. No outstanding planning correction remains from these reviews.** The second pass produced three nonblocking refinements; all were applied and checked by the affected reviewers. This is planning closure, not proof of runtime correctness, performance or platform readiness.

## What changed

| Area | Applied correction | Owning document |
| --- | --- | --- |
| Host and guide integration | Canonical guide corpus, thin qualified host entries, release-bound reads, explicit operation scope and a pre-effect full-release check, including stale guidance retained across reconnect. | [Architecture contracts](../../../proposal.md#host-tool-and-evidence-contracts) |
| Jobs and process ownership | Bounded start/status/cancel/result calls, logical operation identities, effect knowledge, reconnect receipts, connection-owned children and one installation admission record. Abrupt death cannot rely on a graceful shutdown callback. | [Architecture contracts](../../../proposal.md#host-tool-and-evidence-contracts) |
| Artifacts and browsers | Finite inspection/edit operations, complete publication units, saved-revision evidence, scoped browser observations and stale-target rejection. Actual host policy must distinguish inspection from submissions even with asynchronous calls. | [Architecture contracts](../../../proposal.md#host-tool-and-evidence-contracts) |
| Memory and recovery | Forgetting exclusions survive older restores/rollback; source expansion, constraint applicability/overflow and task correlation are explicit. Continuation retains evidence and uncertain effects without collecting transcripts. | [Architecture contracts](../../../proposal.md#host-tool-and-evidence-contracts) |
| Resource behavior | Idle analyzer reclamation on demand, separate queue/execution deadlines, heavy admission for initialization/rebuilds and bounded embedding lifetime. Numeric budgets remain hypotheses to measure. | [Historical implementation defaults](../../../implementation-plan-2026-10-24-historical.md#4-practical-starting-defaults) |
| Toolchain | Removed the invalid Windows Phpactor route; PHPantom remains a native qualification candidate. Prioritized passive Ruby/PHP tests, corrected extraction renderer evidence and required complete feature/native/assembly inventories and an all-target extraction comparison. | [Toolchain](../../../toolchain-proposal.md), [platform matrix](../../../platform-qualification.md) |
| Guide methods and rights | Preserved the original 52 sources and added six focused method references, for 58 unique entries. Added exact section/reuse mappings, link-only treatment for restricted content and controlled comparative guide evaluation. No upstream workflow/runtime was imported. | [References](../../../references.md), [Historical M4](../../../implementation-plan-2026-10-24-historical.md#m4--author-and-exercise-the-guides) |
| Sequence | M0a proves a small real-client guide/tool/inspection and activation path; M0b qualifies bounded risks. Successful slices advance without waiting for the entire fleet. Full capability and all three target gates remain required for release. | [Historical milestones](../../../implementation-plan-2026-10-24-historical.md#6-implementation-milestones) |

Decision 77 records the authorized amendments. Decisions 1–76 remain verbatim. The host-owned architecture, complete private `.mirket` distribution, full 0.0.1 capabilities, two Python owners and simplified release strategy remain intact. No VM, WSL, broker, universal executor, extra memory engine, agent journal or compatibility framework was added.

## Second review and resolution

| Reviewer | Independent conclusion | Refinement and final disposition |
| --- | --- | --- |
| [Architecture](architecture.md) | All six prior findings closed in the plan; no architecture planning blocker. | A2.1: byte-cap eviction can precede seven days. Added a fresh-versus-expired identity rule independent of receipt presence, age/byte-eviction failure fixtures, and available reconciliation under capacity pressure. Reviewer confirmed closure. |
| [Capability](capability.md) | Prior capability gaps addressed; one P2 evaluation correction. | C2-1: added clean matched contexts/configurations, frozen held-out criteria, independent/deterministic outcome checks, retained failures and variability-sensitive repetitions. Reviewer confirmed closure. |
| [Integration](integration.md) | All nine prior integration omissions closed; no planning blocker. | Added pre-effect full-release binding and a same-conversation stale-guide fixture. Explicitly preserved asynchronous tool-policy distinctions and receipt inspection after activation. Reviewer confirmed closure. |
| [Toolchain](toolchain.md) | All six prior toolchain findings closed; no new actionable planning defect. | No further tool replacement, broad alternatives survey or dependency addition was justified. Actual backend/package qualification remains assigned work. |

The affected reviewers checked their final amendments against `baseline.json`’s `final_files` hashes and the preserved original inputs. Their appended closure sections distinguish these checks from the independent frozen-baseline reviews. Toolchain files were unchanged after its review; adjacent lifecycle/evaluation refinements were checked by their respective reviewers and integrated by the lead.

## What still needs executable evidence

The plan deliberately assigns these to implementation after complete-plan agreement and exact dependency admission:

- Actual guide delivery, protocol facilities, long jobs, release binding and teardown in both desktop Code surfaces on macOS ARM64 and Windows x64/ARM64.
- Useful passive analyzers, native PHP feasibility, private Office relocation, Windows ARM64 workers and the complete extraction comparison.
- Saved document/design/media fidelity, memory/recovery failure cases, package/license closure and measured resource behavior.
- Complete normal-user installation/update/recovery/uninstall and release verification, preserving the separate publication authorization.

These are named qualification tasks with failure paths. They are not claimed passes, additional planning blockers merely because they are unexecuted, or permission to shrink the required capabilities silently.

## Verification and preservation

The lead checked the revised documents and all four reports, traced the second-pass refinements to their final wording and verified local Markdown structure/cross-references. The six frozen review inputs and six final document identities are recorded in [baseline.json](baseline.json); final validation results are in [verification.json](verification.json). The reference count was checked as 58 unique entries across 59 memberships. All 53 original memberships and decisions 1–76 were preserved, and all five first-pass report artifacts retain their recorded hashes.

Pre-edit and frozen-review copies plus the complete unified diff are retained outside shipped source under `/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-review-2/`. Historical reviews remain unchanged evidence of the earlier state. The active six planning documents contain the current plan; this report is a review record, not another execution plan.

No application code, tool runtime, dependency installation, client configuration, old harness or existing installation was changed. No commit, push or publication occurred. Review work ran on the supplied macOS workspace; there were no Windows, browser, Office, analyzer or model-runtime tests. The next step is complete-plan agreement followed by M0a and concrete dependency admission.
