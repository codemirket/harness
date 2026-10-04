# Shared AI guidance

This repository holds personal working principles shared by Codex and Claude Code across devices. The current setup links [components/AGENTS.md](components/AGENTS.md) into each agent's global instruction location. It does not install skills or plugins.

| Agent | Global link |
| --- | --- |
| Codex | `~/.codex/AGENTS.md` → this repository's `components/AGENTS.md` |
| Claude Code | `~/.claude/CLAUDE.md` → this repository's `components/AGENTS.md` |

Use [setup/README.md](setup/README.md) to link either agent on macOS or Windows.

Keep architecture, conventions, workflows, and project skills in each project's own `AGENTS.md` and `.agents/skills/` for Codex, or `CLAUDE.md` and `.claude/skills/` for Claude Code. Personal reusable skills and plugins may belong in a future marketplace here, but that marketplace has not been built yet.

Keep credentials, machine paths, caches, and session data on each device. Install third-party skills and plugins through their own managers.
