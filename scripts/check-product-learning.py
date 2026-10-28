#!/usr/bin/env python3
"""Small, deterministic checks for product-learning artifacts.

These checks validate provenance and action safety only.  They do not prove
market demand, conversion lift, or the quality of a piece of copy.
"""

from __future__ import annotations

import hashlib
from datetime import date
from typing import Any, Iterable


EVIDENCE_TYPES = {"real", "synthetic", "inferred"}
CLAIM_STATUSES = {"supported", "conditional", "unknown", "rejected"}


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def _refs(value: Any, label: str) -> list[str]:
    _require(isinstance(value, list), f"{label} must be a list")
    _require(all(isinstance(item, str) and item for item in value), f"{label} contains an invalid reference")
    return value


def _unique(items: Iterable[str], label: str) -> None:
    values = list(items)
    _require(len(values) == len(set(values)), f"{label} contains a duplicate id")


def _known_refs(refs: list[str], known: set[str], label: str) -> None:
    _require(set(refs) <= known, f"{label} contains an unknown evidence id")


def _required_refs(value: Any, label: str) -> list[str]:
    refs = _refs(value, label)
    _require(bool(refs), f"{label} must not be empty")
    return refs


def _iso_date(value: str, label: str) -> date:
    try:
        return date.fromisoformat(value)
    except (TypeError, ValueError) as exc:
        raise ValueError(f"{label} must be an ISO date") from exc


def validate_evidence_register(register: dict[str, Any]) -> bool:
    """Validate a project-local evidence register and its contradiction links."""

    _require(register.get("schema_version") == 1, "evidence register schema_version must be 1")
    evidence = register.get("evidence")
    _require(isinstance(evidence, list) and evidence, "evidence register must contain evidence")
    ids = [item.get("id") for item in evidence]
    _require(all(isinstance(item, str) and item for item in ids), "evidence ids must be non-empty strings")
    _unique(ids, "evidence")
    known = set(ids)

    for item in evidence:
        kind = item.get("type")
        _require(kind in EVIDENCE_TYPES, f"{item.get('id')} has an invalid evidence type")
        _require(isinstance(item.get("segment"), str) and item["segment"], f"{item['id']} needs a segment")
        _require(isinstance(item.get("observation"), str) and item["observation"], f"{item['id']} needs an observation")
        source = item.get("source")
        _require(isinstance(source, dict), f"{item['id']} needs a source")
        _require(isinstance(source.get("locator"), str) and source["locator"], f"{item['id']} needs a source locator")
        _iso_date(source.get("observed_at"), f"{item['id']} source.observed_at")
        if kind == "inferred":
            derived = _required_refs(item.get("derived_from"), f"{item['id']}.derived_from")
            _known_refs(derived, known, f"{item['id']}.derived_from")
            _require(item["id"] not in derived, f"{item['id']} cannot derive from itself")
        if item.get("kind") in {"testimonial", "participant_quote"}:
            _require(isinstance(source.get("quote"), str) and source["quote"], f"{item['id']} needs the quoted text")
            if kind == "real":
                _require(source.get("consent") is True, f"{item['id']} testimonial consent is required")
            else:
                _require(item.get("kind") != "testimonial", f"{item['id']} testimonials require real evidence")

    inferred = {
        item["id"]: item.get("derived_from", [])
        for item in evidence
        if item.get("type") == "inferred"
    }
    states: dict[str, int] = {}

    def visit(node: str) -> None:
        state = states.get(node, 0)
        _require(state != 1, f"inferred evidence cycle includes {node}")
        if state == 2:
            return
        states[node] = 1
        for parent in inferred.get(node, []):
            if parent in inferred:
                visit(parent)
        states[node] = 2

    for node in inferred:
        visit(node)

    contradictions = register.get("contradictions", [])
    _require(isinstance(contradictions, list), "contradictions must be a list")
    contradiction_ids = [item.get("id") for item in contradictions]
    _require(all(isinstance(item, str) and item for item in contradiction_ids), "contradiction ids must be non-empty strings")
    _unique(contradiction_ids, "contradictions")
    for item in contradictions:
        refs = _refs(item.get("evidence_refs"), f"{item['id']}.evidence_refs")
        _require(len(refs) >= 2, f"{item['id']} needs at least two conflicting observations")
        _known_refs(refs, known, f"{item['id']}.evidence_refs")
        _require(item.get("status") in {"open", "resolved", "rejected"}, f"{item['id']} has an invalid status")
        _require(isinstance(item.get("description"), str) and item["description"], f"{item['id']} needs a description")
    return True


