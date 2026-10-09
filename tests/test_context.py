"""Project instruction discovery checks without client, configuration or file effects."""
import contextlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

from lib import context


class ContextTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix='context audit ')
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding='utf-8')
        return path

    def codes(self, report):
        return {item['code'] for item in report['diagnostics']}

    def test_root_to_cwd_order_and_byte_budget(self):
        self.write('AGENTS.md', 'root')
        self.write('app/AGENTS.md', '12345678')
        self.write('app/sub/AGENTS.md', 'last')
        report = context.doctor(self.root, self.root / 'app/sub', max_bytes=10)
        self.assertEqual([row['bytes'] for row in report['files']], [4, 8, 4])
        self.assertEqual([row['included_bytes'] for row in report['files']], [4, 6, 0])
        self.assertEqual([row['status'] for row in report['files']], ['selected', 'truncated', 'excluded'])
        self.assertEqual(report['selected_bytes'], 16)
        self.assertEqual(report['included_bytes'], 10)
        self.assertFalse(report['ok'])

    def test_empty_override_shadows_default_and_fallback_without_consuming_budget(self):
        chosen = self.write('AGENTS.override.md', '')
        lower = self.write('AGENTS.md', 'never loaded')
        fallback = self.write('TEAM.md', 'also shadowed')
        self.write('child/AGENTS.md', 'ok')
        report = context.doctor(self.root, self.root / 'child', max_bytes=2, fallback=['TEAM.md'])
        self.assertTrue(report['ok'])
        self.assertEqual(report['files'][0]['path'], str(chosen))
        self.assertEqual(report['files'][0]['status'], 'empty')
        self.assertEqual(report['files'][0]['shadowed'], [str(lower), str(fallback)])
        self.assertEqual(report['included_bytes'], 2)

    def test_whitespace_prefix_does_not_consume_budget(self):
        self.write('AGENTS.md', ' \n\t')
        self.write('child/AGENTS.md', 'yes')
        report = context.doctor(self.root, self.root / 'child', max_bytes=3)
        self.assertEqual(report['files'][0]['status'], 'empty')
        self.assertEqual(report['included_bytes'], 3)
        self.assertTrue(report['ok'])

    def test_utf8_is_accounted_in_bytes_even_if_cut_inside_character(self):
        self.write('AGENTS.md', 'éé')
        report = context.doctor(self.root, max_bytes=3)
        self.assertEqual(report['files'][0]['bytes'], 4)
        self.assertEqual(report['included_bytes'], 3)
        self.assertIn('truncated', self.codes(report))

    def test_zero_budget_excludes_nonempty_file(self):
        self.write('AGENTS.md', 'instructions')
        report = context.doctor(self.root, max_bytes=0)
        self.assertEqual(report['files'][0]['status'], 'excluded')
        self.assertEqual(report['included_bytes'], 0)
        self.assertFalse(report['ok'])

    def test_no_instructions_is_advisory_and_no_ancestors_are_read(self):
        self.write('AGENTS.md', 'parent must not be read')
        child = self.root / 'child'
        child.mkdir()
        report = context.doctor(child)
        self.assertTrue(report['ok'])
        self.assertEqual(report['files'], [])
        self.assertEqual(self.codes(report), {'no_instructions'})

    def test_fallback_priority_and_duplicates(self):
        self.write('TEAM.md', 'team')
        self.write('OTHER.md', 'other')
        report = context.doctor(self.root, fallback=['TEAM.md', 'OTHER.md', 'TEAM.md', 'AGENTS.md'])
        self.assertEqual(report['fallback_names'], ['TEAM.md', 'OTHER.md'])
        self.assertEqual(report['files'][0]['path'], str(self.root / 'TEAM.md'))
        self.assertEqual(report['files'][0]['shadowed'], [str(self.root / 'OTHER.md')])

    def test_directory_candidate_is_skipped(self):
        (self.root / 'AGENTS.override.md').mkdir()
        self.write('AGENTS.md', 'yes')
        report = context.doctor(self.root)
        self.assertTrue(report['ok'])
        self.assertEqual(report['included_bytes'], 3)

    def test_rejects_escaping_cwd_and_unsafe_fallback_before_reads(self):
        with mock.patch.object(context, 'read_regular') as read:
            with self.assertRaises(ValueError):
                context.doctor(self.root, self.root.parent)
            for name in ('../outside', '/absolute', r'..\outside', 'C:secret', '.', '..', 'bad\nname', 'name. ', 'NUL.md', 'CON', 'a*b', ''):
                with self.subTest(name=name), self.assertRaises(ValueError):
                    context.doctor(self.root, fallback=[name])
        read.assert_not_called()

    def test_rejects_invalid_budget(self):
        for value in (-1, True, 1.5, '100'):
            with self.subTest(value=value), self.assertRaises(ValueError):
                context.doctor(self.root, max_bytes=value)

    @unittest.skipUnless(hasattr(os, 'symlink'), 'Symlinks unavailable')
    def test_links_inside_and_outside_project_are_not_opened(self):
        for target in (self.root / 'private.txt', self.root.parent / 'outside-secret'):
            with self.subTest(target=target):
                link = self.root / 'AGENTS.md'
                try:
                    link.symlink_to(target)
                except OSError:
                    self.skipTest('Symlink creation unavailable')
                with mock.patch.object(context, 'read_regular') as read:
                    report = context.doctor(self.root)
                read.assert_not_called()
                self.assertIn('unsafe_link', self.codes(report))
                self.assertFalse(report['ok'])
                link.unlink()

    @unittest.skipUnless(hasattr(os, 'mkfifo'), 'FIFOs unavailable')
    def test_special_files_are_not_opened_or_allowed_to_block(self):
        os.mkfifo(self.root / 'AGENTS.override.md')
        self.write('AGENTS.md', 'safe')
        report = context.doctor(self.root)
        self.assertIn('special_file', self.codes(report))
        self.assertEqual(report['files'][0]['included_bytes'], 4)
        self.assertFalse(report['ok'])

    def test_unreadable_file_does_not_fall_back_or_leak_exception_contents(self):
        self.write('AGENTS.override.md', 'private')
        self.write('AGENTS.md', 'lower')
        with mock.patch.object(context, 'read_regular', side_effect=PermissionError('SECRET')):
            report = context.doctor(self.root)
        self.assertFalse(report['ok'])
        self.assertEqual(report['files'][0]['status'], 'unreadable')
        self.assertNotIn('SECRET', json.dumps(report))
        self.assertEqual(report['included_bytes'], 0)

    def test_empty_override_still_requires_readability(self):
        selected = self.write('AGENTS.override.md', '')
        self.write('AGENTS.md', 'lower priority')
        with mock.patch.object(context, 'read_regular', side_effect=PermissionError('SECRET')) as read:
            report = context.doctor(self.root)
        read.assert_called_once()
        self.assertEqual(read.call_args[0][0], selected)
        self.assertEqual(read.call_args[0][2], 0)
        self.assertFalse(report['ok'])
        self.assertEqual(report['files'][0]['status'], 'unreadable')
        self.assertNotIn('SECRET', json.dumps(report))

    @unittest.skipUnless(hasattr(os, 'symlink'), 'Symlinks unavailable')
    def test_shadowed_link_does_not_hide_readable_override(self):
        self.write('AGENTS.override.md', 'active')
        try:
            (self.root / 'AGENTS.md').symlink_to(self.root / 'missing')
        except OSError:
            self.skipTest('Symlink creation unavailable')
        report = context.doctor(self.root)
        self.assertTrue(report['ok'])
        self.assertEqual(report['included_bytes'], 6)
        self.assertIn('unsafe_link', self.codes(report))

    def test_read_cap_is_error_even_with_large_user_budget(self):
        self.write('AGENTS.md', 'a' * 20)
        with mock.patch.object(context, 'MAX_READ_BYTES', 10):
            report = context.doctor(self.root, max_bytes=30)
        self.assertFalse(report['ok'])
        self.assertEqual(report['files'][0]['inspected_bytes'], 10)
        self.assertIn('inspection_limit', self.codes(report))

    def test_long_file_is_only_an_advisory(self):
        self.write('AGENTS.md', 'a\n' * 101)
        report = context.doctor(self.root)
        self.assertTrue(report['ok'])
        self.assertIn('long_instructions', self.codes(report))

    def test_no_text_or_writes_and_cli_exit_status(self):
        path = self.write('AGENTS.md', 'SECRET_INSTRUCTION_DO_NOT_PRINT')
        before = (path.read_bytes(), path.stat().st_mtime_ns)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            success = context.main(['doctor', '--project', str(self.root)])
        self.assertEqual(success, 0)
        self.assertNotIn('SECRET_INSTRUCTION_DO_NOT_PRINT', output.getvalue())
        self.assertTrue(json.loads(output.getvalue())['ok'])
        with contextlib.redirect_stdout(io.StringIO()):
            failure = context.main(['doctor', '--project', str(self.root), '--max-bytes', '1'])
        self.assertEqual(failure, 1)
        self.assertEqual(before, (path.read_bytes(), path.stat().st_mtime_ns))
        self.assertEqual(list(self.root.iterdir()), [path])


if __name__ == '__main__':
    unittest.main()
