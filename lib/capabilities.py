"""Inspect concrete capability contracts without downloading or installing tools."""
import argparse
import json
import re
import hashlib
from pathlib import Path

from . import catalog

ROOT = Path(__file__).resolve().parents[1]
TEXT_FIELDS = {'id', 'title', 'lead', 'deliverable', 'failure_probe', 'prerequisites'}
LIST_FIELDS = {'support', 'aliases', 'acceptance'}
FIELDS = TEXT_FIELDS | LIST_FIELDS


def normalized(value):
    """Role lookup is explicit, not an inferred natural-language classifier."""
    return ' '.join(re.findall(r'\w+', value.casefold().replace('_', ' ')))


def contracts(root=ROOT):
    path = root / 'registry/capabilities.json'
    if path.is_symlink() or path.stat().st_size > 1024 * 1024:
        raise ValueError('Invalid capability manifest path or size')
    data = json.loads(path.read_text(encoding='utf-8'))
    if (not isinstance(data, dict) or set(data) != {'schema_version', 'scope', 'capabilities'}
            or type(data['schema_version']) is not int or data['schema_version'] != 1):
        raise ValueError('Unsupported capability schema')
    if not isinstance(data['scope'], str) or not data['scope'].strip():
        raise ValueError('Capability scope is required')
    if not isinstance(data['capabilities'], list) or not data['capabilities']:
        raise ValueError('Capabilities must be a nonempty list')
    seen, names = set(), {}
    for row in data['capabilities']:
        if not isinstance(row, dict) or set(row) != FIELDS:
            raise ValueError('Unknown or missing capability fields')
        for key in TEXT_FIELDS:
            if not isinstance(row[key], str) or not row[key].strip():
                raise ValueError('Capability field must be nonempty text: ' + key)
        identifier = row['id']
        if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', identifier) or identifier in seen:
            raise ValueError('Duplicate or invalid capability ID: ' + identifier)
        seen.add(identifier)
        if not isinstance(row['support'], list) or any(not isinstance(s, str) or not s for s in row['support']):
            raise ValueError('Invalid supporting skills: ' + identifier)
        if len(set(row['support'])) != len(row['support']) or row['lead'] in row['support']:
            raise ValueError('Duplicate lead/support skills: ' + identifier)
        for key in ('aliases', 'acceptance'):
            values = row[key]
            if (not isinstance(values, list) or not values
                    or any(not isinstance(s, str) or not s.strip() for s in values)
                    or len(set(values)) != len(values)):
                raise ValueError('Invalid capability ' + key + ': ' + identifier)
        for name in [identifier, row['title']] + row['aliases']:
            key = normalized(name)
            if not key or (key in names and names[key] != identifier):
                raise ValueError('Ambiguous capability name: ' + name)
            names[key] = identifier
    return data


def resolve(name, root=ROOT):
    """Resolve a declared ID, title or alias; never guess an install selection."""
    key = normalized(name)
    for row in contracts(root)['capabilities']:
        if key in {normalized(s) for s in [row['id'], row['title']] + row['aliases']}:
            return row
    raise ValueError('Unknown capability: ' + name + '. Use capabilities list for IDs and role aliases.')


def selected_skills(names, root=ROOT):
    """Project registration persists concrete leads, not expanding role profiles."""
    globals_ = set(json.loads((Path(root) / 'registry/harness.json').read_text(encoding='utf-8'))['global_skills'])
    leads = [resolve(name, root)['lead'] for name in names]
    return list(dict.fromkeys(lead for lead in leads if lead not in globals_))


