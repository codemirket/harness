"""Target adapters preserve ownership and exact portable skill selections."""
import io
import json
from pathlib import Path
from unittest import mock

from test_harness import HarnessFixture, catalog, harness, snapshot


class TargetAdapterTests(HarnessFixture):
    def setUp(self):
        super().setUp()
        self.config['targets']['zed'] = {
            'instructions': 'instructions/AGENTS.md',
            'instruction_destination': '.config/zed/AGENTS.md',
            'skills_destination': '.agents/skills',
        }
        self.save_registry()

    def declare(self, **changes):
        value = {'schema_version': 2, 'targets': ['codex', 'zed', 'claude'],
                 'profiles': [], 'skills': ['foundation'], 'skip': [], 'target_skills': {}}
        value.update(changes)
        harness.write_json(self.project / '.ai/project.json', value)
        return value

    def test_all_global_targets_publish_shared_skills_once_and_record_each_owner(self):
        before = snapshot(self.home)
        rows = harness.public_jobs(harness.global_plan('all', self.home, 'copy')[3])
        self.assertEqual(snapshot(self.home), before)
        self.assertEqual(len(rows), 9)
        with mock.patch.object(harness, 'replace_item', wraps=harness.replace_item) as replace:
            harness.sync_global('all', self.home, 'copy')
        destinations = [call.args[0] for call in replace.call_args_list]
        self.assertEqual(len(destinations), len(set(destinations)))
        self.assertEqual(len(destinations), 7)
        receipt = harness.read_json(self.home / '.agent-harness/state.json')
        self.assertEqual(len(receipt['items']), 9)
        self.assertEqual(receipt['items']['codex:global-guidance'],
                         receipt['items']['zed:global-guidance'])
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

    def test_zed_can_update_a_shared_skill_previously_owned_by_codex(self):
        harness.sync_global('codex', self.home, 'copy')
        self.revise('global-guidance')
        rows = harness.sync_global('zed', self.home, 'copy')
        self.assertEqual(next(row['action'] for row in rows if row['id'] == 'global-guidance'), 'update')
        self.assertIn('Reviewed revision', (self.home / '.agents/skills/global-guidance/references/guide.md').read_text())

    def test_modified_shared_skill_blocks_other_target_and_preserves_all_files(self):
        harness.sync_global('codex', self.home, 'copy')
        (self.home / '.agents/skills/global-guidance/references/guide.md').write_text('User changes')
        self.revise('global-guidance')
        before = snapshot(self.home, timestamps=True)
        with self.assertRaisesRegex(ValueError, 'Unmanaged or modified destination'):
            harness.sync_global('zed', self.home, 'copy')
        self.assertEqual(snapshot(self.home, timestamps=True), before)

    def test_conflicting_global_destinations_fail_before_writes(self):
        self.config['targets']['zed']['instruction_destination'] = '.claude/CLAUDE.md'
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

    def test_project_shared_root_publishes_once_and_lock_keeps_all_targets(self):
        value = self.declare()
        with mock.patch.object(harness, 'replace_item', wraps=harness.replace_item) as replace:
            rows = harness.sync_project(self.project)
        self.assertEqual(len(replace.call_args_list), 2)
        self.assertEqual({row['target'] for row in rows}, {'codex', 'zed', 'claude'})
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['manifest'], value)
        self.assertEqual(lock['skills'][0]['targets'], ['codex', 'zed', 'claude'])
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['project', 'doctor', '--project', str(self.project)]), 0)

    def test_shared_root_update_is_single_write(self):
        self.declare(targets=['codex', 'zed'])
        harness.sync_project(self.project)
        self.revise('foundation')
        with mock.patch.object(harness, 'replace_item', wraps=harness.replace_item) as replace:
            rows = harness.sync_project(self.project)
        self.assertEqual(len(replace.call_args_list), 1)
        self.assertTrue(all(row['action'] == 'update' for row in rows))

    def test_zed_rejects_claude_only_selection_before_writes(self):
        self.entry('optional')['agents'] = ['claude']
        self.declare(targets=['zed'], skills=['optional'])
        before = snapshot(self.project)
        with self.assertRaisesRegex(ValueError, 'optional does not support target zed'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project), before)

    def test_diverging_shared_root_selections_are_rejected(self):
        self.declare(target_skills={'zed': ['optional']})
        before = snapshot(self.project)
        with self.assertRaisesRegex(ValueError, r'share .agents/skills'):
            harness.sync_project(self.project)
        self.assertEqual(snapshot(self.project), before)

    def test_scoped_claude_skill_stays_separate_from_shared_codex_zed(self):
        self.entry('optional')['agents'] = ['claude']
        self.declare(target_skills={'claude': ['optional']})
        rows = harness.sync_project(self.project)
        self.assertEqual({row['target'] for row in rows if row['id'] == 'optional'}, {'claude'})
        self.assertFalse((self.project / '.agents/skills/optional').exists())

    def test_desktop_aliases_normalize_and_legacy_both_does_not_add_zed(self):
        self.assertEqual(harness.targets('both', self.config), ['codex', 'claude'])
        self.declare(targets=['codex-desktop', 'zed', 'claude-desktop'])
        harness.sync_project(self.project)
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['manifest']['targets'], ['codex', 'zed', 'claude'])
        self.assertEqual(harness.parse_target_skills(['claude-desktop:optional']), {'claude': ['optional']})
        with self.assertRaisesRegex(ValueError, 'Duplicate target_skills target alias'):
            harness.parse_target_skills(['claude-desktop:optional', 'claude-code:foundation'])

    def test_schema_one_zed_manifest_still_reconciles(self):
        value = self.declare(schema_version=1, targets=['zed'])
        value.pop('target_skills')
        harness.write_json(self.project / '.ai/project.json', value)
        harness.sync_project(self.project)
        lock = harness.read_json(self.project / '.ai/project.lock.json')
        self.assertEqual(lock['schema_version'], 1)
        self.assertEqual(lock['manifest']['targets'], ['zed'])
        self.assertTrue((self.project / '.agents/skills/foundation/SKILL.md').is_file())

    def test_legacy_catalog_installer_accepts_zed_and_all_without_duplicate_write(self):
        entry = self.entry('foundation')
        with mock.patch.object(catalog.shutil, 'copytree', wraps=catalog.shutil.copytree) as copytree:
            catalog.install_many(self.data, [entry], self.project, 'all')
        publications = [call.args[1] for call in copytree.call_args_list
                        if Path(call.args[1]).name == 'foundation']
        self.assertEqual(len(publications), 2)
        self.assertEqual(len(set(publications)), 2)
        self.assertTrue((self.project / '.agents/skills/foundation/SKILL.md').is_file())
        self.assertTrue((self.project / '.claude/skills/foundation/SKILL.md').is_file())

    def test_zed_rejects_claude_only_global_skill_before_writes(self):
        self.entry('global-guidance')['agents'] = ['claude']
        before = snapshot(self.home)
        with self.assertRaisesRegex(ValueError, 'Global skill global-guidance does not support target zed'):
            harness.sync_global('zed', self.home, 'copy')
        self.assertEqual(snapshot(self.home), before)

    def test_capability_manifest_without_inline_targets_uses_target_registry(self):
        self.config.pop('targets')
        self.save_registry()
        with mock.patch.object(harness.target_registry, 'load', return_value={
            'zed': {'instructions': 'instructions/AGENTS.md',
                    'instruction_destination': '.config/zed/AGENTS.md',
                    'skills_destination': '.agents/skills'}}):
            rows = harness.sync_global('zed', self.home, 'copy')
        self.assertEqual({row['target'] for row in rows}, {'zed'})
        self.assertEqual((self.home / '.config/zed/AGENTS.md').read_text(), 'Codex guidance\n')

    def test_global_plan_checks_custom_client_roots_before_default_home_access(self):
        with mock.patch.object(harness.target_registry, 'validate_home_environment',
                               side_effect=ValueError('custom config root')) as validate:
            with self.assertRaisesRegex(ValueError, 'custom config root'):
                harness.global_plan('zed')
        validate.assert_called_once_with(None, ['zed'])

    def test_global_cli_commands_honor_declared_default_target(self):
        self.config['default_target'] = 'zed'
        self.save_registry()
        for command in ('plan', 'sync', 'doctor'):
            with self.subTest(command=command), mock.patch('sys.stdout', new_callable=io.StringIO) as output:
                status = harness.main([command, '--home', str(self.home), '--mode', 'copy'])
            self.assertEqual(status, 0)
            self.assertEqual({row['target'] for row in json.loads(output.getvalue())}, {'zed'})
        self.assertFalse((self.home / '.codex').exists())
        self.assertTrue((self.home / '.config/zed/AGENTS.md').is_file())

    def test_project_init_cli_honors_declared_default_target(self):
        self.config['default_target'] = 'zed'
        self.save_registry()
        with mock.patch('sys.stdout', new_callable=io.StringIO):
            self.assertEqual(harness.main(['project', 'init', '--project', str(self.project),
                                           '--skill', 'foundation']), 0)
        config = harness.read_json(self.project / '.ai/project.json')
        self.assertEqual(config['targets'], ['zed'])
