"""Simple integer-cent ledger summary."""


def summarize(entries):
    charged = 0
    refunded = 0
    for entry in entries:
        if not isinstance(entry, dict):
            raise ValueError("entry must be an object")
        kind = entry.get("kind")
        cents = entry.get("cents")
        if kind not in ("charge", "refund") or type(cents) is not int or cents < 0:
            raise ValueError("invalid ledger entry")
        if kind == "charge":
            charged += cents
        else:
            refunded += cents
    return {"charged_cents": charged, "refunded_cents": refunded, "net_cents": charged + refunded}
