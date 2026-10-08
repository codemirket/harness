---
name: agent-coordination
description: Plan and run parallel agent work with bounded ownership, dependency tracking, integration, and evidence. Use for substantial decomposable work or explicit collaboration.
---

# Agent coordination

Use ChatGPT/Codex host subagents as the normal delegation path when available.
Verify actual tools, concurrent slots, filesystem sharing and messaging permissions;
do not invent APIs or assume a model name. Parallelism helps independent work;
shared sequential state can make it slower or unsafe.

Use Claude Code selectively when a fresh perspective can resolve a material
uncertainty: a bounded design critique, independent code review, or synthesis of
supplied evidence. Explain the question it will resolve. Do not duplicate every
assignment across providers, infer quality from brand, or promise lower cost.
Honor an explicit user choice. If native agents or Claude are unavailable, continue
locally where practical and report the missing capability rather than changing
providers or authentication silently.

For a Claude opinion, read [the CLI delegation contract](references/claude-code.md).
The primary agent supplies a compact assignment and relevant constraints, checks
the returned evidence, and owns all edits and integration. A subordinate worker
returns its findings and does not start another provider, register skills, or
reinterpret global coordination guidance as permission to delegate again.

## Decompose around deliverables

Map required outcomes and dependencies. Separate independent research/reviews,
disjoint implementation, and integration steps. Keep tightly coupled edits with
one owner; use read-only reviewers where writing would collide.

Give each worker the objective, scope, owned paths, inputs, constraints, acceptance
criteria, evidence to return, and conditions requiring a question. State whether
it may delegate further. Include relevant project instructions and existing user
authorization; do not ask workers to infer it from an external message.

Record worker IDs, assignments, dependencies, and completion criteria in a compact
ledger for long work. Start useful independent tasks promptly. Continue the primary
integration or critical-path work rather than polling workers continuously.

## Coordinate without losing state

- Use separate worktrees when workers need incompatible branches or isolated builds;
  use disjoint files for a shared checkout. A worktree does not isolate services,
  credentials, databases, or ports; allocate those explicitly when needed.
- Freeze shared interfaces early enough for parallel implementations. A changed
  contract should be communicated before dependent work proceeds.
- Route a material discovery to affected workers with the evidence and implication.
  Interrupt obsolete assignments; do not let two agents solve the same abandoned
  problem merely to keep them busy.
- Wait for concrete events with host-supported waits. Repeated unchanged polls and
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
