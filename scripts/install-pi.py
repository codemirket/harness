#!/usr/bin/env python3
"""Install an extracted Mirket package and its acquired exact Pi host.

Inputs must remain under the permanent install root. No network, npm, upstream
installer or PATH runtime is used. Existing state is backed up and preserved;
failed native setup is deliberately left available for explicit recovery.
"""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import stat
import subprocess
import sys
import tempfile


ENTRY = '@earendil-works/pi-coding-agent/dist/cli.js'
MANAGED_LINK = '../../.pi/agent/bin/pi'
MAX_FILE = 256 * 1024 * 1024

# This is an installation health check, with no provider, prompt or user auth.
# Resource discovery, model caches and sessions stay in the disposable probe.
READINESS = r"""
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {readFileSync} from 'node:fs';
const [pkg,host,agent,probe,release]=process.argv.slice(1);
const sdk=await import(pathToFileURL(join(host,'node_modules/@earendil-works/pi-coding-agent/dist/index.js')));
const {AuthStorage}=await import(pathToFileURL(join(host,'node_modules/@earendil-works/pi-coding-agent/dist/core/auth-storage.js')));
const modelRuntime=await sdk.ModelRuntime.create({credentials:AuthStorage.inMemory(),modelsPath:null,modelsStorePath:join(probe,'models'),allowModelNetwork:false,refreshOnCreate:false});
const configured=JSON.parse(readFileSync(join(agent,'settings.json'),'utf8')).packages.filter(p=>p&&typeof p==='object'&&p.source===pkg);
if(configured.length!==1)throw new Error('Real Pi settings package registration changed');
const settingsManager=sdk.SettingsManager.inMemory({packages:configured});
const loader=new sdk.DefaultResourceLoader({cwd:probe,agentDir:probe,settingsManager,noContextFiles:true,noSkills:true,noPromptTemplates:true,noThemes:true,extensionFactories:[{name:'codemode',factory:sdk.createCodemodeExtension(),builtin:true,replaceable:true},{name:'tool-search',factory:sdk.createToolSearchExtension(),builtin:true,replaceable:true},{name:'mcp',factory:sdk.createMcpExtension(),builtin:true,replaceable:true}]});
await loader.reload();
if(loader.getExtensions().errors.length)throw new Error(JSON.stringify(loader.getExtensions().errors));
const {session}=await sdk.createAgentSession({cwd:probe,agentDir:probe,settingsManager,resourceLoader:loader,modelRuntime,sessionManager:sdk.SessionManager.inMemory(probe),noTools:'builtin'});
try {
  await session.bindExtensions({mode:'print',uiContext:{notify:()=>{}},onError:e=>{throw new Error(JSON.stringify(e));}});
  // Slash commands execute locally before any model request. This inspects
  // session registration; it does not claim an actual runtime_status call.
  await session.prompt('/mirket doctor');
  const message=session.messages.findLast(m=>m.customType==='mirket-doctor');
  const content=typeof message?.content==='string'?message.content:message?.content?.filter(p=>p.type==='text').map(p=>p.text).join('\n');
  const details=JSON.parse(content??'null');
  const registration=loader.getExtensions().runtime.mcpServers.get('mirket');
  const config=registration?.config;
  const expectedArgs=['serve','--home',join(agent,'mirket'),'--expected-release-id',release];
  const pending=['Runtime status unavailable or filtered','Awaiting actual runtime_status identity'];
  if(details?.status!=='not_ready'||!Array.isArray(details.issues)||!details.issues.includes(pending[1])||details.issues.some(issue=>!pending.includes(issue))||details.releaseId!==release||details.project!==probe||registration?.extensionPath!==join(pkg,'dist/extension/index.js')||config?.command!==join(agent,'mirket/bin/mirket')||config?.cwd!==probe||JSON.stringify(config?.args)!==JSON.stringify(expectedArgs)||config?.exposure!=='deferred'||config?.toolExposure?.runtime_status!=='direct')throw new Error(JSON.stringify({details,registration}));
  console.log(JSON.stringify({status:'registered',releaseId:details.releaseId}));
} finally {
  await session.extensionRunner.emit({type:'session_shutdown',reason:'quit'});
  session.dispose();
}
"""


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(path, exists=True):
    if (not path.is_absolute() or path.resolve() != path
            or any(p.is_symlink() for p in (path, *path.parents))):
        raise ValueError('Select canonical absolute paths without symlink ancestry: ' + str(path))
    if exists and not path.exists():
        raise ValueError('Selected input does not exist: ' + str(path))


