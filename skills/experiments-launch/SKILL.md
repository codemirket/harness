---
name: experiments-launch
description: "Design or execute an authorized growth experiment or launch with hypothesis, exposure, measurement and rollback criteria. Skip unsupported launch claims and unmeasured promotion."
---

# Experiments and launch

Define the decision, hypothesis, eligible audience, assignment/exposure unit and meaningful outcome. Establish a baseline, guardrails and measurement validity before interpreting a change. Account for dependence, contamination, seasonality and multiple looks/comparisons where relevant.

Choose a bounded experiment or staged launch that can discriminate the belief. Specify rollout conditions, ownership, stopping/review criteria and recovery before exposure. Separate operational health from evidence of product impact. Avoid promising statistical certainty from a small sample or one favorable metric.

Prepare and verify the artifact, instrumentation and rollback path in the authorized environment. Launch, spending, external messages and production changes must stay within explicit authorization. Inspect an uncertain exposure/effect before replaying it.

Return the hypothesis, plan or executed exposure, measurement evidence and decision limits. Stop when the predeclared gate is reached or a guardrail fails; preserve null/negative outcomes. Do not silently extend exposure, change success criteria after results or claim an unrun launch succeeded.

For a repeated review or launch plan, load `references/loop-and-experiment-contract.md` and keep a project-local `product/experiment-contract.json`. Specify cadence, trigger, input digest, idempotency key, intended action, approval state, stop rule and rollback owner. Repeating the same input must be a no-op or staged result, never a duplicate external action; a new specimen or delegate does not reset the review budget.
