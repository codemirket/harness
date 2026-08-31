# Shared AI Configuration

This directory contains portable configuration shared across AI agents through iCloud.

## Contents

- `AGENTS.md`: Universal working principles and capability rules.
- `SETUP.md`: New-device installation and verification procedure.
- `config/codex/preferences.toml`: Portable Codex preference reference.
- `skills/`: Canonical source for reusable skills.
- `setup/link-configurations.sh`: Safe linker for a new Mac.

## Active Links

- Codex: `~/.codex/AGENTS.md` → `~/Documents/.ai/AGENTS.md`
- Claude: `~/.claude/CLAUDE.md` → `~/Documents/.ai/AGENTS.md`
- Windmill skills: linked individually into `~/.codex/skills/` and `~/.claude/skills/`

## Rules

- Keep this directory portable and human-readable.
- Do not store secrets, tokens, credentials, logs, caches, or session state.
- Keep machine-specific settings in each agent's local configuration.
- Do not copy application-managed or plugin-managed skills here unless this directory is their intended canonical source.
- Add only focused skills for recurring workflows.
- Link shared skills individually; do not replace an agent's complete skills directory.

## New Mac Setup

Follow `SETUP.md`.

## Adding a Shared Skill

1. Create `skills/<group>/<skill-name>/SKILL.md`.
2. Add only required scripts, references, templates, or assets.
3. Link the skill directory into each compatible agent's skills directory.
4. Restart the agent and verify the skill is discovered.
