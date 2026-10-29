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


def runtime_setup():
    """Describe only tools supported by the first-party setup implementation."""
    chrome = json.loads((ROOT / 'packaging/runtime/chrome-macos.json').read_text())
    return {'schema': 1, 'tools': [
        {'id': 'chromium', 'title': 'Chrome for video rendering', 'version': chrome['version'],
         'target': chrome['target'],
         'terms': 'Download directly from Google for your own use or select your independently obtained executable. Google terms apply: https://www.google.com/chrome/terms/ . This exact archive includes proprietary Widevine; Mirket does not redistribute Chrome.',
         'archive': chrome['archive'], 'format': 'zip', 'executable': chrome['executable'],
         'files': chrome['files'], 'links': chrome['links']},
    ]}


def copy_media_sources(source, files, output, ffmpeg_source_root):
    media = {'runtime/video/bin/ffmpeg', 'runtime/video/bin/ffprobe'}
    present = media & files.keys()
    if not present:
        return
    if present != media or ffmpeg_source_root is None:
        raise ValueError('Bundled FFmpeg requires the complete selected pair and --ffmpeg-source-root')
    root = ffmpeg_source_root
    if not root.is_absolute() or root.resolve() != root or root.is_symlink() or not root.is_dir():
        raise ValueError('Select a canonical absolute FFmpeg source packet directory')
    pins = json.loads((ROOT / 'packaging/runtime/ffmpeg-source.json').read_text())
    for entry in pins['binaries']:
        selected = 'runtime/video/' + entry['path']
        if files[selected]['sha256'] != entry['sha256'] or (source / selected).stat().st_size != entry['bytes']:
            raise ValueError('Bundled media differs from corresponding-source identity')
    prepared = []
    for entry in pins['files']:
        path = root / entry['path']
        if any(p.is_symlink() for p in (path, *path.parents)) or not path.is_file() or path.stat().st_size != entry['bytes'] or sha(path) != entry['sha256']:
            raise ValueError('FFmpeg source/notice/build file differs: ' + str(path))
        prepared.append((path, entry['path']))
    target = output / 'third-party/ffmpeg'
    target.mkdir(parents=True)
    for path, relative in prepared:
        destination = target / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
    shutil.copyfile(ROOT / 'packaging/runtime/ffmpeg-source.json', target / 'source-files.json')
    shutil.copyfile(ROOT / 'scripts/rebuild-ffmpeg.py', target / 'rebuild-ffmpeg.py')
    shutil.copyfile(ROOT / 'packaging/ffmpeg-source.md', target / 'README.md')


def copy_video_sources(files, output, video_source_root):
    if not any(path.startswith('runtime/video/node_modules/') for path in files):
        return
    if video_source_root is None:
        raise ValueError('Bundled Video requires --video-source-root with exact sources and notices')
    root = video_source_root
    if not root.is_absolute() or root.resolve() != root or not root.is_dir():
        raise ValueError('Select a canonical absolute Video source packet')
    pin = json.loads((ROOT / 'packaging/runtime/video-source.json').read_text())
    if (root / 'MANIFEST.json').is_symlink() or sha(root / 'MANIFEST.json') != pin['packet_manifest_sha256']:
        raise ValueError('Video source packet manifest differs')
    entries = json.loads((root / 'MANIFEST.json').read_text())['files']
    target = output / 'third-party/video'
    selected = []
    for entry in entries:
        relative = entry['path']
        if relative.startswith('/') or any(part in ('', '.', '..') for part in relative.split('/')):
            raise ValueError('Unsafe source packet path')
        if not any(relative.startswith(prefix) for prefix in pin['include']):
            continue
        path = root / relative
        if any(p.is_symlink() for p in (path, *path.parents)) or not path.is_file() or path.stat().st_size != entry['bytes'] or sha(path) != entry['sha256']:
            raise ValueError('Video source or notice differs: ' + relative)
        selected.append((path, relative, entry))
    for path, relative, _ in selected:
        destination = target / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
    (target / 'source-files.json').write_text(json.dumps({'schema': 1, 'files': [entry for _, _, entry in selected]}, indent=2) + '\n')


