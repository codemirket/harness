# Refund-safe ledger totals

Work only inside the supplied scratch `workspace/`. No network, account, or production data is involved.

The tiny ledger reports refund entries as extra charges. Correct the behavior at its source while keeping the public `summarize(entries)` contract and the behavior of ordinary charges. Each entry has `kind` (`charge` or `refund`) and a nonnegative integer `cents`; invalid input must still raise `ValueError`. The returned `charged_cents`, `refunded_cents`, and `net_cents` should reflect the entries without rounding or floating-point math. Add a focused local check if useful, and report the behavior you verified. Do not change unrelated files.

You may verify locally with `python3 -I ../verify.py .` from this workspace.
