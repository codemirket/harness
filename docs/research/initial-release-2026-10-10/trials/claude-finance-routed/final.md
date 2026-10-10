I've written and checked all three files. The answer for the owner: the business is profitable but can't pay its February loan repayment from its own cash.

**The decision:**
- **Profit:** €20,000 a month, €60,000 for January–March.
- **Base case:** the €70,000 loan principal is due in February, the same month the January invoice is first collected. Cash ends February at −€20,000 and March at exactly €0. Keeping the €10,000 reserve at every month-end needs **€30,000** of extra cash or deferred payments.
- **If customers pay one month late:** cash falls to −€70,000 in February and is still −€50,000 at the end of March. The need rises to **€80,000**.
- **What explains the gap:** profit of €60,000, less €50,000 invoiced but not yet collected (€100,000 if customers pay late), less the €70,000 principal. That gives a cash change of −€60,000 (−€110,000 if late).
- **Recommended next step:** before the principal falls due, confirm the facts and ask the lender for a deferral or a committed facility. Aim for at least €30,000 and plan for €80,000, with a set date by which to decide. The analysis assumes no funding, facility or faster collection, as instructed.

**Two points the owner should see** (both are in `decision.md`):
- **Timing within a month:** the ledger only gives months. If the February principal and costs are paid before the January invoice arrives, the base case alone could need €80,000.
- **The repeated `invoice-jan` record:** it appears twice with identical details and is counted once, as the task requires. If it were actually a second invoice, February would close at +€30,000 and the shortfall would disappear. That's why I recommend checking it against the invoicing system.

**Files:**
- **`analyze.py`:** run with `--input PATH --output PATH`; it uses only Python's standard library. If two records share an ID but differ, it stops with an error, writes no file and leaves any existing output unchanged.
- **`results.json`:** the monthly figures in exactly the shape the task specifies, all integer cents.
- **`decision.md`:** the advice, the tables, the remaining unknowns, the sources and how to reproduce the numbers.

**Checks:**
- Every month matched a separately written recalculation, and opening cash plus receipts minus payments equals closing cash in both scenarios.
- A conflicting duplicate ID makes the program exit with an error and write nothing.
- Identical repeated records give the same result as removing the copy.
- Changing the inputs changes the results.
- A late December payment lands in January of the next year, and cash dated outside the three months is ignored.
- Malformed inputs are rejected.
- `ledger.json` and `TASK.md` are unchanged, and I deleted the temporary test files.
- These checks ran during the session only, so there is no saved test file in the folder.

I didn't register the `financial-analysis` skill in this project, even though your global instructions call for it. Doing so would add `.ai/` and `.claude/` folders, and the task allows only the three requested files. I can register it if you want.
