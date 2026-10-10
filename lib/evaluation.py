"""Prepare and assess local development tasks. Python 3.9+, standard library only.

No model, installer, account or external service is invoked. Explicit `check`
executes a repository verifier and candidate code with normal host permissions;
the scratch directory and timeout are not an operating-system sandbox.
"""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import threading
import time

from . import catalog, harness

ROOT = Path(__file__).resolve().parents[1]
SUITE = ROOT / 'evaluations/suite.json'
DOMAINS = ('frontend', 'engineering', 'documentation', 'integration', 'research',
           'analysis', 'database')
MAX_OUTPUT = 1024 * 1024
MAX_TREE = 64 * 1024 * 1024


def digest(value):
    return hashlib.sha256(value).hexdigest()


def json_digest(value):
    return digest(json.dumps(value, sort_keys=True, ensure_ascii=False).encode())


def timestamp():
    return datetime.now(timezone.utc).isoformat()


def read_json(path):
    if catalog.is_link(path) or not path.is_file() or path.stat().st_size > MAX_OUTPUT:
        raise ValueError('Missing, linked or oversized evaluation JSON: ' + str(path))
    value = json.loads(path.read_text(encoding='utf-8'))
    if not isinstance(value, dict):
        raise ValueError('Evaluation JSON must be an object: ' + str(path))
    return value


def nonempty(value, label):
    if not isinstance(value, str) or not value.strip():
        raise ValueError(label + ' must be nonempty text')
    return value


def suite():
    data = read_json(SUITE)
    if data.get('schema_version') != 1 or not isinstance(data.get('cases'), list):
        raise ValueError('Unsupported evaluation suite')
    seen = set()
    for case in data['cases']:
        allowed = {'id', 'domain', 'title', 'prompt', 'required_evidence',
                   'review_required', 'split', 'immutable_inputs', 'review_artifacts'}
        if not isinstance(case, dict) or set(case) - allowed:
            raise ValueError('Unknown evaluation case fields')
        identifier = nonempty(case.get('id'), 'Case ID')
        if len(catalog.safe_path(identifier).parts) != 1 or identifier in seen:
            raise ValueError('Duplicate or invalid evaluation case: ' + identifier)
        seen.add(identifier)
        if case.get('domain') not in DOMAINS or type(case.get('review_required')) is not bool:
            raise ValueError('Invalid domain or review requirement: ' + identifier)
        for name in ('title', 'prompt'):
            nonempty(case.get(name), name)
        if case['prompt'] != 'cases/' + identifier + '/task.md':
            raise ValueError('Prompt must identify the copied task.md')
        if case.get('split', 'development') != 'development':
            raise ValueError('Only development cases are supported')
        evidence = case.get('required_evidence')
        if not isinstance(evidence, list) or not evidence:
            raise ValueError('A case needs evidence criteria: ' + identifier)
        for item in evidence:
            nonempty(item, 'Evidence criterion')
        for key in ('immutable_inputs', 'review_artifacts'):
            names = case.get(key, [])
            if not isinstance(names, list) or not all(isinstance(name, str) for name in names):
                raise ValueError('Invalid case artifact paths')
            if len(names) != len(set(names)):
                raise ValueError('Invalid case artifact paths')
            for name in names:
                catalog.safe_path(nonempty(name, key))
    if not seen:
        raise ValueError('Empty evaluation suite')
    return data


def tree(path):
    """Fingerprint actual files, excluding interpreter caches, rejecting links."""
    if catalog.is_link(path) or not path.is_dir():
        raise ValueError('Expected a real evaluation directory: ' + str(path))
    values = {}
    total = 0
    for directory, dirs, names in os.walk(path, followlinks=False):
        for name in dirs + names:
            if catalog.is_link(Path(directory) / name):
                raise ValueError('Linked evaluation input: ' + name)
        dirs[:] = [name for name in dirs if name != '__pycache__']
        for name in names:
            if name.endswith('.pyc') or name == '.DS_Store':
                continue
            item = Path(directory) / name
            if not item.is_file():
                raise ValueError('Non-regular evaluation input: ' + name)
            total += item.stat().st_size
            if total > MAX_TREE:
                raise ValueError('Evaluation tree exceeds size limit')
            values[item.relative_to(path).as_posix()] = digest(item.read_bytes())
    catalog.validate_paths(values)
    return values


