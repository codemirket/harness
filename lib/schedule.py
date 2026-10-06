"""Owned native daily maintenance registration; Python 3.9+, standard library.

Cron uses the current user's table. Windows uses a current-user interactive,
least-privilege task: it cannot run while that user is logged out. Registration
is verified, but does not prove a future execution or wake a sleeping computer.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import uuid
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
BEGIN = '# BEGIN personal-ai daily-maintenance v1'
END = '# END personal-ai daily-maintenance v1'
OWNER = 'personal-ai:daily-maintenance:v1'
TASK_PREFIX = 'PersonalAI-DailyMaintenance-'
NS = 'http://schemas.microsoft.com/windows/2004/02/mit/task'
MAX_BYTES = 1024 * 1024
TIMEOUT = 20
# All PowerShell is fixed code. Paths, XML and expected state travel as JSON on
# stdin, never as executable fragments or command-line substitutions.
PS_COMMON = r'''
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = New-Object System.Text.UTF8Encoding($false)
$sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$name = 'PersonalAI-DailyMaintenance-' + $sid
$service = New-Object -ComObject 'Schedule.Service'
$service.Connect()
$folder = $service.GetFolder('\')
function Read-OwnedTask {
    try { return $folder.GetTask($name).Xml }
    catch {
        if ($_.Exception.HResult -eq -2147024894 -or $_.Exception.InnerException.HResult -eq -2147024894) { return $null }
        throw
    }
}
'''
PS_READ = PS_COMMON + r'''
$xml = Read-OwnedTask
@{sid=$sid; xml=$xml} | ConvertTo-Json -Compress
'''
PS_WRITE = PS_COMMON + r'''
$payload = [Console]::In.ReadToEnd() | ConvertFrom-Json
if ($payload.sid -cne $sid) { throw 'Current user changed' }
$current = Read-OwnedTask
if ($current -cne $payload.previous) { throw 'Task changed since preflight' }
$flags = if ($null -eq $current) { 2 } else { 4 }
$null = $folder.RegisterTask($name, $payload.xml, $flags, $sid, $null, 3, $null)
@{sid=$sid; xml=(Read-OwnedTask)} | ConvertTo-Json -Compress
'''


def current_home():
    if os.name != 'nt':
        import pwd
        return Path(pwd.getpwuid(os.getuid()).pw_dir).resolve()
    return Path.home().resolve()


def backend():
    if sys.platform == 'win32':
        return 'windows-task-scheduler'
    if sys.platform == 'darwin' or sys.platform.startswith('linux'):
        return 'crontab'
    raise ValueError('Native scheduling is supported on macOS, Linux and Windows only')


def run(argv, input_text=None):
    """Bounded, noninteractive OS command; callers never expose private output."""
    env = dict(os.environ, LC_ALL='C', LANG='C')
    try:
        result = subprocess.run([str(x) for x in argv],
                                input=input_text.encode('utf-8') if input_text is not None else None,
                                stdin=subprocess.DEVNULL if input_text is None else None,
                                capture_output=True, timeout=TIMEOUT, env=env, check=False)
        if len(result.stdout) + len(result.stderr) > MAX_BYTES:
            raise ValueError('Native scheduler response exceeds the supported size')
        # Decode explicitly: text=True would normalize unrelated crontab CRLF.
        result.stdout = result.stdout.decode('utf-8')
        result.stderr = result.stderr.decode('utf-8')
    except (OSError, UnicodeError, subprocess.TimeoutExpired) as error:
        raise ValueError('Native scheduler command failed or timed out; no success was confirmed') from error
    return result


def crontab_tool():
    for candidate in (Path('/usr/bin/crontab'), Path('/bin/crontab')):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return str(candidate)
    located = shutil.which('crontab')
    if not located:
        raise ValueError('crontab is unavailable; install/enable the OS cron service first')
    return str(Path(located).resolve())


def read_crontab(tool):
    result = run([tool, '-l'])
    if result.returncode == 0:
        if '\x00' in result.stdout:
            raise ValueError('Invalid NUL in crontab')
        return result.stdout
    if result.returncode == 1 and not result.stdout and re.fullmatch(r'(?:crontab: )?no crontab for [^\r\n]+\s*', result.stderr.strip(), re.I):
        return ''
    raise ValueError('Cannot read the current user crontab; review OS cron permissions')


def cron_escape(text):
    # Close/reopen a shell quote around each escaped percent. This also handles
    # a literal backslash immediately before percent without cron seeing an
    # even escape pair followed by an unescaped percent. Input uses shlex.quote.
    return text.replace('%', "'\\%'")


def cron_line(command, home):
    path = ':'.join([str(home / '.local/bin'), '/opt/homebrew/bin', '/usr/local/bin',
                     '/usr/bin', '/bin', '/usr/sbin', '/sbin'])
    shell = 'exec ' + ' '.join(shlex.quote(str(x)) for x in command)
    invocation = ['/usr/bin/env', 'HOME=' + str(home), 'PATH=' + path, '/bin/sh', '-c', shell]
    return '0 0 * * * ' + cron_escape(' '.join(shlex.quote(x) for x in invocation)) + ' >/dev/null 2>&1'


def legacy_line(root, home):
    # Deliberately exact: only the original generated job is a known migration.
    script = str(root / 'setup/macos.sh')
    log = str(home / 'Library/Logs/shared-ai-install.log')
    if any(ch.isspace() or ch in "'\"\\%$`;|&<>" for ch in script + log):
        return None
    return ('0 0 * * * { /bin/date; /bin/sh ' + script + ' codex; codex_status=$?; /bin/sh ' + script +
            ' claude; claude_status=$?; /bin/echo "codex_exit=$codex_status claude_exit=$claude_status"; '
            'test "$codex_status" -eq 0 && test "$claude_status" -eq 0; } >> ' + log + ' 2>&1')


def legacy_variant(row, script):
    """Recognize installer invocations, not comments or arbitrary path mentions."""
    stripped = row.lstrip()
    if not script or not stripped or stripped.startswith('#'):
        return False
    fields = stripped.split(None, 1 if stripped.startswith('@') else 5)
    if len(fields) != (2 if stripped.startswith('@') else 6):
        return False
    try:
        lexer = shlex.shlex(fields[-1], posix=True, punctuation_chars=';&|(){}<>')
        lexer.whitespace_split = True
        tokens = list(lexer)
    except ValueError:
        # A recognizable active invocation with broken quoting needs review too.
        return ('/bin/sh ' + str(script)) in fields[-1]
    separators = {';', '&&', '||', '|', '&', '{', '(', ')', '}'}
    shells = {'sh', '/bin/sh', 'bash', '/bin/bash', 'zsh', '/bin/zsh'}
    for index, token in enumerate(tokens[:-1]):
        if token != str(script) or tokens[index + 1] not in ('codex', 'claude', 'claude-code', 'both'):
            continue
        start = index - 1 if index > 0 and tokens[index - 1] in shells else index
        if start == 0 or tokens[start - 1] in separators:
            return True
    return False


def merge_crontab(text, line, legacy=None, legacy_script=None):
    rows = text.splitlines(keepends=True)
    if legacy_script is None and legacy and '/bin/sh ' in legacy:
        legacy_script = legacy.split('/bin/sh ', 1)[1].split(' codex;', 1)[0]
    for row in rows:
        if row.rstrip('\r\n') != legacy and legacy_variant(row, legacy_script):
            raise ValueError('A modified same-repository legacy installer job requires manual review; preserving all jobs')
    starts = [i for i, row in enumerate(rows) if row.rstrip('\r\n') == BEGIN]
    ends = [i for i, row in enumerate(rows) if row.rstrip('\r\n') == END]
    marker_rows = [row for row in rows if 'personal-ai daily-maintenance' in row]
    if len(starts) != len(ends) or len(starts) > 1 or len(marker_rows) != 2 * len(starts):
        raise ValueError('Ambiguous or malformed owned cron markers; resolve manually')
    if starts and (ends[0] != starts[0] + 2 or not rows[starts[0] + 1].startswith('0 0 * * * ')):
        raise ValueError('Unexpected content inside owned cron block; preserving it')
    legacy_indices = [i for i, row in enumerate(rows) if legacy and row.rstrip('\r\n') == legacy]
    if len(legacy_indices) > 1 or (starts and legacy_indices):
        raise ValueError('Multiple owned or legacy jobs; resolve duplicates manually')
    for row in rows:
        match = re.match(r'\s*SHELL\s*=\s*(.*?)\s*$', row)
        if match and match[1].strip("\"'") not in ('/bin/sh', '/bin/bash', '/bin/zsh', '/bin/dash', '/usr/bin/sh', '/usr/bin/bash', '/usr/bin/zsh', '/usr/bin/dash'):
            raise ValueError('An unsupported cron SHELL requires manual review; unrelated job environment is preserved')
    # An inherited alternate CRON_TZ would undermine the local-midnight contract.
    if any(re.match(r'\s*CRON_TZ\s*=', row) for row in rows):
        raise ValueError('Existing CRON_TZ requires manual review to preserve local midnight and unrelated jobs')
    block = BEGIN + '\n' + line + '\n' + END + '\n'
    if starts:
        return ''.join(rows[:starts[0]]) + block + ''.join(rows[ends[0] + 1:]), False
    if legacy_indices:
        i = legacy_indices[0]
        return ''.join(rows[:i]) + block + ''.join(rows[i + 1:]), True
    prefix = '\n' if text and not text.endswith('\n') else ''
    return text + prefix + block, False


def powershell_tool():
    tool = Path(os.environ.get('SystemRoot', r'C:\Windows')) / 'System32/WindowsPowerShell/v1.0/powershell.exe'
    if not tool.is_file():
        raise ValueError('Windows PowerShell is unavailable; Task Scheduler could not be checked')
    return str(tool)


def ps_call(tool, script, payload=None):
    result = run([tool, '-NoProfile', '-NonInteractive', '-Command', script],
                 json.dumps(payload) if payload is not None else None)
    if result.returncode:
        raise ValueError('Task Scheduler operation failed; inspect current-user task permissions')
    try:
        value = json.loads(result.stdout)
    except ValueError:
        raise ValueError('Task Scheduler returned an invalid response') from None
    if (not isinstance(value, dict) or not isinstance(value.get('sid'), str)
            or not re.fullmatch(r'S-1-[0-9-]+', value.get('sid', ''))
            or value.get('xml') is not None and not isinstance(value['xml'], str)):
        raise ValueError('Task Scheduler returned an invalid task identity')
    return value


def task_xml(command, root, sid):
    ET.register_namespace('', NS)
    def add(parent, name, text=None, **attrs):
        element = ET.SubElement(parent, '{' + NS + '}' + name, attrs)
        element.text = text
        return element
    task = ET.Element('{' + NS + '}Task', {'version': '1.2'})
    registration = add(task, 'RegistrationInfo')
    add(registration, 'Description', OWNER)
    triggers = add(task, 'Triggers')
    daily = add(triggers, 'CalendarTrigger')
    add(daily, 'Enabled', 'true')
    add(daily, 'StartBoundary', '2020-01-01T00:00:00')
    add(add(daily, 'ScheduleByDay'), 'DaysInterval', '1')
    principal = add(add(task, 'Principals'), 'Principal', id='Author')
    add(principal, 'UserId', sid)
    add(principal, 'LogonType', 'InteractiveToken')
    add(principal, 'RunLevel', 'LeastPrivilege')
    options = add(task, 'Settings')
    for key, value in [('MultipleInstancesPolicy', 'IgnoreNew'), ('DisallowStartIfOnBatteries', 'false'),
                       ('StopIfGoingOnBatteries', 'false'), ('StartWhenAvailable', 'true'),
                       ('Enabled', 'true'), ('ExecutionTimeLimit', 'PT1H')]:
        add(options, key, value)
    action = add(add(task, 'Actions', Context='Author'), 'Exec')
    add(action, 'Command', str(command[0]))
    add(action, 'Arguments', subprocess.list2cmdline([str(x) for x in command[1:]]))
    add(action, 'WorkingDirectory', str(root))
    return ET.tostring(task, encoding='unicode')


def task_contract(xml):
    """Compare owned executable behavior, tolerating scheduler-added defaults."""
    if not isinstance(xml, str) or '<!DOCTYPE' in xml.upper() or '<!ENTITY' in xml.upper():
        raise ValueError('Unsafe task XML')
    try:
        task = ET.fromstring(xml)
    except ET.ParseError:
        raise ValueError('Invalid existing task XML') from None
    ns = {'t': NS}
    if task.tag != '{' + NS + '}Task' or task.findtext('t:RegistrationInfo/t:Description', namespaces=ns) != OWNER:
        raise ValueError('An unmanaged task occupies the maintenance name; preserving it')
    for path, child in [('Actions', 'Exec'), ('Triggers', 'CalendarTrigger'), ('Principals', 'Principal')]:
        parent = task.find('t:' + path, ns)
        if parent is None or len(parent) != 1 or parent[0].tag != '{' + NS + '}' + child:
            raise ValueError('Unexpected actions, triggers or principals in owned task; preserving it')
    paths = ['Actions/Exec/Command', 'Actions/Exec/Arguments', 'Actions/Exec/WorkingDirectory',
             'Triggers/CalendarTrigger/StartBoundary', 'Triggers/CalendarTrigger/Enabled',
             'Triggers/CalendarTrigger/ScheduleByDay/DaysInterval',
             'Principals/Principal/UserId', 'Principals/Principal/LogonType', 'Principals/Principal/RunLevel',
             'Settings/MultipleInstancesPolicy', 'Settings/DisallowStartIfOnBatteries',
             'Settings/StopIfGoingOnBatteries', 'Settings/StartWhenAvailable', 'Settings/Enabled',
             'Settings/ExecutionTimeLimit']
    # Repetition or an end boundary must not silently pass as our daily schedule.
    trigger = task.find('t:Triggers/t:CalendarTrigger', ns)
    if any(element.tag.rsplit('}', 1)[-1] not in ('StartBoundary', 'Enabled', 'ScheduleByDay', 'ExecutionTimeLimit', 'RandomDelay') for element in trigger):
        raise ValueError('Unexpected options in owned daily trigger; preserving it')
    # Defaults from Microsoft's Task Scheduler schema. Export-ScheduledTask may
    # add them even when registration XML omitted them. Nondefault execution
    # conditions must cause drift instead of an incorrectly verified no-op.
    defaults = {'RunOnlyIfIdle': 'false', 'RunOnlyIfNetworkAvailable': 'false',
                'WakeToRun': 'false', 'AllowStartOnDemand': 'true',
                'AllowHardTerminate': 'true', 'Priority': '7',
                'DisallowStartOnRemoteAppSession': 'false',
                'UseUnifiedSchedulingEngine': 'false', 'DeleteExpiredTaskAfter': 'PT0S'}
    settings = task.find('t:Settings', ns)
    required = {path.split('/')[-1] for path in paths if path.startswith('Settings/')}
    allowed = required | set(defaults) | {'Hidden', 'IdleSettings', 'NetworkSettings'}
    if settings is None:
        raise ValueError('Owned task has no Settings; preserving it')
    names = [element.tag.rsplit('}', 1)[-1] for element in settings]
    if len(names) != len(set(names)) or any(name not in allowed for name in names):
        raise ValueError('Unknown or duplicate task execution settings require manual review')
    optional = [trigger.findtext('t:ExecutionTimeLimit', default='PT72H', namespaces=ns),
                trigger.findtext('t:RandomDelay', default='PT0M', namespaces=ns)]
    for name, default in defaults.items():
        value = settings.findtext('t:' + name, default=default, namespaces=ns)
        if default in ('true', 'false'):
            value = {'1': 'true', '0': 'false'}.get(value, value)
            if value not in ('true', 'false'):
                raise ValueError('Invalid task execution setting')
        optional.append(value)
    # Idle/network subsettings have no effect while their enabling condition is
    # false; a true condition already differs via the normalized values above.
    return tuple(task.findtext('/'.join('t:' + part for part in path.split('/')), namespaces=ns) for path in paths) + tuple(optional)


def safe_directory(path):
    from .catalog import is_link
    for ancestor in (path, *path.parents):
        if is_link(ancestor) or ancestor.exists() and not ancestor.is_dir():
            raise ValueError('Unsafe scheduler state directory; refusing linked or non-directory ancestors')


def validate_root(root):
    manifest = root / 'registry/harness.json'
    try:
        if manifest.stat().st_size > MAX_BYTES:
            raise ValueError('Harness manifest exceeds the supported size')
        data = json.loads(manifest.read_text(encoding='utf-8'))
    except (OSError, ValueError):
        raise ValueError('Readable registry/harness.json is required for native scheduling') from None
    declaration = data.get('maintenance') if isinstance(data, dict) else None
    if (not isinstance(declaration, dict) or declaration.get('enabled') is not True
            or declaration.get('time') != '00:00' or declaration.get('timezone') != 'system'):
        raise ValueError('Harness maintenance must declare enabled=true, time=00:00 and timezone=system')
    git = shutil.which('git')
    if not git:
        raise ValueError('Git is required to verify the scheduled repository checkout')
    result = run([git, '-c', 'core.hooksPath=/dev/null', '-c', 'core.fsmonitor=false',
                  '-C', str(root), 'rev-parse', '--show-toplevel'])
    if result.returncode or Path(result.stdout.strip()).resolve() != root:
        raise ValueError('Scheduling requires the actual Git checkout root; exported bundles are unsupported')


def _prepare(root=None, home=None, mode='auto', python=None):
    report = {'status': 'blocked', 'ready_to_apply': False, 'changed': False,
              'applied': False, 'verified': False, 'schedule': 'daily 00:00 local time',
              'expression': '0 0 * * *', 'backend': None, 'command': [], 'blockers': [],
              'instructions': [], 'backup': None,
              'limits': ['Registration verification does not prove a future run.',
                         'Cron does not catch up missed runs; Windows catches up while the user is logged in.',
                         'Scheduler updates recheck prior state but cannot lock out external scheduler editors.',
                         'Maintenance owns bounded logs. Startup failures may prevent a status update; cron launcher output is suppressed.']}
    try:
        actual_home = current_home()
        home = Path(home).expanduser().resolve(strict=True) if home is not None else actual_home
        if home != actual_home:
            raise ValueError('Native schedule registration must target the current OS user home')
        root = Path(root or ROOT).expanduser().resolve(strict=True)
        executable = Path(python or sys.executable).expanduser()
        if not executable.is_absolute():
            raise ValueError('Scheduled Python executable must be an absolute path')
        executable = executable.resolve(strict=True)
        if not (root / 'ai.py').is_file() or not executable.is_file() or not os.access(executable, os.X_OK):
            raise ValueError('The repository entry point and executable Python are required')
        validate_root(root)
        if mode not in ('auto', 'copy', 'link'):
            raise ValueError('Unsupported installation mode')
        command = [str(executable), '-E', '-s', '-B', str(root / 'ai.py'), 'maintenance', 'run']
        if mode != 'auto': command += ['--mode', mode]
        if any(any(c in part for c in '\x00\r\n') for part in command + [str(home)]):
            raise ValueError('Newlines and NUL are not supported in scheduler paths')
        state = home / '.agent-harness'
        safe_directory(state)
        safe_directory(state / 'schedule-backups')
        selected = backend()
        report.update(backend=selected, command=command, log=str(state / 'logs/maintenance.log'))
        if selected == 'crontab':
            tool = crontab_tool()
            previous = read_crontab(tool)
            desired, migrated = merge_crontab(previous, cron_line(command, home), legacy_line(root, home), root / 'setup/macos.sh')
            if len(desired.encode('utf-8')) > MAX_BYTES:
                raise ValueError('Resulting crontab exceeds the supported size')
            report['legacy_migration'] = migrated
            changed = desired != previous
            sid = None
        else:
            tool = powershell_tool()
            record = ps_call(tool, PS_READ)
            previous = record['xml']; sid = record['sid']
            desired = task_xml(command, root, sid)
            changed = previous is None or task_contract(previous) != task_contract(desired)
            report['task_name'] = TASK_PREFIX + sid
            report['expression'] = 'CalendarTrigger: daily 00:00; InteractiveToken; StartWhenAvailable'
            report['instructions'].append('Windows runs without passwords or elevation, only while this user is logged in.')
        report.update(status='pending' if changed else 'unchanged', ready_to_apply=True,
                      changed=changed, verified=not changed)
        return report, {'home': home, 'tool': tool, 'previous': previous, 'desired': desired,
                        'sid': sid, 'state': state}
    except (ValueError, OSError) as error:
        report['blockers'].append(str(error))
        report['instructions'].append('Resolve the blocker and rerun schedule plan; no unrelated jobs are removed.')
        return report, None


def plan(root=None, home=None, mode='auto', python=None):
    return _prepare(root, home, mode, python)[0]


check = plan


def apply(root=None, home=None, mode='auto', python=None):
    result, context = _prepare(root, home, mode, python)
    if not context or not result['changed']:
        return result
    lock = context['state'] / 'schedule.lock'
    descriptor = None
    try:
        safe_directory(context['state'])
        context['state'].mkdir(parents=True, exist_ok=True, mode=0o700)
        descriptor = os.open(lock, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        os.close(descriptor); descriptor = -1
        # Recheck both before the backup and immediately before replacement.
        def read():
            if result['backend'] == 'crontab': return read_crontab(context['tool'])
            value = ps_call(context['tool'], PS_READ)
            if value['sid'] != context['sid']: raise ValueError('Current scheduler user changed')
            return value['xml']
        if read() != context['previous']:
            raise ValueError('Scheduler changed since preflight; retry without overwriting it')
        backups = context['state'] / 'schedule-backups'
        safe_directory(backups)
        backups.mkdir(mode=0o700, exist_ok=True)
        backup = backups / (uuid.uuid4().hex + ('.crontab' if result['backend'] == 'crontab' else '.xml'))
        fd = os.open(backup, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, 'w', encoding='utf-8', newline='') as stream:
            stream.write(context['previous'] or '')
            stream.flush(); os.fsync(stream.fileno())
        result['backup'] = str(backup)
        if read() != context['previous']:
            raise ValueError('Scheduler changed during backup; no registration performed')
        if result['backend'] == 'crontab':
            outcome = run([context['tool'], '-'], context['desired'])
            if outcome.returncode: raise ValueError('crontab rejected registration; prior state is backed up')
        else:
            ps_call(context['tool'], PS_WRITE, {'sid': context['sid'], 'previous': context['previous'], 'xml': context['desired']})
        result['applied'] = True
        after = read()
        verified = (after == context['desired'] if result['backend'] == 'crontab' else
                    after is not None and task_contract(after) == task_contract(context['desired']))
        if not verified:
            raise ValueError('Scheduler read-back differs from the requested job; inspect it before retrying')
        result.update(status='registered', changed=False, verified=True)
    except (OSError, ValueError) as error:
        result.update(status='failed', ready_to_apply=False, verified=False)
        result['blockers'].append('Registration did not complete with verified success: ' + str(error))
        result['instructions'].append('Inspect the native scheduler and retained backup; no automatic rollback overwrites concurrent edits.')
    finally:
        if descriptor == -1:
            lock.unlink(missing_ok=True)
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['plan', 'check', 'install', 'apply'])
    parser.add_argument('--root', type=Path)
    parser.add_argument('--home', type=Path)
    parser.add_argument('--mode', choices=['auto', 'copy', 'link'], default='auto')
    parser.add_argument('--python', type=Path)
    args = vars(parser.parse_args(argv))
    operation = args.pop('command')
    report = apply(**args) if operation in ('install', 'apply') else plan(**args)
    print(json.dumps(report, indent=2))
    return 0 if report['ready_to_apply'] and (operation == 'plan' or report['verified']) else 1
