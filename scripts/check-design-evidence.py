#!/usr/bin/env python3
"""Validate a frozen, trust-based design-review evidence packet.

This utility checks packet structure, hashes, relative artifact paths, explicit
states, findings, and one shared repair ledger. It does not open a browser,
render pixels, call an MCP server, or decide whether a visual claim is true.
``visual_report`` checks are retained as reported attestations only; the output
never certifies that a person or model actually saw pixels. This is a
trust-based record-integrity check: a malicious author who rewrites the whole
packet cannot be detected by this utility.

An optional frozen ``artifact_scope`` limits files below the evidence root.
Without it, scope is explicitly unrecorded. The resulting inventory observes
that root only; it does not audit writes elsewhere or establish a sandbox.
Scope entries permit outputs; they do not require optional outputs to exist.

Recorded repairs require candidate source manifests, preserved snapshots and
source-bound captures linked through one shared ledger. Resolved/rejected
findings require explicit dispositions and review references; these establish
record relationships, not the semantic truth of the reported judgment.

Example::

    python3 scripts/check-design-evidence.py --packet review.json --root ./review
"""

from __future__ import annotations

import argparse
from datetime import datetime
import hashlib
import json
import math
import os
from pathlib import Path
import re
import stat
import sys
from typing import Any


SHA256 = re.compile(r"^[0-9a-f]{64}$")
ISO = re.compile(r"^\d{4}-\d{2}-\d{2}T")
VISUAL_REPORT = "visual_report"
CHECK_KINDS = {"file", "automated", VISUAL_REPORT}
SEVERITIES = {"info", "minor", "major", "blocking"}
FINDING_STATUSES = {"open", "resolved", "rejected"}
RESET_FIELDS = {"budget", "repair_limit", "repairs_used"}
PIXEL_CLAIM_FIELDS = {"pixels_seen", "saw_pixels", "pixel_inspection"}
MOTION_CAPTURE_KINDS = {"preview", "decoded_export"}
MEDIA_PROFILE_KEYS = {"width", "height", "duration_seconds", "duration_tolerance_seconds"}
EXECUTION_KEYS = {
    "platform", "architecture", "pi_release_id", "mirket_release_id",
    "model", "reasoning_effort", "catalog_sha256", "skills", "tools",
    "runner_version",
}
REFERENCE_KINDS = {"asset", "reference"}


class EvidenceError(ValueError):
    """Raised when a packet cannot support an auditable review record."""