def validate_opportunity_map(packet: dict[str, Any], evidence_ids: set[str] | None = None) -> bool:
    """Check an outcome -> opportunity -> assumption -> test -> decision map."""

    _require(packet.get("schema_version") == 1, "opportunity map schema_version must be 1")
    _require(isinstance(packet.get("outcome"), str) and packet["outcome"], "opportunity map needs an outcome")
    opportunities = packet.get("opportunities")
    assumptions = packet.get("assumptions")
    tests = packet.get("tests")
    decisions = packet.get("decisions")
    for value, label in ((opportunities, "opportunities"), (assumptions, "assumptions"), (tests, "tests"), (decisions, "decisions")):
        _require(isinstance(value, list) and value, f"{label} must be a non-empty list")
        ids = [item.get("id") for item in value]
        _require(all(isinstance(item, str) and item for item in ids), f"{label} ids must be non-empty strings")
        _unique(ids, label)
    opportunity_ids = {item["id"] for item in opportunities}
    assumption_ids = {item["id"] for item in assumptions}
    test_ids = {item["id"] for item in tests}
    rejected = packet.get("rejected_hypotheses", [])
    _require(isinstance(rejected, list), "rejected_hypotheses must be a list")
    for collection in (opportunities, tests, decisions):
        for item in collection:
            refs = _required_refs(item.get("evidence_refs", []), f"{item.get('id')}.evidence_refs")
            if evidence_ids is not None:
                _known_refs(refs, evidence_ids, f"{item.get('id')}.evidence_refs")
    for item in assumptions:
        refs = _refs(item.get("evidence_refs", []), f"{item.get('id')}.evidence_refs")
        if item.get("status", "unknown") != "unknown":
            refs = _required_refs(refs, f"{item.get('id')}.evidence_refs")
        if evidence_ids is not None:
            _known_refs(refs, evidence_ids, f"{item.get('id')}.evidence_refs")
    for item in packet.get("rejected_hypotheses", []):
        refs = _required_refs(item.get("evidence_refs"), f"{item.get('hypothesis')}.evidence_refs")
        if evidence_ids is not None:
            _known_refs(refs, evidence_ids, f"{item.get('hypothesis')}.evidence_refs")
    for item in opportunities:
        _require(isinstance(item.get("statement"), str) and item["statement"], f"{item['id']} needs a statement")
    for item in assumptions:
        _require(item.get("opportunity_id") in opportunity_ids, f"{item['id']} references an unknown opportunity")
        _require(isinstance(item.get("statement"), str) and item["statement"], f"{item['id']} needs a statement")
    for item in tests:
        _require(item.get("assumption_id") in assumption_ids, f"{item['id']} references an unknown assumption")
        _require(isinstance(item.get("method"), str) and item["method"], f"{item['id']} needs a method")
        _require(isinstance(item.get("decision_rule"), str) and item["decision_rule"], f"{item['id']} needs a decision rule")
    for item in decisions:
        _require(item.get("test_id") in test_ids, f"{item['id']} references an unknown test")
        _require(item.get("result") in {"supports", "weakens", "inconclusive"}, f"{item['id']} has an invalid result")
        _require(isinstance(item.get("choice"), str) and item["choice"], f"{item['id']} needs a choice")
    for item in rejected:
        _require(isinstance(item.get("hypothesis"), str) and item["hypothesis"], "rejected hypothesis needs text")
        _require(isinstance(item.get("reason"), str) and item["reason"], "rejected hypothesis needs a reason")
        _require(_refs(item.get("evidence_refs"), "rejected hypothesis evidence_refs"), "rejected hypothesis needs evidence")
    return True


