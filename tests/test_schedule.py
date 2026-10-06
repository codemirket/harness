"""Native scheduler tests: every OS interaction is mocked, all writes isolated."""
import json
import io
import os
from pathlib import Path
import shlex
import stat
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import xml.etree.ElementTree as ET

from lib import schedule


class ScheduleFixture(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='schedule tests ')
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name).resolve()
        self.home = self.base / 'home'; self.home.mkdir()
        self.root = self.base / 'repo with spaces'; self.root.mkdir()
        (self.root / 'ai.py').write_text('# isolated entry\n')
        self.options = {'root': self.root, 'home': self.home, 'python': Path(sys.executable)}
        self.cron = 'PATH=/usr/local/bin:/usr/bin\n12 4 * * * echo PRIVATE_UNRELATED_JOB\n'
        self.initial = self.cron
        self.writes = []
        self.patch('current_home', return_value=self.home)
        self.patch('validate_root', return_value=None)
        self.patch('backend', return_value='crontab')
        self.patch('crontab_tool', return_value='/mock/crontab')
        self.patch('run', side_effect=self.mock_run)

    def patch(self, name, **kwargs):
        patcher = mock.patch.object(schedule, name, **kwargs)
        value = patcher.start(); self.addCleanup(patcher.stop)
        return value

    def mock_run(self, args, input_text=None):
        self.assertEqual(args[0], '/mock/crontab', 'No real OS commands allowed')
        if args[1] == '-l': return subprocess.CompletedProcess(args, 0, self.cron, '')
        self.assertEqual(args[1], '-')
        self.writes.append(input_text); self.cron = input_text
        return subprocess.CompletedProcess(args, 0, '', '')


