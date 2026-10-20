#!/usr/bin/env python3
"""Verify native distribution journeys and retain success/failure receipts."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import tempfile
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def target_for(system, machine):
    machine = machine.lower()
    arch = {"x86_64": "x86_64", "amd64": "x86_64", "arm64": "aarch64", "aarch64": "aarch64"}.get(machine)
    suffix = {"Darwin": "apple-darwin", "Windows": "pc-windows-msvc", "Linux": "unknown-linux-gnu"}.get(system)
    if not arch or not suffix:
        raise ValueError("unvalidated host platform")
    return arch + "-" + suffix


def candidate(dist, target):
    manifest = json.loads((dist / "mirket-release.json").read_text())
    matches = [a for a in manifest["assets"] if a["target"] == target]
    if len(matches) != 1:
        raise ValueError("exactly one native target required")
    asset = matches[0]
    from urllib.parse import urlsplit
    name = Path(urlsplit(asset["url"]).path).name
    expected = "mirket-" + target + (".exe" if "windows" in target else "")
    if name != expected:
        raise ValueError("unexpected executable name")
    executable = dist / name
    if executable.is_symlink() or sha(executable) != asset["sha256"]:
        raise ValueError("candidate digest drift")
    return executable, asset


def validate(dist, output):
    output.mkdir(parents=True, exist_ok=False)
    dist = dist.resolve(strict=True)
    report = {"ok": False, "host": {"system": platform.system(), "machine": platform.machine()},
              "validator_sha256": sha(Path(__file__)),
              "commands": [], "variants": [],
              "limits": "Native CLI/install/MCP checks only; no desktop GUI activation or contained package acceptance."}

    def run(arguments, name, json_output=False):
        start = time.monotonic()
        with (output / (name + ".stderr")).open("wb") as error:
            try:
                result = subprocess.run([str(a) for a in arguments], stdout=subprocess.PIPE,
                                        stderr=error, timeout=180)
            except subprocess.TimeoutExpired as failure:
                (output / (name + ".stdout")).write_bytes(failure.stdout or b"")
                report["commands"].append({"name": name, "exit": None, "timed_out": True,
                                           "seconds": time.monotonic() - start})
                raise
        (output / (name + ".stdout")).write_bytes(result.stdout)
        report["commands"].append({"name": name, "exit": result.returncode,
                                   "seconds": time.monotonic() - start})
        if result.returncode:
            raise ValueError(name + " failed")
        return json.loads(result.stdout) if json_output else None

    try:
        target = target_for(platform.system(), platform.machine())
        runtime, asset = candidate(dist, target)
        info = run([runtime, "info", "--json"], "identity", True)["process"]
        if info["target"] != target or info["executable_sha256"] != asset["sha256"]:
            raise ValueError("executed candidate identity differs")
        build = json.loads((dist / "mirket-build.json").read_text())
        # The build record owns the source identity; never substitute Git HEAD
        # for a dirty working tree's compiled payload.
        if (build.get("schema") != 1 or build["source_sha256"] != info["source_sha256"]
                or build["binary_sha256"] != asset["sha256"] or build["target"] != target
                or build["version"] != info["version"]):
            raise ValueError("packaged source identity differs")
        report["identity"] = info
        package_hashes = {}
        for variant, host in [("mirket", "all"), ("install-codex", "codex"),
                              ("install-claude", "claude"), ("install-all", "all")]:
            installer = dist / runtime.name.replace("mirket-", variant + "-", 1)
            if installer.is_symlink():
                raise ValueError("installer alias rejected")
            checksums = dict(line.split(None, 1)[::-1] for line in
                             (dist / "SHA256SUMS").read_text().splitlines() if line.strip())
            checksums = {k.strip().lstrip("*"): v for k, v in checksums.items()}
            if checksums.get(installer.name) != sha(installer):
                raise ValueError("installer digest drift")
            package_hashes[installer.name] = sha(installer)
            with tempfile.TemporaryDirectory(prefix="mirket-native-") as temporary:
                home = Path(temporary).resolve()
                arguments = [installer, "--home", home]
                arguments += ["setup", "--target", host, "--yes"] if variant == "mirket" else ["--yes"]
                run(arguments, variant + "-setup")
                managed = home / ".mirket/bin" / ("mirket.exe" if platform.system() == "Windows" else "mirket")
                doctor = run([managed, "--home", home, "doctor", "--json"], variant + "-doctor", True)
                if doctor.get("ok") is not True:
                    raise ValueError("doctor did not attest success")
                run([managed, "--home", home, "update", "--from", runtime, "--sha256", asset["sha256"]], variant + "-update")
                doctor = run([managed, "--home", home, "doctor", "--json"], variant + "-updated-doctor", True)
                if doctor.get("ok") is not True or sha(managed) != asset["sha256"]:
                    raise ValueError("managed update did not preserve candidate")
                protocol = run([managed, "--home", home, "mcp", "doctor", "--iterations", "2"], variant + "-mcp", True)
                if protocol.get("ok") is not True or protocol["runtime_identity"]["process"]["source_sha256"] != info["source_sha256"]:
                    raise ValueError("native MCP did not verify current source")
                report["variants"].append({"name": variant, "ok": True})
        if any(sha(dist / name) != digest for name, digest in package_hashes.items()):
            raise ValueError("packaged executables changed during validation")
        report["package_sha256"] = package_hashes
        report["ok"] = True
    except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
        report["error"] = str(error)
    finally:
        report["validator_unchanged"] = sha(Path(__file__)) == report["validator_sha256"]
        if not report["validator_unchanged"]:
            report["ok"] = False
            report["error"] = "validator source changed during validation"
        report["files"] = {p.name: sha(p) for p in sorted(output.iterdir()) if p.is_file()}
        (output / "receipt.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--dist", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    raise SystemExit(validate(args.dist, args.output))
