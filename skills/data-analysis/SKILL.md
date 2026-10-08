---
name: data-analysis
description: Analyze datasets, metrics and business questions with explicit grain, denominators, reconciled transformations and reproducible calculations. Use for quantitative investigation, cohort or funnel analysis, and interpreting analytical results; skip database administration and simple arithmetic.
---

# Data analysis

Frame the decision and the quantity needed to inform it. Use the available SQL,
Python, spreadsheet or other project toolchain; a notebook or new dependency is
not required. Keep source data intact and retain enough executable calculation or
query detail for another person to reproduce consequential results.

For joined business data, ratios or before/after comparisons, read
[grain and comparison traps](references/grain-and-comparisons.md). Its worked
examples show why plausible totals and charts can support the wrong conclusion.

## Define the measurement before calculating it

- Name the population, entity, row grain, time window, timezone, currency and units.
  Distinguish event time from ingestion time and a snapshot from an event history.
- Specify eligibility, numerator, denominator and treatment of repeated entities.
  State whether a funnel measures users, sessions, attempts, accounts or events.
  Establish order and conversion window for multi-step journeys.
- Locate authoritative fields and business definitions. Paid, booked, invoiced,
  refunded and recognized amounts may represent different states and dates.
  An undocumented column name is not a sufficient metric definition.
- Record input source/version or extraction time, filters and coverage limits.
  Check pagination and access scope. Distinguish a complete empty result from an
  unavailable, truncated or permission-filtered source.

Use a provisional definition only when its effect is explicit. Resolve ambiguity
that could reverse the decision instead of quietly choosing the convenient one.

## Inspect and reconcile the data

Check schema and types, unique keys, row counts, missingness, duplicates, invalid
ranges and important distributions before analysis. Distinguish legitimate
repeated observations from duplicated ingestion. Define duplicate identity and
conflict handling before dropping rows; keep conflicting records visible.

For each consequential transformation, reconcile row/entity counts and relevant
totals against the preceding stage or an independent control. Establish join
cardinality; inspect unmatched keys and fanout. Aggregate each source to a common
grain or use an appropriate existence join before combining measures. A distinct
entity count does not fix duplicated monetary sums.

Treat missing, zero, unknown, not applicable and not yet observed as distinct
states. Explain imputation or exclusions and their effect on coverage. Do not turn
pending amounts into zero or remove inconvenient outliers without a defensible
rule. Preserve exact monetary units, avoid adding unlike currencies and document
exchange-rate source/date if conversion is required.

Use bounded read queries and the engine's existing cost controls for substantial
data access. Avoid exposing personal data in shared samples or charts. Data
analysis does not require changing source records or production configuration.

## Compare like with like

Choose a summary that represents the question: ratio of summed counts for a
pooled rate, explicit weights for standardized comparisons, and distributions or
quantiles when an average hides important tails. State the denominator of every
percentage and whether a change is relative percent or percentage points.

Check cohort maturity, population mix, selection, seasonality, repeated users and
instrumentation changes before attributing a difference to product behavior.
Separate a descriptive association from a causal estimate. Use an experimental or
other defensible identification strategy for causal claims; explain its assumptions.

Distinguish sampling uncertainty, measurement error and uncertainty in definitions.
Choose intervals or resampling methods that match the sampling/assignment unit and
dependence structure. Do not treat events from the same account as independent
people. Label exploratory comparisons, inspect meaningful segments, and account
for multiple comparisons when drawing confirmatory conclusions.

## Produce an auditable decision

Run calculations in the chosen tool and independently check a small control,
boundary case or aggregate. Rerun after changing inputs or definitions. Make chart
units, axes, date ranges, missing values and denominators legible; a polished chart
does not validate its dataset. Use the relevant document or spreadsheet workflow
when the deliverable needs that format.

Lead with the supported finding and implication. Include magnitude, population,
coverage, method and material uncertainty. Separate observations, assumptions and
recommendations. Explain which plausible alternative definition or missing group
could change the answer and what evidence would resolve it. Save query/calculation
and a compact input provenance note when the analysis needs to be reused.
