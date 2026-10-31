#!/usr/bin/env python3
"""Explicit Mirket package-path upgrade for the qualified macOS installation.

Verify this helper and the installer against release SHA256SUMS before use.
All selected package/host paths must be permanent. No downloads, host discovery,
provider requests or automatic rollback run. A failure
can leave the native release updated while the outer Pi configuration is old;
the private record describes the last attempted phase for explicit recovery.
"""
import argparse
from contextlib import ExitStack, contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
import tempfile
import threading
import time
import types


def load_installer(path, expected):
    if not re.fullmatch(r'[a-f0-9]{64}', expected):
        raise ValueError('Select an exact SHA-256 for the verified installer')
    if not path.is_absolute() or path.resolve() != path or any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError('Installer path must be canonical and absolute')
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or info.st_size > 1024 * 1024:
            raise ValueError('Invalid installer file')
        with os.fdopen(fd, 'rb', closefd=False) as source:
            data = source.read(1024 * 1024 + 1)
    finally:
        os.close(fd)
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError('Installer differs from the independently verified release checksum')
    module = types.ModuleType('verified_install_pi')
    module.__file__ = str(path)
    # Execute the verified source bytes, without trusting an adjacent bytecode cache.
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError('Duplicate JSON object key requires inspection: ' + key)
        value[key] = item
    return value


@contextmanager
def existing_lock_idle(installer, path, required=False, empty=False):
    if not path.exists() and not path.is_symlink():
        if required:
            raise ValueError('Required native runtime lock is absent')
        yield
        return
    installer.canonical(path)
    fd = os.open(path, os.O_RDWR | os.O_NOFOLLOW | os.O_NONBLOCK)
    try:
        info = os.fstat(fd)
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
                or info.st_nlink != 1 or info.st_mode & 0o077 or (empty and info.st_size != 0)):
            raise ValueError('Invalid native lock: ' + str(path))
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        current = path.lstat()
        if (info.st_dev, info.st_ino) != (current.st_dev, current.st_ino):
            raise ValueError('Native lock changed during preflight')
        yield
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)


@contextmanager
def settings_lock(path):
    # Pi's proper-lockfile reclaims directories older than ten seconds. Keep
    # this cooperative lock fresh while native verification/update is running.
    stopped = threading.Event()
    failures = []
    identity = fd = worker = primary_error = None

    def owned():
        try:
            current = path.lstat()
        except OSError as error:
            raise ValueError('Pi settings lock ownership lost; inspect preserved state') from error
        if (not stat.S_ISDIR(current.st_mode)
                or (current.st_dev, current.st_ino) != (identity.st_dev, identity.st_ino)
                or current.st_uid != os.getuid() or current.st_mode & 0o077):
            raise ValueError('Pi settings lock ownership lost; replacement preserved')
        return current

    def check():
        current = owned()
        if failures:
            raise ValueError('Pi settings lock heartbeat failed; inspect preserved state') from failures[0]
        if current.st_mtime < time.time() - 5:
            raise ValueError('Pi settings lock renewal expired; inspect preserved state')

    def renew():
        while not stopped.wait(1):
            try:
                check()
                # Updating the held inode cannot touch a replacement directory.
                os.utime(fd, None)
            except BaseException as error:
                failures.append(error)
                return

    path.mkdir(mode=0o700)
    try:
        identity = path.lstat()
        fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        opened = os.fstat(fd)
        if (opened.st_dev, opened.st_ino) != (identity.st_dev, identity.st_ino):
            raise ValueError('Pi settings lock changed during acquisition')
        check()
        worker = threading.Thread(target=renew, name='pi-settings-lock', daemon=True)
        worker.start()
        yield check
        check()
    except BaseException as error:
        primary_error = error
        raise
    finally:
        stopped.set()
        try:
            try:
                if worker is not None and worker.ident is not None:
                    worker.join()
                if identity is not None:
                    owned()
                    path.rmdir()
            finally:
                if fd is not None:
                    os.close(fd)
        except BaseException as cleanup_error:
            if primary_error is None:
                raise
            raise primary_error from cleanup_error


