# Shared AI Configuration

This directory contains portable configuration shared across AI agents. macOS can use iCloud Drive for storage; the Windows setup has no iCloud dependency and works from any local copy of this directory.

## Contents

- `AGENTS.md`: Universal working principles and capability rules.
- `config/codex/preferences.toml`: Portable Codex preference reference.
- `skills/`: Canonical source for custom skills you create and own.
- `setup/MacOS/`: macOS guide plus separate Codex and Claude setup scripts.
- `setup/Windows11/`: Windows 11 guide plus separate Codex and Claude PowerShell scripts.

## Active Links

- Codex: local `AGENTS.md` → shared `AGENTS.md`.
- Claude: local `CLAUDE.md` → shared `AGENTS.md`.
- Custom skills: linked individually into the local Codex and Claude skill directories when present.

The exact local paths and commands are in the [macOS](setup/MacOS/SETUP.MD) and [Windows 11](setup/Windows11/SETUP.MD) guides.

## Rules

- Keep this directory portable and human-readable.
- Do not store secrets, tokens, credentials, logs, caches, or session state.
- Keep machine-specific settings in each agent's local configuration.
- Install third-party and plugin-managed skills separately; do not vendor re-installable skills here.
- Add only focused skills for recurring workflows.
- Link shared skills individually; do not replace an agent's complete skills directory.

## New Device Setup

Follow the guide for the device's operating system:

- [macOS](setup/MacOS/SETUP.MD)
- [Windows 11](setup/Windows11/SETUP.MD)

## Adding a Shared Skill

1. Create `skills/<group>/<skill-name>/SKILL.md`, with a skill name unique across groups.
2. Add only required scripts, references, templates, or assets.
3. Run each installed agent's setup script to link the custom skill.
4. Restart the agent and verify the skill is discovered.

An empty custom skill collection is supported. See [skills/README.md](skills/README.md).

## Setup Tests

Run the macOS setup tests with `python3 -B -m unittest discover -s setup/tests -v`.
They use temporary directories and do not change your installed agent configuration.
