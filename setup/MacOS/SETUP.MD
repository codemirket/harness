# macOS Setup

Use this procedure to apply the shared agent configuration on a new Mac.

## 1. Prerequisites

- Sign in to the same Apple Account.
- Enable iCloud Drive and Desktop & Documents synchronization.
- In Finder, mark the `.ai` folder as **Keep Downloaded**.
- Install Codex or the ChatGPT desktop app.
- Install Claude Desktop or Claude Code.
- Install any third-party skills separately through their installer or plugin.

Do not copy authentication files, sessions, caches, databases, or application-support directories from another device.

## 2. Verify iCloud Sync

Wait until this directory is fully downloaded:

```sh
test -f "$HOME/Documents/.ai/AGENTS.md"
test -f "$HOME/Documents/.ai/setup/MacOS/codex.sh"
test -f "$HOME/Documents/.ai/setup/MacOS/claude.sh"
test -d "$HOME/Documents/.ai/skills"
```

The custom skill collection may be empty.

If iCloud exposes the directory elsewhere, provide its path explicitly:

```sh
AI_SHARED_DIR="/absolute/path/to/.ai" sh "/absolute/path/to/.ai/setup/MacOS/codex.sh"
AI_SHARED_DIR="/absolute/path/to/.ai" sh "/absolute/path/to/.ai/setup/MacOS/claude.sh"
```

## 3. Check Existing Configuration

The setup scripts refuse to replace regular files or directories.

Review these locations before running it:

```sh
ls -ld "$HOME/.codex/AGENTS.md" "$HOME/.claude/CLAUDE.md" 2>/dev/null
ls -ld "$HOME/.codex/skills" "$HOME/.claude/skills" 2>/dev/null
```

If a destination already contains local configuration:

1. Compare it with the shared source.
2. Merge rules or skill changes that must be retained.
3. Move the conflicting item outside the agent configuration directory.
4. Run the applicable setup script again.

Do not force replacement without reviewing the conflict.

## 4. Create the Links

Run the scripts for the agents installed on this Mac:

```sh
sh "$HOME/Documents/.ai/setup/MacOS/codex.sh"
sh "$HOME/Documents/.ai/setup/MacOS/claude.sh"
```

`codex.sh` creates:

- `~/.codex/AGENTS.md` pointing to the shared `AGENTS.md`.
- Individual custom skill links in `~/.codex/skills/`, when present.

`claude.sh` creates:

- `~/.claude/CLAUDE.md` pointing to the shared `AGENTS.md`.
- Individual custom skill links in `~/.claude/skills/`, when present.

Skills are discovered at `skills/<group>/<skill-name>/SKILL.md`. Use unique skill
names across groups. Each script also removes stale skill links pointing into
this shared repository and obsolete configuration links from earlier setup versions.
Rerun setup after removing the previously bundled skills to clean up their links.

## 5. Configure Device-Local Capabilities

Configure the following separately on every device:

- Codex and Claude authentication.
- Plugins and MCP servers.
- Chrome extension and browser permissions.
- Excel, PDF, document, presentation, and computer-control connections.
- Third-party skill dependencies and authentication, when needed.
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

Verify each custom skill you added (replace `my-custom-skill`):

```sh
readlink "$HOME/.codex/skills/my-custom-skill"
readlink "$HOME/.claude/skills/my-custom-skill"
```

Each link should point to its custom skill directory under the shared `skills/`
directory. Skip this check when no custom skills have been created.

Check for broken skill links:

```sh
find -L "$HOME/.codex/skills" "$HOME/.claude/skills" -type l
```

Shared custom skill links should not appear. Review any external links separately.

Verify local capabilities when installed:

```sh
codex doctor --summary
```

## 7. Reload the Agents

Close existing Codex and Claude sessions after linking or changing skills. Start new sessions so each agent rebuilds its instruction and skill discovery state.

Confirm in each agent that:

- Global working principles are active.
- Custom skills appear when present and activate only for their intended tasks.
- Third-party skills are managed separately from this shared directory.

## 8. Maintenance

- Edit the canonical files only under `~/Documents/.ai`.
- Run the installed agents' setup scripts after adding, removing, or renaming shared skills.
- Restart agent sessions after instruction or skill changes.
- Keep secrets and machine-specific configuration outside this directory.
- Do not sync agent caches, session histories, plugin caches, databases, or application-support data.
- Review iCloud conflict copies before accepting either version.

## Troubleshooting

### A setup script reports a non-symlink conflict

Review and move the conflicting local item outside the agent directory. Run the applicable setup script again.

### A link is broken

Confirm iCloud has downloaded `.ai`, then rerun the applicable setup script. Use `AI_SHARED_DIR` if the directory is stored at a nonstandard path.

### Skills do not appear

Confirm the skill links resolve, then fully restart the affected agent. Do not clear application caches first.

### Codex configuration fails to load

Keep `~/.codex/config.toml` local and run:

```sh
codex doctor --summary
```

Correct the reported local configuration issue. Do not replace the file with the portable preference reference.
