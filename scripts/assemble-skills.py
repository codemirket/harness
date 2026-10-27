#!/usr/bin/env python3
"""Copy the exact authored native skill catalog into an empty release slot."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tempfile

SOURCE = Path(__file__).resolve().parents[1] / "skills"
ID = re.compile(r"[a-z0-9]+(?:-[a-z0-9]+)*\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")


class AssemblyError(ValueError):
    pass


def real_directory(path: Path) -> None:
    if path.is_symlink() or not path.is_dir():
        raise AssemblyError(f"expected a real directory: {path}")


def bounded_file(path: Path, limit: int) -> bytes:
    if path.is_symlink() or not path.is_file() or path.stat().st_size > limit:
        raise AssemblyError(f"expected a bounded regular file: {path}")
    data = path.read_bytes()
    if len(data) > limit:
        raise AssemblyError(f"source changed or exceeded its bound: {path}")
    return data


def inventory(source: Path) -> dict[str, bytes]:
    real_directory(source)
    catalog_bytes = bounded_file(source / "catalog.json", 2 * 1024 * 1024)
    catalog = json.loads(catalog_bytes)
    if catalog.get("schema_version") != 1:
        raise AssemblyError("unsupported skill catalog schema")
    basis = catalog.get("authoring_basis")
    if not isinstance(basis, list) or not basis or len(basis) > 16:
        raise AssemblyError("missing or oversized skill authoring basis")
    for pin in basis:
        if not isinstance(pin, dict) or set(pin) != {"path", "sha256"}:
            raise AssemblyError("invalid skill authoring pin")
        relative, expected = pin["path"], pin["sha256"]
        if (
            not isinstance(relative, str)
            or not relative
            or Path(relative).is_absolute()
            or any(character in relative for character in ("\\", ":", "\0"))
            or any(part in ("", ".", "..") for part in relative.split("/"))
            or not isinstance(expected, str)
            or not SHA.fullmatch(expected)
        ):
            raise AssemblyError("unsafe skill authoring pin")
        candidate = source.parent / relative
        if any(ancestor.is_symlink() for ancestor in (candidate, *candidate.parents)):
            raise AssemblyError("skill authoring pin has a symlink ancestor")
        if hashlib.sha256(bounded_file(candidate, 2 * 1024 * 1024)).hexdigest() != expected:
            raise AssemblyError(f"stale skill authoring pin: {relative}")
    globals_, project, profiles = (catalog.get(key) for key in ("global", "project", "profiles"))
    if not isinstance(globals_, list) or not isinstance(project, list) or not isinstance(profiles, dict):
        raise AssemblyError("invalid skill catalog groups")
    if (len(globals_), len(project), len(profiles)) != (9, 58, 8):
        raise AssemblyError("catalog does not have the agreed release shape")
    files = {
        "catalog.json": catalog_bytes,
        "PROVENANCE.md": bounded_file(source / "PROVENANCE.md", 512 * 1024),
    }
    seen = set()
    for entry in [*globals_, *project]:
        if not isinstance(entry, dict):
            raise AssemblyError("invalid skill entry")
        id_ = entry.get("id")
        if not isinstance(id_, str) or not ID.fullmatch(id_) or id_.startswith("mirket-") or id_ in seen:
            raise AssemblyError("invalid, branded or duplicate skill ID")
        seen.add(id_)
        relative = f"{id_}/SKILL.md"
        if entry.get("path") != f"skills/{relative}":
            raise AssemblyError(f"skill path differs from ID: {id_}")
        folder = source / id_
        real_directory(folder)
        if {p.name for p in folder.iterdir()} != {"SKILL.md"}:
            raise AssemblyError(f"skill directory contains unknown entries: {id_}")
        body = bounded_file(folder / "SKILL.md", 128 * 1024)
        digest = entry.get("source_sha256")
        if not isinstance(digest, str) or not SHA.fullmatch(digest):
            raise AssemblyError(f"invalid skill digest: {id_}")
        if entry.get("source_bytes") != len(body) or hashlib.sha256(body).hexdigest() != digest:
            raise AssemblyError(f"skill bytes differ from catalog: {id_}")
        files[relative] = body
    if {p.name for p in source.iterdir()} != {"catalog.json", "PROVENANCE.md", *seen}:
        raise AssemblyError("skill root contains unknown entries")
    return files


def assemble(output_release: Path, source: Path = SOURCE) -> dict:
    if not output_release.is_absolute() or ".." in output_release.parts:
        raise AssemblyError("output release must be an absolute canonical path")
    for ancestor in (output_release, *output_release.parents):
        if ancestor.is_symlink():
            raise AssemblyError("output release ancestry contains a symlink")
    real_directory(output_release)
    destination = output_release / "skills"
    if destination.exists() or destination.is_symlink():
        raise AssemblyError("release skill slot must be absent")
    files = inventory(source)
    stage = Path(tempfile.mkdtemp(prefix=".skills-stage-", dir=output_release))
    try:
        for relative, data in files.items():
            target = stage / relative
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            with target.open("xb") as stream:
                stream.write(data)
                stream.flush()
                os.fsync(stream.fileno())
            target.chmod(0o600)
        if destination.exists() or destination.is_symlink():
            raise AssemblyError("release skill slot changed during assembly")
        stage.rename(destination)
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise
    return {"destination": str(destination), "files": len(files), "bytes": sum(map(len, files.values())), "runtime_executed": False}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-release", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(assemble(args.output_release), indent=2))
    except (AssemblyError, OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        parser.exit(1, f"Skill assembly failed: {error}\n")


if __name__ == "__main__":
    main()
