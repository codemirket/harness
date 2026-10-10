"""Clean-checkout, fast-forward-only maintenance; no models, login or job recursion."""
import argparse
import datetime
import json
import os
from pathlib import Path
import re
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import threading
import time
import uuid
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[1]
MAX_CAPTURE = 1024 * 1024
MAX_LOG = 256 * 1024
GIT_TIMEOUT = 120
SYNC_TIMEOUT = 300
ALLOWED_PROTOCOLS = 'https:ssh'
OPERATIONS = ('MERGE_HEAD', 'CHERRY_PICK_HEAD', 'REVERT_HEAD', 'rebase-merge',
              'rebase-apply', 'sequencer', 'BISECT_START')


class MaintenanceError(ValueError):
    def __init__(self, status, reason):
        super().__init__(reason)
        self.status = status


def _report(status, code=1, **fields):
    return dict(schema_version=1, status=status, exit_code=code,
                checked_at=datetime.datetime.now(datetime.timezone.utc).isoformat(), **fields)


def _safe_path(path, directory=False):
    for item in (path,) + tuple(path.parents):
        try:
            info = item.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(info.st_mode) or getattr(info, 'st_file_attributes', 0) & 0x400:
            raise MaintenanceError('blocked_storage', 'A maintenance path is a symlink or junction.')
        if item != path and not stat.S_ISDIR(info.st_mode):
            raise MaintenanceError('blocked_storage', 'A maintenance parent is not a directory.')
        if item == path:
            expected = stat.S_ISDIR if directory else stat.S_ISREG
            if not expected(info.st_mode) or (not directory and info.st_nlink != 1):
                raise MaintenanceError('blocked_storage', 'A maintenance path has an unsafe file type or hard link.')
            if hasattr(os, 'geteuid') and info.st_uid != os.geteuid():
                raise MaintenanceError('blocked_storage', 'A maintenance path belongs to another user.')


def _storage(home=None, create=False):
    base = Path(home).expanduser() if home is not None else Path.home()
    # Resolve platform aliases (/var, /tmp) without accepting a symlinked home itself.
    if base.is_symlink():
        raise MaintenanceError('blocked_storage', 'The target home is a symlink.')
    base = base.resolve(strict=True)
    if not base.is_dir():
        raise MaintenanceError('blocked_storage', 'The target home must be an existing directory.')
    folder = base / '.agent-harness'
    logs = folder / 'logs'
    for path in (folder, logs):
        _safe_path(path, directory=True)
        if create:
            path.mkdir(mode=0o700, exist_ok=True)
            path.chmod(0o700)
    for path in (folder / 'maintenance.lock', folder / 'maintenance-status.json',
                 logs / 'maintenance.log', logs / 'maintenance.log.1'):
        _safe_path(path)
    return base, folder, logs


def _atomic(path, raw):
    _safe_path(path)
    with tempfile.NamedTemporaryFile(prefix='.maintenance-', dir=str(path.parent), delete=False) as stream:
        temporary = Path(stream.name)
        try:
            os.chmod(temporary, 0o600)
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    try:
        _safe_path(path)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def _record(folder, logs, result):
    status_path = folder / 'maintenance-status.json'
    log = logs / 'maintenance.log'
    backup = logs / 'maintenance.log.1'
    for path in (status_path, log, backup):
        _safe_path(path)
    raw = (json.dumps(result, sort_keys=True, separators=(',', ':')) + '\n').encode('utf-8')
    if len(raw) > MAX_LOG:
        raise MaintenanceError('blocked_storage', 'Maintenance report exceeded the log bound.')
    old = b''
    if log.exists():
        with log.open('rb') as stream:
            old = stream.read(MAX_LOG + 1)
    if len(old) + len(raw) > MAX_LOG:
        # The retained previous log is capped too, including preexisting oversized logs.
        _atomic(backup, old[-MAX_LOG:])
        old = b''
    _atomic(log, old + raw)
    _atomic(status_path, (json.dumps(result, indent=2) + '\n').encode('utf-8'))


