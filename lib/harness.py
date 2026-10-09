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

from . import catalog, targets as target_registry

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'registry/harness.json'
PROJECT_FILE = '.ai/project.json'


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'))


def manifest():
    data = read_json(MANIFEST)
    if data.get('schema_version') != 1:
        raise ValueError('Unsupported harness schema')
    if 'targets' not in data:
        data['targets'] = target_registry.load()
    return data


def targets(choice, config):
    names = target_registry.names(choice)
    if any(name not in config['targets'] for name in names):
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


def project_fingerprint(path):
    """Include execute bits for project publication, preserving global receipts."""
    value = fingerprint(path)
    if value and value['kind'] == 'directory' and os.name != 'nt':
        value['executable_modes'] = catalog.executable_modes(path)
    return value


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name('.' + path.name + '-' + uuid.uuid4().hex)
    try:
        temp.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
        os.replace(temp, path)
    finally:
        if temp.exists():
            temp.unlink()


def global_plan(target='codex', home=None, mode='auto'):
    config = manifest()
    data = catalog.load_catalog()
    selected_targets = targets(target, config)
    target_registry.validate_home_environment(home, selected_targets)
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
    for app in selected_targets:
        adapter = target_registry.global_adapter(app, home, config['targets'][app])
        specs = [('instructions', adapter['instructions'], adapter['instruction_destination'], False)]
        for identifier in config['global_skills']:
            entry = by_id[identifier]
            if not catalog.supports_target(entry, app):
                raise ValueError('Global skill ' + identifier + ' does not support target ' + app)
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
            owned = any(old.get('destination') == dest_rel and old.get('fingerprint') == actual
                        for old in previous['items'].values() if isinstance(old, dict))
            # Existing repository links from the earlier installers are known migrations.
            legacy = set()
            if identifier == 'instructions':
                legacy.add(str(ROOT / 'components/AGENTS.md'))
                if app == 'claude':
                    legacy.add(str(ROOT / 'instructions/CLAUDE.md'))
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
    # Shared client discovery roots may contain the same authored skill. Reject
    # collisions unless every input to the physical publication is identical.
    destinations = {}
    for job in jobs:
        key = str(job['destination']).casefold()
        prior = destinations.setdefault(key, job)
        if any(prior[field] != job[field] for field in
               ('destination', 'id', 'source', 'desired', 'mode', 'payload')):
            raise ValueError('Conflicting global destinations: ' + str(job['destination']))
    return home, receipt, previous, jobs


def public_jobs(jobs):
    return [{k: str(j[k]) if isinstance(j[k], Path) else j[k]
             for k in ('target', 'id', 'source', 'destination', 'mode', 'action')} for j in jobs]


def replace_item(dest, create, expected, snapshot=fingerprint):
    """Stage first, preserve user content, and restore this item if rename fails."""
    if snapshot(dest) != expected:
        raise ValueError('Destination changed since preflight: ' + str(dest))
    dest.parent.mkdir(parents=True, exist_ok=True)
    stage = dest.with_name('.' + dest.name + '.new-' + uuid.uuid4().hex)
    backup = dest.with_name('.' + dest.name + '.previous-' + uuid.uuid4().hex)
    moved = False
    committed = False
    try:
        create(stage)
        if snapshot(dest) != expected:
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


def sync_global(target='codex', home=None, mode='auto'):
    home, receipt, previous, jobs = global_plan(target, home, mode)
    published = set()
    for job in jobs:
        if job['action'] != 'unchanged' and job['destination'] not in published:
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
            published.add(job['destination'])
        previous['items'][job['target'] + ':' + job['id']] = {
            'destination': job['relative_destination'], 'source': str(job['source']),
            'fingerprint': job['desired']}
        # Keep ownership evidence for completed items if a later OS failure occurs.
        write_json(receipt, previous)
    return public_jobs(jobs)


def normalize_project_target(target, context='project target'):
    try:
        return target_registry.normalize(target)
    except ValueError as error:
        raise ValueError('Unsupported ' + context + ': ' + str(target)) from error


