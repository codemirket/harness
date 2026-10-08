"""Check the shipped catalog, especially hashes after edits to authored skills."""
import importlib.util
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    'shipped_catalog', ROOT / 'lib/catalog.py')
catalog = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(catalog)


class CatalogDataTests(unittest.TestCase):
    def test_default_project_skills_do_not_duplicate_global_discovery_names(self):
        import json
        config = json.loads((ROOT / 'registry/harness.json').read_text())
        data = catalog.load_catalog()
        entries = {entry['id']: entry for entry in data['skills']}
        global_names = {entries[identifier]['name'] for identifier in config['global_skills']}
        defaults = catalog.resolve_selection(data, profiles=config['project_defaults']['profiles'])
        self.assertFalse(global_names.intersection(entry['name'] for entry in defaults))

    def test_global_and_project_defaults_retain_the_rich_foundation_with_portable_option(self):
        import json
        config = json.loads((ROOT / 'registry/harness.json').read_text())
        data = catalog.load_catalog()
        self.assertEqual(config['project_defaults']['profiles'], ['project-foundation'])
        selected = catalog.resolve_selection(data, profiles=config['project_defaults']['profiles'])
        expected = {
            'work-planning', 'context-management', 'agent-coordination',
            'research-and-synthesis', 'security-judgment', 'architecture-review',
            'debugging', 'test-design', 'ci-maintenance', 'release-operations',
            'document-parsing', 'office-authoring',
        }
        default_ids = {entry['id'] for entry in selected}
        self.assertEqual(default_ids, {'architecture-review', 'debugging', 'test-design', 'release-operations'})
        self.assertTrue(expected.issubset(default_ids | set(config['global_skills'])))
        portable = catalog.resolve_selection(data, profiles=['portable-foundation'])
        self.assertEqual({entry['id'] for entry in portable}, expected)
        for entry in portable:
            with self.subTest(entry=entry['id']):
                self.assertEqual(entry['scope'], 'project')
                self.assertEqual(entry['delivery'], 'local')
                self.assertEqual(set(entry.get('agents', ['codex', 'claude'])),
                                 {'codex', 'claude'})

    def test_project_entries_have_unique_names_and_reproducible_provenance(self):
        data = catalog.load_catalog()
        names = {}
        for entry in data['skills']:
            with self.subTest(entry=entry['id']):
                self.assertIn(entry['scope'], ('global', 'project', 'manual', 'excluded'))
                if entry['scope'] != 'project':
                    continue
                for other in names.get(entry['name'], []):
                    self.assertIn(other['id'], entry.get('conflicts', []))
                    self.assertIn(entry['id'], other.get('conflicts', []))
                names.setdefault(entry['name'], []).append(entry)
                self.assertRegex(entry['sha256'], r'^[a-f0-9]{64}$')
                if entry['delivery'] == 'upstream':
                    source = data['sources'][entry['source']]
                    self.assertRegex(source['commit'], r'^[a-f0-9]{40}$')
                    self.assertTrue(entry['license_files'])
                else:
                    self.assertEqual(entry['delivery'], 'local')
                    payload = catalog.read_local(ROOT, entry)
                    self.assertEqual(catalog.skill_name(payload['SKILL.md']), entry['name'])
                    self.assertEqual(catalog.payload_hash(payload), entry['sha256'],
                                     'Review the changed local skill and update its catalog hash')

    def test_authored_discovery_descriptions_match_the_installed_frontmatter(self):
        for entry in catalog.load_catalog()['skills']:
            if entry.get('delivery') not in ('local', 'global-link'):
                continue
            with self.subTest(entry=entry['id']):
                body = (ROOT / entry['path'] / 'SKILL.md').read_text()
                description = re.search(r'^description: (.+)$', body.split('---', 2)[1], re.M)
                self.assertIsNotNone(description)
                self.assertEqual(entry['description'], description.group(1))

    def test_profiles_resolve_for_both_agents_and_cannot_hide_manual_entries(self):
        data = catalog.load_catalog()
        self.assertTrue(data['profiles'])
        for profile in data['profiles']:
            with self.subTest(profile=profile):
                selected = catalog.resolve_selection(data, profiles=[profile])
                self.assertTrue(selected)
                for entry in selected:
                    self.assertEqual(entry['scope'], 'project')
                    self.assertEqual(set(entry.get('agents', ['codex', 'claude'])),
                                     {'codex', 'claude'})

    def test_inventory_covers_requested_sources_and_points_to_real_catalog_ids(self):
        import json
        data = catalog.load_catalog()
        index = json.loads(catalog.INDEX.read_text())
        required = {'openai', 'google', 'gstack', 'diagram-design', 'open-design',
                    'impeccable', 'adhd', 'awesome', 'context-mode', 'graphify',
                    'marketing', 'ui-ux-pro-max', 'hallmark', 'taste', 'anthropic',
                    'superpowers', 'matt', 'ecc', 'caveman', 'archify', 'emil',
                    'vercel-agent-browser', 'vercel-agent-skills', 'remotion',
                    'addy', 'planning-with-files', 'geo-seo-claude'}
        self.assertTrue(required.issubset({e['source'] for e in index['skills']}))
        by_id = {e['id']: e for e in data['skills']}
        keys = set()
        for entry in index['skills']:
            key = entry['source'], entry['path']
            self.assertNotIn(key, keys)
            keys.add(key)
            for identifier in entry['catalog_ids']:
                selected = by_id[identifier]
                self.assertEqual((selected['source'], selected['path']), key)
            for identifier in entry['installable_ids']:
                self.assertEqual(by_id[identifier]['scope'], 'project')
            if entry['advertisement_only']:
                self.assertFalse(entry['installable_ids'])

    def test_declared_globals_and_all_authored_sources_exist(self):
        import json
        config = json.loads((ROOT / 'registry/harness.json').read_text())
        entries = {entry['id']: entry for entry in catalog.load_catalog()['skills']}
        names = {entries[identifier]['name'] for identifier in config['global_skills']}
        self.assertEqual(len(names), len(config['global_skills']))
        actual = {path.parent.name for path in (ROOT / 'skills').glob('*/SKILL.md')}
        self.assertTrue(names.issubset(actual))
        self.assertEqual(actual, {e['name'] for e in entries.values()
                                  if e.get('delivery') in ('local', 'global-link')})
        agents = (ROOT / 'instructions/AGENTS.md').read_text()
        delegate = (ROOT / 'instructions/CLAUDE.md').read_text()
        self.assertIn('Codex desktop', agents)
        self.assertIn('supporting', delegate)
        self.assertIn('CLI agent', delegate)
        self.assertEqual(config['primary_client'], 'codex-desktop')
        self.assertEqual(config['targets']['claude']['clients'], ['claude-code'])
        self.assertFalse(config['targets']['claude']['desktop_support'])
        self.assertLessEqual(len(agents.splitlines()), 100)


if __name__ == '__main__':
    unittest.main()
