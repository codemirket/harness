"""Matched study extension for motion-feedback-pilot.py; no production runtime.

Only specialist guidance differs within a pair. Seals are held by the lead
outside author projects; they detect drift, not malicious whole-machine edits.
Process controls belong to native Codex. A structural result is never visual QA.
"""
from __future__ import annotations

import argparse
import contextlib
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import secrets
import shutil
import stat
import time
from typing import Any

SPEC = importlib.util.spec_from_file_location("motion_pilot", Path(__file__).with_name("motion-feedback-pilot.py"))
assert SPEC and SPEC.loader
P = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(P)
REPO = P.REPO
RUNTIME_KEYS = ("codex", "mirket", "node", "adapter", "design_adapter", "chromium", "ffmpeg", "ffprobe")
TREE_KEYS = ("adapter_root", "browser_root")
TOOLS = ["runtime_status", "design_prepare", "design_start", "job_status", "job_cancel",
         "video_prepare", "video_start", "video_status", "video_result", "video_cancel"]
LIMITS = {"initial_seconds": 240, "repair_seconds": 180, "max_repairs": 4,
          "model_seconds": 2700, "wall_seconds": 2700}
SHUTDOWN_RESERVE_SECONDS = 15  # Existing process helper allows two five-second waits.
PROMPT = """Work only in this disposable project. Read BRIEF.md and PRODUCTION.md.
Use relevant project-local skills when present. Preserve every supplied input.
Deliver the requested editable files at the project root; put additional
evidence only under .evidence/. Use the
configured Mirket route for design/video effects; Codex owns shell, editing,
browser, model selection and approvals. Do not use agents, install dependencies,
fetch external assets, read other projects, change configuration, commit or send
messages. Stop after one initial artifact and its checks; no targeted repair
until the lead supplies independent findings in a later turn. Report missing
capabilities or denied permissions exactly. A receipt is not a finished artifact;
inspect saved pixels with an image-capable tool before claiming visual review.
"""
CASES = {
    "responsive-ui": {
        "brief": """# Lumen library\nCreate a distinctive responsive page for a fictional neighbourhood library. Required facts: Tue–Sun 09:00–18:00; free membership; three workshop subjects Print, Repair, Grow. Include a keyboard-operable subject filter, a clear empty state and a deliberately reachable error/retry state. Required copy: Lumen library; Make room for curiosity; Free membership. Do not invent attendance, testimonials or ratings.\n""",
        "skills": ["art-direction", "interface-composition"],
        "outputs": ["index.html", "styles.css", "app.js"],
        "rubric": "Score 0–2 each: distinctive hierarchy and legibility; correct facts and useful filter/empty/error/retry behavior; responsive and keyboard usability at 1280×800 and 390×844. Record actual interactions and saved pixels, not source-only assertions.",
    },
    "illustration": {
        "brief": """# Mending table\nCreate an original editable 640×480 SVG spot illustration for a community repair guide: a seated person threads a needle beside a torn cloth on a work table. The visible action is hand-to-thread-to-needle contact; show a supported seated pose and feet on the floor. Required label: Keep good things going. Palette: paper #f5f0e6, ink #213c41, clay #c87558, leaf #6c8c73, gold #c5a04d. No borrowed characters, external assets or embedded bitmaps.\n""",
        "skills": ["vector-art", "art-direction"],
        "outputs": ["illustration.svg"],
        "rubric": "Score 0–2 each: readable action/contact and supported pose; coherent art direction and hierarchy; editable grouped geometry and full/detail pixel fidelity. Inspect full 640×480, 320×240 and hand/needle detail.",
    },
    "captioned-motion": {
        "brief": """# Fieldnote film\nCreate an editable 640×360, 12fps, 36-frame MP4 for fictional Fieldnote. Frames 0–11 introduce Fieldnote and a leaf mark; frames 12–23 show the leaf travelling to a notebook and a visible contact pause; frames 24–35 hold Start a field journal. Captions must show Fieldnote / Notice the small things / Start a field journal in those respective frame ranges. Keep caption and main-copy regions distinct; no audio or invented metrics.\n""",
        "skills": ["video-production", "motion-direction"],
        "outputs": ["index.html", "film.css", "film.js", "captions.json"],
        "rubric": "Score 0–2 each: intended leaf movement/contact/hold; legible hierarchy and captions without collision; editable source and decoded timing. Decode all 36 frames; inspect frames 0,6,10,11,12,13,18,22,23,24,25,30,35 and neighbours of any suspected defect. Playback and audio claims require their own evidence.",
    },
    "vector-transition": {
        "brief": """# Foldway transition\nCreate a 640×360, 12fps, 36-frame editable vector transition for fictional Foldway. A folded paper plane travels to an outlined envelope, pauses at contact, then the envelope closes while the plane remains visually accounted for. Required copy: Foldway; A note worth keeping. Preserve named plane, envelope, flap and contact SVG groups, with one animation owner per property. No audio or captions; no remote assets or borrowed marks.\n""",
        "skills": ["vector-art", "motion-direction", "video-production"],
        "outputs": ["index.html", "film.css", "film.js", "illustration.svg"],
        "rubric": "Score 0–2 each: approach/contact/closure continuity; coherent vector geometry and legibility; editable groups and preview/export correspondence. Decode all 36 frames; inspect 0,6,12,18,24,30,35 and neighbours at contact/closure. Do not infer intended object motion from whole-frame changes.",
    },
}
CALIBRATION = {
    "route-check": {
        "brief": "# Route check\nCreate illustration.svg, a 160×100 SVG containing a solid #176b65 circle centred at (80,50) with radius 30 on a #f4efe4 background. Use the configured Mirket SVG-to-PNG operation to produce .evidence/preview.png, then open that saved PNG if an image-capable tool is available. No typography or additional artwork. Record exact capability/approval refusal if blocked.\n",
        "skills": ["vector-art"], "outputs": ["illustration.svg"],
        "rubric": "Setup qualification only: saved editable SVG, real Mirket PNG effect and independent pixel check. No aesthetic score or skill-benefit inference.",
    }
}


