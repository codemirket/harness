import json
from pathlib import Path
import tempfile
import unittest
import platform
import subprocess
import contextlib
import io
from unittest.mock import patch
from native_validate import candidate, sha, target_for, validate


class PlatformTests(unittest.TestCase):
    def test_timed_out_native_command_retains_partial_stdout_and_failed_receipt(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            target = target_for(platform.system(), platform.machine())
            name = "mirket-" + target + (".exe" if platform.system() == "Windows" else "")
            binary = root / name
            binary.write_bytes(b"fixture")
            (root / "mirket-release.json").write_text(json.dumps({"assets": [{
                "target": target, "url": "https://example.invalid/" + name, "sha256": sha(binary)}]}))
            out = root / "evidence"
            timeout = subprocess.TimeoutExpired(["fixture"], 180, output=b"partial identity output")
            with patch("native_validate.subprocess.run", side_effect=timeout), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(validate(root, out), 1)
            receipt = json.loads((out / "receipt.json").read_text())
            self.assertFalse(receipt["ok"])
            self.assertTrue(receipt["commands"][0]["timed_out"])
            self.assertEqual((out / "identity.stdout").read_bytes(), b"partial identity output")
            self.assertEqual(receipt["files"]["identity.stdout"], sha(out / "identity.stdout"))

    def test_target_is_native_and_digest_drift_prevents_execution(self):
        self.assertEqual(target_for("Windows", "AMD64"), "x86_64-pc-windows-msvc")
        self.assertEqual(target_for("Linux", "aarch64"), "aarch64-unknown-linux-gnu")
        with self.assertRaises(ValueError): target_for("Linux", "riscv64")
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            target = "x86_64-pc-windows-msvc"
            binary = root / ("mirket-" + target + ".exe")
            binary.write_bytes(b"fixture")
            asset = {"target": target, "url": "https://example.invalid/" + binary.name, "sha256": sha(binary)}
            manifest = root / "mirket-release.json"
            manifest.write_text(json.dumps({"assets": [asset]}))
            self.assertEqual(candidate(root, target)[0], binary)
            with self.assertRaises(ValueError): candidate(root, "aarch64-unknown-linux-gnu")
            binary.write_bytes(b"changed")
            with self.assertRaises(ValueError): candidate(root, target)
            binary.write_bytes(b"fixture")
            manifest.write_text(json.dumps({"assets": [asset, asset]}))
            with self.assertRaises(ValueError): candidate(root, target)
