# Owner decision: resolve the February cash gap before obligations fall due

The supplied business earns **€60,000 of operating profit in January–March 2026**, but the stated cash schedule cannot fund every obligation. Base-case cash reaches **−€20,000 at February month-end**. Preserving the specified **€10,000 reserve** requires **€30,000 of additional available cash** by the shortfall. With every invoice receipt one calendar month later, the lowest month-end cash is **−€70,000**, requiring **€80,000 to preserve that reserve**. These are modeled funding needs; no borrowing, credit facility, payment deferral or accelerated collection is assumed approved.

## Basis and reproducibility

Source: the supplied [ledger.json](ledger.json), interpreted under [the task contract](../task.md). This is a fictional EUR scenario for the three listed months, not verified actual accounts or an approved forecast. Values in the input and [results.json](results.json) are integer EUR cents; tables below convert cents to euros by dividing by 100. The source supplies no preparation date, real entity, agreement status or probability estimates.

There are eight input transaction records and seven unique IDs. The identical repetition of `invoice-jan` counts once: three €50,000 invoices, three €30,000 operating costs and one €70,000 principal repayment. Opening cash is €60,000 and the required reserve is €10,000. The model treats supplied opening cash as available; restriction status needs confirmation before using this for a real decision. Missing taxes, interest, capital spending, other liabilities or cash restrictions are not established to be zero; they are absent from this limited model.

Revenue and operating costs belong to `recognized_period`. Principal is a cash payment, not an operating expense. Base cash uses `cash_period`. Delayed cash moves each invoice receipt one calendar month later, including across year boundaries; cost and principal dates stay fixed. Amounts falling outside the listed horizon have no cash effect inside it. Each closing balance equals opening balance plus receipts minus payments; the next month opens at that closing balance. Minimum cash includes the initial €60,000 and all three month-end balances. The first negative period means the first listed month with a negative closing balance.

Reproduce the machine-readable model with Python's standard library:

```sh
python3 analyze.py --input ledger.json --output results.json
```

## Profit and cash schedules

All amounts below are EUR. Profit is the recognized operating result on the supplied items, before any missing costs.

| Scenario | Month | Revenue | Operating cost | Profit | Cash receipts | Cash payments | Closing cash |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Base | 2026-01 | 50,000 | 30,000 | 20,000 | 0 | 30,000 | 30,000 |
| Base | 2026-02 | 50,000 | 30,000 | 20,000 | 50,000 | 100,000 | −20,000 |
| Base | 2026-03 | 50,000 | 30,000 | 20,000 | 50,000 | 30,000 | 0 |
| Delayed | 2026-01 | 50,000 | 30,000 | 20,000 | 0 | 30,000 | 30,000 |
| Delayed | 2026-02 | 50,000 | 30,000 | 20,000 | 0 | 100,000 | −70,000 |
| Delayed | 2026-03 | 50,000 | 30,000 | 20,000 | 50,000 | 30,000 | −50,000 |

| Measure | Base | Delayed by one month |
| --- | ---: | ---: |
| Recognized revenue, January–March | 150,000 | 150,000 |
| Recognized operating costs | 90,000 | 90,000 |
| Operating profit | 60,000 | 60,000 |
| Receipts inside horizon | 100,000 | 50,000 |
| Payments inside horizon | 160,000 | 160,000 |
| First negative month-end / first reserve breach | February 2026 | February 2026 |
| Minimum cash including opening | −20,000 | −70,000 |
| Funding to avoid negative month-end cash | 20,000 | 70,000 |
| Funding to preserve €10,000 reserve | 30,000 | 80,000 |

February payments comprise €30,000 of operating costs plus €70,000 of principal. In base, €60,000 opening cash + €100,000 receipts − €90,000 operating payments − €70,000 principal = €0 at March-end. The earnings-to-cash bridge is €60,000 profit − €50,000 of recognized invoice revenue not yet received inside the horizon − €70,000 principal = **−€60,000 cash movement**. The corresponding delayed bridge is €60,000 profit − €100,000 not received inside the horizon − €70,000 principal = **−€110,000 cash movement**, leaving −€50,000 at March-end. Recognized operating costs happen to be paid in the same months in this ledger.

March's base receipt schedule leaves `invoice-mar` (€50,000) for April. Delayed leaves `invoice-feb` (€50,000) for April and `invoice-mar` (€50,000) for May. Those receipts cannot cover an earlier February payment without a separately agreed timing or financing change. Negative modeled cash marks unmet obligations or a funding gap, not an executable bank balance.

## Action and conditions that change the decision

Before the February obligations fall due, confirm unrestricted opening cash, exact collection and payment dates, and the €70,000 principal maturity with the relevant records. Build a daily or weekly schedule covering the maturity. On that verified schedule, seek a documented solution sized to maintain the owner's €10,000 reserve: additional cash available in time, an agreed principal deferral, or a credible combination of earlier collection and lower/deferred outflows. Obtain the relevant approvals before executing any option. Do not commit additional discretionary cash against the positive profit figure while the gap remains unresolved.

For the monthly model alone, an extra €30,000 available before the February trough raises the base minimum to €10,000. The delayed case needs €80,000 on the same basis—€50,000 more. Returning the delayed receipts to their base dates only returns the need to €30,000; it does not eliminate the base shortfall. As a conditional sensitivity, deferring the full €70,000 principal beyond March raises the base minimum to €30,000 and delayed minimum to €0, leaving the delayed case €10,000 below reserve. Such a deferral requires lender agreement and a longer forecast because it moves, rather than removes, the obligation. Financing fees and future repayments would also need modeling.

The decision changes once verified, approved cash sources or revised obligations remove the gap while preserving reserve at actual due dates, including a collection downside the owner accepts. Recalculate from changed inputs before committing; the one-month delay is a timing stress, not a probability estimate.

## Limits and checks

This monthly, three-month horizon cannot establish the lowest balance within a month, the precise day a payment becomes unfundable, ability to pay each obligation in sequence, April/May liquidity, full-year profitability, legal solvency, credit availability, or long-term viability. Even financing equal to the modeled reserve need may be insufficient if February payments precede receipts. Costs after March, omitted liabilities and financing terms require a longer forecast and evidence. No external accounts, legal treatment or current financing terms were researched for these fictional inputs.

The accompanying program calculates from its selected input, uses integer cents, deduplicates identical records, and validates all records before writing the result. Conflicting duplicate IDs cause a nonzero exit without creating or replacing the output. Output replacement is atomic after calculation. The delivered model was checked against the source totals and monthly roll-forward, along with changed inputs, calendar rollover, horizon exclusion, zero activity, reserve behavior and duplicate rejection. These are calculation and contract checks, not confirmation of real collection or financing.
