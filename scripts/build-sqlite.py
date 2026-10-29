#!/usr/bin/env python3
"""Build only the pinned SQLite source with explicit existing native tools.

No downloads, installation, upstream helpers, or host-library discovery. This is
an offline project build step; the receipt detects drift, not hostile tampering.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import stat
import struct
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "packaging/runtime/sqlite.json"
OUTPUT = ROOT / "target/native/sqlite"
TARGETS = {"aarch64-apple-darwin": "arm64", "aarch64-pc-windows-msvc": "arm64", "x86_64-pc-windows-msvc": "x64"}


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def regular(path):
    if not stat.S_ISREG(path.lstat().st_mode):
        raise ValueError(f"Expected regular file, not link/special file: {path}")


def no_symlink_path(path):
    for part in [path, *path.parents]:
        if part.is_symlink():
            raise ValueError(f"Symlinked build path refused: {part}")


def read_manifest(path=MANIFEST):
    regular(path)
    data = json.loads(path.read_text())
    if data["sqlite_version"] != "3.54.1" or data["source_modifications"] != "none":
        raise ValueError("Expected pinned unmodified SQLite 3.54.1")
    return data


def library_name(target):
    return "libsqlite3.a" if target == "aarch64-apple-darwin" else "sqlite3.lib"


def validate_receipt(output, manifest, target, manifest_path=MANIFEST):
    if target not in TARGETS:
        raise ValueError(f"Unsupported native target: {target}")
    no_symlink_path(output)
    if {p.name for p in output.iterdir()} != {"lib", "include", "build-receipt.json"}:
        raise ValueError("Unexpected native output entries")
    lib = output / "lib" / library_name(target)
    header = output / "include/sqlite3.h"
    receipt_file = output / "build-receipt.json"
    for directory, names in [(output / "lib", {lib.name}), (output / "include", {"sqlite3.h"})]:
        no_symlink_path(directory)
        if not directory.is_dir() or {p.name for p in directory.iterdir()} != names:
            raise ValueError(f"Unexpected/competing native files: {directory}")
    for path in [lib, header, receipt_file]:
        regular(path)
    if receipt_file.stat().st_size > 128 * 1024:
        raise ValueError("Oversized native receipt")
    r = json.loads(receipt_file.read_text())
    expected_sources = {PurePosixPath(x["path"]).name: x["sha256"] for x in manifest["files"] if x["path"] in manifest["source_selection"]}
    expected = {"schema": 1, "target": target, "sqlite_version": manifest["sqlite_version"], "sqlite_source_id": manifest["sqlite_source_id"], "manifest_sha256": digest(manifest_path), "source_archive_sha256": manifest["sha256"], "source_sha256": expected_sources, "defines": manifest["required_defines"], "object_machine": TARGETS[target]}
    for key, value in expected.items():
        if r.get(key) != value:
            raise ValueError(f"Native receipt mismatch: {key}")
    for key, path in [("library", lib), ("header", header)]:
        if r[key] != {"path": path.relative_to(output).as_posix(), "bytes": path.stat().st_size, "sha256": digest(path)}:
            raise ValueError(f"Native {key} changed after build")
    if digest(header) != expected_sources["sqlite3.h"]:
        raise ValueError("Header is not the pinned source")
    return r


def unpack_sources(archive, destination, manifest):
    regular(archive)
    if archive.stat().st_size != manifest["bytes"] or digest(archive) != manifest["sha256"]:
        raise ValueError("SQLite archive size/SHA-256 mismatch")
    selected = {x["path"]: x for x in manifest["files"] if x["path"] in manifest["source_selection"]}
    if {PurePosixPath(p).name for p in selected} != {"sqlite3.c", "sqlite3.h"} or len(selected) != 2:
        raise ValueError("Expected exactly the two pinned source members")
    found = set()
    seen = set()
    with zipfile.ZipFile(archive) as z:
        for item in z.infolist():
            name = item.filename.rstrip("/")
            path = PurePosixPath(name)
            if name in seen or not name or path.is_absolute() or str(path) != name or ".." in path.parts or "\\" in name or ":" in name:
                raise ValueError(f"Duplicate or unsafe archive member: {name}")
            seen.add(name)
            kind = stat.S_IFMT(item.external_attr >> 16)
            if item.is_dir():
                if kind not in (0, stat.S_IFDIR):
                    raise ValueError("Non-directory archive member")
                continue
            if kind not in (0, stat.S_IFREG):
                raise ValueError("Nonregular archive member")
            if item.filename not in selected:
                continue
            expected = selected[item.filename]
            if item.file_size != expected["bytes"]:
                raise ValueError("Source member size mismatch")
            data = z.read(item)
            if hashlib.sha256(data).hexdigest() != expected["sha256"] or hashlib.sha3_256(data).hexdigest() != expected["sha3_256"]:
                raise ValueError("Source member hash mismatch")
            (destination / path.name).write_bytes(data)
            found.add(item.filename)
    if found != set(selected) or digest(archive) != manifest["sha256"]:
        raise ValueError("Missing source or changed archive")


def object_machine(path, target):
    with path.open("rb") as f:
        head = f.read(12)
    if target == "aarch64-apple-darwin":
        if len(head) < 12 or struct.unpack("<III", head)[:2] != (0xFEEDFACF, 0x0100000C):
            raise ValueError("Compiler did not produce ARM64 Mach-O")
        return "arm64"
    code = int.from_bytes(head[:2], "little")
    expected = 0xAA64 if target == "aarch64-pc-windows-msvc" else 0x8664
    if code != expected:
        raise ValueError("Compiler did not produce requested Windows COFF architecture")
    return TARGETS[target]


def build(archive, target, compiler, archiver):
    manifest = read_manifest()
    if target == "aarch64-apple-darwin":
        if platform.system() != "Darwin" or platform.machine().lower() not in ("arm64", "aarch64"):
            raise ValueError("macOS ARM64 build requires a native macOS ARM64 host")
        config = manifest["macos"]
    else:
        if platform.system() != "Windows":
            raise ValueError("Windows builds require an actual Windows host/toolchain")
        config = manifest["windows"]
    for tool in [compiler, archiver]:
        if not tool.is_absolute() or not tool.is_file():
            raise ValueError(f"Explicit existing absolute tool path required: {tool}")
    no_symlink_path(OUTPUT)
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    # Permit replacing only this generator's validated previous payload.
    if OUTPUT.exists():
        old = json.loads((OUTPUT / "build-receipt.json").read_text())
        validate_receipt(OUTPUT, manifest, old["target"])
    env = {k: v for k, v in os.environ.items() if k not in {"CL", "_CL_", "LINK", "_LINK_", "CFLAGS", "CPPFLAGS", "LDFLAGS", "CPATH", "C_INCLUDE_PATH", "CPLUS_INCLUDE_PATH", "LIBRARY_PATH", "LD_PRELOAD", "MACOSX_DEPLOYMENT_TARGET"} and not k.startswith("DYLD_")}
    # Apple ar honors this to omit wall-clock member timestamps.
    env["ZERO_AR_DATE"] = "1"
    with tempfile.TemporaryDirectory(prefix=".sqlite-build-", dir=OUTPUT.parent) as temp:
        stage = Path(temp)
        source = stage / "source"
        payload = stage / "payload"
        source.mkdir()
        (payload / "include").mkdir(parents=True)
        (payload / "lib").mkdir()
        unpack_sources(archive, source, manifest)
        obj = stage / ("sqlite3.o" if target == "aarch64-apple-darwin" else "sqlite3.obj")
        lib = payload / "lib" / library_name(target)
        if target == "aarch64-apple-darwin":
            version_cmd = [str(compiler), "--version"]
            compile_cmd = [str(compiler), *config["compiler_flags"], *["-D" + x for x in manifest["required_defines"]], "-c", "sqlite3.c", "-o", str(obj)]
            archive_cmd = [str(archiver), "rcs", str(lib), str(obj)]
        else:
            version_cmd = [str(compiler), "/Bv"]
            compile_cmd = [str(compiler), *config["compiler_flags"], *["/D" + x for x in manifest["required_defines"]], "sqlite3.c", "/Fo" + str(obj)]
            archive_cmd = [str(archiver), "/nologo", "/OUT:" + str(lib), str(obj)]
        version = subprocess.run(version_cmd, cwd=stage, env=env, capture_output=True, text=True, timeout=30)
        if target == "aarch64-apple-darwin" and version.returncode:
            raise ValueError(version.stderr)
        # cl /Bv without a translation unit can return a usage error; the actual
        # compile must succeed, and its COFF architecture is checked below.
        compiled = subprocess.run(compile_cmd, cwd=source, env=env, capture_output=True, text=True, timeout=180)
        if compiled.returncode:
            raise ValueError("Native compile failed: " + compiled.stdout + compiled.stderr)
        machine = object_machine(obj, target)
        archived = subprocess.run(archive_cmd, cwd=stage, env=env, capture_output=True, text=True, timeout=60)
        if archived.returncode:
            raise ValueError("Native archive failed: " + archived.stdout + archived.stderr)
        regular(lib)
        with lib.open("rb") as f:
            if f.read(8) != b"!<arch>\n":
                raise ValueError("Expected static archive format")
        shutil.copyfile(source / "sqlite3.h", payload / "include/sqlite3.h")
        receipt = {"schema": 1, "target": target, "sqlite_version": manifest["sqlite_version"], "sqlite_source_id": manifest["sqlite_source_id"], "manifest_sha256": digest(MANIFEST), "source_archive_sha256": manifest["sha256"], "source_sha256": {p.name: digest(p) for p in source.iterdir()}, "defines": manifest["required_defines"], "object_machine": machine, "compiler": str(compiler), "compiler_resolved": str(compiler.resolve()), "compiler_version": version.stdout + version.stderr, "compiler_version_exit": version.returncode, "archiver": str(archiver), "archiver_resolved": str(archiver.resolve()), "compile_command": compile_cmd, "compile_cwd": str(source), "archive_command": archive_cmd, "archive_environment": {"ZERO_AR_DATE": "1"}, "compiler_stdout": compiled.stdout, "compiler_stderr": compiled.stderr}
        for key, file in [("library", lib), ("header", payload / "include/sqlite3.h")]:
            receipt[key] = {"path": file.relative_to(payload).as_posix(), "bytes": file.stat().st_size, "sha256": digest(file)}
        (payload / "build-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        validate_receipt(payload, manifest, target)
        previous = stage / "previous"
        if OUTPUT.exists():
            OUTPUT.rename(previous)
        try:
            payload.rename(OUTPUT)
        except BaseException:
            if previous.exists():
                previous.rename(OUTPUT)
            raise
    print(json.dumps({"target": target, "output": str(OUTPUT), "library": receipt["library"], "runtime_executed": False}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--target", required=True, choices=TARGETS)
    parser.add_argument("--compiler", required=True, type=Path)
    parser.add_argument("--archiver", required=True, type=Path)
    args = parser.parse_args()
    if not args.archive.is_absolute():
        parser.error("--archive must be absolute")
    try:
        build(args.archive, args.target, args.compiler, args.archiver)
    except (ValueError, OSError, KeyError, subprocess.SubprocessError, zipfile.BadZipFile) as exc:
        parser.exit(1, f"SQLite build refused: {exc}\n")


if __name__ == "__main__":
    main()
