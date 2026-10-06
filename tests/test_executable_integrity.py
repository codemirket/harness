"""Project payload modes are a separate contract from source/adapted byte hashes."""
import json
import os
from pathlib import Path
import subprocess
import sys
from types import SimpleNamespace
import unittest
from unittest import mock

from test_harness import HarnessFixture, harness, catalog, reviewed_hash, snapshot


class ExecutableIntegrityTests(HarnessFixture):
    def add_helper(self, relative='scripts/helper.py'):
        directory = self.repo / 'skills/feature'
        path = directory / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b'#!/usr/bin/env python3\nraise SystemExit("Never execute me")\n')
        self.entry('feature')['executable_files'] = [relative]
        self.entry('feature')['sha256'] = reviewed_hash({
            p.relative_to(directory).as_posix(): p.read_bytes()
            for p in directory.rglob('*') if p.is_file()})

    def installed(self, relative='SKILL.md', target='claude'):
        return self.destination('feature', target, True) / relative

    def install_project(self):
        self.initialize()
        harness.sync_project(self.project)

    def legacy_receipts(self):
        for target in ('codex', 'claude'):
            path = self.installed(catalog.RECEIPT, target)
            value = json.loads(path.read_text())
            value.pop('executable_files')
            path.write_text(json.dumps(value))

    def assert_preserved_failure(self, regex=None):
        before = snapshot(self.project, timestamps=True)
        with self.assertRaisesRegex(ValueError, regex or '.'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_catalog_rejects_any_execute_bit_on_ordinary_files_and_receipt(self):
        self.install_project()
        for relative in ('SKILL.md', 'references/guide.md', catalog.RECEIPT):
            path = self.installed(relative)
            original = path.stat().st_mode
            for bit in (0o100, 0o010, 0o001):
                with self.subTest(relative=relative, bit=bit):
                    path.chmod(0o600 | bit)
                    with self.assertRaisesRegex(ValueError, 'Undeclared executable'):
                        catalog.check_destination(self.entry('feature'), self.project, 'claude')
            path.chmod(original)

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_unchanged_project_and_doctor_reject_permission_drift(self):
        self.install_project()
        self.installed().chmod(0o755)
        self.assert_preserved_failure('Undeclared executable')
        result = subprocess.run([sys.executable, '-B', str(self.repo / 'ai.py'), 'project', 'doctor',
                                 '--project', str(self.project)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 1)
        self.assertIn('Undeclared executable', result.stderr)

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_later_target_permission_drift_blocks_all_updates_and_preserves_lock(self):
        self.install_project()
        self.installed('references/guide.md').chmod(0o744)
        self.revise('foundation')
        self.revise('feature')
        self.assert_preserved_failure('Undeclared executable')

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_new_catalog_helper_contract_cannot_hide_prior_ordinary_file_drift(self):
        self.install_project()
        self.installed('references/guide.md').chmod(0o755)
        self.entry('feature')['executable_files'] = ['references/guide.md']
        self.assert_preserved_failure('Undeclared executable')

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_same_bytes_can_change_reviewed_helper_contract_with_updated_receipt(self):
        self.install_project()
        old_hash = self.entry('feature')['sha256']
        self.entry('feature')['executable_files'] = ['references/guide.md']
        jobs = harness.sync_project(self.project)
        self.assertEqual(self.entry('feature')['sha256'], old_hash)
        self.assertEqual([job['action'] for job in jobs if job['id'] == 'feature'], ['update', 'update'])
        for target in ('codex', 'claude'):
            self.assertEqual(self.installed('references/guide.md', target).stat().st_mode & 0o111, 0o111)
            self.assertEqual(json.loads(self.installed(catalog.RECEIPT, target).read_text())['executable_files'],
                             ['references/guide.md'])
        self.entry('feature')['executable_files'] = []
        harness.sync_project(self.project)
        for target in ('codex', 'claude'):
            self.assertEqual(self.installed('references/guide.md', target).stat().st_mode & 0o111, 0)
            self.assertEqual(json.loads(self.installed(catalog.RECEIPT, target).read_text())['executable_files'], [])

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_removed_helper_still_uses_prior_declared_requirement(self):
        self.add_helper()
        self.install_project()
        self.installed('scripts/helper.py').chmod(0o744)
        (self.repo / 'skills/feature/scripts/helper.py').unlink()
        self.entry('feature')['executable_files'] = []
        self.revise('feature')
        self.assert_preserved_failure('executable mode changed')

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_declared_helpers_require_all_execute_bits_for_current_and_updated_copy(self):
        self.add_helper()
        self.install_project()
        helper = self.installed('scripts/helper.py')
        for mode in (0o644, 0o744, 0o754):
            with self.subTest(mode=mode):
                helper.chmod(mode)
                self.assert_preserved_failure('executable mode changed')
        self.revise('feature')
        self.assert_preserved_failure('executable mode changed')

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_ordinary_read_write_changes_and_repeat_sync_preserve_modes_and_timestamps(self):
        self.add_helper()
        self.install_project()
        for target in ('codex', 'claude'):
            self.installed('SKILL.md', target).chmod(0o600)
            self.installed(catalog.RECEIPT, target).chmod(0o640)
            self.installed('scripts/helper.py', target).chmod(0o711)
        before = snapshot(self.project, timestamps=True)
        jobs = harness.sync_project(self.project)
        self.assertTrue(all(job['action'] == 'unchanged' for job in jobs))
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_legacy_receipts_reject_undeclared_bits_before_update(self):
        self.install_project()
        self.legacy_receipts()
        self.installed().chmod(0o755)
        self.add_helper()
        self.assert_preserved_failure('Undeclared executable')

    def test_legacy_receipts_allow_new_helper_paths_and_record_current_contract(self):
        self.install_project()
        self.legacy_receipts()
        self.add_helper()
        harness.sync_project(self.project)
        for target in ('codex', 'claude'):
            self.assertEqual(json.loads(self.installed(catalog.RECEIPT, target).read_text())['executable_files'],
                             ['scripts/helper.py'])

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_new_declaration_cannot_disguise_legacy_ordinary_file_execute_drift(self):
        self.install_project()
        self.legacy_receipts()
        for target in ('codex', 'claude'):
            self.installed('references/guide.md', target).chmod(0o755)
        self.entry('feature')['executable_files'] = ['references/guide.md']
        self.assert_preserved_failure('Undeclared executable')

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_legacy_existing_helper_requires_review_without_prior_mode_contract(self):
        self.add_helper()
        self.install_project()
        self.legacy_receipts()
        self.assert_preserved_failure('Undeclared executable')

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_legacy_receipt_does_not_authorize_unknown_removed_executable(self):
        self.add_helper()
        self.install_project()
        self.legacy_receipts()
        (self.repo / 'skills/feature/scripts/helper.py').unlink()
        self.entry('feature')['executable_files'] = []
        self.revise('feature')
        self.assert_preserved_failure('Undeclared executable')

    def test_invalid_prior_executable_contract_is_not_accepted_as_unchanged(self):
        self.install_project()
        receipt = self.installed(catalog.RECEIPT)
        previous = json.loads(receipt.read_text())
        for invalid in ('SKILL.md', [False], ['../escape'], [catalog.RECEIPT]):
            with self.subTest(invalid=invalid):
                receipt.write_text(json.dumps(dict(previous, executable_files=invalid)))
                self.assert_preserved_failure()

    @unittest.skipIf(os.name == 'nt', 'POSIX fixture modes exercised through Windows branch')
    def test_windows_does_not_treat_posix_permission_bits_as_integrity(self):
        self.add_helper()
        self.install_project()
        self.installed().chmod(0o755)
        self.installed('scripts/helper.py').chmod(0o600)
        # Replace the module's OS facade, not process-global os.name / pathlib.
        windows_os = SimpleNamespace(**{name: getattr(os, name) for name in dir(os)})
        windows_os.name = 'nt'
        with mock.patch.object(catalog, 'os', windows_os), mock.patch.object(harness, 'os', windows_os):
            _, unchanged = catalog.check_destination(self.entry('feature'), self.project, 'claude')
            self.assertTrue(unchanged)
            self.revise('feature')
            jobs = harness.sync_project(self.project)
            self.assertEqual([job['action'] for job in jobs if job['id'] == 'feature'], ['update', 'update'])

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_mode_change_during_preparation_is_detected_before_any_publish(self):
        self.install_project()
        self.revise('foundation')
        self.revise('feature')
        original = catalog.prepare_payload
        after_mutation = []
        def prepare(*args, **kwargs):
            result = original(*args, **kwargs)
            if not after_mutation:
                self.installed().chmod(0o755)
                after_mutation.append(snapshot(self.project, timestamps=True))
            return result
        with mock.patch.object(catalog, 'prepare_payload', side_effect=prepare):
            with self.assertRaisesRegex(ValueError, 'changed since preflight'):
                harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), after_mutation[0])

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_mode_change_after_validation_cannot_become_the_preflight_baseline(self):
        self.install_project()
        original = catalog.check_destination
        after_mutation = []
        def check(entry, project, target):
            result = original(entry, project, target)
            if entry['id'] == 'feature' and target == 'claude':
                self.installed().chmod(0o755)
                after_mutation.append(snapshot(self.project, timestamps=True))
            return result
        with mock.patch.object(catalog, 'check_destination', side_effect=check):
            with self.assertRaisesRegex(ValueError, 'changed since preflight'):
                harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), after_mutation[0])

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_mode_change_during_staging_is_preserved_before_item_replacement(self):
        self.install_project()
        self.revise('feature')
        original = harness.shutil.copytree
        changed = self.installed(target='codex')
        mutated = []
        before = snapshot(self.project, timestamps=True)
        def copytree(source, destination, *args, **kwargs):
            result = original(source, destination, *args, **kwargs)
            destination = Path(destination)
            if destination.parent == changed.parent.parent and '.new-' in destination.name:
                changed.chmod(0o755)
                mutated.append(True)
            return result
        with mock.patch.object(harness.shutil, 'copytree', side_effect=copytree):
            with self.assertRaisesRegex(ValueError, 'changed during staging'):
                harness.sync_project(self.project)
        self.assertEqual(mutated, [True])
        expected = dict(before)
        relative = changed.relative_to(self.project).as_posix()
        expected[relative] = ('file', changed.read_bytes(), 0o755, before[relative][-1])
        # Creating/removing the staging directory necessarily changes its parent
        # directory timestamp; user files and the lock must remain exact.
        observed = snapshot(self.project, timestamps=True)
        self.assertEqual({key: value[:1] if value[0] == 'directory' else value
                          for key, value in observed.items()},
                         {key: value[:1] if value[0] == 'directory' else value
                          for key, value in expected.items()})

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_source_mode_does_not_change_reviewed_hash_or_grant_execute_permission(self):
        source = self.repo / 'skills/feature/SKILL.md'
        source.chmod(0o755)
        self.install_project()
        for target in ('codex', 'claude'):
            self.assertEqual(self.installed(target=target).stat().st_mode & 0o111, 0)
            self.assertEqual(catalog.payload_hash(catalog.existing_payload(self.installed(target=target).parent)),
                             self.entry('feature')['sha256'])


if __name__ == '__main__':
    unittest.main()