def regular(path, limit=MAX_FILE):
    canonical(path)
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size > limit:
            raise ValueError('Expected bounded unaliased regular file: ' + str(path))
        with os.fdopen(fd, 'rb', closefd=False) as source:
            data = source.read(limit + 1)
        if len(data) > limit:
            raise ValueError('File exceeds size bound: ' + str(path))
        return data
    finally:
        os.close(fd)


def directory(path, private=False):
    canonical(path)
    info = path.stat()
    if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid():
        raise ValueError('Expected an owned directory: ' + str(path))
    if private and info.st_mode & 0o077:
        raise ValueError('Expected a private directory: ' + str(path))
    if info.st_mode & 0o022:
        raise ValueError('Writable shared directory is refused: ' + str(path))


def relative(name):
    if not isinstance(name, str) or not name or '\\' in name or any(p in ('', '.', '..') for p in name.split('/')) or name.startswith('/'):
        raise ValueError('Invalid inventory path')
    return name


def run_json(command, cwd, env, timeout=120):
    output = subprocess.check_output([str(p) for p in command], cwd=cwd, env=env, timeout=timeout)
    try:
        return json.loads(output)
    except (ValueError, TypeError) as error:
        raise ValueError('Command did not return one JSON report: ' + str(command[0])) from error


def verify(package, host, env, cwd):
    for path in (package, host, host / 'node_modules'):
        directory(path)
    binding = json.loads(regular(package / 'native-binding.json', 16384))
    if (binding.get('schema') != 1 or binding.get('hostVersion') != '1.4.0'
            or not re.fullmatch(r'sha256:[a-f0-9]{64}', binding.get('releaseId', ''))
            or not all(re.fullmatch(r'[a-f0-9]{64}', binding.get(k, '')) for k in
                       ('coreSha256', 'nodeSha256', 'packageSha256', 'hostInventorySha256'))):
        raise ValueError('Invalid native binding')
    inventory = json.loads(regular(package / 'package-inventory.json', 16 * 1024 * 1024))
    if inventory.get('schema') != 1 or not isinstance(inventory.get('files'), list):
        raise ValueError('Invalid package inventory')
    expected = set()
    for entry in inventory['files']:
        name = relative(entry['path'])
        if name in expected:
            raise ValueError('Duplicate package inventory path')
        expected.add(name)
        data = regular(package / name)
        if len(data) != entry['bytes'] or sha(data) != entry['sha256']:
            raise ValueError('Package file differs from inventory: ' + name)
    for parent, dirs, files in os.walk(package, followlinks=False):
        for name in dirs + files:
            item = Path(parent) / name
            name = item.relative_to(package).as_posix()
            if item.is_symlink():
                if not name.startswith('native/'):
                    raise ValueError('Unexpected package link: ' + name)
            elif item.is_dir():
                continue
            elif name != 'package-inventory.json' and name not in expected:
                raise ValueError('Uninventoried package file: ' + name)
    binary = package / 'native/bin/mirket'
    node = package / 'native/runtime/node/bin/node'
    for path, key in ((binary, 'coreSha256'), (node, 'nodeSha256'),
                      (package / 'package.json', 'packageSha256'),
                      (package / 'host-inventory.json', 'hostInventorySha256')):
        if sha(regular(path)) != binding[key]:
            raise ValueError('Package identity differs: ' + str(path))
    for path in (binary, node):
        if not os.access(path, os.X_OK):
            raise ValueError('Bound executable is not executable: ' + str(path))
    pkg = json.loads(regular(package / 'package.json', 65536))
    if pkg.get('name') != '@codemirket/pi-harness' or pkg.get('version') != binding['packageVersion']:
        raise ValueError('Package identity differs from binding')
    native = run_json([binary, 'verify', '--source', package / 'native'], cwd, env)
    if native.get('release_id') != binding['releaseId']:
        raise ValueError('Native release differs from binding')
    inventory = json.loads(regular(package / 'host-inventory.json', 16 * 1024 * 1024))
    if inventory.get('entry') != ENTRY or inventory.get('hostVersion') != binding['hostVersion']:
        raise ValueError('Unexpected unbundled Pi host entry')
    result = run_json([binary, 'verify-pi-host', '--root', host / 'node_modules',
                       '--inventory', package / 'host-inventory.json', '--sha256', binding['hostInventorySha256']], cwd, env)
    if (result.get('root') != str(host / 'node_modules') or result.get('entry') != str(host / 'node_modules' / ENTRY)
            or result.get('lockSha256') != inventory['lockSha256'] or result.get('verifiedFiles') != len(inventory['files'])):
        raise ValueError('Native host verification response differs')
    return binding, binary, node


