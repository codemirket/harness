import json
import re
import sys
from pathlib import Path


def main(root):
    checks = []

    def check(name, passed, detail):
        checks.append({"id": name, "status": "passed" if passed else "failed", "detail": detail})

    output = root / "research.md"
    if not output.is_file():
        return [{"id": "brief", "status": "failed", "detail": "research.md missing"}]
    content = output.read_text(encoding="utf-8")
    sources = ["deployment.json", "release-4.2.md", "legacy-3.9.md", "field-note.md"]
    linked = [name for name in sources if re.search(r"\[[^]]+\]\(sources/" + re.escape(name) + r"\)", content)]
    check("source-citations", len(linked) == len(sources), f"Linked fixture sources: {', '.join(linked)}")
    check("version-and-dates", "4.2.1" in content and len(set(re.findall(r"20\d{2}-\d{2}-\d{2}", content))) >= 2, "Brief states pinned version and at least two source dates")
    check("brief-present", bool(content.strip()), "Brief exists; interpretation and grounding require qualitative review")
    return checks


if __name__ == "__main__":
    try:
        result = main(Path(sys.argv[1]).resolve()) if len(sys.argv) == 2 else [{"id": "usage", "status": "failed", "detail": "Expected workspace path"}]
    except Exception as exc:
        result = [{"id": "verifier-error", "status": "failed", "detail": f"{type(exc).__name__}: {str(exc)[:120]}"}]
    print(json.dumps({"checks": result}, separators=(",", ":")))
    sys.exit(0 if all(item["status"] == "passed" for item in result) else 1)
