#!/usr/bin/env python3
"""Model recognized profit and cash timing from a ledger, using integer cents.

Usage: python3 analyze.py --input ledger.json --output results.json
Only the listed horizon months are included, in chronological order. Duplicate
records count once; conflicting IDs invalidate the whole input before any write.
"""

import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile


def require_integer(value, label):
    # bool is an int subclass, but is not a monetary amount.
    if type(value) is not int:
        raise ValueError(f"{label} must be an integer number of cents")


def require_period(value, label):
    if (
        not isinstance(value, str)
        or re.fullmatch(r"[0-9]{4}-(0[1-9]|1[0-2])", value) is None
        or value.startswith("0000-")
    ):
        raise ValueError(f"{label} must be a calendar month in YYYY-MM format")


def next_month(period):
    """Advance one calendar month, independently of the modeled horizon."""
    year, month = map(int, period.split("-"))
    if month == 12:
        year, month = year + 1, 1
    else:
        month += 1
    return f"{year:04d}-{month:02d}"


def validate_and_deduplicate(ledger):
    if not isinstance(ledger, dict) or ledger.get("currency") != "EUR":
        raise ValueError("ledger must be an object with currency EUR")
    for name in ("opening_cash_cents", "minimum_reserve_cents"):
        require_integer(ledger.get(name), name)

    periods = ledger.get("periods")
    if not isinstance(periods, list):
        raise ValueError("periods must be an array of calendar months")
    for period in periods:
        require_period(period, "periods entry")
    if len(set(periods)) != len(periods):
        raise ValueError("periods must not contain duplicate months")

    transactions = ledger.get("transactions")
    if not isinstance(transactions, list):
        raise ValueError("transactions must be an array")
    unique = {}
    signatures = {}
    for index, row in enumerate(transactions):
        label = f"transactions[{index}]"
        if not isinstance(row, dict):
            raise ValueError(f"{label} must be an object")
        identifier = row.get("id")
        if not isinstance(identifier, str) or not identifier:
            raise ValueError(f"{label}.id must be a nonempty string")

        # Compare the complete record, ignoring JSON object key order only.
        signature = json.dumps(row, sort_keys=True, allow_nan=False)
        if identifier in signatures:
            if signatures[identifier] != signature:
                raise ValueError(f"conflicting duplicate ID {identifier!r}")
            continue

        kind = row.get("kind")
        if kind not in ("invoice", "operating_cost", "financing_payment"):
            raise ValueError(f"{label}.kind is unsupported: {kind!r}")
        require_integer(row.get("amount_cents"), f"{label}.amount_cents")
        require_period(row.get("cash_period"), f"{label}.cash_period")
        recognized = row.get("recognized_period")
        if kind != "financing_payment" or recognized is not None:
            require_period(recognized, f"{label}.recognized_period")
        signatures[identifier] = signature
        unique[identifier] = row
    return sorted(periods), list(unique.values())


def scenario(periods, transactions, opening_cash, minimum_reserve, delayed):
    months = {
        period: {
            "period": period,
            "revenue_cents": 0,
            "operating_cost_cents": 0,
            "profit_cents": 0,
            "receipts_cents": 0,
            "payments_cents": 0,
            "closing_cash_cents": 0,
        }
        for period in periods
    }
    for row in transactions:
        kind, amount = row["kind"], row["amount_cents"]
        recognized = row.get("recognized_period")
        if kind in ("invoice", "operating_cost") and recognized in months:
            field = "revenue_cents" if kind == "invoice" else "operating_cost_cents"
            months[recognized][field] += amount

        cash_period = row["cash_period"]
        if delayed and kind == "invoice":
            cash_period = next_month(cash_period)
        if cash_period in months:
            field = "receipts_cents" if kind == "invoice" else "payments_cents"
            months[cash_period][field] += amount

    cash = minimum_cash = opening_cash
    first_negative = None
    for month in months.values():
        month["profit_cents"] = month["revenue_cents"] - month["operating_cost_cents"]
        cash += month["receipts_cents"] - month["payments_cents"]
        month["closing_cash_cents"] = cash
        minimum_cash = min(minimum_cash, cash)
        if cash < 0 and first_negative is None:
            first_negative = month["period"]

    return {
        "months": list(months.values()),
        "first_negative_period": first_negative,
        "minimum_cash_cents": minimum_cash,
        "financing_to_reserve_cents": max(0, minimum_reserve - minimum_cash),
    }


def analyze(ledger):
    periods, transactions = validate_and_deduplicate(ledger)
    return {
        "currency": "EUR",
        "scenarios": {
            name: scenario(
                periods,
                transactions,
                ledger["opening_cash_cents"],
                ledger["minimum_reserve_cents"],
                delayed,
            )
            for name, delayed in (("base", False), ("delayed", True))
        },
    }


def write_result(path, result):
    """Replace the destination only after a complete result is serialized."""
    content = json.dumps(result, indent=2, allow_nan=False) + "\n"
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=path.parent,
            prefix=f".{path.name}.", suffix=".tmp", delete=False,
        ) as handle:
            temporary = Path(handle.name)
            handle.write(content)
        os.replace(temporary, path)
    finally:
        if temporary is not None and temporary.exists():
            temporary.unlink()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        if args.input.resolve() == args.output.resolve() or (
            args.output.exists() and args.input.samefile(args.output)
        ):
            raise ValueError("input and output must be different files")
        with args.input.open(encoding="utf-8") as handle:
            ledger = json.load(handle)
        result = analyze(ledger)
        write_result(args.output, result)
    except (OSError, ValueError) as error:
        print(f"analyze.py: error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
