"""Retained domain fixtures and independent oracles for pi qualification.

No host launch, authentication, configuration or legacy transcript handling.
"""

from __future__ import annotations

import fcntl

import hashlib

import json

import re

import shutil

import subprocess

import time

from pathlib import Path

from typing import Any

TASKS = (
    "glasshouse-product",
    "pebble-motion",
    "cinder-vector",
    "malformed-root-cues",
    "valid-control",
    "refusal-local-asset",
)

def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def manifest(root: Path) -> dict[str, str]:
    return {str(p.relative_to(root)): sha(p) for p in sorted(root.rglob("*")) if p.is_file()}

def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")

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