def canonical(value: Any) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()).hexdigest()


def read(path: Path) -> Any:
    return json.loads(path.read_text())


def tree(root: Path, *, links: bool = False) -> dict[str, str]:
    """Materialized inputs reject links; admitted runtime trees hash internal links."""
    if not root.is_dir() or root.is_symlink():
        raise SystemExit(f"expected real directory: {root}")
    result = {}
    for path in sorted(root.rglob("*")):
        rel = path.relative_to(root).as_posix()
        mode = path.lstat().st_mode
        if stat.S_ISLNK(mode):
            if not links or not path.resolve().is_relative_to(root.resolve()) or not path.exists():
                raise SystemExit(f"symlink refused: {path}")
            result[rel] = "link:" + os.readlink(path)
        elif stat.S_ISREG(mode):
            result[rel] = P.sha(path)
        elif not stat.S_ISDIR(mode):
            raise SystemExit(f"special file refused: {path}")
    return result


def bindings(ready: dict[str, Any]) -> dict[str, Any]:
    result = {}
    for key in RUNTIME_KEYS:
        path = Path(ready[key])
        if not path.is_absolute() or not path.is_file() or path.is_symlink():
            raise SystemExit(f"runtime file unavailable: {key}")
        result[key] = {"path": str(path), "sha256": P.sha(path)}
    for key in TREE_KEYS:
        path = Path(ready[key])
        if not path.is_absolute():
            raise SystemExit("runtime roots must be absolute")
        result[key] = {"path": str(path), "tree_sha256": canonical(tree(path, links=True))}
    for key in ("adapter", "design_adapter"):
        if not Path(ready[key]).resolve().is_relative_to(Path(ready["adapter_root"]).resolve()):
            raise SystemExit("adapter entry is outside its frozen runtime tree")
    if not Path(ready["chromium"]).resolve().is_relative_to(Path(ready["browser_root"]).resolve()):
        raise SystemExit("browser entry is outside its frozen runtime tree")
    return result


