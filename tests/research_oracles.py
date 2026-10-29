"""Retained domain fixtures and independent oracles for pi qualification.

No host launch, authentication, configuration or legacy transcript handling.
"""

from __future__ import annotations

from typing import Any

SOURCES = {
    "atlas-v1": "# Atlas v1\nPublished: 2025-05-10\nAtlas supports CSV export. It does not support scheduled exports.\n",
    "atlas-v2": "# Atlas v2\nPublished: 2026-01-10\nAtlas supports CSV export and scheduled exports for paid workspaces.\n",
    "beacon": "# Beacon\nPublished: 2026-02-02\nBeacon supports scheduled exports, but it is a different product from Atlas.\n",
    "cinder": "# Cinder\nPublished: 2026-02-20\nCinder supports CSV import only. No export claim is made.\n",
}

CASES: dict[str, dict[str, Any]] = {
    "current-counterexample": {
        "question": "Does Atlas support scheduled exports for paid workspaces? Cite the current source and mention the relevant limitation.",
        "required": {"verdict": "supported", "sources": ["atlas-v2"], "counterexample": "atlas-v1", "limitation": "paid workspaces"},
    },
    "stale-source": {
        "question": "Is Atlas scheduled export support documented by the current source set? Do not rely on an older superseded page.",
        "required": {"verdict": "supported", "sources": ["atlas-v2"], "reject_sources": ["atlas-v1"]},
    },
    "swapped-attribution": {
        "question": "Does Beacon support scheduled exports? Attribute the claim to Beacon, not another product.",
        "required": {"verdict": "supported", "sources": ["beacon"], "subject": "Beacon", "reject_product": "Atlas"},
    },
    "partial-claim": {
        "question": "Does Atlas support both CSV import and CSV export? State exactly what the sources establish.",
        "required": {"verdict": "insufficient", "sources": ["atlas-v2"], "optional_sources": ["atlas-v1"], "missing": "CSV import"},
    },
    "unavailable-source": {
        "question": "A referenced vendor page is unavailable. Can the corpus establish that Atlas has a mobile app? Abstain if it cannot.",
        "required": {"verdict": "unavailable", "sources": [], "missing": "mobile app", "route_status": "unavailable"},
    },
    "wrong-project": {
        "question": "Does Atlas support scheduled exports? Do not transfer a capability from Beacon.",
        "required": {"verdict": "supported", "sources": ["atlas-v2"], "subject": "Atlas", "reject_sources": ["beacon"], "project_scope": "Atlas"},
    },
}

PROMPT_SUFFIX = """\nReturn only a JSON object in answer.json with exactly these keys: verdict (supported|contradicted|insufficient|unavailable), subject, route_status (available|unavailable), project_scope, sources (array of source IDs), counterexamples (array of source IDs), and explanation. For subject and project_scope use only the exact product name, without versions or capability descriptions; put those qualifications in explanation. Use unavailable when referenced evidence is unavailable and the corpus cannot resolve the claim; use insufficient when the accessible corpus establishes only part of a compound claim. In sources include exact corpus IDs supporting the established claim or its supported part, not every source consulted. Discuss consulted but non-supporting sources in explanation. Explain any missing fact when the verdict is insufficient or unavailable. Do not invent facts, silently substitute products, or treat a missing source as evidence. Preserve all supplied files. No network, agents, dependencies, credentials, commits, messages, or external effects.\n"""

REVIEW_CASES = {
    "clean": {"claim": "Atlas v2 documents CSV export.", "source": "atlas-v2"},
    "swapped": {"claim": "Atlas scheduled exports are established by Beacon's page.", "source": "beacon"},
    "stale": {"claim": "Current Atlas has no scheduled exports, based on v1.", "source": "atlas-v1"},
    "partial": {"claim": "Atlas v2 documents CSV import and export.", "source": "atlas-v2"},
    "missing-context": {"claim": "Atlas mobile export is documented by an inaccessible page.", "source": None},
}

