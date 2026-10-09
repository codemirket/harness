#!/usr/bin/env python3
"""Allowlisted Codex preferences: capture, plan, apply and doctor (Python 3.9+).

Only config.toml is written, with the desktop app closed. Global-state JSON,
credentials, sessions, permissions, marketplaces and plugin accounts are never
copied. TOML statements outside selected scalar/array leaves retain their bytes.
The parser deliberately rejects unsupported selected values rather than guessing.
"""
import argparse
import csv
import io
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import uuid

ROOT = Path(__file__).resolve().parents[1]
SETTINGS = ROOT / 'registry' / 'codex-settings.json'
SCHEMA_VERSION = 1
ROOT_KEYS = {'model', 'model_reasoning_effort', 'model_reasoning_summary',
             'model_verbosity', 'personality', 'service_tier',
             'plan_mode_reasoning_effort'}
DESKTOP_KEYS = {
    'appearanceTheme', 'appearanceLightChromeTheme', 'appearanceDarkChromeTheme',
    'appearanceLightCodeThemeId', 'appearanceDarkCodeThemeId',
    'appearanceDiffMarkerStyle', 'sansFontSize', 'codeFontSize',
    'useFontSmoothing', 'usePointerCursors', 'conversationDetailMode',
    'followUpQueueMode', 'composerEnterBehavior', 'composerPlainTextMode',
    'show-context-window-usage', 'show-educational-tips', 'reviewDelivery',
    'localeOverride', 'browser-show-full-url', 'enabled-reasoning-efforts',
}
MARKETPLACES = {'openai-bundled', 'openai-primary-runtime',
                'openai-curated-remote', 'personal-ai'}
EFFORTS = {'none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max',
           'ultra', 'persistent'}
SKIPPED = [
    'Authentication, tokens, account identifiers and connector settings',
    'Permissions, sandbox, approval policies, browser access and trusted projects',
    'Project paths, worktrees, remote devices, windows, history and recent models',
    'MCP commands, environment variables, executables and cache paths',
    'Private, organization-specific and local marketplace plugins',
    'Telemetry, experiments, feature gates, hooks and notification commands',
    'All desktop global-state and persisted-atom records',
    'OS-specific shell, Dock, power, hotkey and external-link preferences',
]
LIMITATIONS = [
    'Configured allowlisted values only; absent source settings do not delete target settings.',
    'Desktop appearance keys are observed in the installed app schema, not a stable public file API.',
    'Close ChatGPT/Codex desktop before applying changed settings; reopen afterward.',
    'Model, reasoning levels, themes, fonts and plugins may differ by device, client or account.',
    'Plugin enabled flags do not install dependencies or connect accounts. Codex marketplace refresh may fetch configured plugins, even disabled ones.',
    'No plugin permissions, credentials, generic plugin settings or private marketplace locations are synchronized.',
    'Structural file checks do not establish that the UI rendered a theme or a model/plugin is available.',
]
SOURCES = [
    'https://learn.chatgpt.com/docs/config-file/config-reference',
    'https://learn.chatgpt.com/docs/reference/settings',
    'https://developers.openai.com/plugins/build/plugins',
]


def codex_home(home=None):
    """home is a user-home override; honor CODEX_HOME only for the real default."""
    if home is None and os.environ.get('CODEX_HOME'):
        return Path(os.environ['CODEX_HOME']).expanduser()
    return (Path(home).expanduser() if home is not None else Path.home()).resolve() / '.codex'


def is_link(path):
    try:
        info = path.lstat()
    except FileNotFoundError:
        return False
    return stat.S_ISLNK(info.st_mode) or bool(
        getattr(info, 'st_file_attributes', 0) & 0x400)


def check_path(path):
    for item in (path, *path.parents):
        if is_link(item):
            raise ValueError('Refusing a symlink or junction in settings destination')


def read_config(path):
    check_path(path)
    if not path.exists():
        return ''
    if path.stat().st_size > 4 * 1024 * 1024:
        raise ValueError('Configuration exceeds the supported size limit')
    with path.open(encoding='utf-8', newline='') as stream:
        return stream.read()


