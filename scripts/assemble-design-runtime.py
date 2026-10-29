#!/usr/bin/env python3
"""Copy the pinned, already-installed pure-JS design tree into a release source.

The supplied tree must be a disposable hook-free production npm assembly.
This command never invokes npm, executes a package, or changes user settings.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import tempfile


REPO = Path(__file__).resolve().parents[1]
SELECTION = REPO / "packaging/admission/m4-design-chromium-macos.runtime.json"
FROZEN_LOCK = REPO / "packaging/admission/m4-design-package-lock.json"


class AssemblyError(ValueError):
    pass


def require_real_path(path: Path) -> None:
    if not path.is_absolute() or ".." in path.parts:
        raise AssemblyError(f"expected an absolute path without traversal: {path}")
    for member in (*path.parents, path):
        if member.is_symlink():
            raise AssemblyError(f"path has a symlink member: {member}")


def digest(path: Path) -> tuple[int, str]:
    sha = hashlib.sha256()
    size = 0
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    with os.fdopen(os.open(path, flags), "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise AssemblyError(f"not a regular file: {path}")
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            size += len(chunk)
            sha.update(chunk)
    return size, sha.hexdigest()


def copy_exact(source: Path, destination: Path, expected: tuple[int, str]) -> None:
    require_real_path(source)
    require_real_path(destination.parent)
    if not source.is_file() or digest(source) != expected:
        raise AssemblyError(f"source differs from admission: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    with os.fdopen(os.open(source, flags), "rb") as reader, destination.open("xb") as writer:
        shutil.copyfileobj(reader, writer, length=1024 * 1024)
        writer.flush()
        os.fsync(writer.fileno())
    if digest(destination) != expected:
        raise AssemblyError(f"copy changed during assembly: {source}")
    destination.chmod(0o600)


def design_lock_preserved(admitted: Path, current: Path) -> bool:
    old = json.loads(admitted.read_text(encoding="utf-8"))
    now = json.loads(current.read_text(encoding="utf-8"))
    if now.get("name") == "@codemirket/pi-harness":
        old["name"] = "@codemirket/pi-harness"
        old["packages"][""]["name"] = "@codemirket/pi-harness"
    if old.keys() != now.keys() or any(old[key] != now[key] for key in old if key != "packages"):
        return False
    old_packages, now_packages = old.get("packages"), now.get("packages")
    if not isinstance(old_packages, dict) or not isinstance(now_packages, dict):
        return False
    if any(now_packages.get(path) != package for path, package in old_packages.items() if path):
        return False
    old_root, now_root = old_packages.get(""), now_packages.get("")
    if not isinstance(old_root, dict) or not isinstance(now_root, dict):
        return False
    if old_root == now_root:
        return True
    expected_root = json.loads(json.dumps(old_root))
    dependencies = expected_root.get("dependencies")
    if not isinstance(dependencies, dict):
        return False
    dependencies["@hyperframes/producer"] = "0.8.197"
    return expected_root == now_root


def assemble(tree: Path, font: Path, release: Path) -> dict:
    for path in (tree, font, release):
        require_real_path(path)
    data = json.loads(SELECTION.read_text(encoding="utf-8"))
    if data.get("schema") != 1 or data.get("component") != "design" or data.get("target") != "aarch64-apple-darwin":
        raise AssemblyError("unexpected design selection")
    lock = REPO / "pi/package-lock.json"
    expected_lock = data["source_lock_sha256"]
    if (digest(FROZEN_LOCK)[1] != expected_lock
            or digest(tree.parent / "package-lock.json") != digest(lock)
            or not design_lock_preserved(FROZEN_LOCK, lock)):
        raise AssemblyError("production npm tree changed an admitted design package or lock")
    if not tree.is_dir() or tree.is_symlink() or tree.name != "node_modules":
        raise AssemblyError("select a real disposable node_modules tree")
    if not release.is_absolute() or not release.is_dir() or release.is_symlink() or (release / "manifest.json").exists():
        raise AssemblyError("select an unmanifested absolute release source directory")
    for ancestor in (*release.parents, release):
        if ancestor.is_symlink():
            raise AssemblyError("release path has a symlink ancestor")
    adapter_names = ("design-main.js", "design-operations.js", "code-inspect-main.js", "code-source-inspect.js", "code-navigation.js", "code-navigation-main.js")
    destinations = [release / "adapters/node_modules", *(release / "adapters" / name for name in adapter_names), release / data["font"]["path"]]
    destinations += [release / item["destination"] for item in data["notices"]]
    if any(path.exists() or path.is_symlink() for path in destinations):
        raise AssemblyError("design destination already exists")
    font_entry = data["font"]
    if font.is_symlink() or not font.is_file() or digest(font) != (font_entry["bytes"], font_entry["sha256"]):
        raise AssemblyError("private font differs from admission")
    for name in adapter_names:
        source = REPO / "pi/dist/runtime" / name
        if source.is_symlink() or not source.is_file() or source.stat().st_size > 1024 * 1024:
            raise AssemblyError("compiled design adapter is missing or oversize")
    for notice in data["notices"]:
        source = REPO / notice["source"]
        if source.is_symlink() or not source.is_file() or digest(source) != (notice["bytes"], notice["sha256"]):
            raise AssemblyError("design notice differs from admission")
    files = data["files"]
    if data["file_count"] != len(files) or len(files) > 7000:
        raise AssemblyError("invalid selected package file count")
    total = 0
    seen = set()
    adapter_dir = release / "adapters"
    adapter_dir.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".design-staging-", dir=adapter_dir) as temporary:
        stage = Path(temporary)
        for entry in files:
            relative = entry["path"]
            if not isinstance(relative, str) or relative.startswith("/") or "\\" in relative or ":" in relative or any(part in ("", ".", "..") for part in relative.split("/")) or relative in seen:
                raise AssemblyError("unsafe or duplicate package path")
            seen.add(relative)
            expected = (entry["bytes"], entry["sha256"])
            copy_exact(tree / relative, stage / relative, expected)
            total += entry["bytes"]
        if total != data["total_bytes"]:
            raise AssemblyError("selected package total differs")
        # File members are installed first, then the complete tree appears atomically.
        stage.rename(release / "adapters/node_modules")
    for name in adapter_names:
        source = REPO / "pi/dist/runtime" / name
        if source.is_symlink() or not source.is_file() or source.stat().st_size > 1024 * 1024:
            raise AssemblyError("compiled design adapter is missing or oversize")
        copy_exact(source, release / "adapters" / name, digest(source))
    copy_exact(font, release / font_entry["path"], (font_entry["bytes"], font_entry["sha256"]))
    for notice in data["notices"]:
        source = REPO / notice["source"]
        copy_exact(source, release / notice["destination"], (notice["bytes"], notice["sha256"]))
    return {"component": "design", "files": len(files) + 1 + len(adapter_names) + len(data["notices"]), "package_bytes": total, "target": data["target"]}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tree", required=True, type=Path)
    parser.add_argument("--font", required=True, type=Path)
    parser.add_argument("--release", required=True, type=Path)
    args = parser.parse_args()
    print(json.dumps(assemble(args.tree, args.font, args.release)))


if __name__ == "__main__":
    main()
