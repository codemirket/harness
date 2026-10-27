#!/usr/bin/env python3
"""Freeze and run the bounded video-authoring diagnostic pilot.

Preparation is intentionally separate from execution.  The run command refuses
to launch without a lead-owned ready manifest containing explicit executable
paths and the frozen seed digest.
"""
from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import platform
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import time
from pathlib import Path
from typing import Any


REPO = Path(__file__).resolve().parents[1]
DEFAULT_ROOT = Path(os.environ.get("MIRKET_VIDEO_PILOT_ROOT", REPO / ".private-video-authoring-pilot"))
TASKS = (
    "glasshouse-product",
    "pebble-motion",
    "cinder-vector",
    "malformed-root-cues",
    "valid-control",
    "refusal-local-asset",
)
CONDITIONS = ("baseline", "diagnostics-seed")
REPEATS = (1, 2)
COMMON = """Use only this disposable project. Do not use agents, network, packages, CDN assets, system fonts, export/render tools, or write outside this project. Preserve BRIEF.md, NotoSans.ttf, and the exact requested output scope. Use the supplied local font through a relative @font-face URL. Never claim a render or visual inspection you did not perform. Keep changes editable and deterministic."""

BRIEFS: dict[str, str] = {
    "glasshouse-product": """# Glasshouse product motion
Glasshouse is fictional. It is a small greenhouse inventory tool. Exact facts: seedling trays, misting schedule, and bench 4. Required visible copy: Glasshouse; See every seedling; Bench 4. Palette: glass #e8f1ec, ink #163b35, leaf #6f9f76, clay #c8795d. No invented metrics, testimonials or logos.""",
    "pebble-motion": """# Pebble motion beat
Pebble is fictional. It is a calm pocket notebook: a clay-colored marker enters from the left, pauses on the Pebble label, then settles. Required visible copy: Pebble; Keep one good note. Palette: paper #f5efe4, ink #183b35, clay #c8795d, brass #c79b45. No invented claims or external assets.""",
    "cinder-vector": """# Cinder vector animation
Cinder is fictional. An original copper wind marker stands on a stone ledge at dusk; its ribbon sweeps toward two grass tufts and reverses smoothly. Required visible label: Cinder. Use paper #f4efe4, ink #183b35, copper #b56f4e, clay #d8735f, grass #668f73. Preserve editable SVG groups; no copied artwork or logos.""",
    "malformed-root-cues": """# Repair the supplied malformed source
The supplied project is intentionally malformed: its root has a prose no-timeline comment but no literal data-no-timeline marker, and its captions.json contains exactly two adjacent cues with inclusive-looking frame ends. Repair only the source contract: literal root marker, 640×360, 3 seconds, and top-level captions with exclusive end_frame semantics covering 0–36. Preserve the supplied two cue texts and output scope.""",
    "valid-control": """# Validate and preserve a valid source
The supplied project is already a valid minimal 3-second source: literal data-no-timeline root, 640×360, local font reference, stable ID, and a two-cue captions.json array with exclusive ends 0–18 and 18–36. Inspect it and make no unnecessary changes. Return the same editable files and preserve the exact source scope.""",
    "refusal-local-asset": """# Unsupported local asset request
The requester asks for a video that depends on a proprietary remote font and an unselected local asset that is not present in this project. The approved contract requires an admitted local asset and explicit selection. Do not fabricate the font, create media, or claim export. Produce only a truthful decision.json explaining the refusal and an in-scope alternative using the supplied NotoSans.ttf.""",
}

