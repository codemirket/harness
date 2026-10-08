"""Project output explains provider exclusions without inventing install jobs."""
import io
import json
from unittest import mock

from test_harness import HarnessFixture, catalog, harness, snapshot


class ProjectTargetReportingTests(HarnessFixture):
    def setUp(self):
        super().setUp()
        self.entry('optional')['agents'] = ['claude']

    def declare(self, **changes):
        value = {'schema_version': 2, 'targets': ['codex', 'claude'],
                 'profiles': [], 'skills': ['foundation'], 'skip': [],
                 'target_skills': {'claude': ['optional']}}
        value.update(changes)
        harness.write_json(self.project / '.ai/project.json', value)

    def command(self, action):
        with mock.patch('sys.stdout', new_callable=io.StringIO) as output, \
             mock.patch('sys.stderr', new_callable=io.StringIO):
            result = harness.main(['project', action, '--project', str(self.project)])
        return result, json.loads(output.getvalue())

    def assignments(self, rows):
        self.assertIsInstance(rows, list)
        for row in rows:
            self.assertIn('excluded_targets', row, 'Schema v2 rows must explain omitted active targets')
        return {(row['id'], row['target']): row['excluded_targets'] for row in rows}

    def test_plan_sync_and_doctor_report_same_explicit_provider_exclusions(self):
        self.declare()
        expected = {('foundation', 'codex'): [], ('foundation', 'claude'): [],
                    ('optional', 'claude'): [{'target': 'codex', 'reason': 'unsupported'}]}
        before = snapshot(self.project, timestamps=True)
        status, rows = self.command('plan')
        self.assertEqual(status, 0)
        self.assertEqual(self.assignments(rows), expected)
        self.assertEqual(snapshot(self.project, timestamps=True), before)
        status, rows = self.command('sync')
        self.assertEqual(status, 0)
        self.assertEqual(self.assignments(rows), expected)
        self.assertFalse(self.destination('optional', 'codex', True).exists())
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual({(row['id'], target) for row in lock['skills'] for target in row['targets']},
                         set(expected))
        before = snapshot(self.project, timestamps=True)
        for action in ('doctor', 'sync'):
            with self.subTest(action=action):
                status, rows = self.command(action)
                self.assertEqual(status, 0)
                self.assertEqual(self.assignments(rows), expected)
                self.assertTrue(all(row['action'] == 'unchanged' for row in rows))
                self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_compatible_scoped_skill_and_companion_explain_unrequested_target(self):
        self.declare(skills=[], target_skills={'claude': ['feature']})
        expected = {(identifier, 'claude'): [{'target': 'codex', 'reason': 'not_requested'}]
                    for identifier in ('foundation', 'feature')}
        status, rows = self.command('sync')
        self.assertEqual(status, 0)
        self.assertEqual(self.assignments(rows), expected)
        self.assertFalse((self.project / '.agents').exists())
        status, rows = self.command('doctor')
        self.assertEqual(status, 0)
        self.assertEqual(self.assignments(rows), expected)

    def test_required_companion_exclusions_follow_actual_closure_on_both_targets(self):
        self.entry('optional')['requires'] = ['foundation']
        self.declare(skills=[], target_skills={'codex': ['feature'], 'claude': ['optional']})
        expected = {('foundation', 'codex'): [], ('foundation', 'claude'): [],
                    ('feature', 'codex'): [{'target': 'claude', 'reason': 'not_requested'}],
                    ('optional', 'claude'): [{'target': 'codex', 'reason': 'unsupported'}]}
        status, rows = self.command('sync')
        self.assertEqual(status, 0)
        self.assertEqual(self.assignments(rows), expected)

    def test_v2_single_target_does_not_report_inactive_provider_as_excluded(self):
        self.declare(targets=['claude-code'], skills=[], target_skills={'claude-code': ['optional']})
        status, rows = self.command('sync')
        self.assertEqual(status, 0)
        self.assertEqual(self.assignments(rows), {('optional', 'claude'): []})

    def test_v2_shared_only_selection_has_explicit_empty_exclusions(self):
        self.declare(target_skills={'codex': [], 'claude': []})
        status, rows = self.command('plan')
        self.assertEqual(status, 0)
        self.assertEqual(self.assignments(rows), {('foundation', 'codex'): [], ('foundation', 'claude'): []})

    def test_unsupported_explicit_selection_fails_before_preparing_or_writing(self):
        for targets in (['codex'], ['codex', 'claude']):
            self.declare(targets=targets, skills=[], target_skills={'codex': ['optional']})
            before = snapshot(self.project, timestamps=True)
            for action in ('plan', 'sync', 'doctor'):
                with self.subTest(targets=targets, action=action), \
                     mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('No preparation')):
                    with self.assertRaisesRegex(ValueError,
                                                'Project target codex: Skill optional does not support target codex'):
                        self.command(action)
                    self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_empty_selection_on_every_target_fails_without_writes(self):
        self.declare(skills=[], target_skills={'codex': [], 'claude': []})
        before = snapshot(self.project, timestamps=True)
        for action in ('plan', 'sync', 'doctor'):
            with self.subTest(action=action):
                with self.assertRaisesRegex(ValueError, 'Project manifest selects no skills'):
                    self.command(action)
                self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_target_with_no_selection_reports_preserved_copy_as_diagnostic_only(self):
        self.declare(skills=[], target_skills={'claude': ['foundation']})
        catalog.install(self.data, self.entry('foundation'), self.project, 'codex')
        preserved = snapshot(self.project / '.agents', timestamps=True)
        for action in ('plan', 'sync', 'doctor'):
            with self.subTest(action=action):
                status, rows = self.command(action)
                self.assertEqual(status, 0)
                jobs = [row for row in rows if row.get('kind') != 'diagnostic']
                diagnostics = [row for row in rows if row.get('kind') == 'diagnostic']
                self.assertEqual(self.assignments(jobs), {('foundation', 'claude'):
                                                        [{'target': 'codex', 'reason': 'not_requested'}]})
                self.assertEqual(len(diagnostics), 1)
                self.assertEqual(diagnostics[0]['target'], 'codex')
                self.assertEqual(diagnostics[0]['action'], 'preserved')
                self.assertFalse(diagnostics[0]['global_overlap'])
                self.assertEqual(snapshot(self.project / '.agents', timestamps=True), preserved)

    def test_v1_output_and_lock_shape_remain_unchanged(self):
        self.initialize()
        expected_fields = {'id', 'target', 'destination', 'action', 'dependencies', 'caveats'}
        for action in ('plan', 'sync', 'doctor'):
            with self.subTest(action=action):
                status, rows = self.command(action)
                self.assertEqual(status, 0)
                self.assertTrue(rows)
                self.assertTrue(all(set(row) == expected_fields for row in rows))
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['schema_version'], 1)
        self.assertTrue(all(set(row) == {'id', 'sha256', 'installed_sha256', 'source', 'commit'}
                            for row in lock['skills']))