def wrapper(agent, node, entry):
    return ('#!/bin/sh\nset -eu\n'
            'export PI_CODING_AGENT_DIR=' + shlex.quote(str(agent)) + '\n'
            'unset PI_MANAGED_INSTALL_ROOT\n'
            'exec ' + shlex.quote(str(node)) + ' ' + shlex.quote(str(entry)) + ' "$@"\n').encode()


def legacy_wrapper(agent, node, entry):
    # Recognize only the prior first-party exact-path bootstrap, for safe reruns.
    return ('#!/bin/sh\nset -eu\nif [ -z "${PI_CODING_AGENT_DIR:-}" ]; then\n'
            '  PI_CODING_AGENT_DIR=' + shlex.quote(str(agent)) + '\n'
            '  export PI_CODING_AGENT_DIR\nfi\nexec ' + shlex.quote(str(node)) + ' '
            + shlex.quote(str(entry)) + ' "$@"\n').encode()


def settings_check(agent, package, home):
    path = agent / 'settings.json'
    if path.with_name('settings.json.lock').exists():
        raise ValueError('Pi settings are busy')
    data = regular(path, 2 * 1024 * 1024) if path.exists() or path.is_symlink() else None
    settings = json.loads(data) if data is not None else {}
    if not isinstance(settings, dict) or not isinstance(settings.get('packages', []), list):
        raise ValueError('Invalid Pi settings')
    matches = []
    for entry in settings.get('packages', []):
        source = entry if isinstance(entry, str) else entry.get('source') if isinstance(entry, dict) else None
        if not isinstance(source, str):
            raise ValueError('Invalid Pi package source')
        source = source.strip()
        if re.match(r'^npm:@codemirket/pi-harness(?:@|$)', source):
            raise ValueError('Conflicting registry Mirket package')
        if re.match(r'^[a-z]+:|^git@', source, re.I):
            continue
        local = home if source == '~' else home / source[2:] if source.startswith('~/') else agent / source
        local = local.resolve()
        if local == package:
            matches.append(entry)
        elif (local / 'package.json').exists():
            other = json.loads(regular(local / 'package.json', 65536))
            if other.get('name') == '@codemirket/pi-harness':
                raise ValueError('Conflicting local Mirket package')
    if len(matches) > 1:
        raise ValueError('Duplicate Mirket package objects')
    return data, matches


def managed_layout(agent, launcher):
    directory(agent / 'bin')
    regular(agent / 'bin/pi', 65536)
    directory(agent / 'install')
    metadata = json.loads(regular(agent / 'install/managed-install.json', 65536))
    if (metadata.get('kind') != 'pi-managed-install' or metadata.get('schemaVersion') != 1
            or metadata.get('layout') != 'releases-v1'
            or metadata.get('entrypoint') != {'type': 'symlink', 'path': str(launcher)}):
        raise ValueError('Unknown managed Pi layout')
    version = regular(agent / 'install/current-version', 1024).decode().strip()
    if not re.fullmatch(r'[0-9A-Za-z][0-9A-Za-z._+-]*', version) or version in ('.', '..'):
        raise ValueError('Invalid managed Pi version')
    if version != '1.4.0':
        raise ValueError('Managed Pi version differs from the package-bound 1.4.0 host')
    release = agent / 'install/releases' / version
    directory(release)
    cli = release / 'node_modules/@earendil-works/pi-coding-agent/dist/bundle/cli.js'
    regular(cli)
    shim = release / 'node_modules/.bin/pi'
    canonical(shim.parent)
    if not shim.is_symlink() or os.readlink(shim) != '../@earendil-works/pi-coding-agent/dist/bundle/cli.js' or shim.resolve() != cli:
        raise ValueError('Unknown managed Pi entry')
    pkg = json.loads(regular(release / 'node_modules/@earendil-works/pi-coding-agent/package.json', 65536))
    if pkg.get('name') != '@earendil-works/pi-coding-agent' or pkg.get('version') != version:
        raise ValueError('Managed Pi package differs from version metadata')


