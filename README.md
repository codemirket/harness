# Shared AI Configuration

This directory contains portable configuration shared across AI agents. macOS can use iCloud Drive for storage; the Windows setup has no iCloud dependency and works from any local copy of this directory.

## Contents

- `AGENTS.md`: Universal working principles and capability rules.
- `config/codex/preferences.toml`: Portable Codex preference reference.
- `skills/`: Canonical source for reusable skills.
- `setup/MacOS/`: macOS guide plus separate Codex and Claude setup scripts.
- `setup/Windows11/`: Windows 11 guide plus separate Codex and Claude PowerShell scripts.

## Active Links

- Codex: local `AGENTS.md` → shared `AGENTS.md`.
- Claude: local `CLAUDE.md` → shared `AGENTS.md`.
- Windmill skills: linked individually into the local Codex and Claude skill directories.

The exact local paths and commands are in the [macOS](setup/MacOS/SETUP.MD) and [Windows 11](setup/Windows11/SETUP.MD) guides.

## Rules

- Keep this directory portable and human-readable.
- Do not store secrets, tokens, credentials, logs, caches, or session state.
- Keep machine-specific settings in each agent's local configuration.
- Do not copy application-managed or plugin-managed skills here unless this directory is their intended canonical source.
- Add only focused skills for recurring workflows.
- Link shared skills individually; do not replace an agent's complete skills directory.

## New Device Setup

Follow the guide for the device's operating system:

- [macOS](setup/MacOS/SETUP.MD)
- [Windows 11](setup/Windows11/SETUP.MD)

## Adding a Shared Skill

1. Create `skills/<group>/<skill-name>/SKILL.md`.
2. Add only required scripts, references, templates, or assets.
3. Link the skill directory into each compatible agent's skills directory.
4. Restart the agent and verify the skill is discovered.
