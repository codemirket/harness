"""Full-install preflight, partial recovery and portable settings integration."""
import copy
import io
import json
from unittest import mock

from test_harness import HarnessFixture, snapshot
from lib import harness, install, runtime, schedule, settings


class InstallTests(HarnessFixture):
    def setUp(self):
        super().setUp()
        self.preferences = self.repo / 'registry/codex-settings.json'
        self.preferences.write_text(json.dumps({'schema_version': 1,
                                               'config': {'model_verbosity': 'low'}}))
        self.options = {'home': self.home, 'mode': 'copy', 'settings_path': self.preferences}
        self.runtime = {'ready': True, 'authentication_attention': [], 'runtimes': {}}
        self.patch(runtime, 'diagnose', lambda **kwargs: copy.deepcopy(self.runtime))
        self.patch(settings, 'app_running', lambda: False)
        self.schedule_report = {'status': 'unchanged', 'changed': False, 'ready_to_apply': True,
                                'verified': True, 'blockers': []}
        self.patch(schedule, 'plan', mock.Mock(side_effect=lambda **kwargs: copy.deepcopy(self.schedule_report)))
        self.patch(schedule, 'apply', mock.Mock(side_effect=lambda **kwargs: copy.deepcopy(self.schedule_report)))

    def test_default_install_registers_daily_job_after_files(self):
        result = install.install(**self.options)
        self.assertTrue(result['ready'])
        schedule.plan.assert_called_once()
        schedule.apply.assert_called_once_with(root=self.repo, home=self.home, mode='copy')
        self.assertTrue(result['schedule']['verified'])

    def test_schedule_preflight_failure_blocks_other_writes(self):
        self.schedule_report.update(ready_to_apply=False, verified=False, blockers=['Native scheduler inaccessible'])
        result = install.install(**self.options)
        self.assertEqual(result['status'], 'blocked')
        self.assertEqual(snapshot(self.home), {})
        schedule.apply.assert_not_called()

    def test_schedule_failure_reports_completed_files(self):
        schedule.apply.side_effect = OSError('scheduler unavailable')
        result = install.install(**self.options)
        self.assertEqual(result['status'], 'partially_applied')
        self.assertFalse(result['ready'])
        self.assertTrue((self.home / '.codex/config.toml').is_file())

    def test_unverified_scheduler_never_claims_complete_install(self):
        schedule.apply.side_effect = lambda **kwargs: {'verified': False, 'status': 'blocked'}
        result = install.install(**self.options)
        self.assertEqual(result['status'], 'partially_applied')
        self.assertFalse(result['ready'])

    def test_missing_job_is_check_drift_and_dry_run_does_not_register(self):
        install.install(**self.options)
        schedule.apply.reset_mock()
        self.schedule_report.update(changed=True, status='pending')
        result = install.install(dry_run=True, **self.options)
        self.assertTrue(result['ready_to_install'])
        self.assertFalse(result['ready'])
        self.assertTrue(result['changes_pending'])
        schedule.apply.assert_not_called()

    def test_explicit_no_schedule_never_probes_or_registers_os(self):
        result = install.install(register_schedule=False, **self.options)
        self.assertTrue(result['ready'])
        self.assertIsNone(result['schedule'])
        schedule.plan.assert_not_called()
        schedule.apply.assert_not_called()

    def test_alternative_settings_require_explicit_file_only_install(self):
        alternate = self.base / 'alternative.json'
        alternate.write_bytes(self.preferences.read_bytes())
        options = dict(self.options, settings_path=alternate)
        self.assertEqual(install.install(**options)['status'], 'blocked')
        self.assertEqual(snapshot(self.home), {})
        self.assertTrue(install.install(register_schedule=False, **options)['ready'])

    def test_dry_run_and_check_do_not_write(self):
        before = snapshot(self.home)
        report = install.install(dry_run=True, **self.options)
        self.assertTrue(report['ready_to_install'])
        self.assertTrue(report['changes_pending'])
        self.assertFalse(report['applied'])
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            status = install.main(['check', '--home', str(self.home), '--settings', str(self.preferences)])
        self.assertEqual(status, 1)
        self.assertEqual(snapshot(self.home), before)

    def test_combined_install_then_check_and_idempotent_settings(self):
        result = install.install(**self.options)
        self.assertEqual(result['status'], 'installed')
        self.assertTrue(result['ready'])
        self.assertEqual((self.home / '.codex/AGENTS.md').read_text(), 'Codex guidance\n')
        self.assertEqual((self.home / '.claude/CLAUDE.md').read_text(), 'Claude guidance\n')
        config = self.home / '.codex/config.toml'
        self.assertIn('model_verbosity = "low"', config.read_text())
        modified = config.stat().st_mtime_ns
        self.assertTrue(install.preflight(**self.options)['ready'])
        result = install.install(**self.options)
        self.assertEqual(result['settings']['status'], 'unchanged')
        self.assertEqual(config.stat().st_mtime_ns, modified)

    def test_missing_runtime_blocks_all_writes(self):
        self.runtime['ready'] = False
        result = install.install(**self.options)
        self.assertFalse(result['ready_to_install'])
        self.assertFalse(result['applied'])
        self.assertEqual(snapshot(self.home), {})

    def test_open_app_blocks_before_guidance_writes(self):
        with mock.patch.object(settings, 'app_running', return_value=True):
            result = install.install(**self.options)
        self.assertEqual(result['status'], 'blocked')
        self.assertEqual(snapshot(self.home), {})

    def test_unmanaged_guidance_preserved_and_settings_not_written(self):
        directory = self.home / '.codex'
        directory.mkdir()
        (directory / 'AGENTS.md').write_text('Personal user content')
        before = snapshot(self.home)
        result = install.install(**self.options)
        self.assertEqual(result['status'], 'blocked')
        self.assertEqual(snapshot(self.home), before)

    def test_invalid_settings_block_before_any_global_write(self):
        self.preferences.write_text('{"schema_version":1,"config":{"approval_policy":"never"}}')
        result = install.install(**self.options)
        self.assertEqual(result['status'], 'blocked')
        self.assertEqual(snapshot(self.home), {})

    def test_auth_attention_does_not_claim_ready_or_login(self):
        self.runtime['authentication_attention'] = ['claude']
        result = install.install(**self.options)
        self.assertTrue(result['applied'])
        self.assertFalse(result['ready'])
        self.assertEqual(result['status'], 'installed_authentication_attention')

    def test_post_preflight_failure_reports_completed_global_install(self):
        with mock.patch.object(settings, 'apply', side_effect=OSError('disk full')):
            result = install.install(**self.options)
        self.assertEqual(result['status'], 'partially_applied')
        self.assertFalse(result['ready'])
        self.assertTrue((self.home / '.codex/AGENTS.md').is_file())
        self.assertFalse((self.home / '.codex/config.toml').exists())
        self.assertTrue((self.home / '.agent-harness/state.json').is_file())

    def test_custom_codex_home_cannot_claim_complete_installation(self):
        custom = self.base / 'custom-codex'
        with mock.patch.dict('os.environ', {'CODEX_HOME': str(custom)}), \
                mock.patch.object(harness, 'global_plan', return_value=(None, None, None, [])):
            result = install.install(settings_path=self.preferences)
        self.assertEqual(result['status'], 'blocked')
        self.assertFalse(result['ready'])
        self.assertFalse(custom.exists())

    def test_failure_during_global_install_reports_partial_items_and_receipt(self):
        original = harness.replace_item
        count = 0
        def replace(*args):
            nonlocal count
            count += 1
            if count == 2:
                raise OSError('disk full')
            return original(*args)
        with mock.patch.object(harness, 'replace_item', side_effect=replace):
            result = install.install(**self.options)
        self.assertEqual(result['status'], 'partially_applied')
        self.assertIsNone(result['applied'])
        self.assertFalse(result['ready'])
        self.assertTrue((self.home / '.codex/AGENTS.md').is_file())
        self.assertFalse((self.home / '.codex/config.toml').exists())
        receipt = json.loads((self.home / '.agent-harness/state.json').read_text())
        self.assertEqual(len(receipt['items']), 1)

    def test_app_started_after_preflight_reports_partial_state(self):
        with mock.patch.object(settings, 'app_running', side_effect=[False, True]):
            result = install.install(**self.options)
        self.assertEqual(result['status'], 'partially_applied')
        self.assertFalse(result['ready'])
        self.assertTrue((self.home / '.codex/AGENTS.md').is_file())
        self.assertFalse((self.home / '.codex/config.toml').exists())

    def test_runtime_doctor_exit_reports_readiness(self):
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(runtime.main(['doctor', '--json']), 0)
            self.runtime['ready'] = False
            self.assertEqual(runtime.main(['doctor', '--json']), 1)
            self.runtime['ready'] = True
            self.runtime['authentication_attention'] = ['codex']
            self.assertEqual(runtime.main(['doctor', '--json', '--check-auth']), 1)
