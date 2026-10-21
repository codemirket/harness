#!/usr/bin/env python3
"""Extract repository metadata only; never import runtime code or inspect user state."""
import collections
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'docs/harness-map'
outputs = {}
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--check', action='store_true', help='Compare generated outputs without writing')
args = parser.parse_args()

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
    return str(value).replace('|', '\\|').replace('\n', ' ').replace('<', '&lt;').replace('>', '&gt;')

catalog = read('registry/catalog.json')
capabilities = read('registry/capabilities.json')
cargo = tomllib.loads((ROOT / 'Cargo.toml').read_text())
lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
npm_lock = read('runtime/design/tools/package-lock.json')
edges = []
for skill in catalog['skills']:
    # replacements are literal payload patches, not skill replacement edges.
    # Preserve them in the complete catalog records below.
    for field in ('requires', 'conflicts', 'informed_by'):
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
 'dependencies': {'rust_manifest': cargo, 'rust_locked_packages': lock['package'], 'node_manifest': read('runtime/design/tools/package.json'), 'node_locked_packages': npm_lock['packages'], 'python_requirements': (ROOT / 'runtime/agent/requirements.in').read_text().splitlines(), 'python_locked_requirements': [s for s in (ROOT / 'runtime/agent/requirements.lock').read_text().splitlines() if s and not s.startswith('#')], 'embedding': read('runtime/agent/model.json'), 'bm25': read('runtime/agent/bm25.json')},
 'main_mcp_tools': mcp, 'reference_files': refs, 'edges': edges,
 'test_files': [p.relative_to(ROOT).as_posix() for base in ['tests', 'runtime/design/tools/test', 'evals'] for p in sorted((ROOT / base).rglob('*')) if p.is_file() and p.suffix in ('.rs', '.py', '.cjs', '.mjs')],
}
lines = ['# Harness inventory', '', f"Generated from working files based on HEAD `{inventory['revision']}` by [extract.py](../../evals/harness-map/extract.py). Full contracts, provenance, locks, input hashes, source changes and graph edges are in [inventory.json](inventory.json).", '', 'This is a declaration inventory, not an installed-capability or quality report. Read the [system map](../harness-map.md), [professional methods](methodologies.md) and [dependency appendix](dependencies.md).', '', '## Roles', '', '| ID | Lead skill | Optional support | Aliases |', '| --- | --- | --- | --- |']
for r in capabilities['capabilities']:
    lines.append('| ' + ' | '.join(cell(r[k]) for k in ['id', 'lead', 'support', 'aliases']) + ' |')
lines += ['', '## Worker workflows', '', '| ID | Default role | Owner | Floor | Allowed tools | Required invocation | Verification tools |', '| --- | --- | --- | --- | --- | --- | --- |']
for w in capabilities['workflows']:
    lines.append('| ' + ' | '.join(cell(v) for v in [w['id'], w['role'], w.get('execution_owner', 'worker'), w['min_tier'], w['tools'], w['required_tools'], w['verification_tools']]) + ' |')
lines += ['', '**Execution scope:** `editable-interface` uses the four OpenPencil file wrappers in the assigned design workspace. Inventory does not establish runtime readiness or useful output. See [tools.rs](../../src/router/tools.rs).', '', '## Profiles', '', 'Membership is declared selection, before recursive requires and target compatibility resolution. include_globals adds the global guidance set.', '', '| ID | Kind | Include globals | Purpose | Skills |', '| --- | --- | --- | --- | --- |']
for name,p in catalog['profiles'].items():
    lines.append('| ' + ' | '.join(cell(v) for v in [name,p.get('kind'),p.get('include_globals',False),p['description'],p['skills']]) + ' |')
lines += ['', '## Skills', '', '| ID | Delivery | Category | Source or local body | Requires |', '| --- | --- | --- | --- | --- |']
for s in catalog['skills']:
    path = s.get('path', '')
    loc = link(path + '/SKILL.md') if s['delivery'] == 'embedded' else s.get('url', s.get('source', path))
    lines.append('| ' + ' | '.join(cell(v) for v in [s['id'],s['delivery'],s['category'],loc,s.get('requires',[])]) + ' |')
