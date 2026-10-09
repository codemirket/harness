"""Isolated behavior and safety tests for the project skill catalog installer."""
import copy
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import stat
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock

SCRIPT = Path(__file__).resolve().parents[1] / 'lib/catalog.py'
SPEC = importlib.util.spec_from_file_location('personal_skill_catalog', SCRIPT)
catalog = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(catalog)

SKILL = b'---\nname: fixture-skill\ndescription: A fixture skill.\n---\nRead [guide](references/guide.md).\n'
LICENSE = b'MIT License\nCopyright Fixture Author\n'
GUIDE = b'# Guide\nUse the actual project conventions.\n'


def reviewed_hash(files):
    """Produce a review-time fixture digest before any mutation under test."""
    digest = hashlib.sha256()
    for name, content in sorted(files.items()):
        digest.update(name.encode() + b'\0' + hashlib.sha256(content).digest())
    return digest.hexdigest()


def archive_bytes(members):
    """Build test archives in memory, including deliberately unsafe headers."""
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode='w:gz') as archive:
        for name, content, kind in members:
            member = tarfile.TarInfo(name)
            if kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
                member.type = kind
                member.linkname = '/outside/private'
                archive.addfile(member)
            elif kind == tarfile.DIRTYPE:
                member.type = kind
                archive.addfile(member)
            else:
                member.size = len(content)
                archive.addfile(member, io.BytesIO(content))
    return buffer.getvalue()


