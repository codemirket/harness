import hashlib
import io
import tarfile
import unittest
import tempfile
from pathlib import Path
from image_audit import evaluate, inventory, validate_export, ROOT, BUILT, REQUIRED_TYPES
from fetch_scanner import binary


class ImageTests(unittest.TestCase):
    def test_stripped_go_metadata_cannot_make_esbuild_advisories_look_fixed(self):
        report = self.fixture()
        report["Results"].append({"Type":"node-pkg","Packages":[{"Name":"esbuild","Version":"0.28.2"}]})
        result = evaluate(report,"pinned",required_types={"node-pkg","gobinary"})
        self.assertFalse(result["inventory_complete"])
        self.assertIn("gobinary",str(result["coverage_errors"]))
        report["Results"].append({"Type":"gobinary","Packages":[{"Name":"stdlib","Version":"v1.27.3"}]})
        self.assertTrue(evaluate(report,"pinned",required_types={"node-pkg","gobinary"})["ok"])
    def test_current_design_inventory_requires_node_and_owned_d2_metadata(self):
        report = self.fixture()
        self.assertFalse(evaluate(report, 'pinned', required_types=REQUIRED_TYPES['design-tools'])['ok'])
        report['Results'].append({'Type': 'node-pkg', 'Packages': [{'Name': '@open-pencil/cli'}]})
        report['Results'].append({'Type':'gobinary','Packages':[{'Name':'stdlib','Version':'v1.27.3'}]})
        self.assertTrue(evaluate(report, 'pinned', required_types=REQUIRED_TYPES['design-tools'])['ok'])

    def test_candidate_scan_rejects_old_installed_and_wrong_source_archives(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / 'image.tar'
            archive.write_bytes(b'fixture')
            receipt = {'ok':True,'alias':'design-tools','candidate':True,
                       'source_sha256':'a'*64,'archive':str(archive),
                       'archive_sha256':hashlib.sha256(b'fixture').hexdigest()}
            self.assertEqual(validate_export(receipt,'design-tools','a'*64),archive.resolve())
            for change in [{'candidate':False},{'source_sha256':'b'*64},
                           {'alias':'agent-runtime'},{'archive_sha256':'0'*64}]:
                with self.assertRaises(ValueError):
                    validate_export(receipt | change,'design-tools','a'*64)
            self.assertIn('design-tools',BUILT)
            self.assertNotIn('design-tools',inventory(ROOT))
            self.assertIn('node-build',inventory(ROOT))
            self.assertFalse(any('penpot' in alias for alias in inventory(ROOT)))
    def test_scanner_pin_and_regular_member_required_before_execution(self):
        data = io.BytesIO()
        with tarfile.open(fileobj=data, mode="w") as archive:
            member = tarfile.TarInfo("trivy")
            member.type = tarfile.SYMTYPE
            member.linkname = "/private/tool"
            archive.addfile(member)
        payload = data.getvalue()
        with self.assertRaises(ValueError): binary(payload, "0" * 64)
        with self.assertRaises(ValueError): binary(payload, hashlib.sha256(payload).hexdigest())

    def fixture(self):
        return {"SchemaVersion": 2, "ArtifactType": "container_image", "ArtifactName": "pinned",
                "Metadata": {"ImageID": "sha256:fixture", "OS": {"Family": "debian", "Name": "13"},
                             "ImageConfig": {"architecture": "arm64", "os": "linux"}},
                "Results": [{"Class": "os-pkgs", "Type": "debian", "Packages": [{"Name": "libc"}]}]}

    def test_missing_inventory_wrong_target_and_eol_do_not_pass(self):
        self.assertTrue(evaluate(self.fixture(), "pinned")["ok"])
        for mutation in [
            lambda r: r["Metadata"]["ImageConfig"].update(architecture="amd64"),
            lambda r: r.update(ArtifactName="other"),
            lambda r: r.update(SchemaVersion=0),
        ]:
            report = self.fixture()
            mutation(report)
            with self.assertRaises(ValueError): evaluate(report, "pinned")
        with self.assertRaises(ValueError): evaluate(self.fixture(), "sha256:other", True)
        self.assertTrue(evaluate(self.fixture(), "sha256:fixture", True)["ok"])
        for mutation in [lambda r: r.update(Results=[]),
                         lambda r: r["Metadata"]["OS"].update(EOSL=True)]:
            report = self.fixture()
            mutation(report)
            self.assertFalse(evaluate(report, "pinned")["ok"])

    def test_report_policy_retains_advisories_but_never_hides_broken_inventory(self):
        report = self.fixture()
        report["Results"][0]["Vulnerabilities"] = [{"VulnerabilityID":"CVE-fixture", "PkgName":"libc", "Severity":"CRITICAL"}]
        self.assertFalse(evaluate(report,"pinned",advisory_policy="fail")["ok"])
        accepted = evaluate(report,"pinned",advisory_policy="report")
        self.assertTrue(accepted["ok"])
        self.assertTrue(accepted["advisories_present"])
        self.assertEqual(accepted["findings"][0]["Severity"],"CRITICAL")
        self.assertFalse(evaluate(report,"pinned",required_types={"gobinary"},advisory_policy="report")["ok"])
        report["Results"][0]["Packages"] = []
        self.assertFalse(evaluate(report,"pinned",advisory_policy="report")["ok"])
        with self.assertRaises(ValueError): evaluate(report,"other",advisory_policy="report")
        with self.assertRaises(ValueError): evaluate(report,"pinned",advisory_policy="ignore-everything")

    def test_unfixed_and_language_findings_remain_actionable(self):
        report = self.fixture()
        report["Results"].append({"Class": "lang-pkgs", "Type": "node-pkg",
            "Packages": [{"Name": "fixture"}], "Vulnerabilities": [{
                "VulnerabilityID": "CVE-fixture", "PkgName": "fixture",
                "InstalledVersion": "1.0", "FixedVersion": "", "Severity": "HIGH",
                "Description": "PRIVATE_UNNEEDED_CONTENT"}]})
        result = evaluate(report, "pinned", advisory_policy="fail")
        self.assertFalse(result["ok"])
        self.assertEqual(result["packages_by_type"]["node-pkg"], 1)
        self.assertEqual(result["findings"][0]["FixedVersion"], "")
        self.assertNotIn("PRIVATE_UNNEEDED_CONTENT", str(result))
        report["Results"].pop(0)
        incomplete = evaluate(report, "pinned")
        self.assertFalse(incomplete["inventory_complete"])
        self.assertEqual(len(incomplete["findings"]), 1)
        self.assertEqual(len(inventory(ROOT)), 4)

    def test_findings_retain_artifact_paths_and_severity_provenance(self):
        report = self.fixture()
        report['Results'][0].update(Target='Debian 13', Packages=[{
            'Name':'linux-libc-dev','Version':'6.12','SrcName':'linux','SrcVersion':'6.12', 'Arch':'arm64'}],
            Vulnerabilities=[{'VulnerabilityID':'CVE-fixture','PkgName':'linux-libc-dev',
                'PkgPath':'usr/include/linux','Severity':'CRITICAL','SeveritySource':'nvd',
                'VendorSeverity':{'debian':1,'nvd':4},'DataSource':{'ID':'debian'}}])
        result = evaluate(report, 'pinned')
        self.assertEqual(result['packages'][0]['SrcName'], 'linux')
        self.assertEqual(result['findings'][0]['PkgPath'], 'usr/include/linux')
        self.assertEqual(result['findings'][0]['VendorSeverity'], {'debian':1,'nvd':4})
        self.assertEqual(result['findings'][0]['artifact'], 'pinned')
        self.assertIn('not_measured', result['kernel_coverage'])

    def test_missing_native_and_browser_bytes_fail_independently_of_scanner(self):
        from unittest.mock import patch
        from image_contents import payload_inventory
        file = {'type':'file','sha256':'a'*64,'size':10}
        entries = {'usr/local/bin/node':file,'usr/local/bin/d2':file,'usr/local/bin/typst':file,
                   'opt/design/chrome/chrome-linux-arm64/chrome':file,
                   'opt/design/node_modules/@img/sharp-libvips-linux-arm64/lib/libvips-cpp.so.42':file,
                   'opt/design/node_modules/@resvg/resvg-js-linux-arm64-gnu/resvgjs.linux-arm64-gnu.node':file,
                   'opt/design/node_modules/canvaskit-wasm/bin/canvaskit.wasm':file}
        with patch('image_contents.contents',return_value=entries):
            self.assertTrue(payload_inventory('unused','design-tools')['inventory_complete'])
        del entries['opt/design/chrome/chrome-linux-arm64/chrome']
        with patch('image_contents.contents',return_value=entries):
            self.assertIn('chromium', payload_inventory('unused','design-tools')['missing'])
        with patch('image_contents.contents',return_value={'opt/agent/mirket-embed':file}):
            self.assertIn('onnx-runtime', payload_inventory('unused','agent-runtime')['missing'])
