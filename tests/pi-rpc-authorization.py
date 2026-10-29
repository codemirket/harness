#!/usr/bin/env python3
"""Offline Pi RPC grant denial/acceptance against disposable canonical SQLite."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import sqlite3
import subprocess
import sys
import time

from pi_process_group import terminate_owned_group


parser = argparse.ArgumentParser()
for name in ("node", "peers", "state-root"):
    parser.add_argument("--" + name, type=Path, required=True)
args = parser.parse_args()
root = args.state_root
assert all(path.is_absolute() for path in (args.node, args.peers, root))
assert str(root).startswith("/private/tmp/mirket-pi-package-")
os.umask(0o077)
env = {
    key: value for key, value in os.environ.items()
    if not key.startswith(("PI_", "MIRKET_", "NODE_", "OPENAI_", "ANTHROPIC_"))
}
env.update({
    "HOME": str(root / "home"),
    "PI_CODING_AGENT_DIR": str(root / "agent"),
    "PI_CODING_AGENT_SESSION_DIR": str(root / "sessions"),
    "PI_SKIP_UPDATE_CHECK": "1",
    "PI_TELEMETRY_DISABLED": "1",
})
seed = Path(__file__).with_name("pi-domain-session.mjs")
seed_log = root / "rpc-authorization-seed.log"
with seed_log.open("x") as output:
    seed_child = subprocess.Popen(
        [str(args.node), str(seed), str(args.peers), str(root), "package"],
        env={**env, "MIRKET_PI_TUI_AUTH_SEED": "1"},
        stdout=output, stderr=subprocess.STDOUT, start_new_session=True,
    )
    try:
        seed_code = seed_child.wait(timeout=60)
        if seed_code:
            raise subprocess.CalledProcessError(seed_code, seed_child.args)
    finally:
        seed_failure = sys.exc_info()[0] is not None
        seed_cleanup = terminate_owned_group(seed_child, [str(root), str(args.node)])
        (root / "rpc-authorization-seed-cleanup.json").write_text(
            json.dumps(seed_cleanup, indent=2) + "\n"
        )
        if not seed_failure:
            assert not seed_cleanup["after"] and not seed_cleanup["refused_unowned_group"], seed_cleanup
intent = json.loads((root / "tui-auth-intent.json").read_text())
record_id = intent["recordId"]


def canonical_count() -> int:
    count = 0
    for file in (root / "agent/mirket/state/memory").glob("*.sqlite3"):
        with sqlite3.connect(file.as_uri() + "?mode=ro", uri=True) as db:
            count += db.execute(
                "select count(*) from records where record_id=?", (record_id,)
            ).fetchone()[0]
    return count


def native_snapshot():
    """Read logical native state and receipt bytes, excluding Pi grant/session files."""
    state = root / "agent/mirket/state"
    database_files = sorted((state / "memory").glob("*.sqlite3"))
    database_files.append(state / "memory-lifecycle/authority.sqlite3")
    observed = {"databases": {}, "receipts": {}}
    for file in database_files:
        with sqlite3.connect(file.as_uri() + "?mode=ro", uri=True) as db:
            db.execute("PRAGMA query_only=ON")
            db.execute("PRAGMA busy_timeout=1000")
            db.execute("BEGIN")
            tables = [name for (name,) in db.execute(
                "select name from sqlite_master where type='table' and name not like 'sqlite_%' order by name"
            )]
            rows = {}
            for table in tables:
                quoted = '"' + table.replace('"', '""') + '"'
                values = [
                    [value.hex() if isinstance(value, bytes) else value for value in row]
                    for row in db.execute("select * from " + quoted)
                ]
                rows[table] = sorted(values, key=lambda value: json.dumps(value, sort_keys=True))
            observed["databases"][str(file.relative_to(state))] = rows
            db.execute("ROLLBACK")
    for file in sorted((state / "operations").glob("op-*.json")):
        observed["receipts"][file.name] = hashlib.sha256(file.read_bytes()).hexdigest()
    assert len(json.dumps(observed)) < 4 * 1024 * 1024, "Native effect snapshot budget exceeded"
    return observed


def effect_projection(fts_terms=None):
    """Small, schema-named logical state; omit only FTS implementation pages."""
    state = root / "agent/mirket/state"
    observed = {"canonical": {}, "lifecycle": {}, "fts_matches": {}, "receipts": {}}
    for file in sorted((state / "memory").glob("*.sqlite3")) + [
        state / "memory-lifecycle/authority.sqlite3"
    ]:
        with sqlite3.connect(file.as_uri() + "?mode=ro", uri=True) as db:
            db.row_factory = sqlite3.Row
            db.execute("PRAGMA query_only=ON")
            db.execute("PRAGMA busy_timeout=1000")
            db.execute("BEGIN")
            tables = [row[0] for row in db.execute(
                "select name from sqlite_master where type='table' "
                "and name not like 'sqlite_%' order by name"
            ) if not row[0].startswith("record_fts_")]
            rows = {}
            for table in tables:
                quoted = '"' + table.replace('"', '""') + '"'
                query = ("select rowid,content from record_fts" if table == "record_fts"
                         else "select * from " + quoted)
                rows[table] = sorted(
                    [{key: value.hex() if isinstance(value, bytes) else value
                      for key, value in dict(row).items()} for row in db.execute(query)],
                    key=lambda row: json.dumps(row, sort_keys=True, default=str),
                )
            relative = str(file.relative_to(state))
            destination = observed["lifecycle"] if file.name == "authority.sqlite3" else observed["canonical"]
            destination[relative] = rows
            if destination is observed["canonical"]:
                terms = (sorted(set(re.findall(r"[A-Za-z][A-Za-z0-9]{3,}", " ".join(
                    row["content"] for row in rows["record_fts"]
                ))))[:32] if fts_terms is None else list(fts_terms[relative]))
                observed["fts_matches"][relative] = {
                    term: [row[0] for row in db.execute(
                        "select rowid from record_fts where record_fts match ? order by rowid",
                        ('"' + term + '"',),
                    )] for term in terms
                }
            db.execute("ROLLBACK")
    for file in sorted((state / "operations").glob("op-*.json")):
        data = file.read_bytes()
        envelope = json.loads(data)
        observed["receipts"][file.name] = {
            "sha256": hashlib.sha256(data).hexdigest(),
            "payload": json.loads(envelope["payload"]),
        }
    assert len(json.dumps(observed, default=str)) < 4 * 1024 * 1024, "Effect evidence budget exceeded"
    return observed


def assert_accepted_delta(before, after, selected_file, scope_key, operation_id, revision):
    """Assert the complete first-forget logical delta, including unrelated state."""
    assert set(before["canonical"]) == set(after["canonical"]), "Canonical scope set changed"
    assert set(before["lifecycle"]) == set(after["lifecycle"]), "Lifecycle authority set changed"
    delta = {"canonical_removed": {}, "canonical_changed": {},
             "lifecycle_added": {}, "lifecycle_changed": {}, "receipts_removed": [],
             "receipts_changed": []}
    for file, prior in before["canonical"].items():
        current = after["canonical"][file]
        assert set(prior) == set(current), f"Canonical table set changed: {file}"
        if file != selected_file:
            assert current == prior, f"Unrelated canonical scope changed: {file}"
            continue
        target_revisions = {row["row_id"] for row in prior["revisions"]
                            if row["record_id"] == record_id}
        assert target_revisions, "Target had no revision before forgetting"
        before_matches = before["fts_matches"][file]
        after_matches = after["fts_matches"][file]
        assert set(before_matches) == set(after_matches), "FTS probe set changed"
        assert after_matches == {
            term: [rowid for rowid in rowids if rowid not in target_revisions]
            for term, rowids in before_matches.items()
        }, "Unrelated logical FTS search outcome changed"
        ownership = {
            "records": lambda row: row["record_id"] == record_id,
            "revisions": lambda row: row["record_id"] == record_id,
            "sources": lambda row: row["revision_row_id"] in target_revisions,
            "semantic_revisions": lambda row: row["revision_row_id"] in target_revisions,
            "semantic_chunks": lambda row: row["revision_row_id"] in target_revisions,
            "record_fts": lambda row: row["rowid"] in target_revisions,
        }
        for table, old_rows in prior.items():
            new_rows = current[table]
            if table == "mutations":
                unrelated = [row for row in old_rows if row["record_id"] != record_id]
                assert [row for row in new_rows if row["record_id"] != record_id] == unrelated, (
                    "Unrelated canonical mutation changed"
                )
                target_before = [row for row in old_rows if row["record_id"] == record_id]
                target_after = [row for row in new_rows if row["record_id"] == record_id]
                assert target_before, "Target has no consumed mutation identity"
                assert target_after == [dict(row, intent_hash="") for row in target_before], (
                    "Target mutation IDs/revisions or tombstones differ"
                )
                delta["canonical_changed"][table] = {
                    "before": target_before, "after": target_after,
                }
            elif table in ownership:
                removed = [row for row in old_rows if ownership[table](row)]
                assert new_rows == [row for row in old_rows if not ownership[table](row)], (
                    f"Unrelated {table} row changed or target row survived"
                )
                delta["canonical_removed"][table] = removed
            else:
                assert new_rows == old_rows, f"Unrelated canonical table changed: {table}"
    file = "memory-lifecycle/authority.sqlite3"
    prior = before["lifecycle"][file]
    current = after["lifecycle"][file]
    assert set(prior) == set(current), "Lifecycle table set changed"
    for table, old_rows in prior.items():
        new_rows = current[table]
        if table == "exclusions":
            added = [{"scope_key": scope_key, "record_id": record_id,
                      "cleanup_complete": 1, "operation_id": None}]
            assert not any(row["scope_key"] == scope_key and row["record_id"] == record_id
                           for row in old_rows), "Target was excluded before commit"
            assert sorted(new_rows, key=lambda row: json.dumps(row, sort_keys=True)) == sorted(
                old_rows + added, key=lambda row: json.dumps(row, sort_keys=True)
            ), "Unrelated exclusion changed"
            delta["lifecycle_added"][table] = added
        elif table == "forget_ops":
            added = [row for row in new_rows if row["operation_id"] == operation_id]
            assert len(added) == 1 and added[0]["scope_key"] == scope_key
            assert added[0]["record_id"] == record_id and added[0]["expected_revision"] == revision
            assert added[0]["cleanup_complete"] == 1 and added[0]["created_at"] > 0
            assert [row for row in new_rows if row["operation_id"] != operation_id] == old_rows, (
                "Unrelated forget operation changed"
            )
            delta["lifecycle_added"][table] = added
        elif table == "scope_epochs":
            old_epoch = next((row["epoch"] for row in old_rows if row["scope_key"] == scope_key), 0)
            expected = [row for row in old_rows if row["scope_key"] != scope_key] + [
                {"scope_key": scope_key, "epoch": old_epoch + 1}
            ]
            assert sorted(new_rows, key=lambda row: row["scope_key"]) == sorted(
                expected, key=lambda row: row["scope_key"]
            ), "Unrelated epoch changed or selected epoch did not advance once"
            delta["lifecycle_changed"][table] = {"before": old_epoch, "after": old_epoch + 1}
        else:
            assert new_rows == old_rows, f"Unrelated lifecycle table changed: {table}"
    prior = before["receipts"]
    current = after["receipts"]
    forget_name = operation_id + ".json"
    assert forget_name in prior and forget_name in current, "Exact forget receipt missing"
    removed = set(prior) - set(current)
    assert not set(current) - set(prior), "Unexpected operation receipt appeared"
    target_saves = {name for name, value in prior.items()
                    if value["payload"].get("kind") == "memory_mutation"
                    and (value["payload"].get("record") or {}).get("record_id") == record_id}
    assert removed == target_saves, "Unrelated receipt removed or target receipt retained"
    assert current[forget_name]["payload"].get("kind") == "memory_forget"
    assert current[forget_name]["payload"].get("status") == "committed"
    assert current[forget_name]["payload"].get("target") == prior[forget_name]["payload"].get("target")
    assert all(current[name] == value for name, value in prior.items()
               if name not in removed | {forget_name}), "Unrelated receipt changed"
    delta["receipts_removed"] = sorted(removed)
    delta["receipts_changed"] = [forget_name]
    return delta


def lifecycle_rows(scope_key, operation_id):
    file = root / "agent/mirket/state/memory-lifecycle/authority.sqlite3"
    with sqlite3.connect(file.as_uri() + "?mode=ro", uri=True) as db:
        db.execute("PRAGMA query_only=ON")
        epoch = db.execute("select epoch from scope_epochs where scope_key=?", (scope_key,)).fetchone()
        return {
            "epoch": epoch[0] if epoch else 0,
            "forget_ops": db.execute("select scope_key,record_id,expected_revision,cleanup_complete from forget_ops where operation_id=?", (operation_id,)).fetchall(),
            "exclusions": db.execute("select cleanup_complete,operation_id from exclusions where scope_key=? and record_id=?", (scope_key, record_id)).fetchall(),
            "cleanup_receipts": db.execute("select operation_id from cleanup_receipts where scope_key=? and record_id=?", (scope_key, record_id)).fetchall(),
        }


assert canonical_count() == 1, "The disposable seed record is missing"
provider = root / "provider-rpc-authorization.mjs"
provider.write_text(
    "import * as ai from "
    + json.dumps((args.peers / "@earendil-works/pi-ai/dist/index.js").as_uri())
    + ";\nimport {makeFixtureProvider} from "
    + json.dumps(Path(__file__).with_name("pi-offline-provider.mjs").resolve().as_uri())
    + ";\nexport default makeFixtureProvider(ai);\n"
)
cli = args.peers / "@earendil-works/pi-coding-agent/dist/cli.js"
base = [str(args.node), str(cli), "--offline", "--provider", "offline-fixture",
        "--model", "deterministic", "-e", str(provider), "--mode", "rpc"]
stderr = (root / "rpc-authorization.stderr").open("x")
events_file = (root / "rpc-authorization-events.jsonl").open("x")
child = subprocess.Popen(base, cwd=root / "project", env=env, stdin=subprocess.PIPE,
                         stdout=subprocess.PIPE, stderr=stderr, start_new_session=True)
selector = selectors.DefaultSelector()
selector.register(child.stdout, selectors.EVENT_READ)
buffer = b""
events = []
sequence = 0
total_bytes = 0
deadline = time.monotonic() + 90
report = {"platform": os.uname().sysname, "recordId": record_id, "checks": [],
          "node": str(args.node), "piCli": str(cli), "stateRoot": str(root),
          "unrun": ["scope-wide canonical/lifecycle effect snapshot", "exact repeat",
                    "reconcile", "full native effect classes and platform gates"]}


def save() -> None:
    (root / "rpc-authorization-report.json").write_text(json.dumps(report, indent=2) + "\n")


def read():
    global buffer, total_bytes
    while b"\n" not in buffer:
        remaining = deadline - time.monotonic()
        if remaining <= 0 or not selector.select(min(30, remaining)):
            raise TimeoutError("Pi RPC response timeout")
        chunk = os.read(child.stdout.fileno(), 65536)
        if not chunk:
            raise RuntimeError("Pi RPC exited before response")
        total_bytes += len(chunk)
        if total_bytes > 4 * 1024 * 1024:
            raise RuntimeError("Pi RPC event budget exceeded")
        buffer += chunk
    line, buffer = buffer.split(b"\n", 1)
    event = json.loads(line)
    events_file.write(json.dumps(event) + "\n")
    events_file.flush()
    events.append(event)
    return event


def request(kind, **fields):
    global sequence
    sequence += 1
    ident = str(sequence)
    child.stdin.write((json.dumps({"id": ident, "type": kind, **fields}) + "\n").encode())
    child.stdin.flush()
    while True:
        event = read()
        if event.get("type") == "response" and event.get("id") == ident:
            assert event.get("success"), event
            return event.get("data")


def prompt(message):
    start = len(events)
    reply = request("prompt", message=message)
    if reply.get("disposition") != "handled":
        while not any(e.get("type") == "agent_settled" for e in events[start:]):
            read()
    return events[start:]


def tool_end(observed, name):
    matches = [e for e in observed if e.get("type") == "tool_execution_end"
               and e.get("toolName") == name]
    assert matches, observed
    return matches[-1]


try:
    search = tool_end(prompt(json.dumps({"tool": "tool_search", "args": {
        "query": "memory_forget_prepare memory_forget_commit", "limit": 4
    }})), "tool_search")
    assert not search["isError"], search
    prepare = tool_end(prompt(json.dumps(intent["prepare"])),
                       "mcp__mirket__memory_forget_prepare")
    assert not prepare["isError"], prepare
    prepared = json.loads(prepare["result"]["content"][0]["text"])
    intent["request"]["args"]["operation_id"] = prepared["operation_id"]
    request_message = json.dumps(intent["request"])
    canonical_files = []
    for file in (root / "agent/mirket/state/memory").glob("*.sqlite3"):
        with sqlite3.connect(file.as_uri() + "?mode=ro", uri=True) as db:
            if db.execute("select count(*) from records where record_id=?", (record_id,)).fetchone()[0] == 1:
                canonical_files.append(file)
    assert len(canonical_files) == 1, "Expected one exact canonical scope for the target"
    scope_key = canonical_files[0].stem.removeprefix("scope-")
    before_native = native_snapshot()
    before_effect = effect_projection()
    before_lifecycle = lifecycle_rows(scope_key, prepared["operation_id"])
    assert before_lifecycle["forget_ops"] == [] and before_lifecycle["exclusions"] == []
    before_heads = sorted(row[0] for tables in before_native["databases"].values()
                          for row in tables.get("records", []))
    assert record_id in before_heads and len(before_heads) > 1, "Missing unrelated canary record"
    denied = tool_end(prompt(request_message), "mcp__mirket__memory_forget_commit")
    assert denied["isError"] and "Explicit authorization required" in json.dumps(denied), denied
    assert canonical_count() == 1, "An ungranted RPC call changed the target record row"
    assert native_snapshot() == before_native, "Ungranting changed canonical/lifecycle/receipt state"
    report["checks"].append("RPC ungranted commit denied; native effect snapshot unchanged")
    save()

    grant_command = "/mirket-authorize " + json.dumps({
        "tool": "mcp__mirket__memory_forget_commit", "args": intent["request"]["args"]
    })
    prompt(grant_command)
    assert native_snapshot() == before_native, "Pi grant command changed native effect state"
    accepted = tool_end(prompt(request_message), "mcp__mirket__memory_forget_commit")
    assert not accepted["isError"], accepted
    result = json.loads(accepted["result"]["content"][0]["text"])
    assert result["status"] == "forgotten", result
    assert canonical_count() == 0, "An exact RPC grant did not remove the target record row"
    after_native = native_snapshot()
    after_effect = effect_projection({file: matches.keys()
                                      for file, matches in before_effect["fts_matches"].items()})
    after_heads = sorted(row[0] for tables in after_native["databases"].values()
                         for row in tables.get("records", []))
    assert after_heads == [head for head in before_heads if head != record_id], "Unrelated record changed"
    after_lifecycle = lifecycle_rows(scope_key, prepared["operation_id"])
    assert after_lifecycle == {
        "epoch": before_lifecycle["epoch"] + 1,
        "forget_ops": [(scope_key, record_id, intent["prepare"]["args"]["expected_revision"], 1)],
        "exclusions": [(1, None)],
        "cleanup_receipts": [],
    }, after_lifecycle
    receipt_file = root / "agent/mirket/state/operations" / (prepared["operation_id"] + ".json")
    assert json.loads(json.loads(receipt_file.read_text())["payload"])["status"] == "committed"
    delta = assert_accepted_delta(
        before_effect, after_effect, str(canonical_files[0].relative_to(root / "agent/mirket/state")),
        scope_key, prepared["operation_id"], intent["prepare"]["args"]["expected_revision"],
    )
    report["accepted_effect_oracle"] = {
        "schema": "logical-canonical-lifecycle-receipts-v1",
        "before": before_effect, "after": after_effect, "expected_delta": delta,
    }
    report["checks"].append("Exact RPC grant commits only the scoped target delta; all unrelated content, lifecycle rows and receipts unchanged")
    save()
    repeated = tool_end(prompt(request_message), "mcp__mirket__memory_forget_commit")
    if not repeated["isError"]:
        repeated_result = json.loads(repeated["result"]["content"][0]["text"])
        assert repeated_result["status"] == "forgotten" and repeated_result.get("replay_allowed") is False, repeated_result
    assert native_snapshot()["databases"] == after_native["databases"], "Repeat changed native effect tables"
    assert lifecycle_rows(scope_key, prepared["operation_id"]) == after_lifecycle
    report["checks"].append("Exact granted repeat produces no second native effect")
    reconcile_args = dict(intent["request"]["args"])
    reconcile_message = json.dumps({"tool": "mcp__mirket__memory_forget_reconcile", "args": reconcile_args})
    search = tool_end(prompt(json.dumps({"tool": "tool_search", "args": {
        "query": "memory_forget_reconcile", "limit": 4
    }})), "tool_search")
    assert not search["isError"], search
    prompt("/mirket-authorize " + json.dumps({
        "tool": "mcp__mirket__memory_forget_reconcile", "args": reconcile_args
    }))
    reconciled = tool_end(prompt(reconcile_message), "mcp__mirket__memory_forget_reconcile")
    assert not reconciled["isError"], reconciled
    assert json.loads(reconciled["result"]["content"][0]["text"])["status"] == "forgotten"
    assert native_snapshot()["databases"] == after_native["databases"], "Reconcile changed native effect tables"
    assert lifecycle_rows(scope_key, prepared["operation_id"]) == after_lifecycle
    report["checks"].append("Exact separately granted reconcile preserves the committed outcome")
    report["unrun"] = ["full native effect classes and platform gates"]
    save()
    child.stdin.close()
    child.wait(timeout=8)
    assert child.returncode == 0, child.returncode
    assert not any(str(root / "agent/mirket/bin/mirket") + " serve" in line
                   for line in subprocess.check_output(
                       ["/bin/ps", "-axo", "command="], text=True).splitlines()), "Owned MCP child remains"
    report["passed"] = True
    save()
finally:
    active_failure = sys.exc_info()[0] is not None
    cleanup = terminate_owned_group(child, [str(root), str(args.node)])
    report["cleanup"] = cleanup
    if cleanup["after"] or cleanup["refused_unowned_group"]:
        report["passed"] = False
    selector.close()
    events_file.close()
    stderr.close()
    save()
    if not active_failure:
        assert not cleanup["after"] and not cleanup["refused_unowned_group"], cleanup
print(json.dumps(report))
