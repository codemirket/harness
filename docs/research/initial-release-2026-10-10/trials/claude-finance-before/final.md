The business is profitable but can't fund its obligations from its own cash in this ledger. It makes €20,000 profit every month, but the €70,000 loan repayment falls due in February, before most invoices are paid. That pushes cash below zero in February.

| | Base (stated payment dates) | Delayed (customers pay a month later) |
|---|---:|---:|
| Profit over 3 months | €60,000 | €60,000 |
| First month ending negative | 2026-02 | 2026-02 |
| Lowest cash | −€20,000 | −€70,000 |
| Extra cash needed to keep the €10,000 reserve | **€30,000** | **€80,000** |

I wrote `analyze.py`, `results.json` and `decision.md`.

**Things worth knowing:**
- **Duplicate invoice:** `invoice-jan` appears twice with identical details, so it's counted once. Counting it twice would show +€30,000 in February and hide the shortfall.
- **Missing cash, profit vs liquidity:** the March invoice is paid in April, outside the horizon. That €50,000 still owed plus the €70,000 repayment is why €60,000 of profit becomes a €60,000 fall in cash in the base case.
- **Loan deferral:** postponing the whole repayment past March fixes the base case. If customers pay a month late, cash still drops to €0 in February, €10,000 below the reserve.
- **Timing within a month:** the figures are month-end balances. If February's payments go out before the money comes in, the true low point is worse: up to €80,000 needed in the base case and €110,000 in the delayed case. These are worst-case limits, not forecasts.
- **Recommendation:** before the February repayment is made:
  1. Confirm the repayment and that no outflows such as interest, taxes or payroll are missing from the ledger.
  2. Ask the lender in writing for a deferral or split, and get terms for bridge funding of €80,000 (€30,000 minimum).
  3. Build a weekly cash plan for February and March.

  No funding, deferral or faster collection is assumed to be approved.

**Verification:** I ran 59 checks, all passing:
- `results.json` matches a fresh run and my hand calculations.
- Two records with the same ID but different details stop the program with exit status 1, and no output file is created or changed.
- Months roll over correctly from December to January.
- Cash dated outside the three months is left out.
- The loan repayment never counts as an operating cost.
- Bad input is rejected, and using the ledger itself as the output file is refused.

`ledger.json` and `TASK.md` are unchanged. The test script was temporary and I deleted it. The checks are listed in `decision.md`.

The program also rejects a few things the task didn't mention: unknown transaction types, amounts that aren't whole cents, currencies other than EUR, and periods that aren't consecutive months. I made it stop rather than guess.
