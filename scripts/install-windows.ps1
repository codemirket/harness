# Installs one explicitly selected local Mirket release for Codex Desktop.
# Video releases acquire one pinned Chrome archive into private staging.
# Windows execution remains a native release gate until x64 and ARM64 hosts run it.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Source,
    [Parameter(Mandatory = $true)][string]$ManifestSha256,
    [Parameter(Mandatory = $true)][string]$BinarySha256,
    [string]$VideoChromeArchive,
    [string]$UserHome = [Environment]::GetFolderPath('UserProfile'),
    [string]$ProjectRoot,
    [switch]$Update,
    [switch]$Reinstall,
    [switch]$ReconcileReinstall,
    [switch]$ReconcileUninstall
)

$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'This installer requires Windows'
}
function Assert-Digest([string]$Value, [string]$Name) {
    if (-not [regex]::IsMatch($Value, '^[0-9a-f]{64}$')) {
        throw "$Name must be 64 lowercase hexadecimal characters"
    }
}
function Assert-CanonicalDirectory([string]$Value, [string]$Name) {
    $full = [IO.Path]::GetFullPath($Value)
    if ($full.StartsWith('\\', [StringComparison]::Ordinal)) {
        throw "$Name must be on a local volume; UNC paths are unqualified"
    }
    if (-not [string]::Equals($full, $Value, [StringComparison]::OrdinalIgnoreCase)) {
        throw "$Name must be an absolute canonical directory"
    }
    $item = Get-Item -LiteralPath $full -Force
    if (-not $item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "$Name must be a real directory"
    }
    return $full
}
function Assert-FileHash([string]$Path, [string]$Expected) {
    $item = Get-Item -LiteralPath $Path -Force
    if ($item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Release file must be regular and unlinked: $Path"
    }
    $actual = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -cne $Expected) { throw "SHA-256 mismatch for $Path" }
}
function Copy-PlainTree([string]$From, [string]$To) {
    [IO.Directory]::CreateDirectory($To) | Out-Null
    $pending = New-Object 'System.Collections.Generic.Stack[object]'
    $pending.Push(@($From, $To))
    $entries = 0
    $bytes = [long]0
    while ($pending.Count -gt 0) {
        $pair = $pending.Pop()
        foreach ($item in [IO.Directory]::EnumerateFileSystemEntries($pair[0])) {
            $entries++
            if ($entries -gt 16000) { throw 'Release source exceeds staging entry limit' }
            $info = [IO.File]::GetAttributes($item)
            if ($info -band [IO.FileAttributes]::ReparsePoint) {
                throw "Release source contains a link or reparse point: $item"
            }
            $next = Join-Path $pair[1] ([IO.Path]::GetFileName($item))
            if ($info -band [IO.FileAttributes]::Directory) {
                [IO.Directory]::CreateDirectory($next) | Out-Null
                $pending.Push(@($item, $next))
            }
            else {
                $bytes += ([IO.FileInfo]::new($item)).Length
                if ($bytes -gt 4294967296) { throw 'Release source exceeds staging byte limit' }
                [IO.File]::Copy($item, $next, $false)
            }
        }
    }
}

Assert-Digest $ManifestSha256 'ManifestSha256'
Assert-Digest $BinarySha256 'BinarySha256'
if ((@($Update, $Reinstall, $ReconcileReinstall, $ReconcileUninstall) | Where-Object { $_ }).Count -gt 1) {
    throw 'Choose only one lifecycle action'
}
if (($Reinstall -or $ReconcileReinstall -or $ReconcileUninstall) -and $ProjectRoot) {
    throw 'ProjectRoot is not used by lifecycle reconciliation or reinstall'
}
$Source = Assert-CanonicalDirectory $Source 'Source'
$UserHome = Assert-CanonicalDirectory $UserHome 'UserHome'
if ($ProjectRoot) { $ProjectRoot = Assert-CanonicalDirectory $ProjectRoot 'ProjectRoot' }
$manifest = Join-Path $Source 'manifest.json'
$binary = Join-Path (Join-Path $Source 'bin') 'mirket.exe'
Assert-FileHash $manifest $ManifestSha256
$nativeArchitecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
$requiredTarget = switch ($nativeArchitecture) {
    'X64' { 'x86_64-pc-windows-msvc' }
    'Arm64' { 'aarch64-pc-windows-msvc' }
    default { throw "Unsupported Windows architecture: $nativeArchitecture" }
}
$releaseManifest = Get-Content -LiteralPath $manifest -Raw -Encoding UTF8 | ConvertFrom-Json
if ($releaseManifest.target -cne $requiredTarget) {
    throw "Release target must match native Windows architecture: $requiredTarget"
}

