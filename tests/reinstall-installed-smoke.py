"""Disposable setup/reinstall, installed skill resources and canonical MCP recall.

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
        if expected != 0:
            return record
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
    catalog = json.loads((source / "skills/catalog.json").read_text())
    selected = [entry for entry in catalog["project"]
                if entry.get("resources") and not entry["required_capabilities"]]
    assert "browser-research" in {entry["id"] for entry in selected}
    project_personal = project / ".agents/skills/project-personal/SKILL.md"
    project_personal.parent.mkdir(parents=True)
    project_personal.write_text("Keep this project-owned method.\n")

    def snapshot(directory):
        return {str(path.relative_to(directory)): digest(path)
                for path in directory.rglob("*") if path.is_file()}

    def skill_command(operation, *extra, target=project):
        return [home / "bin/mirket", "skills", operation, "--home", home,
                "--project-root", target, *extra]

    def verify_copies(entries, destination):
        count = 0
        for entry in entries:
            folder = destination / entry["id"]
            expected = {"SKILL.md", *(item["path"] for item in entry["resources"])}
            assert {str(p.relative_to(folder)) for p in folder.rglob("*") if p.is_file()} == expected
            for relative in expected:
                assert (folder / relative).read_bytes() == (source / "skills" / entry["id"] / relative).read_bytes()
            count += len(entry["resources"])
        return count

    # Retired product guidance must refuse selection without project writes.
    retired = ("customer-research", "product-strategy", "positioning",
               "marketing-copy", "conversion", "experiments-launch")
    before_retired = snapshot(project)
    for skill_id in retired:
        refused = run(skill_command("project-select", "--include", skill_id),
                      "retired-" + skill_id, expected=1)
        assert "unknown project skill" in refused["stderr"], refused
        assert snapshot(project) == before_retired
    refused = run(skill_command("project-select", "--profile", "content-growth"),
                  "retired-content-growth", expected=1)
    assert "unknown skill profile" in refused["stderr"], refused
    assert snapshot(project) == before_retired

    # Selection must deliver real optional references, not merely a receipt.
    options = [value for entry in selected for value in ("--include", entry["id"])]
    run(skill_command("project-select", *options), "project-select")
    resource_count = verify_copies(selected, project / ".agents/skills")
    assert verify_copies(catalog["global"], user / ".agents/skills") == 0
    project_before = snapshot(project)
    assert run(skill_command("project-sync"), "project-idempotent-sync")["status"] == "unchanged"
    assert snapshot(project) == project_before

    edited = project / ".agents/skills/browser-research/references/research-coverage.md"
    original = edited.read_bytes()
    edited.write_bytes(original + b"\nUser-owned research note.\n")
    drifted = snapshot(project)
    assert run(skill_command("project-doctor"), "project-drift-doctor")["status"] == "drifted"
    refused = run(skill_command("project-sync"), "project-refuse-edited-reference", expected=1)
    assert "managed skill copy has drifted" in refused["stderr"]
    assert snapshot(project) == drifted
    refused = run(skill_command("project-remove"), "project-refuse-edited-removal", expected=1)
    assert "managed skill copy has drifted" in refused["stderr"]
    assert snapshot(project) == drifted
    edited.write_bytes(original)  # Restore only this fixture's deliberate edit.
    assert run(skill_command("project-sync"), "project-restored-sync")["status"] == "unchanged"

    unavailable_project = evidence / "unavailable-project"
    unavailable_project.mkdir(mode=0o700)
    refused = run(skill_command("project-select", "--include", "video-production", target=unavailable_project),
                  "project-unavailable-video", expected=1)
    assert "selected skills require unavailable private capabilities" in refused["stderr"]
    assert not list(unavailable_project.iterdir()), "Unavailable selection must leave no project files or directories"
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
    assert snapshot(project) == project_before, "Reinstall must preserve project skills, selection and receipts"
    assert verify_copies(selected, project / ".agents/skills") == resource_count
    assert verify_copies(catalog["global"], user / ".agents/skills") == 0
    assert run(skill_command("project-doctor"), "project-reinstall-doctor")["status"] == "managed"
    assert run(skill_command("project-sync"), "project-reinstall-sync")["status"] == "unchanged"
    assert run([binary, "reinstall-status", "--home", home, "--user-home", user], "status")["status"] == "reinstalled"
    assert run([binary, "reinstall-reconcile", "--home", home, "--user-home", user, "--source", source], "completed-reconcile")["status"] == "reinstalled"
    client = Client(home)
    assert client.tool("runtime_status", {})["release_id"] == release
    recalled = client.tool("memory_read", dict(**common, record_id=retained["record_id"], revision=None))
    assert recalled["record"]["data"]["content"] == retained["data"]["content"]
    unavailable = client.tool("memory_read", dict(**common, record_id=forgotten["record_id"], revision=None), error=True)
    assert any(word in json.dumps(unavailable).lower() for word in ["forgot", "found", "excluded"])
    client.close()
    run(skill_command("project-remove"), "project-remove")
    assert all(not (project / ".agents/skills" / entry["id"]).exists() for entry in selected)
    assert project_personal.read_text() == "Keep this project-owned method.\n"
    assert (personal / "SKILL.md").read_text() == "Preserve this personal skill.\n"
    assert (user / ".codex/AGENTS.md").read_text().startswith("Preserve this personal instruction.\n")
    assert "model = 'chosen-model'" in (user / ".codex/config.toml").read_text()
    (evidence / "mcp-events.json").write_text(json.dumps(events, ensure_ascii=False, indent=2))
    (evidence / "preserved-file-hashes.json").write_text(json.dumps(before, indent=2))
    summary = dict(status="passed", platform=os.uname().sysname, machine=os.uname().machine,
                   release_id=release, manifest=manifest, core_sha256=digest(binary),
                   source_manifest_sha256=digest(source / "manifest.json"),
                   selected_project_skills=len(selected), installed_reference_files=resource_count,
                   edited_reference_sync_and_removal_refused=True, unavailable_selection_no_effect=True,
                   project_resources_preserved_through_reinstall=True, unmanaged_project_skill_preserved=True,
                   retained_state_backup_files=len(before), actual_canonical_recall=True,
                   forgetting_preserved=True, completed_reconcile_without_replay=True,
                   user_installation_untouched=True, evidence=str(evidence))
    (evidence / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2))
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
