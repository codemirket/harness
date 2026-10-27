"""Keep setup's fixed semantic downloads aligned with the compiled admission."""

import hashlib
import json
from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]
PINS = ROOT / "packaging/admission/m2-semantic-minilm.runtime.json"
INSTALLER = ROOT / "scripts/install-macos.sh"


class SemanticInstallerPins(unittest.TestCase):
    def test_model_sources_and_receipt_match_admission(self):
        manifest_bytes = PINS.read_bytes()
        pins = json.loads(manifest_bytes)
        installer = INSTALLER.read_text()
        base = (
            "https://huggingface.co/"
            + pins["model"]["repo"]
            + "/resolve/"
            + pins["model"]["revision"]
            + "/"
        )
        entries = re.findall(
            r"^\s*acquire_model_file '([^']+)' '([^']+)' (\d+) ([0-9a-f]{64})$",
            installer,
            re.MULTILINE,
        )
        expected = [
            (
                item["path"],
                item["url"].removeprefix(base),
                str(item["bytes"]),
                item["sha256"],
            )
            for item in pins["model"]["selected"]
        ]
        self.assertEqual(entries, expected)
        self.assertTrue(all(item["url"].startswith(base) for item in pins["model"]["selected"]))

        target = pins["targets"]["aarch64-apple-darwin"]
        files = [
            {key: item[key] for key in ("path", "bytes", "sha256")}
            for item in target["files"]
        ] + [
            {"path": "model/" + item["path"], "bytes": item["bytes"], "sha256": item["sha256"]}
            for item in pins["model"]["selected"]
        ]
        receipt = {
            "schema": 1,
            "status": "private-runtime",
            "target": "aarch64-apple-darwin",
            "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
            "runtime_version": pins["runtime_version"],
            "model_revision": pins["model"]["revision"],
            "provider": "CPUExecutionProvider",
            "telemetry": False,
            "files": files,
        }
        receipt_bytes = (json.dumps(receipt, indent=2) + "\n").encode()
        self.assertIn(
            "require_hash \"$semantic_source/package-receipt.json\" "
            + hashlib.sha256(receipt_bytes).hexdigest(),
            installer,
        )
        self.assertIn(
            f'"$(/usr/bin/stat -f %z "$semantic_source/package-receipt.json")" -eq {len(receipt_bytes)}',
            installer,
        )

    def test_runtime_archive_and_selected_members_match_admission(self):
        pins = json.loads(PINS.read_text())
        installer = INSTALLER.read_text()
        target = pins["targets"]["aarch64-apple-darwin"]
        archive = target["archive"]
        self.assertIn(archive["url"], installer)
        self.assertIn(f'-eq {archive["bytes"]}', installer)
        self.assertIn(f'require_hash "$semantic_archive" {archive["sha256"]}', installer)
        entries = re.findall(
            r"^\s*extract_semantic_file '([^']+)' '([^']+)' (\d+) ([0-9a-f]{64})$",
            installer,
            re.MULTILINE,
        )
        expected = [
            (item["member"], item["path"], str(item["bytes"]), item["sha256"])
            for item in target["files"]
        ]
        self.assertEqual(entries, expected)


if __name__ == "__main__":
    unittest.main()
