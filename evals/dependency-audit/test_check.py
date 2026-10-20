import datetime
import hashlib
import tempfile
import unittest
from pathlib import Path
from check import audit, validate_exceptions
from penpot import lock_coordinates, coordinates

class AuditTests(unittest.TestCase):
    def test_penpot_multidocument_and_scoped_packages_are_not_silently_skipped(self):
        lock = """---
lockfileVersion: '9.0'
packages:
  '@pnpm/exe.linux-arm64@12.8.1':
    resolution: {integrity: fake}
snapshots:
  ignored@1.0.0: {}
---
lockfileVersion: '9.0'
packages:
  'zod@4.6.5':
    resolution: {integrity: fake}
  '@scope/parser@1.0.0-beta.1':
    resolution: {integrity: fake}
"""
        self.assertEqual(len(lock_coordinates(lock)), 3)
        self.assertIn(('npm', '@scope/parser', '1.0.0-beta.1'), lock_coordinates(lock))
        for bad in ["packages: {}", "packages:\n  <<: *defaults", "packages:\n  git+https://private/repo:"]:
            with self.assertRaises(ValueError): lock_coordinates(bad)

    def test_checked_in_penpot_capture_is_bound_to_source_and_all_documents(self):
        root = Path(__file__).resolve().parents[2]
        self.assertGreater(len(coordinates(root)), 250)
        with tempfile.TemporaryDirectory() as temp:
            import shutil
            copy = Path(temp)
            shutil.copytree(root / 'evals/dependency-audit/penpot', copy / 'evals/dependency-audit/penpot')
            (copy / 'registry').mkdir()
            shutil.copyfile(root / 'registry/design-stack.json', copy / 'registry/design-stack.json')
            lock = copy / 'evals/dependency-audit/penpot/pnpm-lock.yaml'
            lock.write_text(lock.read_text() + '\n# drift\n')
            with self.assertRaises(ValueError): coordinates(copy)
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
