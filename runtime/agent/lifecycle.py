"""Bounded cursor-based lifecycle. Unresolved workflows are never selected."""
from datetime import datetime
import json
import sqlite3
import time
import uuid

from memory import digest, opened, presented
from workflow import compiled, describe


def revision(connection, advance=False):
    connection.execute("CREATE TABLE IF NOT EXISTS mirket_revision (id INTEGER PRIMARY KEY CHECK(id=1), value INTEGER NOT NULL)")
    connection.execute("INSERT OR IGNORE INTO mirket_revision VALUES(1,0)")
    if advance:
        connection.execute("UPDATE mirket_revision SET value=value+1 WHERE id=1")
    connection.commit()
    return connection.execute("SELECT value FROM mirket_revision WHERE id=1").fetchone()[0]


def fact_page(memory, scope, offset, limit):
    points, after = memory.vector_store.client.scroll(
        collection_name=memory.vector_store.collection_name,
        scroll_filter=memory.vector_store._create_filter({"user_id": scope}),
        offset=offset, limit=limit, with_payload=False, with_vectors=False)
    return [presented(memory.get(str(point.id))) for point in points], str(after) if after is not None else None


def run_record(graph, run_id):
    state = graph.get_state({"configurable": {"thread_id": run_id}})
    if not state.values:
        return None
    return {"run_id": run_id, "checkpoint_id": state.config["configurable"]["checkpoint_id"],
            "updated_at": state.created_at,
            "completed": not state.next and not state.tasks and state.values.get("decision") == "verified"}


def cursor(value, fingerprint=None):
    if value is None:
        return {"phase": "memory", "after": None}
    expected = {"phase", "after"} | ({"sha256"} if fingerprint is not None else set())
    if not isinstance(value, dict) or set(value) != expected or value["phase"] not in ("memory", "workflow"):
        raise ValueError("export changed or cursor invalid; restart traversal")
    if fingerprint is not None and value["sha256"] != fingerprint:
        raise ValueError("export changed; restart export")
    after = value["after"]
    if after is not None:
        if not isinstance(after, str) or len(after) > 64:
            raise ValueError("invalid lifecycle cursor")
        if value["phase"] == "memory":
            uuid.UUID(after)
    return {"phase": value["phase"], "after": after}


def policy(value):
    if not isinstance(value, dict) or set(value) != {"expired_before", "completed_before"}:
        raise ValueError("policy requires expired_before and completed_before Unix timestamps")
    now = int(time.time())
    for cutoff in value.values():
        if type(cutoff) is not int or not 0 <= cutoff <= now:
            raise ValueError("retention cutoffs must be past Unix timestamps")
    return value


def expired(item, rule):
    expiry = item.get("metadata", {}).get("expires_at")
    return expiry is not None and expiry <= rule["expired_before"]


def completed(run, rule):
    return run["completed"] and datetime.fromisoformat(run["updated_at"]).timestamp() <= rule["completed_before"]