def project_selection(config, data):
    """Validate declarations and resolve each target's complete selection."""
    if not isinstance(config, dict) or type(config.get('schema_version')) is not int:
        raise ValueError('Unsupported project manifest schema')
    if config['schema_version'] not in (1, 2):
        raise ValueError('Unsupported project manifest schema')
    if config['schema_version'] == 1 and 'target_skills' in config:
        raise ValueError('Project target_skills requires schema_version 2')
    config = dict(config)
    for field in ('profiles', 'skills', 'skip', 'targets'):
        if (not isinstance(config.get(field, []), list)
                or any(not isinstance(x, str) for x in config.get(field, []))):
            raise ValueError('Project ' + field + ' must be a string list')
    selected_targets = config.get('targets')
    if not selected_targets or len(selected_targets) != len(set(selected_targets)):
        raise ValueError('Select distinct project targets')
    selected_targets = [normalize_project_target(target) for target in selected_targets]
    if len(selected_targets) != len(set(selected_targets)):
        raise ValueError('Select distinct project targets')
    config['targets'] = selected_targets
    for target in selected_targets:
        if target not in manifest()['targets']:
            raise ValueError('Unsupported project target: ' + target)
    declared = config.get('target_skills', {})
    if not isinstance(declared, dict):
        raise ValueError('Project target_skills must map targets to string lists')
    specific = {}
    for target, identifiers in declared.items():
        target = normalize_project_target(target, 'target_skills target')
        if target not in selected_targets:
            raise ValueError('target_skills target is not in project targets: ' + target)
        if target in specific:
            raise ValueError('Duplicate target_skills target alias: ' + target)
        if not isinstance(identifiers, list) or any(not isinstance(x, str) for x in identifiers):
            raise ValueError('Project target_skills.' + target + ' must be a string list')
        overlap = set(identifiers).intersection(config.get('skip', []))
        if overlap:
            raise ValueError('Cannot skip explicit target skill: ' + sorted(overlap)[0])
        specific[target] = list(identifiers)
    if 'target_skills' in config:
        config['target_skills'] = specific
    selections = {}
    for target in selected_targets:
        try:
            entries = catalog.resolve_selection(data, config.get('profiles', []),
                                                config.get('skills', []) + specific.get(target, []),
                                                config.get('skip', []))
            for entry in entries:
                if not catalog.supports_target(entry, target):
                    raise ValueError('Skill ' + entry['id'] + ' does not support target ' + target)
        except ValueError as error:
            raise ValueError('Project target ' + target + ': ' + str(error)) from error
        selections[target] = entries
    if not any(selections.values()):
        raise ValueError('Project manifest selects no skills')
    return config, selections


def selected_entries(selections):
    # A skill shared by targets needs only one source download and provenance row.
    return list({entry['id']: entry for entries in selections.values() for entry in entries}.values())


def project_config(project, per_target=False):
    project = project.expanduser().resolve(strict=True)
    if not project.is_dir():
        raise ValueError('Project must be a directory')
    path = guarded_path(project, PROJECT_FILE)
    if catalog.is_link(path) or not path.is_file():
        raise ValueError('Missing or unsafe .ai/project.json; run project init first')
    config, selections = project_selection(read_json(path), catalog.load_catalog())
    return project, config, selections if per_target else selected_entries(selections)


def project_state(entry, project, target):
    # Metadata rejection must never be treated as permission to update an old copy.
    if entry['scope'] != 'project' or entry.get('delivery') not in ('upstream', 'local'):
        raise ValueError('This entry is not approved for automatic project installation')
    if not catalog.supports_target(entry, target):
        raise ValueError('Skill does not support this agent: ' + target)
    if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', entry['name']):
        raise ValueError('Unsafe installation name')
    catalog.selection_files(entry)
    base = target_registry.project_skills_dir(target) + '/'
    dest = guarded_path(project, base + entry['name'])
    before = project_fingerprint(dest)
    try:
        dest, unchanged = catalog.check_destination(entry, project, target)
        return dest, 'unchanged' if unchanged else 'create', before
    except ValueError as original:
        # Explicit sync may update only an intact previous catalog-owned copy.
        if catalog.is_link(dest) or not dest.is_dir():
            raise original
        receipt = dest / catalog.RECEIPT
        if catalog.is_link(receipt) or not receipt.is_file():
            raise original
        prior = read_json(receipt)
        if (not isinstance(prior, dict)
                or not catalog.supports_target(entry, target)
                or prior.get('id') != entry['id']
                or catalog.payload_hash(catalog.existing_payload(dest)) != prior.get('installed_sha256', prior.get('sha256'))):
            raise original
        # Compare the old installation with its own contract. A new version may
        # add executable helpers that cannot yet exist in the old installation.
        catalog.check_executable_modes(dest, catalog.installed_executables(prior))
        return dest, 'update', before


