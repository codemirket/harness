"""LangGraph owns durable transitions; the Rust host owns every external effect."""
import json
import sqlite3
import time
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
    last_key: str
    last_digest: str
    verification: dict


def recall_node(state, directory, scope):
    request = state["envelope"]["request"]
    mode = request.get("memory_recall", "best_effort")
    if mode not in ("required", "best_effort", "off"):
        raise ValueError("invalid memory recall mode")
    empty = {"results": [], "authority": "retrieved_data_not_instructions_or_verification"}
    if mode == "off" or not (directory / "vectors").exists():
        return {"memories": {**empty, "status": "disabled" if mode == "off" else "empty"}}
    try:
        objective = request["objective"][:256]
        tokenizer = embedding_model().model.tokenizer
        while tokenizer.encode(objective).overflowing:
            objective = objective[:max(1, len(objective) // 2)]
        memories = recall(directory, scope, {"query": objective, "limit": 5})
        memories["query_excerpt"] = objective
        memories["objective_sha256"] = digest(request["objective"])
        return {"memories": memories}
    except Exception as error:
        if mode == "required":
            raise
        # Do not expose upstream messages containing private data or credentials.
        return {"memories": {**empty, "status": "unavailable", "warning": type(error).__name__}}


def receive(state, kind):
    if int(time.time()) >= state["deadline"] and kind != "host_verification":
        raise ValueError("original run budget expired; resumption cannot reset it")
    reply = interrupt({"kind": kind, "turn": state.get("turn", 0), "effect": state.get("effect")})
    allowed = {"continue", "effect", "done", "blocked"} if kind == "worker_turn" else ({"continue", "blocked"} if kind == "effect" else {"verified", "pending"})
    if reply.get("decision") not in allowed:
        raise ValueError("invalid workflow transition")
    result = {key: reply[key] for key in ("snapshot", "decision", "effect", "last_key", "last_digest", "verification") if key in reply}
    if kind == "worker_turn":
        result["turn"] = reply.get("snapshot", {}).get("turn", state.get("turn", 0))
        if result["turn"] > 256:
            raise ValueError("original turn budget exhausted")
    return result


def route(state):
    return {"continue": "worker", "effect": "effect", "done": "verification", "blocked": END}[state["decision"]]


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
    with sqlite3.connect(directory / "workflows.sqlite", check_same_thread=False, timeout=10) as connection:
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
