#!/bin/sh
set -eu

shared_dir=${AI_SHARED_DIR:-"$HOME/Documents/.ai"}
codex_dir="$HOME/.codex"

link_path() {
    source_path=$1
    destination_path=$2

    if [ ! -e "$source_path" ]; then
        printf 'Missing shared source: %s\n' "$source_path" >&2
        exit 1
    fi

    if [ -e "$destination_path" ] && [ ! -L "$destination_path" ]; then
        printf 'Refusing to replace non-symlink: %s\n' "$destination_path" >&2
        exit 1
    fi

    ln -sfn "$source_path" "$destination_path"
}

mkdir -p "$codex_dir/skills"

for deprecated_link in \
    "$codex_dir/CAPABILITIES.md" \
    "$codex_dir/shared.config.toml"
do
    [ -L "$deprecated_link" ] && unlink "$deprecated_link"
done

for legacy_link in "$codex_dir/skills"/*; do
    [ -L "$legacy_link" ] || continue
    link_target=$(readlink "$legacy_link")
    case "$link_target" in
        "$shared_dir"/skills/*)
            [ -f "$legacy_link/SKILL.md" ] || unlink "$legacy_link"
            ;;
    esac
done

link_path "$shared_dir/AGENTS.md" "$codex_dir/AGENTS.md"

for skill_path in "$shared_dir"/skills/*/*; do
    [ -f "$skill_path/SKILL.md" ] || continue
    skill_name=$(basename "$skill_path")
    link_path "$skill_path" "$codex_dir/skills/$skill_name"
done

printf 'Codex shared configuration links are active.\n'

AI_SHARED_DIR="$shared_dir" sh "$shared_dir/setup/MacOS/containers.sh" codex