def config(ready: dict[str, Any], trial: Path, font: Path) -> str:
    args = ["dev", "serve", "--state-dir", str(trial / "mirket-state"), "--release-id", "matched-design",
            "--node", ready["node"], "--design-adapter", ready["design_adapter"], "--design-font", str(font),
            "--video-adapter", ready["adapter"], "--video-chromium", ready["chromium"],
            "--video-ffmpeg", ready["ffmpeg"], "--video-ffprobe", ready["ffprobe"]]
    return (f'model = {json.dumps(ready["model"])}\nmodel_reasoning_effort = {json.dumps(ready["reasoning_effort"])}\n'
            '[mcp_servers.mirket]\n' + f'command = {json.dumps(ready["mirket"])}\nargs = {json.dumps(args)}\n'
            f'enabled_tools = {json.dumps(TOOLS)}\nrequired = true\nstartup_timeout_sec = 60\n')


def production(case: dict[str, Any], ready: dict[str, Any]) -> str:
    return f"""# Common producer contract
Required deliverables at the project root: {', '.join(case['outputs'])}. Additional evidence belongs
only under .evidence/. Preserve BRIEF.md, PRODUCTION.md, RUBRIC.md, NotoSans.ttf
and any supplied .agents/ files. Use only local editable source and the supplied
font. No new packages, network assets, fallback renderer, agents or extra trials.
Call runtime_status before design/video work; use its exact release identity
and this project's absolute root. Design uses design_prepare/design_start and
job_status; video uses video_prepare/video_start/video_status/video_result.
Revision tokens use `sha256:<lowercase hex digest>`, including expected_revision
and selected video asset revisions. A bare shasum digest is not a revision token.
Codex/project tools own UI browser interaction. Stop and report a missing route.
Use exact Node {ready['node']}, FFmpeg {ready['ffmpeg']}, FFprobe {ready['ffprobe']}.

For video only: HyperFrames HTML must have a composition root with stable
data-composition-id, literal data-no-timeline, data-width="640", data-height="360"
and data-duration="3". Use a full HTML body with closing </body>, local film.css
and film.js. Declare and apply a relative @font-face for NotoSans.ttf. Use finite
deterministic CSS/WAAPI animation compatible with adapter seeking. Select every
referenced asset by its current revision; do not change source during a job.
The export is MP4 at 12fps/36 frames, without audio. captions.json, when required,
is a top-level array with exactly start_frame, end_frame, text per cue; start is
inclusive, end exclusive. Put output MP4 and decoded frames under .evidence/.
SVG remains editable vector geometry; rendered previews do not replace source.

Initial turn ceiling: 240 seconds; repairs require a later lead-issued finding.
The whole study shares 45 minutes including reviews, 2700 model seconds and
four 180-second repair turns. New specimens/delegates do not reset these limits.
The lead checks artifacts independently. Report actual saved output and limits;
missing image events in a CLI trace alone do not establish lack of inspection.
"""


