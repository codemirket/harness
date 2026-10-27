#!/usr/bin/env python3
"""Discriminating integrity tests for the disposable design trial runner."""
from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("design-adoption-trials.py")
SPEC = importlib.util.spec_from_file_location("design_adoption_trials", SCRIPT)
assert SPEC and SPEC.loader
TRIALS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(TRIALS)


class TrialIntegrityTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name) / "trials"
        self.root.mkdir()
        frozen = self.root / "frozen"
        frozen.mkdir()
        cases_path = frozen / "cases.json"
        cases_path.write_text(json.dumps(TRIALS.CASES, indent=2) + "\n")
        (frozen / "cases.sha256").write_text(TRIALS.digest(cases_path) + "\n")
        baseline = self.root / "baseline-skills"
        hashes = {}
        for name in TRIALS.CASES["product-page"]["skills"]:
            skill = baseline / name
            skill.mkdir(parents=True)
            (skill / "SKILL.md").write_text(f"# {name}\n")
            hashes[name] = TRIALS.skill_tree_hash(skill)
        (frozen / "baseline-skill-hashes.json").write_text(json.dumps(hashes) + "\n")
        self.make_trial("product-page", "baseline")
        self.make_trial("product-page", "no-skill")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def make_trial(self, case_id: str, condition: str) -> Path:
        case = TRIALS.CASES[case_id]
        trial = self.root / f"{case_id}-{condition}"
        project = trial / "project"
        project.mkdir(parents=True)
        for name, content in case["files"].items():
            (project / name).write_text(content)
        if condition == "baseline":
            for skill in case["skills"]:
                shutil.copytree(self.root / "baseline-skills" / skill, project / ".agents" / "skills" / skill)
        manifest = {str(path.relative_to(project)): TRIALS.digest(path) for path in project.rglob("*") if path.is_file()}
        (trial / "initial-manifest.json").write_text(json.dumps(manifest) + "\n")
        (trial / "prompt.txt").write_text(case["prompt"] + "\n")
        (trial / "budget.json").write_text(json.dumps(case["budget"]) + "\n")
        return trial

    def test_timeout_cannot_exceed_frozen_budget(self) -> None:
        with self.assertRaisesRegex(SystemExit, "exceeds frozen budget"):
            TRIALS.run_trial(
                self.root,
                "product-page",
                "no-skill",
                181,
                Path("/bin/echo"),
                Path("/etc/hosts"),
                self.root,
            )

    def test_prompt_change_is_rejected(self) -> None:
        trial = self.root / "product-page-no-skill"
        (trial / "prompt.txt").write_text("changed\n")
        with self.assertRaisesRegex(SystemExit, "prompt changed"):
            TRIALS.verify_trial_inputs(self.root, "product-page", "no-skill")

    def test_project_brief_change_is_rejected(self) -> None:
        trial = self.root / "product-page-no-skill"
        (trial / "project" / "BRIEF.md").write_text("changed\n")
        with self.assertRaisesRegex(SystemExit, "inputs changed"):
            TRIALS.verify_trial_inputs(self.root, "product-page", "no-skill")

    def test_skill_snapshot_change_is_rejected(self) -> None:
        skill = self.root / "baseline-skills" / "art-direction" / "SKILL.md"
        skill.write_text("changed\n")
        with self.assertRaisesRegex(SystemExit, "skill changed"):
            TRIALS.verify_trial_inputs(self.root, "product-page", "baseline")

    def test_result_records_effective_command_and_timeout(self) -> None:
        TRIALS.run_trial(
            self.root,
            "product-page",
            "no-skill",
            1,
            Path("/bin/echo"),
            Path("/etc/hosts"),
            self.root,
        )
        result = json.loads((self.root / "product-page-no-skill" / "result.json").read_text())
        self.assertEqual(result["effective_timeout_seconds"], 1)
        self.assertEqual(result["command"][0], "/bin/echo")
        self.assertIn("platform", result["runtime_identity"])


if __name__ == "__main__":
    unittest.main()
