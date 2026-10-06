#!/usr/bin/env python3
"""Run the personal harness from an installed skill, without guessing its checkout path."""
from pathlib import Path
import runpy
import sys
from catalog import repository

if __name__ == '__main__':
    try:
        root = repository()
        sys.path.insert(0, str(root))
        runpy.run_path(str(root / 'ai.py'), run_name='__main__')
    except (ValueError, OSError, KeyError) as error:
        print('Error: ' + str(error), file=sys.stderr)
        sys.exit(1)
