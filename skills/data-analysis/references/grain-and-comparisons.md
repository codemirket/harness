# Grain and comparison traps

Read when measures cross tables, rates combine populations, or a comparison might
drive a business decision. These synthetic examples are reasoning checks, not
claims about the project's data. Adapt the definitions before reusing the SQL.

## A join can multiply money while preserving the correct order count

Suppose `orders` has one row per order:

| order_id | status | currency | gross |
| --- | --- | --- | ---: |
| A | complete | EUR | 100 |
| B | complete | EUR | 80 |
| C | cancelled | EUR | 50 |
| D | pending | EUR | NULL |

Order A has two item rows, worth 60 and 40, and two refund rows, worth 10 and 5.
Order B has one item worth 80 and no refund. The requested measure is completed
order gross less recorded refunds, grouped by currency. Assume here that refunds
are in the order currency and the extraction contains the relevant refund window.

A direct join from orders to both items and refunds creates four rows for A:
each item is paired with each refund. Summing order gross gives 400 for A; summing
refunds gives 30. With B included, the false net is 480 - 30 = 450. Counting
distinct orders still returns the correct two, which can hide the monetary error.
`SUM(DISTINCT gross)` is also wrong in general: two different orders may have the
same amount. Deduplication must follow identity, not coincidentally equal values.

The intended gross is 180, refunds 15 and net 165. Aggregate refunds to one row
per order before joining. Items are not needed to answer this question. If item
eligibility matters, use an existence predicate or a separately aggregated source
whose grain and allocation rules match the question.

```sql
WITH refund_by_order AS (
  SELECT order_id, SUM(amount) AS refund_amount
  FROM refunds
  GROUP BY order_id
)
SELECT o.currency,
       COUNT(*) AS completed_orders,
       SUM(o.gross) AS gross,
       SUM(COALESCE(r.refund_amount, 0)) AS refunds,
       SUM(o.gross - COALESCE(r.refund_amount, 0)) AS net
FROM orders AS o
LEFT JOIN refund_by_order AS r ON r.order_id = o.order_id
WHERE o.status = 'complete'
GROUP BY o.currency;
```

The zero here means no matching refund exists in a checked, complete source. It
does not mean every missing amount is zero. Before accepting the result, verify:

- `orders.order_id` and refund event identity are unique at their intended grains.
  If a refund is redelivered, resolve its identity and conflicting values before
  aggregation. Two distinct partial refunds must both survive.
- Completed orders have non-null gross; refunds have non-null amounts. `SUM`
  silently skips null values in many SQL engines, so a plausible total can be
  incomplete. Report pending D separately; cancelled C is outside this definition.
- Refunds have matching orders and valid currency semantics. Preserve unmatched
  records as reconciliation exceptions; a left join can otherwise hide them.
- Eligible order count and gross remain unchanged through the join. Compare
  refunds against an independently grouped control with the same scope.
- Refund cutoff and status are correct. An order completed today may be refunded
  later; an extraction-time net and a settled cohort net answer different questions.

For exact values, use integer minor units or the database's suitable decimal
type. The displayed amounts above are simplified whole currency units.

## A falling overall rate can conceal improvement in both segments

Consider observed conversions, with unchanged definitions within each segment:

| Period | Segment | Converted | Eligible | Rate |
| --- | --- | ---: | ---: | ---: |
| Before | High intent | 80 | 100 | 80% |
| Before | Low intent | 20 | 100 | 20% |
| After | High intent | 9 | 10 | 90% |
| After | Low intent | 27 | 90 | 30% |

Before: 100/200 = 50%. After: 36/100 = 36%. The overall rate falls by 14 percentage
points, or 28% relative to the original rate. Both segment rates rise by 10
percentage points. The share of high-intent observations changed from 50% to 10%.

With the original 50/50 segment weights, the after rate is
0.5 × 90% + 0.5 × 30% = 60%. This standardized comparison describes the observed
segment rates under that chosen mix. It is not the actual after-period rate and
does not prove the product caused a 10-point improvement. Report the actual rate
and mix change alongside it; choose weights to answer the decision's population
question, not to obtain a preferred conclusion.

The small high-intent after group makes its rate uncertain. Investigate assignment,
selection, seasonality and instrumentation before proposing a causal explanation.
If these are different users or periods, an A/B-test interpretation is unsupported.

## Other comparison boundaries worth checking

- **Cohort maturity:** compare retention at the same age since joining. A recent
  cohort has not yet had the chance to exhibit long-term retention or refunds.
- **Weighted rates:** averaging regional percentages equally estimates a different
  quantity from pooling their numerators and denominators. State the weights.
- **Repeated units:** bootstrap or model at the relevant independent unit when
  dependence matters. Thousands of clicks from a few accounts are not thousands
  of independent customer observations.
- **Missing outcomes:** show the analyzed share. Test plausible missing-outcome
  scenarios when they could change the recommendation; an interval conditional
  on observed rows does not account for selection bias by itself.
- **Funnel semantics:** establish event order, identity stitching and the conversion
  window. Counting anyone who ever emitted both events can imply a journey that
  never occurred in order.

Keep the calculation or query, source/extraction identity, scope and reconciliation
controls next to the finding. A reader should be able to reproduce both the headline
number and the boundary that limits its interpretation.
