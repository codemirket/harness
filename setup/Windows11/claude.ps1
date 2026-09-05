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
$claudeHome = Join-Path $env:USERPROFILE '.claude'

$deprecatedLink = Join-Path $claudeHome 'CAPABILITIES.md'
$existingItem = Get-ExistingItem -Path $deprecatedLink
if ($null -ne $existingItem -and (Test-SymbolicLink -Item $existingItem)) {
    Remove-Item -LiteralPath $deprecatedLink -Force
}

Set-SharedLink -SourcePath (Join-Path $SharedDir 'AGENTS.md') -DestinationPath (Join-Path $claudeHome 'CLAUDE.md')

$skillsDirectory = Join-Path $SharedDir 'skills'
$localSkillsDirectory = Join-Path $claudeHome 'skills'
New-Item -ItemType Directory -Path $localSkillsDirectory -Force | Out-Null
$sharedSkillsPrefix = $skillsDirectory + [System.IO.Path]::DirectorySeparatorChar

Get-ChildItem -LiteralPath $localSkillsDirectory -Force | ForEach-Object {
    if (Test-SymbolicLink -Item $_) {
        $targetPath = [string]$_.Target
        if ($targetPath.StartsWith($sharedSkillsPrefix, [System.StringComparison]::OrdinalIgnoreCase) -and
            -not (Test-Path -LiteralPath (Join-Path $_.FullName 'SKILL.md') -PathType Leaf)) {
            Remove-Item -LiteralPath $_.FullName -Force
        }
    }
}

if (Test-Path -LiteralPath $skillsDirectory -PathType Container) {
    Get-ChildItem -LiteralPath $skillsDirectory -Directory | ForEach-Object {
        Get-ChildItem -LiteralPath $_.FullName -Directory | ForEach-Object {
            if (Test-Path -LiteralPath (Join-Path $_.FullName 'SKILL.md') -PathType Leaf) {
                Set-SharedLink -SourcePath $_.FullName -DestinationPath (Join-Path $localSkillsDirectory $_.Name)
            }
        }
    }
}

Write-Output 'Claude shared configuration links are active.'
