"""Portable document inspection and renderer failure behavior; no external tools."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import signal
import sys
import time
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location('workbench_documents', ROOT / 'scripts/workbench/documents.py')
documents = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(documents)

W = documents.NS['w']
P = documents.NS['p']
A = documents.NS['a']
S = documents.NS['s']


class DocumentWorkbenchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()

    def package(self, name, parts):
        path = self.root / name
        with zipfile.ZipFile(path, 'w') as archive:
            archive.writestr('[Content_Types].xml', '<Types/>')
            for member, data in parts.items():
                archive.writestr(member, data)
        return path

    def docx(self, extra=None):
        parts = {'word/document.xml': '<w:document xmlns:w="{}"><w:body><w:p><w:r><w:t>Hello</w:t></w:r></w:p></w:body></w:document>'.format(W)}
        parts.update(extra or {})
        return self.package('input.docx', parts)

    def test_inspection_records_exact_input_without_claiming_visual_review(self):
        path = self.docx()
        before = path.read_bytes()
        out = self.root / 'inspection'
        report = documents.inspect_command(path, out)
        self.assertEqual(path.read_bytes(), before)
        self.assertEqual(report['input']['sha256'], documents.digest(path))
        self.assertEqual(report['machine_checks']['summary']['paragraphs'], 1)
        self.assertEqual(report['machine_checks']['summary']['text_characters'], 5)
        self.assertEqual(report['visual_review']['status'], 'not_performed')
        self.assertTrue((out / 'report.json').exists())

    def test_existing_output_is_never_replaced(self):
        path = self.docx()
        out = self.root / 'existing'; out.mkdir()
        sentinel = out / 'owned'; sentinel.write_text('preserve')
        with self.assertRaisesRegex(ValueError, 'already exists'):
            documents.inspect_command(path, out)
        self.assertEqual(sentinel.read_text(), 'preserve')

    def test_external_resources_and_macros_block_render(self):
        path = self.docx({'word/vbaProject.bin': b'not executed',
                         'word/_rels/document.xml.rels': '<Relationships><Relationship TargetMode="External" Type="image" Target="https://invalid.example/?token=PRIVATE_SENTINEL"/></Relationships>'})
        report = documents.inspect_data(path)
        self.assertNotIn('PRIVATE_SENTINEL', json.dumps(report))
        self.assertEqual({i['code'] for i in report['machine_checks']['issues']},
                         {'external_resource', 'active_or_external_package_part'})
        out = self.root / 'render';out.mkdir()
        with patch.object(documents, 'run') as command:
            with self.assertRaisesRegex(ValueError, 'Rendering blocked'):
                documents.render_into(path, out)
            command.assert_not_called()

    def test_hyperlinks_are_reported_without_fetching_them(self):
        path = self.docx({'word/_rels/document.xml.rels': '<Relationships><Relationship TargetMode="External" Type="urn:test/hyperlink" Target="https://invalid.example"/></Relationships>'})
        report = documents.inspect_data(path)
        self.assertEqual(report['machine_checks']['issues'][0]['severity'], 'warning')
        self.assertNotIn('invalid.example', json.dumps(report))

    def test_zip_traversal_is_rejected_without_extraction(self):
        path = self.docx({'../outside.xml': '<x/>'})
        with self.assertRaisesRegex(ValueError, 'unsafe package'):
            documents.inspect_data(path)
        self.assertFalse((self.root / 'outside.xml').exists())

    def test_xml_entities_rejected_in_utf8_and_utf16(self):
        text = '<!DOCTYPE x [<!ENTITY test "expansion">]><x>&test;</x>'
        for raw in [text.encode('utf-8'), text.encode('utf-16')]:
            with self.subTest(raw=raw[:8]), self.assertRaisesRegex(ValueError, 'DTD/entity'):
                documents.xml(raw)

    def test_offslide_shape_is_detected_without_claiming_text_fit(self):
        path = self.package('input.pptx', {
            'ppt/presentation.xml': '<p:presentation xmlns:p="{}"><p:sldSz cx="100" cy="100"/></p:presentation>'.format(P),
            'ppt/slides/slide1.xml': '<p:sld xmlns:p="{}" xmlns:a="{}"><p:cSld><p:spTree><p:sp><p:spPr><a:xfrm><a:off x="90" y="0"/><a:ext cx="20" cy="20"/></a:xfrm></p:spPr></p:sp></p:spTree></p:cSld></p:sld>'.format(P,A)})
        report = documents.inspect_data(path)
        self.assertEqual(report['machine_checks']['issues'][0]['code'], 'off_slide_bounds')
        self.assertIn('no text-overflow', report['machine_checks']['summary']['geometry_limit'])

    def test_formula_errors_missing_cache_and_external_formula_are_distinct(self):
        path = self.package('input.xlsx', {
            'xl/workbook.xml': '<workbook xmlns="{}"/>'.format(S),
            'xl/worksheets/sheet1.xml': '<worksheet xmlns="{}"><sheetData><row><c r="A1"><f>SUM(B1:B2)</f><v/></c><c r="A2" t="e"><f>1/0</f><v>#DIV/0!</v></c><c r="A3"><f>WEBSERVICE("https://invalid.example")</f><v>0</v></c></row></sheetData></worksheet>'.format(S)})
        report = documents.inspect_data(path)
        self.assertEqual({i['code'] for i in report['machine_checks']['issues']},
                         {'formula_cache_missing_or_empty', 'spreadsheet_error', 'external_formula'})
        self.assertNotIn('invalid.example', json.dumps(report))

    def test_structured_table_formula_is_not_an_external_link(self):
        path = self.package('input.xlsx', {
            'xl/workbook.xml': '<workbook xmlns="{}"/>'.format(S),
            'xl/worksheets/sheet1.xml': '<worksheet xmlns="{}"><sheetData><row><c r="A1"><f>SUM(Table1[Amount])</f><v>25</v></c></row></sheetData></worksheet>'.format(S)})
        report = documents.inspect_data(path)
        self.assertFalse(report['machine_checks']['issues'])

    def test_macro_content_type_is_detected_even_with_nonstandard_filename(self):
        path = self.root / 'macro.docx'
        with zipfile.ZipFile(path, 'w') as archive:
            archive.writestr('[Content_Types].xml', '<Types><Override ContentType="application/vnd.ms-word.document.macroEnabled.main+xml"/></Types>')
            archive.writestr('word/document.xml', '<w:document xmlns:w="{}"/>'.format(W))
        report = documents.inspect_data(path)
        self.assertEqual(report['machine_checks']['issues'][0]['code'], 'active_content_type')

    def test_missing_tool_fails_before_output_creation(self):
        path = self.docx()
        out = self.root / 'render'
        with patch.object(documents.shutil, 'which', return_value=None):
            with self.assertRaisesRegex(ValueError, 'HARNESS_SOFFICE'):
                documents.render_command(path, out)
        self.assertFalse(out.exists())

    def test_zero_exit_without_expected_file_is_not_success(self):
        path = self.docx()
        out = self.root / 'render';out.mkdir()
        with patch.object(documents, 'executable', return_value='/fake/tool'), patch.object(documents, 'run', return_value={'exit_code':0}):
            with self.assertRaisesRegex(ValueError, 'produced no PDF'):
                documents.render_into(path, out)
        self.assertFalse((out / 'report.json').exists())

    def test_timeout_is_clear_and_bounded(self):
        with patch.object(documents.subprocess, 'Popen') as launch, patch.object(documents, 'stop_process_tree') as stop:
            launch.return_value.wait.side_effect = subprocess.TimeoutExpired('renderer', 1)
            with self.assertRaisesRegex(ValueError, 'timed out'):
                documents.run(['/fake/renderer'], timeout=1)
            stop.assert_called_once_with(launch.return_value)
            self.assertEqual(launch.call_args.kwargs['start_new_session'], documents.os.name != 'nt')

    def test_keyboard_interrupt_stops_active_tool_tree(self):
        with patch.object(documents.subprocess, 'Popen') as launch, patch.object(documents, 'stop_process_tree') as stop:
            launch.return_value.wait.side_effect = KeyboardInterrupt
            with self.assertRaises(KeyboardInterrupt):
                documents.run(['/fake/renderer'])
            stop.assert_called_once_with(launch.return_value)

    @unittest.skipIf(os.name == 'nt', 'POSIX SIGTERM and process group regression')
    def test_terminated_cli_stops_renderer_descendants_and_cleans_staging(self):
        path = self.docx()
        ready = self.root / 'renderer-ready.json'
        heartbeat = self.root / 'descendant-heartbeat'
        fake = self.root / 'fake-soffice'
        child_code = 'from pathlib import Path; import sys,time\nwhile True:\n Path(sys.argv[1]).write_text(str(time.monotonic()))\n time.sleep(.02)\n'
        fake.write_text('#!' + sys.executable + '\n'
                        'import json,os,subprocess,sys,time\n'
                        'from pathlib import Path\n'
                        'child=subprocess.Popen([sys.executable,"-c",' + repr(child_code) + ',' + repr(str(heartbeat)) + '])\n'
                        'Path(' + repr(str(ready)) + ').write_text(json.dumps({"pid":os.getpid(),"input":sys.argv[-1]}))\n'
                        'time.sleep(30)\n')
        fake.chmod(0o700)
        env = dict(os.environ)
        for key in ('HARNESS_SOFFICE', 'HARNESS_PDFINFO', 'HARNESS_PDFTOPPM'):
            env[key] = str(fake)
        process = subprocess.Popen([sys.executable, str(ROOT / 'scripts/workbench/documents.py'),
                                    'render', str(path), '--output', str(self.root / 'render')],
                                   env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                   text=True, start_new_session=True)
        renderer = None
        try:
            deadline = time.monotonic() + 5
            while time.monotonic() < deadline and not (ready.exists() and heartbeat.exists()):
                time.sleep(.02)
            self.assertTrue(ready.exists() and heartbeat.exists(), 'renderer fixture did not start')
            renderer = json.loads(ready.read_text())
            self.assertTrue(Path(renderer['input']).is_file())
            process.terminate()
            _, error = process.communicate(timeout=5)
            self.assertEqual(process.returncode, 130)
            self.assertIn('interrupted; active tool processes stopped', error)
            self.assertNotIn('Traceback', error)
            self.assertFalse(Path(renderer['input']).parent.exists(), 'temporary staging survived interruption')
            final_heartbeat = heartbeat.read_text()
            time.sleep(.15)
            self.assertEqual(heartbeat.read_text(), final_heartbeat, 'renderer descendant is still running')
            with self.assertRaises(ProcessLookupError):
                os.kill(renderer['pid'], 0)
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=5)
            if renderer:
                try:
                    os.killpg(renderer['pid'], signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def test_page_limit_rejects_huge_rasterization(self):
        with patch.object(documents, 'run', return_value={'stdout':'Pages: 100000\n'}):
            with self.assertRaisesRegex(ValueError, 'page count outside'):
                documents.pdf_pages(Path('input.pdf'), 'pdfinfo')

    def test_pdf_inspection_states_its_limits(self):
        path = self.root / 'input.pdf';path.write_bytes(b'%PDF-1.7\n/JavaScript')
        report = documents.inspect_data(path)
        self.assertEqual(report['machine_checks']['issues'][0]['code'], 'possible_active_pdf_content')
        self.assertIn('not validated', report['machine_checks']['summary']['inspection_limit'])


if __name__ == '__main__':
    unittest.main()
