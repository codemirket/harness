"""Retained domain fixtures and independent oracles for pi qualification.

No host launch, authentication, configuration or legacy transcript handling.
"""

from __future__ import annotations

import fcntl

import hashlib

import json

import os

import shutil

import stat

import subprocess

import time

from fractions import Fraction

from pathlib import Path

from typing import Any

TASKS = ("kinetic-type", "vector-choreography", "product-ui-sequence")

REVIEW_SAMPLE_FRAMES = (0, 6, 12, 18, 24, 30, 35)

def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def manifest(root: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise SystemExit(f"symlink refused in scoped tree: {path}")
        if path.is_file():
            values[str(path.relative_to(root))] = sha(path)
    return values

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
    """Require an actual successful pi tool result containing image bytes."""
    for line in events.read_text(errors="replace").splitlines() if events.exists() else []:
        try:
            event = json.loads(line)
        except ValueError:
            continue
        if not isinstance(event, dict) or event.get("type") != "tool_execution_end" or event.get("isError"):
            continue
        content = event.get("result", {}).get("content", [])
        if any(isinstance(block, dict) and block.get("type") == "image" and block.get("data") for block in content):
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

def snapshot(trial: Path, label: str) -> dict[str, str]:
    destination = trial / f"{label}-project-snapshot"
    if os.path.lexists(destination):
        raise SystemExit(f"snapshot already exists: {destination}")
    source = trial / "project"
    # Refuse a pre-existing (including broken) destination rather than ever
    # opening through a path that could be redirected by a symlink.
    destination.mkdir(parents=True)
    rejected: list[dict[str, str]] = []
    safe_files: dict[str, str] = {}
    nofollow = getattr(os, "O_NOFOLLOW", 0)
    cloexec = getattr(os, "O_CLOEXEC", 0)

    def copy_tree(current_fd: int, relative_root: Path, target: Path) -> None:
        # Every source lookup is relative to a held directory descriptor. The
        # no-follow open closes the stat-then-open replacement race: a file
        # swapped for a symlink after scandir is rejected at open time.
        try:
            entries = list(os.scandir(current_fd))
        except OSError as exc:
            rejected.append({"path": str(relative_root), "kind": "unreadable", "error": str(exc)})
            return
        for entry in entries:
            relative = relative_root / entry.name
            out = target / entry.name
            try:
                info = os.stat(entry.name, dir_fd=current_fd, follow_symlinks=False)
            except OSError as exc:
                rejected.append({"path": str(relative), "kind": "unstatable", "error": str(exc)})
                continue
            mode = info.st_mode
            if stat.S_ISLNK(mode):
                try:
                    link_target = os.readlink(entry.name, dir_fd=current_fd)
                except OSError as exc:
                    link_target = f"<unreadable: {exc}>"
                rejected.append({"path": str(relative), "kind": "symlink", "target": link_target})
            elif stat.S_ISDIR(mode):
                try:
                    child_fd = os.open(entry.name, os.O_RDONLY | os.O_DIRECTORY | nofollow | cloexec, dir_fd=current_fd)
                except OSError as exc:
                    rejected.append({"path": str(relative), "kind": "directory_open_failed", "error": str(exc)})
                    continue
                try:
                    out.mkdir()
                    copy_tree(child_fd, relative, out)
                except OSError as exc:
                    rejected.append({"path": str(relative), "kind": "directory_copy_failed", "error": str(exc)})
                finally:
                    os.close(child_fd)
            elif stat.S_ISREG(mode):
                out.parent.mkdir(parents=True, exist_ok=True)
                source_fd = None
                destination_fd = None
                try:
                    source_fd = os.open(entry.name, os.O_RDONLY | os.O_NONBLOCK | nofollow | cloexec, dir_fd=current_fd)
                    source_info = os.fstat(source_fd)
                    if not stat.S_ISREG(source_info.st_mode):
                        raise OSError("source changed to a non-regular file")
                    destination_fd = os.open(out, os.O_WRONLY | os.O_CREAT | os.O_EXCL | cloexec, 0o600)
                    digest = hashlib.sha256()
                    while True:
                        chunk = os.read(source_fd, 1024 * 1024)
                        if not chunk:
                            break
                        offset = 0
                        while offset < len(chunk):
                            offset += os.write(destination_fd, chunk[offset:])
                        digest.update(chunk)
                    safe_files[str(relative)] = digest.hexdigest()
                except OSError as exc:
                    rejected.append({"path": str(relative), "kind": "copy_failed", "error": str(exc)})
                    try:
                        os.unlink(out)
                    except FileNotFoundError:
                        pass
                finally:
                    if source_fd is not None:
                        os.close(source_fd)
                    if destination_fd is not None:
                        os.close(destination_fd)
            else:
                rejected.append({"path": str(relative), "kind": "special"})

    try:
        root_fd = os.open(source, os.O_RDONLY | os.O_DIRECTORY | nofollow | cloexec)
    except OSError as exc:
        rejected.append({"path": ".", "kind": "root_open_failed", "error": str(exc)})
        root_fd = None
    if root_fd is not None:
        try:
            copy_tree(root_fd, Path("."), destination)
        finally:
            os.close(root_fd)
    if rejected:
        failure = {"label": label, "source": str(source), "status": "rejected", "entries": rejected, "safe_files": safe_files}
        write_json(trial / f"{label}-project-failure.json", failure)
        raise SystemExit(f"snapshot refused unsafe entries: {len(rejected)}")
    value = manifest(destination)
    write_json(trial / f"{label}-project-manifest.json", value)
    return value
