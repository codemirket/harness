I fixed all three defects in `analyze.py` and `decision.md` and added a test suite, `test_analyze.py`. All 21 tests pass on the repaired files. Against copies of the original files, 9 of them fail (5 failures, 4 errors), each defect caught by at least one. `ledger.json` and `TASK.md` are unchanged, and `results.json` came out byte-identical after regenerating.

**1. A horizon can now skip a month.** `analyze.py` accepts something like January and March only. Flows dated in the skipped month are left out, and cash rolls straight from January to March. It prints a one-line note on stderr naming the skipped month, so the gap isn't silent. Periods that are out of order or repeated are still rejected. The input/output contract is unchanged, and a conflicting duplicate ID still exits nonzero without writing anything.

**2. Funding amounts and deadline are now qualified.** The ledger records months, not dates, so the report now gives a range for each scenario:

| Funding to hold the €10,000 reserve | Base | Customers pay a month late |
| --- | ---: | ---: |
| Checking only month-end balances (the smallest amount that could work) | €30,000 | €80,000 |
| If each month's payments go out before its receipts come in (the most this ledger implies) | €80,000 | €110,000 |

- **Deadline:** the €70,000 principal and February costs could be due as early as 1 February. So funding must be usable before the first February payment, which on monthly data means by 31 January. The report now says money arriving at the end of February is too late for payments due earlier that month.
- **Conditions for the smaller amounts:** €30,000 works only if both outstanding invoices arrive before the payments they fund. For €30,000 in the base case: the January invoice before the principal, and the February invoice before the March costs. €80,000 in the late-payment case needs the January invoice before the March costs.
- **Evidence still needed:** five dated items would turn the range into one figure. They are the principal's exact due date, the cost payment dates, the invoice receipt dates, a dated bank balance and how long a lender or bank needs to make funds available.

**3. The weekly forecast is now a proposal, not a check.** The report separates what the program and tests actually verify from a proposed weekly forecast, which this monthly model cannot validate. That forecast would need checking against bank statements and the loan and facility documents.

While checking the figures I found two rows in the "What would change the decision" table that were incomplete, and corrected them:
- **Lender defers the €70,000 principal:** the late-payment case still needs up to €40,000, not just €10,000.
- **January invoice arrives before the principal:** the base case need falls to €60,000, not €30,000.

**Limits:**
- The report tests check specific wording and figures, so future rewording may require updating them.
- The €80,000/€110,000 figures are arithmetic on the ledger amounts, not a daily forecast. Costs missing from the ledger, such as tax or interest, could make the real low worse.
- The ledger covers January–March 2026, which is before today, so the 31 January deadline is a planning date. Bank statements would show what actually happened.

Nothing has been committed.

Files are in the task folder:
- `analyze.py`
- `results.json`
- `decision.md`
- `test_analyze.py` (run with `python3 -m unittest -v test_analyze`)