lines += ['', '## Source pins', '', '| Source | Repository / URL | Commit | License summary |', '| --- | --- | --- | --- |']
for name,s in catalog['sources'].items():
    lines.append('| ' + ' | '.join(cell(v) for v in [name,s.get('url',s.get('repository','')),s.get('commit',''),s.get('license_summary','')]) + ' |')
lines += ['', '## Team facets', '', '| Facet | Applies when | Assignments |', '| --- | --- | --- |']
for f in capabilities['team_facets']:
    lines.append('| ' + ' | '.join(cell(v) for v in [f['id'],f['when'],[a['role'] + ':' + a['mode'] + ' — ' + a['contribution'] for a in f['assignments']]]) + ' |')
lines += ['', '## Embedded reference files', '', 'All files under authored skill `references/` directories, including data and specimens. This count is distinct from catalog delivery `reference` entries and the 27 design-reference instances.', '']
lines += ['- ' + link(r['path']) for r in refs]
outputs['inventory.md'] = '\n'.join(lines) + '\n'

# Follow the compiled-source framing instead of treating HEAD as working content.
def owned_files(path):
    if path.is_symlink():
        raise ValueError(f'source symlink: {path}')
    if path.name == '.DS_Store':
        return []
    return [p for c in sorted(path.iterdir()) for p in owned_files(c)] if path.is_dir() else [path]

source_roots = re.findall(r'"([^"]+)"', re.search(r'INPUTS:.*?= &\[(.*?)\];', (ROOT / 'src/source_identity.rs').read_text(), re.S)[1])
source_files = sorted(p for name in source_roots for p in owned_files(ROOT / name))
source_hash = hashlib.sha256()
for path in source_files:
    name, data = path.relative_to(ROOT).as_posix().encode(), path.read_bytes()
    source_hash.update(len(name).to_bytes(8, 'little')); source_hash.update(name)
    source_hash.update(len(data).to_bytes(8, 'little')); source_hash.update(data)
extra_inputs = ['AGENTS.md', 'CONTRIBUTING.md', 'docs/architecture.md', '.github/workflows/check.yml',
                'evals/harness-map/extract.py', 'evals/dependency-audit/check.py', 'evals/dependency-audit/exceptions.json']
inventory.update(schema_version=2, source_sha256=source_hash.hexdigest(),
    branch=subprocess.check_output(['git','branch','--show-current'],cwd=ROOT,text=True).strip(),
    source_changes_from_head=subprocess.check_output(['git','diff','HEAD','--name-only','--',*source_roots],cwd=ROOT,text=True).splitlines(),
    audit_changes_from_head=subprocess.check_output(['git','diff','HEAD','--name-only','--','evals/dependency-audit'],cwd=ROOT,text=True).splitlines(),
    input_files=[file_record(p) for p in sorted(set(source_files + [ROOT / p for p in extra_inputs]))])
inventory['scope'] = 'Working-tree source declarations and relationships; not an installed SBOM, adoption result or runtime readiness probe.'

# Parse owned declarations without importing or executing runtime modules.
policies = read('runtime/design/tools/tool-policy.json')
budgets = read('runtime/design/tools/tool-budgets.json')['tools']
server = (ROOT / 'runtime/design/tools/server.mjs').read_text().split('const definitions = [',1)[1].split('\n];',1)[0]
design_tools = [{'name':m[1], 'description':m[2], 'policy':policies[m[1]], 'outer_timeout_ms':budgets[m[1]]}
                for m in re.finditer(r"\{name:\s*'([^']+)',\s*description:\s*'((?:[^'\\]|\\.)*)'", server)]
if {t['name'] for t in design_tools} != set(policies) or set(policies) != set(budgets):
    raise ValueError('design declarations, policy and budgets differ')
main_tools = []
for match in re.finditer(r'#\[tool\((.*?)\)\]\s*async fn (\w+)', (ROOT / 'src/mcp.rs').read_text(), re.S):
    description = re.search(r'description\s*=\s*("(?:[^"\\]|\\.)*")', match[1])
    main_tools.append({'name':match[2], 'description':json.loads(description[1]), 'annotations_source':match[1].strip()})
if [t['name'] for t in main_tools] != mcp:
    raise ValueError('main MCP extraction is incomplete')
