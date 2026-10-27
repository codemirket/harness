#!/usr/bin/env python3
"""Failure-oriented tests for the source-only private candidate assembler."""
from pathlib import Path
import hashlib
import importlib.util
import json
import tempfile
import sys
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
SCRIPT = Path(__file__).resolve().parents[1] / "scripts/openpencil-assemble.py"
spec = importlib.util.spec_from_file_location("openpencil_assemble", SCRIPT)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class AssemblyTests(unittest.TestCase):
    def test_checked_in_selection_is_pinned_independently(self):
        self.assertEqual(hashlib.sha256(module.SELECTION.read_bytes()).hexdigest(), module.PIN)

    def test_drift_symlink_existing_output_and_partial_write_rollback(self):
        with tempfile.TemporaryDirectory(prefix="openpencil-assembly-") as temporary:
            root = Path(temporary).resolve()
            source = root / "source"
            source.mkdir()
            (source / "first.js").write_bytes(b"reviewed source")
            (source / "second.js").write_bytes(b"reviewed second")
            entries = [dict(source=name, path="node_modules/example/" + name,
                            bytes=(source / name).stat().st_size,
                            sha256=hashlib.sha256((source / name).read_bytes()).hexdigest(), kind="module")
                       for name in ("first.js", "second.js")]
            selection = root / "selection.json"
            selection.write_text(json.dumps(dict(files=entries)))
            pin = hashlib.sha256(selection.read_bytes()).hexdigest()
            with patch.object(module, "SELECTION", selection), patch.object(module, "PIN", pin):
                destination = root / "runtime"
                (source / "first.js").write_bytes(b"drift")
                with self.assertRaises(module.AssemblyError):
                    module.assemble(source, destination)
                self.assertFalse(destination.exists())
                (source / "first.js").write_bytes(b"reviewed source")
                (source / "first.js").unlink()
                (source / "first.js").symlink_to(source / "second.js")
                with self.assertRaises(module.AssemblyError):
                    module.assemble(source, destination)
                self.assertFalse(destination.exists())
                (source / "first.js").unlink()
                (source / "first.js").write_bytes(b"reviewed source")
                destination.mkdir()
                sentinel = destination / "unrelated"
                sentinel.write_bytes(b"keep")
                with self.assertRaises(module.AssemblyError):
                    module.assemble(source, destination)
                self.assertEqual(sentinel.read_bytes(), b"keep")
                sentinel.unlink()
                destination.rmdir()
                original_open = Path.open
                def failing_open(path, *args, **kwargs):
                    if path == destination / "node_modules/example/second.js":
                        raise OSError("injected disk failure after first write")
                    return original_open(path, *args, **kwargs)
                with patch.object(Path, "open", failing_open):
                    with self.assertRaises(OSError):
                        module.assemble(source, destination)
                self.assertFalse(destination.exists())
                module.assemble(source, destination)
                self.assertEqual((destination / "node_modules/example/first.js").read_bytes(), b"reviewed source")


if __name__ == "__main__":
    unittest.main()