def prepare(root: Path, ready_path: Path, font: Path, seal: Path, calibration: bool = False) -> str:
    root, seal = root.absolute(), seal.absolute()
    if root.exists() or seal.exists() or seal.is_relative_to(root) or root.is_relative_to(REPO):
        raise SystemExit("use a new disposable root outside the repository and a new external seal")
    if any((parent / "AGENTS.md").exists() for parent in root.parents):
        raise SystemExit("disposable study has ancestor AGENTS.md guidance")
    ready = read(ready_path)
    P.check_ready(ready)
    if ready.get("approval_review") != "auto_review":
        raise SystemExit("matched trials require the authorized automatic approval review")
    runtime = bindings(ready)
    cases = CALIBRATION if calibration else CASES
    root.mkdir(parents=True, mode=0o700)
    frozen = root / "frozen"; frozen.mkdir()
    (frozen / "runner").mkdir()
    for source in (Path(__file__), Path(P.__file__)):
        shutil.copyfile(source, frozen / "runner" / source.name)
    shutil.copy2(font, frozen / "NotoSans.ttf")
    names = sorted({name for case in cases.values() for name in case["skills"]})
    for name in names:
        tree(REPO / "skills" / name)
        shutil.copytree(REPO / "skills" / name, frozen / "skills" / name)
    limits = dict(LIMITS)
    if calibration:
        limits.update(initial_seconds=120, max_repairs=0, model_seconds=270, wall_seconds=600)
    cells = []
    for index, (task, case) in enumerate(cases.items()):
        # Alternate condition order across briefs; random IDs mask labels from reviewers.
        for condition in (("skill", "no-skill") if index % 2 == 0 else ("no-skill", "skill")):
            cid = "artifact-" + secrets.token_hex(4)
            trial = root / "trials" / cid
            project = trial / "project"
            for folder in (project, trial / "user-home", trial / "codex-home", trial / "tmp", trial / "mirket-state"):
                folder.mkdir(parents=True, mode=0o700)
            (project / "BRIEF.md").write_text(case["brief"])
            common = production(case, ready)
            if calibration:
                common = common.replace("Initial turn ceiling: 240 seconds", "Initial turn ceiling: 120 seconds").replace("45 minutes including reviews, 2700 model seconds and\nfour 180-second repair turns", "10 minutes including reviews, 270 model seconds and\nzero repair turns")
            (project / "PRODUCTION.md").write_text(common)
            (project / "RUBRIC.md").write_text(case["rubric"] + "\n")
            shutil.copy2(frozen / "NotoSans.ttf", project / "NotoSans.ttf")
            if condition == "skill":
                for name in case["skills"]:
                    shutil.copytree(frozen / "skills" / name, project / ".agents" / "skills" / name)
            (trial / "prompt.txt").write_text(PROMPT)
            (trial / "codex-home/config.toml").write_text(config(ready, trial, frozen / "NotoSans.ttf"))
            cells.append({"id": cid, "task": task, "condition": condition, "inputs": tree(project),
                          "prompt_sha256": P.sha(trial / "prompt.txt"),
                          "config_sha256": P.sha(trial / "codex-home/config.toml")})
    protocol = {"schema_version": 1, "kind": "calibration" if calibration else "cohort", "cases": cases,
                "limits": limits, "cells": cells, "ready": {k: v for k, v in ready.items() if k != "auth_file"},
                "runtime": runtime, "platform": platform.platform(), "tools": TOOLS,
                "head": P.subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
                "runner": {str(p): P.sha(p) for p in (Path(__file__), Path(P.__file__))},
                "repair_allocation": "Collect all initial reviews first; blocking before major, then frozen cell order; at most one repair per cell. No retry of timeout, denial or failed launch.",
                "masking_limit": "Neutral IDs and excluded guidance/logs; authored source or style may reveal condition. No authenticated blindness or filesystem sandbox."}
    P.write_json(frozen / "protocol.json", protocol)
    # Materialized freeze outside the mutable projects; the lead retains its digest.
    freeze = {"root": str(root), "frozen": tree(frozen)}
    seal.parent.mkdir(parents=True, exist_ok=True)
    P.write_json(seal, freeze)
    P.write_json(root / "budget-ledger.json", {"started_at": None, "deadline_at": None, "model_seconds_used": 0.0,
                                              "model_seconds_reserved": 0, "repairs_used": 0, "runs": {}})
    return P.sha(seal)


def verify(root: Path, seal: Path, seal_sha: str, ready_path: Path) -> tuple[dict, dict]:
    if P.sha(seal) != seal_sha or seal.is_relative_to(root):
        raise SystemExit("external seal changed or is inside author study")
    lock = read(seal)
    if lock["root"] != str(root.absolute()) or lock["frozen"] != tree(root / "frozen"):
        raise SystemExit("frozen study changed")
    protocol, ready = read(root / "frozen/protocol.json"), read(ready_path)
    P.check_ready(ready)
    if {k: v for k, v in ready.items() if k != "auth_file"} != protocol["ready"] or bindings(ready) != protocol["runtime"]:
        raise SystemExit("runtime/settings changed after freeze")
    if any(P.sha(Path(path)) != digest for path, digest in protocol["runner"].items()):
        raise SystemExit("study runner changed after freeze")
    return protocol, ready


