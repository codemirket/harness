"""Focused upgrade preservation/refusal tests; native/Pi execution has a real fixture gate."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


migration = load('migrate_pi_under_test', ROOT / 'scripts/migrate-pi.py')


class MigrationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='mirket-migration-test-', dir='/private/tmp')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.home = self.root / 'home'
        self.install = self.home / 'install'
        self.agent = self.home / '.pi/agent'
        self.native = self.agent / 'mirket'
        self.old_package = self.install / 'old-package'
        self.package = self.install / 'new-package'
        self.host = self.install / 'host'
        self.launcher = self.home / '.local/bin/pi'
        for path in (self.install, self.native / 'bin', self.native / 'state', self.old_package,
                     self.package, self.host, self.launcher.parent):
            path.mkdir(mode=0o700, parents=True, exist_ok=True)
        for path in (self.home, self.agent, self.native):
            path.chmod(0o700)
        self.installer = load('install_pi_under_test', ROOT / 'scripts/install-pi.py')
        self.old_id, self.new_id = 'sha256:' + 'a' * 64, 'sha256:' + 'b' * 64
        self.old_core, self.new_core = b'old native executable', b'new native executable'
        for package in (self.old_package, self.package):
            (package / 'package.json').write_text(json.dumps({'name': '@codemirket/pi-harness'}))
        self.settings = {'defaultProvider': 'test-provider', 'defaultModel': 'keep-model',
                         'packages': ['npm:unrelated', {'source': str(self.old_package),
                         'skills': ['skills/code-change/SKILL.md'], 'extensions': ['custom.js'],
                         'autoload': False}]}
        self.settings_file = self.agent / 'settings.json'
        self.settings_file.write_text(json.dumps(self.settings) + '\n')
        self.settings_file.chmod(0o600)
        (self.native / 'bin/mirket').write_bytes(self.old_core)
        (self.native / 'active.json').write_text(json.dumps({'release_id': self.old_id}))
        for path in (self.native / 'update.lock', self.native / 'state/runtime.lock'):
            path.touch(mode=0o600)
        self.old_wrapper = self.installer.wrapper(self.agent, self.old_package / 'native/runtime/node/bin/node',
                                                 self.host / 'node_modules' / self.installer.ENTRY)
        self.launcher.write_bytes(self.old_wrapper)
        self.launcher.chmod(0o700)
        self.sentinels = {self.agent / 'auth.json': b'{"private":"fixture-only"}',
                          self.agent / 'sessions/session.jsonl': b'{"fixture":"session"}\n',
                          self.native / 'state/canonical-memory.fixture': b'fixture memory bytes'}
        for path, data in self.sentinels.items():
            path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            path.write_bytes(data)
        self.args = type('Args', (), dict(installer=ROOT / 'scripts/install-pi.py',
            installer_sha256=hashlib.sha256((ROOT / 'scripts/install-pi.py').read_bytes()).hexdigest(),
            install_root=self.install, old_package=self.old_package, old_host=self.host,
            package=self.package, host=self.host, old_release_id=self.old_id, release_id=self.new_id))()
        self.calls = []
        self.env = patch.dict(os.environ, HOME=str(self.home), PI_CODING_AGENT_DIR='')
        self.env.start()
        self.addCleanup(self.env.stop)

    def verified(self, package, host, env, cwd):
        old = package == self.old_package
        return ({'releaseId': self.old_id if old else self.new_id,
                 'coreSha256': hashlib.sha256(self.old_core if old else self.new_core).hexdigest()},
                package / 'native/bin/mirket', package / 'native/runtime/node/bin/node')

    def execute(self, command, cwd, env, timeout=120):
        self.calls.append(command)
        if command[1] == 'doctor':
            return {'status': 'healthy', 'active_release_id': json.loads((self.native / 'active.json').read_text())['release_id']}
        if command[1] == 'update':
            (self.native / 'bin/mirket').write_bytes(self.new_core)
            (self.native / 'active.json').write_text(json.dumps({'release_id': self.new_id}))
            return {'activation': {'status': 'active', 'release_id': self.new_id}}
        if command[1] == '--input-type=module':
            return {'status': 'registered', 'releaseId': self.new_id}
        raise AssertionError(command)

    def run_migration(self):
        with patch.object(migration, 'load_installer', return_value=self.installer), \
             patch.object(self.installer, 'verify', side_effect=self.verified), \
             patch.object(self.installer, 'run_json', side_effect=self.execute), \
             patch.object(self.installer, 'install', return_value={'status': 'already-installed', 'releaseId': self.new_id}):
            return migration.migrate(self.args)

    def test_upgrade_preserves_unrelated_settings_auth_sessions_and_state(self):
        original = self.settings_file.read_bytes()
        active = (self.native / 'active.json').read_bytes()
        result = self.run_migration()
        expected = json.loads(json.dumps(self.settings))
        expected['packages'][1]['source'] = str(self.package)
        self.assertEqual(json.loads(self.settings_file.read_text()), expected)
        self.assertEqual(self.settings_file.stat().st_mode & 0o777, 0o600)
        for path, data in self.sentinels.items():
            self.assertEqual(path.read_bytes(), data)
        backup = Path(result['backup'])
        self.assertEqual((backup / 'settings.json').read_bytes(), original)
        self.assertEqual((backup / 'pi.original').read_bytes(), self.old_wrapper)
        self.assertEqual((backup / 'active.json').read_bytes(), active)
        self.assertEqual(json.loads((backup / 'record.json').read_text())['phase'], 'complete')
        self.assertEqual(self.launcher.read_bytes(), self.installer.wrapper(self.agent,
            self.package / 'native/runtime/node/bin/node', self.host / 'node_modules' / self.installer.ENTRY))
        self.assertFalse(self.settings_file.with_name('settings.json.lock').exists())
        self.assertFalse(self.launcher.with_name('.pi.mirket-install.lock').exists())

    def test_same_native_release_updates_package_without_native_update(self):
        self.new_id = self.old_id
        self.new_core = self.old_core
        self.args.release_id = self.old_id
        active_before = (self.native / 'active.json').read_bytes()
        result = self.run_migration()
        self.assertEqual(result['status'], 'updated')
        self.assertFalse(any(command[1] == 'update' for command in self.calls))
        self.assertTrue(any(command[1] == '--input-type=module' for command in self.calls))
        self.assertEqual((self.native / 'bin/mirket').read_bytes(), self.old_core)
        self.assertEqual((self.native / 'active.json').read_bytes(), active_before)
        expected = json.loads(json.dumps(self.settings))
        expected['packages'][1]['source'] = str(self.package)
        self.assertEqual(json.loads(self.settings_file.read_text()), expected)
        for path, data in self.sentinels.items():
            self.assertEqual(path.read_bytes(), data)
        backup = Path(result['backup'])
        self.assertEqual(json.loads((backup / 'record.json').read_text())['nativeUpdate'],
                         {'status': 'unchanged', 'release_id': self.old_id})
        self.assertEqual(self.launcher.read_bytes(), self.installer.wrapper(self.agent,
            self.package / 'native/runtime/node/bin/node', self.host / 'node_modules' / self.installer.ENTRY))

    def test_same_native_release_with_different_verified_core_refuses(self):
        self.new_id = self.old_id
        self.args.release_id = self.old_id
        with self.assertRaisesRegex(ValueError, 'identical verified core identity'):
            self.run_migration()
        self.assertFalse((self.install / 'migration-backups').exists())
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        self.assertEqual(json.loads(self.settings_file.read_text()), self.settings)
        self.assertFalse(any(command[1] == 'update' for command in self.calls))

    def test_busy_runtime_refuses_before_update_or_backup(self):
        with migration.existing_lock_idle(self.installer, self.native / 'state/runtime.lock', required=True, empty=True):
            with self.assertRaises(BlockingIOError):
                self.run_migration()
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        self.assertEqual(json.loads(self.settings_file.read_text()), self.settings)
        self.assertFalse(any(command[1] == 'update' for command in self.calls))
        self.assertFalse((self.install / 'migration-backups').exists())

    def test_pending_native_state_refuses_without_outer_changes(self):
        (self.native / 'staged.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'Pending native maintenance'):
            self.run_migration()
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        self.assertEqual(json.loads(self.settings_file.read_text()), self.settings)
        self.assertEqual(self.calls, [])

    def test_registration_failure_retains_phase_backup_and_old_launcher(self):
        execute = self.execute
        def fail_registration(command, *args, **kwargs):
            if command[1] == '--input-type=module':
                raise ValueError('fixture registration failure')
            return execute(command, *args, **kwargs)
        self.execute = fail_registration
        with self.assertRaisesRegex(ValueError, 'fixture registration failure'):
            self.run_migration()
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        backups = list((self.install / 'migration-backups').iterdir())
        self.assertEqual(len(backups), 1)
        record = json.loads((backups[0] / 'record.json').read_text())
        self.assertEqual(record['phase'], 'failed-state-preserved')
        self.assertEqual(record['failedPhase'], 'settings-new-launcher-old')
        self.assertEqual(json.loads((self.native / 'active.json').read_text())['release_id'], self.new_id)
        self.assertEqual(json.loads(self.settings_file.read_text())['packages'][1]['source'], str(self.package))
        self.assertFalse(self.settings_file.with_name('settings.json.lock').exists())
        for path, data in self.sentinels.items():
            self.assertEqual(path.read_bytes(), data)

    def test_staged_busy_activation_preserves_outer_configuration(self):
        execute = self.execute
        def staged(command, *args, **kwargs):
            if command[1] == 'update':
                return {'activation': {'status': 'staged', 'release_id': self.new_id,
                                       'reason': 'runtime maintenance lock is busy'}}
            return execute(command, *args, **kwargs)
        self.execute = staged
        with self.assertRaisesRegex(ValueError, 'did not activate'):
            self.run_migration()
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        self.assertEqual(json.loads(self.settings_file.read_text()), self.settings)
        self.assertEqual(json.loads((self.native / 'active.json').read_text())['release_id'], self.old_id)
        backup = next((self.install / 'migration-backups').iterdir())
        record = json.loads((backup / 'record.json').read_text())
        self.assertEqual(record['failedPhase'], 'native-not-active-outer-preserved')
        self.assertEqual(record['nativeUpdate']['activation']['status'], 'staged')

    def test_unrecognized_launcher_is_preserved(self):
        self.launcher.write_bytes(b'#!/bin/sh\nexec unrelated-pi "$@"\n')
        original = self.launcher.read_bytes()
        with self.assertRaisesRegex(ValueError, 'exact selected old'):
            self.run_migration()
        self.assertEqual(self.launcher.read_bytes(), original)
        self.assertEqual(json.loads(self.settings_file.read_text()), self.settings)
        self.assertEqual(self.calls, [])

    def test_installer_hash_mismatch_cannot_execute_source(self):
        path = self.root / 'untrusted.py'
        sentinel = self.root / 'executed'
        path.write_text(f'from pathlib import Path\nPath({str(sentinel)!r}).touch()\n')
        with self.assertRaisesRegex(ValueError, 'independently verified'):
            migration.load_installer(path, '0' * 64)
        self.assertFalse(sentinel.exists())

    def test_duplicate_json_settings_keys_refuse(self):
        text = self.settings_file.read_text().replace('"defaultProvider":', '"defaultModel":"duplicate","defaultProvider":')
        self.settings_file.write_text(text)
        with self.assertRaisesRegex(ValueError, 'Duplicate JSON object key'):
            self.run_migration()
        self.assertEqual(self.settings_file.read_text(), text)
        self.assertEqual(self.calls, [])


if __name__ == '__main__':
    unittest.main()
