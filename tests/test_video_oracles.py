import importlib.util
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import video_oracles as PILOT

class VideoAuthoringOracleTests(unittest.TestCase):
    def test_validator_is_independent_of_self_report(self):
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary)
            (project / "SELF_REPORT.md").write_text("success; all checks passed")
            result = PILOT.validate("glasshouse-product", project)
            self.assertFalse(result["passed"])
            self.assertTrue(any("missing required files" in finding for finding in result["findings"]))

    def test_refusal_validator_requires_reason_codes(self):
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary)
            (project / "decision.json").write_text(json.dumps({"status": "refused", "reason_codes": ["external-network"]}))
            result = PILOT.validate("refusal-local-asset", project)
            self.assertFalse(result["passed"])
            self.assertTrue(any("reason codes" in finding for finding in result["findings"]))

    def test_caption_validator_accepts_exclusive_end_36_and_shared_boundary(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "captions.json"
            path.write_text(json.dumps([
                {"start_frame": 0, "end_frame": 18, "text": "first"},
                {"start_frame": 18, "end_frame": 36, "text": "second"},
            ]))
            self.assertEqual(PILOT.valid_caption(path), (True, "caption coverage valid"))

    def test_caption_validator_leaves_extra_fields_to_production_oracle(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "captions.json"
            path.write_text(json.dumps([
                {"start_frame": 0, "end_frame": 36, "text": "cue", "confidence": 1.0},
            ]))
            valid, reason = PILOT.valid_caption(path)
            self.assertTrue(valid, reason)

    def test_validator_ignores_frozen_agents_snapshot(self):
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary)
            (project / ".agents/skills/video-production").mkdir(parents=True)
            (project / ".agents/skills/video-production/SKILL.md").write_text("frozen")
            (project / "index.html").write_text('<main data-no-timeline data-width="640" data-height="360" data-duration="3">Glasshouse</main>')
            (project / "film.css").write_text("@font-face{src:url('./NotoSans.ttf')}")
            (project / "film.js").write_text("// deterministic")
            (project / "captions.json").write_text(json.dumps([
                {"start_frame": 0, "end_frame": 18, "text": "first"},
                {"start_frame": 18, "end_frame": 36, "text": "second"},
            ]))
            result = PILOT.validate("glasshouse-product", project)
            self.assertTrue(result["passed"], result["findings"])

    def test_production_manifest_selects_font_and_only_renderer_sources(self):
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary)
            for name, content in {
                "index.html": "<main data-no-timeline data-width=\"640\" data-height=\"360\" data-duration=\"3\"></main>",
                "film.css": "@font-face{src:url('./NotoSans.ttf')}",
                "film.js": "",
                "captions.json": "[]",
                "NotoSans.ttf": "font",
                "BRIEF.md": "metadata",
                "DIAGNOSTIC_SEED.json": "metadata",
            }.items():
                (project / name).write_text(content)
            manifest_path = PILOT.production_manifest(project, "glasshouse-product")
            selected = {entry["path"] for entry in json.loads(manifest_path.read_text())["sources"]}
            self.assertEqual(selected, {"index.html", "film.css", "film.js", "captions.json", "NotoSans.ttf"})

    def test_temporal_ledger_rejects_duplicate_phases_and_caps_repairs(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "budget-ledger.json").write_text(json.dumps({"shared_max_repairs": 1, "repairs_used": 0, "runs": {}}))
            PILOT.ledger_phase(root, "case", "initial", "started")
            with self.assertRaises(SystemExit):
                PILOT.ledger_phase(root, "case", "initial", "started")
            PILOT.ledger_phase(root, "case", "initial", "completed")
            PILOT.ledger_phase(root, "case", "repair", "started")
            PILOT.ledger_phase(root, "case", "repair", "completed")
            PILOT.ledger_phase(root, "other", "initial", "started")
            PILOT.ledger_phase(root, "other", "initial", "completed")
            with self.assertRaises(SystemExit):
                PILOT.ledger_phase(root, "other", "repair", "started")

    def test_preflight_requires_zero_exit_and_valid_status(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "frozen/scripts").mkdir(parents=True)
            (root / "frozen/scripts/check-video-source.mjs").write_text('console.log(JSON.stringify({status: "valid"})); process.exitCode = 1;')
            trial = root / "trial"
            project = trial / "project"
            project.mkdir(parents=True)
            (project / "index.html").write_text('<main data-no-timeline data-width="640" data-height="360" data-duration="3"></main>')
            (project / "film.css").write_text("@font-face{src:url('./NotoSans.ttf')}")
            (project / "film.js").write_text("")
            (project / "NotoSans.ttf").write_text("font")
            result = PILOT.run_production_preflight(root, trial, "pebble-motion", shutil.which("node") or "node", "test")
            self.assertFalse(result["oracle_passed"])
            self.assertEqual(result["status"], "refused")