def plan(name, supporting=(), root=ROOT):
    """Compose an inspectable execution brief without installing or running code."""
    root = Path(root)
    primary = resolve(name, root)
    rows = [primary]
    for value in supporting:
        row = resolve(value, root)
        if row['id'] not in {r['id'] for r in rows}:
            rows.append(row)
    index = {e['id']: e for e in catalog.load_catalog(root / 'registry/catalog.json')['skills']}
    globals_ = set(json.loads((root / 'registry/harness.json').read_text(encoding='utf-8'))['global_skills'])
    reads = []
    for identifier in dict.fromkeys(row['lead'] for row in rows):
        entry = index.get(identifier)
        if (not entry or entry.get('scope') not in ('project', 'global')
                or entry.get('delivery') not in ('local', 'global-link')
                or set(entry.get('agents', ['codex', 'claude'])) != {'codex', 'claude'}):
            raise ValueError('Capability lead is not portable authored guidance: ' + identifier)
        if entry['scope'] == 'global' and identifier not in globals_:
            raise ValueError('Global capability lead is not registered: ' + identifier)
        payload = catalog.read_local(root, dict(entry, license_files=entry.get('license_files', [])))
        digest = catalog.payload_hash(payload)
        if entry['delivery'] == 'local' and entry.get('sha256') != digest:
            raise ValueError('Capability lead hash changed: ' + identifier)
        body = payload.get('SKILL.md', b'')
        if catalog.skill_name(body) != entry['name']:
            raise ValueError('Capability lead invocation mismatch: ' + identifier)
        reads.append({'id': identifier, 'name': entry['name'],
                      'path': entry['path'] + '/SKILL.md',
                      'entrypoint_bytes': len(body), 'payload_sha256': digest,
                      'delivery': 'shared' if identifier in globals_ else 'project'})
    ids = selected_skills([row['id'] for row in rows], root)
    # The manifest remains the owner of foundation profiles and target membership.
    # This brief never loads all optional support, guesses tools or claims readiness.
    return {
        'schema_version': 1, 'primary': primary['id'],
        'supporting_capabilities': [row['id'] for row in rows[1:]],
        'contracts': rows, 'read_sequence': reads,
        'entrypoint_bytes': sum(row['entrypoint_bytes'] for row in reads),
        'project_skills': ids,
        'registration_arguments': [arg for identifier in ids for arg in ('--skill', identifier)],
        'runtime_status': 'not_checked', 'activation_status': 'not_observed',
        'source_fingerprint': hashlib.sha256(json.dumps(
            {'contracts': rows, 'reads': reads}, sort_keys=True).encode()).hexdigest(),
        'next_steps': [
            'Read the selected lead and only the references relevant to the requested outcome.',
            'Check existing global and project selections; register missing project leads, then sync and doctor.',
            'Inspect actual tools and data; prerequisites here are requirements, not readiness observations.',
            'Execute the requested task and inspect its deliverable and failure probe before claiming completion.'
        ],
        'limits': [
            'Exact role/ID lookup and explicit composition; no prompt classifier or automatic model/tool execution.',
            'Supporting skills listed in contracts are optional; only explicitly composed leads appear in read_sequence.',
            'Shared skills require the global installation; portable-foundation is an explicit fallback on other hosts.',
            'A role grants no authority, tool access, runtime isolation or professional certification.'
        ]}


def check(root=ROOT):
    """Verify source contracts and local payloads, never infer installed readiness."""
    root = Path(root)
    data = contracts(root)
    index = catalog.load_catalog(root / 'registry/catalog.json')
    by_id = {entry['id']: entry for entry in index['skills']}
    if len(by_id) != len(index['skills']):
        raise ValueError('Duplicate catalog ID')
    globals_ = json.loads((root / 'registry/harness.json').read_text(encoding='utf-8'))['global_skills']
    checked, failures = {}, []
    for row in data['capabilities']:
        for identifier in [row['lead']] + row['support']:
            if identifier in checked:
                continue
            try:
                entry = by_id.get(identifier)
                if not entry or entry.get('scope') not in ('global', 'project'):
                    raise ValueError('Missing or noninstallable skill')
                if entry.get('delivery') not in ('local', 'global-link'):
                    raise ValueError('Core capability contract requires inspectable authored local guidance')
                if set(entry.get('agents', ['codex', 'claude'])) != {'codex', 'claude'}:
                    raise ValueError('Skill does not cover both installation targets')
                if entry['scope'] == 'global' and identifier not in globals_:
                    raise ValueError('Global lead/support is not in the shared installation')
                payload = catalog.read_local(root, dict(entry, license_files=entry.get('license_files', [])))
                if catalog.skill_name(payload.get('SKILL.md', b'')) != entry['name']:
                    raise ValueError('Skill invocation name mismatch')
                digest = catalog.payload_hash(payload)
                if entry['delivery'] == 'local' and entry.get('sha256') != digest:
                    raise ValueError('Local skill changed without reviewed catalog hash')
                checked[identifier] = {'status': 'source_verified', 'sha256': digest}
            except (OSError, ValueError, KeyError) as error:
                checked[identifier] = {'status': 'invalid', 'error': str(error)}
                failures.append({'skill': identifier, 'error': str(error)})
    return {'status': 'valid' if not failures else 'invalid', 'capabilities': len(data['capabilities']),
            'skills': checked, 'failures': failures,
            'limits': ['Checks establish local guidance and catalog integrity, not skill activation or task quality.',
                       'Prerequisites and actual project registration must be checked on the target host.',
                       'A delivery is accepted only after its observable criteria and relevant failure probes are exercised.']}


