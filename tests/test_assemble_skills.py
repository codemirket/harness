"""Source-to-release skill inventory refuses changed or extra skill bytes."""

import importlib.util
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "scripts/assemble-skills.py"
SPEC = importlib.util.spec_from_file_location("assemble_skills", SCRIPT)
ASSEMBLER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ASSEMBLER)
SOURCE = Path(__file__).resolve().parents[1] / "skills"


class SkillAssemblyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="mirket-skills-assembly-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.source = self.root / "source"
        shutil.copytree(SOURCE, self.source)
        for pin in (
            "docs/skills.md",
            "docs/architecture.md",
            "docs/references.md",
            "LICENSE",
        ):
            destination = self.root / pin
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(SOURCE.parent / pin, destination)
        self.output = self.root / "release"
        self.output.mkdir()

    def test_exact_catalog_assembles_once(self):
        result = ASSEMBLER.assemble(self.output, self.source)
        catalog = json.loads((self.source / "catalog.json").read_text())
        resources = [
            (entry["id"], resource)
            for entry in [*catalog["global"], *catalog["project"]]
            for resource in entry.get("resources", [])
        ]
        self.assertEqual(result["files"], 63 + len(resources))
        self.assertEqual((self.output / "skills/catalog.json").read_bytes(), (self.source / "catalog.json").read_bytes())
        self.assertEqual(len(list((self.output / "skills").rglob("SKILL.md"))), 61)
        for skill_id, resource in resources:
            output = self.output / "skills" / skill_id / resource["path"]
            self.assertEqual(output.stat().st_size, resource["bytes"])
            self.assertEqual(hashlib.sha256(output.read_bytes()).hexdigest(), resource["sha256"])
        with self.assertRaises(ASSEMBLER.AssemblyError):
            ASSEMBLER.assemble(self.output, self.source)

    def test_changed_source_refuses_before_output(self):
        body = self.source / "runtime-operations/SKILL.md"
        body.write_bytes(body.read_bytes() + b"\nextra\n")
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "differ from catalog"):
            ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(list(self.output.iterdir()), [])

    def test_retired_skill_cannot_leak_into_release_as_an_extra_directory(self):
        retired = self.source / "product-strategy"
        retired.mkdir()
        (retired / "SKILL.md").write_text("Retired guidance must not ship.\n")
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "unknown entries"):
            ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(list(self.output.iterdir()), [])

    def test_stale_authoring_basis_refuses_before_output(self):
        basis = self.root / "docs/skills.md"
        basis.write_bytes(basis.read_bytes() + b"\nchanged after catalog pin\n")
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "stale skill authoring pin"):
            ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(list(self.output.iterdir()), [])

    def test_extra_file_and_symlink_refuse(self):
        (self.source / "unknown.txt").write_text("extra")
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "unknown entries"):
            ASSEMBLER.assemble(self.output, self.source)
        (self.source / "unknown.txt").unlink()
        skill = self.source / "runtime-operations/SKILL.md"
        skill.unlink()
        skill.symlink_to(self.source / "task-planning/SKILL.md")
        with self.assertRaises(ASSEMBLER.AssemblyError):
            ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(list(self.output.iterdir()), [])

    def test_unlisted_nested_reference_refuses_before_output(self):
        reference_dir = self.source / "vector-art/references"
        reference_dir.mkdir(parents=True, exist_ok=True)
        (reference_dir / "unlisted.md").write_text("not in the catalog\n")
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "unknown or missing resources"):
            ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(list(self.output.iterdir()), [])

    def test_empty_unlisted_directory_and_parent_symlink_refuse(self):
        reference_dir = self.source / "vector-art/references"
        reference_dir.mkdir(parents=True, exist_ok=True)
        (reference_dir / "empty").mkdir()
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "unknown or missing resources"):
            ASSEMBLER.assemble(self.output, self.source)
        (reference_dir / "empty").rmdir()
        moved = self.source / "vector-art/references-real"
        reference_dir.rename(moved)
        reference_dir.symlink_to(moved, target_is_directory=True)
        with self.assertRaises(ASSEMBLER.AssemblyError):
            ASSEMBLER.assemble(self.output, self.source)

    def test_versioned_resource_filename_is_admitted_by_path_grammar(self):
        self.assertEqual(
            ASSEMBLER.resource_records(
                {
                    "resources": [
                        {
                            "path": "references/hyperframes-0.8.197-authoring.md",
                            "sha256": "0" * 64,
                            "bytes": 0,
                        }
                    ]
                },
                "video-production",
            )[0]["path"],
            "references/hyperframes-0.8.197-authoring.md",
        )

    def test_listed_resource_drift_refuses_before_output(self):
        catalog = json.loads((self.source / "catalog.json").read_text())
        entries = [*catalog["global"], *catalog["project"]]
        entry = next((entry for entry in entries if entry.get("resources")), None)
        if entry is None:
            self.skipTest("catalog has no admitted supporting resource yet")
        resource = entry["resources"][0]
        path = self.source / entry["id"] / resource["path"]
        path.write_bytes(path.read_bytes() + b"drift")
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "resource differs from catalog"):
            ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(list(self.output.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
