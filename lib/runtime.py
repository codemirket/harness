"""Read-only primary runtime discovery; no installation, login or model inference.

Official contracts reviewed 2026-10-06:
https://learn.chatgpt.com/docs/codex/cli
https://learn.chatgpt.com/docs/windows/windows-app
https://learn.chatgpt.com/docs/linux/linux-app
https://code.claude.com/docs/en/setup
https://code.claude.com/docs/en/cli-reference
"""
import argparse
import json
import os
from pathlib import Path
import plistlib
import re
import shutil
import signal
import subprocess
import sys
import threading

MAX_OUTPUT = 64 * 1024
MAX_METADATA = 1024 * 1024
VERSION = r'(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)'
MAC_IDENTITIES = {'com.openai.codex'}
BUNDLED_CODEX = ('Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex',
                 'Contents/Resources/codex-cli/bin/codex', 'Contents/Resources/codex')
WINDOWS_PACKAGES = ("$ErrorActionPreference='Stop'; "
    "ConvertTo-Json -Compress -InputObject @(Get-AppxPackage | Where-Object { $_.Name -match '^OpenAI[.].*(Codex|ChatGPT)' } | "
    "Select-Object -First 10 Name,Version,InstallLocation)")
WINDOWS_VERSION = ("$ErrorActionPreference='Stop'; "
    "$v=(Get-Item -LiteralPath $env:AI_RUNTIME_DESKTOP_PATH).VersionInfo; "
    "@{ProductName=$v.ProductName;CompanyName=$v.CompanyName;FileVersion=$v.FileVersion} | ConvertTo-Json -Compress")


def platform_name(value=None):
    value = value or sys.platform
    return 'windows' if value in ('win32', 'windows') else 'macos' if value in ('darwin', 'macos') else 'linux' if value.startswith('linux') else value


def run_probe(argv, *, timeout=5, env=None):
    """No shell or stdin; cap captured output and kill on deadline/overflow."""
    child_env = dict(os.environ if env is None else env)
    child_env.update({'CI': '1', 'NO_COLOR': '1', 'DISABLE_AUTOUPDATER': '1', 'DISABLE_UPDATES': '1'})
    options = {'stdin': subprocess.DEVNULL, 'stdout': subprocess.PIPE,
               'stderr': subprocess.STDOUT, 'env': child_env, 'shell': False}
    if os.name == 'nt':
        options['creationflags'] = subprocess.CREATE_NO_WINDOW
    else:
        options['start_new_session'] = True
    try:
        process = subprocess.Popen([str(arg) for arg in argv], **options)
    except OSError:
        return {'status': 'launch_failed', 'exit_code': None, 'text': ''}
    chunks = []
    overflow = threading.Event()

    def stop():
        try:
            if os.name == 'nt': process.kill()
            else: os.killpg(process.pid, signal.SIGKILL)
        except (OSError, ProcessLookupError):
            pass

    def read():
        total = 0
        try:
            while True:
                block = process.stdout.read(4096)
                if not block: break
                total += len(block)
                if total > MAX_OUTPUT:
                    overflow.set()
                    stop()
                    break
                chunks.append(block)
        except (OSError, ValueError):
            pass
        finally:
            process.stdout.close()

    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    status = 'ok'
    try:
        process.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        status = 'timeout'
        stop()
        try:
            process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            pass
    reader.join(timeout=1)
    if reader.is_alive():
        # A descendant retaining the pipe must not keep diagnostics blocked.
        stop()
        status = 'timeout'
    if overflow.is_set(): status = 'output_limit'
    if status == 'ok' and process.returncode != 0: status = 'nonzero'
    return {'status': status, 'exit_code': process.returncode,
            'text': b''.join(chunks).decode('utf-8', errors='replace')}


