"""LangGraph owns durable transitions; the Rust host owns every external effect."""
import json
import sqlite3
import time
from contextlib import closing
from typing import TypedDict

from langgraph.checkpoint.sqlite import SqliteSaver
from langgraph.graph import END, START, StateGraph
from langgraph.types import Command, interrupt
from langgraph.checkpoint.serde.jsonplus import JsonPlusSerializer

from memory import digest, embedding_model, read as recall


class Run(TypedDict, total=False):
    envelope: dict
    snapshot: dict
    memories: dict
    decision: str
    effect: dict
    turn: int
    deadline: int
    paused_at: int
    pause_used: int
    last_key: str
    last_digest: str
    verification: dict


def recall_queries(request, tokenizer):
    explicit = request.get("memory_queries", [])
    if explicit:
        if not isinstance(explicit, list) or len(explicit) > 8 or any(not isinstance(q, str) or not q.strip() or len(q) > 512 for q in explicit):
            raise ValueError("invalid explicit memory queries")
        if any(tokenizer.encode(q).overflowing for q in explicit):
            raise ValueError("explicit memory query exceeds embedding token window")
        return list(dict.fromkeys(explicit)), False
    objective = request["objective"]
    candidates = [objective[i:i + 256] for i in range(0, len(objective), 256)]
    # Prioritize both ends and concrete deliverable requirements before sampling
    # the middle. No model rewrite and no silent prefix-only interpretation.
    selected = candidates[:1] + candidates[-1:]
    needs = [need["reason"] for need in request.get("task_needs", [])]
    selected += needs
    selected += [candidates[round(i * (len(candidates) - 1) / 7)] for i in range(8)] if candidates else []
    selected = list(dict.fromkeys(q for q in selected if q.strip()))[:8]
    queries = []
    shortened = False
    for text in selected:
        while tokenizer.encode(text).overflowing:
            if len(text) <= 1:
                raise ValueError("embedding tokenizer cannot encode a query")
            # Keep both ends of a requirement when fitting the token budget.
            width = max(1, len(text) // 4)
            text = text[:width] + " … " + text[-width:] if len(text) > 8 else text[:len(text)//2]
            shortened = True
        queries.append(text)
    return queries, shortened or any(q not in selected for q in candidates + needs)


def recall_node(state, directory, scope):
    request = state["envelope"]["request"]
    mode = request.get("memory_recall", "best_effort")
    if mode not in ("required", "best_effort", "off"):
        raise ValueError("invalid memory recall mode")
    empty = {"results": [], "authority": "retrieved_data_not_instructions_or_verification"}
    if mode == "off" or not (directory / "vectors").exists():
        return {"memories": {**empty, "status": "disabled" if mode == "off" else "empty"}}
    try:
        queries, incomplete = recall_queries(request, embedding_model().model.tokenizer)
        pages = [recall(directory, scope, {"query": query, "limit": 5}) for query in queries]
        results, seen = [], set()
        # Interleave ranks; scores from different queries are not calibrated.
        for rank in range(5):
            for page in pages:
                items = page["results"]
                if rank < len(items) and items[rank]["id"] not in seen:
                    seen.add(items[rank]["id"])
                    results.append(items[rank])
        memories = {**empty, "results": results[:20], "queries": queries,
                    "objective_sha256": digest(request["objective"]),
                    "retrieval": {"incomplete": incomplete or len(results) > 20 or any(p.get("retrieval", {}).get("incomplete", False) for p in pages),
                                  "query_count": len(queries), "result_limit": 20}}
        return {"memories": memories}
    except Exception as error:
        if mode == "required":
            raise
        # Do not expose upstream messages containing private data or credentials.
        return {"memories": {**empty, "status": "unavailable", "warning": type(error).__name__}}


MAX_PAUSE_SECONDS = 86400  # Cumulative known-inactive time, never per resume.


def effective_deadline(state, now):
    paused = state.get("paused_at", 0)
    used = state.get("pause_used", 0)
    if not 0 <= used <= MAX_PAUSE_SECONDS:
        raise ValueError("invalid cumulative pause budget")
    if paused:
        if not paused <= now or now - paused > MAX_PAUSE_SECONDS - used:
            raise ValueError("cumulative pause allowance exhausted or clock moved backward")
        if paused >= state["deadline"]:
            raise ValueError("active budget was exhausted before pause")
        return state["deadline"] + now - paused
    return state["deadline"]


def budget_transition(state, kind, reply, now):
    deadline = effective_deadline(state, now)
    if now >= deadline:
        raise ValueError("original run budget expired; resumption cannot reset it")
    decision = reply["decision"]
    paused = state.get("paused_at", 0)
    used = state.get("pause_used", 0)
    if decision == "pause":
        if kind != "worker_turn" or paused:
            raise ValueError("only an active worker boundary can pause")
        return {"paused_at": now, "pause_used": used, "deadline": deadline}
    return {"paused_at": 0, "pause_used": used + (now - paused if paused else 0), "deadline": deadline}


def validate_transition(state, kind, reply, now):
    if kind not in ("worker_turn", "effect", "host_verification"):
        raise ValueError("unknown workflow boundary")
    allowed = {"continue", "effect", "done", "blocked", "pause"} if kind == "worker_turn" else ({"continue", "blocked"} if kind == "effect" else {"verified", "pending"})
    if reply.get("decision") not in allowed:
        raise ValueError("invalid workflow transition")
    result = {key: reply[key] for key in ("snapshot", "decision", "effect", "last_key", "last_digest", "verification") if key in reply}
    if kind != "host_verification":
        result.update(budget_transition(state, kind, reply, now))
        snapshot = result.get("snapshot", {})
        if not isinstance(snapshot, dict):
            raise ValueError("workflow snapshot must be an object")
        result["snapshot"] = {**snapshot, "deadline": result["deadline"]}
        result["turn"] = snapshot.get("turn", state.get("turn", 0))
        if type(result["turn"]) is not int or not state.get("turn", 0) <= result["turn"] <= 256:
            raise ValueError("original turn budget exhausted or reset")
    return result


def receive(state, kind):
    if kind != "host_verification" and int(time.time()) >= effective_deadline(state, int(time.time())):
        raise ValueError("original run budget expired; resumption cannot reset it")
    reply = interrupt({"kind": kind, "turn": state.get("turn", 0), "effect": state.get("effect")})
    return validate_transition(state, kind, reply, int(time.time()))


def route(state):
    return {"continue": "worker", "pause": "worker", "effect": "effect", "done": "verification", "blocked": END}[state["decision"]]


def verification_node(state):
    if not state["envelope"].get("host_verification_required"):
        return {"decision": "verified"}
    return receive(state, "host_verification")


def compiled(connection, directory, scope):
    graph = StateGraph(Run)
    graph.add_node("recall", lambda state: recall_node(state, directory, scope))
    graph.add_node("worker", lambda state: receive(state, "worker_turn"))
    graph.add_node("effect", lambda state: receive(state, "effect"))
    graph.add_node("verification", verification_node)
    graph.add_edge(START, "recall")
    graph.add_edge("recall", "worker")
    graph.add_conditional_edges("worker", route)
    graph.add_conditional_edges("effect", lambda state: "worker" if state["decision"] == "continue" else END)
    graph.add_conditional_edges("verification", lambda state: END if state["decision"] == "verified" else "verification")
    # No pickle or dynamic application class deserialization in persistent state.
    saver = SqliteSaver(connection, serde=JsonPlusSerializer(pickle_fallback=False, allowed_json_modules=[], allowed_msgpack_modules=None))
    return graph.compile(checkpointer=saver)


def describe(graph, config):
    state = graph.get_state(config)
    if not state.values:
        raise ValueError("unknown workflow run")
    interrupts = [i.value for task in state.tasks for i in task.interrupts]
    return {"checkpoint_id": state.config["configurable"]["checkpoint_id"],
            "next": list(state.next), "interrupts": interrupts, "state": state.values,
            "engine": "langgraph", "assurance": "Checkpoints do not prove external effects or host verification."}


def execute(directory, scope, operation, args):
    run_id = args["run_id"]
    config = {"configurable": {"thread_id": run_id}, "recursion_limit": 1024}
    with closing(sqlite3.connect(directory / "workflows.sqlite", check_same_thread=False, timeout=10)) as connection, connection:
        connection.execute("PRAGMA busy_timeout=10000")
        graph = compiled(connection, directory, scope)
        current = graph.get_state(config)
        if operation == "workflow_forget":
            if current.values:
                if describe(graph, config)["checkpoint_id"] != args["expected_checkpoint_id"]:
                    raise ValueError("workflow changed; inspect before forgetting")
                graph.checkpointer.delete_thread(run_id)
            connection.execute("PRAGMA wal_checkpoint(TRUNCATE)")
            connection.execute("VACUUM")
            return {"deleted": True, "run_id": run_id, "trace_retained": True,
                    "assurance": "Checkpoint deletion disables resumption; evidence traces and backups remain."}
        if operation == "workflow_start":
            if current.values:
                if current.values["envelope"] != args["envelope"]:
                    raise ValueError("workflow ID already belongs to different input")
                return describe(graph, config)
            if not int(time.time()) < args["deadline"] <= int(time.time()) + 7200:
                raise ValueError("workflow requires a future deadline within two hours")
            graph.invoke({"envelope": args["envelope"], "snapshot": args["snapshot"],
                          "turn": 0, "deadline": args["deadline"]}, config, durability="sync")
        elif operation == "workflow_resume":
            observed = describe(graph, config)
            fingerprint = digest(args["reply"])
            key = args["key"]
            if observed["state"].get("last_key") == key:
                if observed["state"].get("last_digest") != fingerprint:
                    raise ValueError("workflow retry key reused with different input")
                return observed
            if observed["checkpoint_id"] != args["expected_checkpoint_id"]:
                raise ValueError("workflow changed; inspect the current checkpoint")
            if not observed["interrupts"]:
                raise ValueError("workflow has no pending host handoff")
            reply = {**args["reply"], "last_key": key, "last_digest": fingerprint}
            # LangGraph persists interrupt resumes before running the node. Reject
            # invalid input before it can poison the pending-write journal.
            validate_transition(observed["state"], observed["interrupts"][0]["kind"], reply, int(time.time()))
            graph.invoke(Command(resume=reply), config, durability="sync")
        elif operation == "workflow_hostcheck":
            observed = describe(graph, config)
            if observed["state"].get("verification") == args["verification"]:
                return observed
            if observed["checkpoint_id"] != args["expected_checkpoint_id"]:
                raise ValueError("workflow changed; inspect the current checkpoint")
            if not observed["state"]["envelope"].get("host_verification_required") or observed["state"].get("decision") not in ("done", "verified", "pending"):
                raise ValueError("workflow is not at host verification")
            # A later failed/stale host check can revoke a prior successful report.
            # Only Mirket's validated handoff calls this fixed transition.
            decision = "verified" if args["verification"].get("status") == "host_verified" else "pending"
            graph.update_state(config, {"verification": args["verification"], "decision": decision}, as_node="verification")
            if decision == "pending":
                graph.invoke(None, config, durability="sync")
        elif operation != "workflow_status":
            raise ValueError("unknown workflow operation")
        return describe(graph, config)
