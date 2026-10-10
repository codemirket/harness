# Can the business fund its obligations from January to March 2026?

**Decision: no, not as scheduled.** The business is profitable, earning €20,000
a month and €60,000 over the quarter. But it cannot pay the €70,000 loan
principal due in February from its own cash and still keep the €10,000 reserve.
In the base case, cash falls to **−€20,000 at the end of February**. Holding
the reserve at every month-end needs **€30,000** of extra cash or deferred
payments. If customers pay one month later, the shortfall becomes **−€70,000**
and the need is **€80,000**. This analysis assumes no loan, credit facility or
faster collection, because none has been approved.

All amounts are in EUR. `results.json` holds the exact values in cents.

## Key figures

| Measure | Base | Delayed receipts (+1 month) |
| --- | ---: | ---: |
| Revenue, Jan–Mar | €150,000 | €150,000 |
| Operating costs, Jan–Mar | €90,000 | €90,000 |
| Profit, Jan–Mar | €60,000 | €60,000 |
| Receipts, Jan–Mar | €100,000 | €50,000 |
| Payments, Jan–Mar (operating costs + principal) | €160,000 | €160,000 |
| Opening cash, 1 Jan | €60,000 | €60,000 |
| Closing cash, end of March | €0 | −€50,000 |
| First month with negative cash | 2026-02 | 2026-02 |
| Minimum cash (opening and month-ends) | −€20,000 | −€70,000 |
| Funding needed to hold the €10,000 reserve | **€30,000** | **€80,000** |
| Funding needed just to stay at or above €0 | €20,000 | €70,000 |

### Monthly detail

| Period | Revenue | Operating costs | Profit | Receipts | Payments | Base closing cash | Delayed receipts | Delayed closing cash |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2026-01 | 50,000 | 30,000 | 20,000 | 0 | 30,000 | 30,000 | 0 | 30,000 |
| 2026-02 | 50,000 | 30,000 | 20,000 | 50,000 | 100,000 | **−20,000** | 0 | **−70,000** |
| 2026-03 | 50,000 | 30,000 | 20,000 | 50,000 | 30,000 | 0 | 50,000 | −50,000 |

Revenue, costs, profit and payments are the same in both scenarios. Only the
timing of receipts changes. February payments are €30,000 of operating costs
plus €70,000 of principal. Cash ends below the €10,000 reserve in February and
March in both scenarios. A negative balance shows how much funding is missing.
It is not a balance the bank would actually allow.

## Why a profitable business runs out of cash

Profit is earned when invoices and costs are recognized. Cash moves only when
customers pay and the business pays out. Three things cause the gap:

| Bridge, Jan–Mar | Base | Delayed |
| --- | ---: | ---: |
| Profit | 60,000 | 60,000 |
| Less: invoiced but not yet collected by 31 March | −50,000 (March invoice) | −100,000 (February and March invoices) |
| Less: loan principal (a cash payment, not an operating expense) | −70,000 | −70,000 |
| **Change in cash** | **−60,000** | **−110,000** |
| Opening cash → closing cash | 60,000 → 0 | 60,000 → −50,000 |

1. **Collection lag.** Customers pay one month after invoicing, so January
   brings in no cash at all while costs are paid as they arise.
2. **Principal timing.** The €70,000 repayment does not reduce profit, but it
   is the single largest payment in the quarter and falls in the same month as
   the first collection.
3. **Thin opening position.** The €60,000 of opening cash covers January costs
   and leaves €30,000 to meet €100,000 of February payments.

From operations alone, cash in (€100,000) exceeds operating cash out (€90,000)
by €10,000 in the base case. The problem is when the debt falls due and when
customers pay, not losses.

## Downside

- **A one-month delay in customer payments** (the delayed scenario) moves no
  cash into February. The shortfall deepens to −€70,000 and is still −€50,000
  at the end of March. Recovery depends on receipts in April (€50,000) and May
  (€50,000), which fall outside this horizon and so cannot be confirmed here.
- **Timing within a month can be worse than the month-end figures.** In the
  base case, suppose the principal and February costs are paid before the
  January invoice is collected. The low point during February is then
  €30,000 − €100,000 = **−€70,000**, so the reserve would need **€80,000**, the
  same as the delayed month-end figure. In the delayed case, suppose March costs
  are paid before the March receipt. The low point is then −€100,000, so the
  reserve would need up to €110,000. These figures are arithmetic on the ledger
  amounts under adverse timing within the month. The real payment dates are not
  in the ledger.
- **If a customer does not pay at all**, the shortfall is larger. Neither
  scenario models bad debt.

## What would change the decision

