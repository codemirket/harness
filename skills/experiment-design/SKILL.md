---
name: experiment-design
description: Design and interpret product or marketing experiments with defensible metrics, assignment, stopping rules, and uncertainty. Use for A/B test planning or results review, not ordinary copy drafting.
---

# Experiment design

## Professional stance and collaboration

Own the validity of a claim that a change caused an outcome. Ask what was
assigned, what was actually observed, and whether the same comparison population
survived into the analysis. Set a practical decision before reading results and
choose a method that matches the assignment and stopping design. Change advice
when integrity checks fail or plausible effects remain too uncertain for the
cost of the proposed decision.

Challenge growth enthusiasm driven by a significance badge, repeated peeking or
selected exposure. Work with analysts on event integrity and product owners on the
smallest worthwhile effect and guardrails. Hand the decision owner a reproducible
comparison, integrity findings, uncertainty and practical tradeoff; preserve a
useful inconclusive result instead of manufacturing a winner.


Translate the decision into a falsifiable change: for which population, exposure,
and time window should which outcome improve, and by how much would it matter?
Use existing measurements to establish the baseline before proposing a sample size.

Use the project's experiment platform or available SQL and statistical runtime.
Inspect its actual method, allocation history and exported unit-level data; a
dashboard's significance badge does not establish a valid comparison. Reproduce
important counts and estimates from a saved query or calculation. Record method,
version and random seed when they affect reproducibility. A calculator for two
independent proportions does not automatically support clustered users, ratio
metrics or sequential monitoring; choose a supported method for the design.

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

Check assignment, exposure and sample-ratio mismatch before interpreting an effect.
Compare allocation with the configured ratio for the applicable period, including
ramps. Diagnose unexplained missing units, crossover and filtering before calling
a causal win; equalizing group sizes after the fact does not repair selection.
Keep the planned assignment-based population separate from exploratory engagement
segments. A triggered analysis needs a comparable eligibility condition in both
arms that the treatment does not change.

For analysis with filtered exposure logs, read the
[assignment and denominator case](references/assignment-integrity.md).
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
