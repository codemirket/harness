"""Discovery should surface the requested capability and distinguish review levels."""
import unittest
from test_catalog import catalog


class DiscoveryTests(unittest.TestCase):
    def test_exact_manual_capability_is_not_buried_below_description_matches(self):
        rows = [dict(name='cloud-architect', description='Uses Redis', review_status='indexed-only'),
                dict(name='redis-patterns', description='Redis design', review_status='manual-integration'),
                dict(name='redis', description='Redis database', review_status='manual-integration')]
        ordered = sorted(rows, key=lambda row: catalog.search_rank(row, 'redis'))
        self.assertEqual([row['name'] for row in ordered],
                         ['redis', 'redis-patterns', 'cloud-architect'])

    def test_equal_name_prefers_reviewed_source_to_unreviewed_copy(self):
        rows = [dict(name='testing', source='copy', review_status='indexed-only'),
                dict(name='testing', source='reviewed', review_status='installable-static-review')]
        ordered = sorted(rows, key=lambda row: catalog.search_rank(row, 'testing'))
        self.assertEqual(ordered[0]['source'], 'reviewed')

    def test_multiterm_matching_uses_all_terms_and_metadata(self):
        row = dict(name='data-migrations', description='Backfill a database',
                   source='original', tags=['postgresql'])
        self.assertTrue(catalog.matches(row, 'postgresql migration'))
        self.assertFalse(catalog.matches(row, 'postgresql animation'))

    def test_capability_separators_are_interchangeable(self):
        row = dict(name='review', description='Existing interfaces', tags=['ui-design'])
        for query in ('ui-design', 'UI design', 'ui_design'):
            with self.subTest(query=query):
                self.assertTrue(catalog.matches(row, query))
        self.assertFalse(catalog.matches(row, 'ui animation'))


class ShippedTaskDiscoveryTests(unittest.TestCase):
    def test_documented_invocations_and_profiles_resolve_to_catalog_entries(self):
        import fnmatch
        import re
        from pathlib import Path
        data = catalog.load_catalog()
        by_id = {entry['id']: entry for entry in data['skills']}
        routing = (Path(__file__).resolve().parents[1] /
                   'skills/skill-catalog/references/task-routing.md').read_text()
        pairs = re.findall(r'`([^`]+)` \(invocation `([^`]+)`\)', routing)
        self.assertTrue(pairs, 'The routing guide must exercise the invocation checks')
        for identifier, invocation in pairs:
            with self.subTest(identifier=identifier):
                self.assertIn(identifier, by_id)
                self.assertEqual(by_id[identifier]['name'], invocation)
                self.assertEqual(by_id[identifier]['scope'], 'project')
        for profile in re.findall(r'`([^`]+)` profiles?\b', routing):
            with self.subTest(profile=profile):
                matched = fnmatch.filter(data['profiles'], profile)
                self.assertTrue(matched)
                for name in matched:
                    self.assertTrue(catalog.resolve_selection(data, profiles=[name]))

    def matches(self, query):
        return {row['id'] for row in catalog.load_catalog()['skills']
                if row['scope'] == 'project' and catalog.matches(row, query)}

    def test_ui_design_discovers_existing_redesign_and_reference_workflows(self):
        for query in ('ui-design', 'UI design', 'ui_design'):
            with self.subTest(query=query):
                self.assertTrue({'taste-redesign-skill',
                                 'open-design-reference-design-contract'} <= self.matches(query))

    def test_animation_discovers_the_portable_motion_workflow(self):
        self.assertIn('motion-design', self.matches('animation'))

    def test_docx_discovers_portable_authoring_when_native_tools_are_absent(self):
        self.assertIn('office-authoring', self.matches('docx'))

    def test_professional_role_requests_surface_reviewed_workflows(self):
        # These queries previously surfaced only unreviewed inventory entries.
        import json
        data = catalog.load_catalog()
        index = json.loads(catalog.INDEX.read_text())['skills']
        routes = {
            'database administrator': 'database-systems',
            'DevOps': 'release-operations',
            'quality assurance': 'test-design',
            'deep research': 'research-and-synthesis',
            'product manager': 'product-management',
            'data analysis': 'data-analysis',
            'code graph': 'context-management',
            'repository retrieval': 'context-management',
            'agent tool security': 'security-judgment',
        }
        for query, expected in routes.items():
            with self.subTest(query=query):
                rows = catalog.search_entries(data, index, query)
                self.assertTrue(rows)
                self.assertIn(expected, {identifier for row in rows[:3]
                                        for identifier in row['installable_ids']})
                self.assertTrue(rows[0]['installable_ids'])

    def test_role_names_find_the_reviewed_lead_without_claiming_global_installability(self):
        import json
        data = catalog.load_catalog()
        index = json.loads(catalog.INDEX.read_text())['skills']
        routes = {
            'designer': ('interface-design', False),
            'documenter': ('document-workflow', False),
            'tester': ('test-design', True),
            'planner': ('work-planning', True),
            'debugger': ('debugging', True),
            'marketer': ('marketing-writing', False),
            'illustrator': ('svg-creation', True),
            'animator': ('motion-design', True),
            'researcher': ('research-and-synthesis', True),
        }
        for query, (expected, installable) in routes.items():
            with self.subTest(query=query):
                rows = catalog.search_entries(data, index, query)
                self.assertTrue(rows)
                lead = rows[0]
                self.assertIn(expected, lead['catalog_ids'])
                self.assertEqual(expected in lead['installable_ids'], installable)
                self.assertEqual(lead['review_status'], 'authored')


