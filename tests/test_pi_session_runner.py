"""Failure injection for the bounded installed-session fixture supervisor."""

import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

RUNNER_SPEC = importlib.util.spec_from_file_location(
    "pi_session_authorization_runner",
    Path(__file__).with_name("pi-session-authorization-runner.py"),
)
assert RUNNER_SPEC is not None and RUNNER_SPEC.loader is not None
RUNNER = importlib.util.module_from_spec(RUNNER_SPEC)
RUNNER_SPEC.loader.exec_module(RUNNER)


class SessionRunnerCleanupTest(unittest.TestCase):
    def test_timeout_and_exited_leader_reap_owned_child_only(self):
        with tempfile.TemporaryDirectory(prefix="mirket-pi-package-", dir="/private/tmp") as temp:
            marker = str(Path(temp) / "owned-marker")
            unrelated = subprocess.Popen(
                [sys.executable, "-c", "import time;time.sleep(30)"],
                start_new_session=True, stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            try:
                for label, linger, expected in (("timeout", True, "timeout"),
                                                ("exited-leader", False, "leader_exit")):
                    source = (
                        "import subprocess,sys,time;"
                        "subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)',sys.argv[1]]);"
                        "print('owned child started',flush=True);"
                        + ("time.sleep(30)" if linger else "time.sleep(0.2)")
                    )
                    evidence = Path(temp) / label
                    report = RUNNER.run_group(
                        [sys.executable, "-c", source, marker], evidence,
                        timeout_seconds=0.8, markers=[marker], max_log_bytes=4096,
                    )
                    self.assertEqual(report["cause"], expected, report)
                    self.assertFalse(report["passed"], report)
                    self.assertFalse(report["cleanup"]["refused_unowned_group"], report)
                    self.assertEqual(report["cleanup"]["after"], [], report)
                    self.assertIn("owned child started", (evidence / "process.log").read_text())
                    self.assertEqual(json.loads((evidence / "supervisor.json").read_text()), report)
                    self.assertIsNone(unrelated.poll(), "Cleanup killed an unrelated group")
                    print(json.dumps({"injection": label, "cause": report["cause"],
                                      "exitCode": report["exitCode"],
                                      "logBytes": report["logBytes"],
                                      "cleanup": report["cleanup"],
                                      "unrelatedAlive": unrelated.poll() is None}), flush=True)
            finally:
                unrelated.terminate()
                unrelated.wait(timeout=3)


if __name__ == "__main__":
    unittest.main()
