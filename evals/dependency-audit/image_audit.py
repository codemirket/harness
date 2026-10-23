#!/usr/bin/env python3
"""Explicit Trivy scans of pinned Linux ARM64 images and verified local exports.

Never pulls into the runtime engine, starts containers, upgrades packages or
executes image contents. Raw reports and scanner provenance stay in output.
"""
import argparse
import datetime
import time
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from image_contents import payload_inventory

ROOT = Path(__file__).resolve().parents[2]
BUILT = ("design-tools", "agent-runtime", "kroki", "kroki-excalidraw")
REQUIRED_TYPES = {
    "design-tools": {"node-pkg", "gobinary"},
    "kroki": {"jar", "gobinary"},
    "kroki-excalidraw": {"node-pkg"},
    # The 0.0.1 agent image ships native Rust and ONNX assets; it has no
    # Python package inventory. OS package coverage remains mandatory.
    "agent-runtime": set(),
}


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def inventory(root):
    design = json.loads((root / "registry/design-stack.json").read_text())
    images = dict(design["images"])
    agent = json.loads((root / "registry/agent-runtime.json").read_text())
    images.update({"agent-build": agent["build_image"], "agent-base": agent["runtime_image"]})
    if not all(re.fullmatch(r"[^@\s]+@sha256:[0-9a-f]{64}", value) for value in images.values()):
        raise ValueError("all remote images must be digest pinned")
    return images


def validate_export(receipt, alias, expected_source=None):
    if receipt.get("ok") is not True or receipt.get("alias") != alias:
        raise ValueError("export receipt identity mismatch")
    if expected_source is not None and (
        not re.fullmatch(r"[0-9a-f]{64}", expected_source)
        or receipt.get("candidate") is not True
        or receipt.get("source_sha256") != expected_source
    ):
        raise ValueError("candidate export differs from the selected compiled source")
    archive = Path(receipt["archive"]).resolve(strict=True)
    if sha(archive) != receipt["archive_sha256"]:
        raise ValueError("export archive mismatch")
    return archive


def evaluate(report, identity, local=False, required_types=(), advisory_policy="report", architecture="arm64", role="runtime"):
    if advisory_policy not in ("fail", "report"):
        raise ValueError("unknown advisory policy")
    metadata = report.get("Metadata", {})
    if report.get("SchemaVersion") != 2 or report.get("ArtifactType") != "container_image":
        raise ValueError("not a supported container scan")
    if local:
        if metadata.get("ImageID") != identity:
            raise ValueError("scanner image identity differs from verified export")
    elif report.get("ArtifactName") != identity:
        raise ValueError("scanner target differs from pinned reference")
    config = metadata.get("ImageConfig", {})
    if architecture not in ("arm64", "amd64") or config.get("architecture") != architecture or config.get("os") != "linux":
        raise ValueError("image scan platform differs from selected Linux architecture")
    if role not in ("runtime", "build"):
        raise ValueError("invalid artifact role")
    coverage_errors = []
    if not metadata.get("OS", {}).get("Family") or metadata["OS"].get("EOSL") is True:
        coverage_errors.append("OS is missing, unsupported or end of life")
    results = report.get("Results")
    if not isinstance(results, list) or not any(
        item.get("Class") == "os-pkgs" and item.get("Packages") for item in results
    ):
        coverage_errors.append("OS package inventory missing; empty results are not a pass")
    if not isinstance(results, list):
        raise ValueError("scanner result list missing")
    findings = []
    counts = {}
    packages = []
    for item in results:
        kind = item.get("Type", "unknown")
        counts[kind] = counts.get(kind, 0) + len(item.get("Packages") or [])
        for package in item.get("Packages") or []:
            packages.append({k: package.get(k) for k in (
                "ID", "Name", "Version", "SrcName", "SrcVersion", "Arch", "FilePath",
                "Identifier", "Digest", "Licenses", "Layer"
            )} | {"target": item.get("Target"), "type": kind, "class": item.get("Class"),
                  "artifact": identity, "architecture": architecture, "role": role})
        for vulnerability in item.get("Vulnerabilities") or []:
            if not vulnerability.get("VulnerabilityID") or not vulnerability.get("PkgName"):
                raise ValueError("malformed vulnerability")
            findings.append({k: vulnerability.get(k) for k in (
                "VulnerabilityID", "PkgName", "InstalledVersion", "FixedVersion", "Severity",
                "Status", "SeveritySource", "PrimaryURL", "Title", "PkgID", "PkgPath",
                "PkgIdentifier", "Layer", "DataSource", "VendorSeverity", "CVSS", "References"
            )} | {"type": kind, "target": item.get("Target"), "class": item.get("Class"),
                  "artifact": identity, "architecture": architecture, "role": role})
    for kind in sorted(set(required_types)):
        if not counts.get(kind):
            coverage_errors.append(f"required shipped language inventory missing: {kind}")
    return {"ok": not coverage_errors and (advisory_policy == "report" or not findings),
            "advisory_policy": advisory_policy, "advisories_present": bool(findings),
            "inventory_complete": not coverage_errors, "coverage_errors": coverage_errors,
            "packages_by_type": counts, "packages": packages, "findings": findings,
            "architecture": architecture, "role": role,
            "kernel_coverage": "not_measured_image_headers_are_not_the_running_kernel",
            "os": metadata.get("OS"), "image_id": metadata.get("ImageID"),
            "limits": "Detected packages only; an image digest or zero findings cannot prove scanner language coverage or exploitability."}


