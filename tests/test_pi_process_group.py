"""Failure injection for disposable Pi fixture process-group cleanup."""

import subprocess
import sys
import tempfile
import time
import unittest

from pi_process_group import active_group_members, terminate_owned_group


class ProcessGroupCleanupTest(unittest.TestCase):
    def test_leader_exit_and_timeout_both_reap_owned_descendants(self):
        with tempfile.TemporaryDirectory(prefix="mirket-pi-package-") as marker:
            for leader_waits in (False, True):
                source = (
                    "import subprocess,sys,time; "
                    "subprocess.Popen([sys.executable,'-c','import time;time.sleep(30)',sys.argv[1]]); "
                    + ("time.sleep(30)" if leader_waits else "time.sleep(0.2)")
                )
                process = subprocess.Popen(
                    [sys.executable, "-c", source, marker], start_new_session=True,
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                )
                try:
                    if not leader_waits:
                        process.wait(timeout=3)
                    deadline = time.monotonic() + 3
                    while not active_group_members(process.pid) and time.monotonic() < deadline:
                        time.sleep(0.05)
                    self.assertTrue(active_group_members(process.pid))
                finally:
                    # The live leader may replace its process title with "pi".
                    # The exited-leader case must rely on the marked descendant.
                    result = terminate_owned_group(process, [marker] if not leader_waits else ["absent-marker"])
                self.assertFalse(result["refused_unowned_group"])
                self.assertEqual(result["after"], [])


if __name__ == "__main__":
    unittest.main()
