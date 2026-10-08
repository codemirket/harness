"""No real Claude/model requests: a native executable fixture exercises boundaries."""
import contextlib
import io
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import time
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from lib import claude_delegate as delegate

FIXTURE = r'''
import json, os, signal, subprocess, sys, time
from pathlib import Path
args = sys.argv[1:]
flags = "--print --output-format --restricted --safe-mode --tools --disallowedTools --permission-mode --permission-prompts --strict-mcp-config --mcp-config --no-session-persistence --setting-sources --settings --disable-slash-commands --no-chrome --append-system-prompt --effort"
mode = os.environ.get('FIXTURE_MODE', '')
if os.environ.get('DISABLE_AUTOUPDATER') != '1' or os.environ.get('DISABLE_UPDATES') != '1':
    sys.exit(23)
if '--help' in args:
    if mode == 'old': flags = flags.replace('--restricted', '')
    if mode == 'no-effort': flags = flags.replace('--effort', '')
    if mode == 'effort-prefix': flags = flags.replace('--effort', '--effortful')
    print(flags)
elif 'auth' in args:
    if mode == 'bad-auth':
        print('not-json'); sys.exit(0)
    print(json.dumps({'loggedIn': mode != 'logged-out',
                      'authMethod': 'api_key' if mode == 'api' else 'claude.ai',
                      'apiProvider': 'firstParty', 'email': 'PRIVATE-IDENTITY',
                      'token': 'PRIVATE-TOKEN'}))
else:
    prompt = sys.stdin.read()
    Path(os.environ['FIXTURE_CALL']).write_text(json.dumps({'args':args, 'prompt':prompt,
          'depth':os.environ.get('AI_SHARED_CLAUDE_DELEGATE'), 'cwd':os.getcwd()}))
    if mode == 'timeout':
        time.sleep(20)
    if mode == 'tree':
        child = subprocess.Popen([sys.executable, '-c',
            'import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(30)'])
        Path(os.environ['FIXTURE_PID']).write_text(str(child.pid))
        time.sleep(20)
    if mode == 'flood':
        sys.stdout.write('x' * 2000000); sys.stdout.flush(); time.sleep(20)
    if mode == 'stderr-flood':
        sys.stderr.write('x' * 2000000); sys.stderr.flush(); time.sleep(20)
    if mode == 'exit':
        sys.stderr.write('PRIVATE-TOKEN'); sys.exit(7)
    if mode == 'malformed':
        print('PRIVATE-TOKEN'); sys.exit(0)
    value = {'type':'result', 'subtype':'success', 'is_error':False,
             'result':'Concrete finding: src/app.ts:12 omits error handling.',
             'other_private_metadata':'PRIVATE-TOKEN'}
    if mode == 'error': value['is_error'] = True
    if mode == 'limit': value['subtype'] = 'error_max_turns'
    if mode == 'empty': value['result'] = ''
    print(json.dumps(value))
'''


