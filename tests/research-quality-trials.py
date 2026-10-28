#!/usr/bin/env python3
"""Freeze and grade a small research-evidence calibration cohort.

This is deliberately a synthetic corpus.  It tests evidence handling and skill
activation separately from live-web coverage.  The native runner is bounded,
uses disposable homes/projects and never retries or repairs a cell.
Hashes detect accidental drift in trusted local evidence; they do not
authenticate records against an author who rewrites the entire evidence tree.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import time
from typing import Any

REPO = Path(__file__).resolve().parents[1]
SHARED_WALL_SECONDS = 1200
PER_CELL_SECONDS = 90
CONDITIONS = ("no-skill", "with-skill")
MODEL = "gpt-6-astra"
REASONING_EFFORT = "high"
SKILL_ID = "browser-research"

SOURCES = {
    "atlas-v1": "# Atlas v1\nPublished: 2025-05-10\nAtlas supports CSV export. It does not support scheduled exports.\n",
    "atlas-v2": "# Atlas v2\nPublished: 2026-01-10\nAtlas supports CSV export and scheduled exports for paid workspaces.\n",
    "beacon": "# Beacon\nPublished: 2026-02-02\nBeacon supports scheduled exports, but it is a different product from Atlas.\n",
    "cinder": "# Cinder\nPublished: 2026-02-20\nCinder supports CSV import only. No export claim is made.\n",
}

# Oracle facts are withheld from the project and prompt. The host sandbox does
# not establish read isolation from their sibling directory; audit native reads
# and qualify this as trusted prompt compliance, not an access-isolated oracle.
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
        "required": {"verdict": "insufficient", "sources": ["atlas-v2"], "missing": "CSV import"},
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

PROMPT_SUFFIX = """\nReturn only a JSON object in answer.json with exactly these keys: verdict (supported|contradicted|insufficient|unavailable), subject, route_status (available|unavailable), project_scope, sources (array of source IDs), counterexamples (array of source IDs), and explanation. Use exact source IDs from the corpus. Explain any missing fact when the verdict is insufficient or unavailable. Do not invent facts, silently substitute products, or treat a missing source as evidence. Preserve all supplied files. No network, agents, dependencies, credentials, commits, messages, or external effects.\n"""

# Independent review controls: findings must locate a concrete claim and its
# decisive source. These are development controls, not native judge results.
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


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_hash(root: Path) -> str:
    return hashlib.sha256("".join(f"{p.relative_to(root)}:{digest(p)}\n" for p in sorted(root.rglob("*")) if p.is_file()).encode()).hexdigest()


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2) + "\n")


def cohort_budget(root: Path) -> dict[str, Any]:
    """Refuse drift in the original deadline instead of trusting mutable state."""
    path = root / "cohort-state.json"
    if digest(path) != (root / "cohort-state.sha256").read_text().strip():
        raise SystemExit("cohort budget changed")
    state = json.loads(path.read_text())
    frozen = json.loads((root / "frozen/platform.json").read_text())
    for key in ("started_at", "deadline_at"):
        if type(state.get(key)) not in (int, float) or not math.isfinite(state[key]):
            raise SystemExit("invalid cohort clock")
    if (state.get("model") != frozen["model"] or state.get("reasoning_effort") != frozen["reasoning_effort"]
            or state.get("repairs") != 0 or state.get("retries") != 0
            or state["deadline_at"] - state["started_at"] > frozen["shared_wall_seconds"]
            or state["deadline_at"] <= state["started_at"]):
        raise SystemExit("cohort budget differs from freeze")
    return state


def freeze(root: Path, codex: Path, skill: Path | None, model_cache: Path | None = None) -> None:
    if root.exists():
        raise SystemExit(f"refusing existing trial root: {root}")
    if not codex.is_file():
        raise SystemExit(f"missing Codex executable: {codex}")
    if skill is None or skill.name != SKILL_ID or not (skill / "SKILL.md").is_file():
        raise SystemExit("the browser-research skill directory is required")
    if f"name: {SKILL_ID}\n" not in (skill / "SKILL.md").read_text():
        raise SystemExit("unexpected skill identity")
    if model_cache is not None:
        cache = json.loads(model_cache.read_text())
        if not isinstance(cache, dict) or not isinstance(cache.get("models"), list):
            raise SystemExit("invalid host model cache")
    root.mkdir(parents=True, mode=0o700)
    frozen = root / "frozen"; frozen.mkdir()
    if model_cache is not None:
        shutil.copyfile(model_cache, frozen / "models_cache.json")
        (frozen / "models_cache.json").chmod(0o600)
    write_json(frozen / "cases.json", CASES)
    (frozen / "cases.sha256").write_text(digest(frozen / "cases.json") + "\n")
    corpus = frozen / "corpus"; corpus.mkdir()
    for source_id, text in SOURCES.items():
        (corpus / f"{source_id}.md").write_text(text)
    write_json(frozen / "platform.json", {"codex": str(codex), "codex_sha256": digest(codex), "platform": platform.platform(), "model": MODEL, "reasoning_effort": REASONING_EFFORT, "shared_wall_seconds": SHARED_WALL_SECONDS, "per_cell_seconds": PER_CELL_SECONDS})
    if skill is not None:
        if not skill.is_dir(): raise SystemExit(f"missing skill directory: {skill}")
        shutil.copytree(skill, root / "frozen-skill")
        (frozen / "skill.sha256").write_text(tree_hash(root / "frozen-skill") + "\n")
    write_json(frozen / "oracle.json", {case_id: case["required"] for case_id, case in CASES.items()})
    write_json(frozen / "review-cases.json", REVIEW_CASES)
    write_json(frozen / "review-oracle.json", REVIEW_ORACLE)
    for case_id, case in CASES.items():
        for condition in CONDITIONS:
            trial = root / f"{case_id}-{condition}"; project = trial / "project"
            trial.mkdir(); project.mkdir(); (trial / "user-home").mkdir(mode=0o700); (trial / "codex-home").mkdir(mode=0o700); (trial / "tmp").mkdir(mode=0o700)
            for source_id, text in SOURCES.items(): (project / f"{source_id}.md").write_text(text)
            if condition == "with-skill" and skill is not None:
                shutil.copytree(root / "frozen-skill", project / ".agents" / "skills" / skill.name)
            (project / "QUESTION.md").write_text(case["question"] + PROMPT_SUFFIX)
            write_json(trial / "initial-manifest.json", {str(p.relative_to(project)): digest(p) for p in project.rglob("*") if p.is_file()})
            write_json(trial / "budget.json", {"seconds": PER_CELL_SECONDS, "shared_wall_seconds": SHARED_WALL_SECONDS, "repairs": 0, "retries": 0})
            (trial / "prompt.txt").write_text(case["question"] + PROMPT_SUFFIX)
    frozen_paths = [p for p in frozen.rglob("*") if p.is_file() and p.name != "frozen-inputs.json"]
    frozen_paths += [p for pattern in ("*/initial-manifest.json", "*/budget.json", "*/prompt.txt") for p in root.glob(pattern)]
    write_json(frozen / "frozen-inputs.json", {str(p.relative_to(root)): digest(p) for p in frozen_paths})


def verify_frozen(root: Path, case_id: str, condition: str) -> None:
    frozen = root / "frozen"; cases = json.loads((frozen / "cases.json").read_text())
    if digest(frozen / "cases.json") != (frozen / "cases.sha256").read_text().strip(): raise SystemExit("frozen cases changed")
    if cases != CASES: raise SystemExit("case definitions changed")
    seal = frozen / "frozen-inputs.json"
    if not seal.is_file(): raise SystemExit("missing frozen input seal")
    sealed = json.loads(seal.read_text())
    current_sealed = {name: digest(root / name) for name in sealed}
    if sealed != current_sealed: raise SystemExit("frozen inputs changed")
    trial = root / f"{case_id}-{condition}"; project = trial / "project"
    if not project.is_dir(): raise SystemExit("missing trial project")
    all_project = {str(p.relative_to(project)): digest(p) for p in project.rglob("*") if p.is_file()}
    if any(p.is_symlink() for p in project.rglob("*")):
        raise SystemExit("linked trial artifact")
    answer_key = "answer.json"
    if any(key.endswith("/answer.json") for key in all_project):
        raise SystemExit("answer must be at project root")
    current = {key: value for key, value in all_project.items() if key != answer_key}
    expected = json.loads((trial / "initial-manifest.json").read_text())
    expected_dirs = {str(parent) for name in expected for parent in Path(name).parents if str(parent) != "."}
    actual_dirs = {str(p.relative_to(project)) for p in project.rglob("*") if p.is_dir()}
    if actual_dirs != expected_dirs:
        raise SystemExit("trial directories changed")
    if {key: value for key, value in current.items() if key != "answer.json"} != {key: value for key, value in expected.items() if key != "answer.json"}:
        raise SystemExit("trial inputs changed")
    if condition == "with-skill":
        marker = frozen / "skill.sha256"
        if not marker.is_file() or tree_hash(root / "frozen-skill") != marker.read_text().strip(): raise SystemExit("skill snapshot changed")


def grade_answer(case_id: str, answer: dict[str, Any], oracle: dict[str, Any]) -> dict[str, Any]:
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
    if sorted(sources) != sorted(required.get("sources", [])): failures.append("source attribution")
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


def grade_trial(root: Path, case_id: str, condition: str) -> dict[str, Any]:
    verify_frozen(root, case_id, condition)
    trial = root / f"{case_id}-{condition}"; answer_path = trial / "project" / "answer.json"
    if not answer_path.is_file(): return {"case": case_id, "condition": condition, "status": "no_answer", "passed": False, "failures": ["missing answer.json"]}
    try: answer = json.loads(answer_path.read_text())
    except json.JSONDecodeError: return {"case": case_id, "condition": condition, "status": "invalid_answer", "passed": False, "failures": ["invalid JSON"]}
    if not isinstance(answer, dict): return {"case": case_id, "condition": condition, "status": "invalid_answer", "passed": False, "failures": ["answer must be an object"]}
    result = grade_answer(case_id, answer, json.loads((root / "frozen" / "oracle.json").read_text()))
    claimed_pass = answer.get("review") == "PASS"
    read_observed = False
    events_path = trial / "events.jsonl"
    events = []
    skill_body = (root / "frozen-skill" / "SKILL.md").read_text().strip()
    if events_path.is_file():
        for line in events_path.read_text(errors="replace").splitlines():
            try: event = json.loads(line)
            except json.JSONDecodeError: continue
            if not isinstance(event, dict): continue
            events.append(event)
            item = event.get("item") if isinstance(event, dict) else None
            if event.get("type") == "item.completed" and isinstance(item, dict) and item.get("type") == "command_execution":
                # Output containing the exact frozen body establishes a read
                # observation, not native activation or useful application.
                if item.get("exit_code") == 0 and skill_body in str(item.get("aggregated_output", "")):
                    read_observed = True
    native = json.loads((trial / "result.json").read_text()) if (trial / "result.json").is_file() else {}
    final_manifest = {str(p.relative_to(trial / "project")): digest(p) for p in (trial / "project").rglob("*") if p.is_file()}
    bound = events_path.is_file() and (trial / "stderr.log").is_file() and native.get("events_sha256") == digest(events_path) and native.get("stderr_sha256") == digest(trial / "stderr.log") and native.get("final_manifest") == final_manifest
    execution = bound and any(event.get("type") == "turn.completed" for event in events) and native.get("exit_code") == 0 and native.get("turn_completed") is True and native.get("timed_out") is False and native.get("interrupted") is False
    contaminated = condition == "no-skill" and read_observed
    result.update(condition=condition, status="graded", selection_observed=None, skill_read_observed=read_observed, condition_contaminated=contaminated, execution_observed=execution, outcome_observed=(execution and result["passed"] and not contaminated), reviewer_claim=answer.get("review"), reviewer_consistent=(not claimed_pass or result["passed"]), limits="trusted local records; skill read is not native activation or causal benefit")
    write_json(trial / "grade.json", result)
    return result


def run_native(root: Path, case_id: str, condition: str, codex: Path, auth: Path) -> dict[str, Any]:
    started = time.monotonic()
    verify_frozen(root, case_id, condition)
    trial = root / f"{case_id}-{condition}"; project = trial / "project"
    identity = json.loads((root / "frozen" / "platform.json").read_text())
    if str(codex) != identity["codex"] or digest(codex) != identity["codex_sha256"]:
        raise SystemExit("Codex executable differs from freeze")
    state = cohort_budget(root)
    remaining = state["deadline_at"] - time.time()
    if remaining <= 0: raise SystemExit("shared wall budget exhausted")
    if (project / "answer.json").exists(): raise SystemExit("answer exists before execution")
    if auth.is_symlink() or not auth.is_file(): raise SystemExit("auth source must be a regular file")
    with (trial / "started.json").open("x") as marker:
        json.dump({"started_at": time.time()}, marker)
    auth_copy = trial / "codex-home" / "auth.json"
    env = {**os.environ, "HOME": str(trial / "user-home"), "CODEX_HOME": str(trial / "codex-home"), "TMPDIR": str(trial / "tmp"), "TMP": str(trial / "tmp")}
    args = [str(codex), "exec", "--strict-config", "--ignore-user-config", "--ignore-rules", "--ephemeral", "--skip-git-repo-check", "--approve-for-me", "--color", "never", "--json", "--model", MODEL, "-C", str(project), "-c", f'model_reasoning_effort="{REASONING_EFFORT}"', "-"]
    timed_out = False; interrupted = False; process = None; stdout = b""; stderr = b""
    try:
        cache = root / "frozen/models_cache.json"
        if cache.exists():
            shutil.copyfile(cache, trial / "codex-home/models_cache.json")
            (trial / "codex-home/models_cache.json").chmod(0o600)
        shutil.copy2(auth, auth_copy); auth_copy.chmod(0o600)
        remaining = min(state["deadline_at"] - time.time(), PER_CELL_SECONDS - (time.monotonic() - started))
        if remaining <= 0: raise SystemExit("cell budget exhausted during setup")
        process = subprocess.Popen(args, cwd=project, env=env, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        stdout, stderr = process.communicate((trial / "prompt.txt").read_bytes(), timeout=max(0.001, min(state["deadline_at"] - time.time(), PER_CELL_SECONDS - (time.monotonic() - started))))
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as exc:
        timed_out = isinstance(exc, subprocess.TimeoutExpired)
        interrupted = isinstance(exc, KeyboardInterrupt)
        if timed_out: stdout = exc.output or b""; stderr = exc.stderr or b""
        if process is not None:
            try: os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError: pass
            try: stdout, stderr = process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                try: os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError: pass
                stdout, stderr = process.communicate(timeout=5)
    finally:
        if process is not None and process.poll() is None:
            try: os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            process.wait(timeout=5)
        auth_copy.unlink(missing_ok=True)
    (trial / "events.jsonl").write_bytes(stdout); (trial / "stderr.log").write_bytes(stderr)
    turn_completed = False
    for line in stdout.decode(errors="replace").splitlines():
        try:
            if json.loads(line).get("type") == "turn.completed": turn_completed = True
        except (json.JSONDecodeError, AttributeError): pass
    result = {"case": case_id, "condition": condition, "model": MODEL, "reasoning_effort": REASONING_EFFORT, "timed_out": timed_out, "interrupted": interrupted, "turn_completed": turn_completed, "exit_code": process.returncode if process is not None else None, "elapsed_seconds": round(time.monotonic() - started, 3), "events_sha256": digest(trial / "events.jsonl"), "stderr_sha256": digest(trial / "stderr.log"), "final_manifest": {str(p.relative_to(project)): digest(p) for p in project.rglob("*") if p.is_file()}}
    write_json(trial / "result.json", result); return result


def run_cohort(root: Path, codex: Path, auth: Path) -> list[dict[str, Any]]:
    """Run every frozen cell under one wall clock and zero-retry budget."""
    if (root / "cohort-state.json").exists(): raise SystemExit("cohort already started; refusing retry")
    started = time.monotonic(); started_at = time.time(); deadline_at = started_at + SHARED_WALL_SECONDS; results = []
    with (root / "cohort-state.json").open("x") as output:
        json.dump({"started_at": started_at, "deadline_at": deadline_at, "model": MODEL, "reasoning_effort": REASONING_EFFORT, "repairs": 0, "retries": 0}, output)
    (root / "cohort-state.sha256").write_text(digest(root / "cohort-state.json") + "\n")
    for case_id in CASES:
        for condition in CONDITIONS:
            if time.monotonic() - started >= SHARED_WALL_SECONDS:
                raise SystemExit("shared wall budget exhausted before cohort completion")
            results.append(run_native(root, case_id, condition, codex, auth))
            stderr = (root / f"{case_id}-{condition}" / "stderr.log").read_text(errors="replace")
            if "429" in stderr or "Too Many Requests" in stderr:
                write_json(root / "cohort-result.json", {"status": "stopped-provider-rate-limit", "elapsed_seconds": round(time.monotonic()-started, 3), "results": results})
                return results
            latest = results[-1]
            if latest["interrupted"] or latest["timed_out"] or latest["exit_code"] != 0 or not latest["turn_completed"]:
                write_json(root / "cohort-result.json", {"status": "stopped-incomplete-execution", "elapsed_seconds": round(time.monotonic()-started, 3), "results": results})
                return results
    write_json(root / "cohort-result.json", {"shared_wall_seconds": SHARED_WALL_SECONDS, "elapsed_seconds": round(time.monotonic() - started, 3), "repairs": 0, "retries": 0, "results": results})
    return results


def main() -> None:
    parser = argparse.ArgumentParser(); parser.add_argument("--mode", choices=("freeze", "cohort", "grade"), required=True); parser.add_argument("--root", type=Path, required=True); parser.add_argument("--case", choices=tuple(CASES)); parser.add_argument("--condition", choices=CONDITIONS); parser.add_argument("--codex", type=Path); parser.add_argument("--auth-file", type=Path); parser.add_argument("--skill", type=Path); parser.add_argument("--model-cache", type=Path, help="Optional current host catalog snapshot; frozen and shared equally by both conditions")
    args = parser.parse_args(); root = args.root.resolve()
    if args.mode == "freeze":
        if not args.codex: raise SystemExit("--codex required")
        freeze(root, args.codex.resolve(), args.skill.resolve() if args.skill else None, args.model_cache.resolve() if args.model_cache else None)
    elif args.mode == "grade":
        if not args.case or not args.condition: raise SystemExit("--case and --condition required")
        print(json.dumps(grade_trial(root, args.case, args.condition), indent=2))
    elif args.mode == "cohort":
        if not args.codex or not args.auth_file: raise SystemExit("--codex and --auth-file required")
        print(json.dumps(run_cohort(root, args.codex.resolve(), args.auth_file.resolve()), indent=2))


if __name__ == "__main__": main()
