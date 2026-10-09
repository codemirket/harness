"""Retained feedback fixtures must execute and expose independent outcomes."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]


class AssuranceExampleTests(unittest.TestCase):
    def test_local_http_sensor_rejects_false_indexability_and_tracks_changed_bytes(self):
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'seo'
            result=subprocess.run([sys.executable,str(ROOT/'examples/assurance/seo.py'),
                                   '--output',str(output)],capture_output=True,text=True,timeout=20)
            self.assertEqual(result.returncode,0,result.stderr)
            report=json.loads((output/'report.json').read_text())
            self.assertEqual(len(report['findings']),8)
            self.assertEqual(report['findings']['/meta-noindex']['local_control_diagnosis'],'noindex')
            self.assertEqual(report['mutation_finding']['local_control_diagnosis'],'no_local_indexing_block_detected')
            self.assertTrue(any(c['result']=='rejected' for c in report['checks']))

    def test_engineering_result_invariants_and_measurement_artifacts(self):
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'engineering.json'
            result=subprocess.run([sys.executable,str(ROOT/'examples/assurance/engineering.py'),
                                   '--output',str(output)],capture_output=True,text=True,timeout=45)
            self.assertEqual(result.returncode,0,result.stderr)
            report=json.loads(output.read_text())
            self.assertTrue(report['benchmark']['result_equality'])
            self.assertEqual(report['benchmark']['rows_final'],70000)
            for samples in report['benchmark']['read_samples_ms'].values():
                self.assertEqual(len(samples),160)

    def test_staged_copy_and_bounded_overload_preserve_their_contracts(self):
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'systems.json'
            result=subprocess.run([sys.executable,str(ROOT/'examples/assurance/systems.py'),
                                   '--output',str(output)],capture_output=True,text=True,timeout=15)
            self.assertEqual(result.returncode,0,result.stderr)
            report=json.loads(output.read_text())
            self.assertTrue(report['scaffold']['conflict_rejected_without_partial_copy'])
            self.assertEqual(report['queue']['accepted'],report['queue']['completed'])
            self.assertGreater(report['queue']['rejected'],0)
            self.assertTrue(all(report['plan_inputs']['changed_config_state_target_rejected']))