def read_metadata(path, decoder):
    try:
        with Path(path).open('rb') as stream:
            raw = stream.read(MAX_METADATA + 1)
        if len(raw) > MAX_METADATA: return None
        value = decoder(raw)
        return value if isinstance(value, dict) else None
    except (OSError, ValueError, TypeError, plistlib.InvalidFileException):
        return None


def path_lookup(name, env, platform):
    if platform != platform_name(): return None
    # Explicit PATH prevents shutil.which from supplying an OS default.
    return shutil.which(name, path=env.get('PATH', env.get('Path', '')))


def known_cli_paths(name, home, env, platform, desktop_path=None):
    suffix = '.exe' if platform == 'windows' else ''
    candidates = [home / '.local/bin' / (name + suffix)]
    if platform == 'windows':
        local = Path(env.get('LOCALAPPDATA', str(home / 'AppData/Local')))
        roaming = Path(env.get('APPDATA', str(home / 'AppData/Roaming')))
        candidates += [local / 'Microsoft/WinGet/Links' / (name + '.exe'),
                       roaming / 'npm' / (name + '.cmd')]
        if name == 'codex':
            candidates += [local / 'OpenAI/Codex/bin/codex.exe', local / 'OpenAI/Codex/bin/codex.cmd']
            if desktop_path:
                base = desktop_path.parent if desktop_path.suffix.lower() == '.exe' else desktop_path
                candidates += [base / 'resources/codex.exe', base / 'resources/codex-cli/bin/codex.exe',
                               base / 'app/resources/codex.exe']
        for key in ('ProgramFiles', 'ProgramFiles(x86)'):
            if env.get(key): candidates.append(Path(env[key]) / ('Claude' if name == 'claude' else 'OpenAI/Codex') / (name + '.exe'))
    else:
        candidates += [Path('/opt/homebrew/bin') / name, Path('/usr/local/bin') / name, Path('/usr/bin') / name]
        if name == 'codex' and platform == 'macos':
            apps = [desktop_path] if desktop_path else []
            apps += [Path('/Applications/ChatGPT.app'), Path('/Applications/Codex.app'),
                     home / 'Applications/ChatGPT.app', home / 'Applications/Codex.app']
            candidates += [app / relative for app in apps for relative in BUNDLED_CODEX]
    return candidates


def launch_command(path, name, env, platform):
    if platform == 'windows' and path.suffix.lower() in ('.cmd', '.bat', '.ps1'):
        relative = ('@openai/codex/bin/codex.js' if name == 'codex'
                    else '@anthropic-ai/claude-code/cli.js')
        script = path.parent / 'node_modules' / relative
        node = path_lookup('node', env, platform)
        if node and script.is_file(): return [node, str(script)]
        return None  # Never interpolate an arbitrary shim into cmd.exe/PowerShell.
    return [str(path)]


def auth_result(name, result):
    """Expose only documented status values, never raw CLI output/account fields."""
    status, code, text = result['status'], result['exit_code'], result['text']
    output = {'state': 'unknown', 'probe_status': status}
    if status not in ('ok', 'nonzero'): return output
    if name == 'claude':
        try: value = json.loads(text)
        except ValueError: return output
        if not isinstance(value, dict): return output
        method = value.get('authMethod')
        if method in ('none', 'claude.ai', 'oauth_token', 'api_key', 'api_key_helper', 'third_party'):
            output['method'] = method
        logged_in = value.get('loggedIn')
        if logged_in is True and code == 0: output['state'] = 'authenticated'
        elif logged_in is False and code in (0, 1): output['state'] = 'not_authenticated'
    else:
        if code == 0 and re.search(r'(?im)^logged in(?: using| with|$)', text):
            output['state'] = 'authenticated'
        elif code in (0, 1) and re.search(r'(?im)^not logged in\s*$', text):
            output['state'] = 'not_authenticated'
    return output


