import datetime
import hashlib
import tempfile
import unittest
from pathlib import Path
from check import audit, validate_exceptions
from check import coordinates

class AuditTests(unittest.TestCase):
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
        self.assertFalse(audit(packages, set(), query)['ok'])
        self.assertTrue(audit(packages, {('npm', 'fixture', '1.0', 'GHSA-fixture')}, query)['ok'])
        with self.assertRaises(ValueError):
            audit(packages, set(), lambda _: {'results': []})
        with self.assertRaises(ValueError):
            audit(packages, set(), lambda _: {'results': [{'error':'unavailable'}]})

    def test_exception_expires_and_is_bound_to_reviewed_source(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); source = root / 'source.rs'; source.write_text('bounded read')
            entry = dict(ecosystem='crates.io',name='zip',version='6.0.0',advisory='fixture',owner='maintainers',reason='manual bounded read',expires='2026-11-20',source='source.rs',source_sha256=hashlib.sha256(source.read_bytes()).hexdigest())
            validate_exceptions([entry], root, datetime.date(2026, 10, 20))
            with self.assertRaises(ValueError):
                validate_exceptions([entry], root, datetime.date(2026, 11, 20))
            source.write_text('extract all')
            with self.assertRaises(ValueError):
                validate_exceptions([entry], root, datetime.date(2026, 10, 20))

if __name__ == '__main__': unittest.main()