def project_provenance(entry, dest, data):
    """Explain a validated unchanged copy whose recorded source differs today.

    This is diagnostic only: the receipt records the original installation, while
    the project lock records the current selection. Equal installed content and
    executable contracts require neither fetching the new pin nor relabeling old
    bytes as a new installation.
    """
    receipt = dest / catalog.RECEIPT
    if catalog.is_link(receipt) or not receipt.is_file():
        raise ValueError('Missing or unsafe installed skill receipt: ' + str(receipt))
    prior = read_json(receipt)
    source = (data['sources'][entry['source']] if entry['delivery'] == 'upstream'
              else {'repository': 'codemirket/harness', 'commit': None})
    fields = ('repository', 'commit', 'path', 'sha256')
    original = {key: prior.get(key) for key in fields}
    current = {'repository': source['repository'], 'commit': source['commit'],
               'path': entry['path'], 'sha256': entry['sha256']}
    if original == current:
        return None
    return {'status': 'content_matches_current_selection',
            'original_installation': original, 'current_selection': current,
            'installed_sha256': entry.get('installed_sha256', entry['sha256']),
            'executable_files': sorted(entry.get('executable_files', [])),
            'note': 'Installed bytes and executable contract match the current selection. '
                    'The receipt retains original installation provenance; this match '
                    'does not claim the current source was fetched or reinstalled.'}


def project_jobs(project, config, selections):
    """Preflight an already resolved manifest, including unpublished candidates."""
    entries = selected_entries(selections)
    selected_ids = {target: {entry['id'] for entry in selected}
                    for target, selected in selections.items()}
    jobs = []
    data = None
    for entry in entries:
        exclusions = [
            {'target': target,
             'reason': 'unsupported' if not catalog.supports_target(entry, target)
                       else 'not_requested'}
            for target in config['targets'] if entry['id'] not in selected_ids[target]
        ]
        for target in config['targets']:
            if entry['id'] not in selected_ids[target]:
                continue
            dest, action, before = project_state(entry, project, target)
            job = {'entry': entry, 'target': target, 'destination': dest, 'action': action, 'before': before}
            if action == 'unchanged':
                if data is None:
                    data = catalog.load_catalog()
                provenance = project_provenance(entry, dest, data)
                if provenance is not None:
                    job['provenance'] = provenance
            if config['schema_version'] == 2:
                job['excluded_targets'] = exclusions
            jobs.append(job)
    lock = guarded_path(project, '.ai/project.lock.json')
    if catalog.is_link(lock) or (lock.exists() and not lock.is_file()):
        raise ValueError('Unsafe project lock destination')
    return project, config, entries, jobs


def project_plan(project):
    return project_jobs(*project_config(project, per_target=True))


def preserved_project_copies(project, config, jobs, data, global_names):
    """Report unselected paths without treating their receipts as mutation authority."""
    selected = {job['destination'] for job in jobs}
    known = {entry['id'] for entry in data['skills']}
    diagnostics = []

    def report(path, target, state, identifier=None, code='unselected_installed_copy'):
        diagnostics.append({'kind': 'diagnostic', 'action': 'preserved', 'code': code,
                            'id': identifier, 'name': path.name, 'target': target,
                            'destination': str(path), 'receipt_state': state,
                            'catalog_status': ('known' if identifier in known else 'removed_or_unknown'),
                            'global_overlap': path.name in global_names,
                            'global_skill_ids': global_names.get(path.name, []),
                            'note': 'This path is not selected for this target and is preserved. '
                                    'Review its contents and client skill discovery before cleanup or reselection.'})

    for target in config['targets']:
        relative = target_registry.project_skills_dir(target)
        root = project / relative
        try:
            guarded_path(project, relative)
            if catalog.is_link(root):
                report(root, target, 'not_read_link', code='uninspected_skill_root')
                continue
            if not root.exists():
                continue
            if not root.is_dir():
                report(root, target, 'not_read_unsafe_root', code='uninspected_skill_root')
                continue
            children = sorted(root.iterdir())
        except ValueError:
            report(root, target, 'not_read_unsafe_root', code='uninspected_skill_root')
            continue
        except OSError:
            report(root, target, 'not_read_inaccessible_root', code='uninspected_skill_root')
            continue
        for path in children:
            if path in selected:
                continue
            # Unselected copies are advisory only. Even metadata lookup may fail
            # when a visible directory denies search permission; never let this
            # turn a completed publication into a diagnostic exception.
            try:
                if catalog.is_link(path):
                    report(path, target, 'not_read_link', code='uninspected_skill_copy')
                    continue
                if not path.is_dir():
                    continue
                receipt = path / catalog.RECEIPT
                if catalog.is_link(receipt):
                    report(path, target, 'not_read_link')
                    continue
                if not receipt.exists():
                    continue
                state, identifier = 'invalid', None
                if receipt.is_file():
                    try:
                        with receipt.open('rb') as stream:
                            raw = stream.read(64 * 1024 + 1)
                        value = json.loads(raw) if len(raw) <= 64 * 1024 else None
                        if (isinstance(value, dict) and isinstance(value.get('id'), str)
                                and re.fullmatch(r'[a-z0-9][a-z0-9._-]{0,199}', value['id'])):
                            state, identifier = 'readable', value['id']
                    except (ValueError, UnicodeError, RecursionError):
                        pass
                report(path, target, state, identifier)
            except OSError:
                report(path, target, 'unreadable', code='uninspected_skill_copy')
    return diagnostics


