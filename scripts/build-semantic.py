#!/usr/bin/env python3
"""Exact private semantic-payload assembly.

Requires already admitted local archives/tools. Never downloads, installs, runs
payloads, discovers host libraries, or qualifies Windows by cross-compilation.
Receipts detect drift; they are checked against repository source pins.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import subprocess
import tarfile
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parent.parent
RUNTIME_MANIFEST = ROOT / "packaging/admission/m2-semantic-minilm.runtime.json"
spec = importlib.util.spec_from_file_location("mirket_sqlite_build", ROOT / "scripts/build-sqlite.py")
sqlite = importlib.util.module_from_spec(spec)
spec.loader.exec_module(sqlite)
digest, regular, no_links = sqlite.digest, sqlite.regular, sqlite.no_symlink_path
TARGETS = sqlite.TARGETS


def read_json(path):
    regular(path)
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f"Duplicate JSON key: {key}")
            result[key] = value
        return result
    return json.loads(path.read_text(), object_pairs_hook=unique)


def checked_file(path, expected):
    no_links(path)
    regular(path)
    if path.stat().st_size != expected["bytes"] or digest(path) != expected["sha256"]:
        raise ValueError(f"Admitted file hash/size mismatch: {path}")


def safe_member(name):
    while name.startswith("./"):
        name = name[2:]
    name = name.rstrip("/")
    if not name:
        return ""
    p = PurePosixPath(name)
    if p.is_absolute() or str(p) != name or ".." in p.parts or "\\" in name or ":" in name:
        raise ValueError(f"Unsafe archive path: {name}")
    return name


def extract_selected(archive, archive_pin, selected, destination, omitted_links=None):
    checked_file(archive, archive_pin)
    omitted_links = omitted_links or {}
    wanted = {safe_member(x["member"]): x for x in selected}
    if len(wanted) != len(selected) or len({x["path"] for x in selected}) != len(selected):
        raise ValueError("Duplicate selected member/destination")
    for item in selected:
        if safe_member(item["path"]) != item["path"] or not item["path"]:
            raise ValueError("Unsafe selected destination")
    seen, found = set(), set()
    def member(name, kind, size, reader, link=None):
        n = safe_member(name)
        if n in seen:
            raise ValueError(f"Duplicate archive member: {n}")
        seen.add(n)
        if kind == "dir":
            return
        if kind == "symlink" and n in omitted_links and omitted_links[n] == link and n not in wanted:
            # The exact upstream runtime's one compatibility symlink is never extracted.
            return
        if kind != "file" or not n:
            raise ValueError(f"Nonregular archive member: {name}")
        if n not in wanted:
            return
        pin = wanted[n]
        if size != pin["bytes"]:
            raise ValueError("Selected member size mismatch")
        data = reader()
        if len(data) != size or hashlib.sha256(data).hexdigest() != pin["sha256"]:
            raise ValueError("Selected member hash mismatch")
        out = destination / pin["path"]
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_bytes(data)
        found.add(n)
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as z:
            for i in z.infolist():
                mode = stat.S_IFMT(i.external_attr >> 16)
                kind = "dir" if i.is_dir() and mode in (0, stat.S_IFDIR) else "file" if not i.is_dir() and mode in (0, stat.S_IFREG) else "other"
                member(i.filename, kind, i.file_size, lambda i=i: z.read(i))
    else:
        with tarfile.open(archive, "r:gz") as t:
            for i in t:
                kind = "file" if i.isfile() else "dir" if i.isdir() else "symlink" if i.issym() else "other"
                member(i.name, kind, i.size, lambda i=i: t.extractfile(i).read(), i.linkname)
    if found != set(wanted) or digest(archive) != archive_pin["sha256"]:
        raise ValueError("Missing selected member or archive changed during read")


def exact_tree(output, paths):
    no_links(output)
    expected = set(paths)
    expected_dirs = set()
    for name in expected:
        expected_dirs.update(str(p) for p in PurePosixPath(name).parents if str(p) != ".")
    files, directories = set(), set()
    for p in output.rglob("*"):
        if p.is_symlink():
            raise ValueError(f"Symlinked payload entry: {p}")
        name = p.relative_to(output).as_posix()
        if p.is_dir():
            directories.add(name)
        else:
            regular(p)
            files.add(name)
    if files != expected or directories != expected_dirs:
        raise ValueError("Unexpected, missing, or competing payload entries")


def package_files(manifest, target):
    variant = manifest["targets"][target]
    if not variant["assembly_complete"]:
        raise ValueError("Incomplete private runtime admission: " + variant["incomplete_reason"])
    return [*variant["files"], *[{**x, "path": "model/" + x["path"]} for x in manifest["model"]["selected"]]]


def validate_package(output, target):
    m = read_json(RUNTIME_MANIFEST)
    files = package_files(m, target)
    exact_tree(output, {"package-receipt.json", *[x["path"] for x in files]})
    for f in files:
        checked_file(output / f["path"], f)
    r = read_json(output / "package-receipt.json")
    expected = {"schema": 1, "status": "private-runtime", "target": target, "manifest_sha256": digest(RUNTIME_MANIFEST), "runtime_version": m["runtime_version"], "model_revision": m["model"]["revision"], "provider": "CPUExecutionProvider", "telemetry": False, "files": [{k: x[k] for k in ["path", "bytes", "sha256"]} for x in files]}
    if r != expected:
        raise ValueError("Semantic package receipt differs from authoritative pins")
    return r


def assemble(args):
    m = read_json(RUNTIME_MANIFEST)
    package_files(m, args.target)  # Fail Windows completeness before creating output.
    variant = m["targets"][args.target]
    out = args.output
    no_links(out)
    if not out.is_absolute() or not out.parent.is_dir() or (out.exists() and (not out.is_dir() or any(out.iterdir()))):
        raise ValueError("Absolute empty output with an existing parent required")
    with tempfile.TemporaryDirectory(prefix=".semantic-assemble-", dir=out.parent) as temp:
        stage = Path(temp) / "payload"
        stage.mkdir()
        extract_selected(args.runtime_archive, variant["archive"], variant["files"], stage, variant["allowed_omitted_symlinks"])
        for f in m["model"]["selected"]:
            source = args.model_root / f["path"]
            checked_file(source, f)
            dest = stage / "model" / f["path"]
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, dest)
            checked_file(dest, f)
        files = package_files(m, args.target)
        r = {"schema": 1, "status": "private-runtime", "target": args.target, "manifest_sha256": digest(RUNTIME_MANIFEST), "runtime_version": m["runtime_version"], "model_revision": m["model"]["revision"], "provider": "CPUExecutionProvider", "telemetry": False, "files": [{k: x[k] for k in ["path", "bytes", "sha256"]} for x in files]}
        (stage / "package-receipt.json").write_text(json.dumps(r, indent=2) + "\n")
        validate_package(stage, args.target)
        if out.exists():
            out.rmdir()
        stage.rename(out)
    print(json.dumps({"output": str(out), "target": args.target, "payload_bytes": sum(x["bytes"] for x in files), "runtime_executed": False}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    a = sub.add_parser("assemble")
    for name in ["runtime-archive", "model-root", "output"]:
        a.add_argument("--" + name, required=True, type=Path)
    v = sub.add_parser("verify-package")
    v.add_argument("--output", required=True, type=Path)
    for command in [a, v]:
        command.add_argument("--target", required=True, choices=TARGETS)
    args = parser.parse_args()
    for value in vars(args).values():
        if isinstance(value, Path) and not value.is_absolute():
            parser.error("All input/output paths must be explicit absolute paths")
    try:
        if args.command == "assemble":
            assemble(args)
        elif args.command == "verify-package":
            validate_package(args.output, args.target)
            print("Private semantic package pins and completeness verified")
    except (ValueError, OSError, KeyError, subprocess.SubprocessError, tarfile.TarError, zipfile.BadZipFile) as exc:
        parser.exit(1, f"Semantic build input refused: {exc}\n")


if __name__ == "__main__":
    main()
