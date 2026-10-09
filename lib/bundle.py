"""Build verified, self-contained Codex plugin marketplaces without installing them.

Manifest contracts reviewed against official OpenAI documentation:
https://developers.openai.com/plugins/build/plugins
"""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import sys
import tempfile

from . import catalog

ROOT = Path(__file__).resolve().parents[1]
MAX_EXPORT = 512 * 1024 * 1024
MAX_EXPORT_FILES = 100000
MAX_SNAPSHOT = 128 * 1024 * 1024
SNAPSHOT_PATHS = ('ai.py', 'lib', 'registry', 'instructions', 'skills', 'docs', 'setup', 'evaluations', 'scripts/workbench', 'examples')
SKIP_DIRS = {'.git', '__pycache__', 'node_modules', 'build', 'dist', 'coverage',
             '.cache', '.pytest_cache', '.mypy_cache', '.ruff_cache'}
LOCK = 'build-lock.json'


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + '\n').encode('utf-8')


def identifier(value):
    if not isinstance(value, str) or not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', value):
        raise ValueError('Invalid plugin/skill identifier: ' + str(value))
    catalog.safe_path(value)
    return value


def safe_output(output):
    """Do not resolve user-supplied links into a different destination."""
    output = Path(output).expanduser()
    if '..' in output.parts:
        raise ValueError('Output path may not contain parent traversal')
    output = output.absolute()
    for path in reversed([output] + list(output.parents)):
        if catalog.is_link(path):
            raise ValueError('Linked output path: ' + str(path))
        if path != output and path.exists() and not path.is_dir():
            raise ValueError('Non-directory output ancestor: ' + str(path))
    if output.exists():
        raise ValueError('Output already exists: ' + str(output))
    if not output.parent.is_dir():
        raise ValueError('Output parent must already exist: ' + str(output.parent))
    catalog.safe_path(output.name)
    return output


def snapshot(root):
    """Explicit allowlist; no runtime caches, generated builds or linked content."""
    files, executable = {}, set()
    total = 0

    def add(path):
        nonlocal total
        relative = path.relative_to(root).as_posix()
        if catalog.is_link(path) or not stat.S_ISREG(path.stat().st_mode):
            raise ValueError('Snapshot requires regular files: ' + relative)
        catalog.safe_path(relative)
        size = path.stat().st_size
        if total + size > MAX_SNAPSHOT or len(files) >= MAX_EXPORT_FILES:
            raise ValueError('Harness snapshot exceeds size limit')
        with path.open('rb') as stream:
            content = stream.read(MAX_SNAPSHOT - total + 1)
        total += len(content)
        if total > MAX_SNAPSHOT:
            raise ValueError('Harness snapshot exceeds size limit')
        files[relative] = content
        if path.stat().st_mode & 0o111:
            executable.add(relative)

    for relative in SNAPSHOT_PATHS:
        path = root / relative
        if catalog.is_link(path) or not path.exists():
            raise ValueError('Missing or linked harness component: ' + relative)
        if path.is_file():
            add(path)
            continue
        for directory, dirs, names in os.walk(path, followlinks=False):
            dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
            for name in dirs:
                if catalog.is_link(Path(directory) / name):
                    raise ValueError('Linked snapshot directory: ' + name)
            for name in sorted(names):
                if name == '.DS_Store' or name.endswith(('.pyc', '.pyo')):
                    continue
                add(Path(directory) / name)
    renderer = root / 'scripts/render_registry.py'
    if catalog.is_link(root / 'scripts') or catalog.is_link(renderer):
        raise ValueError('Linked harness renderer path')
    if renderer.exists():
        add(renderer)
    for pattern in ('README.md', 'LICENSE*', 'NOTICE*'):
        for path in sorted(root.glob(pattern)):
            add(path)
    catalog.validate_paths(files)
    return files, executable