def public_project(jobs, *, project=None, config=None):
    data = catalog.load_catalog()
    global_ids = set(manifest()['global_skills'])
    global_names = {}
    for entry in data['skills']:
        if entry['id'] in global_ids:
            global_names.setdefault(entry['name'], []).append(entry['id'])
    global_names = {name: sorted(identifiers) for name, identifiers in global_names.items()}
    rows = [{'id': j['entry']['id'], 'target': j['target'], 'destination': str(j['destination']),
             'action': j['action'], 'dependencies': j['entry'].get('dependencies', []),
             'caveats': j['entry'].get('caveats', []),
             **({'provenance': j['provenance']} if 'provenance' in j else {}),
             **({'excluded_targets': j['excluded_targets']} if 'excluded_targets' in j else {})}
            for j in jobs]
    for row, job in zip(rows, jobs):
        if job['entry']['name'] in global_names:
            row['warnings'] = [{'code': 'global_skill_name_overlap',
                                'global_skill_ids': global_names[job['entry']['name']],
                                'note': 'This selected project skill shares a configured global skill name. '
                                        'The selection is preserved; check which copy the client discovers.'}]
    if project is not None:
        rows.extend(preserved_project_copies(project, config, jobs, data, global_names))
    return rows


def project_lock(config, entries, data):
    lock = {'schema_version': config['schema_version'], 'manifest': config,
            'skills': [{'id': e['id'], 'sha256': e['sha256'],
                        'installed_sha256': e.get('installed_sha256', e['sha256']),
                        'source': e.get('source', 'personal'),
                        'commit': data['sources'][e['source']]['commit'] if e.get('source') else None}
                       for e in entries]}
    if config['schema_version'] == 2:
        _, selections = project_selection(config, data)
        selected_ids = {target: {entry['id'] for entry in selected}
                        for target, selected in selections.items()}
        for entry in lock['skills']:
            entry['targets'] = [target for target in config['targets']
                                if entry['id'] in selected_ids[target]]
    return lock


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
        if project_fingerprint(dest) != job['before']:
            raise ValueError('Destination changed since preflight: ' + str(dest))
    # All predictable failures precede writes. Every item gets independent recovery.
    published = set()
    for job in jobs:
        entry = job['entry']; dest = job['destination']
        if job['action'] == 'unchanged' or dest in published:
            continue
        with tempfile.TemporaryDirectory(prefix='personal-ai-skill-') as temp:
            sandbox = Path(temp)
            catalog.install(data, entry, sandbox, job['target'], prepared=prepared[entry['id']])
            built = sandbox / target_registry.project_skills_dir(job['target']) / entry['name']
            guarded_path(project, dest.relative_to(project).as_posix())
            replace_item(dest, lambda stage: shutil.copytree(built, stage), job['before'],
                         snapshot=project_fingerprint)
            published.add(dest)
    if not project_lock_current(project, lock):
        write_json(guarded_path(project, '.ai/project.lock.json'), lock)
    return public_project(jobs, project=project, config=config)


def default_project_profiles(config, data):
    """Defaults affect explicit setup/addition only, never passive reconciliation."""
    if 'project_defaults' not in config:
        return []
    defaults = config['project_defaults']
    if (not isinstance(defaults, dict) or set(defaults) != {'profiles'}
            or not isinstance(defaults['profiles'], list)
            or any(not isinstance(value, str) or not value for value in defaults['profiles'])):
        raise ValueError('Harness project_defaults must contain a profiles string list')
    profiles = list(dict.fromkeys(defaults['profiles']))
    for entry in catalog.resolve_selection(data, profiles):
        if entry.get('delivery') != 'local':
            raise ValueError('Harness project_defaults must use authored local project skills: ' + entry['id'])
    return profiles


