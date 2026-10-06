"""Isolated behavioral tests for declarative global/project harness reconciliation."""
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

REPOSITORY = Path(__file__).resolve().parents[1]
if str(REPOSITORY) not in sys.path:
    sys.path.insert(0, str(REPOSITORY))
from lib import harness

catalog = harness.catalog


def reviewed_hash(files):
    digest = hashlib.sha256()
    for name, content in sorted(files.items()):
        digest.update(name.encode() + b'\0' + hashlib.sha256(content).digest())
    return digest.hexdigest()


def snapshot(root, timestamps=False):
    """Record empty directories, link targets and bytes without following links."""
    found = {}
    for directory, directories, files in os.walk(root, followlinks=False):
        for name in directories + files:
            item = Path(directory) / name
            relative = item.relative_to(root).as_posix()
            if item.is_symlink():
                value = ('link', os.readlink(item))
            elif item.is_dir():
                value = ('directory',)
            else:
                value = ('file', item.read_bytes(), item.stat().st_mode & 0o777)
            if timestamps:
                value += (item.lstat().st_mtime_ns,)
            found[relative] = value
    return found


class HarnessFixture(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='personal harness tests ')
        self.addCleanup(temporary.cleanup)
        self.base = Path(temporary.name).resolve()
        self.repo = self.base / 'repository with spaces'
        self.home = self.base / 'isolated home'
        self.project = self.base / 'isolated project'
        for path in (self.repo, self.home, self.project):
            path.mkdir()
        (self.repo / 'instructions').mkdir()
        (self.repo / 'instructions/AGENTS.md').write_text('Codex guidance\n')
        (self.repo / 'instructions/CLAUDE.md').write_text('Claude guidance\n')
        self.config = {
            'schema_version': 1,
            'global_skills': ['skill-catalog', 'global-guidance'],
            'targets': {
                'codex': {'instructions': 'instructions/AGENTS.md',
                          'instruction_destination': '.codex/AGENTS.md',
                          'skills_destination': '.agents/skills'},
                'claude': {'instructions': 'instructions/CLAUDE.md',
                           'instruction_destination': '.claude/CLAUDE.md',
                           'skills_destination': '.claude/skills'},
            },
        }
        self.data = {'schema_version': 1, 'sources': {}, 'skills': [],
                     'profiles': {'feature': {'description': 'A realistic dependency closure.',
                                               'skills': ['feature', 'optional']}}}
        for name in ('skill-catalog', 'global-guidance', 'foundation', 'feature', 'optional'):
            directory = self.repo / 'skills' / name
            (directory / 'references').mkdir(parents=True)
            files = {'SKILL.md': ('---\nname: ' + name + '\ndescription: Fixture guidance.\n---\n'
                                  'Read [guide](references/guide.md).\n').encode(),
                     'references/guide.md': b'# Guide\nPreserve the existing project.\n'}
            for relative, raw in files.items():
                (directory / relative).write_bytes(raw)
            self.data['skills'].append({
                'id': name, 'name': name, 'path': 'skills/' + name,
                'scope': 'global' if name in self.config['global_skills'] else 'project',
                'delivery': 'local', 'category': 'engineering',
                'description': 'Fixture guidance.', 'license_files': [],
                'sha256': reviewed_hash(files),
                'requires': ['foundation'] if name == 'feature' else [],
            })
        # Exercise the real copied compatibility scripts against a complete local CLI.
        shutil.copytree(REPOSITORY / 'skills/skill-catalog/scripts',
                        self.repo / 'skills/skill-catalog/scripts',
                        ignore=shutil.ignore_patterns('__pycache__', '*.pyc'))
        (self.repo / 'lib').mkdir()
        (self.repo / 'lib/__init__.py').write_text('')
        for filename in ('catalog.py', 'harness.py'):
            shutil.copyfile(REPOSITORY / 'lib' / filename, self.repo / 'lib' / filename)
        shutil.copyfile(REPOSITORY / 'ai.py', self.repo / 'ai.py')
        self.save_registry()
        self.patch(harness, 'ROOT', self.repo)
        self.patch(harness, 'MANIFEST', self.repo / 'registry/harness.json')
        self.patch(catalog, 'ROOT', self.repo)
        self.patch(catalog, 'load_catalog', lambda: self.data)
        self.patch(catalog.urllib.request, 'urlopen',
                   mock.Mock(side_effect=AssertionError('Tests must not use the network')))

    def patch(self, owner, name, value):
        patcher = mock.patch.object(owner, name, value)
        patcher.start()
        self.addCleanup(patcher.stop)

    def save_registry(self):
        (self.repo / 'registry').mkdir(exist_ok=True)
        (self.repo / 'registry/harness.json').write_text(json.dumps(self.config))
        (self.repo / 'registry/catalog.json').write_text(json.dumps(self.data))

    def entry(self, identifier):
        return next(entry for entry in self.data['skills'] if entry['id'] == identifier)

    def destination(self, name='global-guidance', target='codex', project=False):
        base = self.project if project else self.home
        return base / ('.agents' if target == 'codex' else '.claude') / 'skills' / name

    def initialize(self, target='both'):
        return harness.init_project(self.project, ['feature'], [], ['optional'], target)

    def revise(self, name):
        directory = self.repo / 'skills' / name
        (directory / 'references/guide.md').write_bytes(b'# Revised guide\nReviewed revision.\n')
        files = {p.relative_to(directory).as_posix(): p.read_bytes()
                 for p in directory.rglob('*') if p.is_file()}
        self.entry(name)['sha256'] = reviewed_hash(files)

    def symlink(self, link, target, directory=False):
        try:
            link.symlink_to(target, target_is_directory=directory)
        except (OSError, NotImplementedError) as error:
            self.skipTest('Symlink creation unavailable: ' + str(error))