def statements(text):
    """Yield complete TOML statements without interpreting unselected values."""
    start = 0
    quote = None
    depth = 0
    comment = False
    i = 0
    while i < len(text):
        ch = text[i]
        if comment:
            if ch == '\n':
                comment = False
            else:
                i += 1
                continue
        elif quote:
            if quote[0] == '"' and ch == '\\':
                i += 2
                continue
            if text.startswith(quote, i):
                i += len(quote)
                quote = None
                continue
            i += 1
            continue
        elif ch == '#':
            comment = True
        elif ch in '\'"':
            quote = ch * 3 if text.startswith(ch * 3, i) else ch
            i += len(quote)
            continue
        elif ch in '[{':
            depth += 1
        elif ch in ']}':
            depth -= 1
            if depth < 0:
                raise ValueError('Unbalanced configuration statement')
        if ch == '\n' and depth == 0 and quote is None:
            yield start, i + 1, text[start:i + 1]
            start = i + 1
        i += 1
    if quote or depth:
        raise ValueError('Unterminated configuration statement')
    if start < len(text):
        yield start, len(text), text[start:]


def key_parts(raw):
    parts = []
    pos = 0
    while pos < len(raw):
        m = re.match(r'\s*("(?:[^"\\]|\\.)*"|\x27[^\x27]*\x27|[A-Za-z0-9_-]+)\s*', raw[pos:])
        if not m:
            raise ValueError('Unsupported configuration key syntax')
        value = m[1]
        parts.append(json.loads(value) if value.startswith('"') else
                     value[1:-1] if value.startswith("'") else value)
        pos += m.end()
        if pos < len(raw):
            if raw[pos] != '.':
                raise ValueError('Unsupported configuration key syntax')
            pos += 1
    if not parts or raw.rstrip().endswith('.'):
        raise ValueError('Invalid configuration key')
    return tuple(parts)


def without_comments(raw):
    # Selected values are simple strings, booleans, numbers or string arrays.
    # Keep hashes inside strings; do not interpret comments as preferences.
    result = []
    quote = None
    escape = False
    comment = False
    for ch in raw:
        if comment:
            if ch == '\n':
                result.append(ch)
                comment = False
            continue
        if quote:
            result.append(ch)
            if escape:
                escape = False
            elif ch == '\\' and quote == '"':
                escape = True
            elif ch == quote:
                quote = None
        elif ch in '\'"':
            quote = ch
            result.append(ch)
        elif ch == '#':
            comment = True
        else:
            result.append(ch)
    return ''.join(result).strip()


def parse_value(raw):
    value = without_comments(raw)
    if value.startswith(('"""', "'''", '{')):
        raise ValueError('Selected setting uses unsupported multiline or inline-table syntax')
    if value.startswith("'") and value.endswith("'"):
        return value[1:-1]
    try:
        return json.loads(value)
    except (ValueError, TypeError):
        # TOML permits a trailing array comma. Do not strip commas in strings.
        if value.startswith('[') and re.search(r',\s*\]$', value):
            try:
                return json.loads(re.sub(r',\s*\]$', ']', value))
            except ValueError:
                pass
        raise ValueError('Selected setting uses unsupported TOML value syntax') from None


