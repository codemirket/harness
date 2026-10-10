# Can the business fund its obligations from January to March 2026?

**Decision: no, not as scheduled.** The business is profitable, earning €20,000
a month and €60,000 over the quarter. But it cannot pay the €70,000 loan
principal due in February from its own cash and still keep the €10,000 reserve.
In the base case, cash falls to **−€20,000 at the end of February**.

**How much funding, and by when, depends on dates the ledger does not hold.**
The model measures cash only at month-ends:

- **Base case:** holding the reserve at every month-end needs **€30,000** of
  extra cash or deferred payments.
- **One-month payment delay:** if customers pay a month later, the need is
  **€80,000**.

These are the smallest amounts that could work. If a month's payments go out
before its receipts come in, the need rises to **€80,000** in the base case and
**€110,000** with the delay.

The principal and February costs could be paid on any day in February, so
funding has to be usable **before the first February payment**. On a monthly
ledger, that means by 31 January. Money that only arrives at the end of
February is too late for payments due earlier that month.

This analysis assumes no loan, credit facility or faster collection, because
none has been approved.

All amounts are in EUR. `results.json` holds the exact month-end values in cents.

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
| Funding to hold the €10,000 reserve at month-ends | **€30,000** | **€80,000** |
| Funding to hold the reserve if each month's payments precede its receipts | **€80,000** | **€110,000** |
| Funding just to stay at or above €0 at month-ends | €20,000 | €70,000 |

