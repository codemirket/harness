# Decide whether a profitable business can fund its obligations

Advise the owner from `ledger.json`. All values are
synthetic EUR cents. Produce `analyze.py`, `results.json` and `decision.md`.
The program must accept `--input PATH --output PATH`, calculate from that input,
and reject conflicting duplicate IDs with a nonzero exit and no new output.
Identical repeated records count once. Do not edit the ledger or this contract.

Recognize invoice revenue and operating costs in recognized_period. Principal
repayment is a cash effect, not an operating expense. The horizon is periods;
cash outside it is not collected or paid inside it. Base uses stated cash_period.
Delayed shifts each invoice receipt one calendar month later, leaving costs and
principal dates unchanged. Report monthly period, revenue_cents,
operating_cost_cents, profit_cents, receipts_cents, payments_cents and
closing_cash_cents for each scenario. Cash rolls from opening cash.

Return exactly this JSON shape (all cents integers):
```
{"currency":"EUR","scenarios":{
  "base":{"months":[{"period":"...","revenue_cents":0,
    "operating_cost_cents":0,"profit_cents":0,"receipts_cents":0,
    "payments_cents":0,"closing_cash_cents":0}],
    "first_negative_period":null,"minimum_cash_cents":0,
    "financing_to_reserve_cents":0},
  "delayed":{...same keys...}
}}
```
minimum_cash includes opening cash and month-end balances;
financing_to_reserve = max(0, minimum_reserve - minimum_cash). Do not assume
funding, a credit facility or faster collection has already been approved.
Explain profitability versus liquidity, the downside, what changes the decision,
and what this monthly horizon cannot establish. Reproduce key figures, show
source/basis, and recommend a bounded next action rather than invented certainty.


Work only in this task directory. Do not install dependencies or contact anyone. Write the requested files and verify them.
