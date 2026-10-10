"""Behavioral regressions for profile composition and complete-selection preflight."""
import copy
import json
from pathlib import Path
import stat
import tarfile
import tempfile
import unittest
from unittest import mock

from test_catalog import archive_bytes, reviewed_hash, catalog


def selection_entry(identifier, **extra):
    return dict(id=identifier, name=identifier, scope='project', delivery='upstream', **extra)


class ResolveSelectionTests(unittest.TestCase):
    def setUp(self):
        self.entries = [selection_entry('foundation'),
                        selection_entry('review', requires=['foundation']),
                        selection_entry('build', requires=['foundation']),
                        selection_entry('optional')]
        self.data = {'skills': self.entries, 'profiles': {
            'delivery': {'skills': ['build', 'review', 'optional']},
            'review': {'skills': ['review']},
        }}

    def ids(self, **kwargs):
        return [entry['id'] for entry in catalog.resolve_selection(self.data, **kwargs)]

    def test_composes_profiles_and_explicit_ids_in_dependency_order_once(self):
        self.assertEqual(self.ids(profiles=['delivery', 'review'], identifiers=['build']),
                         ['foundation', 'build', 'review', 'optional'])

    def test_can_skip_optional_profile_member(self):
        self.assertEqual(self.ids(profiles=['delivery'], skip=['optional']),
                         ['foundation', 'build', 'review'])

    def test_cannot_skip_required_companion_even_if_explicitly_selected(self):
        with self.assertRaises(ValueError):
            self.ids(profiles=['delivery'], identifiers=['foundation'], skip=['foundation'])

    def test_rejects_self_and_indirect_dependency_cycles(self):
        for requirements in (['foundation'], ['review']):
            self.entries[0]['requires'] = requirements
            with self.subTest(requires=requirements), self.assertRaises(ValueError):
                self.ids(identifiers=['review'])

    def test_rejects_unknown_profile_selection_dependency_and_skip(self):
        for args in ({'profiles': ['missing']}, {'identifiers': ['missing']},
                     {'skip': ['missing']}):
            with self.subTest(args=args), self.assertRaises(ValueError):
                self.ids(**args)
        self.entries[0]['requires'] = ['missing']
        with self.assertRaises(ValueError):
            self.ids(identifiers=['review'])

    def test_conflict_declared_on_either_side_is_enforced(self):
        for owner in ('build', 'review'):
            data = copy.deepcopy(self.data)
            other = 'review' if owner == 'build' else 'build'
            next(e for e in data['skills'] if e['id'] == owner)['conflicts'] = [other]
            with self.subTest(owner=owner), self.assertRaises(ValueError):
                catalog.resolve_selection(data, profiles=['delivery'])

    def test_distinct_ids_with_same_frontmatter_name_are_rejected(self):
        self.entries[2]['name'] = 'review'
        with self.assertRaises(ValueError):
            self.ids(profiles=['delivery'])

    def test_manual_global_and_excluded_entries_cannot_enter_via_dependency(self):
        for scope, delivery in (('manual', 'package'), ('global', 'global-link'),
                                ('excluded', 'upstream')):
            self.entries[0].update(scope=scope, delivery=delivery)
            with self.subTest(scope=scope), self.assertRaises(ValueError):
                self.ids(profiles=['delivery'])


class ProfilePayloadTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.project = self.root / 'project'
        self.project.mkdir()
        self.source = self.root / 'source'
        self.source.mkdir()
        self.license = b'MIT License\nCopyright Fixture Author\n'
        (self.source / 'LICENSE').write_bytes(self.license)
        self.entries = []
        self.files = {}
        self.members = [('repo/LICENSE', self.license, tarfile.REGTYPE)]
        for name in ('alpha', 'beta'):
            raw = ('---\nname: ' + name + '\ndescription: Test skill.\n---\n\n# Original\nUse the guide.\n').encode()
            files = {'SKILL.md': raw, 'references/guide.md': b'# Guide\nStable evidence.\n',
                     '.upstream-licenses/LICENSE': self.license}
            path = 'skills/' + name
            for relative, content in files.items():
                if relative.startswith('.upstream-licenses/'):
                    continue
                destination = self.source / path / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(content)
                self.members.append(('repo/' + path + '/' + relative, content, tarfile.REGTYPE))
            self.files[name] = files
            self.entries.append(dict(id=name, name=name, scope='project', delivery='upstream',
                                     source='fixture', path=path, license_files=['LICENSE'],
                                     sha256=reviewed_hash(files)))
        self.data = {'skills': self.entries, 'sources': {
            'fixture': {'repository': 'example/skills', 'commit': 'a' * 40}}}
        patcher = mock.patch.object(catalog.urllib.request, 'urlopen',
                                    side_effect=AssertionError('Live network forbidden'))
        patcher.start()
        self.addCleanup(patcher.stop)

    def target(self, name='alpha', agent='codex'):
        return self.project / ('.agents' if agent == 'codex' else '.claude') / 'skills' / name

    def install(self, agent='all', entries=None):
        return catalog.install_many(self.data, self.entries if entries is None else entries,
                                    self.project, agent, {'fixture': self.source})

    def assert_empty(self):
        self.assertEqual(list(self.project.iterdir()), [])

    def adapted_entry(self):
        entry = dict(self.entries[0], adaptation='Respect the chosen project scope.')
        # Hand-authored expected bytes, independent of the adapting function.
        expected = dict(self.files['alpha'], **{'SKILL.md': (
            b'---\nname: alpha\ndescription: Test skill.\n---\n\n'
            b'## Personal catalog integration\n\nRespect the chosen project scope.\n'
            b'\n\n# Original\nUse the guide.\n')})
        entry['installed_sha256'] = reviewed_hash(expected)
        return entry, expected

    def test_registers_every_entry_for_both_agents_with_licenses_and_receipts(self):
        results = self.install()
        self.assertEqual(len(results), 4)
        for agent in ('codex', 'claude'):
            for name in ('alpha', 'beta'):
                target = self.target(name, agent)
                self.assertEqual(catalog.existing_payload(target), self.files[name])
                receipt = json.loads((target / catalog.RECEIPT).read_text())
                self.assertEqual(receipt['id'], name)
                self.assertEqual(receipt['commit'], 'a' * 40)

    def test_second_payload_checksum_failure_prevents_all_project_writes(self):
        (self.source / 'skills/beta/references/guide.md').write_bytes(b'Not reviewed')
        with self.assertRaises(ValueError):
            self.install()
        self.assert_empty()

    def test_later_claude_collision_preserves_user_content_and_prevents_codex_writes(self):
        target = self.target('beta', 'claude')
        target.mkdir(parents=True)
        note = target / 'user-notes.md'
        note.write_bytes(b'User-owned')
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('Preflight first')):
            with self.assertRaises(ValueError):
                self.install()
        self.assertEqual(note.read_bytes(), b'User-owned')
        self.assertEqual(list(target.iterdir()), [note])
        self.assertFalse((self.project / '.agents').exists())
        self.assertFalse(self.target('alpha', 'claude').exists())

    def test_both_agents_requires_support_for_each_before_payload_read(self):
        self.entries[1]['agents'] = ['claude']
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('Preflight first')):
            with self.assertRaises(ValueError):
                self.install()
        self.assert_empty()

    def test_download_is_reused_across_entries_and_agents(self):
        with mock.patch.object(catalog, 'download', return_value=archive_bytes(self.members)) as download:
            catalog.install_many(self.data, self.entries, self.project, 'all')
        download.assert_called_once_with(self.data['sources']['fixture'])
        self.assertEqual(catalog.existing_payload(self.target('beta', 'claude')), self.files['beta'])

    def test_unchanged_profile_install_never_reads_sources_or_rewrites_files(self):
        self.install()
        before = {p.relative_to(self.project): (p.read_bytes(), p.stat().st_mtime_ns)
                  for p in self.project.rglob('*') if p.is_file()}
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('No source reread')):
            results = self.install()
        self.assertTrue(all(result.startswith('Already installed') for result in results))
        after = {p.relative_to(self.project): (p.read_bytes(), p.stat().st_mtime_ns)
                 for p in self.project.rglob('*') if p.is_file()}
        self.assertEqual(after, before)

    def test_adaptation_preserves_frontmatter_upstream_body_references_and_license(self):
        entry, expected = self.adapted_entry()
        files, source = catalog.prepare_payload(self.data, entry, self.source)
        self.assertEqual(files, expected)
        self.assertEqual(source, self.data['sources']['fixture'])
        self.assertEqual((self.source / 'skills/alpha/SKILL.md').read_bytes(), self.files['alpha']['SKILL.md'])
        self.assertEqual(catalog.skill_name(files['SKILL.md']), 'alpha')

    def test_adaptation_requires_reviewed_installed_digest_before_writes(self):
        entry, _ = self.adapted_entry()
        for changed in (dict(entry, adaptation='Unreviewed instruction.'),
                        {k: v for k, v in entry.items() if k != 'installed_sha256'}):
            with self.subTest(entry=changed), self.assertRaises(ValueError):
                self.install(entries=[changed])
            self.assert_empty()

    def test_source_hash_still_required_when_installed_hash_is_correct(self):
        entry, _ = self.adapted_entry()
        entry['sha256'] = '0' * 64
        with self.assertRaises(ValueError):
            self.install(entries=[entry])
        self.assert_empty()

    def test_adapted_install_is_idempotent_and_refuses_user_modifications(self):
        entry, expected = self.adapted_entry()
        self.install(entries=[entry])
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('No source reread')):
            self.install(entries=[entry])
        receipt = json.loads((self.target() / catalog.RECEIPT).read_text())
        self.assertEqual(receipt['sha256'], self.entries[0]['sha256'])
        self.assertEqual(receipt['installed_sha256'], entry['installed_sha256'])
        self.assertEqual(catalog.existing_payload(self.target()), expected)
        altered = self.target('alpha', 'claude') / 'SKILL.md'
        altered.write_bytes(altered.read_bytes() + b'\nUser customization\n')
        before = altered.read_bytes()
        with self.assertRaises(ValueError):
            self.install(entries=[entry])
        self.assertEqual(altered.read_bytes(), before)

    @unittest.skipIf(__import__('os').name == 'nt', 'Unix executable permissions')
    def test_declared_helper_retains_executable_permission(self):
        helper = self.source / 'skills/alpha/run.sh'
        helper.write_bytes(b'#!/bin/sh\nexit 0\n')
        files = dict(self.files['alpha'], **{'run.sh': helper.read_bytes()})
        entry = dict(self.entries[0], sha256=reviewed_hash(files), executable_files=['run.sh'])
        self.install(entries=[entry])
        self.assertEqual(stat.S_IMODE((self.target() / 'run.sh').stat().st_mode) & 0o111, 0o111)

    def test_invalid_executable_metadata_is_rejected_before_any_profile_writes(self):
        for index, relative in enumerate(('../outside', 'missing-helper.sh')):
            self.project = self.root / ('invalid-executable-project-' + str(index))
            self.project.mkdir()
            entry = dict(self.entries[1], executable_files=[relative])
            with self.subTest(path=relative):
                with self.assertRaises(ValueError):
                    self.install(entries=[self.entries[0], entry])
                self.assert_empty()

    @unittest.skipIf(__import__('os').name == 'nt', 'Unix executable permissions')
    def test_removed_helper_execute_mode_is_preserved_and_blocks_other_agent_install(self):
        helper = self.source / 'skills/alpha/run.sh'
        helper.write_bytes(b'#!/bin/sh\nexit 0\n')
        files = dict(self.files['alpha'], **{'run.sh': helper.read_bytes()})
        entry = dict(self.entries[0], sha256=reviewed_hash(files), executable_files=['run.sh'])
        self.install(agent='codex', entries=[entry])
        installed = self.target() / 'run.sh'
        installed.chmod(stat.S_IMODE(installed.stat().st_mode) & ~0o111)
        before = (installed.read_bytes(), stat.S_IMODE(installed.stat().st_mode),
                  installed.stat().st_mtime_ns)
        with self.assertRaises(ValueError):
            self.install(entries=[entry])
        self.assertEqual((installed.read_bytes(), stat.S_IMODE(installed.stat().st_mode),
                          installed.stat().st_mtime_ns), before)
        self.assertFalse((self.project / '.claude').exists())

    def test_archive_file_parent_collision_prevents_all_profile_writes(self):
        for index, paths in enumerate((('collision', 'collision/nested.md'),
                                       (catalog.RECEIPT + '/nested.md',),
                                       ('Collision', 'collision/nested.md'))):
            self.project = self.root / ('file-parent-project-' + str(index))
            self.project.mkdir()
            extra = {relative: b'Payload content\n' for relative in paths}
            entry = dict(self.entries[1], sha256=reviewed_hash(dict(self.files['beta'], **extra)))
            members = self.members + [('repo/skills/beta/' + relative, content, tarfile.REGTYPE)
                                      for relative, content in extra.items()]
            with self.subTest(paths=paths), mock.patch.object(
                    catalog, 'download', return_value=archive_bytes(members)):
                with self.assertRaises(ValueError):
                    catalog.install_many(self.data, [self.entries[0], entry], self.project, 'all')
                self.assert_empty()


if __name__ == '__main__':
    unittest.main()
