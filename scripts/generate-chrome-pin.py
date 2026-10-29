#!/usr/bin/env python3
"""Generate a checked-in Chrome for Testing onboarding pin from an acquired ZIP.

The archive is never downloaded by this script. It verifies the local archive,
omits the five framework symlink members as explicit release links, and emits
the exact file/hash inventory used by Pi's explicit runtime setup.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import stat
import zipfile
from pathlib import Path, PurePosixPath


PREFIX = "chrome-mac-arm64/"
DEST_PREFIX = "runtime/browser/chrome-mac-arm64/"
EXECUTABLE = DEST_PREFIX + "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
VERSION_RE = re.compile(r"\d+\.\d+\.\d+\.\d+\Z")


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def safe_member(name: str) -> str:
    if not name.startswith(PREFIX) or "\\" in name or "\x00" in name:
        raise ValueError("unsafe archive member path")
    relative = name[len(PREFIX) :]
    if relative.endswith("/"):
        relative = relative[:-1]
    path = PurePosixPath(relative)
    if (
        not relative
        or path.is_absolute()
        or ".." in path.parts
        or "." in path.parts
        or "" in relative.split("/")
        or str(path) != relative
    ):
        raise ValueError("unsafe archive member path")
    return relative


def safe_link_target(target: str) -> str:
    if not target or "\\" in target or "\x00" in target:
        raise ValueError("unsafe symlink target")
    path = PurePosixPath(target)
    if (
        path.is_absolute()
        or ".." in path.parts
        or "." in path.parts
        or "" in target.split("/")
        or str(path) != target
    ):
        raise ValueError("unsafe symlink target")
    return target


def generate(archive: Path, version: str, url: str) -> dict:
    if not VERSION_RE.fullmatch(version):
        raise ValueError("invalid Chrome for Testing version")
    expected_url = (
        f"https://storage.googleapis.com/chrome-for-testing-public/"
        f"{version}/mac-arm64/chrome-mac-arm64.zip"
    )
    if url != expected_url:
        raise ValueError("Chrome archive URL is not the official Mac arm64 URL for version")
    archive_bytes = archive.read_bytes()
    with zipfile.ZipFile(archive) as source:
        regular: list[dict] = []
        links: list[dict] = []
        seen: set[str] = set()
        for info in source.infolist():
            relative = safe_member(info.filename)
            if relative in seen:
                raise ValueError("duplicate archive member")
            seen.add(relative)
            mode = (info.external_attr >> 16) & 0o777777
            if info.is_dir():
                continue
            if stat.S_ISDIR(mode):
                raise ValueError("directory member must use a trailing slash")
            data = source.read(info)
            if stat.S_ISLNK(mode):
                target = safe_link_target(data.decode("utf-8"))
                links.append(
                    {
                        "relative_path": DEST_PREFIX + relative,
                        "target": target,
                    }
                )
                continue
            if not stat.S_ISREG(mode):
                raise ValueError("unsupported nonregular archive member")
            destination = DEST_PREFIX + relative
            if destination == EXECUTABLE and not mode & 0o111:
                raise ValueError("Chrome executable is not executable")
            regular.append(
                {
                    "path": destination,
                    "bytes": len(data),
                    "sha256": digest(data),
                    "mode": 0o700 if mode & 0o111 else 0o600,
                }
            )
        if not regular or EXECUTABLE not in {item["path"] for item in regular}:
            raise ValueError("Chrome executable missing from archive")
    return {
        "schema": 1,
        "component": "browser",
        "vendor": "Google Chrome for Testing",
        "version": version,
        "target": "aarch64-apple-darwin",
        "archive": {
            "name": archive.name,
            "url": url,
            "bytes": len(archive_bytes),
            "sha256": digest(archive_bytes),
        },
        "executable": EXECUTABLE,
        "file_count": len(regular),
        "link_count": len(links),
        "regular_bytes": sum(item["bytes"] for item in regular),
        "files": regular,
        "links": links,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--url", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if not args.archive.is_file() or args.archive.is_symlink():
        raise SystemExit("archive must be a regular file")
    result = generate(args.archive, args.version, args.url)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
