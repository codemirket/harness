---
name: agent-coordination
description: Plan and run parallel agent work with bounded ownership, dependency tracking, integration, and evidence. Use for substantial decomposable work or explicit collaboration.
---

# Agent coordination

Use the current host's actual delegation tools. Agent availability, concurrent
slots, filesystem sharing, and messaging permissions are capabilities to verify,
not APIs to invent. Parallelism helps independent work; shared sequential state
can make it slower or unsafe.

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

## Integrate and verify

Require each result to identify changed files, observed behavior, tests/checks,
assumptions, unresolved risks, and provenance for external claims. A worker's
success message is a claim to assess, not a substitute for inspecting the result.

Review interfaces and combined behavior after integration; independently sound
parts can conflict. Reuse current evidence for untouched components and rerun
checks affected by integration. Resolve contradictory findings against source and
requirements. A blind vote among agents does not establish correctness.

Cancel unnecessary workers and retain useful artifacts when complete. Report the
integrated outcome and evidence; avoid making the user reconcile separate reports.
External chat/email/task messaging follows the user's authorization and host rules.
