#!/usr/bin/env python3
"""Model recognized operating results and monthly cash in integer EUR cents."""

import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile


def require_cents(value, field):
    if type(value) is not int:
        raise ValueError("{} must be integer cents".format(field))
    return value


def require_period(value, field):
    if not isinstance(value, str) or not re.fullmatch(r"\d{4}-(0[1-9]|1[0-2])", value):
        raise ValueError("{} must be a calendar month YYYY-MM".format(field))
    if int(value[:4]) == 0:
        raise ValueError("{} must have a positive year".format(field))
    return value


def next_month(period):
    year, month = map(int, period.split("-"))
    if month == 12:
        year, month = year + 1, 1
    else:
        month += 1
    return "{:04d}-{:02d}".format(year, month)


def validate_and_deduplicate(ledger):
    if not isinstance(ledger, dict):
        raise ValueError("input must be an object")
    if ledger.get("currency") != "EUR":
        raise ValueError("currency must be EUR")
    periods = ledger.get("periods")
    if not isinstance(periods, list) or not periods:
        raise ValueError("periods must be a nonempty list")
    for period in periods:
        require_period(period, "periods entry")
    if len(set(periods)) != len(periods) or periods != sorted(periods):
        raise ValueError("periods must be unique and in chronological order")
    require_cents(ledger.get("opening_cash_cents"), "opening_cash_cents")
    require_cents(ledger.get("minimum_reserve_cents"), "minimum_reserve_cents")
    records = ledger.get("transactions")
    if not isinstance(records, list):
        raise ValueError("transactions must be a list")

    unique = {}
    for record in records:
        if not isinstance(record, dict):
            raise ValueError("each transaction must be an object")
        identifier = record.get("id")
        if not isinstance(identifier, str) or not identifier:
            raise ValueError("transaction id must be a nonempty string")
        kind = record.get("kind")
        if kind not in ("invoice", "operating_cost", "financing_payment"):
            raise ValueError("unsupported kind for {}".format(identifier))
        require_cents(record.get("amount_cents"), identifier + ".amount_cents")
        require_period(record.get("cash_period"), identifier + ".cash_period")
        recognized = record.get("recognized_period")
        if kind in ("invoice", "operating_cost") or recognized is not None:
            require_period(recognized, identifier + ".recognized_period")
        if identifier in unique and record != unique[identifier]:
            raise ValueError("conflicting duplicate id: {}".format(identifier))
        unique[identifier] = record
    return list(unique.values())


def analyze(ledger):
    records = validate_and_deduplicate(ledger)
    opening_cash = ledger["opening_cash_cents"]
    scenarios = {}
    for scenario_name, delayed in (("base", False), ("delayed", True)):
        months = [
            {
                "period": period,
                "revenue_cents": 0,
                "operating_cost_cents": 0,
                "profit_cents": 0,
                "receipts_cents": 0,
                "payments_cents": 0,
                "closing_cash_cents": 0,
            }
            for period in ledger["periods"]
        ]
        by_period = {month["period"]: month for month in months}
        for record in records:
            kind = record["kind"]
            amount = record["amount_cents"]
            recognized = by_period.get(record.get("recognized_period"))
            if recognized is not None and kind in ("invoice", "operating_cost"):
                field = "revenue_cents" if kind == "invoice" else "operating_cost_cents"
                recognized[field] += amount
            cash_period = record["cash_period"]
            if delayed and kind == "invoice":
                cash_period = next_month(cash_period)
            cash_month = by_period.get(cash_period)
            if cash_month is not None:
                field = "receipts_cents" if kind == "invoice" else "payments_cents"
                cash_month[field] += amount

        balance = opening_cash
        minimum_cash = opening_cash
        first_negative = None
        for month in months:
            month["profit_cents"] = month["revenue_cents"] - month["operating_cost_cents"]
            balance += month["receipts_cents"] - month["payments_cents"]
            month["closing_cash_cents"] = balance
            minimum_cash = min(minimum_cash, balance)
            if balance < 0 and first_negative is None:
                first_negative = month["period"]
        scenarios[scenario_name] = {
            "months": months,
            "first_negative_period": first_negative,
            "minimum_cash_cents": minimum_cash,
            "financing_to_reserve_cents": max(
                0, ledger["minimum_reserve_cents"] - minimum_cash
            ),
        }
    return {"currency": ledger["currency"], "scenarios": scenarios}


def write_atomic(path, result):
    """Replace the destination only after the complete result has been serialized."""
    serialized = json.dumps(result, indent=2, ensure_ascii=False) + "\n"
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=path.parent,
            prefix="." + path.name + ".", suffix=".tmp", delete=False
        ) as handle:
            temporary = Path(handle.name)
            handle.write(serialized)
        os.replace(str(temporary), str(path))
    finally:
        if temporary is not None and temporary.exists():
            temporary.unlink()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.input.resolve() == args.output.resolve():
            raise ValueError("input and output must be different paths")
        with args.input.open(encoding="utf-8") as handle:
            ledger = json.load(handle)
        result = analyze(ledger)
        write_atomic(args.output, result)
    except (OSError, ValueError, TypeError) as error:
        print("error: {}".format(error), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
