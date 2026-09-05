[CmdletBinding()]
param(
    [string]$SharedDir,
    [ValidateSet('all', 'codex', 'claude')][string]$Agent = 'all'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($SharedDir)) {
    $SharedDir = (Get-Item -LiteralPath (Join-Path $PSScriptRoot '..\..')).FullName
}
Get-Command node -ErrorAction Stop | Out-Null
Get-Command npm -ErrorAction Stop | Out-Null
& npm --prefix (Join-Path $SharedDir 'mcp\containers') run setup -- --agent $Agent
if ($LASTEXITCODE -ne 0) { throw 'Containers MCP setup failed.' }