def _map(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise EvidenceError(f"{label} must be an object")
    return value


def _list(value: Any, label: str) -> list[Any]:
    if not isinstance(value, list) or not value:
        raise EvidenceError(f"{label} must be a non-empty array")
    return value


def _text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise EvidenceError(f"{label} must be non-empty text")
    return value


def _sha(value: Any, label: str) -> str:
    value = _text(value, label)
    if not SHA256.fullmatch(value):
        raise EvidenceError(f"{label} must be a lowercase SHA-256 digest")
    return value


def _revision(value: Any, label: str) -> str:
    value = _text(value, label)
    if not value.startswith("sha256:") or not SHA256.fullmatch(value[7:]):
        raise EvidenceError(f"{label} must be a sha256:<lowercase digest> revision")
    return value


def _absolute_path(value: Any, label: str) -> str:
    value = _text(value, label)
    path = Path(value)
    if not path.is_absolute() or "\\" in value or any(part in {"", ".", ".."} for part in path.parts[1:]):
        raise EvidenceError(f"{label} must be a canonical absolute path")
    return value


def _number(value: Any, label: str, *, positive: bool = False) -> float:
    if isinstance(value, str):
        try:
            value = float(value)
        except ValueError as exc:
            raise EvidenceError(f"{label} must be a finite number") from exc
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise EvidenceError(f"{label} must be a finite number")
    if positive and value <= 0:
        raise EvidenceError(f"{label} must be positive")
    if not positive and value < 0:
        raise EvidenceError(f"{label} must be non-negative")
    return float(value)


def _rational(value: Any, label: str) -> tuple[int, int]:
    value = _text(value, label)
    parts = value.split("/")
    if len(parts) != 2:
        raise EvidenceError(f"{label} must be a rational numerator/denominator")
    try:
        numerator, denominator = (int(part) for part in parts)
    except ValueError as exc:
        raise EvidenceError(f"{label} must be a rational numerator/denominator") from exc
    if numerator <= 0 or denominator <= 0:
        raise EvidenceError(f"{label} must be positive")
    return numerator, denominator


def _count(value: Any, label: str) -> int:
    if isinstance(value, bool) or isinstance(value, int):
        result = value
    elif isinstance(value, str) and value.isdigit():
        result = int(value)
    else:
        raise EvidenceError(f"{label} must be a positive frame count")
    if result <= 0:
        raise EvidenceError(f"{label} must be a positive frame count")
    return result


def _canonical_sha(value: Any) -> str:
    data = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def _parse_time(value: Any, label: str) -> datetime:
    value = _text(value, label)
    if not ISO.match(value):
        raise EvidenceError(f"{label} must be an ISO-8601 timestamp")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise EvidenceError(f"{label} is not a valid timestamp") from exc
    if parsed.tzinfo is None:
        raise EvidenceError(f"{label} must include a timezone")
    return parsed


def _nonnegative_int(value: Any, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise EvidenceError(f"{label} must be a non-negative integer")
    return value


def _positive_int(value: Any, label: str) -> int:
    value = _nonnegative_int(value, label)
    if value == 0:
        raise EvidenceError(f"{label} must be positive")
    return value


def _scan_for_fake_verified(value: Any, location: str = "packet") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            key_text = str(key).lower()
            if key_text in {"verified", "verification_status"}:
                raise EvidenceError(f"{location}.{key} is a blanket verification field")
            if key_text in PIXEL_CLAIM_FIELDS:
                raise EvidenceError(f"{location}.{key} cannot certify that pixels were seen")
            if isinstance(child, str) and child.lower() == "verified":
                raise EvidenceError(f"{location}.{key} uses the forbidden verified enum")
            _scan_for_fake_verified(child, f"{location}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _scan_for_fake_verified(child, f"{location}[{index}]")


def _safe_file(root: Path, relative: Any, label: str) -> Path:
    relative = _text(relative, label)
    path = Path(relative)
    if path.is_absolute() or "\\" in relative or ":" in relative:
        raise EvidenceError(f"{label} must be a portable relative path")
    parts = path.parts
    if not parts or any(part in {"", ".", ".."} for part in parts):
        raise EvidenceError(f"{label} contains an unsafe path component")
    current = root
    for part in parts:
        current = current / part
        if current.is_symlink():
            raise EvidenceError(f"{label} traverses a symlink")
        if not current.exists():
            raise EvidenceError(f"{label} does not exist: {relative}")
    resolved_root = root.resolve()
    resolved = current.resolve()
    try:
        resolved.relative_to(resolved_root)
    except ValueError as exc:
        raise EvidenceError(f"{label} escapes the artifact root") from exc
    if not stat.S_ISREG(current.stat(follow_symlinks=False).st_mode):
        raise EvidenceError(f"{label} is not a regular file")
    return current


def _digest(path: Path) -> str:
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    try:
        descriptor = os.open(path, flags)
    except OSError as exc:
        raise EvidenceError(f"cannot open artifact safely: {path}") from exc
    with os.fdopen(descriptor, "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise EvidenceError(f"artifact is not a regular file: {path}")
        digest = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
        return digest.hexdigest()


def _read_json(root: Path, relative: Any, label: str) -> tuple[Path, str, dict[str, Any]]:
    path = _safe_file(root, relative, f"{label}.path")
    digest = _digest(path)
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise EvidenceError(f"{label} must be a readable JSON object") from exc
    return path, digest, _map(value, label)


def _within_window(moment: datetime, label: str, started: datetime, deadline: datetime) -> None:
    if not started <= moment <= deadline:
        raise EvidenceError(f"{label} falls outside the shared budget window")


def _validate_scope(packet: dict[str, Any], contract: dict[str, Any], root: Path,
                    packet_path: Path | None) -> list[dict[str, str]] | None:
    scope = packet.get("artifact_scope")
    if scope is None:
        if "artifact_scope_sha256" in contract:
            raise EvidenceError("frozen artifact scope is missing")
        return None
    scope = _map(scope, "artifact_scope")
    if set(scope) != {"files", "directories"}:
        raise EvidenceError("artifact_scope must contain only files and directories")
    for kind in ("files", "directories"):
        values = scope[kind]
        if not isinstance(values, list):
            raise EvidenceError(f"artifact_scope.{kind} must be an array")
        seen: set[str] = set()
        for value in values:
            value = _text(value, f"artifact_scope.{kind}")
            if (Path(value).is_absolute() or "\\" in value or ":" in value
                    or any(part in {"", ".", ".."} for part in value.split("/"))):
                raise EvidenceError("artifact_scope requires portable relative paths without traversal")
            if value in seen:
                raise EvidenceError("artifact_scope contains duplicate paths")
            seen.add(value)
    files, directories = set(scope["files"]), set(scope["directories"])
    if not files and not directories:
        raise EvidenceError("artifact_scope must permit at least one file or directory")
    if files & directories:
        raise EvidenceError("artifact_scope cannot name a path as both file and directory")
    if contract.get("artifact_scope_sha256") != _canonical_sha(scope):
        raise EvidenceError("artifact scope changed or has a stale hash")

    # Only the caller's actual packet file is excluded, never a packet-declared
    # filename that could hide an unrelated output from the inventory.
    excluded = None
    if packet_path is not None:
        try:
            excluded = packet_path.absolute().relative_to(root).as_posix()
        except ValueError:
            pass
        else:
            _safe_file(root, excluded, "packet_path")

    def inside_directory(relative: str) -> bool:
        return any(relative.startswith(directory + "/") for directory in directories)

    def walk_error(error: OSError) -> None:
        raise EvidenceError(f"cannot enumerate artifact root: {error}") from error

    inventory: list[dict[str, str]] = []
    for current, child_dirs, child_files in os.walk(root, followlinks=False, onerror=walk_error):
        for name in sorted(child_dirs + child_files):
            path = Path(current) / name
            relative = path.relative_to(root).as_posix()
            metadata = path.lstat()
            if stat.S_ISLNK(metadata.st_mode):
                raise EvidenceError(f"artifact scope traverses a symlink: {relative}")
            if stat.S_ISDIR(metadata.st_mode):
                if relative in files:
                    raise EvidenceError(f"artifact scope declares a directory as a file: {relative}")
                allowed_parent = any(value.startswith(relative + "/") for value in files | directories)
                packet_parent = excluded is not None and excluded.startswith(relative + "/")
                if not (relative in directories or inside_directory(relative) or allowed_parent or packet_parent):
                    raise EvidenceError(f"directory outside artifact scope: {relative}")
            elif stat.S_ISREG(metadata.st_mode):
                if relative in directories:
                    raise EvidenceError(f"artifact scope declares a file as a directory: {relative}")
                if relative == excluded:
                    continue
                if relative not in files and not inside_directory(relative):
                    raise EvidenceError(f"file outside artifact scope: {relative}")
                inventory.append({"path": relative, "sha256": _digest(path)})
            else:
                raise EvidenceError(f"artifact scope contains a non-regular entry: {relative}")
    return sorted(inventory, key=lambda item: item["path"])


def _validate_artifact(root: Path, item: Any, label: str, states: dict[str, dict[str, Any]], require_state: bool) -> dict[str, Any]:
    item = _map(item, label)
    artifact_id = _text(item.get("id"), f"{label}.id")
    path = _safe_file(root, item.get("path"), f"{label}.path")
    expected = _sha(item.get("sha256"), f"{label}.sha256")
    actual = _digest(path)
    if actual != expected:
        raise EvidenceError(f"stale artifact hash: {label}.path")
    _text(item.get("role"), f"{label}.role")
    if require_state or "state_id" in item:
        state_id = _text(item.get("state_id"), f"{label}.state_id")
        if state_id not in states:
            raise EvidenceError(f"{label} references an unknown state")
        state = states[state_id]
        if item.get("route_intent") != state["route_intent"]:
            raise EvidenceError(f"{label}.route_intent differs from its state")
        viewport = _map(item.get("viewport"), f"{label}.viewport")
        if viewport != state["viewport"]:
            raise EvidenceError(f"{label}.viewport differs from its state")
        _parse_time(item.get("captured_at"), f"{label}.captured_at")
    return {"id": artifact_id, "path": path.as_posix(), "sha256": actual}


def _validate_motion(packet: dict[str, Any], contract: dict[str, Any], states: dict[str, dict[str, Any]]) -> tuple[dict[str, Any] | None, dict[str, dict[str, Any]]]:
    raw_motion = packet.get("motion")
    if raw_motion is None:
        return None, {}
    motion = _map(raw_motion, "motion")
    if any(field in motion for field in RESET_FIELDS):
        raise EvidenceError("motion cannot reset the shared budget")
    fps = _map(motion.get("fps"), "motion.fps")
    numerator = _positive_int(fps.get("numerator"), "motion.fps.numerator")
    denominator = _positive_int(fps.get("denominator"), "motion.fps.denominator")
    if set(fps) != {"numerator", "denominator"}:
        raise EvidenceError("motion.fps must be a rational numerator and denominator")
    frame_count = _positive_int(motion.get("frame_count"), "motion.frame_count")
    parity_claim = motion.get("parity_claim")
    if not isinstance(parity_claim, bool):
        raise EvidenceError("motion.parity_claim must be boolean")
    beats = _list(motion.get("beats"), "motion.beats")
    beat_map: dict[str, dict[str, Any]] = {}
    for index, raw_beat in enumerate(beats):
        beat = _map(raw_beat, f"motion.beats[{index}]")
        beat_id = _text(beat.get("id"), f"motion.beats[{index}].id")
        if beat_id in beat_map:
            raise EvidenceError("motion beats contain duplicate ids")
        frame = beat.get("frame")
        if isinstance(frame, bool) or not isinstance(frame, int) or not 0 <= frame < frame_count:
            raise EvidenceError(f"motion.beats[{index}].frame is outside frame_count")
        state_id = _text(beat.get("state_id"), f"motion.beats[{index}].state_id")
        if state_id not in states:
            raise EvidenceError(f"motion beat {beat_id} references an unknown state")
        expected = _text(beat.get("expected"), f"motion.beats[{index}].expected")
        beat_map[beat_id] = {"frame": frame, "state_id": state_id, "expected": expected}
    media_profile = motion.get("media_profile")
    if media_profile is not None:
        media_profile = _map(media_profile, "motion.media_profile")
        if set(media_profile) != MEDIA_PROFILE_KEYS:
            raise EvidenceError("motion.media_profile has an unsupported field set")
        width = media_profile.get("width")
        height = media_profile.get("height")
        if isinstance(width, bool) or not isinstance(width, int) or width <= 0:
            raise EvidenceError("motion.media_profile.width must be positive")
        if isinstance(height, bool) or not isinstance(height, int) or height <= 0:
            raise EvidenceError("motion.media_profile.height must be positive")
        duration = _number(media_profile.get("duration_seconds"), "motion.media_profile.duration_seconds", positive=True)
        tolerance = _number(media_profile.get("duration_tolerance_seconds"), "motion.media_profile.duration_tolerance_seconds")
        if tolerance > duration:
            raise EvidenceError("motion.media_profile.duration_tolerance_seconds exceeds duration")
        media_profile = {
            "width": width, "height": height,
            "duration_seconds": duration,
            "duration_tolerance_seconds": tolerance,
        }
    if contract.get("motion_sha256") != _canonical_sha(motion):
        raise EvidenceError("motion expectations changed or have a stale hash")
    # Keep these reads explicit so a future caller cannot accidentally treat the
    # rational timing fields as a float or infer units from a label.
    _ = (numerator, denominator, parity_claim)
    return motion, beat_map


def _validate_execution(contract: dict[str, Any]) -> dict[str, Any] | None:
    execution = contract.get("execution")
    if execution is None:
        if "execution_sha256" in contract:
            raise EvidenceError("frozen execution identity is missing")
        return None
    execution = _map(execution, "contract.execution")
    if set(execution) != EXECUTION_KEYS:
        raise EvidenceError("contract.execution has an unsupported field set")
    for key in ("platform", "architecture", "pi_release_id", "mirket_release_id", "model", "reasoning_effort", "runner_version"):
        _text(execution.get(key), f"contract.execution.{key}")
    _sha(execution.get("catalog_sha256"), "contract.execution.catalog_sha256")
    skills = execution.get("skills")
    if not isinstance(skills, dict):
        raise EvidenceError("contract.execution.skills must be an object")
    for name, digest in skills.items():
        _text(name, "contract.execution.skills name")
        _sha(digest, f"contract.execution.skills.{name}")
    tools = execution.get("tools")
    if not isinstance(tools, dict):
        raise EvidenceError("contract.execution.tools must be an object")
    for name, identity in tools.items():
        _text(name, "contract.execution.tools name")
        identity = _map(identity, f"contract.execution.tools.{name}")
        if set(identity) != {"path", "sha256"}:
            raise EvidenceError(f"contract.execution.tools.{name} has an unsupported field set")
        _absolute_path(identity.get("path"), f"contract.execution.tools.{name}.path")
        _sha(identity.get("sha256"), f"contract.execution.tools.{name}.sha256")
    if contract.get("execution_sha256") != _canonical_sha(execution):
        raise EvidenceError("frozen execution identity changed or has a stale hash")
    return execution


def _validate_references(packet: dict[str, Any], root: Path, contract: dict[str, Any]) -> str:
    references = packet.get("references")
    if references is None:
        if "references_sha256" in contract:
            raise EvidenceError("frozen reference rows are missing")
        return "unrecorded"
    references = _list(references, "references")
    if contract.get("references_sha256") != _canonical_sha(references):
        raise EvidenceError("reference rows changed or have a stale hash")
    seen: set[str] = set()
    for index, raw in enumerate(references):
        reference = _map(raw, f"references[{index}]")
        required = {"id", "kind", "source", "creator", "license", "attribution", "permitted_use"}
        if set(reference) - required - {"revision", "artifact_paths", "artifact_sha256"}:
            raise EvidenceError(f"references[{index}] has an unsupported field")
        reference_id = _text(reference.get("id"), f"references[{index}].id")
        if reference_id in seen:
            raise EvidenceError("references contain duplicate ids")
        seen.add(reference_id)
        kind = _text(reference.get("kind"), f"references[{index}].kind")
        if kind not in REFERENCE_KINDS:
            raise EvidenceError(f"references[{index}].kind must be asset or reference")
        for key in ("source", "creator", "license", "attribution", "permitted_use"):
            _text(reference.get(key), f"references[{index}].{key}")
        if "revision" in reference:
            _text(reference.get("revision"), f"references[{index}].revision")
        paths = reference.get("artifact_paths")
        digests = reference.get("artifact_sha256")
        if paths is not None:
            if not isinstance(paths, list) or (kind == "asset" and not paths):
                raise EvidenceError(f"references[{index}].artifact_paths must name asset files")
            path_values = [
                _text(path, f"references[{index}].artifact_paths[{path_index}]")
                for path_index, path in enumerate(paths)
            ]
            if len(set(path_values)) != len(path_values):
                raise EvidenceError(f"references[{index}].artifact_paths contains duplicates")
            if not isinstance(digests, dict) or set(digests) != set(path_values):
                raise EvidenceError(f"references[{index}] artifact_sha256 must cover every artifact path")
            for path_index, path in enumerate(path_values):
                artifact = _safe_file(root, path, f"references[{index}].artifact_paths[{path_index}]")
                expected = _sha(digests[path], f"references[{index}].artifact_sha256.{path}")
                if _digest(artifact) != expected:
                    raise EvidenceError(f"stale referenced asset bytes: {path}")
        elif digests is not None:
            raise EvidenceError(f"references[{index}] artifact_sha256 needs artifact_paths")
        elif kind == "asset":
            raise EvidenceError(f"references[{index}] asset needs artifact_paths")
    return "recorded"


def _validate_media_probe(
    root: Path, motion: dict[str, Any] | None,
    candidate: dict[str, Any], captures: dict[str, dict[str, Any]],
) -> dict[str, Any] | None:
    raw_probe = candidate.get("media_probe")
    if raw_probe is None:
        return None
    if motion is None or "media_profile" not in motion:
        raise EvidenceError("media_probe requires frozen motion.media_profile")
    probe = _map(raw_probe, f"candidate {candidate['id']}.media_probe")
    required = {"id", "path", "sha256", "export_path", "export_sha256", "export_revision", "capture_refs", "decoder"}
    if set(probe) != required:
        raise EvidenceError(f"candidate {candidate['id']}.media_probe has an unsupported field set")
    probe_id = _text(probe.get("id"), f"candidate {candidate['id']}.media_probe.id")
    probe_path, probe_digest, raw = _read_json(root, probe.get("path"), f"candidate {candidate['id']}.media_probe")
    if probe_digest != _sha(probe.get("sha256"), f"candidate {candidate['id']}.media_probe.sha256"):
        raise EvidenceError("media probe has a stale hash")
    export_path = _safe_file(root, probe.get("export_path"), f"candidate {candidate['id']}.media_probe.export_path")
    export_digest = _digest(export_path)
    if export_digest != _sha(probe.get("export_sha256"), f"candidate {candidate['id']}.media_probe.export_sha256"):
        raise EvidenceError("media export has a stale hash")
    export_revision = _revision(probe.get("export_revision"), f"candidate {candidate['id']}.media_probe.export_revision")
    if export_revision != f"sha256:{export_digest}":
        raise EvidenceError("media export revision does not match its bytes")
    _text(probe.get("decoder"), f"candidate {candidate['id']}.media_probe.decoder")

    refs = probe.get("capture_refs")
    if not isinstance(refs, list) or not refs or len(set(refs)) != len(refs):
        raise EvidenceError("media_probe.capture_refs must be a non-empty unique array")
    for ref_index, capture_id in enumerate(refs):
        capture_id = _text(capture_id, f"media_probe.capture_refs[{ref_index}]")
        capture = captures.get(capture_id)
        if capture is None:
            raise EvidenceError("media_probe references an unknown capture")
        if capture.get("capture_kind") != "decoded_export":
            raise EvidenceError("media_probe capture_refs must point to decoded_export captures")
        if capture.get("media_probe_id") != probe_id:
            raise EvidenceError("decoded_export capture is not linked to its media probe")
        if capture.get("path") == probe["export_path"]:
            raise EvidenceError("media probe export cannot reuse a decoded capture path")

    streams = raw.get("streams")
    if not isinstance(streams, list):
        raise EvidenceError("media probe JSON lacks streams")
    videos = [stream for stream in streams if isinstance(stream, dict) and stream.get("codec_type") == "video"]
    if len(videos) != 1:
        raise EvidenceError("media probe JSON must contain exactly one video stream")
    video = videos[0]
    width = video.get("width")
    height = video.get("height")
    profile = motion["media_profile"]
    if width != profile["width"] or height != profile["height"]:
        raise EvidenceError("media probe dimensions differ from frozen media profile")
    frame_count = _count(video.get("nb_read_frames", video.get("nb_frames")), "media probe frame count")
    if frame_count != motion["frame_count"]:
        raise EvidenceError("media probe frame count differs from frozen motion")
    fps = video.get("avg_frame_rate", video.get("r_frame_rate"))
    numerator, denominator = _rational(fps, "media probe frame rate")
    if numerator != motion["fps"]["numerator"] or denominator != motion["fps"]["denominator"]:
        raise EvidenceError("media probe frame rate differs from frozen motion")
    fmt = raw.get("format")
    if not isinstance(fmt, dict):
        raise EvidenceError("media probe JSON lacks format")
    duration = _number(fmt.get("duration"), "media probe duration", positive=True)
    if abs(duration - profile["duration_seconds"]) > profile["duration_tolerance_seconds"]:
        raise EvidenceError("media probe duration differs from frozen media profile")
    return {"id": probe_id, "export_path": export_path.as_posix(), "export_revision": export_revision,
            "probe_path": probe_path.as_posix(), "capture_refs": refs}


def _validate_operation_binding(
    contract: dict[str, Any], root: Path, selected_candidate: str,
) -> dict[str, Any] | None:
    raw_binding = contract.get("operation_binding")
    if raw_binding is None:
        if "operation_binding_sha256" in contract:
            raise EvidenceError("operation binding is missing")
        return None
    binding = _map(raw_binding, "contract.operation_binding")
    required = {"candidate_id", "receipt_path", "receipt_sha256", "operation_id", "release_id", "project_root", "destination_path", "artifact_path", "artifact_revision"}
    if set(binding) != required:
        raise EvidenceError("contract.operation_binding has an unsupported field set")
    if binding["candidate_id"] != selected_candidate:
        raise EvidenceError("operation binding must name the selected candidate")
    if contract.get("operation_binding_sha256") != _canonical_sha(binding):
        raise EvidenceError("operation binding changed or has a stale hash")
    receipt_path, receipt_digest, stored_receipt = _read_json(root, binding["receipt_path"], "operation receipt")
    if receipt_digest != _sha(binding["receipt_sha256"], "operation_binding.receipt_sha256"):
        raise EvidenceError("operation receipt has a stale hash")
    # The operation store persists an envelope whose UTF-8 payload is the
    # engine receipt. Evidence runners may instead retain the already-decoded
    # report returned by the host. Accept both representations, but authenticate
    # the store payload digest when the envelope is present.
    if "payload" in stored_receipt:
        if stored_receipt.get("schema") != 2:
            raise EvidenceError("operation receipt envelope schema is unsupported")
        payload = _text(stored_receipt.get("payload"), "operation receipt.payload")
        issued = _sha(stored_receipt.get("issued_payload_sha256"), "operation receipt.issued_payload_sha256")
        if hashlib.sha256(payload.encode("utf-8")).hexdigest() != issued:
            raise EvidenceError("operation receipt envelope payload fingerprint differs")
        try:
            receipt = _map(json.loads(payload), "operation receipt.payload")
        except (UnicodeError, json.JSONDecodeError) as exc:
            raise EvidenceError("operation receipt payload is not JSON") from exc
    elif isinstance(stored_receipt.get("result"), dict):
        # Retained qualification runs wrap the host report with runtime and
        # probe observations. Bind only the nested engine report; the outer
        # wrapper remains authenticated by receipt_sha256.
        receipt = stored_receipt["result"]
    else:
        receipt = stored_receipt
    for key in ("operation_id", "release_id", "project_root", "destination_path"):
        _text(receipt.get(key), f"operation receipt.{key}")
    if receipt["operation_id"] != binding["operation_id"]:
        raise EvidenceError("operation receipt ID differs from the binding")
    if receipt["release_id"] != binding["release_id"]:
        raise EvidenceError("operation receipt release differs from the binding")
    if receipt["project_root"] != binding["project_root"]:
        raise EvidenceError("operation receipt project differs from the binding")
    if receipt["destination_path"] != binding["destination_path"]:
        raise EvidenceError("operation receipt destination differs from the binding")
    _absolute_path(binding["project_root"], "operation_binding.project_root")
    _absolute_path(binding["destination_path"], "operation_binding.destination_path")
    if receipt.get("status") != "succeeded" or receipt.get("effect") != "confirmed":
        raise EvidenceError("operation receipt is not a succeeded, confirmed effect")
    receipt_revision = _revision(receipt.get("artifact_revision"), "operation receipt.artifact_revision")
    if binding["artifact_revision"] != receipt_revision:
        raise EvidenceError("operation receipt artifact revision differs from the binding")
    artifact_path = _safe_file(root, binding["artifact_path"], "operation_binding.artifact_path")
    if receipt_revision != f"sha256:{_digest(artifact_path)}":
        raise EvidenceError("operation artifact bytes differ from its receipt revision")
    return {"receipt_path": receipt_path.as_posix(), "artifact_path": artifact_path.as_posix(),
            "artifact_revision": receipt_revision, "operation_id": binding["operation_id"]}


def _validate_lineage(packet: dict[str, Any], root: Path, started: datetime,
                      deadline: datetime, selected: str) -> dict[str, Any]:
    """Check record relationships, never the truth of a reported resolution."""
    candidates = {item["id"]: item for item in packet["candidates"]}
    repairs = {item["id"]: item for item in packet["budget"]["repairs"]}
    contract = packet["contract"]
    parents: dict[str, str] = {}
    events: dict[str, dict[str, Any]] = {}
    addressed: dict[str, set[tuple[str, str]]] = {}
    used: set[str] = set()
    manifests: dict[str, str] = {}
    captures = {cid: {cap["id"]: cap for cap in c["captures"]} for cid, c in candidates.items()}
    findings = {cid: {f["id"]: f for f in c["findings"]} for cid, c in candidates.items()}
    enabled = bool(repairs) or any("lineage" in c or "source_manifest" in c for c in candidates.values())

    for cid, candidate in candidates.items():
        if "lineage" not in candidate:
            continue
        edge = _map(candidate["lineage"], f"candidate {cid}.lineage")
        parent = _text(edge.get("parent_candidate_id"), "lineage.parent_candidate_id")
        if parent not in candidates or parent == cid:
            raise EvidenceError("lineage has an unknown or self parent")
        ref = _text(edge.get("repair_ref"), "lineage.repair_ref")
        if ref not in repairs or ref in used:
            raise EvidenceError("lineage has an unknown or reused repair event")
        event = repairs[ref]
        result_id = event.get("candidate_id") if event["scope"] == "candidate" else event.get("result_candidate_id")
        if result_id != cid:
            raise EvidenceError("repair event does not name its derived candidate")
        parents[cid], events[cid] = parent, event
        used.add(ref)
    if used != set(repairs):
        raise EvidenceError("repair ledger contains an event without candidate lineage")

    def ancestry(cid: str) -> list[str]:
        chain: list[str] = []
        while True:
            if cid in chain:
                raise EvidenceError("candidate lineage contains a cycle")
            chain.append(cid)
            if cid not in parents:
                return chain
            cid = parents[cid]

    chains = {cid: ancestry(cid) for cid in candidates}
    if enabled:
        for cid, candidate in candidates.items():
            manifest = _list(candidate.get("source_manifest"), f"candidate {cid}.source_manifest")
            entries: list[dict[str, str]] = []
            seen: set[str] = set()
            for raw in manifest:
                item = _map(raw, "candidate source entry")
                path = _text(item.get("path"), "candidate source path")
                if path in seen:
                    raise EvidenceError("candidate source manifest has duplicate paths")
                seen.add(path)
                digest = _sha(item.get("sha256"), "candidate source sha256")
                if _digest(_safe_file(root, path, "candidate source")) != digest:
                    raise EvidenceError("stale candidate source manifest hash")
                entries.append({"path": path, "sha256": digest})
            manifest_sha = _canonical_sha(sorted(entries, key=lambda item: item["path"]))
            if candidate.get("source_manifest_sha256") != manifest_sha:
                raise EvidenceError("candidate source manifest reference changed")
            manifests[cid] = manifest_sha
            if cid not in parents and manifest_sha != contract["source_manifest_sha256"]:
                raise EvidenceError("root candidate must retain the frozen initial source manifest")
            snapshots = {s["path"]: s for s in candidate["snapshots"]}
            if len(snapshots) != len(candidate["snapshots"]):
                raise EvidenceError("candidate has duplicate source snapshot paths")
            latest_snapshot = started
            for entry in entries:
                snap = snapshots.get(entry["path"])
                if snap is None or snap["sha256"] != entry["sha256"]:
                    raise EvidenceError("candidate source lacks its preserved snapshot")
                moment = _parse_time(snap.get("captured_at"), "source snapshot.captured_at")
                _within_window(moment, "source snapshot", started, deadline)
                latest_snapshot = max(latest_snapshot, moment)
                if cid in events and moment <= _parse_time(events[cid]["at"], "repair.at"):
                    raise EvidenceError("derived source snapshot must follow its repair event")
            for capture in candidate["captures"]:
                if capture.get("source_manifest_sha256") != manifest_sha:
                    raise EvidenceError("capture is not bound to its candidate source manifest")
                captured_at = _parse_time(capture["captured_at"], "capture.captured_at")
                if cid in parents and captured_at <= latest_snapshot:
                    raise EvidenceError("derived capture must follow its source snapshot")
                if captured_at < latest_snapshot:
                    raise EvidenceError("capture predates its source snapshot")

        for cid, parent in parents.items():
            edge = candidates[cid]["lineage"]
            if edge.get("parent_source_manifest_sha256") != manifests[parent]:
                raise EvidenceError("lineage parent source reference changed")
            moment = _parse_time(events[cid]["at"], "repair.at")
            for capture in candidates[parent]["captures"]:
                if moment <= _parse_time(capture["captured_at"], "parent capture.captured_at"):
                    raise EvidenceError("repair must follow its parent captures")
            ancestor_paths = {cap["path"] for ancestor in chains[cid][1:]
                              for cap in candidates[ancestor]["captures"]}
            if any(cap["path"] in ancestor_paths for cap in candidates[cid]["captures"]):
                raise EvidenceError("derived candidate reuses an ancestor capture path")
            refs: set[tuple[str, str]] = set()
            for raw in _list(edge.get("finding_refs"), "lineage.finding_refs"):
                ref = _map(raw, "lineage finding reference")
                owner = _text(ref.get("candidate_id"), "finding reference.candidate_id")
                fid = _text(ref.get("finding_id"), "finding reference.finding_id")
                if owner not in chains[cid][1:] or fid not in findings[owner]:
                    raise EvidenceError("repair finding must belong to an ancestor candidate")
                if (owner, fid) in refs:
                    raise EvidenceError("repair contains duplicate finding references")
                refs.add((owner, fid))
                finding = findings[owner][fid]
                if moment <= _parse_time(finding["observed_at"], "finding.observed_at"):
                    raise EvidenceError("repair must follow its addressed finding")
                disposition = finding.get("disposition")
                if isinstance(disposition, dict) and moment >= _parse_time(disposition.get("at"), "disposition.at"):
                    raise EvidenceError("repair cannot address an already dispositioned finding")
            addressed[cid] = refs

    resolutions: dict[tuple[str, str], str] = {}
    disposition_count = 0
    for owner, by_id in findings.items():
        for fid, finding in by_id.items():
            disposition = finding.get("disposition")
            if finding["status"] == "open":
                if disposition is not None:
                    raise EvidenceError("open finding cannot carry a final disposition")
                continue
            disposition = _map(disposition, f"finding {fid}.disposition")
            if disposition.get("kind") != finding["status"]:
                raise EvidenceError("finding status differs from its disposition")
            _text(disposition.get("reason"), "disposition.reason")
            for key in ("brief_sha256", "rubric_sha256"):
                if disposition.get(key) != contract[key]:
                    raise EvidenceError("disposition changed its frozen criterion")
            at = _parse_time(disposition.get("at"), "disposition.at")
            _within_window(at, "disposition", started, deadline)
            if at < _parse_time(finding["observed_at"], "finding.observed_at"):
                raise EvidenceError("disposition predates its finding")
            evidence = _map(disposition.get("evidence"), "disposition.evidence")
            target = _text(evidence.get("candidate_id"), "disposition candidate_id")
            cap_id = _text(evidence.get("capture_ref"), "disposition capture_ref")
            if target not in candidates or cap_id not in captures[target]:
                raise EvidenceError("disposition references unknown capture evidence")
            cap = captures[target][cap_id]
            check_id = _text(evidence.get("check_ref"), "disposition check_ref")
            check = next((c for c in candidates[target]["checks"] if c["id"] == check_id), None)
            if check is None or check.get("capture_ref") != cap_id:
                raise EvidenceError("disposition review check is not tied to its capture")
            if check["kind"] == VISUAL_REPORT:
                review_at = _parse_time(check["reported_at"], "review.reported_at")
            elif check["kind"] == "automated" and check["result"] == "passed":
                review_at = _parse_time(check.get("checked_at"), "review.checked_at")
            else:
                raise EvidenceError("disposition requires a visual report or passing automated review")
            _within_window(review_at, "disposition review", started, deadline)
            if not _parse_time(cap["captured_at"], "capture.captured_at") <= review_at <= at:
                raise EvidenceError("disposition review must follow capture and precede disposition")
            before = captures[owner][finding["capture_ref"]]
            if any(cap.get(key) != before.get(key) for key in ("state_id", "beat_id", "frame")):
                raise EvidenceError("disposition evidence changes the finding state or beat")
            if finding["status"] == "rejected":
                if target != owner or cap_id != finding["capture_ref"]:
                    raise EvidenceError("rejected finding must retain review of its original capture")
            else:
                if target not in parents or owner not in chains[target][1:]:
                    raise EvidenceError("resolution requires a descendant repair candidate")
                if (owner, fid) not in addressed[target]:
                    raise EvidenceError("resolution repair did not address this finding")
                if disposition.get("repair_ref") != events[target]["id"]:
                    raise EvidenceError("resolution references the wrong repair event")
                resolutions[(owner, fid)] = target
            disposition_count += 1

    remaining: list[dict[str, str]] = []
    for owner in chains[selected]:
        for fid, finding in findings[owner].items():
            effective = finding["status"] == "rejected" or resolutions.get((owner, fid)) in chains[selected]
            if not effective:
                remaining.append({"candidate_id": owner, "finding_id": fid, "severity": finding["severity"]})
                if finding["severity"] == "blocking":
                    raise EvidenceError("selected lineage retains an unresolved blocking finding")
    return {"repair_lineage": "checked" if enabled else "not_applicable_no_repairs",
            "disposition_count": disposition_count, "remaining_findings": remaining}


def _validate_packet(packet: Any, root: Path, packet_path: Path | None = None) -> dict[str, Any]:
    packet = _map(packet, "packet")
    _scan_for_fake_verified(packet)
    if packet.get("schema_version") != 1:
        raise EvidenceError("schema_version must be 1")
    _text(packet.get("packet_id"), "packet_id")
    if packet.get("artifact_root") != ".":
        raise EvidenceError("artifact_root must be '.' relative to --root")
    root = root.absolute()
    if not root.is_dir() or root.is_symlink():
        raise EvidenceError("--root must be an existing real directory")

    task = _map(packet.get("task"), "task")
    task_id = _text(task.get("id"), "task.id")
    brief = _text(task.get("brief"), "task.brief")
    contract = _map(packet.get("contract"), "contract")
    contract_id = _text(contract.get("id"), "contract.id")
    contract_frozen = _parse_time(contract.get("frozen_at"), "contract.frozen_at")
    if contract.get("task_id") != task_id:
        raise EvidenceError("contract.task_id does not match task.id")
    if contract.get("brief_sha256") != hashlib.sha256(brief.encode("utf-8")).hexdigest():
        raise EvidenceError("frozen task brief hash does not match")
    inventory = _validate_scope(packet, contract, root, packet_path)

    rubric = _map(packet.get("rubric"), "rubric")
    rubric_id = _text(rubric.get("id"), "rubric.id")
    rubric_sha = hashlib.sha256(_text(rubric.get("text"), "rubric.text").encode("utf-8")).hexdigest()
    if rubric.get("sha256") != rubric_sha or contract.get("rubric_sha256") != rubric_sha:
        raise EvidenceError("rubric reference changed or has a stale hash")

    budget = _map(packet.get("budget"), "budget")
    budget_id = _text(budget.get("id"), "budget.id")
    started = _parse_time(budget.get("started_at"), "budget.started_at")
    deadline = _parse_time(budget.get("deadline_at"), "budget.deadline_at")
    if contract_frozen > started:
        raise EvidenceError("contract must be frozen before the shared budget starts")
    if deadline <= started:
        raise EvidenceError("budget deadline must follow its start")
    limit = _nonnegative_int(budget.get("repair_limit"), "budget.repair_limit")
    spec = {"id": budget_id, "started_at": budget["started_at"], "deadline_at": budget["deadline_at"], "repair_limit": limit}
    budget_sha = _canonical_sha(spec)
    if budget.get("spec_sha256") != budget_sha or contract.get("budget_sha256") != budget_sha:
        raise EvidenceError("budget reference changed or has a stale hash")
    repairs = budget.get("repairs")
    if not isinstance(repairs, list) or budget.get("repairs_used") != len(repairs) or len(repairs) > limit:
        raise EvidenceError("repair ledger is missing, inconsistent, or over budget")
    repair_ids: set[str] = set()
    candidate_repair_refs: list[str] = []
    for index, repair in enumerate(repairs):
        repair = _map(repair, f"budget.repairs[{index}]")
        repair_id = _text(repair.get("id"), f"budget.repairs[{index}].id")
        if repair_id in repair_ids:
            raise EvidenceError("repair ledger contains duplicate ids")
        repair_ids.add(repair_id)
        scope = _text(repair.get("scope"), f"budget.repairs[{index}].scope")
        if scope not in {"candidate", "specimen", "delegate"}:
            raise EvidenceError("repair scope must be candidate, specimen, or delegate")
        scope_keys = ("candidate_id", "specimen_id", "delegate_id")
        named = [key for key in scope_keys if repair.get(key)]
        if named != [f"{scope}_id"]:
            raise EvidenceError("repair must name exactly the item matching its scope")
        if scope == "candidate":
            candidate_repair_refs.append(repair["candidate_id"])
        moment = _parse_time(repair.get("at"), f"budget.repairs[{index}].at")
        if not started <= moment <= deadline:
            raise EvidenceError("repair falls outside the shared budget window")

    manifest = _list(packet.get("source_manifest"), "source_manifest")
    manifest_entries: list[dict[str, str]] = []
    seen_manifest: set[str] = set()
    for index, entry in enumerate(manifest):
        entry = _map(entry, f"source_manifest[{index}]")
        relative = _text(entry.get("path"), f"source_manifest[{index}].path")
        if relative in seen_manifest:
            raise EvidenceError("source_manifest contains duplicate paths")
        seen_manifest.add(relative)
        actual_path = _safe_file(root, relative, f"source_manifest[{index}].path")
        expected = _sha(entry.get("sha256"), f"source_manifest[{index}].sha256")
        if _digest(actual_path) != expected:
            raise EvidenceError(f"stale source manifest hash: {relative}")
        manifest_entries.append({"path": relative, "sha256": expected})
    manifest_entries.sort(key=lambda item: item["path"])
    manifest_sha = _canonical_sha(manifest_entries)
    if contract.get("source_manifest_sha256") != manifest_sha:
        raise EvidenceError("source manifest reference changed")

    states_list = _list(packet.get("states"), "states")
    states: dict[str, dict[str, Any]] = {}
    for index, raw_state in enumerate(states_list):
        state = _map(raw_state, f"states[{index}]")
        state_id = _text(state.get("id"), f"states[{index}].id")
        if state_id in states:
            raise EvidenceError("states contain duplicate ids")
        route_intent = _text(state.get("route_intent"), f"states[{index}].route_intent")
        viewport = _map(state.get("viewport"), f"states[{index}].viewport")
        width = viewport.get("width")
        height = viewport.get("height")
        if isinstance(width, bool) or not isinstance(width, int) or width <= 0 or isinstance(height, bool) or not isinstance(height, int) or height <= 0:
            raise EvidenceError(f"states[{index}].viewport must have positive integer width/height")
        _text(state.get("expected"), f"states[{index}].expected")
        states[state_id] = {"route_intent": route_intent, "viewport": {"width": width, "height": height}}
    states_sha = _canonical_sha(states_list)
    if contract.get("states_sha256") != states_sha:
        raise EvidenceError("frozen state expectations changed or have a stale hash")
    motion, beat_map = _validate_motion(packet, contract, states)
    execution = _validate_execution(contract)
    references_status = _validate_references(packet, root, contract)

    candidates = _list(packet.get("candidates"), "candidates")
    candidate_ids: set[str] = set()
    selected: list[str] = []
    visual_reports = 0
    media_probe_results: dict[str, dict[str, Any] | None] = {}
    for index, raw_candidate in enumerate(candidates):
        candidate = _map(raw_candidate, f"candidates[{index}]")
        candidate_id = _text(candidate.get("id"), f"candidates[{index}].id")
        if candidate_id in candidate_ids:
            raise EvidenceError("candidates contain duplicate ids")
        candidate_ids.add(candidate_id)
        if candidate.get("contract_ref") != contract_id or candidate.get("task_id") != task_id:
            raise EvidenceError(f"candidate {candidate_id} does not reference the frozen contract")
        if candidate.get("rubric_sha256") != rubric_sha or candidate.get("budget_ref") != budget_id or candidate.get("budget_sha256") != budget_sha:
            raise EvidenceError(f"candidate {candidate_id} has changed rubric or budget references")
        if any(field in candidate for field in RESET_FIELDS):
            raise EvidenceError(f"candidate {candidate_id} attempts to reset the shared budget")
        snapshots = _list(candidate.get("snapshots"), f"candidate {candidate_id}.snapshots")
        captures = _list(candidate.get("captures"), f"candidate {candidate_id}.captures")
        snapshot_ids: set[str] = set()
        for snap_index, snapshot in enumerate(snapshots):
            data = _validate_artifact(root, snapshot, f"candidate {candidate_id}.snapshots[{snap_index}]", states, False)
            if data["id"] in snapshot_ids:
                raise EvidenceError(f"candidate {candidate_id} has duplicate snapshot ids")
            snapshot_ids.add(data["id"])
        capture_ids: set[str] = set()
        capture_records: dict[str, dict[str, Any]] = {}
        capture_times: dict[str, datetime] = {}
        capture_states: set[str] = set()
        artifact_states: dict[str, str] = {}
        motion_artifacts: dict[str, dict[str, set[str]]] = {}
        beat_captures: dict[str, dict[str, list[str]]] = {}
        for cap_index, capture in enumerate(captures):
            data = _validate_artifact(root, capture, f"candidate {candidate_id}.captures[{cap_index}]", states, True)
            if data["id"] in capture_ids:
                raise EvidenceError(f"candidate {candidate_id} has duplicate capture ids")
            capture_ids.add(data["id"])
            capture_records[data["id"]] = capture
            captured = _parse_time(capture["captured_at"], f"capture {data['id']}.captured_at")
            _within_window(captured, f"capture {data['id']}", started, deadline)
            capture_times[data["id"]] = captured
            capture_states.add(capture["state_id"])
            prior_state = artifact_states.get(data["path"])
            if prior_state is not None and prior_state != capture["state_id"]:
                raise EvidenceError(f"candidate {candidate_id} reuses one artifact path across states")
            artifact_states[data["path"]] = capture["state_id"]
            if "beat_id" in capture or "frame" in capture or "capture_kind" in capture:
                if motion is None:
                    raise EvidenceError(f"candidate {candidate_id} has motion capture metadata without motion")
                beat_id = _text(capture.get("beat_id"), f"candidate {candidate_id}.captures[{cap_index}].beat_id")
                if beat_id not in beat_map:
                    raise EvidenceError(f"candidate {candidate_id} references an unknown motion beat")
                frame = capture.get("frame")
                if isinstance(frame, bool) or not isinstance(frame, int) or frame != beat_map[beat_id]["frame"]:
                    raise EvidenceError(f"candidate {candidate_id} capture frame does not match its beat")
                kind = _text(capture.get("capture_kind"), f"candidate {candidate_id}.captures[{cap_index}].capture_kind")
                if kind not in MOTION_CAPTURE_KINDS:
                    raise EvidenceError(f"candidate {candidate_id} has an invalid motion capture kind")
                if capture["state_id"] != beat_map[beat_id]["state_id"]:
                    raise EvidenceError(f"candidate {candidate_id} capture state does not match its beat")
                by_kind = motion_artifacts.setdefault(beat_id, {})
                for other_kind, other_paths in by_kind.items():
                    if other_kind != kind and data["path"] in other_paths:
                        raise EvidenceError(f"candidate {candidate_id} reuses one artifact path for preview and decoded_export")
                by_kind.setdefault(kind, set()).add(data["path"])
                beat_captures.setdefault(beat_id, {}).setdefault(kind, []).append(data["id"])
        if capture_states != set(states):
            raise EvidenceError(f"candidate {candidate_id} does not capture every expected state")
        if motion is not None:
            if set(beat_captures) != set(beat_map):
                raise EvidenceError(f"candidate {candidate_id} is missing a motion beat capture")
            for beat_id, by_kind in beat_captures.items():
                if motion["parity_claim"] and not MOTION_CAPTURE_KINDS.issubset(by_kind):
                    raise EvidenceError(f"candidate {candidate_id} is missing a motion capture counterpart for beat {beat_id}")
        media_probe_results[candidate_id] = _validate_media_probe(root, motion, candidate, capture_records)

        findings = candidate.get("findings")
        if not isinstance(findings, list):
            raise EvidenceError(f"candidate {candidate_id}.findings must be an array")
        finding_ids: set[str] = set()
        unresolved_blocking = False
        for finding_index, raw_finding in enumerate(findings):
            finding = _map(raw_finding, f"candidate {candidate_id}.findings[{finding_index}]")
            finding_id = _text(finding.get("id"), f"candidate {candidate_id}.findings[{finding_index}].id")
            if finding_id in finding_ids:
                raise EvidenceError(f"candidate {candidate_id} has duplicate finding ids")
            finding_ids.add(finding_id)
            if finding.get("capture_ref") not in capture_ids:
                raise EvidenceError(f"finding {finding_id} is not tied to a capture")
            _text(finding.get("location"), f"finding {finding_id}.location")
            observed = _parse_time(finding.get("observed_at"), f"finding {finding_id}.observed_at")
            _within_window(observed, f"finding {finding_id}", started, deadline)
            if observed < capture_times[finding["capture_ref"]]:
                raise EvidenceError(f"finding {finding_id} predates its capture")
            _text(finding.get("consequence"), f"finding {finding_id}.consequence")
            severity = _text(finding.get("severity"), f"finding {finding_id}.severity")
            status = _text(finding.get("status"), f"finding {finding_id}.status")
            if severity not in SEVERITIES:
                raise EvidenceError(f"finding {finding_id} has an unknown severity")
            if status not in FINDING_STATUSES:
                raise EvidenceError(f"finding {finding_id} has an unknown status; use open, resolved or rejected")
            if severity == "blocking" and status == "open":
                unresolved_blocking = True

        checks = _list(candidate.get("checks"), f"candidate {candidate_id}.checks")
        failed_check = False
        check_ids: set[str] = set()
        for check_index, raw_check in enumerate(checks):
            check = _map(raw_check, f"candidate {candidate_id}.checks[{check_index}]")
            check_id = _text(check.get("id"), "check.id")
            if check_id in check_ids:
                raise EvidenceError("candidate has duplicate check ids")
            check_ids.add(check_id)
            kind = _text(check.get("kind"), f"candidate {candidate_id}.checks[{check_index}].kind")
            if kind not in CHECK_KINDS:
                raise EvidenceError(f"candidate {candidate_id} has an unknown check kind")
            if kind == VISUAL_REPORT:
                if check.get("attestation") != "reported_visual_inspection":
                    raise EvidenceError("visual_report must remain an explicit reported attestation")
                if check.get("capture_ref") not in capture_ids:
                    raise EvidenceError("visual_report must reference a capture")
                _text(check.get("reported_by"), "visual_report.reported_by")
                reported = _parse_time(check.get("reported_at"), "visual_report.reported_at")
                _within_window(reported, "visual_report", started, deadline)
                if reported < capture_times[check["capture_ref"]]:
                    raise EvidenceError("visual_report predates its capture")
                _text(check.get("statement"), "visual_report.statement")
                if "result" in check or "evidence" in check:
                    raise EvidenceError("visual_report cannot masquerade as a file/automated pass")
                visual_reports += 1
            else:
                result = _text(check.get("result"), f"candidate {candidate_id}.checks[{check_index}].result")
                if result not in {"passed", "failed"}:
                    raise EvidenceError("file/automated checks need passed or failed result")
                _text(check.get("evidence"), f"candidate {candidate_id}.checks[{check_index}].evidence")
                failed_check = failed_check or result == "failed"

        if candidate.get("selected") is True:
            selected.append(candidate_id)
            if unresolved_blocking or candidate.get("regressed") is True or failed_check:
                raise EvidenceError(f"selected candidate {candidate_id} has unresolved blocking findings, regression, or failed checks")

    for candidate_ref in candidate_repair_refs:
        if candidate_ref not in candidate_ids:
            raise EvidenceError(f"repair names an unknown candidate: {candidate_ref}")
    if len(selected) != 1:
        raise EvidenceError("exactly one candidate must be selected")
    operation_binding = _validate_operation_binding(contract, root, selected[0])
    selected_probe = media_probe_results[selected[0]]
    if operation_binding is not None and selected_probe is not None:
        if selected_probe["export_path"] != operation_binding["artifact_path"]:
            raise EvidenceError("media probe export is not the operation artifact")
        if selected_probe["export_revision"] != operation_binding["artifact_revision"]:
            raise EvidenceError("media probe export revision differs from the operation receipt")
    lineage = _validate_lineage(packet, root, started, deadline, selected[0])
    return {
        **lineage,
        "valid": True,
        "packet_id": packet["packet_id"],
        "task_id": task_id,
        "candidate_count": len(candidates),
        "selected_candidate": selected[0],
        "repair_count": len(repairs),
        "repair_limit": limit,
        "visual_reports": visual_reports,
        "visual_inspection": "reported_only; pixels_seen_not_certified",
        "source_manifest_sha256": manifest_sha,
        "rubric_sha256": rubric_sha,
        "artifact_scope": "unrecorded" if inventory is None else "checked_against_frozen_allowlist",
        "artifact_inventory": inventory,
        "artifact_inventory_sha256": None if inventory is None else _canonical_sha(inventory),
        "provenance": {
            "operation_receipt": "bound" if operation_binding is not None else "unbound",
            "media_probe": "checked" if selected_probe is not None else "unbound",
            "execution_identity": "frozen_record_only; provider_identity_not_authenticated" if execution is not None else "unbound",
            "rights_references": references_status,
        },
    }


def validate_packet(packet: Any, root: Path, *, packet_path: Path | None = None) -> dict[str, Any]:
    """Validate a loaded packet against files below ``root``."""

    return _validate_packet(packet, Path(root), packet_path)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--packet", required=True, type=Path, help="JSON evidence packet")
    parser.add_argument("--root", required=True, type=Path, help="real artifact root for packet-relative paths")
    args = parser.parse_args()
    if not args.packet.is_file() or args.packet.is_symlink():
        raise EvidenceError("--packet must be an existing regular file")
    try:
        packet = json.loads(args.packet.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise EvidenceError(f"cannot read packet: {exc}") from exc
    result = validate_packet(packet, args.root, packet_path=args.packet)
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (EvidenceError, OSError, json.JSONDecodeError) as exc:
        sys.exit(f"Design evidence refused: {exc}")
