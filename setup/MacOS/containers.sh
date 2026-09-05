#!/bin/sh
set -eu

shared_dir=${AI_SHARED_DIR:-"$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"}
agent=${1:-all}
command -v node >/dev/null 2>&1 || { printf 'Install Node.js 24 or newer.\n' >&2; exit 1; }
command -v npm >/dev/null 2>&1 || { printf 'Install npm with Node.js.\n' >&2; exit 1; }
npm --prefix "$shared_dir/mcp/containers" run setup -- --agent "$agent"