def validate_positioning(positioning: dict[str, Any], evidence_ids: set[str] | None = None) -> bool:
    """Keep substantiated current positioning separate from a future point of view."""

    _require(positioning.get("schema_version") == 1, "positioning schema_version must be 1")
    current = positioning.get("current")
    future = positioning.get("future_point_of_view")
    _require(isinstance(current, dict) and isinstance(future, dict), "current and future positioning are both required")
    _require(isinstance(current.get("audience"), str) and current["audience"], "current audience is required")
    alternatives = current.get("alternatives")
    _require(isinstance(alternatives, list) and alternatives, "current alternatives are required")
    known = evidence_ids or set()
    for item in alternatives:
        _require(item.get("status") == "current", "current alternatives must be marked current")
        _require(isinstance(item.get("name"), str) and item["name"], "current alternative needs a name")
        refs = _required_refs(item.get("buyer_evidence_refs"), f"{item.get('name')}.buyer_evidence_refs")
        if evidence_ids is not None:
            _known_refs(refs, known, f"{item.get('name')}.buyer_evidence_refs")
    _require(isinstance(future.get("point_of_view"), str) and future["point_of_view"], "future point of view is required")
    future_competitors = future.get("competitors", [])
    _require(isinstance(future_competitors, list), "future competitors must be a list")
    current_names = {item["name"] for item in alternatives}
    for item in future_competitors:
        _require(item.get("status") == "future", "future competitors must be marked future")
        _require(item.get("name") not in current_names, "future competitor cannot silently become a current alternative")
        _refs(item.get("evidence_refs", []), f"{item.get('name')}.evidence_refs")
    return True


def validate_copy_review(review: dict[str, Any], saved_copy: str, register: dict[str, Any]) -> bool:
    """Check that saved copy claims have current, permitted proof."""

    validate_evidence_register(register)
    _require(review.get("schema_version") == 1, "copy review schema_version must be 1")
    digest = hashlib.sha256(saved_copy.encode("utf-8")).hexdigest()
    _require(review.get("saved_copy_sha256") == digest, "copy review does not describe the saved copy")
    as_of = _iso_date(review.get("as_of"), "copy review as_of")
    evidence = {item["id"]: item for item in register["evidence"]}
    claims = review.get("claims")
    _require(isinstance(claims, list), "copy review claims must be a list")
    claim_ids = [item.get("id") for item in claims]
    _require(all(isinstance(item, str) and item for item in claim_ids), "claim ids must be non-empty strings")
    _unique(claim_ids, "claims")
    for claim in claims:
        text = claim.get("text")
        status = claim.get("status")
        _require(isinstance(text, str) and text, f"{claim.get('id')} needs claim text")
        _require(status in CLAIM_STATUSES, f"{claim.get('id')} has an invalid status")
        present = text in saved_copy
        if status in {"unknown", "rejected"}:
            _require(not present, f"{claim.get('id')} is not approved for the saved copy")
            continue
        _require(present, f"{claim.get('id')} is not present in the saved copy")
        refs = _required_refs(claim.get("proof_refs"), f"{claim.get('id')}.proof_refs")
        _known_refs(refs, set(evidence), f"{claim.get('id')}.proof_refs")
        if claim.get("kind") == "testimonial":
            for ref in refs:
                item = evidence[ref]
                _require(item["type"] == "real", f"{claim['id']} testimonial proof must be real")
                _require(item.get("kind") in {"testimonial", "participant_quote"}, f"{claim['id']} proof is not a testimonial")
                _require(item["source"].get("consent") is True, f"{claim['id']} testimonial consent is required")
        if claim.get("kind") == "metric":
            _require(all(evidence[ref].get("type") == "real" and evidence[ref].get("kind") == "metric" for ref in refs), f"{claim['id']} proof is not a real metric")
            valid_until = claim.get("valid_until")
            _require(valid_until is not None, f"{claim['id']} metric needs valid_until")
            if _iso_date(valid_until, f"{claim['id']}.valid_until") < as_of:
                revalidated = claim.get("revalidated_at")
                _require(revalidated is not None and _iso_date(revalidated, f"{claim['id']}.revalidated_at") >= as_of, f"{claim['id']} metric is expired")
        for ref in refs:
            _require(evidence[ref].get("status") != "expired", f"{claim['id']} relies on expired evidence")
    return True


