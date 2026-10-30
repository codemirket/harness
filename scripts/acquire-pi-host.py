#!/usr/bin/env python3
"""Explicitly acquire the package-bound external Pi host; never run npm/hooks.

Downloads only the lock's exact npm registry archives. --archive-dir instead
reads SHA-256(URL).tgz files from an existing offline cache without changing it.
The absent output receives node_modules only after complete native verification.
"""
import argparse
import base64
import ctypes
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import urllib.parse
import urllib.request

MAX_ARCHIVE_BYTES = 256 * 1024 * 1024


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(path, exists=True):
    if not path.is_absolute() or any(p.is_symlink() for p in (path, *path.parents)) or path.resolve() != path:
        raise ValueError('Select canonical absolute paths without symlink ancestry')
    if exists and not path.exists():
        raise ValueError('Selected input does not exist: ' + str(path))


def regular(path, limit=MAX_ARCHIVE_BYTES):
    canonical(path)
    mode = path.lstat()
    if not stat.S_ISREG(mode.st_mode) or mode.st_nlink != 1 or mode.st_size > limit:
        raise ValueError('Input must be a bounded, unaliased regular file: ' + str(path))
    return path.read_bytes()


def relative_parts(name):
    parts = name.split('/')
    if not name or '\\' in name or any(part in ('', '.', '..') for part in parts):
        raise ValueError('Unsafe path: ' + name)
    return parts


def registry_url(url):
    parsed = urllib.parse.urlsplit(url)
    if (parsed.scheme != 'https' or parsed.netloc != 'registry.npmjs.org'
            or parsed.query or parsed.fragment or not parsed.path.endswith('.tgz')):
        raise ValueError('Only exact HTTPS npm registry archive URLs are allowed')
    return url


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, url):
        raise ValueError('Archive redirects are refused')


def archive_bytes(url, archive_dir):
    registry_url(url)
    if archive_dir is not None:
        return regular(archive_dir / (sha(url.encode()) + '.tgz'))
    opener = urllib.request.build_opener(NoRedirect())
    with opener.open(url, timeout=60) as response:
        data = response.read(MAX_ARCHIVE_BYTES + 1)
    if len(data) > MAX_ARCHIVE_BYTES:
        raise ValueError('Archive exceeds size bound')
    return data


def extract_archive(data, integrity, package_path, target, expected, observed):
    algorithm, encoded = integrity.split('-', 1)
    if algorithm not in ('sha256', 'sha384', 'sha512'):
        raise ValueError('Unsupported archive integrity algorithm')
    digest = base64.b64decode(encoded, validate=True)
    if hashlib.new(algorithm, data).digest() != digest:
        raise ValueError('Archive integrity differs: ' + package_path)
    placement = relative_parts(package_path)
    if placement[0] != 'node_modules' or len(placement) < 2:
        raise ValueError('Invalid locked package placement')
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
        roots = set()
        for member in archive:
            name = member.name.rstrip('/') if member.isdir() else member.name
            # Published npm archives include harmless internal ./ components.
            # Normalize those only; never normalize traversal or empty segments.
            raw_parts = name.split('/')
            if '\\' in name or any(part in ('', '..') for part in raw_parts):
                raise ValueError('Unsafe archive path: ' + name)
            parts = [part for part in raw_parts if part != '.']
            if not parts:
                raise ValueError('Unsafe archive path: ' + name)
            roots.add(parts[0])
            if len(roots) != 1:
                raise ValueError('Archive must have exactly one package root')
            if member.isdir():
                continue
            if not member.isfile() or len(parts) < 2 or member.issparse():
                raise ValueError('Archive links/special files are refused')
            relative = '/'.join(placement[1:] + parts[1:])
            entry = expected.get(relative)
            if entry is None or member.size != entry['bytes']:
                raise ValueError('Archive member differs from host inventory: ' + relative)
            content = archive.extractfile(member).read()
            if len(content) != entry['bytes'] or sha(content) != entry['sha256']:
                raise ValueError('Archive member bytes differ from host inventory: ' + relative)
            destination = target.joinpath(*relative_parts(relative))
            if relative in observed:
                if regular(destination) != content:
                    raise ValueError('Overlapping archive members differ: ' + relative)
                continue
            destination.parent.mkdir(parents=True, mode=0o700, exist_ok=True)
            with destination.open('xb') as output:
                output.write(content)
            destination.chmod(0o600 | (member.mode & 0o111))
            observed.add(relative)
        if not roots:
            raise ValueError('Empty package archive')


