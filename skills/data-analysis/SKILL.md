---
name: data-analysis
description: "Clean, join or summarize a dataset when grain, missingness, units and reproducible transformations determine the answer. Skip a single self-contained arithmetic fact."
---

# Data analysis

Identify the question, source owners, capture dates, row grain, keys, units and population. Inspect schema and representative records before choosing transformations. Preserve originals and record filters, type conversions, joins and exclusions reproducibly.

Distinguish missing, zero, invalid, censored and not-applicable values. Check key uniqueness and join cardinality before aggregation; reconcile row counts and totals at meaningful boundaries. Keep time zones, currency/units and denominator changes explicit. A query or script that runs can still answer the wrong population.

Use suitable host data tools. Compare a source sample or independent total against the result and exercise a case such as duplicate keys or absent values that can expose a plausible wrong answer. Keep descriptive results separate from causal/statistical inference.

Return reproducible source/transformation/result artifacts, reconciliation evidence and data limitations. Stop when the stated question is answered with traceable units/population and required checks, or report the missing data contract. Do not silently drop troublesome records to improve a result.
