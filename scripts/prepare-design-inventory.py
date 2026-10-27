#!/usr/bin/env python3
"""Freeze a clean, hook-free npm production tree for a private design release.

This maintainer tool does not run npm, execute package code, or install into a
release. The caller supplies a disposable `npm ci --omit=dev --ignore-scripts`
tree and separately checks its package rights before admission.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat


REPO = Path(__file__).resolve().parents[1]
MAX_FILES = 7000
MAX_BYTES = 128 * 1024 * 1024
MAX_FILE_BYTES = 16 * 1024 * 1024
TYPESCRIPT_SUBSET = {
    "typescript/package.json": "822ef7ca6452205657b6288b066481ecf508bfbf43455d715cf7d3ec457561e6",
    "typescript/lib/typescript.js": "3ae902c92cc44dace175c0e69e13a4b0899f6983c6121d76b9ab8dd5795e7675",
    "typescript/LICENSE.txt": "a7d00bfd54525bc694b6e32f64c7ebcf5e6b7ae3657be5cc12767bce74654a47",
    "typescript/ThirdPartyNoticeText.txt": "1af3c68039c57e539422da82a4faada506ce6d0ea6f90e0b699d02dbcdb7a90c",
}


class InventoryError(ValueError):
    pass


def digest(path: Path) -> tuple[int, str]:
    h = hashlib.sha256()
    count = 0
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    with os.fdopen(os.open(path, flags), "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise InventoryError(f"not a regular file: {path}")
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            count += len(chunk)
            if count > MAX_FILE_BYTES:
                raise InventoryError(f"oversize package file: {path}")
            h.update(chunk)
    return count, h.hexdigest()


def prepare(tree: Path, lock: Path) -> dict:
    tree = tree.absolute()
    if not tree.is_dir() or tree.is_symlink() or not tree.name == "node_modules":
        raise InventoryError("select an existing real node_modules directory")
    if tree.parent.joinpath("package-lock.json").read_bytes() != lock.read_bytes():
        raise InventoryError("staging lock differs from the admitted adapter lock")
    lock_sha = hashlib.sha256(lock.read_bytes()).hexdigest()
    entries: list[dict] = []
    total = 0
    for current, dirs, files in os.walk(tree, topdown=True, followlinks=False):
        dirs.sort()
        files.sort()
        base = Path(current)
        for directory in tuple(dirs):
            path = base / directory
            if path.is_symlink():
                if path.relative_to(tree).as_posix() != ".bin":
                    raise InventoryError(f"unexpected package symlink: {path}")
                dirs.remove(directory)
            elif path.relative_to(tree).as_posix() == ".bin":
                dirs.remove(directory)
        for filename in files:
            path = base / filename
            relative = path.relative_to(tree).as_posix()
            if relative == ".package-lock.json":
                continue
            if path.is_symlink():
                raise InventoryError(f"unexpected package symlink: {path}")
            if any(part in ("", ".", "..") for part in relative.split("/")) or "\\" in relative or ":" in relative:
                raise InventoryError(f"unsafe package path: {relative}")
            size, sha = digest(path)
            entries.append({"path": relative, "bytes": size, "sha256": sha})
            total += size
            if len(entries) > MAX_FILES or total > MAX_BYTES:
                raise InventoryError("design package exceeds inventory limits")
    if not any(item["path"] == "style-dictionary/package.json" for item in entries) or not any(
        item["path"] == "svgo/package.json" for item in entries
    ) or not any(item["path"] == "sax/package.json" for item in entries):
        raise InventoryError("required production packages are missing")
    if any("resvg" in item["path"].lower() for item in entries):
        raise InventoryError("opaque native resvg Node binding remains in production tree")
    typescript_files = {item["path"]: item["sha256"] for item in entries if item["path"].startswith("typescript/")}
    if typescript_files != TYPESCRIPT_SUBSET:
        raise InventoryError("TypeScript parser must contain only the exact admitted source/notice subset")
    entries.sort(key=lambda item: item["path"])
    return {
        "schema": 1,
        "target": "aarch64-apple-darwin",
        "source_lock_sha256": lock_sha,
        "file_count": len(entries),
        "total_bytes": total,
        "files": entries,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tree", required=True, type=Path, help="disposable production node_modules")
    parser.add_argument("--out", required=True, type=Path, help="new selection JSON")
    args = parser.parse_args()
    if not args.out.is_absolute() or args.out.exists() or args.out.is_symlink():
        raise InventoryError("output must be a new absolute path")
    inventory = prepare(args.tree, REPO / "adapters/package-lock.json")
    with args.out.open("x", encoding="utf-8") as output:
        json.dump(inventory, output, separators=(",", ":"), ensure_ascii=False)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    print(json.dumps({key: inventory[key] for key in ("target", "source_lock_sha256", "file_count", "total_bytes")}))


if __name__ == "__main__":
    main()
