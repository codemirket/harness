#!/usr/bin/env python3
"""Actual macOS warm supervisor smoke; Python 3.9+, standard library only.

Run only in the coordinated native-model lane. The fixture applies macOS network
denial to the supervisor subtree; its ps driver stays outside that sandbox.
Default mode closes parent stdin; --idle-expiry waits the real 60s.
No package fetch, build, pressure injection, timeout override or recursive delete.
Evidence contains hashes/shapes/process observations, never input text/vectors.
"""
import argparse
import errno
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import selectors
import stat
import subprocess
import sys
import tempfile
import time
import uuid

FRAME = 2 * 1024 * 1024
STREAM = 8 * 1024 * 1024
STDERR = 64 * 1024
SANDBOX = "/usr/bin/sandbox-exec"
NETWORK_PROFILE = "(version 1) (allow default) (deny network*)"
IDLE_SECONDS = 60
IDLE_WAIT_SECONDS = 90
REPO = Path(__file__).resolve().parent.parent
SOURCE_FILES = (
    "src/embedding_session.rs", "src/embedding_session_pressure.rs",
    "src/embedding_job.rs", "src/embedding.rs", "src/process.rs",
    "src/leases.rs", "Cargo.lock", "packaging/admission/m2-semantic-minilm.runtime.json",
    "tests/warm-embedding-smoke.py",
)


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def digest_file(path):
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def encoded(value, sorted_keys=False):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"),
                      sort_keys=sorted_keys, allow_nan=False).encode("utf-8")


def explicit(value, directory=False, new=False):
    path = Path(value)
    require(path.is_absolute(), "paths must be absolute")
    require(path.parent.resolve() == path.parent, "path parent must be canonical")
    if new:
        require(not os.path.lexists(path), "report destination must be new")
    else:
        require(not path.is_symlink() and path.resolve() == path,
                "input paths must be canonical without symlinks")
        require(path.is_dir() if directory else path.is_file(), "input path type mismatch")
    return path


def expected_identity():
    pins = json.loads((REPO / SOURCE_FILES[7]).read_text())
    artifacts = [{"path": item["path"], "sha256": item["sha256"]}
                 for item in pins["model"]["selected"]
                 if item["path"].startswith(("onnx/", "1_Pooling/"))]
    graph = next(item for item in pins["model"]["selected"]
                 if item["path"] == "onnx/model_qint8_arm64.onnx")
    contract = {"repository": "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
                "model_revision": pins["model"]["revision"],
                "onnx_sha256": graph["sha256"], "artifacts": artifacts,
                "contract": pins["model"]["contract"],
                "runtime_version": pins["runtime_version"],
                "provider": "CPUExecutionProvider", "fastembed": "7.5.0",
                "ort": "2.0.0-rc.15", "content_tokens": 126,
                "overlap_tokens": 16, "single_text": True}
    require(pins["model"]["contract"]["dimensions"] == 384, "unexpected dimension pin")
    material = hashlib.sha256(encoded(contract, True)).hexdigest()
    return {"id": "minilm-l12-v2:" + pins["model"]["revision"] + ":" + graph["sha256"] + ":" + material,
            "model_revision": pins["model"]["revision"], "dimensions": 384}


def ps(arguments):
    result = subprocess.run(["/bin/ps", "-ww"] + arguments, capture_output=True,
                            timeout=3, env={"LC_ALL": "C", "PATH": "/usr/bin:/bin"})
    require(len(result.stdout) <= 1024 * 1024 and len(result.stderr) <= 4096,
            "native process observation exceeded bound")
    require(result.returncode in (0, 1), "native process observation failed")
    return result.stdout.decode("utf-8", "strict").strip()


def cpu_seconds(value):
    """macOS ps time is cumulative user+system CPU, normally minutes:seconds."""
    days = 0
    if "-" in value:
        day, value = value.split("-", 1)
        days = int(day)
    parts = value.split(":")
    require(1 <= len(parts) <= 3, "unexpected native CPU time shape")
    total = 0.0
    for part in parts:
        total = total * 60 + float(part)
    require(math.isfinite(total) and total >= 0, "invalid native CPU time")
    return days * 86400 + total


