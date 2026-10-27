#!/usr/bin/env python3
"""Freeze and run a finite author-visible motion feedback pilot.

This runner deliberately does not make Mirket an agent owner. Codex owns each
model turn; the feedback condition only supplies a disposable Codex MCP config
for the already-qualified Mirket video route. Preparation is separate from
execution and never starts a model turn.
"""
from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import platform
import shutil
import signal
import subprocess
import sys
import time
from fractions import Fraction
from pathlib import Path
from typing import Any


REPO = Path(__file__).resolve().parents[1]
DEFAULT_ROOT = Path(os.environ.get("MIRKET_MOTION_FEEDBACK_ROOT", REPO / ".private-motion-feedback-pilot"))
TASKS = ("kinetic-type", "vector-choreography", "product-ui-sequence")
CONDITIONS = ("source-only", "author-visible-feedback")
REPEATS = (1, 2)
INITIAL_SECONDS = 600
REPAIR_SECONDS = 300
SHARED_MAX_REPAIRS = 4
SHARED_MAX_MODEL_SECONDS = 2700
REVIEW_SAMPLE_FRAMES = (0, 6, 12, 18, 24, 30, 35)
REVIEW_RUBRIC = (
    "Review saved decoded frames 0,6,12,18,24,30,35 independently. Score copy/layout "
    "and temporal focus/hold 0-2 each: Northstar title/subtitle handoff, Solace four "
    "objects with directional approach/contact pause/return, or Harborline three required "
    "strings with card entry/hold/button handoff. Do not require an unrequested exact beat "
    "frame; inspect adjacent samples for holds. Assess source editability separately."
)

BRIEFS = {
    "kinetic-type": """# Kinetic type — Northstar
Northstar is a fictional night-shift planning tool. Create a 640x360, 3-second
editable composition. Required copy is exactly `Northstar` and `Plan the next
move`. Make the title enter, hold legibly, and hand emphasis to the subtitle;
use only the supplied local font and no invented claims or external assets.""",
    "vector-choreography": """# Editable vector choreography — Solace
Solace is fictional. Create an original editable vector scene at 640x360 for
3 seconds: a copper compass, a paper path and a small teal marker. The marker
travels to the compass contact point, pauses, and returns; preserve named SVG
groups and one animation owner per property. Required visible copy is `Solace`.
Use the supplied local font; no copied artwork, logos, or external assets.""",
    "product-ui-sequence": """# Product UI sequence — Harborline
Harborline is a fictional tide-log product. Create an editable 640x360,
3-second UI sequence showing `Harborline`, `Next tide`, and `Log a reading`.
A selected card should enter, pause for reading, and hand focus to the action
button while the layout remains readable. Use the supplied local font and no
invented metrics, remote assets, or external dependencies.""",
}

BASE = """Use only this disposable project. Do not use agents, network, packages,
CDN assets, system fonts, or write outside this project. Preserve BRIEF.md and
NotoSans.ttf. Export profile is exactly 640x360, 12 fps, 36 frames (3 seconds), MP4, no audio or captions. Use the local font through a relative @font-face URL. Keep every
requested output editable and deterministic. Do not claim a render, decoded
frame inspection, or visual judgment without the corresponding saved evidence.
"""

PROMPTS = {
    "kinetic-type": BASE + """Read BRIEF.md. Deliver exactly index.html, film.css,
film.js and the requested editable composition. Add a literal data-no-timeline
root with data-width=640, data-height=360, data-duration=3 and a stable ID.
Use CSS/WAAPI motion with one owner per animated property. In the source-only
condition, stop after static checks and do not render or decode. In the
author-visible-feedback condition, use the supplied Mirket video MCP route,
the explicit private FFmpeg/FFprobe paths, and an image-capable view path to
render a short specimen and inspect saved frames before reporting. Save only
in-project feedback evidence under .feedback/. Stop after the initial artifact;
do not repair until a later repair turn.""",
    "vector-choreography": BASE + """Read BRIEF.md. Deliver exactly index.html,
film.css, film.js and illustration.svg with named editable marker, compass,
path and contact groups. Add a literal data-no-timeline root, 640x360, 3
seconds and one owner per animated property. In source-only, static-check only.
In author-visible-feedback, use the supplied Mirket route and explicit private
FFmpeg/FFprobe plus image-capable view path to render/decode and inspect frames;
retain evidence under .feedback/. Stop after the initial artifact; do not repair
until a later repair turn.""",
    "product-ui-sequence": BASE + """Read BRIEF.md. Deliver exactly index.html,
film.css and film.js with an editable UI layout, stable IDs, literal
data-no-timeline, 640x360 and 3-second duration. In source-only, perform only
static source checks. In author-visible-feedback, use the supplied Mirket route
and explicit private FFmpeg/FFprobe plus image-capable view path to render,
decode and inspect representative frames; retain evidence under .feedback/.
Stop after the initial artifact; do not repair until a later repair turn.""",
}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(value: str) -> str:
    return hashlib.sha256(value.encode()).hexdigest()


