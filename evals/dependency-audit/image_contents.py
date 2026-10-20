#!/usr/bin/env python3
"""Compare selected final-image payloads without extracting or executing them."""
import argparse
import hashlib
import json
from pathlib import PurePosixPath
import tarfile


def name(raw):
    value = PurePosixPath(raw)
    if value.is_absolute() or '..' in value.parts:
        raise ValueError('unsafe archive name')
    return str(value)


def overlay(previous, records):
    result = dict(previous)
    # Whiteouts affect the previous layer, independent of tar member ordering.
    for path, _ in records:
        p = PurePosixPath(path)
        if p.name == '.wh..wh..opq':
            prefix = str(p.parent) + '/'
            result = {} if p.parent == PurePosixPath('.') else {
                k:v for k,v in result.items() if not k.startswith(prefix)}
        elif p.name.startswith('.wh.'):
            removed = str(p.parent / p.name[4:])
            result = {k:v for k,v in result.items() if k != removed and not k.startswith(removed + '/')}
    for path, value in records:
        if not PurePosixPath(path).name.startswith('.wh.'):
            if value['type'] != 'dir':
                result = {k:v for k,v in result.items() if not k.startswith(path + '/')}
            result[path] = value
    return result


def contents(path, prefixes):
    result = {}
    with tarfile.open(path, 'r:*') as image:
        manifest = json.load(image.extractfile('manifest.json'))
        if len(manifest) != 1:
            raise ValueError('one image required')
        for layer in manifest[0]['Layers']:
            member = image.getmember(name(layer))
            if not member.isfile():
                raise ValueError('layer must be a regular file')
            records = []
            with tarfile.open(fileobj=image.extractfile(member), mode='r|*') as archive:
                for item in archive:
                    entry = name(item.name)
                    # Include ancestors to honor opaque/remove whiteouts above a payload.
                    if not PurePosixPath(entry).name.startswith('.wh.') and not any(
                        entry == p or entry.startswith(p + '/') or p.startswith(entry + '/')
                        for p in prefixes
                    ):
                        continue
                    value = {'mode':item.mode}
                    if item.isfile():
                        h = hashlib.sha256()
                        stream = archive.extractfile(item)
                        while chunk := stream.read(1024 * 1024):
                            h.update(chunk)
                        value.update(type='file',sha256=h.hexdigest(),size=item.size)
                    elif item.isdir():
                        value.update(type='dir')
                    elif item.issym() or item.islnk():
                        value.update(type='symlink' if item.issym() else 'hardlink',target=item.linkname)
                    else:
                        raise ValueError('special file in payload')
                    records.append((entry, value))
            result = overlay(result, records)
    return {k:v for k,v in result.items() if any(k == p or k.startswith(p + '/') for p in prefixes)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before'); parser.add_argument('after'); parser.add_argument('output')
    parser.add_argument('--prefix', action='append', required=True, help='Explicit final-image payload prefix to compare')
    args = parser.parse_args()
    prefixes = [name(prefix) for prefix in args.prefix]
    before = contents(args.before,prefixes); after = contents(args.after,prefixes)
    changed = [k for k in sorted(before.keys() | after.keys()) if before.get(k) != after.get(k)]
    report = {'ok':bool(before) and before == after,'scope':prefixes,'entries':len(before),
              'changed':changed,'before':before,'after':after,
              'limits':'Content, modes and link targets; ownership intentionally differs. No files extracted or executed.'}
    with open(args.output,'x') as out: json.dump(report,out,indent=2)
    print(json.dumps({k:v for k,v in report.items() if k not in ('before','after')}))
    return 0 if report['ok'] else 1

if __name__ == '__main__':
    raise SystemExit(main())