def parse_document(text):
    try:
        import tomllib  # Python 3.11+: additionally validate the complete document.
    except ImportError:
        tomllib = None
    if tomllib is not None:
        try:
            tomllib.loads(text)
        except ValueError:
            raise ValueError('Invalid TOML configuration; no settings changed') from None
    assignments = {}
    tables = {(): [0, len(text)]}
    section = ()
    array_index = 0
    array_context = {}
    for start, end, raw in statements(text):
        stripped = raw.strip()
        if not stripped or stripped.startswith('#'):
            continue
        if stripped.startswith('['):
            header = without_comments(stripped)
            array = header.startswith('[[')
            if not header.endswith(']]' if array else ']'):
                raise ValueError('Invalid TOML table header')
            path = key_parts(header[2:-2] if array else header[1:-1])
            if section in tables:
                tables[section][1] = start
            # An array table is opaque. Its children must never masquerade as
            # a regular allowlisted table; the sentinel cannot be a setting key.
            if array:
                array_index += 1
                # Starting a new array element also changes the context of its
                # child tables; no descendants from the previous element carry over.
                array_context = {key: value for key, value in array_context.items()
                                 if key[:len(path)] != path}
                array_context[path] = str(array_index)
            parents = [key for key in array_context if path[:len(key)] == key]
            if parents:
                owner = max(parents, key=len)
                section = ('@array', array_context[owner]) + path
            else:
                section = path
            if section in tables and section[:1] != ('@array',):
                raise ValueError('Repeated TOML table; no settings changed')
            tables[section] = [end, len(text)]
            continue
        # Match the assignment's key, not '=' inside the value or a string key.
        m = re.match(r'\s*((?:"(?:[^"\\]|\\.)*"|\x27[^\x27]*\x27|[A-Za-z0-9_. -])+?)\s*=\s*', raw)
        if not m:
            raise ValueError('Unsupported configuration statement')
        path = section + key_parts(m[1])
        if path in assignments:
            raise ValueError('Repeated TOML key; no settings changed')
        value_start = start + m.end()
        # Preserve a trailing inline comment verbatim for single-line values.
        tail = raw[m.end():]
        clean = without_comments(tail)
        value_end = value_start + len(tail.rstrip('\r\n'))
        if '\n' not in tail.rstrip('\r\n'):
            quote = None
            escaped = False
            for n, ch in enumerate(tail):
                if quote:
                    if escaped:
                        escaped = False
                    elif ch == '\\' and quote == '"':
                        escaped = True
                    elif ch == quote:
                        quote = None
                elif ch in '\'"':
                    quote = ch
                elif ch == '#':
                    value_end = value_start + len(tail[:n].rstrip())
                    break
        assignments[path] = (value_start, value_end, tail, clean)
    return assignments, tables


def portable_plugin(identifier):
    if not isinstance(identifier, str) or not re.fullmatch(r'[a-z0-9][a-z0-9_.-]*@[a-z0-9][a-z0-9_.-]*', identifier):
        return False
    return identifier.rsplit('@', 1)[1] in MARKETPLACES


def allowed_path(path):
    if len(path) == 1:
        return path[0] in ROOT_KEYS
    if path[0] == 'plugins':
        return len(path) == 3 and portable_plugin(path[1]) and path[2] == 'enabled'
    if path[0] != 'desktop' or path[1] not in DESKTOP_KEYS:
        return False
    if path[1] in ('appearanceLightChromeTheme', 'appearanceDarkChromeTheme'):
        return (len(path) == 3 and path[2] in {'accent', 'accentSource', 'contrast', 'ink', 'opaqueWindows', 'surface'} or
                len(path) == 4 and path[2] == 'fonts' and path[3] in {'code', 'ui', 'content'} or
                len(path) == 4 and path[2] == 'semanticColors' and path[3] in {'diffAdded', 'diffRemoved', 'skill'})
    return len(path) == 2


def validate_value(path, value):
    if not allowed_path(path):
        raise ValueError('Settings manifest contains a non-allowlisted field')
    key = path[-1]
    enums = {
        'model_reasoning_effort': EFFORTS, 'plan_mode_reasoning_effort': EFFORTS,
        'model_reasoning_summary': {'auto', 'concise', 'detailed', 'none'},
        'model_verbosity': {'low', 'medium', 'high'},
        'personality': {'none', 'friendly', 'pragmatic'},
        'service_tier': {'flex', 'fast', 'priority', 'standard', 'ultrafast'},
        'appearanceTheme': {'system', 'light', 'dark'},
        'appearanceDiffMarkerStyle': {'color', 'symbols'},
        'conversationDetailMode': {'STEPS_PROSE', 'STEPS_COMMANDS', 'STEPS_EXECUTION'},
        'followUpQueueMode': {'queue', 'steer', 'interrupt'},
        'composerEnterBehavior': {'enter', 'cmdIfMultiline', 'cmdAlways'},
        'reviewDelivery': {'inline', 'detached'},
        'accentSource': {'chatgpt', 'custom'},
    }
    booleans = {'enabled', 'opaqueWindows', 'useFontSmoothing', 'usePointerCursors',
                'composerPlainTextMode', 'show-context-window-usage',
                'show-educational-tips', 'browser-show-full-url'}
    if key in enums:
        valid = isinstance(value, str) and value in enums[key]
    elif key in booleans:
        valid = type(value) is bool
    elif key in ('sansFontSize', 'codeFontSize', 'contrast'):
        limits = {'sansFontSize': (11, 16), 'codeFontSize': (8, 24), 'contrast': (0, 100)}[key]
        valid = type(value) in (int, float) and limits[0] <= value <= limits[1]
        if key == 'contrast':
            valid = valid and type(value) is int
    elif key == 'enabled-reasoning-efforts':
        valid = isinstance(value, list) and bool(value) and all(isinstance(v, str) and v in EFFORTS for v in value) and len(set(value)) == len(value)
    elif key in {'accent', 'ink', 'surface', 'diffAdded', 'diffRemoved', 'skill'}:
        valid = isinstance(value, str) and bool(re.fullmatch(r'#[0-9A-Fa-f]{6}', value))
    elif key in ('model', 'appearanceLightCodeThemeId', 'appearanceDarkCodeThemeId'):
        valid = isinstance(value, str) and bool(re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}', value))
    elif key == 'localeOverride':
        valid = isinstance(value, str) and bool(re.fullmatch(r'[A-Za-z]{2,8}(?:-[A-Za-z0-9]{2,8})*', value))
    else:  # Portable font family lists, never font file paths or CSS URLs.
        valid = isinstance(value, str) and 0 < len(value) <= 256 and not any(ch in value for ch in '/\\\n\r;{}') and 'url(' not in value.lower()
    if not valid:
        raise ValueError('Invalid portable setting: ' + '.'.join(path))


