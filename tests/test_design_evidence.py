#!/usr/bin/env python3
"""Adversarial tests for the dependency-free design evidence validator."""

from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/check-design-evidence.py"
SPEC = importlib.util.spec_from_file_location("check_design_evidence", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class DesignEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / "review"
        self.root.mkdir()
        (self.root / "src").mkdir()
        (self.root / "src" / "app.tsx").write_text("export const App = () => null;\n", encoding="utf-8")
        (self.root / "artifacts").mkdir()
        (self.root / "artifacts" / "candidate.svg").write_text("<svg/>\n", encoding="utf-8")
        (self.root / "artifacts" / "candidate.png").write_bytes(b"candidate-pixels")
        self.packet = self.make_packet()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def make_packet(self) -> dict:
        task = {"id": "task-1", "brief": "Repair the empty-state illustration."}
        rubric_text = "focal action; saved pixels; no blocking defect"
        source = [{"path": "src/app.tsx", "sha256": sha((self.root / "src/app.tsx").read_bytes())}]
        source_digest = MODULE._canonical_sha(source)
        rubric_digest = sha(rubric_text.encode())
        budget_spec = {
            "id": "budget-1",
            "started_at": "2026-10-27T09:00:00+03:00",
            "deadline_at": "2026-10-27T10:00:00+03:00",
            "repair_limit": 2,
        }
        budget_sha = MODULE._canonical_sha(budget_spec)
        budget = dict(budget_spec, spec_sha256=budget_sha, repairs_used=1, repairs=[{
            "id": "repair-1", "scope": "candidate", "candidate_id": "candidate-a", "at": "2026-10-27T09:15:00+03:00"
        }])
        brief_sha = sha(task["brief"].encode())
        contract = {
            "id": "contract-1", "task_id": task["id"], "brief_sha256": brief_sha,
            "frozen_at": "2026-10-27T08:59:00+03:00",
            "rubric_sha256": rubric_digest, "budget_sha256": budget_sha,
            "source_manifest_sha256": source_digest,
        }
        state = {"id": "desktop", "route_intent": "/empty", "viewport": {"width": 1200, "height": 800}, "expected": "empty illustration and retry affordance"}
        contract["states_sha256"] = MODULE._canonical_sha([state])
        capture = {
            "id": "capture-a", "path": "artifacts/candidate.png", "sha256": sha((self.root / "artifacts/candidate.png").read_bytes()),
            "role": "final", "state_id": "desktop", "route_intent": "/empty", "viewport": {"width": 1200, "height": 800},
            "captured_at": "2026-10-27T09:18:00+03:00",
        }
        snapshot = {"id": "snapshot-a", "path": "artifacts/candidate.svg", "sha256": sha((self.root / "artifacts/candidate.svg").read_bytes()), "role": "source"}
        candidate = {
            "id": "candidate-a", "task_id": task["id"], "contract_ref": contract["id"],
            "rubric_sha256": rubric_digest, "budget_ref": budget["id"], "budget_sha256": budget_sha,
            "snapshots": [snapshot], "captures": [capture], "findings": [],
            "checks": [
                {"id": "file-1", "kind": "file", "result": "passed", "evidence": "sha256 and source manifest"},
                {"id": "visual-1", "kind": MODULE.VISUAL_REPORT, "attestation": "reported_visual_inspection", "capture_ref": "capture-a", "reported_by": "reviewer-1", "reported_at": "2026-10-27T09:20:00+03:00", "statement": "Reviewer reports inspecting the saved capture."},
            ], "selected": True,
        }
        return {
            "schema_version": 1, "packet_id": "packet-1", "artifact_root": ".", "task": task,
            "contract": contract, "rubric": {"id": "rubric-1", "text": rubric_text, "sha256": rubric_digest},
            "budget": budget, "source_manifest": source, "states": [state], "candidates": [candidate],
        }

    def assert_refused(self, packet=None, message="") -> None:
        with self.assertRaisesRegex(MODULE.EvidenceError, message or ".+"):
            MODULE.validate_packet(packet or self.packet, self.root)

    def add_motion_evidence(self) -> None:
        motion = {
            "fps": {"numerator": 30, "denominator": 1},
            "frame_count": 90,
            "parity_claim": True,
            "beats": [{"id": "intro", "frame": 0, "state_id": "desktop", "expected": "intro illustration is legible"}],
        }
        self.packet["motion"] = motion
        self.packet["contract"]["motion_sha256"] = MODULE._canonical_sha(motion)
        preview = self.packet["candidates"][0]["captures"][0]
        preview.update({"beat_id": "intro", "frame": 0, "capture_kind": "preview"})
        export = json.loads(json.dumps(preview))
        export.update({"id": "capture-export", "capture_kind": "decoded_export"})
        self.packet["candidates"][0]["captures"].append(export)

    def test_valid_packet_reports_visual_attestation_without_certifying_pixels(self) -> None:
        result = MODULE.validate_packet(self.packet, self.root)
        self.assertTrue(result["valid"])
        self.assertEqual(result["selected_candidate"], "candidate-a")
        self.assertEqual(result["visual_inspection"], "reported_only; pixels_seen_not_certified")
        self.assertNotIn("pixels_seen", result)

    def test_stale_capture_hash_is_rejected(self) -> None:
        self.packet["candidates"][0]["captures"][0]["sha256"] = "0" * 64
        self.assert_refused(message="stale artifact hash")

    def test_stale_source_manifest_hash_is_rejected(self) -> None:
        self.packet["source_manifest"][0]["sha256"] = "0" * 64
        self.assert_refused(message="stale source manifest hash")

    def test_absolute_and_parent_paths_are_rejected(self) -> None:
        for path in (str(self.root / "artifacts/candidate.png"), "../outside.png"):
            with self.subTest(path=path):
                self.packet["candidates"][0]["captures"][0]["path"] = path
                self.assert_refused(message="portable relative path|unsafe path component")
                self.packet = self.make_packet()

    def test_symlink_artifact_is_rejected(self) -> None:
        link = self.root / "artifacts" / "linked.png"
        link.symlink_to(self.root / "artifacts" / "candidate.png")
        self.packet["candidates"][0]["captures"][0]["path"] = "artifacts/linked.png"
        self.packet["candidates"][0]["captures"][0]["sha256"] = sha((self.root / "artifacts/candidate.png").read_bytes())
        self.assert_refused(message="traverses a symlink")

    def test_missing_capture_and_missing_expected_state_are_rejected(self) -> None:
        self.packet["candidates"][0]["captures"] = []
        self.assert_refused(message="must be a non-empty array")
        self.packet = self.make_packet()
        self.packet["states"][0]["expected"] = ""
        self.assert_refused(message=r"states\[0\]\.expected")

    def test_capture_must_match_explicit_state_viewport_and_route(self) -> None:
        self.packet["candidates"][0]["captures"][0]["viewport"]["width"] = 800
        self.assert_refused(message="viewport differs")
        self.packet = self.make_packet()
        self.packet["candidates"][0]["captures"][0]["route_intent"] = "/wrong"
        self.assert_refused(message="route_intent differs")

    def test_fake_verified_enum_and_pixel_seen_claim_are_rejected(self) -> None:
        self.packet["candidates"][0]["status"] = "verified"
        self.assert_refused(message="blanket verification|forbidden verified")
        self.packet = self.make_packet()
        self.packet["candidates"][0]["checks"][1]["pixels_seen"] = True
        self.assert_refused(message="blanket verification|pixels were seen")

    def test_changed_rubric_or_budget_reference_is_rejected(self) -> None:
        self.packet["candidates"][0]["rubric_sha256"] = "0" * 64
        self.assert_refused(message="changed rubric or budget")
        self.packet = self.make_packet()
        self.packet["candidates"][0]["budget_sha256"] = "0" * 64
        self.assert_refused(message="changed rubric or budget")

    def test_nested_budget_reset_and_overbudget_repairs_are_rejected(self) -> None:
        self.packet["candidates"][0]["repair_limit"] = 99
        self.assert_refused(message="reset")
        self.packet = self.make_packet()
        self.packet["budget"]["repairs_used"] = 3
        self.packet["budget"]["repairs"].extend([
            {"id": "repair-2", "scope": "delegate", "delegate_id": "d1", "at": "2026-10-27T09:30:00+03:00"},
            {"id": "repair-3", "scope": "specimen", "specimen_id": "s1", "at": "2026-10-27T09:31:00+03:00"},
        ])
        self.assert_refused(message="over budget")

    def test_repair_outside_window_is_rejected(self) -> None:
        self.packet["budget"]["repairs"][0]["at"] = "2026-10-27T11:00:00+03:00"
        self.assert_refused(message="outside the shared budget")

    def test_repair_scope_cannot_name_unknown_candidate(self) -> None:
        self.packet["budget"]["repairs"][0]["candidate_id"] = "ghost"
        self.assert_refused(message="unknown candidate")

    def test_findings_must_reference_captures_and_selected_blockers_must_resolve(self) -> None:
        self.packet["candidates"][0]["findings"] = [{
            "id": "finding-1", "capture_ref": "not-a-capture", "location": "hero", "observed_at": "2026-10-27T09:22:00+03:00", "consequence": "focal action is unclear", "severity": "blocking", "status": "open"
        }]
        self.assert_refused(message="not tied to a capture")
        self.packet = self.make_packet()
        self.packet["candidates"][0]["findings"] = [{
            "id": "finding-1", "capture_ref": "capture-a", "location": "hero", "observed_at": "2026-10-27T09:22:00+03:00", "consequence": "focal action is unclear", "severity": "blocking", "status": "open"
        }]
        self.assert_refused(message="unresolved blocking")

    def test_selected_candidate_cannot_have_failed_file_or_automated_check(self) -> None:
        self.packet["candidates"][0]["checks"][0]["result"] = "failed"
        self.assert_refused(message="failed checks")

    def test_frozen_state_expectation_hash_rejects_mutation(self) -> None:
        self.packet["states"][0]["expected"] = "different expected pixels"
        self.assert_refused(message="state expectations changed")

    def test_motion_parity_requires_explicit_paired_samples(self) -> None:
        self.add_motion_evidence()
        self.assertTrue(MODULE.validate_packet(self.packet, self.root)["valid"])
        self.packet["candidates"][0]["captures"].pop()
        self.assert_refused(message="capture counterpart")

    def test_motion_capture_frame_must_match_frozen_beat(self) -> None:
        self.add_motion_evidence()
        self.packet["candidates"][0]["captures"][0]["frame"] = 1
        self.assert_refused(message="frame does not match")

    def test_motion_rejects_invalid_units_and_bounds(self) -> None:
        self.add_motion_evidence()
        self.packet["motion"]["fps"]["denominator"] = 0
        self.assert_refused(message="fps.denominator")
        self.packet = self.make_packet()
        self.add_motion_evidence()
        self.packet["motion"]["beats"][0]["frame"] = 90
        self.packet["contract"]["motion_sha256"] = MODULE._canonical_sha(self.packet["motion"])
        self.assert_refused(message="outside frame_count")

    def test_motion_expectations_and_budget_cannot_be_rewritten(self) -> None:
        self.add_motion_evidence()
        self.packet["motion"]["beats"][0]["expected"] = "changed expectation"
        self.assert_refused(message="motion expectations changed")
        self.packet = self.make_packet()
        self.add_motion_evidence()
        self.packet["motion"]["repair_limit"] = 99
        self.assert_refused(message="reset the shared budget")

    def test_selected_candidate_cannot_be_regressed_or_ambiguous(self) -> None:
        self.packet["candidates"][0]["regressed"] = True
        self.assert_refused(message="regression")
        self.packet = self.make_packet()
        clone = json.loads(json.dumps(self.packet["candidates"][0]))
        clone["id"] = "candidate-b"
        clone["selected"] = True
        self.packet["candidates"].append(clone)
        self.assert_refused(message="exactly one candidate")


if __name__ == "__main__":
    unittest.main()
