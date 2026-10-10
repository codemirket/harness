#!/usr/bin/env python3
"""Monthly profit and cash scenarios for a ledger of EUR-cent transactions.

Usage: python3 analyze.py --input ledger.json --output results.json

Revenue and operating costs are recognized in recognized_period. Cash moves in
cash_period and only counts when that period is one of the ledger's declared
periods. The horizon may omit months (for example January and March only); flows
dated in an omitted month are excluded and cash rolls directly between the listed
months, so the omission is reported on stderr. Financing payments (principal) are
cash only, never operating cost. The delayed scenario moves every invoice receipt
one calendar month later.

An invalid ledger -- including two records that share an ID but differ -- exits
nonzero and writes nothing. Identical repeated records count once.
"""
import argparse
import json
import os
import re
import sys
import tempfile

KINDS = ("invoice", "operating_cost", "financing_payment")
PERIOD_RE = re.compile(r"^\d{4}-(0[1-9]|1[0-2])$")
SCENARIOS = (("base", 0), ("delayed", 1))  # name, months added to invoice receipts


class LedgerError(ValueError):
    pass


def check_period(value, where):
    if not isinstance(value, str) or not PERIOD_RE.match(value):
        raise LedgerError(f"{where}: expected a YYYY-MM period, got {value!r}")
    return value


def shift_month(period, months):
    index = int(period[:4]) * 12 + int(period[5:]) - 1 + months
    return f"{index // 12:04d}-{index % 12 + 1:02d}"


def check_cents(value, where, minimum=None):
    if isinstance(value, bool) or not isinstance(value, int):
        raise LedgerError(f"{where}: expected integer cents, got {value!r}")
    if minimum is not None and value < minimum:
        raise LedgerError(f"{where}: must be at least {minimum}, got {value}")
    return value


def reject_duplicate_keys(pairs):
    keys = [key for key, _ in pairs]
    repeated = sorted({key for key in keys if keys.count(key) > 1})
    if repeated:
        raise LedgerError(f"repeated JSON key(s) in one object: {', '.join(repeated)}")
    return dict(pairs)


def validate(ledger):
    """Return (periods, opening, reserve, unique transactions) or raise LedgerError."""
    if not isinstance(ledger, dict):
        raise LedgerError("ledger must be a JSON object")
    if ledger.get("currency") != "EUR":
        raise LedgerError(f"currency must be EUR, got {ledger.get('currency')!r}")

    periods = ledger.get("periods")
    if not isinstance(periods, list) or not periods:
        raise LedgerError("periods must be a non-empty list")
    for i, period in enumerate(periods):
        check_period(period, f"periods[{i}]")
        if i and period <= periods[i - 1]:
            raise LedgerError(
                f"periods must be ascending without repeats; {periods[i - 1]} is followed by {period}"
            )

    opening = check_cents(ledger.get("opening_cash_cents"), "opening_cash_cents")
    reserve = check_cents(ledger.get("minimum_reserve_cents"), "minimum_reserve_cents", 0)

    records = ledger.get("transactions")
    if not isinstance(records, list):
        raise LedgerError("transactions must be a list")
    unique = {}
    for i, record in enumerate(records):
        where = f"transactions[{i}]"
        if not isinstance(record, dict):
            raise LedgerError(f"{where}: expected an object")
        tid = record.get("id")
        if not isinstance(tid, str) or not tid:
            raise LedgerError(f"{where}: id must be a non-empty string")
        where = f"{where} ({tid})"
        if tid in unique:
            if record != unique[tid]:
                raise LedgerError(f"{where}: conflicting duplicate of an earlier record with id {tid!r}")
            continue  # identical repeat counts once
        kind = record.get("kind")
        if kind not in KINDS:
            raise LedgerError(f"{where}: kind must be one of {', '.join(KINDS)}, got {kind!r}")
        check_cents(record.get("amount_cents"), f"{where}.amount_cents", 0)
        check_period(record.get("cash_period"), f"{where}.cash_period")
        recognized = record.get("recognized_period")
        if kind != "financing_payment" or recognized is not None:
            check_period(recognized, f"{where}.recognized_period")
        unique[tid] = record
    return periods, opening, reserve, list(unique.values())