def case_inputs(case):
    source = harness.guarded_path(SUITE.parent, 'cases/' + case['id'])
    files = tree(source)
    for required in ('task.md', 'rubric.md', 'verify.py'):
        if required not in files:
            raise ValueError('Missing case file: ' + required)
    if not any(name.startswith('workspace/') for name in files):
        raise ValueError('Missing case workspace')
    for name in case.get('immutable_inputs', []):
        if 'workspace/' + name not in files:
            raise ValueError('Missing immutable fixture input: ' + name)
    return source, files


def harness_inputs(source):
    source = source.expanduser().resolve(strict=True)
    files = {}
    for name in ('skills', 'instructions', 'lib'):
        if (source / name).is_dir():
            files.update({name + '/' + key: value for key, value in tree(source / name).items()})
    for name in ('ai.py', 'AGENTS.md', 'registry/catalog.json', 'registry/harness.json',
                 'registry/capabilities.json', 'registry/targets.json', 'registry/mcp.json',
                 'registry/codex-settings.json'):
        item = source / name
        if item.exists():
            if catalog.is_link(item) or not item.is_file():
                raise ValueError('Linked harness source: ' + name)
            files[name] = digest(item.read_bytes())
    if not files:
        raise ValueError('No harness instructions or code in source directory')
    return {'path': str(source), 'sha256': json_digest(files), 'files': files}


def prepare(identifier, output, model, condition, source=ROOT, settings='not-recorded'):
    data = suite()
    case = next((value for value in data['cases'] if value['id'] == identifier), None)
    if case is None:
        raise ValueError('Unknown evaluation case: ' + identifier)
    directory, files = case_inputs(case)
    provenance = harness_inputs(source)
    output = output.expanduser().absolute()
    # Resolve the explicitly supplied parent once, then refuse an existing leaf.
    output = output.parent.resolve(strict=True) / output.name
    if output.exists() or catalog.is_link(output):
        raise ValueError('Evaluation output must be absent')
    record = {
        'schema_version': 1, 'created_at': timestamp(), 'case': case,
        'suite_sha256': digest(SUITE.read_bytes()),
        'case_sha256': json_digest({'case': case, 'files': files}),
        'case_files': files, 'harness': provenance,
        'model': nonempty(model, 'Model label'),
        'settings': nonempty(settings, 'Settings label'),
        'condition': nonempty(condition, 'Condition label'),
    }
    output.mkdir()  # No replacement; later I/O failures may leave a partial run.
    for name in files:
        destination = output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes((directory / name).read_bytes())
    harness.write_json(output / 'run.json', record)
    return {'run': str(output), 'case': identifier, 'task': str(output / 'task.md'),
            'workspace': str(output / 'workspace'), 'status': 'prepared'}


def load_run(path):
    if catalog.is_link(path):
        raise ValueError('Linked evaluation run')
    path = path.expanduser().resolve(strict=True)
    record = read_json(path / 'run.json')
    if record.get('schema_version') != 1:
        raise ValueError('Unsupported evaluation run')
    case = record.get('case', {})
    if not isinstance(case, dict):
        raise ValueError('Invalid run case')
    current = next((value for value in suite()['cases'] if value['id'] == case.get('id')), None)
    if current is None:
        raise ValueError('Unknown run case')
    _, files = case_inputs(current)
    expected = json_digest({'case': current, 'files': files})
    if record.get('case_sha256') != expected or case != current or record.get('case_files') != files:
        raise ValueError('Case definition changed; prepare a fresh run')
    for name in ('task.md', 'rubric.md', 'verify.py'):
        item = path / name
        if catalog.is_link(item) or not item.is_file() or digest(item.read_bytes()) != files[name]:
            raise ValueError('Evaluation control file changed: ' + name)
    for key in ('model', 'condition', 'settings'):
        nonempty(record.get(key), key)
    inputs = tree(path / 'workspace')
    for name in current.get('immutable_inputs', []):
        if inputs.get(name) != files['workspace/' + name]:
            raise ValueError('Immutable fixture input changed: ' + name)
    return path, record


