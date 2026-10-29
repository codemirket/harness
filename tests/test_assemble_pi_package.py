"""Synthetic package assembly contract; no native executable is run."""

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/assemble-pi-package.py"
SPEC = importlib.util.spec_from_file_location("assemble_pi_package", SCRIPT)
ASSEMBLER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ASSEMBLER)


def digest(data):
    return hashlib.sha256(data).hexdigest()


class PiPackageAssemblyTests(unittest.TestCase):
    def test_assembles_bound_package_without_repository_root_license(self):
        with tempfile.TemporaryDirectory(prefix="mirket-pi-package-") as temporary:
            root = Path(temporary).resolve()

            def put(relative, data):
                target = root / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
                return target

            verifier = put("verifier/mirket", b"selected-core")
            source = root / "native-source"
            put("native-source/bin/mirket", verifier.read_bytes())
            node = put("native-source/runtime/node/bin/node", b"private-node")
            put("pi/dist/extension/index.js", b"compiled extension")
            put("pi/scripts/setup.mjs", b"setup script")
            put("packaging/admission/pi-host-files.json.proposed", b"{}\n")
            put("scripts/assemble-skills.py", b"def inventory(root):\n    return {'catalog.json': b'{}\\n'}\n")
            self.assertFalse((root / "LICENSE").exists())

            verified = {
                "release_id": "sha256:synthetic-release",
                "manifest": {
                    "version": "0.1.0",
                    "files": [
                        {"relative_path": "bin/mirket", "sha256": digest(verifier.read_bytes())},
                        {"relative_path": "runtime/node/bin/node", "sha256": digest(node.read_bytes())},
                    ],
                },
            }
            output = root / "package"
            with patch.object(ASSEMBLER, "ROOT", root), \
                 patch.object(ASSEMBLER.subprocess, "check_output", return_value=json.dumps(verified).encode()) as verify, \
                 patch.object(ASSEMBLER.subprocess, "run") as copied_verify:
                result = ASSEMBLER.assemble(verifier, source, output)

            verify.assert_called_once_with([str(verifier), "verify", "--source", str(source)], timeout=60)
            copied_verify.assert_called_once_with(
                [str(verifier), "verify", "--source", str(output / "native")],
                check=True, stdout=subprocess.DEVNULL, timeout=60,
            )
            self.assertEqual(result["release_id"], verified["release_id"])
            self.assertFalse((output / "LICENSE").exists())
            inventory = json.loads((output / "package-inventory.json").read_text())
            paths = {entry["path"] for entry in inventory["files"]}
            self.assertNotIn("LICENSE", paths)
            self.assertIn("native/bin/mirket", paths)
            self.assertIn("scripts/setup.mjs", paths)
            self.assertIn("dist/extension/index.js", paths)
            for entry in inventory["files"]:
                payload = (output / entry["path"]).read_bytes()
                self.assertEqual(entry["bytes"], len(payload))
                self.assertEqual(entry["sha256"], digest(payload))
            binding = json.loads((output / "native-binding.json").read_text())
            self.assertEqual(binding["releaseId"], verified["release_id"])
            self.assertEqual(binding["coreSha256"], digest(verifier.read_bytes()))
            self.assertEqual(binding["nodeSha256"], digest(node.read_bytes()))
            self.assertEqual(binding["packageSha256"], digest((output / "package.json").read_bytes()))


if __name__ == "__main__":
    unittest.main()