class GlobalHarnessTests(HarnessFixture):
    def test_plan_and_doctor_are_read_only_for_both_targets(self):
        for mode in ('copy', 'link'):
            with self.subTest(mode=mode):
                before = snapshot(self.home, timestamps=True)
                _, _, _, jobs = harness.global_plan('both', self.home, mode)
                self.assertEqual(len(jobs), 6)
                self.assertEqual({j['target'] for j in jobs}, {'codex', 'claude'})
                self.assertTrue(all(j['action'] == 'create' for j in jobs))
                with mock.patch('sys.stdout', new_callable=io.StringIO):
                    result = harness.main(['doctor', '--home', str(self.home), '--mode', mode])
                self.assertEqual(result, 1)
                self.assertEqual(snapshot(self.home, timestamps=True), before)

    def test_copy_install_has_expected_app_payloads_and_is_idempotent(self):
        unrelated = self.destination('user-skill', 'claude')
        unrelated.mkdir(parents=True)
        (unrelated / 'notes.md').write_bytes(b'Personal unrelated content')
        harness.sync_global('both', self.home, 'copy')
        self.assertEqual((self.home / '.codex/AGENTS.md').read_text(), 'Codex guidance\n')
        self.assertEqual((self.home / '.claude/CLAUDE.md').read_text(), 'Claude guidance\n')
        for target in ('codex', 'claude'):
            self.assertFalse(self.destination(target=target).is_symlink())
            self.assertEqual((self.destination(target=target) / 'references/guide.md').read_bytes(),
                             b'# Guide\nPreserve the existing project.\n')
        before = {target: snapshot(self.destination(target=target), timestamps=True)
                  for target in ('codex', 'claude')}
        jobs = harness.sync_global('both', self.home, 'copy')
        self.assertTrue(all(j['action'] == 'unchanged' for j in jobs))
        for target in before:
            self.assertEqual(snapshot(self.destination(target=target), timestamps=True), before[target])
        self.assertEqual((unrelated / 'notes.md').read_bytes(), b'Personal unrelated content')
        self.assertEqual(len(json.loads((self.home / '.agent-harness/state.json').read_text())['items']), 6)
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['doctor', '--home', str(self.home), '--mode', 'copy']), 0)

    def test_link_install_is_idempotent_and_each_app_uses_its_instruction_source(self):
        probe = self.base / 'probe'
        self.symlink(probe, self.repo, True)
        probe.unlink()
        jobs = harness.sync_global('both', self.home, 'link')
        for job in jobs:
            self.assertTrue(Path(job['destination']).is_symlink())
            self.assertEqual(os.readlink(job['destination']), job['source'])
        links_before = {job['destination']: Path(job['destination']).lstat().st_mtime_ns for job in jobs}
        jobs = harness.sync_global('both', self.home, 'link')
        self.assertTrue(all(job['action'] == 'unchanged' for job in jobs))
        self.assertEqual({job['destination']: Path(job['destination']).lstat().st_mtime_ns for job in jobs},
                         links_before)

    def test_copy_catalog_wrapper_resolves_checkout_from_an_unrelated_cwd(self):
        harness.sync_global('both', self.home, 'copy')
        environment = dict(os.environ, HOME=str(self.home), PYTHONDONTWRITEBYTECODE='1')
        for target in ('codex', 'claude'):
            directory = self.destination('skill-catalog', target)
            self.assertEqual(json.loads((directory / '.harness-source.json').read_text()),
                             {'repository': str(self.repo)})
            for script, args in (('catalog.py', ['show', 'feature']),
                                 ('harness.py', ['plan', '--home', str(self.home), '--mode', 'copy'])):
                with self.subTest(target=target, script=script):
                    completed = subprocess.run([sys.executable, str(directory / 'scripts' / script)] + args,
                                               cwd=self.base, env=environment, text=True, capture_output=True)
                    self.assertEqual(completed.returncode, 0, completed.stderr)
                    result = json.loads(completed.stdout)
                    if script == 'catalog.py':
                        self.assertEqual(result['id'], 'feature')
                    else:
                        self.assertTrue(all(row['action'] == 'unchanged' for row in result))

    def test_managed_copy_update_keeps_unrelated_directories_and_other_target_receipts(self):
        harness.sync_global('both', self.home, 'copy')
        other = self.destination('unrelated')
        other.mkdir()
        (other / 'file').write_bytes(b'Keep')
        self.revise('global-guidance')
        jobs = harness.sync_global('codex', self.home, 'copy')
        self.assertEqual(next(j['action'] for j in jobs if j['id'] == 'global-guidance'), 'update')
        self.assertEqual((self.destination() / 'references/guide.md').read_bytes(),
                         b'# Revised guide\nReviewed revision.\n')
        self.assertEqual((self.destination(target='claude') / 'references/guide.md').read_bytes(),
                         b'# Guide\nPreserve the existing project.\n')
        self.assertEqual((other / 'file').read_bytes(), b'Keep')
        receipt = json.loads((self.home / '.agent-harness/state.json').read_text())
        self.assertEqual(len(receipt['items']), 6)

    def test_user_edited_copy_blocks_all_updates_and_preserves_receipt(self):
        harness.sync_global('both', self.home, 'copy')
        edited = self.destination(target='claude') / 'SKILL.md'
        edited.write_bytes(edited.read_bytes() + b'\nPersonal edits.\n')
        self.revise('global-guidance')
        before = snapshot(self.home, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_global('both', self.home, 'copy')
        self.assertEqual(snapshot(self.home, timestamps=True), before)

    def test_known_legacy_instruction_links_migrate_without_following_them(self):
        for target, destination in (('codex', '.codex/AGENTS.md'), ('claude', '.claude/CLAUDE.md')):
            link = self.home / destination
            link.parent.mkdir(parents=True, exist_ok=True)
            self.symlink(link, self.repo / 'components/AGENTS.md')  # Deliberately dangling legacy link.
        harness.sync_global('both', self.home, 'copy')
        self.assertEqual((self.home / '.codex/AGENTS.md').read_text(), 'Codex guidance\n')
        self.assertEqual((self.home / '.claude/CLAUDE.md').read_text(), 'Claude guidance\n')
        self.assertFalse((self.repo / 'components').exists())

    def test_unknown_link_is_not_claimed_as_legacy(self):
        foreign = self.base / 'foreign-instructions'
        foreign.write_text('Personal guidance')
        link = self.home / '.claude/CLAUDE.md'
        link.parent.mkdir()
        self.symlink(link, foreign)
        before = snapshot(self.home, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_global('both', self.home, 'copy')
        self.assertEqual(snapshot(self.home, timestamps=True), before)
        self.assertEqual(foreign.read_text(), 'Personal guidance')

    def test_later_target_collision_prevents_any_global_writes(self):
        destination = self.destination(target='claude')
        destination.mkdir(parents=True)
        (destination / 'SKILL.md').write_text('User-owned file')
        before = snapshot(self.home, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_global('both', self.home, 'copy')
        self.assertEqual(snapshot(self.home, timestamps=True), before)
        self.assertFalse((self.home / '.agents').exists())

    def test_missing_later_instruction_source_prevents_any_global_writes(self):
        (self.repo / 'instructions/CLAUDE.md').unlink()
        with self.assertRaises(ValueError):
            harness.sync_global('both', self.home, 'copy')
        self.assertEqual(snapshot(self.home), {})

    def test_completed_global_items_keep_receipts_after_later_os_failure_and_retry(self):
        real_replace_item = harness.replace_item
        calls = []

        def replace_item(destination, create, expected):
            calls.append(destination)
            if len(calls) == 2:
                raise OSError('Simulated later destination failure')
            return real_replace_item(destination, create, expected)

        with mock.patch.object(harness, 'replace_item', side_effect=replace_item):
            with self.assertRaises(OSError):
                harness.sync_global('both', self.home, 'copy')
        receipt = json.loads((self.home / '.agent-harness/state.json').read_text())
        self.assertEqual(set(receipt['items']), {'codex:instructions'})
        self.assertEqual((self.home / '.codex/AGENTS.md').read_text(), 'Codex guidance\n')
        jobs = harness.sync_global('both', self.home, 'copy')
        self.assertEqual(jobs[0]['action'], 'unchanged')
        self.assertTrue(all(job['action'] == 'create' for job in jobs[1:]))
        self.assertEqual(len(json.loads((self.home / '.agent-harness/state.json').read_text())['items']), 6)

    def test_symlinked_destination_parent_cannot_escape_the_isolated_home(self):
        outside = self.base / 'outside'
        outside.mkdir()
        self.symlink(self.home / '.agents', outside, True)
        before = snapshot(self.home, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_global('both', self.home, 'copy')
        self.assertEqual(snapshot(self.home, timestamps=True), before)
        self.assertEqual(snapshot(outside), {})


class ProjectHarnessTests(HarnessFixture):
    def add_helper(self, name='feature', relative='scripts/helper.py'):
        directory = self.repo / 'skills' / name
        helper = directory / relative
        helper.parent.mkdir(parents=True, exist_ok=True)
        helper.write_bytes(b'#!/usr/bin/env python3\nprint("reviewed helper")\n')
        self.entry(name)['executable_files'] = [relative]
        self.entry(name)['sha256'] = reviewed_hash({
            p.relative_to(directory).as_posix(): p.read_bytes()
            for p in directory.rglob('*') if p.is_file()})

    def test_managed_update_can_add_a_new_executable(self):
        self.initialize()
        harness.sync_project(self.project)
        self.add_helper()
        harness.sync_project(self.project)
        for target in ('codex', 'claude'):
            dest = self.destination('feature', target, True)
            receipt = json.loads((dest / catalog.RECEIPT).read_text())
            self.assertEqual(receipt['executable_files'], ['scripts/helper.py'])
            helper = dest / 'scripts/helper.py'
            self.assertEqual(helper.read_bytes(), b'#!/usr/bin/env python3\nprint("reviewed helper")\n')
            if os.name != 'nt':
                self.assertEqual(helper.stat().st_mode & 0o111, 0o111)

    def test_legacy_receipt_allows_new_executable_without_requiring_it_in_old_copy(self):
        self.initialize('codex')
        harness.sync_project(self.project)
        receipt = self.destination('feature', 'codex', True) / catalog.RECEIPT
        previous = json.loads(receipt.read_text())
        previous.pop('executable_files')
        receipt.write_text(json.dumps(previous))
        self.add_helper()
        harness.sync_project(self.project)
        self.assertTrue((receipt.parent / 'scripts/helper.py').is_file())

    @unittest.skipIf(os.name == 'nt', 'POSIX executable modes')
    def test_removed_upstream_helper_does_not_hide_user_mode_change(self):
        self.add_helper()
        self.initialize()
        harness.sync_project(self.project)
        helper = self.destination('feature', 'claude', True) / 'scripts/helper.py'
        helper.chmod(helper.stat().st_mode & ~0o111)
        (self.repo / 'skills/feature/scripts/helper.py').unlink()
        self.entry('feature')['executable_files'] = []
        self.revise('feature')
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_update_fallback_never_overrides_invalid_selection_metadata(self):
        self.initialize('codex')
        harness.sync_project(self.project)
        before = snapshot(self.project, timestamps=True)
        for changes, target in (({'scope': 'manual'}, 'codex'),
                                ({'delivery': 'reference'}, 'codex'),
                                ({'name': 'Invalid-name'}, 'codex'),
                                ({'path': '../escape'}, 'codex'),
                                ({'extra_files': {'extra.md': catalog.RECEIPT}}, 'codex'),
                                ({'agents': ['claude']}, 'codex'),
                                ({}, 'other')):
            with self.subTest(changes=changes, target=target):
                entry = dict(self.entry('feature'), **changes)
                with self.assertRaises(ValueError):
                    harness.project_state(entry, self.project, target)
                self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_regular_parent_collision_is_rejected_before_any_project_writes(self):
        self.initialize()
        (self.project / '.claude').write_text('User-owned placeholder')
        before = snapshot(self.project, timestamps=True)
        with self.assertRaisesRegex(ValueError, 'Expected real destination directory'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_changed_parent_after_preparation_cannot_redirect_project_writes(self):
        self.initialize('codex')
        outside = self.base / 'outside project'
        outside.mkdir()
        prepare = catalog.prepare_payload
        def redirected(*args, **kwargs):
            prepared = prepare(*args, **kwargs)
            if not (self.project / '.agents').exists():
                self.symlink(self.project / '.agents', outside, True)
            return prepared
        with mock.patch.object(catalog, 'prepare_payload', side_effect=redirected):
            with self.assertRaisesRegex(ValueError, 'Expected real destination directory'):
                harness.sync_project(self.project)
        self.assertEqual(snapshot(outside), {})
        self.assertFalse((self.project / '.ai/project.lock.json').exists())

    def test_doctor_requires_current_lock_and_preserves_missing_or_stale_state(self):
        self.initialize('codex')
        harness.sync_project(self.project)
        path = self.project / '.ai/project.lock.json'
        expected = json.loads(path.read_text())
        stale = json.loads(path.read_text())
        stale['manifest']['skip'] = []
        stale_hash = json.loads(path.read_text())
        stale_hash['skills'][0]['sha256'] = '0' * 64
        for content in (None, '{broken JSON', json.dumps(stale), json.dumps(stale_hash)):
            with self.subTest(content=content):
                if path.exists():
                    path.unlink()
                if content is not None:
                    path.write_text(content)
                before = snapshot(self.project, timestamps=True)
                with mock.patch('sys.stdout', new_callable=io.StringIO), \
                     mock.patch('sys.stderr', new_callable=io.StringIO):
                    status = harness.main(['project', 'doctor', '--project', str(self.project)])
                self.assertEqual(status, 1)
                self.assertEqual(snapshot(self.project, timestamps=True), before)
        path.write_text(json.dumps(expected))
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['project', 'doctor', '--project', str(self.project)]), 0)

    def test_init_resolves_profile_and_skip_and_plan_is_read_only(self):
        result = self.initialize()
        self.assertEqual(result['targets'], ['codex', 'claude'])
        path = self.project / '.ai/project.json'
        self.assertEqual(json.loads(path.read_text()), result)
        before = snapshot(self.project, timestamps=True)
        self.assertEqual(self.initialize(), result)
        _, _, entries, jobs = harness.project_plan(self.project)
        self.assertEqual([e['id'] for e in entries], ['foundation', 'feature'])
        self.assertEqual([(j['entry']['id'], j['target']) for j in jobs],
                         [('foundation', 'codex'), ('foundation', 'claude'),
                          ('feature', 'codex'), ('feature', 'claude')])
        self.assertEqual(snapshot(self.project, timestamps=True), before)
        self.assertFalse((self.project / '.ai/project.lock.json').exists())

    def test_init_rejects_invalid_or_conflicting_manifest_without_overwriting(self):
        with self.assertRaises(ValueError):
            harness.init_project(self.project, ['missing'], [], [], 'both')
        self.assertEqual(snapshot(self.project), {})
        self.initialize()
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.init_project(self.project, [], ['foundation'], [], 'codex')
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_sync_installs_dependency_closure_both_targets_and_records_exact_lock(self):
        config = self.initialize()
        jobs = harness.sync_project(self.project)
        self.assertEqual(len(jobs), 4)
        lock = json.loads((self.project / '.ai/project.lock.json').read_text())
        self.assertEqual(lock['manifest'], config)
        self.assertEqual([row['id'] for row in lock['skills']], ['foundation', 'feature'])
        for row in lock['skills']:
            self.assertEqual(row['sha256'], self.entry(row['id'])['sha256'])
            self.assertEqual(row['installed_sha256'], row['sha256'])
            self.assertEqual(row['source'], 'personal')
            self.assertIsNone(row['commit'])
        for target in ('codex', 'claude'):
            for name in ('foundation', 'feature'):
                directory = self.destination(name, target, True)
                self.assertEqual((directory / 'SKILL.md').read_bytes(),
                                 (self.repo / 'skills' / name / 'SKILL.md').read_bytes())
                self.assertEqual(json.loads((directory / catalog.RECEIPT).read_text())['id'], name)
            self.assertFalse(self.destination('optional', target, True).exists())

    def test_upstream_source_provenance_survives_project_sync_and_reviewed_update(self):
        entry = self.entry('feature')
        entry.update(delivery='upstream', source='fixture-upstream')
        self.data['sources']['fixture-upstream'] = {
            'repository': 'fixture/skills', 'commit': 'a' * 40}
        self.initialize()
        harness.sync_project(self.project, {'fixture-upstream': self.repo})
        lock = json.loads((self.project / '.ai/project.lock.json').read_text())
        self.assertEqual(lock['skills'][1]['commit'], 'a' * 40)
        self.assertEqual(lock['skills'][1]['source'], 'fixture-upstream')
        self.revise('feature')
        self.data['sources']['fixture-upstream']['commit'] = 'b' * 40
        harness.sync_project(self.project, {'fixture-upstream': self.repo})
        lock = json.loads((self.project / '.ai/project.lock.json').read_text())
        self.assertEqual(lock['skills'][1]['commit'], 'b' * 40)
        for target in ('codex', 'claude'):
            receipt = json.loads((self.destination('feature', target, True) / catalog.RECEIPT).read_text())
            self.assertEqual(receipt['repository'], 'fixture/skills')
            self.assertEqual(receipt['commit'], 'b' * 40)
            self.assertEqual(receipt['sha256'], self.entry('feature')['sha256'])

    def test_unchanged_sync_does_not_reprepare_or_rewrite_installed_payloads(self):
        self.initialize()
        harness.sync_project(self.project)
        before = {target: snapshot(self.project / target, timestamps=True)
                  for target in ('.agents', '.claude')}
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('Unneeded source read')):
            jobs = harness.sync_project(self.project)
        self.assertTrue(all(job['action'] == 'unchanged' for job in jobs))
        for target in before:
            self.assertEqual(snapshot(self.project / target, timestamps=True), before[target])
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['project', 'doctor', '--project', str(self.project)]), 0)

    def test_managed_update_changes_payloads_receipts_and_lock_for_both_targets(self):
        self.initialize()
        harness.sync_project(self.project)
        old_hash = self.entry('feature')['sha256']
        self.revise('feature')
        jobs = harness.sync_project(self.project)
        self.assertEqual([j['action'] for j in jobs], ['unchanged', 'unchanged', 'update', 'update'])
        for target in ('codex', 'claude'):
            directory = self.destination('feature', target, True)
            self.assertEqual((directory / 'references/guide.md').read_bytes(),
                             b'# Revised guide\nReviewed revision.\n')
            receipt = json.loads((directory / catalog.RECEIPT).read_text())
            self.assertNotEqual(receipt['sha256'], old_hash)
            self.assertEqual(receipt['sha256'], self.entry('feature')['sha256'])
        lock = json.loads((self.project / '.ai/project.lock.json').read_text())
        self.assertEqual(lock['skills'][1]['sha256'], self.entry('feature')['sha256'])

    def test_user_edit_in_later_target_blocks_updates_and_preserves_old_lock(self):
        self.initialize()
        harness.sync_project(self.project)
        edited = self.destination('feature', 'claude', True) / 'references/guide.md'
        edited.write_bytes(b'Personal customization')
        self.revise('foundation')
        self.revise('feature')
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_user_added_payload_file_also_prevents_managed_update(self):
        self.initialize()
        harness.sync_project(self.project)
        note = self.destination('feature', 'claude', True) / 'my-note.md'
        note.write_bytes(b'Personal addition')
        self.revise('feature')
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_later_destination_collision_prevents_earlier_project_installs(self):
        self.initialize()
        existing = self.destination('feature', 'claude', True)
        existing.mkdir(parents=True)
        (existing / 'notes.md').write_bytes(b'Independent skill')
        before = snapshot(self.project, timestamps=True)
        with mock.patch.object(catalog, 'prepare_payload', side_effect=AssertionError('Preflight first')):
            with self.assertRaises(ValueError):
                harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_missing_second_source_prevents_all_project_payload_and_lock_writes(self):
        self.initialize()
        shutil.rmtree(self.repo / 'skills/feature')
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_unreviewed_second_source_bytes_prevent_all_writes(self):
        self.initialize()
        (self.repo / 'skills/feature/references/guide.md').write_bytes(b'Unreviewed source edit')
        before = snapshot(self.project, timestamps=True)
        with self.assertRaises(ValueError):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project, timestamps=True), before)

    def test_unrelated_project_skills_are_preserved_during_sync(self):
        self.initialize()
        unrelated = self.destination('custom', 'codex', True)
        unrelated.mkdir(parents=True)
        (unrelated / 'SKILL.md').write_text('---\nname: custom\ndescription: User skill.\n---\n')
        before = snapshot(unrelated, timestamps=True)
        harness.sync_project(self.project)
        self.assertEqual(snapshot(unrelated, timestamps=True), before)


class ReplacementRecoveryTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.dest = self.root / 'owned.txt'
        self.dest.write_bytes(b'Original owned bytes')
        self.before = harness.fingerprint(self.dest)
        self.real_replace = os.replace

    def create(self, stage):
        stage.write_bytes(b'Reviewed new bytes')

    def test_stage_failure_preserves_original_and_cleans_partial_stage(self):
        def fail(stage):
            stage.write_bytes(b'Incomplete stage')
            raise OSError('Simulated disk failure')
        with self.assertRaises(OSError):
            harness.replace_item(self.dest, fail, self.before)
        self.assertEqual(self.dest.read_bytes(), b'Original owned bytes')
        self.assertEqual(list(self.root.iterdir()), [self.dest])

    def test_final_rename_failure_restores_original(self):
        def replace(source, destination):
            if '.new-' in Path(source).name:
                raise OSError('Simulated rename failure')
            return self.real_replace(source, destination)
        with mock.patch.object(harness.os, 'replace', side_effect=replace):
            with self.assertRaises(OSError):
                harness.replace_item(self.dest, self.create, self.before)
        self.assertEqual(self.dest.read_bytes(), b'Original owned bytes')
        self.assertEqual(list(self.root.iterdir()), [self.dest])

    def test_failed_restore_retains_recoverable_backup_bytes(self):
        def replace(source, destination):
            if '.new-' in Path(source).name or '.previous-' in Path(source).name:
                raise OSError('Simulated publication and restore failure')
            return self.real_replace(source, destination)
        with mock.patch.object(harness.os, 'replace', side_effect=replace):
            with self.assertRaises(OSError):
                harness.replace_item(self.dest, self.create, self.before)
        backups = list(self.root.glob('.owned.txt.previous-*'))
        self.assertEqual(len(backups), 1)
        self.assertEqual(backups[0].read_bytes(), b'Original owned bytes')
        self.assertFalse(self.dest.exists())
        self.assertEqual(list(self.root.glob('*.new-*')), [])

    def test_concurrent_edit_during_staging_is_preserved(self):
        def create(stage):
            self.create(stage)
            self.dest.write_bytes(b'Concurrent user edit')
        with self.assertRaises(ValueError):
            harness.replace_item(self.dest, create, self.before)
        self.assertEqual(self.dest.read_bytes(), b'Concurrent user edit')
        self.assertEqual(list(self.root.iterdir()), [self.dest])


if __name__ == '__main__':
    unittest.main()
