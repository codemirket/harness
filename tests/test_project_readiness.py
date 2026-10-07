"""Behavior checks for opt-in project readiness, without installed tools/services."""
import contextlib
import io
import http.client
import json
from pathlib import Path
import struct
import sys
import tempfile
import threading
import time
import zlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from lib import runtime


class ProjectReadinessTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='project readiness ')
        self.addCleanup(self.tmp.cleanup)
        self.project = Path(self.tmp.name)
        self.browser = {'state': 'available', 'path': str(self.project / 'browser'), 'launch_verified': False}
        self.env = {'PATH': ''}

    def package(self, **values):
        (self.project / 'package.json').write_text(json.dumps(values))

    def report(self, **kwargs):
        with mock.patch.object(runtime, 'discover_browser', return_value=dict(self.browser)):
            return runtime.diagnose_project(self.project, env=self.env, **kwargs)

    def test_missing_node_or_manager_blocks_prerequisites_without_running_scripts(self):
        self.package(packageManager='pnpm@11.0.0', scripts={'dev': 'touch should-not-exist'})
        with mock.patch.object(runtime, 'run_probe') as run:
            report = self.report()
        run.assert_not_called()
        self.assertFalse(report['prerequisites_ready'])
        self.assertFalse(report['requested_checks_passed'])
        self.assertEqual(report['available_scripts'], ['dev'])
        for name in ('startup', 'build', 'render'):
            self.assertEqual(report[name]['state'], 'not_checked')
        self.assertFalse((self.project / 'should-not-exist').exists())

    def test_declared_manager_takes_precedence_over_other_lockfiles(self):
        self.package(packageManager='yarn@4.0.0', engines={'node': '>=24'})
        (self.project / 'pnpm-lock.yaml').touch()
        with mock.patch.object(runtime, 'diagnose_tool', return_value={
                'state': 'available', 'path': '/tool', 'version': '4.0.0'}) as tool:
            report = self.report()
        self.assertEqual(set(report['tools']), {'node', 'yarn'})
        self.assertEqual(report['declared_engines'], {'node': '>=24'})
        self.assertEqual([call.args[0] for call in tool.call_args_list], ['node', 'yarn'])
        self.assertTrue(all(call.kwargs['cwd'] == self.project.resolve() for call in tool.call_args_list))

    def test_lockfile_manager_missing_dependencies_and_optional_browser(self):
        self.package(devDependencies={'example': '1.0.0'})
        (self.project / 'pnpm-lock.yaml').touch()
        with mock.patch.object(runtime, 'diagnose_tool', return_value={
                'state': 'available', 'path': '/tool', 'version': '1.0.0'}):
            self.assertFalse(self.report()['prerequisites_ready'])
            (self.project / 'node_modules').mkdir()
            self.browser['state'] = 'not_found'
            report = self.report()
        self.assertEqual(set(report['tools']), {'node', 'pnpm'})
        self.assertTrue(report['requested_checks_passed'])
        self.assertFalse(report['browser']['launch_verified'])

    def test_unsupported_manager_is_not_executed(self):
        self.package(packageManager='malicious-command@1.0.0')
        with mock.patch.object(runtime, 'diagnose_tool', return_value={
                'state': 'available', 'path': '/node', 'version': '24.0.0'}) as tool:
            report = self.report()
        self.assertEqual(tool.call_count, 1)
        self.assertEqual(report['tools']['malicious-command']['state'], 'unsupported_package_manager')
        self.assertFalse(report['requested_checks_passed'])

    def test_invalid_manifest_and_invalid_capture_options_fail_before_probes(self):
        (self.project / 'package.json').write_text('[]')
        with mock.patch.object(runtime, 'run_probe') as run, self.assertRaises(ValueError):
            self.report()
        run.assert_not_called()
        with self.assertRaises(ValueError): self.report(screenshot=self.project / 'new.png')
        with self.assertRaises(ValueError): self.report(url='http://127.0.0.1:3000', screenshot=self.project / 'new.jpg')
        image = self.project / 'existing.png'
        image.write_bytes(b'preserve')
        with self.assertRaises(ValueError): self.report(url='http://127.0.0.1:3000', screenshot=image)
        self.assertEqual(image.read_bytes(), b'preserve')

    def test_known_tool_version_and_no_download_environment(self):
        with mock.patch.object(runtime, 'path_lookup', return_value='/tool'), \
                mock.patch.object(runtime, 'run_probe', return_value={
                    'status': 'ok', 'text': 'git version 2.50.1 (Apple Git-155)', 'exit_code': 0}) as run:
            report = runtime.diagnose_tool('git', self.env, 5)
        self.assertEqual(report['version'], '2.50.1')
        self.assertEqual(report['state'], 'available')
        self.assertEqual(run.call_args.kwargs['env']['COREPACK_ENABLE_NETWORK'], '0')
        self.assertEqual(run.call_args.args[0], ['/tool', '--version'])

    def test_failed_or_unrecognized_version_does_not_claim_availability(self):
        with mock.patch.object(runtime, 'path_lookup', return_value='/tool'):
            for status, text in [('timeout', '24.0.0'), ('nonzero', '24.0.0'), ('ok', 'secret-value')]:
                with self.subTest(status=status), mock.patch.object(runtime, 'run_probe', return_value={
                        'status': status, 'text': text, 'exit_code': 0}):
                    report = runtime.diagnose_tool('node', self.env, 5)
                self.assertNotEqual(report['state'], 'available')
                self.assertNotIn(text, json.dumps(report))

    def test_local_urls_reject_external_access_credentials_and_sensitive_query(self):
        for url in ('http://127.0.0.1:3000/', 'https://localhost:3000/path', 'http://[::1]:3000/'):
            self.assertEqual(runtime.local_url(url), url)
        for url in ('https://example.com', 'file:///tmp/page', 'http://0.0.0.0:3000',
                    'http://user:password@localhost:3000', 'http://localhost:3000/?token=secret',
                    'http://localhost:3000/#secret', 'http://localhost:bad', 'http://[broken'):
            with self.subTest(url=url), self.assertRaises(ValueError): runtime.local_url(url)

    def test_http_response_errors_and_redirect_scope_on_real_loopback_server(self):
        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                if self.path == '/redirect':
                    self.send_response(302); self.send_header('Location', '/'); self.end_headers()
                elif self.path == '/external':
                    self.send_response(302); self.send_header('Location', 'https://example.com'); self.end_headers()
                else:
                    self.send_response(404 if self.path == '/missing' else 200); self.end_headers()

            def log_message(self, *args): pass

        server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        base = 'http://127.0.0.1:' + str(server.server_port)
        with mock.patch.dict('os.environ', {'HTTP_PROXY': 'http://127.0.0.1:1'}):
            self.assertEqual(runtime.probe_url(base + '/redirect', 1)['state'], 'responding')
        self.assertEqual(runtime.probe_url(base + '/missing', 1)['http_status'], 404)
        self.assertNotEqual(runtime.probe_url(base + '/external', 1)['state'], 'responding')

    def test_unreachable_app_skips_capture_and_fails_requested_checks(self):
        with mock.patch.object(runtime, 'probe_url', return_value={'state': 'unreachable', 'http_status': None}), \
                mock.patch.object(runtime, 'probe_render') as capture:
            report = self.report(url='http://127.0.0.1:3000', screenshot=self.project / 'new.png')
        capture.assert_not_called()
        self.assertEqual(report['render']['state'], 'startup_unavailable')
        self.assertFalse(report['requested_checks_passed'])

    def test_malformed_http_response_remains_a_failed_check(self):
        with mock.patch.object(runtime.urllib.request, 'build_opener') as opener:
            opener.return_value.open.side_effect = http.client.BadStatusLine('malformed')
            self.assertEqual(runtime._probe_url_once('http://127.0.0.1:3000', 1)['state'], 'unreachable')

    def test_http_wall_deadline_handles_slow_headers(self):
        class SlowHandler(BaseHTTPRequestHandler):
            def do_GET(self):
                try:
                    for byte in b'HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n':
                        self.connection.sendall(bytes([byte]))
                        time.sleep(.02)
                except OSError:
                    pass
            def log_message(self, *args): pass

        server = ThreadingHTTPServer(('127.0.0.1', 0), SlowHandler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        start = time.monotonic()
        report = runtime.probe_url('http://127.0.0.1:' + str(server.server_port), .2)
        self.assertEqual(report['state'], 'timeout')
        self.assertLess(time.monotonic() - start, 2)

    def test_windows_deadline_targets_only_probe_pid_and_its_descendants(self):
        process = mock.Mock(pid=123)
        with mock.patch.object(runtime.subprocess, 'run') as kill:
            runtime.stop_process(process, {}, windows=True)
        self.assertEqual(kill.call_args.args[0][1:], ['/PID', '123', '/T', '/F'])
        self.assertEqual(kill.call_args.kwargs['timeout'], 2)
        self.assertFalse(kill.call_args.kwargs['shell'])
        process.kill.assert_called_once()

    def capture(self, blob, *, text='<html><body>private fixture</body></html>', status='ok'):
        output = self.project / 'capture.png'

        def run(argv, **kwargs):
            path = Path(next(arg.split('=', 1)[1] for arg in argv if arg.startswith('--screenshot=')))
            path.write_bytes(blob)
            self.assertNotIn('--no-sandbox', argv)
            self.assertTrue(any(arg.startswith('--user-data-dir=') for arg in argv))
            return {'status': status, 'exit_code': 0, 'text': text}

        with mock.patch.object(runtime, 'run_probe', side_effect=run):
            report = runtime.probe_render('http://127.0.0.1:3000', self.browser, output, self.env, 5)
        return report, output

    @staticmethod
    def png(width=1280, height=900):
        def chunk(kind, data):
            return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
        return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
                + chunk(b'IDAT', zlib.compress((b'\x00' + b'\xff\xff\xff' * width) * height))
                + chunk(b'IEND', b''))

    def test_capture_is_new_bounded_and_not_a_visual_review(self):
        blob = self.png()
        report, output = self.capture(blob)
        self.assertEqual(report['state'], 'captured')
        self.assertEqual(output.read_bytes(), blob)
        self.assertEqual(report['visual_review'], 'not_performed')
        self.assertNotIn('private fixture', json.dumps(report))
        self.assertEqual(len(report['sha256']), 64)

    def test_capture_rejects_jpeg_wrong_dimensions_browser_error_and_output_limit(self):
        cases = [(b'\xff\xd8\xff' + b'0' * 40, {}, 'invalid_capture'),
                 (self.png()[:24], {}, 'invalid_capture'),
                 (self.png(412, 900), {}, 'unexpected_dimensions'),
                 (self.png(), {'text': '<html>chrome-error://chromewebdata</html>'}, 'page_error'),
                 (self.png(), {'status': 'output_limit'}, 'output_limit')]
        for blob, kwargs, expected in cases:
            with self.subTest(expected=expected):
                report, output = self.capture(blob, **kwargs)
                self.assertEqual(report['state'], expected)
                self.assertFalse(output.exists())

    def test_cli_exit_and_overall_readiness_include_requested_project_check(self):
        with mock.patch.object(runtime, 'diagnose_project', return_value={'requested_checks_passed': False}), \
                mock.patch.object(runtime, 'diagnose', return_value={'ready': True, 'authentication_attention': []}), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            code = runtime.main(['doctor', '--project', str(self.project), '--json'])
        self.assertEqual(code, 1)
        self.assertFalse(json.loads(output.getvalue())['ready'])
