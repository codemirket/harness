"""Compatibility wrappers preserve target dispatch, root discovery and failure exit codes."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SetupWrapperTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='harness wrapper ')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.repo = self.base / 'repository with spaces'
        self.repo.mkdir()
        self.home = self.base / 'isolated home'
        self.home.mkdir()
        self.entry = self.repo / 'ai.py'
        self.entry.write_text('import json,sys\nprint(json.dumps(sys.argv[1:]))\n')

    def run_wrapper(self, args, override=True, script=None):
        # Apple Python otherwise writes its own stdlib bytecode cache to HOME.
        env = dict(os.environ, HOME=str(self.home), PYTHONDONTWRITEBYTECODE='1')
        env.pop('AI_SHARED_DIR', None)
        if override:
            env['AI_SHARED_DIR'] = str(self.repo)
        return subprocess.run(['sh', str(script or ROOT / 'setup/macos.sh')] + args,
                              cwd=self.base, env=env, text=True, capture_output=True)

    def test_dispatches_each_target_without_shell_interpolation(self):
        for target in ('codex', 'claude', 'both'):
            result = self.run_wrapper([target])
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout), ['sync', '--target', target])
            self.assertEqual(list(self.home.iterdir()), [])

    def test_discovers_checkout_from_own_location(self):
        (self.repo / 'setup').mkdir()
        wrapper = self.repo / 'setup/macos.sh'
        shutil.copyfile(ROOT / 'setup/macos.sh', wrapper)
        result = self.run_wrapper(['both'], override=False, script=wrapper)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout), ['sync', '--target', 'both'])

    def test_rejects_invalid_target_or_argument_count(self):
        for args in ([], ['other'], ['codex', 'extra'], ['codex;touch injected']):
            self.assertEqual(self.run_wrapper(args).returncode, 2)
        self.assertFalse((self.base / 'injected').exists())

    def test_missing_checkout_and_harness_errors_propagate(self):
        self.entry.unlink()
        self.assertNotEqual(self.run_wrapper(['codex']).returncode, 0)
        self.entry.write_text('raise SystemExit(7)\n')
        self.assertEqual(self.run_wrapper(['codex']).returncode, 7)


if __name__ == '__main__':
    unittest.main()