def leaf_observation(supervisor_pid, binary, leaf_pid=None):
    if leaf_pid is None:
        children = []
        for line in ps(["-axo", "pid=,ppid="]).splitlines():
            pid, parent = map(int, line.split())
            if parent == supervisor_pid:
                children.append(pid)
        require(len(children) == 1, "expected exactly one native supervisor child")
        leaf_pid = children[0]
    observations = {}
    for line in ps(["-p", str(supervisor_pid) + "," + str(leaf_pid), "-o",
                    "pid=,ppid=,rss=,time=,lstart=,comm="]).splitlines():
        fields = line.split(None, 9)
        require(len(fields) == 10, "unexpected native process observation shape")
        require(Path(fields[9]).name == binary.name, "unexpected native executable")
        observations[int(fields[0])] = {
            "pid": int(fields[0]), "parent_pid": int(fields[1]),
            "rss_bytes": int(fields[2]) * 1024,
            "cpu_time_raw": fields[3], "cpu_seconds": cpu_seconds(fields[3]),
            "start_identity": " ".join(fields[4:9]), "command_name": fields[9]}
    require(set(observations) == {supervisor_pid, leaf_pid},
            "native owner or leaf disappeared before stop proof")
    require(observations[leaf_pid]["parent_pid"] == supervisor_pid,
            "native leaf parent identity changed")
    leaf = observations[leaf_pid]
    leaf["supervisor"] = observations[supervisor_pid]
    return leaf


def markers(state, session_id, expected=None):
    found = list(state.glob("heavy-slot-*.unresolved"))
    require(len(found) == 1, "expected one resident heavy ownership marker")
    path = found[0]
    info = path.lstat()
    require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1
            and info.st_uid == os.geteuid() and info.st_mode & 0o077 == 0
            and info.st_size <= 512, "invalid resident ownership marker")
    body = path.read_bytes()
    require(body == ("mirket-unresolved-v1\n" + session_id + "\n").encode(),
            "resident marker session mismatch")
    identity = {"name": path.name, "device": info.st_dev, "inode": info.st_ino,
                "sha256": hashlib.sha256(body).hexdigest()}
    if expected is not None:
        require(identity == expected, "resident marker changed during reuse")
    # Independent descriptor proves the marker accompanies an actually held lock.
    lock_path = state / path.name.replace(".unresolved", ".lock")
    with lock_path.open("rb") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            require(error.errno in (errno.EAGAIN, errno.EACCES), "heavy lock observation failed")
        else:
            fcntl.flock(handle, fcntl.LOCK_UN)
            raise RuntimeError("resident heavy slot unexpectedly unlocked")
    return identity


class Pipes:
    """One bounded reader for stdout and stderr; no background thread survives it."""
    def __init__(self, process):
        self.process = process
        self.selector = selectors.DefaultSelector()
        self.buffer = bytearray()
        self.frames = []
        self.stdout_bytes = 0
        self.stderr_bytes = 0
        self.stderr_hash = hashlib.sha256()
        for stream, name in ((process.stdout, "stdout"), (process.stderr, "stderr")):
            os.set_blocking(stream.fileno(), False)
            self.selector.register(stream, selectors.EVENT_READ, name)

    def pump(self, timeout):
        for key, _ in self.selector.select(timeout):
            block = os.read(key.fileobj.fileno(), 65536)
            if not block:
                self.selector.unregister(key.fileobj)
                continue
            if key.data == "stderr":
                self.stderr_bytes += len(block)
                self.stderr_hash.update(block)
                require(self.stderr_bytes <= STDERR, "supervisor stderr exceeded bound")
                continue
            self.stdout_bytes += len(block)
            require(self.stdout_bytes <= STREAM, "supervisor stdout exceeded bound")
            self.buffer.extend(block)
            while b"\n" in self.buffer:
                end = self.buffer.index(10) + 1
                require(end <= FRAME, "supervisor frame exceeded bound")
                raw = bytes(self.buffer[:end])
                del self.buffer[:end]
                try:
                    message = json.loads(raw.decode("utf-8"),
                                         parse_constant=lambda _: (_ for _ in ()).throw(ValueError()))
                except (UnicodeError, ValueError):
                    raise RuntimeError("supervisor emitted invalid JSON frame") from None
                self.frames.append(message)
                require(len(self.frames) <= 4, "unexpected queued supervisor frames")
            require(len(self.buffer) <= FRAME, "unterminated supervisor frame exceeded bound")

    def next(self, timeout, periodic=None):
        until = time.monotonic() + timeout
        while not self.frames:
            require(time.monotonic() < until, "supervisor response deadline elapsed")
            self.pump(min(1.0, max(0.0, until - time.monotonic())))
            if not self.frames and periodic:
                periodic()
            if not self.selector.get_map() and not self.frames:
                raise RuntimeError("supervisor streams closed before expected frame")
        return self.frames.pop(0)

    def finish(self, timeout):
        until = time.monotonic() + timeout
        while self.selector.get_map():
            require(time.monotonic() < until, "supervisor streams did not close")
            self.pump(min(0.25, max(0.0, until - time.monotonic())))
        require(not self.frames and not self.buffer, "supervisor emitted trailing output")
        return self.process.wait(timeout=max(0.01, until - time.monotonic()))

    def summary(self):
        return {"stdout_bytes": self.stdout_bytes, "stderr_bytes": self.stderr_bytes,
                "stderr_sha256": self.stderr_hash.hexdigest(), "stderr_limit": STDERR}

    def close(self):
        self.selector.close()


