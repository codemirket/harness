#!/bin/sh
# Compatibility entry point for existing cron jobs and manual setup.
set -eu
if [ "$#" -ne 1 ]; then
    printf 'Usage: sh %s codex|claude|both\n' "$0" >&2
    exit 2
fi
case $1 in codex|claude|both) ;; *) printf 'Choose codex, claude or both.\n' >&2; exit 2 ;; esac
shared_dir=${AI_SHARED_DIR:-"$(CDPATH='' cd "$(dirname "$0")/.." && pwd -P)"}
if [ ! -f "$shared_dir/ai.py" ]; then
    printf 'Missing personal harness: %s/ai.py\n' "$shared_dir" >&2
    exit 1
fi
if ! command -v python3 >/dev/null 2>&1 || ! python3 -c 'import sys; raise SystemExit(sys.version_info < (3, 9))'; then
    printf 'Python 3.9 or later is required.\n' >&2
    exit 1
fi
exec python3 "$shared_dir/ai.py" sync --target "$1"
