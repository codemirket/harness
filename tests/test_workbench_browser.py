"""Real browser scenarios against isolated loopback fixtures, never a user session."""
import hashlib
import http.server
import json
import os
from pathlib import Path
import subprocess
import tempfile
import sys
import threading
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'scripts/workbench/browser.mjs'
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))
from lib import workbench
FIXTURE = b'''<!doctype html><meta name="viewport" content="width=device-width"><title>Fixture</title>
<style>body{margin:20px}dialog{opacity:1;transition:opacity .2s}#wide{width:200vw}@media(prefers-reduced-motion:reduce){dialog{transition:none}}</style>
<label>Name <input id="name"></label><button id="open" onclick="document.querySelector('dialog').showModal();document.querySelector('#result').textContent=document.querySelector('#name').value">Preview</button>
<p id="result">Empty</p><div id="wide">Wide fixture</div><dialog>Ready <button onclick="this.closest('dialog').close()">Close</button></dialog>
<script>if(location.pathname==='/errors'){console.error('seeded console defect');fetch('/missing');fetch('https://outside.invalid/blocked');}</script>'''


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/missing':
            self.send_response(503)
            body = b'unavailable'
        elif self.path == '/favicon.ico':
            self.send_response(204)
            body = b''
        else:
            self.send_response(200)
            body = FIXTURE
        self.send_header('Content-Type', 'text/html')
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


class BrowserWorkbenchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        tools = workbench.load_tools()
        cls.node = tools['node']
        if not cls.node or not Path(cls.node).is_file():
            raise unittest.SkipTest('Node runtime unavailable; browser integration not run')
        cls.env = workbench.environment(tools)
        # Verify availability without installing, and close the temporary browser.
        probe = "const p=require(process.env.HARNESS_NODE_MODULES?require('path').join(process.env.HARNESS_NODE_MODULES,'playwright'):'playwright');p.chromium.launch({headless:true,executablePath:process.env.HARNESS_BROWSER,timeout:10000}).then(b=>b.close()).catch(e=>{console.error(e.message);process.exit(1)})"
        check = subprocess.run([cls.node, '-e', probe], env=cls.env, text=True,
                               capture_output=True, timeout=20)
        if check.returncode:
            raise unittest.SkipTest('Playwright/Chromium unavailable; browser integration not run: ' + check.stderr[:300])
        cls.server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()
        cls.url = 'http://127.0.0.1:' + str(cls.server.server_port)

    @classmethod
    def tearDownClass(cls):
        if hasattr(cls, 'server'):
            cls.server.shutdown()
            cls.server.server_close()
            cls.thread.join(timeout=5)

    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='browser-evidence-')
        self.addCleanup(temporary.cleanup)
        self.base = Path(temporary.name).resolve()
        self.scenario = {'schema_version': 1, 'url': self.url,
                         'viewports': [{'name': 'mobile', 'width': 390, 'height': 844}],
                         'motion': ['no-preference'], 'timeout_ms': 10000,
                         'actions': [], 'assertions': []}

    def run_scenario(self, expected_code=0, environment=None):
        source = self.base / 'scenario.json'
        source.write_text(json.dumps(self.scenario), encoding='utf-8')
        output = self.base / 'evidence'
        result = subprocess.run([self.node, str(SCRIPT), '--scenario', str(source), '--output', str(output)],
                                env=environment or self.env, text=True, capture_output=True, timeout=30)
        self.assertEqual(result.returncode, expected_code, result.stdout + result.stderr)
        self.assertTrue((output / 'report.json').is_file(), result.stderr)
        return json.loads((output / 'report.json').read_text()), output, result

    def test_interactions_motion_artifacts_and_actual_hashes(self):
        self.scenario.update(motion=['no-preference', 'reduce'], actions=[
            {'op': 'fill', 'selector': '#name', 'value': 'Clay'},
            {'op': 'click', 'selector': '#open'},
            {'op': 'capture', 'label': 'opening', 'times_ms': [0, 100, 250]}], assertions=[
            {'op': 'visible', 'selector': 'dialog'},
            {'op': 'value', 'selector': '#name', 'expected': 'Clay'},
            {'op': 'text', 'selector': '#result', 'expected': 'Clay'},
            {'op': 'count', 'selector': 'dialog', 'expected': 1}])
        report, output, _ = self.run_scenario()
        self.assertEqual(report['checks'], 'pass')
        self.assertEqual(report['visual_review'], 'required')
        self.assertEqual(len(report['runs']), 2)
        self.assertEqual(len(report['artifacts']), 8)
        self.assertTrue(all(run['status'] == 'pass' for run in report['runs']))
        self.assertTrue(all(any(item['id'] == 'wide' for item in run['overflow']) for run in report['runs']))
        for artifact in report['artifacts']:
            raw = (output / artifact['file']).read_bytes()
            self.assertTrue(raw.startswith(b'\x89PNG'))
            self.assertEqual((artifact['width'], artifact['height']), (390, 844))
            self.assertTrue(any(item['id'] == 'wide' for item in artifact['overflow_candidates']))
            self.assertEqual(hashlib.sha256(raw).hexdigest(), artifact['sha256'])
            self.assertGreaterEqual(artifact['completed_offset_ms'], artifact['actual_offset_ms'])
        self.assertEqual(report['scenario_sha256'], hashlib.sha256((self.base / 'scenario.json').read_bytes()).hexdigest())

    def test_failed_state_assertion_remains_red_and_preserves_capture(self):
        self.scenario['assertions'] = [{'op': 'text', 'selector': '#result', 'expected': 'Wrong'}]
        report, _, _ = self.run_scenario(1)
        self.assertEqual(report['status'], 'fail')
        self.assertFalse(report['runs'][0]['assertions'][0]['pass'])
        self.assertEqual(report['runs'][0]['assertions'][0]['actual'], 'Empty')
        self.assertEqual(len(report['artifacts']), 1)

    def test_http_console_and_policy_failures_are_separate(self):
        self.scenario.update(url=self.url + '/errors', actions=[{'op': 'wait', 'ms': 200}])
        report, _, _ = self.run_scenario(1)
        run = report['runs'][0]
        self.assertTrue(any('seeded console defect' in line for line in run['console_errors']))
        self.assertTrue(any(item['status'] == 503 for item in run['http_errors']))
        self.assertTrue(any('outside.invalid' in url for url in run['blocked_requests']))
        self.assertTrue(any('outside.invalid' in item['url'] for item in run['failed_requests']))

    def test_capture_without_assertions_is_explicitly_limited(self):
        report, _, _ = self.run_scenario()
        self.assertEqual(report['checks'], 'limited')
        self.assertEqual(report['visual_review'], 'required')
        self.assertTrue(any('No state assertions' in text for text in report['limits']))

    def test_invalid_nonloopback_scenario_reports_blocked_without_launch(self):
        self.scenario['url'] = 'https://localhost.attacker.invalid/'
        report, _, _ = self.run_scenario(2)
        self.assertEqual(report['status'], 'blocked')
        self.assertIn('loopback', report['error'])
        self.assertEqual(report['artifacts'], [])

    def test_missing_runtime_is_blocked_not_a_green_empty_run(self):
        environment = dict(self.env, HARNESS_NODE_MODULES=str(self.base / 'absent'))
        report, _, _ = self.run_scenario(2, environment)
        self.assertEqual(report['status'], 'blocked')
        self.assertEqual(report['checks'], 'not_run')
        self.assertIn('No installation attempted', report['error'])

    def test_existing_output_is_preserved(self):
        report, output, _ = self.run_scenario()
        before = {p.name: p.read_bytes() for p in output.iterdir()}
        result = subprocess.run([self.node, str(SCRIPT), '--scenario', str(self.base / 'scenario.json'),
                                 '--output', str(output)], env=self.env, text=True, capture_output=True, timeout=15)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(before, {p.name: p.read_bytes() for p in output.iterdir()})

    def test_timeout_keeps_failure_report_and_closes_browser(self):
        self.scenario.update(timeout_ms=1000, actions=[{'op': 'click', 'selector': '#never-exists'}])
        report, _, _ = self.run_scenario(1)
        self.assertEqual(report['status'], 'fail')
        self.assertEqual(report['checks'], 'fail')
        self.assertIn('finished_at', report)

    def test_malformed_capture_and_screenshot_budget_are_rejected(self):
        cases = [
            {'viewports': [{'width': 390, 'height': 844}]},
            {'actions': [{'op': 'capture', 'times_ms': [0]}]},
            {'actions': [{'op': 'capture', 'label': 'sample', 'times_ms': [100, 0]}]},
            {'actions': [{'op': 'evaluate', 'value': 'process.exit(0)'}]},
            {'actions': [{'op': 'capture', 'label': 'sample', 'times_ms': list(range(8))}] * 11},
        ]
        for index, changes in enumerate(cases):
            with self.subTest(changes=changes):
                scenario = dict(self.scenario, **changes)
                source = self.base / f'invalid-{index}.json'
                source.write_text(json.dumps(scenario), encoding='utf-8')
                output = self.base / f'invalid-output-{index}'
                result = subprocess.run([self.node, str(SCRIPT), '--scenario', str(source), '--output', str(output)],
                                        env=self.env, text=True, capture_output=True, timeout=10)
                self.assertEqual(result.returncode, 2)
                report = json.loads((output / 'report.json').read_text())
                self.assertEqual(report['status'], 'blocked')
                self.assertEqual(report['artifacts'], [])
