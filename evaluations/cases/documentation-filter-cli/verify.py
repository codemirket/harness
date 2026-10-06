import csv
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path


def main(root):
    checks = []

    def check(name, passed, detail):
        checks.append({"id": name, "status": "passed" if passed else "failed", "detail": detail})

    readme = root / "README.md"
    if not readme.is_file():
        return [{"id": "readme", "status": "failed", "detail": "README.md missing"}]
    content = readme.read_text(encoding="utf-8")
    check("fixture-citations", bool(re.search(r"\[[^]]+\]\(mini_lookup\.py\)", content)) and bool(re.search(r"\[[^]]+\]\(records\.csv\)", content)), "README links the supplied implementation and data")
    blocks = re.findall(r"```bash\s*\n(.*?)\n```", content, re.DOTALL | re.IGNORECASE)
    commands = []
    for block in blocks:
        for line in block.splitlines():
            line = line.strip()
            if line.startswith("python3 mini_lookup.py "):
                try:
                    args = shlex.split(line)
                except ValueError:
                    continue
                if all(not any(char in token for char in (";", "|", "&", ">", "<", "`", "$")) for token in args):
                    commands.append(args)
    check("example-count", len(commands) >= 2, "At least two safe local Python commands appear in bash fences")
    try:
        with (root / "records.csv").open(newline="", encoding="utf-8") as handle:
            ids = {row["id"] for row in csv.DictReader(handle)}
    except Exception:
        ids = set()
    successes = 0
    useful = 0
    for args in commands[:6]:
        if args[0:2] != ["python3", "mini_lookup.py"] or "records.csv" not in args or any("/" in arg for arg in args[2:] if not arg.startswith("--")):
            continue
        try:
            run = subprocess.run([sys.executable, '-I'] + args[1:], cwd=root, capture_output=True, text=True, timeout=3)
            rows = [json.loads(line) for line in run.stdout.splitlines()]
            if run.returncode == 0 and all(isinstance(row, dict) and row.get("id") in ids for row in rows):
                successes += 1
                if rows:
                    useful += 1
        except (OSError, ValueError, subprocess.TimeoutExpired):
            pass
    check("runnable-examples", successes >= 2 and useful >= 1, "At least two examples execute successfully; at least one returns fixture records")
    check("guide-present", bool(content.strip()), "README exists; prose accuracy requires review")
    return checks


if __name__ == "__main__":
    try:
        result = main(Path(sys.argv[1]).resolve()) if len(sys.argv) == 2 else [{"id": "usage", "status": "failed", "detail": "Expected workspace path"}]
    except Exception as exc:
        result = [{"id": "verifier-error", "status": "failed", "detail": f"{type(exc).__name__}: {str(exc)[:120]}"}]
    print(json.dumps({"checks": result}, separators=(",", ":")))
    sys.exit(0 if all(item["status"] == "passed" for item in result) else 1)
