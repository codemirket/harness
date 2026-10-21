import datetime
import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from check import audit, validate_exceptions
from check import coordinates

class AuditTests(unittest.TestCase):
    def test_repository_review_records_are_well_formed(self):
        root = Path(__file__).resolve().parents[2]
        entries = json.loads((root / 'evals/dependency-audit/exceptions.json').read_text())
        validate_exceptions(entries, root, datetime.date.today(), [])

    def test_openpencil_and_nested_alias_coordinates_are_covered(self):
        root = Path(__file__).resolve().parents[2]
        found = coordinates(root)
        self.assertIn(('npm', '@open-pencil/cli', '0.16.0'), found)
        self.assertIn(('npm', '@open-pencil/core', '0.16.0'), found)
        self.assertIn(('npm', '@open-pencil/yoga-layout', '3.3.0-grid.3'), found)
        self.assertIn(('npm', 'canvaskit-wasm', '0.41.1'), found)

    def test_new_findings_and_incomplete_responses_fail(self):
        packages = [('npm', 'fixture', '1.0')]
        query = lambda _: {'results': [{'vulns': [{'id': 'GHSA-fixture'}]}]}
        self.assertFalse(audit(packages, set(), query, advisory_policy='fail')['ok'])
        self.assertTrue(audit(packages, {('npm', 'fixture', '1.0', 'GHSA-fixture')}, query)['ok'])
        with self.assertRaises(ValueError):
            audit(packages, set(), lambda _: {'results': []})
        with self.assertRaises(ValueError):
            audit(packages, set(), lambda _: {'results': [{'error':'unavailable'}]})

    def test_report_policy_keeps_findings_and_rejects_invalid_inventory(self):
        query = lambda _: {'results': [{'vulns': [{'id': 'GHSA-fixture'}]}]}
        result = audit([('npm', 'fixture', '1.0')], set(), query)
        self.assertTrue(result['ok'])
        self.assertTrue(result['advisories_present'])
        self.assertEqual(len(result['untriaged']), 1)
        for invalid in ([], [('npm', '', '1.0')]):
            with self.assertRaises(ValueError): audit(invalid, set(), query)
        for bad in ({'vulns': None}, {'vulns': [{}]}, {'vulns': [{'id': ''}]}):
            with self.assertRaises(ValueError):
                audit([('npm', 'fixture', '1.0')], set(), lambda _: {'results': [bad]})

    def test_exception_expires_and_is_bound_to_reviewed_source(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); source = root / 'source.rs'; source.write_text('bounded read')
            entry = dict(ecosystem='crates.io',name='zip',version='6.0.0',advisory='fixture',owner='maintainers',reason='manual bounded read',expires='2026-11-20',source='source.rs',source_sha256=hashlib.sha256(source.read_bytes()).hexdigest())
            validate_exceptions([entry], root, datetime.date(2026, 10, 20))
            with self.assertRaises(ValueError):
                validate_exceptions([entry], root, datetime.date(2026, 11, 20))
            stale = []
            self.assertEqual(validate_exceptions([entry], root, datetime.date(2026, 11, 20), stale), set())
            self.assertIn('expired', stale[0]['reason'])
            source.write_text('extract all')
            stale = []
            self.assertEqual(validate_exceptions([entry], root, datetime.date(2026, 10, 20), stale), set())
            self.assertIn('source changed', stale[0]['reason'])
            with self.assertRaises(ValueError):
                validate_exceptions([entry], root, datetime.date(2026, 10, 20))

if __name__ == '__main__': unittest.main()