def diagnose_cli(name, *, home, env, platform, explicit=None, desktop_path=None,
                 check_auth=False, timeout=5):
    on_path = path_lookup(name, env, platform)
    choices = [(Path(explicit).expanduser(), 'explicit')] if explicit else []
    if not explicit:
        if on_path: choices.append((Path(on_path), 'PATH'))
        choices.extend((path, 'known_location') for path in known_cli_paths(name, home, env, platform, desktop_path))
    found = next(((path, via) for path, via in choices if path.is_file()), None)
    record = {'present': bool(found), 'selected_path': str(found[0]) if found else None,
              'discovered_via': found[1] if found else None, 'on_path': bool(on_path),
              'path_executable': on_path, 'version': None, 'can_execute': None,
              'probe_status': 'not_found', 'auth': {'state': 'not_checked'}}
    if not found: return record
    path, _ = found
    record['selected_on_path'] = bool(on_path and Path(on_path).resolve() == path.resolve())
    if platform != platform_name():
        record['probe_status'] = 'not_native_platform'
        return record
    command = launch_command(path, name, env, platform)
    if command is None:
        record['probe_status'] = 'unsupported_shell_wrapper'
        return record
    result = run_probe(command + ['--version'], timeout=timeout, env=env)
    record.update(probe_status=result['status'], can_execute=result['status'] in ('ok', 'nonzero'),
                  exit_code=result['exit_code'])
    match = re.search((r'(?im)^codex-cli\s+' + VERSION if name == 'codex'
                       else r'(?im)^' + VERSION + r'\s+\(Claude Code\)'), result['text'])
    if result['status'] == 'ok' and match:
        record['version'] = match.group(1)
    elif result['status'] == 'ok':
        record['probe_status'] = 'unrecognized_version'
    if check_auth and record['version']:
        auth = ['login', 'status'] if name == 'codex' else ['auth', 'status', '--json']
        record['auth'] = auth_result(name, run_probe(command + auth, timeout=timeout, env=env))
    return record


def windows_packages(env, timeout):
    powershell = Path(env.get('SystemRoot', r'C:\Windows')) / 'System32/WindowsPowerShell/v1.0/powershell.exe'
    if not powershell.is_file(): return [], 'powershell_unavailable'
    result = run_probe([powershell, '-NoProfile', '-NonInteractive', '-Command', WINDOWS_PACKAGES], env=env, timeout=timeout)
    if result['status'] != 'ok': return [], result['status']
    try: records = json.loads(result['text'])
    except ValueError: return [], 'invalid_package_metadata'
    if isinstance(records, dict): records = [records]
    if not isinstance(records, list): return [], 'invalid_package_metadata'
    valid = [r for r in records[:10] if isinstance(r, dict)
             and isinstance(r.get('Name'), str) and re.fullmatch(r'OpenAI\.[A-Za-z0-9.\-]*(?:Codex|ChatGPT)[A-Za-z0-9.\-]*', r['Name'], re.I)
             and isinstance(r.get('InstallLocation'), str) and Path(r['InstallLocation']).is_absolute()]
    return valid, 'ok'


