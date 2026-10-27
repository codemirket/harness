"""Actual disposable setup/uninstall/reinstall with canonical MCP recall.

Supply explicit current core, an existing admitted base release, and an external
evidence parent. Original inputs and the real user installation are untouched.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import tempfile
import time


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            value.update(block)
    return value.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mirket", type=Path, required=True)
    parser.add_argument("--base-release", type=Path, required=True)
    parser.add_argument("--evidence-root", type=Path, required=True)
    args = parser.parse_args()
    for path in vars(args).values():
        assert path.is_absolute() and path.resolve() == path, path
    assert args.mirket.is_file() and args.base_release.is_dir() and args.evidence_root.is_dir()
    repo = Path(__file__).resolve().parents[1]
    evidence = Path(tempfile.mkdtemp(prefix="same-home-reinstall-", dir=args.evidence_root))
    evidence.chmod(0o700)
    source = evidence / "source"
    source.mkdir(mode=0o700)
    pins = {item["relative_path"]: item for item in json.loads((args.base_release / "manifest.json").read_text())["files"]}
    for relative in ["runtime/node/bin/node", "runtime/node/LICENSE"]:
        old = args.base_release / relative
        pin = pins[relative]
        assert old.is_file() and not old.is_symlink() and old.stat().st_size == pin["byte_len"] and digest(old) == pin["sha256"]
        new = source / relative
        new.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(old, new)
        new.chmod(pin["mode"])
    binary = source / "bin/mirket"
    binary.parent.mkdir()
    shutil.copyfile(args.mirket, binary)
    binary.chmod(0o700)
    events = []

    def run(argv, name, expected=0):
        began = time.monotonic()
        result = subprocess.run([str(value) for value in argv], cwd=repo, text=True, capture_output=True, timeout=180)
        record = dict(argv=[str(value) for value in argv], code=result.returncode,
                      seconds=time.monotonic() - began, stdout=result.stdout, stderr=result.stderr)
        (evidence / (name + ".json")).write_text(json.dumps(record, ensure_ascii=False, indent=2))
        assert result.returncode == expected, record
        return json.loads(result.stdout) if result.stdout.strip().startswith("{") else result.stdout

    class Client:
        def __init__(self, home):
            self.process = subprocess.Popen([str(home / "bin/mirket"), "serve", "--home", str(home)],
                                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                            text=True, bufsize=1)
            self.serial = 0
            self.rpc("initialize", dict(protocolVersion="2025-11-25", capabilities={},
                                         clientInfo=dict(name="same-home-reinstall-smoke", version="1")))
            self.process.stdin.write(json.dumps(dict(jsonrpc="2.0", method="notifications/initialized", params={})) + "\n")
            self.process.stdin.flush()

        def rpc(self, method, params):
            self.serial += 1
            request = dict(jsonrpc="2.0", id=self.serial, method=method, params=params)
            self.process.stdin.write(json.dumps(request, ensure_ascii=False) + "\n")
            self.process.stdin.flush()
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                readable, _, _ = select.select([self.process.stdout], [], [], max(0, deadline - time.monotonic()))
                assert readable, "MCP deadline"
                line = self.process.stdout.readline()
                assert line, "MCP EOF"
                response = json.loads(line)
                events.append(dict(request=request, response=response))
                (evidence / "mcp-events.json").write_text(json.dumps(events, ensure_ascii=False, indent=2))
                if response.get("id") == self.serial:
                    assert "error" not in response, response
                    return response["result"]
            raise AssertionError("MCP deadline")

        def tool(self, name, arguments, error=False):
            result = self.rpc("tools/call", dict(name=name, arguments=arguments))
            assert bool(result.get("isError")) == error, result
            if error:
                return result
            return result.get("structuredContent") or json.loads(next(part["text"] for part in result["content"] if part["type"] == "text"))

        def close(self):
            self.process.stdin.close()
            code = self.process.wait(timeout=40)
            assert code == 0, self.process.stderr.read()

    run(["python3", repo / "scripts/assemble-skills.py", "--output-release", source], "assemble-skills")
    manifest = run([binary, "dev", "manifest", "--directory", source], "manifest")
    user = evidence / "user-home"
    user.mkdir(mode=0o700)
    (user / ".codex").mkdir(mode=0o700)
    (user / ".codex/config.toml").write_text("# Preserve this user choice.\nmodel = 'chosen-model'\n")
    (user / ".codex/config.toml").chmod(0o600)
    (user / ".codex/AGENTS.md").write_text("Preserve this personal instruction.\n")
    (user / ".codex/AGENTS.md").chmod(0o600)
    personal = user / ".agents/skills/personal-method"
    personal.mkdir(parents=True)
    (personal / "SKILL.md").write_text("Preserve this personal skill.\n")
    (personal / "SKILL.md").chmod(0o600)
    home = user / ".mirket"
    run([binary, "setup", "--home", home, "--source", source, "--user-home", user], "setup")
    project = evidence / "project İ"
    project.mkdir(mode=0o700)
    client = Client(home)
    release = client.tool("runtime_status", {})["release_id"]
    common = dict(expected_release_id=release, scope=dict(kind="project", project_root=str(project)))

    def save(content):
        record = dict(kind="fact", basis="observed", content=content,
                      sources=[dict(source_id="fixture:reinstall", revision="source-revision-1",
                                    locator="fixture://reinstall/record", captured_at=123)],
                      status="current", critical=False,
                      applicability=dict(task_key=None, checkout_id=None, operation=None), valid_from=None, valid_until=None)
        prepared = client.tool("memory_prepare", dict(**common, mutation=dict(kind="create", record=record, supersedes=None)))
        return client.tool("memory_commit", dict(**common, operation_id=prepared["operation_id"]))["record"]

    retained = save("The preserved project uses a green repair-workshop palette.")
    forgotten = save("This private fixture fact must remain forgotten after reinstall.")
    (evidence / "memory-records.json").write_text(json.dumps(dict(retained=retained, forgotten=forgotten), ensure_ascii=False, indent=2))
    prepared = client.tool("memory_forget_prepare", dict(**common, record_id=forgotten["record_id"], expected_revision=forgotten["revision"]))
    assert client.tool("memory_forget_commit", dict(**common, operation_id=prepared["operation_id"]))["status"] == "forgotten"
    client.close()
    before = {str(path.relative_to(home)): digest(path)
              for name in ["state", "backups"] for path in (home / name).rglob("*")
              if path.is_file() and path.name not in ["codex-config.json", "codex-guidance.json", "codex-skills.json"]}
    assert any(name.startswith("backups/") for name in before), "Actual canonical writes must create managed backup evidence"
    run([home / "bin/mirket", "uninstall", "--home", home, "--user-home", user], "self-uninstall")
    assert not (home / "bin/mirket").exists()
    result = run([binary, "reinstall", "--home", home, "--user-home", user, "--source", source], "reinstall")
    assert result["status"] == "reinstalled"
    assert all((home / name).is_file() and digest(home / name) == value for name, value in before.items())
    assert run([home / "bin/mirket", "doctor", "--home", home, "--user-home", user, "--json"], "doctor")["status"] == "healthy"
    assert run([binary, "reinstall-status", "--home", home, "--user-home", user], "status")["status"] == "reinstalled"
    assert run([binary, "reinstall-reconcile", "--home", home, "--user-home", user, "--source", source], "completed-reconcile")["status"] == "reinstalled"
    client = Client(home)
    assert client.tool("runtime_status", {})["release_id"] == release
    recalled = client.tool("memory_read", dict(**common, record_id=retained["record_id"], revision=None))
    assert recalled["record"]["data"]["content"] == retained["data"]["content"]
    unavailable = client.tool("memory_read", dict(**common, record_id=forgotten["record_id"], revision=None), error=True)
    assert any(word in json.dumps(unavailable).lower() for word in ["forgot", "found", "excluded"])
    client.close()
    assert (personal / "SKILL.md").read_text() == "Preserve this personal skill.\n"
    assert (user / ".codex/AGENTS.md").read_text().startswith("Preserve this personal instruction.\n")
    assert "model = 'chosen-model'" in (user / ".codex/config.toml").read_text()
    (evidence / "mcp-events.json").write_text(json.dumps(events, ensure_ascii=False, indent=2))
    (evidence / "preserved-file-hashes.json").write_text(json.dumps(before, indent=2))
    summary = dict(status="passed", platform=os.uname().sysname, machine=os.uname().machine,
                   release_id=release, manifest=manifest, core_sha256=digest(binary),
                   source_manifest_sha256=digest(source / "manifest.json"),
                   retained_state_backup_files=len(before), actual_canonical_recall=True,
                   forgetting_preserved=True, completed_reconcile_without_replay=True,
                   user_installation_untouched=True, evidence=str(evidence))
    (evidence / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2))
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