def selected_projects(installer, projects, old_package, package, home, agent):
    """Prepare only explicitly named project settings; never discover projects."""
    if len(projects) > 32 or len(set(projects)) != len(projects):
        raise ValueError('Select at most 32 distinct project roots')
    changes = []
    for project in projects:
        installer.canonical(project)
        installer.directory(project)
        directory = project / '.pi'
        installer.canonical(directory)
        installer.directory(directory)
        if directory == agent:
            raise ValueError('Project settings must differ from global agent settings')
        original, matches = installer.settings_check(directory, old_package, home)
        if original is None or len(matches) != 1 or not isinstance(matches[0], dict):
            raise ValueError('Selected project must contain exactly one old Mirket package object')
        settings = json.loads(original, object_pairs_hook=unique_object)
        index = next(i for i, entry in enumerate(settings['packages']) if entry == matches[0])
        settings['packages'][index] = dict(matches[0], source=str(package))
        file = directory / 'settings.json'
        changes.append({'file': file, 'original': original,
                        'expected': json.dumps(settings, indent=2).encode() + b'\n',
                        'mode': stat.S_IMODE(file.stat().st_mode)})
    return changes


def migrate(args):
    installer = load_installer(args.installer, args.installer_sha256)
    home = Path(os.environ['HOME'])
    root, old_package, old_host = args.install_root, args.old_package, args.old_host
    package, host = args.package, args.host
    for path in (home, root, old_package, old_host, package, host):
        installer.canonical(path)
    installer.directory(home)
    installer.directory(root, private=True)
    for path in (old_package, old_host, package, host):
        if root not in path.parents:
            raise ValueError('All package and host paths must be permanent children of the install root')
    if old_package == package:
        raise ValueError('Use separate permanent old and new package paths')
    for release in (args.old_release_id, args.release_id):
        if not re.fullmatch(r'sha256:[a-f0-9]{64}', release):
            raise ValueError('Select exact old and new native release IDs')
    standard = home / '.pi/agent'
    agent = standard if standard.exists() or standard.is_symlink() else root / 'agent'
    installer.directory(agent, private=True)
    native_home = agent / 'mirket'
    installer.directory(native_home, private=True)
    installer.directory(native_home / 'state', private=True)
    if os.environ.get('PI_CODING_AGENT_DIR') not in (None, '', str(agent)):
        raise ValueError('PI_CODING_AGENT_DIR selects a different agent')
    launcher = home / '.local/bin/pi'
    installer.directory(launcher.parent)
    settings_file = agent / 'settings.json'
    original, matches = installer.settings_check(agent, old_package, home)
    if original is None or len(matches) != 1 or not isinstance(matches[0], dict) or matches[0].get('source') != str(old_package):
        raise ValueError('Settings must contain exactly the selected old local Mirket package object')
    settings = json.loads(original, object_pairs_hook=unique_object)
    selected_index = next(index for index, item in enumerate(settings['packages']) if item == matches[0])
    replacement = dict(matches[0], source=str(package))
    settings['packages'][selected_index] = replacement
    expected_settings = json.dumps(settings, indent=2).encode() + b'\n'
    settings_mode = stat.S_IMODE(settings_file.stat().st_mode)
    projects = selected_projects(installer, getattr(args, 'project', []), old_package, package, home, agent)
    content = installer.regular(launcher, 65536)
    old_node = old_package / 'native/runtime/node/bin/node'
    old_entry = old_host / 'node_modules' / installer.ENTRY
    if content not in (installer.wrapper(agent, old_node, old_entry), installer.legacy_wrapper(agent, old_node, old_entry)):
        raise ValueError('Outer launcher is not the exact selected old bound/legacy Mirket wrapper')
    if not os.access(launcher, os.X_OK):
        raise ValueError('Old outer launcher is not executable')
    launcher_before = installer.launcher_snapshot(launcher)
    active_file = native_home / 'active.json'
    active_before = installer.regular(active_file, 65536)
    active = json.loads(active_before)
    if active.get('release_id') != args.old_release_id:
        raise ValueError('Native active pointer differs from selected old release')
    for name in ('staged.json', 'state/activation.pending.json', 'state/launcher.previous',
                 'state/staging.pending.json', 'state/staging.pending.json.next'):
        candidate = native_home / name
        if candidate.exists() or candidate.is_symlink():
            raise ValueError('Pending native maintenance state requires inspection: ' + name)
    backup = None
    record = {'schema': 1, 'phase': 'preflight', 'agentDir': str(agent), 'oldPackage': str(old_package),
              'package': str(package), 'oldHost': str(old_host), 'host': str(host),
              'oldReleaseId': args.old_release_id, 'releaseId': args.release_id,
              'settingsMode': settings_mode, 'launcherMode': stat.S_IMODE(launcher.stat().st_mode),
              'installerSha256': args.installer_sha256,
              'projects': [{'settings': str(change['file']), 'mode': change['mode'],
                            'backup': f'project-settings-{index}.json',
                            'beforeSha256': hashlib.sha256(change['original']).hexdigest(),
                            'afterSha256': hashlib.sha256(change['expected']).hexdigest()}
                           for index, change in enumerate(projects)]}

    def phase(name):
        record['phase'] = name
        if backup is not None:
            installer.atomic_write(backup / 'record.json', json.dumps(record, indent=2).encode() + b'\n')

    try:
        with tempfile.TemporaryDirectory(prefix='mirket-migration-check-', dir='/private/tmp') as temporary:
            probe = Path(temporary).resolve()
            env = {**os.environ, 'HOME': str(probe), 'TMPDIR': str(probe), 'PATH': '/nonexistent',
                   'PI_CODING_AGENT_DIR': str(agent), 'PI_CODING_AGENT_SESSION_DIR': str(probe / 'sessions'),
                   'PI_SKIP_UPDATE_CHECK': '1', 'PI_TELEMETRY_DISABLED': '1'}
            old_binding, old_binary, _ = installer.verify(old_package, old_host, env, probe)
            binding, binary, node = installer.verify(package, host, env, probe)
            if old_binding['releaseId'] != args.old_release_id or binding['releaseId'] != args.release_id:
                raise ValueError('Verified package release IDs differ from explicit selections')
            same_native = args.old_release_id == args.release_id
            if same_native and old_binding['coreSha256'] != binding['coreSha256']:
                raise ValueError('Identical native release IDs require identical verified core identity')
            installed = native_home / 'bin/mirket'
            if installer.sha(installer.regular(installed)) != old_binding['coreSha256']:
                raise ValueError('Installed native launcher differs from selected old package')
            old_health = installer.run_json([old_binary, 'doctor', '--home', native_home], probe, env)
            if old_health.get('status') != 'healthy' or old_health.get('active_release_id') != args.old_release_id:
                raise ValueError('Selected old native doctor did not establish installation health')
            desired = installer.wrapper(agent, node, host / 'node_modules' / installer.ENTRY)
            with installer.launcher_lock(launcher):
                with settings_lock(settings_file.with_name('settings.json.lock')) as check_lock, ExitStack() as project_locks:
                    project_checks = [project_locks.enter_context(settings_lock(change['file'].with_name('settings.json.lock')))
                                      for change in projects]

                    def projects_unchanged(published=False):
                        for change, check in zip(projects, project_checks):
                            check()
                            expected = change['expected'] if published else change['original']
                            if (installer.regular(change['file'], 2 * 1024 * 1024) != expected
                                    or stat.S_IMODE(change['file'].stat().st_mode) != change['mode']):
                                raise ValueError('Selected project settings changed; preserved state requires inspection')

                    def outer_unchanged():
                        projects_unchanged()
                        check_lock()
                        if installer.regular(settings_file, 2 * 1024 * 1024) != original or installer.launcher_snapshot(launcher) != launcher_before:
                            raise ValueError('Original Pi settings or outer launcher changed; preserved')

                    outer_unchanged()
                    if installer.regular(active_file, 65536) != active_before:
                        raise ValueError('Native active pointer changed during preflight')
                    with existing_lock_idle(installer, native_home / 'state/runtime.lock', required=True, empty=True), existing_lock_idle(installer, native_home / 'update.lock'):
                        for name in ('heavy-slot-0.unresolved', 'heavy-slot-1.unresolved'):
                            marker = native_home / 'state' / name
                            if marker.exists() or marker.is_symlink():
                                raise ValueError('Unresolved native ownership requires inspection')
                        backups = root / 'migration-backups'
                        installer.make_parents(backups)
                        installer.directory(backups, private=True)
                        backup = Path(tempfile.mkdtemp(prefix='migration-', dir=backups))
                        installer.atomic_write(backup / 'settings.json', original)
                        installer.atomic_write(backup / 'pi.original', content)
                        installer.atomic_write(backup / 'active.json', active_before)
                        for index, change in enumerate(projects):
                            installer.atomic_write(backup / f'project-settings-{index}.json', change['original'])
                        phase('backed-up')
                    if same_native:
                        record['nativeUpdate'] = {'status': 'unchanged', 'release_id': args.release_id}
                        phase('native-unchanged-outer-old')
                    else:
                        # The native command takes its own update and maintenance locks.
                        phase('native-update-started')
                        result = installer.run_json([binary, 'update', '--home', native_home, '--source', package / 'native'], probe, env, timeout=660)
                        record['nativeUpdate'] = result
                        if result.get('activation', {}).get('status') != 'active' or result['activation'].get('release_id') != args.release_id:
                            phase('native-not-active-outer-preserved')
                            raise ValueError('Native update did not activate the selected release; outer configuration preserved')
                        phase('native-active-outer-old')
                    outer_unchanged()
                    if installer.sha(installer.regular(installed)) != binding['coreSha256']:
                        raise ValueError('New installed native launcher differs from package')
                    health = installer.run_json([installed, 'doctor', '--home', native_home], probe, env)
                    if health.get('status') != 'healthy' or health.get('active_release_id') != args.release_id:
                        raise ValueError('New native doctor did not establish selected release health')
                    phase('settings-publication-started')
                    check_lock()
                    installer.atomic_write(settings_file, expected_settings, settings_mode)
                    phase('settings-new-launcher-old')
                    # This check avoids the install helper's busy-settings preflight
                    # while this script owns Pi's cooperative settings lock.
                    if installer.regular(settings_file, 2 * 1024 * 1024) != expected_settings:
                        raise ValueError('Settings changed after publication')
                    ready = installer.run_json([node, '--input-type=module', '--eval', installer.READINESS, package, host, agent, probe, binding['releaseId']], probe, env)
                    if ready != {'status': 'registered', 'releaseId': args.release_id}:
                        raise ValueError('New Pi package/MCP registration readiness differs')
                    check_lock()
                    if installer.regular(settings_file, 2 * 1024 * 1024) != expected_settings or installer.launcher_snapshot(launcher) != launcher_before:
                        raise ValueError('Settings or old launcher drifted before launcher publication')
                    projects_unchanged()
                    for index, (change, check) in enumerate(zip(projects, project_checks)):
                        phase(f'project-settings-{index}-publication-started')
                        check_lock()
                        check()
                        if installer.regular(change['file'], 2 * 1024 * 1024) != change['original']:
                            raise ValueError('Selected project settings changed before publication')
                        installer.atomic_write(change['file'], change['expected'], change['mode'])
                        phase(f'project-settings-{index}-published')
                    projects_unchanged(published=True)
                    phase('launcher-publication-started')
                    check_lock()
                    installer.atomic_write(launcher, desired, 0o700)
                    phase('outer-new-final-check-pending')
            final = installer.install(package, host, root)
            if final.get('status') != 'already-installed' or final.get('releaseId') != args.release_id:
                raise ValueError('Final exact installer rerun did not report already-installed')
            if installer.regular(settings_file, 2 * 1024 * 1024) != expected_settings or installer.regular(launcher, 65536) != desired:
                raise ValueError('Outer configuration changed during final installer check')
            for change in projects:
                if (installer.regular(change['file'], 2 * 1024 * 1024) != change['expected']
                        or stat.S_IMODE(change['file'].stat().st_mode) != change['mode']):
                    raise ValueError('Selected project settings changed during final installer check')
            record['finalInstaller'] = final
            phase('complete')
            return {'status': 'updated', 'releaseId': args.release_id, 'backup': str(backup),
                    'agentDir': str(agent), 'package': str(package), 'host': str(host),
                    'projects': [str(change['file'].parent.parent) for change in projects],
                    'projectNextStep': 'Reload Pi and run /mirket-project init in each selected project to renew release-bound consent',
                    'runtimeSelections': 'preserved; native release unchanged' if same_native else
                        'preserved; release-bound optional tools require explicit /mirket setup'}
    except BaseException as error:
        record['error'] = str(error)
        record['failedPhase'] = record['phase']
        try:
            phase('failed-state-preserved')
        except BaseException:
            pass
        print(json.dumps({'status': 'failed-state-preserved', 'phase': record.get('failedPhase'),
                          'backup': str(backup) if backup else None, 'error': str(error),
                          'recovery': 'Inspect exact native active state, settings and launcher; no rollback was attempted.'}), file=sys.stderr)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('installer', 'old-package', 'old-host', 'package', 'host', 'install-root'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--project', type=Path, action='append', default=[],
                        help='Explicit project root whose old package object should be migrated; repeat for selected projects only')
    for name in ('installer-sha256', 'old-release-id', 'release-id'):
        parser.add_argument('--' + name, required=True)
    try:
        print(json.dumps(migrate(parser.parse_args())))
    except Exception as error:
        print('Mirket migration refused or failed: ' + str(error), file=sys.stderr)
        sys.exit(1)
