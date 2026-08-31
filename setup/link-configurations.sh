#!/bin/sh
set -eu

shared_dir=${AI_SHARED_DIR:-"$HOME/Documents/.ai"}

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

mkdir -p "$HOME/.codex/skills" "$HOME/.claude/skills"

for deprecated_link in \
    "$HOME/.codex/CAPABILITIES.md" \
    "$HOME/.claude/CAPABILITIES.md" \
    "$HOME/.codex/shared.config.toml"
do
    [ -L "$deprecated_link" ] && unlink "$deprecated_link"
done

for skills_dir in "$HOME/.codex/skills" "$HOME/.claude/skills"; do
    for legacy_link in "$skills_dir"/*; do
        [ -L "$legacy_link" ] || continue
        link_target=$(readlink "$legacy_link")
        link_name=$(basename "$legacy_link")

        case "$link_target" in
            "$shared_dir"/skills/windmill/*)
                case "$link_name" in
                    windmill-*) ;;
                    *) unlink "$legacy_link" ;;
                esac
                ;;
            ../../.agents/skills/*)
                [ -e "$legacy_link" ] || unlink "$legacy_link"
                ;;
        esac
    done
done

link_path "$shared_dir/AGENTS.md" "$HOME/.codex/AGENTS.md"
link_path "$shared_dir/AGENTS.md" "$HOME/.claude/CLAUDE.md"

for skill_path in "$shared_dir"/skills/windmill/*; do
    [ -d "$skill_path" ] || continue
    skill_name=$(basename "$skill_path")
    link_path "$skill_path" "$HOME/.codex/skills/$skill_name"
    link_path "$skill_path" "$HOME/.claude/skills/$skill_name"
done

printf 'Shared agent configuration links are active.\n'