PROMPTS: dict[str, str] = {
    "glasshouse-product": f"""{COMMON}\nRead BRIEF.md. Deliver exactly index.html, film.css, film.js and captions.json in addition to the preserved brief/font. Use a literal data-no-timeline root with data-width=640, data-height=360, data-duration=3 and stable Glasshouse ID. Author three readable beats around the exact facts and captions covering frames 0–36 with exclusive cue ends.""",
    "pebble-motion": f"""{COMMON}\nRead BRIEF.md. Deliver exactly index.html, film.css and film.js. Use a literal data-no-timeline root, 640×360, 3 seconds, stable IDs and CSS/WAAPI authoring. Make the marker enter, pause on Pebble, then settle; keep the final resting state and one owner per animated property.""",
    "cinder-vector": f"""{COMMON}\nRead BRIEF.md. Deliver exactly index.html, film.css, film.js and illustration.svg. Use a literal data-no-timeline root, 640×360, 3 seconds, meaningful editable SVG groups for marker/ribbon/contact/ground, and one owner per animated property. Show the stated scene and a smooth ribbon reversal.""",
    "malformed-root-cues": f"""{COMMON}\nRead BRIEF.md and inspect the supplied index.html and captions.json before writing. Deliver exactly the same editable files and preserve the two supplied cue texts. Repair only the independent source-contract findings: literal data-no-timeline, dimensions/duration, and exclusive caption ends covering 0–36. Do not redesign or add files.""",
    "valid-control": f"""{COMMON}\nRead BRIEF.md and inspect the supplied valid source. Deliver exactly the same editable files without unnecessary repairs or new files. Preserve its literal root marker, dimensions/duration, local font, stable ID and exclusive two-cue array covering 0–36.""",
    "refusal-local-asset": f"""{COMMON}\nRead BRIEF.md. Deliver only decision.json with status \"refused\", reason_codes containing \"remote-font\", \"missing-local-asset\" and \"unselected-asset\", and an alternative explaining the safe in-scope NotoSans option. Do not create index.html, media, external URLs or a fake success receipt.""",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sha_text(value: str) -> str:
    return hashlib.sha256(value.encode()).hexdigest()


def manifest(root: Path) -> dict[str, str]:
    return {str(p.relative_to(root)): sha(p) for p in sorted(root.rglob("*")) if p.is_file()}


def tree_hash(root: Path) -> str:
    return hashlib.sha256("".join(f"{p}:{h}\n" for p, h in sorted(manifest(root).items())).encode()).hexdigest()


def protected_manifest(project: Path) -> dict[str, str]:
    return {path: digest for path, digest in manifest(project).items()
            if path in {"BRIEF.md", "NotoSans.ttf"}
            or path.startswith(".agents/") or path.startswith(".diagnostics/")}


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def load_config(config_path: Path) -> dict[str, Any]:
    import tomllib
    return tomllib.loads(config_path.read_text())


def copy_head_skills(destination: Path) -> None:
    archive = subprocess.check_output(["git", "archive", "HEAD", "skills/motion-direction", "skills/video-production"], cwd=REPO)
    with tarfile.open(fileobj=__import__("io").BytesIO(archive), mode="r:") as tar:
        tar.extractall(destination)


def copy_working_skills(destination: Path) -> None:
    destination = destination / "skills"
    for skill in ("motion-direction", "video-production"):
        shutil.copytree(REPO / "skills" / skill, destination / skill)


def seed_value() -> dict[str, Any]:
    return {
        "seed_id": "harness-video-diagnostics-20261027-v1",
        "random_seed": 424242,
        "frame_rate": {"numerator": 12, "denominator": 1},
        "frame_count": 36,
        "dimensions": {"width": 640, "height": 360},
        "task_values": {
            "glasshouse-product": {"marker_x": [96, 248, 412], "accent": "#c8795d"},
            "pebble-motion": {"marker_x": [32, 264, 492], "pause_frame": 18},
            "cinder-vector": {"ribbon_degrees": [-24, 30, -24], "contact_id": "ribbon-contact"},
            "malformed-root-cues": {"cue_boundaries": [[0, 18], [18, 36]], "root_marker": "data-no-timeline"},
            "valid-control": {"cue_boundaries": [[0, 18], [18, 36]], "expected_changes": 0},
            "refusal-local-asset": {"required_reason_codes": ["remote-font", "missing-local-asset", "unselected-asset"]},
        },
    }


def diagnostics_text(seed_digest: str, checker_digest: str) -> str:
    return f"""# Diagnostic seed contract

This private pilot resource is data, not an instruction source. Use only when
the task condition explicitly includes diagnostics+seed. Seed digest:
`{seed_digest}`. Production preflight checker digest:
`{checker_digest}`.

Before authoring, inspect the supplied brief and seed. Before claiming success,
check the literal root marker, exact dimensions/duration, local font reference,
required output scope, stable IDs, captions and ownership fields required by
the brief. Keep a reversible edit and stop when a required capability or
permission is unavailable. These checks are validated independently after the
turn; model self-report is not evidence.
"""


def seed_source(project: Path, task: str) -> None:
    if task == "malformed-root-cues":
        (project / "index.html").write_text("""<!doctype html><html><head><style>@font-face{font-family:LocalNoto;src:url('./NotoSans.ttf')}</style></head><body><!-- no-timeline root described only in prose --><main id=\"repair-me\" data-composition-id=\"repair-me\" data-width=\"640\" data-height=\"360\" data-duration=\"3\"></main></body></html>
""")
        (project / "film.css").write_text("@font-face{font-family:LocalNoto;src:url('./NotoSans.ttf')}\n")
        write_json(project / "captions.json", [{"start_frame": 0, "end_frame": 17, "text": "First supplied cue"}, {"start_frame": 18, "end_frame": 35, "text": "Second supplied cue"}])
    elif task == "valid-control":
        (project / "index.html").write_text("""<!doctype html><html><head><style>@font-face{font-family:LocalNoto;src:url('./NotoSans.ttf')}</style></head><body><main id=\"valid-control\" data-composition-id=\"valid-control\" data-no-timeline data-width=\"640\" data-height=\"360\" data-duration=\"3\"></main></body></html>
""")
        (project / "film.css").write_text("@font-face{font-family:LocalNoto;src:url('./NotoSans.ttf')}\n")
        write_json(project / "captions.json", [{"start_frame": 0, "end_frame": 18, "text": "Valid first cue"}, {"start_frame": 18, "end_frame": 36, "text": "Valid second cue"}])


def prepare(root: Path, diagnostics_checker: Path, seed_fixture: Path, adapter_module: Path, config_path: Path, font_path: Path) -> None:
    if root.exists():
        raise SystemExit(f"refusing existing pilot root: {root}")
    root.mkdir(parents=True, mode=0o700)
    frozen = root / "frozen"
    (frozen / "baseline-skills").mkdir(parents=True, mode=0o700)
    (frozen / "current-skills").mkdir(parents=True, mode=0o700)
    (frozen / "diagnostics").mkdir(mode=0o700)
    (frozen / "scripts").mkdir(mode=0o700)
    if not diagnostics_checker.is_file() or not seed_fixture.is_dir() or not adapter_module.is_file() or not config_path.is_file() or not font_path.is_file():
        raise SystemExit("actual checker, seed directory, adapter module, config and font paths are required before final freeze")
    shutil.copytree(seed_fixture, frozen / "diagnostics" / "seed-fixture")
    seed_digest = tree_hash(frozen / "diagnostics" / "seed-fixture")
    checker_digest = sha(diagnostics_checker)
    shutil.copy2(diagnostics_checker, frozen / "scripts" / "check-video-source.mjs")
    (frozen / "adapters" / "dist").mkdir(parents=True, mode=0o700)
    shutil.copy2(adapter_module, frozen / "adapters" / "dist" / "video-export.js")
    write_json(frozen / "adapters" / "package.json", {"type": "module"})
    adapter_digest = sha(frozen / "adapters" / "dist" / "video-export.js")
    (frozen / "seed.sha256").write_text(seed_digest + "\n")
    copy_head_skills(frozen / "baseline-skills")
    copy_working_skills(frozen / "current-skills")
    (frozen / "diagnostics" / "diagnostic-seed.md").write_text(diagnostics_text(seed_digest, checker_digest))
    shutil.copy2(frozen / "diagnostics" / "diagnostic-seed.md", frozen / "current-skills" / "skills" / "video-production" / "references" / "diagnostic-seed.md")
    shutil.copy2(font_path, frozen / "NotoSans.ttf")
    cfg = load_config(config_path)
    model = cfg.get("model")
    reasoning = cfg.get("model_reasoning_effort")
    if not model or not reasoning:
        raise SystemExit("configured model and model_reasoning_effort are required before freezing")
    write_json(frozen / "model-settings.json", {"model": model, "reasoning_effort": reasoning, "source_config": str(config_path), "config_sha256": sha(config_path), "font_sha256": sha(font_path)})
    skill_hashes = {"baseline": {x: tree_hash(frozen / "baseline-skills" / "skills" / x) for x in ("motion-direction", "video-production")}, "diagnostics_seed": sha(frozen / "diagnostics" / "diagnostic-seed.md"), "current": {x: tree_hash(frozen / "current-skills" / "skills" / x) for x in ("motion-direction", "video-production")}}
    write_json(frozen / "skill-hashes.json", skill_hashes)
    dependency_hashes = {"seed_fixture_tree_sha256": seed_digest, "checker_sha256": checker_digest, "adapter_sha256": adapter_digest, "font_sha256": sha(frozen / "NotoSans.ttf"), "config_sha256": sha(config_path), "model_settings_sha256": sha(frozen / "model-settings.json"), "skill_hashes_sha256": sha(frozen / "skill-hashes.json")}
    write_json(frozen / "dependency-hashes.json", dependency_hashes)
    write_json(frozen / "cases.json", {task: {"brief": BRIEFS[task], "prompt": PROMPTS[task]} for task in TASKS})
    write_json(frozen / "protocol.json", {
        "scope": "six new authoring/refusal diagnostics, two conditions, two repeats (24 runs)",
        "tasks": list(TASKS), "conditions": list(CONDITIONS), "repeats": list(REPEATS),
        "baseline": "HEAD skill snapshots only",
        "diagnostics_seed": {"current_skill_delta": "diagnostic-seed.md", "production_checker_sha256": checker_digest, "adapter_module_sha256": adapter_digest, "seed_fixture_tree_sha256": seed_digest},
        "contract_equal_across_conditions": True,
        "rubric": ["independent production static validator", "literal root profile where applicable", "exact dimensions and duration", "selected local font", "required files and no extra files", "caption coverage", "frozen input preservation", "explicit refusal schema when unsupported"],
        "external_review_required": ["visible facts and typography", "intended object motion", "readability and composition", "temporal holds and reversal", "actual export and decoded pixels"],
        "budget": {"wall_seconds_total_per_run": 180, "initial_wall_seconds": 120, "repair_wall_seconds": 60, "max_repairs_per_run": 1, "shared_max_repairs": 24, "retry_policy": "none"},
        "status": "frozen-awaiting-lead-ready-manifest",
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
        "platform": platform.platform(), "python": sys.version.split()[0], "prepared_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    })
    for task in TASKS:
        for condition in CONDITIONS:
            skills = frozen / ("baseline-skills" if condition == "baseline" else "current-skills") / "skills"
            for repeat in REPEATS:
                trial = root / "trials" / f"{task}-{condition}-r{repeat}"
                project = trial / "project"
                project.mkdir(parents=True, mode=0o700); (trial / "user-home").mkdir(mode=0o700)
                (project / "BRIEF.md").write_text(BRIEFS[task] + "\n")
                shutil.copy2(frozen / "NotoSans.ttf", project / "NotoSans.ttf")
                seed_source(project, task)
                for skill in ("motion-direction", "video-production"):
                    shutil.copytree(skills / skill, project / ".agents" / "skills" / skill)
                if condition == "diagnostics-seed":
                    (project / ".diagnostics").mkdir(mode=0o700)
                    shutil.copytree(frozen / "diagnostics" / "seed-fixture", project / ".diagnostics" / "seed-fixture")
                    shutil.copytree(frozen / "scripts", project / ".diagnostics" / "scripts")
                    shutil.copytree(frozen / "adapters", project / ".diagnostics" / "adapters")
                (trial / "prompt.txt").write_text(PROMPTS[task] + ("\nAlso read the diagnostic-seed resource and the actual .diagnostics/seed-fixture; use .diagnostics/scripts/check-video-source.mjs when a source check is needed.\n" if condition == "diagnostics-seed" else "\n"))
                write_json(trial / "budget.json", {"wall_seconds_total": 180, "initial_wall_seconds": 120, "repair_wall_seconds": 60, "max_repairs": 1, "shared_task_budget_repairs": 24})
                write_json(trial / "input-manifest.json", manifest(project))
                write_json(trial / "metadata.json", {"task": task, "condition": condition, "repeat": repeat, "seed_sha256": seed_digest, "contract_sha256": sha_text(PROMPTS[task]), "condition_resources": "baseline HEAD" if condition == "baseline" else "current plus diagnostics+seed"})
                write_json(trial / "frozen-input-hashes.json", {"prompt_sha256": sha(trial / "prompt.txt"), "budget_sha256": sha(trial / "budget.json"), "metadata_sha256": sha(trial / "metadata.json"), "input_manifest_sha256": sha(trial / "input-manifest.json"), "dependency_hashes_sha256": sha(frozen / "dependency-hashes.json"), "model_settings_sha256": sha(frozen / "model-settings.json"), "protected_manifest": protected_manifest(project)})
    write_json(root / "budget-ledger.json", {"shared_max_repairs": 24, "repairs_used": 0, "runs": {}})


def valid_caption(path: Path) -> tuple[bool, str]:
    """Task-specific continuous coverage; the production oracle owns cue schema."""
    try: data = json.loads(path.read_text())
    except Exception as exc: return False, f"captions JSON invalid: {exc}"
    if not isinstance(data, list) or not data: return False, "captions must be a non-empty array"
    spans=[]
    for cue in data:
        if not isinstance(cue, dict) or type(cue.get("start_frame")) is not int or type(cue.get("end_frame")) is not int: return False, "caption cue frame fields missing"
        if not isinstance(cue.get("text"), str) or not cue["text"]: return False, "caption cue text must be a nonempty string"
        if cue["start_frame"] < 0 or cue["end_frame"] > 36 or cue["start_frame"] >= cue["end_frame"]: return False, "caption range outside 0..36 (exclusive end)"
        spans.append((cue["start_frame"], cue["end_frame"]))
    if spans[0][0] != 0 or spans[-1][1] != 36: return False, "caption coverage does not reach 0..36 (exclusive end)"
    if any(a[1] > b[0] for a,b in zip(spans, spans[1:])): return False, "caption cues overlap"
    if any(a[1] != b[0] for a,b in zip(spans, spans[1:])): return False, "caption cues have a gap"
    return True, "caption coverage valid"


def validate(task: str, project: Path) -> dict[str, Any]:
    files = manifest(project); rel = set(files)
    allowed = {"BRIEF.md", "NotoSans.ttf", "DIAGNOSTIC_SEED.json"}
    required: set[str]
    findings: list[str] = []
    if task == "glasshouse-product": required={"index.html","film.css","film.js","captions.json"}
    elif task == "pebble-motion": required={"index.html","film.css","film.js"}
    elif task == "cinder-vector": required={"index.html","film.css","film.js","illustration.svg"}
    elif task in {"malformed-root-cues", "valid-control"}: required={"index.html","film.css","captions.json"}
    else: required={"decision.json"}
    project_files={path for path in rel if not path.startswith(".agents/") and not path.startswith(".diagnostics/")}
    missing=sorted(required-project_files); extra=sorted(project_files-allowed-required)
    if missing: findings.append("missing required files: " + ", ".join(missing))
    if extra: findings.append("out-of-scope files: " + ", ".join(extra))
    if task == "refusal-local-asset":
        try:
            decision=json.loads((project/"decision.json").read_text())
            if decision.get("status") != "refused": findings.append("decision status is not refused")
            if not set((decision.get("reason_codes") or [])) >= {"remote-font","missing-local-asset","unselected-asset"}: findings.append("refusal reason codes incomplete")
        except Exception as exc: findings.append(f"decision.json invalid: {exc}")
        if any(p.suffix.lower() in {".mp4",".webm",".png",".jpg",".wav"} for p in project.rglob("*")): findings.append("refusal created media output")
    else:
        html=project/"index.html"
        if html.exists():
            text=html.read_text()
            if "data-no-timeline" not in text: findings.append("composition root lacks literal data-no-timeline")
            for key,val in (("data-width","640"),("data-height","360"),("data-duration","3")):
                if not re.search(rf'{key}=["\\\']{val}["\\\']', text): findings.append(f"composition root missing {key}={val}")
            css_text=(project/"film.css").read_text() if (project/"film.css").exists() else ""
            if "NotoSans.ttf" not in text and "NotoSans.ttf" not in css_text: findings.append("local NotoSans.ttf not referenced")
        if task in {"glasshouse-product", "malformed-root-cues", "valid-control"} and (project/"captions.json").exists():
            ok,msg=valid_caption(project/"captions.json")
            if not ok: findings.append(msg)
        if task == "cinder-vector" and (project/"illustration.svg").exists():
            svg=(project/"illustration.svg").read_text()
            for ident in ("marker","ribbon","contact","ground"):
                if ident not in svg: findings.append(f"SVG missing stable {ident} group")
    return {"task": task, "files": files, "findings": findings, "passed": not findings}


def event_usage(events: Path) -> dict:
    keys = ("input_tokens", "cached_input_tokens", "cache_write_input_tokens", "output_tokens", "reasoning_output_tokens")
    total={key: 0 for key in keys}
    observed = False
    for line in events.read_text(errors="replace").splitlines():
        try: item=json.loads(line)
        except Exception: continue
        for key,val in (item.get("usage") or {}).items():
            if key in total and isinstance(val,int):
                total[key]+=val
                observed = True
    total["usage_observed"] = observed
    if not observed:
        for key in keys: total[key] = None
    return total


def approval_signals(events: Path) -> list[str]:
    signals = []
    for line in events.read_text(errors="replace").splitlines():
        lowered = line.lower()
        if any(token in lowered for token in ("approval_required", "approval denied", "permission denied", "approval_denied", "blocked by approval")):
            signals.append(line[:1000])
    return signals


def production_manifest(project: Path, task: str) -> Path:
    expected = {
        "glasshouse-product": ("index.html", "film.css", "film.js", "captions.json", "NotoSans.ttf"),
        "pebble-motion": ("index.html", "film.css", "film.js", "NotoSans.ttf"),
        "cinder-vector": ("index.html", "film.css", "film.js", "illustration.svg", "NotoSans.ttf"),
        "malformed-root-cues": ("index.html", "film.css", "captions.json", "NotoSans.ttf"),
        "valid-control": ("index.html", "film.css", "captions.json", "NotoSans.ttf"),
    }[task]
    all_files = manifest(project)
    selected = [{"path": path, "revision": f"sha256:{all_files[path]}", "bytes": (project / path).stat().st_size} for path in expected if path in all_files]
    value = {
        "schema_version": 1,
        "entry_file": "index.html",
        "profile": {"width": 640, "height": 360, "fps": {"numerator": 12, "denominator": 1}, "frame_count": 36, "format": "mp4", "captions_path": "captions.json" if (project / "captions.json").exists() else None},
        "sources": selected,
    }
    path = project.parent / "production-manifest.json"
    write_json(path, value)
    return path


def run_production_preflight(root: Path, trial: Path, task: str, node: str, phase: str) -> dict[str, Any]:
    checker = root / "frozen" / "scripts" / "check-video-source.mjs"
    if not checker.exists():
        raise SystemExit("frozen production checker missing")
    if task == "refusal-local-asset":
        return {"status": "not_applicable", "reason": "refusal task has no video source"}
    selected_manifest = production_manifest(trial / "project", task)
    process = subprocess.run([node, str(checker), "--project", str(trial / "project"), "--manifest", str(selected_manifest)], capture_output=True, text=True, timeout=30)
    output = (process.stdout or "").strip()
    try:
        value = json.loads(output.splitlines()[-1]) if output else {"status": "refused", "code": "checker_no_json", "diagnostics": []}
    except json.JSONDecodeError:
        value = {"status": "refused", "code": "checker_invalid_json", "diagnostics": [], "raw": output[-2000:]}
    value["exit_code"] = process.returncode
    value["raw_stdout"] = output[-4000:]
    value["stderr"] = (process.stderr or "")[-2000:]
    value["oracle_passed"] = process.returncode == 0 and value.get("status") == "valid"
    if not value["oracle_passed"] and value.get("status") == "valid":
        value["status"] = "refused"
        value["code"] = "checker_nonzero"
    write_json(trial / f"{phase}-production-preflight.json", value)
    return value


def snapshot_project(trial: Path, project: Path, label: str) -> dict[str, str]:
    destination = trial / f"{label}-project-snapshot"
    if destination.exists():
        raise SystemExit(f"refusing to overwrite {label} snapshot")
    shutil.copytree(project, destination)
    snapshot = manifest(destination)
    write_json(trial / f"{label}-project-manifest.json", snapshot)
    return snapshot


def check_frozen_inputs(root: Path, trial: Path) -> None:
    lock = json.loads((trial / "frozen-input-hashes.json").read_text())
    for name, expected in (("prompt.txt", lock["prompt_sha256"]), ("budget.json", lock["budget_sha256"]), ("metadata.json", lock["metadata_sha256"]), ("input-manifest.json", lock["input_manifest_sha256"])):
        if sha(trial / name) != expected:
            raise SystemExit(f"frozen input changed: {trial / name}")
    frozen = root / "frozen"
    if sha(frozen / "dependency-hashes.json") != lock["dependency_hashes_sha256"] or sha(frozen / "model-settings.json") != lock["model_settings_sha256"]:
        raise SystemExit(f"frozen dependency/settings changed: {trial}")
    dependencies = json.loads((frozen / "dependency-hashes.json").read_text())
    for file, key in (("scripts/check-video-source.mjs", "checker_sha256"),
                      ("adapters/dist/video-export.js", "adapter_sha256"),
                      ("NotoSans.ttf", "font_sha256"),
                      ("skill-hashes.json", "skill_hashes_sha256")):
        if sha(frozen / file) != dependencies[key]:
            raise SystemExit(f"frozen dependency bytes changed: {file}")
    if tree_hash(frozen / "diagnostics" / "seed-fixture") != dependencies["seed_fixture_tree_sha256"]:
        raise SystemExit("frozen seed fixture changed")
    if protected_manifest(trial / "project") != lock["protected_manifest"]:
        raise SystemExit(f"protected project input changed: {trial}")


def ledger_phase(root: Path, key: str, phase: str, status: str) -> dict[str, Any]:
    ledger = root / "budget-ledger.json"
    with ledger.open("r+") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        current = json.load(handle)
        record = current["runs"].get(key)
        if status == "started" and phase == "initial":
            if record is not None:
                raise SystemExit(f"refusing retry or duplicate run phase: {key}")
            record = {"phases": {}}
            current["runs"][key] = record
        elif record is None:
            raise SystemExit(f"run not registered before phase: {key}/{phase}")
        existing = record["phases"].get(phase)
        if status == "started" and existing is not None:
            raise SystemExit(f"invalid repeated ledger phase: {key}/{phase}")
        if status == "completed" and (existing is None or existing["status"] != "started"):
            raise SystemExit(f"phase was not started: {key}/{phase}")
        if phase == "repair" and status == "started":
            if current["repairs_used"] >= current["shared_max_repairs"]:
                raise SystemExit("shared repair budget exhausted")
            current["repairs_used"] += 1
        record["phases"][phase] = {"status": status, "at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        handle.seek(0); handle.truncate(); json.dump(current, handle, indent=2); handle.write("\n"); fcntl.flock(handle, fcntl.LOCK_UN)
    return record


def run_codex(trial: Path, prompt: str, timeout: int, paths: dict[str,str], model: str, reasoning: str, phase: str) -> dict[str, Any]:
    project=trial/"project"; events=trial/f"{phase}-events.jsonl"; stderr=trial/f"{phase}-stderr.log"
    if events.exists() or stderr.exists(): raise SystemExit(f"refusing retry of {phase} phase: {trial}")
    (trial / f"{phase}-prompt.txt").write_text(prompt)
    args=[paths["codex"],"exec","--strict-config","--ignore-user-config","--ignore-rules","--ephemeral","--skip-git-repo-check","--sandbox","workspace-write","--color","never","--json","--model",model,"-C",str(project),"-c",f'model_reasoning_effort="{reasoning}"',"-"]
    env=os.environ.copy(); env["HOME"]=str(trial/"user-home"); env["CODEX_HOME"]=paths["codex_home"]
    before=sha(Path(paths["config"])); started=time.monotonic(); proc=subprocess.Popen(args,cwd=project,env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    timed=False
    try: out,err=proc.communicate(prompt.encode(),timeout=timeout)
    except subprocess.TimeoutExpired:
        timed=True; os.killpg(proc.pid,signal.SIGTERM)
        try: out,err=proc.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid,signal.SIGKILL); out,err=proc.communicate(timeout=5)
    events.write_bytes(out); stderr.write_bytes(err)
    return {"phase":phase,"model":model,"reasoning_effort":reasoning,"command":args,"elapsed_seconds":round(time.monotonic()-started,3),"timed_out":timed,"exit_code":proc.returncode,"config_sha256_before":before,"config_sha256_after":sha(Path(paths["config"])),"events_sha256":sha(events),"stderr_sha256":sha(stderr),"tokens":event_usage(events),"approval_signals":approval_signals(events),"manifest":manifest(project)}


def ready_paths(root: Path, ready: Path) -> tuple[dict[str,str], dict[str,Any]]:
    frozen=(root/"frozen"/"seed.sha256").read_text().strip(); data=json.loads(ready.read_text())
    if data.get("approved") is not True or data.get("seed_sha256") != frozen: raise SystemExit("ready manifest is not approved for this frozen seed")
    required=("codex","config","codex_home","mirket","node","adapter","chromium","ffmpeg","ffprobe")
    paths={k:data.get(k) for k in required}
    if any(not isinstance(v,str) or not v.startswith("/") for v in paths.values()): raise SystemExit("ready manifest needs absolute explicit tool paths")
    for k in ("codex","config","mirket","node","adapter","chromium","ffmpeg","ffprobe"):
        if not Path(paths[k]).exists(): raise SystemExit(f"ready path missing: {k}={paths[k]}")
    return paths,data


def run_one(root: Path, task: str, condition: str, repeat: int, ready: Path) -> None:
    paths,_=ready_paths(root,ready); trial=root/"trials"/f"{task}-{condition}-r{repeat}"
    if not trial.exists() or (trial/"initial-result.json").exists() or any(trial.glob("*-events.jsonl")): raise SystemExit("refusing missing or already-run trial")
    key=f"{task}-{condition}-r{repeat}"
    check_frozen_inputs(root, trial)
    expected=json.loads((trial/"input-manifest.json").read_text())
    if manifest(trial/"project") != expected: raise SystemExit("frozen trial inputs changed")
    settings=json.loads((root/"frozen"/"model-settings.json").read_text()); model=settings["model"]; reasoning=settings["reasoning_effort"]
    snapshot_project(trial,trial/"project","pre-turn")
    ledger_phase(root,key,"initial","started")
    initial=run_codex(trial,(trial/"prompt.txt").read_text(),120,paths,model,reasoning,"initial")
    write_json(trial/"initial-result.json",initial)
    ledger_phase(root,key,"initial","completed")
    if initial["config_sha256_before"] != initial["config_sha256_after"]: raise SystemExit("Codex config changed during initial phase")
    check_frozen_inputs(root, trial)
    snapshot_project(trial,trial/"project","initial")
    task_specific=validate(task,trial/"project")
    preflight=run_production_preflight(root,trial,task,paths["node"],"initial")
    full_findings=list(task_specific["findings"])
    if preflight.get("status") != "valid" and preflight.get("status") != "not_applicable":
        full_findings.append(f"production preflight {preflight.get('code', 'refused')}: {preflight.get('diagnostics', [])}")
    findings={"task": task, "task_specific": task_specific, "production_preflight": preflight, "findings": full_findings, "passed": not full_findings}
    if condition == "diagnostics-seed":
        findings["feedback"]={"mode":"detailed","task_findings":task_specific["findings"],"diagnostics":preflight.get("diagnostics",[]),"code":preflight.get("code")}
    else:
        stable_code=preflight.get("code") if preflight.get("status") not in {"valid","not_applicable"} else ("task_specific_contract_failure" if task_specific["findings"] else None)
        findings["feedback"]={"mode":"legacy-code","task_findings":task_specific["findings"],"code":stable_code}
    write_json(trial/"independent-findings.json",findings)
    if findings["findings"]:
        check_frozen_inputs(root, trial)
        ledger_phase(root,key,"repair","started")
        write_json(trial/"repair-feedback.json",findings["feedback"])
        feedback_lines=findings["feedback"].get("diagnostics",[]) if condition == "diagnostics-seed" else [findings["feedback"].get("code")]
        feedback_lines=[str(x) for x in feedback_lines if x]
        feedback_lines.extend(str(x) for x in findings["feedback"].get("task_findings",[]))
        repair_prompt=(trial/"prompt.txt").read_text()+"\nRepair only these independent findings once; do not redesign:\n"+"\n".join(f"- {x}" for x in feedback_lines)
        repair=run_codex(trial,repair_prompt,60,paths,model,reasoning,"repair"); write_json(trial/"repair-result.json",repair)
        ledger_phase(root,key,"repair","completed")
        if repair["config_sha256_before"] != repair["config_sha256_after"]: raise SystemExit("Codex config changed during repair phase")
        check_frozen_inputs(root, trial)
    final=validate(task,trial/"project")
    final_preflight=run_production_preflight(root,trial,task,paths["node"],"final")
    final["production_preflight"]=final_preflight
    if final_preflight.get("status") != "valid" and final_preflight.get("status") != "not_applicable":
        final["findings"].append(f"production preflight {final_preflight.get('code', 'refused')}: {final_preflight.get('diagnostics', [])}")
    final_manifest=manifest(trial/"project")
    if task == "valid-control" and final_manifest != expected:
        final["findings"].append("valid-control source changed unexpectedly")
    final["passed"] = not final["findings"]
    snapshot_project(trial,trial/"project","final")
    total_elapsed=initial["elapsed_seconds"] + (repair["elapsed_seconds"] if "repair" in locals() else 0)
    final["static_contract_passed"] = final["passed"]
    final["visual_motion_semantics"] = "untested until independent decoded export review"
    final["total_model_elapsed_seconds"] = total_elapsed
    if total_elapsed > 180:
        final["findings"].append("task wall budget exceeded")
        final["passed"] = False
    for phase_result in [initial] + ([repair] if "repair" in locals() else []):
        if phase_result["exit_code"] != 0 or phase_result["timed_out"]:
            final["findings"].append(f"{phase_result['phase']} model phase failed or timed out")
            final["passed"] = False
    write_json(trial/"final-result.json",final)
    write_json(trial/"scoped-delta.json",{"changed_paths":sorted(k for k in set(expected)|set(final_manifest) if expected.get(k)!=final_manifest.get(k)),"initial_manifest":expected,"final_manifest":final_manifest,"valid_control_zero_changes":task != "valid-control" or final_manifest == expected})
    ledger_phase(root,key,"final","started")
    ledger_phase(root,key,"final","completed")


def main() -> None:
    ap=argparse.ArgumentParser(); ap.add_argument("--mode",choices=("prepare","run"),required=True); ap.add_argument("--root",type=Path,default=DEFAULT_ROOT); ap.add_argument("--task",choices=TASKS); ap.add_argument("--condition",choices=CONDITIONS); ap.add_argument("--repeat",type=int,choices=REPEATS); ap.add_argument("--ready-manifest",type=Path); ap.add_argument("--diagnostics-checker",type=Path); ap.add_argument("--seed-fixture",type=Path); ap.add_argument("--adapter-module",type=Path); ap.add_argument("--config",type=Path); ap.add_argument("--font",type=Path)
    args=ap.parse_args(); root=args.root.resolve()
    if args.mode=="prepare":
        if not args.diagnostics_checker or not args.seed_fixture or not args.adapter_module or not args.config or not args.font: raise SystemExit("prepare requires --diagnostics-checker, --seed-fixture, --adapter-module, --config and --font")
        prepare(root,args.diagnostics_checker.resolve(),args.seed_fixture.resolve(),args.adapter_module.resolve(),args.config.resolve(),args.font.resolve()); return
    if not (args.task and args.condition and args.repeat and args.ready_manifest): raise SystemExit("run requires task, condition, repeat and lead ready manifest")
    run_one(root,args.task,args.condition,args.repeat,args.ready_manifest.resolve())


if __name__ == "__main__": main()