def describe(identifier, root=ROOT):
    row = dict(resolve(identifier, root))
    index = {e['id']: e for e in catalog.load_catalog(root / 'registry/catalog.json')['skills']}
    globals_ = set(json.loads((root / 'registry/harness.json').read_text(encoding='utf-8'))['global_skills'])
    row['lead_source'] = index[row['lead']]['path'] + '/SKILL.md'
    row['project_lead'] = [] if row['lead'] in globals_ else [row['lead']]
    row['optional_project_support'] = [s for s in row['support'] if s not in globals_]
    row['next_step'] = 'Read the lead; add needed project IDs with project add, then project sync and project doctor. Do not install all supporting skills by default.'
    row['runtime_status'] = 'not_checked'
    return row


def markdown(root=ROOT):
    data = contracts(root)
    lines = ['# Capability contracts', '', data['scope'], '',
             'Generated from `registry/capabilities.json` by `python3 ai.py capabilities list --markdown`.',
             'Use `capabilities show ID` for selection details and `capabilities check` for source integrity.',
             'Compose a task with `capabilities plan "CFO" --with "Excel Expert"`; register a project lead with `project add --capability "CFO"`.',
             'Lookup accepts exact IDs, titles and listed role aliases. Composition loads only selected leads; optional support stays conditional.',
             'These are required outcomes and failure probes, not claims that every domain has passed a production trial.', '',
             '| Capability / ID | Lead | Deliverable | Discriminating failure probe | Prerequisites |',
             '| --- | --- | --- | --- | --- |']
    for row in data['capabilities']:
        cells = [row['title'] + ' (`' + row['id'] + '`)', '`' + row['lead'] + '`', row['deliverable'], row['failure_probe'], row['prerequisites']]
        lines.append('| ' + ' | '.join(c.replace('|', '\\|').replace('\n', ' ') for c in cells) + ' |')
    lines += ['', '## Role names and acceptance', '', '| Capability | Role aliases | Acceptance evidence |', '| --- | --- | --- |']
    for row in data['capabilities']:
        cells = [row['id'], '; '.join(row['aliases']), '; '.join(row['acceptance'])]
        lines.append('| ' + ' | '.join(c.replace('|', '\\|').replace('\n', ' ') for c in cells) + ' |')
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    listing = commands.add_parser('list')
    listing.add_argument('--markdown', action='store_true')
    show = commands.add_parser('show'); show.add_argument('id')
    planning = commands.add_parser('plan', help='Compose an explicit role and optional supporting roles without writes')
    planning.add_argument('role')
    planning.add_argument('--with', dest='supporting', action='append', default=[], metavar='ROLE')
    commands.add_parser('check')
    args = parser.parse_args(argv)
    if args.command == 'list':
        print(markdown() if args.markdown else json.dumps(contracts(), indent=2), end='' if args.markdown else '\n')
        return 0
    if args.command == 'show':
        print(json.dumps(describe(args.id), indent=2))
        return 0
    if args.command == 'plan':
        print(json.dumps(plan(args.role, args.supporting), indent=2))
        return 0
    report = check()
    print(json.dumps(report, indent=2))
    return 0 if report['status'] == 'valid' else 1
