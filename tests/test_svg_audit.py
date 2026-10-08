"""Observable structural failures in the optional SVG authoring helper."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'skills/svg-creation/scripts/audit_svg.py'
SPEC = importlib.util.spec_from_file_location('svg_audit', SCRIPT)
audit_module = importlib.util.module_from_spec(SPEC)
# Imported skill helpers must not create unreviewed bytes in their payload.
previous = sys.dont_write_bytecode
sys.dont_write_bytecode = True
try:
    SPEC.loader.exec_module(audit_module)
finally:
    sys.dont_write_bytecode = previous


class SvgAuditTests(unittest.TestCase):
    def inspect(self, body='', box='0 0 24 24'):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'art.svg'
            path.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="' + box + '">' + body + '</svg>')
            return audit_module.audit(path)

    def test_valid_definition_href_and_accessibility_references(self):
        result = self.inspect('<title id="label">Boat</title><defs><path id="hull" d="M0 0H20"/></defs>'
                              '<g aria-labelledby="label"><use href="#hull"/></g>')
        self.assertFalse(result['errors'])
        self.assertEqual(result['local_references'], 2)

    def test_duplicate_ids_rejected(self):
        self.assertIn('Duplicate ID in asset.', self.inspect('<g id="a"/><path id="a"/>')['errors'])

    def test_missing_paint_clip_mask_and_style_references_rejected(self):
        for body in ('<path fill="url(#missing)"/>', '<g clip-path="url(\' #missing \')"/>',
                     '<style>.a { mask: url("#missing"); }</style>'):
            with self.subTest(body=body):
                self.assertTrue(self.inspect(body)['errors'])

    def test_missing_aria_target_rejected(self):
        self.assertTrue(self.inspect('<g aria-describedby="missing"/>')['errors'])

    def test_viewbox_invalid_or_degenerate_rejected(self):
        for box in ('', '0 0 24', '0 0 0 24', '0 0 -1 24', '0 0 nan 24', '0 0 1e999 24',
                    '0 0 2_4 24', '0,,0,24,24', '٠ ٠ ٢٤ ٢٤', '0\u00a00 24 24', '0 0 24 24,'):
            with self.subTest(box=box):
                self.assertTrue(self.inspect(box=box)['errors'])

    def test_valid_viewbox_exponents_commas_and_negative_origin(self):
        self.assertFalse(self.inspect(box='-1,-.5,2.4e1,24')['errors'])
        self.assertFalse(self.inspect(box=' -1 , -.5  ,  24,24 ')['errors'])

    def test_css_comments_strings_and_metadata_are_not_references(self):
        result = self.inspect('<style>/* old fill: url(#removed) */ .a { content: "url(#example)"; }</style>'
                              '<g data-note="url(#not-a-resource)" style="--label: \'url(#literal)\';"/>')
        self.assertFalse(result['errors'])
        self.assertEqual(result['local_references'], 0)

    def test_commented_import_is_not_a_dependency(self):
        result = self.inspect('<style>/* @import "old.css"; */ .a { content: "@import"; }</style>')
        self.assertFalse(result['warnings'])

    def test_non_svg_root_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'bad.svg'
            path.write_text('<svg viewBox="0 0 24 24"/>')
            self.assertTrue(audit_module.audit(path)['errors'])

    def test_malformed_xml_does_not_echo_source(self):
        secret = 'private-token-example'
        result = self.inspect('<' + secret)
        self.assertTrue(result['errors'])
        self.assertNotIn(secret, json.dumps(result))

    def test_dtd_not_expanded(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'bad.svg'
            path.write_text('<!DOCTYPE svg [<!ENTITY x "example">]><svg/>')
            self.assertTrue(audit_module.audit(path)['errors'])

    def test_warning_content_is_reported_without_echoing_dependencies(self):
        result = self.inspect('<image href="data:image/png;base64,example"/><foreignObject/>'
                              '<script/><g onclick="private-example()"/><text>hello</text>'
                              '<style>@import "https://example.invalid/private-token";</style>')
        self.assertFalse(result['errors'])
        self.assertEqual(len(result['warnings']), 5)
        self.assertNotIn('private-token', json.dumps(result))

    def test_size_limit(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'large.svg'
            path.write_bytes(b' ' * (audit_module.MAX_BYTES + 1))
            self.assertTrue(audit_module.audit(path)['errors'])

    def test_cli_status_and_json_for_mixed_files(self):
        with tempfile.TemporaryDirectory() as directory:
            good, bad = Path(directory) / 'good.svg', Path(directory) / 'bad.svg'
            good.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"/>')
            bad.write_text('<svg/>')
            result = subprocess.run([sys.executable, str(SCRIPT), str(good), str(bad), '--json'],
                                    capture_output=True, text=True, timeout=5)
            self.assertEqual(result.returncode, 1)
            reports = json.loads(result.stdout)['reports']
            self.assertFalse(reports[0]['errors'])
            self.assertTrue(reports[1]['errors'])


if __name__ == '__main__':
    unittest.main()
