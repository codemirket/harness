#!/usr/bin/env python3
"""Regression tests for analyze.py, results.json and decision.md.

Run from this directory: python3 -m unittest -v test_analyze

The program is exercised through its command line. Expected values are
hand-calculated literals, or are recalculated here without importing analyze.py.
"""
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ANALYZE = os.path.join(HERE, "analyze.py")
LEDGER = os.path.join(HERE, "ledger.json")
RESULTS = os.path.join(HERE, "results.json")
DECISION = os.path.join(HERE, "decision.md")
LEDGER_SHA256 = "5be1e094f76fde050cd4c8a0c0648f2fe296009b04bfc10720c04e276a967eeb"


def tx(tid, kind, amount, cash, recognized=None):
    return {"id": tid, "kind": kind, "recognized_period": recognized,
            "cash_period": cash, "amount_cents": amount}


def ledger(periods, transactions, opening=0, reserve=0):
    return {"currency": "EUR", "periods": periods, "opening_cash_cents": opening,
            "minimum_reserve_cents": reserve, "transactions": transactions}


def month(period, revenue, cost, profit, receipts, payments, closing):
    return {"period": period, "revenue_cents": revenue, "operating_cost_cents": cost,
            "profit_cents": profit, "receipts_cents": receipts,
            "payments_cents": payments, "closing_cash_cents": closing}


