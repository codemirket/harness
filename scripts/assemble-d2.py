#!/usr/bin/env python3
"""Assemble the approved private D2 payload without running upstream code."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tarfile
import tempfile
import unittest


DEFAULT_MANIFEST = Path(__file__).resolve().parents[1] / "packaging/admission/m4-d2.manifest.json"
TARGET = "aarch64-apple-darwin"
MAX_ARCHIVE_BYTES = 256 * 1024 * 1024
SHA256 = re.compile(r"[0-9a-f]{64}\Z")


class AssemblyError(ValueError):
    """The source archive or destination did not satisfy the receipt."""


def safe_name(value: str) -> str:
    if not isinstance(value, str) or not value or "\\" in value or ":" in value or "\0" in value:
        raise AssemblyError("unsafe archive or manifest path")
    value = value.removesuffix("/")
    if value.startswith("/") or any(part in ("", ".", "..") for part in value.split("/")):
        raise AssemblyError("non-canonical or escaping path: " + value)
    return value


def digest_stream(stream) -> tuple[int, str]:
    digest = hashlib.sha256()
    size = 0
    stream.seek(0)
    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
        size += len(chunk)
        digest.update(chunk)
    stream.seek(0)
    return size, digest.hexdigest()


def check_identity(entry: dict, *, archive: bool = False) -> None:
    size_key = "bytes"
    if type(entry.get(size_key)) is not int or entry[size_key] < 0:
        raise AssemblyError("invalid byte count")
    if not isinstance(entry.get("sha256"), str) or not SHA256.fullmatch(entry["sha256"]):
        raise AssemblyError("invalid SHA-256")
    if archive and entry[size_key] > MAX_ARCHIVE_BYTES:
        raise AssemblyError("archive exceeds the assembly bound")


def load_manifest(path: Path) -> dict:
    try:
        manifest = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise AssemblyError("cannot read admission manifest: " + str(error)) from error
    if manifest.get("schema") != 1 or manifest.get("component") != "d2" or manifest.get("version") != "0.9.0":
        raise AssemblyError("manifest is not the approved D2 v0.9.0 receipt")
    if manifest.get("target") != TARGET:
        raise AssemblyError("manifest target is not the qualified macOS ARM64 target")

    archive = manifest.get("archive")
    if not isinstance(archive, dict):
        raise AssemblyError("manifest archive is missing")
    archive_name = safe_name(archive.get("name"))
    if "/" in archive_name or not archive_name.endswith(".tar.gz"):
        raise AssemblyError("D2 archive must be a tar.gz basename")
    check_identity({"bytes": archive.get("bytes"), "sha256": archive.get("sha256")}, archive=True)

    allowed = manifest.get("allowed_archive_members")
    if not isinstance(allowed, list) or not allowed:
        raise AssemblyError("manifest archive member allowlist is missing")
    allowed_by_name: dict[str, dict] = {}
    for entry in allowed:
        name = safe_name(entry.get("path"))
        if name in allowed_by_name or entry.get("kind") not in ("directory", "file"):
            raise AssemblyError("duplicate or nonregular allowlisted archive member")
        if type(entry.get("bytes")) is not int or entry["bytes"] < 0:
            raise AssemblyError("invalid allowlisted member size")
        allowed_by_name[name] = {"kind": entry["kind"], "bytes": entry["bytes"]}

    selected = manifest.get("selected_files")
    if not isinstance(selected, list) or not selected:
        raise AssemblyError("manifest selected files are missing")
    selected_by_name: dict[str, dict] = {}
    destinations: set[str] = set()
    for entry in selected:
        name = safe_name(entry.get("archive_path"))
        destination = safe_name(entry.get("destination"))
        if name in selected_by_name or destination in destinations:
            raise AssemblyError("duplicate selected member or destination")
        if name not in allowed_by_name or allowed_by_name[name]["kind"] != "file":
            raise AssemblyError("selected member is not an allowlisted regular file")
        if entry.get("mode") not in ("0644", "0755"):
            raise AssemblyError("selected file mode is not normalized")
        check_identity(entry)
        if entry["bytes"] != allowed_by_name[name]["bytes"]:
            raise AssemblyError("selected size disagrees with archive allowlist")
        selected_by_name[name] = entry
        destinations.add(destination)
    if "runtime/d2/d2" not in destinations or "runtime/d2/LICENSE.txt" not in destinations or "runtime/d2/THIRD_PARTY_NOTICES.txt" not in destinations:
        raise AssemblyError("D2 executable and notices must all be selected")
    notices = manifest.get("notices")
    if not isinstance(notices, dict) or safe_name(notices.get("spdx_name")) != "d2.spdx.json":
        raise AssemblyError("D2 SPDX notice receipt is missing")
    notice_destination = safe_name(notices.get("spdx_destination"))
    if notice_destination != "runtime/d2/d2.spdx.json":
        raise AssemblyError("D2 SPDX notice destination is not fixed")
    check_identity({"bytes": notices.get("spdx_bytes"), "sha256": notices.get("spdx_sha256")})
    return {"archive": archive, "allowed": allowed_by_name, "selected": selected_by_name, "notice": notices}


def check_output(path: Path) -> None:
    if not path.is_absolute() or ".." in path.parts or path.is_symlink() or not path.is_dir() or any(path.iterdir()):
        raise AssemblyError("output release directory must be an existing empty non-symlink directory")
    for ancestor in path.parents:
        if ancestor.is_symlink():
            raise AssemblyError("output path has a symlink ancestor")


def archive_member(item: tarfile.TarInfo) -> tuple[str, str, int]:
    name = safe_name(item.name)
    if item.isdir():
        kind = "directory"
    elif item.isfile():
        kind = "file"
    else:
        kind = "special"
    return name, kind, item.size


def copy_verified(reader, destination: Path, expected: dict) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    size = 0
    with destination.open("xb") as writer:
        for chunk in iter(lambda: reader.read(1024 * 1024), b""):
            size += len(chunk)
            if size > expected["bytes"]:
                raise AssemblyError("selected member exceeds its receipt size")
            digest.update(chunk)
            writer.write(chunk)
        writer.flush()
        os.fsync(writer.fileno())
    if (size, digest.hexdigest()) != (expected["bytes"], expected["sha256"]):
        raise AssemblyError("selected member content identity mismatch")
    destination.chmod(int(expected["mode"], 8))


def assemble(archive_root: Path, output_release: Path, manifest_path: Path = DEFAULT_MANIFEST, spdx_path: Path | None = None) -> dict:
    receipt = load_manifest(manifest_path)
    if spdx_path is None:
        raise AssemblyError("the exact D2 SPDX document is required for assembly")
    check_output(output_release)
    source = archive_root / receipt["archive"]["name"]
    if source.is_symlink() or not source.is_file():
        raise AssemblyError("archive must be a regular file, not a symlink")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0)
    with os.fdopen(os.open(source, flags), "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise AssemblyError("archive is not a regular file")
        expected_archive = (receipt["archive"]["bytes"], receipt["archive"]["sha256"])
        if digest_stream(stream) != expected_archive:
            raise AssemblyError("archive size or SHA-256 mismatch")
        with tempfile.TemporaryDirectory(prefix=".d2-assembly-", dir=output_release.parent) as scratch:
            stage = Path(scratch)
            seen: set[str] = set()
            found: set[str] = set()
            with tarfile.open(fileobj=stream, mode="r:gz") as archive:
                for item in archive:
                    name, kind, size = archive_member(item)
                    if name in seen:
                        raise AssemblyError("duplicate archive member: " + name)
                    seen.add(name)
                    allow = receipt["allowed"].get(name)
                    if allow is None or allow["kind"] != kind or allow["bytes"] != size:
                        raise AssemblyError("archive member differs from receipt: " + name)
                    selected = receipt["selected"].get(name)
                    if selected is None:
                        continue
                    reader = archive.extractfile(item)
                    if reader is None:
                        raise AssemblyError("selected member could not be read: " + name)
                    with reader:
                        copy_verified(reader, stage / selected["destination"], selected)
                    found.add(name)
            if seen != set(receipt["allowed"]):
                raise AssemblyError("archive member set differs from receipt")
            if found != set(receipt["selected"]):
                raise AssemblyError("selected member missing from archive")
            if digest_stream(stream) != expected_archive:
                raise AssemblyError("archive changed during assembly")
            notice_source = Path(spdx_path)
            if notice_source.is_symlink() or not notice_source.is_file():
                raise AssemblyError("SPDX notice must be a regular file, not a symlink")
            with notice_source.open("rb") as notice_reader:
                notice_size, notice_sha = digest_stream(notice_reader)
                expected_notice = (receipt["notice"]["spdx_bytes"], receipt["notice"]["spdx_sha256"])
                if (notice_size, notice_sha) != expected_notice:
                    raise AssemblyError("SPDX notice size or SHA-256 mismatch")
                notice_reader.seek(0)
                copy_verified(
                    notice_reader,
                    stage / receipt["notice"]["spdx_destination"],
                    {
                        "bytes": receipt["notice"]["spdx_bytes"],
                        "sha256": receipt["notice"]["spdx_sha256"],
                        "mode": "0644",
                    },
                )
            check_output(output_release)
            runtime = output_release / "runtime"
            runtime.mkdir()
            os.rename(stage / "runtime/d2", runtime / "d2")
    return {
        "target": TARGET,
        "output_release": str(output_release),
        "files": list(receipt["selected"].values()) + [{"destination": receipt["notice"]["spdx_destination"], "bytes": receipt["notice"]["spdx_bytes"], "sha256": receipt["notice"]["spdx_sha256"], "mode": "0644"}],
        "runtime_executed": False,
    }


class AssemblyTests(unittest.TestCase):
    """Synthetic receipt tests; no upstream archive or executable is run."""

    def make_fixture(self, *, symlink: bool = False):
        scratch_parent = Path("/private/tmp") if Path("/private/tmp").is_dir() else Path(tempfile.gettempdir())
        root = Path(tempfile.mkdtemp(prefix="d2-assembly-test-", dir=scratch_parent))
        archive_root = root / "archive"
        archive_root.mkdir()
        output = root / "output"
        output.mkdir()
        files = {
            "d2-v0.9.0/bin/d2": b"synthetic-d2",
            "d2-v0.9.0/LICENSE.txt": b"license",
            "d2-v0.9.0/THIRD_PARTY_NOTICES.txt": b"notices",
        }
        archive_path = archive_root / "fixture.tar.gz"
        with tarfile.open(archive_path, "w:gz") as archive:
            for directory in ("d2-v0.9.0", "d2-v0.9.0/bin"):
                info = tarfile.TarInfo(directory)
                info.type = tarfile.DIRTYPE
                info.mode = 0o755
                archive.addfile(info)
            for name, content in files.items():
                info = tarfile.TarInfo(name)
                info.size = len(content)
                info.mode = 0o755 if name.endswith("/d2") else 0o644
                archive.addfile(info, __import__("io").BytesIO(content))
            if symlink:
                info = tarfile.TarInfo("d2-v0.9.0/bin/link")
                info.type = tarfile.SYMTYPE
                info.linkname = "d2"
                archive.addfile(info)
        with archive_path.open("rb") as stream:
            archive_size, archive_sha = digest_stream(stream)
        allowed = [{"path": "d2-v0.9.0", "kind": "directory", "bytes": 0}, {"path": "d2-v0.9.0/bin", "kind": "directory", "bytes": 0}, {"path": "d2-v0.9.0/LICENSE.txt", "kind": "file", "bytes": 7}, {"path": "d2-v0.9.0/THIRD_PARTY_NOTICES.txt", "kind": "file", "bytes": 7}, {"path": "d2-v0.9.0/bin/d2", "kind": "file", "bytes": 12}]
        if symlink:
            allowed.append({"path": "d2-v0.9.0/bin/link", "kind": "directory", "bytes": 0})
        selected = []
        for name, destination, mode in (("d2-v0.9.0/bin/d2", "runtime/d2/d2", "0755"), ("d2-v0.9.0/LICENSE.txt", "runtime/d2/LICENSE.txt", "0644"), ("d2-v0.9.0/THIRD_PARTY_NOTICES.txt", "runtime/d2/THIRD_PARTY_NOTICES.txt", "0644")):
            path = root / name
            content = files[name]
            selected.append({"archive_path": name, "destination": destination, "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest(), "mode": mode})
        manifest = {"schema": 1, "component": "d2", "version": "0.9.0", "target": TARGET, "archive": {"name": "fixture.tar.gz", "bytes": archive_size, "sha256": archive_sha}, "selected_files": selected, "allowed_archive_members": allowed}
        manifest_path = root / "manifest.json"
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        spdx = root / "d2.spdx.json"
        spdx.write_bytes(b"synthetic SPDX")
        manifest["notices"] = {"spdx_name": "d2.spdx.json", "spdx_destination": "runtime/d2/d2.spdx.json", "spdx_bytes": len(spdx.read_bytes()), "spdx_sha256": hashlib.sha256(spdx.read_bytes()).hexdigest()}
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        return root, archive_root, output, manifest_path, spdx

    def test_synthetic_archive_assembles_only_selected_files(self):
        root, archive_root, output, manifest, spdx = self.make_fixture()
        try:
            result = assemble(archive_root, output, manifest, spdx)
            self.assertFalse(result["runtime_executed"])
            self.assertEqual((output / "runtime/d2/d2").read_bytes(), b"synthetic-d2")
            self.assertFalse((output / "runtime/d2/README.md").exists())
        finally:
            __import__("shutil").rmtree(root)

    def test_symlink_archive_member_is_rejected(self):
        root, archive_root, output, manifest, spdx = self.make_fixture(symlink=True)
        try:
            with self.assertRaises(AssemblyError):
                assemble(archive_root, output, manifest, spdx)
        finally:
            __import__("shutil").rmtree(root)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive-root", type=Path)
    parser.add_argument("--output-release", type=Path)
    parser.add_argument("--spdx", type=Path, help="verified d2.spdx.json receipt artifact")
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--self-test", action="store_true", help="run synthetic receipt tests without executing D2")
    args = parser.parse_args()
    if args.self_test:
        unittest.main(argv=["assemble-d2.py"], verbosity=2)
        return
    if args.archive_root is None or args.output_release is None or args.spdx is None:
        parser.error("--archive-root, --output-release and --spdx are required unless --self-test is used")
    try:
        result = assemble(args.archive_root, args.output_release, args.manifest, args.spdx)
    except (AssemblyError, OSError, KeyError, TypeError, tarfile.TarError) as error:
        parser.exit(1, "D2 assembly failed: " + str(error) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