class CombinedSearchTests(unittest.TestCase):
    def setUp(self):
        self.curated = dict(id='reviewed-redesign', name='redesign', source='vendor',
                            scope='project', delivery='upstream', category='design',
                            description='Improve an existing interface.', tags=['ui-design'])
        self.local = dict(id='office', name='office', scope='project', delivery='local',
                          description='Editable document authoring.', tags=['docx'])
        self.indexed = dict(name='redesign', source='vendor', path='skills/redesign',
                            description='Original upstream description.',
                            catalog_ids=['reviewed-redesign'], installable_ids=['reviewed-redesign'],
                            review_status='installable-static-review')
        self.data = dict(skills=[self.curated, self.local])

    def test_broad_search_uses_reviewed_task_tags_without_mutating_source_inventory(self):
        rows = catalog.search_entries(self.data, [self.indexed], 'UI-design')
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]['catalog_ids'], ['reviewed-redesign'])
        self.assertEqual(rows[0]['description'], 'Improve an existing interface.')
        self.assertEqual(rows[0]['source_description'], 'Original upstream description.')
        self.assertNotIn('tags', self.indexed)
        self.assertEqual(self.indexed['description'], 'Original upstream description.')

    def test_authored_skills_are_discoverable_and_source_filter_still_applies(self):
        rows = catalog.search_entries(self.data, [self.indexed], 'docx')
        self.assertEqual([(r['source'], r['catalog_ids']) for r in rows],
                         [('personal', ['office'])])
        self.assertEqual(catalog.search_entries(self.data, [self.indexed], 'docx', 'vendor'), [])

    def test_global_authored_result_does_not_claim_project_installability(self):
        self.local['scope'] = 'global'
        rows = catalog.search_entries(self.data, [self.indexed], 'docx')
        self.assertEqual(rows[0]['installable_ids'], [])

    def test_unreviewed_inventory_remains_a_discovery_candidate(self):
        row = dict(name='new-tool', source='vendor', description='Unreviewed document tool',
                   catalog_ids=[], review_status='indexed-only')
        rows = catalog.search_entries(self.data, [row], 'document')
        candidate = next(r for r in rows if r['name'] == 'new-tool')
        self.assertEqual(candidate['review_status'], 'indexed-only')
        self.assertEqual(candidate['catalog_ids'], [])

    def test_reviewed_exact_capability_tag_precedes_unreviewed_exact_name(self):
        row = dict(name='docx', source='unknown', description='Document workflow',
                   catalog_ids=[], review_status='indexed-only')
        rows = catalog.search_entries(self.data, [row], 'docx')
        self.assertEqual([r['name'] for r in rows], ['office', 'docx'])

    def test_reviewed_ui_tag_precedes_description_and_name_substring_matches(self):
        row = dict(name='other-ui-design-tool', source='unknown', description='Design tool',
                   catalog_ids=[], review_status='indexed-only')
        rows = catalog.search_entries(self.data, [self.indexed, row], 'UI_design')
        self.assertEqual([r['name'] for r in rows], ['redesign', 'other-ui-design-tool'])


if __name__ == '__main__':
    unittest.main()