def load(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


class CliCase(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.output = os.path.join(self.tmp.name, "results.json")

    def run_cli(self, ledger_data=None, input_path=None, output=None):
        if input_path is None:
            input_path = os.path.join(self.tmp.name, "ledger.json")
            with open(input_path, "w", encoding="utf-8") as handle:
                if isinstance(ledger_data, str):
                    handle.write(ledger_data)
                else:
                    json.dump(ledger_data, handle)
        return subprocess.run(
            [sys.executable, ANALYZE, "--input", input_path, "--output", output or self.output],
            capture_output=True, text=True, check=False)

    def assert_ok(self, proc):
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return load(self.output)

    def assert_rejected(self, proc):
        self.assertNotEqual(proc.returncode, 0)
        self.assertFalse(os.path.exists(self.output), "rejected input must not write output")


class SuppliedLedgerTest(CliCase):
    EXPECTED = {
        "currency": "EUR",
        "scenarios": {
            "base": {
                "months": [
                    month("2026-01", 5000000, 3000000, 2000000, 0, 3000000, 3000000),
                    month("2026-02", 5000000, 3000000, 2000000, 5000000, 10000000, -2000000),
                    month("2026-03", 5000000, 3000000, 2000000, 5000000, 3000000, 0),
                ],
                "first_negative_period": "2026-02",
                "minimum_cash_cents": -2000000,
                "financing_to_reserve_cents": 3000000,
            },
            "delayed": {
                "months": [
                    month("2026-01", 5000000, 3000000, 2000000, 0, 3000000, 3000000),
                    month("2026-02", 5000000, 3000000, 2000000, 0, 10000000, -7000000),
                    month("2026-03", 5000000, 3000000, 2000000, 5000000, 3000000, -5000000),
                ],
                "first_negative_period": "2026-02",
                "minimum_cash_cents": -7000000,
                "financing_to_reserve_cents": 8000000,
            },
        },
    }

    def test_ledger_is_unchanged(self):
        with open(LEDGER, "rb") as handle:
            self.assertEqual(hashlib.sha256(handle.read()).hexdigest(), LEDGER_SHA256)

    def test_matches_hand_calculation(self):
        self.assertEqual(self.assert_ok(self.run_cli(input_path=LEDGER)), self.EXPECTED)

    def test_saved_results_are_current(self):
        self.assertEqual(load(RESULTS), self.assert_ok(self.run_cli(input_path=LEDGER)))

    def test_cash_rolls_forward_from_opening(self):
        results = self.assert_ok(self.run_cli(input_path=LEDGER))
        for name, outcome in results["scenarios"].items():
            cash = load(LEDGER)["opening_cash_cents"]
            for row in outcome["months"]:
                cash += row["receipts_cents"] - row["payments_cents"]
                self.assertEqual(row["closing_cash_cents"], cash, (name, row["period"]))

    def test_output_cannot_overwrite_input(self):
        proc = self.run_cli(input_path=LEDGER, output=LEDGER)
        self.assertNotEqual(proc.returncode, 0)
        self.test_ledger_is_unchanged()


class HorizonTest(CliCase):
    def test_horizon_may_omit_a_month(self):
        # January and March only. Every February flow is excluded, receipts that
        # the delay moves into February or April are dropped, and cash rolls
        # straight from January to March.
        data = ledger(["2026-01", "2026-03"], [
            tx("inv-a", "invoice", 100, "2026-02", "2026-01"),
            tx("inv-b", "invoice", 200, "2026-03", "2026-02"),
            tx("inv-c", "invoice", 40, "2026-01", "2026-01"),
            tx("cost-a", "operating_cost", 50, "2026-02", "2026-02"),
            tx("cost-b", "operating_cost", 30, "2026-01", "2026-03"),
            tx("loan-feb", "financing_payment", 1000, "2026-02"),
            tx("loan-mar", "financing_payment", 200, "2026-03"),
        ], opening=100, reserve=50)
        proc = self.run_cli(data)
        self.assertEqual(self.assert_ok(proc), {"currency": "EUR", "scenarios": {
            "base": {
                "months": [month("2026-01", 140, 0, 140, 40, 30, 110),
                           month("2026-03", 0, 30, -30, 200, 200, 110)],
                "first_negative_period": None,
                "minimum_cash_cents": 100,
                "financing_to_reserve_cents": 0,
            },
            "delayed": {
                "months": [month("2026-01", 140, 0, 140, 0, 30, 70),
                           month("2026-03", 0, 30, -30, 100, 200, -30)],
                "first_negative_period": "2026-03",
                "minimum_cash_cents": -30,
                "financing_to_reserve_cents": 80,
            },
        }})
        self.assertIn("2026-02", proc.stderr, "an omitted month must be reported, not silent")

    def test_consecutive_horizon_reports_no_gap(self):
        proc = self.run_cli(ledger(["2026-01", "2026-02"], []))
        self.assert_ok(proc)
        self.assertEqual(proc.stderr, "")

    def test_periods_out_of_order_or_repeated_are_rejected(self):
        for periods in (["2026-03", "2026-01"], ["2026-01", "2026-01"]):
            with self.subTest(periods=periods):
                self.assert_rejected(self.run_cli(ledger(periods, [])))

    def test_delayed_december_receipt_lands_in_next_january(self):
        data = ledger(["2026-12", "2027-01"], [tx("inv", "invoice", 500, "2026-12", "2026-12")])
        scenarios = self.assert_ok(self.run_cli(data))["scenarios"]
        self.assertEqual([m["receipts_cents"] for m in scenarios["base"]["months"]], [500, 0])
        self.assertEqual([m["receipts_cents"] for m in scenarios["delayed"]["months"]], [0, 500])


class DuplicateTest(CliCase):
    def test_conflicting_duplicate_writes_nothing(self):
        data = load(LEDGER)
        data["transactions"].append(dict(data["transactions"][0], amount_cents=1))
        self.assert_rejected(self.run_cli(data))

        with open(self.output, "w", encoding="utf-8") as handle:
            handle.write("previous output\n")
        self.assertNotEqual(self.run_cli(data).returncode, 0)
        with open(self.output, encoding="utf-8") as handle:
            self.assertEqual(handle.read(), "previous output\n")

    def test_identical_repeat_counts_once(self):
        repeated = self.assert_ok(self.run_cli(input_path=LEDGER))
        data = load(LEDGER)
        ids = [t["id"] for t in data["transactions"]]
        self.assertEqual(len(ids) - len(set(ids)), 1, "fixture should hold one identical repeat")
        data["transactions"] = [t for i, t in enumerate(data["transactions"]) if t["id"] not in ids[:i]]
        self.assertEqual(self.assert_ok(self.run_cli(data)), repeated)


class MalformedInputTest(CliCase):
    def test_rejected_without_output(self):
        good = tx("inv", "invoice", 100, "2026-01", "2026-01")
        cases = {
            "decimal cents": ledger(["2026-01"], [dict(good, amount_cents=100.5)]),
            "unknown kind": ledger(["2026-01"], [dict(good, kind="dividend")]),
            "other currency": dict(ledger(["2026-01"], [good]), currency="USD"),
            "repeated JSON key": '{"currency": "EUR", "currency": "EUR", "periods": ["2026-01"],'
                                 ' "opening_cash_cents": 0, "minimum_reserve_cents": 0, "transactions": []}',
        }
        for label, data in cases.items():
            with self.subTest(label):
                self.assert_rejected(self.run_cli(data))


# ---- decision.md ----------------------------------------------------------

def euros(cell):
    """'**−€20,000**' -> -20000."""
    text = cell.replace("*", "").replace("−", "-").replace("€", "").replace(",", "").strip()
    return int(text)


def section(markdown, heading):
    """Text under `heading` up to the next heading of the same or a higher level."""
    level = len(heading) - len(heading.lstrip("#"))
    start = markdown.index(heading + "\n") + len(heading) + 1
    following = re.compile(r"^#{1,%d} " % level, re.M).search(markdown, start)
    return markdown[start:following.start() if following else len(markdown)]


def table(text):
    """Body rows of the markdown tables in `text`, as lists of stripped cells."""
    rows = [line.strip().strip("|").split("|") for line in text.splitlines() if line.strip().startswith("|")]
    return [[c.strip() for c in row] for row in rows if not set("".join(row)) <= set("-: ")]


def units(markdown):
    """Table rows, plus sentences of other text with wrapped lines joined and
    emphasis removed, so a bold sentence is not merged with the next one."""
    rows, blocks, current = [], [], []
    for line in markdown.replace("*", "").splitlines():
        stripped = line.strip()
        if stripped.startswith("|"):
            rows.append(stripped)
        if (not stripped or stripped.startswith(("|", "#", "- "))
                or re.match(r"\d+\. ", stripped)) and current:
            blocks.append(" ".join(current))
            current = []
        if stripped and not stripped.startswith(("|", "#")):
            current.append(stripped)
    if current:
        blocks.append(" ".join(current))
    return rows + [s for block in blocks for s in re.split(r"(?<=[.!?])\s+", block)]


NEGATED = re.compile(r"\b(no|not|never|cannot)\b|too late", re.I)


def timing_rows(ledger_data, receipt_shift):
    """Per month: (period, opening, payments, low before receipts, receipts, closing)
    in cents, assuming each month's payments all precede its receipts."""
    unique = {}
    for t in ledger_data["transactions"]:
        unique.setdefault(t["id"], t)
    periods = ledger_data["periods"]
    payments, receipts = dict.fromkeys(periods, 0), dict.fromkeys(periods, 0)
    for t in unique.values():
        if t["kind"] == "invoice":
            year, mon = map(int, t["cash_period"].split("-"))
            mon += receipt_shift
            received = f"{year + (mon - 1) // 12:04d}-{(mon - 1) % 12 + 1:02d}"
            if received in receipts:
                receipts[received] += t["amount_cents"]
        elif t["cash_period"] in payments:
            payments[t["cash_period"]] += t["amount_cents"]
    rows, cash = [], ledger_data["opening_cash_cents"]
    for p in periods:
        low = cash - payments[p]
        rows.append((p, cash, payments[p], low, receipts[p], low + receipts[p]))
        cash = low + receipts[p]
    return rows


def timing_bound_need(ledger_data, receipt_shift):
    lows = [row[3] for row in timing_rows(ledger_data, receipt_shift)]
    return max(0, ledger_data["minimum_reserve_cents"] - min([ledger_data["opening_cash_cents"]] + lows))


class DecisionReportTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with open(DECISION, encoding="utf-8") as handle:
            cls.doc = handle.read()
        cls.results = load(RESULTS)["scenarios"]
        cls.ledger = load(LEDGER)

    def key_figures(self):
        rows = table(section(self.doc, "## Key figures").split("### Monthly detail")[0])
        return {row[0]: row[1:] for row in rows[1:]}

    def test_timing_bound_recalculates_from_ledger(self):
        # Hand check: base 30,000 - 100,000 = -70,000 in February; delayed
        # -70,000 - 30,000 = -100,000 in March. Plus the 10,000 reserve.
        self.assertEqual(timing_bound_need(self.ledger, 0), 8000000)
        self.assertEqual(timing_bound_need(self.ledger, 1), 11000000)

    def test_key_figures_match_results(self):
        figures = self.key_figures()
        for col, name in enumerate(("base", "delayed")):
            outcome = self.results[name]
            months = outcome["months"]
            with self.subTest(name):
                for label, key in (("Revenue, Jan–Mar", "revenue_cents"),
                                   ("Operating costs, Jan–Mar", "operating_cost_cents"),
                                   ("Profit, Jan–Mar", "profit_cents"),
                                   ("Receipts, Jan–Mar", "receipts_cents"),
                                   ("Payments, Jan–Mar (operating costs + principal)", "payments_cents")):
                    self.assertEqual(euros(figures[label][col]) * 100, sum(m[key] for m in months), label)
                self.assertEqual(euros(figures["Closing cash, end of March"][col]) * 100,
                                 months[-1]["closing_cash_cents"])
                self.assertEqual(figures["First month with negative cash"][col], outcome["first_negative_period"])
                self.assertEqual(euros(figures["Minimum cash (opening and month-ends)"][col]) * 100,
                                 outcome["minimum_cash_cents"])
                # The results.json amount is labeled as a month-end figure, and
                # the within-month bound is reported beside it.
                self.assertEqual(euros(figures["Funding to hold the €10,000 reserve at month-ends"][col]) * 100,
                                 outcome["financing_to_reserve_cents"])
                self.assertEqual(
                    euros(figures["Funding to hold the reserve if each month's payments precede its receipts"][col]) * 100,
                    timing_bound_need(self.ledger, col))

    def test_monthly_detail_matches_results(self):
        rows = table(section(self.doc, "### Monthly detail"))[1:]
        base, delayed = (self.results[n]["months"] for n in ("base", "delayed"))
        self.assertEqual([r[0] for r in rows], [m["period"] for m in base])
        for row, b, d in zip(rows, base, delayed):
            values = [euros(c) * 100 for c in row[1:]]
            self.assertEqual(values, [b["revenue_cents"], b["operating_cost_cents"], b["profit_cents"],
                                      b["receipts_cents"], b["payments_cents"], b["closing_cash_cents"],
                                      d["receipts_cents"], d["closing_cash_cents"]], row[0])
            self.assertEqual(d["payments_cents"], b["payments_cents"])

    def test_timing_table_matches_recalculation(self):
        funding = section(self.doc, "## Funding amount and deadline")
        tables = table(funding)
        monthly = [r for r in tables if r[0] in ("Base", "Delayed")]
        expected = [(name, p, *cents) for shift, name in enumerate(("Base", "Delayed"))
                    for p, *cents in timing_rows(self.ledger, shift)]
        self.assertEqual([(r[0], r[1], *[euros(c) * 100 for c in r[2:]]) for r in monthly], expected)
        needs = {r[0]: r[1:] for r in tables if r[0].startswith(("At month-ends", "Timing bound"))}
        self.assertEqual([euros(c) * 100 for c in needs["At month-ends only (`results.json`)"]],
                         [self.results[n]["financing_to_reserve_cents"] for n in ("base", "delayed")])
        self.assertEqual([euros(c) * 100 for c in needs["Timing bound (lowest point above + €10,000)"]],
                         [timing_bound_need(self.ledger, s) for s in (0, 1)])

    def test_no_end_of_february_arrival_called_sufficient(self):
        for unit in units(self.doc):
            if re.search(r"end[ -]of[ -]February", unit) and re.search(
                    r"\b(sufficient|enough|cover(s|ed)?|holds?|held)\b", unit):
                self.assertRegex(unit, NEGATED, "end-of-February funding presented as sufficient")

    def test_funding_deadline_precedes_first_february_payment(self):
        plain = self.doc.replace("*", "")
        self.assertIn("before the first February payment", plain.split("## Key figures")[0])
        changes = table(section(self.doc, "## What would change the decision"))[1:]
        funding_rows = [r for r in changes if re.match(r"(At least )?€[\d,]+ ", r[0])]
        self.assertGreaterEqual(len(funding_rows), 3)
        for row in funding_rows:
            self.assertIn("before the first February payment", row[0])

    def test_date_level_evidence_is_named(self):
        funding = section(self.doc, "## Funding amount and deadline")
        evidence = funding.split("Date-level evidence still needed")[1]
        for needed in ("due date", "payment dates", "receipt dates", "bank balance", "lead time"):
            self.assertIn(needed, evidence)

    def test_weekly_forecast_is_a_proposal_not_a_check(self):
        heading = "### Proposed operational follow-up (not performed or validated here)"
        proposed = section(self.doc, heading) if heading in self.doc else ""
        # The limits section may name a weekly forecast as something it cannot establish.
        limits = section(self.doc, "## What this monthly horizon cannot establish")
        for unit in units(self.doc.replace(proposed, "").replace(limits, "")):
            if "weekly" in unit.lower() or "against this model" in unit:
                self.assertRegex(unit, NEGATED, "weekly forecasting presented outside the proposal")
        self.assertIn("weekly", proposed)
        self.assertIn("### Checked by this artifact", self.doc)

    def test_gap_horizon_is_not_called_malformed(self):
        self.assertNotIn("non-consecutive", self.doc)
        checks = section(self.doc, "## Source, basis and reproduction")
        self.assertIn("A January/March horizon is accepted", checks)


if __name__ == "__main__":
    unittest.main()
