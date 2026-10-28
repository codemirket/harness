from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import Mock, patch

SCRIPT = Path(__file__).with_name('research-quality-trials.py')
SPEC = importlib.util.spec_from_file_location('research_quality_trials', SCRIPT)
TRIALS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(TRIALS)


class ResearchQualityTrialTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / 'trials'
        self.skill = Path(__file__).parents[1] / 'skills/browser-research'
        TRIALS.freeze(self.root, Path('/bin/echo'), self.skill)
        self.oracle = {key: value['required'] for key, value in TRIALS.CASES.items()}

    def answer(self, case):
        required = self.oracle[case]
        return dict(verdict=required['verdict'], subject=required.get('subject', 'Atlas'),
                    route_status=required.get('route_status', 'available'),
                    project_scope=required.get('project_scope', 'Atlas'),
                    sources=required.get('sources', []),
                    counterexamples=[required['counterexample']] if 'counterexample' in required else [],
                    explanation=' '.join(required.get(key, '') for key in ('missing', 'limitation')))

    def native_fixture(self, case='current-counterexample', condition='with-skill', read=True):
        # Synthetic records test drift checks, not native execution/authenticity.
        trial = self.root / f'{case}-{condition}'
        TRIALS.write_json(trial / 'project/answer.json', self.answer(case))
        events = [{'type': 'turn.completed'}]
        if read:
            events.insert(0, {'type': 'item.completed', 'item': {
                'type': 'command_execution', 'exit_code': 0,
                'command': 'cat .agents/skills/browser-research/SKILL.md',
                'aggregated_output': (self.skill / 'SKILL.md').read_text()}})
        (trial / 'events.jsonl').write_text('\n'.join(json.dumps(event) for event in events))
        (trial / 'stderr.log').write_text('')
        result = dict(exit_code=0, turn_completed=True, timed_out=False, interrupted=False,
                      events_sha256=TRIALS.digest(trial / 'events.jsonl'),
                      stderr_sha256=TRIALS.digest(trial / 'stderr.log'),
                      final_manifest={str(p.relative_to(trial / 'project')): TRIALS.digest(p)
                                      for p in (trial / 'project').rglob('*') if p.is_file()})
        TRIALS.write_json(trial / 'result.json', result)
        return trial, result

    def test_six_cases_with_matched_conditions_and_no_forced_activation(self):
        self.assertEqual(len(TRIALS.CASES), 6)
        for case in TRIALS.CASES:
            for condition in TRIALS.CONDITIONS:
                TRIALS.verify_frozen(self.root, case, condition)
                self.assertFalse((self.root / f'{case}-{condition}/project/AGENTS.md').exists())

    def test_optional_model_cache_is_frozen_and_drift_is_rejected(self):
        cache = self.root.parent / 'models_cache.json'
        cache.write_text('{"models": []}')
        root = self.root.parent / 'cached-trials'
        TRIALS.freeze(root, Path('/bin/echo'), self.skill, cache)
        TRIALS.verify_frozen(root, 'current-counterexample', 'no-skill')
        (root / 'frozen/models_cache.json').write_text('{"models": [{"slug": "changed"}]}')
        with self.assertRaisesRegex(SystemExit, 'frozen inputs changed'):
            TRIALS.verify_frozen(root, 'current-counterexample', 'no-skill')

    def test_missing_or_wrong_skill_refused_before_creating_trial(self):
        for skill in (None, self.root):
            with self.assertRaises(SystemExit):
                TRIALS.freeze(self.root.parent / 'invalid', Path('/bin/echo'), skill)
        self.assertFalse((self.root.parent / 'invalid').exists())

    def test_frozen_oracle_corpus_and_prompt_mutation_rejected(self):
        for relative in ('frozen/oracle.json', 'frozen/corpus/atlas-v2.md', 'current-counterexample-no-skill/prompt.txt'):
            with self.subTest(relative=relative):
                path = self.root / relative
                original = path.read_bytes()
                path.write_bytes(original + b' ')
                with self.assertRaisesRegex(SystemExit, 'frozen inputs changed'):
                    TRIALS.verify_frozen(self.root, 'current-counterexample', 'no-skill')
                path.write_bytes(original)

    def test_clean_answers_and_discriminating_claim_defects(self):
        for case in TRIALS.CASES:
            self.assertTrue(TRIALS.grade_answer(case, self.answer(case), self.oracle)['passed'])
        for case, field, value in (
            ('current-counterexample', 'counterexamples', []),
            ('current-counterexample', 'explanation', 'Available to everyone.'),
            ('stale-source', 'sources', ['atlas-v1']),
            ('swapped-attribution', 'subject', 'Atlas'),
            ('partial-claim', 'verdict', 'supported'),
            ('unavailable-source', 'verdict', 'contradicted'),
            ('wrong-project', 'project_scope', 'Beacon')):
            answer = self.answer(case); answer[field] = value
            self.assertFalse(TRIALS.grade_answer(case, answer, self.oracle)['passed'], (case, field))

    def test_source_paths_and_invalid_counterexample_types_refused(self):
        for value in ('/tmp/atlas-v2.md', '../atlas-v2.md', 'arbitrary/atlas-v2'):
            answer = self.answer('stale-source'); answer['sources'] = [value]
            self.assertFalse(TRIALS.grade_answer('stale-source', answer, self.oracle)['passed'])
        for value in ('atlas-v1', [None], ['unknown'], ['atlas-v1', 'atlas-v1']):
            answer = self.answer('current-counterexample'); answer['counterexamples'] = value
            self.assertFalse(TRIALS.grade_answer('current-counterexample', answer, self.oracle)['passed'])
        self.assertFalse(TRIALS.grade_answer('stale-source', [], self.oracle)['passed'])

    def test_skill_read_is_not_activation_and_control_contamination_is_reported(self):
        for condition in TRIALS.CONDITIONS:
            self.native_fixture(condition=condition)
            result = TRIALS.grade_trial(self.root, 'current-counterexample', condition)
            self.assertTrue(result['skill_read_observed'])
            self.assertIsNone(result['selection_observed'])
            self.assertTrue(result['execution_observed'])
            self.assertEqual(result['condition_contaminated'], condition == 'no-skill')
            self.assertEqual(result['outcome_observed'], condition == 'with-skill')

    def test_command_string_alone_does_not_prove_read(self):
        trial, result = self.native_fixture(read=False)
        (trial / 'events.jsonl').write_text(json.dumps({'type': 'item.completed', 'item': {
            'type': 'command_execution', 'command': 'echo cat .agents/skills/browser-research/SKILL.md', 'exit_code': 0}}))
        result['events_sha256'] = TRIALS.digest(trial / 'events.jsonl')
        TRIALS.write_json(trial / 'result.json', result)
        grade = TRIALS.grade_trial(self.root, 'current-counterexample', 'with-skill')
        self.assertFalse(grade['skill_read_observed'])
        self.assertFalse(grade['execution_observed'])

    def test_timeout_nonzero_or_changed_native_artifact_cannot_pass_outcome(self):
        trial, result = self.native_fixture()
        for change in ({'timed_out': True}, {'exit_code': 1}, {'events_sha256': '0'*64}):
            TRIALS.write_json(trial / 'result.json', {**result, **change})
            self.assertFalse(TRIALS.grade_trial(self.root, 'current-counterexample', 'with-skill')['outcome_observed'])
        TRIALS.write_json(trial / 'result.json', result)
        path = trial / 'project/answer.json'; path.write_text(path.read_text() + ' ')
        self.assertFalse(TRIALS.grade_trial(self.root, 'current-counterexample', 'with-skill')['outcome_observed'])

    def test_extra_output_is_not_hidden_by_answer_basename(self):
        trial, _ = self.native_fixture()
        (trial / 'project/nested').mkdir()
        (trial / 'project/nested/answer.json').write_text('{}')
        with self.assertRaisesRegex(SystemExit, 'answer must be at project root'):
            TRIALS.grade_trial(self.root, 'current-counterexample', 'with-skill')

    def test_empty_out_of_scope_directory_is_rejected(self):
        (self.root / 'stale-source-no-skill/project/unrequested').mkdir()
        with self.assertRaisesRegex(SystemExit, 'trial directories changed'):
            TRIALS.verify_frozen(self.root, 'stale-source', 'no-skill')

    def test_deadline_runtime_identity_and_launch_cleanup(self):
        state = self.root / 'cohort-state.json'
        def budget(deadline):
            TRIALS.write_json(state, {'started_at': deadline-1200, 'deadline_at': deadline,
                                     'model': TRIALS.MODEL, 'reasoning_effort': TRIALS.REASONING_EFFORT,
                                     'repairs': 0, 'retries': 0})
            (self.root / 'cohort-state.sha256').write_text(TRIALS.digest(state))
        budget(time.time()-1)
        with self.assertRaisesRegex(SystemExit, 'budget exhausted'):
            TRIALS.run_native(self.root, 'stale-source', 'no-skill', Path('/bin/echo'), Path('/nonexistent'))
        budget(time.time()+60)
        with self.assertRaisesRegex(SystemExit, 'differs from freeze'):
            TRIALS.run_native(self.root, 'stale-source', 'no-skill', Path('/bin/cat'), Path('/nonexistent'))
        auth = self.root.parent / 'fixture-auth'; auth.write_text('synthetic credential')
        with patch.object(TRIALS.subprocess, 'Popen', side_effect=OSError('launch failed')):
            with self.assertRaisesRegex(OSError, 'launch failed'):
                TRIALS.run_native(self.root, 'stale-source', 'no-skill', Path('/bin/echo'), auth)
        self.assertFalse((self.root / 'stale-source-no-skill/codex-home/auth.json').exists())
        with self.assertRaises(FileExistsError):
            TRIALS.run_native(self.root, 'stale-source', 'no-skill', Path('/bin/echo'), auth)
        with self.assertRaisesRegex(SystemExit, 'already started'):
            TRIALS.run_cohort(self.root, Path('/bin/echo'), auth)

    def test_interruption_retains_output_cleans_auth_and_stops_cohort(self):
        auth = self.root.parent / 'fixture-auth'; auth.write_text('synthetic credential')
        process = Mock(pid=12345, returncode=-15)
        process.communicate.side_effect = [KeyboardInterrupt(), (b'{"type":"turn.started"}\n', b'startup interrupted')]
        process.poll.return_value = -15
        with patch.object(TRIALS.subprocess, 'Popen', return_value=process) as launch, patch.object(TRIALS.os, 'killpg'):
            results = TRIALS.run_cohort(self.root, Path('/bin/echo'), auth)
        self.assertEqual(launch.call_count, 1)
        self.assertEqual(len(results), 1)
        self.assertTrue(results[0]['interrupted'])
        self.assertFalse(results[0]['turn_completed'])
        trial = self.root / 'current-counterexample-no-skill'
        self.assertEqual((trial / 'stderr.log').read_text(), 'startup interrupted')
        self.assertIn('turn.started', (trial / 'events.jsonl').read_text())
        self.assertFalse((trial / 'codex-home/auth.json').exists())
        self.assertEqual(json.loads((self.root / 'cohort-result.json').read_text())['status'], 'stopped-incomplete-execution')
        self.assertFalse((self.root / 'current-counterexample-with-skill/started.json').exists())

    def test_real_interrupted_child_retains_streams_and_leaves_no_auth(self):
        executable = (self.root.parent / 'fake-codex').resolve()
        executable.write_text(f'#!{sys.executable}\n' +
            'import os,signal,time\n' +
            "print('{\"type\":\"turn.started\"}',flush=True)\n" +
            'time.sleep(0.2)\nos.kill(os.getppid(),signal.SIGINT)\ntime.sleep(60)\n')
        executable.chmod(0o700)
        root = self.root.parent / 'process-trial'
        TRIALS.freeze(root, executable, self.skill)
        auth = self.root.parent / 'process-auth'; auth.write_text('synthetic credential')
        result = subprocess.run([sys.executable, str(SCRIPT), '--mode', 'cohort', '--root', str(root),
                                 '--codex', str(executable), '--auth-file', str(auth)], capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        record = json.loads((root / 'cohort-result.json').read_text())
        self.assertEqual(record['status'], 'stopped-incomplete-execution')
        self.assertEqual(len(record['results']), 1)
        self.assertTrue(record['results'][0]['interrupted'])
        self.assertIn('turn.started', (root / 'current-counterexample-no-skill/events.jsonl').read_text())
        self.assertFalse(list(root.glob('*/codex-home/auth.json')))

    def test_timeout_stops_before_spending_another_cell(self):
        result = dict(interrupted=False, timed_out=True, exit_code=-15, turn_completed=False)
        (self.root / 'current-counterexample-no-skill/stderr.log').write_text('routing timeout')
        with patch.object(TRIALS, 'run_native', return_value=result) as run:
            self.assertEqual(TRIALS.run_cohort(self.root, Path('/bin/echo'), Path('/unused')), [result])
        self.assertEqual(run.call_count, 1)

    def test_changed_deadline_and_budget_settings_are_refused(self):
        path = self.root / 'cohort-state.json'
        start = time.time()
        original = {'started_at': start, 'deadline_at': start+1200, 'model': TRIALS.MODEL,
                    'reasoning_effort': TRIALS.REASONING_EFFORT, 'repairs': 0, 'retries': 0}
        TRIALS.write_json(path, original)
        marker = self.root / 'cohort-state.sha256'
        marker.write_text(TRIALS.digest(path))
        self.assertEqual(TRIALS.cohort_budget(self.root), original)
        TRIALS.write_json(path, {**original, 'deadline_at': start+2400})
        with self.assertRaisesRegex(SystemExit, 'budget changed'):
            TRIALS.cohort_budget(self.root)
        for change in ({'deadline_at': start+2400}, {'retries': 1}, {'model': 'another'}, {'started_at': float('nan')}):
            TRIALS.write_json(path, {**original, **change})
            marker.write_text(TRIALS.digest(path))
            with self.assertRaises(SystemExit): TRIALS.cohort_budget(self.root)

    def test_constant_support_answer_does_not_pass_every_case(self):
        # Same syntactically valid answer for all cases, not an invalid schema.
        answer = self.answer('current-counterexample')
        outcomes = [TRIALS.grade_answer(case, answer, self.oracle)['passed'] for case in TRIALS.CASES]
        self.assertTrue(outcomes[0])
        self.assertGreaterEqual(outcomes.count(False), 3)

    def test_review_calibration_rejects_constant_pass_false_findings_and_wrong_locator(self):
        good = {key: {'verdict': verdict, **TRIALS.REVIEW_CASES[key]}
                for key, verdict in TRIALS.REVIEW_ORACLE.items()}
        self.assertTrue(TRIALS.calibrate_reviews(good)['passed'])
        constant = {key: {'verdict': 'pass'} for key in good}
        self.assertEqual(len(TRIALS.calibrate_reviews(constant)['failed_cases']), 4)
        false_finding = {**good, 'clean': {'verdict': 'defect', 'claim': 'Missing import', 'source': 'atlas-v2'}}
        self.assertEqual(TRIALS.calibrate_reviews(false_finding)['failed_cases'], ['clean'])
        wrong_source = {**good, 'swapped': {**good['swapped'], 'source': 'atlas-v2'}}
        self.assertEqual(TRIALS.calibrate_reviews(wrong_source)['failed_cases'], ['swapped'])


if __name__ == '__main__': unittest.main()
