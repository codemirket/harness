---
name: agent-coordination
description: Coordinate distinct professional agents with task-scoped assignments, independent judgments, dependencies, evidence-based disagreement resolution and integrated verification. Use for substantial multidisciplinary work or explicit collaboration.
---

# Agent coordination

## Work as distinct professionals

Use independent assignments when their separate judgment or work would help,
or when the user/project requests them. Derive boundaries from the outcome and
sources. Mirket's `team_facets` and `team_plan` can compose complementary briefs;
choose applicable boundaries with task-specific reasons rather than filling roles
because they share a skill tag.

Dispatch selected independent assignments through Mirket's routed spawner
(`mirket agent spawn --request FILE` or `spawn_subagent`). The coordinator may
own the lead role. Queue assignments within actual concurrency limits;
dependencies determine when an
implementer or reviewer can use another's artifact. With unavailable independent
agents, state that limit and distinguish sequential perspectives from independent
professional contributions. Never invent a worker, credential or review.

Give the specialist its professional profile and relevant skill body, the actual
objective, raw source material, owned paths, allowed effects, dependencies and
observable return requirements. Ask it to apply its own method, consider credible
alternatives and identify evidence that would change its conclusion. For a fresh
review, withhold the author's recommendation until the reviewer has formed an
initial view when the task permits; retain necessary requirements and context.

Each professional returns an artifact or recommendation, source-linked evidence,
tradeoffs, uncertainty, checks, and a handoff to affected peers. It should challenge
unsupported assumptions in its domain and state when it has no supported objection.
Do not manufacture conflict or force agreement. The integrator resolves disputes
against the user's constraints and observed evidence, records consequential dissent,
and verifies the combined result. Reopen affected work when a peer exposes a real
defect. A majority vote, role title or plausible narrative does not prove quality.

Keep the roster, actual routed worker IDs, ownership, dependencies and contribution
paths in the chosen record. When using Mirket task tracking, use one coordinator
for its mutations and revision numbers, record selected guidance delivery and
attach the roster and meaningful contribution evidence before completion.

For deliberately selected machine-checked coverage, save the original team roster
and a version 1 contributions receipt, then use `task_validate` and
`task_evidence` with `contract: "contributions"` (CLI: `--contract contributions`).
Include each original role exactly once, actual worker ID, attempt and matching
result attempt, declared input/evidence file identities, dependencies, validation,
unverified paths and integrator disposition. Keep failed, interrupted or unknown
assignments visible; re-dispatch increments the attempt and requires current
results. Completion requires accepted completed/reused work or an explicit reasoned
waiver. Recorded running status does not establish a live worker. Bind verification
receipts to explicit input, checker and result files with `contract: "verification"`;
completion rechecks those identities and requires a passed outcome. These opt-in
contracts do not authenticate worker IDs, grade testimony or infer undeclared inputs.
Read [task evidence](references/task-evidence.md) for the exact JSON shapes,
status rules, limits and recovery commands before writing a structured receipt.
Mirket validates delivery and artifact consistency; the coordinator observes
execution and assesses each professional's work.

Use Mirket's [model routing](references/model-routing.md) to inspect available
targets and route each assignment before spawning. Verify actual tools,
concurrent slots, filesystem sharing and messaging permissions; do not invent
APIs or assume a model name. Native host subagents are outside this routing
boundary. Parallelism helps independent work;
shared sequential state can make it slower or unsafe.

Select a provider through the routed model registry, task and assigned-tool
floor. Use a fresh independent perspective when it resolves a material
uncertainty, such as a bounded design critique or code review. Do not duplicate
every assignment across providers, infer quality from brand, or promise lower
cost. Honor an explicit user choice using `model_selection` with the exact live
target, model ID and supported effort; this narrows eligibility and prevents
cross-model failover. Host model selection is not inherited automatically.
If no discovered model satisfies the floor and that choice,
continue locally where practical and report the missing capability rather than
changing providers or authentication silently.

For a Claude candidate, read [the native adapter contract](references/claude-code.md).
The primary agent supplies a compact assignment and relevant constraints, checks
the returned evidence, and owns all edits and integration. A subordinate worker
returns its findings and does not start another provider, register skills, or
reinterpret global coordination guidance as permission to delegate again.

## Decompose around deliverables

Map required outcomes and dependencies. Separate independent research/reviews,
disjoint implementation, and integration steps. Keep tightly coupled edits with
one owner; use read-only reviewers where writing would collide.

Reuse the assignment contract above; do not create a second roster or briefing
format. Start independent work within the selected execution boundary and keep
integration and the critical path moving.

Source workers return `awaiting_host_verification` after exact file readback.
The integrator runs the declared checks through authorized host tools and records
hash-bound logs with `verification_handoff`; readback and a worker's "verified"
claim cannot satisfy this handoff. See [model routing](references/model-routing.md)
for the request/result contract. Existing durable task records may reference this
run trace instead of duplicating its verification log.

## Coordinate without losing state

- Use separate worktrees when workers need incompatible branches or isolated builds;
  use disjoint files for a shared checkout. A worktree does not isolate services,
  credentials, databases, or ports; allocate those explicitly when needed.
- Freeze shared interfaces early enough for parallel implementations. A changed
  contract should be communicated before dependent work proceeds.
- Route a material discovery to affected workers with the evidence and implication.
  Interrupt obsolete assignments; do not let two agents solve the same abandoned
  problem merely to keep them busy.
- Wait for concrete routed worker events. Repeated unchanged polls and
  large full-history reads add cost without new information.
- Preserve user steering across all workers. The primary agent owns scope decisions,
  user communication, conflict resolution, and final verification.
- After cancellation or a changed assignment, reconcile late results against the
  current scope and actual files before integration. Interrupting a worker may not
  undo its writes or stop remote work; inspect owned artifacts and operation state
  before reassigning the same destination or retrying an uncertain effect.

## Integrate and verify

Require each result to identify changed files, observed behavior, tests/checks,
assumptions, unresolved risks, and provenance for external claims. A worker's
success message is a claim to assess, not a substitute for inspecting the result.
For a substantial deliverable, connect acceptance conditions to retrievable artifact
paths and executed checks with their relevant inputs. Distinguish work completed,
partial/capped output, failed verification and unexecuted suggestions. Verify a
referenced artifact exists; a finished process or a polished summary is not an
accepted deliverable. Retrieve omitted evidence before drawing conclusions from
truncated output.

Review interfaces and combined behavior after integration; independently sound
parts can conflict. Reuse current evidence for untouched components and rerun
checks affected by integration. Resolve contradictory findings against source and
requirements. A blind vote among agents does not establish correctness.

Cancel unnecessary workers and retain useful artifacts when complete. Report the
integrated outcome and evidence; avoid making the user reconcile separate reports.
External chat/email/task messaging follows the user's authorization and host rules.
