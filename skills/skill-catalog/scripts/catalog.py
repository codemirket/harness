#!/usr/bin/env python3
"""Compatibility entry point; the canonical registry and implementation live in the harness."""
import json
from pathlib import Path
import runpy
import sys


def repository():
    skill = Path(__file__).resolve().parents[1]
    locator = skill / '.harness-source.json'
    if locator.is_file():
        value = json.loads(locator.read_text(encoding='utf-8'))['repository']
        root = Path(value)
        if not root.is_absolute():
            root = (skill / root).resolve()
    else:
        root = skill.parents[1]
    if not (root / 'registry/harness.json').is_file() or not (root / 'ai.py').is_file():
        raise ValueError('Personal harness checkout is unavailable; restore it or rerun setup.')
    return root


if __name__ == '__main__':
    try:
        root = repository()
        sys.path.insert(0, str(root))
        # The old catalog command remains usable from a live link or a Windows copy.
        sys.argv.insert(1, 'catalog')
        runpy.run_path(str(root / 'ai.py'), run_name='__main__')
    except (ValueError, OSError, KeyError) as error:
        print('Error: ' + str(error), file=sys.stderr)
        sys.exit(1)
