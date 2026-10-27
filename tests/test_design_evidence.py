#!/usr/bin/env python3
"""Adversarial tests for the dependency-free design evidence validator."""

from __future__ import annotations

import hashlib
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
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
        (self.root / "artifacts" / "candidate-export.png").write_bytes(b"candidate-pixels")
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
        budget = dict(budget_spec, spec_sha256=budget_sha, repairs_used=0, repairs=[])
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
        export.update({
            "id": "capture-export", "path": "artifacts/candidate-export.png",
            "sha256": sha((self.root / "artifacts/candidate-export.png").read_bytes()),
            "capture_kind": "decoded_export",
        })
        self.packet["candidates"][0]["captures"].append(export)

    def add_scope(self, directories=None) -> None:
        scope = {"files": ["src/app.tsx"], "directories": directories or ["artifacts"]}
        self.packet["artifact_scope"] = scope
        self.packet["contract"]["artifact_scope_sha256"] = MODULE._canonical_sha(scope)

    def add_repair(self) -> None:
        parent = self.packet["candidates"][0]
        parent["source_manifest"] = copy.deepcopy(self.packet["source_manifest"])
        parent["source_manifest_sha256"] = self.packet["contract"]["source_manifest_sha256"]
        parent["snapshots"] = [dict(parent["source_manifest"][0], id="source-a", role="source", captured_at="2026-10-27T09:10:00+03:00")]
        parent["captures"][0]["source_manifest_sha256"] = parent["source_manifest_sha256"]
        parent["selected"] = False
        finding = {"id": "finding-1", "capture_ref": "capture-a", "location": "action",
                   "observed_at": "2026-10-27T09:22:00+03:00", "consequence": "action unreadable",
                   "severity": "blocking", "status": "resolved"}
        parent["findings"] = [finding]
        child = copy.deepcopy(parent)
        (self.root / "src/repaired.tsx").write_text("export const App = () => 'Retry';\n")
        (self.root / "artifacts/repaired.png").write_bytes(b"readable action pixels")
        manifest = [{"path": "src/repaired.tsx", "sha256": sha((self.root / "src/repaired.tsx").read_bytes())}]
        digest = MODULE._canonical_sha(manifest)
        child.update(id="candidate-b", selected=True, findings=[], source_manifest=manifest, source_manifest_sha256=digest)
        child["snapshots"] = [dict(manifest[0], id="source-b", role="source", captured_at="2026-10-27T09:26:00+03:00")]
        child["captures"][0].update(id="capture-b", path="artifacts/repaired.png", sha256=sha(b"readable action pixels"), captured_at="2026-10-27T09:27:00+03:00", source_manifest_sha256=digest)
        child["checks"][1].update(capture_ref="capture-b", reported_at="2026-10-27T09:28:00+03:00")
        child["lineage"] = {"parent_candidate_id": "candidate-a", "parent_source_manifest_sha256": parent["source_manifest_sha256"], "repair_ref": "repair-1", "finding_refs": [{"candidate_id": "candidate-a", "finding_id": "finding-1"}]}
        self.packet["candidates"].append(child)
        self.packet["budget"].update(repairs_used=1, repairs=[{"id": "repair-1", "scope": "candidate", "candidate_id": "candidate-b", "at": "2026-10-27T09:25:00+03:00"}])
        finding["disposition"] = {"kind": "resolved", "reason": "Action is readable in the revised capture.", "brief_sha256": self.packet["contract"]["brief_sha256"], "rubric_sha256": self.packet["contract"]["rubric_sha256"], "at": "2026-10-27T09:29:00+03:00", "repair_ref": "repair-1", "evidence": {"candidate_id": "candidate-b", "capture_ref": "capture-b", "check_ref": "visual-1"}}

    def test_repair_links_preserved_source_finding_event_and_review(self) -> None:
        self.add_repair()
        result = MODULE.validate_packet(self.packet, self.root)
        self.assertEqual(result["repair_lineage"], "checked")
        self.assertEqual(result["disposition_count"], 1)
        self.assertEqual(result["remaining_findings"], [])

    def test_resolved_word_without_evidence_is_rejected(self) -> None:
        self.add_repair()
        del self.packet["candidates"][0]["findings"][0]["disposition"]
        self.assert_refused(message="disposition must be an object")

    def test_legacy_repair_without_lineage_is_rejected(self) -> None:
        self.packet["budget"].update(repairs_used=1, repairs=[{"id": "old-repair", "scope": "candidate", "candidate_id": "candidate-a", "at": "2026-10-27T09:25:00+03:00"}])
        self.assert_refused(message="without candidate lineage")

    def test_unknown_self_and_cyclic_parents_are_rejected(self) -> None:
        self.add_repair()
        child = self.packet["candidates"][1]
        for parent in ("unknown", "candidate-b"):
            child["lineage"]["parent_candidate_id"] = parent
            self.assert_refused(message="unknown or self parent")
        child["lineage"]["parent_candidate_id"] = "candidate-a"
        self.packet["candidates"][0]["lineage"] = dict(child["lineage"], parent_candidate_id="candidate-b", repair_ref="repair-2")
        self.packet["budget"]["repairs"].append({"id": "repair-2", "scope": "candidate", "candidate_id": "candidate-a", "at": "2026-10-27T09:30:00+03:00"})
        self.packet["budget"]["repairs_used"] = 2
        self.assert_refused(message="cycle")

    def test_event_is_charged_once_and_names_derived_candidate(self) -> None:
        self.add_repair()
        child = self.packet["candidates"][1]
        child["lineage"]["repair_ref"] = "uncharged"
        self.assert_refused(message="unknown or reused repair")
        child["lineage"]["repair_ref"] = "repair-1"
        self.packet["budget"]["repairs"][0]["candidate_id"] = "candidate-a"
        self.assert_refused(message="does not name its derived candidate")
        self.packet["budget"]["repairs"][0]["candidate_id"] = "candidate-b"
        sibling = copy.deepcopy(child); sibling.update(id="candidate-c", selected=False)
        self.packet["candidates"].append(sibling)
        self.assert_refused(message="unknown or reused repair")

    def test_delegate_and_specimen_events_use_same_ledger_and_result_candidate(self) -> None:
        self.add_repair()
        event = self.packet["budget"]["repairs"][0]
        del event["candidate_id"]
        for scope in ("delegate", "specimen"):
            event.update(scope=scope, result_candidate_id="candidate-b")
            event[scope + "_id"] = "worker-or-specimen"
            self.assertTrue(MODULE.validate_packet(self.packet, self.root)["valid"])
            del event["result_candidate_id"]
            self.assert_refused(message="does not name its derived candidate")
            del event[scope + "_id"]

    def test_missing_lineage_cannot_turn_changed_source_into_new_root(self) -> None:
        self.add_repair()
        del self.packet["candidates"][1]["lineage"]
        self.packet["budget"].update(repairs_used=0, repairs=[])
        self.assert_refused(message="root candidate must retain")

    def test_candidate_and_parent_source_bindings_reject_stale_changes(self) -> None:
        self.add_repair()
        child = self.packet["candidates"][1]
        child["lineage"]["parent_source_manifest_sha256"] = "0" * 64
        self.assert_refused(message="parent source reference changed")
        child["lineage"]["parent_source_manifest_sha256"] = self.packet["contract"]["source_manifest_sha256"]
        child["source_manifest_sha256"] = "0" * 64
        self.assert_refused(message="candidate source manifest reference changed")

    def test_overwriting_parent_source_is_rejected(self) -> None:
        self.add_repair()
        (self.root / "src/app.tsx").write_text("overwritten source")
        self.assert_refused(message="stale source manifest hash")

    def test_child_cannot_reuse_parent_capture_or_source_binding(self) -> None:
        self.add_repair()
        parent, child = self.packet["candidates"]
        capture = child["captures"][0]
        capture["source_manifest_sha256"] = parent["source_manifest_sha256"]
        self.assert_refused(message="not bound to its candidate source")
        capture["source_manifest_sha256"] = child["source_manifest_sha256"]
        capture.update(path=parent["captures"][0]["path"], sha256=parent["captures"][0]["sha256"])
        self.assert_refused(message="reuses an ancestor capture")

    def test_repair_snapshot_capture_and_finding_order(self) -> None:
        self.add_repair()
        original = copy.deepcopy(self.packet)
        for target, stamp, error in (("snapshot", "2026-10-27T09:25:00+03:00", "snapshot must follow"), ("capture", "2026-10-27T09:24:00+03:00", "capture must follow its source"), ("capture", "2026-10-27T09:26:00+03:00", "capture must follow its source"), ("event", "2026-10-27T09:21:00+03:00", "repair must follow its addressed finding")):
            with self.subTest(target=target):
                self.packet = copy.deepcopy(original)
                if target == "event": self.packet["budget"]["repairs"][0]["at"] = stamp
                else: self.packet["candidates"][1]["snapshots" if target == "snapshot" else "captures"][0]["captured_at"] = stamp
                self.assert_refused(message=error)

    def test_duplicate_source_snapshot_paths_are_rejected(self) -> None:
        self.add_repair()
        child = self.packet["candidates"][1]
        duplicate = dict(child["snapshots"][0], id="another-source-record")
        child["snapshots"].append(duplicate)
        self.assert_refused(message="duplicate source snapshot paths")

    def test_repair_must_name_an_ancestor_finding_unambiguously(self) -> None:
        self.add_repair()
        edge = self.packet["candidates"][1]["lineage"]
        edge["finding_refs"] = []
        self.assert_refused(message="non-empty array")
        edge["finding_refs"] = [{"candidate_id": "candidate-b", "finding_id": "finding-1"}]
        self.assert_refused(message="ancestor candidate")

    def test_resolution_cannot_use_parent_capture_or_unrelated_check(self) -> None:
        self.add_repair()
        evidence = self.packet["candidates"][0]["findings"][0]["disposition"]["evidence"]
        evidence["check_ref"] = "file-1"
        self.assert_refused(message="not tied to its capture")
        evidence.update(candidate_id="candidate-a", capture_ref="capture-a", check_ref="visual-1")
        self.assert_refused(message="descendant repair candidate")

    def test_disposition_cannot_rewrite_criterion_or_predate_review(self) -> None:
        self.add_repair()
        disposition = self.packet["candidates"][0]["findings"][0]["disposition"]
        disposition["brief_sha256"] = "0" * 64
        self.assert_refused(message="changed its frozen criterion")
        disposition["brief_sha256"] = self.packet["contract"]["brief_sha256"]
        disposition["at"] = "2026-10-27T09:27:30+03:00"
        self.assert_refused(message="precede disposition")

    def test_selected_child_cannot_drop_unrelated_ancestor_blocker(self) -> None:
        self.add_repair()
        other = copy.deepcopy(self.packet["candidates"][0]["findings"][0])
        other.update(id="unrelated", status="open"); del other["disposition"]
        self.packet["candidates"][0]["findings"].append(other)
        self.assert_refused(message="selected lineage retains an unresolved blocking")
        other["severity"] = "minor"
        result = MODULE.validate_packet(self.packet, self.root)
        self.assertEqual(result["remaining_findings"], [{"candidate_id": "candidate-a", "finding_id": "unrelated", "severity": "minor"}])

    def test_resolved_parent_cannot_be_selected_instead_of_repaired_child(self) -> None:
        self.add_repair()
        self.packet["candidates"][0]["selected"] = True
        self.packet["candidates"][1]["selected"] = False
        self.assert_refused(message="selected lineage retains an unresolved blocking")

    def test_rejected_claim_retains_original_and_reason_without_charging_repair(self) -> None:
        self.add_repair()
        parent = self.packet["candidates"][0]
        self.packet["candidates"] = [parent]; parent["selected"] = True
        self.packet["budget"].update(repairs_used=0, repairs=[])
        finding = parent["findings"][0]; finding["status"] = "rejected"
        d = finding["disposition"]
        d.update(kind="rejected", reason="The frozen brief requires a readable action, not a changed fill.")
        d.pop("repair_ref")
        d["evidence"].update(candidate_id="candidate-a", capture_ref="capture-a")
        original_claim = finding["consequence"]
        result = MODULE.validate_packet(self.packet, self.root)
        self.assertEqual(result["repair_count"], 0)
        self.assertEqual(finding["consequence"], original_claim)
        d["reason"] = ""
        self.assert_refused(message="reason must be non-empty")

    def test_disposition_automated_review_requires_pass_and_capture_time(self) -> None:
        self.add_repair()
        child = self.packet["candidates"][1]
        child["checks"][1] = {"id": "visual-1", "kind": "automated", "capture_ref": "capture-b", "checked_at": "2026-10-27T09:28:00+03:00", "result": "passed", "evidence": "independent contrast measurement"}
        self.assertTrue(MODULE.validate_packet(self.packet, self.root)["valid"])
        child["checks"][1]["checked_at"] = "2026-10-27T09:26:00+03:00"
        self.assert_refused(message="review must follow capture")

    def test_duplicate_check_ids_and_unknown_finding_status_are_rejected(self) -> None:
        self.add_repair()
        parent = self.packet["candidates"][0]
        parent["checks"][1]["id"] = "file-1"
        self.assert_refused(message="duplicate check ids")
        parent["checks"][1]["id"] = "visual-1"
        parent["findings"][0]["status"] = "closed"
        self.assert_refused(message="unknown status")

    def test_second_repair_preserves_ancestor_resolution_and_candidate_local_finding_ids(self) -> None:
        self.add_repair()
        initial, middle = self.packet["candidates"]
        finding = copy.deepcopy(initial["findings"][0])
        finding.update(capture_ref="capture-b", observed_at="2026-10-27T09:31:00+03:00", consequence="arrow still needs repair")
        middle["findings"] = [finding]  # Same local ID, distinct candidate.
        middle["selected"] = False
        final = copy.deepcopy(middle)
        (self.root / "src/final.tsx").write_text("export const App = () => 'Retry →';\n")
        (self.root / "artifacts/final.png").write_bytes(b"readable action and arrow pixels")
        manifest = [{"path": "src/final.tsx", "sha256": sha((self.root / "src/final.tsx").read_bytes())}]
        digest = MODULE._canonical_sha(manifest)
        final.update(id="candidate-c", selected=True, findings=[], source_manifest=manifest, source_manifest_sha256=digest)
        final["snapshots"] = [dict(manifest[0], id="source-c", role="source", captured_at="2026-10-27T09:33:00+03:00")]
        final["captures"][0].update(id="capture-c", path="artifacts/final.png", sha256=sha(b"readable action and arrow pixels"), captured_at="2026-10-27T09:34:00+03:00", source_manifest_sha256=digest)
        final["checks"][1].update(capture_ref="capture-c", reported_at="2026-10-27T09:35:00+03:00")
        final["lineage"].update(parent_candidate_id="candidate-b", parent_source_manifest_sha256=middle["source_manifest_sha256"], repair_ref="repair-2", finding_refs=[{"candidate_id": "candidate-b", "finding_id": "finding-1"}])
        self.packet["candidates"].append(final)
        self.packet["budget"]["repairs"].append({"id": "repair-2", "scope": "candidate", "candidate_id": "candidate-c", "at": "2026-10-27T09:32:00+03:00"})
        self.packet["budget"]["repairs_used"] = 2
        finding["disposition"].update(at="2026-10-27T09:36:00+03:00", repair_ref="repair-2")
        finding["disposition"]["evidence"].update(candidate_id="candidate-c", capture_ref="capture-c")
        self.assertEqual(MODULE.validate_packet(self.packet, self.root)["disposition_count"], 2)
        final["lineage"]["finding_refs"][0]["candidate_id"] = "candidate-a"
        self.assert_refused(message="already dispositioned finding")

    def test_resolution_in_another_branch_does_not_clear_selected_branch(self) -> None:
        self.add_repair()
        initial, child = self.packet["candidates"]
        # A second root can retain the same initial source, but does not inherit
        # resolution of another root's finding merely because local IDs match.
        other = copy.deepcopy(initial)
        other.update(id="other-root", selected=True)
        child["selected"] = False
        self.packet["candidates"].append(other)
        self.assert_refused(message="descendant repair candidate")

    def test_resolution_requires_its_own_event_and_existing_capture(self) -> None:
        self.add_repair()
        d = self.packet["candidates"][0]["findings"][0]["disposition"]
        d["repair_ref"] = "wrong"
        self.assert_refused(message="wrong repair event")
        d["repair_ref"] = "repair-1"
        d["evidence"]["capture_ref"] = "missing"
        self.assert_refused(message="unknown capture evidence")

    def test_valid_packet_reports_visual_attestation_without_certifying_pixels(self) -> None:
        result = MODULE.validate_packet(self.packet, self.root)
        self.assertTrue(result["valid"])
        self.assertEqual(result["selected_candidate"], "candidate-a")
        self.assertEqual(result["visual_inspection"], "reported_only; pixels_seen_not_certified")
        self.assertNotIn("pixels_seen", result)
        self.assertEqual(result["artifact_scope"], "unrecorded")
        self.assertIsNone(result["artifact_inventory"])

    def test_frozen_scope_inventories_all_allowed_files_including_unreferenced_evidence(self) -> None:
        self.add_scope()
        (self.root / "artifacts/detail.png").write_bytes(b"additional saved evidence")
        result = MODULE.validate_packet(self.packet, self.root)
        self.assertEqual(result["artifact_scope"], "checked_against_frozen_allowlist")
        inventory = result["artifact_inventory"]
        self.assertEqual({item["path"] for item in inventory}, {
            "src/app.tsx", "artifacts/candidate.svg", "artifacts/candidate.png",
            "artifacts/candidate-export.png", "artifacts/detail.png",
        })
        self.assertEqual(result["artifact_inventory_sha256"], MODULE._canonical_sha(inventory))
        self.assertEqual(result["visual_inspection"], "reported_only; pixels_seen_not_certified")

    def test_scope_rejects_unlisted_files_hidden_entries_and_prefix_lookalikes(self) -> None:
        self.add_scope()
        for relative in ("unrequested.txt", ".hidden", "artifacts-other/frame.png"):
            with self.subTest(relative=relative):
                path = self.root / relative
                path.parent.mkdir(exist_ok=True)
                path.write_bytes(b"unscoped")
                self.assert_refused(message="outside artifact scope")
                path.unlink()
                if path.parent != self.root:
                    path.parent.rmdir()

    def test_scope_permissions_do_not_require_optional_outputs(self) -> None:
        self.add_scope(["artifacts", "optional-review"])
        self.packet["artifact_scope"]["files"].append("optional.txt")
        self.packet["contract"]["artifact_scope_sha256"] = MODULE._canonical_sha(self.packet["artifact_scope"])
        self.assertTrue(MODULE.validate_packet(self.packet, self.root)["valid"])

    def test_scope_rejects_existing_entries_of_the_wrong_declared_type(self) -> None:
        self.add_scope()
        self.packet["artifact_scope"]["files"].append("src")
        self.packet["contract"]["artifact_scope_sha256"] = MODULE._canonical_sha(self.packet["artifact_scope"])
        self.assert_refused(message="directory as a file: src")
        self.add_scope(["artifacts", "src/app.tsx"])
        self.packet["artifact_scope"]["files"] = []
        self.packet["contract"]["artifact_scope_sha256"] = MODULE._canonical_sha(self.packet["artifact_scope"])
        self.assert_refused(message="file as a directory: src/app.tsx")

    def test_scope_follows_frozen_condition_not_a_hardcoded_feedback_folder(self) -> None:
        evidence = self.root / "evidence"
        evidence.mkdir()
        (evidence / "review.txt").write_text("saved repair evidence")
        self.add_scope(["artifacts", "evidence"])
        self.assertTrue(MODULE.validate_packet(self.packet, self.root)["valid"])
        self.add_scope(["artifacts", ".feedback"])
        self.assert_refused(message="outside artifact scope: evidence")

    def test_scope_change_and_removal_refuse_without_rewriting_frozen_contract(self) -> None:
        self.add_scope()
        self.packet["artifact_scope"]["directories"].append("anything")
        self.assert_refused(message="artifact scope changed")
        del self.packet["artifact_scope"]
        self.assert_refused(message="frozen artifact scope is missing")

    def test_scope_rejects_traversal_and_ambiguous_paths(self) -> None:
        for relative in (".", "../escape", "/tmp", "artifacts/../escape", "artifacts//nested", "artifacts\\nested"):
            with self.subTest(relative=relative):
                self.add_scope([relative])
                self.assert_refused(message="portable relative paths")

    def test_scope_rejects_unreferenced_symlinks_even_in_allowed_directory(self) -> None:
        self.add_scope()
        link = self.root / "artifacts/unreferenced"
        link.symlink_to(self.root / "missing")
        self.assert_refused(message="symlink")
        link.unlink()
        link.symlink_to(self.root / "src", target_is_directory=True)
        self.assert_refused(message="symlink")

    def test_cli_excludes_only_the_actual_packet_file(self) -> None:
        self.add_scope()
        packet_path = self.root / "review.json"
        packet_path.write_text(json.dumps(self.packet))
        result = subprocess.run([sys.executable, str(SCRIPT), "--packet", str(packet_path), "--root", str(self.root)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn("review.json", {item["path"] for item in json.loads(result.stdout)["artifact_inventory"]})
        (self.root / "another-packet.json").write_text("{}")
        refused = subprocess.run([sys.executable, str(SCRIPT), "--packet", str(packet_path), "--root", str(self.root)], capture_output=True, text=True)
        self.assertNotEqual(refused.returncode, 0)
        self.assertIn("another-packet.json", refused.stderr)

    def test_capture_and_visual_report_must_stay_in_shared_budget(self) -> None:
        for before in (True, False):
            for kind in ("capture", "report"):
                with self.subTest(before=before, kind=kind):
                    self.packet = self.make_packet()
                    stamp = "2026-10-27T08:58:00+03:00" if before else "2026-10-27T10:00:01+03:00"
                    candidate = self.packet["candidates"][0]
                    if kind == "capture":
                        candidate["captures"][0]["captured_at"] = stamp
                    else:
                        candidate["checks"][1]["reported_at"] = stamp
                    self.assert_refused(message="outside the shared budget window")

    def test_visual_report_cannot_predate_capture_even_inside_budget(self) -> None:
        self.packet["candidates"][0]["checks"][1]["reported_at"] = "2026-10-27T09:17:00+03:00"
        self.assert_refused(message="visual_report predates its capture")

    def test_finding_cannot_predate_capture_or_outlive_shared_budget(self) -> None:
        finding = {"id": "finding-1", "capture_ref": "capture-a", "location": "hero",
                   "observed_at": "2026-10-27T09:17:00+03:00", "consequence": "label unreadable",
                   "severity": "minor", "status": "open"}
        self.packet["candidates"][0]["findings"] = [finding]
        self.assert_refused(message="finding finding-1 predates its capture")
        finding["observed_at"] = "2026-10-27T10:01:00+03:00"
        self.assert_refused(message="outside the shared budget window")

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

    def test_distinct_states_cannot_reuse_one_artifact_path(self) -> None:
        state = {"id": "mobile", "route_intent": "/empty", "viewport": {"width": 390, "height": 844}, "expected": "mobile empty illustration"}
        self.packet["states"].append(state)
        self.packet["contract"]["states_sha256"] = MODULE._canonical_sha(self.packet["states"])
        reused = json.loads(json.dumps(self.packet["candidates"][0]["captures"][0]))
        reused.update({"id": "capture-mobile", "state_id": "mobile", "viewport": state["viewport"], "captured_at": "2026-10-27T09:21:00+03:00"})
        self.packet["candidates"][0]["captures"].append(reused)
        self.assert_refused(message="reuses one artifact path across states")

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
        self.add_repair()
        self.packet["budget"]["repairs_used"] = 3
        self.packet["budget"]["repairs"].extend([
            {"id": "repair-2", "scope": "delegate", "delegate_id": "d1", "at": "2026-10-27T09:30:00+03:00"},
            {"id": "repair-3", "scope": "specimen", "specimen_id": "s1", "at": "2026-10-27T09:31:00+03:00"},
        ])
        self.assert_refused(message="over budget")

    def test_repair_outside_window_is_rejected(self) -> None:
        self.add_repair()
        self.packet["budget"]["repairs"][0]["at"] = "2026-10-27T11:00:00+03:00"
        self.assert_refused(message="outside the shared budget")

    def test_repair_scope_cannot_name_unknown_candidate(self) -> None:
        self.add_repair()
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

    def test_motion_parity_rejects_preview_export_path_reuse_but_allows_equal_hashes(self) -> None:
        self.add_motion_evidence()
        self.assertEqual(
            self.packet["candidates"][0]["captures"][0]["sha256"],
            self.packet["candidates"][0]["captures"][1]["sha256"],
        )
        self.assertTrue(MODULE.validate_packet(self.packet, self.root)["valid"])
        self.packet["candidates"][0]["captures"][1]["path"] = "artifacts/candidate.png"
        self.assert_refused(message="reuses one artifact path for preview and decoded_export")

    def test_motion_capture_frame_must_match_frozen_beat(self) -> None:
        self.add_motion_evidence()
        self.packet["candidates"][0]["captures"][0]["frame"] = 1
        self.assert_refused(message="frame does not match")

    def test_motion_allows_multiple_distinct_captures_of_one_kind(self) -> None:
        self.add_motion_evidence()
        extra = dict(self.packet["candidates"][0]["captures"][0])
        extra.update({"id": "preview-again", "path": "artifacts/preview-again.png"})
        (self.root / extra["path"]).write_bytes((self.root / "artifacts/candidate.png").read_bytes())
        self.packet["candidates"][0]["captures"].append(extra)
        self.assertTrue(MODULE.validate_packet(self.packet, self.root)["valid"])
        self.packet["candidates"][0]["captures"][1]["path"] = extra["path"]
        self.assert_refused(message="reuses one artifact path for preview and decoded_export")

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
