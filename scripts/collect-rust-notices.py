#!/usr/bin/env python3
"""Collect original notices from the exact locked macOS build input archives.

Offline inspection only: Cargo metadata does not compile dependencies. No
upstream code, installers or hooks execute. Identical notice bytes are stored
once and mapped back to every original crate/archive member.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def collect():
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    checksums = {(p['name'], p['version']): p.get('checksum') for p in lock['package']}
    metadata = json.loads(subprocess.check_output([
        'cargo', 'metadata', '--locked', '--offline', '--format-version', '1',
        '--filter-platform', 'aarch64-apple-darwin'], cwd=ROOT))
    nodes = {n['id']: n for n in metadata['resolve']['nodes']}
    pending, reachable = [metadata['resolve']['root']], set()
    while pending:
        selected = pending.pop()
        if selected in reachable:
            continue
        reachable.add(selected)
        pending.extend(d['pkg'] for d in nodes[selected]['deps'])
    packages = [p for p in metadata['packages'] if p['id'] in reachable and p['source']]
    output = ROOT / 'packaging/notices'
    receipt = json.loads((output / 'sources.json').read_text())
    receipt['files'] = [entry for entry in receipt['files'] if not entry['path'].startswith('rust/')]
    texts, records = {}, []
    for package in sorted(packages, key=lambda p: (p['name'], p['version'])):
        source = Path(package['manifest_path']).parent
        archive = source.parent.parent.parent / 'cache' / source.parent.name / (source.name + '.crate')
        archive_hash = hashlib.sha256(archive.read_bytes()).hexdigest()
        if archive_hash != checksums[(package['name'], package['version'])]:
            raise ValueError('Registry archive differs from Cargo.lock: ' + package['name'])
        notices = []
        with tarfile.open(archive) as crate:
            for member in crate:
                if not member.isfile() or not re.match(r'^(LICENSE|LICENCE|COPYING|COPYRIGHT|NOTICE|UNLICENSE)([.\-_]|$)', member.name.rsplit('/', 1)[-1], re.I):
                    continue
                data = crate.extractfile(member).read()
                digest = hashlib.sha256(data).hexdigest()
                relative = 'rust/texts/' + digest + '.txt'
                texts[relative] = (data, 'https://crates.io/api/v1/crates/' + package['name'] + '/' + package['version'] + '/download', member.name)
                notices.append({'archive_member': member.name, 'text': relative, 'sha256': digest})
        if not notices:
            raise ValueError('No original notice text found: ' + package['name'])
        records.append({'name': package['name'], 'version': package['version'], 'license_declared': package['license'],
                        'archive_sha256': archive_hash, 'notices': notices})
    for relative, (data, url, member) in sorted(texts.items()):
        path = output / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        receipt['files'].append({'path': relative, 'source': url, 'archive_member': member,
                                 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
    data = (json.dumps({'schema': 1, 'target': 'aarch64-apple-darwin',
                       'scope': 'Full reachable locked target build-input closure, including build dependencies and notices from uncompiled archive source members; not a binary symbol/feature attribution claim.',
                       'cargo_lock_sha256': hashlib.sha256((ROOT / 'Cargo.lock').read_bytes()).hexdigest(),
                       'packages': records}, indent=2) + '\n').encode()
    (output / 'rust/closure.json').write_bytes(data)
    receipt['files'].append({'path': 'rust/closure.json', 'source': 'Cargo.lock and checksum-verified registry source archives',
                             'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()})
    (output / 'sources.json').write_text(json.dumps(receipt, indent=2) + '\n')
    return {'packages': len(records), 'original_notice_members': sum(len(p['notices']) for p in records), 'unique_texts': len(texts)}


if __name__ == '__main__':
    print(json.dumps(collect()))
