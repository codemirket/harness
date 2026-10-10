"""Runtime diagnostics verify observable states without real auth or installations."""
import io
import json
import os
from pathlib import Path
import plistlib
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from lib import runtime


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='runtime tests ')
        self.addCleanup(self.tmp.cleanup)
        self.home = Path(self.tmp.name).resolve()
        self.codex = self.home / 'with spaces/codex'
        self.codex.parent.mkdir()
        self.codex.write_text('fixture; not executable')
        self.claude = self.home / 'with spaces/claude'
        self.claude.write_text('fixture; not executable')
        self.missing = self.home / 'missing'
        self.env = {'PATH': ''}

    def probe_cli(self, name='codex', **kwargs):
        return runtime.diagnose_cli(name, home=self.home, env=self.env,
                                   platform=runtime.platform_name(), explicit=getattr(self, name), **kwargs)

    def test_doctor_exit_reports_readiness_and_authentication_attention(self):
        report = {'ready': True, 'authentication_attention': [], 'runtimes': {}}
        with mock.patch.object(runtime, 'diagnose', return_value=report), \
             mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(runtime.main(['doctor', '--json']), 0)
            report['ready'] = False
            self.assertEqual(runtime.main(['doctor', '--json']), 1)
            report['ready'] = True
            report['authentication_attention'] = ['codex']
            self.assertEqual(runtime.main(['doctor', '--json', '--check-auth']), 1)

    def test_missing_explicit_path_does_not_fall_back_or_execute(self):
        with mock.patch.object(runtime, 'run_probe') as run:
            report = runtime.diagnose(home=self.home, codex=self.missing, claude=self.missing,
                                      codex_desktop=self.missing, claude_desktop=self.missing, env=self.env)
        run.assert_not_called()
        for name in ('codex', 'claude'):
            self.assertFalse(report['runtimes'][name]['present'])
            self.assertIsNone(report['runtimes'][name]['can_execute'])
            self.assertIn('https://', report['guidance'][name]['url'])

    def test_version_identity_and_space_path_passed_as_single_argument(self):
        with mock.patch.object(runtime, 'run_probe', return_value={
                'status': 'ok', 'exit_code': 0, 'text': 'codex-cli 0.160.1\n'}) as run:
            report = self.probe_cli()
        self.assertEqual(run.call_args.args[0], [str(self.codex), '--version'])
        self.assertEqual(report['version'], '0.160.1')
        self.assertTrue(report['can_execute'])
        self.assertFalse(report['on_path'])
        self.assertEqual(report['auth']['state'], 'not_checked')

    def test_timeout_nonzero_and_unrecognized_version_are_distinct(self):
        for status, code, text, expected in (
            ('timeout', -9, '', 'timeout'), ('nonzero', 1, 'secret error', 'nonzero'),
            ('launch_failed', None, '', 'launch_failed'), ('ok', 0, 'different tool 1.2.3', 'unrecognized_version')):
            with self.subTest(status=status), mock.patch.object(runtime, 'run_probe', return_value={
                    'status': status, 'exit_code': code, 'text': text}):
                report = self.probe_cli(check_auth=True)
            self.assertEqual(report['probe_status'], expected)
            self.assertIsNone(report['version'])
            self.assertNotIn('secret', json.dumps(report))

    def test_path_and_known_bundled_candidate_are_reported_separately(self):
        with mock.patch.object(runtime, 'path_lookup', return_value=None), \
             mock.patch.object(runtime, 'known_cli_paths', return_value=[self.codex]), \
             mock.patch.object(runtime, 'run_probe', return_value={'status': 'ok', 'exit_code': 0, 'text': 'codex-cli 1.2.3'}):
            report = runtime.diagnose_cli('codex', home=self.home, env=self.env, platform=runtime.platform_name())
        self.assertEqual(report['discovered_via'], 'known_location')
        self.assertFalse(report['on_path'])
        self.assertTrue(report['present'])
        self.assertEqual(report['version'], '1.2.3')

    def test_auth_allowlists_drop_account_identity_and_tokens(self):
        text = json.dumps({'loggedIn': True, 'authMethod': 'claude.ai', 'email': 'private@example.com',
                           'apiKey': 'secret-value', 'organizationId': 'private-org', 'configDirectory': '/private'})
        report = runtime.auth_result('claude', {'status': 'ok', 'exit_code': 0, 'text': text})
        self.assertEqual(report, {'state': 'authenticated', 'method': 'claude.ai', 'probe_status': 'ok'})
        codex = runtime.auth_result('codex', {'status': 'ok', 'exit_code': 0,
                                             'text': 'Logged in using an API key - sk-secret\n'})
        self.assertEqual(codex, {'state': 'authenticated', 'probe_status': 'ok'})
        for text in ('unknown error private@example.com', '{"loggedIn":true} garbage'):
            report = runtime.auth_result('claude', {'status': 'nonzero', 'exit_code': 1, 'text': text})
            self.assertEqual(report['state'], 'unknown')
            self.assertNotIn('private', json.dumps(report))

    def test_auth_not_logged_in_and_failed_probe_are_not_confused(self):
        self.assertEqual(runtime.auth_result('codex', {'status': 'nonzero', 'exit_code': 1,
            'text': 'Not logged in\n'})['state'], 'not_authenticated')
        self.assertEqual(runtime.auth_result('claude', {'status': 'nonzero', 'exit_code': 1,
            'text': '{"loggedIn":false,"authMethod":"none"}'})['state'], 'not_authenticated')
        self.assertEqual(runtime.auth_result('codex', {'status': 'timeout', 'exit_code': -9,
            'text': 'Logged in using ChatGPT'})['state'], 'unknown')

    def test_auth_uses_only_status_command_after_verified_version(self):
        replies = [{'status': 'ok', 'exit_code': 0, 'text': '2.1.287 (Claude Code)'},
                   {'status': 'ok', 'exit_code': 0, 'text': '{"loggedIn":true,"authMethod":"oauth_token"}'}]
        with mock.patch.object(runtime, 'run_probe', side_effect=replies) as run:
            report = self.probe_cli('claude', check_auth=True)
        self.assertEqual(run.call_args_list[1].args[0], [str(self.claude), 'auth', 'status', '--json'])
        self.assertEqual(report['auth']['state'], 'authenticated')

    def test_discovery_home_override_does_not_claim_auth_for_that_user(self):
        with mock.patch.object(runtime, 'run_probe', return_value={'status': 'ok', 'exit_code': 0, 'text': 'codex-cli 1.2.3'}) as run:
            report = runtime.diagnose(home=self.home, codex=self.codex, claude=self.missing,
                                      codex_desktop=self.missing, claude_desktop=self.missing, env=self.env, check_auth=True)
        self.assertEqual(run.call_count, 1)
        self.assertEqual(report['runtimes']['codex']['auth']['reason'], 'discovery_home_override')

    def test_mac_desktop_bundle_identity_and_version_without_launch(self):
        app = self.home / 'ChatGPT.app'
        (app / 'Contents/MacOS').mkdir(parents=True)
        (app / 'Contents/MacOS/ChatGPT').write_text('not launched')
        metadata = {'CFBundleIdentifier': 'com.openai.codex', 'CFBundleShortVersionString': '26.930.61225',
                    'CFBundleExecutable': 'ChatGPT'}
        (app / 'Contents/Info.plist').write_bytes(plistlib.dumps(metadata))
        with mock.patch.object(runtime, 'run_probe') as run:
            report = runtime.diagnose_desktop('codex', home=self.home, env=self.env, platform='macos', explicit=app)
        run.assert_not_called()
        self.assertEqual(report['identity'], 'com.openai.codex')
        self.assertEqual(report['version'], '26.930.61225')
        self.assertTrue(report['executable_present'])
        self.assertFalse(report['launch_verified'])
        self.assertIsNone(report['can_execute'])
        metadata['CFBundleIdentifier'] = 'org.unrelated.app'
        (app / 'Contents/Info.plist').write_bytes(plistlib.dumps(metadata))
        self.assertFalse(runtime.diagnose_desktop('codex', home=self.home, env=self.env, platform='macos', explicit=app)['identity_verified'])

    def test_claude_mac_bundle_is_identified_independently_without_launch(self):
        app = self.home / 'Claude.app'
        executable = app / 'Contents/MacOS/Claude'
        executable.parent.mkdir(parents=True)
        executable.write_text('must never launch')
        executable.chmod(0o755)
        metadata = {'CFBundleIdentifier': 'com.anthropic.claudefordesktop',
                    'CFBundleShortVersionString': '2.31226.1', 'CFBundleExecutable': 'Claude'}
        (app / 'Contents/Info.plist').write_bytes(plistlib.dumps(metadata))
        with mock.patch.object(runtime, 'run_probe', side_effect=AssertionError('No app launch')):
            report = runtime.diagnose_desktop('claude', home=self.home, env=self.env,
                                              platform='macos', explicit=app)
            wrong = runtime.diagnose_desktop('codex', home=self.home, env=self.env,
                                             platform='macos', explicit=app)
        self.assertEqual(report['identity'], 'com.anthropic.claudefordesktop')
        self.assertEqual(report['version'], '2.31226.1')
        self.assertTrue(report['identity_verified'])
        self.assertTrue(report['executable_present'])
        self.assertTrue(report['executable_accessible'])
        self.assertFalse(report['launch_verified'])
        self.assertEqual(report['auth']['state'], 'not_checked')
        self.assertFalse(wrong['identity_verified'])

    def test_authenticated_clis_cannot_hide_missing_claude_desktop(self):
        codex_app = {'present': True, 'selected_path': str(self.home), 'version': '1.2.3',
                     'identity_verified': True, 'executable_accessible': True,
                     'probe_status': 'metadata_only', 'auth': {'state': 'not_checked'}}
        missing_app = {'present': False, 'selected_path': None, 'version': None,
                       'identity_verified': False, 'executable_accessible': False,
                       'probe_status': 'not_found', 'auth': {'state': 'not_checked'}}
        cli = {'present': True, 'selected_path': str(self.codex), 'version': '1.2.3',
               'can_execute': True, 'selected_on_path': True, 'probe_status': 'ok',
               'auth': {'state': 'authenticated'}}
        with mock.patch.object(runtime, 'diagnose_desktop',
                               side_effect=[codex_app, missing_app]), \
             mock.patch.object(runtime, 'diagnose_cli', side_effect=lambda *a, **k: dict(cli)) as probe, \
             mock.patch.object(runtime.Path, 'home', return_value=self.home):
            report = runtime.diagnose(home=self.home, env=self.env, check_auth=True)
        self.assertFalse(report['ready'])
        self.assertEqual(set(report['runtimes']), {'codex_desktop', 'claude_desktop', 'codex', 'claude'})
        self.assertTrue(report['runtimes']['codex_desktop']['ready'])
        self.assertFalse(report['runtimes']['claude_desktop']['ready'])
        self.assertEqual(report['authentication_attention'], [])
        self.assertEqual(probe.call_args_list[0].kwargs['desktop_path'], self.home)
        self.assertIsNone(probe.call_args_list[1].kwargs['desktop_path'])

    def test_claude_desktop_on_unverified_platform_never_claims_identity(self):
        for platform in ('windows', 'linux'):
            with self.subTest(platform=platform), \
                 mock.patch.object(runtime, 'platform_name', return_value=platform), \
                 mock.patch.object(runtime, 'run_probe', side_effect=AssertionError('No app execution')):
                report = runtime.diagnose_desktop('claude', home=self.home, env=self.env,
                                                  platform=platform, explicit=self.claude)
                unavailable = runtime.diagnose_desktop('claude', home=self.home, env=self.env,
                                                       platform=platform)
            self.assertTrue(report['present'])
            self.assertEqual(report['probe_status'], 'presence_only')
            self.assertFalse(report['identity_verified'])
            self.assertFalse(report['launch_verified'])
            self.assertIsNone(report['can_execute'])
            self.assertEqual(unavailable['probe_status'], 'discovery_unverified')

    def test_explicit_desktop_flags_are_distinct_and_old_flag_is_rejected(self):
        report = {'ready': False, 'authentication_attention': [], 'runtimes': {}}
        with mock.patch.object(runtime, 'diagnose', return_value=report) as diagnose, \
             mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(runtime.main(['doctor', '--json', '--codex-desktop', str(self.codex),
                                           '--claude-desktop', str(self.claude)]), 1)
        self.assertEqual(diagnose.call_args.kwargs['codex_desktop'], self.codex)
        self.assertEqual(diagnose.call_args.kwargs['claude_desktop'], self.claude)
        with mock.patch('sys.stderr', new_callable=io.StringIO), self.assertRaises(SystemExit) as error:
            runtime.main(['doctor', '--desktop', str(self.codex)])
        self.assertEqual(error.exception.code, 2)

    def test_unsupported_desktop_platform_is_distinct_from_a_missing_app(self):
        for client in ('codex', 'claude'):
            with self.subTest(client=client):
                report = runtime.diagnose_desktop(client, home=self.home, env=self.env,
                                                  platform='unsupported-os', explicit=self.claude)
            self.assertEqual(report['probe_status'], 'unsupported_platform')
            self.assertFalse(report['identity_verified'])

    def test_windows_discovery_on_other_host_never_claims_execution(self):
        if runtime.platform_name() == 'windows': self.skipTest('Requires non-Windows host')
        exe = self.home / 'Codex.exe'
        exe.write_bytes(b'not Windows executable')
        with mock.patch.object(runtime, 'run_probe') as run:
            report = runtime.diagnose(home=self.home, codex=exe, claude=exe, codex_desktop=exe, claude_desktop=exe,
                                      env=self.env, platform='windows', check_auth=True)
        run.assert_not_called()
        self.assertFalse(report['native_validation'])
        for record in report['runtimes'].values():
            self.assertEqual(record['probe_status'], 'not_native_platform')
            self.assertIsNone(record['can_execute'])

    def test_windows_shell_shims_are_not_executed_through_shell(self):
        shim = self.home / 'npm/codex.cmd'
        shim.parent.mkdir()
        shim.write_text('arbitrary command')
        with mock.patch.object(runtime, 'path_lookup', return_value=None):
            self.assertIsNone(runtime.launch_command(shim, 'codex', self.env, 'windows'))
        script = shim.parent / 'node_modules/@openai/codex/bin/codex.js'
        script.parent.mkdir(parents=True)
        script.write_text('fixture')
        with mock.patch.object(runtime, 'path_lookup', return_value='C:/Program Files/nodejs/node.exe'):
            self.assertEqual(runtime.launch_command(shim, 'codex', self.env, 'windows'),
                             ['C:/Program Files/nodejs/node.exe', str(script)])

    def test_windows_package_query_is_static_and_filters_unrelated_metadata(self):
        ps = self.home / 'System32/WindowsPowerShell/v1.0/powershell.exe'
        ps.parent.mkdir(parents=True)
        ps.write_text('fixture')
        output = json.dumps([{'Name': 'OpenAI.ChatGPT', 'Version': '1.2.3', 'InstallLocation': str(self.home)},
                             {'Name': 'Bad.App', 'InstallLocation': str(self.home)}])
        with mock.patch.object(runtime, 'run_probe', return_value={'status': 'ok', 'exit_code': 0, 'text': output}) as run:
            records, status = runtime.windows_packages({'SystemRoot': str(self.home)}, 3)
        self.assertEqual(len(records), 1)
        self.assertEqual(status, 'ok')
        self.assertEqual(run.call_args.args[0][-1], runtime.WINDOWS_PACKAGES)
        self.assertIn('-NonInteractive', run.call_args.args[0])

    def test_probe_deadline_overflow_exit_code_and_no_stdin(self):
        result = runtime.run_probe([sys.executable, '-c', 'import time; time.sleep(10)'], timeout=.05)
        self.assertEqual(result['status'], 'timeout')
        result = runtime.run_probe([sys.executable, '-c', 'import sys; print(len(sys.stdin.read())); sys.exit(3)'])
        self.assertEqual(result['status'], 'nonzero')
        self.assertEqual(result['exit_code'], 3)
        self.assertEqual(result['text'].strip(), '0')
        with mock.patch.object(runtime, 'MAX_OUTPUT', 100):
            result = runtime.run_probe([sys.executable, '-c', 'print("x"*5000)'])
        self.assertEqual(result['status'], 'output_limit')
        self.assertLessEqual(len(result['text']), 100)

    def test_linux_desktop_support_is_preview_not_unsupported(self):
        record = runtime.diagnose_desktop('codex', home=self.home, env=self.env, platform='linux', explicit=self.missing)
        self.assertEqual(record['support'], 'preview')
        self.assertFalse(record['launch_verified'])

    def test_ready_is_separate_from_optional_auth_attention(self):
        desktop = {'present': True, 'selected_path': str(self.home), 'version': '1.2.3',
                   'identity_verified': True, 'executable_present': True, 'executable_accessible': True,
                   'probe_status': 'metadata_only', 'auth': {'state': 'not_checked'}}
        cli = {'present': True, 'selected_path': str(self.codex), 'version': '1.2.3',
               'can_execute': True, 'selected_on_path': True, 'probe_status': 'ok',
               'auth': {'state': 'not_authenticated'}}
        with mock.patch.object(runtime, 'diagnose_desktop', return_value=desktop), \
             mock.patch.object(runtime, 'diagnose_cli', side_effect=lambda *a, **k: dict(cli)), \
             mock.patch.object(runtime.Path, 'home', return_value=self.home):
            report = runtime.diagnose(home=self.home, env=self.env, check_auth=True)
        self.assertTrue(report['ready'])
        self.assertEqual(report['authentication_attention'], ['codex', 'claude'])
        self.assertEqual(report['runtimes']['claude']['next_step'], 'claude auth login')