def selections(config, data, names):
    if config.get('schema_version') != 1:
        raise ValueError('Unsupported harness schema')
    identifier(config['name'])
    if not isinstance(config.get('version'), str) or not re.fullmatch(
            r'\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?', config['version']):
        raise ValueError('Harness version must be semantic versioning')
    if not names:
        raise ValueError('Select at least one bundle')
    by_id = {entry['id']: entry for entry in data['skills']}
    if len(by_id) != len(data['skills']):
        raise ValueError('Duplicate catalog identifiers')
    selected = {}
    for name in sorted(set(names)):
        identifier(name)
        if name not in config['bundles']:
            raise ValueError('Unknown bundle: ' + name)
        definition = config['bundles'][name]
        entries = []
        if definition.get('global_skills'):
            globals_ = (config['global_skills'] if definition['global_skills'] is True
                        else definition['global_skills'])
            if not isinstance(globals_, list):
                raise ValueError('Bundle global_skills must be true or a list')
            for key in globals_:
                entry = by_id[key]
                if entry['scope'] not in ('global', 'project') or entry['delivery'] not in ('local', 'global-link'):
                    raise ValueError('Global bundle selection must be authored local: ' + key)
                entries.append(entry)
        entries.extend(catalog.resolve_selection(data, profiles=definition.get('profiles', []),
                                                  identifiers=definition.get('skills', [])))
        distinct, skill_names = {}, {}
        for entry in entries:
            identifier(entry['name'])
            if entry['id'] in distinct:
                continue
            if entry['name'] in skill_names:
                raise ValueError('Competing bundle skill name: ' + entry['name'])
            if 'codex' not in entry.get('agents', ['codex', 'claude']):
                raise ValueError('Codex bundle requires Codex support: ' + entry['id'])
            skill_names[entry['name']] = entry['id']
            distinct[entry['id']] = entry
        if not distinct:
            raise ValueError('Bundle has no skills: ' + name)
        for entry in distinct.values():
            if set(entry.get('conflicts', [])).intersection(distinct):
                raise ValueError('Conflicting bundle selections: ' + entry['id'])
        selected[name] = [distinct[key] for key in sorted(distinct)]
    return selected


def prepare(data, entry, root, source_trees, cache):
    if entry['delivery'] in ('local', 'global-link'):
        normalized = dict(entry, license_files=entry.get('license_files', []))
        original = catalog.read_local(root, normalized)
        if entry['scope'] == 'global':
            normalized.setdefault('sha256', catalog.payload_hash(original))
        normalized.update(delivery='upstream', source='__bundle_local__')
        local_data = dict(data, sources=dict(data['sources'], __bundle_local__={
            'repository': 'codemirket/harness', 'commit': None}))
        files, source = catalog.prepare_payload(local_data, normalized, root, cache)
        # Global links track authored working files rather than a pinned payload.
        # Exclude interpreter and Finder residue after ordinary safety validation;
        # never redefine the byte contract of a reviewed/pinned selection.
        if entry['scope'] == 'global' and not {'sha256', 'installed_sha256'}.intersection(entry):
            files = {path: content for path, content in files.items()
                     if '__pycache__' not in path.split('/')
                     and path.rsplit('/', 1)[-1] != '.DS_Store'
                     and not path.endswith(('.pyc', '.pyo'))}
            catalog.check_selected(files, normalized)
            for relative in catalog.executable_contract(entry.get('executable_files', [])):
                if relative not in files:
                    raise ValueError('Excluded runtime file is declared executable: ' + relative)
        return files, source
    return catalog.prepare_payload(data, entry, (source_trees or {}).get(entry['source']), cache)


def publish(staging, output):
    """Atomic directory publication that cannot replace an existing destination."""
    if os.name == 'nt':
        os.rename(staging, output)  # Windows fails if the destination exists.
        return
    libc = ctypes.CDLL(None, use_errno=True)
    src, dest = os.fsencode(staging), os.fsencode(output)
    if sys.platform == 'darwin':
        operation = libc.renamex_np
        operation.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint]
        result = operation(src, dest, 0x00000004)  # RENAME_EXCL
    elif sys.platform.startswith('linux') and hasattr(libc, 'renameat2'):
        operation = libc.renameat2
        operation.argtypes = [ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint]
        result = operation(-100, src, -100, dest, 1)  # AT_FDCWD, RENAME_NOREPLACE
    else:
        raise ValueError('Atomic no-replace directory export is unsupported on this platform')
    if result != 0:
        code = ctypes.get_errno()
        raise OSError(code, os.strerror(code), str(output))


