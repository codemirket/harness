"""Focused upgrade preservation/refusal tests; native/Pi execution has a real fixture gate."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import time
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

    def project_fixture(self, name='project'):
        project = self.root / name
        directory = project / '.pi'
        directory.mkdir(parents=True, mode=0o700)
        file = directory / 'settings.json'
        file.write_text(json.dumps({'theme': 'keep-project-theme', 'packages': [
            'npm:unrelated', {'source': str(self.old_package), 'autoload': False,
                              'skills': ['!skills/**', '+skills/code-change/SKILL.md'],
                              'extensions': ['+./dist/extension/index.js']}]}))
        file.chmod(0o640)
        return project, file

    def test_explicit_project_upgrade_preserves_filters_mode_and_unselected_project(self):
        project, file = self.project_fixture()
        _, unselected = self.project_fixture('unselected')
        before, unselected_before = file.read_bytes(), unselected.read_bytes()
        self.args.project = [project]
        result = self.run_migration()
        expected = json.loads(before)
        expected['packages'][1]['source'] = str(self.package)
        self.assertEqual(json.loads(file.read_bytes()), expected)
        self.assertEqual(file.stat().st_mode & 0o777, 0o640)
        self.assertEqual(unselected.read_bytes(), unselected_before)
        backup = Path(result['backup'])
        self.assertEqual((backup / 'project-settings-0.json').read_bytes(), before)
        record = json.loads((backup / 'record.json').read_bytes())
        self.assertEqual(record['projects'][0]['settings'], str(file))
        self.assertEqual(record['projects'][0]['afterSha256'], hashlib.sha256(file.read_bytes()).hexdigest())
        self.assertEqual(result['projects'], [str(project)])
        self.assertFalse(file.with_name('settings.json.lock').exists())

    def test_project_busy_or_duplicate_selection_refuses_before_native_effects(self):
        project, file = self.project_fixture()
        before = file.read_bytes()
        self.args.project = [project, project]
        with self.assertRaisesRegex(ValueError, 'distinct project'):
            self.run_migration()
        self.args.project = [project]
        file.with_name('settings.json.lock').mkdir()
        with self.assertRaisesRegex(ValueError, 'busy'):
            self.run_migration()
        self.assertEqual(file.read_bytes(), before)
        self.assertFalse(self.calls)

    def test_project_without_old_object_and_symlink_refuse(self):
        project, file = self.project_fixture()
        self.args.project = [project]
        file.write_text('{"packages": []}')
        with self.assertRaisesRegex(ValueError, 'exactly one old'):
            self.run_migration()
        file.unlink()
        file.symlink_to(self.settings_file)
        with self.assertRaises(ValueError):
            self.run_migration()
        self.assertFalse(self.calls)

    def test_project_duplicate_json_keys_refuse_without_effects(self):
        project, file = self.project_fixture()
        self.args.project = [project]
        file.write_text('{"theme": "first", "theme": "second", "packages": [{"source": '
                        + json.dumps(str(self.old_package)) + '}]}')
        with self.assertRaisesRegex(ValueError, 'Duplicate JSON'):
            self.run_migration()
        self.assertFalse(self.calls)

    def test_project_drift_during_native_update_is_preserved(self):
        project, file = self.project_fixture()
        self.args.project = [project]
        original_execute = self.execute
        def execute(command, cwd, env, timeout=120):
            result = original_execute(command, cwd, env, timeout)
            if command[1] == 'update':
                file.write_text('{"theme": "concurrent edit"}')
            return result
        self.execute = execute
        with self.assertRaisesRegex(ValueError, 'Selected project settings changed'):
            self.run_migration()
        self.assertEqual(json.loads(file.read_bytes()), {'theme': 'concurrent edit'})
        self.assertEqual(json.loads(self.settings_file.read_bytes()), self.settings)
        self.assertFalse(file.with_name('settings.json.lock').exists())

    def test_partial_project_publication_keeps_exact_recovery_backups(self):
        first, first_file = self.project_fixture('first')
        second, second_file = self.project_fixture('second')
        before = [first_file.read_bytes(), second_file.read_bytes()]
        self.args.project = [first, second]
        atomic = self.installer.atomic_write
        def write(path, data, mode=0o600):
            if path == second_file:
                raise OSError('fixture project publication failure')
            return atomic(path, data, mode)
        with patch.object(self.installer, 'atomic_write', side_effect=write):
            with self.assertRaisesRegex(OSError, 'project publication failure'):
                self.run_migration()
        backup = next((self.install / 'migration-backups').iterdir())
        record = json.loads((backup / 'record.json').read_bytes())
        self.assertEqual(record['failedPhase'], 'project-settings-1-publication-started')
        self.assertEqual(json.loads(first_file.read_bytes())['packages'][1]['source'], str(self.package))
        self.assertEqual(second_file.read_bytes(), before[1])
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        for index, file in enumerate([first_file, second_file]):
            self.assertEqual((backup / f'project-settings-{index}.json').read_bytes(), before[index])
            self.assertFalse(file.with_name('settings.json.lock').exists())

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

    def test_removed_lock_after_native_update_refuses_settings_publication(self):
        execute = self.execute
        lock = self.settings_file.with_name('settings.json.lock')
        def remove_lock(command, *args, **kwargs):
            result = execute(command, *args, **kwargs)
            if command[1] == 'update':
                lock.rmdir()
            return result
        self.execute = remove_lock
        with self.assertRaisesRegex(ValueError, 'lock ownership lost'):
            self.run_migration()
        self.assertEqual(json.loads(self.settings_file.read_bytes()), self.settings)
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        self.assertEqual(json.loads((self.native / 'active.json').read_bytes())['release_id'], self.new_id)
        record = json.loads(next((self.install / 'migration-backups').iterdir()).joinpath('record.json').read_bytes())
        self.assertEqual(record['failedPhase'], 'native-active-outer-old')
        for path, data in self.sentinels.items():
            self.assertEqual(path.read_bytes(), data)

    def test_replaced_lock_during_readiness_preserves_replacement_and_old_launcher(self):
        execute = self.execute
        lock = self.settings_file.with_name('settings.json.lock')
        def replace_lock(command, *args, **kwargs):
            result = execute(command, *args, **kwargs)
            if command[1] == '--input-type=module':
                lock.rename(self.root / 'displaced-lock')
                lock.mkdir(mode=0o700)
                (lock / 'other-owner').write_bytes(b'preserve replacement')
            return result
        self.execute = replace_lock
        with self.assertRaisesRegex(ValueError, 'lock ownership lost'):
            self.run_migration()
        self.assertEqual((lock / 'other-owner').read_bytes(), b'preserve replacement')
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)
        self.assertEqual(json.loads(self.settings_file.read_bytes())['packages'][1]['source'], str(self.package))
        record = json.loads(next((self.install / 'migration-backups').iterdir()).joinpath('record.json').read_bytes())
        self.assertEqual(record['failedPhase'], 'settings-new-launcher-old')
        for path, data in self.sentinels.items():
            self.assertEqual(path.read_bytes(), data)

    def test_heartbeat_failure_refuses_even_when_original_lock_still_exists(self):
        lock = self.settings_file.with_name('settings.json.lock')
        with patch.object(migration.os, 'utime', side_effect=OSError('fixture renewal denied')):
            with self.assertRaisesRegex(ValueError, 'heartbeat failed'):
                with migration.settings_lock(lock) as check:
                    deadline = time.monotonic() + 5
                    while time.monotonic() < deadline:
                        time.sleep(0.05)
                        check()
                    self.fail('Heartbeat failure was not reported')
        self.assertFalse(lock.exists())
        self.assertEqual(json.loads(self.settings_file.read_bytes()), self.settings)
        self.assertEqual(self.launcher.read_bytes(), self.old_wrapper)

    def test_acquisition_failure_cleans_owned_lock_and_keeps_primary_error(self):
        lock = self.settings_file.with_name('settings.json.lock')
        with patch.object(migration.os, 'open', side_effect=OSError('fixture open denied')):
            with self.assertRaisesRegex(OSError, 'fixture open denied'):
                with migration.settings_lock(lock):
                    self.fail('Failed acquisition entered the lock context')
        self.assertFalse(lock.exists())

    def test_cleanup_preserves_replacement_and_original_failure(self):
        lock = self.settings_file.with_name('settings.json.lock')
        with self.assertRaisesRegex(ValueError, 'fixture primary failure') as caught:
            with migration.settings_lock(lock):
                lock.rename(self.root / 'displaced-lock')
                lock.mkdir(mode=0o700)
                raise ValueError('fixture primary failure')
        self.assertIn('lock ownership lost', str(caught.exception.__cause__))
        self.assertTrue(lock.is_dir())

    def test_expired_lock_refuses_without_waiting_for_another_owner(self):
        lock = self.settings_file.with_name('settings.json.lock')
        with self.assertRaisesRegex(ValueError, 'renewal expired'):
            with migration.settings_lock(lock) as check:
                old = time.time() - 20
                os.utime(lock, (old, old))
                check()
        self.assertFalse(lock.exists())

    @unittest.skipUnless(os.environ.get('MIRKET_TEST_PACKAGE') and os.environ.get('MIRKET_TEST_PI_HOST'),
                         'Select explicit package/private Node and Pi host for the real lock protocol gate')
    def test_verified_pi_reclaims_stale_lock_but_cannot_reclaim_renewed_lock(self):
        package = Path(os.environ['MIRKET_TEST_PACKAGE'])
        host = Path(os.environ['MIRKET_TEST_PI_HOST'])
        env = {**os.environ, 'HOME': str(self.home), 'TMPDIR': str(self.root), 'PATH': '/nonexistent',
               'PI_CODING_AGENT_DIR': str(self.agent), 'PI_CODING_AGENT_SESSION_DIR': str(self.root / 'sessions')}
        _, _, node = self.installer.verify(package, host, env, self.root)
        source = """
import {createRequire} from 'node:module';
import {join} from 'node:path';
const [host,file]=process.argv.slice(1);
const lockfile=createRequire(import.meta.url)(join(host,'node_modules/proper-lockfile'));
try {const release=lockfile.lockSync(file,{realpath:false});release();console.log(JSON.stringify({status:'acquired'}));}
catch(error) {if(error.code!=='ELOCKED')throw error;console.log(JSON.stringify({status:'busy'}));}
"""
        def contender():
            return self.installer.run_json([node, '--input-type=module', '--eval', source, host, self.settings_file], self.root, env)
        lock = self.settings_file.with_name('settings.json.lock')
        lock.mkdir(mode=0o700)
        old = time.time() - 20
        os.utime(lock, (old, old))
        self.assertEqual(contender(), {'status': 'acquired'})
        self.assertFalse(lock.exists())
        with migration.settings_lock(lock) as check:
            identity = lock.stat().st_ino
            time.sleep(11)
            self.assertEqual(contender(), {'status': 'busy'})
            check()
            self.assertEqual(lock.stat().st_ino, identity)
        self.assertEqual(contender(), {'status': 'acquired'})

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