pencil = read('runtime/design/tools/openpencil-tools.json')
inventory.update(main_mcp_contracts=main_tools, design_mcp_tools=design_tools, openpencil=pencil)
bodies = []
for skill in catalog['skills']:
    if skill['delivery'] == 'embedded':
        path = ROOT / skill['path'] / 'SKILL.md'
        body = path.read_text()
        bodies.append({'id':skill['id'], **file_record(path),
            'description':re.search(r'^description: (.+)$',body,re.M)[1],
            'headings':re.findall(r'^#{2,3} (.+)$',body,re.M)})
inventory['embedded_bodies'] = bodies

# Typed expertise graph. Dependency locks retain their own resolved graphs below.
nodes = {}
def add_node(key, kind):
    nodes[key] = {'id':key, 'kind':kind}
def edge(source, relation, target):
    edges.append({'from':source, 'relation':relation, 'to':target})
for kind, rows in [('skill',catalog['skills']),('role',capabilities['capabilities']),('workflow',capabilities['workflows']),('facet',capabilities['team_facets'])]:
    for row in rows: add_node(kind+':'+row['id'],kind)
for kind, names in [('source',catalog['sources']),('profile',catalog['profiles']),('package',['design','agent-runtime']),('host',['codex','claude'])]:
    for name in names: add_node(kind+':'+name,kind)
for name in ['workspace_read','workspace_list','workspace_write','inspect_image'] + ['design/'+t['name'] for t in design_tools]: add_node('tool:'+name,'worker_tool')
for t in main_tools: add_node('main_mcp:'+t['name'],'main_mcp')
for r in refs + bodies: add_node('file:'+r['path'],'file')
for e in edges:
    if ':' not in e['to']:
        # Relation semantics disambiguate IDs shared by a skill and its source.
        prefixes = ['source'] if e['relation'] in ('source', 'informed_by') else ['skill']
        candidates = [prefix+':'+e['to'] for prefix in prefixes if prefix+':'+e['to'] in nodes]
        if len(candidates) == 1: e['to'] = candidates[0]
        else:
            e['to'] = 'declaration:'+e['to']; add_node(e['to'],'unresolved_declaration')
for profile, value in catalog['profiles'].items():
    if value.get('include_globals'):
        for skill in inventory['global']['global_skills']: edge('profile:'+profile,'includes_global','skill:'+skill)
for host in ['codex','claude']:
    for skill in inventory['global']['global_skills']: edge('host:'+host,'setup_global','skill:'+skill)
for skill in catalog['skills']:
    if skill['delivery'] != 'embedded': continue
    edge('skill:'+skill['id'],'body','file:'+skill['path']+'/SKILL.md')
    for r in refs:
        if r['path'].startswith(skill['path']+'/'): edge('skill:'+skill['id'],'reference','file:'+r['path'])
for w in capabilities['workflows']:
    for role in w.get('alternative_roles',[]): edge('workflow:'+w['id'],'alternative_role','role:'+role)
    for field in ['required_tools','verification_tools']:
        for tool in w[field]: edge('workflow:'+w['id'],field,'tool:'+tool)
for t in design_tools: edge('package:design','exposes','tool:design/'+t['name'])
for t in pencil['tools']:
    key='openpencil:'+t['name']; add_node(key,'openpencil_operation')
    edge('tool:design/openpencil_'+('inspect' if t['effect']=='read' else 'edit'),'dispatches',key)
if any(e['from'] not in nodes or e['to'] not in nodes for e in edges): raise ValueError('unresolved typed graph endpoint')
inventory.update(nodes=sorted(nodes.values(),key=lambda n:n['id']), edges=sorted(edges,key=lambda e:(e['from'],e['relation'],e['to'])))

python_wheels=[]
for line in inventory['dependencies']['python_locked_requirements']:
    name, rest=line.split(' @ ',1); url, sha=rest.split(' --hash=sha256:',1)
    wheel=url.rsplit('/',1)[1]; parts=wheel.removesuffix('.whl').split('-')
    python_wheels.append({'name':name,'version':parts[1],'url':url,'sha256':sha,'wheel':wheel,'python_tag':parts[-3],'abi_tag':parts[-2],'platform_tag':parts[-1]})