class _Lock:
    """Kernel ownership, not age/PID guessing; stale files are safe to reuse."""
    def __init__(self, path):
        self.path, self.descriptor = path, None
        self.token = uuid.uuid4().hex

    def acquire(self):
        _safe_path(self.path)
        flags = os.O_RDWR | os.O_CREAT | getattr(os, 'O_NOFOLLOW', 0) | getattr(os, 'O_BINARY', 0)
        descriptor = os.open(str(self.path), flags, 0o600)
        self.descriptor = descriptor
        try:
            info = os.fstat(descriptor)
            current = self.path.lstat()
            if (info.st_dev, info.st_ino) != (current.st_dev, current.st_ino) or info.st_nlink != 1:
                raise MaintenanceError('blocked_storage', 'Maintenance lock changed during acquisition.')
            if os.name == 'nt':
                import msvcrt
                if info.st_size == 0:
                    os.write(descriptor, b'\n')
                os.lseek(descriptor, 0, os.SEEK_SET)
                try:
                    msvcrt.locking(descriptor, msvcrt.LK_NBLCK, 1)
                except OSError:
                    os.close(descriptor); self.descriptor = None
                    return False
            else:
                import fcntl
                try:
                    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except BlockingIOError:
                    os.close(descriptor); self.descriptor = None
                    return False
            os.chmod(self.path, 0o600)
            os.lseek(descriptor, 0, os.SEEK_SET)
            os.ftruncate(descriptor, 0)
            os.write(descriptor, json.dumps({'pid': os.getpid(), 'locked_at': time.time(), 'token': self.token}).encode('ascii'))
            os.fsync(descriptor)
            return True
        except BaseException:
            if self.descriptor is not None:
                os.close(descriptor); self.descriptor = None
            raise

    def close(self):
        if self.descriptor is not None:
            # Closing releases the OS lock. Never unlink: that would allow two owners
            # to lock different inodes under the same filename.
            os.close(self.descriptor)
            self.descriptor = None


def _stop(process):
    if os.name == 'nt':
        taskkill = shutil.which('taskkill')
        if taskkill:
            try:
                subprocess.run([taskkill, '/PID', str(process.pid), '/T', '/F'],
                               stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10)
            except (OSError, subprocess.TimeoutExpired):
                pass
        if process.poll() is None:
            process.kill()
    else:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            if process.poll() is None:
                process.kill()
    process.wait(timeout=10)


def _process(argv, *, root, env, timeout):
    options = {'start_new_session': True} if os.name != 'nt' else {
        'creationflags': subprocess.CREATE_NEW_PROCESS_GROUP}
    process = subprocess.Popen(argv, cwd=str(root), env=env, stdin=subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, shell=False, **options)
    output, exceeded = bytearray(), threading.Event()

    def read():
        try:
            while True:
                chunk = process.stdout.read(4096)
                if not chunk:
                    break
                remaining = max(0, MAX_CAPTURE - len(output))
                output.extend(chunk[:remaining])
                if len(chunk) > remaining:
                    exceeded.set()
                    break
        finally:
            process.stdout.close()

    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    previous = None
    main_thread = threading.current_thread() is threading.main_thread()
    if main_thread:
        def terminate(signum, frame):
            raise KeyboardInterrupt
        previous = signal.signal(signal.SIGTERM, terminate)
    try:
        deadline = time.monotonic() + timeout
        while process.poll() is None or reader.is_alive():
            if exceeded.is_set() or time.monotonic() >= deadline:
                _stop(process)
                return None, b''
            time.sleep(0.01)
        if exceeded.is_set():
            return None, b''
        return process.returncode, bytes(output)
    except BaseException:
        _stop(process)
        raise
    finally:
        if main_thread:
            signal.signal(signal.SIGTERM, previous)
        reader.join(timeout=2)


def _environment():
    # Preserve effective credential helpers, CA/SSH configuration and host auth.
    # Exclude repository-location overrides so --root remains the checkout boundary.
    excluded = {'GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'GIT_COMMON_DIR',
                'GIT_OBJECT_DIRECTORY', 'GIT_ALTERNATE_OBJECT_DIRECTORIES',
                'AI_HARNESS_MAINTENANCE_TOKEN'}
    env = {key: value for key, value in os.environ.items()
           if key not in excluded and not key.startswith('PYTHON')}
    env.update({'GIT_TERMINAL_PROMPT': '0', 'GIT_OPTIONAL_LOCKS': '0',
                'GIT_ALLOW_PROTOCOL': ALLOWED_PROTOCOLS, 'GCM_INTERACTIVE': 'never',
                'SSH_ASKPASS_REQUIRE': 'never', 'DISABLE_AUTOUPDATER': '1', 'DISABLE_UPDATES': '1'})
    return env


