import importlib.util
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("video-authoring-pilot.py")
SPEC = importlib.util.spec_from_file_location("video_authoring_pilot", SCRIPT)
PILOT = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(PILOT)

class VideoAuthoringPilotTests(unittest.TestCase):
    def test_frozen_matrix_is_24_runs_and_waits_for_ready_manifest(self):
        self.assertEqual(len(PILOT.TASKS) * len(PILOT.CONDITIONS) * len(PILOT.REPEATS), 24)
        self.assertEqual(PILOT.CONDITIONS, ("baseline", "diagnostics-seed"))
        self.assertEqual(PILOT.REPEATS, (1, 2))

    def test_ready_gate_rejects_unapproved_or_wrong_seed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "pilot"
            (root / "frozen").mkdir(parents=True)
            (root / "frozen/seed.sha256").write_text("seed-digest\n")
            path = root / "ready.json"
            path.write_text(json.dumps({"approved": False, "seed_sha256": "wrong"}))
            with self.assertRaises(SystemExit):
                PILOT.ready_paths(root, path)

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

    def test_event_usage_reads_turn_completed_and_marks_missing_usage(self):
        with tempfile.TemporaryDirectory() as temporary:
            events = Path(temporary) / "events.jsonl"
            events.write_text(json.dumps({"type": "turn.completed", "usage": {
                "input_tokens": 192078,
                "cached_input_tokens": 169600,
                "cache_write_input_tokens": 7,
                "output_tokens": 3740,
                "reasoning_output_tokens": 513,
            }}) + "\n")
            self.assertEqual(PILOT.event_usage(events), {
                "input_tokens": 192078,
                "cached_input_tokens": 169600,
                "cache_write_input_tokens": 7,
                "output_tokens": 3740,
                "reasoning_output_tokens": 513,
                "usage_observed": True,
            })
            empty = Path(temporary) / "empty.jsonl"
            empty.write_text(json.dumps({"type": "turn.started"}) + "\n")
            missing = PILOT.event_usage(empty)
            self.assertFalse(missing["usage_observed"])
            self.assertIsNone(missing["input_tokens"])

    @unittest.skipUnless(
        all(os.environ.get(name) for name in ("VIDEO_PILOT_CHECKER", "VIDEO_PILOT_SEED_FIXTURE", "VIDEO_PILOT_ADAPTER", "VIDEO_PILOT_NODE", "VIDEO_PILOT_FONT")),
        "requires the lead-supplied real checker, valid seed fixture, adapter, Node and font",
    )
    def test_full_prepare_and_valid_control_preflight(self):
        with tempfile.TemporaryDirectory(dir=str(PILOT.REPO)) as temporary:
            root = Path(temporary) / "pilot"
            config = root.parent / "config.toml"
            config.write_text('model = "gpt-6-astra"\nmodel_reasoning_effort = "high"\n')
            PILOT.prepare(root, Path(os.environ["VIDEO_PILOT_CHECKER"]), Path(os.environ["VIDEO_PILOT_SEED_FIXTURE"]), Path(os.environ["VIDEO_PILOT_ADAPTER"]), config, Path(os.environ["VIDEO_PILOT_FONT"]))
            for condition in PILOT.CONDITIONS:
                trial = root / "trials" / f"valid-control-{condition}-r1"
                self.assertTrue(PILOT.validate("valid-control", trial / "project")["passed"])
                preflight = PILOT.run_production_preflight(root, trial, "valid-control", os.environ["VIDEO_PILOT_NODE"], "test")
                self.assertEqual(preflight.get("status"), "valid", preflight)
                PILOT.check_frozen_inputs(root, trial)
                if condition == "diagnostics-seed":
                    checked = subprocess.run([
                        os.environ["VIDEO_PILOT_NODE"], str(trial / "project/.diagnostics/scripts/check-video-source.mjs"),
                        "--project", str(trial / "project"), "--manifest", str(trial / "production-manifest.json"),
                    ], capture_output=True, text=True)
                    self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
                    self.assertEqual(json.loads(checked.stdout)["status"], "valid")
            frozen_adapter = root / "frozen/adapters/dist/video-export.js"
            frozen_adapter.write_text("// tampered oracle")
            with self.assertRaises(SystemExit):
                PILOT.check_frozen_inputs(root, trial)


if __name__ == "__main__":
    unittest.main()