def flatten(value, prefix=()):
    if not isinstance(value, dict):
        yield prefix, value
        return
    for key, nested in value.items():
        if not isinstance(key, str):
            raise ValueError('Settings keys must be strings')
        yield from flatten(nested, prefix + (key,))


def set_nested(target, path, value):
    for key in path[:-1]:
        target = target.setdefault(key, {})
    target[path[-1]] = value


def manifest_values(data):
    if data.get('schema_version') != SCHEMA_VERSION:
        raise ValueError('Unsupported settings schema')
    if set(data) - {'schema_version', 'config', 'desktop', 'plugins', 'provenance', 'skipped_categories', 'limitations', 'capture_summary'}:
        raise ValueError('Unknown settings manifest section')
    result = {}
    for section in ('config', 'desktop', 'plugins'):
        value = data.get(section, {})
        if not isinstance(value, dict):
            raise ValueError('Settings sections must be objects')
        for path, item in flatten(value, () if section == 'config' else (section,)):
            validate_value(path, item)
            result[path] = item
    return result


def capture(home=None, output=None):
    """Read only allowlisted config values; output is an explicit export path."""
    text = read_config(codex_home(home) / 'config.toml')
    assignments, _ = parse_document(text)
    data = {'schema_version': SCHEMA_VERSION, 'config': {}, 'desktop': {}, 'plugins': {},
            'provenance': {'configuration': 'Codex user config.toml allowlist',
                           'official_sources': SOURCES,
                           'desktop_contract': 'Observed installed ChatGPT/Codex desktop setting keys; version-dependent',
                           'reviewed_desktop_version': '26.930.61225',
                           'reviewed_desktop_build': '13232',
                           'reviewed_desktop_module': '.vite/build/src-C1dW0Du8.js',
                           'reviewed_on': '2026-10-06'},
            'skipped_categories': list(SKIPPED), 'limitations': list(LIMITATIONS)}
    skipped_plugins = set()
    for path, (_, _, raw, _) in assignments.items():
        if path[:1] == ('plugins',) and len(path) > 1 and not portable_plugin(path[1]):
            skipped_plugins.add(path[1])
        if not allowed_path(path):
            continue
        value = parse_value(raw)
        validate_value(path, value)
        if path[0] in ('desktop', 'plugins'):
            set_nested(data, path, value)
        else:
            data['config'][path[0]] = value
    data['capture_summary'] = {'portable_values': len(manifest_values(data)),
                               'portable_plugins': len(data['plugins']),
                               'private_or_local_plugins_skipped': len(skipped_plugins)}
    if output is not None:
        destination = Path(output)
        if destination.resolve() == (codex_home(home) / 'config.toml').resolve():
            raise ValueError('Capture output must not replace the active Codex configuration')
        check_path(destination)
        destination.parent.mkdir(parents=True, exist_ok=True)
        _atomic_write(destination, (json.dumps(data, indent=2) + '\n').encode(), 0o600)
    return data


