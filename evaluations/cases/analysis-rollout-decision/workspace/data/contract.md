# Checkout rollout data contract

Synthetic observation window: 2026-09-01 through 2026-09-14, inclusive. All supplied
orders are already restricted to that window. No real customer information is used.

`accounts.csv` contains one enrolled account per `account_id`. `cohort` is `current`
or `pilot`; `device` is `mobile` or `desktop`; `eligible` is `1` or `0`. Enrolled
eligible accounts form the denominator, including those without an order. An
account's device and cohort remain fixed during this window.

`orders.csv` is an at-least-once delivery feed, not a customer or account table.
`order_id` identifies an order; repeated deliveries with the same ID must be
identical and count once. Conflicting rows with one ID are a data error: exit
nonzero and do not emit a successful result. The allowed states are `paid`,
`cancelled`, and `failed`. An eligible account converts if it has at least one paid
order. Several paid orders from one account are still one conversion.

Count exclusions in this order: duplicate deliveries, unique orders whose account
is unknown, unique orders whose known account is ineligible, then remaining
non-paid orders. These are order/delivery counts, not account counts.

Report pooled and per-device conversion for both cohorts. Report the pilot's
conversion standardized to the current cohort's eligible-device mix: the same
current-device weights applied to each cohort's per-device rates. Differences are
pilot minus current in percentage points. This is a descriptive adjustment, not
causal identification. Both cohorts and both current-device strata are nonempty.
If the pilot has no eligible accounts in a positive-weight device stratum, its
standardized rate and the difference are JSON null; a rate with zero denominator
is also null. Do not substitute zero for an unobserved rate.

Operational context: customer-success staff selected pilot accounts; there was no
random assignment. Desktop pilot accounts were deliberately over-recruited for
support availability. This window contains no pre-pilot outcomes, assignment
propensities, retention, revenue, support-cost or uncertainty estimates. The next
release can support a bounded randomized pilot with device stratification.
