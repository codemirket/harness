#!/usr/bin/env python3
"""Create frozen, disposable coordination/feedback trial inputs.

The runner itself lives in the private qualification state directory. This
script only creates new trial directories and records the frozen rubric; it
does not touch the user's Pi installation or client settings.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

CASES = {
    "coord-migration": {
        "prompt": """Work only in this disposable project. Implement the requested migration without changing schema.json or CONTRACT.md. Create migrate.py and test_migrate.py, run the tests, and report the actual result. Use native delegation only if it materially helps: if you delegate, give each worker disjoint ownership and keep integration with the lead. Keep all outputs under this project root; do not write previews or temporary files elsewhere. No dependencies, network, credentials, messages, commits, or external effects.""",
        "files": {
            "CONTRACT.md": "# Migration contract\nEvery event occurrence is retained in input order. Repeated IDs are valid and must not be deduplicated. A malformed event is rejected without partially writing the output.\n",
            "schema.json": '{"events":[{"id":"a","value":1},{"id":"a","value":2}]}\n',
            "README.md": "Migration is unimplemented.\n",
        },
        "rubric": [
            "schema.json and CONTRACT.md are byte-identical.",
            "migrate.py preserves repeated event IDs in input order.",
            "test_migrate.py contains a duplicate-ID regression and malformed-input check.",
            "Tests run and the report distinguishes pass/fail of the current implementation.",
            "No files outside the project root and no irrelevant edits.",
        ],
        "budget": {
            "wall_seconds": 180,
            "max_repairs": 1,
            "shared_across": ["lead", "reviewer", "delegate", "retry", "specimen"],
        },
        "skill": "agent-coordination-strategy",
    },
    "coord-creative": {
        "prompt": """Work only in this disposable project. Create invitation.svg for a fictional neighborhood repair workshop using the exact copy in BRIEF.md and the supplied palette. Keep the SVG editable and preserve BRIEF.md. If visual review is materially useful, keep direction and integration with a capable lead and use a comparable reviewer; do not use a lower-capability delegate for visual judgment. Render and inspect the saved pixels with an image-capable tool when available. Keep every output, preview, and temporary file under this project root. No dependencies, network, credentials, messages, commits, or external effects.""",
        "files": {
            "BRIEF.md": "# Repair workshop\nRequired text: Mend & Meet; Saturday 14 June; Bring one broken thing; Community repair workshop; 10:00–13:00; Library courtyard.\nPalette: cream #f5efdf, green #173f35, ochre #d89b3d.\n",
        },
        "rubric": [
            "BRIEF.md remains byte-identical.",
            "invitation.svg is editable SVG with all six exact text strings and the supplied palette.",
            "Saved pixels are independently rendered and inspected for legibility/cropping.",
            "Every artifact is inside the project root.",
            "A lead/reviewer choice and one shared repair budget are recorded; no budget reset through a new specimen.",
        ],
        "budget": {
            "wall_seconds": 180,
            "max_repairs": 1,
            "shared_across": ["lead", "reviewer", "delegate", "retry", "specimen"],
        },
        "skill": "agent-coordination-strategy",
    },
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    STATE = args.output.resolve()
    if STATE.exists():
        raise SystemExit(f"refusing existing output root: {STATE}")
    STATE.mkdir(mode=0o700, parents=True)
    frozen = STATE / "deep-review-frozen"
    frozen.mkdir(mode=0o700)
    (frozen / "cases.json").write_text(json.dumps(CASES, indent=2) + "\n")
    (frozen / "cases.sha256").write_text(digest(frozen / "cases.json") + "\n")
    for case_id, case in CASES.items():
        for condition in ("baseline", "skill"):
            trial = STATE / f"{case_id}-{condition}-20261027"
            if trial.exists():
                raise SystemExit(f"refusing existing case root: {trial}")
            project = trial / "project"
            project.mkdir(parents=True)
            (trial / "user-home").mkdir(mode=0o700)
            for name, content in case["files"].items():
                (project / name).write_text(content)
            if condition == "skill":
                skill_dir = project / ".agents" / "skills" / case["skill"]
                skill_dir.mkdir(parents=True, exist_ok=True)
                shutil.copy2(REPO / "skills" / case["skill"] / "SKILL.md", skill_dir / "SKILL.md")
                (trial / "skill-snapshot.sha256").write_text(digest(skill_dir / "SKILL.md") + "\n")
            manifest = {
                str(path.relative_to(project)): digest(path)
                for path in project.rglob("*")
                if path.is_file()
            }
            (trial / "initial-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
            (trial / "prompt.txt").write_text(case["prompt"] + "\n")
            (trial / "budget.json").write_text(json.dumps(case["budget"], indent=2) + "\n")
    print(frozen)


if __name__ == "__main__":
    main()
