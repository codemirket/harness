#!/usr/bin/env python3
"""Derive the exact test-candidate host identity from already reviewed archives.

Offline data inspection only. Does not install packages or confer admission.
"""
import argparse
import base64
import hashlib
import json
from pathlib import Path, PurePosixPath
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def freeze(archives, repaired_undici, output):
    inventory = json.loads((ROOT/'packaging/admission/pi-p0.inventory.json.proposed').read_text())
    lock_file = ROOT/'packaging/admission/pi-host-repair-package-lock.json.proposed'
    lock = json.loads(lock_file.read_text())
    files = []
    for package in inventory['records']:
        entry = lock['packages'][package['path']]
        archive = repaired_undici if package['path'] == 'node_modules/undici' else archives/(hashlib.sha256(package['resolved'].encode()).hexdigest()+'.tgz')
        data = archive.read_bytes()
        kind, digest = entry['integrity'].split('-', 1)
        if base64.b64encode(hashlib.new(kind, data).digest()).decode() != digest:
            raise ValueError(f'Archive integrity differs: {package["path"]}')
        with tarfile.open(archive) as tar:
            roots = {PurePosixPath(member.name).parts[0] for member in tar.getmembers()}
            if len(roots) != 1:
                raise ValueError("Archive must have one package root")
            for member in tar:
                path = PurePosixPath(member.name)
                if not path.parts or path.is_absolute() or '..' in path.parts:
                    raise ValueError(f'Unsafe archive path: {package["path"]}: {member.name}')
                if member.isdir():
                    continue
                if not member.isfile():
                    raise ValueError('Archive links/special files are not admitted')
                relative = (PurePosixPath(package['path']).relative_to('node_modules')/PurePosixPath(*path.parts[1:])).as_posix()
                content = tar.extractfile(member).read()
                files.append({'path': relative, 'bytes': len(content), 'sha256': hashlib.sha256(content).hexdigest()})
        # npm's .bin links are non-importable command conveniences; never invoke them.
    unique = {}
    for file in files:
        if file['path'] in unique and unique[file['path']] != file:
            raise ValueError('Overlapping archive members differ')
        unique[file['path']] = file
    result = {'schema': 1, 'status': 'test-candidate-only-not-production-admitted',
              'hostVersion': '1.4.0', 'entry': '@earendil-works/pi-coding-agent/dist/cli.js',
              'lockSha256': hashlib.sha256(lock_file.read_bytes()).hexdigest(),
              'archiveMemberCount': len(files),
              'files': sorted(unique.values(), key=lambda f: f['path'])}
    output.write_text(json.dumps(result, separators=(',', ':'))+'\n')
    return {'files': len(unique), 'sha256': hashlib.sha256(output.read_bytes()).hexdigest()}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archives', type=Path, required=True)
    parser.add_argument('--repaired-undici', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(freeze(args.archives, args.repaired_undici, args.output)))
