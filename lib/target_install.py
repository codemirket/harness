"""Install portable capabilities through explicit client adapters, without launching clients."""
import argparse
import json
import os
from pathlib import Path
import re
import sys
import tempfile

from . import catalog, configuration, harness, settings, targets


def mcp_sources():
    source = harness.ROOT / harness.manifest().get('mcp_source', 'registry/mcp.json')
    data = harness.read_json(source)
    if data.get('schema_version') != 1 or not isinstance(data.get('servers'), dict):
        raise ValueError('Invalid MCP source registry')
    result = {}
    for name, value in data['servers'].items():
        if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', name) or not isinstance(value, dict):
            raise ValueError('Invalid MCP source identifier or declaration')
        if set(value) - {'transport', 'url', 'command', 'args', 'env', 'description', 'enabled'}:
            raise ValueError('Unsupported MCP source fields: ' + name)
        if type(value.get('enabled', True)) is not bool:
            raise ValueError('MCP enabled must be a boolean: ' + name)
        transport = value.get('transport')
        if transport == 'http':
            if not isinstance(value.get('url'), str) or not value['url'].startswith(('https://', 'http://localhost:', 'http://127.0.0.1:')):
                raise ValueError('MCP HTTP source requires HTTPS or loopback URL: ' + name)
            if any(key in value for key in ('command', 'args', 'env')):
                raise ValueError('MCP source mixes transports: ' + name)
        elif transport == 'stdio':
            if not isinstance(value.get('command'), str) or not value['command'] or 'url' in value:
                raise ValueError('MCP stdio source requires one command: ' + name)
            if not isinstance(value.get('args', []), list) or any(not isinstance(v, str) for v in value.get('args', [])):
                raise ValueError('MCP args must be strings: ' + name)
            if not isinstance(value.get('env', {}), dict) or any(not isinstance(k, str) or not isinstance(v, str) for k, v in value.get('env', {}).items()):
                raise ValueError('MCP env must contain string values: ' + name)
        else:
            raise ValueError('Unsupported MCP transport: ' + name)
        if value.get('enabled', True):
            result[name] = value
    return result


def mcp_fields(source):
    """Render transport fields shared by desktop and account handoffs."""
    fields = ('url',) if source['transport'] == 'http' else ('command', 'args', 'env')
    value = {key: source[key] for key in fields if key in source}
    if source['transport'] == 'stdio':
        value.setdefault('args', [])
    return value


def mcp_config(target, servers):
    target = targets.normalize(target)
    result = {}
    for name, source in servers.items():
        value = mcp_fields(source)
        if target == 'claude':
            value = dict(type=source['transport'], **value)
        result[name] = value
    return result


def scalar(path, value):
    if not path or any(not isinstance(part, str) or not part for part in path):
        raise ValueError('Invalid configuration path')
    if type(value) not in (str, bool, int, float, list):
        raise ValueError('Unsupported configuration value at ' + '.'.join(path))
    if isinstance(value, list) and any(type(v) not in (str, bool, int, float) for v in value):
        raise ValueError('Unsupported TOML list at ' + '.'.join(path))


def configs_for(target, home, servers):
    """Register integrations without taking ownership of host preferences."""
    adapter = targets.global_adapter(target, home)
    key = 'mcp_servers' if target == 'codex' else 'mcpServers'
    yield adapter['mcp_destination'], adapter['mcp_format'], {key: mcp_config(target, servers)}


def check_transports(actual, desired, root):
    existing = actual.get(root, {})
    if not isinstance(existing, dict):
        raise ValueError('Existing MCP registry is not an object')
    for name, expected in desired.get(root, {}).items():
        current = existing.get(name, {})
        if not isinstance(current, dict):
            raise ValueError('Existing MCP server is not an object: ' + name)
        forbidden = ('command', 'args', 'env') if 'url' in expected else ('url', 'headers', 'http_headers', 'bearer_token_env_var')
        if any(key in current for key in forbidden) or ('type' in current and current['type'] != expected.get('type', current['type'])):
            raise ValueError('MCP transport conflict; reconcile existing server before installing: ' + name)
        # Never retarget an existing credential-bearing server. Even a public
        # endpoint change can leak preserved headers/env to the new process/host.
        for key in ('url', 'command', 'args'):
            if key in current and key in expected and current[key] != expected[key]:
                raise ValueError('MCP identity conflict; reconcile existing server before installing: ' + name)