class ScheduleTests(ScheduleFixture):
    def test_plan_reads_only_and_hides_unrelated_jobs(self):
        result = schedule.plan(**self.options)
        self.assertTrue(result['ready_to_apply']); self.assertTrue(result['changed'])
        self.assertFalse(result['verified']); self.assertEqual(result['expression'], '0 0 * * *')
        self.assertNotIn('PRIVATE_UNRELATED_JOB', json.dumps(result))
        self.assertFalse((self.home / '.agent-harness').exists()); self.assertEqual(self.writes, [])

    def test_apply_preserves_content_backups_and_is_idempotent(self):
        result = schedule.apply(**self.options)
        self.assertEqual(result['status'], 'registered'); self.assertTrue(result['verified'])
        self.assertTrue(self.cron.startswith(self.initial))
        self.assertEqual(self.cron.count(schedule.BEGIN), 1)
        backup = Path(result['backup'])
        self.assertEqual(backup.read_text(), self.initial)
        if os.name != 'nt': self.assertEqual(stat.S_IMODE(backup.stat().st_mode), 0o600)
        again = schedule.apply(**self.options)
        self.assertEqual(again['status'], 'unchanged'); self.assertEqual(len(self.writes), 1)
        self.assertFalse((self.home / '.agent-harness/schedule.lock').exists())

    def test_changed_mode_updates_one_owned_job(self):
        schedule.apply(**self.options)
        result = schedule.apply(**dict(self.options, mode='copy'))
        self.assertTrue(result['verified']); self.assertIn('--mode copy', self.cron)
        self.assertEqual(self.cron.count(schedule.BEGIN), 1)
        self.assertTrue(self.cron.startswith(self.initial))

    def test_alternate_home_never_probes_or_registers(self):
        other = self.base / 'other'; other.mkdir()
        with mock.patch.object(schedule, 'run', side_effect=AssertionError('No scheduler access')):
            result = schedule.apply(**dict(self.options, home=other))
        self.assertFalse(result['ready_to_apply']); self.assertFalse((other / '.agent-harness').exists())

    def test_unavailable_scheduler_returns_blocker_without_writes(self):
        with mock.patch.object(schedule, 'crontab_tool', side_effect=ValueError('Unavailable')):
            result = schedule.apply(**self.options)
        self.assertEqual(result['status'], 'blocked'); self.assertEqual(self.writes, [])

    def test_exact_legacy_migration_only(self):
        root = Path('/Users/alice/repo'); home = Path('/Users/alice')
        legacy = schedule.legacy_line(root, home)
        self.assertIsNotNone(legacy)
        before = '# retain\n' + legacy + '\n0 1 * * * echo keep\n'
        after, migrated = schedule.merge_crontab(before, '0 0 * * * new', legacy)
        self.assertTrue(migrated); self.assertNotIn(legacy, after)
        self.assertTrue(after.startswith('# retain\n')); self.assertTrue(after.endswith('0 1 * * * echo keep\n'))
        similar = legacy.replace('codex_status=$?', 'codex_status=0')
        with self.assertRaisesRegex(ValueError, 'modified same-repository'):
            schedule.merge_crontab(similar + '\n', '0 0 * * * new', legacy)

    def test_legacy_mentions_in_comments_echo_and_other_repos_are_preserved(self):
        script = '/Users/alice/repo/setup/macos.sh'
        text = ('# /bin/sh ' + script + ' codex\n'
                '0 1 * * * echo ' + script + ' codex\n'
                '0 1 * * * echo "/bin/sh ' + script + ' codex"\n'
                '0 2 * * * /bin/sh /other/repo/setup/macos.sh codex\n'
                '0 3 * * * echo ok # /bin/sh ' + script + ' codex\n')
        merged, migrated = schedule.merge_crontab(text, '0 0 * * * new', legacy_script=script)
        self.assertTrue(merged.startswith(text)); self.assertFalse(migrated)
        for command in ['/bin/sh ' + script + ' codex', "'/Users/alice/repo/setup/macos.sh' both"]:
            with self.assertRaises(ValueError):
                schedule.merge_crontab('3 2 * * * '+command+'\n', '0 0 * * * new', legacy_script=script)

    def test_launcher_ignores_inherited_python_environment(self):
        report = schedule.plan(**self.options)
        self.assertEqual(report['command'][1:4], ['-E', '-s', '-B'])
        env = dict(os.environ, PYTHONHOME='/nonexistent-personal-ai-test', PYTHONPATH='/nonexistent-personal-ai-test')
        result = subprocess.run(report['command'][:4] + ['-c', 'import sys; assert sys.flags.ignore_environment and sys.flags.no_user_site and sys.dont_write_bytecode'], env=env, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_ambiguous_markers_or_duplicate_legacy_refuse(self):
        line = '0 0 * * * command'
        cases = [schedule.BEGIN+'\n', schedule.END+'\n',
                 schedule.BEGIN+'\n'+line+'\nextra\n'+schedule.END+'\n',
                 (schedule.BEGIN+'\n'+line+'\n'+schedule.END+'\n')*2,
                 '# BEGIN personal-ai daily-maintenance v99\n',
                 'legacy\nlegacy\n']
        for text in cases:
            with self.subTest(text=text), self.assertRaises(ValueError):
                schedule.merge_crontab(text, line, 'legacy')

    def test_cron_timezone_conflict_refused(self):
        self.cron = 'CRON_TZ=UTC\n' + self.cron
        result = schedule.apply(**self.options)
        self.assertFalse(result['ready_to_apply']); self.assertEqual(self.writes, [])

    def test_quoted_spaces_percent_backslash_and_apostrophe_survive_cron(self):
        command = ['/some python/bin/python', "/repo% a\\%b'c/ai.py", 'maintenance', 'run']
        line = schedule.cron_line(command, Path('/home/a b'))
        encoded = line[len('0 0 * * * '):]
        decoded = []; i = 0
        # Cron escape pass independent of shlex, matching crontab's percent rule.
        while i < len(encoded):
            char = encoded[i]
            if char == '\\' and i + 1 < len(encoded):
                i += 1
                if encoded[i] != '%': decoded.append('\\')
                decoded.append(encoded[i])
            elif char == '%': self.fail('Unescaped cron percent')
            else: decoded.append(char)
            i += 1
        outer = shlex.split(''.join(decoded))
        # Shell quote contains literal escaped backslashes; cron leaves pairs.
        self.assertEqual(outer[:2], ['/usr/bin/env', 'HOME=/home/a b'])
        self.assertEqual(outer[-4], '-c')
        self.assertEqual(shlex.split(outer[-3])[1:], command)

    def test_newline_paths_rejected(self):
        root = self.base / 'bad\nroot'; root.mkdir(); (root / 'ai.py').write_text('')
        self.assertFalse(schedule.plan(**dict(self.options, root=root))['ready_to_apply'])

    def test_concurrent_edit_before_backup_is_preserved(self):
        calls = 0
        def reading(tool):
            nonlocal calls
            calls += 1
            return self.initial if calls == 1 else 'new user content\n'
        with mock.patch.object(schedule, 'read_crontab', side_effect=reading):
            result = schedule.apply(**self.options)
        self.assertEqual(result['status'], 'failed'); self.assertEqual(self.writes, [])
        self.assertIsNone(result['backup'])

    def test_concurrent_edit_after_backup_is_preserved(self):
        with mock.patch.object(schedule, 'read_crontab', side_effect=[self.initial, self.initial, 'changed\n']):
            result = schedule.apply(**self.options)
        self.assertEqual(result['status'], 'failed'); self.assertEqual(self.writes, [])
        self.assertEqual(Path(result['backup']).read_text(), self.initial)

    def test_registration_failure_does_not_claim_applied(self):
        def failing(args, input_text=None):
            return self.mock_run(args, input_text) if args[1] == '-l' else subprocess.CompletedProcess(args, 1, '', 'PRIVATE ERROR')
        with mock.patch.object(schedule, 'run', side_effect=failing): result = schedule.apply(**self.options)
        self.assertFalse(result['applied']); self.assertFalse(result['verified'])
        self.assertNotIn('PRIVATE ERROR', json.dumps(result)); self.assertEqual(self.cron, self.initial)

    def test_readback_mismatch_retains_backup_without_rollback(self):
        def mismatch(args, input_text=None):
            outcome = self.mock_run(args, input_text)
            if args[1] == '-': self.cron += '# simultaneous user edit\n'
            return outcome
        with mock.patch.object(schedule, 'run', side_effect=mismatch): result = schedule.apply(**self.options)
        self.assertTrue(result['applied']); self.assertFalse(result['verified'])
        self.assertEqual(len(self.writes), 1); self.assertIn('simultaneous', self.cron)

    def test_existing_lock_is_not_removed(self):
        state = self.home / '.agent-harness'; state.mkdir()
        lock = state / 'schedule.lock'; lock.write_text('another process')
        result = schedule.apply(**self.options)
        self.assertEqual(result['status'], 'failed'); self.assertEqual(lock.read_text(), 'another process')
        self.assertEqual(self.writes, [])

    def test_symlink_state_directory_is_refused(self):
        external = self.base / 'outside'; external.mkdir()
        try: (self.home / '.agent-harness').symlink_to(external, target_is_directory=True)
        except OSError: self.skipTest('Symlink privilege unavailable')
        self.assertFalse(schedule.apply(**self.options)['ready_to_apply'])
        self.assertEqual(list(external.iterdir()), [])

    def test_no_crontab_and_read_failure_distinguished(self):
        with mock.patch.object(schedule, 'run', return_value=subprocess.CompletedProcess([], 1, '', 'no crontab for alice\n')):
            self.assertEqual(schedule.read_crontab('/mock/crontab'), '')
        for error in ['permission denied', 'command missing']:
            with mock.patch.object(schedule, 'run', return_value=subprocess.CompletedProcess([], 1, '', error)), self.assertRaises(ValueError):
                schedule.read_crontab('/mock/crontab')

    def test_runner_preserves_crlf_bytes_and_limits_output(self):
        completed = subprocess.CompletedProcess([], 0, b'PATH=/bin\r\n# untouched\r\n', b'')
        with mock.patch.object(schedule.subprocess, 'run', return_value=completed) as run:
            result = ORIGINAL_RUN(['/mock/tool'], 'new\r\n')
            self.assertEqual(result.stdout, 'PATH=/bin\r\n# untouched\r\n')
            self.assertEqual(run.call_args.kwargs['input'], b'new\r\n')
        with mock.patch.object(schedule.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, b'x' * (schedule.MAX_BYTES+1), b'')):
            with self.assertRaises(ValueError): ORIGINAL_RUN(['/mock/tool'])

    def test_subprocess_timeout_redacts_output(self):
        # Restore the real runner while mocking subprocess itself.
        with mock.patch.object(schedule.subprocess, 'run', side_effect=subprocess.TimeoutExpired('tool', 20, output='PRIVATE')):
            with self.assertRaisesRegex(ValueError, 'timed out'):
                ORIGINAL_RUN(['/mock/tool'])