def diagnose_desktop(*, home, env, platform, explicit=None, timeout=5):
    record = {'present': False, 'selected_path': None, 'version': None, 'identity': None,
              'identity_verified': False, 'can_execute': None, 'launch_verified': False,
              'probe_status': 'not_found', 'auth': {'state': 'not_checked'},
              'support': 'preview' if platform == 'linux' else 'available' if platform in ('macos', 'windows') else 'unknown'}
    if platform == 'macos':
        choices = [Path(explicit).expanduser()] if explicit else [Path('/Applications/ChatGPT.app'),
                  Path('/Applications/Codex.app'), home / 'Applications/ChatGPT.app', home / 'Applications/Codex.app']
        for app in choices:
            if not app.exists(): continue
            record.update(present=True, selected_path=str(app), probe_status='unrecognized_app')
            metadata = read_metadata(app / 'Contents/Info.plist', plistlib.loads)
            if not metadata or metadata.get('CFBundleIdentifier') not in MAC_IDENTITIES: continue
            record.update(identity=metadata['CFBundleIdentifier'], identity_verified=True,
                          version=str(metadata.get('CFBundleShortVersionString', '')) or None,
                          probe_status='metadata_only')
            binary = metadata.get('CFBundleExecutable')
            if isinstance(binary, str) and Path(binary).name == binary:
                executable = app / 'Contents/MacOS' / binary
                record['executable_present'] = executable.is_file()
                record['executable_accessible'] = executable.is_file() and os.access(executable, os.X_OK)
            return record
    elif platform == 'windows':
        local = Path(env.get('LOCALAPPDATA', str(home / 'AppData/Local')))
        choices = [Path(explicit).expanduser()] if explicit else [
            local / 'Programs/ChatGPT/ChatGPT.exe', local / 'OpenAI/ChatGPT/ChatGPT.exe',
            local / 'OpenAI/Codex/Codex.exe']
        if not explicit:
            for key in ('ProgramFiles', 'ProgramFiles(x86)'):
                if env.get(key): choices += [Path(env[key]) / 'ChatGPT/ChatGPT.exe', Path(env[key]) / 'OpenAI/Codex/Codex.exe']
        path = next((path for path in choices if path.is_file()), None)
        if path:
            record.update(present=True, selected_path=str(path), executable_present=True, executable_accessible=os.access(path, os.R_OK), probe_status='metadata_unavailable')
        if platform != platform_name():
            record['probe_status'] = 'not_native_platform'
            return record
        if path:
            ps = Path(env.get('SystemRoot', r'C:\Windows')) / 'System32/WindowsPowerShell/v1.0/powershell.exe'
            if ps.is_file():
                result = run_probe([ps, '-NoProfile', '-NonInteractive', '-Command', WINDOWS_VERSION],
                    env=dict(env, AI_RUNTIME_DESKTOP_PATH=str(path)), timeout=timeout)
                if result['status'] == 'ok':
                    try: value = json.loads(result['text'])
                    except ValueError: value = None
                    if (isinstance(value, dict) and value.get('ProductName') in ('ChatGPT', 'Codex')
                            and 'OpenAI' in str(value.get('CompanyName', ''))):
                        record.update(identity=value['ProductName'], identity_verified=True,
                                      version=value.get('FileVersion'), probe_status='metadata_only')
            return record
        packages, status = windows_packages(env, timeout)
        record['package_query_status'] = status
        for package in packages:
            location = Path(package['InstallLocation'])
            if location.is_dir():
                record.update(present=True, selected_path=str(location), identity=package['Name'],
                              identity_verified=True, version=str(package.get('Version', '')) or None,
                              probe_status='package_metadata_only',
                              executable_present=any((location / name).is_file() for name in
                                  ('ChatGPT.exe', 'Codex.exe', 'app/ChatGPT.exe', 'app/Codex.exe')))
                record['executable_accessible'] = record['executable_present']
                return record
    elif platform == 'linux':
        located = path_lookup('chatgpt', env, platform)
        path = Path(explicit).expanduser() if explicit else Path(located) if located else Path('/opt/ChatGPT/chatgpt')
        if path.is_file():
            record.update(present=True, selected_path=str(path), probe_status='presence_only',
                          executable_present=os.access(path, os.X_OK))
    return record


