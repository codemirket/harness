#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    printf 'Usage: sh %s codex|claude\n' "$0" >&2
    exit 2
fi

case $1 in
    codex) agent_dir="$HOME/.codex"; instruction_file=AGENTS.md ;;
    claude) agent_dir="$HOME/.claude"; instruction_file=CLAUDE.md ;;
    *) printf 'Choose codex or claude.\n' >&2; exit 2 ;;
esac

shared_dir=${AI_SHARED_DIR:-"$(CDPATH='' cd "$(dirname "$0")/.." && pwd -P)"}
if [ ! -d "$shared_dir" ]; then
    printf 'Missing shared directory: %s\n' "$shared_dir" >&2
    exit 1
fi
shared_dir=$(CDPATH='' cd "$shared_dir" && pwd -P)
source_path="$shared_dir/components/AGENTS.md"
destination_path="$agent_dir/$instruction_file"

if [ ! -f "$source_path" ]; then
    printf 'Missing shared source: %s\n' "$source_path" >&2
    exit 1
fi
if [ -e "$destination_path" ] && [ ! -L "$destination_path" ]; then
    printf 'Refusing to replace non-symlink: %s\n' "$destination_path" >&2
    exit 1
fi
if [ -L "$destination_path" ] && [ "$(readlink "$destination_path")" = "$source_path" ]; then
    printf '%s is already linked.\n' "$destination_path"
    exit 0
fi

mkdir -p "$agent_dir"
if [ -L "$destination_path" ]; then
    unlink "$destination_path"
fi
ln -s "$source_path" "$destination_path"
printf 'Linked %s -> %s\n' "$destination_path" "$source_path"
