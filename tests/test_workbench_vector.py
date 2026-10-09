"""Real multi-size vector rendering and rejection of malformed/active sources."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from lib import workbench

ROOT=Path(__file__).resolve().parents[1]

class VectorWorkbenchTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tools=workbench.load_tools()
        cls.env=workbench.environment(cls.tools)
        if not cls.tools['node']:
            raise unittest.SkipTest('Node unavailable; vector renderer not tested')
        probe=workbench.probe([cls.tools['node'],'-e',"require(require('path').resolve(process.env.HARNESS_NODE_MODULES||'node_modules','sharp'))"],cls.env)
        if not probe['available']:
            raise unittest.SkipTest('sharp unavailable; vector renderer not tested')

    def run_vector(self, source, output):
        return subprocess.run([self.tools['node'],str(ROOT/'scripts/workbench/vector.mjs'),str(source),'--output',str(output)],env=self.env,capture_output=True,text=True,timeout=30)

    def test_render_retains_editable_source_and_pngs(self):
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp).resolve()/'result'
            source=ROOT/'examples/craft-lab/fieldwork.svg'
            result=self.run_vector(source,output)
            self.assertEqual(result.returncode,0,result.stderr)
            report=json.loads((output/'report.json').read_text())
            self.assertEqual((output/'source.svg').read_bytes(),source.read_bytes())
            self.assertTrue(report['review_required'])
            for item in report['artifacts']:
                data=(output/item['path']).read_bytes()
                self.assertEqual(hashlib.sha256(data).hexdigest(),item['sha256'])
                if item['path'].endswith('.png'):
                    self.assertTrue(data.startswith(b'\x89PNG\r\n\x1a\n'))
            self.assertNotEqual(self.run_vector(source,output).returncode,0)

    def test_rejects_active_external_duplicate_and_invalid_geometry(self):
        cases=['<script>alert(1)</script>','<image href="https://example.com/a.png"/>','<g id="x"/><path id="x"/>','<rect style="fill:red"/>']
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp).resolve()
            for index, body in enumerate(cases):
                source=root/f'{index}.svg'
                source.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">'+body+'</svg>')
                result=self.run_vector(source,root/f'out-{index}')
                self.assertNotEqual(result.returncode,0)
                self.assertFalse((root/f'out-{index}').exists())
