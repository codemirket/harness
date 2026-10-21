#!/usr/bin/env python3
"""Extract repository metadata only; never import runtime code or inspect user state."""
import collections
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/harness-map'

def read(path):
    return json.loads((ROOT / path).read_text())

def file_record(path):
    raw = path.read_bytes()
    return {'path': path.relative_to(ROOT).as_posix(), 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}

def link(path, label=None):
    return f'[{label or path}](../../{path})'

def cell(value):
    if isinstance(value, list):
        value = ', '.join(str(v) for v in value)
    return str(value).replace('|', '\\|').replace('\n', ' ')

catalog = read('registry/catalog.json')
capabilities = read('registry/capabilities.json')
cargo = tomllib.loads((ROOT / 'Cargo.toml').read_text())
lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
node = read('runtime/design/tools/package-lock.json')
edges = []
for skill in catalog['skills']:
    for field in ('requires', 'conflicts', 'replacements', 'informed_by'):
        values = skill.get(field, [])
        if isinstance(values, str):
            values = [values]
        for value in values:
            edges.append({'from': 'skill:' + skill['id'], 'relation': field, 'to': value})
    if skill.get('source'):
        edges.append({'from': 'skill:' + skill['id'], 'relation': 'source', 'to': skill['source']})
for name, profile in catalog['profiles'].items():
    for skill in profile['skills']:
        edges.append({'from': 'profile:' + name, 'relation': 'includes', 'to': 'skill:' + skill})
for role in capabilities['capabilities']:
    for skill in [role['lead']] + role['support']:
        edges.append({'from': 'role:' + role['id'], 'relation': 'lead' if skill == role['lead'] else 'optional_support', 'to': 'skill:' + skill})
for workflow in capabilities['workflows']:
    edges.append({'from': 'workflow:' + workflow['id'], 'relation': 'default_role', 'to': 'role:' + workflow['role']})
    for tool in workflow['tools']:
        edges.append({'from': 'workflow:' + workflow['id'], 'relation': 'allows', 'to': 'tool:' + tool})
for facet in capabilities['team_facets']:
    for assignment in facet['assignments']:
        edges.append({'from': 'facet:' + facet['id'], 'relation': assignment['mode'], 'to': 'role:' + assignment['role']})
