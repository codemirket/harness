"""Synthetic-only assembly tests: no official archive or runtime execution."""
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import stat
import tarfile
import tempfile
import unittest
import warnings
import zipfile

SCRIPT = Path(__file__).resolve().parents[1] / "scripts/assemble-m0-runtime.py"
SPEC = importlib.util.spec_from_file_location("assemble_m0_runtime", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class AssemblyTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        self.addCleanup(self.temporary.cleanup)
        self.output = self.root / "release"
        self.output.mkdir()
        self.manifest = self.root / "fixture.json"

    def fixture(self, kind="tar", changes=None, entries=None):
        target = "aarch64-apple-darwin" if kind == "tar" else "x86_64-pc-windows-msvc"
        binary = "bin/node" if kind == "tar" else "node.exe"
        self.archive = self.root / ("fixture.tar.gz" if kind == "tar" else "fixture.zip")
        self.target = target
        self.payload = {"fixture/" + binary: b"synthetic-not-executable\x00\xff", "fixture/LICENSE": b"test license\r\n"}
        members = entries if entries is not None else [(n, "file", data) for n, data in self.payload.items()] + [("fixture/ignored", "file", b"not selected")]
        if kind == "tar":
            with tarfile.open(self.archive, "w:gz") as archive:
                for name, filekind, data in members:
                    info = tarfile.TarInfo(name)
                    info.size = len(data) if filekind == "file" else 0
                    if filekind == "symlink":
                        info.type = tarfile.SYMTYPE
                        info.linkname = data.decode()
                    elif filekind == "hardlink":
                        info.type = tarfile.LNKTYPE
                        info.linkname = data.decode()
                    archive.addfile(info, io.BytesIO(data) if filekind == "file" else None)
        else:
            with warnings.catch_warnings():
                warnings.simplefilter("ignore", UserWarning)
                with zipfile.ZipFile(self.archive, "w") as archive:
                    for name, filekind, data in members:
                        info = zipfile.ZipInfo(name)
                        info.create_system = 3
                        info.external_attr = ((stat.S_IFLNK if filekind == "symlink" else stat.S_IFREG) | 0o644) << 16
                        archive.writestr(info, data)
        archive_bytes = self.archive.read_bytes()
        files = [{"path": n, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "destination_relative": "runtime/node/" + n.removeprefix("fixture/"), "mode": "0755" if n.endswith("bin/node") else "0644", "content_transform": "none"} for n, data in self.payload.items()]
        self.selection = {"target": target, "archive": self.archive.name, "archive_bytes": len(archive_bytes), "archive_sha256": hashlib.sha256(archive_bytes).hexdigest(), "selected_file_bytes": sum(x["bytes"] for x in files), "selected_files": files}
        if changes:
            changes(self.selection)
        self.save_manifest()

    def save_manifest(self):
        self.manifest.write_text(json.dumps({"targets": [self.selection]}), encoding="utf-8")

    def assemble(self):
        return MODULE.assemble(self.root, self.target, self.output, self.manifest)

    def rejected(self):
        with self.assertRaises((MODULE.AssemblyError, OSError)):
            self.assemble()
        self.assertEqual(list(self.output.iterdir()), [])
        self.assertEqual(list(self.root.glob(".m0-runtime-*")), [])

    def test_tar_and_zip_exact_selection(self):
        for kind in ("tar", "zip"):
            with self.subTest(kind=kind):
                if any(self.output.iterdir()):
                    self.output = self.root / "release-zip"
                    self.output.mkdir()
                self.fixture(kind)
                result = self.assemble()
                self.assertFalse(result["runtime_executed"])
                self.assertEqual(len([p for p in self.output.rglob("*") if p.is_file()]), 2)
                for entry in self.selection["selected_files"]:
                    destination = self.output / entry["destination_relative"]
                    self.assertEqual(destination.read_bytes(), self.payload[entry["path"]])
                    if os.name != "nt":
                        self.assertEqual(stat.S_IMODE(destination.stat().st_mode), int(entry["mode"], 8))

    def test_wrong_archive_digest(self):
        self.fixture(changes=lambda s: s.update(archive_sha256="0" * 64))
        self.rejected()

    def test_wrong_selected_digest_leaves_output_empty(self):
        self.fixture(changes=lambda s: s["selected_files"][1].update(sha256="0" * 64))
        self.rejected()

    def test_wrong_member_size(self):
        self.fixture(changes=lambda s: (s["selected_files"][0].update(bytes=1), s.update(selected_file_bytes=15)))
        self.rejected()

    def test_missing_member(self):
        self.fixture(entries=[("fixture/LICENSE", "file", b"test license\r\n")])
        self.rejected()

    def test_duplicates_even_when_not_selected(self):
        for kind in ("tar", "zip"):
            with self.subTest(kind=kind):
                binary = "bin/node" if kind == "tar" else "node.exe"
                entries = [("fixture/" + binary, "file", b"synthetic-not-executable\x00\xff"), ("fixture/LICENSE", "file", b"test license\r\n"), ("fixture/ignored", "file", b"1"), ("fixture/ignored", "file", b"2")]
                self.fixture(kind, entries=entries)
                self.rejected()

    def test_selected_symlinks_rejected(self):
        for kind in ("tar", "zip"):
            with self.subTest(kind=kind):
                binary = "bin/node" if kind == "tar" else "node.exe"
                self.fixture(kind, entries=[("fixture/" + binary, "symlink", b"elsewhere"), ("fixture/LICENSE", "file", b"test license\r\n")])
                self.rejected()

    def test_unselected_symlink_is_not_materialized(self):
        self.fixture(entries=[("fixture/bin/node", "file", b"synthetic-not-executable\x00\xff"), ("fixture/LICENSE", "file", b"test license\r\n"), ("fixture/bin/npm", "symlink", b"../lib/node_modules/npm/bin/npm-cli.js")])
        self.assemble()
        self.assertFalse((self.output / "runtime/node/bin/npm").exists())

    def test_hardlink_rejected(self):
        self.fixture(entries=[("fixture/ignored", "hardlink", b"fixture/LICENSE")])
        self.rejected()

    def test_traversal_and_nonportable_names_rejected_globally(self):
        for kind in ("tar", "zip"):
            for name in ("../escape", "/absolute", "fixture/../escape", "fixture//alias", "C:/escape", "fixture\\escape"):
                with self.subTest(kind=kind, name=name):
                    self.fixture(kind, entries=[(name, "file", b"bad")])
                    self.rejected()
        self.assertFalse((self.root.parent / "escape").exists())

    def test_manifest_destination_cannot_escape(self):
        self.fixture(changes=lambda s: s["selected_files"][0].update(destination_relative="../escape"))
        self.rejected()

    def test_nonempty_output_preserved(self):
        self.fixture()
        original = self.output / "user.txt"
        original.write_bytes(b"preserve")
        with self.assertRaises(MODULE.AssemblyError):
            self.assemble()
        self.assertEqual(original.read_bytes(), b"preserve")
        self.assertEqual(list(self.output.iterdir()), [original])

    def test_relative_or_missing_output_rejected(self):
        self.fixture()
        for path in (Path("relative-output"), self.root / "absent"):
            with self.subTest(path=str(path)), self.assertRaises(MODULE.AssemblyError):
                MODULE.assemble(self.root, self.target, path, self.manifest)
        self.assertFalse((self.root / "absent").exists())

    def test_symlink_output_and_archive_rejected(self):
        self.fixture()
        link = self.root / "linked-release"
        try:
            link.symlink_to(self.output, target_is_directory=True)
        except OSError:
            if os.name == "nt":
                self.skipTest("Windows host does not permit creating fixture symlinks")
            raise
        with self.assertRaises(MODULE.AssemblyError):
            MODULE.assemble(self.root, self.target, link, self.manifest)
        real = self.archive.with_name("real.tar.gz")
        self.archive.rename(real)
        self.archive.symlink_to(real)
        self.rejected()

    def test_repeat_assembly_refuses_existing_payload(self):
        self.fixture()
        self.assemble()
        original = (self.output / "runtime/node/bin/node").read_bytes()
        with self.assertRaises(MODULE.AssemblyError):
            self.assemble()
        self.assertEqual((self.output / "runtime/node/bin/node").read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