ORIGINAL_RUN = schedule.run
ORIGINAL_VALIDATE_ROOT = schedule.validate_root


class RootValidationTests(ScheduleFixture):
    def test_root_requires_declared_midnight_maintenance(self):
        registry = self.root / 'registry'; registry.mkdir()
        config = registry / 'harness.json'
        for declaration in ({}, {'enabled':False,'time':'00:00','timezone':'system'},
                            {'enabled':True,'time':'12:00','timezone':'system'}):
            config.write_text(json.dumps({'maintenance':declaration}))
            with self.assertRaises(ValueError): ORIGINAL_VALIDATE_ROOT(self.root)

    def test_root_must_match_git_toplevel_and_probe_disables_hooks(self):
        registry = self.root / 'registry'; registry.mkdir()
        (registry/'harness.json').write_text(json.dumps({'maintenance':{'enabled':True,'time':'00:00','timezone':'system'}}))
        with mock.patch.object(schedule.shutil,'which',return_value='/mock/git'), mock.patch.object(schedule,'run',return_value=subprocess.CompletedProcess([],0,str(self.root)+'\n','')) as run:
            ORIGINAL_VALIDATE_ROOT(self.root)
            args=run.call_args.args[0]
            self.assertIn('core.hooksPath=/dev/null',args)
            self.assertIn('core.fsmonitor=false',args)
        with mock.patch.object(schedule.shutil,'which',return_value='/mock/git'), mock.patch.object(schedule,'run',return_value=subprocess.CompletedProcess([],0,str(self.base)+'\n','')):
            with self.assertRaises(ValueError): ORIGINAL_VALIDATE_ROOT(self.root)

    def test_schedule_install_alias_and_check_exit(self):
        args=['--root',str(self.root),'--home',str(self.home),'--python',sys.executable]
        with mock.patch('sys.stdout',new_callable=io.StringIO):
            self.assertEqual(schedule.main(['plan']+args),0)
            self.assertEqual(schedule.main(['check']+args),1)
            self.assertEqual(schedule.main(['install']+args),0)
            self.assertEqual(schedule.main(['check']+args),0)

    def test_macos_no_crontab_prefix_is_empty(self):
        with mock.patch.object(schedule,'run',return_value=subprocess.CompletedProcess([],1,'','crontab: no crontab for alice\n')):
            self.assertEqual(schedule.read_crontab('/mock/crontab'),'')

    def test_non_posix_cron_shell_requires_manual_review(self):
        self.cron='SHELL=/bin/tcsh\n'+self.cron
        self.assertFalse(schedule.apply(**self.options)['ready_to_apply'])
        self.assertEqual(self.writes,[])