def inspect_project(root: Path, cell: dict, case: dict, *, before: bool = False) -> dict:
    trial = root / "trials" / cell["id"]; project = trial / "project"
    if P.sha(trial / "prompt.txt") != cell["prompt_sha256"] or P.sha(trial / "codex-home/config.toml") != cell["config_sha256"]:
        raise SystemExit("prompt or tool configuration changed")
    actual = tree(project)
    if before and actual != cell["inputs"]:
        raise SystemExit("initial project changed")
    for path, digest in cell["inputs"].items():
        if actual.get(path) != digest:
            raise SystemExit(f"protected input changed: {path}")
    extra = [p for p in actual if p not in cell["inputs"] and p not in case["outputs"] and not p.startswith(".evidence/")]
    # Empty directories also matter, especially accidental guidance injection.
    allowed_files = set(cell["inputs"]) | set(case["outputs"])
    for path in project.rglob("*"):
        if path.is_dir():
            rel = path.relative_to(project).as_posix()
            if rel != ".evidence" and not rel.startswith(".evidence/") and not any(p.startswith(rel + "/") for p in allowed_files):
                extra.append(rel + "/")
    if extra:
        raise SystemExit("out-of-scope output: " + ", ".join(sorted(extra)))
    missing = sorted(set(case["outputs"]) - set(actual))
    return {"manifest": actual, "missing_outputs": missing, "structure_only": True,
            "visual_quality": "unqualified; independent artifact review required"}


@contextlib.contextmanager
def locked(root: Path):
    with (root / "study.lock").open("a+") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit("another study phase is active")
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def reserve(root: Path, protocol: dict, cid: str, phase: str) -> int:
    if (root / "study-result.json").exists(): raise SystemExit("study already finalized")
    ledger = read(root / "budget-ledger.json"); limits = protocol["limits"]
    if cid not in {c["id"] for c in protocol["cells"]} or phase not in {"initial", "repair"}:
        raise SystemExit("unknown cell or phase; new specimens do not reset budgets")
    runs = ledger["runs"].setdefault(cid, {})
    if phase in runs or (phase == "repair" and runs.get("initial", {}).get("status") != "finished"):
        raise SystemExit("duplicate phase or repair before finished initial")
    if phase == "repair" and ledger["repairs_used"] >= limits["max_repairs"]:
        raise SystemExit("shared repair budget exhausted")
    if phase == "initial":
        for cell in protocol["cells"]:
            if cell["id"] == cid: break
            if ledger["runs"].get(cell["id"], {}).get("initial", {}).get("status") != "finished":
                raise SystemExit("initial turns must follow the frozen cell order")
    now = time.time()
    if ledger["started_at"] is None:
        ledger["started_at"], ledger["deadline_at"] = now, now + limits["wall_seconds"]
    if ledger["deadline_at"] != ledger["started_at"] + limits["wall_seconds"]:
        raise SystemExit("shared deadline changed")
    if now < ledger["started_at"]:
        raise SystemExit("clock moved before the shared study start")
    seconds = limits[phase + "_seconds"]
    reservation = seconds + SHUTDOWN_RESERVE_SECONDS
    if now + reservation > ledger["deadline_at"]:
        raise SystemExit("shared wall window cannot admit the full turn")
    if ledger["model_seconds_used"] + ledger["model_seconds_reserved"] + reservation > limits["model_seconds"]:
        raise SystemExit("shared model budget exhausted")
    ledger["model_seconds_reserved"] += reservation
    if phase == "repair": ledger["repairs_used"] += 1
    runs[phase] = {"status": "started", "at": now, "reserved": reservation, "turn_ceiling": seconds}
    P.write_json_atomic(root / "budget-ledger.json", ledger)
    return seconds


def settle(root: Path, cid: str, phase: str, elapsed: float, result: dict) -> None:
    ledger = read(root / "budget-ledger.json")
    ledger["model_seconds_reserved"] -= ledger["runs"][cid][phase]["reserved"]
    ledger["model_seconds_used"] += elapsed
    ledger["runs"][cid][phase].update(status="finished", elapsed_seconds=elapsed,
                                     completed_at=time.time(), eligible_for_repair=result.get("author_completed", False))
    P.write_json_atomic(root / "budget-ledger.json", ledger)