def execute(directory, scope, operation, args):
    with opened(directory) as memory, sqlite3.connect(directory / "workflows.sqlite", check_same_thread=False, timeout=10) as connection:
        graph = compiled(connection, directory, scope)
        graph.checkpointer.setup()
        if operation == "data_export":
            page_bytes = args.get("page_bytes", 3 * 1024 * 1024)
            if type(page_bytes) is not int or not 4096 <= page_bytes <= 3 * 1024 * 1024:
                raise ValueError("invalid export page byte budget")
            count = memory.vector_store.client.count(collection_name=memory.vector_store.collection_name,
                count_filter=memory.vector_store._create_filter({"user_id": scope}), exact=True).count
            runs = connection.execute("SELECT COUNT(DISTINCT thread_id) FROM checkpoints").fetchone()[0]
            fingerprint = digest({"scope": scope, "revision": revision(connection), "facts": count, "runs": runs})
            position = cursor(args.get("cursor"), fingerprint)
            items, size = [], 0
            # One bounded page; no scan of previous pages. Revision changes before
            # every owned mutation, including failed/uncertain effects.
            while len(items) < 20 and position is not None:
                phase, after = position["phase"], position["after"]
                if phase == "memory":
                    facts, next_after = fact_page(memory, scope, after, 1)
                    if not facts:
                        position = {"phase": "workflow", "after": None}
                        continue
                    item = {"kind": "memory", "value": facts[0]}
                    following = {"phase": "memory", "after": next_after} if next_after else {"phase": "workflow", "after": None}
                else:
                    rows = connection.execute("SELECT DISTINCT thread_id FROM checkpoints WHERE thread_id > ? ORDER BY thread_id LIMIT 2", (after or "",)).fetchall()
                    if not rows:
                        position = None
                        break
                    run_id = rows[0][0]
                    item = {"kind": "workflow", "run_id": run_id, "value": describe(graph, {"configurable": {"thread_id": run_id}})}
                    following = {"phase": "workflow", "after": run_id} if len(rows) > 1 else None
                length = len(json.dumps(item, ensure_ascii=False).encode())
                if length > 3 * 1024 * 1024:
                    raise ValueError("single export record exceeds 3 MiB; export incomplete")
                if length > page_bytes:
                    raise ValueError("record exceeds this transport page budget; use CLI agent export")
                if size + length > page_bytes:
                    break
                items.append(item)
                size += length
                position = following
            if position == {"phase": "workflow", "after": None} and runs == 0:
                position = None
            return {"format": "mirket-data-export-v2", "scope": scope, "sha256": fingerprint,
                    "items": items, "total": count + runs,
                    "next_cursor": {**position, "sha256": fingerprint} if position else None,
                    "coverage": "All facts and latest snapshots through revision-bound pages. Not a restorable database backup. External database edits are unsupported."}
        if operation == "data_plan":
            rule = policy(args["policy"])
            position = cursor(args.get("cursor"))
            memories, workflows = [], []
            scanned = 0
            while position is not None and scanned < 256 and len(memories) < 100 and len(workflows) < 100:
                phase, after = position["phase"], position["after"]
                if phase == "memory":
                    facts, next_after = fact_page(memory, scope, after, 1)
                    if facts and expired(facts[0], rule):
                        memories.append({"id": facts[0]["id"], "sha256": facts[0]["sha256"]})
                    position = {"phase": "memory", "after": next_after} if next_after else {"phase": "workflow", "after": None}
                else:
                    rows = connection.execute("SELECT DISTINCT thread_id FROM checkpoints WHERE thread_id > ? ORDER BY thread_id LIMIT 2", (after or "",)).fetchall()
                    if not rows:
                        position = None
                        break
                    run = run_record(graph, rows[0][0])
                    if completed(run, rule):
                        workflows.append({"run_id": run["run_id"], "checkpoint_id": run["checkpoint_id"]})
                    position = {"phase": "workflow", "after": rows[0][0]} if len(rows) > 1 else None
                scanned += 1
            plan = {"scope": scope, "policy": rule, "memories": memories, "workflows": workflows}
            return {"plan": plan, "sha256": digest(plan), "has_more": position is not None,
                    "next_cursor": position, "scanned": scanned,
                    "retained": "Unresolved work, nonexpired facts, traces and backups. Follow next_cursor even if this batch is empty; review each exact batch."}
        if operation != "data_apply":
            raise ValueError("unknown lifecycle operation")
        plan = args["plan"]
        if (not isinstance(plan, dict) or set(plan) != {"scope", "policy", "memories", "workflows"}
                or plan["scope"] != scope or digest(plan) != args["expected_sha256"]):
            raise ValueError("retention plan identity mismatch")
        rule = policy(plan["policy"])
        for name, identity, checksum in (("memories", "id", "sha256"), ("workflows", "run_id", "checkpoint_id")):
            selected = plan[name]
            if (not isinstance(selected, list) or len(selected) > 100
                    or any(not isinstance(x, dict) or set(x) != {identity, checksum} or any(not isinstance(v, str) or not v for v in x.values()) for x in selected)
                    or len({x[identity] for x in selected}) != len(selected)):
                raise ValueError("invalid retention batch")
            for entry in selected:
                if name == "memories":
                    item = presented(memory.get(entry[identity]))
                    valid = item is None or (item["sha256"] == entry[checksum] and expired(item, rule))
                else:
                    run = run_record(graph, entry[identity])
                    valid = run is None or (run["checkpoint_id"] == entry[checksum] and completed(run, rule))
                if not valid:
                    raise ValueError("retention candidate changed or is unresolved; preview again")
        for item in plan["memories"]:
            if memory.get(item["id"]):
                memory.delete(item["id"])
            memory.db.connection.execute("DELETE FROM history WHERE memory_id=?", (item["id"],))
            memory.db.connection.commit()
        for run in plan["workflows"]:
            graph.checkpointer.delete_thread(run["run_id"])
        connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
        connection.execute("VACUUM")
        memory.db.connection.execute("VACUUM")
        return {"applied": True, "sha256": digest(plan), "memories": len(plan["memories"]),
                "workflows": len(plan["workflows"]), "traces_retained": True}
