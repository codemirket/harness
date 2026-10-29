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

import research_oracles as TRIALS

class ResearchQualityOracleTests(unittest.TestCase):
    def setUp(self):
        self.oracle = {key: value["required"] for key, value in TRIALS.CASES.items()}

    def answer(self, case):
        required = self.oracle[case]
        return dict(verdict=required['verdict'], subject=required.get('subject', 'Atlas'),
                    route_status=required.get('route_status', 'available'),
                    project_scope=required.get('project_scope', 'Atlas'),
                    sources=required.get('sources', []),
                    counterexamples=[required['counterexample']] if 'counterexample' in required else [],
                    explanation=' '.join(required.get(key, '') for key in ('missing', 'limitation')))

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

    def test_partial_claim_allows_relevant_history_without_replacing_current_source(self):
        answer = self.answer('partial-claim')
        answer['sources'] = ['atlas-v1.md', 'atlas-v2.md']
        self.assertTrue(TRIALS.grade_answer('partial-claim', answer, self.oracle)['passed'])
        for sources in (['atlas-v1'], ['atlas-v2', 'beacon'], ['atlas-v2', 'atlas-v2']):
            answer['sources'] = sources
            self.assertFalse(TRIALS.grade_answer('partial-claim', answer, self.oracle)['passed'])
        answer = self.answer('stale-source')
        answer['sources'] = ['atlas-v1', 'atlas-v2']
        self.assertFalse(TRIALS.grade_answer('stale-source', answer, self.oracle)['passed'])

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