def publish(source, destination):
    """Exclusive atomic rename: an existing user directory is never replaced."""
    libc = ctypes.CDLL(None, use_errno=True)
    if sys.platform == 'darwin':
        rename = libc.renamex_np
        rename.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint]
        result = rename(os.fsencode(source), os.fsencode(destination), 4)  # RENAME_EXCL
    elif sys.platform.startswith('linux') and hasattr(libc, 'renameat2'):
        rename = libc.renameat2
        rename.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
        result = rename(-100, os.fsencode(source), -100, os.fsencode(destination), 1)  # RENAME_NOREPLACE
    else:
        raise ValueError('Exclusive atomic publication requires macOS or Linux rename support')
    if result != 0:
        number = ctypes.get_errno()
        raise OSError(number, os.strerror(number), str(destination))


def acquire(package, lock_path, output, archive_dir=None):
    for path in (package, lock_path, output):
        canonical(path, exists=path != output)
    if output.exists() or not output.parent.is_dir():
        raise ValueError('Output must be absent with an existing canonical parent')
    if archive_dir is not None:
        canonical(archive_dir)
        if not archive_dir.is_dir():
            raise ValueError('Archive cache must be a directory')
    binding = json.loads(regular(package / 'native-binding.json'))
    inventory_path = package / 'host-inventory.json'
    inventory_bytes = regular(inventory_path, 8 * 1024 * 1024)
    if sha(inventory_bytes) != binding['hostInventorySha256']:
        raise ValueError('Host inventory differs from native binding')
    inventory = json.loads(inventory_bytes)
    lock_bytes = regular(lock_path, 8 * 1024 * 1024)
    if sha(lock_bytes) != inventory['lockSha256']:
        raise ValueError('Lock differs from package-bound host inventory')
    expected = {}
    for entry in inventory['files']:
        relative_parts(entry['path'])
        if entry['path'] in expected or not isinstance(entry['bytes'], int) or entry['bytes'] < 0:
            raise ValueError('Invalid host inventory')
        expected[entry['path']] = entry
    verifier = package / 'native/bin/mirket'
    if sha(regular(verifier)) != binding['coreSha256']:
        raise ValueError('Native verifier differs from package binding')
    lock = json.loads(lock_bytes)
    placements = [(path, entry) for path, entry in lock['packages'].items() if path]
    for path, entry in placements:
        relative_parts(path)
        registry_url(entry['resolved'])
    staged = Path(tempfile.mkdtemp(prefix='.' + output.name + '-acquire-', dir=output.parent))
    try:
        host = staged / 'node_modules'
        host.mkdir(mode=0o700)
        observed, cache = set(), {}
        for path, entry in placements:
            url = entry['resolved']
            if url not in cache:
                cache[url] = archive_bytes(url, archive_dir)
            extract_archive(cache[url], entry['integrity'], path, host, expected, observed)
        if observed != expected.keys():
            raise ValueError('Locked archives do not supply the complete host inventory')
        report = json.loads(subprocess.check_output([
            str(verifier), 'verify-pi-host', '--root', str(host),
            '--inventory', str(inventory_path), '--sha256', binding['hostInventorySha256'],
        ], cwd=staged, env={**os.environ, 'HOME': str(staged), 'TMPDIR': str(staged), 'PATH': '/nonexistent'}, timeout=60))
        if (report['root'] != str(host) or report['entry'] != str(host / inventory['entry'])
                or report['lockSha256'] != inventory['lockSha256'] or report['verifiedFiles'] != len(expected)):
            raise ValueError('Native host verification response differs')
        publish(staged, output)
        return {'output': str(output), 'root': str(output / 'node_modules'),
                'entry': str(output / 'node_modules' / inventory['entry']),
                'lockSha256': inventory['lockSha256'], 'verifiedFiles': len(expected),
                'packageInstances': len(placements), 'uniqueArchives': len(cache)}
    finally:
        if staged.exists():
            shutil.rmtree(staged)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--lock', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--archive-dir', type=Path, help='Read exact cached archives instead of downloading')
    args = parser.parse_args()
    print(json.dumps(acquire(args.package, args.lock, args.output, args.archive_dir)))
