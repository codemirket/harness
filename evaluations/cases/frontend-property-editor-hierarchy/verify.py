import json
import struct
import sys
from html.parser import HTMLParser
from pathlib import Path


class Elements(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = {}
        self.labels = set()
        self.assets = set()
        self.controls = []
        self.text = []

    def handle_starttag(self, tag, attrs):
        values = dict(attrs)
        if values.get("id"):
            self.ids[values["id"]] = tag
        if tag in {"select", "input", "textarea", "button"}:
            self.controls.append((tag, values))
        if tag == "label" and values.get("for"):
            self.labels.add(values["for"])
        if tag == "link" and values.get("href"):
            self.assets.add(values["href"])
        if tag == "script" and values.get("src"):
            self.assets.add(values["src"])

    def handle_data(self, data):
        self.text.append(data)


def png_size(path):
    data = path.read_bytes()[:24]
    if len(data) != 24 or not data.startswith(b"\x89PNG\r\n\x1a\n"):
        return None
    return struct.unpack(">II", data[16:24])


def main(root):
    checks = []

    def check(name, condition, detail):
        checks.append({"id": name, "status": "passed" if condition else "failed", "detail": detail})

    page = root / "index.html"
    if page.is_file():
        parser = Elements()
        parser.feed(page.read_text(encoding="utf-8"))
        editable = [(tag, attrs) for tag, attrs in parser.controls if tag in {"select", "input", "textarea"}]
        labeled = [attrs.get("id") in parser.labels or bool(attrs.get("aria-label")) or bool(attrs.get("aria-labelledby")) for _tag, attrs in editable]
        words = " ".join(parser.text).casefold()
        check("semantic-controls", len(editable) >= 3 and all(labeled) and any(tag == "select" for tag, _ in editable) and any(tag == "button" for tag, _ in parser.controls) and "draft" in words and "save" in words, "Editable controls are labeled and page exposes Draft and a Save action")
        check("local-assets", {"styles.css", "app.js"} <= parser.assets and all((root / asset).is_file() for asset in ("styles.css", "app.js")), "Page loads local styles and behavior")
    else:
        check("semantic-controls", False, "index.html missing")
        check("local-assets", False, "index.html missing")

    evidence = root / "evidence"
    expected = ["light-desktop", "dark-desktop", "light-narrow", "dark-narrow"]
    sizes = {name: png_size(evidence / f"{name}.png") if (evidence / f"{name}.png").is_file() else None for name in expected}
    valid = all(size and size[1] >= 300 for size in sizes.values())
    if valid:
        valid = (all(sizes[name][0] >= 900 for name in expected if name.endswith('desktop'))
                 and all(300 <= sizes[name][0] <= 480 for name in expected if name.endswith('narrow')))
    check("render-captures", valid, "Desktop and narrow PNG captures supplied; content, theme and design require review")
    note = evidence / "review.md"
    check("review-note", note.is_file() and bool(note.read_text(encoding="utf-8").strip()), "A render-review note is present; its claims require review")
    return checks


if __name__ == "__main__":
    try:
        result = main(Path(sys.argv[1]).resolve()) if len(sys.argv) == 2 else [{"id": "usage", "status": "failed", "detail": "Expected workspace path"}]
    except Exception as exc:
        result = [{"id": "verifier-error", "status": "failed", "detail": f"{type(exc).__name__}: {str(exc)[:120]}"}]
    print(json.dumps({"checks": result}, separators=(",", ":")))
    sys.exit(0 if all(item["status"] == "passed" for item in result) else 1)