class WindowsScheduleTests(ScheduleFixture):
    def setUp(self):
        super().setUp()
        self.sid = 'S-1-5-21-1000'
        self.xml = None
        self.windows_writes = []
        self.patch('backend', return_value='windows-task-scheduler')
        self.patch('powershell_tool', return_value='/mock/powershell')
        self.patch('ps_call', side_effect=self.ps)

    def ps(self, tool, script, payload=None):
        self.assertEqual(tool, '/mock/powershell')
        if script == schedule.PS_WRITE:
            self.assertEqual(payload['previous'], self.xml)
            self.assertEqual(payload['sid'], self.sid)
            self.windows_writes.append(payload); self.xml = payload['xml']
        else: self.assertEqual(script, schedule.PS_READ)
        return {'sid': self.sid, 'xml': self.xml}

    # Do not inherit cron tests: Windows behavior has its own contract assertions.
    def test_windows_register_idempotent_and_current_user_only(self):
        report = schedule.apply(**self.options)
        self.assertTrue(report['verified']); self.assertTrue(report['applied'])
        self.assertEqual(len(self.windows_writes), 1)
        parsed = ET.fromstring(self.xml); ns = {'t':schedule.NS}
        self.assertEqual(parsed.findtext('t:Principals/t:Principal/t:UserId', namespaces=ns), self.sid)
        self.assertEqual(parsed.findtext('t:Principals/t:Principal/t:LogonType', namespaces=ns), 'InteractiveToken')
        self.assertEqual(parsed.findtext('t:Principals/t:Principal/t:RunLevel', namespaces=ns), 'LeastPrivilege')
        self.assertEqual(parsed.findtext('t:Settings/t:StartWhenAvailable', namespaces=ns), 'true')
        trigger_names = [node.tag.rsplit('}',1)[-1] for node in parsed.find('t:Triggers/t:CalendarTrigger', ns)]
        self.assertEqual(trigger_names, ['Enabled','StartBoundary','ScheduleByDay'])
        self.assertEqual(schedule.apply(**self.options)['status'], 'unchanged')
        self.assertEqual(len(self.windows_writes), 1)

    def test_windows_unmanaged_collision_is_preserved(self):
        self.xml = schedule.task_xml(['python','ai.py','maintenance','run'], self.root, self.sid).replace(schedule.OWNER, 'user task')
        before = self.xml
        report = schedule.apply(**self.options)
        self.assertFalse(report['ready_to_apply']); self.assertEqual(self.xml, before)
        self.assertEqual(self.windows_writes, [])

    def test_windows_extra_action_refused(self):
        self.xml = schedule.task_xml(['python','ai.py'], self.root, self.sid).replace('</Actions>', '<Exec><Command>other</Command></Exec></Actions>')
        self.assertFalse(schedule.plan(**self.options)['ready_to_apply'])

    def test_windows_repetition_is_not_a_daily_noop(self):
        self.xml = schedule.task_xml(['python','ai.py'], self.root, self.sid).replace('</CalendarTrigger>', '<Repetition><Interval>PT1M</Interval></Repetition></CalendarTrigger>')
        self.assertFalse(schedule.plan(**self.options)['ready_to_apply'])

    def test_windows_extra_execution_conditions_report_drift(self):
        schedule.apply(**self.options)
        original = self.xml
        for name, value in [('RunOnlyIfIdle','true'), ('RunOnlyIfNetworkAvailable','true'),
                            ('AllowStartOnDemand','false'), ('DisallowStartOnRemoteAppSession','true')]:
            self.xml = original.replace('</Settings>', '<'+name+'>'+value+'</'+name+'></Settings>')
            with self.subTest(name=name):
                report = schedule.plan(**self.options)
                self.assertTrue(report['changed']); self.assertFalse(report['verified'])
        self.xml = original.replace('</Settings>', '<MaintenanceSettings><Period>P1D</Period></MaintenanceSettings></Settings>')
        self.assertFalse(schedule.plan(**self.options)['ready_to_apply'])

    def test_windows_os_added_default_settings_are_a_noop(self):
        schedule.apply(**self.options)
        extra = ('<RunOnlyIfIdle>false</RunOnlyIfIdle><RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>'
                 '<WakeToRun>false</WakeToRun><AllowStartOnDemand>true</AllowStartOnDemand>'
                 '<AllowHardTerminate>true</AllowHardTerminate><Priority>7</Priority><Hidden>false</Hidden>'
                 '<IdleSettings><StopOnIdleEnd>true</StopOnIdleEnd><RestartOnIdle>false</RestartOnIdle></IdleSettings>'
                 '<DisallowStartOnRemoteAppSession>false</DisallowStartOnRemoteAppSession>'
                 '<DeleteExpiredTaskAfter>PT0S</DeleteExpiredTaskAfter>')
        self.xml = self.xml.replace('</Settings>',extra+'</Settings>')
        self.xml = self.xml.replace('<ScheduleByDay>', '<ExecutionTimeLimit>PT72H</ExecutionTimeLimit><RandomDelay>PT0M</RandomDelay><ScheduleByDay>')
        report = schedule.plan(**self.options)
        self.assertTrue(report['verified']); self.assertFalse(report['changed'])
        self.assertEqual(report['command'][1:4],['-E','-s','-B'])

    def test_windows_changed_command_updates_owned_task_and_backups_xml(self):
        schedule.apply(**self.options); before = self.xml
        report = schedule.apply(**dict(self.options, mode='copy'))
        self.assertTrue(report['verified']); self.assertEqual(len(self.windows_writes), 2)
        self.assertEqual(Path(report['backup']).read_text(), before)

    def test_windows_cli_arguments_quote_spaces_and_no_shell(self):
        xml = schedule.task_xml([r'C:\Program Files\Python\python.exe', r'C:\User Data\repo\ai.py', 'maintenance','run'], Path('C:/repo'), self.sid)
        parsed = ET.fromstring(xml); ns={'t':schedule.NS}
        self.assertEqual(parsed.findtext('t:Actions/t:Exec/t:Command', namespaces=ns), r'C:\Program Files\Python\python.exe')
        self.assertEqual(parsed.findtext('t:Actions/t:Exec/t:Arguments', namespaces=ns), '"C:\\User Data\\repo\\ai.py" maintenance run')


if __name__ == '__main__': unittest.main()
