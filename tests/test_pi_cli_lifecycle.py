"""Lifecycle driver checks with a synthetic PTY peer and disposable private state."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


DRIVER = Path(__file__).with_name('pi-cli-lifecycle.py').resolve()
PEER = r'''
import os, select, sys, time, tty
from pathlib import Path
tty.setraw(0)
def output(text):
 os.write(1,text.encode())
if TRUST:
 output('Trust project \x1b[32mfolder?\x1b[0m')
 while not os.read(0,1).endswith(b'\r'):pass
output('\x1b[32m[Exten')
time.sleep(.05)
output('sions]\x1b[0m provider.mjs')
command=bytearray()
while True:
 value=os.read(0,1)
 if value==b'\x04':break
 if value==b'\x1b':
  output('early menu redraw')
  if select.select([0],[],[],.1)[0]:raise RuntimeError('Command arrived before editor focus restored')
  output('\x1b[7m \x1b[27m');continue
 if value!=b'\r':
  command.extend(value);continue
 text=command.decode();command.clear()
 with Path(__file__).with_suffix('.commands').open('a') as log:log.write(text+'\n')
 if text=='/mcp':output('MCP servers: mirket connected · \x1b[32m25 tools\x1b[0m')
 elif text=='/reload':
  if HANG_RELOAD:
   output('Reload still pending');time.sleep(30)
  if FAIL_RELOAD:
   output('Reload failed: synthetic failure');sys.exit(3)
  time.sleep(RELOAD_DELAY)
  output('Reloaded keybindings, extensions, skills,\r\n prompts, themes, and context files')
 elif text=='/mirket doctor':output('{"model": "host-owned", "nativeOnly": false}')
 else:raise RuntimeError('Unexpected command: '+text)
'''


class LifecycleDriverTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(
            prefix='mirket-lifecycle-driver-test-',
            dir='/private/tmp' if os.path.isdir('/private/tmp') else '/tmp')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        for name in ('home', 'agent', 'sessions', 'project', 'peers'):
            (self.root / name).mkdir(mode=0o700)

    def run_driver(self, observations, *, trust=False, delay=0, fail_reload=False, hang_reload=False):
        report = {'entry': 'synthetic-peer', 'cases': ['prior functional evidence'],
                  'first_tool_ms': observations}
        self.report_path = self.root / 'cli-lifecycle-report.json'
        self.report_path.write_text(json.dumps(report))
        peer = self.root / 'synthetic-node'
        peer.write_text(f'#!{sys.executable}\nTRUST={trust!r}\n'
                        f'RELOAD_DELAY={delay!r}\nFAIL_RELOAD={fail_reload!r}\n'
                        f'HANG_RELOAD={hang_reload!r}\n' + PEER)
        peer.chmod(0o700)
        return subprocess.run(
            [sys.executable, str(DRIVER), '--node', str(peer), '--peers',
             str(self.root / 'peers'), '--state-root', str(self.root), '--only', 'tui'],
            cwd=self.root / 'project', capture_output=True, text=True, timeout=30,
            env={**os.environ, 'HOME': str(self.root / 'home')})

    def test_budget_boundary_passes_and_preserves_evidence(self):
        result = self.run_driver([5000, 2499, 2500])
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(self.report_path.read_text())
        self.assertTrue(report['startup_budget_pass'])
        self.assertEqual(report['first_tool_ms'], [5000, 2499, 2500])
        self.assertEqual(report['startup_budget_ms'],
                         {'first_observed_process': 5000, 'repeat_process': 2500})
        self.assertEqual(report['prior_first_tool_budget_ms'], 1500)
        self.assertFalse(report['prior_1500_ms_budget_pass'])
        self.assertEqual([sample['phase'] for sample in report['startup_budget_results']],
                         ['first_observed_process', 'repeat_process', 'repeat_process'])
        self.assertEqual(report['cases'][0], 'prior functional evidence')
        self.assertIn('interactive PTY', report['cases'][-1])
        self.assertEqual((self.root / 'synthetic-node.commands').read_text(),
                         '/mcp\n/reload\n/mirket doctor\n')
        self.assertEqual(len(list(self.root.glob('tui-transcript-*.txt'))), 1)

    def test_over_budget_fails_after_functional_success_and_slow_reload(self):
        # A completion later than the previous four-second sleep must still be observed.
        result = self.run_driver([5000, 2500, 2500.01], trust=True, delay=4.2)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('FAIL startup budget', result.stderr)
        report = json.loads(self.report_path.read_text())
        self.assertFalse(report['startup_budget_pass'])
        self.assertEqual(report['first_tool_ms'], [5000, 2500, 2500.01])
        self.assertIn('interactive PTY', report['cases'][-1])
        self.assertIn('"startup_budget_pass": false', result.stdout)
        transcript = next(self.root.glob('tui-transcript-*.txt')).read_bytes()
        self.assertIn(b'Trust project \x1b[32mfolder?', transcript)
        self.assertIn(b'Reloaded keybindings', transcript)
        self.assertIn(b'\x1b[32m', transcript)

    def test_first_process_over_budget_fails(self):
        result = self.run_driver([5000.01, 1800, 1800])
        self.assertNotEqual(result.returncode, 0)
        report = json.loads(self.report_path.read_text())
        self.assertFalse(report['startup_budget_pass'])
        self.assertEqual([sample['pass'] for sample in report['startup_budget_results']],
                         [False, True, True])

    def test_incomplete_or_invalid_observations_cannot_pass(self):
        for observations in ([], [1000], [1000, 1000], [1000] * 4,
                             [1000, -1, 1000], [1000, float('nan'), 1000],
                             [1000, float('inf'), 1000], [1000, True, 1000]):
            with self.subTest(observations=observations):
                result = self.run_driver(observations)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn('FAIL startup budget', result.stderr)
                self.assertFalse(json.loads(self.report_path.read_text())['startup_budget_pass'])

    def test_early_exit_retains_transcript_and_names_missing_state(self):
        result = self.run_driver([1000], fail_reload=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('waiting for reload completion', result.stderr)
        self.assertIn('transcript:', result.stderr)
        self.assertIn('synthetic failure', result.stderr)
        transcript = next(self.root.glob('tui-transcript-*.txt')).read_bytes()
        self.assertIn(b'Reload failed: synthetic failure', transcript)
        self.assertEqual(json.loads(self.report_path.read_text())['cases'],
                         ['prior functional evidence'])

    def test_missing_completion_times_out_and_retains_raw_output(self):
        result = self.run_driver([1000], hang_reload=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('timed out waiting for reload completion', result.stderr)
        self.assertIn('Reload still pending', result.stderr)
        transcript = next(self.root.glob('tui-transcript-*.txt')).read_bytes()
        self.assertIn(b'Reload still pending', transcript)


if __name__ == '__main__':
    unittest.main()
