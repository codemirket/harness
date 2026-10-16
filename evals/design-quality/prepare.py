#!/usr/bin/env python3
"""Stage fixed host trials and collect role-eval v1 records. Never launch a model."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / 'evals/design-quality'
OWNER = ROOT / '.mirket/implementation/design-quality-2026-10-15'
EXP = OWNER / 'experiment'
ARMS = ['baseline', 'improved']


def select_experiment_root(value):
    global OWNER, EXP
    if value is None:
        return
    parts = value.split('/')
    if not value or Path(value).is_absolute() or any(x in ('', '.', '..') for x in parts):
        raise ValueError('--experiment-root must use normal project-relative components')
    candidate = ROOT
    for part in parts:
        candidate /= part
        if candidate.is_symlink():
            raise ValueError('--experiment-root refuses symlink components')
    if candidate.exists() and not candidate.is_dir():
        raise ValueError('--experiment-root must name a directory')
    EXP = candidate
    OWNER = EXP.parent


def read(p):
    if not p.is_file():
        raise ValueError(f'Missing required caller-owned input: {p.relative_to(ROOT)}; see evals/design-quality/reuse.md')
    return json.loads(p.read_text())


def write(p, v):
    p = p.absolute()
    if ROOT not in p.parents:
        raise ValueError('Expected project-owned output')
    for cursor in [p, *p.parents]:
        if cursor == ROOT:
            break
        if cursor.is_symlink():
            raise ValueError('Symlink output component refused')
    p.parent.mkdir(parents=True, exist_ok=True)
    if p.exists():
        raise ValueError(f'Refusing overwrite: {p.relative_to(ROOT)}')
    p.write_text(json.dumps(v, indent=2, ensure_ascii=False) + '\n')


def binding(p):
    p = p.absolute()
    if ROOT not in p.parents or p.is_symlink() or not p.is_file():
        raise ValueError('Expected project-owned regular file')
    for parent in p.parents:
        if parent == ROOT:
            break
        if parent.is_symlink():
            raise ValueError('Symlink component refused')
    return {'path': str(p.relative_to(ROOT)), 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}


def verify(b):
    p = Path(b['path'])
    if p.is_absolute() or any(x in ('.', '..') for x in p.parts):
        raise ValueError('Expected normal project-relative path')
    if binding(ROOT / p) != b:
        raise ValueError(f'Stale evidence: {p}')
    return ROOT / p


def context_manifest(context):
    p = context.resolve(strict=True)
    if ROOT not in p.parents:
        raise ValueError('Context must be project-owned')
    library = read(p / 'skills/design-inspiration/references/design-languages/library.json')
    if len(library.get('instances', [])) != 27:
        raise ValueError('Both arms require the current twenty-seven instances')
    return {'schema_version': 1, 'context_root': str(p.relative_to(ROOT)),
            'files': [binding(x) for x in sorted(p.rglob('*')) if x.is_file()]}


def configuration(arm, manifest):
    return {'schema_version': 1, 'status': 'frozen_for_host_dispatch',
            'host': 'Codex Desktop fresh no-fork host subagent',
            'model': 'Same inherited host model; exact backend identity not exposed',
            'model_settings': {'exact_model': None, 'reasoning_effort': None, 'seed': None,
                               'temperature': None, 'override': False},
            'tools': ['local shell/file tools', 'saved-image inspection', 'existing Playwright and Chrome'],
            'authority': 'Local owned artifacts only; no installs, downloads, external services, commits, delegation or task mutation',
            'harness_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip() + ' + context-bound working bytes',
            'budget': {'wall_minutes': 12, 'tool_calls': 28, 'trials_per_case': 2},
            'guidance_hashes': {'assigned_context_manifest': binding(manifest),
                                'ordinary_context_manifest': binding(EXP / 'manifests/context-baseline.json')},
            'condition': arm,
            'unknown_measurements': ['exact model/version', 'reasoning effort', 'seed', 'temperature', 'tokens', 'cost', 'model latency']}


def initialize():
    dev = read(OWNER / 'dev-dispatch/freeze.json')
    for key in ['template', 'helper', 'input_manifest', 'diagnostic_rubric', 'primary_craft_rubric']:
        verify(dev[key])
    for seal in ['hidden-content-seal.json', 'hidden-probe-seal.json']:
        for b in read(EXP / 'manifests' / seal)['files']:
            verify(b)
    cases = []
    devrubric = read(OWNER / 'development/development-rubrics.json')
    for name in ['A', 'B']:
        guards = devrubric['briefs'][name]['objective_guards']
        rub = EXP / f'development-{name}.rubric.json'
        write(rub, {'schema_version': 1, 'criteria': [g['id'] for g in guards],
                    'assertions': {g['id']: g['expect'] for g in guards},
                    'source': binding(OWNER / 'development/development-rubrics.json'),
                    'human_acceptance': 'not_collected'})
        cases.append({'id': 'DEV_' + name, 'split': 'development',
                      'input': binding(OWNER / f'development/{name}-brief.md'),
                      'rubric': binding(rub), 'criteria': [g['id'] for g in guards]})
    for name in ['Q01', 'Q02', 'Q03', 'Q04']:
        rub = EXP / f'hidden/{name}.rubric.json'
        cases.append({'id': name, 'split': 'held_out', 'input': binding(EXP / f'hidden/{name}.md'),
                      'rubric': binding(rub), 'criteria': read(rub)['criteria']})
    context = OWNER / 'contexts/context-0'
    m = EXP / 'manifests/context-baseline.json'
    write(m, context_manifest(context))
    write(EXP / 'configurations/baseline.json', configuration('baseline', m))
    cells = []
    for n, name, repeat, arm in [(1,'A',1,'baseline'),(2,'B',1,'baseline'),(3,'B',2,'baseline'),(4,'A',2,'baseline'),
                                  (5,'A',1,'improved'),(6,'B',1,'improved'),(7,'B',2,'improved'),(8,'A',2,'improved')]:
        label = f'D{n:02d}'
        old = next((c for c in dev['cells'] if c['label'] == label), None)
        cells.append({'label': label, 'case_id': 'DEV_' + name, 'split': 'development', 'trial': repeat,
                      'configuration': arm, 'raw_input': binding(OWNER / f'development/{name}-brief.md'),
                      'prompt_path': old['prompt']['path'] if old else str((EXP / f'dispatch/{label}.md').relative_to(ROOT)),
                      'prompt_binding': old['prompt'] if old else None,
                      'artifact_dir': str((OWNER / f'outputs/{label}').relative_to(ROOT)),
                      'observation_dir': str((EXP / f'observations/{label}').relative_to(ROOT)),
                      'receipt': str((EXP / f'receipts/{label}.json').relative_to(ROOT)),
                      'grade': str((EXP / f'guardrail-grades/{label}.json').relative_to(ROOT))})
    order = [('Q03','baseline',1),('Q03','improved',1),('Q01','improved',1),('Q01','baseline',1),
             ('Q04','baseline',1),('Q04','improved',1),('Q02','improved',1),('Q02','baseline',1),
             ('Q03','improved',2),('Q03','baseline',2),('Q01','baseline',2),('Q01','improved',2),
             ('Q04','improved',2),('Q04','baseline',2),('Q02','baseline',2),('Q02','improved',2)]
    for n, (name, arm, repeat) in enumerate(order, 1):
        label = f'H{n:02d}'
        cells.append({'label': label, 'case_id': name, 'split': 'held_out', 'trial': repeat, 'configuration': arm,
                      'raw_input': binding(EXP / f'hidden/{name}.md'),
                      'probe': binding(EXP / f'hidden/{name}.probe.json'),
                      'prompt_path': str((EXP / f'dispatch/{label}.md').relative_to(ROOT)), 'prompt_binding': None,
                      'artifact_dir': str((EXP / f'outputs/{label}').relative_to(ROOT)),
                      'observation_dir': str((EXP / f'observations/{label}').relative_to(ROOT)),
                      'receipt': str((EXP / f'receipts/{label}.json').relative_to(ROOT)),
                      'grade': str((EXP / f'guardrail-grades/{label}.json').relative_to(ROOT))})
    protocol = [BASE / x for x in ['README.md', 'worker-prompt.md', 'craft-rubric.json', 'reference.py']]
    write(EXP / 'initialization.json', {'schema_version': 1, 'kind': 'sealed_first_wave_24_cells',
          'expected_cells': 24, 'held_out_cells': 16, 'development_cells': 8,
          'cases': cases, 'cells': cells, 'protocol_bindings': [binding(p) for p in protocol],
          'checker': binding(ROOT / 'src/role_eval.rs'),
          'baseline_context': str(context.relative_to(ROOT)),
          'ordinary_context': str(context.relative_to(ROOT)),
          'sealed_before_dispatch': [binding(EXP / 'manifests' / s) for s in ['hidden-content-seal.json','hidden-probe-seal.json']]})
    print(json.dumps({'initialization': binding(EXP / 'initialization.json'), 'cells': 24, 'held_out': 16, 'development': 8}))


def initial():
    v = read(EXP / 'initialization.json')
    for b in v['protocol_bindings'] + v['sealed_before_dispatch'] + [v['checker']]:
        verify(b)
    for case in v['cases']:
        verify(case['input']); verify(case['rubric'])
    return v


def freeze(context):
    v = initial()
    m = EXP / 'manifests/context-improved.json'
    write(m, context_manifest(context))
    write(EXP / 'configurations/improved.json', configuration('improved', m))
    for c in v['cells']:
        if c['prompt_binding']:
            verify(c['prompt_binding'])
            continue
        out = ROOT / c['artifact_dir']
        out.mkdir(parents=True, exist_ok=False)
        source_context = ROOT / (v['baseline_context'] if c['configuration'] == 'baseline' else str(context.relative_to(ROOT)))
        text = (BASE / 'worker-prompt.md').read_text()
        for old,new in [('REFERENCE_CONTEXT_ROOT',str(source_context)),('SHARED_CONTEXT_ROOT',str(ROOT / v['ordinary_context'])),
                        ('HELPER_PATH',str(BASE / 'reference.py')),('ARTIFACT_DIR',str(out)),
                        ('RAW_INPUT_BYTES',verify(c['raw_input']).read_text())]:
            text = text.replace(old,new)
        p = ROOT / c['prompt_path']; p.parent.mkdir(parents=True,exist_ok=True)
        if p.exists():
            raise ValueError('Refusing overwrite of a dispatch prompt')
        p.write_text(text)
        c['prompt_binding'] = binding(p)
    suite = {'schema_version':1, 'id':'fresh-design-quality-wave-1', 'checker':v['checker'],
             'configurations':[{'id':a,'file':binding(EXP / f'configurations/{a}.json')} for a in ARMS],
             'trials_per_case':2,'development':['DEV_A','DEV_B'],'held_out':['Q01','Q02','Q03','Q04'],'cases':v['cases']}
    write(EXP / 'suite.json',suite)
    observers = [BASE / 'observe.cjs', BASE / 'controls/observer-contract.md']
    v.update({'kind':'frozen_host_experiment','suite':binding(EXP / 'suite.json'),
              'improved_context':str(context.relative_to(ROOT)),
              'observer_bindings':[binding(x) for x in observers],
              'source_manifests':[binding(EXP / f'manifests/context-{a}.json') for a in ARMS]})
    write(EXP / 'freeze.json',v)
    print(json.dumps({'freeze':binding(EXP / 'freeze.json'),'suite':v['suite'],'cells':24}))


def frozen():
    v = read(EXP / 'freeze.json')
    for b in v['protocol_bindings']+v['observer_bindings']+[v['suite']]+v['source_manifests']:
        verify(b)
    for m in v['source_manifests']:
        for b in read(verify(m))['files']:
            verify(b)
    return v


def seal(label, worker, status, returned, note):
    v = read(EXP / ('freeze.json' if (EXP / 'freeze.json').exists() else 'initialization.json'))
    c = next(c for c in v['cells'] if c['label']==label)
    if not worker.strip():
        raise ValueError('Actual host worker ID required')
    if c['split']=='held_out' and not (EXP / 'freeze.json').exists():
        raise ValueError('Holdout dispatch requires frozen improved context')
    files = [binding(p) for p in sorted((ROOT / c['artifact_dir']).rglob('*')) if p.is_file()]
    prompt = c.get('prompt_binding') or binding(ROOT / c['prompt_path'])
    verify(prompt)
    value = {'schema_version':1,'kind':'host_trial_receipt','label':label,'worker_id':worker,
             'case_id':c['case_id'],'configuration':c['configuration'],'trial':c['trial'],
             'execution_status':status,'prompt':prompt,'output_bindings':files,
             'returned_response':binding(returned) if returned else None,
             'cost_usd':None,'latency_ms':None,'host_note':note or 'Actual unrepaired first return retained.',
             'retrieval_limit':'Helper delivery events show instrumented bytes delivered, not authenticated attention/use; historical host source reads unavailable.',
             'human_acceptance':'not_collected'}
    write(ROOT / c['receipt'],value)
    print(json.dumps(binding(ROOT / c['receipt'])))


def collect():
    v = frozen(); suite = read(verify(v['suite']))
    report={'schema_version':1,'provenance':'host_observed','suite_sha256':v['suite']['sha256'],
            'checker_sha256':suite['checker']['sha256'],'trials':[],
            'limitations':['Four hand-authored heldout brief clusters; repeated trials and judges are not independent briefs.',
                          'Continuous agent craft ratings are separate evidence; version1 scores record functional/access guards only.',
                          'Exact backend model/settings, cost, token use and model latency unavailable; null remains unknown.',
                          'Instrumented delivery and host receipts do not authenticate model execution or attention.',
                          'Read/write boundaries are instructions, not a sandbox; human acceptance not collected.',
                          'Same-host agent judging and imperfect blinding cannot establish human visual acceptance or general gains.']}
    for c in v['cells']:
        receipt_path=ROOT / c['receipt']; r=read(receipt_path)
        if r['execution_status'] not in ('completed','infrastructure_failed') or not r['worker_id']:
            raise ValueError('Every planned actual trial is required')
        verify(r['prompt'])
        for b in r['output_bindings']:
            verify(b)
        evidence=[binding(receipt_path)]
        if r.get('returned_response'): verify(r['returned_response'])
        if r['execution_status']=='infrastructure_failed':
            verdicts={};score=None;outcome='infrastructure_failed';rationale=r['host_note']
        else:
            p=ROOT / c['grade']; g=read(p)
            case=next(x for x in suite['cases'] if x['id']==c['case_id'])
            if g.get('reviewer_id') in (None,'',r['worker_id']) or g.get('case_id')!=c['case_id']:
                raise ValueError('Independent reviewer and matching case required')
            verdicts=g['verdicts']
            if set(verdicts)!=set(case['criteria']) or not all(isinstance(x,bool) for x in verdicts.values()) or not g.get('rationale','').strip():
                raise ValueError('All fixed guardrail verdicts and located rationale required')
            score=sum(verdicts.values())/len(verdicts);outcome='passed' if score==1 else 'task_failed';rationale=g['rationale']
            evidence.append(binding(p))
            for p in [ROOT / c['artifact_dir'] / 'index.html',ROOT / c['artifact_dir'] / 'decision.md',ROOT / c['observation_dir'] / 'observations.json']:
                if p.is_file() and p.stat().st_size<=1024*1024:evidence.append(binding(p))
        case=next(x for x in suite['cases'] if x['id']==c['case_id'])
        config=next(x for x in suite['configurations'] if x['id']==c['configuration'])
        report['trials'].append({'case_id':c['case_id'],'configuration':c['configuration'],'trial':c['trial'],
             'input_sha256':case['input']['sha256'],'rubric_sha256':case['rubric']['sha256'],
             'configuration_sha256':config['file']['sha256'],'outcome':outcome,'score':score,'verdicts':verdicts,
             'evidence':evidence,'rationale':rationale,'cost_usd':r.get('cost_usd'),'latency_ms':r.get('latency_ms')})
    write(EXP / 'report.json',report)
    print(json.dumps({'report':binding(EXP / 'report.json'),'actual_retained_trials':len(report['trials'])}))


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--experiment-root', help='Normal project-relative experiment directory; its parent owns development, contexts and dev-dispatch (default: current private run)')
    sub=p.add_subparsers(dest='command',required=True)
    sub.add_parser('initialize')
    f=sub.add_parser('freeze');f.add_argument('--improved-context',type=Path,required=True)
    s=sub.add_parser('seal');s.add_argument('label');s.add_argument('--worker-id',required=True)
    s.add_argument('--status',choices=['completed','infrastructure_failed'],default='completed')
    s.add_argument('--returned-response',type=Path);s.add_argument('--note',default='')
    sub.add_parser('collect')
    a=p.parse_args()
    try:
        select_experiment_root(a.experiment_root)
    except ValueError as error:
        p.error(str(error))
    if a.command=='initialize':initialize()
    elif a.command=='freeze':freeze(a.improved_context.resolve(strict=True))
    elif a.command=='seal':seal(a.label,a.worker_id,a.status,a.returned_response,a.note)
    else:collect()

if __name__=='__main__':
    try:
        main()
    except ValueError as error:
        raise SystemExit('error: ' + str(error))