| Change | Effect on the result |
| --- | --- |
| At least €30,000 more cash, or €30,000 less paid out, by the end of February | Base month-end reserve is held (€20,000 avoids going negative). |
| At least €80,000 by the end of February | The reserve also holds under the one-month delay, and under adverse timing within February in the base case. |
| The lender defers the whole €70,000 beyond March (calculated by rerunning the model without the principal) | Base: month-end cash is €30,000 / €50,000 / €70,000, so no funding is needed within the horizon. Delayed: €30,000 / €0 / €20,000, so €10,000 is still needed for the reserve. The obligation remains and only moves later. |
| Confirmation that the January invoice is collected before the principal is paid in February | This removes the worse low point within February in the base case. The €30,000 month-end gap remains. |
| The duplicate `invoice-jan` record is actually a second, separate invoice | The ledger lists `invoice-jan` twice with identical fields. Under the task's rule it counts once. Counted twice, January revenue would be €100,000 and February cash would close at +€30,000 with no funding need. That reading would hide the shortfall, so check it against the invoicing system before relying on either answer. |
| An approved facility, deferral or early collection | None is assumed. Each enters the model only after it is confirmed in writing. |

## What this monthly horizon cannot establish

- **Daily lows.** Only month-end balances are modeled. The figures under
  Downside show how adverse timing within a month enlarges the need.
- **Anything after March.** Recovery from April onwards, any later principal
  instalments, and whether €20,000 a month of profit continues.
- **Items missing from the ledger.** There are no opening receivables or
  payables, interest, VAT or other taxes, payroll timing or capital
  expenditure. Missing entries are treated as zero here, and that may not be
  true.
- **Whether customers pay, and how likely a delay is.** The delayed scenario
  is an assumption, not a probability.
- **The nature of the €10,000 reserve.** It may be a loan covenant or an
  internal policy, and the consequence of breaching it differs.
- **The accounting treatment.** Recognition periods are used as stated, not
  audited. The ledger periods (January–March 2026) are before today's date
  (10 October 2026). This analysis treats them as a forecast, as the task
  frames them. For a review of past results, reconcile them against bank
  statements instead.

## Recommended next action

Before the February principal falls due, do the following. Nothing here is
approved yet, and this advice does not authorize any payment or borrowing.

1. **Confirm three facts:**
   - the exact due date and amount of the €70,000 principal
   - the expected payment date of the January invoice, confirmed with the
     customer
   - whether the repeated `invoice-jan` line is a duplicate export
2. **Ask the lender for a written deferral or rescheduling of the February
   principal.** In parallel, ask the bank about a committed facility. Aim for
   at least €30,000 to cover the base month-end gap, and plan for €80,000 to
   cover a one-month payment delay or adverse timing within February.
3. **Track cash weekly against this model until March.** The trigger: if the
   January invoice has not arrived by the principal date, treat the delayed
   scenario (€80,000) as the working case.
4. **Set a decision date before the principal is due.** If neither a deferral
   nor a facility is confirmed by then, the owner must choose how to cover the
   gap, for example by injecting their own capital or agreeing a partial
   payment with the lender. That choice needs the owner's own authority and
   the lender's agreement.

## Source, basis and reproduction

- **Source:** `ledger.json` (SHA-256 `5be1e094…6a967eeb`), read-only. It has 8
  transaction records with 7 unique IDs; `invoice-jan` appears twice with
  identical fields and is counted once.
- **Basis:**
  - Revenue and operating costs fall in the month given by
    `recognized_period`.
  - Receipts and payments fall in the month given by `cash_period`, and only
    if that month is within 2026-01 to 2026-03. The March invoice is collected
    in 2026-04, so it is not received inside the horizon.
  - The principal (`financing_payment`) is a cash payment only, never an
    operating cost.
  - The delayed scenario moves each invoice's `cash_period` one calendar month
    later. Costs and the principal keep their dates.
  - Cash rolls forward from €60,000 of opening cash.
  - Minimum cash is the lowest of the opening balance and the month-end
    balances.
  - Funding needed = max(0, €10,000 − minimum cash).
- **Reproduce:** `python3 analyze.py --input ledger.json --output results.json`
  (Python 3.9+, standard library only).
- **Checks performed:**
  - Every month matches a separately written recalculation, and opening cash
    plus receipts minus payments equals closing cash in both scenarios.
  - A conflicting duplicate ID makes the program exit nonzero without writing
    or changing any output.
  - Identical repeated records give the same result as a ledger with the
    duplicate removed.
  - Changing the inputs changes the outputs.
  - A delayed December receipt lands in January of the next year.
  - Cash dated outside the horizon is ignored.
  - Malformed inputs are rejected: decimal cents, unknown record kinds,
    non-consecutive periods, repeated JSON keys and a currency other than EUR.
