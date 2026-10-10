"""Provider-specific manifests reconcile exact skill/target edges in isolated projects."""
import copy
import io
import json
import subprocess
import sys
from unittest import mock

from test_harness import HarnessFixture, catalog, harness, reviewed_hash, snapshot


class ProjectTargetTests(HarnessFixture):
    def setUp(self):
        super().setUp()
        self.entry('optional')['agents'] = ['claude']

    def declare(self, **changes):
        value = {'schema_version': 1, 'targets': ['codex', 'claude'],
                 'profiles': [], 'skills': ['foundation'], 'skip': [],
                 'target_skills': {'claude': ['optional']}}
        value.update(changes)
        harness.write_json(self.project / '.ai/project.json', value)
        return value

    def doctor(self):
        with mock.patch('sys.stdout', new_callable=io.StringIO), \
             mock.patch('sys.stderr', new_callable=io.StringIO):
            return harness.main(['project', 'doctor', '--project', str(self.project)])

    def edges(self, jobs):
        return {(j['id'], j['target']) for j in jobs}

    def test_mixed_targets_share_only_shared_selection_and_record_exact_lock(self):
        config = self.declare()
        before = snapshot(self.project, timestamps=True)
        _, _, entries, plan = harness.project_plan(self.project)
        expected = {('foundation', 'codex'), ('foundation', 'claude'), ('optional', 'claude')}
        self.assertEqual(self.edges(harness.public_project(plan)), expected)
        self.assertEqual([e['id'] for e in entries], ['foundation', 'optional'])
        self.assertEqual(snapshot(self.project, timestamps=True), before)
        with mock.patch.object(catalog, 'prepare_payload', wraps=catalog.prepare_payload) as prepare:
            self.assertEqual(self.edges(harness.sync_project(self.project)), expected)
        self.assertEqual([call.args[1]['id'] for call in prepare.call_args_list], ['foundation', 'optional'])
        self.assertFalse(self.destination('optional', 'codex', True).exists())
        self.assertTrue(self.destination('optional', 'claude', True).is_dir())
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['schema_version'], 1)
        self.assertEqual(lock['manifest'], config)
        self.assertEqual({row['id']: row['targets'] for row in lock['skills']},
                         {'foundation': ['codex', 'claude'], 'optional': ['claude']})
        self.assertEqual(self.doctor(), 0)

    def test_companions_resolve_only_for_the_target_that_requires_them(self):
        self.entry('feature')['agents'] = ['claude']
        self.declare(skills=[], target_skills={'claude': ['feature']})
        result = harness.sync_project(self.project)
        self.assertEqual(self.edges(result), {('foundation', 'claude'), ('feature', 'claude')})
        self.assertFalse((self.project / '.agents').exists())
        rows = harness.read_json(self.project / '.ai/project.lock.json')['skills']
        self.assertTrue(all(row['targets'] == ['claude'] for row in rows))
        self.assertEqual(self.doctor(), 0)

    def test_shared_and_targeted_companions_are_deduplicated(self):
        self.declare(target_skills={'claude': ['foundation', 'feature', 'feature']})
        result = harness.sync_project(self.project)
        self.assertEqual(len(result), 3)
        self.assertEqual(self.edges(result), {('foundation', 'codex'), ('foundation', 'claude'),
                                              ('feature', 'claude')})

    def test_unsupported_required_companion_blocks_all_writes(self):
        self.entry('feature')['requires'] = ['optional']
        self.declare(target_skills={'codex': ['feature'], 'claude': ['optional']})
        before = snapshot(self.project, timestamps=True)
        with self.assertRaisesRegex(ValueError, 'Project target codex: Skill optional does not support target codex'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_shared_provider_only_selection_is_rejected(self):
        self.declare(skills=['optional'])
        before = snapshot(self.project, timestamps=True)
        for command in ('plan', 'sync', 'doctor'):
            with self.subTest(command=command):
                with self.assertRaisesRegex(ValueError, 'optional does not support target codex'):
                    harness.main(['project', command, '--project', str(self.project)])
                self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_conflicts_are_local_to_each_target(self):
        self.entry('feature')['conflicts'] = ['optional']
        self.declare(target_skills={'codex': ['feature'], 'claude': ['optional']})
        self.assertEqual(self.edges(harness.sync_project(self.project)),
                         {('foundation', 'codex'), ('foundation', 'claude'),
                          ('feature', 'codex'), ('optional', 'claude')})
        self.declare(target_skills={'claude': ['feature', 'optional']})
        before = snapshot(self.project, timestamps=True)
        with self.assertRaisesRegex(ValueError, 'Project target claude: Conflicting selections'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_names_can_repeat_across_targets_but_not_within_one(self):
        entry = self.entry('optional')
        entry['name'] = 'feature'
        source = self.repo / 'skills/optional'
        skill = source / 'SKILL.md'
        skill.write_bytes(skill.read_bytes().replace(b'name: optional', b'name: feature'))
        entry['sha256'] = reviewed_hash({p.relative_to(source).as_posix(): p.read_bytes()
                                        for p in source.rglob('*') if p.is_file()})
        self.declare(target_skills={'codex': ['feature'], 'claude': ['optional']})
        harness.sync_project(self.project)
        for target, identifier in (('codex', 'feature'), ('claude', 'optional')):
            receipt = self.destination('feature', target, True) / catalog.RECEIPT
            self.assertEqual(harness.read_json(receipt)['id'], identifier)
        self.declare(target_skills={'claude': ['feature', 'optional']})
        with self.assertRaisesRegex(ValueError, 'Project target claude: Competing skill name'):
            harness.project_plan(self.project)

    def test_skip_still_applies_to_shared_profiles_and_cannot_remove_companions(self):
        self.declare(profiles=['feature'], skip=['optional'], target_skills={})
        self.assertEqual(self.edges(harness.sync_project(self.project)),
                         {('foundation', 'codex'), ('foundation', 'claude'),
                          ('feature', 'codex'), ('feature', 'claude')})
        self.declare(skills=[], skip=['foundation'], target_skills={'claude': ['feature']})
        with self.assertRaisesRegex(ValueError, 'Cannot skip required companion: foundation'):
            harness.project_plan(self.project)

    def test_explicit_target_skill_cannot_be_silently_skipped(self):
        self.declare(skip=['optional'])
        with self.assertRaisesRegex(ValueError, 'Cannot skip explicit target skill: optional'):
            harness.project_plan(self.project)

    def test_malformed_target_declarations_and_obsolete_schemas_are_read_only(self):
        cases = [
            ({'schema_version': 2}, 'Unsupported project manifest schema'),
            ({'schema_version': 0}, 'Unsupported project manifest schema'),
            ({'schema_version': '1'}, 'Unsupported project manifest schema'),
            ({'schema_version': True}, 'Unsupported project manifest schema'),
            ({'target_skills': None}, 'must map targets'),
            ({'target_skills': []}, 'must map targets'),
            ({'target_skills': {'claude': 'optional'}}, 'must be a string list'),
            ({'target_skills': {'claude': [None]}}, 'must be a string list'),
            ({'target_skills': {'other': ['optional']}}, 'Unsupported target_skills target'),
            ({'target_skills': {'all': ['optional']}}, 'Unsupported target_skills target'),
            ({'targets': ['codex']}, 'not in project targets'),
            ({'targets': ['codex', 'claude', 'claude']}, 'distinct project targets'),
            ({'targets': ['codex', 'claude', 'claude-code']}, 'Unsupported project target'),
            ({'target_skills': {'claude': [], 'claude-code': ['optional']}}, 'Unsupported target_skills target'),
            ({'target_skills': {'claude': ['missing']}}, 'Unknown catalog id'),
            ({'target_skills': {'claude': ['global-guidance']}}, 'not installable'),
        ]
        for changes, message in cases:
            with self.subTest(changes=changes):
                self.declare(**changes)
                before = snapshot(self.project, timestamps=True)
                with self.assertRaisesRegex(ValueError, message):
                    harness.sync_project(self.project)
                self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_unreviewed_and_manual_target_entries_are_not_promoted(self):
        for field, value in (('delivery', 'reference'), ('scope', 'manual')):
            with self.subTest(field=field):
                original = self.entry('optional')[field]
                self.entry('optional')[field] = value
                self.declare()
                with self.assertRaisesRegex(ValueError, 'Selection is not installable: optional'):
                    harness.sync_project(self.project)
                self.entry('optional')[field] = original

    def test_obsolete_target_names_are_rejected_without_rewriting_input(self):
        for target in ('codex-desktop', 'claude-desktop', 'claude-code', 'both', 'all'):
            with self.subTest(target=target):
                original = self.declare(targets=[target], target_skills={})
                before = snapshot(self.project, timestamps=True)
                with self.assertRaisesRegex(ValueError, 'Unsupported project target'):
                    harness.sync_project(self.project)
                self.assertEqual(harness.read_json(self.project / '.ai/project.json'), original)
                self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_cli_initializes_and_validates_target_declarations(self):
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            status = harness.main(['project', 'init', '--project', str(self.project), '--target', 'all',
                                   '--skill', 'foundation', '--target-skill', 'claude:optional',
                                   '--target-skill', 'claude:feature'])
        self.assertEqual(status, 0)
        value = harness.read_json(self.project / '.ai/project.json')
        self.assertEqual(value['schema_version'], 1)
        self.assertEqual(value['target_skills'], {'claude': ['optional', 'feature']})
        for arguments in (['optional'], ['claude:'], [':optional'], ['claude:optional:other'],
                          ['claude:optional', 'claude-code:feature']):
            with self.subTest(arguments=arguments), self.assertRaises(ValueError):
                harness.parse_target_skills(arguments)

    def test_invalid_initial_target_combination_leaves_no_manifest(self):
        with self.assertRaisesRegex(ValueError, 'not in project targets'):
            harness.init_project(self.project, [], ['foundation'], [], 'codex', {'claude': ['optional']})
        self.assertEqual(snapshot(self.project), {})

    def test_modified_provider_copy_prevents_all_updates_and_preserves_lock(self):
        self.declare()
        harness.sync_project(self.project)
        edited = self.destination('optional', 'claude', True) / 'references/guide.md'
        edited.write_bytes(b'Project-owned edits must survive.')
        self.revise('foundation')
        self.revise('optional')
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_targeted_missing_source_prevents_shared_installs(self):
        self.declare()
        (self.repo / 'skills/optional/SKILL.md').unlink()
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_lock_detects_target_edge_drift_and_reconciles_without_payload_changes(self):
        self.declare()
        harness.sync_project(self.project)
        path = self.project / '.ai/project.lock.json'
        expected = harness.read_json(path)
        mutations = [lambda lock: lock['skills'][-1].update(targets=['codex', 'claude']),
                     lambda lock: lock['skills'][0].update(targets=['claude']),
                     lambda lock: lock['skills'][-1].pop('targets'),
                     lambda lock: lock.update(schema_version=2)]
        for mutate in mutations:
            lock = copy.deepcopy(expected)
            mutate(lock)
            harness.write_json(path, lock)
            before = snapshot(self.project, timestamps=True)
            self.assertEqual(self.doctor(), 1)
            self.assertEqual(snapshot(self.project, timestamps=True), before)
            with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('Do not reread source')):
                result = harness.sync_project(self.project)
            self.assertTrue(all(j['action'] == 'unchanged' for j in result))
            self.assertEqual(harness.read_json(path), expected)
            self.assertEqual(self.doctor(), 0)

    def test_one_off_reviewed_copy_is_adopted_without_rewrite_or_sidecar(self):
        catalog.install(self.data, self.entry('optional'), self.project, 'claude')
        before = snapshot(self.destination('optional', 'claude', True), timestamps=True)
        self.declare()
        result = harness.sync_project(self.project)
        optional = next(j for j in result if j['id'] == 'optional')
        self.assertEqual(optional['action'], 'unchanged')
        self.assertEqual(snapshot(self.destination('optional', 'claude', True), timestamps=True), before)
        self.assertEqual(self.doctor(), 0)

    def test_repeat_sync_preserves_payloads_and_lock_bytes_modes_and_timestamps(self):
        self.declare()
        harness.sync_project(self.project)
        before = snapshot(self.project, timestamps=True)
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('No source work')):
            self.assertTrue(all(j['action'] == 'unchanged' for j in harness.sync_project(self.project)))
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_removing_a_target_selection_preserves_its_installed_copy(self):
        self.declare()
        harness.sync_project(self.project)
        destination = self.destination('optional', 'claude', True)
        (destination / 'personal.txt').write_text('Keep me even though removed from selection.')
        before = snapshot(destination, timestamps=True)
        self.declare(target_skills={})
        harness.sync_project(self.project)
        self.assertEqual(snapshot(destination, timestamps=True), before)
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual([row['id'] for row in lock['skills']], ['foundation'])
        self.assertEqual(self.doctor(), 0)

    def test_default_init_and_lock_use_the_full_initial_release_contract(self):
        config = self.initialize()
        self.assertEqual(config['schema_version'], 1)
        self.assertEqual(config['target_skills'], {})
        harness.sync_project(self.project)
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['schema_version'], 1)
        self.assertTrue(all(set(row) == {'id', 'sha256', 'installed_sha256', 'source', 'commit', 'targets'}
                            for row in lock['skills']))
        self.assertTrue(all(row['targets'] == ['codex', 'claude'] for row in lock['skills']))
        before = snapshot(self.project, timestamps=True)
        self.assertEqual(self.doctor(), 0)
        harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_real_cli_entry_point_reconciles_both_targets(self):
        self.save_registry()
        arguments = [sys.executable, str(self.repo / 'ai.py'), 'project']
        init = subprocess.run(arguments + ['init', '--project', str(self.project), '--target', 'all',
                              '--skill', 'foundation', '--target-skill', 'claude:optional'],
                              cwd=self.repo, text=True, capture_output=True)
        self.assertEqual(init.returncode, 0, init.stderr)
        self.assertEqual(json.loads(init.stdout)['schema_version'], 1)
        for action in ('plan', 'sync', 'doctor', 'sync'):
            result = subprocess.run(arguments + [action, '--project', str(self.project)],
                                    cwd=self.repo, text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(self.edges(json.loads(result.stdout)),
                             {('foundation', 'codex'), ('foundation', 'claude'), ('optional', 'claude')})
        self.assertTrue(all(j['action'] == 'unchanged' for j in json.loads(result.stdout)))
