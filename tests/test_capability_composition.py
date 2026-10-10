"""Role selection must be explicit, bounded, portable and preserve project state."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from lib import capabilities
import test_capabilities as fixtures

ROOT = Path(__file__).resolve().parents[1]


class CompositionUnitTests(unittest.TestCase):
    setUp = fixtures.CapabilityTests.setUp
    write = fixtures.CapabilityTests.write

    def test_aliases_cannot_silently_select_another_role(self):
        self.assertEqual(capabilities.resolve(' EXAMPLE-role ', self.root)['id'], 'capability')
        self.assertEqual(capabilities.plan('Example Role', root=self.root)['project_skills'], ['lead'])
        with self.assertRaises(ValueError):
            capabilities.resolve('Please be an example role', self.root)
        self.manifest['capabilities'].append(dict(self.row, id='second', title='Second'))
        self.write()
        with self.assertRaisesRegex(ValueError, 'Ambiguous'):
            capabilities.contracts(self.root)

    def test_brief_rejects_changed_payload_and_never_loads_optional_support(self):
        self.row['support'] = ['optional-not-installed']
        self.write()
        result = capabilities.plan('capability', ['Capability'], self.root)
        self.assertEqual(len(result['read_sequence']), 1)
        self.assertEqual(result['supporting_capabilities'], [])
        self.assertEqual(result['runtime_status'], 'not_checked')
        self.assertEqual(result['activation_status'], 'not_observed')
        self.assertEqual(result['entrypoint_bytes'], len(self.body))
        (self.root / 'skills/lead/SKILL.md').write_bytes(self.body + b'Changed\n')
        with self.assertRaisesRegex(ValueError, 'hash changed'):
            capabilities.plan('capability', root=self.root)

    def test_contract_change_invalidates_brief_fingerprint(self):
        before = capabilities.plan('capability', root=self.root)
        self.row['deliverable'] = 'A different user outcome'
        self.write()
        self.assertNotEqual(before['source_fingerprint'],
                            capabilities.plan('capability', root=self.root)['source_fingerprint'])

    def test_global_leads_do_not_become_project_duplicates(self):
        self.entry.update(scope='global', delivery='global-link')
        self.write()
        (self.root / 'registry/harness.json').write_text('{"global_skills":["lead"]}')
        result = capabilities.plan('capability', root=self.root)
        self.assertEqual(result['project_skills'], [])
        self.assertEqual(result['read_sequence'][0]['delivery'], 'shared')

    def test_contract_requires_aliases_and_acceptance(self):
        for field in ('aliases', 'acceptance'):
            with self.subTest(field=field):
                previous = self.row.pop(field)
                self.write()
                with self.assertRaisesRegex(ValueError, 'missing capability fields'):
                    capabilities.contracts(self.root)
                self.row[field] = []
                self.write()
                with self.assertRaisesRegex(ValueError, 'Invalid capability ' + field):
                    capabilities.contracts(self.root)
                self.row[field] = previous

    def test_contract_accepts_only_initial_release_schema(self):
        for version in (True, 0, 2, '1', None):
            with self.subTest(version=version):
                self.manifest['schema_version'] = version
                self.write()
                with self.assertRaisesRegex(ValueError, 'Unsupported capability schema'):
                    capabilities.contracts(self.root)


class CompositionIntegrationTests(unittest.TestCase):
    def test_every_requested_role_has_a_working_execution_brief(self):
        roles = ['Frontend Designer', 'Backend Engineer', 'Database Admin', 'Devops Engineer',
                 'QA Engineer for Design', 'Graphic Designer', 'CEO', 'CFO', 'CTO',
                 'Deep Researcher', 'Product Manager', 'Marketing Expert', 'Native Translator',
                 'Project Manager', 'IT Expert', 'Optimization Engineer', 'Senior Team Lead Engineer',
                 'Market Analyst Expert', 'Entreprenuer', 'Excel Expert', 'Documentation Expert',
                 'Mobile App Designer', 'Mobile App Engineer', 'System Engineer']
        for role in roles:
            with self.subTest(role=role):
                brief = capabilities.plan(role, root=ROOT)
                self.assertTrue(brief['contracts'][0]['acceptance'])
                self.assertEqual(len(brief['read_sequence']), 1)
                self.assertTrue((ROOT / brief['read_sequence'][0]['path']).is_file())
                self.assertEqual(brief['activation_status'], 'not_observed')

    def test_finance_composition_installs_exact_leads_and_preserves_existing_work(self):
        with tempfile.TemporaryDirectory() as tmp:
            project = Path(tmp)
            sentinel = project / 'draft.txt'
            sentinel.write_text('User work')
            cli = [sys.executable, str(ROOT / 'ai.py'), 'project']
            def run(action, *args):
                result = subprocess.run(cli + [action, '--project', str(project)] + list(args),
                                        text=True, capture_output=True, timeout=30)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                return json.loads(result.stdout)
            run('init', '--target', 'all')
            manifest = project / '.ai/project.json'
            original = manifest.read_bytes()
            run('add', '--capability', 'CFO', '--capability', 'Excel Expert', '--dry-run')
            self.assertEqual(manifest.read_bytes(), original)
            run('add', '--capability', 'CFO', '--capability', 'Excel Expert')
            self.assertEqual(json.loads(manifest.read_text())['skills'],
                             ['financial-analysis', 'spreadsheet-analysis'])
            run('sync'); run('doctor')
            for target in ('.agents', '.claude'):
                actual = {p.name for p in (project / target / 'skills').iterdir()}
                self.assertEqual(actual, {'architecture-review', 'debugging', 'test-design',
                                          'release-operations', 'financial-analysis', 'spreadsheet-analysis'})
            self.assertEqual(sentinel.read_text(), 'User work')
            before = manifest.read_bytes()
            bad = subprocess.run(cli + ['add', '--project', str(project), '--capability', 'Unknown Role'],
                                 text=True, capture_output=True, timeout=30)
            self.assertNotEqual(bad.returncode, 0)
            self.assertEqual(manifest.read_bytes(), before)

    def test_selection_deduplicates_shared_leads_without_including_all_support(self):
        brief = capabilities.plan('CTO', ['Senior Team Lead Engineer', 'Deep Researcher'])
        self.assertEqual([r['id'] for r in brief['read_sequence']],
                         ['technical-leadership', 'research-and-synthesis'])
        self.assertEqual(brief['project_skills'], ['technical-leadership'])
        self.assertNotIn('systems-engineering', brief['project_skills'])


if __name__ == '__main__':
    unittest.main()