def export_bundles(names, output, *, root=None, data=None, config=None, source_trees=None):
    root = Path(root or ROOT).resolve(strict=True)
    output = safe_output(output)
    config = config if config is not None else json.loads((root / 'registry/harness.json').read_text())
    data = data if data is not None else catalog.load_catalog(root / 'registry/catalog.json')
    selected = selections(config, data, names)
    payloads, cache = {}, {}
    for entries in selected.values():
        for entry in entries:
            if entry['id'] not in payloads:
                payloads[entry['id']] = prepare(data, entry, root, source_trees, cache)
    include_harness = any(e['id'] == 'skill-catalog' for entries in selected.values() for e in entries)
    harness_files, harness_modes = snapshot(root) if include_harness else ({}, set())
    files, modes, records = {}, {}, {}
    total = 0

    def add(path, content, executable=False):
        nonlocal total
        catalog.safe_path(path)
        if path in files:
            raise ValueError('Duplicate exported file: ' + path)
        total += len(content)
        if total > MAX_EXPORT or len(files) >= MAX_EXPORT_FILES:
            raise ValueError('Bundle export exceeds size/file-count limit')
        files[path] = content
        modes[path] = 0o755 if executable else 0o644

    codex_entries = []
    for name, entries in selected.items():
        prefix = 'plugins/' + name + '/'
        description = config['bundles'][name]['description']
        manifest = {'name': name, 'version': config['version'], 'description': description}
        add(prefix + 'plugin.json', json_bytes(dict(manifest, **{
            '$schema': 'https://agent-plugins.org/schemas/1.0.0/plugin.schema.json'})))
        attributed = dict(manifest, author={'name': config.get('owner', 'Personal AI')})
        add(prefix + '.codex-plugin/plugin.json', json_bytes(dict(attributed, skills='./skills/')))
        records[name] = []
        for entry in entries:
            prepared, source = payloads[entry['id']]
            payload = dict(prepared)
            if entry['id'] == 'skill-catalog':
                if '.harness-source.json' in payload:
                    raise ValueError('Source skill-catalog already contains a locator')
                payload['.harness-source.json'] = json_bytes({'repository': '../../_harness'})
                for path, content in harness_files.items():
                    add(prefix + '_harness/' + path, content, path in harness_modes)
            executables = set(entry.get('executable_files', []))
            for path, content in payload.items():
                add(prefix + 'skills/' + entry['name'] + '/' + path, content, path in executables)
            record = {'id': entry['id'], 'name': entry['name'], 'source': source,
                      'path': entry['path'], 'source_sha256': entry.get('sha256', catalog.payload_hash(prepared)),
                      'installed_sha256': catalog.payload_hash(prepared),
                      'export_sha256': catalog.payload_hash(payload),
                      'executable_files': sorted(executables),
                      'license_files': entry.get('license_files', [])}
            records[name].append(record)
        add(prefix + 'provenance.json', json_bytes({'schema_version': 1, 'skills': records[name]}))
        codex_entries.append({'name': name, 'source': {'source': 'local', 'path': './plugins/' + name},
                              'policy': {'installation': 'AVAILABLE', 'authentication': 'ON_INSTALL'},
                              'category': 'Productivity'})
    add('.agents/plugins/marketplace.json', json_bytes({'name': config['name'], 'plugins': codex_entries}))
    catalog.validate_paths(files)
    lock = {'schema_version': 1, 'name': config['name'], 'version': config['version'],
            'clients': ['codex-desktop', 'codex-cli'],
            'bundles': records, 'files': {path: {'sha256': hashlib.sha256(content).hexdigest(),
            'mode': format(modes[path], '04o')} for path, content in sorted(files.items())}}
    add(LOCK, json_bytes(lock))
    catalog.validate_paths(files)
    # No output/staging directory exists until every payload and manifest is verified.
    safe_output(output)
    staging = Path(tempfile.mkdtemp(prefix='.ai-export-', dir=output.parent))
    try:
        for relative, content in sorted(files.items()):
            path = staging.joinpath(*catalog.safe_path(relative).parts)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            path.chmod(modes[relative])
        # Verify bytes on disk, not only the in-memory build plan.
        if catalog.payload_hash(catalog.existing_payload(staging)) != catalog.payload_hash(files):
            raise ValueError('Staged export integrity check failed')
        safe_output(output)
        publish(staging, output)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return lock


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', action='append', required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(argv)
    result = export_bundles(args.bundle, args.output)
    print('Exported ' + ', '.join(result['bundles']) + ' to ' + str(args.output))
    return 0
