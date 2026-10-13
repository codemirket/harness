# Worked case: an exposure filter inflates the reported lift

Use this fictional case when an experiment result joins assignment, exposure and
outcome tables. The decision concerns purchases per assigned eligible account
within seven days. Random assignment is stable at account level, 50/50; all
accounts have a complete observation window. Purchase records are independently
complete, with no cross-account interference in this example.

| Measure | Control | Treatment |
| --- | ---: | ---: |
| Unique assigned accounts | 1,000 | 1,000 |
| Accounts with retained exposure logs | 800 | 600 |
| Unique purchasing accounts in complete outcome data | 80 | 90 |

Suppose all purchasing accounts appear in the retained exposure logs. A dashboard
that inner-joins exposure rows reports 80/800 = 10% versus 90/600 = 15%, a five
percentage-point or 50% relative increase. The assignment-based calculation is
80/1,000 = 8% versus 90/1,000 = 9%, a one percentage-point or 12.5% relative
increase. These are different populations; the larger filtered estimate is not
evidence of the intended effect.

Inspect why exposure coverage differs. Was a logging event renamed, did a join
discard rows, or did the treatment change the engagement used for filtering?
Use the configured allocation to check assignment and appropriate analysis
populations separately. Balanced original assignment cannot validate an arbitrary
filtered comparison. Do not randomly discard control accounts to make the
retained totals equal.

For a triggered analysis, establish a common condition observable in both arms
before the treatment can affect it. The diagnosis and importance of comparable
trigger logging are described in
[Microsoft's sample-ratio mismatch guidance](https://www.microsoft.com/en-us/research/articles/diagnosing-sample-ratio-mismatch-in-a-b-testing/).
Its operational thresholds belong to that platform; they are not universal defaults.

Rebuild the intended account-level calculation from assignment and complete outcome
data. Validate the join cardinality, seven-day windows and purchase identity before
inferring a nonpurchase from a missing outcome row. Under the stated complete-data
assumptions, an account without a purchase is a nonbuyer. If outcomes are missing,
neither zero imputation nor an assignment-based label repairs that gap.

Then apply the planned uncertainty method and stopping rule to the valid dataset.
The positive point estimate alone does not establish significance, equivalence,
or a rollout decision. Evaluate the primary outcome and material guardrails using
their declared populations and methods.

As a failure probe, duplicate an exposure event for one account and recompute.
Assigned account counts, distinct purchasing accounts and the account-level result
must remain unchanged. Save the query, input identities and reconciliations so the
dashboard claim can be checked independently. No live traffic change is needed
to complete this analysis.
