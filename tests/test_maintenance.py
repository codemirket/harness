"""Updater integration uses only local bare Git remotes and isolated homes."""
import contextlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from lib import maintenance

CONFIG = {'schema_version': 1, 'maintenance': {'enabled': True, 'time': '00:00',
          'timezone': 'system', 'remote': 'origin', 'branch': 'main',
          'expected_repository': 'github.com/example/harness'}}


def entry(version, status='synced'):
    return '''import json,sys
from pathlib import Path
assert sys.argv[1:3] == ['maintenance','sync']
home=Path(sys.argv[sys.argv.index('--home')+1])
(home/'fresh-sync.txt').write_text(%r)
print(json.dumps({'schema_version':1,'status':%r,'exit_code':%s,'guidance_completed':True,'partially_applied':%s}))
raise SystemExit(%s)
''' % (version, status, 1 if status == 'sync_failed' else 0,
       status == 'sync_failed', 1 if status == 'sync_failed' else 0)


@unittest.skipUnless(shutil.which('git'), 'Git required for local integration fixtures')
class MaintenanceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='maintenance fixture ')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name).resolve()
        self.home = self.base/'home'; self.home.mkdir()
        self.remote = self.base/'remote.git'
        self.source = self.base/'author'; self.source.mkdir()
        self.root = self.base/'installed checkout'
        self.env = dict(os.environ, GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=os.devnull,
                        GIT_AUTHOR_NAME='Fixture', GIT_AUTHOR_EMAIL='fixture@example.invalid',
                        GIT_COMMITTER_NAME='Fixture', GIT_COMMITTER_EMAIL='fixture@example.invalid')
        environment = mock.patch.dict(os.environ, self.env)
        environment.start(); self.addCleanup(environment.stop)
        self.git(self.base, 'init', '--bare', str(self.remote))
        self.git(self.source, 'init', '-b', 'main')
        (self.source/'registry').mkdir()
        (self.source/'registry/harness.json').write_text(json.dumps(CONFIG))
        (self.source/'ai.py').write_text(entry('initial'))
        self.git(self.source, 'add', '.')
        self.git(self.source, 'commit', '-m', 'initial fixture')
        self.git(self.source, 'remote', 'add', 'origin', str(self.remote))
        self.git(self.source, 'push', '-u', 'origin', 'main')
        self.git(self.base, 'clone', '--branch', 'main', str(self.remote), str(self.root))
        self.initial = self.git(self.root, 'rev-parse', 'HEAD')
        # Production accepts only the configured GitHub identity and https/ssh.
        # Only these two boundaries are injected; fetch/merge are real local Git.
        matcher = mock.patch.object(maintenance, '_remote_matches',
                                    side_effect=lambda actual, expected: actual == str(self.remote))
        matcher.start(); self.addCleanup(matcher.stop)
        protocols = mock.patch.object(maintenance, 'ALLOWED_PROTOCOLS', 'file')
        protocols.start(); self.addCleanup(protocols.stop)

    def git(self, root, *args):
        result = subprocess.run(['git', '-c', 'commit.gpgsign=false', *args], cwd=root,
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, 'Fixture git command failed: ' + args[0])
        return result.stdout.strip()

    def run_job(self):
        return maintenance.run(self.root, self.home, 'copy')

    def push(self, version='updated', status='synced'):
        (self.source/'ai.py').write_text(entry(version, status))
        self.git(self.source, 'add', 'ai.py')
        self.git(self.source, 'commit', '-m', 'new fixture installer')
        self.git(self.source, 'push', 'origin', 'main')
        return self.git(self.source, 'rev-parse', 'HEAD')

    def test_fast_forward_executes_fresh_installer_and_records_private_state(self):
        tip = self.push()
        state = self.home/'.agent-harness'; state.mkdir()
        (state/'state.json').write_text('existing ownership receipt')
        result = self.run_job()
        self.assertEqual(result['status'], 'updated', result)
        self.assertEqual(result['exit_code'], 0)
        self.assertEqual(result['revision_before'], self.initial)
        self.assertEqual(result['revision_after'], tip)
        self.assertEqual((self.home/'fresh-sync.txt').read_text(), 'updated')
        self.assertEqual((state/'state.json').read_text(), 'existing ownership receipt')
        self.assertEqual(maintenance.status(self.home)['status'], 'updated')
        for path in (state/'maintenance-status.json', state/'logs/maintenance.log', state/'maintenance.lock'):
            if os.name != 'nt': self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(self.git(self.root, 'status', '--porcelain'), '')
        self.assertEqual(self.run_job()['status'], 'current')

    def test_dirty_staged_or_untracked_never_fetches_or_runs_installer(self):
        self.push()
        for kind in ('tracked', 'staged', 'untracked'):
            with self.subTest(kind=kind):
                path = self.root/('ai.py' if kind == 'tracked' else 'uncommitted.py')
                previous = path.read_bytes() if path.exists() else None
                path.write_text('uncommitted user bytes')
                if kind == 'staged': self.git(self.root, 'add', path.name)
                result = self.run_job()
                self.assertEqual(result['status'], 'skipped_dirty', result)
                self.assertEqual(path.read_text(), 'uncommitted user bytes')
                self.assertEqual(self.git(self.root, 'rev-parse', 'origin/main'), self.initial)
                self.assertFalse((self.home/'fresh-sync.txt').exists())
                if kind == 'staged': self.git(self.root, 'reset', '--', path.name)
                if previous is None: path.unlink()
                else: path.write_bytes(previous)

    def test_clean_operation_and_hidden_index_changes_are_rejected(self):
        marker = self.root/'.git/rebase-merge'; marker.mkdir()
        self.assertEqual(self.run_job()['status'], 'skipped_operation')
        marker.rmdir()
        for flag, undo in (('--assume-unchanged','--no-assume-unchanged'),
                           ('--skip-worktree','--no-skip-worktree')):
            self.git(self.root, 'update-index', flag, 'ai.py')
            self.assertEqual(self.run_job()['status'], 'skipped_index_flags')
            self.git(self.root, 'update-index', undo, 'ai.py')
        self.assertFalse((self.home/'fresh-sync.txt').exists())

    def test_submodule_index_is_rejected_before_recursive_status(self):
        self.git(self.root,'update-index','--add','--cacheinfo','160000',self.initial,'nested')
        calls=[]
        original=maintenance._Git.call
        def observe(instance,*args,**kwargs):
            calls.append(args[0])
            return original(instance,*args,**kwargs)
        with mock.patch.object(maintenance._Git,'call',new=observe): result=self.run_job()
        self.assertEqual(result['status'],'blocked_repository',result)
        self.assertNotIn('status',calls)
        self.assertNotIn('fetch',calls)
        self.assertFalse((self.home/'fresh-sync.txt').exists())

    def test_ahead_and_divergent_commits_remain_untouched(self):
        (self.root/'local.txt').write_text('local committed content')
        self.git(self.root, 'add', 'local.txt'); self.git(self.root, 'commit', '-m', 'local work')
        local = self.git(self.root, 'rev-parse', 'HEAD')
        self.assertEqual(self.run_job()['status'], 'skipped_ahead')
        self.push()
        self.assertEqual(self.run_job()['status'], 'skipped_divergent')
        self.assertEqual(self.git(self.root, 'rev-parse', 'HEAD'), local)
        self.assertFalse((self.home/'fresh-sync.txt').exists())

    def test_concurrent_edit_after_fetch_stops_integration(self):
        self.push()
        call = maintenance._Git.call
        def edit_after_fetch(instance, *args, **kwargs):
            result = call(instance, *args, **kwargs)
            if args[0] == 'fetch': (self.root/'ai.py').write_text('concurrent editor bytes')
            return result
        with mock.patch.object(maintenance._Git, 'call', new=edit_after_fetch): result = self.run_job()
        self.assertEqual(result['status'], 'skipped_dirty', result)
        self.assertEqual(self.git(self.root, 'rev-parse', 'HEAD'), self.initial)
        self.assertEqual((self.root/'ai.py').read_text(), 'concurrent editor bytes')
        self.assertFalse((self.home/'fresh-sync.txt').exists())

    @unittest.skipIf(os.name == 'nt', 'POSIX hook fixture')
    def test_post_merge_and_fsmonitor_hooks_do_not_execute(self):
        self.push()
        sentinel = self.home/'hook-ran'
        hooks = self.base/'custom hooks'; hooks.mkdir()
        hook = hooks/'post-merge'
        hook.write_text('#!/bin/sh\ntouch "' + str(sentinel) + '"\n'); hook.chmod(0o755)
        self.git(self.root, 'config', 'core.hooksPath', str(hooks))
        self.git(self.root, 'config', 'core.fsmonitor', str(hook))
        result = self.run_job()
        self.assertEqual(result['status'], 'updated', result)
        self.assertFalse(sentinel.exists())

    def test_filters_and_wrong_branch_upstream_are_blocked(self):
        self.git(self.root, 'config', 'filter.fixture.smudge', 'touch should-not-run')
        self.assertEqual(self.run_job()['status'], 'blocked_repository')
        self.git(self.root, 'config', '--unset', 'filter.fixture.smudge')
        self.git(self.root, 'switch', '-c', 'other')
        self.assertEqual(self.run_job()['status'], 'skipped_branch')
        self.git(self.root, 'switch', 'main')
        self.git(self.root, 'branch', '--unset-upstream')
        self.assertEqual(self.run_job()['status'], 'blocked_repository')
        self.assertFalse((self.home/'fresh-sync.txt').exists())

    def test_effective_filters_are_rejected_before_status_and_git_auth_is_preserved(self):
        global_config = self.base/'git-global-config'
        global_config.write_text('[credential]\n    helper = fixture-helper\n')
        with mock.patch.dict(os.environ, {'GIT_CONFIG_GLOBAL': str(global_config)}):
            env = maintenance._environment()
            self.assertEqual(env['GIT_CONFIG_GLOBAL'], str(global_config))
            with tempfile.TemporaryDirectory() as hooks:
                git = maintenance._Git(self.root, Path(hooks), env)
                self.assertNotIn('credential.helper=', git.argv)
                self.assertEqual(git.text('config', '--get', 'credential.helper'), 'fixture-helper')
            global_config.write_text('[filter "fixture"]\n    clean = DO-NOT-EXECUTE\n')
            calls=[]
            original=maintenance._Git.call
            def observe(instance,*args,**kwargs):
                calls.append(args[0])
                return original(instance,*args,**kwargs)
            with mock.patch.object(maintenance._Git,'call',new=observe):
                result=self.run_job()
            self.assertEqual(result['status'],'blocked_repository',result)
            self.assertNotIn('status',calls)
            self.assertNotIn('fetch',calls)
        self.assertFalse((self.home/'fresh-sync.txt').exists())

    def test_partial_installation_remains_nonzero_after_successful_source_update(self):
        self.push(status='sync_failed')
        result = self.run_job()
        self.assertEqual(result['status'], 'sync_failed', result)
        self.assertEqual(result['exit_code'], 1)
        self.assertTrue(result['partially_applied'])
        self.assertTrue(result['guidance_completed'])

    def test_unsafe_status_log_or_lock_preserves_external_content(self):
        external = self.base/'private'; external.write_text('PRIVATE-SENTINEL')
        _, folder, logs = maintenance._storage(self.home, True)
        for path in (folder/'maintenance.lock', folder/'maintenance-status.json', logs/'maintenance.log'):
            with self.subTest(path=path.name):
                if path.exists(): path.unlink()
                path.symlink_to(external)
                result = self.run_job()
                self.assertEqual(result['status'], 'blocked_storage', result)
                self.assertEqual(external.read_text(), 'PRIVATE-SENTINEL')
                self.assertNotIn('PRIVATE-SENTINEL', json.dumps(result))
                path.unlink()

    def test_lock_is_process_safe_and_stale_file_does_not_require_deletion(self):
        _, folder, _ = maintenance._storage(self.home, True)
        lock = folder/'maintenance.lock'
        lock.write_text('{"pid":999999,"locked_at":1}')
        script = '''from pathlib import Path
import sys,time
from lib.maintenance import _Lock
lock=_Lock(Path(sys.argv[1])); assert lock.acquire()
print('locked',flush=True)
time.sleep(20)
'''
        owner = subprocess.Popen([sys.executable, '-c', script, str(lock)], cwd=ROOT,
                                 stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            self.assertEqual(owner.stdout.readline().strip(), 'locked')
            result = self.run_job()
            self.assertEqual(result['status'], 'skipped_locked', result)
            self.assertFalse((self.home/'fresh-sync.txt').exists())
        finally:
            owner.kill(); owner.communicate(timeout=5)
        self.assertEqual(self.run_job()['status'], 'current')
        self.assertTrue(lock.exists())

    def test_logs_rotate_with_caps_and_do_not_contain_subprocess_diagnostics(self):
        _, folder, logs = maintenance._storage(self.home, True)
        (logs/'maintenance.log').write_bytes(b'x' * (maintenance.MAX_LOG+100))
        with mock.patch.object(maintenance._Git, 'call', side_effect=maintenance.MaintenanceError('fetch_failed','Git operation failed.')):
            result = self.run_job()
        self.assertEqual(result['status'], 'fetch_failed')
        self.assertLessEqual((logs/'maintenance.log').stat().st_size, maintenance.MAX_LOG)
        self.assertLessEqual((logs/'maintenance.log.1').stat().st_size, maintenance.MAX_LOG)

    def test_invalid_child_json_never_counts_as_success(self):
        (self.source/'ai.py').write_text("print('PRIVATE-RAW-DIAGNOSTIC')\n")
        self.git(self.source,'add','ai.py'); self.git(self.source,'commit','-m','bad output'); self.git(self.source,'push')
        result=self.run_job()
        self.assertEqual(result['status'],'sync_failed',result)
        self.assertNotIn('PRIVATE-RAW-DIAGNOSTIC',json.dumps(result))
        self.assertNotIn('PRIVATE-RAW-DIAGNOSTIC',(self.home/'.agent-harness/logs/maintenance.log').read_text())

    def test_child_success_without_completed_guidance_is_rejected(self):
        content = entry('claimed').replace("'guidance_completed':True", "'guidance_completed':False")
        (self.source / 'ai.py').write_text(content)
        self.git(self.source, 'add', 'ai.py')
        self.git(self.source, 'commit', '-m', 'incomplete installer claim')
        self.git(self.source, 'push')
        result = self.run_job()
        self.assertEqual(result['status'], 'sync_failed', result)
        self.assertEqual(result['exit_code'], 1)


class SyncAndIdentityTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='maintenance sync fixture ')
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name).resolve()

    def test_remote_identity_rejects_other_hosts_ports_and_transports(self):
        expected='github.com/example/harness'
        for remote in ('git@github.com:example/harness.git','https://github.com/example/harness.git',
                       'ssh://git@github.com/example/harness.git'):
            self.assertTrue(maintenance._remote_matches(remote,expected),remote)
        for remote in ('https://evil.example/example/harness.git','git@github.com:example/other.git',
                       'file:///tmp/harness','https://github.com:444/example/harness',
                       'http://github.com/example/harness','ext::sh malicious',
                       'https://github.com/example/harness?token=private'):
            self.assertFalse(maintenance._remote_matches(remote,expected))

    def test_sync_uses_shared_installer_and_does_not_apply_preferences(self):
        from lib import settings, target_install
        report = {'status': 'installed', 'ready': True,
                  'guidance_completed': True, 'guidance': [{}]}
        with mock.patch.object(target_install, 'run', return_value=report) as install, \
             mock.patch.object(settings, 'plan', side_effect=AssertionError('Preferences are user-owned')), \
             mock.patch.object(settings, 'apply', side_effect=AssertionError('Preferences are user-owned')):
            result = maintenance.sync(home=self.home, mode='copy')
        install.assert_called_once_with(target='all', home=self.home, mode='copy')
        self.assertEqual(result['status'], 'synced')
        self.assertEqual(result['exit_code'], 0)
        self.assertTrue(result['guidance_completed'])
        self.assertFalse(result['partially_applied'])
        self.assertEqual(result['global_items'], 1)

    def test_sync_preserves_partial_installation_state_without_private_errors(self):
        from lib import target_install
        report = {'status': 'partially_applied', 'ready': False,
                  'guidance_completed': True, 'guidance': [{}], 'blockers': ['PRIVATE']}
        with mock.patch.object(target_install, 'run', return_value=report):
            result = maintenance.sync(home=self.home)
        self.assertEqual(result['status'], 'sync_failed')
        self.assertEqual(result['exit_code'], 1)
        self.assertTrue(result['guidance_completed'])
        self.assertTrue(result['partially_applied'])
        self.assertNotIn('PRIVATE', json.dumps(result))

    def test_sync_preflight_failure_never_claims_completed_installation(self):
        from lib import target_install
        with mock.patch.object(target_install, 'run', side_effect=ValueError('PRIVATE')):
            result = maintenance.sync(home=self.home)
        self.assertEqual(result['status'], 'sync_failed')
        self.assertFalse(result['guidance_completed'])
        self.assertFalse(result['partially_applied'])
        self.assertNotIn('PRIVATE', json.dumps(result))

    def test_status_is_readonly_before_first_run(self):
        with tempfile.TemporaryDirectory() as directory:
            home=Path(directory)
            self.assertEqual(maintenance.status(home)['status'],'not_run')
            self.assertEqual(list(home.iterdir()),[])

    def test_default_custom_codex_home_blocks_before_writes(self):
        with mock.patch.dict(os.environ, {'CODEX_HOME':str(self.home/'different-client')}), \
             mock.patch.object(maintenance,'_storage') as storage:
            self.assertEqual(maintenance.run()['status'],'blocked_configuration')
            self.assertEqual(maintenance.sync()['status'],'blocked_configuration')
            storage.assert_not_called()

    def test_standalone_sync_records_and_respects_active_updater_lock(self):
        _,folder,_=maintenance._storage(self.home,True)
        lock=maintenance._Lock(folder/'maintenance.lock')
        self.assertTrue(lock.acquire())
        try:
            with mock.patch.object(maintenance,'_sync_apply') as apply:
                self.assertEqual(maintenance.sync(self.home)['status'],'skipped_locked')
                apply.assert_not_called()
        finally:
            lock.close()
        with mock.patch.object(maintenance,'_sync_apply',return_value=maintenance._report('synced',0)):
            self.assertEqual(maintenance.sync(self.home)['status'],'synced')
        self.assertEqual(maintenance.status(self.home)['operation'],'sync')

    def test_fresh_child_reuses_parent_lock_without_overwriting_parent_status(self):
        _,folder,_=maintenance._storage(self.home,True)
        lock=maintenance._Lock(folder/'maintenance.lock')
        self.assertTrue(lock.acquire())
        try:
            with mock.patch.dict(os.environ,{'AI_HARNESS_MAINTENANCE_TOKEN':lock.token}), \
                 mock.patch.object(maintenance.os,'getppid',return_value=os.getpid()), \
                 mock.patch.object(maintenance,'_sync_apply',return_value=maintenance._report('synced',0)):
                self.assertEqual(maintenance.sync(self.home)['status'],'synced')
            self.assertFalse((folder/'maintenance-status.json').exists())
            with mock.patch.dict(os.environ,{'AI_HARNESS_MAINTENANCE_TOKEN':'0'*32}), \
                 mock.patch.object(maintenance,'_sync_apply') as apply:
                self.assertEqual(maintenance.sync(self.home)['status'],'blocked_storage')
                apply.assert_not_called()
        finally:
            lock.close()

    @unittest.skipIf(os.name == 'nt', 'POSIX descendant verification; native Windows requires device coverage')
    def test_bounded_timeout_terminates_descendants_and_drops_output(self):
        pidfile=self.home/'child.pid'
        script = """import subprocess,sys,time
from pathlib import Path
child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)'])
Path(sys.argv[1]).write_text(str(child.pid))
print('PRIVATE-CHILD-OUTPUT',flush=True)
time.sleep(30)
"""
        code,output=maintenance._process([sys.executable,'-c',script,str(pidfile)],
            root=self.home,env=maintenance._environment(),timeout=0.3)
        self.assertIsNone(code)
        self.assertEqual(output,b'')
        pid=int(pidfile.read_text())
        state=subprocess.run(['ps','-o','stat=','-p',str(pid)],capture_output=True,text=True).stdout.strip()
        self.assertTrue(not state or state.startswith('Z'),state)
        code,output=maintenance._process([sys.executable,'-c',
            "import sys,time; sys.stdout.write('x'*2000000); sys.stdout.flush(); time.sleep(30)"],
            root=self.home,env=maintenance._environment(),timeout=3)
        self.assertIsNone(code)
        self.assertEqual(output,b'')

    def test_main_returns_report_exit_code(self):
        with mock.patch.object(maintenance,'run',return_value={'status':'skipped_dirty','exit_code':1}), \
             contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(maintenance.main(['run']),1)
            self.assertEqual(json.loads(output.getvalue())['status'],'skipped_dirty')


if __name__ == '__main__':
    unittest.main()
