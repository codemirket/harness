# Windows 11 Setup

Use this procedure to apply the shared agent configuration on a new Windows 11 device.

## 1. Prerequisites

- Create or obtain a complete local copy of the shared `.ai` directory.
- If Git is the chosen persistence method, install Git for Windows and clone or update the repository before setup.
- Install Codex and its CLI.
- Install Node.js 24 or newer with npm.
- Install Docker Desktop and select Linux containers.
- Install Claude Code CLI. Claude Desktop chat is also registered when its local configuration directory exists.
- Install PowerShell 5.1 or later (included with Windows 11).
- Install any third-party skills separately through their installer or plugin.

Do not copy authentication files, sessions, caches, databases, or application-support directories from another device.

The Windows setup has no iCloud dependency. Store the `.ai` directory at any stable local path. Each script discovers the shared directory from its own location; use `-SharedDir` only when the scripts are invoked from elsewhere.

## 2. Verify the Shared Directory

In PowerShell, set the shared directory to its actual local path:

```powershell
$sharedDir = 'C:\path\to\.ai'
Test-Path -LiteralPath (Join-Path $sharedDir 'AGENTS.md') -PathType Leaf
Test-Path -LiteralPath (Join-Path $sharedDir 'setup\Windows11\codex.ps1') -PathType Leaf
Test-Path -LiteralPath (Join-Path $sharedDir 'setup\Windows11\claude.ps1') -PathType Leaf
Test-Path -LiteralPath (Join-Path $sharedDir 'skills') -PathType Container
```

All four commands should return `True`. The custom skill collection may be empty.

## 3. Check Existing Configuration

The setup scripts refuse to replace regular files, directories, junctions, or other non-symbolic-link items.

Review these locations before running it:

```powershell
Get-Item "$HOME\.codex\AGENTS.md", "$HOME\.claude\CLAUDE.md" -Force -ErrorAction SilentlyContinue
Get-Item "$HOME\.codex\skills", "$HOME\.claude\skills" -Force -ErrorAction SilentlyContinue
```

If a destination already contains local configuration:

1. Compare it with the shared source.
2. Merge rules or skill changes that must be retained.
3. Move the conflicting item outside the agent configuration directory.
4. Run the applicable setup script again.

Do not force replacement without reviewing the conflict.

## 4. Create the Links

Run the following in PowerShell:

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
& (Join-Path $sharedDir 'setup\Windows11\codex.ps1')
& (Join-Path $sharedDir 'setup\Windows11\claude.ps1')
```

`codex.ps1` creates:

- `%USERPROFILE%\.codex\AGENTS.md` pointing to the shared `AGENTS.md`.
- Individual custom skill links in `%USERPROFILE%\.codex\skills\`, when present.

`claude.ps1` creates:

- `%USERPROFILE%\.claude\CLAUDE.md` pointing to the shared `AGENTS.md`.
- Individual custom skill links in `%USERPROFILE%\.claude\skills\`, when present.

Each script replaces only existing symbolic links. If Windows prevents symbolic-link creation, enable Developer Mode or run PowerShell as Administrator, then run the command again.

Skills are discovered at `skills/<group>/<skill-name>/SKILL.md`. Use unique skill
names across groups. Each script removes stale skill links pointing into this
shared repository. Rerun setup after removing the previously bundled skills to
clean up their links.

## Containers MCP

The per-agent setup scripts also install containers MCP and register it globally
for that agent. Claude setup registers Claude Code at user scope and merges a
registration into Claude Desktop's configuration when its directory exists.
Other MCP entries are preserved. Restart sessions after setup.

The server starts on demand through stdio. The catalog initially offers PostgreSQL,
Redis, and MinIO; containers are created only when requested through MCP tools.
All bind data is stored under `~/.dev.volumes/<service-alias>`, with private catalog
state and credentials under `~/.dev.mcp`. Do not synchronize these directories.

To install only containers MCP for both agents:

```powershell
& (Join-Path $sharedDir 'setup\Windows11\containers.ps1') -SharedDir $sharedDir
```

See [containers MCP](../../mcp/containers/README.md) for requirements, tools,
project separation, custom catalog entries, runtime locations, and diagnostics.

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
<sharedDir>\config\codex\preferences.toml
```

Apply only the required values to `$HOME\.codex\config.toml`. Do not replace the complete local file because it contains machine-specific paths and runtime configuration.

## 6. Verify the Installation

Verify global instruction links:

```powershell
(Get-Item "$HOME\.codex\AGENTS.md" -Force).Target
(Get-Item "$HOME\.claude\CLAUDE.md" -Force).Target
```

Both should resolve to `$sharedDir\AGENTS.md`.

Verify each custom skill you added (replace `my-custom-skill`):

```powershell
(Get-Item "$HOME\.codex\skills\my-custom-skill" -Force).Target
(Get-Item "$HOME\.claude\skills\my-custom-skill" -Force).Target
```

Each link should point to its custom skill directory under `$sharedDir\skills`.
Skip this check when no custom skills have been created.

Verify that the links resolve:

```powershell
Get-ChildItem "$HOME\.codex\skills", "$HOME\.claude\skills" -Force |
    Where-Object LinkType -eq 'SymbolicLink' |
    ForEach-Object { "{0}: {1}" -f $_.FullName, (Test-Path -LiteralPath $_.Target) }
```

Each shared custom skill link should report `True`. Review external links separately.

Verify local capabilities when installed:

```powershell
codex doctor --summary
```

## 7. Reload the Agents

Close existing Codex and Claude sessions after linking or changing skills. Start new sessions so each agent rebuilds its instruction and skill discovery state.

Confirm in each agent that:

- Global working principles are active.
- Custom skills appear when present and activate only for their intended tasks.
- Third-party skills are managed separately from this shared directory.

## 8. Maintenance

- Edit the canonical files only in the shared `.ai` directory.
- Run the installed agents' setup scripts after adding, removing, or renaming shared skills.
- Restart agent sessions after instruction or skill changes.
- Keep secrets and machine-specific configuration outside this directory.
- Do not sync agent caches, session histories, plugin caches, databases, or application-support data.
- Review synchronization conflict copies before accepting either version.

## Troubleshooting

### A setup script reports a non-symbolic-link conflict

Review and move the conflicting local item outside the agent directory. Run the applicable setup script again.

### The shared directory is missing or incomplete

Confirm that the local `.ai` directory contains `AGENTS.md`, `skills`, and `setup`. If Git provides persistence, update the working tree, then run the verification commands again.

### Windows cannot create symbolic links

Enable Developer Mode in Windows Settings, or run PowerShell as Administrator for the setup command. Then rerun the applicable script.

### Skills do not appear

Confirm the skill links resolve, then fully restart the affected agent. Do not clear application caches first.

### Codex configuration fails to load

Keep `$HOME\.codex\config.toml` local and run:

```powershell
codex doctor --summary
```

Correct the reported local configuration issue. Do not replace the file with the portable preference reference.