@unittest.skipIf(os.name == 'nt', 'Executable fixture uses a POSIX shebang; native Windows needs a real executable fixture.')
class DelegateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='delegate fixture ')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name).resolve()
        self.project = self.base / 'project with spaces'
        self.project.mkdir()
        self.binary = self.base / 'claude test'
        self.binary.write_text('#!' + sys.executable + '\n' + FIXTURE)
        self.binary.chmod(0o755)
        self.env = {'PATH': os.environ.get('PATH', ''), 'HOME': str(self.base),
                    'FIXTURE_CALL': str(self.base/'call.json'), 'FIXTURE_PID': str(self.base/'child.pid')}
        self.prompt = 'Review src/app.ts; return file:line evidence. Do not edit.'

    def run_delegate(self, **kwargs):
        return delegate.run_delegate(self.project, self.prompt, executable=self.binary,
                                     env=self.env, **kwargs)

    def test_preview_has_no_inference_or_prompt_leak_and_reports_scope(self):
        result = self.run_delegate(plan=True)
        self.assertFalse((self.base/'call.json').exists())
        self.assertEqual(result['action'], 'planned')
        self.assertEqual(result['runtime']['billing_route'], 'claude-subscription')
        text = json.dumps(result)
        self.assertNotIn(self.prompt, text)
        self.assertNotIn('PRIVATE-', text)
        args = result['argv']
        self.assertEqual(args[args.index('--tools')+1], 'Read,Glob,Grep')
        self.assertEqual(args[args.index('--permission-mode')+1], 'dontAsk')
        self.assertEqual(args[args.index('--permission-prompts')+1], 'none')
        self.assertIn('--restricted', args)
        self.assertIn('--safe-mode', args)
        self.assertIn('--strict-mcp-config', args)
        self.assertIn('--no-session-persistence', args)
        self.assertNotIn('--bare', args)
        self.assertNotIn('--allowedTools', args)
        self.assertNotIn('--dangerously-skip-permissions', args)
        self.assertNotIn('--fallback-model', args)
        self.assertNotIn('--effort', args)

    def test_read_exclusions_preserve_positive_user_and_project_denies_only(self):
        user_settings = self.base/'.claude/settings.json'
        user_settings.parent.mkdir()
        user_settings.write_text(json.dumps({'permissions': {'deny': [
            'Read(/private/**)', 'Read(~/private/**)', 'Read(!.env)', 'Bash(*)'],
            'allow': ['Bash(*)']}, 'env': {'ANTHROPIC_API_KEY': 'PRIVATE-TOKEN'},
            'hooks': {'SessionStart': [{'command': 'DO-NOT-EXECUTE'}]}}))
        project_settings = self.project/'.claude/settings.json'
        project_settings.parent.mkdir()
        project_settings.write_text(json.dumps({'permissions': {'deny': ['Read(customer-data/**)']}}))
        plan = self.run_delegate(plan=True)
        args = plan['argv']
        settings = json.loads(args[args.index('--settings')+1])
        denies = settings['permissions']['deny']
        for rule in ('Read(//**/.env)', 'Read(//**/.env.*)', 'Read(//**/secrets/**)',
                     'Read(//**/id_rsa*)', 'Read(//**/auth.json)', 'Read(//**/.claude/**)',
                     'Read(customer-data/**)', 'Read(~/private/**)'):
            self.assertIn(rule, denies)
        self.assertIn('Read(/' + str(user_settings.parent) + '/private/**)', denies)
        self.assertNotIn('Read(!.env)', denies)
        self.assertNotIn('allow', settings['permissions'])
        self.assertNotIn('hooks', settings)
        self.assertNotIn('env', settings)
        self.assertNotIn('PRIVATE-TOKEN', json.dumps(plan))
        self.assertNotIn('DO-NOT-EXECUTE', json.dumps(plan))
        self.assertFalse((self.base/'call.json').exists())

    def test_unreadable_policy_fails_closed_without_reading_symlink_target(self):
        settings = self.project/'.claude/settings.json'
        settings.parent.mkdir()
        settings.write_text('{invalid json')
        with self.assertRaisesRegex(delegate.DelegateError, 'Cannot parse'):
            self.run_delegate(plan=True)
        settings.unlink()
        settings.symlink_to(self.base/'credentials.json')
        with self.assertRaisesRegex(delegate.DelegateError, 'symlinked'):
            self.run_delegate(plan=True)
        self.assertFalse((self.base/'call.json').exists())

    def test_success_stdin_no_shell_expansion_and_new_private_output(self):
        self.prompt = 'Review $(touch injected); `touch injected2` "quoted"\nsecond line'
        output = self.base/'answer.json'
        result = self.run_delegate(output=output, model='sonnet')
        call = json.loads((self.base/'call.json').read_text())
        self.assertEqual(call['prompt'], self.prompt)
        self.assertEqual(call['cwd'], str(self.project))
        self.assertEqual(call['depth'], '1')
        self.assertEqual(call['args'][-2:], ['--model', 'sonnet'])
        self.assertNotIn('--effort', call['args'])
        self.assertNotIn(self.prompt, call['args'])
        self.assertEqual(result, json.loads(output.read_text()))
        self.assertEqual(output.stat().st_mode & 0o777, 0o600)
        self.assertFalse((self.project/'injected').exists())
        self.assertNotIn('PRIVATE-', output.read_text())

    def test_explicit_effort_is_preserved_in_plan_and_model_invocation(self):
        for effort in ('low', 'medium', 'high', 'xhigh', 'max'):
            with self.subTest(effort=effort):
                call_path = self.base/'call.json'
                if call_path.exists():
                    call_path.unlink()
                plan = self.run_delegate(plan=True, model='claude-opus-5-5', effort=effort)
                self.assertFalse(call_path.exists())
                self.assertEqual(plan['argv'][-4:], ['--model', 'claude-opus-5-5', '--effort', effort])
                result = self.run_delegate(model='claude-opus-5-5', effort=effort)
                call = json.loads(call_path.read_text())
                self.assertEqual(call['args'], plan['argv'][1:])
                self.assertEqual(result['argv'], plan['argv'])

    def test_invalid_effort_fails_before_any_cli_request(self):
        for effort in ('', 'maximum', 'MAX', '--help', 'max high', 1, True, [], {}):
            with self.subTest(effort=effort), mock.patch.object(delegate, '_capture') as capture:
                with self.assertRaisesRegex(delegate.DelegateError, 'Effort'):
                    self.run_delegate(effort=effort)
                capture.assert_not_called()

    def test_missing_effort_capability_fails_closed_only_when_requested(self):
        for mode in ('no-effort', 'effort-prefix'):
            with self.subTest(mode=mode):
                self.env['FIXTURE_MODE'] = mode
                for plan in (True, False):
                    with self.assertRaisesRegex(delegate.DelegateError, '--effort'):
                        self.run_delegate(plan=plan, effort='max')
                    self.assertFalse((self.base/'call.json').exists())
                self.assertNotIn('--effort', self.run_delegate(plan=True)['argv'])
                self.assertEqual(self.run_delegate()['action'], 'completed')
                (self.base/'call.json').unlink()

    def test_api_and_unknown_provider_require_explicit_optin_before_inference(self):
        for name in ('ANTHROPIC_API_KEY', 'ANTHROPIC_BASE_URL', 'ANTHROPIC_AUTH_TOKEN', 'ANTHROPIC_PROFILE', 'CLAUDE_CODE_USE_VERTEX'):
            with self.subTest(name=name):
                self.env[name] = 'PRIVATE-TOKEN'
                plan = self.run_delegate(plan=True)
                self.assertTrue(plan['requires_api_opt_in'])
                self.assertIn(name, plan['runtime']['billing_environment'])
                self.assertNotIn('PRIVATE-TOKEN', json.dumps(plan))
                with self.assertRaisesRegex(delegate.DelegateError, 'allow-api'):
                    self.run_delegate()
                self.assertFalse((self.base/'call.json').exists())
                del self.env[name]
        self.env['FIXTURE_MODE'] = 'api'
        with self.assertRaisesRegex(delegate.DelegateError, 'allow-api'):
            self.run_delegate()
        self.assertEqual(self.run_delegate(allow_api=True)['action'], 'completed')

    def test_output_conflicts_are_preflighted_and_preserve_bytes(self):
        output = self.base/'answer.json'
        output.write_text('user draft')
        with self.assertRaisesRegex(delegate.DelegateError, 'already exists'):
            self.run_delegate(output=output)
        self.assertEqual(output.read_text(), 'user draft')
        output.unlink()
        output.symlink_to(self.base/'missing')
        with self.assertRaisesRegex(delegate.DelegateError, 'already exists'):
            self.run_delegate(output=output)
        with self.assertRaisesRegex(delegate.DelegateError, 'read-only'):
            self.run_delegate(plan=True, output=self.base/'unused')
        self.assertFalse((self.base/'call.json').exists())

    def test_partial_write_failure_leaves_no_result_or_staging_file(self):
        import errno
        output = self.base/'answer.json'
        original = os.fdopen

        @contextlib.contextmanager
        def partial_writer(fd, mode):
            with original(fd, mode) as stream:
                class Writer:
                    def write(self, data):
                        stream.write(data[:12])
                        stream.flush()
                        raise OSError(errno.ENOSPC, 'simulated disk full')
                yield Writer()

        with mock.patch.object(delegate.os, 'fdopen', side_effect=partial_writer):
            with self.assertRaisesRegex(OSError, 'disk full'):
                self.run_delegate(output=output)
        self.assertFalse(output.exists())
        self.assertEqual(list(self.base.glob('.claude-result-*')), [])

    def test_flush_failure_and_publish_race_preserve_destination(self):
        output = self.base/'answer.json'
        with mock.patch.object(delegate.os, 'fsync', side_effect=OSError('flush failure')):
            with self.assertRaisesRegex(OSError, 'flush failure'):
                self.run_delegate(output=output)
        self.assertFalse(output.exists())
        self.assertEqual(list(self.base.glob('.claude-result-*')), [])
        original = os.link

        def concurrent_publish(source, destination):
            Path(destination).write_text('concurrent user result')
            return original(source, destination)

        with mock.patch.object(delegate.os, 'link', side_effect=concurrent_publish):
            with self.assertRaises(FileExistsError):
                self.run_delegate(output=output)
        self.assertEqual(output.read_text(), 'concurrent user result')
        self.assertEqual(list(self.base.glob('.claude-result-*')), [])

    def test_unsupported_cli_or_failed_auth_never_requests_model(self):
        for mode, error in [('old','Update Claude Code'), ('bad-auth','auth status'), ('logged-out','not signed in')]:
            with self.subTest(mode=mode):
                self.env['FIXTURE_MODE'] = mode
                with self.assertRaisesRegex(delegate.DelegateError, error):
                    self.run_delegate()
                self.assertFalse((self.base/'call.json').exists())

    def test_no_recursive_delegation_or_invalid_input(self):
        for name in ('AI_SHARED_CLAUDE_DELEGATE', 'CLAUDECODE'):
            self.env[name] = '1'
            with self.assertRaisesRegex(delegate.DelegateError, 'Recursive'):
                self.run_delegate()
            del self.env[name]
        for timeout in (0, -1, float('nan'), float('inf'), 3601):
            with self.assertRaises(delegate.DelegateError):self.run_delegate(timeout=timeout)
        with self.assertRaises(delegate.DelegateError):self.run_delegate(model='--dangerously-skip-permissions')
        self.prompt = 'x' * (delegate.MAX_PROMPT_BYTES+1)
        with self.assertRaises(delegate.DelegateError):self.run_delegate()
        self.assertFalse((self.base/'call.json').exists())

    def test_failures_do_not_publish_partial_or_raw_diagnostics(self):
        for mode in ('exit','malformed','error','limit','empty'):
            with self.subTest(mode=mode):
                self.env['FIXTURE_MODE'] = mode
                output = self.base/'failed.json'
                with self.assertRaises(delegate.DelegateError) as error:self.run_delegate(output=output)
                self.assertNotIn('PRIVATE-TOKEN', str(error.exception))
                self.assertFalse(output.exists())

    def test_timeout_and_output_limits_cancel_promptly(self):
        for mode in ('timeout', 'flood', 'stderr-flood'):
            with self.subTest(mode=mode):
                self.env['FIXTURE_MODE'] = mode
                start = time.monotonic()
                with self.assertRaisesRegex(delegate.DelegateError, 'timed out|output limit'):
                    self.run_delegate(timeout=0.3 if mode == 'timeout' else 3)
                self.assertLess(time.monotonic()-start, 5)

    def test_timeout_kills_descendant_even_when_it_ignores_term(self):
        self.env['FIXTURE_MODE'] = 'tree'
        with self.assertRaisesRegex(delegate.DelegateError, 'timed out'):
            self.run_delegate(timeout=0.4)
        pid = int((self.base/'child.pid').read_text())
        # On some platforms a terminated orphan briefly remains a zombie.
        import subprocess
        state = subprocess.run(['ps','-o','stat=','-p',str(pid)],capture_output=True,text=True).stdout.strip()
        self.assertTrue(not state or state.startswith('Z'), state)

    def test_reaps_exited_process_when_macos_rejects_group_signal(self):
        process = mock.Mock()
        process.poll.return_value = 0
        with mock.patch.object(delegate.os, 'killpg', side_effect=PermissionError()):
            delegate._stop_process(process)
        process.wait.assert_called_once_with(timeout=5)
        process.kill.assert_not_called()
        process.terminate.assert_not_called()

    def test_sigterm_to_runner_cancels_child_tree(self):
        import subprocess
        self.env['FIXTURE_MODE'] = 'tree'
        prompt = self.base/'prompt.txt'; prompt.write_text(self.prompt)
        runner = subprocess.Popen([sys.executable, '-m', 'lib.claude_delegate',
            '--project', str(self.project), '--prompt-file', str(prompt),
            '--executable', str(self.binary)], cwd=str(ROOT), env=self.env,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic()+5
            while not (self.base/'child.pid').exists() and time.monotonic()<deadline:
                time.sleep(0.02)
            self.assertTrue((self.base/'child.pid').exists())
            runner.terminate()
            stdout, stderr = runner.communicate(timeout=5)
            self.assertNotEqual(runner.returncode, 0)
            self.assertIn(b'cancelled', stderr)
            pid = int((self.base/'child.pid').read_text())
            state = subprocess.run(['ps','-o','stat=','-p',str(pid)],capture_output=True,text=True).stdout.strip()
            self.assertTrue(not state or state.startswith('Z'), state)
        finally:
            if runner.poll() is None:
                runner.kill()
            runner.communicate(timeout=5)

    def test_main_returns_nonzero_and_reads_utf8_file(self):
        prompt = self.base/'prompt.txt';prompt.write_text(self.prompt)
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.dict(os.environ,self.env,clear=True), contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = delegate.main(['--project',str(self.project),'--prompt-file',str(prompt),
                                  '--executable',str(self.binary),'--plan'])
        self.assertEqual(code, 0, err.getvalue())
        self.assertEqual(json.loads(out.getvalue())['action'],'planned')
        self.assertFalse((self.base/'call.json').exists())
        prompt.write_bytes(b'\xff')
        with contextlib.redirect_stderr(err):
            code = delegate.main(['--project',str(self.project),'--prompt-file',str(prompt)])
        self.assertEqual(code,1)

    def test_main_passes_explicit_effort_and_rejects_invalid_choices(self):
        prompt = self.base/'prompt.txt'; prompt.write_text(self.prompt)
        args = ['--project', str(self.project), '--prompt-file', str(prompt),
                '--executable', str(self.binary), '--model', 'claude-opus-5-5']
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.dict(os.environ, self.env, clear=True), contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = delegate.main(args + ['--effort', 'max'])
        self.assertEqual(code, 0, err.getvalue())
        call = json.loads((self.base/'call.json').read_text())
        self.assertEqual(call['args'][-4:], ['--model', 'claude-opus-5-5', '--effort', 'max'])
        with contextlib.redirect_stderr(err), mock.patch.object(delegate, '_capture') as capture:
            with self.assertRaises(SystemExit) as rejected:
                delegate.main(args + ['--effort', 'maximum'])
        self.assertEqual(rejected.exception.code, 2)
        capture.assert_not_called()


if __name__ == '__main__':
    unittest.main()