inventory['dependencies'].update(python_wheels=python_wheels,
    graph_limits='Cargo and npm records preserve their native dependency graph fields. The Python wheel lock is flat; Requires-Dist edges are unavailable and not inferred. Optional/platform npm records are not necessarily installed. OS inventories require separate image evidence.')
environment=collections.defaultdict(set)
for path in source_files:
    if path.suffix not in ('.rs','.py','.mjs') or path.relative_to(ROOT).parts[0] not in ('src','runtime'): continue
    for name in re.findall(r'\b(?:MIRKET_[A-Z0-9_]+|OPENAI_[A-Z0-9_]+|ANTHROPIC_[A-Z0-9_]+|OPENROUTER_[A-Z0-9_]+|HYPERFRAMES_[A-Z0-9_]+|HF_HUB_OFFLINE|FASTEMBED_CACHE_PATH|MEM0_TELEMETRY|LANGSMITH_TRACING|TOKENIZERS_PARALLELISM)\b',path.read_text()):
        environment[name].add(path.relative_to(ROOT).as_posix())
inventory['environment_names']={k:sorted(v) for k,v in sorted(environment.items())}
inventory['environment_limit']='Lexical references, including tests; not current values or a supported settings schema. No environment values were read.'
inventory['counts'].update(global_skills=len(inventory['global']['global_skills']),runtime_packages=2,
    design_mcp_tools=len(design_tools),worker_tools=4+len(design_tools),
    openpencil_operations=dict(collections.Counter(t['effect'] for t in pencil['tools'])),
    cargo_lock_records=len(lock['package']),npm_lock_records_including_root=len(inventory['dependencies']['node_locked_packages']),
    python_locked_wheels=len(python_wheels),graph_nodes=len(nodes),graph_edges=len(edges))

def table(headers, rows):
    return ['| '+' | '.join(headers)+' |','| '+' | '.join(['---']*len(headers))+' |'] + ['| '+' | '.join(cell(v) for v in row)+' |' for row in rows]
intro=[f"Working-source SHA-256: `{inventory['source_sha256']}`; base HEAD `{inventory['revision']}`.",'',
       'Generated by [extract.py](../../evals/harness-map/extract.py). Read the [system map](../harness-map.md) for ownership, side effects and evidence limits. Full records: [inventory.json](inventory.json).','']
