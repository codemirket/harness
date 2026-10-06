"""Portable settings behavior; every write stays inside a temporary home."""
import builtins
import importlib.util
import io
import json
from pathlib import Path
import stat
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('personal_settings', ROOT / 'lib/settings.py')
settings = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(settings)


class SettingsTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name) / 'home'
        self.home.mkdir()
        self.config = self.home / '.codex/config.toml'
        self.manifest = Path(self.temp.name) / 'preferences.json'
        self.desired = {'schema_version': 1, 'config': {'model': 'gpt-test', 'model_reasoning_effort': 'high'},
                        'desktop': {'appearanceTheme': 'dark', 'followUpQueueMode': 'queue',
                                    'appearanceLightChromeTheme': {'accent': '#112233', 'fonts': {'code': 'ui-monospace'}}},
                        'plugins': {'pdf@openai-primary-runtime': {'enabled': True}}}
        self.manifest.write_text(json.dumps(self.desired))

    def write(self, text):
        self.config.parent.mkdir(parents=True, exist_ok=True)
        self.config.write_bytes(text.encode())
        self.config.chmod(0o600)

    def snapshot(self):
        return {str(p.relative_to(self.home)): (p.read_bytes(), p.stat().st_mtime_ns)
                for p in self.home.rglob('*') if p.is_file()}

    def test_capture_excludes_secrets_permissions_private_plugins_and_state(self):
        self.write('model = "gpt-test"\napi_key = "PRIVATE_SENTINEL"\napproval_policy = "never"\n'
                   '[desktop]\nappearanceTheme = "dark"\nprojectlessWorkspaceRoot = "/private/work"\n'
                   '[plugins."pdf@openai-primary-runtime"]\nenabled = true\n'
                   '[plugins."employer@private"]\nenabled = true\n'
                   '[plugins."pdf@openai-primary-runtime".mcp_servers.server]\n'
                   'url = "https://secret.invalid/token"\n'
                   '[projects."/private/project"]\ntrust_level = "trusted"\n')
        state = self.config.parent / '.codex-global-state.json'
        state.write_text('{not valid JSON: MUST_NOT_BE_READ}')
        before = self.snapshot()
        data = settings.capture(self.home)
        serialized = json.dumps(data)
        for forbidden in ['PRIVATE_SENTINEL', '/private/', 'employer', 'secret.invalid', 'trust_level']:
            self.assertNotIn(forbidden, serialized)
        self.assertEqual(data['config'], {'model': 'gpt-test'})
        self.assertEqual(data['desktop'], {'appearanceTheme': 'dark'})
        self.assertEqual(data['capture_summary']['private_or_local_plugins_skipped'], 1)
        self.assertEqual(before, self.snapshot())

    def test_repeated_unknown_array_tables_do_not_break_capture(self):
        self.write('model="gpt-test"\n[[skills.config]]\npath="/one"\nenabled=true\n'
                   '[[skills.config]]\npath="/two"\nenabled=false\n')
        self.assertEqual(settings.capture(self.home)['config']['model'], 'gpt-test')

    def test_multiline_unknown_string_cannot_inject_selected_keys(self):
        self.write('developer_instructions = """\nmodel = "gpt-fake"\n[desktop]\nappearanceTheme="dark"\n"""\nmodel="gpt-real"\n')
        data = settings.capture(self.home)
        self.assertEqual(data['config'], {'model': 'gpt-real'})
        self.assertEqual(data['desktop'], {})

    def test_plan_new_home_does_not_write_and_marks_app_closure(self):
        before = self.snapshot()
        with patch.object(settings, 'app_running', return_value=True):
            result = settings.plan(self.home, self.manifest)
        self.assertTrue(result['changed'])
        self.assertFalse(result['ready_to_apply'])
        self.assertTrue(result['requires_app_closed'])
        self.assertEqual(before, self.snapshot())
        self.assertFalse(self.config.parent.exists())

    def test_blocked_apply_never_creates_destination_or_lock(self):
        with patch.object(settings, 'app_running', return_value=True):
            result = settings.apply(self.home, self.manifest)
        self.assertEqual(result['status'], 'deferred_app_must_close')
        self.assertFalse(result['applied'])
        self.assertFalse(self.config.parent.exists())

    def test_unknown_app_status_blocks_changed_apply(self):
        with patch.object(settings, 'app_running', return_value=None):
            result = settings.apply(self.home, self.manifest)
        self.assertFalse(result['ready_to_apply'])
        self.assertEqual(result['status'], 'deferred_app_must_close')
        self.assertFalse(self.config.parent.exists())

    def test_apply_preserves_unknown_fields_comments_permissions_and_backups(self):
        original = ('# my preferences\r\nmodel = "old" # keep this note\r\n'
                    'api_key = "SECRET_NEVER_EXPORTED"\r\n'
                    '[desktop]\r\nappearanceTheme = "light" # UI note\r\nunknown = { x = "literal#hash" }\r\n'
                    '[projects."C:\\work"]\r\ntrust_level = "trusted"\r\n')
        # TOML basic strings require escaped backslashes in quoted keys.
        original = original.replace('"C:\\work"', "'C:\\work'")
        self.write(original)
        with patch.object(settings, 'app_running', return_value=False):
            result = settings.apply(self.home, self.manifest)
        self.assertTrue(result['applied'])
        self.assertTrue(result['pending_restart'])
        actual = self.config.read_bytes().decode()
        self.assertIn('model = "gpt-test" # keep this note\r\n', actual)
        self.assertIn('api_key = "SECRET_NEVER_EXPORTED"\r\n', actual)
        self.assertIn('unknown = { x = "literal#hash" }\r\n', actual)
        self.assertIn("[projects.'C:\\work']\r\ntrust_level = \"trusted\"\r\n", actual)
        backup = Path(result['backup'])
        self.assertEqual(backup.read_bytes(), original.encode())
        self.assertEqual(stat.S_IMODE(backup.stat().st_mode), 0o600)
        self.assertNotIn('SECRET_NEVER_EXPORTED', json.dumps(result))
        self.assertFalse((self.config.parent / '.personal-ai-settings.lock').exists())
        self.assertEqual(settings.capture(self.home)['desktop']['appearanceTheme'], 'dark')

    def test_repeat_apply_is_noop_even_with_app_open(self):
        with patch.object(settings, 'app_running', return_value=False):
            settings.apply(self.home, self.manifest)
        before = self.snapshot()
        with patch.object(settings, 'app_running', side_effect=AssertionError('No process check for no-op')):
            result = settings.apply(self.home, self.manifest)
        self.assertEqual(result['status'], 'unchanged')
        self.assertTrue(result['ready_to_apply'])
        self.assertFalse(result['applied'])
        self.assertEqual(before, self.snapshot())

    def test_closed_app_new_configuration_contains_exact_desired_fields(self):
        with patch.object(settings, 'app_running', return_value=False):
            settings.apply(self.home, self.manifest)
        captured = settings.capture(self.home)
        for key in ['config', 'desktop', 'plugins']:
            self.assertEqual(captured[key], self.desired[key])
        self.assertEqual(stat.S_IMODE(self.config.stat().st_mode), 0o600)

    def test_unknown_manifest_fields_refused_before_writes(self):
        for section, key, value in [('config', 'approval_policy', 'never'),
                                     ('desktop', 'projectlessWorkspaceRoot', '/secret'),
                                     ('plugins', 'private@employer', {'enabled': True})]:
            data = json.loads(json.dumps(self.desired))
            data[section][key] = value
            self.manifest.write_text(json.dumps(data))
            with self.assertRaises(ValueError):
                settings.apply(self.home, self.manifest)
            self.assertFalse(self.config.parent.exists())

    def test_invalid_enum_font_path_and_plugin_credentials_rejected(self):
        cases = [(('model_reasoning_effort',), 'invented'),
                 (('desktop', 'appearanceLightChromeTheme', 'fonts', 'code'), '/home/fonts/font.ttf'),
                 (('plugins', 'pdf@openai-primary-runtime', 'token'), 'secret'),
                 (('desktop', 'contrast'), 40)]
        for path, value in cases:
            with self.subTest(path=path), self.assertRaises(ValueError):
                settings.validate_value(path, value)

    def test_inline_structured_selected_value_is_refused_not_replaced(self):
        self.write('[desktop]\nappearanceLightChromeTheme = {accent = "#112233", private = "retain"}\n')
        before = self.snapshot()
        with self.assertRaises(ValueError):
            settings.plan(self.home, self.manifest)
        self.assertEqual(before, self.snapshot())

    def test_missing_source_does_not_delete_target_unspecified_preferences(self):
        self.write('[desktop]\ncodeFontSize=20\nunknownColor="custom"\n')
        with patch.object(settings, 'app_running', return_value=False):
            settings.apply(self.home, self.manifest)
        self.assertIn('codeFontSize=20', self.config.read_text())
        self.assertIn('unknownColor="custom"', self.config.read_text())

    def test_symlinked_configuration_refused(self):
        self.config.parent.mkdir()
        external = Path(self.temp.name) / 'external.toml'
        external.write_text('model="old"\n')
        self.config.symlink_to(external)
        with self.assertRaises(ValueError):
            settings.apply(self.home, self.manifest)
        self.assertEqual(external.read_text(), 'model="old"\n')

    def test_process_detection_failure_is_unknown(self):
        with patch.object(settings.subprocess, 'run', side_effect=OSError('unavailable')):
            self.assertIsNone(settings.app_running())

    def test_concurrent_edit_does_not_get_overwritten(self):
        self.write('model="old"\n')
        calls = 0
        def state():
            nonlocal calls
            calls += 1
            if calls == 2:
                self.config.write_text('model="user-change"\n')
            return False
        with patch.object(settings, 'app_running', side_effect=state), self.assertRaises(ValueError):
            settings.apply(self.home, self.manifest)
        self.assertEqual(self.config.read_text(), 'model="user-change"\n')
        self.assertFalse((self.config.parent / '.personal-ai-settings.lock').exists())

    def test_app_reopened_after_preflight_defers_without_config_changes(self):
        self.write('model="old"\n')
        before = self.config.read_bytes()
        with patch.object(settings, 'app_running', side_effect=[False, True]):
            result = settings.apply(self.home, self.manifest)
        self.assertEqual(result['status'], 'deferred_app_must_close')
        self.assertEqual(self.config.read_bytes(), before)

    def test_failed_atomic_replace_keeps_original_and_backup(self):
        self.write('model="old"\n')
        with patch.object(settings, 'app_running', return_value=False), patch.object(settings.os, 'replace', side_effect=OSError('disk issue')):
            with self.assertRaises(OSError):
                settings.apply(self.home, self.manifest)
        self.assertEqual(self.config.read_text(), 'model="old"\n')
        backups = list((self.config.parent / 'backups').glob('*/config.toml'))
        self.assertEqual(len(backups), 1)
        self.assertEqual(backups[0].read_text(), 'model="old"\n')
        self.assertFalse(list(self.config.parent.glob('.personal-ai-settings-*')))

    def test_python39_fallback_preserves_opaque_statements(self):
        original_import = builtins.__import__
        def importing(name, *args, **kwargs):
            if name == 'tomllib':
                raise ImportError()
            return original_import(name, *args, **kwargs)
        text = '# start\nmodel="old"\n[other]\nsecret="do not touch"\n[[skills.config]]\npath="x"\n[[skills.config]]\npath="y"\n'
        with patch('builtins.__import__', side_effect=importing):
            merged, _ = settings.merge_config(text, {('model',): 'new'})
        self.assertEqual(merged, text.replace('model="old"', 'model="new"'))

    def test_trailing_array_comma_and_hash_inside_strings(self):
        text = '[desktop]\nenabled-reasoning-efforts = [\n"low", # note\n"high",\n]\n'
        self.write(text)
        self.assertEqual(settings.capture(self.home)['desktop']['enabled-reasoning-efforts'], ['low', 'high'])
        self.assertEqual(settings.parse_value('"x#y" # comment'), 'x#y')

    def test_doctor_reports_missing_availability_without_connecting(self):
        with patch.object(settings, 'app_running', return_value=False):
            result = settings.doctor(self.home, self.manifest)
        self.assertFalse(result['healthy'])
        self.assertTrue(any('Plugin availability not confirmed' in message for message in result['warnings']))
        self.assertFalse(self.config.parent.exists())

    def test_unrecognized_target_value_is_not_echoed_in_plan(self):
        self.write('model="https://private.invalid/secret"\n')
        with patch.object(settings, 'app_running', return_value=False):
            result = settings.plan(self.home, self.manifest)
        self.assertNotIn('private.invalid', json.dumps(result))
        self.assertEqual(next(c for c in result['changes'] if c['key']=='model')['before'], '<unrecognized configured value>')


    def test_dotted_parent_is_extended_without_redefining_table(self):
        self.write('desktop.appearanceTheme="light"\nmodel="old"\n')
        with patch.object(settings, 'app_running', return_value=False):
            settings.apply(self.home, self.manifest)
        self.assertNotIn('[desktop]', self.config.read_text())
        self.assertEqual(settings.capture(self.home)['desktop'], self.desired['desktop'])

    def test_cli_statuses_support_install_preflight(self):
        with patch.object(settings, 'app_running', return_value=True), patch('sys.stdout', new_callable=io.StringIO):
            base = ['--home', str(self.home), '--settings', str(self.manifest)]
            self.assertEqual(settings.main(['plan'] + base), 2)
            self.assertEqual(settings.main(['apply'] + base), 2)
            self.assertEqual(settings.main(['doctor'] + base), 1)
        self.assertFalse(self.config.parent.exists())

    def test_preserve_multiline_unselected_content_during_apply(self):
        original = 'developer_instructions="""\n[desktop]\nappearanceTheme="secret theme"\nmodel="embedded"\n"""\n'
        self.write(original)
        with patch.object(settings, 'app_running', return_value=False):
            result = settings.apply(self.home, self.manifest)
        self.assertTrue(self.config.read_text().startswith(original))
        self.assertNotIn('secret theme', json.dumps(result))
        self.assertEqual(settings.capture(self.home)['config'], self.desired['config'])


    def test_capture_cannot_replace_active_configuration(self):
        self.write('model="gpt-test"\n')
        before = self.snapshot()
        with self.assertRaises(ValueError):
            settings.capture(self.home, self.config)
        self.assertEqual(before, self.snapshot())

    def test_windows_process_detection_uses_names_without_accounts(self):
        completed = settings.subprocess.CompletedProcess([], 0, '"ChatGPT.exe","123","Console","1","1024 K"\n', '')
        with patch.object(settings.os, 'name', 'nt'), patch.object(settings.subprocess, 'run', return_value=completed) as run:
            self.assertTrue(settings.app_running())
            self.assertEqual(run.call_args.args[0], ['tasklist', '/FO', 'CSV', '/NH'])


    def test_table_scalar_type_conflicts_block_before_any_writes(self):
        cases = [
            ('[model]\n', {'config': {'model': 'gpt-test'}}),
            ('[model.nested]\n', {'config': {'model': 'gpt-test'}}),
            ('[desktop.appearanceTheme]\n', {'desktop': {'appearanceTheme': 'dark'}}),
            ('[[desktop]]\n[desktop.appearanceLightChromeTheme]\naccent="#112233"\n',
             {'desktop': {'appearanceTheme': 'dark'}}),
        ]
        for original, selected in cases:
            with self.subTest(original=original):
                self.write(original)
                self.manifest.write_text(json.dumps(dict(schema_version=1, **selected)))
                before = self.snapshot()
                with patch.object(settings, 'app_running', return_value=False), self.assertRaises(ValueError):
                    settings.apply(self.home, self.manifest)
                self.assertEqual(before, self.snapshot())

    def test_array_descendant_cannot_masquerade_as_matching_setting(self):
        self.write('[[desktop]]\n[desktop.appearanceLightChromeTheme]\naccent="#112233"\n')
        self.manifest.write_text(json.dumps({'schema_version': 1, 'desktop': {
            'appearanceLightChromeTheme': {'accent': '#112233'}}}))
        before = self.snapshot()
        with self.assertRaises(ValueError):
            settings.plan(self.home, self.manifest)
        self.assertEqual(before, self.snapshot())
        self.assertEqual(settings.capture(self.home)['desktop'], {})

    def test_unrelated_array_tables_and_descendants_are_preserved(self):
        original = ('model="old"\n[[other]]\nname="one"\n'
                    '[other.child]\nvalue="keep one"\n[[other]]\n'
                    'name="two"\n[other.child]\nvalue="keep two"\n')
        merged, changes = settings.merge_config(original, {('model',): 'gpt-test'})
        self.assertEqual(merged, original.replace('model="old"', 'model="gpt-test"'))
        self.assertEqual(len(changes), 1)


if __name__ == '__main__':
    unittest.main()
