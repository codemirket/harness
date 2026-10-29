#!/usr/bin/env python3
"""Assemble the pinned, disposable M0 Node runtime; never install/run it."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tarfile
import tempfile
import zipfile

DEFAULT_MANIFEST = Path(__file__).resolve().parents[1] / "packaging/runtime/node.json"
TARGETS = {
    "aarch64-apple-darwin": "bin/node",
    "x86_64-pc-windows-msvc": "node.exe",
    "aarch64-pc-windows-msvc": "node.exe",
}


class AssemblyError(ValueError):
    pass


def safe_name(name):
    """Accept canonical portable archive paths, including directory trailing slash."""
    if not isinstance(name, str) or not name or "\\" in name or ":" in name or "\0" in name:
        raise AssemblyError("unsafe archive/manifest path")
    name = name.removesuffix("/")
    if name.startswith("/") or any(p in ("", ".", "..") for p in name.split("/")):
        raise AssemblyError("non-canonical or escaping path: " + name)
    return name


def checksum(stream):
    digest = hashlib.sha256()
    size = 0
    stream.seek(0)
    for chunk in iter(lambda: stream.read(1024 * 1024), b""):
        size += len(chunk)
        digest.update(chunk)
    stream.seek(0)
    return size, digest.hexdigest()


def identity(value):
    if type(value.get("bytes")) is not int or value["bytes"] < 0:
        raise AssemblyError("invalid byte count")
    if not isinstance(value.get("sha256"), str) or not re.fullmatch("[0-9a-f]{64}", value["sha256"]):
        raise AssemblyError("invalid SHA-256")


def load_selection(manifest, target):
    data = json.loads(Path(manifest).read_text(encoding="utf-8"))
    matches = [x for x in data["targets"] if x["target"] == target]
    if target not in TARGETS or len(matches) != 1:
        raise AssemblyError("target must have exactly one supported manifest entry")
    selected = matches[0]
    archive = safe_name(selected["archive"])
    if "/" in archive or not archive.endswith((".tar.gz", ".zip")):
        raise AssemblyError("archive must be a tar.gz/zip basename")
    identity({"bytes": selected["archive_bytes"], "sha256": selected["archive_sha256"]})
    files = selected["selected_files"]
    if len(files) != 2:
        raise AssemblyError("M0 selection must contain exactly two files")
    destinations = {"runtime/node/" + TARGETS[target], "runtime/node/LICENSE"}
    seen = set()
    for entry in files:
        name = safe_name(entry["path"])
        destination = safe_name(entry["destination_relative"])
        if name in seen or destination not in destinations:
            raise AssemblyError("duplicate member or unexpected destination")
        seen.add(name)
        destinations.remove(destination)
        identity(entry)
        expected_mode = "0755" if target == "aarch64-apple-darwin" and destination.endswith("/bin/node") else "0644"
        if entry["mode"] != expected_mode or entry["content_transform"] != "none":
            raise AssemblyError("unexpected mode/content transform")
    if selected["selected_file_bytes"] != sum(x["bytes"] for x in files):
        raise AssemblyError("selected size total mismatch")
    return selected


def check_output(path):
    if not path.is_absolute() or ".." in path.parts:
        raise AssemblyError("output release path must be absolute and canonical")
    for part in (path, *path.parents):
        if part.is_symlink():
            raise AssemblyError("output path or ancestor is a symlink")
    if not path.is_dir() or any(path.iterdir()):
        raise AssemblyError("output release directory must already exist and be empty")


def archive_members(archive, is_zip):
    for item in archive.infolist() if is_zip else archive:
        name = safe_name(item.filename if is_zip else item.name)
        if is_zip:
            mode = item.external_attr >> 16
            kind = "directory" if item.is_dir() else "symlink" if stat.S_ISLNK(mode) else "file" if stat.S_IFMT(mode) in (0, stat.S_IFREG) else "special"
            size = item.file_size
        else:
            kind = "file" if item.isfile() else "directory" if item.isdir() else "symlink" if item.issym() else "special"
            size = item.size
        yield name, kind, size, item


def copy_verified(reader, destination, entry):
    destination.parent.mkdir(parents=True, exist_ok=True)
    digest = hashlib.sha256()
    size = 0
    with destination.open("xb") as writer:
        for chunk in iter(lambda: reader.read(1024 * 1024), b""):
            size += len(chunk)
            if size > entry["bytes"]:
                raise AssemblyError("selected member exceeds expected size")
            digest.update(chunk)
            writer.write(chunk)
        writer.flush()
        os.fsync(writer.fileno())
    if size != entry["bytes"] or digest.hexdigest() != entry["sha256"]:
        raise AssemblyError("selected member content identity mismatch")
    destination.chmod(int(entry["mode"], 8))


def assemble(archive_root, target, output_release, manifest=DEFAULT_MANIFEST):
    """Only fake fixtures may use an alternate manifest in production assembly."""
    selection = load_selection(manifest, target)
    output = Path(output_release)
    check_output(output)
    source = Path(archive_root) / selection["archive"]
    if source.is_symlink() or not source.is_file():
        raise AssemblyError("archive must be a regular file, not a symlink")
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_BINARY", 0)
    with os.fdopen(os.open(source, flags), "rb") as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise AssemblyError("archive is not a regular file")
        expected = (selection["archive_bytes"], selection["archive_sha256"])
        if checksum(stream) != expected:
            raise AssemblyError("archive size or SHA-256 mismatch")
        # Stage outside the empty release; never extract archive paths wholesale.
        with tempfile.TemporaryDirectory(prefix=".m0-runtime-", dir=output.parent) as scratch:
            stage = Path(scratch)
            wanted = {x["path"]: x for x in selection["selected_files"]}
            seen = set()
            found = set()
            is_zip = source.name.endswith(".zip")
            archive = zipfile.ZipFile(stream) if is_zip else tarfile.open(fileobj=stream, mode="r:gz")
            with archive:
                for name, kind, size, item in archive_members(archive, is_zip):
                    if name in seen:
                        raise AssemblyError("duplicate archive member: " + name)
                    seen.add(name)
                    if kind == "special":
                        raise AssemblyError("unsupported nonregular archive member: " + name)
                    if name not in wanted:
                        # Official macOS archive has unused npm links; never materialize them.
                        continue
                    entry = wanted[name]
                    if kind != "file" or size != entry["bytes"]:
                        raise AssemblyError("selected member is nonregular or has wrong size: " + name)
                    reader = archive.open(item) if is_zip else archive.extractfile(item)
                    with reader:
                        copy_verified(reader, stage / entry["destination_relative"], entry)
                    found.add(name)
            if found != set(wanted):
                raise AssemblyError("selected member missing from archive")
            if checksum(stream) != expected:
                raise AssemblyError("archive changed during assembly")
            check_output(output)
            # Exclusive directory creation prevents another assembler's output being replaced.
            runtime = output / "runtime"
            runtime.mkdir()
            try:
                os.rename(stage / "runtime/node", runtime / "node")
            except BaseException:
                # Never recursively delete a destination someone else might have populated.
                try:
                    runtime.rmdir()
                except OSError:
                    pass
                raise
    return {"target": target, "output_release": str(output), "files": selection["selected_files"], "runtime_executed": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive-root", required=True, type=Path)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--output-release", required=True, type=Path)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST, help="Authoritative selection by default; override for synthetic fixtures only")
    args = parser.parse_args()
    try:
        result = assemble(args.archive_root, args.target, args.output_release, args.manifest)
    except (AssemblyError, OSError, KeyError, TypeError, json.JSONDecodeError, tarfile.TarError, zipfile.BadZipFile) as error:
        parser.exit(1, "M0 runtime assembly failed: " + str(error) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