refs = [file_record(p) for p in sorted((ROOT / 'skills').rglob('*')) if p.is_file() and 'references' in p.relative_to(ROOT).parts and p.name != '.DS_Store']
inputs = ['registry/catalog.json', 'registry/capabilities.json', 'registry/harness.json', 'registry/design-stack.json', 'registry/agent-runtime.json', 'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'runtime/design/tools/package.json', 'runtime/design/tools/package-lock.json', 'runtime/design/tools/tool-policy.json', 'runtime/agent/requirements.in', 'runtime/agent/requirements.lock', 'runtime/agent/model.json', 'runtime/agent/bm25.json', 'src/mcp.rs']
mcp = re.findall(r'#\[tool\([\s\S]*?\)\]\s*async fn (\w+)', (ROOT / 'src/mcp.rs').read_text())
inventory = {
 'schema_version': 1,
 'scope': 'Static repository inventory; declarations do not establish runtime availability, dependency freshness or quality.',
 'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
 'version': cargo['package']['version'],
 'input_files': [file_record(ROOT / p) for p in inputs],
 'counts': {'skills': len(catalog['skills']), 'delivery': dict(collections.Counter(s['delivery'] for s in catalog['skills'])), 'profiles': len(catalog['profiles']), 'profile_kinds': dict(collections.Counter(p.get('kind') for p in catalog['profiles'].values())), 'roles': len(capabilities['capabilities']), 'facets': len(capabilities['team_facets']), 'workflows': len(capabilities['workflows']), 'sources': len(catalog['sources']), 'reference_files': len(refs), 'main_mcp_tools': len(mcp)},
 'catalog': catalog, 'capabilities': capabilities, 'global': read('registry/harness.json'),
 'packages': {'design': read('registry/design-stack.json'), 'agent-runtime': read('registry/agent-runtime.json')},
 'dependencies': {'rust_manifest': cargo, 'rust_locked_packages': lock['package'], 'node_manifest': read('runtime/design/tools/package.json'), 'node_locked_packages': node['packages'], 'python_requirements': (ROOT / 'runtime/agent/requirements.in').read_text().splitlines(), 'python_locked_requirements': [s for s in (ROOT / 'runtime/agent/requirements.lock').read_text().splitlines() if s and not s.startswith('#')], 'embedding': read('runtime/agent/model.json'), 'bm25': read('runtime/agent/bm25.json')},
 'main_mcp_tools': mcp, 'reference_files': refs, 'edges': edges,
 'test_files': [p.relative_to(ROOT).as_posix() for base in ['tests', 'runtime/design/tools/test', 'evals'] for p in sorted((ROOT / base).rglob('*')) if p.is_file() and p.suffix in ('.rs', '.py', '.cjs', '.mjs')],
}
(OUT / 'inventory.json').write_text(json.dumps(inventory, indent=2, ensure_ascii=False) + '\n')
lines = ['# Harness inventory', '', f"Generated from `{inventory['revision']}` by [extract.py](../../evals/harness-map/extract.py). Full contracts, provenance, locks and graph edges are in [inventory.json](inventory.json).", '', 'This is a declaration inventory, not an installed-capability or quality report.', '', '## Roles', '', '| ID | Lead skill | Optional support | Aliases |', '| --- | --- | --- | --- |']
for r in capabilities['capabilities']:
    lines.append('| ' + ' | '.join(cell(r[k]) for k in ['id', 'lead', 'support', 'aliases']) + ' |')
lines += ['', '## Worker workflows', '', '| ID | Default role | Allowed tools | Required invocation | Verification tools |', '| --- | --- | --- | --- | --- |']
for w in capabilities['workflows']:
    lines.append('| ' + ' | '.join(cell(w.get(k, [])) for k in ['id', 'role', 'tools', 'required_tools', 'verification_tools']) + ' |')
lines += ['', '**Execution scope:** `editable-interface` uses the four OpenPencil file wrappers in the assigned design workspace. Inventory does not establish runtime readiness or useful output. See [tools.rs](../../src/router/tools.rs).', '', '## Profiles', '', '| ID | Kind | Skills |', '| --- | --- | --- |']
for name,p in catalog['profiles'].items():
    lines.append('| ' + ' | '.join(cell(v) for v in [name,p.get('kind'),p['skills']]) + ' |')
lines += ['', '## Skills', '', '| ID | Delivery | Category | Source or local body | Requires |', '| --- | --- | --- | --- | --- |']
for s in catalog['skills']:
    path = s.get('path', '')
    loc = link(path + '/SKILL.md') if (ROOT / path / 'SKILL.md').is_file() else s.get('source', s.get('url', path))
    lines.append('| ' + ' | '.join(cell(v) for v in [s['id'],s['delivery'],s['category'],loc,s.get('requires',[])]) + ' |')
lines += ['', '## Source pins', '', '| Source | Repository / URL | Commit | License summary |', '| --- | --- | --- | --- |']
for name,s in catalog['sources'].items():
    lines.append('| ' + ' | '.join(cell(v) for v in [name,s.get('url',s.get('repository','')),s.get('commit',''),s.get('license_summary','')]) + ' |')
lines += ['', '## Team facets', '', '| Facet | Applies when | Assignments |', '| --- | --- | --- |']
for f in capabilities['team_facets']:
    lines.append('| ' + ' | '.join(cell(v) for v in [f['id'],f['when'],[a['role'] + ':' + a['mode'] for a in f['assignments']]]) + ' |')
lines += ['', '## Embedded reference files', '', 'All files under authored skill `references/` directories, including data and specimens. This count is distinct from catalog delivery `reference` entries and the 27 design-reference instances.', '']
lines += ['- ' + link(r['path']) for r in refs]
(OUT / 'inventory.md').write_text('\n'.join(lines) + '\n')
print(json.dumps(inventory['counts'], indent=2))