def run(args):
    args.output.mkdir(parents=True, exist_ok=False)
    scanner = args.scanner.resolve(strict=True)
    before = sha(scanner)
    env = {k: v for k, v in os.environ.items() if not k.startswith("TRIVY_")}
    cache = tempfile.TemporaryDirectory(prefix="mirket-scanner-cache-")
    version_args = [str(scanner), "--cache-dir", cache.name, "version", "--format", "json"]
    version = subprocess.check_output(version_args, env=env, timeout=30)
    (args.output / "scanner-before.json").write_bytes(version)
    exports = json.loads(args.exports.read_text()) if args.exports else {}
    registry_hashes = {str(p.relative_to(ROOT)): sha(p) for p in
                       [ROOT / "registry/design-stack.json", ROOT / "registry/agent-runtime.json"]}
    images = inventory(ROOT)
    full_inventory = list(images) + list(BUILT)
    required = list(images) if args.remote_only else full_inventory
    targets = args.only or required
    if any(name not in required for name in targets) or len(set(targets)) != len(targets):
        raise ValueError("unknown or duplicate image alias")
    records = {}
    for name in targets:
        try:
            local = name in BUILT
            if local:
                receipt = exports[name]
                archive = validate_export(receipt, name, args.candidate_source_sha256)
                identity = receipt["image_id"]
                target = ["--input", str(archive)]
            else:
                identity = images[name]
                target = ["--image-src", "remote", identity]
            report_path = (args.output / (name + ".json")).resolve()
            with tempfile.TemporaryDirectory(prefix="mirket-image-scan-") as clean:
                folder = Path(clean)
                (folder / "trivy.yaml").write_text("{}\n")
                (folder / "ignore").write_text("")
                command = [str(scanner), "--cache-dir", cache.name, "image", "--quiet", "--config", str(folder / "trivy.yaml"),
                           "--ignorefile", str(folder / "ignore"), "--scanners", "vuln",
                           "--vuln-type", "os,library", "--severity", "UNKNOWN,LOW,MEDIUM,HIGH,CRITICAL",
                           "--ignore-unfixed=false", "--list-all-pkgs", "--platform", "linux/" + args.architecture,
                           "--timeout", "10m", "--format", "json", "--output", str(report_path), *target]
                with (args.output / (name + ".log")).open("wb") as log:
                    result = subprocess.run(command, env=env, cwd=clean, stdout=log, stderr=log, timeout=660)
                if result.returncode:
                    raise ValueError("scanner failed; inspect bounded target log")
            records[name] = evaluate(json.loads(report_path.read_text()), identity, local,
                                     REQUIRED_TYPES.get(name, ()), args.advisory_policy, args.architecture,
                                     "build" if not local else "runtime")
            records[name].update(report_sha256=sha(report_path), target=identity)
            if local:
                payloads = payload_inventory(archive, name)
                records[name]["payloads"] = payloads
                records[name]["archive_sha256"] = receipt["archive_sha256"]
                if not payloads["inventory_complete"]:
                    records[name]["coverage_errors"].append("missing executable/native payloads: " + ", ".join(payloads["missing"]))
                    records[name]["inventory_complete"] = False
                    records[name]["ok"] = False
        except (KeyError, ValueError, OSError, subprocess.SubprocessError) as error:
            records[name] = {"ok": False, "error": str(error)}
        print(json.dumps({"image": name, "ok": records[name]["ok"],
                          "findings": len(records[name].get("findings", [])),
                          "error": records[name].get("error")}), flush=True)
    after = subprocess.check_output(version_args, env=env, timeout=30)
    (args.output / "scanner-after.json").write_bytes(after)
    scanner_metadata = json.loads(after)
    database_updated_unix = {key: int(datetime.datetime.fromisoformat(value['UpdatedAt'].replace('Z','+00:00')).timestamp())
                             for key, value in scanner_metadata.items() if key in ('VulnerabilityDB','JavaDB')}
    missing = sorted(set(required) - records.keys())
    registries_unchanged = all(sha(ROOT / name) == digest for name, digest in registry_hashes.items())
    summary = {"schema": 1, "ok": not missing and all(v["ok"] for v in records.values()) and sha(scanner) == before and registries_unchanged,
               "scope": "pinned_remote_images" if args.remote_only else "remote_and_built_images",
               "coverage_complete": not (set(full_inventory) - records.keys()) and all(v.get("inventory_complete") for v in records.values()),
               "unscanned_built_images": sorted(set(BUILT) - records.keys()),
               "missing": missing, "images": records, "scanner_sha256": before,
               "scanner_unchanged": sha(scanner) == before,
               "created_unix": int(time.time()), "scanner_metadata": scanner_metadata,
               "database_updated_unix": database_updated_unix,
               "registry_sha256": registry_hashes, "registries_unchanged": registries_unchanged,
               "candidate_source_sha256": args.candidate_source_sha256,
               "advisory_policy": args.advisory_policy,
               "advisories_present": any(v.get("advisories_present") for v in records.values()),
               "limits": "All findings are retained. Report policy makes advisories non-blocking, not fixed or proven safe. Missing targets/inventory and scanner or identity failures still fail. Detected-package coverage is not complete upstream dependency resolution."}
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    cache.cleanup()
    return 0 if summary["ok"] else 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--architecture", choices=("arm64", "amd64"), default="arm64")
    parser.add_argument("--scanner", type=Path, required=True)
    parser.add_argument("--exports", type=Path, help="Alias to mirket dev image-export receipt map")
    parser.add_argument("--candidate-source-sha256", help="Require candidate exports from this exact compiled source; installed or stale receipts fail")
    parser.add_argument("--output", type=Path, required=True, help="New evidence directory")
    parser.add_argument("--advisory-policy", choices=("fail", "report"), default="report", help="Report keeps advisories as warnings; scan, identity and inventory failures remain fatal")
    parser.add_argument("--only", nargs="+", help="Bounded triage subset; never establishes full coverage")
    parser.add_argument("--remote-only", action="store_true", help="CI scope: pinned bases/services only; built image coverage stays explicitly incomplete")
    raise SystemExit(run(parser.parse_args()))
