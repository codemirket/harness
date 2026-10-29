import importlib.util
import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch, MagicMock
from pathlib import Path

import motion_oracles as PILOT

def ready_fixture(root):
    # Only explicit native media paths are needed by these independent oracles.
    paths = {name: str(root / name) for name in ("ffprobe", "ffmpeg")}
    return paths, None


class MotionFeedbackOracleTests(unittest.TestCase):
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

    def test_snapshot_retains_regular_siblings_and_records_symlink_without_following(self):
        with tempfile.TemporaryDirectory() as temporary:
            trial = Path(temporary)
            project = trial / "project"
            project.mkdir()
            (project / "index.html").write_text("safe")
            outside = trial / "outside.txt"
            outside.write_text("secret")
            (project / "linked.txt").symlink_to(outside)
            with self.assertRaisesRegex(SystemExit, "snapshot refused unsafe entries"):
                PILOT.snapshot(trial, "initial")
            snapshot = trial / "initial-project-snapshot"
            self.assertEqual((snapshot / "index.html").read_text(), "safe")
            self.assertFalse((snapshot / "linked.txt").exists())
            failure = json.loads((trial / "initial-project-failure.json").read_text())
            self.assertEqual(failure["status"], "rejected")
            self.assertEqual(failure["entries"][0]["kind"], "symlink")
            self.assertEqual(failure["safe_files"]["index.html"], PILOT.sha(project / "index.html"))
            self.assertFalse((trial / "initial-project-manifest.json").exists())

    def test_snapshot_rejects_symlinked_directory_and_fifo_without_blocking(self):
        with tempfile.TemporaryDirectory() as temporary:
            trial = Path(temporary)
            project = trial / "project"
            project.mkdir()
            (project / "safe.txt").write_text("safe")
            external = trial / "external"
            external.mkdir()
            (external / "secret.txt").write_text("secret")
            (project / "linked-dir").symlink_to(external, target_is_directory=True)
            fifo = project / "named-pipe"
            os.mkfifo(fifo)
            with self.assertRaises(SystemExit):
                PILOT.snapshot(trial, "initial")
            snapshot = trial / "initial-project-snapshot"
            self.assertEqual((snapshot / "safe.txt").read_text(), "safe")
            self.assertFalse((snapshot / "linked-dir").exists())
            failure = json.loads((trial / "initial-project-failure.json").read_text())
            self.assertEqual({entry["kind"] for entry in failure["entries"]}, {"symlink", "special"})
            self.assertEqual((external / "secret.txt").read_text(), "secret")

    def test_snapshot_no_follow_rejects_file_replaced_after_stat(self):
        with tempfile.TemporaryDirectory() as temporary:
            trial = Path(temporary)
            project = trial / "project"
            project.mkdir()
            (project / "race.txt").write_text("safe")
            outside = trial / "outside.txt"
            outside.write_text("secret")
            original_stat = PILOT.os.stat
            swapped = False

            def stat_then_swap(path, *args, **kwargs):
                nonlocal swapped
                result = original_stat(path, *args, **kwargs)
                if path == "race.txt" and kwargs.get("dir_fd") is not None and not swapped:
                    swapped = True
                    os.unlink(path, dir_fd=kwargs["dir_fd"])
                    os.symlink(outside, path, dir_fd=kwargs["dir_fd"])
                return result

            with patch.object(PILOT.os, "stat", side_effect=stat_then_swap):
                with self.assertRaises(SystemExit):
                    PILOT.snapshot(trial, "initial")
            failure = json.loads((trial / "initial-project-failure.json").read_text())
            self.assertEqual(failure["entries"][0]["kind"], "copy_failed")
            self.assertNotIn("race.txt", failure["safe_files"])
            self.assertEqual((outside).read_text(), "secret")

    def test_snapshot_no_follow_rejects_directory_replaced_after_stat(self):
        with tempfile.TemporaryDirectory() as temporary:
            trial = Path(temporary)
            project = trial / "project"
            project.mkdir()
            (project / "race-dir").mkdir()
            (project / "race-dir/file.txt").write_text("safe")
            outside = trial / "outside"
            outside.mkdir()
            original_stat = PILOT.os.stat
            swapped = False

            def stat_then_swap(path, *args, **kwargs):
                nonlocal swapped
                result = original_stat(path, *args, **kwargs)
                if path == "race-dir" and kwargs.get("dir_fd") is not None and not swapped:
                    swapped = True
                    os.rmdir(path, dir_fd=kwargs["dir_fd"])
                    os.symlink(outside, path, dir_fd=kwargs["dir_fd"], target_is_directory=True)
                return result

            with patch.object(PILOT.os, "stat", side_effect=stat_then_swap):
                with self.assertRaises(SystemExit):
                    PILOT.snapshot(trial, "initial")
            failure = json.loads((trial / "initial-project-failure.json").read_text())
            self.assertIn(failure["entries"][0]["kind"], {"directory_open_failed", "unstatable"})
            self.assertFalse((trial / "initial-project-snapshot/race-dir").exists())

    def test_snapshot_clean_tree_writes_manifest_once(self):
        with tempfile.TemporaryDirectory() as temporary:
            trial = Path(temporary)
            project = trial / "project"
            project.mkdir()
            (project / "nested").mkdir()
            (project / "nested/file.txt").write_text("safe")
            manifest = PILOT.snapshot(trial, "initial")
            self.assertEqual(manifest, {"nested/file.txt": PILOT.sha(trial / "initial-project-snapshot/nested/file.txt")})
            self.assertTrue((trial / "initial-project-manifest.json").is_file())
            with self.assertRaises(SystemExit):
                PILOT.snapshot(trial, "initial")

    def test_snapshot_refuses_broken_destination_symlink(self):
        with tempfile.TemporaryDirectory() as temporary:
            trial = Path(temporary)
            project = trial / "project"
            project.mkdir()
            (project / "safe.txt").write_text("safe")
            (trial / "initial-project-snapshot").symlink_to(trial / "missing-target")
            with self.assertRaisesRegex(SystemExit, "snapshot already exists"):
                PILOT.snapshot(trial, "initial")

    def test_feedback_evidence_is_not_inferred_from_static_oracle(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            ready, _ = ready_fixture(root)
            project = root / "project"
            project.mkdir()
            events = root / "events.jsonl"
            events.write_text(json.dumps({"type": "message_end", "message": {"role": "assistant", "content": [{"type": "text", "text": "I inspected the image"}]}}) + "\n")
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