def load_settings(path=None):
    data = json.loads(Path(path or SETTINGS).read_text(encoding='utf-8'))
    manifest_values(data)
    return data


def render_key(path):
    return '.'.join(k if re.fullmatch(r'[A-Za-z0-9_-]+', k) else json.dumps(k) for k in path)


def merge_config(text, desired, validator=validate_value):
    assignments, tables = parse_document(text)
    replacements = []
    additions = {}
    changes = []
    for path, value in desired.items():
        validator(path, value)
        # Table declarations are values too: a scalar cannot replace a table,
        # and a setting under an array of tables is not a singleton preference.
        # Check before the same-value fast path to reject deceptive no-ops.
        for table in tables:
            array_table = table[:1] == ('@array',)
            declared = table[2:] if array_table else table
            if not declared:
                continue
            if (declared[:len(path)] == path or
                    array_table and path[:len(declared)] == declared):
                raise ValueError('Selected setting conflicts with an existing table or array table; edit manually')
        present = assignments.get(path)
        for existing in assignments:
            if existing != path and (existing == path[:len(existing)] or path == existing[:len(path)]):
                raise ValueError('Selected setting overlaps an existing structured value; edit manually')
        if present:
            current = parse_value(present[2])
            if current == value and type(current) is type(value):
                continue
            replacements.append((present[0], present[1], json.dumps(value, ensure_ascii=False)))
        else:
            current = None
            # Insert dotted keys under the nearest existing table. Creating a
            # new [table] can illegally redefine parents already established by
            # dotted keys, especially on Python 3.9 without a full TOML parser.
            table = max((t for t in tables if len(t) < len(path) and path[:len(t)] == t), key=len)
            additions.setdefault(table, []).append((path[len(table):], value))
        if present:
            try:
                validator(path, current)
            except ValueError:
                current = '<unrecognized configured value>'
        changes.append({'key': render_key(path), 'before': current, 'after': value})
    for table, values in additions.items():
        lines = ''.join(render_key(key) + ' = ' + json.dumps(value, ensure_ascii=False) + '\n' for key, value in values)
        position = tables[table][1]
        prefix = '\n' if position > 0 and text[position - 1] != '\n' else ''
        replacements.append((position, position, prefix + lines))
    insertions = {}
    edits = []
    for start, end, value in replacements:
        if start == end:
            insertions.setdefault(start, []).append(value)
        else:
            edits.append((start, end, value))
    edits.extend((position, position, ''.join(values)) for position, values in insertions.items())
    for start, end, value in sorted(edits, key=lambda edit: (edit[0], edit[1]), reverse=True):
        text = text[:start] + value + text[end:]
    actual, _ = parse_document(text)
    for path, expected in desired.items():
        if path not in actual or parse_value(actual[path][2]) != expected:
            raise ValueError('Post-merge preference verification failed')
    return text, changes


def app_running():
    """Conservative process-name check; None means status cannot be established."""
    try:
        if os.name == 'nt':
            result = subprocess.run(['tasklist', '/FO', 'CSV', '/NH'], capture_output=True, text=True, timeout=10, check=True)
            names = [row[0].lower() for row in csv.reader(io.StringIO(result.stdout)) if row]
        else:
            result = subprocess.run(['ps', '-A', '-o', 'comm='], capture_output=True, text=True, timeout=10, check=True)
            names = [Path(row.strip()).name.lower() for row in result.stdout.splitlines()]
        return any(name in {'chatgpt', 'chatgpt.exe', 'codex', 'codex.exe'} or
                   name.startswith(('chatgpt helper', 'codex helper')) for name in names)
    except (OSError, subprocess.SubprocessError):
        return None


def availability(data, home=None):
    warnings = []
    cache = codex_home(home) / 'plugins' / 'cache'
    for identifier in data.get('plugins', {}):
        name, marketplace = identifier.split('@')
        # Existence is a discovery hint, not an account or runtime health check.
        candidate = cache / marketplace / name
        if not candidate.is_dir() or is_link(candidate):
            warnings.append('Plugin availability not confirmed on this device: ' + identifier)
    if data.get('config', {}).get('model'):
        warnings.append('Model and reasoning availability must be checked in this device/account; preferences do not grant access.')
    if any('ChromeTheme' in key for key in data.get('desktop', {})):
        warnings.append('Custom font availability and rendered theme appearance are not verified by configuration checks.')
    return warnings