def registration_check(agent, package, home, original=None, selected=None):
    data, matches = settings_check(agent, package, home)
    if len(matches) != 1 or not isinstance(matches[0], dict) or matches[0].get('source') != str(package):
        raise ValueError('Real Pi settings do not register exactly this local Mirket package')
    if selected is not None and matches[0].get('skills') != selected:
        raise ValueError('Real Pi settings differ from the selected skill filter')
    if original is not None:
        before = json.loads(original) if original else {}
        after = json.loads(data)
        def unrelated(settings):
            settings = dict(settings)
            packages = settings.pop('packages', [])
            kept = []
            for entry in packages:
                source = entry if isinstance(entry, str) else entry['source']
                source = source.strip()
                if re.match(r'^[a-z]+:|^git@', source, re.I):
                    kept.append(entry)
                    continue
                local = home if source == '~' else home / source[2:] if source.startswith('~/') else agent / source
                if local.resolve() != package:
                    kept.append(entry)
            return settings, kept
        if unrelated(before) != unrelated(after):
            raise ValueError('Unrelated Pi settings changed during setup; preserved for inspection')


def atomic_write(path, data, mode=0o600, exclusive=False):
    fd, temporary = tempfile.mkstemp(prefix='.' + path.name + '-', dir=path.parent)
    try:
        with os.fdopen(fd, 'wb') as target:
            os.fchmod(target.fileno(), mode)
            target.write(data)
            target.flush()
            os.fsync(target.fileno())
        if exclusive:
            # A concurrently created launcher must never be replaced.
            os.link(temporary, path)
        else:
            os.replace(temporary, path)
    finally:
        if os.path.lexists(temporary):
            os.unlink(temporary)


def make_parents(path):
    canonical(path, exists=False)
    missing = []
    parent = path
    while not parent.exists():
        missing.append(parent)
        parent = parent.parent
    directory(parent)
    for item in reversed(missing):
        item.mkdir(mode=0o700)
    directory(path)


def launcher_snapshot(path):
    if not os.path.lexists(path):
        return None
    info = path.lstat()
    identity = (info.st_dev, info.st_ino, info.st_mode, info.st_size,
                info.st_mtime_ns, info.st_ctime_ns, info.st_nlink)
    if stat.S_ISLNK(info.st_mode):
        return identity, os.readlink(path)
    return identity, sha(regular(path, 65536))


@contextmanager
def launcher_lock(launcher):
    make_parents(launcher.parent)
    lock = launcher.with_name('.pi.mirket-install.lock')
    try:
        lock.mkdir(mode=0o700)
    except FileExistsError as error:
        raise ValueError('Pi launcher is busy; existing cooperative install lock is preserved') from error
    try:
        yield
    finally:
        lock.rmdir()


def completed_setup_backup(root, agent, package, host, old_launcher):
    backups = root / 'install-backups'
    if not backups.exists():
        return None
    directory(backups, private=True)
    matches = []
    for backup in backups.iterdir():
        directory(backup, private=True)
        record = json.loads(regular(backup / 'record.json', 65536))
        if (record.get('schema') == 1 and record.get('phase') == 'setup-complete'
                and record.get('agentDir') == str(agent) and record.get('package') == str(package)
                and record.get('host') == str(host) and record.get('launcher') == old_launcher):
            matches.append((backup, record))
    if len(matches) > 1:
        raise ValueError('Multiple completed setup records require explicit inspection')
    return matches[0] if matches else None


