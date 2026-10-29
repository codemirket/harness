"""Creative fixtures and artifact-bound independent review validation for pi."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import time
from pathlib import Path
from typing import Any

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

UI_HANDOFF = {
    "responsive-ui": {
        **CASES["responsive-ui"],
        "brief": """# Lantern repair club
Create an original responsive page for a fictional community repair club. Required copy: Lantern repair club; Give everyday things another chapter; Find a session. Facts: Sunday 10:00–14:00; free entry; three sessions named Lamp clinic, Jacket repair and Chair care, with categories Electrical, Textiles and Furniture respectively. Include keyboard-operable All/Electrical/Textiles/Furniture filters and search. An unmatched query shows a useful empty state. Refresh sessions deliberately fails once with an accessible error; Retry restores all three sessions. Preserve display-text descenders, usable narrow layouts and reduced-motion behavior. No invented attendance, ratings or testimonials, external assets or new packages.
""",
    },
}

ACCEPTANCE_SAMPLE = {
    "responsive-ui": {
        **CASES["responsive-ui"],
        "brief": """# Tidepool field station
Create an original, art-directed responsive page for a fictional evening field station. Required copy: Tidepool; Stay curious after sunset; Reserve a place. Facts: Saturday 17:30–20:00; free entry; three sessions named Shore walk, Sky watch and Soil stories, with categories Shore, Sky and Soil respectively. Do not invent attendance, testimonials or ratings. Provide keyboard-operable All/Shore/Sky/Soil filters and session search; an unmatched search shows a useful empty state. Refresh sessions deliberately fails once with an accessible error; Retry restores the session list. Keep the interface usable without motion and protect lowercase descenders in large display text. No external assets or packages.
""",
        "rubric": "Score 0–2 each: distinctive hierarchy, legibility and intact display text; exact facts and working filter/search/empty/error/retry; keyboard operation, responsive bounds and reduced-motion usability at 1280×800 and 390×844. Use actual interactions and saved pixels. Do not score a different art direction as defective merely by preference.",
    },
    "captioned-motion": {
        **CASES["captioned-motion"],
        "brief": """# Tidepool night-watch film
Create an original editable 640×360, 12fps, 36-frame MP4. Frames 0–11 introduce Tidepool with a small star; frames 12–23 move that star into the centre of a telescope lens, with a clearly visible contact pause before the next beat; frames 24–35 hold Stay curious. Captions in the three respective half-open frame ranges: Tidepool / Look a little closer / Stay curious. Keep caption and main-copy regions distinct, including during transitions. Use a coherent night-observation art direction, preserve the y descender and keep the star visually accounted for after contact. No audio, borrowed marks, remote assets or invented metrics.
""",
        "rubric": "Score 0–2 each: readable star approach/lens contact/pause and final hold; coherent hierarchy, intact descenders and captions without collisions through transitions; editable source and correct decoded 36-frame timing. Decode the entire export and inspect 0,6,10,11,12,13,18,22,23,24,25,30,35 plus neighbours of suspected defects. Do not infer object movement from whole-frame differences.",
    },
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
            result[rel] = hashlib.sha256(path.read_bytes()).hexdigest()
        elif not stat.S_ISDIR(mode):
            raise SystemExit(f"special file refused: {path}")
    return result

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
