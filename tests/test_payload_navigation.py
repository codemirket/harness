"""Offline navigation regressions using reviewed pinned-link occurrence fixtures.

Full pinned payload verification is recorded separately in the review evidence;
these fixtures exercise the shipped adaptations without a network dependency.
"""
import copy
import json
from pathlib import Path
import re
import tempfile
import unittest
from unittest import mock

from test_catalog import catalog, reviewed_hash


ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/vercel-navigation-2026-10-06.json'


class PayloadNavigationTests(unittest.TestCase):
    def setUp(self):
        self.review = json.loads(EVIDENCE.read_text(encoding='utf-8'))
        self.data = catalog.load_catalog()
        ids = {item['id'] for item in self.review['skills']}
        self.shipped = [entry for entry in self.data['skills'] if entry['id'] in ids]
        self.entries = copy.deepcopy(self.shipped)
        temporary = tempfile.TemporaryDirectory(prefix='payload-navigation-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.source = self.root / 'source'
        self.source.mkdir()
        self.project = self.root / 'project'
        self.project.mkdir()
        (self.source / 'README.md').write_bytes(b'# Fixture source\nMIT declaration.\n')
        self.originals = {}
        for entry in self.entries:
            review = self.review_for(entry)
            directory = self.source / entry['path']
            directory.mkdir(parents=True)
            (directory / 'SKILL.md').write_text(
                '---\nname: ' + entry['name'] + '\ndescription: Fixture.\n---\n'
                '# Root skill\n', encoding='utf-8')
            # Exact URL occurrences retain fragments and duplicate destinations.
            (directory / 'AGENTS.md').write_text('\n'.join(
                '[reviewed navigation](' + item['original'] + ')'
                for item in review['occurrences']) + '\n', encoding='utf-8')
            for resource in {item['included_resource_at'] for item in review['occurrences']}:
                if resource == 'SKILL.md':
                    continue
                path = directory / resource
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('# Included resource\n'
                                'Keep [root](../SKILL.md) relative to this source.\n',
                                encoding='utf-8')
            original = catalog.read_local(self.source, entry)
            self.originals[entry['id']] = original
            # Review the fixture bytes independently of the hash implementation.
            entry['sha256'] = reviewed_hash(original)
            adapted = dict(original)
            adapted['AGENTS.md'] = ('\n'.join(
                '[reviewed navigation](' + item['adapted'] + ')'
                for item in review['occurrences']) + '\n').encode()
            # Retain the independently existing integration-note adaptation.
            note_only = dict(entry, replacements=[])
            adapted['SKILL.md'] = catalog.adapt_payload(original, note_only)['SKILL.md']
            entry['installed_sha256'] = reviewed_hash(adapted)
        network = mock.patch.object(catalog.urllib.request, 'urlopen',
                                    side_effect=AssertionError('Live network forbidden'))
        network.start()
        self.addCleanup(network.stop)

    def review_for(self, entry):
        return next(item for item in self.review['skills'] if item['id'] == entry['id'])

    def install(self):
        return catalog.install_many(self.data, self.entries, self.project, 'both',
                                    {'vercel-agent-skills': self.source})

    def test_catalog_changes_remain_scoped_to_reviewed_navigation_and_provenance(self):
        self.assertEqual(self.review['occurrences'], 30)
        self.assertEqual([len(item['occurrences']) for item in self.review['skills']], [3, 27])
        for entry in self.shipped:
            with self.subTest(entry=entry['id']):
                review = self.review_for(entry)
                source = self.data['sources'][entry['source']]
                self.assertEqual({key: source[key] for key in ('repository', 'commit')},
                                 self.review['source'])
                self.assertEqual(entry['sha256'], review['source_sha256'])
                self.assertEqual(entry['installed_sha256'], review['installed_sha256'])
                self.assertEqual(entry['license_files'], ['README.md'])
                self.assertEqual({item['path'] for item in entry['replacements']}, {'AGENTS.md'})
                self.assertEqual(sum(item['count'] for item in entry['replacements']),
                                 len(review['occurrences']))

    def test_both_targets_install_resolving_links_preserving_fragments_and_source_files(self):
        self.assertEqual(len(self.install()), 4)
        for entry in self.entries:
            expected = [item['adapted'] for item in self.review_for(entry)['occurrences']]
            for agent in ('codex', 'claude'):
                with self.subTest(entry=entry['id'], agent=agent):
                    target, unchanged = catalog.check_destination(entry, self.project, agent)
                    self.assertTrue(unchanged)
                    installed = catalog.existing_payload(target)
                    links = re.findall(r'\]\(([^\s)]+)\)', installed['AGENTS.md'].decode())
                    self.assertEqual(links, expected)
                    for link in links:
                        self.assertTrue((target / link.split('#', 1)[0]).is_file(), link)
                    for path, raw in self.originals[entry['id']].items():
                        if path not in ('AGENTS.md', 'SKILL.md'):
                            self.assertEqual(installed[path], raw, path)
                    receipt = json.loads((target / catalog.RECEIPT).read_text())
                    self.assertEqual(receipt['sha256'], entry['sha256'])
                    self.assertEqual(receipt['installed_sha256'], entry['installed_sha256'])
                    self.assertEqual(receipt['commit'], self.review['source']['commit'])
        before = {str(path): (path.read_bytes(), path.stat().st_mtime_ns)
                  for path in self.project.rglob('*') if path.is_file()}
        self.assertTrue(all(result.startswith('Already installed and unchanged:')
                            for result in self.install()))
        self.assertEqual(before, {str(path): (path.read_bytes(), path.stat().st_mtime_ns)
                                  for path in self.project.rglob('*') if path.is_file()})

    def test_changed_source_is_rejected_before_any_target_writes(self):
        path = self.source / self.entries[-1]['path'] / 'AGENTS.md'
        path.write_bytes(path.read_bytes() + b'Unreviewed upstream change.\n')
        with self.assertRaisesRegex(ValueError, 'Payload differs from the reviewed content'):
            self.install()
        self.assertEqual(list(self.project.iterdir()), [])

    def test_previous_adapted_hash_is_rejected_before_any_target_writes(self):
        for entry in self.entries:
            old_payload = catalog.adapt_payload(self.originals[entry['id']],
                                                dict(entry, replacements=[]))
            entry['installed_sha256'] = reviewed_hash(old_payload)
        with self.assertRaisesRegex(ValueError, 'Adapted payload differs from reviewed installed hash'):
            self.install()
        self.assertEqual(list(self.project.iterdir()), [])

    def test_changed_navigation_adaptation_is_rejected_by_installed_hash(self):
        self.entries[-1]['replacements'][0]['new'] = '](wrong-directory/css-recipes.md'
        with self.assertRaisesRegex(ValueError, 'Adapted payload differs from reviewed installed hash'):
            self.install()
        self.assertEqual(list(self.project.iterdir()), [])

    def test_unexpected_link_occurrence_count_is_rejected_before_writes(self):
        self.entries[-1]['replacements'][0]['count'] += 1
        with self.assertRaisesRegex(ValueError, 'Replacement occurrence count differs: AGENTS.md'):
            self.install()
        self.assertEqual(list(self.project.iterdir()), [])


if __name__ == '__main__':
    unittest.main()