def diagnose(*, home=None, codex=None, claude=None, desktop=None, check_auth=False,
             timeout=5, platform=None, env=None):
    if not isinstance(timeout, (int, float)) or not 0 < timeout <= 15:
        raise ValueError('Probe timeout must be greater than zero and at most 15 seconds')
    env = dict(os.environ if env is None else env)
    platform = platform_name(platform)
    home = Path(home).expanduser().resolve() if home else Path.home()
    if not home.is_dir(): raise ValueError('Discovery home must be an existing directory')
    app = diagnose_desktop(home=home, env=env, platform=platform, explicit=desktop, timeout=timeout)
    desktop_path = Path(app['selected_path']) if app['identity_verified'] else None
    # --home controls discovery only, never silently changes which user's auth is checked.
    auth_allowed = check_auth and home.resolve() == Path.home().resolve()
    runtimes = {'desktop': app}
    for name, explicit in (('codex', codex), ('claude', claude)):
        runtimes[name] = diagnose_cli(name, home=home, env=env, platform=platform, explicit=explicit,
                                      desktop_path=desktop_path, check_auth=auth_allowed, timeout=timeout)
        if check_auth and not auth_allowed:
            runtimes[name]['auth'] = {'state': 'not_checked', 'reason': 'discovery_home_override'}
    guidance = {
        'codex': {'url': 'https://learn.chatgpt.com/docs/codex/cli',
                  'install': 'Review the official standalone CLI installer for this OS.', 'login': 'codex login'},
        'claude': {'url': 'https://code.claude.com/docs/en/setup',
                   'install': 'winget install Anthropic.ClaudeCode' if platform == 'windows' else 'Review the official native installer or package-manager instructions.',
                   'login': 'claude auth login'},
        'desktop': {'url': ('https://learn.chatgpt.com/docs/linux/linux-app' if platform == 'linux' else
                           'https://learn.chatgpt.com/docs/windows/windows-app' if platform == 'windows' else
                           'https://chatgpt.com/download'), 'install': 'Download the official desktop application for your OS.'}}
    for name, record in runtimes.items():
        record['ready'] = (bool(record['present'] and record.get('executable_accessible')
                              and record['identity_verified'] and platform == platform_name())
                           if name == 'desktop' else bool(record['version'] and record['can_execute']
                                                          and record['probe_status'] == 'ok'))
        if not record['present']: record['next_step'] = guidance[name]['install']
        elif name != 'desktop' and not record.get('selected_on_path'):
            record['next_step'] = 'Use the discovered absolute path, or add its directory to PATH in your shell configuration.'
        elif record['probe_status'] not in ('ok', 'metadata_only', 'package_metadata_only'):
            record['next_step'] = 'Review the probe status and verify this runtime using the official setup documentation.'
        if record['auth']['state'] == 'not_authenticated': record['next_step'] = guidance[name]['login']
    return {'schema_version': 1, 'platform': platform, 'host_platform': platform_name(),
            'native_validation': platform == platform_name(), 'runtimes': runtimes, 'guidance': guidance,
            'ready': all(record['ready'] for record in runtimes.values()),
            'authentication_attention': [name for name in ('codex', 'claude')
                if check_auth and runtimes[name]['auth']['state'] != 'authenticated'],
            'limits': ['Desktop apps are not launched; metadata is not signature or login validation.',
                       'No auth files are read. Auth status checks are optional and do not establish model/service access.',
                       'Discovery covers PATH and bounded known locations; an undiscovered custom installation may still exist.',
                       'Linux desktop is an official preview; feature support differs from macOS/Windows.']}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    doctor = commands.add_parser('doctor', help='Discover runtimes without installing or signing in')
    doctor.add_argument('--json', action='store_true')
    doctor.add_argument('--home', type=Path)
    for name in ('codex', 'claude', 'desktop'): doctor.add_argument('--' + name, type=Path)
    doctor.add_argument('--check-auth', action='store_true')
    args = parser.parse_args(argv)
    report = diagnose(home=args.home, codex=args.codex, claude=args.claude,
                      desktop=args.desktop, check_auth=args.check_auth)
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        for name, record in report['runtimes'].items():
            print(name + ': ' + record['probe_status'] + (' (' + record['version'] + ')' if record['version'] else ''))
            if record['selected_path']: print('  ' + record['selected_path'])
            if record.get('next_step'): print('  ' + record['next_step'] + ' ' + report['guidance'][name]['url'])
            if args.check_auth: print('  auth: ' + record['auth']['state'])
    return 0 if report['ready'] and not report['authentication_attention'] else 1
