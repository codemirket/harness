"""Portable marketplace exports use isolated sources and never activate plugins."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from lib import bundle, catalog

SKILL = b'---\nname: test-skill\ndescription: Test fixture.\n---\nRead references/guide.md.\n'


class BundleTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name).resolve()
        self.root = self.base / 'harness'
        self.root.mkdir()
        for name in ('registry', 'instructions', 'skills', 'docs', 'setup', 'lib'):
            (self.root / name).mkdir()
        self.source = self.base / 'upstream'
        (self.source / 'skills/test/references').mkdir(parents=True)
        (self.source / 'skills/test/SKILL.md').write_bytes(SKILL)
        (self.source / 'skills/test/references/guide.md').write_bytes(b'Reviewed guide')
        (self.source / 'skills/test/run.sh').write_bytes(b'#!/bin/sh\nexit 0\n')
        (self.source / 'LICENSE').write_bytes(b'MIT fixture notice')
        self.files = {'SKILL.md': SKILL, 'references/guide.md': b'Reviewed guide',
                      'run.sh': b'#!/bin/sh\nexit 0\n', '.upstream-licenses/LICENSE': b'MIT fixture notice'}
        self.entry = {'id': 'test', 'name': 'test-skill', 'scope': 'project', 'delivery': 'upstream',
                      'source': 'fixture', 'path': 'skills/test', 'license_files': ['LICENSE'],
                      'sha256': catalog.payload_hash(self.files), 'executable_files': ['run.sh']}
        self.data = {'schema_version': 1, 'sources': {'fixture': {'repository': 'test/fixture', 'commit': 'a' * 40}},
                     'skills': [self.entry], 'profiles': {'test-profile': {'skills': ['test']}}}
        self.config = {'schema_version': 1, 'name': 'test-marketplace', 'version': '1.0.0',
                       'global_skills': [], 'bundles': {
                       'one': {'description': 'Test bundle', 'profiles': ['test-profile']},
                       'two': {'description': 'Second bundle', 'profiles': ['test-profile']}}}
        self.output = self.base / 'export'
        patcher = mock.patch.object(catalog.urllib.request, 'urlopen', side_effect=AssertionError('No live network'))
        patcher.start()
        self.addCleanup(patcher.stop)

    def export(self, names=('one',), output=None):
        return bundle.export_bundles(names, output or self.output, root=self.root,
                                     data=self.data, config=self.config, source_trees={'fixture': self.source})

    def read(self, path):
        return json.loads((self.output / path).read_text())

    def test_full_payload_manifests_provenance_lock_and_executable_modes(self):
        lock = self.export()
        plugin = self.output / 'plugins/one'
        self.assertEqual(catalog.existing_payload(plugin / 'skills/test-skill'), self.files)
        self.assertEqual(self.read('plugins/one/plugin.json')['$schema'],
                         'https://agent-plugins.org/schemas/1.0.0/plugin.schema.json')
        self.assertNotIn('skills', self.read('plugins/one/plugin.json'))
        self.assertEqual(self.read('plugins/one/.codex-plugin/plugin.json')['skills'], './skills/')
        self.assertFalse((plugin / '.claude-plugin').exists())
        codex = self.read('.agents/plugins/marketplace.json')['plugins'][0]
        self.assertEqual(codex['source'], {'source': 'local', 'path': './plugins/one'})
        self.assertEqual(codex['policy']['installation'], 'AVAILABLE')
        self.assertFalse((self.output / '.claude-plugin').exists())
        self.assertEqual(lock['clients'], ['codex-desktop', 'codex-cli'])
        self.assertEqual(self.read('plugins/one/provenance.json')['skills'][0]['source_sha256'], self.entry['sha256'])
        self.assertNotIn(bundle.LOCK, lock['files'])
        for path, record in lock['files'].items():
            self.assertEqual(hashlib.sha256((self.output / path).read_bytes()).hexdigest(), record['sha256'])
            if os.name != 'nt':
                self.assertEqual((self.output / path).stat().st_mode & 0o777, int(record['mode'], 8))
        self.assertEqual(lock['files']['plugins/one/skills/test-skill/run.sh']['mode'], '0755')

    def test_order_duplicates_and_different_output_paths_produce_same_bytes(self):
        self.export(('two', 'one', 'one'))
        other = self.base / 'second'
        self.export(('one', 'two'), other)
        self.assertEqual(catalog.existing_payload(self.output), catalog.existing_payload(other))

    def test_shared_source_preparation_happens_once_across_bundles(self):
        with mock.patch.object(catalog, 'prepare_payload', wraps=catalog.prepare_payload) as prepare:
            self.export(('one', 'two'))
            self.assertEqual(prepare.call_count, 1)

    def test_no_output_or_staging_on_late_source_integrity_failure(self):
        bad = dict(self.entry, id='bad', name='other-skill', path='skills/other')
        self.data['skills'].append(bad)
        self.data['profiles']['test-profile']['skills'].append('bad')
        with self.assertRaises(ValueError):
            self.export()
        self.assertFalse(self.output.exists())
        self.assertFalse(list(self.base.glob('.ai-export-*')))

    def test_existing_output_files_directories_links_and_link_ancestors_are_preserved(self):
        for kind in ('file', 'directory', 'link'):
            with self.subTest(kind=kind):
                if kind == 'file':
                    self.output.write_bytes(b'user data')
                elif kind == 'directory':
                    self.output.mkdir()
                else:
                    self.output.symlink_to(self.source, target_is_directory=True)
                with self.assertRaises(ValueError):
                    self.export()
                if kind == 'directory': self.output.rmdir()
                else: self.output.unlink()
        parent = self.base / 'alias'
        parent.symlink_to(self.source, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.export(output=parent / 'new')
        self.assertFalse((self.source / 'new').exists())

    def test_publish_does_not_replace_even_an_empty_directory_created_concurrently(self):
        staging = self.base / 'stage'
        staging.mkdir()
        (staging / 'content').write_text('new')
        self.output.mkdir()
        with self.assertRaises(OSError):
            bundle.publish(staging, self.output)
        self.assertEqual(list(self.output.iterdir()), [])
        self.assertTrue((staging / 'content').exists())

    def test_failed_publish_cleans_staging(self):
        with mock.patch.object(bundle, 'publish', side_effect=OSError('simulated failure')):
            with self.assertRaises(OSError):
                self.export()
        self.assertFalse(self.output.exists())
        self.assertFalse(list(self.base.glob('.ai-export-*')))

    def test_blocks_manual_unsupported_host_conflicts_bad_names_and_traversal(self):
        for change in ({'scope': 'manual'}, {'agents': ['claude']}, {'name': 'CON'},
                       {'conflicts': ['test']}, {'name': '../escape'}):
            before = dict(self.entry)
            self.entry.update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.export()
            self.entry.clear(); self.entry.update(before)
        with self.assertRaises(ValueError): self.export(('../escape',))
        with self.assertRaises(ValueError): self.export(output=self.base / 'nested/../escape')
        self.assertFalse(self.output.exists())

    def test_export_size_limit_checked_before_output_writes(self):
        with mock.patch.object(bundle, 'MAX_EXPORT', 1):
            with self.assertRaises(ValueError):
                self.export()
        self.assertFalse(self.output.exists())
        self.assertFalse(list(self.base.glob('.ai-export-*')))

    def foundation(self):
        for filename in ('ai.py', 'lib/catalog.py', 'lib/harness.py', 'lib/bundle.py',
                         'skills/skill-catalog/scripts/catalog.py', 'skills/skill-catalog/scripts/harness.py',
                         'skills/skill-catalog/SKILL.md', 'scripts/render_registry.py'):
            target = self.root / filename
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / filename, target)
        global_ = {'id': 'skill-catalog', 'name': 'skill-catalog', 'scope': 'global',
                   'delivery': 'global-link', 'path': 'skills/skill-catalog'}
        self.data['skills'].append(global_)
        self.config['global_skills'] = ['skill-catalog']
        self.config['bundles']['foundation'] = {'description': 'Self contained engine', 'global_skills': True, 'profiles': []}
        (self.root / 'registry/catalog.json').write_text(json.dumps(self.data))
        (self.root / 'registry/harness.json').write_text(json.dumps(self.config))
        (self.root / 'registry/source-index.json').write_text('{"sources": [], "skills": []}')
        (self.root / 'instructions/AGENTS.md').write_text('Fixture instructions')
        (self.root / 'docs/guide.md').write_text('Fixture documentation')
        (self.root / 'setup/fixture.txt').write_text('No execution')

    def test_foundation_cli_runs_from_unrelated_cwd_after_original_checkout_removed(self):
        self.foundation()
        (self.root / 'lib/__pycache__').mkdir()
        (self.root / 'lib/__pycache__/junk.pyc').write_bytes(b'ignored')
        (self.root / 'docs/build').mkdir()
        (self.root / 'docs/build/ignored.txt').write_bytes(b'ignored')
        (self.root / '.git').mkdir()
        (self.root / '.git/private').write_text('not copied')
        self.export(('foundation',))
        plugin = self.output / 'plugins/foundation'
        locator = json.loads((plugin / 'skills/skill-catalog/.harness-source.json').read_text())
        self.assertEqual(locator['repository'], '../../_harness')
        self.assertFalse(list(plugin.rglob('*.pyc')))
        self.assertFalse((plugin / '_harness/docs/build').exists())
        self.assertFalse((plugin / '_harness/.git').exists())
        self.assertTrue((plugin / '_harness/scripts/render_registry.py').is_file())
        shutil.rmtree(self.root)
        unrelated = self.base / 'unrelated'
        unrelated.mkdir()
        result = subprocess.run([sys.executable, '-B', str(plugin / 'skills/skill-catalog/scripts/catalog.py'),
                                 'list', '--json'], cwd=unrelated, text=True, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual({e['id'] for e in json.loads(result.stdout)}, {'test', 'skill-catalog'})

    def test_snapshot_symlink_fails_without_export(self):
        self.foundation()
        (self.root / 'docs/escape').symlink_to(self.source, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.export(('foundation',))
        self.assertFalse(self.output.exists())

    def test_harness_declared_globals_include_project_selectable_local_originals(self):
        self.foundation()
        skill = self.root / 'skills/authored'
        skill.mkdir()
        content = SKILL.replace(b'test-skill', b'authored-skill')
        (skill / 'SKILL.md').write_bytes(content)
        entry = dict(id='authored', name='authored-skill', scope='project', delivery='local',
                     path='skills/authored', license_files=[], sha256=catalog.payload_hash({'SKILL.md': content}))
        self.data['skills'].append(entry)
        self.config['global_skills'].append('authored')
        self.export(('foundation',))
        self.assertEqual((self.output / 'plugins/foundation/skills/authored-skill/SKILL.md').read_bytes(), content)
