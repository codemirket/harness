"""Inspect concrete capability contracts without downloading or installing tools."""
import argparse
import json
import re
from pathlib import Path

from . import catalog

ROOT = Path(__file__).resolve().parents[1]
FIELDS = {'id', 'title', 'lead', 'support', 'deliverable', 'failure_probe', 'prerequisites'}


def contracts(root=ROOT):
    path = root / 'registry/capabilities.json'
    if path.is_symlink() or path.stat().st_size > 1024 * 1024:
        raise ValueError('Invalid capability manifest path or size')
    data = json.loads(path.read_text())
    if not isinstance(data, dict) or set(data) != {'schema_version', 'scope', 'capabilities'} or data['schema_version'] != 1:
        raise ValueError('Unsupported capability schema')
    if not isinstance(data['scope'], str) or not data['scope'].strip():
        raise ValueError('Capability scope is required')
    if not isinstance(data['capabilities'], list) or not data['capabilities']:
        raise ValueError('Capabilities must be a nonempty list')
    seen = set()
    for row in data['capabilities']:
        if not isinstance(row, dict) or set(row) != FIELDS:
            raise ValueError('Unknown or missing capability fields')
        for key in FIELDS - {'support'}:
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
    return data


def check(root=ROOT):
    """Verify source contracts and local payloads, never infer installed readiness."""
    root = Path(root)
    data = contracts(root)
    index = catalog.load_catalog(root / 'registry/catalog.json')
    by_id = {entry['id']: entry for entry in index['skills']}
    if len(by_id) != len(index['skills']):
        raise ValueError('Duplicate catalog ID')
    globals_ = json.loads((root / 'registry/harness.json').read_text())['global_skills']
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
    data = contracts(root)
    rows = [row for row in data['capabilities'] if row['id'] == identifier]
    if not rows:
        raise ValueError('Unknown capability: ' + identifier)
    row = dict(rows[0])
    index = {e['id']: e for e in catalog.load_catalog(root / 'registry/catalog.json')['skills']}
    globals_ = set(json.loads((root / 'registry/harness.json').read_text())['global_skills'])
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
             'These are required outcomes and failure probes, not claims that every domain has passed a production trial.', '',
             '| Capability / ID | Lead | Deliverable | Discriminating failure probe | Prerequisites |',
             '| --- | --- | --- | --- | --- |']
    for row in data['capabilities']:
        cells = [row['title'] + ' (`' + row['id'] + '`)', '`' + row['lead'] + '`', row['deliverable'], row['failure_probe'], row['prerequisites']]
        lines.append('| ' + ' | '.join(c.replace('|', '\\|').replace('\n', ' ') for c in cells) + ' |')
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    listing = commands.add_parser('list')
    listing.add_argument('--markdown', action='store_true')
    show = commands.add_parser('show'); show.add_argument('id')
    commands.add_parser('check')
    args = parser.parse_args(argv)
    if args.command == 'list':
        print(markdown() if args.markdown else json.dumps(contracts(), indent=2), end='' if args.markdown else '\n')
        return 0
    if args.command == 'show':
        print(json.dumps(describe(args.id), indent=2))
        return 0
    report = check()
    print(json.dumps(report, indent=2))
    return 0 if report['status'] == 'valid' else 1