def stop(process):
    if os.name == 'nt':
        try:
            subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5)
        except (OSError, subprocess.TimeoutExpired):
            process.kill()
    else:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            if process.poll() is None:
                process.kill()
    process.wait(timeout=5)


def execute(argv, cwd, timeout):
    """Drain bounded output and terminate a timed-out verifier process tree."""
    process = subprocess.Popen(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               stdin=subprocess.DEVNULL, start_new_session=os.name != 'nt')
    chunks = [bytearray(), bytearray()]
    exceeded = threading.Event()
    lock = threading.Lock()
    size = [0]

    def reader(stream, index):
        try:
            while True:
                block = stream.read(4096)
                if not block:
                    break
                with lock:
                    remaining = max(0, MAX_OUTPUT - size[0])
                    chunks[index].extend(block[:remaining])
                    size[0] += len(block)
                    if size[0] > MAX_OUTPUT:
                        exceeded.set()
                if exceeded.is_set():
                    break
        finally:
            stream.close()

    workers = [threading.Thread(target=reader, args=(process.stdout, 0), daemon=True),
               threading.Thread(target=reader, args=(process.stderr, 1), daemon=True)]
    for worker in workers:
        worker.start()
    deadline = time.monotonic() + timeout
    try:
        while process.poll() is None or any(worker.is_alive() for worker in workers):
            if exceeded.is_set():
                raise ValueError('Verifier exceeded output limit')
            if time.monotonic() >= deadline:
                raise ValueError('Verifier timed out')
            time.sleep(0.01)
        if exceeded.is_set():
            raise ValueError('Verifier exceeded output limit')
        return process.returncode, bytes(chunks[0])
    finally:
        # Also stop descendants that survived a leader or held output pipes.
        stop(process)
        for worker in workers:
            worker.join(timeout=1)


def check(path, timeout=30):
    path, record = load_run(path)
    if type(timeout) is not int or not 1 <= timeout <= 60:
        raise ValueError('Verifier timeout must be between 1 and 60 seconds')
    inputs = tree(path / 'workspace')
    started = time.monotonic()
    result = {'schema_version': 1, 'run_sha256': json_digest(record),
              'workspace_sha256': json_digest(inputs), 'artifacts': inputs,
              'checked_at': timestamp(), 'checks': [], 'status': 'error'}
    try:
        code, output = execute([sys.executable, '-I', str(path / 'verify.py'),
                                str(path / 'workspace')], path, timeout)
        value = json.loads(output.decode('utf-8'))
        checks = value.get('checks') if isinstance(value, dict) else None
        if not isinstance(checks, list) or not checks:
            raise ValueError('Verifier returned no checks')
        identifiers = set()
        for item in checks:
            if not isinstance(item, dict):
                raise ValueError('Invalid verifier check')
            name = nonempty(item.get('id'), 'Check ID')
            if name in identifiers or item.get('status') not in ('passed', 'failed'):
                raise ValueError('Invalid or duplicate verifier check')
            identifiers.add(name)
            nonempty(item.get('detail'), 'Check detail')
        if tree(path / 'workspace') != inputs:
            raise ValueError('Verifier changed workspace inputs')
        result['checks'] = checks
        result['status'] = 'passed' if code == 0 and all(
            item['status'] == 'passed' for item in checks) else 'failed'
        result['exit_code'] = code
    except (ValueError, OSError, UnicodeError) as error:
        result['detail'] = str(error)
    result['elapsed_seconds'] = round(time.monotonic() - started, 3)
    harness.write_json(path / 'checks.json', result)
    return result


