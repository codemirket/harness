"""Coordinate-only reader for the reviewed PNPM v9 lock, including all documents.

This deliberately supports only ordinary scalar package keys with exact npm
versions. It does not resolve YAML, imports, aliases, git dependencies or hooks.
An unfamiliar package-key form fails the audit rather than disappearing.
"""
import hashlib
import json
import re
from pathlib import Path


def lock_coordinates(text):
    packages = set()
    sections = 0
    in_packages = False
    for line in text.splitlines():
        if "\t" in line:
            raise ValueError("unsupported tab in pinned PNPM lock")
        if line and not line[0].isspace():
            in_packages = line == "packages:"
            sections += in_packages
        if not in_packages or not line.startswith("  ") or line.startswith("   ") or not line.strip():
            continue
        key = line[2:]
        if not key.endswith(":"):
            raise ValueError("unsupported PNPM package key")
        key = key[:-1]
        if key.startswith(("'", '"')):
            if key[-1:] != key[:1]:
                raise ValueError("unclosed PNPM package key")
            key = key[1:-1]
        match = re.fullmatch(r"(@[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+|[a-zA-Z0-9_.-]+)@"
                             r"([0-9]+\.[0-9]+\.[0-9]+(?:-[a-zA-Z0-9.-]+)?(?:\+[a-zA-Z0-9.-]+)?)", key)
        if not match:
            raise ValueError("unsupported PNPM package identity: " + key[:120])
        packages.add(("npm", *match.groups()))
    if not sections or not packages:
        raise ValueError("PNPM lock omitted packages")
    return sorted(packages)


def coordinates(root):
    folder = root / "evals/dependency-audit/penpot"
    provenance = json.loads((folder / "provenance.json").read_text())
    pin = json.loads((root / "registry/design-stack.json").read_text())["builds"]["penpot-mcp"]
    if provenance["source_archive"] != pin:
        raise ValueError("Penpot source pin changed; recapture reviewed lock")
    data = (folder / "pnpm-lock.yaml").read_bytes()
    if hashlib.sha256(data).hexdigest() != provenance["lock_sha256"]:
        raise ValueError("Penpot lock drift")
    if hashlib.sha256((folder / "LICENSE").read_bytes()).hexdigest() != provenance["license_sha256"]:
        raise ValueError("Penpot license capture drift")
    result = lock_coordinates(data.decode())
    if len(result) != provenance["coordinates"]:
        raise ValueError("Penpot coordinate coverage changed")
    return result