def send(process, message):
    data = encoded(message) + b"\n"
    require(len(data) < 4096, "fixture request exceeds bounded atomic pipe write")
    # Three tiny frames in total; no unbounded model input is written here.
    process.stdin.write(data)
    process.stdin.flush()


def checked(message, kind, release_id, session_id):
    require(isinstance(message, dict) and message.get("kind") == kind,
            "unexpected supervisor response kind")
    require(message.get("protocol_version") == 2
            and message.get("release_id") == release_id
            and message.get("session_id") == session_id, "supervisor identity mismatch")


def validate_result(message, sequence, job_id, input_value, identity):
    require(message.get("sequence") == sequence and message.get("job_id") == job_id,
            "inference response correlation mismatch")
    output = message.get("output", {})
    revision = "sha256:" + hashlib.sha256(encoded(input_value)).hexdigest()
    require(set(output) == {"schema", "input_revision", "query", "passages"}
            and output["schema"] == 1 and output["input_revision"] == revision
            and output["passages"] == [], "inference output binding mismatch")
    query = output["query"]
    require(isinstance(query, dict) and set(query) == {
        "model_identity", "source_sha256", "input_bytes", "input_tokens", "vector"},
        "query output shape mismatch")
    source = input_value["query"].encode("utf-8")
    require(query["model_identity"] == identity and query["input_bytes"] == len(source)
            and query["source_sha256"] == hashlib.sha256(source).hexdigest()
            and type(query["input_tokens"]) is int and 0 < query["input_tokens"] <= 512,
            "query model/source binding mismatch")
    vector = query["vector"]
    require(isinstance(vector, list) and len(vector) == 384
            and all(type(v) in (int, float) and math.isfinite(v) for v in vector),
            "query vector shape or finiteness mismatch")
    norm = math.sqrt(sum(v * v for v in vector))
    require(abs(norm - 1.0) < 0.001, "query vector not normalized")
    return {"input_revision": revision, "source_sha256": query["source_sha256"],
            "input_bytes": len(source), "input_tokens": query["input_tokens"],
            "dimensions": len(vector), "norm": norm,
            "output_sha256": hashlib.sha256(encoded(output, True)).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mirket", required=True)
    parser.add_argument("--embedding-package", required=True)
    parser.add_argument("--work-root", help="absolute existing task-owned temp parent")
    parser.add_argument("--report", help="absolute new JSON report (default: task temp/evidence.json)")
    parser.add_argument("--idle-expiry", action="store_true", help="wait real 60s instead of closing stdin")
    args = parser.parse_args()
    require(sys.platform == "darwin", "this native process fixture is qualified only for macOS")
    binary = explicit(args.mirket)
    package = explicit(args.embedding_package, directory=True)
    parent = explicit(args.work_root, directory=True) if args.work_root else Path(tempfile.gettempdir()).resolve()
    report_path = explicit(args.report, new=True) if args.report else None
    root = Path(tempfile.mkdtemp(prefix="mirket-warm-embedding-", dir=str(parent))).resolve()
    os.chmod(root, 0o700)
    report_path = report_path or root / "evidence.json"
    state = root / "state"
    state.mkdir(mode=0o700)
    jobs = state / "jobs"
    jobs.mkdir(mode=0o700)
    session_id = "warm-smoke-" + uuid.uuid4().hex
    release_id = "warm-smoke-release"
    scratch = jobs / session_id
    scratch.mkdir(mode=0o700)
    scratch_stamp = (scratch.stat().st_dev, scratch.stat().st_ino)
    supervisor_argv = [SANDBOX, "-p", NETWORK_PROFILE, str(binary), "__embedding_supervisor"]
    evidence = {"schema": 1, "status": "running", "mode": "idle-expiry" if args.idle_expiry else "stdin-close",
                "platform": platform.platform(), "architecture": platform.machine(),
                "python": platform.python_version(), "binary": str(binary),
                "binary_sha256": digest_file(binary), "embedding_package": str(package),
                "source_sha256": {name: digest_file(REPO / name) for name in SOURCE_FILES},
                "task_root": str(root), "state_root": str(state), "scratch": str(scratch),
                "model_identity": expected_identity(), "requests": [], "process_samples": [],
                "network_isolation": {"argv": supervisor_argv, "profile": NETWORK_PROFILE,
                    "scope": "supervisor and inherited model leaf; fixture and ps driver outside",
                    "os_network_denied": False},
                "ps_method": "/bin/ps -p supervisor,leaf -o pid=,ppid=,rss=,time=,lstart=,comm=; time is cumulative user+system CPU, RSS KiB converted to bytes",
                "limits": {"idle_seconds": IDLE_SECONDS, "idle_wait_seconds": IDLE_WAIT_SECONDS, "request_seconds": 60},
                "limitations": ["OS network denial records the successful fixed sandbox invocation; no packet capture or egress probe is performed.",
                    "RSS and CPU are selected process samples; CPU deltas cover the stated observed interval, not post-exit accounting, peak RSS or a performance acceptance gate.",
                    "Observed source hashes do not independently attest which sources built the executable.",
                    "Only this supervisor and one actual model leaf are qualified; no Windows claim."]}
    process = None
    pipes = None
    started = time.monotonic()
    try:
        require(Path(SANDBOX).is_file() and os.access(SANDBOX, os.X_OK),
                "required macOS sandbox executable unavailable")
        env = {"ORT_DISABLE_TELEMETRY": "1", "TOKENIZERS_PARALLELISM": "false",
               "TMPDIR": str(scratch), "TMP": str(scratch), "TEMP": str(scratch)}
        process = subprocess.Popen(supervisor_argv, cwd=str(scratch),
                                   env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, bufsize=0, start_new_session=True)
        pipes = Pipes(process)
        evidence["supervisor_pid"] = process.pid
        send(process, {"protocol_version": 2, "state_root": str(state), "release_id": release_id,
                       "session_id": session_id, "package_root": str(package),
                       "scratch_dir": str(scratch), "remaining_ms": 60000})
        checked(pipes.next(65), "ready", release_id, session_id)
        # Ready proves sandbox-exec accepted the fixed policy and exec'd the
        # supervisor. Policy/exec failures reach EOF or fail Popen; never retry
        # without this wrapper. The model leaf inherits the process sandbox.
        evidence["network_isolation"]["os_network_denied"] = True
        evidence["cold_ready_ms"] = (time.monotonic() - started) * 1000
        leaf = leaf_observation(process.pid, binary)
        evidence["process_samples"].append({"phase": "ready", **leaf})
        marker = markers(state, session_id)
        evidence["resident_marker"] = marker
        queries = ("Which unit measures the timeout?", "Zaman aşımı hangi birimle ölçülür?")
        for sequence, text in enumerate(queries, 1):
            input_value = {"query": text, "passages": []}
            revision = "sha256:" + hashlib.sha256(encoded(input_value)).hexdigest()
            job_id = session_id + "-request-" + str(sequence)
            before = time.monotonic()
            send(process, {"protocol_version": 2, "release_id": release_id,
                           "session_id": session_id, "remaining_ms": 60000,
                           "request": {"sequence": sequence, "job_id": job_id,
                                       "input_revision": revision, "input": input_value}})
            message = pipes.next(65)
            elapsed = (time.monotonic() - before) * 1000
            checked(message, "result", release_id, session_id)
            summary = validate_result(message, sequence, job_id, input_value, evidence["model_identity"])
            evidence["requests"].append({"sequence": sequence, "warm_request_ms": elapsed, **summary})
            observed = leaf_observation(process.pid, binary, leaf["pid"])
            require(observed["start_identity"] == leaf["start_identity"], "leaf was replaced during reuse")
            evidence["process_samples"].append({"phase": "result-" + str(sequence), **observed})
            markers(state, session_id, marker)
        stop_started = time.monotonic()
        if args.idle_expiry:
            print(f"Two inferences complete; waiting for real {IDLE_SECONDS}s idle expiry.", flush=True)
            idle_start = leaf_observation(process.pid, binary, leaf["pid"])
            evidence["process_samples"].append({"phase": "idle-start", "idle_seconds": 0, **idle_start})
            last_sample = [stop_started]
            def observe_idle():
                now = time.monotonic()
                # Finish live sampling before expiry; the typed frame and EOF
                # prove retirement. CPU deltas honestly end at the last live sample.
                if now - stop_started >= IDLE_SECONDS - 2:
                    return
                if now - last_sample[0] >= (1 if now - stop_started >= IDLE_SECONDS - 5 else 30):
                    observed = leaf_observation(process.pid, binary, leaf["pid"])
                    require(observed["start_identity"] == leaf["start_identity"]
                            and observed["supervisor"]["start_identity"] == idle_start["supervisor"]["start_identity"],
                            "idle native process identity changed")
                    markers(state, session_id, marker)
                    evidence["process_samples"].append({"phase": "idle", "idle_seconds": now - stop_started, **observed})
                    last_sample[0] = now
                    if now - stop_started < IDLE_SECONDS - 5:
                        print("Idle ownership retained: %ds." % (now - stop_started), flush=True)
            stopped = pipes.next(IDLE_WAIT_SECONDS, observe_idle)
            samples = [sample for sample in evidence["process_samples"] if sample["phase"] == "idle"]
            if samples:
                end = samples[-1]
                evidence["idle_cpu_observation"] = {
                    "observed_wall_seconds": end["idle_seconds"],
                    "leaf_cpu_seconds_delta": end["cpu_seconds"] - idle_start["cpu_seconds"],
                    "supervisor_cpu_seconds_delta": end["supervisor"]["cpu_seconds"] - idle_start["supervisor"]["cpu_seconds"],
                    "leaf_start_identity": idle_start["start_identity"],
                    "supervisor_start_identity": idle_start["supervisor"]["start_identity"],
                    "note": "Last live ps sample before retirement; no CPU acceptance threshold."}
        else:
            process.stdin.close()
            stopped = pipes.next(20)
        elapsed = time.monotonic() - stop_started
        checked(stopped, "stopped", release_id, session_id)
        report = stopped.get("report", {})
        expected_reason = "idle_expired" if args.idle_expiry else "parent_gone"
        require(report.get("reason") == expected_reason and report.get("cleanup_confirmed") is True,
                "supervisor did not confirm expected cleanup")
        if args.idle_expiry:
            require(IDLE_SECONDS - 1 <= elapsed <= IDLE_WAIT_SECONDS,
                    "idle expiry did not use the configured deadline")
            process.stdin.close()
        require(pipes.finish(10) == 0, "supervisor did not exit successfully")
        require(not ps(["-p", str(leaf["pid"]), "-o", "pid="]), "native leaf still exists after stop proof")
        require(not list(state.glob("heavy-slot-*.unresolved")), "heavy ownership marker survived cleanup")
        require((scratch.stat().st_dev, scratch.stat().st_ino) == scratch_stamp,
                "owned scratch identity changed")
        scratch.rmdir()  # Only empty fixture-owned scratch after actual stop proof.
        evidence["stop"] = {"reason": expected_reason, "cleanup_confirmed": True,
                            "wait_ms": elapsed * 1000, "exit_code": 0,
                            "leaf_gone": True, "markers_gone": True, "empty_scratch_removed": True}
        require(digest_file(binary) == evidence["binary_sha256"], "binary changed during fixture")
        evidence["status"] = "passed"
    except Exception as error:
        # Never emit child output, query text or vectors in failure diagnostics.
        evidence["status"] = "failed"
        evidence["failure"] = str(error) if isinstance(error, RuntimeError) else type(error).__name__
        if process is not None:
            if process.stdin and not process.stdin.closed:
                process.stdin.close()
            try:
                process.wait(timeout=20)
                evidence["failure_cleanup"] = {"supervisor_exit": process.returncode,
                    "note": "Exit alone is not typed leaf cleanup proof; state retained."}
            except subprocess.TimeoutExpired:
                # Popen retains this direct child identity. Do not signal a leaf
                # numeric PID or reclaim its marker after losing ownership proof.
                process.kill()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    pass
                evidence["failure_cleanup"] = {"supervisor_killed": True,
                    "cleanup_confirmed": False, "note": "Leaf ownership may remain; inspect retained private state."}
    finally:
        evidence["elapsed_ms"] = (time.monotonic() - started) * 1000
        if pipes:
            evidence["streams"] = pipes.summary()
            pipes.close()
        descriptor = os.open(str(report_path), os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(descriptor, "w") as handle:
            json.dump(evidence, handle, indent=2, ensure_ascii=False, allow_nan=False)
            handle.write("\n")
        print(json.dumps({"status": evidence["status"], "report": str(report_path)}, separators=(",", ":")), flush=True)
    return 0 if evidence["status"] == "passed" else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print("Fixture setup failed: " + type(error).__name__, file=sys.stderr)
        sys.exit(2)
