---
name: ai-evaluation
description: "Compare prompt, model or agent behavior with fixed representative tasks, matched conditions and independently checked outcomes. Skip judging a single response by style alone."
---

# AI evaluation

Define the user outcome and failure costs before choosing metrics. Build representative tasks with source/expected-result evidence, including ambiguity, refusals, tool failures and difficult negatives. Separate development fixtures from held-out acceptance material.

Compare candidates and a meaningful baseline under matched inputs, model/settings, tools and budgets. Preserve exact versions, traces, retries and cost/latency observations. Repeat variable cases enough to expose instability rather than presenting one lucky run as capability. Do not silently change providers or run paid evaluations outside authorization.

Inspect actual answers, saved artifacts and system effects independently of the generating agent. Use deterministic checks where they fit; calibrate any judge against human/source evidence and inspect disagreements. Measure unnecessary calls and failure recovery alongside task success.

Return task coverage, candidate/baseline results, variability, error analysis and limits. Stop when the decision is supported at the agreed confidence or a material evidence gap remains. Test success, a judge score or schema validity alone does not establish user acceptance or safety.