def manifest(root: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise SystemExit(f"symlink refused in scoped tree: {path}")
        if path.is_file():
            values[str(path.relative_to(root))] = sha(path)
    return values


def tree_hash(root: Path) -> str:
    return hashlib.sha256("".join(f"{p}:{h}\n" for p, h in sorted(manifest(root).items())).encode()).hexdigest()


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def write_json_atomic(path: Path, value: Any) -> None:
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n")
    os.replace(temporary, path)


def expected_files(task: str) -> set[str]:
    if task == "kinetic-type":
        return {"index.html", "film.css", "film.js"}
    if task == "vector-choreography":
        return {"index.html", "film.css", "film.js", "illustration.svg"}
    return {"index.html", "film.css", "film.js"}


def feedback_config_text(ready: dict[str, Any], trial: Path, task: str, repeat: int) -> str:
    state = trial / "mirket-state"
    args = ("dev", "serve", "--state-dir", str(state), "--release-id", f"motion-feedback-{task}-r{repeat}", "--node", ready["node"], "--video-adapter", ready["adapter"], "--video-chromium", ready["chromium"], "--video-ffmpeg", ready["ffmpeg"], "--video-ffprobe", ready["ffprobe"])
    return "\n".join([
        f'model = "{ready["model"]}"',
        f'model_reasoning_effort = "{ready["reasoning_effort"]}"',
        "[mcp_servers.mirket_video]",
        f'command = {json.dumps(ready["mirket"])}',
        "args = [" + ", ".join(json.dumps(value) for value in args) + "]",
        "startup_timeout_sec = 60",
        "required = true",
        'enabled_tools = ["runtime_status", "video_prepare", "video_start", "video_status", "video_result", "video_cancel"]',
        "",
    ])


def validate_source(task: str, project: Path, condition: str = "source-only") -> dict[str, Any]:
    files = set(manifest(project))
    allowed = expected_files(task) | {"BRIEF.md", "NotoSans.ttf"}
    findings: list[str] = []
    missing = sorted(expected_files(task) - files)
    extra = sorted(p for p in files if not p.startswith(".agents/") and not p.startswith(".diagnostics/") and not p.startswith(".feedback/") and not p.startswith(".checks/") and p not in allowed)
    if ".feedback" in files or any(p.startswith(".feedback/") for p in files):
        # Feedback artifacts are part of the feedback arm only.
        if condition == "source-only":
            findings.append("feedback evidence is out of scope for source-only")
    if missing:
        findings.append("missing required files: " + ", ".join(missing))
    if extra:
        findings.append("out-of-scope files: " + ", ".join(extra))
    html = project / "index.html"
    if html.exists():
        text = html.read_text(errors="replace")
        for key, value in (("data-width", "640"), ("data-height", "360"), ("data-duration", "3")):
            if f'{key}="{value}"' not in text and f"{key}='{value}'" not in text:
                findings.append(f"root missing {key}={value}")
        if "data-no-timeline" not in text:
            findings.append("composition root lacks literal data-no-timeline")
        css = (project / "film.css").read_text(errors="replace") if (project / "film.css").exists() else ""
        if "NotoSans.ttf" not in text and "NotoSans.ttf" not in css:
            findings.append("local NotoSans.ttf is not referenced")
    if task == "vector-choreography" and (project / "illustration.svg").exists():
        svg = (project / "illustration.svg").read_text(errors="replace")
        for ident in ("marker", "compass", "path", "contact"):
            if ident not in svg:
                findings.append(f"SVG missing stable {ident} group")
    return {"task": task, "files": manifest(project), "findings": findings, "passed": not findings}


def image_event_observed(events: Path) -> bool:
    for line in events.read_text(errors="replace").splitlines() if events.exists() else []:
        try:
            value = json.loads(line)
        except ValueError:
            continue
        item = value.get("item") if isinstance(value, dict) else None
        if isinstance(item, dict) and item.get("type") in {"image", "view_image", "computer_screenshot"}:
            return True
        content = value.get("content") if isinstance(value, dict) else None
        if isinstance(content, list) and any(isinstance(block, dict) and block.get("type") == "image" for block in content):
            return True
    return False


def validate_feedback_evidence(project: Path, events: Path, ready: dict[str, Any], condition: str, task: str) -> dict[str, Any]:
    """Check that feedback left independently inspectable evidence.

    This does not infer visual inspection from a receipt. It requires a saved
    render and separately decodable frames; image-capable event evidence is
    recorded as observed or unknown and is never failed solely because a CLI
    trace omitted it.
    """
    if condition != "author-visible-feedback":
        return {"findings": [], "observations": {"image_view": "not_applicable"}}
    evidence = project / ".feedback"
    if not evidence.is_dir():
        return {"findings": ["author-visible feedback evidence directory missing"], "observations": {"image_view": "unknown"}}
    findings: list[str] = []
    videos = sorted(evidence.glob("*.mp4")) + sorted(evidence.glob("*.webm"))
    if not videos:
        findings.append("feedback render output missing")
    else:
        try:
            probe = subprocess.run(
                [ready["ffprobe"], "-v", "error", "-select_streams", "v:0", "-count_frames", "-show_entries", "stream=width,height,nb_read_frames,avg_frame_rate", "-of", "json", str(videos[0])],
                capture_output=True, text=True, timeout=30,
            )
        except subprocess.TimeoutExpired:
            probe = None
            findings.append("ffprobe timed out during independent verification")
        if probe is None:
            pass
        elif probe.returncode != 0:
            findings.append("feedback render cannot be independently decoded by ffprobe")
        else:
            try:
                streams = json.loads(probe.stdout).get("streams", [])
                if not streams:
                    findings.append("feedback render has no video stream")
                    rate = Fraction(0, 1)
                else:
                    rate = Fraction(str(streams[0].get("avg_frame_rate", "0")))
                if streams and (streams[0].get("width") != 640 or streams[0].get("height") != 360 or streams[0].get("nb_read_frames") not in {"36", 36} or rate != Fraction(12, 1)):
                    findings.append("feedback render dimensions are not independently verified")
            except (ValueError, TypeError, ZeroDivisionError):
                findings.append("feedback ffprobe output or frame rate is invalid")
    oracle_frames = project.parent / (events.stem + "-oracle-frames")
    # A repair must never inherit decoded pixels from an earlier validation.
    # These directories are runner-owned evidence paths, not model outputs.
    if oracle_frames.is_symlink():
        oracle_frames.unlink()
    elif oracle_frames.exists():
        shutil.rmtree(oracle_frames)
    oracle_frames.mkdir(mode=0o700, exist_ok=True)
    if videos:
        decode_failures = []
        for frame_number in REVIEW_SAMPLE_FRAMES:
            output = oracle_frames / f"frame-{frame_number:02d}.png"
            try:
                decoded = subprocess.run(
                    [ready["ffmpeg"], "-v", "error", "-nostdin", "-y", "-i", str(videos[0]), "-ss", str(frame_number / 12), "-frames:v", "1", str(output)],
                    capture_output=True, text=True, timeout=30,
                )
            except subprocess.TimeoutExpired:
                decoded = None
                findings.append("independent ffmpeg frame decode timed out")
            if decoded is None or decoded.returncode != 0 or not output.is_file() or output.stat().st_size == 0 or output.read_bytes()[:8] != b"\x89PNG\r\n\x1a\n":
                decode_failures.append(frame_number)
        independent_frames = sorted(oracle_frames.glob("*.png"))
        if decode_failures:
            findings.append("independent ffmpeg frame decode failed")
    else:
        independent_frames = []
    if not independent_frames:
        findings.append("decoded feedback frame missing")
    observations = {"image_view": "observed" if image_event_observed(events) else "unknown"}
    # A model-authored receipt is not proof of what the renderer consumed.
    # Source binding remains unknown unless an independent renderer request
    # record is retained by the host workflow.
    observations["source_binding"] = "unknown"
    observations["oracle_frames"] = [str(frame) for frame in independent_frames]
    return {"findings": findings, "observations": observations}


def check_ready(ready: dict[str, Any], *, require_feedback: bool = True) -> None:
    ready.setdefault("checker", str(REPO / "scripts" / "check-video-source.mjs"))
    ready.setdefault("adapter_export", str(REPO / "adapters" / "dist" / "video-export.js"))
    required = ("codex", "mirket", "node", "adapter", "chromium", "ffmpeg", "ffprobe", "auth_file", "model", "reasoning_effort")
    missing = [key for key in required if not ready.get(key)]
    if missing:
        raise SystemExit("ready manifest missing: " + ", ".join(missing))
    for key in ("codex", "mirket", "node", "adapter", "chromium", "ffmpeg", "ffprobe", "auth_file", "checker", "adapter_export"):
        if ready.get(key) and not Path(ready[key]).exists():
            raise SystemExit(f"ready path missing: {key}={ready[key]}")
    if Path(ready["auth_file"]).stat().st_mode & 0o077:
        raise SystemExit("auth_file must be owner-readable only")
    if ready.get("approval_review", "never") not in {"never", "auto_review"}:
        raise SystemExit("unsupported approval review selection")
    if ready.get("model") != "gpt-6-astra" or ready.get("reasoning_effort") != "high":
        raise SystemExit("pilot requires the frozen gpt-6-astra/high setting")


def bound_ready_path(ready: dict[str, Any], key: str) -> str | None:
    """Resolve repository-owned diagnostics without requiring mutable ready fields."""
    defaults = {
        "checker": str(REPO / "scripts" / "check-video-source.mjs"),
        "adapter_export": str(REPO / "adapters" / "dist" / "video-export.js"),
    }
    return ready.get(key) or defaults.get(key)


def prepare(root: Path, ready_path: Path, font: Path) -> None:
    if root.exists():
        raise SystemExit(f"refusing existing pilot root: {root}")
    ready = json.loads(ready_path.read_text())
    check_ready(ready)
    if not font.is_file():
        raise SystemExit(f"font path missing: {font}")
    root.mkdir(parents=True, mode=0o700)
    frozen = root / "frozen"
    (frozen / "skills").mkdir(parents=True)
    shutil.copytree(REPO / "skills" / "motion-direction", frozen / "skills" / "motion-direction", symlinks=True)
    shutil.copytree(REPO / "skills" / "video-production", frozen / "skills" / "video-production", symlinks=True)
    shutil.copy2(font, frozen / "NotoSans.ttf")
    (frozen / "diagnostics" / "scripts").mkdir(parents=True)
    (frozen / "diagnostics" / "adapters" / "dist").mkdir(parents=True)
    shutil.copy2(ready["checker"], frozen / "diagnostics" / "scripts" / "check-video-source.mjs")
    shutil.copy2(ready["adapter_export"], frozen / "diagnostics" / "adapters" / "dist" / "video-export.js")
    model = {"model": ready["model"], "reasoning_effort": ready["reasoning_effort"], "ready_manifest_sha256": sha(ready_path), "font_sha256": sha(font)}
    write_json(frozen / "model-settings.json", model)
    skill_hash = {name: tree_hash(frozen / "skills" / name) for name in ("motion-direction", "video-production")}
    write_json(frozen / "skill-hashes.json", skill_hash)
    write_json(frozen / "protocol.json", {
        "scope": "three held-out motion briefs, source-only versus author-visible feedback, two repeats (12 cells)",
        "tasks": list(TASKS), "conditions": list(CONDITIONS), "repeats": list(REPEATS),
        "assets": {"font": "NotoSans.ttf", "font_sha256": sha(frozen / "NotoSans.ttf")},
        "diagnostics": {"checker_sha256": sha(frozen / "diagnostics/scripts/check-video-source.mjs"), "adapter_export_sha256": sha(frozen / "diagnostics/adapters/dist/video-export.js"), "coverage": "shared read-only references in both arms; independent Python source checker remains the acceptance oracle"},
        "review_rubric": REVIEW_RUBRIC,
        "approval_review": ready.get("approval_review", "never"),
        "review_sample_frames": list(REVIEW_SAMPLE_FRAMES),
        "model": model, "skill_hashes": skill_hash,
        "budget": {"initial_seconds": INITIAL_SECONDS, "repair_seconds": REPAIR_SECONDS, "max_repairs_per_cell": 1, "shared_max_repairs": SHARED_MAX_REPAIRS, "aggregate_model_turn_seconds": 2700, "lead_enforced_wall_seconds_including_review": 2700, "retries": 0},
        "feedback_boundary": "only the author-visible condition receives disposable MCP video configuration; source-only has no Mirket MCP server",
        "visual_evidence": "feedback claims require MCP/runtime event, saved output, decoded frame evidence and image-capable view evidence; otherwise blocked/unverified",
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
        "platform": platform.platform(), "prepared_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": "frozen-awaiting-lead-runtime-confirmation",
    })
    write_json(frozen / "briefs.json", BRIEFS)
    write_json(frozen / "runtime-inputs.json", {key: {"path": ready[key], "sha256": sha(Path(ready[key]))} for key in ("codex", "mirket", "node", "adapter", "chromium", "ffmpeg", "ffprobe", "checker", "adapter_export")})
    shutil.copy2(Path(__file__), frozen / "motion-feedback-pilot.py")
    shutil.copytree(REPO / "adapters" / "src", frozen / "adapter-src", symlinks=True)
    shutil.copytree(REPO / "adapters" / "dist", frozen / "adapter-dist", symlinks=True)
    runner_hash = sha(frozen / "motion-feedback-pilot.py")
    adapter_source_hash = tree_hash(frozen / "adapter-src")
    adapter_dist_hash = tree_hash(frozen / "adapter-dist")
    protocol = json.loads((frozen / "protocol.json").read_text())
    protocol["static_checker"] = {"name": "validate_source", "runner_sha256": runner_hash}
    protocol["adapter_closure"] = {"source_tree_sha256": adapter_source_hash, "dist_tree_sha256": adapter_dist_hash}
    write_json(frozen / "protocol.json", protocol)
    write_json(root / "budget-ledger.json", {"shared_max_repairs": SHARED_MAX_REPAIRS, "repairs_used": 0, "shared_max_model_seconds": SHARED_MAX_MODEL_SECONDS, "model_seconds_used": 0.0, "model_seconds_reserved": 0.0, "runs": {}})
    for task in TASKS:
        for condition in CONDITIONS:
            for repeat in REPEATS:
                trial = root / "trials" / f"{task}-{condition}-r{repeat}"
                project = trial / "project"
                project.mkdir(parents=True, mode=0o700)
                (trial / "user-home").mkdir(mode=0o700)
                (trial / "tmp").mkdir(mode=0o700)
                codex_home = trial / "codex-home"
                codex_home.mkdir(mode=0o700)
                shutil.copy2(ready["auth_file"], codex_home / "auth.json")
                (codex_home / "auth.json").chmod(0o600)
                if condition == "author-visible-feedback":
                    (trial / "mirket-state").mkdir(mode=0o700)
                    (codex_home / "config.toml").write_text(feedback_config_text(ready, trial, task, repeat))
                else:
                    (codex_home / "config.toml").write_text(f'model = "{ready["model"]}"\nmodel_reasoning_effort = "{ready["reasoning_effort"]}"\n')
                (project / "BRIEF.md").write_text(BRIEFS[task] + "\n")
                shutil.copy2(frozen / "NotoSans.ttf", project / "NotoSans.ttf")
                (project / ".agents" / "skills").mkdir(parents=True)
                for name in ("motion-direction", "video-production"):
                    shutil.copytree(frozen / "skills" / name, project / ".agents" / "skills" / name, symlinks=True)
                shutil.copytree(frozen / "diagnostics", project / ".diagnostics", symlinks=True)
                common_diagnostics = f"\nThe shared diagnostics are available read-only under `.diagnostics`. Use Node `{ready['node']}` (not PATH) to run `.diagnostics/scripts/check-video-source.mjs` against a manifest you may write at `.checks/source-manifest.json`; do not treat its receipt as visual proof. `.diagnostics` is immutable. Independent reviewers sample decoded frames {','.join(str(frame) for frame in REVIEW_SAMPLE_FRAMES)}.\n"
                prompt = PROMPTS[task] + common_diagnostics + (f"The author-visible feedback route is enabled for this cell. Use FFmpeg `{ready['ffmpeg']}` and FFprobe `{ready['ffprobe']}` exactly; do not substitute PATH tools.\n" if condition == "author-visible-feedback" else "This cell intentionally has no Mirket MCP server; do not attempt a render or decode. Save all static-check evidence under .checks/, never .feedback/.\n")
                (trial / "prompt.txt").write_text(prompt)
                write_json(trial / "budget.json", {"initial_seconds": INITIAL_SECONDS, "repair_seconds": REPAIR_SECONDS, "max_repairs": 1, "shared_max_repairs": SHARED_MAX_REPAIRS})
                write_json(trial / "metadata.json", {"task": task, "condition": condition, "repeat": repeat, "model": model, "skill_hashes": skill_hash, "expected_files": sorted(expected_files(task)), "font": "NotoSans.ttf", "prompt_sha256": digest_text(prompt), "codex_config_sha256": sha(codex_home / "config.toml")})
                write_json(trial / "input-manifest.json", manifest(project))
                write_json(trial / "frozen-input-hashes.json", {"prompt_sha256": sha(trial / "prompt.txt"), "budget_sha256": sha(trial / "budget.json"), "metadata_sha256": sha(trial / "metadata.json"), "input_manifest_sha256": sha(trial / "input-manifest.json"), "skill_hashes_sha256": sha(frozen / "skill-hashes.json"), "model_settings_sha256": sha(frozen / "model-settings.json"), "protocol_sha256": sha(frozen / "protocol.json"), "runtime_inputs_sha256": sha(frozen / "runtime-inputs.json"), "runner_sha256": runner_hash})


def lock_ledger(root: Path) -> tuple[Any, dict[str, Any]]:
    handle = (root / "budget-ledger.lock").open("a+")
    fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
    ledger = json.loads((root / "budget-ledger.json").read_text())
    return handle, ledger


def ledger_phase(root: Path, cell: str, phase: str, status: str) -> None:
    handle, ledger = lock_ledger(root)
    try:
        row = ledger["runs"].get(cell)
        if row is None:
            if not (phase == "initial" and status == "started"):
                raise SystemExit(f"phase {phase}/{status} cannot start before initial: {cell}")
            row = {"phases": {}}
            ledger["runs"][cell] = row
        phases = row["phases"]
        existing = phases.get(phase)
        if phase == "initial" and status == "completed" and phases.get("initial", {}).get("status") != "started":
            raise SystemExit(f"initial completion without start: {cell}")
        if phase == "repair" and status == "started":
            if phases.get("initial", {}).get("status") != "completed" or existing:
                raise SystemExit(f"repair cannot start before completed initial or after retry: {cell}")
            if ledger["repairs_used"] >= ledger["shared_max_repairs"]:
                raise SystemExit("shared repair budget exhausted")
            ledger["repairs_used"] += 1
        if phase == "repair" and status == "completed" and phases.get("repair", {}).get("status") != "started":
            raise SystemExit(f"repair completion without start: {cell}")
        if phase == "final" and status == "started":
            if phases.get("initial", {}).get("status") != "completed" or phases.get("repair", {}).get("status") in {"started"} or existing:
                raise SystemExit(f"final cannot start before completed authoring phases: {cell}")
        if phase == "final" and status == "completed" and phases.get("final", {}).get("status") != "started":
            raise SystemExit(f"final completion without start: {cell}")
        if existing:
            if not (existing["status"] == "started" and status == "completed"):
                raise SystemExit(f"duplicate phase/retry refused: {cell}/{phase}")
            existing["status"] = status
            existing["completed_at_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        else:
            phases[phase] = {"status": status, "at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())}
        write_json_atomic(root / "budget-ledger.json", ledger)
    finally:
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        handle.close()


def reserve_model_seconds(root: Path, seconds: int) -> None:
    handle, ledger = lock_ledger(root)
    try:
        used = float(ledger.get("model_seconds_used", 0.0))
        reserved = float(ledger.get("model_seconds_reserved", 0.0))
        if used + reserved + seconds > float(ledger["shared_max_model_seconds"]):
            raise SystemExit("shared model wall budget exhausted")
        ledger["model_seconds_reserved"] = reserved + seconds
        write_json_atomic(root / "budget-ledger.json", ledger)
    finally:
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        handle.close()


def settle_model_seconds(root: Path, reserved: int, elapsed: float) -> None:
    handle, ledger = lock_ledger(root)
    try:
        ledger["model_seconds_reserved"] = max(0.0, float(ledger.get("model_seconds_reserved", 0.0)) - reserved)
        ledger["model_seconds_used"] = float(ledger.get("model_seconds_used", 0.0)) + elapsed
        write_json_atomic(root / "budget-ledger.json", ledger)
    finally:
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        handle.close()


def release_model_seconds(root: Path, reserved: int) -> None:
    """Roll back a reservation when its phase transition cannot be committed."""
    handle, ledger = lock_ledger(root)
    try:
        ledger["model_seconds_reserved"] = max(0.0, float(ledger.get("model_seconds_reserved", 0.0)) - reserved)
        write_json_atomic(root / "budget-ledger.json", ledger)
    finally:
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        handle.close()


def reserve_phase_budget(root: Path, cell: str, phase: str, seconds: int) -> None:
    """Reserve time and commit the phase start without leaking on rejection."""
    reserved = False
    try:
        reserve_model_seconds(root, seconds)
        reserved = True
        ledger_phase(root, cell, phase, "started")
    except BaseException:
        if reserved:
            release_model_seconds(root, seconds)
        raise


def failure_signals(stdout: bytes, stderr: bytes) -> tuple[list[str], list[str]]:
    """Retain refusals from host errors, not echoed commands or author prose.

    Generic OS permission diagnostics (for example Chrome's updater) are not
    approval-review denials. Raw traces remain available for those diagnostics.
    """
    candidates: list[tuple[str, bool]] = []
    for line in (stdout + b"\n" + stderr).decode(errors="replace").splitlines():
        try:
            event = json.loads(line)
        except ValueError:
            candidates.append((line, "mcp tool call" in line.lower()))
            continue
        if not isinstance(event, dict):
            continue
        item = event.get("item") or {}
        if event.get("type") == "item.completed" and isinstance(item, dict):
            kind = item.get("type")
            result = item.get("result") or {}
            failed = item.get("status") == "failed" or bool(item.get("error")) or (isinstance(result, dict) and result.get("isError") is True)
            if kind == "mcp_tool_call" and failed:
                candidates.append((json.dumps(item.get("error") or result), True))
            # Even a failed command can print source, fixtures or arbitrary
            # prose. Its output is not a host approval decision.
        elif event.get("type") in {"error", "turn.failed"}:
            text = json.dumps(event.get("error") or event.get("message") or {})
            candidates.append((text, "mcp tool call" in text.lower()))
    approvals = ("approval_required", "approval denied", "approval_denied", "blocked by approval", "requires approval", "require approval")
    unavailable = ("mcp server failed", "mcp startup failed", "required mcp servers failed", "mcp unavailable", "tool unavailable", "runtime unavailable")
    def matching(tokens: tuple[str, ...], *, tool_permission: bool = False) -> list[str]:
        matches = []
        for text, tool_failure in candidates:
            terms = tokens + (("permission denied",) if tool_permission and tool_failure else ())
            positions = [text.lower().find(token) for token in terms if token in text.lower()]
            if positions:
                start = max(0, min(positions) - 160)
                matches.append(text[start:start + 1000])
        return matches
    return matching(approvals, tool_permission=True), matching(unavailable)


def run_model(trial: Path, prompt: str, phase: str, ready: dict[str, Any], condition: str, seconds: int) -> dict[str, Any]:
    events = trial / f"{phase}-events.jsonl"
    stderr = trial / f"{phase}-stderr.log"
    if events.exists() or stderr.exists():
        raise SystemExit(f"refusing retry: {trial}/{phase}")
    codex_home = trial / "codex-home"
    env = {**os.environ, "CODEX_HOME": str(codex_home), "HOME": str(trial / "user-home"), "TMPDIR": str(trial / "tmp"), "TMP": str(trial / "tmp"), "TEMP": str(trial / "tmp")}
    args = [ready["codex"], "exec", "--strict-config", "--ignore-rules", "--ephemeral", "--skip-git-repo-check", "--sandbox", "workspace-write", "--color", "never", "--json", "--model", ready["model"], "-C", str(trial / "project"), "-c", f'model_reasoning_effort="{ready["reasoning_effort"]}"', "-"]
    if ready.get("approval_review") == "auto_review":
        sandbox_index = args.index("--sandbox")
        del args[sandbox_index:sandbox_index + 2]
        args.insert(2, "--approve-for-me")
    started = time.monotonic()
    proc = subprocess.Popen(args, cwd=trial / "project", env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
    out, err = b"", b""
    interrupted = False
    try:
        out, err = proc.communicate(prompt.encode(), timeout=seconds)
        timed_out = False
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as exc:
        out = getattr(exc, "output", None) or b""; err = getattr(exc, "stderr", None) or b""
        timed_out = isinstance(exc, subprocess.TimeoutExpired)
        interrupted = isinstance(exc, KeyboardInterrupt)
        try:
            os.killpg(proc.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            out, err = proc.communicate(timeout=5)
        except subprocess.TimeoutExpired as tail:
            out, err = tail.output or out, tail.stderr or err
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                out, err = proc.communicate(timeout=5)
            except subprocess.TimeoutExpired as last:
                out, err = last.output or out, last.stderr or err
                proc.stdout.close()
                proc.stderr.close()
        # communicate() returns cumulative output after a timeout; appending
        # the earlier exception output would duplicate events and usage.
    events.write_bytes(out); stderr.write_bytes(err)
    approval, blocked = failure_signals(out, err)
    usage = {"input_tokens": 0, "cached_input_tokens": 0, "cache_write_input_tokens": 0, "output_tokens": 0, "reasoning_output_tokens": 0}
    usage_observed = False
    turn_completed = False
    for line in out.decode(errors="replace").splitlines():
        try: value = json.loads(line)
        except ValueError: continue
        if not isinstance(value, dict): continue
        turn_completed = turn_completed or value.get("type") == "turn.completed"
        for key, number in (value.get("usage") or {}).items():
            if key in usage and isinstance(number, int): usage[key] += number; usage_observed = True
    return {"phase": phase, "condition": condition, "model": ready["model"], "reasoning_effort": ready["reasoning_effort"], "command": args, "elapsed_seconds": round(time.monotonic() - started, 3), "timed_out": timed_out, "interrupted": interrupted, "exit_code": proc.returncode, "turn_completed": turn_completed, "events_sha256": sha(events), "stderr_sha256": sha(stderr), "approval_signals": approval, "blocked_signals": blocked, "usage": usage if usage_observed else None, "usage_observed": usage_observed}


def snapshot(trial: Path, label: str) -> dict[str, str]:
    destination = trial / f"{label}-project-snapshot"
    if destination.exists():
        raise SystemExit(f"snapshot already exists: {destination}")
    shutil.copytree(trial / "project", destination, symlinks=True)
    value = manifest(destination)
    write_json(trial / f"{label}-project-manifest.json", value)
    return value


def check_frozen_inputs(root: Path, trial: Path, ready: dict[str, Any], ready_path: Path, *, before_initial: bool = False) -> None:
    lock = json.loads((trial / "frozen-input-hashes.json").read_text())
    actual = {
        "prompt_sha256": sha(trial / "prompt.txt"),
        "budget_sha256": sha(trial / "budget.json"),
        "metadata_sha256": sha(trial / "metadata.json"),
        "input_manifest_sha256": sha(trial / "input-manifest.json"),
        "skill_hashes_sha256": sha(root / "frozen/skill-hashes.json"),
        "model_settings_sha256": sha(root / "frozen/model-settings.json"),
        "protocol_sha256": sha(root / "frozen/protocol.json"),
        "runtime_inputs_sha256": sha(root / "frozen/runtime-inputs.json"),
        "runner_sha256": sha(Path(__file__)),
    }
    expected = {key: lock[key] for key in actual if key in lock}
    if set(actual) != set(lock) or actual != expected:
        raise SystemExit(f"frozen pilot inputs changed: {trial}")
    metadata = json.loads((trial / "metadata.json").read_text())
    if metadata.get("codex_config_sha256") != sha(trial / "codex-home/config.toml"):
        raise SystemExit(f"cell Codex config changed: {trial}")
    locked_project = json.loads((trial / "input-manifest.json").read_text())
    if before_initial and manifest(trial / "project") != locked_project:
        raise SystemExit(f"frozen project input changed: {trial}")
    protected = {path: digest for path, digest in locked_project.items() if path in {"BRIEF.md", "NotoSans.ttf"} or path.startswith(".agents/") or path.startswith(".diagnostics/")}
    current_manifest = manifest(trial / "project")
    current_protected = {path: digest for path, digest in current_manifest.items() if path in protected or path.startswith(".diagnostics/") or path.startswith(".agents/")}
    if current_protected != protected:
        raise SystemExit(f"protected project input changed: {trial}")
    runtime = json.loads((root / "frozen/runtime-inputs.json").read_text())
    for key, value in runtime.items():
        path = Path(value["path"])
        if bound_ready_path(ready, key) != value["path"]:
            raise SystemExit(f"ready runtime binding changed: {key}")
        if not path.is_file() or sha(path) != value["sha256"]:
            raise SystemExit(f"frozen runtime changed: {path}")
    settings = json.loads((root / "frozen/model-settings.json").read_text())
    if settings.get("ready_manifest_sha256") != sha(ready_path):
        raise SystemExit("ready manifest changed after freeze")
    protocol = json.loads((root / "frozen/protocol.json").read_text())
    if protocol.get("static_checker", {}).get("runner_sha256") != sha(Path(__file__)):
        raise SystemExit("static checker/runner changed after freeze")
    closure = protocol.get("adapter_closure", {})
    if (closure.get("source_tree_sha256") != tree_hash(REPO / "adapters" / "src")
            or closure.get("dist_tree_sha256") != tree_hash(REPO / "adapters" / "dist")
            or closure.get("source_tree_sha256") != tree_hash(root / "frozen" / "adapter-src")
            or closure.get("dist_tree_sha256") != tree_hash(root / "frozen" / "adapter-dist")):
        raise SystemExit("adapter source/dist closure changed after freeze")


def run_cell(root: Path, ready_path: Path, cell: str) -> None:
    """Run only the initial authoring turn; leave review/repair explicit."""
    ready = json.loads(ready_path.read_text()); check_ready(ready)
    trial = root / "trials" / cell
    if not trial.is_dir() or (trial / "initial-result.json").exists():
        raise SystemExit(f"missing or already-run cell: {cell}")
    meta = json.loads((trial / "metadata.json").read_text())
    check_frozen_inputs(root, trial, ready, ready_path, before_initial=True)
    snapshot(trial, "pre-turn")
    reserve_phase_budget(root, cell, "initial", INITIAL_SECONDS)
    initial = run_model(trial, (trial / "prompt.txt").read_text(), "initial", ready, meta["condition"], INITIAL_SECONDS)
    settle_model_seconds(root, INITIAL_SECONDS, initial["elapsed_seconds"])
    write_json(trial / "initial-result.json", initial)
    ledger_phase(root, cell, "initial", "completed")
    snapshot(trial, "initial")
    check_frozen_inputs(root, trial, ready, ready_path)
    findings = validate_source(meta["task"], trial / "project", meta["condition"])
    feedback = validate_feedback_evidence(trial / "project", trial / "initial-events.jsonl", ready, meta["condition"], meta["task"])
    findings["findings"].extend(feedback["findings"])
    findings["observations"] = feedback["observations"]
    findings["passed"] = not findings["findings"]
    write_json(trial / "initial-independent-findings.json", findings)
    write_json(trial / "initial-state.json", {"status": "awaiting_independent_review"})


def repair_cell(root: Path, ready_path: Path, cell: str, review_path: Path) -> None:
    ready = json.loads(ready_path.read_text()); check_ready(ready)
    trial = root / "trials" / cell
    if not (trial / "initial-result.json").is_file() or (trial / "repair-result.json").exists() or (trial / "final-result.json").exists():
        raise SystemExit(f"repair requires an unfinished initial cell: {cell}")
    meta = json.loads((trial / "metadata.json").read_text())
    check_frozen_inputs(root, trial, ready, ready_path)
    review = json.loads(review_path.read_text())
    if review.get("cell") != cell or review.get("kind") not in {"static", "visual"} or not isinstance(review.get("findings"), list) or not review["findings"]:
        raise SystemExit("review findings must identify this cell, a kind, and non-empty findings")
    if review["kind"] == "visual" and meta["condition"] != "author-visible-feedback":
        raise SystemExit("source-only cells cannot receive visual repair findings")
    initial = json.loads((trial / "initial-result.json").read_text())
    if initial.get("exit_code") != 0 or initial.get("timed_out") or initial.get("approval_signals") or initial.get("blocked_signals"):
        raise SystemExit("environment/approval/timeout initial result cannot receive automatic repair")
    review_destination = trial / "review-findings.json"
    if review_path.resolve() != review_destination.resolve():
        shutil.copy2(review_path, review_destination)
    reserve_phase_budget(root, cell, "repair", REPAIR_SECONDS)
    prompt = (trial / "prompt.txt").read_text() + "\nIndependent reviewer findings (repair only these; do not redesign):\n" + json.dumps(review["findings"], indent=2) + "\nInspect existing feedback evidence only when this condition authorizes it. Stop after this one targeted repair."
    result = run_model(trial, prompt, "repair", ready, meta["condition"], REPAIR_SECONDS)
    settle_model_seconds(root, REPAIR_SECONDS, result["elapsed_seconds"])
    write_json(trial / "repair-result.json", result)
    ledger_phase(root, cell, "repair", "completed")
    snapshot(trial, "repair")
    check_frozen_inputs(root, trial, ready, ready_path)


def finalize_cell(root: Path, ready_path: Path, cell: str) -> None:
    ready = json.loads(ready_path.read_text()); check_ready(ready)
    trial = root / "trials" / cell
    if not (trial / "initial-result.json").is_file() or (trial / "final-result.json").exists():
        raise SystemExit(f"finalize requires an unfinished initial cell: {cell}")
    meta = json.loads((trial / "metadata.json").read_text())
    check_frozen_inputs(root, trial, ready, ready_path)
    if (trial / "repair-result.json").exists() and not (trial / "review-findings.json").exists():
        raise SystemExit("repair result lacks its independent review findings")
    final = validate_source(meta["task"], trial / "project", meta["condition"])
    events = trial / "repair-events.jsonl" if (trial / "repair-events.jsonl").exists() else trial / "initial-events.jsonl"
    feedback = validate_feedback_evidence(trial / "project", events, ready, meta["condition"], meta["task"])
    final["findings"].extend(feedback["findings"])
    # A structurally valid tree is not a successful authoring turn. Keep
    # timeout, nonzero and approval-blocked phases as failures in the final
    # record instead of allowing a later static check to turn them green.
    for phase in ("initial", "repair"):
        result_path = trial / f"{phase}-result.json"
        if not result_path.is_file():
            continue
        result = json.loads(result_path.read_text())
        if result.get("exit_code") != 0 or result.get("timed_out") or result.get("approval_signals") or result.get("blocked_signals"):
            final["findings"].append(f"{phase} model phase did not complete successfully")
    final["observations"] = feedback["observations"]
    final["passed"] = not final["findings"]
    final["visual_motion_semantics"] = "requires independent decoded-pixel review"
    ledger_phase(root, cell, "final", "started")
    write_json(trial / "final-result.json", final)
    snapshot(trial, "final")
    ledger_phase(root, cell, "final", "completed")
    (trial / "codex-home" / "auth.json").unlink(missing_ok=True)


def main() -> int:
    if len(sys.argv) > 1 and sys.argv[1] == "matched":
        # Separate study definition; reuse this runner's native Codex process
        # and snapshot helpers without changing the historical feedback pilot.
        import matched_design_study
        return matched_design_study.main(sys.argv[2:])
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    prep = sub.add_parser("prepare")
    prep.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    prep.add_argument("--ready-manifest", type=Path, required=True)
    prep.add_argument("--font", type=Path, required=True)
    run = sub.add_parser("initial")
    run.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    run.add_argument("--ready-manifest", type=Path, required=True)
    run.add_argument("--cell", choices=[f"{t}-{c}-r{r}" for t in TASKS for c in CONDITIONS for r in REPEATS], required=True)
    repair = sub.add_parser("repair")
    repair.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    repair.add_argument("--ready-manifest", type=Path, required=True)
    repair.add_argument("--cell", choices=[f"{t}-{c}-r{r}" for t in TASKS for c in CONDITIONS for r in REPEATS], required=True)
    repair.add_argument("--review-findings", type=Path, required=True)
    finalize = sub.add_parser("finalize")
    finalize.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    finalize.add_argument("--ready-manifest", type=Path, required=True)
    finalize.add_argument("--cell", choices=[f"{t}-{c}-r{r}" for t in TASKS for c in CONDITIONS for r in REPEATS], required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.root, args.ready_manifest, args.font)
    elif args.command == "initial":
        run_cell(args.root, args.ready_manifest, args.cell)
    elif args.command == "repair":
        repair_cell(args.root, args.ready_manifest, args.cell, args.review_findings)
    else:
        finalize_cell(args.root, args.ready_manifest, args.cell)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