def validate_review(root: Path, protocol: dict, cell: dict, phase: str) -> dict:
    ledger = read(root / "budget-ledger.json")
    run = ledger["runs"].get(cell["id"], {}).get(phase, {})
    name = cell["id"] + ("" if phase == "initial" else "-repair") + ".json"
    path = root / "reviews" / name
    if run.get("status") != "finished" or not path.is_file():
        raise SystemExit("collect results and neutral reviews before allocation/finalization")
    review = read(path)
    pack_name = cell["id"] + "-" + phase
    pack = root / "reviewer" / pack_name
    pack_record = root / "review-packs" / (pack_name + ".json")
    if not pack_record.is_file():
        raise SystemExit("review requires its retained neutral pack")
    pack_meta = read(pack_record)
    if tree(pack) != pack_meta["manifest"]:
        raise SystemExit("neutral review pack changed")
    template = read(pack / "review-template.json")
    if any(review.get(key) != template[key] for key in ("artifact_id", "phase", "manifest_sha256", "rubric_sha256")):
        raise SystemExit("review differs from its neutral pack binding")
    snapshot = root / "trials" / cell["id"] / (phase + "-project-snapshot")
    saved = tree(snapshot)
    if saved != read(root / "trials" / cell["id"] / (phase + "-project-manifest.json")):
        raise SystemExit("reviewed artifact snapshot changed")
    if review.get("artifact_id") != cell["id"] or review.get("phase") != phase or review.get("manifest_sha256") != canonical(saved):
        raise SystemExit("review does not bind its phase snapshot")
    if review.get("rubric_sha256") != canonical(protocol["cases"][cell["task"]]["rubric"]):
        raise SystemExit("review criterion changed")
    if review.get("status") != "reviewed" or not isinstance(review.get("reviewed_by"), str) or not review["reviewed_by"].strip():
        raise SystemExit("review is still unreviewed or lacks reviewer attestation")
    at = review.get("reviewed_at")
    if not isinstance(at, (int, float)) or isinstance(at, bool) or not max(run["completed_at"], pack_meta["created_at"]) <= at <= min(time.time(), ledger["deadline_at"]):
        raise SystemExit("review lies outside the post-author shared review window")
    inspected = review.get("inspected_artifacts")
    if not isinstance(inspected, list) or not inspected or any(not isinstance(p, str) or p not in saved or p not in pack_meta["manifest"] for p in inspected):
        raise SystemExit("review must identify saved inspected artifacts")
    findings = review.get("findings")
    if not isinstance(findings, list): raise SystemExit("review requires findings array")
    seen = set()
    for finding in findings:
        if not isinstance(finding, dict) or finding.get("severity") not in {"minor", "major", "blocking"} or not all(isinstance(finding.get(k), str) and finding[k].strip() for k in ("id", "location", "consequence")) or finding.get("artifact_path") not in inspected:
            raise SystemExit("review finding lacks severity/artifact/location/consequence")
        if finding["id"] in seen: raise SystemExit("duplicate review finding id")
        seen.add(finding["id"])
    return review


def repair_review(root: Path, protocol: dict, cid: str) -> dict:
    ledger = read(root / "budget-ledger.json")
    ranked = []
    for order, cell in enumerate(protocol["cells"]):
        run = ledger["runs"].get(cell["id"], {})
        if run.get("initial", {}).get("status") != "finished":
            raise SystemExit("collect all initial results before repair allocation")
        if not run["initial"].get("eligible_for_repair"):
            continue  # Denied/failed cells remain failed, not candidates for a retry.
        review = validate_review(root, protocol, cell, "initial")
        findings = review["findings"]
        actionable = [f for f in findings if f["severity"] in {"major", "blocking"}]
        if actionable and run["initial"].get("eligible_for_repair") and "repair" not in run:
            rank = 0 if any(f["severity"] == "blocking" for f in actionable) else 1
            ranked.append((rank, order, cell["id"], review))
    ranked.sort(key=lambda row: row[:2])
    if not ranked or ranked[0][2] != cid:
        raise SystemExit("cell is not next in frozen severity/order repair allocation")
    return ranked[0][3]


