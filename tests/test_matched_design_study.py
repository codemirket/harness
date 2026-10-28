"""Adversarial controls; fake CLI tests are not model or visual qualification."""
from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import MagicMock, patch

SPEC = importlib.util.spec_from_file_location("matched_study", Path(__file__).with_name("matched_design_study.py"))
assert SPEC and SPEC.loader
S = importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(S)


class MatchedStudyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name)
        self.root = self.base / "study"
        self.seal = self.base / "lead-seal.json"
        self.ready_path = self.base / "ready.json"
        self.font = self.base / "font.ttf"; self.font.write_bytes(b"fixture font")
        self.ready = {}
        for key in S.RUNTIME_KEYS:
            file = self.base / (key + ".bin")
            file.write_text("fixture " + key)
            self.ready[key] = str(file)
        adapter = self.base / "adapter-tree"; adapter.mkdir()
        browser = self.base / "browser-tree"; browser.mkdir()
        for key, parent in [("adapter", adapter), ("design_adapter", adapter), ("chromium", browser)]:
            file = parent / key; file.write_text("fixture " + key); self.ready[key] = str(file)
        self.ready.update(adapter_root=str(adapter), browser_root=str(browser), model="gpt-6-astra", reasoning_effort="high", approval_review="auto_review")
        auth = self.base / "auth.json"; auth.write_text('secret-test-credential'); auth.chmod(0o600)
        self.ready["auth_file"] = str(auth)
        Path(self.ready["codex"]).write_text("#!/usr/bin/env python3\nfrom pathlib import Path\nimport json\nPath('illustration.svg').write_text('<svg/>')\np=Path('.evidence'); p.mkdir(exist_ok=True)\n(p/'preview.png').write_bytes(b'unit-fixture-not-pixels')\nprint(json.dumps({'type':'turn.completed','usage':{'output_tokens':1}}))\n")
        Path(self.ready["codex"]).chmod(0o700)
        self.ready_path.write_text(json.dumps(self.ready))
        self.seal_sha = S.prepare(self.root, self.ready_path, self.font, self.seal, calibration=True)
        self.protocol = S.read(self.root / "frozen/protocol.json")
        self.a, self.b = self.protocol["cells"]

    def tearDown(self):
        self.temp.cleanup()

    def run_cell(self, cell=None):
        return S.run(self.root, self.seal, self.seal_sha, self.ready_path, (cell or self.a)["id"])

    def test_pair_has_equal_prompt_contract_assets_tools_and_only_skill_difference(self):
        a, b = self.a, self.b
        remove_skills = lambda files: {k: v for k, v in files.items() if not k.startswith('.agents/')}
        self.assertEqual(remove_skills(a["inputs"]), remove_skills(b["inputs"]))
        self.assertEqual(a["prompt_sha256"], b["prompt_sha256"])
        self.assertNotEqual(a["inputs"], b["inputs"])
        configs = []
        for cell in [a, b]:
            trial = self.root / "trials" / cell["id"]
            configs.append((trial / "codex-home/config.toml").read_text().replace(str(trial), '<trial>'))
            self.assertFalse((trial / "codex-home/auth.json").exists())
        self.assertEqual(*configs)
        self.assertNotIn('secret-test-credential', ''.join(p.read_text(errors='replace') for p in (self.root/'frozen').rglob('*') if p.is_file()))

    def test_acceptance_sample_freezes_four_matched_cells_and_one_shared_repair(self):
        root, seal = self.base / 'sample', self.base / 'sample-seal.json'
        S.prepare(root, self.ready_path, self.font, seal, acceptance_sample=True)
        protocol = S.read(root / 'frozen/protocol.json')
        self.assertEqual(protocol['kind'], 'acceptance-sample')
        self.assertEqual(len(protocol['cells']), 4)
        self.assertEqual(protocol['limits']['max_repairs'], 1)
        self.assertEqual(protocol['limits']['wall_seconds'], 1800)
        for task in protocol['cases']:
            a, b = [c for c in protocol['cells'] if c['task'] == task]
            strip = lambda c: {k: v for k, v in c['inputs'].items() if not k.startswith('.agents/')}
            self.assertEqual(strip(a), strip(b))
            self.assertEqual(a['prompt_sha256'], b['prompt_sha256'])
            text = (root / 'trials' / a['id'] / 'project/PRODUCTION.md').read_text()
            self.assertIn('one 180-second repair turn across all four cells', text)
        with S.locked(root):
            a, b = protocol['cells'][:2]
            for cell in (a, b):
                S.reserve(root, protocol, cell['id'], 'initial')
                S.settle(root, cell['id'], 'initial', 1, {})
            S.reserve(root, protocol, a['id'], 'repair')
            S.settle(root, a['id'], 'repair', 1, {})
            with self.assertRaisesRegex(SystemExit, 'shared repair budget exhausted'):
                S.reserve(root, protocol, b['id'], 'repair')

    def test_prompt_and_production_agree_on_editable_output_location(self):
        trial = self.root/'trials'/self.a['id']
        self.assertIn('editable files at the project root', (trial/'prompt.txt').read_text())
        self.assertIn('Required deliverables at the project root: illustration.svg',
                      (trial/'project/PRODUCTION.md').read_text())
        result = self.run_cell()
        self.assertEqual(result['project_check']['missing_outputs'], [])

    def test_external_seal_detects_rewritten_protocol_and_seal(self):
        p = self.root / "frozen/protocol.json"; p.write_text(p.read_text()+' ')
        with self.assertRaisesRegex(SystemExit, 'frozen study changed'):
            S.verify(self.root, self.seal, self.seal_sha, self.ready_path)
        self.seal.write_text(json.dumps({'root':str(self.root),'frozen':S.tree(self.root/'frozen')}))
        with self.assertRaisesRegex(SystemExit, 'external seal changed'):
            S.verify(self.root, self.seal, self.seal_sha, self.ready_path)

    def test_runtime_closure_and_ready_model_changes_refuse_before_launch(self):
        (Path(self.ready['adapter_root'])/'extra-runtime.js').write_text('drift')
        with self.assertRaisesRegex(SystemExit, 'runtime/settings changed'):
            self.run_cell()
        self.assertEqual(S.read(self.root/'budget-ledger.json')['runs'], {})
        (Path(self.ready['adapter_root'])/'extra-runtime.js').unlink()
        self.ready['reasoning_effort']='low'; self.ready_path.write_text(json.dumps(self.ready))
        with self.assertRaises(SystemExit): self.run_cell()

    def test_runtime_parent_traversal_cannot_escape_frozen_tree(self):
        ready=copy.deepcopy(self.ready)
        outside=self.base/'outside.js';outside.write_text('outside closure')
        ready['adapter']=str(Path(ready['adapter_root'])/'..'/'outside.js')
        with self.assertRaisesRegex(SystemExit,'outside its frozen runtime tree'): S.bindings(ready)

    def test_skill_prompt_config_and_no_skill_injection_are_protected(self):
        for cell, relative in [(self.a,'prompt.txt'),(self.a,'codex-home/config.toml'),(self.a,'project/.agents/skills/vector-art/SKILL.md')]:
            path=self.root/'trials'/cell['id']/relative; original=path.read_bytes(); path.write_bytes(original+b'changed')
            with self.assertRaises(SystemExit): self.run_cell(cell)
            path.write_bytes(original)
        project=self.root/'trials'/self.b['id']/'project'
        (project/'.agents').mkdir()
        with self.assertRaisesRegex(SystemExit, 'out-of-scope'):
            S.inspect_project(self.root,self.b,self.protocol['cases'][self.b['task']])

    def test_symlink_and_empty_unrequested_directory_refuse(self):
        project=self.root/'trials'/self.a['id']/'project'
        link=project/'alias';link.symlink_to(self.font)
        with self.assertRaisesRegex(SystemExit,'symlink'): self.run_cell()
        link.unlink();(project/'unrequested').mkdir()
        with self.assertRaisesRegex(SystemExit,'out-of-scope'): self.run_cell()

    def test_fake_cli_actual_subprocess_retains_outputs_and_removes_auth(self):
        result=self.run_cell()
        self.assertTrue(result['author_completed'])
        self.assertEqual(result['outcome'],'awaiting_independent_review')
        trial=self.root/'trials'/self.a['id']
        self.assertFalse((trial/'codex-home/auth.json').exists())
        self.assertTrue((trial/'initial-project-snapshot/illustration.svg').is_file())
        ledger=S.read(self.root/'budget-ledger.json')
        self.assertEqual(ledger['model_seconds_reserved'],0)
        self.assertGreater(ledger['model_seconds_used'],0)
        self.assertIn('--approve-for-me',result['command'])
        self.assertNotIn('--dangerously-bypass-approvals-and-sandbox',result['command'])
        with self.assertRaises(SystemExit): self.run_cell()

    def test_missing_output_never_becomes_quality_pass(self):
        def fake(*args): return dict(exit_code=0,turn_completed=True,timed_out=False,approval_signals=[],blocked_signals=[])
        with patch.object(S.P,'run_model',side_effect=fake): result=self.run_cell()
        self.assertEqual(result['project_check']['missing_outputs'],['illustration.svg'])
        self.assertEqual(result['outcome'],'awaiting_independent_review')
        self.assertNotIn('passed',result)

    def test_exit_zero_without_native_turn_completion_is_not_author_completion(self):
        with patch.object(S.P,'run_model',return_value=dict(exit_code=0,timed_out=False,approval_signals=[],blocked_signals=[])):
            result=self.run_cell()
        self.assertEqual(result['outcome'],'failed_or_blocked')

    def test_denial_unavailable_timeout_and_launch_error_are_retained_and_no_retry(self):
        for name,result in [('denied',dict(exit_code=0,timed_out=False,approval_signals=['approval denied'],blocked_signals=[])),('unavailable',dict(exit_code=1,timed_out=False,approval_signals=[],blocked_signals=['tool unavailable'])),('timeout',dict(exit_code=-15,timed_out=True,approval_signals=[],blocked_signals=[]))]:
            with self.subTest(name=name):
                # Fresh study instance for distinct unit cases, never a model retry.
                ledger={'started_at':None,'deadline_at':None,'model_seconds_used':0,'model_seconds_reserved':0,'repairs_used':0,'runs':{}}
                S.P.write_json(self.root/'budget-ledger.json',ledger)
                trial=self.root/'trials'/self.a['id']
                for label in ['before-initial','initial']:
                    import shutil
                    shutil.rmtree(trial/(label+'-project-snapshot'),ignore_errors=True)
                with patch.object(S.P,'run_model',return_value=result.copy()): observed=self.run_cell()
                self.assertEqual(observed['outcome'],'failed_or_blocked')
                self.assertFalse((trial/'codex-home/auth.json').exists())
                with self.assertRaises(SystemExit): self.run_cell()

    def test_launch_exception_settles_budget_and_cleans_credential(self):
        with patch.object(S.P,'run_model',side_effect=OSError('launch failed')): result=self.run_cell()
        self.assertIn('launch failed',result['runner_error'])
        self.assertFalse((self.root/'trials'/self.a['id']/'codex-home/auth.json').exists())
        self.assertEqual(S.read(self.root/'budget-ledger.json')['model_seconds_reserved'],0)
        with self.assertRaises(SystemExit): self.run_cell()

    def test_dangling_credential_link_is_neither_followed_nor_deleted(self):
        auth=self.root/'trials'/self.a['id']/'codex-home/auth.json'
        outside=self.base/'must-not-be-created'
        auth.symlink_to(outside)
        with patch.object(S.P,'run_model') as launch:
            result=self.run_cell()
        launch.assert_not_called()
        self.assertEqual(result['outcome'],'failed_or_blocked')
        self.assertFalse(outside.exists())
        self.assertTrue(auth.is_symlink())

    def test_credential_cleanup_uses_original_directory_after_rename(self):
        trial=self.root/'trials'/self.a['id']; moved=trial/'moved-home'
        def interrupted(*args):
            (trial/'codex-home').rename(moved)
            (trial/'codex-home').symlink_to(self.base,target_is_directory=True)
            raise OSError('directory changed')
        with patch.object(S.P,'run_model',side_effect=interrupted): result=self.run_cell()
        self.assertEqual(result['outcome'],'failed_or_blocked')
        self.assertFalse((moved/'auth.json').exists())
        self.assertEqual((self.base/'auth.json').read_text(),'secret-test-credential')

    def test_native_interrupt_stops_process_group_and_retains_partial_trace(self):
        trial=self.root/'trials'/self.a['id']
        process=MagicMock(pid=987654,returncode=-15)
        process.communicate.side_effect=[KeyboardInterrupt(),(b'{"type":"partial"}\n',b'interrupted')]
        with patch.object(S.P.subprocess,'Popen',return_value=process), patch.object(S.P.os,'killpg') as kill:
            result=S.P.run_model(trial,'prompt','initial',self.ready,'skill',1)
        self.assertTrue(result['interrupted'])
        self.assertFalse(result['turn_completed'])
        kill.assert_called_once_with(987654,S.P.signal.SIGTERM)
        self.assertEqual((trial/'initial-events.jsonl').read_text(),'{"type":"partial"}\n')

    def test_new_cell_out_of_order_and_duplicate_phase_do_not_reset_budget(self):
        for cell in ['new-specimen',self.b['id']]:
            with self.assertRaises(SystemExit): S.reserve(self.root,self.protocol,cell,'initial')
        S.reserve(self.root,self.protocol,self.a['id'],'initial')
        before=(self.root/'budget-ledger.json').read_bytes()
        with self.assertRaises(SystemExit): S.reserve(self.root,self.protocol,self.a['id'],'initial')
        self.assertEqual(before,(self.root/'budget-ledger.json').read_bytes())

    def test_shared_wall_and_model_budgets_refuse_full_turn(self):
        path=self.root/'budget-ledger.json'; ledger=S.read(path)
        ledger.update(started_at=time.time()-590,deadline_at=time.time()+10)
        ledger['deadline_at']=ledger['started_at']+self.protocol['limits']['wall_seconds']
        S.P.write_json(path,ledger)
        with self.assertRaisesRegex(SystemExit,'wall window'): S.reserve(self.root,self.protocol,self.a['id'],'initial')
        ledger.update(started_at=None,deadline_at=None,model_seconds_used=151)
        S.P.write_json(path,ledger)
        with self.assertRaisesRegex(SystemExit,'model budget'): S.reserve(self.root,self.protocol,self.a['id'],'initial')

    def test_shutdown_grace_is_reserved_and_overrun_is_not_refunded(self):
        seconds=S.reserve(self.root,self.protocol,self.a['id'],'initial')
        ledger=S.read(self.root/'budget-ledger.json')
        self.assertEqual(ledger['model_seconds_reserved'],seconds+S.SHUTDOWN_RESERVE_SECONDS)
        S.settle(self.root,self.a['id'],'initial',280,{'author_completed':False})
        ledger=S.read(self.root/'budget-ledger.json')
        self.assertEqual(ledger['model_seconds_used'],280)
        self.assertEqual(ledger['model_seconds_reserved'],0)
        with self.assertRaisesRegex(SystemExit,'model budget'): S.reserve(self.root,self.protocol,self.b['id'],'initial')

    def test_calibration_admits_both_full_turns_including_shutdown_reservations(self):
        for cell in (self.a,self.b):
            S.reserve(self.root,self.protocol,cell['id'],'initial')
            S.settle(self.root,cell['id'],'initial',120,{'author_completed':True})
        self.assertEqual(S.read(self.root/'budget-ledger.json')['model_seconds_used'],240)

    def test_review_pack_masks_guidance_logs_claims_and_is_immutable(self):
        self.run_cell()
        dest=S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,self.a['id'])
        files=set(S.tree(dest))
        self.assertEqual(files,{'BRIEF.md','PRODUCTION.md','RUBRIC.md','NotoSans.ttf','illustration.svg','.evidence/preview.png','review-template.json'})
        self.assertNotIn('condition',S.read(dest/'review-template.json'))
        with self.assertRaisesRegex(SystemExit,'already exists'):
            S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,self.a['id'])

    def test_neutral_pack_keeps_editable_svg_asset_but_not_author_reports(self):
        self.run_cell()
        snapshot = self.root/'trials'/self.a['id']/'initial-project-snapshot'
        evidence = snapshot/'.evidence'
        (evidence/'library-art.svg').write_text('<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0h1"/></svg>')
        (evidence/'review.md').write_text('Author says verified; condition labels must stay masked')
        (evidence/'checks.json').write_text('{"author_claim":"passed"}')
        S.P.write_json(snapshot.parent/'initial-project-manifest.json',S.tree(snapshot))
        dest = S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,self.a['id'])
        self.assertEqual((dest/'.evidence/library-art.svg').read_bytes(),(evidence/'library-art.svg').read_bytes())
        self.assertFalse((dest/'.evidence/review.md').exists())
        self.assertFalse((dest/'.evidence/checks.json').exists())

    def test_review_pack_refuses_changed_snapshot_and_expired_window(self):
        self.run_cell()
        file=self.root/'trials'/self.a['id']/'initial-project-snapshot/illustration.svg'; file.write_text('changed')
        with self.assertRaisesRegex(SystemExit,'snapshot changed'):
            S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,self.a['id'])
        ledger=S.read(self.root/'budget-ledger.json');ledger['deadline_at']=time.time()-1;S.P.write_json(self.root/'budget-ledger.json',ledger)
        with self.assertRaisesRegex(SystemExit,'window expired'):
            S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,self.a['id'])

    def test_review_allocation_requires_all_results_review_and_bound_criterion(self):
        self.run_cell(); self.run_cell(self.b)
        (self.root/'reviews').mkdir()
        for i,cell in enumerate([self.a,self.b]):
            S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,cell['id'])
            snap=self.root/'trials'/cell['id']/'initial-project-snapshot'
            review={'artifact_id':cell['id'],'phase':'initial','manifest_sha256':S.canonical(S.tree(snap)),'rubric_sha256':S.canonical(self.protocol['cases'][cell['task']]['rubric']),'status':'reviewed','reviewed_by':'unit reviewer attestation','reviewed_at':time.time(),'inspected_artifacts':['.evidence/preview.png'],'findings':[{'id':'f','severity':'major' if i==0 else 'blocking','artifact_path':'.evidence/preview.png','location':'circle','consequence':'wrong saved color'}]}
            S.P.write_json(self.root/'reviews'/(cell['id']+'.json'),review)
        with self.assertRaisesRegex(SystemExit,'not next'): S.repair_review(self.root,self.protocol,self.a['id'])
        self.assertEqual(S.repair_review(self.root,self.protocol,self.b['id'])['artifact_id'],self.b['id'])
        path=self.root/'reviews'/(self.b['id']+'.json');review=S.read(path);review['rubric_sha256']='0'*64;S.P.write_json(path,review)
        with self.assertRaisesRegex(SystemExit,'neutral pack binding'): S.repair_review(self.root,self.protocol,self.b['id'])

    def test_review_cannot_bypass_or_modify_neutral_pack(self):
        self.run_cell()
        (self.root/'reviews').mkdir()
        path=self.root/'reviews'/(self.a['id']+'.json')
        S.P.write_json(path,{})
        with self.assertRaisesRegex(SystemExit,'retained neutral pack'): S.validate_review(self.root,self.protocol,self.a,'initial')
        dest=S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,self.a['id'])
        (dest/'illustration.svg').write_text('changed pack')
        with self.assertRaisesRegex(SystemExit,'neutral review pack changed'): S.validate_review(self.root,self.protocol,self.a,'initial')

    def test_finalize_requires_actual_review_record_and_closes_remaining_cells(self):
        self.run_cell()
        with self.assertRaisesRegex(SystemExit,'collect results and neutral reviews'):
            S.finalize(self.root,self.seal,self.seal_sha,self.ready_path)
        dest=S.review_pack(self.root,self.seal,self.seal_sha,self.ready_path,self.a['id'])
        review=S.read(dest/'review-template.json')
        review.update(status='reviewed',reviewed_by='unit reviewer attestation',reviewed_at=time.time(),inspected_artifacts=['.evidence/preview.png'])
        (self.root/'reviews').mkdir()
        S.P.write_json(self.root/'reviews'/(self.a['id']+'.json'),review)
        result=S.finalize(self.root,self.seal,self.seal_sha,self.ready_path)
        self.assertEqual(result['status'],'recorded_not_quality_certified')
        self.assertEqual(result['cells'][1]['status'],'not_run')
        with self.assertRaisesRegex(SystemExit,'already finalized'): self.run_cell(self.b)

    def test_calibration_cannot_charge_repairs(self):
        self.run_cell()
        with self.assertRaisesRegex(SystemExit,'repair budget'): S.reserve(self.root,self.protocol,self.a['id'],'repair')


if __name__ == '__main__': unittest.main()
