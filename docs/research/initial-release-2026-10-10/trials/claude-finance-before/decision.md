# Can the business fund its obligations? (2026-01 to 2026-03)

## Decision

**No, not from its own cash as the ledger stands.** The business makes a profit
of €20,000 every month. The €70,000 principal repayment is due in 2026-02, and
customers pay about a month after they are invoiced. Together these push cash
below zero in February.

- **Base (stated collection dates):** cash bottoms out at **−€20,000** at the
  end of 2026-02. Keeping the €10,000 minimum reserve needs **€30,000** of
  extra cash by that month-end.
- **Delayed (every receipt one month later):** cash bottoms out at
  **−€70,000** at the end of 2026-02 and is still −€50,000 at the end of
  2026-03. Keeping the reserve needs **€80,000**.

No funding, credit facility, principal deferral or faster collection is in the
ledger or treated as approved. These amounts are a gap to close. They are not
a plan.

**Recommended next action:** before paying the February principal, confirm the
obligations, then get a written answer from the lender on deferral and
committed bridge-funding terms. Plan for €80,000 and treat €30,000 as the
minimum. Details are in [Recommended next action](#recommended-next-action).

## Key figures

Source: `results.json`, produced by
`python3 analyze.py --input ledger.json --output results.json` from `ledger.json`
(SHA-256 `5be1e094f76fde050cd4c8a0c0648f2fe296009b04bfc10720c04e276a967eeb`).
Amounts are shown in EUR. The JSON stores integer cents.

| Period | Revenue | Operating cost | Profit | Payments | Receipts (base) | Closing cash (base) | Receipts (delayed) | Closing cash (delayed) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Opening | | | | | | 60,000 | | 60,000 |
| 2026-01 | 50,000 | 30,000 | 20,000 | 30,000 | 0 | 30,000 | 0 | 30,000 |
| 2026-02 | 50,000 | 30,000 | 20,000 | 100,000 | 50,000 | **−20,000** | 0 | **−70,000** |
| 2026-03 | 50,000 | 30,000 | 20,000 | 30,000 | 50,000 | 0 | 50,000 | −50,000 |
| **Total** | 150,000 | 90,000 | **60,000** | 160,000 | 100,000 | | 50,000 | |

Revenue, cost, profit and payments are the same in both scenarios. February's
payments are €30,000 of operating cost plus the €70,000 principal.

| Metric | Base | Delayed |
|---|---:|---:|
| First negative month-end | 2026-02 | 2026-02 |
| Minimum cash (opening and month-ends) | −€20,000 | −€70,000 |
| Financing to reserve = max(0, €10,000 − minimum) | **€30,000** | **€80,000** |

## Profitable but short of cash

The ledger records revenue when the work is invoiced and cost when it is
incurred. Cash moves later: each invoice is paid about one month after it is
issued. Principal is a cash payment but not an operating expense, so it does
not reduce profit. Over the three months:

| | Base | Delayed |
|---|---:|---:|
| Profit | +60,000 | +60,000 |
| Less: invoiced but not yet collected | −50,000 (March invoice, due April) | −100,000 (February and March invoices) |
| Less: principal repayment | −70,000 | −70,000 |
| **Change in cash** | **−60,000** (60,000 → 0) | **−110,000** (60,000 → −50,000) |

Profit shows the business covers its operating costs. It does not show that
cash on hand will cover a €70,000 lump-sum repayment that falls before the
invoiced revenue is collected. The February shortfall comes from timing and
debt. The underlying operations are not losing money.

## Downside

If every customer pays one month later than stated, cash goes negative in
February and stays negative through March. In that case the February invoice
(due April) and the March invoice (due May) are outside the horizon. The
trough is €50,000 deeper than in the base case. Even the base case ends March
with €0, which is €10,000 below the reserve.

## Basis and data handling

- `invoice-jan` appears twice in the ledger with identical content. It is
  counted once, as the contract requires. This matters: counting it twice would
  show base cash at **+€30,000** in February and no financing need, which
  hides the real shortfall. A record that reused an ID with different content
  would stop the program with a nonzero exit and no output.
- `principal` (`financing_payment`, €70,000, cash 2026-02) is counted in
  payments and excluded from operating cost.
- Cash dated outside 2026-01 to 2026-03 is excluded. The €50,000 March invoice
  (cash 2026-04) is therefore not collected inside the horizon, even in the
  base case.
- The delayed case moves only invoice cash dates. Cost and principal dates,
  and the timing of revenue and cost in profit, are unchanged.
- The ledger's periods are synthetic, so timing advice here refers to those
  periods, not to today's date.

## What changes the decision

Each item below was tested by running `analyze.py` on a modified copy of the
ledger. None of these changes is assumed to have happened.

| Change | Base minimum / need | Delayed minimum / need |
|---|---:|---:|
| As stated | −€20,000 / €30,000 | −€70,000 / €80,000 |
| Lender defers all €70,000 principal past March | €30,000 / €0 | €0 / €10,000 |
| Lender defers €30,000 (only €40,000 paid in Feb) | €10,000 / €0 | −€40,000 / €50,000 |

- **Better decision:** a written principal deferral, or committed funding of at
  least €30,000 available before the February payment (€80,000 to cover a
  one-month collection slip). Evidence that customers reliably pay on the
  stated dates would also make the base case the planning case. Deferring the
  full principal is enough in the base case but still leaves the delayed case
  €10,000 short of the reserve.
- **Worse decision:** any obligation missing from the ledger, such as interest,
  taxes or VAT, payroll timing, capex or other debt. Bad debt, slower
  collection, or payments falling earlier in a month than receipts would also
  make things worse (see the next section).

## What this monthly horizon cannot establish

- **Cash within a month.** Month-end balances net receipts against payments.
  If February's €100,000 of payments go out before the €50,000 receipt
  arrives, base cash falls to −€70,000 during the month, not −€20,000. Keeping
  the reserve at every moment would then need up to €80,000 in the base case
  and up to €110,000 in the delayed case (March: −€70,000 − €30,000 before
  receipts). These are worst-case limits, worked out by hand from the monthly
  receipts and payments, not forecasts. Only a weekly or daily schedule can
  settle this.
- **After March.** The €50,000 March invoice is collected in April (May if
  delayed). The ledger does not show whether further principal or other
  obligations fall due then, or whether cash recovers.
- **Completeness and quality.** The ledger has three identical months, no
  interest, no tax, no collection risk and no other creditors. It cannot show
  seasonality, customer concentration or the risk that customers do not pay.
- **Funding feasibility.** The ledger cannot show whether a lender or investor
  will provide the money, on what terms, or whether a covenant constrains the
  reserve. €10,000 is taken as given.

## Recommended next action

Close this decision before the February principal is paid, not by assuming
funding:

1. **Confirm the obligations.** Check the principal amount and due date
   against the loan agreement. Confirm the ledger has no missing outflows
   such as interest, taxes and payroll timing.
2. **Ask for options in parallel.** Request in writing (a) a deferral or split
   of the principal and (b) committed bridge funding terms. Size it at €80,000
   to cover a one-month collection slip, and treat €30,000 as the minimum for
   the stated dates. Accepting either option remains the owner's decision.
3. **Replace the monthly netting with a weekly cash schedule for February and
   March.** Rerun `analyze.py` on the corrected ledger.

**Decision rule:** if no written deferral or committed funding of at least
€30,000 is in place before the principal date, the business cannot meet that
payment and keep its reserve. Raise this with the lender before the due date,
not after.

## Verification

- `results.json` matches a fresh run of `analyze.py` and the figures worked out
  by hand above, including the profit-to-cash reconciliation for both
  scenarios.
- Repeated identical records count once. Removing the duplicate gives the same
  output.
- A conflicting duplicate (different amount or date under the same ID) exits
  with status 1. It creates no output file and leaves an existing output file
  unchanged.
- December-to-January rollover, exclusion of cash outside the horizon,
  principal never counting as a cost, malformed input, and `--output` equal to
  `--input` were also checked. `ledger.json` and `TASK.md` are unchanged.
