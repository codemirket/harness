#!/usr/bin/env python3
"""Audit every locked Cargo/npm coordinate using public OSV metadata.

No package code is imported or installed. Advisories are reported by default;
query/inventory failures remain errors. Images require image_audit.py.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import tomllib
import urllib.request
import urllib.parse

ROOT = Path(__file__).resolve().parents[2]


def coordinates(root):
    packages = set()
    for lock in (root / 'Cargo.lock', root / 'runtime/agent/native/Cargo.lock'):
        for item in tomllib.loads(lock.read_text())['package']:
            if item.get('source', '').startswith('registry+'):
                packages.add(('crates.io', item['name'], item['version']))
    for lock in sorted((root / 'runtime/design').glob('*/package-lock.json')):
        for path, item in json.loads(lock.read_text())['packages'].items():
            if path and item.get('version') and not path.startswith('../'):
                packages.add(('npm', item.get('name', path.split('node_modules/')[-1]), item['version']))
    # The 0.0.1 agent-runtime has no Python production closure. Native ONNX
    # members and model assets are audited by the package/image receipts.
    return sorted(packages)


def validate_exceptions(entries, root, today, stale=None):
    keys = set()
    seen = set()
    for entry in entries:
        key = (entry['ecosystem'], entry['name'], entry['version'], entry['advisory'])
        if key in seen:
            raise ValueError('duplicate advisory exception')
        seen.add(key)
        if not entry['owner'] or not entry['reason']:
            raise ValueError('missing owner/reason')
        expired = datetime.date.fromisoformat(entry['expires']) <= today
        if 'package_checksum' in entry:
            if entry['ecosystem'] != 'crates.io':
                raise ValueError('checksum disposition requires registry crate')
            selected = [p for lock in (root / 'Cargo.lock', root / 'runtime/agent/native/Cargo.lock')
                        for p in tomllib.loads(lock.read_text())['package']
                        if p['name'] == entry['name'] and p['version'] == entry['version']]
            changed = not selected or any(p.get('checksum') != entry['package_checksum'] for p in selected)
        else:
            source = (root / entry['source']).resolve()
            if not source.is_relative_to(root.resolve()):
                raise ValueError('exception source outside repository')
            changed = not source.is_file() or hashlib.sha256(source.read_bytes()).hexdigest() != entry['source_sha256']
        if expired or changed:
            reason = 'exception expired' if expired else 'exception source changed; repeat reachability review'
            if stale is None:
                raise ValueError(reason)
            stale.append({'advisory': entry['advisory'], 'source': entry.get('source', 'registry artifact checksum'), 'reason': reason})
        else:
            keys.add(key)
    return keys


def fetch(payload):
    request = urllib.request.Request('https://api.osv.dev/v1/querybatch', data=json.dumps(payload).encode(),
                                     headers={'Content-Type': 'application/json', 'User-Agent': 'Mirket-dependency-audit/1'})
    with urllib.request.urlopen(request, timeout=45) as response:
        return json.load(response)


def audit(packages, exceptions, query=fetch, advisory_policy='report'):
    if advisory_policy not in ('report', 'fail'):
        raise ValueError('unknown advisory policy')
    if not packages or any(not all(isinstance(v, str) and v.strip() for v in p) or len(p) != 3 for p in packages):
        raise ValueError('missing or malformed package inventory')
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
            vulnerabilities = item.get('vulns', [])
            if not isinstance(vulnerabilities, list):
                raise ValueError('malformed OSV vulnerabilities')
            for advisory in vulnerabilities:
                if not isinstance(advisory, dict) or not isinstance(advisory.get('id'), str) or not advisory['id'].strip():
                    raise ValueError('malformed OSV advisory')
                key = (eco, name, version, advisory['id'])
                finding = {'ecosystem': eco, 'name': name, 'version': version, 'advisory': advisory['id'], 'excepted': key in exceptions}
                results.append(finding)
                if key not in exceptions:
                    untriaged.append(finding)
    return {'queried': len(packages), 'findings': results, 'untriaged': untriaged,
            'advisory_policy': advisory_policy, 'advisories_present': bool(results),
            'ok': advisory_policy == 'report' or not untriaged}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--advisory-policy', choices=('report', 'fail'), default='report',
                        help='Report advisories without blocking; query and inventory errors still fail')
    args = parser.parse_args()
    today = datetime.date.today()
    entries = json.loads((ROOT / 'evals/dependency-audit/exceptions.json').read_text())
    stale = []
    keys = validate_exceptions(entries, ROOT, today, stale if args.advisory_policy == 'report' else None)
    packages = coordinates(ROOT)
    report = audit(packages, keys, advisory_policy=args.advisory_policy)
    # Batch results only contain IDs. Preserve the primary advisory documents
    # and publisher severity instead of silently treating every ID as equivalent.
    advisories = {}
    for advisory_id in sorted({f['advisory'] for f in report['findings']}):
        url = 'https://api.osv.dev/v1/vulns/' + urllib.parse.quote(advisory_id, safe='')
        with urllib.request.urlopen(url, timeout=45) as response:
            advisory = json.load(response)
        if advisory.get('id') != advisory_id:
            raise ValueError('advisory identity mismatch')
        advisories[advisory_id] = advisory
    report['advisories'] = advisories
    for finding in report['findings']:
        advisory = advisories[finding['advisory']]
        severity = advisory.get('database_specific', {}).get('severity', 'UNKNOWN').upper()
        finding['severity'] = severity if severity in ('LOW', 'MODERATE', 'MEDIUM', 'HIGH', 'CRITICAL') else 'UNKNOWN'
        if finding['severity'] == 'MODERATE': finding['severity'] = 'MEDIUM'
    report['lockfiles'] = {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                         for p in [ROOT/'Cargo.lock', ROOT/'runtime/agent/native/Cargo.lock',
                                   *sorted((ROOT/'runtime/design').glob('*/package-lock.json'))]}
    report['stale_reviews'] = stale
    report.update(date=today.isoformat(),
                  coverage={'cargo':True,'owned_npm':True,'agent_python':False,
                            'openpencil_locked_coordinates':sum(1 for e,n,v in packages if n.startswith('@open-pencil/')),
                            'os_and_images':'separate image_audit.py receipts required'},
                  limits='Report policy does not mean advisories are fixed or safe. Stale reviews never suppress findings. Advisory matching is not exploitability analysis. OpenPencil and transitive packages are covered by the design npm lock; native/WASM and OS payloads, including the ONNX members, require separate image and artifact scans.')
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    return 0 if report['ok'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
