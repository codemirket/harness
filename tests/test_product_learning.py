"""Adversarial checks for the project/product/marketing learning contracts."""

from __future__ import annotations

import hashlib
import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
# This is repository qualification tooling, not an installed skill resource.
SCRIPT = ROOT / "scripts" / "check-product-learning.py"
SPEC = importlib.util.spec_from_file_location("check_product_learning", SCRIPT)
CHECKER = importlib.util.module_from_spec(SPEC)
assert SPEC and SPEC.loader
SPEC.loader.exec_module(CHECKER)


def evidence_register() -> dict:
    return {
        "schema_version": 1,
        "evidence": [
            {
                "id": "real-quote",
                "type": "real",
                "kind": "participant_quote",
                "segment": "independent teams",
                "observation": "A participant used the saved preview before sharing it.",
                "source": {
                    "locator": "interview://2026-10-01#p3",
                    "observed_at": "2026-10-01",
                    "consent": True,
                    "quote": "I can see the result before I send it.",
                },
            },
            {
                "id": "synthetic-fixture",
                "type": "synthetic",
                "kind": "scenario",
                "segment": "fixture user",
                "observation": "The fixture starts a preview.",
                "source": {"locator": "fixture://preview-start", "observed_at": "2026-10-02"},
            },
            {
                "id": "inferred-need",
                "type": "inferred",
                "segment": "independent teams",
                "observation": "Preview confidence may reduce hesitation.",
                "source": {"locator": "analysis://preview-confidence", "observed_at": "2026-10-03"},
                "derived_from": ["real-quote", "synthetic-fixture"],
            },
            {
                "id": "current-alt",
                "type": "real",
                "kind": "win_loss",
                "segment": "independent teams",
                "observation": "The buyer compared a manual export workflow.",
                "source": {"locator": "winloss://2026-10-04#p1", "observed_at": "2026-10-04"},
            },
            {
                "id": "real-metric",
                "type": "real",
                "kind": "metric",
                "segment": "independent teams",
                "observation": "The measured preview completion rate was 98% during the dated window.",
                "source": {"locator": "analytics://preview-completion/2026-09", "observed_at": "2026-09-30"},
            },
        ],
        "contradictions": [
            {
                "id": "c1",
                "evidence_refs": ["real-quote", "synthetic-fixture"],
                "status": "open",
                "description": "Observed participant confidence differs from the fixture assumption.",
            }
        ],
    }