def config_plan(home, relative, format_, desired):
    path = harness.guarded_path(home, relative)
    if catalog.is_link(path) or (path.exists() and not path.is_file()):
        raise ValueError('Configuration must be a regular file: ' + str(path))
    before = path.read_bytes() if path.exists() else None
    text = before.decode('utf-8') if before is not None else ''
    if format_ == 'toml':
        parsed, tables = settings.parse_document(text)
        actual = {}
        # Only inspect selected MCP fields, never print unrelated configuration.
        for key, value in parsed.items():
            if len(key) >= 3 and key[0] == 'mcp_servers' and key[1] in desired.get('mcp_servers', {}):
                # Preserve arbitrary valid sibling values (for example TOML
                # inline http_headers). Only identity values need parsing.
                parsed_value = settings.parse_value(value[2]) if len(key) == 3 and key[2] in ('url', 'command', 'args', 'type') else None
                settings.set_nested(actual, key, parsed_value)
        for key in tables:
            if len(key) == 3 and key[0] == 'mcp_servers' and key[1] in desired.get('mcp_servers', {}):
                actual.setdefault(key[0], {}).setdefault(key[1], {}).setdefault(key[2], {})
        check_transports(actual, desired, 'mcp_servers')
        merged, changes = settings.merge_config(text, dict(settings.flatten(desired)), validator=scalar)
        keys = [change['key'] for change in changes]
    else:
        if format_ == 'json' and text:
            try:
                json.loads(text)
            except ValueError:
                raise ValueError('Claude configuration requires strict JSON: ' + str(path)) from None
        actual = configuration.parse_json(text) if text.strip() else {}
        check_transports(actual, desired, 'mcpServers')
        merged, keys = configuration.merge_json(text, desired)
    return {'path': path, 'relative': relative, 'before': before,
            'content': merged.encode('utf-8'), 'keys': keys, 'changed': before != merged.encode('utf-8')}


def prepare(target='all', home=None, mode='auto'):
    selected = targets.names(target)
    targets.validate_home_environment(home, selected)
    base, _, _, jobs = harness.global_plan(target, home, mode)
    sources = mcp_sources()
    configs = []
    for name in selected:
        for relative, format_, desired in configs_for(name, base, sources):
            configs.append(config_plan(base, relative, format_, desired))
    public = {'targets': selected, 'status': 'planned', 'ready': False,
              'guidance': harness.public_jobs(jobs), 'guidance_completed': False,
              'configuration': [{'path': str(job['path']), 'changed': job['changed'], 'keys': job['keys']} for job in configs],
              'mcp_servers': list(sources), 'blockers': [],
              'limits': ['Checks verify files and declared values, not client discovery, model access or MCP connectivity.',
                         'Host model, approval, sandbox and appearance preferences remain unchanged. Start a fresh client session to load new guidance and integrations.',
                         'Removed selections are preserved. Review obsolete installed skills and MCP entries deliberately.']}
    public['changes_pending'] = any(job['action'] != 'unchanged' for job in jobs) or any(job['changed'] for job in configs)
    public['ready'] = not public['changes_pending'] and not public['blockers']
    return base, configs, public


def write_config(base, job):
    path = harness.guarded_path(base, job['relative'])
    def unchanged():
        current = harness.guarded_path(base, job['relative'])
        if catalog.is_link(current) or (current.read_bytes() if current.exists() else None) != job['before']:
            raise ValueError('Configuration changed after preflight: ' + str(current))
    unchanged()
    if not job['changed']:
        return None
    path.parent.mkdir(parents=True, exist_ok=True)
    backup = None
    if job['before'] is not None:
        with tempfile.NamedTemporaryFile(prefix=path.name + '.harness-backup-', dir=path.parent, delete=False) as stream:
            os.chmod(stream.name, 0o600)
            stream.write(job['before'])
            backup = stream.name
    # Validate parents and bytes after backup and after staging, before publication.
    settings._atomic_write(path, job['content'], 0o600, validate=unchanged)
    return backup


def run(command='install', target='all', home=None, mode='auto', dry_run=False):
    base, configs, report = prepare(target, home, mode)
    if command == 'check' or dry_run or report['blockers']:
        return report
    report['backups'] = []
    try:
        # Revalidate every config before guidance writes; mutations recheck again.
        for job in configs:
            path = harness.guarded_path(base, job['relative'])
            if catalog.is_link(path) or (path.read_bytes() if path.exists() else None) != job['before']:
                raise ValueError('Configuration changed after preflight: ' + str(path))
        report['guidance'] = harness.sync_global(target, base, mode)
        report['guidance_completed'] = True
        for job in configs:
            backup = write_config(base, job)
            if backup:
                report['backups'].append(backup)
        _, _, verified = prepare(target, base, mode)
    except (ValueError, OSError) as error:
        report.update(status='partially_applied', ready=False)
        report['blockers'].append(str(error))
        return report
    report.update(status='installed', ready=verified['ready'], changes_pending=verified['changes_pending'])
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['install', 'check'])
    parser.add_argument('--target', choices=targets.CHOICES, default=harness.manifest().get('default_target', 'all'))
    parser.add_argument('--home', type=Path, help='Existing isolated home for rehearsal or portable preparation')
    parser.add_argument('--mode', choices=['auto', 'link', 'copy'], default='auto')
    parser.add_argument('--dry-run', action='store_true')
    args = vars(parser.parse_args(argv))
    report = run(**args)
    print(json.dumps(report, indent=2))
    return 0 if not report['blockers'] and (report['ready'] or args['dry_run']) else 1
