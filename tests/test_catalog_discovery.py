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


if __name__ == '__main__':
    unittest.main()