def review(path, reviewer, kind, decision, evidence, rationale):
    path, record = load_run(path)
    if kind not in ('human', 'agent') or decision not in ('accepted', 'needs_changes'):
        raise ValueError('Invalid review kind or decision')
    inputs = tree(path / 'workspace')
    selected = {}
    for name in evidence:
        catalog.safe_path(name)
        if name not in inputs:
            raise ValueError('Review evidence must exist inside workspace: ' + name)
        selected[name] = inputs[name]
    if not selected:
        raise ValueError('A review needs actual evidence files')
    required = set(record['case'].get('review_artifacts', []))
    if not required.issubset(selected):
        raise ValueError('Review needs case output artifacts: ' + ', '.join(sorted(required)))
    if not any(record['case_files'].get('workspace/' + name) != value for name, value in selected.items()):
        raise ValueError('Review must inspect a new or changed output artifact')
    result = {'schema_version': 1, 'run_sha256': json_digest(record),
              'workspace_sha256': json_digest(inputs), 'reviewed_at': timestamp(),
              'reviewer': nonempty(reviewer, 'Reviewer'), 'kind': kind,
              'decision': decision, 'evidence': selected,
              'rationale': nonempty(rationale, 'Review rationale')}
    harness.write_json(path / 'review.json', result)
    return result


def stored_check_status(value):
    """Reject inconsistent edited labels; derive outcome from recorded checks."""
    if value.get('schema_version') != 1:
        return 'error'
    checks = value.get('checks')
    if value.get('status') == 'error' and checks == []:
        return 'error'
    if not isinstance(checks, list) or not checks or type(value.get('exit_code')) is not int:
        return 'error'
    identifiers = set()
    for item in checks:
        if (not isinstance(item, dict) or not isinstance(item.get('id'), str)
                or not item['id'].strip() or item['id'] in identifiers
                or item.get('status') not in ('passed', 'failed')
                or not isinstance(item.get('detail'), str) or not item['detail'].strip()):
            return 'error'
        identifiers.add(item['id'])
    derived = 'passed' if value['exit_code'] == 0 and all(
        item['status'] == 'passed' for item in checks) else 'failed'
    return derived if value.get('status') == derived else 'error'


def stored_review_status(value, record, inputs):
    evidence = value.get('evidence')
    if (value.get('schema_version') != 1 or value.get('kind') not in ('human', 'agent')
            or value.get('decision') not in ('accepted', 'needs_changes')
            or not isinstance(value.get('reviewer'), str) or not value['reviewer'].strip()
            or not isinstance(value.get('rationale'), str) or not value['rationale'].strip()
            or not isinstance(evidence, dict) or not evidence
            or not set(record['case'].get('review_artifacts', [])).issubset(evidence)):
        return 'error'
    if any(inputs.get(name) != checksum for name, checksum in evidence.items()):
        return 'error'
    if not any(record['case_files'].get('workspace/' + name) != checksum for name, checksum in evidence.items()):
        return 'error'
    return value['decision']


