"""Observable evaluation lifecycle, bounded execution and stale evidence checks."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

from lib import evaluation


class EvaluationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.suite = self.root / 'evaluations/suite.json'
        case = self.suite.parent / 'cases/sum-test'
        (case / 'workspace').mkdir(parents=True)
        self.case = case
        self.row = {'id': 'sum-test', 'domain': 'engineering', 'title': 'Sum entries',
                    'prompt': 'cases/sum-test/task.md', 'required_evidence': ['Correct result'],
                    'review_required': True}
        self.suite.write_text(json.dumps({'schema_version': 1, 'cases': [self.row]}))
        (case / 'task.md').write_text('Fix totals in the supplied workspace.')
        (case / 'rubric.md').write_text('Check correctness and clarity.')
        (case / 'workspace/candidate.py').write_text('def total(values):\n    return 0\n')
        (case / 'workspace/inputs.json').write_text('[2, -3, 5]')
        self.row['immutable_inputs'] = ['inputs.json']
        self.suite.write_text(json.dumps({'schema_version': 1, 'cases': [self.row]}))
        (case / 'verify.py').write_text('''import json, sys
sys.path.insert(0, sys.argv[1])
from candidate import total
passed = total([2, -3, 5]) == 4 and total([]) == 0
print(json.dumps({'checks': [{'id': 'totals', 'status': 'passed' if passed else 'failed', 'detail': 'Signed entries and empty input'}]}))
sys.exit(0 if passed else 1)
''')
        self.source = self.root / 'harness'
        (self.source / 'skills/example').mkdir(parents=True)
        (self.source / 'skills/example/SKILL.md').write_text('Example source snapshot')
        self.patch = mock.patch.object(evaluation, 'SUITE', self.suite)
        self.patch.start()
        self.addCleanup(self.patch.stop)
        self.run = self.root / 'run'

    def prepare(self):
        return evaluation.prepare('sum-test', self.run, 'fixture-model', 'candidate', self.source)

    def solve(self):
        (self.run / 'workspace/candidate.py').write_text('def total(values):\n    return sum(values)\n')

    def accept(self, kind='agent'):
        return evaluation.review(self.run, 'fixture reviewer', kind, 'accepted',
                                 ['candidate.py'], 'Reviewed behavior and implementation.')

    def test_role_contract_changes_are_part_of_harness_provenance(self):
        registry = self.source / 'registry'
        registry.mkdir()
        roles = registry / 'capabilities.json'
        roles.write_text('{"role":"cash"}')
        before = evaluation.harness_inputs(self.source)
        self.assertIn('registry/capabilities.json', before['files'])
        roles.write_text('{"role":"profit"}')
        self.assertNotEqual(before['sha256'], evaluation.harness_inputs(self.source)['sha256'])

    def assert_registry_mutation_changes_provenance(self, name, original, changed):
        path = self.source / 'registry' / name
        path.parent.mkdir(exist_ok=True)
        path.write_text(json.dumps(original))
        before = evaluation.harness_inputs(self.source)
        key = 'registry/' + name
        self.assertIn(key, before['files'])
        path.write_text(json.dumps(changed))
        after = evaluation.harness_inputs(self.source)
        self.assertNotEqual(before['files'][key], after['files'][key])
        self.assertNotEqual(before['sha256'], after['sha256'])

    def test_target_destinations_are_part_of_harness_provenance(self):
        self.assert_registry_mutation_changes_provenance('targets.json',
            {'codex': {'instructions': '.codex/AGENTS.md'}},
            {'codex': {'instructions': '.codex/OTHER.md'}})

    def test_mcp_configuration_is_part_of_harness_provenance(self):
        self.assert_registry_mutation_changes_provenance('mcp.json',
            {'openai-docs': {'enabled': True}}, {'openai-docs': {'enabled': False}})

    def test_explicit_codex_preferences_are_part_of_harness_provenance(self):
        self.assert_registry_mutation_changes_provenance('codex-settings.json',
            {'config': {'model_verbosity': 'low'}}, {'config': {'model_verbosity': 'high'}})

    def test_prepare_copies_inputs_and_does_not_replace_existing_work(self):
        result = self.prepare()
        self.assertEqual(result['status'], 'prepared')
        self.assertEqual((self.run / 'workspace/candidate.py').read_bytes(),
                         (self.case / 'workspace/candidate.py').read_bytes())
        self.solve()
        with self.assertRaises(ValueError):
            self.prepare()
        self.assertIn('sum(values)', (self.run / 'workspace/candidate.py').read_text())

    def test_failed_candidate_then_correct_behavior_needs_separate_review(self):
        self.prepare()
        failed = evaluation.check(self.run)
        self.assertEqual(failed['status'], 'failed')
        self.solve()
        passed = evaluation.check(self.run)
        self.assertEqual(passed['status'], 'passed')
        report = evaluation.report([self.run])['runs'][0]
        self.assertEqual(report['automated'], 'passed')
        self.assertFalse(report['accepted'])
        self.accept()
        self.assertTrue(evaluation.report([self.run])['runs'][0]['accepted'])

    def test_changed_artifact_invalidates_both_checks_and_review(self):
        self.prepare()
        self.solve()
        evaluation.check(self.run)
        self.accept()
        (self.run / 'workspace/candidate.py').write_text('def total(values):\n    return 1\n')
        row = evaluation.report([self.run])['runs'][0]
        self.assertEqual((row['automated'], row['review']), ('stale', 'stale'))
        self.assertFalse(row['accepted'])

    def test_report_preserves_current_review_identity_and_kind(self):
        for kind in ('agent', 'human'):
            with self.subTest(kind=kind):
                self.run = self.root / ('run-' + kind)
                self.prepare()
                self.solve()
                evaluation.check(self.run)
                self.accept(kind)
                row = evaluation.report([self.run])['runs'][0]
                self.assertTrue(row['accepted'])
                self.assertEqual(row['review_kind'], kind)
                self.assertEqual(row['reviewer'], 'fixture reviewer')
                self.assertEqual(row['accepted_by'], kind)

    def test_report_without_required_review_has_no_reviewer_acceptance(self):
        self.row['review_required'] = False
        self.suite.write_text(json.dumps({'schema_version': 1, 'cases': [self.row]}))
        self.prepare()
        self.solve()
        evaluation.check(self.run)
        report = evaluation.report([self.run])
        row = report['runs'][0]
        self.assertTrue(row['accepted'])
        self.assertEqual(row['review'], 'not_required')
        self.assertIsNone(row['review_kind'])
        self.assertIsNone(row['reviewer'])
        self.assertEqual(row['accepted_by'], 'none')
        self.assertEqual(report['review_kind_counts'], {})
        self.assertEqual(report['accepted_by_counts'], {'none': 1})

    def test_report_counts_only_current_valid_reviews_and_completed_acceptance(self):
        runs = []
        for scenario in ('agent', 'human', 'declined', 'invalid', 'stale', 'pending', 'failed-check'):
            self.run = self.root / ('run-' + scenario)
            self.prepare()
            self.solve()
            if scenario == 'failed-check':
                (self.run / 'workspace/candidate.py').write_text('def total(values):\n    return 1\n')
            evaluation.check(self.run)
            if scenario == 'declined':
                evaluation.review(self.run, 'human reviewer', 'human', 'needs_changes',
                                  ['candidate.py'], 'Needs a clearer implementation.')
            elif scenario != 'pending':
                self.accept('human' if scenario == 'human' else 'agent')
            if scenario == 'invalid':
                value = json.loads((self.run / 'review.json').read_text())
                value['reviewer'] = ''
                (self.run / 'review.json').write_text(json.dumps(value))
            elif scenario == 'stale':
                (self.run / 'workspace/candidate.py').write_text('def total(values):\n    return 2\n')
            runs.append(self.run)
        report = evaluation.report(runs)
        rows = {Path(row['run']).name: row for row in report['runs']}
        self.assertEqual(report['review_kind_counts'], {'agent': 2, 'human': 2})
        self.assertEqual(report['accepted_by_counts'], {'agent': 1, 'human': 1, 'none': 5})
        for scenario, status in (('invalid', 'error'), ('stale', 'stale'), ('pending', 'pending')):
            with self.subTest(scenario=scenario):
                row = rows['run-' + scenario]
                self.assertEqual(row['review'], status)
                self.assertIsNone(row['review_kind'])
                self.assertIsNone(row['reviewer'])
                self.assertFalse(row['accepted'])
                self.assertEqual(row['accepted_by'], 'none')
        for scenario, kind in (('declined', 'human'), ('failed-check', 'agent')):
            with self.subTest(scenario=scenario):
                row = rows['run-' + scenario]
                self.assertEqual(row['review_kind'], kind)
                self.assertFalse(row['accepted'])
                self.assertEqual(row['accepted_by'], 'none')

    def test_editing_a_success_label_does_not_accept_a_failed_check(self):
        self.prepare()
        evaluation.check(self.run)
        checks = json.loads((self.run / 'checks.json').read_text())
        checks['status'] = 'passed'
        (self.run / 'checks.json').write_text(json.dumps(checks))
        row = evaluation.report([self.run])['runs'][0]
        self.assertEqual(row['automated'], 'error')
        self.assertFalse(row['accepted'])

    def test_immutable_fixture_changes_are_rejected(self):
        self.prepare()
        (self.run / 'workspace/inputs.json').write_text('[0]')
        with self.assertRaises(ValueError):
            evaluation.check(self.run)
        with self.assertRaises(ValueError):
            evaluation.report([self.run])

    def test_review_of_only_seed_inputs_does_not_accept_output(self):
        self.prepare()
        self.solve()
        evaluation.check(self.run)
        with self.assertRaises(ValueError):
            evaluation.review(self.run, 'Reviewer', 'agent', 'accepted', ['inputs.json'], 'Read inputs.')

    def test_case_specific_review_artifacts_are_required(self):
        self.row['review_artifacts'] = ['candidate.py']
        self.suite.write_text(json.dumps({'schema_version': 1, 'cases': [self.row]}))
        self.prepare()
        self.solve()
        (self.run / 'workspace/note.md').write_text('An unrelated new note')
        with self.assertRaises(ValueError):
            evaluation.review(self.run, 'Reviewer', 'agent', 'accepted', ['note.md'], 'Read note.')
        self.accept()

    def test_changed_model_label_invalidates_recorded_evidence(self):
        self.prepare()
        self.solve()
        evaluation.check(self.run)
        record = json.loads((self.run / 'run.json').read_text())
        record['model'] = 'different-model'
        (self.run / 'run.json').write_text(json.dumps(record))
        self.assertEqual(evaluation.report([self.run])['runs'][0]['automated'], 'stale')

    def test_changed_case_definition_or_control_file_is_not_executed(self):
        self.prepare()
        (self.run / 'verify.py').write_text("raise RuntimeError('Should not run')")
        with self.assertRaises(ValueError):
            evaluation.check(self.run)
        (self.run / 'verify.py').write_bytes((self.case / 'verify.py').read_bytes())
        (self.case / 'task.md').write_text('Different task')
        with self.assertRaises(ValueError):
            evaluation.check(self.run)

    def test_review_requires_existing_contained_evidence(self):
        self.prepare()
        for files in ([], ['missing.png'], ['../run.json']):
            with self.subTest(files=files), self.assertRaises(ValueError):
                evaluation.review(self.run, 'Reviewer', 'human', 'accepted', files, 'Inspected.')
        self.assertFalse((self.run / 'review.json').exists())

    def test_workspace_links_are_rejected_before_check(self):
        self.prepare()
        try:
            (self.run / 'workspace/external').symlink_to(self.source, target_is_directory=True)
        except OSError:
            self.skipTest('Host does not support symlinks')
        with self.assertRaises(ValueError):
            evaluation.check(self.run)
        self.assertFalse((self.run / 'checks.json').exists())

    def test_verifier_timeout_and_large_output_do_not_produce_success(self):
        for program in ('import time; time.sleep(10)', "print('x' * 2000000)"):
            with self.subTest(program=program):
                code = self.root / 'child.py'
                code.write_text(program)
                with self.assertRaises(ValueError):
                    evaluation.execute([sys.executable, '-I', str(code)], self.root, 1)

    def test_malformed_or_nonzero_verifier_cannot_pass(self):
        for program in ("print('not-json')", "print('{\"checks\":[]}')",
                        "print('{\"checks\":[{\"id\":\"ok\",\"status\":\"passed\",\"detail\":\"Claim\"}]}'); raise SystemExit(1)"):
            with self.subTest(program=program):
                (self.case / 'verify.py').write_text(program)
                run = self.root / ('run-' + str(len(list(self.root.glob('run-*')))))
                evaluation.prepare('sum-test', run, 'fixture-model', 'candidate', self.source)
                self.assertNotEqual(evaluation.check(run)['status'], 'passed')

    def test_verifier_workspace_mutation_is_an_error(self):
        (self.case / 'verify.py').write_text('''import pathlib, json, sys
pathlib.Path(sys.argv[1], 'candidate.py').write_text('overwritten')
print(json.dumps({'checks':[{'id':'ok','status':'passed','detail':'Claim'}]}))
''')
        self.prepare()
        result = evaluation.check(self.run)
        self.assertEqual(result['status'], 'error')
        self.assertIn('changed workspace', result['detail'])

    def test_suite_rejects_duplicate_cases_and_malformed_json_root(self):
        self.suite.write_text(json.dumps({'schema_version': 1, 'cases': [self.row, self.row]}))
        with self.assertRaises(ValueError):
            evaluation.suite()
        self.suite.write_text('[]')
        with self.assertRaises(ValueError):
            evaluation.suite()


class ShippedEvaluationTests(unittest.TestCase):
    def test_http_fixture_detects_misclassified_refusal_and_partial_retrieval(self):
        source = '''import json
from urllib.error import HTTPError
from urllib.parse import urlencode
from urllib.request import Request, urlopen
class MutationUncertain(Exception): pass
def list_all(base, token):
    items, cursor = [], None
    while True:
        url = base + '/items' + ('?' + urlencode({'cursor':cursor}) if cursor else '')
        try:
            with urlopen(Request(url, headers={'Authorization':'Bearer '+token}), timeout=2) as response:
                page = json.load(response)
        except HTTPError:
            if PARTIAL: return items
            raise
        items.extend(page['items'])
        cursor = page['next_cursor']
        if cursor is None: return items
def create_item(base, token, name):
    request = Request(base+'/items', data=json.dumps({'name':name}).encode(),
                      headers={'Authorization':'Bearer '+token, 'Content-Type':'application/json'})
    try:
        with urlopen(request, timeout=2) as response: return json.load(response)
    except HTTPError as error:
        if REFUSAL: raise MutationUncertain() from error
        raise
    except Exception as error:
        raise MutationUncertain() from error
'''
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            for refusal, partial, failed in ((False, False, set()),
                                             (True, False, {'refused-write'}),
                                             (False, True, {'failed-later-page'})):
                run = root / ('run-' + str(refusal) + '-' + str(partial))
                evaluation.prepare('integration-paginated-client', run, 'fixture-model', 'verifier-regression')
                (run / 'workspace/client.py').write_text(
                    'REFUSAL = ' + repr(refusal) + '\nPARTIAL = ' + repr(partial) + '\n' + source)
                result = evaluation.check(run)
                self.assertEqual({item['id'] for item in result['checks'] if item['status'] == 'failed'}, failed)
                self.assertEqual(result['status'], 'failed' if failed else 'passed')

    def test_development_cases_cover_domains_and_broken_seeds_fail(self):
        self.assertEqual({case['domain'] for case in evaluation.suite()['cases']}, set(evaluation.DOMAINS))
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            for case in evaluation.suite()['cases']:
                with self.subTest(case=case['id']):
                    run = root / case['id']
                    evaluation.prepare(case['id'], run, 'fixture-model', 'broken-seed')
                    before = evaluation.tree(run / 'workspace')
                    result = evaluation.check(run)
                    self.assertEqual(result['status'], 'failed')
                    self.assertEqual(before, evaluation.tree(run / 'workspace'))


if __name__ == '__main__':
    unittest.main()
