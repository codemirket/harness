import hashlib
import io
import tarfile
import unittest
import tempfile
from pathlib import Path
from image_audit import evaluate, inventory, validate_export, ROOT, BUILT
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
    def test_candidate_scan_rejects_old_installed_and_wrong_source_archives(self):
        with tempfile.TemporaryDirectory() as temporary:
            archive = Path(temporary) / 'image.tar'
            archive.write_bytes(b'fixture')
            receipt = {'ok':True,'alias':'penpot-backend','candidate':True,
                       'source_sha256':'a'*64,'archive':str(archive),
                       'archive_sha256':hashlib.sha256(b'fixture').hexdigest()}
            self.assertEqual(validate_export(receipt,'penpot-backend','a'*64),archive.resolve())
            for change in [{'candidate':False},{'source_sha256':'b'*64},
                           {'alias':'design-tools'},{'archive_sha256':'0'*64}]:
                with self.assertRaises(ValueError):
                    validate_export(receipt | change,'penpot-backend','a'*64)
            self.assertIn('penpot-backend',BUILT)
            self.assertNotIn('penpot-backend',inventory(ROOT))
            self.assertIn('penpot-backend-payload',inventory(ROOT))
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

    def test_unfixed_and_language_findings_remain_actionable(self):
        report = self.fixture()
        report["Results"].append({"Class": "lang-pkgs", "Type": "node-pkg",
            "Packages": [{"Name": "fixture"}], "Vulnerabilities": [{
                "VulnerabilityID": "CVE-fixture", "PkgName": "fixture",
                "InstalledVersion": "1.0", "FixedVersion": "", "Severity": "HIGH",
                "Description": "PRIVATE_UNNEEDED_CONTENT"}]})
        result = evaluate(report, "pinned")
        self.assertFalse(result["ok"])
        self.assertEqual(result["packages_by_type"]["node-pkg"], 1)
        self.assertEqual(result["findings"][0]["FixedVersion"], "")
        self.assertNotIn("PRIVATE_UNNEEDED_CONTENT", str(result))
        report["Results"].pop(0)
        incomplete = evaluate(report, "pinned")
        self.assertFalse(incomplete["inventory_complete"])
        self.assertEqual(len(incomplete["findings"]), 1)
        self.assertEqual(len(inventory(ROOT)), 10)