class _Git:
    def __init__(self, root, hooks, env):
        self.root, self.env = root, env
        binary = shutil.which('git')
        if not binary:
            raise MaintenanceError('blocked_repository', 'Git is unavailable.')
        self.argv = [binary, '-c', 'core.hooksPath=' + str(hooks), '-c', 'core.fsmonitor=false',
                     '-c', 'core.untrackedCache=false', '-c', 'merge.autoStash=false',
                     '-c', 'submodule.recurse=false', '-c', 'fetch.recurseSubmodules=false',
                     '-c', 'gc.auto=0', '-c', 'maintenance.auto=false',
                     '-c', 'commit.gpgSign=false']
        self.check_filters()

    def call(self, *args, allowed=(0,), failure='blocked_repository', timeout=GIT_TIMEOUT):
        code, raw = _process(self.argv + list(args), root=self.root, env=self.env, timeout=timeout)
        if code not in allowed:
            raise MaintenanceError(failure, 'Git operation failed or exceeded its time/output bound; no raw diagnostics were retained.')
        return code, raw

    def text(self, *args, **kwargs):
        return self.call(*args, **kwargs)[1].decode('utf-8', errors='strict').strip()

    def check_filters(self):
        code, _ = self.call('config', '--get-regexp', r'^filter\..*\.(process|smudge|clean|required)$', allowed=(0, 1))
        if code == 0:
            raise MaintenanceError('blocked_repository', 'Git filters require manual review before unattended worktree checks.')


def _remote_matches(remote, expected):
    if not isinstance(expected, str) or not re.fullmatch(r'github\.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', expected):
        return False
    match = re.fullmatch(r'git@([^:/]+):([^?#]+)', remote)
    if match:
        host, path = match.groups()
    else:
        try:
            url = urlsplit(remote)
            if url.scheme not in ('https', 'ssh') or url.query or url.fragment or url.port not in (None, 443 if url.scheme == 'https' else 22):
                return False
            if url.scheme == 'ssh' and url.username != 'git':
                return False
            host, path = url.hostname, url.path.lstrip('/')
        except ValueError:
            return False
    path = path[:-4] if path.endswith('.git') else path
    return bool(host and (host + '/' + path).lower() == expected.lower())


def _checkout(git):
    git.check_filters()
    top = Path(git.text('rev-parse', '--show-toplevel')).resolve()
    if top != git.root:
        raise MaintenanceError('blocked_repository', 'Maintenance requires the checkout root.')
    if any(record.startswith(b'160000 ') for record in git.call('ls-files', '--stage', '-z')[1].split(b'\0')):
        raise MaintenanceError('blocked_repository', 'Submodule checkouts require manual maintenance.')
    if git.call('status', '--porcelain=v1', '-z', '--untracked-files=all')[1]:
        raise MaintenanceError('skipped_dirty', 'Checkout has local changes; further update and installation stopped.')
    files = git.call('ls-files', '-v', '-z')[1].split(b'\0')
    if any(item and (item[:1].islower() or item[:1] == b'S') for item in files):
        raise MaintenanceError('skipped_index_flags', 'Tracked files have assume-unchanged or skip-worktree flags.')
    for operation in OPERATIONS:
        path = Path(git.text('rev-parse', '--git-path', operation))
        if not path.is_absolute():
            path = git.root / path
        if path.exists():
            raise MaintenanceError('skipped_operation', 'An unfinished Git operation requires manual completion.')
    branch = git.text('symbolic-ref', '--quiet', '--short', 'HEAD')
    if branch != 'main':
        raise MaintenanceError('skipped_branch', 'Maintenance updates main only.')
    upstream = git.text('rev-parse', '--abbrev-ref', '--symbolic-full-name', '@{upstream}')
    if upstream != 'origin/main':
        raise MaintenanceError('blocked_repository', 'Main must track origin/main.')
    head = git.text('rev-parse', 'HEAD')
    if not re.fullmatch(r'[a-f0-9]{40,64}', head):
        raise MaintenanceError('blocked_repository', 'Unsupported Git revision format.')
    return head


def _configuration(root, git):
    path = root / 'registry' / 'harness.json'
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 1024 * 1024:
        raise MaintenanceError('blocked_configuration', 'Maintenance configuration is missing or unsafe.')
    try:
        config = json.loads(path.read_text(encoding='utf-8'))['maintenance']
    except (ValueError, KeyError, TypeError):
        raise MaintenanceError('blocked_configuration', 'Maintenance configuration is missing or invalid.') from None
    if not isinstance(config, dict) or config.get('enabled') is not True or config.get('remote') != 'origin' or config.get('branch') != 'main':
        raise MaintenanceError('blocked_configuration', 'Maintenance requires enabled origin/main configuration.')
    remote = git.text('remote', 'get-url', '--all', 'origin')
    if '\n' in remote or not _remote_matches(remote, config.get('expected_repository')):
        raise MaintenanceError('blocked_repository', 'Origin does not match the configured repository identity.')
    return config


