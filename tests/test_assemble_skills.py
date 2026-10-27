"""Source-to-release skill inventory refuses changed or extra skill bytes."""

import importlib.util
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
            "planning/skill-catalog-design.md",
            "planning/implementation-plan.md",
            "planning/references.md",
            "LICENSE",
        ):
            destination = self.root / pin
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(SOURCE.parent / pin, destination)
        self.output = self.root / "release"
        self.output.mkdir()

    def test_exact_catalog_assembles_once(self):
        result = ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(result["files"], 69)
        self.assertEqual((self.output / "skills/catalog.json").read_bytes(), (self.source / "catalog.json").read_bytes())
        self.assertEqual(len(list((self.output / "skills").rglob("SKILL.md"))), 67)
        with self.assertRaises(ASSEMBLER.AssemblyError):
            ASSEMBLER.assemble(self.output, self.source)

    def test_changed_source_refuses_before_output(self):
        body = self.source / "runtime-operations/SKILL.md"
        body.write_bytes(body.read_bytes() + b"\nextra\n")
        with self.assertRaisesRegex(ASSEMBLER.AssemblyError, "differ from catalog"):
            ASSEMBLER.assemble(self.output, self.source)
        self.assertEqual(list(self.output.iterdir()), [])

    def test_stale_authoring_basis_refuses_before_output(self):
        basis = self.root / "planning/skill-catalog-design.md"
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


if __name__ == "__main__":
    unittest.main()
