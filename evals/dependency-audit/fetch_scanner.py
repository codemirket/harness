#!/usr/bin/env python3
"""Fetch the explicit development scanner pin; never execute an installer."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import tarfile
import urllib.request


def binary(archive, expected):
    if hashlib.sha256(archive).hexdigest() != expected:
        raise ValueError("scanner archive digest mismatch")
    with tarfile.open(fileobj=io.BytesIO(archive)) as members:
        matches = [m for m in members if m.name == "trivy"]
        if len(matches) != 1 or not matches[0].isfile() or matches[0].size > 256 * 1024 * 1024:
            raise ValueError("invalid scanner archive member")
        return members.extractfile(matches[0]).read()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True, choices=["linux-amd64", "darwin-arm64"])
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    pin = json.loads(Path(__file__).with_name("scanner.json").read_text())["targets"][args.target]
    with urllib.request.urlopen(pin["url"], timeout=120) as response:
        data = response.read(128 * 1024 * 1024 + 1)
    if len(data) > 128 * 1024 * 1024:
        raise ValueError("scanner download too large")
    content = binary(data, pin["sha256"])
    with args.output.open("xb") as out:
        out.write(content)
    args.output.chmod(0o755)