def validate_funnel_diagnosis(diagnosis: dict[str, Any]) -> bool:
    """Validate a measured bottleneck without pretending it proves causality."""

    _require(diagnosis.get("schema_version") == 1, "funnel diagnosis schema_version must be 1")
    stages = diagnosis.get("stages")
    _require(isinstance(stages, list) and stages, "funnel diagnosis needs stages")
    rates: dict[str, float] = {}
    stage_ids: list[str] = []
    for stage in stages:
        _require(isinstance(stage.get("id"), str) and stage["id"], "stage needs an id")
        stage_ids.append(stage["id"])
        eligible, converted = stage.get("eligible"), stage.get("converted")
        _require(type(eligible) is int and eligible > 0, f"{stage['id']} eligible must be a positive integer")
        _require(type(converted) is int and 0 <= converted <= eligible, f"{stage['id']} converted is invalid")
        _require(abs(float(stage.get("rate", -1)) - converted / eligible) < 1e-9, f"{stage['id']} rate is not reproducible")
        rates[stage["id"]] = converted / eligible
    _unique(stage_ids, "stages")
    bottleneck = diagnosis.get("bottleneck_stage")
    _require(bottleneck in rates, "bottleneck_stage is unknown")
    _require(rates[bottleneck] == min(rates.values()), "bottleneck must be the lowest observed stage rate")
    _require(diagnosis.get("causal_status") in {"unproven", "tested"}, "causal status must not claim proof")
    return True


def validate_loop_run(spec: dict[str, Any], previous_runs: list[dict[str, Any]] | None = None) -> bool:
    """Make recurring review actions bounded, approval-aware and idempotent."""

    _require(spec.get("schema_version") == 1, "loop run schema_version must be 1")
    _require(isinstance(spec.get("review_id"), str) and spec["review_id"], "loop run needs a review_id")
    _require(isinstance(spec.get("input_digest"), str) and spec["input_digest"], "loop run needs an input_digest")
    actions = spec.get("actions")
    _require(isinstance(actions, list), "loop run actions must be a list")
    idempotency_keys = [item.get("idempotency_key") for item in actions]
    intended_ids = [item.get("intended_action_id") for item in actions]
    _require(all(isinstance(item, str) and item for item in idempotency_keys), "action idempotency keys must be non-empty strings")
    _require(all(isinstance(item, str) and item for item in intended_ids), "intended action ids must be non-empty strings")
    _unique(idempotency_keys, "actions")
    _unique(intended_ids, "actions")
    for action in actions:
        _require(action.get("mode") in {"staged", "executed", "skipped_duplicate"}, "action mode is invalid")
        if action.get("external") is True and action.get("mode") == "executed":
            _require(action.get("approved") is True, "external action requires approval")
    for previous in previous_runs or []:
        if previous.get("input_digest") != spec["input_digest"]:
            continue
        prior_ids = {item.get("intended_action_id") for item in previous.get("actions", []) if item.get("mode") == "executed"}
        duplicate = [item for item in actions if item.get("intended_action_id") in prior_ids and item.get("mode") == "executed"]
        _require(not duplicate, "repeated review would execute a duplicate intended action")
    return True
