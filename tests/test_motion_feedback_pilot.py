import importlib.util
import json
import os
import stat
import subprocess
import tempfile
import unittest
from unittest.mock import patch, MagicMock
from pathlib import Path


SCRIPT = Path(__file__).with_name("motion-feedback-pilot.py")
SPEC = importlib.util.spec_from_file_location("motion_feedback_pilot", SCRIPT)
PILOT = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(PILOT)


def ready_fixture(root: Path) -> tuple[dict, Path]:
    tools = {}
    for name in ("codex", "mirket", "node", "adapter", "chromium", "ffmpeg", "ffprobe"):
        tools[name] = root / name
        tools[name].write_text(name)
    auth = root / "auth.json"
    auth.write_text("{}")
    auth.chmod(0o600)
    ready = {
        "codex": str(tools["codex"]), "mirket": str(tools["mirket"]), "node": str(tools["node"]),
        "adapter": str(tools["adapter"]), "chromium": str(tools["chromium"]), "ffmpeg": str(tools["ffmpeg"]),
        "ffprobe": str(tools["ffprobe"]), "auth_file": str(auth),
        "model": "gpt-6-astra", "reasoning_effort": "high",
    }
    path = root / "ready.json"
    path.write_text(json.dumps(ready))
    return ready, path


class MotionFeedbackPilotTests(unittest.TestCase):
    def test_matrix_is_twelve_cells_and_shared_repair_budget_is_four(self):
        self.assertEqual(len(PILOT.TASKS) * len(PILOT.CONDITIONS) * len(PILOT.REPEATS), 12)
        self.assertEqual(PILOT.SHARED_MAX_REPAIRS, 4)
        self.assertEqual(PILOT.INITIAL_SECONDS, 600)
        self.assertEqual(PILOT.REPAIR_SECONDS, 300)

    def test_ready_requires_feedback_runtime_and_frozen_model(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            PILOT.check_ready(ready)
            missing = dict(ready)
            del missing["auth_file"]
            with self.assertRaises(SystemExit):
                PILOT.check_ready(missing)
            wrong_model = dict(ready, model="gpt-6-sol")
            with self.assertRaises(SystemExit):
                PILOT.check_ready(wrong_model)
            with self.assertRaises(SystemExit):
                PILOT.check_ready(dict(ready, approval_review="bypass"))

    def test_prepare_freezes_same_skills_and_distinct_feedback_prompt(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, ready_path = ready_fixture(root)
            font = root / "font.ttf"
            font.write_bytes(b"font")
            pilot_root = root / "pilot"
            PILOT.prepare(pilot_root, ready_path, font)
            source = (pilot_root / "trials/kinetic-type-source-only-r1/prompt.txt").read_text()
            feedback = (pilot_root / "trials/kinetic-type-author-visible-feedback-r1/prompt.txt").read_text()
            self.assertIn("no Mirket MCP server", source)
            self.assertIn("Mirket video MCP route", feedback)
            source_meta = json.loads((pilot_root / "trials/kinetic-type-source-only-r1/metadata.json").read_text())
            feedback_meta = json.loads((pilot_root / "trials/kinetic-type-author-visible-feedback-r1/metadata.json").read_text())
            self.assertEqual(source_meta["skill_hashes"], feedback_meta["skill_hashes"])
            self.assertNotEqual(source_meta["codex_config_sha256"], feedback_meta["codex_config_sha256"])
            feedback_config = (pilot_root / "trials/kinetic-type-author-visible-feedback-r1/codex-home/config.toml").read_text()
            self.assertIn(f'command = {json.dumps(ready["mirket"])}', feedback_config)
            self.assertNotIn(f'command = {json.dumps(ready["node"])}', feedback_config)
            self.assertIn('"dev", "serve"', feedback_config)
            self.assertTrue((pilot_root / "trials/kinetic-type-author-visible-feedback-r1/mirket-state").is_dir())
            self.assertIn("never .feedback/", source)
            self.assertEqual(len(list((pilot_root / "trials").iterdir())), 12)
            self.assertTrue((pilot_root / "frozen/diagnostics/scripts/check-video-source.mjs").is_file())
            self.assertTrue((pilot_root / "frozen/diagnostics/adapters/dist/video-export.js").is_file())
            self.assertTrue((pilot_root / "trials/kinetic-type-source-only-r1/project/.diagnostics/adapters/dist/video-export.js").is_file())
            self.assertIn(".checks/source-manifest.json", source)
            self.assertIn("frames 0,6,12,18,24,30,35", source)

    def test_validate_rejects_extra_files_but_allows_feedback_evidence(self):
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary)
            for name, content in {
                "BRIEF.md": "brief", "NotoSans.ttf": "font",
                "index.html": '<main data-no-timeline data-width="640" data-height="360" data-duration="3"></main>',
                "film.css": "@font-face{src:url('./NotoSans.ttf')}", "film.js": "",
            }.items():
                (project / name).write_text(content)
            (project / ".feedback").mkdir()
            (project / ".feedback/frame.png").write_bytes(b"frame")
            self.assertTrue(PILOT.validate_source("kinetic-type", project, "author-visible-feedback")["passed"])
            source_only = PILOT.validate_source("kinetic-type", project, "source-only")
            self.assertFalse(source_only["passed"])
            self.assertIn("feedback evidence is out of scope for source-only", source_only["findings"])
            (project / "unscoped.txt").write_text("bad")
            result = PILOT.validate_source("kinetic-type", project)
            self.assertFalse(result["passed"])
            self.assertIn("out-of-scope files: unscoped.txt", result["findings"])
            (project / ".checks").mkdir()
            (project / ".checks/source-manifest.json").write_text("{}")
            (project / "unscoped.txt").unlink()
            self.assertTrue(PILOT.validate_source("kinetic-type", project, "author-visible-feedback")["passed"])

    def test_manifest_refuses_symlink_in_scoped_tree(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "real").write_text("x")
            (root / "link").symlink_to(root / "real")
            with self.assertRaises(SystemExit):
                PILOT.manifest(root)

    def test_feedback_evidence_is_not_inferred_from_static_oracle(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            project = root / "project"
            project.mkdir()
            events = root / "events.jsonl"
            events.write_text(json.dumps({"type": "item.completed", "item": {"type": "agent_message", "text": "view_image completed"}}) + "\n")
            self.assertEqual(PILOT.validate_feedback_evidence(project, events, ready, "source-only", "kinetic-type")["findings"], [])
            findings = PILOT.validate_feedback_evidence(project, events, ready, "author-visible-feedback", "kinetic-type")
            self.assertIn("author-visible feedback evidence directory missing", findings["findings"])
            self.assertEqual(findings["observations"]["image_view"], "unknown")

    def test_feedback_decode_uses_count_frames_and_rejects_wrong_fps(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            project = root / "project"
            evidence = project / ".feedback"
            evidence.mkdir(parents=True)
            (evidence / "result.mp4").write_bytes(b"not-a-real-video")
            ffprobe = root / "ffprobe.py"
            probe_log = root / "probe-argv"
            ffprobe.write_text("""#!/usr/bin/env python3
import json, os, pathlib, sys
pathlib.Path(os.environ['PILOT_PROBE_LOG']).write_text(' '.join(sys.argv))
print(json.dumps({'streams':[{'width':640,'height':360,'nb_read_frames':'36','avg_frame_rate':'24/1'}]}))
""")
            ffmpeg = root / "ffmpeg.py"
            ffmpeg.write_text("""#!/usr/bin/env python3
from pathlib import Path
import sys
target = Path(sys.argv[-1])
for n in range(1, 4): target.with_name(target.name.replace('%02d', f'{n:02d}')).write_bytes(b'\\x89PNG\\r\\n\\x1a\\nframe')
""")
            for script in (ffprobe, ffmpeg):
                script.chmod(script.stat().st_mode | stat.S_IXUSR)
            ready["ffprobe"] = str(ffprobe)
            ready["ffmpeg"] = str(ffmpeg)
            os.environ["PILOT_PROBE_LOG"] = str(probe_log)
            result = PILOT.validate_feedback_evidence(project, root / "events.jsonl", ready, "author-visible-feedback", "kinetic-type")
            self.assertIn("feedback render dimensions are not independently verified", result["findings"])
            self.assertEqual(result["observations"]["source_binding"], "unknown")
            self.assertIn("-count_frames", probe_log.read_text())
            os.environ.pop("PILOT_PROBE_LOG", None)

    def test_feedback_decode_does_not_reuse_stale_oracle_frames(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            project = root / "project"
            evidence = project / ".feedback"
            evidence.mkdir(parents=True)
            (evidence / "result.mp4").write_bytes(b"video")
            events = root / "events.jsonl"
            stale = root / "events-oracle-frames"
            stale.mkdir()
            for frame in PILOT.REVIEW_SAMPLE_FRAMES:
                (stale / f"frame-{frame:02d}.png").write_bytes(b"stale")
            failed = MagicMock(returncode=1, stdout="", stderr="decode failed")
            with patch.object(PILOT.subprocess, "run", return_value=failed):
                result = PILOT.validate_feedback_evidence(project, events, ready, "author-visible-feedback", "kinetic-type")
            self.assertEqual(result["observations"]["oracle_frames"], [])
            self.assertIn("decoded feedback frame missing", result["findings"])

    def test_feedback_media_timeouts_are_retained_as_findings(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            project = root / "project"
            evidence = project / ".feedback"
            evidence.mkdir(parents=True)
            (evidence / "result.mp4").write_bytes(b"video")
            timeout = subprocess.TimeoutExpired(cmd="media", timeout=30)
            with patch.object(PILOT.subprocess, "run", side_effect=timeout):
                result = PILOT.validate_feedback_evidence(project, root / "events.jsonl", ready, "author-visible-feedback", "kinetic-type")
            self.assertIn("ffprobe timed out during independent verification", result["findings"])
            self.assertIn("independent ffmpeg frame decode timed out", result["findings"])
            self.assertIn("decoded feedback frame missing", result["findings"])

    def test_ledger_rejects_repair_overrun_and_duplicate_phase(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "budget-ledger.json").write_text(json.dumps({"shared_max_repairs": 1, "repairs_used": 0, "runs": {}}))
            with self.assertRaises(SystemExit):
                PILOT.ledger_phase(root, "cell-a", "final", "started")
            PILOT.ledger_phase(root, "cell-a", "initial", "started")
            with self.assertRaises(SystemExit):
                PILOT.ledger_phase(root, "cell-a", "initial", "started")
            PILOT.ledger_phase(root, "cell-a", "initial", "completed")
            PILOT.ledger_phase(root, "cell-a", "repair", "started")
            with self.assertRaises(SystemExit):
                PILOT.ledger_phase(root, "cell-b", "repair", "started")

    def test_auto_review_uses_cli_preset_without_conflicting_sandbox_flag(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            ready["approval_review"] = "auto_review"
            child = MagicMock()
            child.communicate.return_value = (b"", b"")
            child.returncode = 0
            with patch.object(PILOT.subprocess, "Popen", return_value=child) as start:
                result = PILOT.run_model(root, "test", "initial", ready, "source-only", 1)
            args = start.call_args.args[0]
            self.assertIn("--approve-for-me", args)
            self.assertNotIn("--sandbox", args)
            self.assertNotIn("--dangerously-bypass-approvals-and-sandbox", args)
            self.assertEqual(result["command"], args)

    def test_actual_host_denial_and_startup_error_are_classified(self):
        approval, blocked = PILOT.failure_signals(b'MCP tool call requires approval, but approval policy is never', b'')
        self.assertTrue(approval)
        self.assertFalse(blocked)
        approval, blocked = PILOT.failure_signals(b'', b'required MCP servers failed to initialize: mirket_video')
        self.assertFalse(approval)
        self.assertTrue(blocked)

    def test_updater_permission_warning_and_echoed_denials_are_not_host_refusals(self):
        events = [
            {'type':'item.completed','item':{'type':'command_execution','status':'failed',
             'command':'chrome --headless','aggregated_output':'mkdir /Library/GoogleUpdater/Crashpad: Permission denied (13)'}},
            {'type':'item.started','item':{'type':'command_execution',
             'command':'echo "MCP tool call requires approval"'}},
            {'type':'item.completed','item':{'type':'agent_message',
             'text':'No approval denied; runtime unavailable is a negative fixture string.'}},
            {'type':'item.completed','item':{'type':'command_execution','status':'completed',
             'command':'cat fixture.py','aggregated_output':'requires approval'}},
            {'type':'item.completed','item':{'type':'command_execution','status':'failed',
             'aggregated_output':'echo "MCP tool call requires approval"'}},
        ]
        self.assertEqual(PILOT.failure_signals('\n'.join(map(json.dumps,events)).encode(),b''),([],[]))

    def test_failed_native_tool_permission_and_unavailable_errors_remain_visible(self):
        for error, expected in [('Permission denied',(True,False)),
                                ('MCP tool call requires approval, but approval policy is never',(True,False)),
                                ('runtime unavailable',(False,True))]:
            event={'type':'item.completed','item':{'type':'mcp_tool_call','status':'failed',
                    'arguments':{'text':'not evidence'},'error':{'message':error},'result':None}}
            signals=PILOT.failure_signals(json.dumps(event).encode(),b'')
            self.assertEqual(tuple(map(bool,signals)),expected)

    def test_success_status_does_not_hide_mcp_error_result(self):
        event={'type':'item.completed','item':{'type':'mcp_tool_call','status':'completed',
                'result':{'isError':True,'content':[{'type':'text','text':'approval denied'}]}}}
        approval,blocked=PILOT.failure_signals(json.dumps(event).encode(),b'')
        self.assertTrue(approval)
        self.assertFalse(blocked)

    def test_top_level_mcp_permission_error_is_not_lost(self):
        for value in [b'MCP tool call failed: Permission denied',
                      json.dumps({'type':'error','message':'MCP tool call failed: Permission denied'}).encode()]:
            approval,blocked=PILOT.failure_signals(value,b'')
            self.assertTrue(approval)
            self.assertFalse(blocked)

    def test_total_model_budget_stops_future_cells(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "budget-ledger.json").write_text(json.dumps({"shared_max_repairs": 4, "repairs_used": 0, "shared_max_model_seconds": 600, "model_seconds_used": 0, "model_seconds_reserved": 0, "runs": {}}))
            PILOT.reserve_model_seconds(root, 600)
            with self.assertRaises(SystemExit):
                PILOT.reserve_model_seconds(root, 1)
            PILOT.settle_model_seconds(root, 600, 12.5)
            ledger = json.loads((root / "budget-ledger.json").read_text())
            self.assertEqual(ledger["model_seconds_used"], 12.5)
            self.assertEqual(ledger["model_seconds_reserved"], 0.0)

    def test_rejected_phase_rolls_back_reserved_model_time(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "budget-ledger.json").write_text(json.dumps({
                "shared_max_repairs": 4, "repairs_used": 0,
                "shared_max_model_seconds": 600, "model_seconds_used": 0,
                "model_seconds_reserved": 0, "runs": {
                    "cell-a": {"phases": {"initial": {"status": "started"}}}
                },
            }))
            with self.assertRaises(SystemExit):
                PILOT.reserve_phase_budget(root, "cell-a", "initial", 600)
            ledger = json.loads((root / "budget-ledger.json").read_text())
            self.assertEqual(ledger["model_seconds_reserved"], 0.0)

    def test_run_refuses_changed_frozen_project_before_model_launch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, ready_path = ready_fixture(root)
            font = root / "font.ttf"
            font.write_bytes(b"font")
            pilot_root = root / "pilot"
            PILOT.prepare(pilot_root, ready_path, font)
            project = pilot_root / "trials/kinetic-type-source-only-r1/project"
            (project / "drift.txt").write_text("changed")
            with self.assertRaises(SystemExit):
                PILOT.run_cell(pilot_root, ready_path, "kinetic-type-source-only-r1")
            self.assertFalse((pilot_root / "trials/kinetic-type-source-only-r1/initial-events.jsonl").exists())

    def test_freeze_detects_protocol_runtime_and_added_skill_drift(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, ready_path = ready_fixture(root)
            font = root / "font.ttf"
            font.write_bytes(b"font")
            pilot = root / "pilot"
            PILOT.prepare(pilot, ready_path, font)
            trial = pilot / "trials/kinetic-type-source-only-r1"
            PILOT.check_ready(ready)
            PILOT.check_frozen_inputs(pilot, trial, ready, ready_path)
            for name in ("protocol.json", "runtime-inputs.json"):
                target = pilot / "frozen" / name
                original = target.read_bytes()
                target.write_bytes(original + b" ")
                with self.assertRaises(SystemExit):
                    PILOT.check_frozen_inputs(pilot, trial, ready, ready_path)
                target.write_bytes(original)
            (trial / "project/.agents/extra.md").write_text("new instructions")
            with self.assertRaises(SystemExit):
                PILOT.check_frozen_inputs(pilot, trial, ready, ready_path)

    def test_timeout_retains_each_event_once(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            for folder in ("project", "codex-home", "user-home", "tmp"):
                (root / folder).mkdir()
            executable = Path(ready["codex"])
            executable.write_text("#!/usr/bin/env python3\nimport time\nprint('{\"type\":\"test-event\"}', flush=True)\ntime.sleep(10)\n")
            executable.chmod(0o700)
            result = PILOT.run_model(root, "test", "initial", ready, "source-only", 1)
            self.assertTrue(result["timed_out"])
            self.assertLess(result["elapsed_seconds"], 6)
            self.assertEqual((root / "initial-events.jsonl").read_text().count("test-event"), 1)

    def test_run_refuses_changed_ready_binding_before_model_launch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, ready_path = ready_fixture(root)
            font = root / "font.ttf"
            font.write_bytes(b"font")
            pilot_root = root / "pilot"
            PILOT.prepare(pilot_root, ready_path, font)
            ready["ffmpeg"] = ready["ffprobe"]
            ready_path.write_text(json.dumps(ready))
            with self.assertRaises(SystemExit):
                PILOT.run_cell(pilot_root, ready_path, "kinetic-type-source-only-r1")
            self.assertFalse((pilot_root / "trials/kinetic-type-source-only-r1/initial-events.jsonl").exists())


if __name__ == "__main__":
    unittest.main()