def _sync_apply(home=None, mode='auto'):
    """Fresh-process installer entry; no fetch, runtime probe, models or scheduler."""
    from . import target_install
    if mode not in ('auto', 'link', 'copy'):
        return _report('sync_failed', reason='Unsupported installation mode.')
    try:
        installation = target_install.run(target='all', home=home, mode=mode)
    except (ValueError, OSError, KeyError):
        return _report('sync_failed', installation_status='failed',
                       guidance_completed=False, partially_applied=False,
                       reason='Installation preflight failed; no private diagnostics were retained.')
    ready = (installation.get('ready') is True
             and installation.get('guidance_completed') is True
             and installation.get('status') != 'partially_applied')
    return _report('synced' if ready else 'sync_failed', 0 if ready else 1,
                   installation_status=installation['status'],
                   guidance_completed=installation.get('guidance_completed') is True,
                   global_items=len(installation.get('guidance', [])),
                   partially_applied=installation.get('status') == 'partially_applied')


def _check_default_home(home):
    if home is None and os.environ.get('CODEX_HOME'):
        configured = Path(os.environ['CODEX_HOME']).expanduser().resolve()
        if configured != (Path.home() / '.codex').resolve():
            raise MaintenanceError('blocked_configuration', 'Default maintenance requires the standard Codex home; CODEX_HOME points elsewhere.')


def sync(home=None, mode='auto'):
    """Serialize standalone sync; a verified fresh child reuses its parent's lock."""
    lock = None
    folder = logs = None
    child = False
    try:
        _check_default_home(home)
        _, folder, logs = _storage(home, create=True)
        token = os.environ.get('AI_HARNESS_MAINTENANCE_TOKEN')
        if token:
            # Internal coordination only, not a security boundary against this user.
            path = folder / 'maintenance.lock'
            _safe_path(path)
            if not re.fullmatch(r'[a-f0-9]{32}', token) or not path.exists() or path.stat().st_size > 4096:
                raise MaintenanceError('blocked_storage', 'Invalid parent maintenance lock marker.')
            metadata = json.loads(path.read_text(encoding='utf-8'))
            if metadata.get('token') != token or metadata.get('pid') != os.getppid():
                raise MaintenanceError('blocked_storage', 'Parent maintenance lock ownership changed.')
            child = True
        else:
            lock = _Lock(folder / 'maintenance.lock')
            if not lock.acquire():
                return _report('skipped_locked', reason='Another maintenance process holds the OS lock.')
        result = _sync_apply(home, mode)
        if not child:
            result['operation'] = 'sync'
            _record(folder, logs, result)
        return result
    except MaintenanceError as error:
        result = _report(error.status, reason=str(error))
    except (OSError, ValueError, KeyError):
        result = _report('sync_failed', reason='Maintenance sync failed; no private diagnostics were retained.')
    finally:
        if lock is not None:
            lock.close()
    return result


