# Link shared instructions

Keep a local copy of this repository with `components/AGENTS.md` available. Run setup separately for each agent you use. The scripts link only the global instruction file; neither agent needs to be installed for the link itself to be created.

## macOS

From the repository root:

```sh
sh setup/macos.sh codex
sh setup/macos.sh claude
```

The script finds the repository root from its own location, even when run from another working directory. To use a different shared copy, set `AI_SHARED_DIR` to that repository's root:

```sh
AI_SHARED_DIR="/absolute/path/to/.ai" sh setup/macos.sh codex
```

## Windows

In PowerShell, from the repository root:

```powershell
& .\setup\windows.ps1 codex
& .\setup\windows.ps1 claude
```

The script finds the repository root from its own location. Use `-SharedDir` to choose another repository root:

```powershell
& .\setup\windows.ps1 codex -SharedDir 'C:\path\to\.ai'
```

Windows may require Developer Mode or an elevated PowerShell session to create symbolic links. If script execution is disabled, allow it for the current PowerShell process according to your device policy.

## Conflicts and verification

Setup refuses to replace a regular file or directory at the selected destination. Review and move any existing instruction file yourself, then rerun setup. An existing symbolic link is updated, including a broken link; running setup again is safe. The source `components/AGENTS.md` must exist before setup changes the destination.

Verify the link for the agent you selected:

```sh
readlink "$HOME/.codex/AGENTS.md" # or "$HOME/.claude/CLAUDE.md"
```

```powershell
(Get-Item "$HOME\.codex\AGENTS.md" -Force).Target # or "$HOME\.claude\CLAUDE.md"
```

The target should be the absolute path to this repository's `components/AGENTS.md`. Rerun setup for each linked agent on other devices to refresh links created before the file moved. Start a new agent session after changing shared instructions so they are reloaded.

Authentication, MCP servers, plugins, app permissions, and machine-specific settings remain local.

Earlier setup versions managed shared skill links and removed deprecated configuration links. This version leaves those existing links untouched; inspect and remove obsolete links manually if needed. Skills and plugins are planned for a later marketplace stage.
