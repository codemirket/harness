"""Bounded private stdio protocol. No arbitrary graphs, imports, paths or code."""
import contextlib
import fcntl
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import re
import sys
import sqlite3
import time

os.umask(0o077)
os.environ["MEM0_TELEMETRY"] = "false"
os.environ["LANGSMITH_TRACING"] = "false"
os.environ["LANGCHAIN_TRACING_V2"] = "false"
os.environ["HF_HUB_OFFLINE"] = "1"
os.environ["MEM0_DIR"] = "/tmp/mem0"
ROOT = Path(os.environ.get("MIRKET_AGENT_DATA", "/data"))
LIMIT = 4 * 1024 * 1024


def identity():
    files = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(Path(__file__).parent.iterdir())
             if p.is_file() and (p.name in ("bridge.py", "memory.py", "workflow.py", "lifecycle.py") or p.name in ("requirements.lock", "model.json", "bm25.json", "recovery-contract.json"))}
    return {"sha256": hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest(), "files": files,
            "versions": {name: importlib.metadata.version(name) for name in ("mem0ai", "langgraph", "langgraph-checkpoint-sqlite", "fastembed")},
            "network": "runtime container has no network", "memory": "explicit facts; project scope; local embeddings"}


IDENTITY = identity()


def audit():
    components = []
    total = 0
    for dist in sorted(importlib.metadata.distributions(), key=lambda d: d.metadata["Name"].lower()):
        size = sum(p.size or 0 for p in (dist.files or []))
        total += size
        name, version = dist.metadata["Name"], dist.version
        components.append({"type": "library", "name": name, "version": version,
                           "purl": f"pkg:pypi/{name.lower().replace('_', '-')}@{version}",
                           "properties": [{"name": "mirket:recorded-file-bytes", "value": str(size)},
                                          {"name": "mirket:license-metadata", "value": (dist.metadata.get("License-Expression") or dist.metadata.get("License") or "unknown")[:2048]}]})
    return {"sbom": {"bomFormat": "CycloneDX", "specVersion": "1.6", "version": 1, "components": components},
            "distributions": len(components), "recorded_file_bytes": total,
            "coverage": "Python distribution RECORD metadata only; excludes base OS, models, private data and runtime RSS. License strings are upstream metadata. No vulnerability scan performed."}


@contextlib.contextmanager
def scope_directory(scope, reclaim=False):
    if not isinstance(scope, str) or not re.fullmatch("[a-f0-9]{64}", scope):
        raise ValueError("invalid project scope")
    directory = ROOT / scope
    if ROOT.is_symlink() or directory.is_symlink():
        raise ValueError("refusing symlink state")
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    descriptor = os.open(directory / ".lock", os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    try:
        # Bursts across desktop clients queue behind real embedding/inventory
        # work. Reserve half of Rust's 120-second response budget for that work
        # after admission; ten seconds rejected healthy eight-request bursts.
        deadline = time.monotonic() + 60
        while True:
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    raise ValueError("project runtime is busy; inspect before retrying")
                time.sleep(0.05)
        files = list(directory.rglob("*"))
        if any(p.is_symlink() for p in files):
            raise ValueError("refusing symlink state")
        if not reclaim and sum(p.stat().st_size for p in files if p.is_file()) > 512 * 1024 * 1024:
            raise ValueError("project runtime exceeds its 512 MiB storage budget")
        yield directory
    finally:
        fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def dispatch(request):
    if set(request) - {"operation", "scope", "arguments"}:
        raise ValueError("unknown runtime request fields")
    operation = request["operation"]
    if operation == "identity":
        return IDENTITY
    if operation == "audit":
        return audit()
    args = request["arguments"]
    if not isinstance(args, dict):
        raise ValueError("arguments must be an object")
    allowed = {
        "data_export": {"cursor", "page_bytes"},
        "data_plan": {"policy", "cursor"},
        "data_apply": {"plan", "expected_sha256"},
        "memory_read": {"query", "id", "limit", "cursor"},
        "memory_put": {"text", "kind", "provenance", "expires_at", "key", "supersedes"},
        "memory_forget": {"id", "expected_sha256"},
        "worker_recall": {"request"},
        "workflow_start": {"run_id", "envelope", "snapshot", "deadline"},
        "workflow_status": {"run_id"},
        "workflow_forget": {"run_id", "expected_checkpoint_id"},
        "workflow_hostcheck": {"run_id", "expected_checkpoint_id", "verification"},
        "workflow_resume": {"run_id", "expected_checkpoint_id", "key", "reply"},
    }
    if operation not in allowed or set(args) - allowed[operation]:
        raise ValueError("unknown operation or arguments")
    if "run_id" in args and not re.fullmatch(r"routed-[A-Za-z0-9_-]{1,56}", args["run_id"]):
        raise ValueError("invalid run ID")
    with scope_directory(request["scope"], operation.startswith("data_") or operation in ("memory_forget", "workflow_forget", "workflow_status")) as directory:
        if operation in {"memory_put", "memory_forget", "workflow_start", "workflow_resume", "workflow_hostcheck", "workflow_forget", "data_apply"}:
            from lifecycle import revision
            with sqlite3.connect(directory / "workflows.sqlite", timeout=10) as connection:
                revision(connection, advance=True)
        if operation.startswith("data_"):
            from lifecycle import execute
            return execute(directory, request["scope"], operation, args)
        if operation.startswith("memory_"):
            from memory import read, put, forget
            return {"memory_read": read, "memory_put": put, "memory_forget": forget}[operation](directory, request["scope"], args)
        if operation == "worker_recall":
            from workflow import recall_node
            return {"state": recall_node({"envelope": args}, directory, request["scope"])}
        from workflow import execute
        return execute(directory, request["scope"], operation, args)


def main():
    while True:
        line = sys.stdin.buffer.readline(LIMIT + 1)
        if not line:
            return
        if len(line) > LIMIT or not line.endswith(b"\n"):
            raise ValueError("runtime input exceeds bound")
        try:
            with contextlib.redirect_stdout(sys.stderr):
                result = dispatch(json.loads(line))
            response = {"result": result, "runtime": IDENTITY}
        except Exception as error:
            response = {"error": str(error)[:2048], "runtime": IDENTITY}
        encoded = json.dumps(response, ensure_ascii=False).encode()
        if len(encoded) > LIMIT:
            encoded = b'{"error":"runtime response exceeds bound; inspect saved state"}'
        sys.stdout.buffer.write(encoded + b"\n")
        sys.stdout.buffer.flush()


if __name__ == "__main__":
    main()
