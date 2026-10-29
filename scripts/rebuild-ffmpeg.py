#!/usr/bin/env python3
"""Replay the selected macOS ARM64 media build from the delivered sources.

This is an explicit developer build, never an installation hook. It executes
upstream configure/Makefiles only when the recipient invokes this command.
The recipe transcribes the preserved configuration logs; a fresh replay and
byte-reproducibility comparison have not been run for this recipe.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import tarfile


def extract(archive, output, strip_root):
    output.mkdir()
    with tarfile.open(archive) as source:
        seen = set()
        for member in source:
            path = PurePosixPath(member.name)
            if path.is_absolute() or '..' in path.parts or not (member.isfile() or member.isdir()):
                raise ValueError('Unexpected source archive member')
            parts = path.parts[1:] if strip_root else path.parts
            if not parts:
                continue
            name = '/'.join(parts)
            if name in seen:
                raise ValueError('Duplicate source member')
            seen.add(name)
            destination = output.joinpath(*parts)
            if member.isdir():
                destination.mkdir(parents=True, exist_ok=True)
            else:
                destination.parent.mkdir(parents=True, exist_ok=True)
                with source.extractfile(member) as reader, destination.open('xb') as writer:
                    shutil.copyfileobj(reader, writer)
                destination.chmod(member.mode & 0o777)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--packet', type=Path, required=True, help='Delivered third-party/ffmpeg directory')
    parser.add_argument('--output', type=Path, required=True, help='New disposable build directory')
    for name in ('cc', 'ar', 'ranlib', 'strip', 'make', 'pkg-config', 'sdk'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--jobs', type=int, default=2)
    args = parser.parse_args()
    for path in [args.packet, args.output, args.cc, args.ar, args.ranlib, args.strip, args.make, args.pkg_config, args.sdk]:
        if not path.is_absolute() or path.resolve() != path:
            raise ValueError('Use explicit canonical absolute paths')
    if args.output.exists() or not 1 <= args.jobs <= 64:
        raise ValueError('Output must be absent; jobs must be 1..64')
    pins = json.loads((args.packet / 'source-files.json').read_text())
    for entry in pins['files']:
        path = args.packet / entry['path']
        if path.is_symlink() or not path.is_file() or path.stat().st_size != entry['bytes'] or hashlib.sha256(path.read_bytes()).hexdigest() != entry['sha256']:
            raise ValueError('Source packet differs: ' + str(path))
    args.output.mkdir(mode=0o700)
    prefix = args.output / 'prefix'
    flags = f'-O2 -arch arm64 -isysroot {args.sdk} -mmacosx-version-min=26.0'
    link = f'-arch arm64 -isysroot {args.sdk} -mmacosx-version-min=26.0'
    env = {**os.environ, 'PATH': '/usr/bin:/bin', 'CC': str(args.cc), 'AR': str(args.ar),
           'RANLIB': str(args.ranlib), 'STRIP': str(args.strip), 'CFLAGS': flags,
           'LDFLAGS': link, 'MACOSX_DEPLOYMENT_TARGET': '26.0',
           'PKG_CONFIG_PATH': str(prefix / 'lib/pkgconfig'), 'PKG_CONFIG_LIBDIR': str(prefix / 'lib/pkgconfig')}
    archives = [('x264', 'x264-0480cb05fa188d37ae87e8f4fd8f1aea3711f7ee.tar.gz', True),
                ('libvpx', 'libvpx-6df3ec34557879fff673706f4a1d9fbd0f3a6f0e.tar.gz', False),
                ('opus', 'opus-1.6.1.tar.gz', True), ('ffmpeg', 'ffmpeg-9.0.2.tar.xz', True)]
    commands = {
        'x264': [f'--prefix={prefix}', '--enable-static', '--disable-cli', '--disable-opencl', '--disable-avs', '--disable-swscale', '--disable-lavf', '--disable-ffms', '--disable-gpac', '--disable-lsmash', '--host=aarch64-apple-darwin', f'--sysroot={args.sdk}'],
        'libvpx': ['--target=arm64-darwin25-gcc', f'--prefix={prefix}', '--disable-examples', '--disable-tools', '--disable-docs', '--disable-unit-tests', '--disable-install-bins', '--disable-install-docs', '--disable-shared', '--enable-pic', f'--extra-cflags={flags}'],
        'opus': [f'--prefix={prefix}', '--disable-shared', '--enable-static', '--disable-doc', '--disable-extra-programs'],
        'ffmpeg': [f'--prefix={prefix}', f'--cc={args.cc}', f'--host-cc={args.cc}', f'--host-cflags={flags}', f'--host-ldflags={link}', f'--ld={args.cc}', f'--ar={args.ar}', f'--ranlib={args.ranlib}', f'--strip={args.strip}', f'--pkg-config={args.pkg_config}', '--pkg-config-flags=--static', f'--extra-cflags={flags} -I{prefix}/include', f'--extra-ldflags={link} -L{prefix}/lib', '--enable-gpl', '--enable-libx264', '--enable-libvpx', '--enable-libopus', '--enable-static', '--disable-shared', '--disable-autodetect', '--disable-network', '--disable-devices', '--disable-hwaccels', '--disable-avfoundation', '--disable-audiotoolbox', '--disable-videotoolbox', '--disable-doc', '--disable-debug', '--disable-ffplay', '--enable-zlib'],
    }
    for name, archive, strip_root in archives:
        work = args.output / name
        extract(args.packet / 'source' / archive, work, strip_root)
        for command in [['/bin/sh', './configure', *commands[name]], [str(args.make), f'-j{args.jobs}'], [str(args.make), 'install']]:
            subprocess.run(command, cwd=work, env=env, check=True)
    print(prefix / 'bin')


if __name__ == '__main__':
    main()
