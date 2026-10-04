[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('codex', 'claude')]
    [string]$Agent,
    [string]$SharedDir
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($SharedDir)) {
    $SharedDir = Join-Path $PSScriptRoot '..'
}
$sharedItem = Get-Item -LiteralPath $SharedDir -Force -ErrorAction SilentlyContinue
if ($null -eq $sharedItem -or -not $sharedItem.PSIsContainer) {
    throw "Missing shared directory: $SharedDir"
}
$sourcePath = Join-Path $sharedItem.FullName 'components\AGENTS.md'
if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
    throw "Missing shared source: $sourcePath"
}

if ($Agent -eq 'codex') {
    $agentDir = Join-Path $env:USERPROFILE '.codex'
    $instructionFile = 'AGENTS.md'
}
else {
    $agentDir = Join-Path $env:USERPROFILE '.claude'
    $instructionFile = 'CLAUDE.md'
}
$destinationPath = Join-Path $agentDir $instructionFile
$existingItem = Get-Item -LiteralPath $destinationPath -Force -ErrorAction SilentlyContinue
if ($null -ne $existingItem) {
    if ($existingItem.LinkType -ne 'SymbolicLink') {
        throw "Refusing to replace non-symbolic-link: $destinationPath"
    }
    if ([string]::Equals([string]$existingItem.Target, $sourcePath, [System.StringComparison]::OrdinalIgnoreCase)) {
        Write-Output "$destinationPath is already linked."
        return
    }
}

New-Item -ItemType Directory -Path $agentDir -Force | Out-Null
$temporaryPath = Join-Path $agentDir ('.{0}.new-{1}' -f $instructionFile, [guid]::NewGuid().ToString('N'))
try {
    New-Item -ItemType SymbolicLink -Path $temporaryPath -Target $sourcePath | Out-Null
}
catch {
    throw "Unable to create a symbolic link. Enable Windows Developer Mode or run PowerShell as Administrator, then retry. Original error: $($_.Exception.Message)"
}

try {
    if ($null -ne $existingItem) {
        Remove-Item -LiteralPath $destinationPath -Force
    }
    Move-Item -LiteralPath $temporaryPath -Destination $destinationPath
}
finally {
    if (Test-Path -LiteralPath $temporaryPath) {
        Remove-Item -LiteralPath $temporaryPath -Force
    }
}
Write-Output "Linked $destinationPath -> $sourcePath"
