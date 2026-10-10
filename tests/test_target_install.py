"""Exercise the actual multi-client installation contract without touching user state."""
from contextlib import contextmanager
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import zipfile

from lib import handoff, target_install


class TargetInstallTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name).resolve()

    def run_install(self, target=None, **kwargs):
        if target is not None:
            kwargs['target'] = target
        return target_install.run(home=self.home, mode='copy', **kwargs)

    def test_default_is_both_desktops_and_dry_run_writes_nothing(self):
        report = self.run_install(dry_run=True)
        self.assertEqual(report['targets'], ['codex', 'claude'])
        self.assertEqual(report['status'], 'planned')
        self.assertFalse(report['guidance_completed'])
        self.assertFalse(report['ready'])
        self.assertEqual(list(self.home.iterdir()), [])

    def test_install_does_not_query_or_change_host_preferences(self):
        codex = self.home / '.codex/config.toml'
        codex.parent.mkdir()
        original_codex = ('model = "personal-model"\n'
                          'approval_policy = "untrusted"\n'
                          'sandbox_mode = "read-only"\n'
                          '[desktop]\nappearance = "system"\n')
        codex.write_text(original_codex)
        claude = self.home / '.claude/settings.json'
        claude.parent.mkdir()
        original_claude = b'{"theme":"Personal", "permissions":{"defaultMode":"plan"}}\n'
        claude.write_bytes(original_claude)
        claude.chmod(0o640)
        before = (claude.read_bytes(), claude.stat().st_mtime_ns, claude.stat().st_mode)
        with mock.patch.object(target_install.settings, 'app_running',
                               side_effect=AssertionError('Integration setup must not control the client')):
            report = self.run_install()
        self.assertTrue(report['ready'], report)
        inserted = 'mcp_servers.openai-docs.url = "https://developers.openai.com/mcp"\n'
        self.assertEqual(codex.read_text().replace(inserted, ''), original_codex)
        self.assertEqual((claude.read_bytes(), claude.stat().st_mtime_ns, claude.stat().st_mode), before)
        self.assertEqual({row['path'] for row in report['configuration']},
                         {str(codex), str(self.home / '.claude.json')})
        self.assertTrue(all('mcp' in key.lower()
                            for row in report['configuration'] for key in row['keys']))

    def test_all_targets_install_check_and_noop(self):
        report = self.run_install('all')
        self.assertTrue(report['ready'], report)
        self.assertEqual(report['status'], 'installed')
        self.assertTrue(report['guidance_completed'])
        self.assertFalse(report['changes_pending'])
        self.assertEqual(report['blockers'], [])
        self.assertEqual((self.home / '.claude/CLAUDE.md').read_bytes(),
                         (self.home / '.codex/AGENTS.md').read_bytes())
        files = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in self.home.rglob('*') if p.is_file() and p.name != 'state.json'}
        self.assertTrue(self.run_install('all', command='check')['ready'])
        self.assertTrue(self.run_install('all')['ready'])
        for path, expected in files.items():
            self.assertEqual((path.read_bytes(), path.stat().st_mtime_ns), expected, path)
        self.assertEqual(report['targets'], ['codex', 'claude'])
        self.assertFalse((self.home / '.config/zed').exists())
        claude = json.loads((self.home / '.claude.json').read_text())
        self.assertEqual(claude['mcpServers']['openai-docs']['type'], 'http')
        codex = (self.home / '.codex/config.toml').read_text()
        self.assertIn('mcp_servers.openai-docs.url', codex)
        self.assertNotIn('sandbox_mode', codex)
        self.assertNotIn('approval_policy', codex)
        self.assertFalse((self.home / '.claude/settings.json').exists())

    def test_stale_configuration_snapshot_blocks_before_guidance_writes(self):
        prepare = target_install.prepare
        path = self.home / '.claude.json'
        concurrent = '{"projects":{"new-project":{"trusted":true}}}\n'

        def changed_after_prepare(*args, **kwargs):
            result = prepare(*args, **kwargs)
            path.write_text(concurrent)
            return result

        with mock.patch.object(target_install, 'prepare', side_effect=changed_after_prepare):
            report = self.run_install()
        self.assertEqual(report['status'], 'partially_applied')
        self.assertFalse(report['ready'])
        self.assertFalse(report['guidance_completed'])
        self.assertIn('changed after preflight', report['blockers'][0])
        self.assertEqual(path.read_text(), concurrent)
        self.assertEqual(list(self.home.iterdir()), [path])

    def test_preserves_unrelated_mcp_configuration_and_makes_private_exact_backup(self):
        path = self.home / '.claude.json'
        original = ('{"theme":"Personal", "projects":{"private-project":{"trusted":true}}, '
                    '"mcpServers":{"personal":{"type":"stdio","command":"local-server"}}}\n')
        path.write_text(original)
        result = self.run_install('claude')
        self.assertTrue(result['ready'], result)
        merged = json.loads(path.read_text())
        self.assertEqual(merged['theme'], 'Personal')
        self.assertEqual(merged['projects'], {'private-project': {'trusted': True}})
        self.assertEqual(merged['mcpServers']['personal'], {'type': 'stdio', 'command': 'local-server'})
        self.assertEqual(merged['mcpServers']['openai-docs'],
                         {'type': 'http', 'url': 'https://developers.openai.com/mcp'})
        self.assertEqual(len(result['backups']), 1)
        backup = Path(result['backups'][0])
        self.assertEqual(backup.parent, path.parent)
        self.assertTrue(backup.name.startswith(path.name + '.harness-backup-'))
        self.assertEqual(backup.read_bytes(), original.encode())
        if target_install.os.name != 'nt':
            self.assertEqual(backup.stat().st_mode & 0o777, 0o600)

    def test_unowned_malformed_settings_are_not_read_or_changed(self):
        path = self.home / '.claude/settings.json'
        path.parent.mkdir(parents=True)
        original = b'{broken\xff unowned preferences'
        path.write_bytes(original)
        path.chmod(0o640)
        before = (path.read_bytes(), path.stat().st_mtime_ns, path.stat().st_mode)
        self.assertTrue(self.run_install()['ready'])
        self.assertEqual((path.read_bytes(), path.stat().st_mtime_ns, path.stat().st_mode), before)

    def test_strict_claude_json_rejects_comments(self):
        path = self.home / '.claude.json'
        path.write_text('// comment\n{}')
        with self.assertRaisesRegex(ValueError, 'strict JSON'):
            self.run_install('claude')
        self.assertFalse((self.home / '.claude/skills').exists())

    def test_transport_conflict_preserves_all_targets(self):
        path = self.home / '.claude.json'
        path.write_text(json.dumps({'mcpServers': {'openai-docs': {'command': '/old/server'}}}))
        with self.assertRaisesRegex(ValueError, 'transport conflict'):
            self.run_install('all')
        self.assertEqual(list(self.home.iterdir()), [path])

    def test_symlink_config_is_never_followed(self):
        original = self.home / 'outside'
        original.write_text('{}')
        (self.home / '.claude.json').symlink_to(original)
        with self.assertRaisesRegex(ValueError, 'regular file'):
            self.run_install('claude')
        self.assertEqual(original.read_text(), '{}')

    def test_endpoint_change_cannot_forward_existing_credentials(self):
        path = self.home / '.claude.json'
        original = json.dumps({'mcpServers': {'openai-docs': {'url': 'https://internal.invalid/mcp', 'headers': {'Authorization': 'TEST_SENTINEL'}}}})
        path.write_text(original)
        with self.assertRaisesRegex(ValueError, 'identity conflict'):
            self.run_install()
        self.assertEqual(path.read_text(), original)
        self.assertFalse((self.home / '.agents').exists())

    def test_codex_inline_headers_preserved_when_endpoint_matches(self):
        path = self.home / '.codex/config.toml'
        path.parent.mkdir(parents=True)
        original = '[mcp_servers.openai-docs]\nurl = "https://developers.openai.com/mcp"\nhttp_headers = {"X-Custom" = "ordinary-value"}\n'
        path.write_text(original)
        self.assertTrue(self.run_install('codex')['ready'])
        self.assertIn(original, path.read_text())

    def test_check_reports_configuration_drift_without_repair(self):
        self.run_install()
        path = self.home / '.claude.json'
        path.write_text('{"mcpServers":{},"theme":"unchanged"}\n')
        before = (path.read_bytes(), path.stat().st_mtime_ns)
        result = self.run_install(command='check')
        self.assertFalse(result['ready'])
        self.assertTrue(result['changes_pending'])
        self.assertFalse(result['guidance_completed'])
        self.assertEqual(result['status'], 'planned')
        self.assertTrue(next(row['changed'] for row in result['configuration'] if row['path'] == str(path)))
        self.assertEqual((path.read_bytes(), path.stat().st_mtime_ns), before)

    def test_stdio_is_translated_without_launching_command(self):
        source = {'local': {'transport': 'stdio', 'command': '/not/executed', 'args': ['--read-only']}}
        with mock.patch.object(target_install, 'mcp_sources', return_value=source):
            self.assertTrue(self.run_install('all')['ready'])
        value = json.loads((self.home / '.claude.json').read_text())['mcpServers']['local']
        self.assertEqual(value, {'type': 'stdio', 'command': '/not/executed', 'args': ['--read-only']})

    def test_partial_write_failure_never_claims_ready(self):
        with mock.patch.object(target_install, 'write_config', side_effect=OSError('disk full')):
            result = self.run_install()
        self.assertEqual(result['status'], 'partially_applied')
        self.assertFalse(result['ready'])
        self.assertTrue(result['guidance_completed'])
        self.assertTrue((self.home / '.agents/skills/skill-catalog/SKILL.md').exists())

    def test_guidance_failure_is_not_reported_as_completed(self):
        with mock.patch.object(target_install.harness, 'sync_global', side_effect=OSError('guidance disk full')):
            result = self.run_install()
        self.assertEqual(result['status'], 'partially_applied')
        self.assertFalse(result['ready'])
        self.assertFalse(result['guidance_completed'])
        self.assertEqual(result['blockers'], ['guidance disk full'])
        self.assertEqual(list(self.home.iterdir()), [])

    def test_stale_preflight_does_not_overwrite_concurrent_edit(self):
        _, configs, _ = target_install.prepare('claude', self.home, 'copy')
        path = configs[0]['path']
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('{"theme":"new"}')
        with self.assertRaisesRegex(ValueError, 'changed after preflight'):
            target_install.write_config(self.home, configs[0])
        self.assertEqual(path.read_text(), '{"theme":"new"}')

    def test_parent_replaced_during_backup_cannot_redirect_configuration_write(self):
        path = self.home / '.codex/config.toml'
        path.parent.mkdir()
        original = b'model = "personal-model"\n'
        path.write_bytes(original)
        outside = self.home / 'unowned-directory'
        outside.mkdir()
        (outside / path.name).write_bytes(original)
        moved = self.home / 'original-codex-directory'
        _, configs, _ = target_install.prepare('codex', self.home, 'copy')
        named_temporary_file = target_install.tempfile.NamedTemporaryFile

        @contextmanager
        def replace_parent_after_backup(*args, **kwargs):
            with named_temporary_file(*args, **kwargs) as stream:
                yield stream
            if kwargs.get('prefix') == path.name + '.harness-backup-':
                path.parent.rename(moved)
                path.parent.symlink_to(outside, target_is_directory=True)

        with mock.patch.object(target_install.tempfile, 'NamedTemporaryFile',
                               side_effect=replace_parent_after_backup):
            with self.assertRaisesRegex(ValueError, 'directory|changed'):
                target_install.write_config(self.home, configs[0])
        self.assertEqual((outside / path.name).read_bytes(), original)
        self.assertEqual((moved / path.name).read_bytes(), original)

    def test_concurrent_edit_during_staging_is_preserved_before_publication(self):
        path = self.home / '.claude.json'
        original = b'{"theme":"before"}\n'
        concurrent = b'{"theme":"concurrent-app-write"}\n'
        path.write_bytes(original)
        _, configs, _ = target_install.prepare('claude', self.home, 'copy')
        fsync = target_install.os.fsync

        def edit_during_flush(descriptor):
            fsync(descriptor)
            path.write_bytes(concurrent)

        with mock.patch.object(target_install.os, 'fsync', side_effect=edit_during_flush):
            with self.assertRaisesRegex(ValueError, 'changed'):
                target_install.write_config(self.home, configs[0])
        self.assertEqual(path.read_bytes(), concurrent)
        backups = list(path.parent.glob(path.name + '.harness-backup-*'))
        self.assertEqual(len(backups), 1)
        self.assertEqual(backups[0].read_bytes(), original)
        self.assertEqual({item.name for item in path.parent.iterdir()}, {path.name, backups[0].name})

    def test_handoff_is_manual_and_does_not_include_host_catalog(self):
        output = self.home / 'handoff'
        report = handoff.export(output)
        self.assertEqual(report['activation'], 'manual_account_customize')
        self.assertFalse((output / 'skill-catalog.zip').exists())
        with zipfile.ZipFile(output / 'engineering-judgment.zip') as archive:
            self.assertIn('engineering-judgment/SKILL.md', archive.namelist())
            self.assertIn('engineering-judgment/references/boundary-decisions.md', archive.namelist())
        with self.assertRaisesRegex(ValueError, 'already exists'):
            handoff.export(output)

    def test_removed_target_is_rejected_before_writes(self):
        with self.assertRaisesRegex(ValueError, 'Unsupported target: zed'):
            self.run_install('zed')
        self.assertEqual(list(self.home.iterdir()), [])

    def test_existing_zed_configuration_is_untouched(self):
        path = self.home / '.config/zed/settings.json'
        path.parent.mkdir(parents=True)
        original = '{"context_servers":{"personal":{"command":"local-server"}}}\n'
        path.write_text(original)
        before = path.stat().st_mtime_ns
        self.assertTrue(self.run_install()['ready'])
        self.assertEqual(path.read_text(), original)
        self.assertEqual(path.stat().st_mtime_ns, before)

    def test_handoff_preserves_stdio_transport_fields_without_a_zed_adapter(self):
        servers = {'local': {'transport': 'stdio', 'command': '/not/executed',
                             'args': ['--read-only'], 'env': {'MODE': 'test'}}}
        output = self.home / 'handoff'
        with mock.patch.object(target_install, 'mcp_sources', return_value=servers):
            handoff.export(output)
        value = json.loads((output / 'chat-mcp-fragment.json').read_text())
        self.assertEqual(value, {'mcpServers': {'local': {
            'command': '/not/executed', 'args': ['--read-only'], 'env': {'MODE': 'test'}}}})



if __name__ == '__main__':
    unittest.main()
