"""Retained domain fixtures and independent oracles for pi qualification.

No host launch, authentication, configuration or legacy transcript handling.
"""

from __future__ import annotations

import hashlib

from pathlib import Path

import shutil

CASES = {
    "product-page": {
        "prompt": """Work only in this disposable project. Create a distinctive, responsive product page for a fictional public transit service using the exact facts and copy in BRIEF.md. Deliver index.html, styles.css and app.js. Include one interactive route filter component with default, hover, focus-visible, active, empty and error states; keep keyboard order useful. Use no external assets, packages, network or invented facts. Save a local page-preview.png using an available local renderer if possible, and inspect the saved pixels if an image-capable tool is available. Preserve BRIEF.md. No agents, dependencies, credentials, messages, commits or external effects.""",
        "files": {"BRIEF.md": "# Driftline\nDriftline is a fictional public transit service. Facts: weekday service 05:30–23:45; three lines named Harbor, Orchard and North Loop; day pass $6; wheelchair-accessible vehicles on every line. Required copy: Driftline; Move with the tide; Choose a line; Harbor; Orchard; North Loop; Day pass $6; Accessible vehicles on every line. Do not invent ridership, performance or customer claims.\n"},
        "skills": ["art-direction", "interface-composition"],
        "budget": {"wall_seconds": 180, "max_repairs": 1, "shared_across": ["lead", "reviewer", "delegate", "specimen", "repair"]},
        "oracle": "page",
    },
    "illustration": {
        "prompt": """Work only in this disposable project. Create an original editable SVG illustration for the exact scene in BRIEF.md. Deliver illustration.svg and a local preview.png rendered from the saved SVG when possible. The person must visibly water a plant while standing beside a stool: show a convincing weight-bearing foot, hand-to-can grip, bent arm, and can-to-plant contact. Preserve editable paths and meaningful groups; use only the supplied palette and no external assets or copied artwork. Inspect the rendered pixels at full size and a detail crop when an image-capable tool is available. Preserve BRIEF.md. No agents, dependencies, network, credentials, messages, commits or external effects.""",
        "files": {"BRIEF.md": "# Balcony watering\nAudience: a calm community-garden guide. Scene: one person in profile waters a leafy plant from a metal can beside a simple wooden stool. The visible action is the pour; the near foot bears weight on the ground and the far foot supports balance. Show the hand gripping the can handle and the water stream touching soil. Palette: paper #f4efe4, ink #183b35, leaf #648b64, coral #d8735f, brass #c79b45. Required visible label: Water what you grow. Avoid logos, photographs and borrowed characters.\n"},
        "skills": ["vector-art", "art-direction"],
        "budget": {"wall_seconds": 180, "max_repairs": 1, "shared_across": ["lead", "reviewer", "delegate", "specimen", "repair"]},
        "oracle": "svg",
    },
    "video": {
        "prompt": """Work only in this disposable project. Author a short three-scene HyperFrames HTML/CSS/JS film from BRIEF.md with local editable source and captions.json. Scene 1 introduces the Driftline tide mark, scene 2 transitions to the three lines, and scene 3 resolves on the day pass and accessibility message. Use only local source, no CDN or remote assets. Keep a deliberate visual continuity and ensure the intended tide mark and line map move, rather than relying on whole-frame changes. If the qualified Mirket video route is available, use runtime_status first, then prepare a bounded 640x360 MP4 and inspect decoded frames; if approval or capability is unavailable, preserve sources and report that exact limit without claiming an export. No agents, dependencies, network, credentials, messages, commits or external effects.""",
        "files": {"BRIEF.md": "# Driftline short film\nFormat 640x360, 12 fps, 3 seconds. Scene 1 (0–1s): Driftline wordmark and a tide mark draw on. Scene 2 (1–2s): Harbor, Orchard and North Loop route strokes appear in sequence while the tide mark travels between them. Scene 3 (2–3s): Day pass $6 and Accessible vehicles on every line hold on screen. Captions must contain the three scene messages with frame ranges. Palette: #f4efe4, #183b35, #c79b45.\n"},
        "skills": ["video-production", "motion-direction"],
        "budget": {"wall_seconds": 180, "max_repairs": 1, "shared_across": ["lead", "reviewer", "delegate", "specimen", "repair"]},
        "oracle": "video",
    },
}

def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def snapshot_skills(destination: Path, source_root: Path, names: list[str]) -> dict[str, str]:
    destination.mkdir(parents=True)
    result = {}
    for name in names:
        source = source_root / name
        if not source.is_dir():
            raise SystemExit(f"missing skill snapshot source: {source}")
        target = destination / name
        shutil.copytree(source, target)
        result[name] = hashlib.sha256(
            "".join(
                f"{path.relative_to(target)}:{digest(path)}\n"
                for path in sorted(target.rglob("*"))
                if path.is_file()
            ).encode()
        ).hexdigest()
    return result

def expected_manifest(project: Path, case: dict, skill_root: Path | None, skills: list[str]) -> dict[str, str]:
    expected: dict[str, str] = {}
    for name, content in case["files"].items():
        expected[name] = hashlib.sha256(content.encode()).hexdigest()
    if skill_root is not None:
        for skill in skills:
            source = skill_root / skill
            for path in source.rglob("*"):
                if path.is_file():
                    expected[str(Path(".agents") / "skills" / skill / path.relative_to(source))] = digest(path)
    return expected
