#!/usr/bin/env python3
"""Assemble the current Sharp-free Video recipient source/notice candidate.

It reads already-acquired npm archives, notices and the neutral FFmpeg
recipient fixture, including the selected FFmpeg binaries; it never downloads,
installs, executes or admits a runtime. The output is a new directory and is
refused if it already exists.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile
from pathlib import PurePosixPath


REPO = Path(__file__).resolve().parents[1]
DEFAULT_INVENTORY = REPO / "packaging/admission/m3-video-sharp-free-macos.runtime.json.proposed"
DEFAULT_LOCK = REPO / "packaging/admission/pi-native-base-package-lock.json"
TARGET = "aarch64-apple-darwin"
VIDEO_PREFIX = "runtime/video/"
FIRST_PARTY_VIDEO_FILES = frozenset(
    {
        "runtime/video/dist/video-export.js",
        "runtime/video/dist/video-main.js",
        "runtime/video/dist/video-policy.js",
    }
)
SUPPLEMENTAL_NOTICE_DIRS = frozenset(
    {"embedded-puppeteer", "embedded-fonts", "embedded-wawoff2", "supplemental"}
)
SHARP_PACKAGES = frozenset(
    {
        "sharp",
        "@img/colour",
        "@img/sharp-darwin-arm64",
        "@img/sharp-libvips-darwin-arm64",
        "detect-libc",
        "semver",
    }
)
HEX = set("0123456789abcdef")


class AssemblyError(ValueError):
    """The selected source or destination does not match its receipt."""


def digest(path: Path) -> tuple[int, str]:
    if path.is_symlink() or not path.is_file() or path.resolve(strict=True) != path:
        raise AssemblyError(f"regular file required: {path}")
    size = 0
    sha = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            size += len(chunk)
            sha.update(chunk)
    return size, sha.hexdigest()


def require_directory(path: Path) -> None:
    if (
        not path.is_absolute()
        or ".." in path.parts
        or path.is_symlink()
        or not path.is_dir()
        or path.resolve(strict=True) != path
    ):
        raise AssemblyError(f"directory is missing, linked or non-canonical: {path}")


def safe_relative_path(value: object, label: str) -> str:
    if not isinstance(value, str) or not value or "\\" in value or "\x00" in value:
        raise AssemblyError(f"invalid {label}")
    path = PurePosixPath(value)
    if path.is_absolute() or path.as_posix() != value or any(part in {"", ".", ".."} for part in path.parts):
        raise AssemblyError(f"unsafe {label}: {value}")
    return path.as_posix()


def safe_leaf(value: object, label: str) -> str:
    normalized = safe_relative_path(value, label)
    if "/" in normalized or normalized in {".", ".."}:
        raise AssemblyError(f"unsafe {label}: {value}")
    allowed = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@._-")
    if any(character not in allowed for character in normalized):
        raise AssemblyError(f"unsafe {label}: {value}")
    return normalized


def safe_prefixed_leaf(value: object, prefix: str, label: str) -> str:
    normalized = safe_relative_path(value, label)
    if not normalized.startswith(prefix) or normalized.count("/") != prefix.count("/"):
        raise AssemblyError(f"unsafe {label}: {value}")
    return safe_leaf(normalized.removeprefix(prefix), label)


def safe_prefixed_basename(value: object, prefix: str, label: str) -> str:
    normalized = safe_relative_path(value, label)
    if not normalized.startswith(prefix):
        raise AssemblyError(f"unsafe {label}: {value}")
    tail = normalized.removeprefix(prefix)
    parts = tail.split("/")
    if not parts or any(not part for part in parts):
        raise AssemblyError(f"unsafe {label}: {value}")
    for part in parts:
        safe_leaf(part, label)
    return parts[-1]


def require_hex(value: object) -> str:
    if not isinstance(value, str) or len(value) != 64 or any(c not in HEX for c in value):
        raise AssemblyError("invalid SHA-256")
    return value


def package_name(lock_path: str) -> str:
    tail = lock_path.rsplit("node_modules/", 1)[-1]
    parts = tail.split("/")
    if parts[0].startswith("@"):
        if len(parts) < 2:
            raise AssemblyError(f"invalid scoped package path: {lock_path}")
        return "/".join(parts[:2])
    return parts[0]


def copy_file(source: Path, destination: Path) -> None:
    if destination.exists() or destination.is_symlink():
        raise AssemblyError(f"destination already exists: {destination}")
    if source.is_symlink() or not source.is_file() or source.resolve(strict=True) != source:
        raise AssemblyError(f"source is not a regular file: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(0o644)


def copy_tree(source: Path, destination: Path) -> list[str]:
    if source.is_symlink() or not source.is_dir():
        raise AssemblyError(f"source directory is missing or linked: {source}")
    copied: list[str] = []
    for item in sorted(source.rglob("*")):
        relative = item.relative_to(source)
        target = destination / relative
        if item.is_symlink():
            raise AssemblyError(f"source tree contains a link: {item}")
        if item.is_dir():
            continue
        if not item.is_file() or item.resolve(strict=True) != item:
            raise AssemblyError(f"source tree contains a non-regular member: {item}")
        copy_file(item, target)
        copied.append(relative.as_posix())
    return copied


def load_json(path: Path) -> dict:
    if path.is_symlink() or not path.is_file():
        raise AssemblyError(f"JSON receipt is missing or linked: {path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise AssemblyError(f"invalid JSON receipt: {path}") from error
    if not isinstance(value, dict):
        raise AssemblyError(f"JSON receipt must be an object: {path}")
    return value


def package_instances(inventory: dict, lock: dict, strict_current: bool) -> tuple[dict, list[dict]]:
    if (
        inventory.get("schema") != 1
        or inventory.get("target") != TARGET
        or inventory.get("release_admitted") is not False
        or inventory.get("status") != "sharp_absent_four_format_development_candidate_not_release_admitted"
    ):
        raise AssemblyError("Video inventory is not the current unadmitted Sharp-free candidate")
    runtime = inventory.get("runtime_inventory")
    files = runtime.get("files") if isinstance(runtime, dict) else None
    if not isinstance(files, list) or runtime.get("regular_files") != len(files):
        raise AssemblyError("Video inventory file count is invalid")
    expected_paths: set[str] = set()
    for entry in files:
        if not isinstance(entry, dict) or not isinstance(entry.get("path"), str):
            raise AssemblyError("Video inventory entry is invalid")
        path = entry["path"]
        require_hex(entry.get("sha256"))
        if type(entry.get("bytes")) is not int or entry["bytes"] < 0 or path in expected_paths:
            raise AssemblyError("Video inventory entry is invalid")
        expected_paths.add(path)
    if runtime.get("regular_bytes") != sum(entry["bytes"] for entry in files):
        raise AssemblyError("Video inventory byte total is invalid")
    media = inventory.get("media_files")
    if not isinstance(media, list) or {x.get("path") for x in media} != {
        "runtime/video/bin/ffmpeg",
        "runtime/video/bin/ffprobe",
    }:
        raise AssemblyError("Video media inventory is incomplete")
    for entry in media:
        require_hex(entry.get("sha256"))
        if type(entry.get("bytes")) is not int or entry["bytes"] <= 0:
            raise AssemblyError("Video media inventory entry is invalid")

    lock_packages = lock.get("packages")
    if not isinstance(lock_packages, dict):
        raise AssemblyError("package lock has no package map")
    selected: list[dict] = []
    for path, package in lock_packages.items():
        if not path.startswith("node_modules/") or not isinstance(package, dict):
            continue
        package_json = f"{VIDEO_PREFIX}{path}/package.json"
        if package_json not in expected_paths:
            continue
        name = package_name(path)
        version = package.get("version")
        if not isinstance(version, str) or not version:
            raise AssemblyError(f"package lock version missing: {path}")
        if name in SHARP_PACKAGES:
            raise AssemblyError(f"Sharp package selected in the Video inventory: {name}")
        selected.append({"path": path, "name": name, "version": version})
    if len(selected) != inventory.get("package_instances"):
        raise AssemblyError("Video package instance count does not match package lock")
    unique = {(item["name"], item["version"]): item for item in selected}
    if strict_current and (
        inventory.get("package_instances") != 82
        or runtime.get("regular_files") != 5892
        or runtime.get("regular_bytes") != 65459967
        or len(unique) != 80
    ):
        raise AssemblyError("not the current 82-instance Sharp-free inventory")
    return unique, media


def verify_release_source_files(
    manifest_path: Path,
    source_root: Path,
    inventory: dict,
    media: list[dict],
    strict_current: bool,
) -> None:
    if not strict_current:
        return
    require_directory(source_root)
    manifest = load_json(manifest_path)
    entries = manifest.get("files")
    if not isinstance(entries, list):
        raise AssemblyError("current release manifest has no file list")
    manifest_files: dict[str, dict] = {}
    for entry in entries:
        if not isinstance(entry, dict):
            raise AssemblyError("current release manifest entry is invalid")
        path = safe_relative_path(entry.get("relative_path"), "release manifest path")
        if path in manifest_files:
            raise AssemblyError(f"duplicate current release manifest path: {path}")
        if type(entry.get("byte_len")) is not int or entry["byte_len"] < 0:
            raise AssemblyError(f"invalid current release manifest size: {path}")
        require_hex(entry.get("sha256"))
        manifest_files[path] = entry
    selected_entries = inventory["runtime_inventory"]["files"] + media
    selected = [entry["path"] for entry in selected_entries]
    inventory_by_path = {entry["path"]: entry for entry in selected_entries}
    for path in selected:
        path = safe_relative_path(path, "selected release path")
        manifest_entry = manifest_files.get(path)
        if manifest_entry is None:
            raise AssemblyError(f"current release manifest lacks selected Video file: {path}")
        source = source_root / PurePosixPath(path)
        if source.is_symlink() or not source.is_file() or source.resolve(strict=True) != source:
            raise AssemblyError(f"current release source file is missing, linked or non-canonical: {path}")
        actual = digest(source)
        expected = (manifest_entry["byte_len"], manifest_entry["sha256"])
        if actual != expected:
            raise AssemblyError(f"current release source file does not match manifest: {path}")
        inventory_entry = inventory_by_path.get(path)
        # The three first-party adapters are intentionally rebound to the
        # current release manifest.  Their old vendor inventory hashes are
        # historical pins, not a reason to reject the current authored files.
        if path in FIRST_PARTY_VIDEO_FILES:
            continue
        if inventory_entry is None or actual != (inventory_entry["bytes"], inventory_entry["sha256"]):
            raise AssemblyError(f"current release source file does not match selected inventory: {path}")


def verify_ffmpeg_fixture(fixture: Path, fixture_manifest: dict, media: list[dict]) -> None:
    entries = fixture_manifest.get("files")
    if not isinstance(entries, list) or not entries:
        raise AssemblyError("FFmpeg fixture manifest is incomplete")
    expected: dict[str, dict] = {}
    for entry in entries:
        if not isinstance(entry, dict):
            raise AssemblyError("FFmpeg fixture manifest entry is invalid")
        relative = safe_relative_path(entry.get("path"), "FFmpeg fixture path")
        if relative == "delivery-manifest.json" or relative in expected:
            raise AssemblyError("FFmpeg fixture manifest has duplicate or reserved path")
        if type(entry.get("bytes")) is not int or entry["bytes"] < 0:
            raise AssemblyError(f"invalid FFmpeg fixture size: {relative}")
        require_hex(entry.get("sha256"))
        expected[relative] = entry
    actual: dict[str, Path] = {}
    for item in fixture.rglob("*"):
        relative = item.relative_to(fixture).as_posix()
        if item.is_symlink():
            raise AssemblyError(f"FFmpeg fixture contains a link: {relative}")
        if item.is_dir():
            continue
        if not item.is_file() or item.resolve(strict=True) != item:
            raise AssemblyError(f"FFmpeg fixture contains a non-regular member: {relative}")
        if relative != "delivery-manifest.json":
            actual[safe_relative_path(relative, "FFmpeg fixture path")] = item
    if set(actual) != set(expected):
        raise AssemblyError("FFmpeg fixture file set does not match its manifest")
    for relative, source in actual.items():
        if digest(source) != (expected[relative]["bytes"], expected[relative]["sha256"]):
            if relative in {"bin/ffmpeg", "bin/ffprobe"}:
                raise AssemblyError("FFmpeg fixture binary/source mapping is stale")
            raise AssemblyError(f"FFmpeg fixture file does not match its manifest: {relative}")
    media_by_name = {Path(entry["path"]).name: entry for entry in media}
    for name, media_entry in media_by_name.items():
        relative = f"bin/{name}"
        if relative not in expected or (expected[relative]["bytes"], expected[relative]["sha256"]) != (media_entry["bytes"], media_entry["sha256"]):
            raise AssemblyError("FFmpeg fixture binary/source mapping is stale")


def source_records(source_index: dict, notice_index: dict) -> tuple[dict, dict]:
    records = {}
    for entry in source_index.get("published_npm_archives", []):
        key = (entry.get("name"), entry.get("version"))
        if not all(isinstance(value, str) for value in key) or key in records:
            raise AssemblyError("duplicate or invalid npm source record")
        records[key] = entry
    notices = {}
    for entry in notice_index.get("packages", []):
        key = (entry.get("name"), entry.get("version"))
        if not all(isinstance(value, str) for value in key):
            raise AssemblyError("duplicate or invalid notice record")
        if key in notices:
            prior = notices[key]
            if (
                prior.get("notice_directory") != entry.get("notice_directory")
                or prior.get("upstream_package_notice_members") != entry.get("upstream_package_notice_members")
            ):
                raise AssemblyError("conflicting duplicate notice record")
            continue
        notices[key] = entry
    return records, notices


def first_party_rebind(manifest_path: Path | None, strict_current: bool) -> tuple[dict | None, str | None]:
    if manifest_path is None:
        if strict_current:
            raise AssemblyError("current release manifest is required for first-party Video rebind")
        return None, None
    manifest = load_json(manifest_path)
    entries = {
        entry.get("relative_path"): entry
        for entry in manifest.get("files", [])
        if isinstance(entry, dict)
    }
    selected = []
    for path in sorted(FIRST_PARTY_VIDEO_FILES):
        entry = entries.get(path)
        if not isinstance(entry, dict) or type(entry.get("byte_len")) is not int:
            raise AssemblyError(f"current release manifest lacks first-party Video file: {path}")
        selected.append(
            {
                "path": path,
                "bytes": entry["byte_len"],
                "sha256": require_hex(entry.get("sha256")),
                "mode": entry.get("mode"),
            }
        )
    return {"manifest_path_name": manifest_path.name, "files": selected}, digest(manifest_path)[1]


def assemble(
    *,
    inventory_path: Path,
    package_lock_path: Path,
    source_index_path: Path,
    notice_index_path: Path,
    npm_archive_root: Path,
    package_notice_root: Path,
    ffmpeg_fixture: Path,
    output: Path,
    source_archive_root: Path | None = None,
    release_source_manifest: Path | None = None,
    release_source_root: Path | None = None,
    supplemental_notice_root: Path | None = None,
    strict_current: bool = True,
) -> dict:
    if output.exists() or output.is_symlink():
        raise AssemblyError("output already exists; refusing overwrite")
    require_directory(output.parent)
    require_directory(npm_archive_root)
    require_directory(package_notice_root)
    require_directory(ffmpeg_fixture)
    if source_archive_root is not None:
        require_directory(source_archive_root)
    if supplemental_notice_root is not None:
        require_directory(supplemental_notice_root)
    if release_source_manifest is not None and release_source_root is None:
        release_source_root = release_source_manifest.parent
    if strict_current:
        verify_pi_layout(package_lock_path, REPO / "pi/package-lock.json", REPO / "packaging/admission/pi-native-layout.json")
    inventory = load_json(inventory_path)
    lock = load_json(package_lock_path)
    lock_sha = digest(package_lock_path)[1]
    if inventory.get("package_lock_sha256") != lock_sha:
        raise AssemblyError("package lock does not match Video inventory")
    selected, media = package_instances(inventory, lock, strict_current)
    first_party, release_manifest_sha = first_party_rebind(release_source_manifest, strict_current)
    if release_source_manifest is not None:
        verify_release_source_files(release_source_manifest, release_source_root, inventory, media, strict_current)
    old_sources = load_json(source_index_path)
    old_notices = load_json(notice_index_path)
    archive_records, notice_records = source_records(old_sources, old_notices)
    for key in selected:
        if key not in archive_records or key not in notice_records:
            raise AssemblyError(f"source or notice record missing for {key[0]}@{key[1]}")

    fixture_manifest_path = ffmpeg_fixture / "delivery-manifest.json"
    fixture_manifest = load_json(fixture_manifest_path)
    if fixture_manifest.get("release_admitted") is not False:
        raise AssemblyError("FFmpeg fixture is unexpectedly admitted")
    verify_ffmpeg_fixture(ffmpeg_fixture, fixture_manifest, media)

    with tempfile.TemporaryDirectory(prefix=".video-recipient-", dir=output.parent) as temporary:
        stage = Path(temporary)
        npm_out = stage / "source/video/npm"
        notice_out = stage / "notices/packages"
        selected_archives = []
        selected_notices = []
        for name, version in sorted(selected):
            archive = archive_records[(name, version)]
            archive_name = safe_prefixed_leaf(archive.get("release_relative_path"), "source/video/npm/", "npm source archive path")
            if archive_name.endswith(".sha256") or name in SHARP_PACKAGES:
                raise AssemblyError("invalid or Sharp npm source archive")
            source = npm_archive_root / archive_name
            if digest(source) != (archive.get("bytes"), archive.get("sha256")):
                raise AssemblyError(f"npm archive identity mismatch: {name}@{version}")
            destination = npm_out / archive_name
            copy_file(source, destination)
            selected_archives.append(
                {
                    "name": name,
                    "version": version,
                    "path": f"source/video/npm/{archive_name}",
                    "bytes": archive["bytes"],
                    "sha256": archive["sha256"],
                    "integrity": archive.get("integrity"),
                }
            )

            notice = notice_records[(name, version)]
            notice_directory = safe_prefixed_leaf(notice.get("notice_directory"), "notices/packages/", "package notice directory")
            if not notice_directory or name in SHARP_PACKAGES:
                raise AssemblyError("invalid or Sharp notice directory")
            source_notice = package_notice_root / notice_directory
            copied = copy_tree(source_notice, notice_out / notice_directory)
            selected_notices.append(
                {
                    "name": name,
                    "version": version,
                    "path": f"notices/packages/{notice_directory}",
                    "files": copied,
                    "upstream_package_notice_members": notice.get("upstream_package_notice_members", []),
                }
            )

        supplemental_notices = []
        if supplemental_notice_root is not None:
            for directory in sorted(SUPPLEMENTAL_NOTICE_DIRS):
                source_notice = supplemental_notice_root / directory
                if source_notice.is_symlink() or not source_notice.is_dir():
                    raise AssemblyError(f"supplemental notice directory is missing or linked: {directory}")
                copied = copy_tree(source_notice, stage / "notices" / directory)
                supplemental_notices.append(
                    {
                        "directory": directory,
                        "path": f"notices/{directory}",
                        "files": copied,
                    }
                )

        ffmpeg_out = stage / "source/ffmpeg"
        copy_tree(ffmpeg_fixture, ffmpeg_out)
        fixture_identity = {
            "path": "source/ffmpeg/delivery-manifest.json",
            "bytes": fixture_manifest_path.stat().st_size,
            "sha256": digest(fixture_manifest_path)[1],
            "release_admitted": False,
            "files": fixture_manifest["files"],
        }

        available_sources = []
        missing_sources = []
        for entry in old_sources.get("complete_upstream_archives", []):
            relative = entry.get("release_relative_path")
            filename = safe_prefixed_basename(relative, "source/video/", "upstream source archive path") if relative else ""
            source = source_archive_root / filename if source_archive_root and filename else None
            if source and source.is_file() and not source.is_symlink():
                if digest(source) != (entry.get("bytes"), entry.get("sha256")):
                    raise AssemblyError(f"upstream source identity mismatch: {filename}")
                destination = stage / "source/video/upstream" / filename
                copy_file(source, destination)
                available_sources.append({**entry, "output_path": f"source/video/upstream/{filename}", "available": True})
            else:
                missing_sources.append({**entry, "available": False, "reason": "not present in supplied offline source root"})

        provenance = stage / "provenance"
        copy_file(inventory_path, provenance / "m3-video-sharp-free-macos.runtime.json.proposed")
        copy_file(package_lock_path, provenance / "native-base-package-lock.json")
        if strict_current:
            copy_file(REPO / "pi/package-lock.json", provenance / "pi-package-lock.json")
            copy_file(REPO / "packaging/admission/pi-native-layout.json", provenance / "pi-native-layout.json")
        if release_source_manifest is not None:
            copy_file(release_source_manifest, provenance / "current-release-manifest.json")
        source_manifest = {
            "schema": 2,
            "status": "sharp_free_video_recipient_source_notice_candidate_not_release_admitted",
            "release_admitted": False,
            "target": TARGET,
            "hyperframes_version": inventory.get("hyperframes_version"),
            "package_instances": inventory["package_instances"],
            "distinct_npm_archives": len(selected_archives),
            "npm_archives": selected_archives,
            "notice_packages": selected_notices,
            "supplemental_notices": supplemental_notices,
            "supplemental_notice_metadata": {
                key: old_notices[key]
                for key in (
                    "supplemental_components",
                    "embedded_font_faces",
                    "embedded_font_packages",
                    "embedded_fonts_license",
                    "unresolved_embedded_font_faces",
                    "scope_limit",
                )
                if key in old_notices
            },
            "excluded_sharp_packages": sorted(SHARP_PACKAGES),
            "ffmpeg_fixture": fixture_identity,
            "first_party_video_rebind": first_party,
            "current_release_manifest_sha256": release_manifest_sha,
            "additional_source_archives_available": available_sources,
            "additional_source_archives_missing": missing_sources,
            "additional_source_archive_obligation": {
                "classification": "supplementary_provenance_for_selected_payloads",
                "required_for_selected_license_notice_delivery": False,
                "basis": "The packet carries each selected published npm archive plus its package notice and the retained supplemental/embedded notices. The unavailable complete upstream archives are separate provenance for the recorded MIT, Apache-2.0 and OFL-1.1 components; this classification must be revisited if a selected payload or license changes.",
                "publication_route_not_tested": True,
            },
            "gaps": [
                "Actual recipient source/notice access and final combined delivery were not tested",
                "Neutral FFmpeg final source export was not recompiled; prior guarded build evidence is retained separately",
                "Missing upstream archives remain unavailable in this offline source root" if missing_sources else "",
                "Aggregate rights, codec patent/grant and release authentication review remain open",
            ],
        }
        source_manifest["gaps"] = [gap for gap in source_manifest["gaps"] if gap]
        notice_manifest = {
            "schema": 2,
            "status": "sharp_free_video_recipient_source_notice_candidate_not_release_admitted",
            "release_admitted": False,
            "packages": selected_notices,
            "supplemental_notices": supplemental_notices,
            "supplemental_components": old_notices.get("supplemental_components", []),
            "embedded_font_faces": old_notices.get("embedded_font_faces", 0),
            "embedded_font_packages": old_notices.get("embedded_font_packages", 0),
            "embedded_fonts_license": old_notices.get("embedded_fonts_license"),
            "unresolved_embedded_font_faces": old_notices.get("unresolved_embedded_font_faces", []),
            "excluded_sharp_packages": sorted(SHARP_PACKAGES),
            "ffmpeg_notice_manifest": fixture_identity["path"],
        }
        (stage / "SOURCE-INDEX.json").write_text(json.dumps(source_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        (stage / "NOTICE-INDEX.json").write_text(json.dumps(notice_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        (stage / "README.md").write_text(
            "# Sharp-free Video recipient source/notice candidate\n\n"
            "This packet includes selected FFmpeg binaries plus source and notices;\n"
            "it is not a release and is not a source offer.\n"
            "It is bound to the current 82-instance Video inventory, excludes the\n"
            "six Sharp packages, and includes the neutral FFmpeg fixture. No\n"
            "downloads, installers, hooks, runtime execution or publication ran.\n"
            "See SOURCE-INDEX.json for supplementary source evidence and remaining rights\n"
            "and recipient-delivery gates.\n",
            encoding="utf-8",
        )
        manifest_files = []
        for path in sorted(stage.rglob("*")):
            if path.is_file() and not path.is_symlink():
                size, sha = digest(path)
                manifest_files.append({"path": path.relative_to(stage).as_posix(), "bytes": size, "sha256": sha})
        (stage / "MANIFEST.json").write_text(
            json.dumps({"schema": 1, "status": "private_recipient_source_notice_candidate", "release_admitted": False, "files": manifest_files}, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        os.rename(stage, output)
    return {
        "status": "private_recipient_source_notice_candidate",
        "release_admitted": False,
        "output": str(output),
        "package_instances": inventory["package_instances"],
        "distinct_npm_archives": len(selected_archives),
        "source_archive_missing": len(missing_sources),
        "current_release_manifest_sha256": release_manifest_sha,
        "ffmpeg_fixture": fixture_identity,
    }


def verify_pi_layout(base: Path, current: Path, receipt: Path) -> None:
    layout = load_json(receipt)
    if digest(base)[1] != layout.get("base_lock_sha256") or digest(current)[1] != layout.get("current_lock_sha256"):
        raise AssemblyError("pi layout lock identity changed")
    before = {key: value for key, value in load_json(base)["packages"].items() if key}
    after = {key: value for key, value in load_json(current)["packages"].items() if key}
    if before != after:
        raise AssemblyError("pi layout changed dependency instances")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory", type=Path, default=DEFAULT_INVENTORY)
    parser.add_argument("--package-lock", type=Path, default=DEFAULT_LOCK)
    parser.add_argument("--source-index", type=Path, required=True)
    parser.add_argument("--notice-index", type=Path, required=True)
    parser.add_argument("--npm-archive-root", type=Path, required=True)
    parser.add_argument("--package-notice-root", type=Path, required=True)
    parser.add_argument("--ffmpeg-fixture", type=Path, required=True)
    parser.add_argument("--source-archive-root", type=Path)
    parser.add_argument("--release-source-manifest", type=Path)
    parser.add_argument("--release-source-root", type=Path)
    parser.add_argument("--supplemental-notice-root", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(
            json.dumps(
                assemble(
                    inventory_path=args.inventory,
                    package_lock_path=args.package_lock,
                    source_index_path=args.source_index,
                    notice_index_path=args.notice_index,
                    npm_archive_root=args.npm_archive_root,
                    package_notice_root=args.package_notice_root,
                    ffmpeg_fixture=args.ffmpeg_fixture,
                    source_archive_root=args.source_archive_root,
                    release_source_manifest=args.release_source_manifest,
                    release_source_root=args.release_source_root,
                    supplemental_notice_root=args.supplemental_notice_root,
                    output=args.output,
                ),
                indent=2,
                sort_keys=True,
            )
        )
    except AssemblyError as error:
        parser.error(str(error))


if __name__ == "__main__":
    main()
