"""Incremental project declarations stay selective, durable and separately synced."""
import io
import json
import os
import subprocess
import sys
import unittest
from unittest import mock

from test_harness import HarnessFixture, catalog, harness, snapshot


class ProjectAddTests(HarnessFixture):
    def baseline(self):
        self.data['profiles']['project-foundation'] = {
            'description': 'Fundamental project capabilities.', 'skills': ['foundation']}
        self.config['project_defaults'] = {'profiles': ['project-foundation']}
        self.save_registry()

    def declare(self, **changes):
        value = {'schema_version': 1, 'targets': ['codex', 'claude'],
                 'profiles': [], 'skills': ['foundation'], 'skip': []}
        value.update(changes)
        harness.write_json(self.project / '.ai/project.json', value)
        return value

    def assert_add_failure_preserved(self, message='.', **arguments):
        before = snapshot(self.project, timestamps=True)
        with self.assertRaisesRegex(ValueError, message):
            harness.add_project(self.project, **arguments)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_init_uses_configured_baseline_without_installing(self):
        self.baseline()
        value = harness.init_project(self.project, [], [], [], 'all')
        self.assertEqual(value['profiles'], ['project-foundation'])
        self.assertEqual(value['skills'], [])
        self.assertEqual(value['schema_version'], 1)
        self.assertFalse((self.project / '.agents').exists())
        self.assertFalse((self.project / '.ai/project.lock.json').exists())
        self.assertEqual({e['id'] for e in harness.project_plan(self.project)[2]}, {'foundation'})

    def test_init_prepends_baseline_and_deduplicates_explicit_profiles(self):
        self.baseline()
        value = harness.init_project(self.project, ['feature', 'project-foundation', 'feature'],
                                     [], ['optional'], 'codex')
        self.assertEqual(value['profiles'], ['project-foundation', 'feature'])
        self.assertEqual({e['id'] for e in harness.project_plan(self.project)[2]}, {'foundation', 'feature'})

    def test_init_without_default_profiles_uses_explicit_selection(self):
        expected = {'schema_version': 1, 'targets': ['codex', 'claude'],
                    'profiles': ['feature'], 'skills': [], 'skip': ['optional'],
                    'target_skills': {}}
        self.assertEqual(self.initialize(), expected)

    def test_plan_sync_doctor_do_not_inject_new_defaults_into_existing_manifest(self):
        value = self.declare(skills=['optional'])
        self.baseline()
        harness.sync_project(self.project)
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['project', 'doctor', '--project', str(self.project)]), 0)
        self.assertEqual(harness.read_json(self.project / '.ai/project.json'), value)
        self.assertEqual({e['id'] for e in harness.project_plan(self.project)[2]}, {'optional'})
        self.assertFalse(self.destination('foundation', 'codex', True).exists())

    def test_add_shared_skill_keeps_metadata_and_changes_only_manifest(self):
        self.declare(notes={'owner': 'project', 'why': ['Preserve this']}, skip=['optional'])
        harness.sync_project(self.project)
        before = snapshot(self.project, timestamps=True)
        self.baseline()
        result = harness.add_project(self.project, skills=['feature', 'feature'])
        self.assertEqual(result['status'], 'declared')
        self.assertTrue(result['changed'])
        self.assertFalse(result['dry_run'])
        self.assertEqual(result['installation'], 'not_performed')
        self.assertEqual(result['manifest']['profiles'], ['project-foundation'])
        self.assertEqual(result['manifest']['skills'], ['foundation', 'feature'])
        self.assertEqual(result['manifest']['notes'], {'owner': 'project', 'why': ['Preserve this']})
        self.assertEqual(result['manifest']['skip'], ['optional'])
        self.assertEqual(result['manifest']['schema_version'], 1)
        after = snapshot(self.project, timestamps=True)
        for key, value in before.items():
            if key not in ('.ai', '.ai/project.json'):
                self.assertEqual(after[key], value)
        self.assertFalse(self.destination('feature', 'codex', True).exists())
        self.assertEqual({(j['id'], j['target']) for j in result['installations']},
                         {('foundation', 'codex'), ('foundation', 'claude'),
                          ('feature', 'codex'), ('feature', 'claude')})

    def test_add_profile_is_incremental_and_includes_required_companions(self):
        self.declare(skills=['optional'])
        result = harness.add_project(self.project, profiles=['feature'])
        self.assertEqual(result['manifest']['skills'], ['optional'])
        self.assertEqual(result['manifest']['profiles'], ['feature'])
        self.assertEqual({j['id'] for j in result['installations']}, {'optional', 'feature', 'foundation'})

    def test_target_addition_uses_initial_schema_and_preserves_metadata(self):
        self.entry('optional')['agents'] = ['claude']
        self.declare(metadata={'keep': True})
        result = harness.add_project(self.project, target_skills={'claude': ['optional', 'optional']})
        value = result['manifest']
        self.assertEqual(value['schema_version'], 1)
        self.assertEqual(value['targets'], ['codex', 'claude'])
        self.assertEqual(value['target_skills'], {'claude': ['optional']})
        self.assertEqual(value['metadata'], {'keep': True})
        self.assertEqual({(j['id'], j['target']) for j in result['installations']},
                         {('foundation', 'codex'), ('foundation', 'claude'), ('optional', 'claude')})
        harness.sync_project(self.project)
        rows = harness.read_json(self.project / '.ai/project.lock.json')['skills']
        self.assertEqual(next(row['targets'] for row in rows if row['id'] == 'optional'), ['claude'])

    def test_existing_scoped_selection_is_preserved_when_adding_another_target(self):
        self.declare(target_skills={'claude': ['optional']})
        result = harness.add_project(self.project, target_skills={'codex': ['feature']})
        self.assertEqual(result['manifest']['target_skills'],
                         {'claude': ['optional'], 'codex': ['feature']})

    def test_same_addition_is_byte_mode_and_timestamp_idempotent(self):
        self.baseline()
        self.declare()
        harness.add_project(self.project, profiles=['feature'], skills=['feature'])
        path = self.project / '.ai/project.json'
        value = harness.read_json(path)
        path.write_text(json.dumps(value, separators=(',', ':')))
        path.chmod(0o600)
        before = snapshot(self.project, timestamps=True)
        result = harness.add_project(self.project, profiles=['feature'], skills=['feature', 'feature'])
        self.assertEqual(result['status'], 'unchanged')
        self.assertFalse(result['changed'])
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_dry_run_returns_candidate_and_never_changes_files_or_fetches(self):
        self.declare()
        before = snapshot(self.project, timestamps=True)
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('Do not fetch')):
            result = harness.add_project(self.project, skills=['feature'], dry_run=True)
        self.assertEqual(result['status'], 'planned')
        self.assertTrue(result['changed'])
        self.assertTrue(result['dry_run'])
        self.assertEqual(result['manifest']['skills'], ['foundation', 'feature'])
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_existing_addition_does_not_materialize_omitted_empty_fields(self):
        value = {'schema_version': 1, 'targets': ['codex'], 'skills': ['foundation'],
                 'metadata': {'omit_empty_fields': True}}
        harness.write_json(self.project / '.ai/project.json', value)
        before = snapshot(self.project, timestamps=True)
        result = harness.add_project(self.project, skills=['foundation'])
        self.assertEqual(result['manifest'], value)
        self.assertFalse(result['changed'])
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_missing_manifest_or_empty_request_never_initializes_implicitly(self):
        self.assert_add_failure_preserved('project init', skills=['feature'])
        self.declare()
        self.assert_add_failure_preserved('at least one')

    def test_new_explicit_ids_cannot_be_silently_skipped(self):
        self.declare(skip=['optional'])
        self.assert_add_failure_preserved('Cannot add skipped skill: optional', skills=['optional'])
        self.assert_add_failure_preserved('Cannot add skipped skill: optional',
                                          target_skills={'claude': ['optional']})
        result = harness.add_project(self.project, profiles=['feature'])
        self.assertNotIn('optional', {j['id'] for j in result['installations']})

    def test_required_companion_cannot_be_skipped_or_incompatible(self):
        self.declare(skills=['optional'], skip=['foundation'])
        self.assert_add_failure_preserved('Cannot skip required companion', skills=['feature'])
        self.declare(skills=['optional'])
        self.entry('foundation')['agents'] = ['claude']
        self.assert_add_failure_preserved('does not support target codex', skills=['feature'])

    def test_unknown_unreviewed_and_incompatible_additions_preserve_existing_files(self):
        self.declare()
        self.assert_add_failure_preserved('Unknown catalog id', skills=['missing'])
        self.assert_add_failure_preserved('Unknown profile', profiles=['missing'])
        self.assert_add_failure_preserved('not installable', skills=['global-guidance'])
        self.entry('optional')['scope'] = 'manual'
        self.assert_add_failure_preserved('not installable', skills=['optional'])
        self.entry('optional')['scope'] = 'project'
        self.entry('optional')['delivery'] = 'reference'
        self.assert_add_failure_preserved('not installable', skills=['optional'])
        self.entry('optional')['delivery'] = 'local'
        self.entry('optional')['agents'] = ['claude']
        self.assert_add_failure_preserved('does not support target codex', skills=['optional'])

    def test_scoped_add_never_enables_an_inactive_target(self):
        self.declare(targets=['codex'])
        self.assert_add_failure_preserved('not in project targets', target_skills={'claude': ['optional']})
        self.assert_add_failure_preserved('Unsupported target_skills', target_skills={'other': ['optional']})

    def test_conflicts_and_name_collisions_are_checked_per_target(self):
        self.declare(target_skills={'claude': ['optional']})
        self.entry('feature')['conflicts'] = ['optional']
        self.assert_add_failure_preserved('Conflicting selections', skills=['feature'])
        result = harness.add_project(self.project, target_skills={'codex': ['feature']})
        self.assertEqual(result['manifest']['target_skills']['codex'], ['feature'])
        self.entry('feature').pop('conflicts')
        self.entry('optional')['name'] = 'feature'
        self.assert_add_failure_preserved('Competing skill name', target_skills={'claude': ['feature']})

    def test_malformed_default_metadata_blocks_initialization_and_add(self):
        self.declare()
        cases = [None, [], {'profiles': 'feature'}, {'profiles': [1]},
                 {'profiles': ['']}, {'unknown': []}, {'profiles': [], 'extra': True}]
        for defaults in cases:
            with self.subTest(defaults=defaults):
                self.config['project_defaults'] = defaults
                self.save_registry()
                self.assert_add_failure_preserved('project_defaults', skills=['feature'])
                empty = self.base / 'new-project'
                empty.mkdir(exist_ok=True)
                with self.assertRaisesRegex(ValueError, 'project_defaults'):
                    harness.init_project(empty, [], ['feature'], [], 'codex')
                self.assertEqual(snapshot(empty), {})

    def test_defaults_must_resolve_to_approved_authored_project_entries(self):
        self.declare()
        self.baseline()
        self.entry('foundation')['delivery'] = 'upstream'
        self.assert_add_failure_preserved('authored local', skills=['optional'])
        self.entry('foundation')['delivery'] = 'local'
        self.data['profiles']['project-foundation']['skills'] = ['global-guidance']
        self.assert_add_failure_preserved('not installable', skills=['optional'])

    def test_unmanaged_collision_and_modified_copy_preflight_preserve_manifest_and_lock(self):
        self.declare()
        harness.sync_project(self.project)
        collision = self.destination('feature', 'claude', True)
        collision.mkdir()
        (collision / 'SKILL.md').write_text('Project-owned instructions')
        self.assert_add_failure_preserved(skills=['feature'])
        (collision / 'SKILL.md').unlink()
        collision.rmdir()
        modified = self.destination('foundation', 'claude', True) / 'SKILL.md'
        modified.write_bytes(modified.read_bytes() + b'\nPersonal edits')
        self.assert_add_failure_preserved(skills=['feature'])

    @unittest.skipIf(os.name == 'nt', 'POSIX execute-bit integrity')
    def test_permission_drift_blocks_manifest_edit(self):
        self.declare()
        harness.sync_project(self.project)
        ordinary = self.destination('foundation', 'claude', True) / 'SKILL.md'
        ordinary.chmod(0o755)
        self.assert_add_failure_preserved('Undeclared executable', skills=['feature'])

    def test_unsafe_manifest_parent_and_lock_leave_project_unchanged(self):
        self.declare()
        outside = self.base / 'external'
        (self.project / '.ai').rename(outside)
        self.symlink(self.project / '.ai', outside, True)
        self.assert_add_failure_preserved('real destination directory', skills=['feature'])
        (self.project / '.ai').unlink()
        outside.rename(self.project / '.ai')
        (self.project / '.ai/project.lock.json').mkdir()
        self.assert_add_failure_preserved('Unsafe project lock', skills=['feature'])

    def test_manifest_change_during_preflight_is_preserved(self):
        self.declare()
        real_state = harness.project_state
        mutated = []
        def state(*args):
            result = real_state(*args)
            if not mutated:
                path = self.project / '.ai/project.json'
                value = harness.read_json(path)
                value['concurrent_note'] = 'Keep this new user input'
                path.write_text(json.dumps(value))
                mutated.append(snapshot(self.project, timestamps=True))
            return result
        with mock.patch.object(harness, 'project_state', side_effect=state):
            with self.assertRaisesRegex(ValueError, 'manifest changed'):
                harness.add_project(self.project, skills=['feature'])
        self.assertEqual(snapshot(self.project, timestamps=True), mutated[0])

    def test_same_content_parent_replacement_during_preflight_is_rejected(self):
        self.declare()
        original_bytes = (self.project / '.ai/project.json').read_bytes()
        real_state = harness.project_state
        mutated = []
        def state(*args):
            result = real_state(*args)
            if not mutated:
                (self.project / '.ai').rename(self.base / 'previous-ai')
                (self.project / '.ai').mkdir()
                (self.project / '.ai/project.json').write_bytes(original_bytes)
                mutated.append(snapshot(self.project, timestamps=True))
            return result
        with mock.patch.object(harness, 'project_state', side_effect=state):
            with self.assertRaisesRegex(ValueError, 'changed'):
                harness.add_project(self.project, skills=['feature'])
        self.assertEqual(snapshot(self.project, timestamps=True), mutated[0])

    def test_malformed_additions_and_unsupported_targets_are_read_only(self):
        self.declare()
        cases = [({'profiles': 'feature'}, 'must be a string list'),
                 ({'skills': [None]}, 'must be a string list'),
                 ({'skills': ['']}, 'must be a string list'),
                 ({'target_skills': []}, 'must map targets'),
                 ({'target_skills': {'claude': 'optional'}}, 'must map targets'),
                 ({'target_skills': {'claude': [None]}}, 'must map targets'),
                 ({'target_skills': {'claude': ['optional'], 'claude-code': ['feature']}},
                  'Unsupported target_skills target')]
        for arguments, message in cases:
            with self.subTest(arguments=arguments):
                self.assert_add_failure_preserved(message, **arguments)

    def test_add_checks_all_destinations_again_after_preflight(self):
        self.declare()
        harness.sync_project(self.project)
        real_state = harness.project_state
        mutated = []
        def state(entry, project, target):
            result = real_state(entry, project, target)
            if entry['id'] == 'feature' and target == 'claude':
                installed = self.destination('foundation', 'codex', True) / 'SKILL.md'
                installed.write_bytes(installed.read_bytes() + b'\nConcurrent project edit')
                mutated.append(snapshot(self.project, timestamps=True))
            return result
        with mock.patch.object(harness, 'project_state', side_effect=state):
            with self.assertRaisesRegex(ValueError, 'Destination changed since preflight'):
                harness.add_project(self.project, skills=['feature'])
        self.assertEqual(snapshot(self.project, timestamps=True), mutated[0])

    def test_publication_failure_preserves_original_manifest_and_cleans_stage(self):
        self.declare()
        manifest = self.project / '.ai/project.json'
        manifest.chmod(0o600)
        before = snapshot(self.project, timestamps=True)
        with mock.patch.object(harness.os, 'replace', side_effect=OSError('Simulated publication failure')):
            with self.assertRaisesRegex(OSError, 'publication failure'):
                harness.add_project(self.project, skills=['feature'])
        after = snapshot(self.project, timestamps=True)
        self.assertEqual(set(after), set(before))
        for key in before:
            if key != '.ai':  # Creating and removing the temporary file changes parent mtime.
                self.assertEqual(after[key], before[key])

    def test_changed_manifest_keeps_existing_permissions(self):
        self.declare()
        manifest = self.project / '.ai/project.json'
        manifest.chmod(0o600)
        harness.add_project(self.project, skills=['feature'])
        if os.name != 'nt':
            self.assertEqual(manifest.stat().st_mode & 0o777, 0o600)

    def test_cli_add_declares_then_explicit_sync_installs_and_doctor_passes(self):
        self.baseline()
        self.entry('optional')['agents'] = ['claude']
        self.save_registry()
        cli = [sys.executable, '-B', str(self.repo / 'ai.py'), 'project']
        def run(action, *arguments, expected=0):
            result = subprocess.run(cli + [action, '--project', str(self.project)] + list(arguments),
                                    cwd=self.base, text=True, capture_output=True)
            self.assertEqual(result.returncode, expected, result.stderr)
            return json.loads(result.stdout)
        run('init', '--target', 'all')
        run('sync')
        lock_before = (self.project / '.ai/project.lock.json').read_bytes()
        preview = run('add', '--skill', 'feature', '--target-skill', 'claude:optional', '--dry-run')
        self.assertEqual(preview['status'], 'planned')
        result = run('add', '--skill', 'feature', '--target-skill', 'claude:optional')
        self.assertEqual(result['status'], 'declared')
        self.assertEqual(result['installation'], 'not_performed')
        self.assertEqual((self.project / '.ai/project.lock.json').read_bytes(), lock_before)
        self.assertFalse(self.destination('optional', 'claude', True).exists())
        run('doctor', expected=1)
        installed = run('sync')
        self.assertEqual({(j['id'], j['target']) for j in installed},
                         {('foundation', 'codex'), ('foundation', 'claude'),
                          ('feature', 'codex'), ('feature', 'claude'), ('optional', 'claude')})
        run('doctor')
        before = snapshot(self.project, timestamps=True)
        run('add', '--skill', 'feature', '--target-skill', 'claude:optional')
        self.assertTrue(all(j['action'] == 'unchanged' for j in run('sync')))
        self.assertEqual(snapshot(self.project, timestamps=True), before)
