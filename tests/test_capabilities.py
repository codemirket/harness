"""Capability checks fail on missing/unreviewed guidance, not on runtime absence."""
import json
from pathlib import Path
import tempfile
import unittest
import subprocess
import sys
from lib import capabilities, catalog

ROOT = Path(__file__).resolve().parents[1]

class CapabilityTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name).resolve()
        (self.root/'registry').mkdir()
        (self.root/'skills/lead').mkdir(parents=True)
        self.body = b'---\nname: lead\ndescription: Exercise a real task.\n---\nRead the contract and verify the result.\n'
        (self.root/'skills/lead/SKILL.md').write_bytes(self.body)
        self.row = dict(id='capability',title='Capability',lead='lead',support=[],deliverable='Observable result',failure_probe='Inject failed state',prerequisites='Existing runtime',aliases=['Example Role'],acceptance=['Observe the consumer'])
        self.manifest = dict(schema_version=1,scope='Guidance, not runtime proof',capabilities=[self.row])
        self.entry = dict(id='lead',name='lead',scope='project',delivery='local',path='skills/lead',license_files=[],sha256=catalog.payload_hash({'SKILL.md':self.body}))
        self.index = dict(schema_version=1,skills=[self.entry],sources={},profiles={})
        self.write()

    def write(self):
        (self.root/'registry/capabilities.json').write_text(json.dumps(self.manifest))
        (self.root/'registry/catalog.json').write_text(json.dumps(self.index))
        (self.root/'registry/harness.json').write_text(json.dumps({'global_skills':[]}))

    def test_source_integrity_never_claims_runtime_or_quality(self):
        report=capabilities.check(self.root)
        self.assertEqual(report['status'],'valid')
        self.assertEqual(report['skills']['lead']['status'],'source_verified')
        description=capabilities.describe('capability',self.root)
        self.assertEqual(description['runtime_status'],'not_checked')
        self.assertEqual(description['project_lead'],['lead'])

    def test_changed_source_fails_without_refreshing_pin(self):
        (self.root/'skills/lead/SKILL.md').write_bytes(self.body+b'Changed behavior\n')
        result=capabilities.check(self.root)
        self.assertEqual(result['status'],'invalid')
        self.assertIn('hash',result['failures'][0]['error'])

    def test_real_global_link_schema_needs_no_license_files(self):
        self.entry.update(scope='global', delivery='global-link')
        self.entry.pop('license_files')
        self.entry.pop('sha256')
        self.write()
        (self.root/'registry/harness.json').write_text(json.dumps({'global_skills':['lead']}))
        self.assertEqual(capabilities.check(self.root)['status'],'valid')
        self.assertEqual(capabilities.describe('capability',self.root)['project_lead'],[])

    def test_missing_manual_and_single_host_entries_cannot_count(self):
        for patch in ({'scope':'manual'},{'agents':['codex']},{'delivery':'upstream'}):
            with self.subTest(patch=patch):
                self.index['skills']=[dict(self.entry,**patch)];self.write()
                self.assertEqual(capabilities.check(self.root)['status'],'invalid')
        self.index['skills']=[];self.write()
        self.assertEqual(capabilities.check(self.root)['status'],'invalid')

    def test_missing_support_and_global_registration_are_errors(self):
        self.row['support']=['missing'];self.write()
        self.assertEqual(capabilities.check(self.root)['failures'][0]['skill'],'missing')
        self.row['support']=[];self.entry.update(scope='global',delivery='global-link');self.write()
        self.assertEqual(capabilities.check(self.root)['status'],'invalid')
        (self.root/'registry/harness.json').write_text('{"global_skills":["lead"]}')
        self.assertEqual(capabilities.describe('capability',self.root)['project_lead'],[])
        self.assertEqual(capabilities.check(self.root)['status'],'valid')

    def test_contract_rejects_empty_outcome_duplicate_and_path_traversal(self):
        for field,value in [('deliverable',''),('failure_probe',' '),('support',['lead']),('id','../escape')]:
            previous=self.row[field];self.row[field]=value;self.write()
            with self.assertRaises(ValueError):capabilities.contracts(self.root)
            self.row[field]=previous
        self.manifest['capabilities'].append(dict(self.row));self.write()
        with self.assertRaises(ValueError):capabilities.contracts(self.root)

    def test_shipped_contracts_cover_requested_domains_and_generated_guide(self):
        expected={'frontend-engineering','backend-engineering','brand-guidelines','web-design','mobile-design','seo','deep-research','interface-qa','illustration','visual-assets','animation','project-review','planning','product-management','database','systems-engineering','devops-infrastructure','marketing','executive-strategy','data-analysis','project-scaffolding','design-revision','optimization'}
        expected |= {'financial-analysis','technical-leadership','market-analysis','venture-validation',
                     'localization','spreadsheet-analysis','mobile-engineering','it-operations',
                     'documentation','graphic-design'}
        self.assertEqual({r['id'] for r in capabilities.contracts(ROOT)['capabilities']},expected)
        self.assertEqual((ROOT/'docs/capabilities.md').read_text(),capabilities.markdown(ROOT))
        self.assertEqual(capabilities.check(ROOT)['status'],'valid')

    def test_new_specialists_install_on_both_targets_without_replacing_user_work(self):
        identifiers=['backend-engineering','brand-guidelines','mobile-design',
                     'systems-engineering','infrastructure-engineering',
                     'project-scaffolding','performance-engineering','executive-strategy']
        project=self.root/'project';project.mkdir()
        sentinel=project/'existing.txt';sentinel.write_text('User draft\n')
        cli=[sys.executable,str(ROOT/'ai.py'),'project']
        additions=[]
        for identifier in identifiers:additions+=['--skill',identifier]
        for action,extra in [('init',['--target','all']+additions),('sync',[]),('doctor',[])]:
            result=subprocess.run(cli+[action,'--project',str(project)]+extra,
                                  text=True,capture_output=True,timeout=30)
            self.assertEqual(result.returncode,0,result.stdout+result.stderr)
        self.assertEqual(sentinel.read_text(),'User draft\n')
        for target in ['.agents','.claude']:
            for identifier in identifiers:
                installed=project/target/'skills'/identifier
                self.assertEqual((installed/'SKILL.md').read_bytes(),
                                 (ROOT/'skills'/identifier/'SKILL.md').read_bytes())
                self.assertTrue(list((installed/'references').glob('*.md')))
