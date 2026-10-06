"""Filter a local CSV and emit matching records as JSON lines."""

import argparse
import csv
import json
import sys


def positive_int(raw):
    value = int(raw)
    if value < 1:
        raise argparse.ArgumentTypeError("limit must be at least 1")
    return value


def main(argv=None):
    parser = argparse.ArgumentParser(description="Find records by name or city")
    parser.add_argument("csv_file", help="UTF-8 CSV with id,name,city columns")
    parser.add_argument("--contains", required=True, help="case-insensitive substring")
    parser.add_argument("--limit", type=positive_int, default=10, help="maximum matches (default: 10)")
    args = parser.parse_args(argv)
    try:
        with open(args.csv_file, newline="", encoding="utf-8") as handle:
            rows = csv.DictReader(handle)
            if not {"id", "name", "city"}.issubset(rows.fieldnames or []):
                raise ValueError("CSV needs id,name,city columns")
            count = 0
            for row in rows:
                haystack = f"{row['name']} {row['city']}".casefold()
                if args.contains.casefold() in haystack:
                    print(json.dumps({key: row[key] for key in ("id", "name", "city")}, ensure_ascii=False))
                    count += 1
                    if count >= args.limit:
                        break
    except (OSError, ValueError) as exc:
        print(f"mini_lookup: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
