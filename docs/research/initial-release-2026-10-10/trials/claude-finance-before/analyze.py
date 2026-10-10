#!/usr/bin/env python3
"""Monthly profit-versus-cash analysis of a ledger in integer EUR cents.

Usage: python3 analyze.py --input ledger.json --output results.json

Basis:
- Invoice revenue and operating costs are recognized in recognized_period.
- Cash moves in cash_period. A financing_payment (principal repayment) is a
  cash payment only and never an operating cost.
- The horizon is the ledger's periods. Cash dated outside it is neither
  collected nor paid inside it.
- "base" uses the stated cash_period. "delayed" moves every invoice receipt one
  calendar month later; cost and principal cash dates are unchanged.
- Cash rolls forward from opening_cash_cents. minimum_cash_cents covers opening
  cash and every month-end balance; financing_to_reserve_cents is
  max(0, minimum_reserve_cents - minimum_cash_cents).

Records that repeat an id with identical content count once. Records that share
an id with different content abort the run with exit status 1, and the output
file is neither created nor changed. Every check runs before anything is
written, and the output is replaced atomically.
"""

import argparse
import json
import os
import re
import sys
import tempfile

PERIOD_PATTERN = re.compile(r"^\d{4}-(0[1-9]|1[0-2])$")
KINDS = ("invoice", "operating_cost", "financing_payment")
SCENARIO_INVOICE_SHIFTS = (("base", 0), ("delayed", 1))


class LedgerError(Exception):
    """The input cannot be analysed without guessing."""


