#!/usr/bin/env python3
"""Reproduce the closed macOS Raster candidate from six pinned npm archives.

This is a qualification assembler. It does not admit Raster to a release or
run npm, package hooks, or any upstream code.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tarfile
import tempfile
import unittest


PROPOSAL = Path(__file__).resolve().parents[1] / "packaging/admission/m4-raster-prebuilt-macos.runtime.json.proposed"
PROPOSAL_SHA256 = "705cd057ebc0cc25c6cffe92d05d442663dad544dcd62b0e67cc30adae74e41b"
TARGET = "aarch64-apple-darwin"
ROOT = "runtime/design/raster/node_modules"
MODULE = ROOT + "/sharp/dist/index.mjs"
PACKAGES = (
    ("sharp", "0.35.5"),
    ("@img/colour", "1.1.0"),
    ("detect-libc", "2.1.2"),
    ("semver", "7.8.5"),
    ("@img/sharp-darwin-arm64", "0.35.5"),
    ("@img/sharp-libvips-darwin-arm64", "1.3.4"),
)
HEX = re.compile(r"[0-9a-f]{64}\Z")
MAX_ARCHIVE_BYTES = 16 * 1024 * 1024
MAX_MEMBER_BYTES = 20 * 1024 * 1024
MAX_MEMBERS = 200


class AssemblyError(ValueError):
    """A source or destination does not match the immutable candidate."""


def safe_path(value: str) -> str:
    if (not isinstance(value, str) or not value or len(value) > 4096
            or value.startswith("/") or "\\" in value or ":" in value or "\0" in value
            or any(part in ("", ".", "..") for part in value.split("/"))):
        raise AssemblyError("unsafe path")
    return value


def real_directory(path: Path) -> None:
    if not path.is_absolute() or ".." in path.parts:
        raise AssemblyError("directory path must be absolute and canonical")
    for part in (path, *path.parents):
        if part.is_symlink():
            raise AssemblyError("directory path contains a symlink")
    if not path.is_dir():
        raise AssemblyError("directory is missing")


def load_candidate() -> dict:
    raw = PROPOSAL.read_bytes()
    if hashlib.sha256(raw).hexdigest() != PROPOSAL_SHA256:
        raise AssemblyError("Raster proposal hash differs from the compiled pin")
    candidate = json.loads(raw)
    return validate_candidate(candidate)


def validate_candidate(candidate: dict) -> dict:
    if (candidate.get("schema") != 1 or candidate.get("target") != TARGET
            or candidate.get("status") != "upstream_archive_candidate_not_release_admitted"
            or candidate.get("release_admitted") is not False or candidate.get("node_module") != MODULE
            or candidate.get("archive_count") != 6 or len(candidate.get("archives", [])) != 6):
        raise AssemblyError("candidate identity is invalid or Raster was incorrectly admitted")
    archives = candidate["archives"]
    names = set()
    for archive, (package, version) in zip(archives, PACKAGES):
        if archive.get("package") != package or archive.get("version") != version:
            raise AssemblyError("archive package identity changed")
        url = archive.get("url")
        if not isinstance(url, str) or not url.startswith("https://registry.npmjs.org/"):
            raise AssemblyError("archive URL is not the pinned npm registry")
        name = safe_path(url.rsplit("/", 1)[-1])
        if "/" in name or not name.endswith(".tgz") or name in names:
            raise AssemblyError("archive name is invalid or duplicated")
        names.add(name)
        if (type(archive.get("bytes")) is not int or not 0 < archive["bytes"] <= MAX_ARCHIVE_BYTES
                or not isinstance(archive.get("sha256"), str) or not HEX.fullmatch(archive["sha256"])):
            raise AssemblyError("archive size or SHA-256 identity is invalid")
        try:
            encoded = archive["sha512_integrity"].removeprefix("sha512-")
            decoded = base64.b64decode(encoded, validate=True)
        except (KeyError, AttributeError, ValueError) as error:
            raise AssemblyError("archive SHA-512 integrity is invalid") from error
        if not archive["sha512_integrity"].startswith("sha512-") or len(decoded) != 64:
            raise AssemblyError("archive SHA-512 integrity is invalid")
        archive["name"] = name
    inventory = candidate.get("runtime_inventory", {})
    files = inventory.get("files", [])
    if (inventory.get("regular_files") != 125 or len(files) != 125
            or inventory.get("regular_bytes") != 19_698_667):
        raise AssemblyError("candidate inventory bounds changed")
    expected = {}
    total = 0
    for entry in files:
        path = safe_path(entry.get("path"))
        if (not path.startswith(ROOT + "/") or path in expected
                or type(entry.get("bytes")) is not int or not 0 < entry["bytes"] <= MAX_MEMBER_BYTES
                or not isinstance(entry.get("sha256"), str) or not HEX.fullmatch(entry["sha256"])):
            raise AssemblyError("candidate file identity is invalid")
        expected[path] = entry
        total += entry["bytes"]
    if total != inventory["regular_bytes"] or MODULE not in expected:
        raise AssemblyError("candidate file inventory is incomplete")
    candidate["expected_files"] = expected
    return candidate


def digest_archive(stream) -> tuple[int, str, str]:
    stream.seek(0)
    sha256 = hashlib.sha256()
    sha512 = hashlib.sha512()
    size = 0
    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
        size += len(chunk)
        if size > MAX_ARCHIVE_BYTES:
            raise AssemblyError("archive exceeds the acquisition bound")
        sha256.update(chunk)
        sha512.update(chunk)
    stream.seek(0)
    return size, sha256.hexdigest(), base64.b64encode(sha512.digest()).decode("ascii")


def extract_archive(source: Path, archive_receipt: dict, expected: dict, stage: Path) -> set[str]:
    if source.is_symlink() or not source.is_file():
        raise AssemblyError("archive is missing or linked")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0)
    with os.fdopen(os.open(source, flags), "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise AssemblyError("archive is not a regular file")
        identity = (archive_receipt["bytes"], archive_receipt["sha256"],
                    archive_receipt["sha512_integrity"].removeprefix("sha512-"))
        if digest_archive(stream) != identity:
            raise AssemblyError("archive SHA-256, SHA-512 or size mismatch")
        found: set[str] = set()
        count = 0
        with tarfile.open(fileobj=stream, mode="r:gz") as tar:
            for member in tar:
                count += 1
                if count > MAX_MEMBERS or not member.isfile() or member.size > MAX_MEMBER_BYTES:
                    raise AssemblyError("archive contains a special, oversized or excess member")
                member_path = safe_path(member.name)
                if not member_path.startswith("package/"):
                    raise AssemblyError("archive member is outside package/")
                relative = member_path.removeprefix("package/")
                path = ROOT + "/" + archive_receipt["package"] + "/" + relative
                if path in found or path not in expected or expected[path]["bytes"] != member.size:
                    raise AssemblyError("archive member differs from the candidate inventory")
                found.add(path)
                destination = stage / path
                destination.parent.mkdir(parents=True, exist_ok=True)
                sha256 = hashlib.sha256()
                size = 0
                reader = tar.extractfile(member)
                if reader is None:
                    raise AssemblyError("regular archive member could not be read")
                with reader, destination.open("xb") as writer:
                    for chunk in iter(lambda: reader.read(1024 * 1024), b""):
                        size += len(chunk)
                        if size > expected[path]["bytes"]:
                            raise AssemblyError("member exceeds candidate size")
                        sha256.update(chunk)
                        writer.write(chunk)
                    writer.flush()
                    os.fsync(writer.fileno())
                if (size, sha256.hexdigest()) != (expected[path]["bytes"], expected[path]["sha256"]):
                    raise AssemblyError("archive member content differs from candidate pin")
                destination.chmod(0o600)
        if digest_archive(stream) != identity:
            raise AssemblyError("archive changed while assembling")
    return found


def assemble(archive_root: Path, output_release: Path, candidate: dict | None = None) -> dict:
    candidate = load_candidate() if candidate is None else validate_candidate(candidate)
    real_directory(archive_root)
    real_directory(output_release)
    design_root = output_release / "runtime/design"
    real_directory(design_root)
    raster_root = design_root / "raster"
    if raster_root.exists() or raster_root.is_symlink():
        raise AssemblyError("Raster destination already exists")
    with tempfile.TemporaryDirectory(prefix=".raster-assembly-", dir=output_release.parent) as scratch:
        stage = Path(scratch)
        found: set[str] = set()
        for archive in candidate["archives"]:
            new = extract_archive(archive_root / archive["name"], archive,
                                  candidate["expected_files"], stage)
            if found.intersection(new):
                raise AssemblyError("archive file ownership overlaps")
            found.update(new)
        if found != set(candidate["expected_files"]):
            raise AssemblyError("assembled file set differs from candidate inventory")
        if raster_root.exists() or raster_root.is_symlink():
            raise AssemblyError("Raster destination appeared during assembly")
        os.rename(stage / "runtime/design/raster", raster_root)
    return {"target": TARGET, "files": len(found), "bytes": 19_698_667,
            "output": str(raster_root), "release_admitted": False, "runtime_executed": False}


class AssemblyTests(unittest.TestCase):
    def test_rejects_special_member_without_touching_destination(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "fixture.tgz"
            with tarfile.open(source, "w:gz") as tar:
                item = tarfile.TarInfo("package/link")
                item.type = tarfile.SYMTYPE
                item.linkname = "/etc/passwd"
                tar.addfile(item)
            with source.open("rb") as stream:
                size, sha256, sha512 = digest_archive(stream)
            receipt = {"package": "sharp", "bytes": size, "sha256": sha256,
                       "sha512_integrity": "sha512-" + sha512}
            stage = root / "stage"
            stage.mkdir()
            with self.assertRaisesRegex(AssemblyError, "special"):
                extract_archive(source, receipt, {}, stage)
            self.assertEqual(list(stage.iterdir()), [])

    def test_rejects_corrupt_archive_before_extraction(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "fixture.tgz"
            source.write_bytes(b"incorrect archive")
            stage = root / "stage"
            stage.mkdir()
            receipt = {"package": "sharp", "bytes": 17, "sha256": "0" * 64,
                       "sha512_integrity": "sha512-" + base64.b64encode(bytes(64)).decode()}
            with self.assertRaisesRegex(AssemblyError, "mismatch"):
                extract_archive(source, receipt, {}, stage)
            self.assertEqual(list(stage.iterdir()), [])


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive-root", type=Path, help="directory containing the six exact .tgz archives")
    parser.add_argument("--output-release", type=Path, help="existing release staging root with runtime/design")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        unittest.main(argv=["assemble-raster-prebuilt.py"], verbosity=2)
        return
    if args.archive_root is None or args.output_release is None:
        parser.error("--archive-root and --output-release are required")
    try:
        print(json.dumps(assemble(args.archive_root, args.output_release), indent=2))
    except (AssemblyError, OSError, KeyError, TypeError, json.JSONDecodeError,
            tarfile.TarError) as error:
        parser.exit(1, f"raster assembly refused: {error}\n")


if __name__ == "__main__":
    main()
