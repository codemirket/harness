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
    def test_bundled_media_requires_matching_sources_and_copies_them(self):
        with tempfile.TemporaryDirectory(prefix="mirket-media-source-") as temporary:
            root = Path(temporary).resolve()
            source, packet, output = (root / name for name in ("native", "packet", "package"))
            output.mkdir()
            packet.mkdir()
            files = {}
            binaries = []
            for name in ("ffmpeg", "ffprobe"):
                path = source / "runtime/video/bin" / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(name.encode())
                files["runtime/video/bin/" + name] = {"sha256": digest(path.read_bytes())}
                binaries.append({"path": "bin/" + name, "bytes": path.stat().st_size, "sha256": digest(path.read_bytes())})
            with self.assertRaisesRegex(ValueError, "--ffmpeg-source-root"):
                ASSEMBLER.copy_media_sources(source, files, output, None)
            notice = packet / "COPYING"
            notice.write_bytes(b"exact notice")
            pin_dir = root / "packaging/runtime"
            pin_dir.mkdir(parents=True)
            (pin_dir / "ffmpeg-source.json").write_text(json.dumps({"binaries": binaries, "files": [
                {"path": "COPYING", "bytes": notice.stat().st_size, "sha256": digest(notice.read_bytes())}]}))
            (root / "scripts").mkdir()
            (root / "scripts/rebuild-ffmpeg.py").write_text("replay recipe")
            (root / "packaging/ffmpeg-source.md").write_text("source instructions")
            with patch.object(ASSEMBLER, "ROOT", root):
                notice.write_bytes(b"drift")
                with self.assertRaisesRegex(ValueError, "file differs"):
                    ASSEMBLER.copy_media_sources(source, files, output, packet)
                self.assertFalse((output / "third-party").exists())
                notice.write_bytes(b"exact notice")
                ASSEMBLER.copy_media_sources(source, files, output, packet)
            self.assertEqual((output / "third-party/ffmpeg/COPYING").read_bytes(), b"exact notice")
            self.assertTrue((output / "third-party/ffmpeg/rebuild-ffmpeg.py").is_file())
            self.assertTrue((output / "third-party/ffmpeg/source-files.json").is_file())

    def test_assembles_bound_package_with_exact_repository_license(self):
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
            put("packaging/runtime/host-files.json", b"{}\n")
            put("packaging/notices/sources.json", b'{"files":[]}')
            put("packaging/notices/README.md", b"fixture notice attribution")
            put("packaging/THIRD-PARTY.md", b"fixture notice scope")
            put("packaging/runtime/chrome-macos.json", b'{"version":"156.0.8078.73","target":"aarch64-apple-darwin","archive":{},"executable":"runtime/browser/chrome", "files":[], "links":[]}')
            put("scripts/assemble-skills.py", b"def inventory(root):\n    return {'catalog.json': b'{}\\n'}\n")
            license_bytes = b"fixture project license\n"
            put("LICENSE", license_bytes)

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
            self.assertEqual((output / "LICENSE").read_bytes(), license_bytes)
            inventory = json.loads((output / "package-inventory.json").read_text())
            paths = {entry["path"] for entry in inventory["files"]}
            self.assertIn("LICENSE", paths)
            self.assertIn("native/bin/mirket", paths)
            self.assertIn("scripts/setup.mjs", paths)
            self.assertIn("runtime-setup.json", paths)
            setup = json.loads((output / "runtime-setup.json").read_text())
            self.assertEqual([tool["id"] for tool in setup["tools"]], ["chromium"])
            self.assertEqual([tool["id"] for tool in setup["tools"] if "archive" in tool], ["chromium"])
            self.assertIn("dist/extension/index.js", paths)
            for entry in inventory["files"]:
                payload = (output / entry["path"]).read_bytes()
                self.assertEqual(entry["bytes"], len(payload))
                self.assertEqual(entry["sha256"], digest(payload))
            binding = json.loads((output / "native-binding.json").read_text())
            self.assertEqual(binding["releaseId"], verified["release_id"])
            self.assertEqual(binding["runtimeSetupSha256"], digest((output / "runtime-setup.json").read_bytes()))
            self.assertEqual(binding["coreSha256"], digest(verifier.read_bytes()))
            self.assertEqual(binding["nodeSha256"], digest(node.read_bytes()))
            self.assertEqual(binding["packageSha256"], digest((output / "package.json").read_bytes()))


if __name__ == "__main__":
    unittest.main()
