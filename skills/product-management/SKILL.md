---
name: product-management
description: Turn product evidence, feature requests and roadmap choices into explicit outcomes, scope, delivery slices and measurable acceptance. Use for product prioritization, requirements or launch decisions; skip routine implementation with settled requirements.
---

# Product management

Start from the decision the user needs to make and the product's current state.
Use existing issues, product notes and analytics; do not require a new PRD, scorecard
or interview cycle when the evidence already resolves the task. An implementation
request with settled scope does not need to reopen product strategy.

For competing requests, uncertain demand or a feature crossing multiple owners,
read [decision to delivery](references/decision-to-delivery.md). It includes a
worked example of turning a vocal customer's request into a bounded release.

## Establish the problem and authority

- Name the affected user, situation, existing workaround and outcome that should
  improve. Separate a requested solution from the need it may serve.
- Link material evidence to its origin and date. Distinguish observed behavior,
  reported pain, contractual obligations and hypotheses. Repository features or a
  pricing page do not establish usage, revenue or product-market fit.
- Identify who may decide policy, pricing, access and other consequential tradeoffs.
  Keep unresolved authority visible; an agent's recommendation is not a commitment
  by the business. Reuse decisions already made by the user.
- Check which segments are represented and missing. A few repeated requests may
  represent one large account; ticket counts are not a measure of customer demand.
  Use customer research when new evidence is needed, within the authorized scope.

## Choose an outcome and a scope

Describe the observable improvement, relevant population and time horizon. Use an
existing baseline when available; label a proposed target as proposed. Tie output
metrics such as shipped features or clicks to the user outcome they approximate.
Identify material guardrails such as errors, support burden, access, retention or
operating cost. Do not invent numerical targets to make a plan look complete.

Compare viable options, including a smaller change or retaining current behavior
when appropriate. Explain the decisive tradeoff: impact, confidence, effort,
dependencies, risk or reversibility. Use ranges and sensitivity when estimates are
weak. A prioritization formula helps organize assumptions; its score does not turn
guesses into evidence or override a contractual or safety constraint.

State what the chosen slice includes, what it defers and which assumption could
change the choice. Avoid expanding scope just because another capability is easy
to imagine. Resolve blocking decisions before implementing dependent behavior;
continue independent work when useful.

## Make delivery testable

Translate the decision into the smallest coherent user journey that delivers or
tests the intended value. A slice should include its consequential states and
recovery path, not just a screen or isolated service. Use the project's existing
planning format and naming.

For each material requirement, connect:

- **Context and actor:** who can do what, from which starting state.
- **Observable result:** what changes and what must remain true.
- **Failure and recovery:** invalid input, permission failure, interruption,
  duplication or concurrency where they affect this journey.
- **Acceptance evidence:** a behavior, data invariant or rendered state a reviewer
  can inspect independently of the implementation's own success message.

Expose data ownership, policy, interface and lifecycle implications that engineering
must preserve. Let engineers choose implementation details unless the product
contract constrains them. Use architecture or specialist review for a concrete
boundary problem; do not mandate a particular technology or design pattern.

## Close the feedback loop

Define how the team will learn whether the outcome happened: event meaning,
eligibility, denominator, observation window and important exclusions. Check that
measurement can distinguish retries, repeated users and failed attempts. Plan
necessary instrumentation with the slice; avoid unnecessary personal data.

Connect rollout size and recovery to the consequence of being wrong. Record who
owns follow-up and the evidence that would justify expanding, revising or stopping.
Use experiment design for causal comparisons and data analysis for observational
results. A before/after improvement alone does not prove the feature caused it.

Deliver the decision, supporting evidence, scope, acceptance and unresolved items
at the depth the task needs. Distinguish proposed, implemented, behaviorally
verified and outcome-measured work. Preserve consequential decisions in the
project's established location so later agents can continue without inventing them.