def shift_period(period, months):
    year, month = int(period[:4]), int(period[5:7])
    index = year * 12 + (month - 1) + months
    return "%04d-%02d" % (index // 12, index % 12 + 1)


def check_period(value, where):
    if not isinstance(value, str) or not PERIOD_PATTERN.match(value):
        raise LedgerError("%s must be a YYYY-MM string, got %r" % (where, value))
    return value


def check_cents(value, where, allow_negative=False):
    if isinstance(value, bool) or not isinstance(value, int):
        raise LedgerError("%s must be integer cents, got %r" % (where, value))
    if value < 0 and not allow_negative:
        raise LedgerError("%s must not be negative, got %r" % (where, value))
    return value


def reject_duplicate_keys(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise LedgerError("JSON object repeats key %r" % key)
        obj[key] = value
    return obj


def reject_constant(name):
    raise LedgerError("JSON constant %s is not a valid amount" % name)


def load_json(path):
    try:
        with open(path, encoding="utf-8") as handle:
            return json.load(
                handle,
                object_pairs_hook=reject_duplicate_keys,
                parse_constant=reject_constant,
            )
    except json.JSONDecodeError as exc:
        raise LedgerError("%s is not valid JSON: %s" % (path, exc))
    except OSError as exc:
        raise LedgerError("cannot read %s: %s" % (path, exc))


def canonical(record):
    return json.dumps(record, sort_keys=True, separators=(",", ":"))


def unique_transactions(raw):
    """Collapse identical repeats; reject different records sharing an id."""
    if not isinstance(raw, list):
        raise LedgerError("transactions must be a list")
    by_id = {}
    for position, record in enumerate(raw):
        if not isinstance(record, dict):
            raise LedgerError("transactions[%d] must be an object" % position)
        record_id = record.get("id")
        if not isinstance(record_id, str) or not record_id:
            raise LedgerError("transactions[%d].id must be a non-empty string" % position)
        if record_id in by_id:
            if canonical(by_id[record_id]) != canonical(record):
                raise LedgerError(
                    "conflicting records share id %r (transactions[%d] differs "
                    "from the earlier record)" % (record_id, position)
                )
            continue
        by_id[record_id] = record
    return list(by_id.values())


def check_transaction(record):
    where = "transaction %r" % record["id"]
    kind = record.get("kind")
    if kind not in KINDS:
        raise LedgerError("%s has unknown kind %r" % (where, kind))
    check_cents(record.get("amount_cents"), where + " amount_cents")
    check_period(record.get("cash_period"), where + " cash_period")
    recognized = record.get("recognized_period")
    if kind == "financing_payment":
        # Principal has no operating recognition; any stated period is ignored.
        if recognized is not None:
            check_period(recognized, where + " recognized_period")
    else:
        check_period(recognized, where + " recognized_period")


def parse_ledger(data):
    if not isinstance(data, dict):
        raise LedgerError("ledger must be a JSON object")
    if data.get("currency") != "EUR":
        raise LedgerError("currency must be 'EUR', got %r" % (data.get("currency"),))

    periods = data.get("periods")
    if not isinstance(periods, list) or not periods:
        raise LedgerError("periods must be a non-empty list")
    for position, period in enumerate(periods):
        check_period(period, "periods[%d]" % position)
        if position and period != shift_period(periods[position - 1], 1):
            raise LedgerError(
                "periods must be consecutive calendar months; %r follows %r"
                % (period, periods[position - 1])
            )

    opening = check_cents(data.get("opening_cash_cents"), "opening_cash_cents", True)
    reserve = check_cents(data.get("minimum_reserve_cents"), "minimum_reserve_cents")
    transactions = unique_transactions(data.get("transactions"))
    for record in transactions:
        check_transaction(record)
    return periods, opening, reserve, transactions


def build_scenario(periods, opening, reserve, transactions, invoice_shift):
    totals = {
        period: {"revenue": 0, "cost": 0, "receipts": 0, "payments": 0}
        for period in periods
    }

    def add(period, field, amount):
        if period in totals:  # outside the horizon contributes nothing
            totals[period][field] += amount

    for record in transactions:
        kind = record["kind"]
        amount = record["amount_cents"]
        if kind == "invoice":
            add(record["recognized_period"], "revenue", amount)
            add(shift_period(record["cash_period"], invoice_shift), "receipts", amount)
        elif kind == "operating_cost":
            add(record["recognized_period"], "cost", amount)
            add(record["cash_period"], "payments", amount)
        else:
            add(record["cash_period"], "payments", amount)

    cash = opening
    minimum = opening
    first_negative = None
    months = []
    for period in periods:
        month = totals[period]
        cash += month["receipts"] - month["payments"]
        minimum = min(minimum, cash)
        if cash < 0 and first_negative is None:
            first_negative = period
        months.append({
            "period": period,
            "revenue_cents": month["revenue"],
            "operating_cost_cents": month["cost"],
            "profit_cents": month["revenue"] - month["cost"],
            "receipts_cents": month["receipts"],
            "payments_cents": month["payments"],
            "closing_cash_cents": cash,
        })

    return {
        "months": months,
        "first_negative_period": first_negative,
        "minimum_cash_cents": minimum,
        "financing_to_reserve_cents": max(0, reserve - minimum),
    }


def analyze(data):
    periods, opening, reserve, transactions = parse_ledger(data)
    return {
        "currency": "EUR",
        "scenarios": {
            name: build_scenario(periods, opening, reserve, transactions, shift)
            for name, shift in SCENARIO_INVOICE_SHIFTS
        },
    }


def write_atomically(path, payload):
    directory = os.path.dirname(os.path.abspath(path))
    descriptor, temporary = tempfile.mkstemp(prefix=".analyze-", suffix=".tmp", dir=directory)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
            json.dump(payload, handle, indent=2)
            handle.write("\n")
        os.replace(temporary, path)
    except BaseException:
        if os.path.exists(temporary):
            os.unlink(temporary)
        raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--input", required=True, help="ledger JSON to read")
    parser.add_argument("--output", required=True, help="results JSON to write")
    args = parser.parse_args(argv)

    try:
        if os.path.realpath(args.input) == os.path.realpath(args.output):
            raise LedgerError("--output must differ from --input; the ledger is never overwritten")
        results = analyze(load_json(args.input))
        write_atomically(args.output, results)
    except (LedgerError, OSError) as exc:
        print("error: %s" % exc, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
