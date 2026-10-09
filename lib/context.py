"""Read-only project instruction audit, not an active client's effective context."""
import argparse
import json
import os
from pathlib import Path
import stat

from . import catalog

DEFAULT_MAX_BYTES = 32 * 1024
MAX_READ_BYTES = 1024 * 1024
MAX_CHAIN_DIRECTORIES = 256
MAX_FALLBACKS = 32


def fallback_names(values):
    names = ['AGENTS.override.md', 'AGENTS.md']
    if len(values) > MAX_FALLBACKS:
        raise ValueError('At most 32 fallback names are supported')
    for name in values:
        if (not isinstance(name, str) or not name or name in ('.', '..')
                or any(char in name for char in '/\\:')
                or any(ord(char) < 32 for char in name)
                or name.rstrip(' .') != name):
            raise ValueError('Fallback must be a plain portable filename')
        catalog.safe_path(name)  # Also reject Windows device names and reserved characters.
        if name not in names:
            names.append(name)
    return names


def read_regular(path, expected, limit):
    """Never block on a FIFO substituted after discovery; verify before reading."""
    flags = (os.O_RDONLY | getattr(os, 'O_BINARY', 0) | getattr(os, 'O_NONBLOCK', 0)
             | getattr(os, 'O_NOFOLLOW', 0))
    descriptor = os.open(path, flags)
    try:
        actual = os.fstat(descriptor)
        if (not stat.S_ISREG(actual.st_mode) or catalog.is_link(path)
                or (actual.st_dev, actual.st_ino) != (expected.st_dev, expected.st_ino)):
            raise ValueError('Instruction file changed during inspection')
        with os.fdopen(descriptor, 'rb', closefd=False) as stream:
            data = stream.read(limit)
        after = os.fstat(descriptor)
        if (after.st_size, after.st_mtime_ns) != (expected.st_size, expected.st_mtime_ns):
            raise ValueError('Instruction file changed during inspection')
        return data
    finally:
        os.close(descriptor)