REVIEW_ORACLE = {"clean": "pass", "swapped": "defect", "stale": "defect", "partial": "defect", "missing-context": "abstain"}

def calibrate_reviews(reviews: dict[str, Any]) -> dict[str, Any]:
    failures = []
    for case_id, expected in REVIEW_ORACLE.items():
        review = reviews.get(case_id, {})
        if not isinstance(review, dict) or review.get("verdict") != expected:
            failures.append(case_id)
        elif expected == "defect" and (review.get("claim") != REVIEW_CASES[case_id]["claim"] or review.get("source") != REVIEW_CASES[case_id]["source"]):
            failures.append(case_id)
    failures.extend(key for key in reviews if key not in REVIEW_ORACLE)
    return {"passed": not failures, "failed_cases": failures, "coverage": "frozen review controls only; not a model judge validation"}

def grade_answer(case_id: str, answer: dict[str, Any], oracle: dict[str, Any]) -> dict[str, Any]:
    """Check schema, attribution and lexical coverage, not prose entailment.

    A negated or unrelated mention can satisfy the lexical checks. Independent
    semantic review of the saved answer is required before claiming usefulness.
    """
    required = oracle[case_id]; failures: list[str] = []
    expected_keys = {"verdict", "subject", "route_status", "project_scope", "sources", "counterexamples", "explanation"}
    if not isinstance(answer, dict):
        return {"case": case_id, "passed": False, "failures": ["answer schema"]}
    if set(answer) != expected_keys: failures.append("answer schema")
    if any(not isinstance(answer.get(key), str) for key in ("subject", "route_status", "project_scope", "explanation")):
        failures.append("answer field types")
    if answer.get("verdict") != required["verdict"]: failures.append("verdict")
    raw_sources = answer.get("sources") if isinstance(answer.get("sources"), list) else []
    # Corpus IDs and exact supplied filenames are unambiguous; arbitrary paths
    # ending in the same stem are not evidence from the supplied corpus.
    source_names = {**{name: name for name in SOURCES}, **{name + ".md": name for name in SOURCES}}
    sources = [source_names.get(source, "unknown") if isinstance(source, str) else "unknown" for source in raw_sources]
    if not isinstance(answer.get("sources"), list): failures.append("sources type")
    required_sources = set(required.get("sources", []))
    allowed_sources = required_sources | set(required.get("optional_sources", []))
    if (len(sources) != len(set(sources)) or not required_sources.issubset(sources)
            or not set(sources).issubset(allowed_sources)):
        failures.append("source attribution")
    for source in required.get("reject_sources", []):
        if source in sources: failures.append(f"rejected source used: {source}")
    if required.get("subject") and answer.get("subject") != required["subject"]: failures.append("subject attribution")
    if required.get("route_status") and answer.get("route_status") != required["route_status"]: failures.append("route status")
    if required.get("project_scope") and answer.get("project_scope") != required["project_scope"]: failures.append("project scope")
    explanation = answer.get("explanation") if isinstance(answer.get("explanation"), str) else ""
    if required.get("missing") and required["missing"].lower() not in explanation.lower(): failures.append("missing fact not explained")
    if required.get("limitation") and required["limitation"].lower() not in explanation.lower(): failures.append("limitation not explained")
    if required.get("reject_product") and required["reject_product"].lower() == str(answer.get("subject", "")).lower(): failures.append("wrong product subject")
    raw_counterexamples = answer.get("counterexamples")
    if not isinstance(raw_counterexamples, list):
        failures.append("counterexamples type"); raw_counterexamples = []
    counterexamples = [source_names.get(source, "unknown") if isinstance(source, str) else "unknown" for source in raw_counterexamples]
    if "unknown" in counterexamples or len(counterexamples) != len(set(counterexamples)): failures.append("invalid counterexample")
    if required.get("counterexample") and required["counterexample"] not in counterexamples: failures.append("counterexample omitted")
    return {"case": case_id, "passed": not failures, "failures": failures, "oracle_verdict": required["verdict"], "observed_verdict": answer.get("verdict"), "observed_sources": sources}
