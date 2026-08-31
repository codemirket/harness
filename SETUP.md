# Shared AI Setup

Use this procedure to apply the shared agent configuration on a new Mac.

## 1. Prerequisites

- Sign in to the same iCloud account.
- Enable iCloud Drive and Documents synchronization.
- Install Codex or the ChatGPT desktop app.
- Install Claude Desktop or Claude Code.
- Install the Windmill CLI if Windmill skills will be used.

Do not copy authentication files, sessions, caches, databases, or application-support directories from another device.

## 2. Verify iCloud Sync

Wait until this directory is fully downloaded:

```sh
test -f "$HOME/Documents/.ai/AGENTS.md"
test -f "$HOME/Documents/.ai/setup/link-configurations.sh"
find "$HOME/Documents/.ai/skills/windmill" -name SKILL.md | wc -l
```

The expected Windmill skill count is `28`.

If iCloud exposes the directory elsewhere, provide its path explicitly:

```sh
AI_SHARED_DIR="/absolute/path/to/.ai" sh "/absolute/path/to/.ai/setup/link-configurations.sh"
```

## 3. Check Existing Configuration

The linker refuses to replace regular files or directories.

Review these locations before running it:

```sh
ls -ld "$HOME/.codex/AGENTS.md" "$HOME/.claude/CLAUDE.md" 2>/dev/null
ls -ld "$HOME/.codex/skills" "$HOME/.claude/skills" 2>/dev/null
```

If a destination already contains local configuration:

1. Compare it with the shared source.
2. Merge rules or skill changes that must be retained.
3. Move the conflicting item outside the agent configuration directory.
4. Run the linker again.

Do not force replacement without reviewing the conflict.

## 4. Create the Links

Run:

```sh
sh "$HOME/Documents/.ai/setup/link-configurations.sh"
```

The script creates:

- `~/.codex/AGENTS.md` pointing to the shared `AGENTS.md`.
- `~/.claude/CLAUDE.md` pointing to the shared `AGENTS.md`.
- Individual Windmill skill links in `~/.codex/skills/`.
- Individual Windmill skill links in `~/.claude/skills/`.

It also removes known obsolete links created by earlier versions of this setup.

## 5. Configure Device-Local Capabilities

Configure the following separately on every device:

- Codex and Claude authentication.
- Plugins and MCP servers.
- Chrome extension and browser permissions.
- Excel, PDF, document, presentation, and computer-control connections.
- Windmill CLI authentication and workspace selection.
- Project trust and repository-specific configuration.
- Application paths, notification commands, and local runtime paths.

For Codex preferences, review:

```text
~/Documents/.ai/config/codex/preferences.toml
```

Apply only the required values to `~/.codex/config.toml`. Do not replace the complete local file because it contains machine-specific paths and runtime configuration.

## 6. Verify the Installation

Verify global instruction links:

```sh
readlink "$HOME/.codex/AGENTS.md"
readlink "$HOME/.claude/CLAUDE.md"
```

Both should resolve to:

```text
~/Documents/.ai/AGENTS.md
```

Verify shared skills:

```sh
find -L "$HOME/.codex/skills" -maxdepth 2 -name SKILL.md | grep '/windmill-' | wc -l
find -L "$HOME/.claude/skills" -maxdepth 2 -name SKILL.md | grep '/windmill-' | wc -l
```

Both expected counts are `28`.

Verify that no managed links are broken:

```sh
find -L "$HOME/.codex/skills" "$HOME/.claude/skills" -type l
```

Expected output: none.

Verify local capabilities when installed:

```sh
codex doctor --summary
wmill --version
```

## 7. Reload the Agents

Close existing Codex and Claude sessions after linking or changing skills. Start new sessions so each agent rebuilds its instruction and skill discovery state.

Confirm in each agent that:

- Global working principles are active.
- Windmill skills appear with `windmill-` prefixes.
- Ordinary Python, Bash, SQL, browser, or scheduling tasks do not trigger Windmill skills unless the task involves Windmill.

## 8. Maintenance

- Edit the canonical files only under `~/Documents/.ai`.
- Run the linker after adding, removing, or renaming shared skills.
- Restart agent sessions after instruction or skill changes.
- Keep secrets and machine-specific configuration outside this directory.
- Do not sync agent caches, session histories, plugin caches, databases, or application-support data.
- Review iCloud conflict copies before accepting either version.

## Troubleshooting

### The linker reports a non-symlink conflict

Review and move the conflicting local item outside the agent directory. Run the linker again.

### A link is broken

Confirm iCloud has downloaded `.ai`, then rerun the linker. Use `AI_SHARED_DIR` if the directory is stored at a nonstandard path.

### Skills do not appear

Confirm the skill links resolve, then fully restart the affected agent. Do not clear application caches first.

### Codex configuration fails to load

Keep `~/.codex/config.toml` local and run:

```sh
codex doctor --summary
```

Correct the reported local configuration issue. Do not replace the file with the portable preference reference.

## References

- [OpenAI Docs: AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
- [OpenAI Docs: skills](https://learn.chatgpt.com/docs/build-skills)
- [OpenAI Docs: configuration](https://learn.chatgpt.com/docs/config-file/config-basic)