def init_project(project, profiles, skills, skip, target, target_skills=None):
    project = project.expanduser().resolve(strict=True)
    if not project.is_dir():
        raise ValueError('Project must be a directory')
    config = manifest()
    data = catalog.load_catalog()
    defaults = default_project_profiles(config, data)
    value = {'schema_version': 1, 'targets': targets(target, config),
             'profiles': list(dict.fromkeys(defaults + list(profiles))) if defaults else profiles,
             'skills': skills, 'skip': skip}
    if target_skills is not None:
        value.update(schema_version=2, target_skills=target_skills)
    value, _ = project_selection(value, data)
    dest = guarded_path(project, PROJECT_FILE)
    if catalog.is_link(dest) or (dest.exists() and (not dest.is_file() or read_json(dest) != value)):
        raise ValueError('Project manifest already exists; edit it deliberately: ' + str(dest))
    if not dest.exists():
        write_json(dest, value)
    return value


def add_project(project, profiles=(), skills=(), target_skills=None, dry_run=False):
    """Declare incremental capabilities without installing copies or changing a lock."""
    for field, values in (('profiles', profiles), ('skills', skills)):
        if not isinstance(values, (list, tuple)) or any(not isinstance(x, str) or not x for x in values):
            raise ValueError('Project additions ' + field + ' must be a string list')
    if target_skills is None:
        target_skills = {}
    if (not isinstance(target_skills, dict)
            or any(not isinstance(values, list)
                   or any(not isinstance(x, str) or not x for x in values)
                   for values in target_skills.values())):
        raise ValueError('Project target_skills additions must map targets to string lists')
    if not (profiles or skills or any(target_skills.values())):
        raise ValueError('Select at least one --profile, --skill or --target-skill addition')
    project = project.expanduser().resolve(strict=True)
    if not project.is_dir():
        raise ValueError('Project must be a directory')
    path = guarded_path(project, PROJECT_FILE)
    if catalog.is_link(path) or not path.is_file():
        raise ValueError('Missing or unsafe .ai/project.json; run project init first')

    def identity(item):
        stat = item.lstat()
        return stat.st_dev, stat.st_ino, stat.st_mode

    # The existing manifest's bytes and identity must survive the whole preflight;
    # parent identity catches an otherwise indistinguishable replacement directory.
    parent_identity = {item: identity(item) for item in (project, path.parent)}
    original_identity = identity(path)
    original_bytes = path.read_bytes()
    original = json.loads(original_bytes.decode('utf-8'))
    data = catalog.load_catalog()
    canonical, _ = project_selection(original, data)
    candidate = dict(original)
    defaults = default_project_profiles(manifest(), data)
    if defaults or profiles or 'profiles' in original:
        candidate['profiles'] = list(dict.fromkeys(defaults + original.get('profiles', []) + list(profiles)))
    if skills or 'skills' in original:
        candidate['skills'] = list(dict.fromkeys(original.get('skills', []) + list(skills)))
    scoped = {target: list(dict.fromkeys(identifiers))
              for target, identifiers in canonical.get('target_skills', {}).items()}
    spellings = set()
    for target, identifiers in target_skills.items():
        target = normalize_project_target(target, 'target_skills target')
        if target in spellings:
            raise ValueError('Duplicate target_skills target alias: ' + str(target))
        spellings.add(target)
        if target not in canonical['targets']:
            raise ValueError('target_skills target is not in project targets: ' + target)
        scoped[target] = list(dict.fromkeys(scoped.get(target, []) + identifiers))
    explicit = set(skills).union(identifier for ids in target_skills.values() for identifier in ids)
    skipped = explicit.intersection(original.get('skip', []))
    if skipped:
        raise ValueError('Cannot add skipped skill: ' + sorted(skipped)[0])
    if target_skills or 'target_skills' in original:
        candidate.update(schema_version=2, target_skills=scoped)
    normalized, selections = project_selection(candidate, data)
    _, _, _, jobs = project_jobs(project, normalized, selections)

    def check_unchanged():
        guarded_path(project, PROJECT_FILE)
        if any(catalog.is_link(item) or not item.is_dir() or identity(item) != previous
               for item, previous in parent_identity.items()):
            raise ValueError('Project manifest parent changed since preflight')
        if (catalog.is_link(path) or not path.is_file() or identity(path) != original_identity
                or path.read_bytes() != original_bytes):
            raise ValueError('Project manifest changed since preflight')
        for job in jobs:
            dest = job['destination']
            guarded_path(project, dest.relative_to(project).as_posix())
            if project_fingerprint(dest) != job['before']:
                raise ValueError('Destination changed since preflight: ' + str(dest))

    check_unchanged()
    changed = candidate != original
    if changed and not dry_run:
        stage = path.with_name('.' + path.name + '-' + uuid.uuid4().hex)
        try:
            with stage.open('x', encoding='utf-8') as output:
                output.write(json.dumps(candidate, indent=2, ensure_ascii=False) + '\n')
            stage.chmod(original_identity[2] & 0o777)
            check_unchanged()
            os.replace(stage, path)
        finally:
            if stage.exists():
                stage.unlink()
    return {'status': 'planned' if dry_run else 'declared' if changed else 'unchanged',
            'changed': changed, 'dry_run': dry_run, 'project': str(project),
            'manifest_path': str(path), 'manifest': candidate,
            'installations': public_project(jobs), 'installation': 'not_performed',
            'next_steps': ['project sync', 'project doctor']}