The month-end rows come from `results.json`. The second funding row is the
timing bound explained under
[Funding amount and deadline](#funding-amount-and-deadline). `analyze.py` does
not output that bound. The test suite recalculates it from the ledger.

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

## Funding amount and deadline

The ledger gives every payment and receipt a month, not a date. Within a month,
the order matters. This section therefore gives a range and a conservative
deadline instead of a single dated figure.

**Timing bound.** Assume every payment in a month goes out before any receipt
in that month comes in. Every flow stays in its stated month. Flows outside
January to March stay excluded.

| Scenario | Month | Opening cash | Payments | Low before receipts | Receipts | Closing cash |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Base | 2026-01 | 60,000 | 30,000 | 30,000 | 0 | 30,000 |
| Base | 2026-02 | 30,000 | 100,000 | **−70,000** | 50,000 | −20,000 |
| Base | 2026-03 | −20,000 | 30,000 | −50,000 | 50,000 | 0 |
| Delayed | 2026-01 | 60,000 | 30,000 | 30,000 | 0 | 30,000 |
| Delayed | 2026-02 | 30,000 | 100,000 | −70,000 | 0 | −70,000 |
| Delayed | 2026-03 | −70,000 | 30,000 | **−100,000** | 50,000 | −50,000 |

| Funding to hold the €10,000 reserve | Base | Delayed |
| --- | ---: | ---: |
| At month-ends only (`results.json`) | €30,000 | €80,000 |
| Timing bound (lowest point above + €10,000) | €80,000 | €110,000 |

**How to read the range.**

- **The month-end figure is the least that could work.** It needs receipts to
  arrive before the payments they fund:
  - **Base, €30,000:** the January invoice must arrive before the principal is
    paid, and the February invoice must arrive in March before any March costs
    are paid. If the payments come first in both months, €30,000 leaves a low
    of −€40,000 in February.
  - **Delayed, €80,000:** February is covered whatever the order, because
    February has no receipts. But the January invoice must arrive in March
    before any March costs are paid. Otherwise cash dips to −€20,000.
- **The timing bound is the most this ledger implies.** It is arithmetic on
  ledger amounts, not a forecast of daily balances. Items missing from the
  ledger, such as payroll taxes, VAT or interest, could make the real low point
  worse.

**Deadline.**

- Month-end balances show that the gap exists by the end of February. They do
  not show when in February it opens. The principal and February costs could
  fall on any day in February, including 1 February.
- February starts with €30,000, which leaves €20,000 above the reserve. So
  before any receipt arrives, any combination of February payments larger than
  €20,000 breaches the reserve.
- Funding must therefore be cleared and usable before the first February
  payment it is meant to cover. Without dates, the only safe deadline this model
  can support is **31 January 2026**.
- A facility, deferral or owner injection that only takes effect at the end of
  February is not enough, at any amount, for payments made earlier in the
  month.
- In the delayed case, the extra €30,000 (from €80,000 up to €110,000) must
  arrive before the first March cost payment. It can be skipped if the January
  invoice is received before those costs are paid.
- The ledger periods are before today (10 October 2026). If these months have
  already passed, the bank statements show what actually happened.

**Date-level evidence still needed** to replace the range and the 31 January
deadline with exact figures and dates:

1. The principal's exact due date and amount, and whether the lender will
   split or move it.
2. The payment dates of February and March operating costs (payroll, rent,
   suppliers), and which of them can move.
3. The expected receipt dates of the January and February invoices: payment
   terms, the customer's confirmation and their payment history. What matters
   most is whether they arrive before the principal and before the March costs.
4. A bank balance on a stated date, and any dated flows missing from the
   ledger (VAT, payroll taxes, interest, card settlements).
5. The lead time for a deferral or facility to become usable after approval,
   which sets how early the request must be made.

## Downside

- **A one-month delay in customer payments** (the delayed scenario) moves no
  cash into February. The shortfall deepens to −€70,000 and is still −€50,000
  at the end of March. Recovery depends on receipts in April (€50,000) and May
  (€50,000), which fall outside this horizon and so cannot be confirmed here.
- **Timing within a month can be worse than the month-end figures.** The
  bound above raises the need to €80,000 in the base case and €110,000 with the
  delay. The real payment dates are not in the ledger.
- **If a customer does not pay at all**, the shortfall is larger. Neither
  scenario models bad debt.

## What would change the decision

| Change | Effect on the result |
| --- | --- |
| €30,000 more cash, or €30,000 less paid out, usable before the first February payment | Holds the base reserve at month-ends (€20,000 avoids negative month-end cash). Keeping the reserve throughout each month additionally requires the January invoice before the February principal payment and the February invoice before the March costs. It does not cover the delayed case. |
| €80,000 usable before the first February payment | Holds the base reserve whatever the order within each month. In the delayed case the reserve holds at month-ends regardless of within-month order; keeping it throughout March additionally requires the January invoice before the March costs. |
| €110,000 usable before the first February payment (or €80,000 by then plus €30,000 before the first March payment) | Holds the reserve under the one-month delay even if every payment precedes every receipt within the month. This is the largest need the ledger implies, and missing items could raise it. |
| Any of these amounts arriving only at the end of February | Does not cover a principal or costs paid earlier in February. Arrival after a payment's due date never helps with that payment. |
| The lender defers the whole €70,000 beyond March (calculated by rerunning the model without the principal) | Base: month-end cash is €30,000 / €50,000 / €70,000. No month-end funding is needed, and the timing bound is €10,000. Delayed: €30,000 / €0 / €20,000, so €10,000 is needed at month-ends and up to €40,000 under the timing bound. The obligation remains and only moves later. |
| Confirmation that the January invoice arrives before the principal is paid | Removes the February part of the base-case timing risk. The base need falls from €80,000 to €60,000, or to €30,000 if the February invoice also arrives before the March costs. |
| The duplicate `invoice-jan` record is actually a second, separate invoice | The ledger lists `invoice-jan` twice with identical fields. Under the task's rule it counts once. Counted twice, January revenue would be €100,000. Base February cash would close at +€30,000, with no month-end funding need. The timing bound would still be €80,000, and the delayed case would be unchanged at €80,000 / €110,000. Check it against the invoicing system before relying on either reading. |
| An approved facility, deferral or early collection | None is assumed. Each enters the model only after it is confirmed in writing. |

## What this monthly horizon cannot establish

- **Lows within a month, or the date funding must arrive.** Only month-end
  balances are modeled. The timing bound limits the need on these ledger
  amounts, but it cannot say on which day cash runs short.
- **Whether a weekly or daily cash forecast is right.** The model has no
  dates. A weekly forecast can at most be reconciled to these monthly totals,
  which checks consistency, not timing.
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

Nothing here is approved yet, and this advice does not authorize any payment
or borrowing.

1. **Gather the five items of date-level evidence listed above.** Use them to
   roll cash forward by date from a stated bank balance to the end of March.
   That turns the €30,000–€110,000 range and the 31 January deadline into one
   amount and one date.
2. **At the same time, start funding talks sized to the bound.**
   - Ask the lender for a written deferral or rescheduling of the February
     principal.
   - Ask the bank about a committed facility that can be drawn **before the
     first February payment**.
   - Seek availability of up to €110,000, and draw only what the dated
     evidence requires.
   - If only €80,000 can be arranged, the base case is covered. The delayed
     case then also depends on the January invoice arriving before the March
     costs.
3. **Set a decision date early enough to allow for the lender's or bank's
   lead time before the first February payment.** If neither a deferral nor a
   facility is confirmed in writing by then, the owner must choose how to cover
   the gap, for example by injecting their own capital or agreeing a partial
   payment with the lender. That choice needs the owner's own authority and
   the lender's agreement.

## What this artifact checks, and what it only proposes

### Checked by this artifact

- `analyze.py` calculates monthly revenue, operating costs, profit, receipts,
  payments and month-end closing cash for both scenarios from `ledger.json`.
  It also calculates minimum cash, the first negative month and funding to the
  reserve.
- `test_analyze.py` checks the following:
  - those month-end results
  - input validation
  - that `results.json` matches a fresh run
  - that the key figures and the timing bound in this report match an
    independent recalculation from the ledger
- Neither the program nor the tests know any transaction date. They cannot
  confirm a low within a month, a funding arrival date, or any weekly or daily
  forecast.

### Proposed operational follow-up (not performed or validated here)

- Once the dated evidence exists, keep a weekly cash forecast from a stated
  bank balance until the principal is settled and the March costs are paid.
- Check that forecast against bank statements and signed documents (the loan
  schedule, the facility letter, customer remittances), not against this
  model. This model can confirm only that the forecast's ledger items add up to
  the same monthly totals.
- Proposed trigger for that forecast: if the January invoice has not arrived by
  the principal due date, treat the delayed scenario and its €110,000 timing
  bound as the working case until the invoice is received.

## Source, basis and reproduction

This is a development exercise using synthetic inputs, not an assessment of a real business.

- **Source:** `ledger.json` (SHA-256 `5be1e094…6a967eeb`), read-only. It has 8
  transaction records with 7 unique IDs; `invoice-jan` appears twice with
  identical fields and is counted once.
- **Basis:**
  - Revenue and operating costs fall in the month given by
    `recognized_period`.
  - Receipts and payments fall in the month given by `cash_period`, and only
    if that month is one of the declared periods (2026-01 to 2026-03). The
    March invoice is collected in 2026-04, so it is not received inside the
    horizon.
  - A horizon may leave out a month, such as January and March only. The
    program accepts it and leaves out flows dated in the missing month. Cash
    then rolls straight from one listed month to the next, and stderr names
    the gap. This ledger's horizon has no gaps.
  - The principal (`financing_payment`) is a cash payment only, never an
    operating cost.
  - The delayed scenario moves each invoice's `cash_period` one calendar month
    later. Costs and the principal keep their dates.
  - Cash rolls forward from €60,000 of opening cash.
  - Minimum cash is the lowest of the opening balance and the month-end
    balances.
  - Funding needed = max(0, €10,000 − minimum cash).
  - Timing bound = max(0, €10,000 − lowest balance). The lowest balance is
    taken over opening cash and, for each month, opening cash minus that
    month's payments, with receipts not yet counted.
- **Reproduce:**
  - `python3 analyze.py --input ledger.json --output results.json`
    (Python 3.9+, standard library only)
  - `python3 -m unittest -v test_analyze`
- **Checks performed (`test_analyze.py`):**
  - Every month of both scenarios matches hand-calculated values. Opening cash
    plus receipts minus payments equals closing cash, and `results.json`
    matches a fresh run.
  - A January/March horizon is accepted. February flows are left out, a
    receipt delayed into the missing month is dropped, and stderr names the
    gap.
  - A conflicting duplicate ID makes the program exit nonzero without writing
    or changing any output.
  - Identical repeated records give the same result as a ledger with the
    duplicate removed.
  - A delayed December receipt lands in January of the next year.
  - Cash dated outside the horizon is ignored.
  - Malformed inputs are rejected: decimal cents, unknown record kinds,
    periods out of order or repeated, repeated JSON keys, and a currency other
    than EUR.
  - The key figures and timing bound in this report match an independent
    recalculation. No funding row is called sufficient if it arrives at the
    end of February. Weekly forecasting appears only as a proposal, not as a
    check.
