[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('codex', 'claude', 'both')]
    [string]$Agent,
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
if (Get-Command py -ErrorAction SilentlyContinue) {
    & py -3 -c 'import sys; raise SystemExit(sys.version_info < (3, 9))'
    if ($LASTEXITCODE -ne 0) { throw 'Python 3.9 or later is required.' }
    & py -3 $entry sync --target $Agent --mode $Mode
}
elseif (Get-Command python -ErrorAction SilentlyContinue) {
    & python -c 'import sys; raise SystemExit(sys.version_info < (3, 9))'
    if ($LASTEXITCODE -ne 0) { throw 'Python 3.9 or later is required.' }
    & python $entry sync --target $Agent --mode $Mode
}
else { throw 'Python 3.9 or later is required.' }
exit $LASTEXITCODE