class ProductLearningTests(unittest.TestCase):
    def assert_invalid(self, function, *args) -> None:
        with self.assertRaises(ValueError):
            function(*args)

    def test_evidence_and_decision_map_preserve_types_contradictions_and_rejections(self) -> None:
        register = evidence_register()
        self.assertTrue(CHECKER.validate_evidence_register(register))
        packet = {
            "schema_version": 1,
            "outcome": "teams confidently share a preview",
            "opportunities": [{"id": "o1", "statement": "Preview confidence is uncertain", "evidence_refs": ["real-quote"]}],
            "assumptions": [{"id": "a1", "opportunity_id": "o1", "statement": "An early preview reduces hesitation", "evidence_refs": ["inferred-need"]}],
            "tests": [{"id": "t1", "assumption_id": "a1", "method": "observe authorized preview task", "decision_rule": "continue if the task completes without a second export", "evidence_refs": ["inferred-need"]}],
            "decisions": [{"id": "d1", "test_id": "t1", "result": "inconclusive", "choice": "run one authorized follow-up", "evidence_refs": ["real-quote"]}],
            "rejected_hypotheses": [{"hypothesis": "The fixture is customer prevalence", "reason": "It is synthetic", "evidence_refs": ["synthetic-fixture"]}],
        }
        self.assertTrue(CHECKER.validate_opportunity_map(packet, {item["id"] for item in register["evidence"]}))
        packet["opportunities"][0]["evidence_refs"] = []
        self.assert_invalid(CHECKER.validate_opportunity_map, packet)
        packet["opportunities"][0]["evidence_refs"] = ["real-quote"]
        packet["assumptions"][0]["evidence_refs"] = []
        packet["assumptions"][0]["status"] = "unknown"
        self.assertTrue(CHECKER.validate_opportunity_map(packet, {item["id"] for item in register["evidence"]}))

    def test_unsupported_testimonial_and_expired_metric_fail_closed(self) -> None:
        register = evidence_register()
        register["evidence"].append({
            "id": "synthetic-metric", "type": "synthetic", "kind": "metric", "segment": "fixture user",
            "observation": "A fabricated completion rate.", "source": {"locator": "fixture://metric", "observed_at": "2026-10-02"},
        })
        saved = "Teams trust this preview. 98% of teams finish."
        digest = hashlib.sha256(saved.encode()).hexdigest()
        review = {
            "schema_version": 1,
            "as_of": "2026-10-28",
            "saved_copy_sha256": digest,
            "claims": [
                {"id": "testimonial", "text": "Teams trust this preview.", "kind": "testimonial", "status": "supported", "proof_refs": ["synthetic-fixture"]},
                {"id": "metric", "text": "98% of teams finish.", "kind": "metric", "status": "supported", "proof_refs": ["synthetic-metric"], "valid_until": "2026-10-01"},
            ],
        }
        self.assert_invalid(CHECKER.validate_copy_review, review, saved, register)
        review["claims"][0]["proof_refs"] = []
        self.assert_invalid(CHECKER.validate_copy_review, review, saved, register)
        review["claims"][0]["proof_refs"] = ["real-quote"]
        review["claims"][1]["proof_refs"] = ["real-metric"]
        self.assert_invalid(CHECKER.validate_copy_review, review, saved, register)
        review["claims"][1]["revalidated_at"] = "2026-10-28"
        self.assertTrue(CHECKER.validate_copy_review(review, saved, register))

    def test_future_competitor_is_not_smuggled_into_current_positioning(self) -> None:
        register = evidence_register()
        positioning = {
            "schema_version": 1,
            "current": {
                "audience": "independent teams",
                "alternatives": [{"name": "manual export", "status": "current", "buyer_evidence_refs": ["current-alt"]}],
            },
            "future_point_of_view": {
                "point_of_view": "Preview becomes a trust boundary for agentic work.",
                "competitors": [{"name": "future autonomous studio", "status": "future", "evidence_refs": []}],
            },
        }
        self.assertTrue(CHECKER.validate_positioning(positioning, {item["id"] for item in register["evidence"]}))
        positioning["current"]["alternatives"].append({"name": "future autonomous studio", "status": "current", "buyer_evidence_refs": []})
        self.assert_invalid(CHECKER.validate_positioning, positioning, {item["id"] for item in register["evidence"]})

    def test_bottleneck_selects_activation_without_claiming_causal_lift(self) -> None:
        diagnosis = {
            "schema_version": 1,
            "stages": [
                {"id": "acquisition", "eligible": 1000, "converted": 500, "rate": 0.5},
                {"id": "activation", "eligible": 500, "converted": 25, "rate": 0.05},
                {"id": "retention", "eligible": 25, "converted": 2, "rate": 0.08},
            ],
            "bottleneck_stage": "activation",
            "causal_status": "unproven",
        }
        self.assertTrue(CHECKER.validate_funnel_diagnosis(diagnosis))
        diagnosis["stages"][0]["rate"] = 0.6
        self.assert_invalid(CHECKER.validate_funnel_diagnosis, diagnosis)
        diagnosis["stages"][0]["rate"] = 0.5
        diagnosis["bottleneck_stage"] = "acquisition"
        self.assert_invalid(CHECKER.validate_funnel_diagnosis, diagnosis)
        diagnosis["bottleneck_stage"] = "activation"
        diagnosis["stages"].append({"id": "activation", "eligible": 1, "converted": 1, "rate": 1.0})
        self.assert_invalid(CHECKER.validate_funnel_diagnosis, diagnosis)
        diagnosis["stages"].pop()
        diagnosis["stages"][0]["eligible"] = True
        self.assert_invalid(CHECKER.validate_funnel_diagnosis, diagnosis)

    def test_repeated_review_cannot_duplicate_external_action(self) -> None:
        previous = {
            "schema_version": 1,
            "review_id": "r1",
            "input_digest": "same-input",
            "actions": [{"intended_action_id": "send-copy", "idempotency_key": "send-copy:1", "mode": "executed", "external": True, "approved": True}],
        }
        repeated = {
            "schema_version": 1,
            "review_id": "r2",
            "input_digest": "same-input",
            "actions": [{"intended_action_id": "send-copy", "idempotency_key": "send-copy:2", "mode": "executed", "external": True, "approved": True}],
        }
        self.assert_invalid(CHECKER.validate_loop_run, repeated, [previous])
        repeated["actions"] = [{"intended_action_id": "send-copy", "idempotency_key": "send-copy:2", "mode": "skipped_duplicate", "external": True, "approved": False}]
        self.assertTrue(CHECKER.validate_loop_run(repeated, [previous]))

    def test_empty_required_references_and_inferred_cycles_fail(self) -> None:
        register = evidence_register()
        register["evidence"][2]["derived_from"] = []
        self.assert_invalid(CHECKER.validate_evidence_register, register)
        register = evidence_register()
        register["evidence"][2]["derived_from"] = ["inferred-cycle"]
        register["evidence"].append({
            "id": "inferred-cycle", "type": "inferred", "segment": "independent teams",
            "observation": "Cycle fixture", "source": {"locator": "analysis://cycle", "observed_at": "2026-10-03"},
            "derived_from": ["inferred-need"],
        })
        self.assert_invalid(CHECKER.validate_evidence_register, register)

        positioning = {
            "schema_version": 1,
            "current": {"audience": "teams", "alternatives": [{"name": "manual", "status": "current", "buyer_evidence_refs": []}]},
            "future_point_of_view": {"point_of_view": "future", "competitors": []},
        }
        self.assert_invalid(CHECKER.validate_positioning, positioning)


if __name__ == "__main__":
    unittest.main()
