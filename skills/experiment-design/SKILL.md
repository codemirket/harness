---
name: experiment-design
description: Design and interpret product or marketing experiments with defensible metrics, assignment, stopping rules, and uncertainty. Use for A/B test planning or results review, not ordinary copy drafting.
---

# Experiment design

Translate the decision into a falsifiable change: for which population, exposure,
and time window should which outcome improve, and by how much would it matter?
Use existing measurements to establish the baseline before proposing a sample size.

## Before launch

- Specify assignment and analysis units. Account for repeated users, shared
  accounts, interference, and channels that expose people to both treatments.
- Choose the primary metric, denominator, eligibility, exposure event, and
  attribution window. Add guardrails for material harms such as refunds,
  latency, accessibility, retention, or support burden.
- Set the smallest worthwhile effect and the analysis method before inspecting
  results. Have a qualified method or validated calculator justify sample size,
  power, duration, and uncertainty; document its assumptions.
- Decide the stopping rule, exclusions, missing-data handling, and treatment of
  multiple variants or metrics. Repeated peeking needs a method designed for it.
- Check assignment balance, event integrity, and the unchanged control experience.
  Changing allocation, eligibility, or a metric mid-test changes the interpretation.

## Read the evidence

Check exposure and sample-ratio mismatch before reading an effect estimate.
Report absolute and relative effects with denominators and uncertainty intervals.
Statistical significance is not the probability that a result happened by chance,
and it does not establish that an effect is commercially meaningful.

Distinguish exploratory segments from planned comparisons. A result from one
population, season, or channel does not establish a universal conversion uplift.
Do not turn an inconclusive test into a claim of equivalence; quantify which
plausible effects remain unresolved. Consider implementation and operational cost
alongside effect size when making the decision.

Deliver a concise experiment specification or results assessment, the supported
decision, and remaining uncertainty. Publishing a variant or changing traffic
allocation is a separate external action from planning or analysis.
