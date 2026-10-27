#!/usr/bin/env python3
"""Freeze and run bounded design-adoption trials in disposable projects.

The script records inputs, skill snapshots, manifests and traces. It performs
mechanical checks only; visual and motion quality are reviewed independently.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import time

REPO = Path(__file__).resolve().parents[1]

CASES = {
    "product-page": {
        "prompt": """Work only in this disposable project. Create a distinctive, responsive product page for a fictional public transit service using the exact facts and copy in BRIEF.md. Deliver index.html, styles.css and app.js. Include one interactive route filter component with default, hover, focus-visible, active, empty and error states; keep keyboard order useful. Use no external assets, packages, network or invented facts. Save a local page-preview.png using an available local renderer if possible, and inspect the saved pixels if an image-capable tool is available. Preserve BRIEF.md. No agents, dependencies, credentials, messages, commits or external effects.""",
        "files": {"BRIEF.md": "# Driftline\nDriftline is a fictional public transit service. Facts: weekday service 05:30–23:45; three lines named Harbor, Orchard and North Loop; day pass $6; wheelchair-accessible vehicles on every line. Required copy: Driftline; Move with the tide; Choose a line; Harbor; Orchard; North Loop; Day pass $6; Accessible vehicles on every line. Do not invent ridership, performance or customer claims.\n"},
        "skills": ["art-direction", "interface-composition"],
        "budget": {"wall_seconds": 180, "max_repairs": 1, "shared_across": ["lead", "reviewer", "delegate", "specimen", "repair"]},
        "oracle": "page",
    },
    "illustration": {
        "prompt": """Work only in this disposable project. Create an original editable SVG illustration for the exact scene in BRIEF.md. Deliver illustration.svg and a local preview.png rendered from the saved SVG when possible. The person must visibly water a plant while standing beside a stool: show a convincing weight-bearing foot, hand-to-can grip, bent arm, and can-to-plant contact. Preserve editable paths and meaningful groups; use only the supplied palette and no external assets or copied artwork. Inspect the rendered pixels at full size and a detail crop when an image-capable tool is available. Preserve BRIEF.md. No agents, dependencies, network, credentials, messages, commits or external effects.""",
        "files": {"BRIEF.md": "# Balcony watering\nAudience: a calm community-garden guide. Scene: one person in profile waters a leafy plant from a metal can beside a simple wooden stool. The visible action is the pour; the near foot bears weight on the ground and the far foot supports balance. Show the hand gripping the can handle and the water stream touching soil. Palette: paper #f4efe4, ink #183b35, leaf #648b64, coral #d8735f, brass #c79b45. Required visible label: Water what you grow. Avoid logos, photographs and borrowed characters.\n"},
        "skills": ["vector-art", "art-direction"],
        "budget": {"wall_seconds": 180, "max_repairs": 1, "shared_across": ["lead", "reviewer", "delegate", "specimen", "repair"]},
        "oracle": "svg",
    },
    "video": {
        "prompt": """Work only in this disposable project. Author a short three-scene HyperFrames HTML/CSS/JS film from BRIEF.md with local editable source and captions.json. Scene 1 introduces the Driftline tide mark, scene 2 transitions to the three lines, and scene 3 resolves on the day pass and accessibility message. Use only local source, no CDN or remote assets. Keep a deliberate visual continuity and ensure the intended tide mark and line map move, rather than relying on whole-frame changes. If the qualified Mirket video route is available, use runtime_status first, then prepare a bounded 640x360 MP4 and inspect decoded frames; if approval or capability is unavailable, preserve sources and report that exact limit without claiming an export. No agents, dependencies, network, credentials, messages, commits or external effects.""",
        "files": {"BRIEF.md": "# Driftline short film\nFormat 640x360, 12 fps, 3 seconds. Scene 1 (0–1s): Driftline wordmark and a tide mark draw on. Scene 2 (1–2s): Harbor, Orchard and North Loop route strokes appear in sequence while the tide mark travels between them. Scene 3 (2–3s): Day pass $6 and Accessible vehicles on every line hold on screen. Captions must contain the three scene messages with frame ranges. Palette: #f4efe4, #183b35, #c79b45.\n"},
        "skills": ["video-production", "motion-direction"],
        "budget": {"wall_seconds": 180, "max_repairs": 1, "shared_across": ["lead", "reviewer", "delegate", "specimen", "repair"]},
        "oracle": "video",
    },
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def snapshot_skills(destination: Path, source_root: Path, names: list[str]) -> dict[str, str]:
    destination.mkdir(parents=True)
    result = {}
    for name in names:
        source = source_root / name
        if not source.is_dir():
            raise SystemExit(f"missing skill snapshot source: {source}")
        target = destination / name
        shutil.copytree(source, target)
        result[name] = hashlib.sha256(
            "".join(
                f"{path.relative_to(target)}:{digest(path)}\n"
                for path in sorted(target.rglob("*"))
                if path.is_file()
            ).encode()
        ).hexdigest()
    return result


def all_skill_names() -> list[str]:
    return sorted({skill for case in CASES.values() for skill in case["skills"]})


def skill_tree_hash(root: Path) -> str:
    return hashlib.sha256(
        "".join(
            f"{path.relative_to(root)}:{digest(path)}\n"
            for path in sorted(root.rglob("*"))
            if path.is_file()
        ).encode()
    ).hexdigest()


def expected_manifest(project: Path, case: dict, skill_root: Path | None, skills: list[str]) -> dict[str, str]:
    expected: dict[str, str] = {}
    for name, content in case["files"].items():
        expected[name] = hashlib.sha256(content.encode()).hexdigest()
    if skill_root is not None:
        for skill in skills:
            source = skill_root / skill
            for path in source.rglob("*"):
                if path.is_file():
                    expected[str(Path(".agents") / "skills" / skill / path.relative_to(source))] = digest(path)
    return expected


def load_frozen_cases(root: Path) -> dict:
    frozen = root / "frozen"
    cases_path = frozen / "cases.json"
    marker_path = frozen / "cases.sha256"
    if not cases_path.is_file() or not marker_path.is_file():
        raise SystemExit(f"missing frozen case record under {frozen}")
    if marker_path.read_text().strip() != digest(cases_path):
        raise SystemExit("frozen cases hash does not match cases.json")
    try:
        cases = json.loads(cases_path.read_text())
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid frozen cases.json: {exc}") from exc
    if cases != CASES:
        raise SystemExit("trial definitions changed since freeze; refusing to run")
    return cases


def verify_trial_inputs(root: Path, case_id: str, condition: str) -> None:
    cases = load_frozen_cases(root)
    case = cases[case_id]
    trial = root / f"{case_id}-{condition}"
    project = trial / "project"
    prompt = trial / "prompt.txt"
    budget = trial / "budget.json"
    initial = trial / "initial-manifest.json"
    if not project.is_dir() or not prompt.is_file() or not budget.is_file() or not initial.is_file():
        raise SystemExit(f"incomplete frozen trial inputs: {trial}")
    if prompt.read_text() != case["prompt"] + "\n":
        raise SystemExit(f"trial prompt changed after freeze: {prompt}")
    try:
        if json.loads(budget.read_text()) != case["budget"]:
            raise SystemExit(f"trial budget changed after freeze: {budget}")
        stored_manifest = json.loads(initial.read_text())
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid frozen trial metadata: {exc}") from exc
    actual_manifest = {str(path.relative_to(project)): digest(path) for path in project.rglob("*") if path.is_file()}
    if actual_manifest != stored_manifest:
        raise SystemExit(f"trial inputs changed after freeze: {trial}")
    skill_root: Path | None = None
    if condition == "baseline":
        skill_root = root / "baseline-skills"
        hashes_path = root / "frozen" / "baseline-skill-hashes.json"
    elif condition == "current":
        skill_root = root / "current-skills"
        hashes_path = root / "frozen" / "current-skill-hashes.json"
    else:
        hashes_path = None
    if skill_root is not None:
        if not skill_root.is_dir() or hashes_path is None or not hashes_path.is_file():
            raise SystemExit(f"missing frozen {condition} skill snapshot: {skill_root}")
        hashes = json.loads(hashes_path.read_text())
        for skill in case["skills"]:
            source = skill_root / skill
            if not source.is_dir() or hashes.get(skill) != skill_tree_hash(source):
                raise SystemExit(f"frozen {condition} skill changed: {skill}")
    expected = expected_manifest(project, case, skill_root, case["skills"] if skill_root else [])
    if actual_manifest != expected:
        raise SystemExit(f"trial files do not match frozen case and skill snapshot: {trial}")


def freeze(root: Path, baseline: Path, codex: Path) -> None:
    if root.exists():
        raise SystemExit(f"refusing existing trial root: {root}")
    root.mkdir(parents=True, mode=0o700)
    frozen = root / "frozen"
    frozen.mkdir()
    write_json(frozen / "cases.json", CASES)
    (frozen / "cases.sha256").write_text(digest(frozen / "cases.json") + "\n")
    write_json(frozen / "platform.json", {"codex": str(codex), "codex_sha256": digest(codex) if codex.is_file() else None})
    baseline_hashes = snapshot_skills(root / "baseline-skills", baseline, all_skill_names())
    write_json(frozen / "baseline-skill-hashes.json", baseline_hashes)
    (root / "README.txt").write_text("Frozen inputs are immutable. Run --freeze-current only before any current-condition trial.\n")
    for case_id, case in CASES.items():
        for condition in ("baseline", "no-skill"):
            trial = root / f"{case_id}-{condition}"
            project = trial / "project"
            trial.mkdir()
            project.mkdir()
            (trial / "user-home").mkdir(mode=0o700)
            for name, content in case["files"].items():
                (project / name).write_text(content)
            if condition == "baseline":
                for skill in case["skills"]:
                    shutil.copytree(
                        root / "baseline-skills" / skill,
                        project / ".agents" / "skills" / skill,
                    )
            manifest = {str(path.relative_to(project)): digest(path) for path in project.rglob("*") if path.is_file()}
            write_json(trial / "initial-manifest.json", manifest)
            (trial / "prompt.txt").write_text(case["prompt"] + "\n")
            write_json(trial / "budget.json", case["budget"])


def freeze_current(root: Path) -> None:
    frozen = root / "frozen"
    marker = frozen / "current-skill-hashes.json"
    if not frozen.is_dir() or marker.exists():
        raise SystemExit("current snapshot requires a frozen root and may be written only once")
    hashes = snapshot_skills(root / "current-skills", REPO / "skills", all_skill_names())
    write_json(marker, hashes)
    for case_id, case in CASES.items():
        trial = root / f"{case_id}-current"
        project = trial / "project"
        if trial.exists():
            raise SystemExit(f"refusing existing current trial: {trial}")
        trial.mkdir()
        project.mkdir()
        (trial / "user-home").mkdir(mode=0o700)
        for name, content in case["files"].items():
            (project / name).write_text(content)
        for skill in case["skills"]:
            shutil.copytree(root / "current-skills" / skill, project / ".agents" / "skills" / skill)
        manifest = {str(path.relative_to(project)): digest(path) for path in project.rglob("*") if path.is_file()}
        write_json(trial / "initial-manifest.json", manifest)
        (trial / "prompt.txt").write_text(case["prompt"] + "\n")
        write_json(trial / "budget.json", case["budget"])
        for skill in case["skills"]:
            source = root / "current-skills" / skill / "SKILL.md"
            if not source.is_file():
                raise SystemExit(f"current snapshot missing {skill}")


def run_trial(
    root: Path,
    case_id: str,
    condition: str,
    timeout: int,
    codex: Path,
    config: Path,
    codex_home: Path,
) -> None:
    trial = root / f"{case_id}-{condition}"
    project = trial / "project"
    events = trial / "events.jsonl"
    if events.exists() or (trial / "result.json").exists():
        raise SystemExit(f"refusing completed trial: {trial}")
    if not codex.is_file():
        raise SystemExit(f"Codex executable unavailable: {codex}")
    if not config.is_file():
        raise SystemExit(f"Codex config unavailable: {config}")
    case = load_frozen_cases(root)[case_id]
    wall_seconds = int(case["budget"]["wall_seconds"])
    if timeout <= 0 or timeout > wall_seconds:
        raise SystemExit(f"timeout {timeout}s exceeds frozen budget {wall_seconds}s")
    verify_trial_inputs(root, case_id, condition)
    before = digest(config)
    args = [str(codex), "exec", "--strict-config", "--ignore-user-config", "--ignore-rules", "--ephemeral", "--skip-git-repo-check", "--sandbox", "workspace-write", "--color", "never", "--json", "-C", str(project), "-c", "project_doc_max_bytes=0", "-"]
    env = os.environ.copy()
    env["HOME"] = str(trial / "user-home")
    env["CODEX_HOME"] = str(codex_home)
    started = time.monotonic()
    process = subprocess.Popen(args, cwd=project, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    timed_out = False
    try:
        stdout, stderr = process.communicate((trial / "prompt.txt").read_bytes(), timeout=timeout)
    except subprocess.TimeoutExpired:
        timed_out = True
        os.killpg(process.pid, signal.SIGTERM)
        stdout, stderr = process.communicate(timeout=10)
    events.write_bytes(stdout)
    (trial / "stderr.log").write_bytes(stderr)
    result = {
        "case": case_id,
        "condition": condition,
        "effective_timeout_seconds": timeout,
        "command": args,
        "runtime_identity": {"platform": platform.platform(), "python": sys.version.split()[0]},
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "timed_out": timed_out,
        "exit_code": process.returncode,
        "codex_sha256": digest(codex),
        "config_sha256_before": before,
        "config_sha256_after": digest(config),
        "events_sha256": digest(events),
        "stderr_sha256": digest(trial / "stderr.log"),
        "final_manifest": {str(path.relative_to(project)): digest(path) for path in project.rglob("*") if path.is_file()},
    }
    write_json(trial / "result.json", result)
    if result["config_sha256_after"] != before:
        raise SystemExit("real Codex config changed; halt further trials")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("freeze", "freeze-current", "run"), required=True)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--case", choices=tuple(CASES))
    parser.add_argument("--condition", choices=("baseline", "no-skill", "current"))
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--baseline-skills", type=Path)
    parser.add_argument("--codex", type=Path)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--codex-home", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    if args.mode == "freeze":
        if not args.baseline_skills or not args.codex:
            raise SystemExit("--baseline-skills and --codex are required for --mode freeze")
        freeze(root, args.baseline_skills.resolve(), args.codex.resolve())
    elif args.mode == "freeze-current":
        freeze_current(root)
    else:
        if not args.case or not args.condition:
            raise SystemExit("--case and --condition are required for --mode run")
        if args.condition == "current" and not (root / "current-skills").is_dir():
            raise SystemExit("current skills are not frozen")
        if not args.codex or not args.config or not args.codex_home:
            raise SystemExit("--codex, --config, and --codex-home are required for --mode run")
        run_trial(
            root,
            args.case,
            args.condition,
            args.timeout,
            args.codex.resolve(),
            args.config.resolve(),
            args.codex_home.resolve(),
        )


if __name__ == "__main__":
    main()
