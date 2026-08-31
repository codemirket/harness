[CmdletBinding()]
param(
    [string]$SharedDir
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($SharedDir)) {
    $SharedDir = Join-Path $PSScriptRoot '..\..'
}

function Get-ExistingItem {
    param([Parameter(Mandatory = $true)][string]$Path)

    Get-Item -LiteralPath $Path -Force -ErrorAction SilentlyContinue
}

function Test-SymbolicLink {
    param([Parameter(Mandatory = $true)][System.IO.FileSystemInfo]$Item)

    $isReparsePoint = ($Item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0
    return $isReparsePoint -and $Item.LinkType -eq 'SymbolicLink'
}

function Set-SharedLink {
    param(
        [Parameter(Mandatory = $true)][string]$SourcePath,
        [Parameter(Mandatory = $true)][string]$DestinationPath
    )

    if (-not (Test-Path -LiteralPath $SourcePath)) {
        throw "Missing shared source: $SourcePath"
    }

    $parentPath = Split-Path -Parent $DestinationPath
    New-Item -ItemType Directory -Path $parentPath -Force | Out-Null

    $existingItem = Get-ExistingItem -Path $DestinationPath
    if ($null -ne $existingItem -and -not (Test-SymbolicLink -Item $existingItem)) {
        throw "Refusing to replace non-symbolic-link: $DestinationPath"
    }

    $temporaryPath = Join-Path $parentPath ('.{0}.new-{1}' -f (Split-Path -Leaf $DestinationPath), [guid]::NewGuid().ToString('N'))

    try {
        New-Item -ItemType SymbolicLink -Path $temporaryPath -Target $SourcePath | Out-Null
    }
    catch {
        throw "Unable to create a symbolic link. Enable Windows Developer Mode or run PowerShell as Administrator, then try again. Original error: $($_.Exception.Message)"
    }

    try {
        if ($null -ne $existingItem) {
            Remove-Item -LiteralPath $DestinationPath -Force
        }

        Move-Item -LiteralPath $temporaryPath -Destination $DestinationPath
    }
    finally {
        $temporaryItem = Get-ExistingItem -Path $temporaryPath
        if ($null -ne $temporaryItem) {
            Remove-Item -LiteralPath $temporaryPath -Force
        }
    }
}

$sharedDirItem = Get-ExistingItem -Path $SharedDir
if ($null -eq $sharedDirItem -or -not $sharedDirItem.PSIsContainer) {
    throw "Shared directory was not found: $SharedDir. Set -SharedDir to the local .ai directory."
}

$SharedDir = $sharedDirItem.FullName
$codexHome = Join-Path $env:USERPROFILE '.codex'

foreach ($deprecatedLink in @(
    (Join-Path $codexHome 'CAPABILITIES.md'),
    (Join-Path $codexHome 'shared.config.toml')
)) {
    $existingItem = Get-ExistingItem -Path $deprecatedLink
    if ($null -ne $existingItem -and (Test-SymbolicLink -Item $existingItem)) {
        Remove-Item -LiteralPath $deprecatedLink -Force
    }
}

Set-SharedLink -SourcePath (Join-Path $SharedDir 'AGENTS.md') -DestinationPath (Join-Path $codexHome 'AGENTS.md')

$windmillSkillsDirectory = Join-Path $SharedDir 'skills\windmill'
if (-not (Test-Path -LiteralPath $windmillSkillsDirectory -PathType Container)) {
    throw "Missing shared skills directory: $windmillSkillsDirectory"
}

Get-ChildItem -LiteralPath $windmillSkillsDirectory -Directory | ForEach-Object {
    Set-SharedLink -SourcePath $_.FullName -DestinationPath (Join-Path (Join-Path $codexHome 'skills') $_.Name)
}

Write-Output 'Codex shared configuration links are active.'
