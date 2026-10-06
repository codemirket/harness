"""Reviewed shared payload mapping and exact adaptation; no live network or installs."""
import copy
import json
import tarfile
from unittest import mock
import unittest
from urllib.parse import unquote

from test_catalog import CatalogFixture, catalog, reviewed_hash, archive_bytes, SKILL, LICENSE


class ExtraFilesTests(CatalogFixture):
    def setUp(self):
        super().setUp()
        (self.source / 'shared').mkdir()
        (self.source / 'shared/method.md').write_bytes(b'# Method\nKeep ../shared/example.md literal.\n')
        self.entry['extra_files'] = {'shared/method.md': 'references/shared-method.md'}
        self.files['references/shared-method.md'] = (self.source / 'shared/method.md').read_bytes()
        self.entry['sha256'] = reviewed_hash(self.files)

    def raw(self):
        return {p.relative_to(self.source).as_posix(): p.read_bytes()
                for p in self.source.rglob('*') if p.is_file() and not p.is_symlink()}

    def prepare(self, mode, entry=None, raw=None):
        entry = entry or self.entry
        if mode == 'local':
            return catalog.prepare_payload(self.data, entry, self.source)[0]
        raw = self.raw() if raw is None else raw
        if mode == 'archive':
            members = [('repo/' + name, content, tarfile.REGTYPE) for name, content in raw.items()]
            with mock.patch.object(catalog, 'download', return_value=archive_bytes(members)):
                return catalog.prepare_payload(self.data, entry)[0]
        data = copy.deepcopy(self.data)
        data['sources']['fixture']['fetch_mode'] = 'files'
        tree = {'sha': 'c' * 40, 'truncated': False, 'tree': [
            dict(path=name, type='blob', mode='100644', sha='b' * 40, size=len(content))
            for name, content in raw.items()]}

        def fetch(url, limit):
            if '/git/trees/' in url:
                return json.dumps(tree).encode()
            path = unquote(url.split('/' + 'a' * 40 + '/', 1)[1])
            self.assertIn(path, set(entry.get('extra_files', {})) | set(entry['license_files'])
                          | {p for p in raw if p.startswith(entry['path'] + '/')})
            self.assertLessEqual(len(raw[path]), limit)
            return raw[path]

        with mock.patch.object(catalog, 'fetch_bytes', side_effect=fetch):
            return catalog.prepare_payload(data, entry)[0]

    def test_all_modes_select_shared_file_retain_license_and_skip_unrelated(self):
        for mode in ('local', 'archive', 'files'):
            with self.subTest(mode=mode):
                self.assertEqual(self.prepare(mode), self.files)

    def test_license_also_explicit_extra_retains_both_distinct_destinations(self):
        entry = dict(self.entry, extra_files={'LICENSE': 'references/license-copy.md'})
        expected = {p: b for p, b in self.files.items() if p != 'references/shared-method.md'}
        expected['references/license-copy.md'] = LICENSE
        entry['sha256'] = reviewed_hash(expected)
        for mode in ('local', 'archive', 'files'):
            with self.subTest(mode=mode):
                self.assertEqual(self.prepare(mode, entry), expected)

    def test_missing_extra_cannot_be_disguised_by_existing_target(self):
        for target in ('missing.md', 'SKILL.md'):
            entry = dict(self.entry, extra_files={'shared/missing.md': target})
            for mode in ('local', 'archive', 'files'):
                with self.subTest(mode=mode, target=target), self.assertRaises(ValueError):
                    self.prepare(mode, entry)

    def test_rejects_duplicate_case_unicode_parent_and_receipt_collisions(self):
        for target in ('SKILL.md', 'skill.md', 'references', '.skill-catalog.json',
                       '.SKILL-CATALOG.json', '.upstream-licenses/LICENSE'):
            entry = dict(self.entry, extra_files={'shared/method.md': target})
            for mode in ('local', 'archive', 'files'):
                with self.subTest(mode=mode, target=target), self.assertRaises(ValueError):
                    self.prepare(mode, entry)
        (self.source / 'shared/other.md').write_bytes(b'another')
        for targets in [('same', 'same'), ('caf\u00e9.md', 'cafe\u0301.md'), ('a', 'a/b')]:
            entry = dict(self.entry, extra_files=dict(zip(['shared/method.md', 'shared/other.md'], targets)))
            with self.subTest(targets=targets), self.assertRaises(ValueError):
                self.prepare('local', entry)

    def test_rejects_unsafe_mapping_names_before_fetch(self):
        for value in ('../escape', '/absolute', 'C:/Windows/file', 'CON.md', 'x\\y', 'trail.'):
            for mapping in ({value: 'valid.md'}, {'shared/method.md': value}):
                entry = dict(self.entry, extra_files=mapping)
                with self.subTest(mapping=mapping), self.assertRaises(ValueError):
                    self.prepare('local', entry)
                with mock.patch.object(catalog, 'fetch_bytes') as fetch:
                    with self.assertRaises(ValueError):
                        catalog.read_remote_files(self.data['sources']['fixture'], entry)
                    fetch.assert_not_called()

    def test_rejects_selected_file_and_ancestor_symlinks_even_within_source(self):
        (self.source / 'shared/link.md').symlink_to(self.source / 'shared/method.md')
        (self.source / 'alias').symlink_to(self.source / 'shared', target_is_directory=True)
        for path in ('shared/link.md', 'alias/method.md'):
            entry = dict(self.entry, extra_files={path: 'extra.md'})
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.prepare('local', entry)
        for path in ('shared/method.md', 'shared'):
            members = self.members + [('repo/' + path, b'', tarfile.SYMTYPE),
                                      ('repo/shared/method.md', b'content', tarfile.REGTYPE)]
            with self.subTest(archive_path=path), self.assertRaises(ValueError):
                catalog.read_archive(archive_bytes(members), self.entry)
        data = copy.deepcopy(self.data['sources']['fixture'])
        for path in ('shared/method.md', 'shared'):
            raw = self.raw()
            tree = {'sha': 'c' * 40, 'truncated': False, 'tree': [
                dict(path=p, type='blob', mode='120000' if p == path else '100644',
                     sha='b' * 40, size=len(b)) for p, b in raw.items()]}
            if path == 'shared':
                tree['tree'].append(dict(path='shared', type='blob', mode='120000', sha='b' * 40, size=1))
            with mock.patch.object(catalog, 'fetch_bytes', return_value=json.dumps(tree).encode()) as fetch:
                with self.subTest(remote_path=path), self.assertRaises(ValueError):
                    catalog.read_remote_files(data, self.entry)
                self.assertEqual(fetch.call_count, 1)

    def test_limits_include_shared_and_duplicate_license_payload_bytes(self):
        for mode in ('local', 'archive', 'files'):
            for limit in ('MAX_PAYLOAD', 'MAX_FILES'):
                size = sum(map(len, self.files.values())) - 1 if limit == 'MAX_PAYLOAD' else len(self.files) - 1
                with mock.patch.object(catalog, limit, size):
                    with self.subTest(mode=mode, limit=limit), self.assertRaises(ValueError):
                        self.prepare(mode)

    def test_exact_replacement_then_note_preserves_source_hash_for_all_modes(self):
        entry = dict(self.entry, replacements=[dict(path='references/shared-method.md',
                     old='../shared/example.md', new='example.md', count=1)], adaptation='Use the shared local reference.')
        expected = dict(self.files)
        expected['references/shared-method.md'] = b'# Method\nKeep example.md literal.\n'
        expected['SKILL.md'] = SKILL.replace(b'\n---\nRead', b'\n---\n\n## Personal catalog integration\n\nUse the shared local reference.\n\nRead')
        entry['installed_sha256'] = reviewed_hash(expected)
        for mode in ('local', 'archive', 'files'):
            with self.subTest(mode=mode):
                self.assertEqual(self.prepare(mode, entry), expected)
        self.assertEqual(entry['sha256'], reviewed_hash(self.files))
        self.assertEqual((self.source / 'shared/method.md').read_bytes(), self.files['references/shared-method.md'])
        catalog.install_many(self.data, [entry], self.project, 'both', {'fixture': self.source})
        self.assertTrue(all('Already installed' in result for result in
                            catalog.install_many(self.data, [entry], self.project, 'both', {'fixture': self.source})))

    def test_source_checksum_is_checked_before_replacement(self):
        entry = dict(self.entry, replacements=[dict(path='references/shared-method.md', old='Method', new='Recipe', count=1)])
        (self.source / 'shared/method.md').write_bytes(b'# Recipe\nKeep ../shared/example.md literal.\n')
        with mock.patch.object(catalog, 'adapt_payload', side_effect=AssertionError('Must not adapt unreviewed source')):
            with self.assertRaisesRegex(ValueError, 'reviewed content'):
                catalog.install_many(self.data, [entry], self.project, 'both', {'fixture': self.source})
        self.assert_project_empty()

    def test_bad_replacements_and_hashes_fail_before_project_writes(self):
        base = dict(path='references/shared-method.md', old='Method', new='Recipe', count=1)
        variants = [dict(base, count=2), dict(base, count=True), dict(base, count=0),
                    dict(base, old=''), dict(base, old='missing'), dict(base, path='../escape'),
                    dict(base, path='missing.md'), dict(base, new=None), dict(base, surprise='x')]
        for change in variants:
            entry = dict(self.entry, replacements=[change])
            with self.subTest(change=change), self.assertRaises(ValueError):
                catalog.install_many(self.data, [entry], self.project, 'both', {'fixture': self.source})
            self.assert_project_empty()
        entry = dict(self.entry, replacements=[base], installed_sha256='0' * 64)
        with self.assertRaisesRegex(ValueError, 'installed hash'):
            catalog.install_many(self.data, [entry], self.project, 'both', {'fixture': self.source})
        self.assert_project_empty()

    def test_extra_directory_is_not_a_file_and_mapping_same_source_collision_fails(self):
        for mapping in ({'shared': 'extra'}, {'skills/fixture/SKILL.md': 'SKILL.md'}):
            entry = dict(self.entry, extra_files=mapping)
            for mode in ('local', 'archive', 'files'):
                with self.subTest(mode=mode, mapping=mapping), self.assertRaises(ValueError):
                    self.prepare(mode, entry)

    def test_archive_rejects_regular_file_as_extra_source_ancestor(self):
        members = self.members + [('repo/shared', b'not a directory', tarfile.REGTYPE),
                                  ('repo/shared/method.md', b'payload', tarfile.REGTYPE)]
        with self.assertRaisesRegex(ValueError, 'ancestor'):
            catalog.read_archive(archive_bytes(members), self.entry)

    def test_replacements_reject_binary_and_skill_name_changes(self):
        binary = dict(self.files, **{'references/shared-method.md': b'\xff'})
        entry = dict(self.entry, replacements=[dict(path='references/shared-method.md', old='x', new='y', count=1)])
        with self.assertRaisesRegex(ValueError, 'UTF-8'):
            catalog.adapt_payload(binary, entry)
        entry = dict(self.entry, replacements=[dict(path='SKILL.md', old='fixture-skill', new='other-skill', count=1)])
        expected = dict(self.files, **{'SKILL.md': SKILL.replace(b'fixture-skill', b'other-skill')})
        entry['installed_sha256'] = reviewed_hash(expected)
        with self.assertRaisesRegex(ValueError, 'skill name'):
            self.prepare('local', entry)

    def test_replacement_expansion_cannot_bypass_payload_limit(self):
        entry = dict(self.entry, replacements=[dict(path='references/shared-method.md', old='Method', new='x' * 1000, count=1)])
        with mock.patch.object(catalog, 'MAX_PAYLOAD', sum(map(len, self.files.values()))):
            with self.assertRaisesRegex(ValueError, 'size limit'):
                self.prepare('local', entry)


if __name__ == '__main__':
    unittest.main()
