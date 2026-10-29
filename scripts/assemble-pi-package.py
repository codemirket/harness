#!/usr/bin/env python3
"""Assemble an offline local pi package from an explicitly verified native payload.

No installer, package manager, downloads or host settings are invoked. The
selected verifier is explicit; native payload programs are copied, never run.
"""
import argparse
import importlib.util
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def sha(file):
    digest = hashlib.sha256()
    with file.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def assemble(verifier, source, output):
    for path in (verifier, source, output):
        if not path.is_absolute() or any(p.is_symlink() for p in (path, *path.parents)):
            raise ValueError('Select absolute paths without symlink ancestry')
    if output.exists():
        raise ValueError('Output must not exist')
    verified = json.loads(subprocess.check_output([str(verifier), 'verify', '--source', str(source)], timeout=60))
    manifest = verified['manifest']
    files = {entry['relative_path']: entry for entry in manifest['files']}
    if sha(verifier) != files['bin/mirket']['sha256']:
        raise ValueError('Verifier must be the same core as the selected payload')
    # Copy only compiled first-party extension modules. Runtime operation modules
    # and their exact dependency closure already belong to the native manifest.
    extension = ROOT / 'pi/dist/extension'
    if not (extension / 'index.js').is_file():
        raise ValueError('Build the compiled extension first')
    spec = importlib.util.spec_from_file_location('skill_assembly', ROOT / 'scripts/assemble-skills.py')
    skill_assembly = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(skill_assembly)
    skill_files = skill_assembly.inventory(ROOT / 'skills')
    output.mkdir(mode=0o700)
    for relative, data in skill_files.items():
        destination = output / 'skills' / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
    shutil.copytree(source, output / 'native', symlinks=True)
    subprocess.run([str(verifier), 'verify', '--source', str(output / 'native')], check=True, stdout=subprocess.DEVNULL, timeout=60)
    shutil.copytree(extension, output / 'dist/extension')
    (output / 'scripts').mkdir()
    shutil.copyfile(ROOT / 'pi/scripts/setup.mjs', output / 'scripts/setup.mjs')
    shutil.copyfile(ROOT / 'LICENSE', output / 'LICENSE')
    package = {'name': '@codemirket/pi-harness', 'version': manifest['version'], 'private': True,
               'type': 'module', 'engines': {'node': '>=22.19.0'},
               'peerDependencies': {'@earendil-works/pi-coding-agent': '1.4.0', 'typebox': '1.3.27'},
               'pi': {'extensions': ['./dist/extension/index.js'], 'skills': ['./skills']}}
    (output / 'package.json').write_text(json.dumps(package, indent=2) + '\n')
    shutil.copyfile(ROOT / 'packaging/admission/pi-host-files.json.proposed', output / 'host-inventory.json')
    binding = {'schema': 1, 'packageVersion': manifest['version'], 'hostVersion': '1.4.0',
               'hostInventorySha256': sha(output / 'host-inventory.json'),
               'releaseId': verified['release_id'], 'coreSha256': files['bin/mirket']['sha256'],
               'nodeSha256': files['runtime/node/bin/node']['sha256'], 'packageSha256': sha(output / 'package.json')}
    (output / 'native-binding.json').write_text(json.dumps(binding, indent=2) + '\n')
    inventory = [{'path': p.relative_to(output).as_posix(), 'bytes': p.stat().st_size, 'sha256': sha(p)}
                 for p in sorted(output.rglob('*')) if p.is_file() and not p.is_symlink()]
    (output / 'package-inventory.json').write_text(json.dumps({'schema': 1, 'files': inventory}, indent=2) + '\n')
    return {'package': str(output), 'release_id': verified['release_id'], 'file_count': len(inventory)}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mirket', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(assemble(args.mirket, args.source, args.output)))
