import importlib.util
import json
import sys
from pathlib import Path


def main(root):
    checks = []

    def check(name, passed, detail):
        checks.append({"id": name, "status": "passed" if passed else "failed", "detail": detail})

    source = root / "ledger.py"
    if not source.is_file():
        return [{"id": "module", "status": "failed", "detail": "ledger.py missing"}]
    try:
        spec = importlib.util.spec_from_file_location("case_ledger", source)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        summarize = module.summarize
    except Exception as exc:
        return [{"id": "module", "status": "failed", "detail": f"Import failed: {type(exc).__name__}: {str(exc)[:100]}"}]

    samples = [
        ([], {"charged_cents": 0, "refunded_cents": 0, "net_cents": 0}),
        ([{"kind": "charge", "cents": 107}, {"kind": "charge", "cents": 42}], {"charged_cents": 149, "refunded_cents": 0, "net_cents": 149}),
        ([{"kind": "refund", "cents": 65}], {"charged_cents": 0, "refunded_cents": 65, "net_cents": -65}),
        ([{"kind": "charge", "cents": 1003}, {"kind": "refund", "cents": 212}, {"kind": "refund", "cents": 5}], {"charged_cents": 1003, "refunded_cents": 217, "net_cents": 786}),
    ]
    for index, (entries, expected) in enumerate(samples):
        try:
            actual = summarize(entries)
            check(f"totals-{index}", actual == expected, f"Case {index}: observed {str(actual)[:100]}")
        except Exception as exc:
            check(f"totals-{index}", False, f"Case {index} raised {type(exc).__name__}")
    invalid = [[{"kind": "charge", "cents": -1}], [{"kind": "other", "cents": 4}], [{"kind": "refund", "cents": 1.5}], [{"kind": "charge", "cents": True}]]
    okay = True
    for entries in invalid:
        try:
            summarize(entries)
            okay = False
        except ValueError:
            pass
        except Exception:
            okay = False
    check("invalid-input", okay, "Invalid kind, negative, fractional, and boolean cents raise ValueError")
    return checks


if __name__ == "__main__":
    try:
        result = main(Path(sys.argv[1]).resolve()) if len(sys.argv) == 2 else [{"id": "usage", "status": "failed", "detail": "Expected workspace path"}]
    except Exception as exc:
        result = [{"id": "verifier-error", "status": "failed", "detail": f"{type(exc).__name__}: {str(exc)[:120]}"}]
    print(json.dumps({"checks": result}, separators=(",", ":")))
    sys.exit(0 if all(item["status"] == "passed" for item in result) else 1)
