# Windows 11 Setup

Use this procedure to apply the shared agent configuration on a new Windows 11 device.

## 1. Prerequisites

- Create or obtain a complete local copy of the shared `.ai` directory.
- If Git is the chosen persistence method, install Git for Windows and clone or update the repository before setup.
- Install Codex or the ChatGPT desktop app.
- Install Claude Desktop or Claude Code.
- Install PowerShell 5.1 or later (included with Windows 11).
- Install the Windmill CLI if Windmill skills will be used.

Do not copy authentication files, sessions, caches, databases, or application-support directories from another device.

The Windows setup has no iCloud dependency. Store the `.ai` directory at any stable local path. Each script discovers the shared directory from its own location; use `-SharedDir` only when the scripts are invoked from elsewhere.

## 2. Verify the Shared Directory

In PowerShell, set the shared directory to its actual local path:

```powershell
$sharedDir = 'C:\path\to\.ai'
Test-Path -LiteralPath (Join-Path $sharedDir 'AGENTS.md') -PathType Leaf
Test-Path -LiteralPath (Join-Path $sharedDir 'setup\Windows11\codex.ps1') -PathType Leaf
Test-Path -LiteralPath (Join-Path $sharedDir 'setup\Windows11\claude.ps1') -PathType Leaf
(Get-ChildItem -LiteralPath (Join-Path $sharedDir 'skills\windmill') -Directory | Measure-Object).Count
```

The first three commands should return `True`. The expected Windmill skill count is `28`.

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
- Individual Windmill skill links in `%USERPROFILE%\.codex\skills\`.

`claude.ps1` creates:

- `%USERPROFILE%\.claude\CLAUDE.md` pointing to the shared `AGENTS.md`.
- Individual Windmill skill links in `%USERPROFILE%\.claude\skills\`.

Each script replaces only existing symbolic links. If Windows prevents symbolic-link creation, enable Developer Mode or run PowerShell as Administrator, then run the command again.

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

Verify shared skills:

```powershell
(Get-ChildItem "$HOME\.codex\skills" -Directory | Where-Object Name -Like 'windmill-*' | Measure-Object).Count
(Get-ChildItem "$HOME\.claude\skills" -Directory | Where-Object Name -Like 'windmill-*' | Measure-Object).Count
```

Both expected counts are `28`.

Verify that the links resolve:

```powershell
Get-ChildItem "$HOME\.codex\skills", "$HOME\.claude\skills" -Force |
    Where-Object LinkType -eq 'SymbolicLink' |
    ForEach-Object { "{0}: {1}" -f $_.FullName, (Test-Path -LiteralPath $_.Target) }
```

Each managed link should report `True`.

Verify local capabilities when installed:

```powershell
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
