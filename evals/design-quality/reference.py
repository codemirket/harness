#!/usr/bin/env python3
"""Instrument frozen design-reference delivery in a caller-owned experiment.

This records helper events, not authenticated model attention or a sandbox.
It never installs, downloads, launches a model or mutates Mirket task state.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--context', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('operation', choices=['route', 'read'])
    parser.add_argument('arguments', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    context = args.context.resolve(strict=True)
    out = args.out.resolve(strict=True)
    if not context.is_dir() or not out.is_dir():
        parser.error('context and output must be existing directories')
    if context == out or context in out.parents or out in context.parents:
        parser.error('context and output must be disjoint')
    if args.operation == 'read':
        if len(args.arguments) != 2:
            parser.error('read requires skill ID and relative resource path')
        skill_id, resource_path = args.arguments
        if skill_id not in ('design-inspiration', 'interface-design'):
            parser.error('only the two frozen guidance owners can be read')
        relative = Path(resource_path)
        if relative.is_absolute() or not relative.parts or any(p in ('.', '..') for p in relative.parts):
            parser.error('read requires a normal relative resource path')
        owner = context / 'skills' / skill_id
        source = owner / relative
        # Check each component rather than accepting an in-root symlink alias.
        if any(p.is_symlink() for p in (owner, source, *source.parents)):
            parser.error('symlink source is unsupported')
        source = source.resolve(strict=True)
        if owner not in source.parents or not source.is_file():
            parser.error('source must be an owned regular file')
        payload = source.read_bytes()
        if not payload or len(payload) > 48 * 1024:
            parser.error('source must be nonempty bounded text (48 KiB)')
        payload.decode('utf-8')
        details = {'skill_id': skill_id, 'path': resource_path, 'source': str(source),
                   'source_sha256': hashlib.sha256(payload).hexdigest()}
        returncode = 0
    else:
        if not args.arguments:
            parser.error('route requires catalog search arguments')
        binary = context / 'bin' / 'mirket'
        if binary.is_symlink() or not binary.is_file():
            parser.error('frozen binary must be a regular file')
        argv = [str(binary), 'catalog', 'search', *args.arguments]
        # Exact argv, no shell. Search is a read-only command.
        proc = subprocess.run(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                              timeout=60, check=False)
        payload, returncode = proc.stdout, proc.returncode
        if len(payload) + len(proc.stderr) > 128 * 1024:
            raise ValueError('route response exceeds bounded observation')
        details = {'argv': argv, 'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
                   'stderr': proc.stderr.decode('utf-8', errors='replace')}
    event = {'schema_version': 1, 'kind': 'instrumented_reference_delivery',
             'operation': args.operation, 'at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
             'context': str(context), 'response_sha256': hashlib.sha256(payload).hexdigest(),
             'response_bytes': len(payload), 'returncode': returncode, **details,
             'limit': 'Caller-owned helper events record delivered bytes; they do not authenticate the caller, prevent bypass or prove attention/application.'}
    with (out / 'reference-delivery.ndjson').open('a', encoding='utf-8') as stream:
        stream.write(json.dumps(event, ensure_ascii=False) + '\n')
    sys.stdout.buffer.write(payload)
    if args.operation == 'route' and details['stderr']:
        sys.stderr.write(details['stderr'])
    return returncode


if __name__ == '__main__':
    sys.exit(main())