def doctor(project, cwd=None, max_bytes=DEFAULT_MAX_BYTES, fallback=()):
    """Inspect only the explicitly supplied root-to-cwd chain; return no file text."""
    if isinstance(max_bytes, bool) or not isinstance(max_bytes, int) or max_bytes < 0:
        raise ValueError('Maximum bytes must be a nonnegative integer')
    names = fallback_names(fallback)
    project = Path(project).expanduser().resolve(strict=True)
    cwd = Path(cwd).expanduser().resolve(strict=True) if cwd is not None else project
    if not project.is_dir() or not cwd.is_dir():
        raise ValueError('Project and cwd must be existing directories')
    try:
        relative = cwd.relative_to(project)
    except ValueError:
        raise ValueError('Cwd must be within the supplied project') from None
    if len(relative.parts) + 1 > MAX_CHAIN_DIRECTORIES:
        raise ValueError('Instruction chain exceeds 256 directories')
    directories = [project]
    for part in relative.parts:
        directories.append(directories[-1] / part)
    report = {'schema_version': 1, 'project': str(project), 'cwd': str(cwd),
              'max_bytes': max_bytes, 'fallback_names': names[2:],
              'selected_bytes': 0, 'included_bytes': 0, 'files': [], 'diagnostics': [],
              'limits': [
                  'Static project-only audit of the supplied root-to-cwd chain; no client is launched.',
                  'Root, cwd, fallbacks and byte budget are supplied assumptions, not resolved client configuration.',
                  'Global instructions, project trust, root markers, extra environments, skills and session caches are not inspected.',
                  'Links and special files are refused; Codex itself may support linked instruction files.',
                  'Reads are capped at 1 MiB per selected file; line counts describe only the inspected prefix.',
                  'No instruction text is returned. This snapshot does not prove active client discovery or compliance.',
                  'Filesystem checks do not provide isolation from concurrent adversarial directory replacement.']}

    def diagnostic(code, path, message, severity='error'):
        report['diagnostics'].append({'code': code, 'severity': severity,
                                      'path': str(path), 'message': message})

    remaining = max_bytes
    for directory in directories:
        if catalog.is_link(directory) or not directory.is_dir():
            diagnostic('unsafe_directory', directory, 'Directory changed or became linked; remaining chain was not inspected.')
            break
        candidates = []
        blocked = False
        for name in names:
            path = directory / name
            try:
                metadata = path.lstat()
            except FileNotFoundError:
                continue
            except OSError:
                diagnostic('unreadable', path, 'Cannot inspect instruction candidate; shadowing inspection is incomplete.',
                           'warning' if candidates else 'error')
                blocked = not candidates
                break
            if catalog.is_link(path):
                diagnostic('unsafe_link', path, 'Linked instruction candidate was not followed; review it manually.',
                           'warning' if candidates else 'error')
                blocked = not candidates
                break
            if stat.S_ISDIR(metadata.st_mode):
                continue
            if not stat.S_ISREG(metadata.st_mode):
                diagnostic('special_file', path, 'Special instruction candidate was not opened; replace it with a regular file.')
                continue
            candidates.append((path, metadata))
        if blocked:
            break
        if not candidates:
            continue
        path, metadata = candidates[0]
        shadowed = [str(candidate) for candidate, _ in candidates[1:]]
        row = {'path': str(path), 'status': 'selected', 'bytes': metadata.st_size,
               'included_bytes': 0, 'inspected_bytes': 0, 'lines': 0, 'shadowed': shadowed}
        report['files'].append(row)
        report['selected_bytes'] += metadata.st_size
        if shadowed:
            diagnostic('shadowed', path, 'This candidate takes precedence over other instruction files in this directory.', 'warning')
        if remaining == 0 and metadata.st_size > 0:
            row['status'] = 'excluded'
            diagnostic('budget_exhausted', path, 'No project byte budget remains for this file; shorten earlier instructions or review the supplied budget.')
            continue
        # Even an empty selected override must be safely opened: an unreadable
        # empty file is not evidence that the client can load empty guidance.
        limit = min(metadata.st_size, remaining, MAX_READ_BYTES)
        try:
            data = read_regular(path, metadata, limit)
        except (OSError, ValueError):
            row['status'] = 'unreadable'
            diagnostic('unreadable', path, 'Selected instructions could not be read safely; later budget accounting is unknown.')
            # Do not claim a known remaining budget after failure.
            break
        row['inspected_bytes'] = len(data)
        text = data.decode('utf-8', errors='replace')
        row['lines'] = len(text.splitlines())
        if row['lines'] > 100:
            diagnostic('long_instructions', path, 'More than 100 lines were inspected; consider a short map with linked detail. This is an advisory, not a runtime limit.', 'warning')
        if metadata.st_size > limit and limit == MAX_READ_BYTES and remaining > MAX_READ_BYTES:
            row['status'] = 'inspection_limit'
            diagnostic('inspection_limit', path, 'File exceeds the audit read cap; shorten it or review manually. Later budget accounting is unknown.')
            break
        if metadata.st_size > remaining:
            row['status'] = 'truncated'
            diagnostic('truncated', path, 'File exceeds the remaining project byte budget; its tail would be omitted under these assumptions.')
        # Codex charges the truncated byte prefix only when its lossy text is nonempty.
        if text.strip():
            row['included_bytes'] = len(data)
            remaining -= len(data)
            report['included_bytes'] += len(data)
        elif row['status'] == 'selected':
            row['status'] = 'empty'
            diagnostic('empty', path, 'Whitespace-only project instructions consume no budget but still shadow other candidates.', 'warning')
    if not report['files'] and not report['diagnostics']:
        diagnostic('no_instructions', project, 'No instruction files were found in the supplied chain.', 'warning')
    report['ok'] = not any(item['severity'] == 'error' for item in report['diagnostics'])
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    check = commands.add_parser('doctor', help='Audit a supplied project instruction chain without reading global settings')
    check.add_argument('--project', type=Path, required=True)
    check.add_argument('--cwd', type=Path)
    check.add_argument('--max-bytes', type=int, default=DEFAULT_MAX_BYTES)
    check.add_argument('--fallback', action='append', default=[])
    args = parser.parse_args(argv)
    report = doctor(args.project, args.cwd, args.max_bytes, args.fallback)
    print(json.dumps(report, indent=2, ensure_ascii=False))
    return 0 if report['ok'] else 1
