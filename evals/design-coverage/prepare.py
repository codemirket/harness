#!/usr/bin/env python3
"""Pin local contexts and collect the existing role-eval report. No model calls."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / 'evals/design-coverage'
TASK = ROOT / '.mirket/implementation/design-coverage-2026-10-15/evaluation'
CASES = ['development-checker', 'field-library', 'cue-review', 'native-handover']
ARMS = ['baseline', 'assisted']


def digest(data):
    return hashlib.sha256(data).hexdigest()


def load(path):
    return json.loads(path.read_text())


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def binding(path):
    if path.is_symlink() or not path.is_file():
        raise ValueError(f'Expected regular file: {path}')
    return {'path': str(path.relative_to(ROOT)), 'sha256': digest(path.read_bytes())}


def verify(value):
    relative = Path(value['path'])
    if relative.is_absolute() or any(p in ('', '.', '..') for p in relative.parts):
        raise ValueError('Expected normal project-relative binding')
    path = ROOT / relative
    cursor = ROOT
    for part in relative.parts:
        cursor /= part
        if cursor.is_symlink():
            raise ValueError(f'Symlink refused: {cursor}')
    if binding(path) != value:
        raise ValueError(f'Stale binding: {relative}')
    return path


def context_manifest(root):
    return {'schema_version': 1, 'files': [binding(p) for p in sorted(root.rglob('*')) if p.is_file()]}


def snapshot_baseline(baseline_source, interface_source, provenance):
    source = Path(baseline_source).resolve()
    ordinary = Path(interface_source).resolve()
    library = load(source / 'references/design-languages/library.json')
    if library.get('schema_version') != 1 or len(library.get('instances', [])) != 19 or (source / 'references/design-languages/coverage.json').exists():
        raise ValueError('Supply the verified pre-extension nineteen-reference source; current extended guidance is not a baseline')
    if not (ordinary / 'SKILL.md').is_file() or not provenance.strip():
        raise ValueError('Explicit ordinary guidance source and historical provenance required')
    for name in ('baseline-design-inspiration', 'shared-interface-design', 'baseline-source-bindings.json', 'freeze.json'):
        if (TASK / name).exists():
            raise ValueError('Refusing to replace existing historical snapshots or experiment')
    TASK.mkdir(parents=True, exist_ok=True)
    shutil.copytree(source, TASK / 'baseline-design-inspiration')
    shutil.copytree(ordinary, TASK / 'shared-interface-design')
    files = context_manifest(TASK / 'baseline-design-inspiration')['files'] + context_manifest(TASK / 'shared-interface-design')['files']
    target = TASK / 'baseline-source-bindings.json'
    save(target, {'schema_version': 1, 'purpose': 'Caller-supplied verified pre-extension baseline and matched ordinary guidance',
                 'historical_source_provenance': provenance, 'baseline_reference_count': 19, 'files': files,
                 'limit': 'Caller provenance identifies the supplied history; file hashes alone cannot establish that it is historically correct.'})
    print(json.dumps(binding(target)))


def frozen():
    value = load(TASK / 'freeze.json')
    verify(value['suite'])
    for item in value['protocol_bindings']:
        verify(item)
    for item in value['source_manifests']:
        path = verify(item)
        for source in load(path)['files']:
            verify(source)
    return value


def make_suite():
    cases = []
    for case in CASES:
        rubric = BASE / 'graders' / f'{case}.json'
        cases.append({'id': case, 'split': 'development' if case == CASES[0] else 'held_out',
                      'input': binding(BASE / 'inputs' / f'{case}.md'), 'rubric': binding(rubric),
                      'criteria': load(rubric)['criteria']})
    return {'schema_version': 1, 'id': 'design-coverage-transfer-v1',
            'checker': binding(ROOT / 'src/role_eval.rs'),
            'configurations': [{'id': arm, 'file': binding(BASE / 'configurations' / f'{arm}-template.json')} for arm in ARMS],
            'trials_per_case': 1, 'development': [CASES[0]], 'held_out': CASES[1:], 'cases': cases}


def freeze():
    if (TASK / 'freeze.json').exists():
        raise ValueError('Freeze already exists; do not overwrite a prespecified experiment')
    prespec = TASK / 'protocol-source-bindings.json'
    if prespec.exists():
        for item in load(prespec)['files']:
            verify(item)
    baseline = TASK / 'baseline-source-bindings.json'
    for item in load(baseline)['files']:
        verify(item)
    if load(baseline)['baseline_reference_count'] != 19:
        raise ValueError('Baseline must have nineteen references')
    source = ROOT / 'skills/design-inspiration'
    library = load(source / 'references/design-languages/library.json')
    instances = library.get('instances', [])
    count = len(instances) if isinstance(instances, list) else 0
    if count != 27:
        raise ValueError(f'Assisted source is not the completed twenty-seven-reference library: {count}')
    assisted = TASK / 'assisted-design-inspiration'
    if assisted.exists():
        raise ValueError('Refusing to overwrite an existing assisted snapshot')
    if not (source / 'references/design-languages/coverage.json').is_file():
        raise ValueError('Structured assisted coverage is missing')
    shutil.copytree(source, assisted)
    assisted_manifest = TASK / 'assisted-source-bindings.json'
    save(assisted_manifest, context_manifest(assisted))
    shared_manifest = TASK / 'shared-interface-source-bindings.json'
    save(shared_manifest, context_manifest(TASK / 'shared-interface-design'))
    suite = make_suite()
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    for config in suite['configurations']:
        arm = config['id']
        value = load(BASE / 'configurations' / f'{arm}-template.json')
        value['status'] = 'frozen_for_host_dispatch'
        value['harness_revision'] = revision + ' + working source bytes pinned by this experiment'
        selected_manifest = baseline if arm == 'baseline' else assisted_manifest
        value['guidance_hashes'] = {'reference_source_manifest': binding(selected_manifest), 'ordinary_interface_manifest': binding(shared_manifest)}
        value['reference_context_root'] = str((TASK / ('baseline-design-inspiration' if arm == 'baseline' else 'assisted-design-inspiration')).relative_to(ROOT))
        value['ordinary_interface_root'] = str((TASK / 'shared-interface-design').relative_to(ROOT))
        actual_path = TASK / 'configurations' / f'{arm}.json'
        save(actual_path, value)
        config['file'] = binding(actual_path)
    suite_path = TASK / 'suite.json'
    save(suite_path, suite)
    protocol = [BASE / 'README.md', BASE / 'check-artifact.cjs', BASE / 'prepare.py', BASE / 'suite.json']
    protocol.extend(BASE / 'inputs' / f'{case}.md' for case in CASES)
    protocol.extend(BASE / 'graders' / f'{case}.json' for case in CASES)
    if prespec.exists():
        protocol.append(prespec)
    common = re.search(r'## Common worker prompt\n\n```\n(.*?)\n```', (BASE / 'README.md').read_text(), re.S).group(1)
    order = [(CASES[0], 'baseline'), (CASES[0], 'assisted'),
             ('field-library', 'assisted'), ('field-library', 'baseline'),
             ('cue-review', 'baseline'), ('cue-review', 'assisted'),
             ('native-handover', 'assisted'), ('native-handover', 'baseline')]
    cells = []
    for index, (case, arm) in enumerate(order, 1):
        label = f'R{index:02d}'
        artifact = TASK / 'outputs' / label
        artifact.mkdir(parents=True, exist_ok=False)
        config = next(c for c in suite['configurations'] if c['id'] == arm)
        actual = load(ROOT / config['file']['path'])
        raw = BASE / 'inputs' / f'{case}.md'
        prompt = common.replace('ARTIFACT_DIR', str(artifact)).replace('REFERENCE_CONTEXT_ROOT', str(ROOT / actual['reference_context_root'])).replace('SHARED_INTERFACE_ROOT', str(TASK / 'shared-interface-design')).replace('[paste the exact bound input file bytes here]', raw.read_text())
        prompt_path = TASK / 'prompts' / f'{label}.md'
        prompt_path.parent.mkdir(parents=True, exist_ok=True)
        prompt_path.write_text(prompt + '\n')
        cell = {'neutral_label': label, 'case_id': case, 'configuration': arm, 'trial': 1,
                'raw_input': binding(raw), 'prompt': binding(prompt_path),
                'artifact_dir': str(artifact.relative_to(ROOT)),
                'observation_dir': str((TASK / 'observations' / label).relative_to(ROOT)),
                'receipt': str((TASK / 'receipts' / f'{label}.json').relative_to(ROOT)),
                'grade': str((TASK / 'grades' / f'{label}.json').relative_to(ROOT))}
        cells.append(cell)
        save(ROOT / cell['receipt'], {'schema_version': 1, 'kind': 'host_trial_receipt',
             'neutral_label': label, 'case_id': case, 'configuration': arm, 'trial': 1,
             'execution_status': 'not_started', 'worker_id': None,
             'prompt_sha256': cell['prompt']['sha256'], 'observed_source_reads': [],
             'source_read_observation_limit': 'Not yet observed; worker self-report alone is not host evidence.',
             'output_bindings': [], 'cost_usd': None, 'latency_ms': None,
             'host_note': 'Template; no execution is recorded.'})
    value = {'schema_version': 1, 'kind': 'prespecified_host_experiment', 'suite': binding(suite_path),
             'actual_planned_executions': 8, 'held_out_planned_executions': 6,
             'protocol_bindings': [binding(p) for p in protocol],
             'source_manifests': [binding(baseline), binding(assisted_manifest), binding(shared_manifest)],
             'cells': cells, 'limits': ['Same inherited host model/settings, exact backend version and effort unavailable.', 'File identities and host receipts do not authenticate model execution or human approval.', 'Worker/grade read boundaries are instructions, not a sandbox.', 'Held-out n=3, one trial per arm; no general gain or variance claim.']}
    save(TASK / 'freeze.json', value)
    print(json.dumps({'freeze': binding(TASK / 'freeze.json'), 'suite': value['suite'], 'cells': cells}, indent=2))


def seal(label, worker_id, status, reads, note):
    manifest = frozen()
    cell = next(c for c in manifest['cells'] if c['neutral_label'] == label)
    receipt_path = ROOT / cell['receipt']
    receipt = load(receipt_path)
    if receipt['execution_status'] != 'not_started':
        raise ValueError('Receipt already sealed; retain the first trial rather than replacing it')
    if not worker_id.strip():
        raise ValueError('Actual host worker identity required')
    sources = load(Path(reads)) if reads else []
    for value in sources:
        verify(value)
    receipt.update({'execution_status': status, 'worker_id': worker_id,
                    'observed_source_reads': sources,
                    'source_read_observation_limit': 'Host supplied observed reads; authenticity remains the host assertion.' if sources else 'Host source-read observations unavailable; do not infer retrieval from the worker declaration.',
                    'output_bindings': [binding(p) for p in sorted((ROOT / cell['artifact_dir']).rglob('*')) if p.is_file()],
                    'host_note': note or 'Host sealed the actual returned worker artifacts; no repairs applied.'})
    save(receipt_path, receipt)
    print(json.dumps(binding(receipt_path)))


def collect():
    manifest = frozen()
    suite_path = verify(manifest['suite'])
    suite = load(suite_path)
    report = {'schema_version': 1, 'provenance': 'host_observed',
              'suite_sha256': manifest['suite']['sha256'], 'checker_sha256': suite['checker']['sha256'],
              'trials': [], 'limitations': manifest['limits'] + [
                  'Human design acceptance not collected. Desktop Chromium checks do not establish native/Arabic/assistive usability or real collaboration.',
                  'Exact seed, temperature, reasoning effort, token usage, model cost and model latency unavailable; unknown metrics remain null.',
                  'Reference context and structured routing are the intentional intervention; source payload length is not token matched.',
                  'Independent grader judgments are fallible; this pilot is exploratory.']}
    for cell in manifest['cells']:
        receipt_path = ROOT / cell['receipt']
        receipt = load(receipt_path)
        if receipt['execution_status'] not in ('completed', 'infrastructure_failed') or not receipt.get('worker_id'):
            raise ValueError(f'Actual execution not recorded for {cell["neutral_label"]}')
        verify(cell['prompt'])
        for output in receipt['output_bindings']:
            verify(output)
        case = next(c for c in suite['cases'] if c['id'] == cell['case_id'])
        config = next(c for c in suite['configurations'] if c['id'] == cell['configuration'])
        evidence = [binding(receipt_path)]
        if receipt['execution_status'] == 'infrastructure_failed':
            verdicts, score, outcome = {}, None, 'infrastructure_failed'
            rationale = receipt['host_note']
        else:
            grade_path = ROOT / cell['grade']
            grade = load(grade_path)
            if grade['case_id'] != case['id'] or not grade.get('reviewer_id') or not grade.get('rationale', '').strip():
                raise ValueError('Independent grade requires matching case, real reviewer and rationale')
            if grade['reviewer_id'] == receipt['worker_id']:
                raise ValueError('The implementation worker cannot be its own independent grader')
            verdicts = grade['verdicts']
            if set(verdicts) != set(case['criteria']) or not all(isinstance(v, bool) for v in verdicts.values()):
                raise ValueError('Missing/unknown/non-boolean criterion verdicts')
            score = sum(verdicts.values()) / len(verdicts)
            outcome = 'passed' if score == 1 else 'task_failed'
            rationale = grade['rationale']
            evidence.append(binding(grade_path))
            output_root = ROOT / cell['artifact_dir']
            for name in ('index.html', 'decision.md', 'decision.json'):
                if (output_root / name).is_file():
                    evidence.append(binding(output_root / name))
            observation = ROOT / cell['observation_dir'] / 'observations.json'
            if observation.is_file():
                evidence.append(binding(observation))
                for shot in load(observation).get('screenshots', []):
                    shot_path = Path(shot['path'])
                    value = binding(shot_path)
                    if value['sha256'] != shot['sha256']:
                        raise ValueError('Screenshot changed since observation')
                    evidence.append(value)
        if len(evidence) > 16 or any((ROOT / value['path']).stat().st_size > 1024 * 1024 for value in evidence):
            raise ValueError('Evidence exceeds existing evaluator limits; preserve originals and select a documented bounded evidence set')
        report['trials'].append({'case_id': case['id'], 'configuration': config['id'], 'trial': 1,
             'input_sha256': case['input']['sha256'], 'rubric_sha256': case['rubric']['sha256'],
             'configuration_sha256': config['file']['sha256'], 'outcome': outcome,
             'score': score, 'verdicts': verdicts, 'evidence': evidence, 'rationale': rationale,
             'cost_usd': receipt.get('cost_usd'), 'latency_ms': receipt.get('latency_ms')})
    target = TASK / 'report.json'
    if target.exists():
        raise ValueError('Report exists; preserve the first graded report')
    save(target, report)
    print(json.dumps({'report': binding(target), 'suite': manifest['suite'], 'actual_receipts': len(report['trials'])}, indent=2))


def main():
    global TASK
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evaluation-root', help='Caller-owned directory inside this project; default is the ignored 2026-10-15 pilot directory')
    sub = parser.add_subparsers(dest='command', required=True)
    snapshot = sub.add_parser('snapshot-baseline', help='Capture explicitly supplied verified historical sources; never infer a baseline from current guidance')
    snapshot.add_argument('--baseline-source', required=True)
    snapshot.add_argument('--interface-source', required=True)
    snapshot.add_argument('--provenance', required=True, help='Exact historical source pin or other verifiable caller-owned source identity')
    sub.add_parser('public-suite', help='Refresh authored schema fixture pins before the experiment is frozen')
    sub.add_parser('freeze', help='Snapshot final assisted context and freeze all dispatch inputs')
    seal_parser = sub.add_parser('seal', help='Seal the actual returned worker artifacts and real host identity')
    seal_parser.add_argument('label')
    seal_parser.add_argument('--worker-id', required=True)
    seal_parser.add_argument('--status', choices=['completed', 'infrastructure_failed'], default='completed')
    seal_parser.add_argument('--observed-reads', help='JSON array of actual host-observed {path,sha256} reads, not worker guesses')
    seal_parser.add_argument('--note', default='')
    sub.add_parser('collect', help='Collect all eight existing receipts and independent grades into role-eval schema v1')
    args = parser.parse_args()
    if args.evaluation_root:
        selected = Path(args.evaluation_root)
        TASK = (selected if selected.is_absolute() else ROOT / selected).resolve()
        if not TASK.is_relative_to(ROOT) or TASK == ROOT:
            raise ValueError('Existing role-eval bindings require an evaluation root inside the selected project')
    if args.command == 'snapshot-baseline':
        snapshot_baseline(args.baseline_source, args.interface_source, args.provenance)
    elif args.command == 'public-suite':
        if (TASK / 'freeze.json').exists():
            raise ValueError('Cannot re-pin public inputs after freeze')
        save(BASE / 'suite.json', make_suite())
        print(json.dumps(binding(BASE / 'suite.json')))
    elif args.command == 'freeze':
        freeze()
    elif args.command == 'seal':
        seal(args.label, args.worker_id, args.status, args.observed_reads, args.note)
    else:
        collect()


if __name__ == '__main__':
    main()