extra=['','## Snapshot counts','']+table(['Measure','Value'],inventory['counts'].items())
extra+=['','## Global guidance','','Default host setup: '+cell(inventory['global']['global_skills']), '', '## Main MCP surface','','Annotations are hints; read-like operations can still open state, refresh discovery or launch a contained runtime.','']
extra+=table(['Tool','Declared contract'],([t['name'],t['description']] for t in main_tools))
extra+=['','## Design MCP surface','','Outer per-call limits differ from child-process limits and whole-worker budgets.','']
extra+=table(['Tool','Floor','Vision required','Outer seconds','Declared contract'],([t['name'],t['policy']['min_tier'],t['policy']['requires_vision'],t['outer_timeout_ms']//1000,t['description']] for t in design_tools))
extra+=['','## OpenPencil lazy operations','','These are 87 operations behind four wrappers, not 87 top-level MCP tools. Schemas are retained in inventory.json.','']
extra+=table(['Operation','Effect','Purpose'],([t['name'],t['effect'],t['description']] for t in pencil['tools']))
outputs['inventory.md']+='\n'.join(extra)+'\n'

methods=['# Methodologies and professional contracts','']+intro
methods+=['Role fields below reproduce the authored registry contracts. Skill section headings index the current local bodies. Neither establishes that a host follows the method or that it improves outcomes. External bodies were not fetched; their provenance, adaptations, dependencies and caveats remain in inventory.json.','','## Role contracts','']
for r in capabilities['capabilities']:
    methods += [f"### {r['id']} — {r['title']}",'',f"Lead: `{r['lead']}`. Optional support: {cell(r['support'])}.",'',
        '**Deliverable:** '+r['deliverable'],'','**Prerequisites:** '+r['prerequisites'],'','**Acceptance:**','']+['- '+x for x in r['acceptance']]
    methods += ['','**Failure probe:** '+r['failure_probe'],'']
    for name,value in r['professional'].items():
        methods += ['**'+name.replace('_',' ').capitalize()+':**','']
        methods += ['- '+x for x in value] if isinstance(value,list) else [value]
        methods += ['']
methods += ['## Embedded skill methods index','']
for b in bodies:
    methods += ['### '+b['id'],'',b['description'],'',link(b['path'],'Current body'),'', 'Method sections: '+cell(b['headings']),'']
methods += ['## Workflow rules','']
for w in capabilities['workflows']:
    methods += ['### '+w['id'],'','**When:** '+w['when'],'','**Rule:** '+w['rule'],'','**Alternative roles:** '+cell(w.get('alternative_roles',[])),'']
outputs['methodologies.md']='\n'.join(methods)+'\n'

d=inventory['dependencies']; depdoc=['# Runtime and environmental dependencies','']+intro
depdoc += ['## Coverage limits','',d['graph_limits'], '', 'Locked records are not installed process counts or measured image sizes. No package manager, provider or scanner is run by this extractor.', '', '## Direct Rust requirements','']
depdoc += table(['Scope','Package','Requirement / features'], ([scope,k,v] for scope in ['dependencies','build-dependencies','dev-dependencies'] for k,v in cargo.get(scope,{}).items()))
depdoc += ['','## Resolved Rust packages','']+table(['Name','Version','Locked dependencies'],([p['name'],p['version'],p.get('dependencies',[])] for p in lock['package']))
depdoc += ['','## Direct design JavaScript requirements','']+table(['Name','Exact version'], d['node_manifest']['dependencies'].items())
depdoc += ['','Overrides: '+cell(d['node_manifest'].get('overrides',{})), '', '## Resolved npm lock paths','','Root, nested duplicates and optional platform records are retained separately.','']
depdoc += table(['Lock path','Actual name','Version','Optional','Platform'],([p or '<root>',v.get('name',p.split('node_modules/')[-1]),v.get('version',''),v.get('optional',False),{'os':v.get('os',[]),'cpu':v.get('cpu',[])}] for p,v in d['node_locked_packages'].items()))
depdoc += ['','## Direct Python requirements','','```text',*d['python_requirements'],'```','','## Locked Python wheels','']
depdoc += table(['Name','Version','Python / ABI','Platform'],([p['name'],p['version'],p['python_tag']+' / '+p['abi_tag'],p['platform_tag']] for p in python_wheels))
depdoc += ['','## Pinned base and service images','']
depdoc += table(['Consumer','Alias','Pin'], [['design',k,v] for k,v in inventory['packages']['design']['images'].items()]+[['agent-runtime','python',inventory['packages']['agent-runtime']['python_image']]])
for kind in ['embedding','bm25']:
    model=d[kind]
    depdoc += ['',f"### {kind}: {model['model']}",'',f"Revision `{model['revision']}`; {len(model['files'])} files; {sum(f['bytes'] for f in model['files'].values()):,} pinned bytes. This is asset size, not RAM or compressed transfer size.",'']
    depdoc += table(['File','Bytes','SHA-256'],([name,v['bytes'],v['sha256']] for name,v in model['files'].items()))
depdoc += ['','## Catalog-declared environmental prerequisites','','Declarations do not establish installed executables, SDKs, credentials, accounts or assets. Full caveats, companions and adaptations remain in inventory.json.','']
depdoc += table(['Skill','Delivery','Dependencies','Companion requirements'],([s['id'],s['delivery'],s.get('dependencies',[]),s.get('companion_requirements',[])] for s in catalog['skills'] if s.get('dependencies') or s.get('companion_requirements')))
depdoc += ['','## Environment name references','',inventory['environment_limit'],'']
depdoc += table(['Name','Owners'],([name,[link(p) for p in paths]] for name,paths in inventory['environment_names'].items()))
outputs['dependencies.md']='\n'.join(depdoc)+'\n'
outputs['inventory.json']=json.dumps(inventory,indent=2,ensure_ascii=False)+'\n'
stale=[]
for name,contents in outputs.items():
    path=OUT/name
    if args.check:
        if not path.exists() or path.read_text()!=contents: stale.append(name)
    else:
        path.write_text(contents)
print(json.dumps({'counts':inventory['counts'],'source_sha256':inventory['source_sha256'],'stale_outputs':stale,'mode':'check' if args.check else 'write'},indent=2))
raise SystemExit(1 if stale else 0)