def install(package, host, install_root=None):
    home = Path(os.environ['HOME'])
    canonical(home)
    directory(home)
    root = install_root if install_root is not None else home / '.local/share/mirket'
    canonical(root)
    directory(root, private=True)
    for path in (package, host):
        canonical(path)
        if path == root or root not in path.parents:
            raise ValueError('Package and host must be permanent paths inside --install-root')
    standard = home / '.pi/agent'
    existing = standard.exists() or standard.is_symlink()
    agent = standard if existing else root / 'agent'
    canonical(agent, exists=existing)
    launcher = home / '.local/bin/pi'
    canonical(launcher.parent, exists=False)
    if os.environ.get('PI_CODING_AGENT_DIR') not in (None, '', str(agent)):
        raise ValueError('PI_CODING_AGENT_DIR selects a different agent; unset it before installation')
    if existing:
        directory(agent)
        if stat.S_IMODE(agent.stat().st_mode) not in (0o700, 0o755):
            raise ValueError('Existing Pi agent must have mode 0700 or 0755')
    elif agent.exists():
        directory(agent, private=True)
    data, matches = settings_check(agent, package, home) if agent.exists() else (None, [])
    entry = host / 'node_modules' / ENTRY
    node = package / 'native/runtime/node/bin/node'
    desired = wrapper(agent, node, entry)
    initial_launcher = launcher_snapshot(launcher)
    old_launcher = None
    if launcher.is_symlink():
        if not existing or os.readlink(launcher) != MANAGED_LINK:
            raise ValueError('Unrelated Pi launcher is preserved')
        managed_layout(agent, launcher)
        old_launcher = {'symlink': MANAGED_LINK}
    elif launcher.exists():
        content = regular(launcher, 65536)
        if content not in (desired, legacy_wrapper(agent, node, entry)):
            raise ValueError('Existing unrelated Pi launcher is preserved')
        if not launcher.stat().st_mode & stat.S_IXUSR or not os.access(launcher, os.X_OK):
            raise ValueError('Existing bound Pi launcher is not executable; preserved for inspection')
        old_launcher = {'sha256': sha(content)}
    elif existing:
        raise ValueError('Existing Pi agent has no recognized outer launcher')
    recovery = None
    if (agent / 'mirket').exists() or (agent / 'mirket').is_symlink():
        directory(agent / 'mirket', private=True)
        complete = bool(matches and old_launcher is not None and 'sha256' in old_launcher)
        if not complete and matches:
            recovery = completed_setup_backup(root, agent, package, host, old_launcher)
        if not complete and recovery is None:
            raise ValueError('Existing Mirket home is preserved; inspect it before installation')
    else:
        complete = False
    if not complete and old_launcher and 'sha256' in old_launcher:
        raise ValueError('Existing bound Pi launcher has no complete Mirket home; preserved for inspection')
    if launcher_snapshot(launcher) != initial_launcher:
        raise ValueError('Pi launcher changed during preflight; preserved')
    migrate_launcher = complete and old_launcher['sha256'] != sha(desired)
    # Native verification runs with a disposable HOME before install mutation.
    with tempfile.TemporaryDirectory(prefix='mirket-install-check-', dir='/private/tmp' if sys.platform == 'darwin' else '/tmp') as temporary:
        probe = Path(temporary).resolve()
        env = {**os.environ, 'HOME': str(probe), 'TMPDIR': str(probe), 'PATH': '/nonexistent',
               'PI_CODING_AGENT_DIR': str(agent), 'PI_CODING_AGENT_SESSION_DIR': str(probe / 'sessions'),
               'PI_SKIP_UPDATE_CHECK': '1', 'PI_TELEMETRY_DISABLED': '1'}
        binding, binary, node = verify(package, host, env, probe)
        backup, record = recovery if recovery else (None, None)
        selected = None
        if not complete and recovery is None:
            backups = root / 'install-backups'
            make_parents(backups)
            directory(backups, private=True)
            backup = Path(tempfile.mkdtemp(prefix='install-', dir=backups))
            record = {'schema': 1, 'agentDir': str(agent), 'package': str(package), 'host': str(host),
                      'agentMode': stat.S_IMODE(agent.stat().st_mode) if agent.exists() else None,
                      'launcher': old_launcher, 'settingsPresent': data is not None, 'phase': 'backed-up'}
            if data is not None:
                atomic_write(backup / 'settings.json', data)
                record['settingsMode'] = stat.S_IMODE((agent / 'settings.json').stat().st_mode)
            if old_launcher and 'symlink' in old_launcher:
                (backup / 'pi.original-link').symlink_to(old_launcher['symlink'])
            atomic_write(backup / 'record.json', json.dumps(record, indent=2).encode() + b'\n')
            try:
                if not agent.exists():
                    agent.mkdir(mode=0o700)
                else:
                    agent.chmod(0o700)
                # Profile paths such as ~/packages/foo belong to the target
                # user's HOME; the setup script does not load auth or sessions.
                result = run_json([node, package / 'scripts/setup.mjs', '--agent-dir', agent], probe, {**env, 'HOME': str(home)}, timeout=660)
                if result.get('releaseId') != binding['releaseId'] or result.get('home') != str(agent / 'mirket') or result.get('packageSource') != str(package) + '/':
                    # setup uses fileURLToPath(new URL('../',...)), with trailing slash.
                    if result.get('packageSource') != str(package) or result.get('releaseId') != binding['releaseId'] or result.get('home') != str(agent / 'mirket'):
                        raise ValueError('Setup response differs from selected package')
                selected = result.get('selectedSkills')
                if not isinstance(selected, list) or not all(isinstance(s, str) for s in selected):
                    raise ValueError('Setup did not report its selected skill filter')
                record['phase'] = 'setup-complete'
                atomic_write(backup / 'record.json', json.dumps(record, indent=2).encode() + b'\n')
            except BaseException:
                record['phase'] = 'setup-failed-state-preserved'
                atomic_write(backup / 'record.json', json.dumps(record, indent=2).encode() + b'\n')
                raise
        registration_check(agent, package, home, original=(data or b'{}') if not complete else None, selected=selected)
        installed = agent / 'mirket/bin/mirket'
        if sha(regular(installed)) != binding['coreSha256']:
            raise ValueError('Installed native launcher differs from package')
        doctor = run_json([installed, 'doctor', '--home', agent / 'mirket'], probe, env)
        if doctor.get('status') != 'healthy' or doctor.get('active_release_id') != binding['releaseId']:
            raise ValueError('Native doctor did not establish selected release health')
        ready = run_json([node, '--input-type=module', '--eval', READINESS, package, host, agent, probe, binding['releaseId']], probe, env)
        if ready != {'status': 'registered', 'releaseId': binding['releaseId']}:
            raise ValueError('Pi registration readiness differs')
        registration_check(agent, package, home, original=(data or b'{}') if not complete else None, selected=selected)
        with launcher_lock(launcher):
            if launcher_snapshot(launcher) != initial_launcher:
                raise ValueError('Pi launcher changed during verification or setup; preserved')
            if migrate_launcher:
                backups = root / 'install-backups'
                make_parents(backups)
                directory(backups, private=True)
                backup = Path(tempfile.mkdtemp(prefix='launcher-', dir=backups))
                atomic_write(backup / 'pi.original', content)
                record = {'schema': 1, 'agentDir': str(agent), 'package': str(package), 'host': str(host),
                          'launcher': old_launcher, 'launcherMode': stat.S_IMODE(launcher.stat().st_mode),
                          'phase': 'launcher-migration-backed-up'}
                atomic_write(backup / 'record.json', json.dumps(record, indent=2).encode() + b'\n')
                if launcher_snapshot(launcher) != initial_launcher:
                    raise ValueError('Pi launcher changed during backup; preserved')
            if not complete or migrate_launcher:
                # The lock excludes cooperating installer publications, and
                # snapshots reject observed drift. Replacing an existing link
                # or legacy wrapper is not an OS compare-and-swap: an arbitrary
                # non-cooperating same-user writer can still race between this
                # check and os.replace. Mirket processes are not a sandbox.
                # Fresh publication is exclusive even against such a writer.
                atomic_write(launcher, desired, 0o700, exclusive=initial_launcher is None)
                record['phase'] = 'complete'
                atomic_write(backup / 'record.json', json.dumps(record, indent=2).encode() + b'\n')
        return {'status': 'launcher-updated' if migrate_launcher else 'already-installed' if complete else 'installed', 'agentDir': str(agent),
                'launcher': str(launcher), 'package': str(package), 'host': str(host),
                'releaseId': binding['releaseId'], 'backup': str(backup) if backup else None}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--host', type=Path, required=True, help='Permanent acquisition output containing node_modules')
    parser.add_argument('--install-root', type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(install(args.package, args.host, args.install_root)))
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print('Mirket installation refused or failed: ' + str(error)
              + '\nExisting state and any private install-backups were preserved; inspect them before recovery.', file=sys.stderr)
        sys.exit(1)
