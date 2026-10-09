"""Exercise the actual multi-client installation contract without touching user state."""
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import zipfile

from lib import configuration, handoff, target_install, targets


class TargetInstallTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name).resolve()

    def run_install(self, target='zed', **kwargs):
        return target_install.run(target=target, home=self.home, mode='copy', **kwargs)

    def test_default_is_zed_and_dry_run_writes_nothing(self):
        report = self.run_install(dry_run=True)
        self.assertEqual(report['targets'], ['zed'])
        self.assertEqual(list(self.home.iterdir()), [])

    def test_running_real_home_client_blocks_before_writes(self):
        with mock.patch.object(Path, 'home', return_value=self.home), mock.patch.object(target_install, 'running_clients', return_value=['zed']):
            report = self.run_install()
        self.assertFalse(report['ready'])
        self.assertIn('Close clients', report['blockers'][0])
        self.assertEqual(list(self.home.iterdir()), [])

    def test_all_targets_install_check_and_noop(self):
        report = self.run_install('all')
        self.assertTrue(report['ready'], report)
        self.assertEqual((self.home / '.claude/CLAUDE.md').read_bytes(),
                         (self.home / '.codex/AGENTS.md').read_bytes())
        files = {p: (p.read_bytes(), p.stat().st_mtime_ns) for p in self.home.rglob('*') if p.is_file() and p.name != 'state.json'}
        self.assertTrue(self.run_install('all', command='check')['ready'])
        self.assertTrue(self.run_install('all')['ready'])
        for path, expected in files.items():
            self.assertEqual((path.read_bytes(), path.stat().st_mtime_ns), expected, path)
        zed = configuration.parse_json((self.home / '.config/zed/settings.json').read_text())
        self.assertEqual(zed['auto_install_extensions']['toml'], True)
        self.assertEqual(zed['context_servers']['openai-docs'], {'url': 'https://developers.openai.com/mcp'})
        claude = json.loads((self.home / '.claude.json').read_text())
        self.assertEqual(claude['mcpServers']['openai-docs']['type'], 'http')
        codex = (self.home / '.codex/config.toml').read_text()
        self.assertIn('sandbox_mode = "workspace-write"', codex)
        self.assertIn('mcp_servers.openai-docs.url', codex)

    def test_client_starting_after_preflight_defers_configuration(self):
        with mock.patch.object(Path, 'home', return_value=self.home), mock.patch.object(target_install, 'running_clients', side_effect=[[], ['zed']]):
            report = self.run_install()
        self.assertEqual(report['status'], 'partially_applied')
        self.assertFalse(report['ready'])
        self.assertFalse((self.home / '.config/zed/settings.json').exists())

    def test_preserves_jsonc_and_makes_private_exact_backup(self):
        path = self.home / '.config/zed/settings.json'
        path.parent.mkdir(parents=True)
        original = '// private preference\n{"theme":"Personal", "agent":{"tool_permissions":{"default":"allow"}},}\n'
        path.write_text(original)
        result = self.run_install()
        self.assertTrue(result['ready'])
        self.assertIn('// private preference', path.read_text())
        self.assertEqual(configuration.parse_json(path.read_text())['theme'], 'Personal')
        backup = Path(result['backups'][0])
        self.assertEqual(backup.read_text(), original)
        if target_install.os.name != 'nt':
            self.assertEqual(backup.stat().st_mode & 0o777, 0o600)

    def test_malformed_settings_preflight_does_not_write_guidance(self):
        path = self.home / '.config/zed/settings.json'
        path.parent.mkdir(parents=True)
        path.write_text('{broken')
        with self.assertRaises(ValueError):
            self.run_install()
        self.assertFalse((self.home / '.agents').exists())

    def test_strict_claude_json_rejects_comments(self):
        path = self.home / '.claude/settings.json'
        path.parent.mkdir()
        path.write_text('// comment\n{}')
        with self.assertRaisesRegex(ValueError, 'strict JSON'):
            self.run_install('claude-desktop')
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
            self.run_install('claude-desktop')
        self.assertEqual(original.read_text(), '{}')

    def test_endpoint_change_cannot_forward_existing_credentials(self):
        path = self.home / '.config/zed/settings.json'
        path.parent.mkdir(parents=True)
        original = json.dumps({'context_servers': {'openai-docs': {'url': 'https://internal.invalid/mcp', 'headers': {'Authorization': 'TEST_SENTINEL'}}}})
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
        path = self.home / '.config/zed/settings.json'
        path.write_text(path.read_text().replace('"confirm"', '"allow"'))
        before = path.read_bytes()
        result = self.run_install(command='check')
        self.assertFalse(result['ready'])
        self.assertEqual(path.read_bytes(), before)

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
        self.assertTrue((self.home / '.agents/skills/skill-catalog/SKILL.md').exists())

    def test_stale_preflight_does_not_overwrite_concurrent_edit(self):
        _, configs, _ = target_install.prepare('zed', self.home, 'copy')
        path = configs[0]['path']
        path.parent.mkdir(parents=True)
        path.write_text('{"theme":"new"}')
        with self.assertRaisesRegex(ValueError, 'changed after preflight'):
            target_install.write_config(self.home, configs[0])
        self.assertEqual(path.read_text(), '{"theme":"new"}')

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

    def test_windows_zed_adapter_respects_isolated_home(self):
        with mock.patch.object(targets.sys, 'platform', 'win32'):
            adapter = targets.global_adapter('zed', self.home)
        self.assertEqual(adapter['settings_destination'], 'AppData/Roaming/Zed/settings.json')


if __name__ == '__main__':
    unittest.main()
