**The business is profitable in the model, but cannot fund its scheduled obligations from the stated cash and collections.** Both scenarios first show negative month-end cash in February 2026. Keeping the owner's EUR 10,000 minimum reserve requires EUR 30,000 of additional cash support in the base case, or EUR 80,000 under the stipulated one-month collection delay. These are calculated gaps; no financing, credit facility, accelerated collection or payment rescheduling is assumed to be approved.

The basis is the synthetic [ledger.json](ledger.json), applying the rules in [TASK.md](TASK.md), for January–March 2026. All tables below show **EUR**, converted from integer cents at 100 cents per EUR. Opening cash is 6,000,000 cents (EUR 60,000), and the reserve is 1,000,000 cents (EUR 10,000). The eight input records contain seven unique IDs: the identical second `invoice-jan` record counts once. The source ledger's SHA-256 is `5be1e094f76fde050cd4c8a0c0648f2fe296009b04bfc10720c04e276a967eeb`.

Revenue and operating costs follow `recognized_period`, independently of collection or payment dates. The three unique invoices (`invoice-jan`, `invoice-feb`, `invoice-mar`) each contribute EUR 50,000 of revenue. The three operating costs (`cost-jan`, `cost-feb`, `cost-mar`) each contribute EUR 30,000 of expense. The `principal` record is a EUR 70,000 February cash payment, excluded from operating expense and profit.

| Recognized period | Revenue | Operating cost | Operating profit |
| --- | ---: | ---: | ---: |
| 2026-01 | 50,000 | 30,000 | 20,000 |
| 2026-02 | 50,000 | 30,000 | 20,000 |
| 2026-03 | 50,000 | 30,000 | 20,000 |
| Total, either scenario | 150,000 | 90,000 | 60,000 |

Base receipts follow `cash_period`: January's invoice is collected in February, February's in March, and March's in April. The delayed case shifts every invoice receipt one calendar month later, to March, April and May respectively. Only receipts in the listed horizon count. Operating payments remain EUR 30,000 each month, with principal adding EUR 70,000 to February payments. Recognition and all payment dates remain unchanged in the delayed case.

| Period | Base receipts | Delayed receipts | Payments, either case | Base closing cash | Delayed closing cash |
| --- | ---: | ---: | ---: | ---: | ---: |
| 2026-01 | 0 | 0 | 30,000 | 30,000 | 30,000 |
| 2026-02 | 50,000 | 0 | 100,000 | -20,000 | -70,000 |
| 2026-03 | 50,000 | 50,000 | 30,000 | 0 | -50,000 |
| Horizon totals / final closing cash | 100,000 | 50,000 | 160,000 | 0 | -50,000 |

The cash reconciliation is opening cash + receipts − payments. Base closing cash is EUR 60,000 + EUR 100,000 − EUR 160,000 = EUR 0; delayed closing cash is EUR 60,000 + EUR 50,000 − EUR 160,000 = EUR -50,000. February specifically rolls from EUR 30,000 to EUR -20,000 in base, or EUR -70,000 with no February receipts in delayed. Negative balances represent unfunded obligations, not an available overdraft. The base recovery to zero in March does not resolve February's shortfall.

Profit is not available cash: base leaves EUR 50,000 of recognized invoices uncollected at March-end and repays EUR 70,000 of principal. Thus EUR 60,000 profit − EUR 50,000 uncollected revenue − EUR 70,000 principal = EUR -60,000 cash movement. Delayed leaves EUR 100,000 uncollected, giving EUR -110,000 cash movement. Profit remains EUR 60,000 because this downside changes timing only; it assumes neither bad debts nor additional costs.

| Liquidity measure | Base | Delayed |
| --- | ---: | ---: |
| First negative month-end | 2026-02 | 2026-02 |
| Minimum cash, including opening cash | -20,000 | -70,000 |
| Support needed merely to avoid negative month-ends | 20,000 | 70,000 |
| Support needed to preserve EUR 10,000 reserve | 30,000 | 80,000 |

The reserve calculation is `max(0, minimum_reserve_cents - minimum_cash_cents)`: base `1,000,000 - (-2,000,000) = 3,000,000` cents; delayed `1,000,000 - (-7,000,000) = 8,000,000` cents. The delay adds EUR 50,000 to the funding gap. These thresholds assume net support is available before the cash shortfall and remains available through the horizon; any funding fees, interest or repayments would need to be added to the schedule.

**The decision changes when timely, confirmed cash or agreed payment changes close the relevant gap.** EUR 30,000 of additional net cash covers the base monthly reserve requirement; EUR 80,000 covers both modeled scenarios. Restoring base collection timing alone still leaves the EUR 30,000 reserve gap. As a separate, unapproved sensitivity, moving the entire EUR 70,000 principal payment to April would produce base monthly balances of EUR 30,000, EUR 50,000 and EUR 70,000, but delayed balances of EUR 30,000, EUR 0 and EUR 20,000. Even that deferral leaves a EUR 10,000 delayed reserve gap, and the debt remains payable beyond this horizon. This sensitivity does not alter the supplied ledger or the two scenarios in results.json.

The bounded next action is to prepare a dated cash schedule for the modeled February obligations, reconcile available bank cash, confirm invoice collection dates and the principal terms, and take a specific funding or payment-change proposal to the owner for approval. Use EUR 80,000 as the net monthly reserve-gap target if the owner wants the plan to withstand the stipulated delay; EUR 30,000 supports only the base case. Do not commit additional discretionary cash on the strength of the profit figure while that gap remains unresolved. This analysis authorizes no borrowing, spending or external contact.

Month-end totals cannot establish whether receipts arrive before individual payments within February—or whether an earlier intra-month cash low is worse. The reserve gaps are therefore monthly model requirements, not proof that a particular facility is sufficient on every payment date. The January–March horizon also cannot establish April onward affordability, long-term solvency, full-year profit, collectability, financing eligibility or covenant compliance. The ledger does not establish restricted cash, taxes, interest, other liabilities, additional capital spending or financing terms; missing items are not verified zero. Extend the dated forecast through any deferred debt and funding repayment before approving a solution. The delayed case is a specified stress, not a probability forecast or a worst-case bound.

Reproduce the saved [results.json](results.json) with the standard-library-only [analyze.py](analyze.py):

```sh
python3 analyze.py --input ledger.json --output results.json
```

Verification on Python 3.9.6 compared the complete saved JSON with independently calculated expected values, checked integer cents and all cash roll-forwards, and exercised the real CLI with changed opening cash and reserve, repeated records, calendar-year rollover, receipts moving into and out of the horizon, separate recognition/payment dates, and opening-cash minima. Thirty conflicting-duplicate invocations rejected altered amounts, dates, kinds or metadata without creating a new output or changing an existing output. Additional checks covered invalid cents/months, an empty horizon, zero balances, source-overwrite rejection, byte-identical regeneration and the principal-deferral sensitivity. TASK.md and ledger.json retained their original SHA-256 hashes; temporary verification files were removed.
