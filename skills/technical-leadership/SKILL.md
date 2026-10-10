---
name: technical-leadership
description: Evaluate technology investment and lead changes spanning teams or components. Use for CTO-level build/buy, platform and migration choices, or senior engineering lead work on interfaces, sequencing, delegation and integration. Use architecture-review for a bounded code-structure question.
---

# Technical leadership

Select the level of decision. A CTO question concerns technology's contribution
to business outcomes, capacity and long-term obligations. A senior lead question
concerns making an actual change work across owners and consumers. A title does
not confer authority to spend, hire, change providers or publish a release.

## Technology portfolio and architecture economics

Start from the business outcome, time horizon, binding constraints and current
operating evidence. Inspect existing commitments, failure data, delivery lead
time and maintenance burden before proposing a platform. Distinguish an observed
cost from an engineer's frustration or a vendor's performance claim.

Compare credible options, including a narrow adapter, managed product, staged
migration and maintaining the current approach when applicable. Account for:

- Delivery capacity displaced by discovery, migration, compatibility and support.
- Recurring operating cost, ownership skills, on-call burden and failure recovery.
- Data/control ownership, deployment constraints, integration contracts and exit
  cost; a product's feature list does not establish suitability for this workload.
- Dependency concentration, reversibility and what a bounded experiment can
  resolve before a larger commitment.

Use compatible units and explicit assumptions. Separate one-time costs, recurring
costs and sunk costs; a cheaper invoice can increase engineering effort. Reject
options that violate a hard constraint before applying a weighted comparison.
Do not hide missing evidence inside an invented confidence score. For consequential
choices, identify the assumption or threshold that would reverse the recommendation.

Read [the worked investment and integration decision](references/platform-change.md)
when capacity, migration or multiple implementers determine the outcome. Use
`executive-strategy` for broader business allocation and `systems-engineering`
for reliability/capacity design when those capabilities are available. Preserve
the project's chosen architecture; change it when evidence justifies the cost.

## Lead an integrated change

Trace the real entry point through data owners and downstream consumers. Make
shared contracts concrete: identifiers, ordering, errors, compatibility windows,
state transitions and who owns retries. Record decisions where future consumers
will find them, using the project's existing record rather than a new tracker.

Sequence around dependencies. Build the smallest end-to-end slice that exercises
the risky boundary, then parallelize independent work once its contract is usable.
Keep tightly coupled edits with one owner; separate worktrees do not isolate
ports, databases or external accounts. Delegate when independent progress exceeds
briefing and integration cost. Use `agent-coordination` for actual worker setup,
not a fixed team for every request.

Give each owner an outcome, owned paths, input contract, failure probe and required
evidence. Communicate interface changes before dependents proceed. Reconcile late
results against current source and user steering; a worker's completion statement
is not integration proof. Inherit the selected host/model unless a different
choice is authorized and supported.

## Review the decision and the result

Review requirements and implementation as distinct questions. For a defect, name
the triggering state, affected consumer and consequence; distinguish an actual
failure from a preferred style. Inspect consumers beyond the diff, particularly
dispatch tables, enum handling, schema readers and mixed-version deployments.
Choose branch, staged, unstaged and untracked scope deliberately.

Verify the combined path and the failure that motivated the work. Bind results
to relevant source, configuration and environment; reuse unchanged evidence and
rerun checks affected by integration. Report the supported decision, important
tradeoff, operating owner, completion evidence and remaining uncertainty. An
approved plan, working implementation and deployed outcome remain separate states.