def run(root=None, home=None, mode='auto'):
    """Update a clean trusted main checkout, then run the newly fetched installer."""
    lock = None
    folder = logs = None
    before = after = None
    try:
        if mode not in ('auto', 'link', 'copy'):
            raise MaintenanceError('blocked_configuration', 'Unsupported installation mode.')
        _check_default_home(home)
        root = Path(root or ROOT).expanduser().resolve(strict=True)
        user_home, folder, logs = _storage(home, create=True)
        lock = _Lock(folder / 'maintenance.lock')
        if not lock.acquire():
            # Never race the active owner's status/log writes.
            return _report('skipped_locked', reason='Another maintenance process holds the OS lock.')
        with tempfile.TemporaryDirectory(prefix='.maintenance-run-', dir=str(folder)) as temporary:
            temp = Path(temporary)
            hooks = temp / 'hooks'; hooks.mkdir()
            env = _environment()
            git = _Git(root, hooks, env)
            before = after = _checkout(git)
            _configuration(root, git)
            git.call('fetch', '--no-tags', '--no-recurse-submodules', '--no-write-fetch-head',
                     'origin', 'refs/heads/main:refs/remotes/origin/main', failure='fetch_failed')
            if _checkout(git) != before:
                raise MaintenanceError('skipped_changed', 'Checkout changed during fetch; no integration was attempted.')
            _configuration(root, git)
            target = git.text('rev-parse', 'refs/remotes/origin/main')
            counts = git.text('rev-list', '--left-right', '--count', before + '...' + target).split()
            if len(counts) != 2 or not all(value.isdigit() for value in counts):
                raise MaintenanceError('blocked_repository', 'Cannot classify local and remote history.')
            if int(counts[0]):
                raise MaintenanceError('skipped_divergent' if int(counts[1]) else 'skipped_ahead',
                                       'Local commits require manual reconciliation; no merge or reset was attempted.')
            if _checkout(git) != before:
                raise MaintenanceError('skipped_changed', 'Checkout changed before integration.')
            if target != before:
                git.call('merge', '--ff-only', '--no-edit', '--no-stat', target, failure='update_failed')
            after = _checkout(git)
            if after != target:
                raise MaintenanceError('skipped_changed', 'Checkout changed before installer launch.')
            _configuration(root, git)
            entry = root / 'ai.py'
            if entry.is_symlink() or not entry.is_file():
                raise MaintenanceError('sync_failed', 'Updated installer entrypoint is missing or unsafe.')
            if git.text('ls-files', '--error-unmatch', 'ai.py') != 'ai.py':
                raise MaintenanceError('sync_failed', 'Installer entrypoint must be tracked.')
            cache = temp / 'pycache'; cache.mkdir()
            command = [sys.executable, '-E', '-s', '-B', '-X', 'pycache_prefix=' + str(cache),
                       str(entry), 'maintenance', 'sync', '--mode', mode, '--home', str(user_home)]
            env['AI_HARNESS_MAINTENANCE_TOKEN'] = lock.token
            code, output = _process(command, root=root, env=env, timeout=SYNC_TIMEOUT)
            try:
                child = json.loads(output)
            except (ValueError, UnicodeError):
                raise MaintenanceError('sync_failed', 'Fresh installer failed or returned invalid bounded output.') from None
            if not isinstance(child, dict) or child.get('status') not in ('synced', 'sync_failed'):
                raise MaintenanceError('sync_failed', 'Fresh installer reported failure; no raw diagnostics were retained.')
            failed = child['status'] == 'sync_failed'
            if code != (1 if failed else 0) or child.get('exit_code') != code:
                raise MaintenanceError('sync_failed', 'Fresh installer exit status did not match its report.')
            if (type(child.get('guidance_completed')) is not bool
                    or type(child.get('partially_applied')) is not bool
                    or not failed and not child['guidance_completed']):
                raise MaintenanceError('sync_failed', 'Fresh installer did not confirm its installation state.')
            result = _report('sync_failed' if failed else 'updated' if after != before else 'current',
                             1 if failed else 0, revision_before=before, revision_after=after,
                             guidance_completed=child['guidance_completed'],
                             partially_applied=child['partially_applied'])
    except MaintenanceError as error:
        result = _report(error.status, reason=str(error), revision_before=before, revision_after=after)
    except KeyboardInterrupt:
        result = _report('interrupted', 130, reason='Maintenance was interrupted; inspect state before retrying.',
                         revision_before=before, revision_after=after)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError):
        result = _report('maintenance_failed', reason='Maintenance failed; no raw subprocess diagnostics were retained.',
                         revision_before=before, revision_after=after)
    try:
        if folder is not None and logs is not None and lock is not None and lock.descriptor is not None:
            result['operation'] = 'run'
            _record(folder, logs, result)
    except (OSError, ValueError):
        result = _report('blocked_storage', reason='Could not safely record maintenance status.')
    finally:
        if lock is not None:
            lock.close()
    return result


def status(home=None):
    try:
        _, folder, _ = _storage(home)
        path = folder / 'maintenance-status.json'
        _safe_path(path)
        if not path.exists():
            return _report('not_run', 0)
        if path.stat().st_size > MAX_LOG:
            raise ValueError('oversized')
        result = json.loads(path.read_text(encoding='utf-8'))
        if not isinstance(result, dict) or result.get('schema_version') != 1:
            raise ValueError('invalid')
        return result
    except (ValueError, OSError):
        return _report('blocked_storage', reason='Maintenance status is missing, invalid or unsafe to read.')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['run', 'sync', 'status'])
    parser.add_argument('--home', type=Path)
    parser.add_argument('--root', type=Path, help='Checkout root for run; otherwise use this harness checkout.')
    parser.add_argument('--mode', choices=['auto', 'link', 'copy'], default='auto')
    args = parser.parse_args(argv)
    if args.command == 'run':
        result = run(args.root, args.home, args.mode)
    elif args.command == 'sync':
        if args.root is not None:
            parser.error('sync runs the current loaded checkout; --root applies to run only')
        result = sync(args.home, args.mode)
    else:
        result = status(args.home)
    print(json.dumps(result, indent=2))
    return result.get('exit_code', 1)


if __name__ == '__main__':
    sys.exit(main())