class CatalogFixture(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / 'source'
        self.project = self.root / 'project'
        self.project.mkdir()
        self.skill_directory = self.source / 'skills/fixture'
        (self.skill_directory / 'references').mkdir(parents=True)
        (self.skill_directory / 'SKILL.md').write_bytes(SKILL)
        (self.skill_directory / 'references/guide.md').write_bytes(GUIDE)
        (self.source / 'LICENSE').write_bytes(LICENSE)
        (self.source / 'unrelated.txt').write_text('Never install this file.')
        self.files = {
            'SKILL.md': SKILL,
            'references/guide.md': GUIDE,
            '.upstream-licenses/LICENSE': LICENSE,
        }
        self.entry = {
            'id': 'fixture/skill', 'name': 'fixture-skill',
            'scope': 'project', 'delivery': 'upstream', 'source': 'fixture',
            'path': 'skills/fixture', 'license_files': ['LICENSE'],
            'sha256': reviewed_hash(self.files),
        }
        self.data = {'schema_version': 1, 'sources': {
            'fixture': {'repository': 'example/skills', 'commit': 'a' * 40},
        }, 'skills': [self.entry]}
        self.members = [
            ('repo/skills/fixture/SKILL.md', SKILL, tarfile.REGTYPE),
            ('repo/skills/fixture/references/guide.md', GUIDE, tarfile.REGTYPE),
            ('repo/LICENSE', LICENSE, tarfile.REGTYPE),
            ('repo/other/SKILL.md', b'Unrelated skill', tarfile.REGTYPE),
        ]
        # Accidental network requests are errors in every test, not a fallback.
        self.network = mock.patch.object(catalog.urllib.request, 'urlopen',
                                         side_effect=AssertionError('Live network forbidden'))
        self.network.start()
        self.addCleanup(self.network.stop)

    def install(self, agent='codex', entry=None):
        return catalog.install(self.data, entry or self.entry, self.project,
                               agent, self.source)

    def target(self, agent='codex'):
        return self.project / ('.agents' if agent == 'codex' else '.claude') / 'skills/fixture-skill'

    def assert_project_empty(self):
        self.assertEqual(list(self.project.iterdir()), [])


class PathTests(unittest.TestCase):
    def test_detects_windows_junctions_without_requiring_windows(self):
        path = mock.Mock(spec=Path)
        for mode, attributes, expected in (
                (stat.S_IFDIR | 0o755, 0x400, True),
                (stat.S_IFREG | 0o644, 0x400, True),
                (stat.S_IFLNK | 0o777, 0, True),
                (stat.S_IFDIR | 0o755, 0, False)):
            path.lstat.return_value = SimpleNamespace(
                st_mode=mode, st_file_attributes=attributes)
            with self.subTest(mode=mode, attributes=attributes):
                self.assertEqual(catalog.is_link(path), expected)
        path.lstat.side_effect = FileNotFoundError
        self.assertFalse(catalog.is_link(path))

    def test_rejects_absolute_traversal_and_ambiguous_paths(self):
        for value in ('', '.', '..', '../escape', 'a/../escape', '/absolute',
                      'a//b', 'a/./b', 'a/', 'C:/Windows/file',
                      'C:\\Windows\\file', '\\\\server\\share', 'file:stream'):
            with self.subTest(path=value), self.assertRaises(ValueError):
                catalog.safe_path(value)

    def test_rejects_windows_reserved_or_unrepresentable_segments(self):
        for value in ('CON', 'nul.txt', 'aux/data', 'COM1.log', 'LPT9',
                      'references/con.txt', 'name.', 'name ', 'a?/file',
                      'a*/file', 'bad<name', 'bad|name', 'bad"name', 'bad\x00name'):
            with self.subTest(path=value), self.assertRaises(ValueError):
                catalog.safe_path(value)

    def test_allows_portable_unicode_spaces_and_punctuation(self):
        for value in ('SKILL.md', 'references/use cases.md', 'references/ışık.md',
                      '.upstream-licenses/LICENSE', 'assets/version-2.1.txt'):
            with self.subTest(path=value):
                self.assertEqual(str(catalog.safe_path(value)), value)


class ArchiveTests(CatalogFixture):
    def test_selects_complete_skill_payload_and_licenses_only(self):
        self.assertEqual(catalog.read_archive(archive_bytes(self.members), self.entry), self.files)

    def test_license_inside_skill_is_kept_at_original_and_notice_paths(self):
        entry = copy.deepcopy(self.entry)
        entry['license_files'] = ['skills/fixture/LICENSE']
        members = self.members + [('repo/skills/fixture/LICENSE', LICENSE, tarfile.REGTYPE)]
        result = catalog.read_archive(archive_bytes(members), entry)
        self.assertEqual(result['LICENSE'], LICENSE)
        self.assertEqual(result['.upstream-licenses/skills/fixture/LICENSE'], LICENSE)
        self.assertNotIn('.upstream-licenses/LICENSE', result)

    def test_rejects_unsafe_tar_names_even_outside_selected_payload(self):
        for name in ('../escape', '/absolute/file', 'repo/../escape',
                     'repo/skills/fixture/../../escape', 'repo/a\\b', 'C:/escape'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                catalog.read_archive(archive_bytes(self.members + [(name, b'x', tarfile.REGTYPE)]), self.entry)

    def test_rejects_selected_symlinks_and_hardlinks(self):
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
            with self.subTest(kind=kind), self.assertRaises(ValueError):
                catalog.read_archive(archive_bytes(self.members + [
                    ('repo/skills/fixture/link', b'', kind)]), self.entry)

    def test_rejects_duplicate_payload_file(self):
        with self.assertRaises(ValueError):
            catalog.read_archive(archive_bytes(self.members + [self.members[0]]), self.entry)

    def test_rejects_case_collisions_that_alias_on_windows_and_macos(self):
        members = self.members + [('repo/skills/fixture/skill.md', b'Alias', tarfile.REGTYPE)]
        with self.assertRaises(ValueError):
            catalog.read_archive(archive_bytes(members), self.entry)

    def test_rejects_unicode_normalization_aliases_before_extraction(self):
        # Build names in memory: APFS cannot preserve both distinct fixture paths.
        for first, second in (('caf\u00e9.txt', 'cafe\u0301.txt'),
                              ('caf\u00e9/one.md', 'cafe\u0301/two.md')):
            members = self.members + [
                ('repo/skills/fixture/' + first, b'First', tarfile.REGTYPE),
                ('repo/skills/fixture/' + second, b'Second', tarfile.REGTYPE),
            ]
            with self.subTest(first=first), self.assertRaises(ValueError):
                catalog.read_archive(archive_bytes(members), self.entry)

    def test_rejects_multiple_archive_roots(self):
        with self.assertRaises(ValueError):
            catalog.read_archive(archive_bytes(self.members + [
                ('other/LICENSE', b'other', tarfile.REGTYPE)]), self.entry)

    def test_rejects_oversized_selected_payload(self):
        with mock.patch.object(catalog, 'MAX_PAYLOAD', 5), self.assertRaises(ValueError):
            catalog.read_archive(archive_bytes(self.members), self.entry)


class LocalSourceTests(CatalogFixture):
    def test_reads_only_selected_tree_and_retains_licenses(self):
        self.assertEqual(catalog.read_local(self.source, self.entry), self.files)

    def test_rejects_symlink_file_and_directory_within_payload(self):
        outside = self.root / 'outside'
        outside.mkdir()
        (outside / 'secret').write_text('private')
        for is_directory in (False, True):
            link = self.skill_directory / 'linked'
            link.symlink_to(outside if is_directory else outside / 'secret',
                            target_is_directory=is_directory)
            try:
                with self.subTest(directory=is_directory), self.assertRaises(ValueError):
                    catalog.read_local(self.source, self.entry)
            finally:
                link.unlink()

    def test_rejects_parent_directory_symlink_escape(self):
        outside = self.root / 'outside'
        (outside / 'fixture').mkdir(parents=True)
        (outside / 'fixture/SKILL.md').write_bytes(SKILL)
        (self.source / 'redirect').symlink_to(outside, target_is_directory=True)
        entry = dict(self.entry, path='redirect/fixture')
        with self.assertRaises(ValueError):
            catalog.read_local(self.source, entry)

    def test_rejects_symlinked_skill_directory(self):
        (self.source / 'alias').symlink_to(self.skill_directory, target_is_directory=True)
        with self.assertRaises(ValueError):
            catalog.read_local(self.source, dict(self.entry, path='alias'))

    def test_rejects_missing_and_symlinked_license(self):
        license_path = self.source / 'LICENSE'
        license_path.unlink()
        with self.assertRaises(ValueError):
            catalog.read_local(self.source, self.entry)
        other = self.root / 'outside-license'
        other.write_bytes(LICENSE)
        license_path.symlink_to(other)
        with self.assertRaises(ValueError):
            catalog.read_local(self.source, self.entry)


class InstallTests(CatalogFixture):
    def test_registers_for_each_agent_and_preserves_payload_and_provenance(self):
        for agent in ('codex', 'claude'):
            with self.subTest(agent=agent):
                result = self.install(agent)
                self.assertIn('Installed', result)
                target = self.target(agent)
                self.assertEqual(catalog.existing_payload(target), self.files)
                receipt = json.loads((target / catalog.RECEIPT).read_text())
                self.assertEqual(receipt, {
                    'id': self.entry['id'], 'repository': 'example/skills',
                    'commit': 'a' * 40, 'path': 'skills/fixture',
                    'sha256': self.entry['sha256'],
                    'executable_files': [],
                })
        self.assertFalse((self.project / '.codex').exists())
        self.assertFalse((self.project / 'unrelated.txt').exists())

    def test_archive_install_retains_references_and_license(self):
        with mock.patch.object(catalog, 'download', return_value=archive_bytes(self.members)):
            catalog.install(self.data, self.entry, self.project, 'codex')
        self.assertEqual(catalog.existing_payload(self.target()), self.files)

    def test_checksum_mismatch_is_rejected_before_creating_metadata(self):
        (self.skill_directory / 'references/guide.md').write_bytes(b'Unreviewed change')
        with self.assertRaises(ValueError):
            self.install()
        self.assert_project_empty()

    def test_archive_checksum_mismatch_is_rejected_before_writes(self):
        members = list(self.members)
        members[1] = (members[1][0], b'Unreviewed change', tarfile.REGTYPE)
        with mock.patch.object(catalog, 'download', return_value=archive_bytes(members)):
            with self.assertRaises(ValueError):
                catalog.install(self.data, self.entry, self.project, 'codex')
        self.assert_project_empty()

    def test_upstream_name_mismatch_is_rejected_even_with_matching_checksum(self):
        changed = SKILL.replace(b'fixture-skill', b'other-skill')
        (self.skill_directory / 'SKILL.md').write_bytes(changed)
        files = dict(self.files, **{'SKILL.md': changed})
        with self.assertRaises(ValueError):
            self.install(entry=dict(self.entry, sha256=reviewed_hash(files)))
        self.assert_project_empty()

    def test_windows_device_skill_name_is_rejected_before_writes(self):
        changed = SKILL.replace(b'fixture-skill', b'con')
        (self.skill_directory / 'SKILL.md').write_bytes(changed)
        files = dict(self.files, **{'SKILL.md': changed})
        with self.assertRaises(ValueError):
            self.install(entry=dict(self.entry, name='con', sha256=reviewed_hash(files)))
        self.assert_project_empty()

    def test_missing_skill_frontmatter_is_rejected_before_writes(self):
        raw = b'# Missing metadata\n'
        (self.skill_directory / 'SKILL.md').write_bytes(raw)
        files = dict(self.files, **{'SKILL.md': raw})
        with self.assertRaises(ValueError):
            self.install(entry=dict(self.entry, sha256=reviewed_hash(files)))
        self.assert_project_empty()

    def test_preserves_existing_unmanaged_destination(self):
        target = self.target()
        target.mkdir(parents=True)
        sentinel = target / 'notes.txt'
        sentinel.write_text('My own skill')
        with self.assertRaises(ValueError):
            self.install()
        self.assertEqual(sentinel.read_text(), 'My own skill')
        self.assertEqual(list(target.iterdir()), [sentinel])

    def test_duplicate_registered_name_in_other_directory_is_preserved(self):
        other = self.target().parent / 'custom-folder'
        other.mkdir(parents=True)
        (other / 'SKILL.md').write_bytes(SKILL)
        with self.assertRaises(ValueError):
            self.install()
        self.assertEqual((other / 'SKILL.md').read_bytes(), SKILL)
        self.assertFalse(self.target().exists())

    def test_identical_install_is_noop_without_rereading_or_rewriting(self):
        self.install()
        before = {f.relative_to(self.target()).as_posix(): (f.read_bytes(), f.stat().st_mtime_ns)
                  for f in self.target().rglob('*') if f.is_file()}
        with mock.patch.object(catalog, 'read_local', side_effect=AssertionError('Unneeded source read')):
            self.assertIn('Already installed', self.install())
        after = {f.relative_to(self.target()).as_posix(): (f.read_bytes(), f.stat().st_mtime_ns)
                 for f in self.target().rglob('*') if f.is_file()}
        self.assertEqual(before, after)

    def test_modified_installed_file_is_refused_and_preserved(self):
        self.install()
        guide = self.target() / 'references/guide.md'
        guide.write_bytes(b'User customized this')
        with self.assertRaises(ValueError):
            self.install()
        self.assertEqual(guide.read_bytes(), b'User customized this')

    def test_additional_user_file_makes_install_modified(self):
        self.install()
        notes = self.target() / 'personal-notes.md'
        notes.write_text('Keep me')
        with self.assertRaises(ValueError):
            self.install()
        self.assertEqual(notes.read_text(), 'Keep me')

    def test_refuses_symlinked_project_metadata_destination(self):
        outside = self.root / 'other-home'
        outside.mkdir()
        (self.project / '.agents').symlink_to(outside, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.install()
        self.assertEqual(list(outside.iterdir()), [])

    def test_refuses_symlinked_target_without_modifying_its_destination(self):
        outside = self.root / 'other-skill'
        outside.mkdir()
        self.target().parent.mkdir(parents=True)
        self.target().symlink_to(outside, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.install()
        self.assertEqual(list(outside.iterdir()), [])

    def test_refuses_windows_reparse_destinations_without_writes(self):
        original_lstat = Path.lstat
        for relative in ('.agents', '.agents/skills', '.agents/skills/fixture-skill'):
            junction = self.project.resolve() / relative

            def windows_lstat(path, *args, **kwargs):
                if path == junction:
                    return SimpleNamespace(st_mode=stat.S_IFDIR | 0o755,
                                           st_file_attributes=0x400)
                return original_lstat(path, *args, **kwargs)

            with self.subTest(path=relative), mock.patch.object(Path, 'lstat', windows_lstat):
                with self.assertRaises(ValueError):
                    self.install()
            self.assert_project_empty()

    def test_refuses_upstream_receipt_filename_before_writes(self):
        (self.skill_directory / catalog.RECEIPT).write_text('{}')
        files = dict(self.files, **{catalog.RECEIPT: b'{}'})
        with self.assertRaises(ValueError):
            self.install(entry=dict(self.entry, sha256=reviewed_hash(files)))
        self.assert_project_empty()

    def test_refuses_case_alias_of_receipt_before_writes(self):
        alias = catalog.RECEIPT.upper()
        files = dict(self.files, **{alias: b'Upstream content'})
        members = self.members + [
            ('repo/skills/fixture/' + alias, files[alias], tarfile.REGTYPE),
        ]
        entry = dict(self.entry, sha256=reviewed_hash(files))
        with mock.patch.object(catalog, 'download', return_value=archive_bytes(members)):
            with self.assertRaises(ValueError):
                catalog.install(self.data, entry, self.project, 'codex')
        self.assert_project_empty()

    def test_blocks_noninstallable_scopes_and_deliveries(self):
        for scope, delivery in (('global', 'upstream'), ('manual', 'upstream'),
                                ('excluded', 'upstream'), ('project', 'manual'),
                                ('project', 'plugin'), ('project', 'reference')):
            with self.subTest(scope=scope, delivery=delivery), self.assertRaises(ValueError):
                self.install(entry=dict(self.entry, scope=scope, delivery=delivery))
            self.assert_project_empty()

    def test_local_catalog_entry_uses_own_repository_and_reviewed_checksum(self):
        # Local catalog entries read from the registrar's resolved repository root.
        entry = dict(self.entry, delivery='local')
        with mock.patch.object(catalog, 'ROOT', self.source):
            self.install(entry=entry)
        receipt = json.loads((self.target() / catalog.RECEIPT).read_text())
        self.assertEqual(receipt['repository'], 'codemirket/harness')
        self.assertIsNone(receipt['commit'])
        self.assertEqual(catalog.existing_payload(self.target()), self.files)


if __name__ == '__main__':
    unittest.main()
