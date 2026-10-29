#!/usr/bin/env python3
"""Supervise the disposable installed Pi session fixture as one owned process group."""

import argparse
import json
import os
from pathlib import Path
import re
import selectors
import shutil
import subprocess
import sys
import tempfile
import time

from pi_process_group import terminate_owned_group

MAX_EVIDENCE = 64 * 1024 * 1024
MAX_LOG = 4 * 1024 * 1024
MAX_RECEIPTS = MAX_EVIDENCE - MAX_LOG - 1024 * 1024


def run_group(command, evidence_dir, *, timeout_seconds, markers, env=None, cwd=None,
              max_log_bytes=MAX_LOG):
    """Return the observed cause and cleanup, including leader-exited survivors."""
    if not 0 < timeout_seconds <= 900 or not 0 < max_log_bytes <= MAX_LOG:
        raise ValueError("Fixture limits exceed the admitted 15-minute/4-MiB log bounds")
    evidence_dir = Path(evidence_dir)
    evidence_dir.mkdir(mode=0o700, parents=True, exist_ok=False)
    start = time.monotonic()
    captured = 0
    reason = None
    truncated = False
    with (evidence_dir / "process.log").open("xb") as log:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, start_new_session=True,
                                   stdin=subprocess.DEVNULL)
        selector = selectors.DefaultSelector()
        selector.register(process.stdout, selectors.EVENT_READ)
        try:
            while reason is None:
                if time.monotonic() - start >= timeout_seconds:
                    reason = "timeout"
                    break
                # A child may inherit stdout after its leader exits. Reap the
                # owned group without waiting indefinitely for that pipe's EOF.
                if process.poll() is not None:
                    reason = "leader_exit"
                    break
                for key, _ in selector.select(0.1):
                    chunk = os.read(key.fd, min(65536, max_log_bytes - captured + 1))
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    available = max_log_bytes - captured
                    log.write(chunk[:available])
                    log.flush()
                    captured += min(len(chunk), available)
                    if len(chunk) > available:
                        reason = "log_budget"
                        break
        finally:
            cleanup = terminate_owned_group(process, markers)
            selector.close()
            if not cleanup["after"] and not cleanup["refused_unowned_group"]:
                os.set_blocking(process.stdout.fileno(), False)
                while True:
                    try:
                        chunk = os.read(process.stdout.fileno(), 65536)
                    except BlockingIOError:
                        break
                    if not chunk:
                        break
                    available = max_log_bytes - captured
                    log.write(chunk[:available])
                    captured += min(len(chunk), available)
                    truncated |= len(chunk) > available
            process.stdout.close()
        # Drain already buffered tail only when the group is empty. The log is
        # bounded and the recorded cause is never replaced by cleanup status.
        report = {
            "command": list(map(str, command)), "cause": reason,
            "exitCode": process.poll(), "elapsedMs": round((time.monotonic() - start) * 1000),
            "logBytes": captured, "logTruncated": truncated, "cleanup": cleanup,
            "passed": reason == "leader_exit" and process.returncode == 0
                      and not truncated and not cleanup["before"] and not cleanup["after"]
                      and not cleanup["refused_unowned_group"],
        }
        (evidence_dir / "supervisor.json").write_text(json.dumps(report, indent=2) + "\n")
        return report


def archive_fixture(root, evidence_dir):
    """Copy bounded receipts only; never treat an installed payload as evidence."""
    root = Path(root)
    if not re.fullmatch(r"/private/tmp/mirket-pi-package-[A-Za-z0-9_-]+", str(root)):
        raise ValueError("Unexpected fixture root")
    if root.resolve() != root or not root.is_dir():
        raise ValueError("Fixture root is missing or has symlink ancestry")
    source_files = [root / name for name in (
        "session-cases.json", "read-before.json", "read-after-reload.json",
        "pending-before.json", "pending-after.json")]
    source_files.extend((root / "agent/mirket/state").rglob("*.sqlite3"))
    source_files.extend((root / "agent/mirket/state/operations").glob("op-*.json"))
    source_files.extend([root / "agent/mirket/authorizations.json",
                         root / "agent/settings.json"])
    total = 0
    archived = []
    for source in source_files:
        if not source.is_file() or source.is_symlink():
            continue
        size = source.stat().st_size
        total += size
        if total > MAX_RECEIPTS:
            raise ValueError("Receipt evidence exceeds 59 MiB")
        destination = evidence_dir / "fixture" / source.relative_to(root)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        archived.append(str(destination.relative_to(evidence_dir)))
    return {"sourceRoot": str(root), "bytes": total, "files": archived}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("node", "peers", "binary", "license", "evidence-dir"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    for path in (args.node, args.peers, args.binary, args.license, args.evidence_dir):
        if not path.is_absolute():
            parser.error("All paths must be absolute")
    script = Path(__file__).with_name("pi-session-authorization.mjs").resolve()
    with tempfile.TemporaryDirectory(prefix="mirket-pi-package-", dir="/private/tmp") as prelaunch:
        prelaunch = Path(prelaunch)
        for name in ("home", "agent", "sessions", "project"):
            (prelaunch / name).mkdir(mode=0o700)
        env = {key: value for key, value in os.environ.items()
               if not key.startswith(("PI_", "MIRKET_", "NODE_", "OPENAI_", "ANTHROPIC_"))}
        env.update({"HOME": str(prelaunch / "home"),
                    "PI_CODING_AGENT_DIR": str(prelaunch / "agent"),
                    "PI_CODING_AGENT_SESSION_DIR": str(prelaunch / "sessions"),
                    "PI_SKIP_UPDATE_CHECK": "1", "PI_TELEMETRY_DISABLED": "1",
                    "MIRKET_SESSION_SUPERVISED": "1"})
        command = [str(args.node), str(script), str(args.peers), str(args.binary),
                   str(args.node), str(args.license)]
        report = run_group(command, args.evidence_dir, timeout_seconds=900,
                           markers=[str(script), "/private/tmp/mirket-pi-package-"],
                           env=env, cwd=script.parents[1])
    output = (args.evidence_dir / "process.log").read_text(errors="replace")
    match = re.search(r"(?m)^Fixture: (/private/tmp/mirket-pi-package-[A-Za-z0-9_-]+)$", output)
    if match and not report["cleanup"]["after"]:
        try:
            report["archive"] = archive_fixture(match.group(1), args.evidence_dir)
        except Exception as error:
            report["archiveError"] = str(error)
    if report["passed"] and "archive" in report:
        try:
            saved = args.evidence_dir / "fixture"
            fixture_report = json.loads((saved / "session-cases.json").read_text())
            expected = ["fresh private package installed",
                        "exact unexpired shared read grant survives actual Pi reload and rebind",
                        "pending TUI confirmation blocked after actual Pi shutdown and reload"]
            if [item["label"] for item in fixture_report["results"]] != expected \
                    or not fixture_report.get("finishedAt") or fixture_report.get("errors"):
                raise ValueError("Incomplete fixture report")
            for before, after in (("read-before.json", "read-after-reload.json"),
                                  ("pending-before.json", "pending-after.json")):
                if (saved / before).read_bytes() != (saved / after).read_bytes():
                    raise ValueError("Archived native state snapshots differ")
            report["fixtureChecks"] = expected
        except Exception as error:
            report["verificationError"] = str(error)
    report["passed"] = report["passed"] and "archive" in report and "fixtureChecks" in report
    (args.evidence_dir / "supervisor.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"passed": report["passed"], "cause": report["cause"],
                      "exitCode": report["exitCode"], "cleanup": report["cleanup"],
                      "evidence": str(args.evidence_dir)}))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
