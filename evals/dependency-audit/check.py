#!/usr/bin/env python3
"""Audit every locked Cargo/npm/Python coordinate using public OSV metadata.

No package code is imported or installed. Unknown findings, query failures and
expired or source-stale exceptions fail closed. Images require image_audit.py.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[2]


def coordinates(root):
    packages = set()
    for item in tomllib.loads((root / 'Cargo.lock').read_text())['package']:
        if item.get('source', '').startswith('registry+'):
            packages.add(('crates.io', item['name'], item['version']))
    for path, item in json.loads((root / 'runtime/design/tools/package-lock.json').read_text())['packages'].items():
        if path and item.get('version'):
            packages.add(('npm', item.get('name', path.split('node_modules/')[-1]), item['version']))
    for line in (root / 'runtime/agent/requirements.lock').read_text().splitlines():
        if ' @ ' in line:
            name, url = line.split(' @ ', 1)
            packages.add(('PyPI', name, url.split(' ')[0].split('/')[-1].split('-')[1]))
    return sorted(packages)


def validate_exceptions(entries, root, today):
    keys = set()
    for entry in entries:
        key = (entry['ecosystem'], entry['name'], entry['version'], entry['advisory'])
        if key in keys:
            raise ValueError('duplicate advisory exception')
        keys.add(key)
        if not entry['owner'] or not entry['reason'] or datetime.date.fromisoformat(entry['expires']) <= today:
            raise ValueError('missing owner/reason or expired exception')
        source = (root / entry['source']).resolve()
        if not source.is_relative_to(root.resolve()) or hashlib.sha256(source.read_bytes()).hexdigest() != entry['source_sha256']:
            raise ValueError('exception source changed; repeat reachability review')
    return keys


def fetch(payload):
    request = urllib.request.Request('https://api.osv.dev/v1/querybatch', data=json.dumps(payload).encode(),
                                     headers={'Content-Type': 'application/json', 'User-Agent': 'Mirket-dependency-audit/1'})
    with urllib.request.urlopen(request, timeout=45) as response:
        return json.load(response)


def audit(packages, exceptions, query=fetch):
    results = []
    untriaged = []
    for offset in range(0, len(packages), 100):
        batch = packages[offset:offset + 100]
        reply = query({'queries': [{'package': {'ecosystem': e, 'name': n}, 'version': v} for e, n, v in batch]})
        if len(reply['results']) != len(batch):
            raise ValueError('incomplete OSV batch response')
        for (eco, name, version), item in zip(batch, reply['results']):
            if not isinstance(item, dict) or 'error' in item:
                raise ValueError('OSV package query failed')
            for advisory in item.get('vulns', []):
                key = (eco, name, version, advisory['id'])
                finding = {'ecosystem': eco, 'name': name, 'version': version, 'advisory': advisory['id'], 'excepted': key in exceptions}
                results.append(finding)
                if key not in exceptions:
                    untriaged.append(finding)
    return {'queried': len(packages), 'findings': results, 'untriaged': untriaged, 'ok': not untriaged}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    today = datetime.date.today()
    entries = json.loads((ROOT / 'evals/dependency-audit/exceptions.json').read_text())
    keys = validate_exceptions(entries, ROOT, today)
    report = audit(coordinates(ROOT), keys)
    report.update(date=today.isoformat(),
                  coverage={'cargo':True,'owned_npm':True,'agent_python':True,
                            'openpencil_locked_coordinates':sum(1 for e,n,v in coordinates(ROOT) if n.startswith('@open-pencil/')),
                            'os_and_images':'separate image_audit.py receipts required'},
                  limits='Advisory matching is not exploitability analysis. OpenPencil and transitive packages are covered by the design npm lock; native/WASM and OS payloads also require image scans.')
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    return 0 if report['ok'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
