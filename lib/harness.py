"""Declarative personal app/project setup. Python 3.9+, standard library only."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import sys
import tempfile
import uuid

from . import catalog

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'registry/harness.json'
PROJECT_FILE = '.ai/project.json'


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'))


def manifest():
    data = read_json(MANIFEST)
    if data.get('schema_version') != 1:
        raise ValueError('Unsupported harness schema')
    return data


def targets(choice, config):
    names = list(config['targets']) if choice == 'both' else [choice]
    if any(name not in ('codex', 'claude') or name not in config['targets'] for name in names):
        raise ValueError('Unsupported target')
    return names


def guarded_path(base, relative):
    """Reject linked parents, including Windows junctions, before reading/writing."""
    parts = catalog.safe_path(relative).parts
    path = base
    for part in parts[:-1]:
        path = path / part
        if catalog.is_link(path) or (path.exists() and not path.is_dir()):
            raise ValueError('Expected real destination directory: ' + str(path))
    return base.joinpath(*parts)


def tree_files(path):
    files = {}
    for directory, dirs, names in os.walk(path, followlinks=False):
        dirs[:] = [d for d in dirs if d != '__pycache__']
        for name in dirs + names:
            if catalog.is_link(Path(directory) / name):
                raise ValueError('Symlink in managed payload: ' + str(Path(directory) / name))
        for name in names:
            if name == '.DS_Store' or name.endswith('.pyc'):
                continue
            item = Path(directory) / name
            if not item.is_file():
                raise ValueError('Non-regular managed file: ' + str(item))
            files[item.relative_to(path).as_posix()] = item.read_bytes()
    catalog.validate_paths(files)
    return files


def fingerprint(path):
    if catalog.is_link(path):
        return {'kind': 'link', 'target': os.readlink(path)}
    if path.is_dir():
        return {'kind': 'directory', 'sha256': catalog.payload_hash(tree_files(path))}
    if path.is_file():
        return {'kind': 'file', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
    if path.exists():
        raise ValueError('Unsupported destination: ' + str(path))
    return None


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name('.' + path.name + '-' + uuid.uuid4().hex)
    try:
        temp.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
        os.replace(temp, path)
    finally:
        if temp.exists():
            temp.unlink()


def global_plan(target='both', home=None, mode='auto'):
    config = manifest()
    data = catalog.load_catalog()
    home = (home or Path.home()).expanduser().resolve(strict=True)
    if not home.is_dir():
        raise ValueError('Home must be an existing directory')
    mode = ('copy' if os.name == 'nt' else 'link') if mode == 'auto' else mode
    if mode not in ('link', 'copy'):
        raise ValueError('Unsupported installation mode')
    receipt = guarded_path(home, '.agent-harness/state.json')
    if catalog.is_link(receipt) or (receipt.exists() and not receipt.is_file()):
        raise ValueError('Unsafe harness receipt')
    previous = read_json(receipt) if receipt.exists() else {'schema_version': 1, 'items': {}}
    if previous.get('schema_version') != 1 or not isinstance(previous.get('items'), dict):
        raise ValueError('Invalid harness receipt')
    by_id = {entry['id']: entry for entry in data['skills']}
    jobs = []
    for app in targets(target, config):
        adapter = config['targets'][app]
        specs = [('instructions', adapter['instructions'], adapter['instruction_destination'], False)]
        for identifier in config['global_skills']:
            entry = by_id[identifier]
            if entry.get('delivery') not in ('local', 'global-link'):
                raise ValueError('Default global skills must be authored local selections: ' + identifier)
            specs.append((identifier, entry['path'], adapter['skills_destination'] + '/' + entry['name'], True))
        for identifier, source_rel, dest_rel, directory in specs:
            source = ROOT.joinpath(*catalog.safe_path(source_rel).parts)
            if catalog.is_link(source) or not source.exists():
                raise ValueError('Missing or symlinked source: ' + str(source))
            if directory:
                if not source.is_dir() or not (source / 'SKILL.md').is_file():
                    raise ValueError('Missing global SKILL.md: ' + str(source))
                payload = tree_files(source)
                if mode == 'copy' and identifier == 'skill-catalog':
                    payload['.harness-source.json'] = (json.dumps({'repository': str(ROOT)}) + '\n').encode()
                desired = {'kind': 'directory', 'sha256': catalog.payload_hash(payload)}
            else:
                if not source.is_file():
                    raise ValueError('Instruction source must be a file')
                payload = source.read_bytes()
                desired = {'kind': 'file', 'sha256': hashlib.sha256(payload).hexdigest()}
            if mode == 'link':
                desired = {'kind': 'link', 'target': str(source)}
            dest = guarded_path(home, dest_rel)
            actual = fingerprint(dest)
            key = app + ':' + identifier
            old = previous['items'].get(key)
            owned = old and old.get('destination') == dest_rel and old.get('fingerprint') == actual
            # Existing repository links from the earlier installers are known migrations.
            legacy = set()
            if identifier == 'instructions':
                legacy.add(str(ROOT / 'components/AGENTS.md'))
            legacy.add(str(source))
            is_legacy = actual and actual['kind'] == 'link' and actual['target'] in legacy
            if actual == desired:
                action = 'unchanged'
            elif actual is None:
                action = 'create'
            elif owned or is_legacy:
                action = 'update'
            else:
                raise ValueError('Unmanaged or modified destination; preserving it: ' + str(dest))
            jobs.append({'target': app, 'id': identifier, 'source': source, 'destination': dest,
                         'relative_destination': dest_rel, 'directory': directory, 'mode': mode,
                         'action': action, 'before': actual, 'desired': desired, 'payload': payload})
    # No hidden duplicate writes, even if the declarative target manifest is edited.
    paths = [str(j['destination']).casefold() for j in jobs]
    if len(paths) != len(set(paths)):
        raise ValueError('Duplicate global destinations')
    return home, receipt, previous, jobs


def public_jobs(jobs):
    return [{k: str(j[k]) if isinstance(j[k], Path) else j[k]
             for k in ('target', 'id', 'source', 'destination', 'mode', 'action')} for j in jobs]


def replace_item(dest, create, expected):
    """Stage first, preserve user content, and restore this item if rename fails."""
    if fingerprint(dest) != expected:
        raise ValueError('Destination changed since preflight: ' + str(dest))
    dest.parent.mkdir(parents=True, exist_ok=True)
    stage = dest.with_name('.' + dest.name + '.new-' + uuid.uuid4().hex)
    backup = dest.with_name('.' + dest.name + '.previous-' + uuid.uuid4().hex)
    moved = False
    committed = False
    try:
        create(stage)
        if fingerprint(dest) != expected:
            raise ValueError('Destination changed during staging: ' + str(dest))
        if expected is not None:
            os.replace(dest, backup)
            moved = True
        try:
            os.replace(stage, dest)
            committed = True
        except BaseException:
            if moved:
                os.replace(backup, dest)
                moved = False
            raise
    finally:
        # A failed restore must retain the previous bytes for manual recovery.
        for item in (stage, backup if moved and committed else None):
            if item is not None and (item.exists() or catalog.is_link(item)):
                if catalog.is_link(item) or item.is_file():
                    item.unlink()
                else:
                    shutil.rmtree(item)


def sync_global(target='both', home=None, mode='auto'):
    home, receipt, previous, jobs = global_plan(target, home, mode)
    for job in jobs:
        if job['action'] != 'unchanged':
            def create(stage):
                if job['mode'] == 'link':
                    stage.symlink_to(job['source'], target_is_directory=job['directory'])
                elif job['directory']:
                    stage.mkdir()
                    for relative, raw in job['payload'].items():
                        out = stage.joinpath(*catalog.safe_path(relative).parts)
                        out.parent.mkdir(parents=True, exist_ok=True)
                        out.write_bytes(raw)
                else:
                    stage.write_bytes(job['payload'])
            replace_item(job['destination'], create, job['before'])
        previous['items'][job['target'] + ':' + job['id']] = {
            'destination': job['relative_destination'], 'source': str(job['source']),
            'fingerprint': job['desired']}
        # Keep ownership evidence for completed items if a later OS failure occurs.
        write_json(receipt, previous)
    return public_jobs(jobs)


def project_config(project):
    project = project.expanduser().resolve(strict=True)
    if not project.is_dir():
        raise ValueError('Project must be a directory')
    path = guarded_path(project, PROJECT_FILE)
    if catalog.is_link(path) or not path.is_file():
        raise ValueError('Missing or unsafe .ai/project.json; run project init first')
    data = read_json(path)
    if data.get('schema_version') != 1:
        raise ValueError('Unsupported project manifest schema')
    for field in ('profiles', 'skills', 'skip', 'targets'):
        if not isinstance(data.get(field, []), list) or any(not isinstance(x, str) for x in data.get(field, [])):
            raise ValueError('Project ' + field + ' must be a string list')
    selected_targets = data.get('targets')
    if not selected_targets or len(selected_targets) != len(set(selected_targets)):
        raise ValueError('Select distinct project targets')
    for target in selected_targets:
        if target not in manifest()['targets']:
            raise ValueError('Unsupported project target: ' + target)
    entries = catalog.resolve_selection(catalog.load_catalog(), data.get('profiles', []),
                                        data.get('skills', []), data.get('skip', []))
    if not entries:
        raise ValueError('Project manifest selects no skills')
    return project, data, entries


def project_state(entry, project, target):
    # Metadata rejection must never be treated as permission to update an old copy.
    if entry['scope'] != 'project' or entry.get('delivery') not in ('upstream', 'local'):
        raise ValueError('This entry is not approved for automatic project installation')
    if target not in ('codex', 'claude') or target not in entry.get('agents', ['codex', 'claude']):
        raise ValueError('Skill does not support this agent: ' + target)
    if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', entry['name']):
        raise ValueError('Unsafe installation name')
    catalog.selection_files(entry)
    base = '.agents/skills/' if target == 'codex' else '.claude/skills/'
    dest = guarded_path(project, base + entry['name'])
    try:
        dest, unchanged = catalog.check_destination(entry, project, target)
        return dest, 'unchanged' if unchanged else 'create', fingerprint(dest)
    except ValueError as original:
        # Explicit sync may update only an intact previous catalog-owned copy.
        if catalog.is_link(dest) or not dest.is_dir():
            raise original
        receipt = dest / catalog.RECEIPT
        if catalog.is_link(receipt) or not receipt.is_file():
            raise original
        prior = read_json(receipt)
        if (target not in entry.get('agents', ['codex', 'claude'])
                or prior.get('id') != entry['id']
                or catalog.payload_hash(catalog.existing_payload(dest)) != prior.get('installed_sha256', prior.get('sha256'))):
            raise original
        # Compare the old installation with its own contract. A new version may
        # add executable helpers that cannot yet exist in the old installation.
        executable_files = prior.get('executable_files')
        if executable_files is None:
            # Legacy receipts did not record modes. Preserve checks for known
            # existing helpers without rejecting newly added paths.
            executable_files = [relative for relative in entry.get('executable_files', [])
                                if dest.joinpath(*catalog.safe_path(relative).parts).exists()]
        if (not isinstance(executable_files, list)
                or any(not isinstance(relative, str) for relative in executable_files)):
            raise ValueError('Invalid executable list in installed receipt')
        for relative in executable_files:
            path = dest.joinpath(*catalog.safe_path(relative).parts)
            if os.name != 'nt' and (not path.is_file() or path.stat().st_mode & 0o111 != 0o111):
                raise original
        return dest, 'update', fingerprint(dest)


def project_plan(project):
    project, config, entries = project_config(project)
    jobs = []
    for entry in entries:
        for target in config['targets']:
            dest, action, before = project_state(entry, project, target)
            jobs.append({'entry': entry, 'target': target, 'destination': dest, 'action': action, 'before': before})
    lock = guarded_path(project, '.ai/project.lock.json')
    if catalog.is_link(lock) or (lock.exists() and not lock.is_file()):
        raise ValueError('Unsafe project lock destination')
    return project, config, entries, jobs


def public_project(jobs):
    return [{'id': j['entry']['id'], 'target': j['target'], 'destination': str(j['destination']),
             'action': j['action'], 'dependencies': j['entry'].get('dependencies', []),
             'caveats': j['entry'].get('caveats', [])} for j in jobs]


def project_lock(config, entries, data):
    return {'schema_version': 1, 'manifest': config,
            'skills': [{'id': e['id'], 'sha256': e['sha256'],
                        'installed_sha256': e.get('installed_sha256', e['sha256']),
                        'source': e.get('source', 'personal'),
                        'commit': data['sources'][e['source']]['commit'] if e.get('source') else None}
                       for e in entries]}


def project_lock_current(project, expected):
    path = guarded_path(project, '.ai/project.lock.json')
    if catalog.is_link(path) or not path.is_file():
        return False
    try:
        return read_json(path) == expected
    except ValueError:
        return False


def sync_project(project, source_trees=None):
    project, config, entries, jobs = project_plan(project)
    data = catalog.load_catalog()
    lock = project_lock(config, entries, data)
    cache = {}; prepared = {}
    for job in jobs:
        entry = job['entry']
        if job['action'] != 'unchanged' and entry['id'] not in prepared:
            source_tree = (source_trees or {}).get(entry.get('source'))
            prepared[entry['id']] = catalog.prepare_payload(data, entry, source_tree, cache)
    # Downloads can take time. Recheck every destination and parent before any
    # writes, then again before publishing each item. This is not a filesystem lock.
    for job in jobs:
        dest = job['destination']
        guarded_path(project, dest.relative_to(project).as_posix())
        if fingerprint(dest) != job['before']:
            raise ValueError('Destination changed since preflight: ' + str(dest))
    # All predictable failures precede writes. Every item gets independent recovery.
    for job in jobs:
        entry = job['entry']; dest = job['destination']
        if job['action'] == 'unchanged':
            continue
        with tempfile.TemporaryDirectory(prefix='personal-ai-skill-') as temp:
            sandbox = Path(temp)
            catalog.install(data, entry, sandbox, job['target'], prepared=prepared[entry['id']])
            built = sandbox / ('.agents' if job['target'] == 'codex' else '.claude') / 'skills' / entry['name']
            guarded_path(project, dest.relative_to(project).as_posix())
            replace_item(dest, lambda stage: shutil.copytree(built, stage), job['before'])
    write_json(guarded_path(project, '.ai/project.lock.json'), lock)
    return public_project(jobs)


def init_project(project, profiles, skills, skip, target):
    project = project.expanduser().resolve(strict=True)
    if not project.is_dir():
        raise ValueError('Project must be a directory')
    selected = catalog.resolve_selection(catalog.load_catalog(), profiles, skills, skip)
    if not selected:
        raise ValueError('Choose project profiles or skills')
    value = {'schema_version': 1, 'targets': targets(target, manifest()),
             'profiles': profiles, 'skills': skills, 'skip': skip}
    dest = guarded_path(project, PROJECT_FILE)
    if catalog.is_link(dest) or (dest.exists() and (not dest.is_file() or read_json(dest) != value)):
        raise ValueError('Project manifest already exists; edit it deliberately: ' + str(dest))
    if not dest.exists():
        write_json(dest, value)
    return value


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    for name in ('plan', 'sync', 'doctor'):
        p = commands.add_parser(name, help=name + ' declared global instructions and skills')
        p.add_argument('--target', choices=['codex', 'claude', 'both'], default='both')
        p.add_argument('--home', type=Path)
        p.add_argument('--mode', choices=['auto', 'link', 'copy'], default='auto')
    project = commands.add_parser('project').add_subparsers(dest='action', required=True)
    for name in ('init', 'plan', 'sync', 'doctor'):
        p = project.add_parser(name)
        p.add_argument('--project', type=Path, required=True)
        if name == 'init':
            p.add_argument('--target', choices=['codex', 'claude', 'both'], default='both')
            p.add_argument('--profile', action='append', default=[])
            p.add_argument('--skill', action='append', default=[])
            p.add_argument('--skip', action='append', default=[])
    args = parser.parse_args(argv)
    if args.command in ('plan', 'sync', 'doctor'):
        if args.command == 'sync':
            result = sync_global(args.target, args.home, args.mode)
        else:
            result = public_jobs(global_plan(args.target, args.home, args.mode)[3])
        print(json.dumps(result, indent=2, ensure_ascii=False))
        return 1 if args.command == 'doctor' and any(j['action'] != 'unchanged' for j in result) else 0
    if args.action == 'init':
        result = init_project(args.project, args.profile, args.skill, args.skip, args.target)
    elif args.action == 'sync':
        result = sync_project(args.project)
    else:
        project, config, entries, jobs = project_plan(args.project)
        result = public_project(jobs)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    if args.action == 'doctor':
        lock_ok = project_lock_current(project, project_lock(config, entries, catalog.load_catalog()))
        if not lock_ok:
            print('Missing or stale .ai/project.lock.json; run project sync to reconcile it.', file=sys.stderr)
        return 1 if not lock_ok or any(j['action'] != 'unchanged' for j in result) else 0
    return 0
