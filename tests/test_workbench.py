"""Behavioral checks for local tool configuration and Markdown evidence."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from lib import workbench

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('markdown_check', ROOT / 'scripts/workbench/markdown.py')
markdown = importlib.util.module_from_spec(spec)
spec.loader.exec_module(markdown)


class WorkbenchTests(unittest.TestCase):
    def test_dispatch_preserves_option_order(self):
        with patch.object(workbench, 'load_tools', return_value={'node':'node'}), patch.object(workbench.subprocess, 'Popen') as run:
            run.return_value.wait.return_value = 0
            self.assertEqual(workbench.main(['browser','--scenario','a.json','--output','evidence']), 0)
            self.assertEqual(run.call_args.args[0][-4:], ['--scenario','a.json','--output','evidence'])

    def test_configuration_validates_and_preserves_existing_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp).resolve() / 'toolchain.json'
            with patch.dict(os.environ, {'HARNESS_TOOLCHAIN':str(path)}):
                workbench.configure({'python':os.sys.executable})
                workbench.configure({'node_modules':str(path.parent)})
                self.assertEqual(json.loads(path.read_text())['python'], str(Path(os.sys.executable).resolve()))
                with self.assertRaises(ValueError):
                    workbench.configure({'node':'/missing/definitely-not-installed'})
                path.write_text('{"schema_version":999}')
                with self.assertRaises(ValueError):
                    workbench.load_tools()

    def test_markdown_real_destinations_and_code_heading(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root/'guide.md').write_text('# Use `workbench`\n\n## Example\n')
            source=root/'readme.md'
            source.write_text('# Start\n[Guide](guide.md#use-workbench)\n[Example][ex]\n\n[ex]: guide.md#example\n```sh\n[ignored](absent)\n```\n')
            self.assertTrue(markdown.inspect(source)['checks_passed'])
            source.write_text('# Start\n### Jump\n[bad](guide.md#absent)\n[missing](missing.md)\n[ref][absent]\n```\n')
            report=markdown.inspect(source)
            self.assertFalse(report['checks_passed'])
            self.assertEqual({x['kind'] for x in report['issues']}, {'heading_level_jump','unresolved_anchor','missing_file','missing_reference','unclosed_fence'})

    def test_external_links_are_not_claimed_verified(self):
        with tempfile.TemporaryDirectory() as temp:
            source=Path(temp)/'readme.md'
            source.write_text('# Title\n[Link](https://example.invalid)\n')
            report=markdown.inspect(source)
            self.assertEqual(report['links'][0]['status'],'external_not_checked')
            self.assertTrue(report['review_required'])