def omitted_months(periods):
    """Months between the first and last declared period that the horizon leaves out."""
    gaps = []
    for earlier, later in zip(periods, periods[1:]):
        first, last = shift_month(earlier, 1), shift_month(later, -1)
        if first <= last:
            gaps.append(first if first == last else f"{first}..{last}")
    return gaps


def scenario(periods, opening, reserve, transactions, receipt_shift):
    totals = {p: {"revenue": 0, "cost": 0, "receipts": 0, "payments": 0} for p in periods}
    for t in transactions:
        amount, kind = t["amount_cents"], t["kind"]
        recognized = t.get("recognized_period")
        if kind == "invoice":
            if recognized in totals:
                totals[recognized]["revenue"] += amount
            received = shift_month(t["cash_period"], receipt_shift)
            if received in totals:
                totals[received]["receipts"] += amount
            continue
        if kind == "operating_cost" and recognized in totals:
            totals[recognized]["cost"] += amount
        if t["cash_period"] in totals:  # operating cost or principal paid
            totals[t["cash_period"]]["payments"] += amount

    cash = minimum = opening
    first_negative = None
    months = []
    for period in periods:
        m = totals[period]
        cash += m["receipts"] - m["payments"]
        minimum = min(minimum, cash)
        if cash < 0 and first_negative is None:
            first_negative = period
        months.append({
            "period": period,
            "revenue_cents": m["revenue"],
            "operating_cost_cents": m["cost"],
            "profit_cents": m["revenue"] - m["cost"],
            "receipts_cents": m["receipts"],
            "payments_cents": m["payments"],
            "closing_cash_cents": cash,
        })
    return {
        "months": months,
        "first_negative_period": first_negative,
        "minimum_cash_cents": minimum,
        "financing_to_reserve_cents": max(0, reserve - minimum),
    }


def analyze(ledger):
    periods, opening, reserve, transactions = validate(ledger)
    return {
        "currency": "EUR",
        "scenarios": {
            name: scenario(periods, opening, reserve, transactions, shift)
            for name, shift in SCENARIOS
        },
    }


def write_json_atomic(path, payload):
    directory = os.path.dirname(os.path.abspath(path))
    fd, tmp = tempfile.mkstemp(prefix=".analyze-", suffix=".tmp", dir=directory)
    umask = os.umask(0)
    os.umask(umask)
    try:
        os.fchmod(fd, 0o666 & ~umask)  # mkstemp creates 0600; use normal file mode
        with os.fdopen(fd, "w", encoding="utf-8") as handle:
            json.dump(payload, handle, indent=2)
            handle.write("\n")
        os.replace(tmp, path)
    except BaseException:
        if os.path.exists(tmp):
            os.unlink(tmp)
        raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--input", required=True, help="ledger JSON path")
    parser.add_argument("--output", required=True, help="results JSON path")
    args = parser.parse_args(argv)

    if all(map(os.path.exists, (args.input, args.output))) and os.path.samefile(args.input, args.output):
        print("error: --output must not be the input ledger", file=sys.stderr)
        return 2
    try:
        with open(args.input, encoding="utf-8") as handle:
            ledger = json.load(handle, object_pairs_hook=reject_duplicate_keys)
        results = analyze(ledger)
    except (OSError, json.JSONDecodeError, LedgerError) as exc:
        print(f"error: {args.input}: {exc}", file=sys.stderr)
        return 1
    gaps = omitted_months(ledger["periods"])
    if gaps:
        print(f"note: horizon omits {', '.join(gaps)}; flows dated there are excluded", file=sys.stderr)
    try:
        write_json_atomic(args.output, results)
    except OSError as exc:
        print(f"error: {args.output}: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
