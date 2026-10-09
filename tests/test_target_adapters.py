"""Target adapters preserve ownership and exact portable skill selections."""
import io
import json
from pathlib import Path
from unittest import mock

from test_harness import HarnessFixture, catalog, harness, snapshot


class TargetAdapterTests(HarnessFixture):
    def declare(self, **changes):
        value = {'schema_version': 2, 'targets': ['codex', 'claude'],
                 'profiles': [], 'skills': ['foundation'], 'skip': [], 'target_skills': {}}
        value.update(changes)
        harness.write_json(self.project / '.ai/project.json', value)
        return value

    def test_all_global_targets_publish_separate_skills_and_record_each_owner(self):
        before = snapshot(self.home)
        rows = harness.public_jobs(harness.global_plan('all', self.home, 'copy')[3])
        self.assertEqual(snapshot(self.home), before)
        self.assertEqual(len(rows), 6)
        with mock.patch.object(harness, 'replace_item', wraps=harness.replace_item) as replace:
            harness.sync_global('all', self.home, 'copy')
        destinations = [call.args[0] for call in replace.call_args_list]
        self.assertEqual(len(destinations), len(set(destinations)))
        self.assertEqual(len(destinations), 6)
        receipt = harness.read_json(self.home / '.agent-harness/state.json')
        self.assertEqual(len(receipt['items']), 6)
        self.assertIn('codex:global-guidance', receipt['items'])
        self.assertIn('claude:global-guidance', receipt['items'])
        before = snapshot(self.home, timestamps=True)
        rows = harness.sync_global('all', self.home, 'copy')
        self.assertTrue(all(row['action'] == 'unchanged' for row in rows))
        # Receipts are persisted per successful edge; payload timestamps stay stable.
        after = snapshot(self.home, timestamps=True)
        before.pop('.agent-harness/state.json')
        after.pop('.agent-harness/state.json')
        before.pop('.agent-harness')
        after.pop('.agent-harness')
        self.assertEqual(before, after)

    def test_modified_skill_blocks_all_targets_and_preserves_all_files(self):
        harness.sync_global('codex', self.home, 'copy')
        (self.home / '.agents/skills/global-guidance/references/guide.md').write_text('User changes')
        self.revise('global-guidance')
        before = snapshot(self.home, timestamps=True)
        with self.assertRaisesRegex(ValueError, 'Unmanaged or modified destination'):
            harness.sync_global('all', self.home, 'copy')
        self.assertEqual(snapshot(self.home, timestamps=True), before)

    def test_conflicting_global_destinations_fail_before_writes(self):
        self.config['targets']['codex']['instruction_destination'] = '.claude/CLAUDE.md'
        self.save_registry()
        before = snapshot(self.home)
        with self.assertRaisesRegex(ValueError, 'Conflicting global destinations'):
            harness.sync_global('all', self.home, 'copy')
        self.assertEqual(snapshot(self.home), before)

    def test_legacy_claude_instruction_link_migrates_to_shared_source(self):
        destination = self.home / '.claude/CLAUDE.md'
        destination.parent.mkdir()
        self.symlink(destination, self.repo / 'instructions/CLAUDE.md')
        self.config['targets']['claude']['instructions'] = 'instructions/AGENTS.md'
        self.save_registry()
        harness.sync_global('claude-desktop', self.home, 'copy')
        self.assertEqual(destination.read_text(), 'Codex guidance\n')
        self.assertFalse(destination.is_symlink())

    def test_project_separate_roots_publish_and_lock_keeps_both_targets(self):
        value = self.declare()
        with mock.patch.object(harness, 'replace_item', wraps=harness.replace_item) as replace:
            rows = harness.sync_project(self.project)
        self.assertEqual(len(replace.call_args_list), 2)
        self.assertEqual({row['target'] for row in rows}, {'codex', 'claude'})
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['manifest'], value)
        self.assertEqual(lock['skills'][0]['targets'], ['codex', 'claude'])
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['project', 'doctor', '--project', str(self.project)]), 0)

    def test_codex_root_update_is_single_write(self):
        self.declare(targets=['codex'])
        harness.sync_project(self.project)
        self.revise('foundation')
        with mock.patch.object(harness, 'replace_item', wraps=harness.replace_item) as replace:
            rows = harness.sync_project(self.project)
        self.assertEqual(len(replace.call_args_list), 1)
        self.assertTrue(all(row['action'] == 'update' for row in rows))

    def test_codex_rejects_claude_only_selection_before_writes(self):
        self.entry('optional')['agents'] = ['claude']
        self.declare(targets=['codex'], skills=['optional'])
        before = snapshot(self.project)
        with self.assertRaisesRegex(ValueError, 'optional does not support target codex'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project), before)

    def test_removed_project_target_is_rejected_before_writes(self):
        self.declare(targets=['codex', 'zed', 'claude'])
        before = snapshot(self.project)
        with self.assertRaisesRegex(ValueError, 'Unsupported.*zed'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project), before)

    def test_scoped_claude_skill_stays_separate_from_codex(self):
        self.entry('optional')['agents'] = ['claude']
        self.declare(target_skills={'claude': ['optional']})
        rows = harness.sync_project(self.project)
        self.assertEqual({row['target'] for row in rows if row['id'] == 'optional'}, {'claude'})
        self.assertFalse((self.project / '.agents/skills/optional').exists())

    def test_desktop_aliases_normalize_and_all_matches_legacy_both(self):
        self.assertEqual(harness.targets('both', self.config), ['codex', 'claude'])
        self.assertEqual(harness.targets('all', self.config), ['codex', 'claude'])
        self.declare(targets=['codex-desktop', 'claude-desktop'])
        harness.sync_project(self.project)
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['manifest']['targets'], ['codex', 'claude'])
        self.assertEqual(harness.parse_target_skills(['claude-desktop:optional']), {'claude': ['optional']})
        with self.assertRaisesRegex(ValueError, 'Duplicate target_skills target alias'):
            harness.parse_target_skills(['claude-desktop:optional', 'claude-code:foundation'])

    def test_schema_one_codex_manifest_still_reconciles(self):
        value = self.declare(schema_version=1, targets=['codex'])
        value.pop('target_skills')
        harness.write_json(self.project / '.ai/project.json', value)
        harness.sync_project(self.project)
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['schema_version'], 1)
        self.assertEqual(lock['manifest']['targets'], ['codex'])
        self.assertTrue((self.project / '.agents/skills/foundation/SKILL.md').is_file())

    def test_legacy_catalog_installer_accepts_all_without_duplicate_write(self):
        entry = self.entry('foundation')
        with mock.patch.object(catalog.shutil, 'copytree', wraps=catalog.shutil.copytree) as copytree:
            catalog.install_many(self.data, [entry], self.project, 'all')
        publications = [call.args[1] for call in copytree.call_args_list
                        if Path(call.args[1]).name == 'foundation']
        self.assertEqual(len(publications), 2)
        self.assertEqual(len(set(publications)), 2)
        self.assertTrue((self.project / '.agents/skills/foundation/SKILL.md').is_file())
        self.assertTrue((self.project / '.claude/skills/foundation/SKILL.md').is_file())

    def test_codex_rejects_claude_only_global_skill_before_writes(self):
        self.entry('global-guidance')['agents'] = ['claude']
        before = snapshot(self.home)
        with self.assertRaisesRegex(ValueError, 'Global skill global-guidance does not support target codex'):
            harness.sync_global('codex', self.home, 'copy')
        self.assertEqual(snapshot(self.home), before)

    def test_capability_manifest_without_inline_targets_uses_target_registry(self):
        self.config.pop('targets')
        self.save_registry()
        with mock.patch.object(harness.target_registry, 'load', return_value={
            'codex': {'instructions': 'instructions/AGENTS.md',
                    'instruction_destination': '.codex/AGENTS.md',
                    'skills_destination': '.agents/skills'}}):
            rows = harness.sync_global('codex', self.home, 'copy')
        self.assertEqual({row['target'] for row in rows}, {'codex'})
        self.assertEqual((self.home / '.codex/AGENTS.md').read_text(), 'Codex guidance\n')

    def test_global_plan_checks_custom_client_roots_before_default_home_access(self):
        with mock.patch.object(harness.target_registry, 'validate_home_environment',
                               side_effect=ValueError('custom config root')) as validate:
            with self.assertRaisesRegex(ValueError, 'custom config root'):
                harness.global_plan('codex')
        validate.assert_called_once_with(None, ['codex'])

    def test_global_cli_commands_honor_declared_default_target(self):
        self.config['default_target'] = 'all'
        self.save_registry()
        for command in ('plan', 'sync', 'doctor'):
            with self.subTest(command=command), mock.patch('sys.stdout', new_callable=io.StringIO) as output:
                status = harness.main([command, '--home', str(self.home), '--mode', 'copy'])
            self.assertEqual(status, 0)
            self.assertEqual({row['target'] for row in json.loads(output.getvalue())}, {'codex', 'claude'})
        self.assertTrue((self.home / '.claude/CLAUDE.md').is_file())
        self.assertTrue((self.home / '.codex/AGENTS.md').is_file())

    def test_project_init_cli_honors_declared_default_target(self):
        self.config['default_target'] = 'all'
        self.save_registry()
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['project', 'init', '--project', str(self.project),
                                           '--skill', 'foundation']), 0)
        config = harness.read_json(self.project / '.ai/project.json')
        self.assertEqual(config['targets'], ['codex', 'claude'])