def plan(home=None, settings_path=None):
    data = load_settings(settings_path)
    destination = codex_home(home) / 'config.toml'
    text = read_config(destination)
    _, changes = merge_config(text, manifest_values(data))
    running = app_running() if changes else None
    return {'status': 'unchanged' if not changes else 'pending', 'changes': changes,
            'changed': bool(changes), 'ready_to_apply': not changes or running is False,
            'requires_app_closed': bool(changes), 'applied': False, 'config_path': str(destination),
            'destination': str(destination), 'app_must_close': bool(changes) and running is not False,
            'app_status': 'not_checked_no_changes' if not changes else 'running' if running else 'closed' if running is False else 'unknown',
            'pending_restart': bool(changes), 'warnings': availability(data, home),
            'skipped_categories': list(SKIPPED),
            'instruction': 'Quit ChatGPT/Codex desktop and active Codex clients, rerun settings apply, then reopen the app.' if changes else 'Portable configured preferences already match.'}


def _atomic_write(path, content, mode):
    with tempfile.NamedTemporaryFile(prefix='.personal-ai-settings-', dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            os.chmod(temporary, mode)
            stream.write(content)
            stream.flush()
            os.fsync(stream.fileno())
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def apply(home=None, settings_path=None):
    result = plan(home, settings_path)
    if not result['changes']:
        return result
    if result['app_must_close']:
        result['status'] = 'deferred_app_must_close'
        return result
    data = load_settings(settings_path)
    destination = codex_home(home) / 'config.toml'
    original = read_config(destination)
    merged, changes = merge_config(original, manifest_values(data))
    if not changes:
        return plan(home, settings_path)
    destination.parent.mkdir(parents=True, exist_ok=True)
    lock = destination.parent / '.personal-ai-settings.lock'
    check_path(lock)
    try:
        descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    except FileExistsError:
        raise ValueError('Another preference update holds the lock; inspect it before retrying') from None
    backup = None
    try:
        os.close(descriptor)
        check_path(destination)
        if app_running() is not False:
            result['status'] = 'deferred_app_must_close'
            result['app_must_close'] = True
            result['ready_to_apply'] = False
            return result
        if read_config(destination) != original:
            raise ValueError('Configuration changed during preparation; retry without overwriting it')
        mode = stat.S_IMODE(destination.stat().st_mode) if destination.exists() else 0o600
        if destination.exists():
            backup = destination.parent / 'backups' / ('settings-' + uuid.uuid4().hex) / 'config.toml'
            check_path(backup)
            backup.parent.mkdir(parents=True, mode=0o700)
            backup.write_bytes(original.encode('utf-8'))
            backup.chmod(0o600)
        if read_config(destination) != original or app_running() is not False:
            raise ValueError('Configuration or app state changed; no replacement performed')
        _atomic_write(destination, merged.encode('utf-8'), mode)
        result.update(status='applied', applied=True, changes=changes, backup=str(backup) if backup else None,
                      app_must_close=False, pending_restart=True,
                      instruction='Reopen ChatGPT/Codex. Check model/plugin availability and rendered appearance on this device.')
        return result
    finally:
        lock.unlink(missing_ok=True)


def doctor(home=None, settings_path=None):
    result = plan(home, settings_path)
    result['healthy'] = not result['changes']
    result['verification_scope'] = 'Configured portable values only; no runtime, account, theme-rendering or model-access validation.'
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['capture', 'plan', 'apply', 'doctor'])
    parser.add_argument('--home', type=Path, help='User-home override, primarily for isolated verification')
    parser.add_argument('--settings', type=Path, help='Shared preference manifest')
    parser.add_argument('--output', type=Path, help='Explicit capture export path; otherwise print only allowlisted values')
    args = parser.parse_args(argv)
    try:
        if args.command == 'capture':
            result = capture(args.home, args.output)
        else:
            result = globals()[args.command](args.home, args.settings)
        print(json.dumps(result, indent=2))
        if args.command == 'doctor' and not result.get('healthy', False):
            return 1
        if args.command in ('plan', 'apply') and not result.get('ready_to_apply', False):
            return 2
        return 0
    except (ValueError, OSError) as error:
        # ValueErrors never include unselected configuration content.
        print('Settings error: ' + str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
