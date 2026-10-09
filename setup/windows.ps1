[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [ValidateSet('install', 'codex-desktop', 'claude-desktop', 'all', 'codex', 'claude-code', 'claude', 'both')]
    [string]$Agent = 'install',
    [string]$SharedDir,
    [ValidateSet('auto', 'copy', 'link')]
    [string]$Mode = 'auto'
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($SharedDir)) { $SharedDir = Join-Path $PSScriptRoot '..' }
$entry = Join-Path $SharedDir 'ai.py'
if (-not (Test-Path -LiteralPath $entry -PathType Leaf)) { throw "Missing personal harness: $entry" }
# auto uses managed copies on Windows; Developer Mode is unnecessary for copies.
$harnessArgs = if ($Agent -eq 'install') { @('install', '--mode', $Mode) } elseif ($Agent -in @('codex-desktop', 'claude-desktop', 'all')) { @('install', '--target', $Agent, '--mode', $Mode) } else { @('sync', '--target', $Agent, '--mode', $Mode) }
if (Get-Command py -ErrorAction SilentlyContinue) {
    & py -3 -c 'import sys; raise SystemExit(sys.version_info < (3, 9))'
    if ($LASTEXITCODE -ne 0) { throw 'Python 3.9 or later is required.' }
    & py -3 $entry @harnessArgs
}
elseif (Get-Command python -ErrorAction SilentlyContinue) {
    & python -c 'import sys; raise SystemExit(sys.version_info < (3, 9))'
    if ($LASTEXITCODE -ne 0) { throw 'Python 3.9 or later is required.' }
    & python $entry @harnessArgs
}
else { throw 'Python 3.9 or later is required.' }
exit $LASTEXITCODE
