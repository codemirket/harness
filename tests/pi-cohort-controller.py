#!/usr/bin/env python3
"""Run the frozen matched cohort in new disposable state; stop on provider errors."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser(description=__doc__)
for name in ('node', 'peers', 'state-root', 'cohort-root', 'auth-file', 'catalog', 'browser-peers', 'chrome', 'trace', 'spec'):
    parser.add_argument('--' + name, type=Path, required=True)
parser.add_argument('--authorized-auth-refresh', action='store_true', help='Only after explicit user permission for pi to persist normal refreshes to the selected auth store')
a = parser.parse_args()
repo = Path(__file__).resolve().parents[1]
assert all(value.is_absolute() for value in vars(a).values() if isinstance(value, Path))
assert str(a.state_root).startswith('/private/tmp/mirket-pi-package-')
assert a.cohort_root.is_relative_to(a.state_root)
a.cohort_root.mkdir(mode=0o700)
source = a.spec.read_bytes()
spec = json.loads(source)
if spec.get('expansionOf'):
    original = Path(spec['expansionOf'])
    assert original.is_relative_to(a.state_root) and original.is_file()
    assert spec['limits']['expansionPairs'] == len(spec['tasks']) <= 4
    assert spec['limits']['authorRuns'] == 2 * len(spec['tasks'])
    assert json.loads(original.read_text())['limits']['authorRuns'] == 24
else:
    assert len(spec['tasks']) == 12 and spec['limits']['authorRuns'] == 24
(a.cohort_root / 'frozen.json').write_bytes(source)
files = ['tests/pi-cohort-run.mjs', 'tests/pi-cohort-render.mjs', 'tests/pi-cohort-oracle.mjs']
archive = a.cohort_root / 'frozen-runner'
archive.mkdir(mode=0o700)
identities = {}
for name in files:
    data = (repo / name).read_bytes()
    identities[name] = hashlib.sha256(data).hexdigest()
    (archive / Path(name).name).write_bytes(data)
order = []
for index, task in enumerate(spec['tasks']):
    for condition in (['vanilla', 'mirket'] if index % 2 == 0 else ['mirket', 'vanilla']):
        order.append([task['id'], condition])
manifest = dict(specSha256=hashlib.sha256(source).hexdigest(), frozenAt=time.time(), runnerFiles=identities,
                nativeBinding=json.loads((a.state_root / 'package/native-binding.json').read_text()),
                model=spec['model'], limits=spec['limits'], order=order,
                authRefresh='user-authorized-pi-store' if a.authorized_auth_refresh else 'read-only')
(a.cohort_root / 'manifest.json').write_text(json.dumps(manifest, indent=2))
queue, active, results, blocked = list(order), [], [], False
try:
    while queue or active:
        # First real author request also checks provider access before dispatching
        # the remaining cells. A normal turn with poor output remains a result.
        concurrency = 2 if results else 1
        while queue and not blocked and len(active) < concurrency:
            task, condition = queue.pop(0)
            log = (a.cohort_root / (task + '-' + condition + '.log')).open('w')
            env = {**os.environ, 'PI_REPAIR_TRACE': str(a.cohort_root / (task + '-' + condition + '-modules.jsonl'))}
            env.pop('PI_COHORT_PREFLIGHT', None)
            env.pop('MIRKET_COHORT_AUTH_REFRESH', None)
            if a.authorized_auth_refresh:
                env['MIRKET_COHORT_AUTH_REFRESH'] = 'authorized'
            args = [str(a.node), '--import', str(a.trace), str(repo / files[0]), str(a.peers), str(a.state_root),
                    str(a.cohort_root), task, condition, str(a.auth_file), str(a.catalog), str(a.browser_peers), str(a.chrome)]
            child = subprocess.Popen(args, cwd=repo, env=env, stdout=log, stderr=log)
            active.append((child, task, condition, log, time.monotonic()))
            print('START', task, condition, child.pid, flush=True)
        for entry in list(active):
            child, task, condition, log, start = entry
            if child.poll() is None:
                continue
            log.close()
            path = a.cohort_root / (task + '-' + condition) / 'report.json'
            report = json.loads(path.read_text()) if path.exists() else {'status': 'infrastructure_failure'}
            result = dict(task=task, condition=condition, exit=child.returncode, elapsed=time.monotonic()-start, status=report['status'])
            results.append(result)
            active.remove(entry)
            print('END', json.dumps(result), flush=True)
            if child.returncode or report['status'] in ('failed', 'infrastructure_failure'):
                blocked = True
            (a.cohort_root / 'controller.json').write_text(json.dumps({'results': results, 'undispatched': queue, 'blocked': blocked}, indent=2))
        if blocked and not active:
            break
        time.sleep(1)
finally:
    for child, _, _, log, _ in active:
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=5)
        log.close()
print('BLOCKED' if blocked else 'AUTHOR_COHORT_COMPLETE', a.cohort_root, flush=True)
raise SystemExit(1 if blocked else 0)
