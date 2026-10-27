#!/usr/bin/env python3
"""Assemble a private, exact editable-file candidate. This command installs no dependency."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil

SELECTION = Path(__file__).resolve().parents[1] / "packaging/admission/openpencil-editable.candidate.json"
PIN = "fd966b3140dfe08a47fbaa4481cebaecd4c1a79856b821edb2be2cf7b238fb0f"
class AssemblyError(Exception): pass

def digest(data: bytes) -> str: return hashlib.sha256(data).hexdigest()
def relative(value: str) -> Path:
    p = PurePosixPath(value)
    if p.is_absolute() or not p.parts or any(x in (".", "..") for x in p.parts) or "\\" in value or "\0" in value:
        raise AssemblyError("candidate paths must be bounded relative paths")
    if len(value) > 4096: raise AssemblyError("candidate path exceeds bound")
    return Path(*p.parts)
def real_file(root: Path, value: str) -> Path:
    p = root / relative(value)
    if p.is_symlink() or not p.is_file() or p.resolve() != p: raise AssemblyError("candidate source must be a canonical regular file")
    return p

def assemble(source_root: Path, output: Path) -> dict:
    selection_bytes = SELECTION.read_bytes()
    if digest(selection_bytes) != PIN: raise AssemblyError("candidate selection differs from script pin")
    selection = json.loads(selection_bytes)
    if not source_root.is_absolute() or source_root.is_symlink() or source_root.resolve() != source_root or not source_root.is_dir(): raise AssemblyError("select an explicit canonical package-source root")
    if not output.is_absolute() or output.exists() or output.is_symlink() or output.parent.resolve() != output.parent or not output.parent.is_dir(): raise AssemblyError("candidate output must be absent under a canonical existing parent")
    prepared = []
    destinations = set()
    files = []
    for entry in selection["files"]:
        dest = relative(entry["path"])
        key = entry["path"].casefold()
        if key in destinations or any(x in key for x in ("canvaskit", "yoga-layout", ".wasm")): raise AssemblyError("duplicate or forbidden candidate destination")
        destinations.add(key)
        data = real_file(source_root, entry["source"]).read_bytes()
        if "source_slice" in entry:
            if digest(data) != entry["source_sha256"]: raise AssemblyError("notice source differs from review")
            start, end = entry["source_slice"]
            data = data[start:end]
        if len(data) != entry["bytes"] or digest(data) != entry["sha256"]: raise AssemblyError("candidate source differs from reviewed file inventory")
        if b"\0asm" in data or b"AGFzbQ" in data: raise AssemblyError("WASM payload is unavailable in editable-file mode")
        prepared.append((dest, data))
        files.append({key: entry[key] for key in ("path", "bytes", "sha256", "kind")})
    manifest = {"schema": 1, "status": "candidate_not_release_admission", "selection_sha256": PIN, "engine_version": "0.16.0", "mode": "editable_file", "files": files}
    manifest_bytes = (json.dumps(manifest, separators=(",", ":")) + "\n").encode()
    # All inputs and destinations are validated before creating this task-owned directory.
    output.mkdir(mode=0o700)
    try:
        for dest, data in prepared:
            target = output / dest
            target.parent.mkdir(parents=True, exist_ok=True)
            with target.open("xb") as f:
                f.write(data)
                f.flush()
                os.fsync(f.fileno())
        (output / "runtime.json").write_bytes(manifest_bytes)
    except BaseException:
        shutil.rmtree(output)
        raise
    return {"status": "candidate_not_release_admission", "root": str(output), "runtime_sha256": digest(manifest_bytes), "selection_sha256": PIN, "files": len(files), "bytes": sum(len(data) for _, data in prepared), "native_engines": []}

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try: print(json.dumps(assemble(args.source_root, args.output)))
    except (AssemblyError, OSError, ValueError, KeyError) as e: parser.exit(1, f"openpencil candidate assembly refused: {e}\n")
if __name__ == "__main__": main()