def parse_target_skills(values):
    """Parse repeatable CLI TARGET:ID declarations without ambiguous aliases."""
    if not values:
        return None
    result = {}
    spellings = {}
    for value in values:
        target, separator, identifier = value.partition(':')
        if not separator or not target or not identifier or ':' in identifier:
            raise ValueError('Use --target-skill TARGET:ID')
        canonical = normalize_project_target(target, 'target_skills target')
        if canonical in spellings and spellings[canonical] != target:
            raise ValueError('Duplicate target_skills target alias: ' + canonical)
        spellings[canonical] = target
        result.setdefault(canonical, []).append(identifier)
    return result


def main(argv=None):
    default_target = manifest().get('default_target', 'codex')
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    for name in ('plan', 'sync', 'doctor'):
        p = commands.add_parser(name, help=name + ' declared global instructions and skills')
        p.add_argument('--target', choices=target_registry.CHOICES, default=default_target,
                       help='Client target; both and all mean Codex and Claude')
        p.add_argument('--home', type=Path)
        p.add_argument('--mode', choices=['auto', 'link', 'copy'], default='auto')
    project = commands.add_parser('project').add_subparsers(dest='action', required=True)
    for name in ('init', 'add', 'plan', 'sync', 'doctor'):
        description = ('Declare additional capabilities; run project sync then doctor to install and verify.'
                       if name == 'add' else None)
        p = project.add_parser(name, description=description, help=description)
        p.add_argument('--project', type=Path, required=True)
        if name == 'init':
            p.add_argument('--target', choices=target_registry.CHOICES, default=default_target)
            p.add_argument('--skip', action='append', default=[])
        if name in ('init', 'add'):
            p.add_argument('--profile', action='append', default=[])
            p.add_argument('--skill', action='append', default=[])
            p.add_argument('--target-skill', action='append', default=[], metavar='TARGET:ID',
                           help='Add a skill only to an active target (creates a version 2 manifest)')
        if name == 'add':
            p.add_argument('--dry-run', action='store_true', help='Validate and show the candidate without writes')
    args = parser.parse_args(argv)
    if args.command in ('plan', 'sync', 'doctor'):
        if args.command == 'sync':
            result = sync_global(args.target, args.home, args.mode)
        else:
            result = public_jobs(global_plan(args.target, args.home, args.mode)[3])
        print(json.dumps(result, indent=2, ensure_ascii=False))
        return 1 if args.command == 'doctor' and any(j['action'] != 'unchanged' for j in result) else 0
    if args.action == 'init':
        result = init_project(args.project, args.profile, args.skill, args.skip, args.target,
                              parse_target_skills(args.target_skill))
    elif args.action == 'add':
        result = add_project(args.project, args.profile, args.skill,
                             parse_target_skills(args.target_skill), args.dry_run)
    elif args.action == 'sync':
        result = sync_project(args.project)
    else:
        project, config, entries, jobs = project_plan(args.project)
        result = public_project(jobs, project=project, config=config)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    if args.action == 'doctor':
        lock_ok = project_lock_current(project, project_lock(config, entries, catalog.load_catalog()))
        if not lock_ok:
            print('Missing or stale .ai/project.lock.json; run project sync to reconcile it.', file=sys.stderr)
        return 1 if not lock_ok or any(j['action'] != 'unchanged' for j in jobs) else 0
    return 0