def copy_notices(output):
    root = ROOT / 'packaging/notices'
    pin = json.loads((root / 'sources.json').read_text())
    for entry in pin['files']:
        path = root / entry['path']
        if path.is_symlink() or path.stat().st_size != entry['bytes'] or sha(path) != entry['sha256']:
            raise ValueError('Supplemental notice differs: ' + entry['path'])
    target = output / 'third-party/supplemental'
    target.mkdir(parents=True, exist_ok=True)
    for entry in pin['files']:
        destination = target / entry['path']
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / entry['path'], destination)
    for name in ('README.md', 'sources.json'):
        shutil.copyfile(root / name, target / name)
    shutil.copyfile(ROOT / 'packaging/THIRD-PARTY.md', output / 'THIRD-PARTY.md')
    shutil.copyfile(ROOT / 'LICENSE', output / 'LICENSE')


def assemble(verifier, source, output, ffmpeg_source_root=None, video_source_root=None):
    for path in (verifier, source, output):
        if not path.is_absolute() or any(p.is_symlink() for p in (path, *path.parents)):
            raise ValueError('Select absolute paths without symlink ancestry')
    if output.exists():
        raise ValueError('Output must not exist')
    verified = json.loads(subprocess.check_output([str(verifier), 'verify', '--source', str(source)], timeout=60))
    manifest = verified['manifest']
    files = {entry['relative_path']: entry for entry in manifest['files']}
    if any(path.startswith('runtime/design/raster/') for path in files):
        raise ValueError('Raster is excluded until its native source/notice obligations are resolved')
    if sha(verifier) != files['bin/mirket']['sha256']:
        raise ValueError('Verifier must be the same core as the selected payload')
    has_video = any(path.startswith('runtime/video/node_modules/') for path in files)
    if has_video and video_source_root is None:
        raise ValueError('Bundled Video requires --video-source-root; source/notice delivery cannot be omitted')
    if ffmpeg_source_root is None and video_source_root is not None:
        ffmpeg_source_root = video_source_root / 'source/ffmpeg'
    if {'runtime/video/bin/ffmpeg', 'runtime/video/bin/ffprobe'} & files.keys() and ffmpeg_source_root is None:
        raise ValueError('Bundled FFmpeg requires --ffmpeg-source-root; source delivery cannot be omitted')
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
    copy_media_sources(source, files, output, ffmpeg_source_root)
    copy_video_sources(files, output, video_source_root)
    copy_notices(output)
    package = {'name': '@codemirket/pi-harness', 'version': manifest['version'], 'private': True,
               'type': 'module', 'engines': {'node': '>=22.19.0'},
               'peerDependencies': {'@earendil-works/pi-coding-agent': '1.4.0', 'typebox': '1.3.27'},
               'pi': {'extensions': ['./dist/extension/index.js'], 'skills': ['./skills']}}
    (output / 'package.json').write_text(json.dumps(package, indent=2) + '\n')
    shutil.copyfile(ROOT / 'packaging/runtime/host-files.json', output / 'host-inventory.json')
    (output / 'runtime-setup.json').write_text(json.dumps(runtime_setup(), indent=2) + '\n')
    binding = {'schema': 1, 'packageVersion': manifest['version'], 'hostVersion': '1.4.0',
               'hostInventorySha256': sha(output / 'host-inventory.json'),
               'runtimeSetupSha256': sha(output / 'runtime-setup.json'),
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
    parser.add_argument('--ffmpeg-source-root', type=Path, help='Exact FFmpeg source/notice/build packet; required with Video')
    parser.add_argument('--video-source-root', type=Path, help='Exact Video source/notice packet (also supplies source/ffmpeg)')
    args = parser.parse_args()
    print(json.dumps(assemble(args.mirket, args.source, args.output, args.ffmpeg_source_root, args.video_source_root)))
