#!/usr/bin/env python3
"""Check admitted native input identity and SDK layout before Cargo compiles.

Metadata inspection does not build dependencies. This guard catches accidental
source/build drift; it is not an audit or an adversarial filesystem sandbox.
"""
import argparse
import importlib.util
import hashlib
import json
import tarfile
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--semantic-package", type=Path, help="Verify an explicit assembled runtime/model payload too")
    parser.add_argument("--target", default=os.environ.get("CARGO_BUILD_TARGET"))
    args = parser.parse_args()
    target = args.target
    if target is None:
        info = subprocess.run(["rustc", "-vV"], cwd=ROOT, capture_output=True, text=True, check=True).stdout
        target = next(line.removeprefix("host: ") for line in info.splitlines() if line.startswith("host: "))
    spec = importlib.util.spec_from_file_location("mirket_sqlite_build", ROOT / "scripts/build-sqlite.py")
    native = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(native)
    native.validate_receipt(native.OUTPUT, native.read_manifest(), target)
    spec = importlib.util.spec_from_file_location("mirket_semantic_build", ROOT / "scripts/build-semantic.py")
    semantic = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(semantic)
    if args.semantic_package is not None:
        if not args.semantic_package.is_absolute():
            raise ValueError("--semantic-package must be absolute")
        semantic.validate_package(args.semantic_package, target)
    result = subprocess.run(["cargo", "metadata", "--locked", "--offline", "--format-version", "1", "--filter-platform", target], cwd=ROOT, capture_output=True, text=True, check=True)
    metadata = json.loads(result.stdout)
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    packages = metadata["packages"]
    for name, version in [("rmcp", "3.7.0"), ("rusqlite", "0.41.0"), ("libsqlite3-sys", "0.39.0")]:
        selected = [p for p in packages if p["name"] == name]
        if len(selected) != 1 or selected[0]["version"] != version:
            raise ValueError(f"Expected admitted {name} {version}")
        package = selected[0]
        features = set(nodes[package["id"]]["features"])
        if name == "rmcp":
            source = Path(package["manifest_path"]).parent.resolve()
            ancestor = source.parent.parent
            if (ancestor / ".git").exists() and (ancestor / ".githooks").exists():
                raise ValueError("rmcp source layout would configure Git hooks")
        elif name == "rusqlite" and features != {"backup", "modern_sqlite"}:
            raise ValueError(f"Unreviewed rusqlite features: {features}")
        elif name == "libsqlite3-sys" and features != {"bundled_bindings", "default", "min_sqlite_version_3_45_3", "pkg-config", "vcpkg"}:
            raise ValueError(f"Unreviewed libsqlite3-sys features: {features}")
    admitted = json.loads((ROOT / "packaging/admission/m2-semantic.inventory.json").read_text())
    guarded = {"fastembed", "ort", "ort-sys", "tokenizers", "onig", "onig_sys", "esaxx-rs", "serde_json"}
    for pin in [p for p in admitted["packages"] if p["name"] in guarded]:
        selected = [p for p in packages if p["name"] == pin["name"]]
        if len(selected) != 1 or selected[0]["version"] != pin["version"]:
            raise ValueError(f"Expected admitted semantic version: {pin['name']}")
        package = selected[0]
        if package["source"] != "registry+https://github.com/rust-lang/crates.io-index":
            raise ValueError(f"Unadmitted semantic package source: {pin['name']}")
        expected_features = set(pin["features"])
        if pin["name"] == "serde_json":
            # Reviewed feature-only amendment; no new packages or edges.
            # Keep the original admission inventory as historical evidence.
            expected_features = {"alloc", "default", "raw_value", "std"}
        if set(nodes[package["id"]]["features"]) != expected_features:
            raise ValueError(f"Unreviewed native/JSON features: {pin['name']}")
        source = Path(package["manifest_path"]).parent
        archive = source.parent.parent.parent / "cache" / source.parent.name / (source.name + ".crate")
        semantic.checked_file(archive, {"bytes": pin["archive_bytes"], "sha256": pin["crate_sha256"]})
        # Cargo's archive checksum does not detect a changed extracted build script
        # or vendored C file. Compare every archived regular source to the tree.
        seen = set()
        with tarfile.open(archive, "r:gz") as crate:
            for member in crate:
                name = semantic.safe_member(member.name)
                if name in seen:
                    raise ValueError("Duplicate semantic source archive member")
                seen.add(name)
                if member.isdir():
                    continue
                if not member.isfile():
                    raise ValueError("Nonregular semantic source archive member")
                parts = Path(name).parts
                if not parts or parts[0] != source.name:
                    raise ValueError("Unexpected semantic source archive root")
                local = source.joinpath(*parts[1:])
                semantic.no_links(local)
                semantic.regular(local)
                original = crate.extractfile(member).read()
                if local.stat().st_size != member.size or semantic.digest(local) != hashlib.sha256(original).hexdigest():
                    raise ValueError(f"Extracted semantic dependency source drift: {local}")
    print(f"Admitted SQLite receipt, semantic features/sources, and rmcp guards passed ({target})")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, KeyError, StopIteration, subprocess.SubprocessError, tarfile.TarError) as exc:
        sys.exit(f"Build inputs refused: {exc}")