def report(paths):
    rows = []
    for path in paths:
        path, record = load_run(path)
        inputs = tree(path / 'workspace')
        current = {'run_sha256': json_digest(record), 'workspace_sha256': json_digest(inputs)}
        automated = 'not_run'
        reviewed = 'pending' if record['case']['review_required'] else 'not_required'
        review_kind = reviewer = None
        for filename, label in (('checks.json', 'automated'), ('review.json', 'review')):
            if not (path / filename).exists():
                continue
            value = read_json(path / filename)
            state = 'stale' if any(value.get(key) != expected for key, expected in current.items()) else (
                stored_check_status(value) if label == 'automated'
                else stored_review_status(value, record, inputs))
            if label == 'automated':
                automated = state
            else:
                reviewed = state
                if state in ('accepted', 'needs_changes'):
                    review_kind, reviewer = value['kind'], value['reviewer']
        accepted = automated == 'passed' and reviewed in ('accepted', 'not_required')
        accepted_by = review_kind if accepted and reviewed == 'accepted' else 'none'
        rows.append({'run': str(path), 'case': record['case']['id'], 'domain': record['case']['domain'],
                     'condition': record['condition'], 'model': record['model'],
                     'settings': record['settings'], 'suite_sha256': record['suite_sha256'],
                     'case_sha256': record['case_sha256'], 'harness_sha256': record['harness']['sha256'],
                     'automated': automated, 'review': reviewed, 'review_kind': review_kind,
                     'reviewer': reviewer, 'accepted': accepted, 'accepted_by': accepted_by})
    return {'runs': rows, 'automated_counts': dict(Counter(row['automated'] for row in rows)),
            'review_counts': dict(Counter(row['review'] for row in rows)),
            'review_kind_counts': dict(Counter(row['review_kind'] for row in rows
                                              if row['review_kind'] is not None)),
            'accepted_by_counts': dict(Counter(row['accepted_by'] for row in rows)),
            'limits': ['Development cases, not a held-out benchmark.',
                       'Model/settings and reviewer identity are recorded labels, not runtime attestation.',
                       'Recorded review does not establish human approval or production readiness.',
                       'Compare matching cases, model/settings, inputs, tools and budgets; repeat trials.']}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    listing = commands.add_parser('list', help='List development cases and evidence criteria')
    listing.add_argument('--domain', choices=DOMAINS)
    preparing = commands.add_parser('prepare', help='Copy a case into a fresh scratch run')
    preparing.add_argument('--case', required=True)
    preparing.add_argument('--output', required=True, type=Path)
    preparing.add_argument('--model', required=True, help='Exact model/version or explicit unavailable label')
    preparing.add_argument('--condition', required=True, help='For example baseline or candidate')
    preparing.add_argument('--settings', default='not-recorded', help='Reasoning/settings/budget label')
    preparing.add_argument('--harness-source', type=Path, default=ROOT)
    checking = commands.add_parser('check', help='Execute local verifier and candidate code; normal host permissions')
    checking.add_argument('--run', required=True, type=Path)
    checking.add_argument('--timeout', type=int, default=30)
    reviewing = commands.add_parser('review', help='Record output review tied to current artifact bytes')
    reviewing.add_argument('--run', required=True, type=Path)
    reviewing.add_argument('--reviewer', required=True)
    reviewing.add_argument('--kind', choices=('human', 'agent'), required=True)
    reviewing.add_argument('--decision', choices=('accepted', 'needs_changes'), required=True)
    reviewing.add_argument('--evidence', action='append', required=True, help='File path relative to workspace')
    reviewing.add_argument('--rationale', required=True)
    reporting = commands.add_parser('report', help='Report observed checks and separately recorded reviews')
    reporting.add_argument('--run', type=Path, action='append', required=True)
    args = parser.parse_args(argv)
    if args.command == 'list':
        value = {'cases': [case for case in suite()['cases']
                           if args.domain is None or case['domain'] == args.domain]}
    elif args.command == 'prepare':
        value = prepare(args.case, args.output, args.model, args.condition,
                        args.harness_source, args.settings)
    elif args.command == 'check':
        value = check(args.run, args.timeout)
    elif args.command == 'review':
        value = review(args.run, args.reviewer, args.kind, args.decision, args.evidence, args.rationale)
    else:
        value = report(args.run)
    print(json.dumps(value, indent=2, ensure_ascii=False))
    return 1 if args.command == 'check' and value['status'] != 'passed' else 0