# Execute only a verified private copy. Setup/update owns full payload and
# managed-destination verification; this script never fetches a package or
# resolves another Mirket executable from PATH.
$scratch = Join-Path ([IO.Path]::GetTempPath()) ('mirket-install-' + [guid]::NewGuid().ToString('N'))
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$acl = New-Object Security.AccessControl.DirectorySecurity
$acl.SetAccessRuleProtection($true, $false)
$inheritance = [Security.AccessControl.InheritanceFlags]::ContainerInherit -bor [Security.AccessControl.InheritanceFlags]::ObjectInherit
$rule = [Security.AccessControl.FileSystemAccessRule]::new(
    $identity.User,
    [Security.AccessControl.FileSystemRights]::FullControl,
    $inheritance,
    [Security.AccessControl.PropagationFlags]::None,
    [Security.AccessControl.AccessControlType]::Allow
)
$acl.AddAccessRule($rule)
# Both APIs apply the ACL during creation. The Directory overload exists only
# in .NET Framework; PowerShell 7 uses the .NET ACL extension instead.
if ($PSVersionTable.PSEdition -eq 'Core') {
    [IO.FileSystemAclExtensions]::CreateDirectory($acl, $scratch) | Out-Null
}
else {
    [IO.Directory]::CreateDirectory($scratch, $acl) | Out-Null
}
try {
    $privateBinary = Join-Path $scratch 'mirket.exe'
    Copy-Item -LiteralPath $binary -Destination $privateBinary -ErrorAction Stop
    Assert-FileHash $privateBinary $BinarySha256
    $installSource = $Source
    if (-not $ReconcileUninstall -and @($releaseManifest.components) -ccontains 'video') {
        $runtime = Join-Path $Source 'runtime'
        $browser = Join-Path $runtime 'browser'
        if (-not (Test-Path -LiteralPath $runtime -PathType Container) -or
            ([IO.File]::GetAttributes($runtime) -band [IO.FileAttributes]::ReparsePoint) -or
            (Test-Path -LiteralPath $browser)) {
            throw 'Video Chrome source must be absent and runtime must be a real directory'
        }
        $installSource = Join-Path $scratch 'release'
        Copy-PlainTree $Source $installSource
        $archive = Join-Path $scratch 'chrome-win64.zip'
        if ($VideoChromeArchive) {
            if (-not [IO.Path]::IsPathFullyQualified($VideoChromeArchive)) {
                throw 'VideoChromeArchive must be an absolute path'
            }
            $VideoChromeArchive = [IO.Path]::GetFullPath($VideoChromeArchive)
            Assert-FileHash $VideoChromeArchive 'a4e0d57245e167e98a9d465af1e3774d2b4ec9f04e483d19bcd97a6f63cf46dd'
            [IO.File]::Copy($VideoChromeArchive, $archive, $false)
        }
        else {
            Invoke-WebRequest -Uri 'https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.47/win64/chrome-win64.zip' -OutFile $archive -MaximumRedirection 0
        }
        if (([IO.FileInfo]::new($archive)).Length -ne 209154801) {
            throw 'Video Chrome archive size mismatch'
        }
        Assert-FileHash $archive 'a4e0d57245e167e98a9d465af1e3774d2b4ec9f04e483d19bcd97a6f63cf46dd'
        $browserStage = Join-Path (Join-Path $installSource 'runtime') 'browser'
        [IO.Directory]::CreateDirectory($browserStage) | Out-Null
        Expand-Archive -LiteralPath $archive -DestinationPath $browserStage -Force
        # Mirket checks every extracted file against the compiled release
        # inventory before touching managed destinations.
    }
    elseif ($VideoChromeArchive) {
        throw 'VideoChromeArchive was supplied for a release without video'
    }
    $action = if ($ReconcileUninstall) { 'uninstall-reconcile' }
        elseif ($ReconcileReinstall) { 'reinstall-reconcile' }
        elseif ($Reinstall) { 'reinstall' }
        elseif ($Update) { 'update' }
        else { 'setup' }
    $arguments = @($action, '--home', (Join-Path $UserHome '.mirket'), '--user-home', $UserHome)
    if (-not $ReconcileUninstall) { $arguments += @('--source', $installSource) }
    if ($ProjectRoot) { $arguments += @('--project-root', $ProjectRoot) }
    & $privateBinary @arguments
    if ($LASTEXITCODE -ne 0) { throw "Mirket $action failed with exit code $LASTEXITCODE" }
}
finally {
    if (Test-Path -LiteralPath $scratch) {
        Remove-Item -LiteralPath $scratch -Recurse -Force
    }
}
