"""Bounded pinned file fetching, isolated from live networks and user directories."""
import copy
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
from urllib.parse import quote

from test_catalog import catalog, reviewed_hash, SKILL, LICENSE, GUIDE


class RemoteFileFetchTests(unittest.TestCase):
    def setUp(self):
        self.source = {'repository': 'example/skills', 'commit': 'a' * 40,
                       'fetch_mode': 'files'}
        self.entry = dict(id='fixture', name='fixture-skill', scope='project',
                          delivery='upstream', source='fixture', path='skills/fixture',
                          license_files=['LICENSE'])
        self.files = {'SKILL.md': SKILL, 'references/guide.md': GUIDE,
                      '.upstream-licenses/LICENSE': LICENSE}
        self.entry['sha256'] = reviewed_hash(self.files)
        self.data = {'sources': {'fixture': self.source}, 'skills': [self.entry]}
        self.raw = {'skills/fixture/SKILL.md': SKILL,
                    'skills/fixture/references/guide.md': GUIDE, 'LICENSE': LICENSE,
                    'huge-unrelated.bin': b'Never fetch this file'}
        self.tree = {'sha': 'c' * 40, 'truncated': False,
                     'tree': [self.item(p, len(b)) for p, b in self.raw.items()]}
        self.tree_url = ('https://api.github.com/repos/example/skills/git/trees/'
                         + 'a' * 40 + '?recursive=1')
        self.urls = []
        self.read_limits = []
        self.tree_override = None
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.project = Path(self.temp.name)
        patcher = mock.patch.object(catalog.urllib.request, 'urlopen', side_effect=self.respond)
        patcher.start()
        self.addCleanup(patcher.stop)

    @staticmethod
    def item(path, size=0, **extra):
        return dict(path=path, type='blob', mode='100644', sha='b' * 40, size=size, **extra)

    def raw_url(self, path):
        return 'https://raw.githubusercontent.com/example/skills/' + 'a' * 40 + '/' + quote(path, safe='/')

    def respond(self, request, timeout):
        self.assertEqual(timeout, 60)
        self.assertFalse(request.has_header('Authorization'))
        self.urls.append(request.full_url)
        if request.full_url == self.tree_url:
            body = self.tree_override if self.tree_override is not None else json.dumps(self.tree).encode()
        else:
            paths = {self.raw_url(path): path for path in self.raw}
            self.assertIn(request.full_url, paths, 'Only pinned expected URLs may be fetched')
            path = paths[request.full_url]
            self.assertNotEqual(path, 'huge-unrelated.bin')
            body = self.raw[path]
        limits = self.read_limits

        class BoundedResponse(io.BytesIO):
            def read(self, n=-1):
                if n < 0:
                    raise AssertionError('Unbounded network read')
                limits.append(n)
                return super().read(n)

        return BoundedResponse(body)

    def prepare(self, entry=None, cache=None):
        return catalog.prepare_payload(self.data, entry or self.entry, archive_cache=cache)

    def assert_no_raw_fetch(self):
        self.assertEqual(self.urls, [self.tree_url])

    def test_prepares_exact_reviewed_payload_and_licenses_without_unrelated_download(self):
        files, source = self.prepare()
        self.assertEqual(files, self.files)
        self.assertEqual(source, self.source)
        self.assertEqual(set(self.urls), {self.tree_url, *(self.raw_url(p) for p in self.raw if p != 'huge-unrelated.bin')})
        self.assertEqual(self.read_limits[0], catalog.MAX_TREE + 1)

    def test_nested_license_is_retained_at_original_and_notice_locations(self):
        self.raw['skills/fixture/LICENSE'] = LICENSE
        self.tree['tree'].append(self.item('skills/fixture/LICENSE', len(LICENSE)))
        files = dict(self.files)
        del files['.upstream-licenses/LICENSE']
        files.update({'LICENSE': LICENSE, '.upstream-licenses/skills/fixture/LICENSE': LICENSE})
        entry = dict(self.entry, license_files=['skills/fixture/LICENSE'], sha256=reviewed_hash(files))
        self.assertEqual(self.prepare(entry)[0], files)
        self.assertNotIn(self.raw_url('LICENSE'), self.urls)

    def test_caches_tree_and_shared_notice_across_entries_and_both_agents(self):
        second_skill = SKILL.replace(b'fixture-skill', b'other-skill')
        self.raw['skills/other/SKILL.md'] = second_skill
        self.tree['tree'].append(self.item('skills/other/SKILL.md', len(second_skill)))
        second_files = {'SKILL.md': second_skill, '.upstream-licenses/LICENSE': LICENSE}
        second = dict(self.entry, id='other', name='other-skill', path='skills/other',
                      sha256=reviewed_hash(second_files))
        catalog.install_many(self.data, [self.entry, second], self.project, 'both')
        self.assertEqual(self.urls.count(self.tree_url), 1)
        self.assertEqual(self.urls.count(self.raw_url('LICENSE')), 1)
        self.assertEqual(len(self.urls), 5)
        for parent in ('.agents', '.claude'):
            self.assertEqual(catalog.existing_payload(self.project / parent / 'skills/other-skill'), second_files)

    def test_same_size_content_tampering_fails_full_reviewed_hash_before_writes(self):
        self.raw['skills/fixture/references/guide.md'] = b'X' * len(GUIDE)
        with self.assertRaisesRegex(ValueError, 'reviewed content'):
            catalog.install_many(self.data, [self.entry], self.project, 'both')
        self.assertEqual(list(self.project.iterdir()), [])

    def test_rejects_malformed_missing_and_truncated_tree(self):
        for payload in (b'not-json', b'\xff', b'[]', b'{}',
                        b'{"tree": [], "truncated": true}',
                        b'{"tree": {}, "truncated": false}',
                        b'{"tree": [], "truncated": 0}',
                        b'{"tree": [], "truncated": false}',
                        b'{"tree": [], "truncated": false, "sha": "main"}'):
            self.urls.clear()
            self.tree_override = payload
            with self.subTest(payload=payload), self.assertRaises(ValueError):
                self.prepare()
            self.assert_no_raw_fetch()

    def test_rejects_invalid_duplicate_or_unsafe_tree_records(self):
        valid = copy.deepcopy(self.tree)
        invalid = [None, {}, self.item('../escape'), self.item('/absolute'),
                   self.item('a//b'), dict(self.item('x'), size=True),
                   dict(self.item('x'), size=-1), dict(self.item('x'), mode=[]),
                   dict(self.item('x'), type='unexpected'), dict(self.item('x'), sha='main'),
                   copy.deepcopy(valid['tree'][0])]
        for item in invalid:
            self.urls.clear()
            self.tree = copy.deepcopy(valid)
            self.tree['tree'].append(item)
            with self.subTest(item=item), self.assertRaises(ValueError):
                self.prepare()
            self.assert_no_raw_fetch()

    def test_rejects_selected_symlink_submodule_and_symlinked_skill_root(self):
        valid = copy.deepcopy(self.tree)
        for item in (dict(self.item('skills/fixture/link'), mode='120000'),
                     dict(self.item('skills/fixture/module'), mode='160000', type='commit'),
                     dict(self.item('skills/fixture'), mode='120000'),
                     dict(self.item('skills'), mode='120000')):
            self.urls.clear()
            self.tree = copy.deepcopy(valid)
            self.tree['tree'].append(item)
            with self.subTest(item=item), self.assertRaises(ValueError):
                self.prepare()
            self.assert_no_raw_fetch()

    def test_missing_or_directory_license_is_rejected_before_raw_fetch(self):
        for directory in (False, True):
            self.urls.clear()
            self.tree['tree'] = [i for i in self.tree['tree'] if i['path'] != 'LICENSE']
            if directory:
                self.tree['tree'].append(dict(self.item('LICENSE'), mode='040000', type='tree'))
            with self.subTest(directory=directory), self.assertRaises(ValueError):
                self.prepare()
            self.assert_no_raw_fetch()

    def test_declared_payload_size_and_file_count_limits_prevent_raw_fetch(self):
        for constant, limit in (('MAX_PAYLOAD', 1), ('MAX_FILES', 1)):
            self.urls.clear()
            with self.subTest(limit=constant), mock.patch.object(catalog, constant, limit):
                with self.assertRaises(ValueError):
                    self.prepare()
            self.assert_no_raw_fetch()

    def test_index_size_is_bounded(self):
        with mock.patch.object(catalog, 'MAX_TREE', 10):
            with self.assertRaises(ValueError):
                self.prepare()
        self.assertEqual(self.read_limits, [11])
        self.assert_no_raw_fetch()

    def test_raw_response_size_must_match_tree_and_read_is_bounded(self):
        for suffix in (b'extra', None):
            self.raw['skills/fixture/SKILL.md'] = SKILL + suffix if suffix else SKILL[:-1]
            self.read_limits.clear()
            with self.subTest(suffix=suffix), self.assertRaises(ValueError):
                self.prepare()
            self.assertEqual(self.read_limits[-1], len(SKILL) + 1)

    def test_selected_portable_path_and_receipt_collisions_fail_before_raw_fetch(self):
        valid = copy.deepcopy(self.tree)
        for paths in (('references/CON.txt',), ('.skill-catalog.json/child',),
                      ('file', 'file/child'), ('Case', 'case')):
            self.urls.clear()
            self.tree = copy.deepcopy(valid)
            self.tree['tree'].extend(self.item('skills/fixture/' + p) for p in paths)
            with self.subTest(paths=paths), self.assertRaises(ValueError):
                self.prepare()
            self.assert_no_raw_fetch()

    def test_quotes_unicode_spaces_hash_and_percent_in_pinned_raw_url(self):
        path = 'skills/fixture/references/ışık #% guide.md'
        self.raw[path] = b'Quoted path'
        self.tree['tree'].append(self.item(path, len(self.raw[path])))
        files = dict(self.files, **{'references/ışık #% guide.md': self.raw[path]})
        self.assertEqual(self.prepare(dict(self.entry, sha256=reviewed_hash(files)))[0], files)
        self.assertIn(self.raw_url(path), self.urls)
        self.assertIn('%23%25', self.raw_url(path))

    def test_unselected_symlink_and_large_blob_do_not_block_or_get_downloaded(self):
        self.tree['tree'].append(dict(self.item('unrelated-link'), mode='120000'))
        next(item for item in self.tree['tree'] if item['path'] == 'huge-unrelated.bin')['size'] = 10**12
        self.assertEqual(self.prepare()[0], self.files)
        self.assertFalse(any('unrelated' in url for url in self.urls))

    def test_rejects_unpinned_source_and_unknown_mode_without_network(self):
        for field, value in (('commit', 'main'), ('repository', 'owner/repo/extra'),
                             ('fetch_mode', 'unknown')):
            original = self.source[field]
            self.source[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                self.prepare()
            self.source[field] = original
        self.assertEqual(self.urls, [])


if __name__ == '__main__':
    unittest.main()