def run(root: Path, seal: Path, seal_sha: str, ready_path: Path, cid: str, phase: str = "initial") -> dict:
    with locked(root):
        protocol, ready = verify(root, seal, seal_sha, ready_path)
        cell = next((c for c in protocol["cells"] if c["id"] == cid), None)
        if cell is None: raise SystemExit("unknown frozen cell")
        case = protocol["cases"][cell["task"]]; trial = root / "trials" / cid
        for directory in (root, root / "trials", trial, *(trial / name for name in ("codex-home", "user-home", "tmp", "mirket-state"))):
            if directory.is_symlink() or not directory.is_dir():
                raise SystemExit("disposable session directory changed")
        inspect_project(root, cell, case, before=phase == "initial")
        prompt = (trial / "prompt.txt").read_text()
        if phase == "repair":
            if tree(trial / "project") != read(trial / "initial-project-manifest.json"):
                raise SystemExit("current project differs from reviewed initial artifact")
            review = repair_review(root, protocol, cid)
            P.write_json(trial / "repair-findings.json", review)
            prompt += "\nOne targeted repair; preserve the brief and fix only these independently recorded findings:\n" + json.dumps(review["findings"])
        seconds = reserve(root, protocol, cid, phase)
        auth = trial / "codex-home/auth.json"
        started = time.monotonic()
        result: dict[str, Any] = {"author_completed": False}
        model_elapsed = 0.0
        auth_directory = None
        auth_created = False
        try:
            auth_directory = os.open(auth.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            # Exclusive creation refuses existing or dangling symlinks. Cleanup
            # uses the opened directory even if its pathname changes in a turn.
            descriptor = os.open("auth.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=auth_directory)
            auth_created = True
            with os.fdopen(descriptor, "wb") as output, Path(ready["auth_file"]).open("rb") as source:
                shutil.copyfileobj(source, output)
            P.snapshot(trial, "before-" + phase)
            model_start = time.monotonic()
            try:
                result = P.run_model(trial, prompt, phase, ready, cell["condition"], seconds)
            finally:
                model_elapsed = time.monotonic() - model_start
            # Host completion, not output quality. Structural absence is checked separately.
            result["author_completed"] = result["exit_code"] == 0 and result.get("turn_completed", False) and not result["timed_out"] and not result.get("interrupted") and not result["approval_signals"] and not result["blocked_signals"]
            result["project_check"] = inspect_project(root, cell, case)
            verify(root, seal, seal_sha, ready_path)
        except (Exception, SystemExit, KeyboardInterrupt) as exc:
            result.update(author_completed=False, runner_error=f"{type(exc).__name__}: {exc}")
        finally:
            if auth_directory is not None:
                try:
                    if auth_created:
                        try: os.unlink("auth.json", dir_fd=auth_directory)
                        except FileNotFoundError: pass
                finally:
                    os.close(auth_directory)
            elapsed = round(time.monotonic() - started, 3)
            ledger = read(root / "budget-ledger.json")
            if model_elapsed > seconds + SHUTDOWN_RESERVE_SECONDS or ledger["model_seconds_used"] + model_elapsed > protocol["limits"]["model_seconds"] or time.time() > ledger["deadline_at"]:
                result.update(author_completed=False, budget_overrun=True)
            try:
                P.snapshot(trial, phase)
            except (Exception, SystemExit) as exc:
                result.update(author_completed=False, snapshot_error=str(exc))
            result.update(total_elapsed_seconds=elapsed, model_elapsed_seconds=model_elapsed, phase=phase, artifact_id=cid,
                          outcome="awaiting_independent_review" if result.get("author_completed") else "failed_or_blocked")
            P.write_json(trial / (phase + "-result.json"), result)
            settle(root, cid, phase, model_elapsed, result)
        return result


def review_pack(root: Path, seal: Path, seal_sha: str, ready_path: Path, cid: str, phase: str = "initial") -> Path:
    with locked(root):
        protocol, _ = verify(root, seal, seal_sha, ready_path)
        cell = next((c for c in protocol["cells"] if c["id"] == cid), None)
        if cell is None or phase not in {"initial", "repair"}: raise SystemExit("unknown cell/phase")
        ledger = read(root / "budget-ledger.json")
        if ledger["deadline_at"] is None or time.time() > ledger["deadline_at"]:
            raise SystemExit("review window expired or not started; retain raw evidence without extending it")
        trial = root / "trials" / cid; snapshot = trial / (phase + "-project-snapshot")
        saved = read(trial / (phase + "-project-manifest.json"))
        if tree(snapshot) != saved: raise SystemExit("saved artifact snapshot changed")
        dest = root / "reviewer" / (cid + "-" + phase)
        if dest.exists(): raise SystemExit("review pack already exists")
        dest.mkdir(parents=True)
        case = protocol["cases"][cell["task"]]
        for name in ["BRIEF.md", "PRODUCTION.md", "RUBRIC.md", *case["outputs"], "NotoSans.ttf"]:
            if (snapshot / name).is_file(): shutil.copy2(snapshot / name, dest / name)
        # Retain visual assets/exports (including SVG referenced by a page),
        # not reports, guidance, trace or author claims.
        for path in (snapshot / ".evidence").rglob("*") if (snapshot / ".evidence").is_dir() else []:
            if path.is_file() and path.suffix.lower() in {".svg", ".png", ".jpg", ".webp", ".mp4", ".webm"}:
                target = dest / path.relative_to(snapshot); target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(path, target)
        P.write_json(dest / "review-template.json", {"artifact_id": cid, "phase": phase, "manifest_sha256": canonical(saved),
                     "rubric_sha256": canonical(case["rubric"]), "findings": [],
                     "status": "unreviewed", "inspected_artifacts": [],
                     "masking_limit": "source/comments/style may reveal guidance; not authenticated blindness"})
        (root / "review-packs").mkdir(exist_ok=True)
        P.write_json(root / "review-packs" / (dest.name + ".json"), {"manifest": tree(dest), "created_at": time.time()})
        return dest


def finalize(root: Path, seal: Path, seal_sha: str, ready_path: Path) -> dict:
    """Retain incomplete/failed cells too; never turn a review attestation into a quality pass."""
    with locked(root):
        protocol, _ = verify(root, seal, seal_sha, ready_path)
        if (root / "study-result.json").exists(): raise SystemExit("study already finalized")
        ledger = read(root / "budget-ledger.json")
        rows = []
        for cell in protocol["cells"]:
            phases = ledger["runs"].get(cell["id"], {})
            row = {"artifact_id": cell["id"], "task": cell["task"], "condition": cell["condition"], "phases": {}}
            for phase in ("initial", "repair"):
                if phase not in phases: continue
                if phases[phase]["status"] != "finished": raise SystemExit("cannot finalize an interrupted/unsettled phase")
                result = read(root / "trials" / cell["id"] / (phase + "-result.json"))
                item = {"outcome": result["outcome"]}
                if result["author_completed"]:
                    review = validate_review(root, protocol, cell, phase)
                    item.update(findings=review["findings"], review_sha256=canonical(review), visual_quality="review_attestation_only")
                row["phases"][phase] = item
            row["status"] = "not_run" if not row["phases"] else "recorded"
            rows.append(row)
        value = {"status": "recorded_not_quality_certified", "cells": rows, "budget": ledger,
                 "frozen_seal_sha256": seal_sha, "model_benefit": "not_established_by_runner"}
        P.write_json(root / "study-result.json", value)
        return value


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "verify", "initial", "repair", "review-pack", "finalize"))
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--ready-manifest", type=Path, required=True)
    parser.add_argument("--seal", type=Path, required=True)
    parser.add_argument("--seal-sha256")
    parser.add_argument("--font", type=Path)
    parser.add_argument("--calibration", action="store_true")
    parser.add_argument("--cell")
    parser.add_argument("--phase", choices=("initial", "repair"), default="initial")
    args = parser.parse_args(argv)
    if args.command == "prepare":
        if args.font is None: parser.error("prepare requires --font")
        print(json.dumps({"seal_sha256": prepare(args.root, args.ready_manifest, args.font, args.seal, args.calibration)}))
    else:
        if not args.seal_sha256: parser.error("retain and supply --seal-sha256 from preparation")
        if args.command == "verify":
            protocol, _ = verify(args.root, args.seal, args.seal_sha256, args.ready_manifest)
            print(json.dumps({"status": "frozen_inputs_match", "cells": len(protocol["cells"])}))
        elif args.command == "finalize":
            print(json.dumps(finalize(args.root, args.seal, args.seal_sha256, args.ready_manifest)))
        elif not args.cell:
            parser.error("execution/review requires --cell")
        elif args.command == "review-pack":
            print(review_pack(args.root, args.seal, args.seal_sha256, args.ready_manifest, args.cell, args.phase))
        else:
            result = run(args.root, args.seal, args.seal_sha256, args.ready_manifest, args.cell, args.command)
            print(json.dumps(result))
            return 0 if result["author_completed"] else 1
    return 0
