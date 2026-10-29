"""Exercise the design copier with a small, disposable admitted tree."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "scripts/assemble-design-runtime.py"
SPEC = importlib.util.spec_from_file_location("assemble_design_runtime", SCRIPT)
assembly = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(assembly)


class DesignAssemblyTests(unittest.TestCase):
    def test_selected_files_and_font_without_copied_notices(self):
        with tempfile.TemporaryDirectory(prefix="mirket-design-assembly-") as temporary:
            root = Path(temporary).resolve()
            repository = root / "repository"
            source = root / "source"
            tree = source / "node_modules"
            release = root / "release"
            tree.mkdir(parents=True)
            release.mkdir()
            package = tree / "style-dictionary" / "index.js"
            package.parent.mkdir()
            package.write_bytes(b"selected package")
            font = root / "font.ttf"
            font.write_bytes(b"selected font")

            lock = {"name": "design-fixture", "packages": {"": {"name": "design-fixture", "dependencies": {}}}}
            lock_bytes = (json.dumps(lock) + "\n").encode()
            frozen_lock = repository / "packaging/runtime/design-lock.json"
            frozen_lock.parent.mkdir(parents=True)
            frozen_lock.write_bytes(lock_bytes)
            (source / "package-lock.json").write_bytes(lock_bytes)
            current_lock = repository / "pi/package-lock.json"
            current_lock.parent.mkdir(parents=True)
            current_lock.write_bytes(lock_bytes)
            adapters = repository / "pi/dist/runtime"
            adapters.mkdir(parents=True)
            for name in ("design-main.js", "design-operations.js", "code-inspect-main.js", "code-source-inspect.js", "code-navigation.js", "code-navigation-main.js"):
                (adapters / name).write_bytes(b"compiled adapter")

            selection = repository / "packaging/runtime/design.json"
            selection.write_text(json.dumps({
                "schema": 1, "component": "design", "target": "aarch64-apple-darwin",
                "source_lock_sha256": assembly.digest(frozen_lock)[1],
                "file_count": 1, "total_bytes": package.stat().st_size,
                "files": [{"path": "style-dictionary/index.js", "bytes": package.stat().st_size,
                           "sha256": assembly.digest(package)[1]}],
                "font": {"path": "runtime/design/fonts/NotoSans.ttf", "bytes": font.stat().st_size,
                         "sha256": assembly.digest(font)[1]},
            }), encoding="utf-8")
            old = assembly.REPO, assembly.SELECTION, assembly.FROZEN_LOCK
            try:
                assembly.REPO, assembly.SELECTION, assembly.FROZEN_LOCK = repository, selection, frozen_lock
                receipt = assembly.assemble(tree, font, release)
                stale = root / "stale"
                (stale / "notices/design").mkdir(parents=True)
                with self.assertRaisesRegex(assembly.AssemblyError, "design destination already exists"):
                    assembly.assemble(tree, font, stale)
            finally:
                assembly.REPO, assembly.SELECTION, assembly.FROZEN_LOCK = old
            self.assertEqual(receipt["files"], 8)
            self.assertEqual((release / "adapters/node_modules/style-dictionary/index.js").read_bytes(), b"selected package")
            self.assertEqual((release / "runtime/design/fonts/NotoSans.ttf").read_bytes(), b"selected font")
            self.assertFalse((release / "notices").exists())


if __name__ == "__main__":
    unittest.main()
