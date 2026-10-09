#!/bin/sh
# Compatibility entry point for existing cron jobs and manual setup.
set -eu
if [ "$#" -gt 1 ]; then
    printf 'Usage: sh %s [codex-desktop|claude-desktop|all|codex|claude-code|both]\n' "$0" >&2
    exit 2
fi
if [ "$#" -eq 1 ]; then
    case $1 in codex-desktop|claude-desktop|all|codex|claude-code|claude|both) ;; *) printf 'Choose codex-desktop, claude-desktop or all.\n' >&2; exit 2 ;; esac
fi
shared_dir=${AI_SHARED_DIR:-"$(CDPATH='' cd "$(dirname "$0")/.." && pwd -P)"}
if [ ! -f "$shared_dir/ai.py" ]; then
    printf 'Missing personal harness: %s/ai.py\n' "$shared_dir" >&2
    exit 1
fi
if ! command -v python3 >/dev/null 2>&1 || ! python3 -c 'import sys; raise SystemExit(sys.version_info < (3, 9))'; then
    printf 'Python 3.9 or later is required.\n' >&2
    exit 1
fi
if [ "$#" -eq 0 ]; then
    exec python3 "$shared_dir/ai.py" install
fi
case $1 in
    codex-desktop|claude-desktop|all) exec python3 "$shared_dir/ai.py" install --target "$1" ;;
esac
# Explicit legacy targets remain suitable for existing scheduled skill syncs.
exec python3 "$shared_dir/ai.py" sync --target "$1"
