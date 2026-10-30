"""Installer tests use only disposable HOME, package and host fixtures."""
import importlib.util
import json
import os
from pathlib import Path
import stat
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('install_pi', Path(__file__).resolve().parents[1] / 'scripts/install-pi.py')
INSTALL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALL)


class InstallationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='mirket-auto-install-test-', dir='/private/tmp' if os.path.isdir('/private/tmp') else '/tmp')
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name).resolve() / 'home'
        self.home.mkdir(mode=0o700)
        self.root = self.home / '.local/share/mirket'
        self.root.mkdir(parents=True, mode=0o700)
        self.package = self.root / 'releases/0.0.1/package'
        self.host = self.root / 'host/pi-1.4.0'
        self.agent = self.home / '.pi/agent'
        self.launcher = self.home / '.local/bin/pi'
        self.release = 'sha256:' + 'a' * 64
        self.files = {
            'native/bin/mirket': b'bound native verifier',
            'native/runtime/node/bin/node': b'bound private node',
            'scripts/setup.mjs': b'bound setup script',
            'dist/extension/index.js': b'bound extension',
            'package.json': json.dumps({'name': '@codemirket/pi-harness', 'version': '0.0.1'}).encode(),
            'host-inventory.json': json.dumps({'schema': 1, 'hostVersion': '1.4.0', 'entry': INSTALL.ENTRY,
                                              'lockSha256': 'b' * 64, 'files': [{'path': INSTALL.ENTRY}]}).encode(),
        }
        self.bind()
        (self.host / 'node_modules').mkdir(parents=True)
        self.calls = []
        self.setup_failure = False
        self.doctor_status = 'healthy'
        self.readiness_status = 'registered'
        self.after_setup = None
        self.after_readiness = None
        self.environment = patch.dict(os.environ, {'HOME': str(self.home), 'PATH': '/nonexistent', 'PI_CODING_AGENT_DIR': ''})
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.runner = patch.object(INSTALL.subprocess, 'check_output', side_effect=self.command)
        self.mock_runner = self.runner.start()
        self.addCleanup(self.runner.stop)

    def bind(self):
        binding = {'schema': 1, 'packageVersion': '0.0.1', 'hostVersion': '1.4.0', 'releaseId': self.release,
                   'coreSha256': INSTALL.sha(self.files['native/bin/mirket']),
                   'nodeSha256': INSTALL.sha(self.files['native/runtime/node/bin/node']),
                   'packageSha256': INSTALL.sha(self.files['package.json']),
                   'hostInventorySha256': INSTALL.sha(self.files['host-inventory.json'])}
        self.files['native-binding.json'] = json.dumps(binding).encode()
        for name, data in self.files.items():
            target = self.package / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            if name in ('native/bin/mirket', 'native/runtime/node/bin/node'):
                target.chmod(0o700)
        entries = [{'path': name, 'bytes': len(data), 'sha256': INSTALL.sha(data)} for name, data in self.files.items()]
        (self.package / 'package-inventory.json').write_text(json.dumps({'schema': 1, 'files': entries}))

    def command(self, args, **kwargs):
        self.calls.append(args)
        self.assertEqual(kwargs['env']['PATH'], '/nonexistent')
        self.assertEqual(kwargs['env']['PI_CODING_AGENT_DIR'], str(self.agent if self.agent.exists() else self.root / 'agent'))
        command = args[1]
        if command == str(self.package / 'scripts/setup.mjs'):
            self.assertEqual(kwargs['env']['HOME'], str(self.home))
        else:
            self.assertNotEqual(kwargs['env']['HOME'], str(self.home))
        if command == 'verify':
            return json.dumps({'release_id': self.release}).encode()
        if command == 'verify-pi-host':
            return json.dumps({'root': str(self.host / 'node_modules'), 'entry': str(self.host / 'node_modules' / INSTALL.ENTRY),
                               'lockSha256': 'b' * 64, 'verifiedFiles': 1}).encode()
        if command == str(self.package / 'scripts/setup.mjs'):
            agent = Path(args[-1])
            installed = agent / 'mirket/bin/mirket'
            installed.parent.mkdir(parents=True)
            (agent / 'mirket').chmod(0o700)
            installed.write_bytes(self.files['native/bin/mirket'])
            settings_file = agent / 'settings.json'
            settings = json.loads(settings_file.read_bytes()) if settings_file.exists() else {}
            packages = settings.setdefault('packages', [])
            replacement = {'source': str(self.package), 'skills': ['skills/core/SKILL.md']}
            for index, package in enumerate(packages):
                source = package if isinstance(package, str) else package['source']
                if source in (str(self.package), '~/' + str(self.package.relative_to(self.home))):
                    packages[index] = {**(package if isinstance(package, dict) else {}), **replacement}
                    break
            else:
                packages.append(replacement)
            settings_file.write_text(json.dumps(settings))
            if self.after_setup:
                self.after_setup(agent)
            if self.setup_failure:
                raise subprocess.CalledProcessError(1, args)
            return json.dumps({'home': str(agent / 'mirket'), 'packageSource': str(self.package) + '/',
                               'releaseId': self.release, 'selectedSkills': ['skills/core/SKILL.md']}).encode()
        if command == 'doctor':
            return json.dumps({'status': self.doctor_status, 'active_release_id': self.release}).encode()
        if command == '--input-type=module':
            self.assertIn('AuthStorage.inMemory()', args[3])
            self.assertIn("session.prompt('/mirket doctor')", args[3])
            if self.after_readiness:
                self.after_readiness()
            return json.dumps({'status': self.readiness_status, 'releaseId': self.release}).encode()
        self.fail('Unexpected command: ' + repr(args))

    def install(self):
        return INSTALL.install(self.package, self.host)

    def existing(self):
        self.agent.mkdir(parents=True, mode=0o755)
        self.agent.chmod(0o755)
        (self.agent / 'bin').mkdir()
        (self.agent / 'bin/pi').write_bytes(b'#!/bin/sh\n# Existing Pi managed launcher, never modified.\n')
        managed = self.agent / 'install'
        managed.mkdir()
        (managed / 'managed-install.json').write_text(json.dumps({'kind': 'pi-managed-install', 'schemaVersion': 1,
             'layout': 'releases-v1', 'entrypoint': {'type': 'symlink', 'path': str(self.launcher)}}))
        (managed / 'current-version').write_text('1.4.0\n')
        module = managed / 'releases/1.4.0/node_modules/@earendil-works/pi-coding-agent'
        (module / 'dist/bundle').mkdir(parents=True)
        (module / 'dist/bundle/cli.js').write_bytes(b'existing bundled Pi')
        (module / 'package.json').write_text(json.dumps({'name': '@earendil-works/pi-coding-agent', 'version': '1.4.0'}))
        (module.parent.parent / '.bin').mkdir()
        (module.parent.parent / '.bin/pi').symlink_to('../@earendil-works/pi-coding-agent/dist/bundle/cli.js')
        self.launcher.parent.mkdir(parents=True)
        self.launcher.symlink_to(INSTALL.MANAGED_LINK)
        (self.agent / 'settings.json').write_text(json.dumps({'theme': 'preserve', 'packages': ['npm:unrelated@1']}))
        (self.agent / 'auth.json').write_bytes(b'private auth sentinel')
        (self.agent / 'sessions').mkdir()
        (self.agent / 'sessions/keep.jsonl').write_bytes(b'private session sentinel')

    def backup(self):
        return next((self.root / 'install-backups').iterdir())

    def test_fresh_installs_private_agent_and_exact_launcher_after_readiness(self):
        result = self.install()
        self.assertEqual(result['status'], 'installed')
        self.assertEqual(result['agentDir'], str(self.root / 'agent'))
        self.assertEqual(stat.S_IMODE((self.root / 'agent').stat().st_mode), 0o700)
        self.assertEqual(self.launcher.read_bytes(), INSTALL.wrapper(self.root / 'agent', self.package / 'native/runtime/node/bin/node', self.host / 'node_modules' / INSTALL.ENTRY))
        self.assertEqual(stat.S_IMODE(self.launcher.stat().st_mode), 0o700)
        self.assertFalse(self.agent.exists())
        self.assertEqual(json.loads((self.backup() / 'record.json').read_bytes())['phase'], 'complete')

    def test_existing_managed_pi_preserves_original_files_settings_auth_and_sessions(self):
        self.existing()
        managed_bytes = (self.agent / 'bin/pi').read_bytes()
        settings_bytes = (self.agent / 'settings.json').read_bytes()
        result = self.install()
        self.assertEqual(result['agentDir'], str(self.agent))
        self.assertFalse(self.launcher.is_symlink())
        self.assertEqual((self.agent / 'bin/pi').read_bytes(), managed_bytes)
        self.assertEqual((self.agent / 'auth.json').read_bytes(), b'private auth sentinel')
        self.assertEqual((self.agent / 'sessions/keep.jsonl').read_bytes(), b'private session sentinel')
        self.assertEqual(json.loads((self.agent / 'settings.json').read_bytes())['theme'], 'preserve')
        self.assertEqual((self.backup() / 'settings.json').read_bytes(), settings_bytes)
        self.assertEqual(os.readlink(self.backup() / 'pi.original-link'), INSTALL.MANAGED_LINK)
        self.assertEqual(json.loads((self.backup() / 'record.json').read_bytes())['agentMode'], 0o755)
        self.assertEqual(stat.S_IMODE(self.agent.stat().st_mode), 0o700)
        self.assertFalse((self.launcher.parent / 'pi-managed').exists())

    def test_same_complete_install_is_verified_without_setup_or_settings_write(self):
        self.existing()
        self.install()
        settings = (self.agent / 'settings.json').read_bytes()
        launcher = self.launcher.read_bytes()
        self.calls.clear()
        self.assertEqual(self.install()['status'], 'already-installed')
        self.assertEqual((self.agent / 'settings.json').read_bytes(), settings)
        self.assertEqual(self.launcher.read_bytes(), launcher)
        self.assertEqual(len(list((self.root / 'install-backups').iterdir())), 1)
        self.assertNotIn(str(self.package / 'scripts/setup.mjs'), [call[1] for call in self.calls])

    def test_legacy_wrapper_migrates_after_registration_with_private_backup(self):
        self.existing()
        self.install()
        legacy = INSTALL.legacy_wrapper(self.agent, self.package / 'native/runtime/node/bin/node', self.host / 'node_modules' / INSTALL.ENTRY)
        self.launcher.write_bytes(legacy)
        settings = (self.agent / 'settings.json').read_bytes()
        self.calls.clear()
        self.after_readiness = lambda: self.assertEqual(self.launcher.read_bytes(), legacy)
        result = self.install()
        self.assertEqual(result['status'], 'launcher-updated')
        self.assertEqual(self.launcher.read_bytes(), INSTALL.wrapper(self.agent, self.package / 'native/runtime/node/bin/node', self.host / 'node_modules' / INSTALL.ENTRY))
        self.assertEqual((Path(result['backup']) / 'pi.original').read_bytes(), legacy)
        self.assertEqual(stat.S_IMODE((Path(result['backup']) / 'pi.original').stat().st_mode), 0o600)
        self.assertEqual((self.agent / 'settings.json').read_bytes(), settings)
        self.assertNotIn(str(self.package / 'scripts/setup.mjs'), [call[1] for call in self.calls])

    def test_failed_registration_never_migrates_legacy_wrapper(self):
        self.existing()
        self.install()
        legacy = INSTALL.legacy_wrapper(self.agent, self.package / 'native/runtime/node/bin/node', self.host / 'node_modules' / INSTALL.ENTRY)
        self.launcher.write_bytes(legacy)
        self.readiness_status = 'not_registered'
        with self.assertRaisesRegex(ValueError, 'readiness differs'):
            self.install()
        self.assertEqual(self.launcher.read_bytes(), legacy)
        self.assertEqual(len(list((self.root / 'install-backups').iterdir())), 1)

    def test_nonexecutable_exact_wrapper_does_not_claim_complete_install(self):
        self.existing()
        self.install()
        self.calls.clear()
        self.launcher.chmod(0o600)
        with self.assertRaisesRegex(ValueError, 'not executable'):
            self.install()
        self.assertEqual(stat.S_IMODE(self.launcher.stat().st_mode), 0o600)
        self.assertEqual(self.calls, [])

    def test_complete_rerun_detects_launcher_replacement_during_registration(self):
        self.existing()
        self.install()
        original = self.launcher.read_bytes()
        def replace():
            self.launcher.unlink()
            self.launcher.write_bytes(original)
            self.launcher.chmod(0o700)
        self.after_readiness = replace
        with self.assertRaisesRegex(ValueError, 'launcher changed'):
            self.install()
        self.assertEqual(self.launcher.read_bytes(), original)

    def test_launcher_replacement_during_preflight_refuses_before_setup(self):
        self.existing()
        real_layout = INSTALL.managed_layout
        def check(agent, launcher):
            real_layout(agent, launcher)
            launcher.unlink()
            launcher.symlink_to(INSTALL.MANAGED_LINK)
        with patch.object(INSTALL, 'managed_layout', side_effect=check):
            with self.assertRaisesRegex(ValueError, 'launcher changed during preflight'):
                self.install()
        self.assertEqual(self.calls, [])

    def test_existing_cooperative_lock_refuses_publication_and_is_preserved(self):
        self.existing()
        lock = self.launcher.with_name('.pi.mirket-install.lock')
        lock.mkdir(mode=0o700)
        with self.assertRaisesRegex(ValueError, 'launcher is busy'):
            self.install()
        self.assertTrue(lock.is_dir())
        self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)

    def test_launcher_publication_holds_lock_and_removes_it_after_success(self):
        self.existing()
        real_write = INSTALL.atomic_write
        def write(path, *args, **kwargs):
            if path == self.launcher:
                self.assertTrue(self.launcher.with_name('.pi.mirket-install.lock').is_dir())
            return real_write(path, *args, **kwargs)
        with patch.object(INSTALL, 'atomic_write', side_effect=write):
            self.install()
        self.assertFalse(self.launcher.with_name('.pi.mirket-install.lock').exists())

    def test_fresh_publication_cannot_replace_launcher_created_after_final_check(self):
        real_link = INSTALL.os.link
        def create(source, target):
            self.assertEqual(target, self.launcher)
            self.launcher.write_bytes(b'concurrent unrelated launcher')
            return real_link(source, target)
        with patch.object(INSTALL.os, 'link', side_effect=create):
            with self.assertRaises(FileExistsError):
                self.install()
        self.assertEqual(self.launcher.read_bytes(), b'concurrent unrelated launcher')
        self.assertFalse(self.launcher.with_name('.pi.mirket-install.lock').exists())

    def test_bound_package_tamper_refuses_before_native_execution_or_mutation(self):
        for name in ('native/runtime/node/bin/node', 'native-binding.json', 'scripts/setup.mjs'):
            with self.subTest(name=name):
                path = self.package / name
                before = path.read_bytes()
                path.write_bytes(before + b' ')
                with self.assertRaisesRegex(ValueError, 'differs'):
                    self.install()
                self.assertEqual(self.calls, [])
                self.assertFalse((self.root / 'agent').exists())
                self.assertFalse((self.root / 'install-backups').exists())
                path.write_bytes(before)

    def test_changed_host_verification_refuses_before_backup_or_setup(self):
        with patch.object(INSTALL, 'run_json', side_effect=[{'release_id': self.release}, {'root': 'wrong'}]):
            with self.assertRaisesRegex(ValueError, 'verification response differs'):
                self.install()
        self.assertFalse((self.root / 'install-backups').exists())

    def test_unrelated_and_aliased_launchers_are_preserved(self):
        self.launcher.parent.mkdir(parents=True)
        for target in ('../../elsewhere/pi', None):
            with self.subTest(target=target):
                if target:
                    self.launcher.symlink_to(target)
                else:
                    self.launcher.write_bytes(b'#!/bin/sh\nexec unrelated-pi "$@"\n')
                with self.assertRaisesRegex(ValueError, 'launcher'):
                    self.install()
                if target:
                    self.assertEqual(os.readlink(self.launcher), target)
                else:
                    self.assertIn(b'unrelated-pi', self.launcher.read_bytes())
                self.launcher.unlink()
        self.assertEqual(self.calls, [])

    def test_unknown_managed_metadata_refuses_without_touching_install(self):
        self.existing()
        path = self.agent / 'install/managed-install.json'
        path.write_text('{}')
        with self.assertRaisesRegex(ValueError, 'Unknown managed Pi layout'):
            self.install()
        self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)
        self.assertEqual(stat.S_IMODE(self.agent.stat().st_mode), 0o755)
        self.assertEqual(self.calls, [])

    def test_different_managed_host_version_is_not_silently_replaced(self):
        self.existing()
        (self.agent / 'install/current-version').write_text('1.5.0\n')
        with self.assertRaisesRegex(ValueError, 'Managed Pi version differs'):
            self.install()
        self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)
        self.assertEqual(self.calls, [])

    def test_existing_tilde_package_source_uses_target_home_and_preserves_own_options(self):
        self.existing()
        source = '~/' + str(self.package.relative_to(self.home))
        (self.agent / 'settings.json').write_text(json.dumps({'theme': 'preserve', 'packages': [
            {'source': source, 'extensions': ['./dist/extension/index.js']}, 'npm:unrelated@1']}))
        self.install()
        packages = json.loads((self.agent / 'settings.json').read_bytes())['packages']
        self.assertEqual(len(packages), 2)
        self.assertEqual(packages[0]['source'], str(self.package))
        self.assertEqual(packages[0]['extensions'], ['./dist/extension/index.js'])

    def test_conflicting_package_settings_and_duplicate_registration_refuse(self):
        self.existing()
        for packages in (['npm:@codemirket/pi-harness@0.0.1'], [str(self.package), str(self.package)]):
            with self.subTest(packages=packages):
                (self.agent / 'settings.json').write_text(json.dumps({'packages': packages}))
                with self.assertRaisesRegex(ValueError, 'Conflicting|Duplicate'):
                    self.install()
        self.assertEqual(self.calls, [])
        self.assertFalse((self.agent / 'mirket').exists())

    def test_existing_unknown_mirket_home_is_never_overwritten(self):
        self.existing()
        (self.agent / 'mirket').mkdir(mode=0o700)
        sentinel = self.agent / 'mirket/keep'
        sentinel.write_bytes(b'canonical memory sentinel')
        with self.assertRaisesRegex(ValueError, 'Existing Mirket home is preserved'):
            self.install()
        self.assertEqual(sentinel.read_bytes(), b'canonical memory sentinel')
        self.assertEqual(self.calls, [])

    def test_setup_failure_preserves_partial_home_settings_and_original_launcher(self):
        self.existing()
        self.setup_failure = True
        with self.assertRaises(subprocess.CalledProcessError):
            self.install()
        self.assertTrue((self.agent / 'mirket').is_dir())
        self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)
        self.assertEqual(json.loads((self.backup() / 'record.json').read_bytes())['phase'], 'setup-failed-state-preserved')
        self.assertEqual(len(json.loads((self.agent / 'settings.json').read_bytes())['packages']), 2)

    def test_post_setup_health_failure_preserves_state_and_never_publishes_launcher(self):
        self.existing()
        self.doctor_status = 'error'
        with self.assertRaisesRegex(ValueError, 'Native doctor'):
            self.install()
        self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)
        self.assertTrue((self.agent / 'mirket/bin/mirket').exists())

    def test_existing_completed_setup_recovers_publication_without_setup_or_settings_write(self):
        self.existing()
        self.readiness_status = 'not_registered'
        with self.assertRaisesRegex(ValueError, 'readiness differs'):
            self.install()
        original_settings = (self.agent / 'settings.json').read_bytes()
        backup = self.backup()
        self.readiness_status = 'registered'
        self.calls.clear()
        result = self.install()
        self.assertEqual(result['status'], 'installed')
        self.assertEqual(result['backup'], str(backup))
        self.assertEqual(json.loads((backup / 'record.json').read_bytes())['phase'], 'complete')
        self.assertEqual((self.agent / 'settings.json').read_bytes(), original_settings)
        self.assertNotIn(str(self.package / 'scripts/setup.mjs'), [call[1] for call in self.calls])
        self.assertFalse(self.launcher.is_symlink())

    def test_fresh_completed_setup_recovers_publication_without_setup(self):
        self.readiness_status = 'not_registered'
        with self.assertRaisesRegex(ValueError, 'readiness differs'):
            self.install()
        self.assertFalse(self.launcher.exists())
        self.readiness_status = 'registered'
        self.calls.clear()
        result = self.install()
        self.assertEqual(result['status'], 'installed')
        self.assertEqual(result['agentDir'], str(self.root / 'agent'))
        self.assertNotIn(str(self.package / 'scripts/setup.mjs'), [call[1] for call in self.calls])
        self.assertTrue(self.launcher.is_file())

    def test_incomplete_or_different_setup_record_cannot_authorize_recovery(self):
        self.existing()
        self.readiness_status = 'not_registered'
        with self.assertRaisesRegex(ValueError, 'readiness differs'):
            self.install()
        path = self.backup() / 'record.json'
        original = json.loads(path.read_bytes())
        for change in ({'phase': 'setup-failed-state-preserved'}, {'host': str(self.root / 'different-host')}):
            with self.subTest(change=change):
                path.write_text(json.dumps({**original, **change}))
                self.calls.clear()
                with self.assertRaisesRegex(ValueError, 'Existing Mirket home is preserved'):
                    self.install()
                self.assertEqual(self.calls, [])
                self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)

    def test_failed_registration_preserves_original_launcher(self):
        self.existing()
        self.readiness_status = 'not_ready'
        with self.assertRaisesRegex(ValueError, 'readiness differs'):
            self.install()
        self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)

    def test_concurrent_settings_edit_is_preserved_and_refuses_publication(self):
        self.existing()
        def edit(agent):
            path = agent / 'settings.json'
            settings = json.loads(path.read_bytes())
            settings['theme'] = 'concurrent edit preserved'
            path.write_text(json.dumps(settings))
        self.after_setup = edit
        with self.assertRaisesRegex(ValueError, 'Unrelated Pi settings changed'):
            self.install()
        self.assertEqual(json.loads((self.agent / 'settings.json').read_bytes())['theme'], 'concurrent edit preserved')
        self.assertEqual(os.readlink(self.launcher), INSTALL.MANAGED_LINK)

    def test_concurrent_launcher_edit_is_preserved(self):
        self.existing()
        def edit():
            self.launcher.unlink()
            self.launcher.write_bytes(b'concurrent unrelated launcher')
        self.after_readiness = edit
        with self.assertRaisesRegex(ValueError, 'launcher changed'):
            self.install()
        self.assertEqual(self.launcher.read_bytes(), b'concurrent unrelated launcher')

    def test_removed_real_package_registration_is_not_hidden_by_isolated_probe(self):
        self.existing()
        self.install()
        (self.agent / 'settings.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'Existing Mirket home is preserved'):
            self.install()

    def test_symlink_ancestry_and_external_package_locations_fail_closed(self):
        alias = self.root / 'alias'
        alias.symlink_to(self.package, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'canonical'):
            INSTALL.install(alias, self.host)
        with self.assertRaisesRegex(ValueError, 'permanent paths'):
            INSTALL.install(self.home, self.host)
        self.assertEqual(self.calls, [])


if __name__ == '__main__':
    unittest.main()
