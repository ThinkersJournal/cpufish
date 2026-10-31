# SPDX-License-Identifier: MIT OR Apache-2.0
# Installs OpenBLAS for CPUFish into the admin-owned, read-only location described in docs/NATIVE-LIBS.md:
#
#   %ProgramFiles%\OpenBLAS\<version>\lp64\libopenblas.dll     (from OpenBLAS-<version>-x64.zip,    32-bit ints)
#   %ProgramFiles%\OpenBLAS\<version>\ilp64\libopenblas.dll    (from OpenBLAS-<version>-x64-64.zip, 64-bit ints)
#
# It downloads the two release zips (or takes them from -ZipDir), checks BOTH SHA-256 values against the pinned table
# below, and only then extracts the one file it needs. A mismatch stops the script before anything is extracted or
# written to Program Files. It never edits PATH and never loads the DLL.
#
#   Install (elevated PowerShell):   .\install-openblas.ps1
#   Check only (no admin, no install): .\install-openblas.ps1 -VerifyOnly
#   Zips already downloaded:           .\install-openblas.ps1 -ZipDir D:\downloads
#
# Works in Windows PowerShell 5.1 and PowerShell 7.
[CmdletBinding()]
param(
    [string]$Version = '0.3.34',
    [string]$ZipDir,
    [switch]$VerifyOnly
)
$ErrorActionPreference = 'Stop'

# Pinned releases: zip name and SHA-256 per integer width. Verified by the PM against the OpenMathLib release API
# (2026-10-31). Adding a version means adding a row here, in the same change that updates docs/NATIVE-LIBS.md.
$pins = @{
    '0.3.34' = @{
        lp64  = @{ Zip = 'OpenBLAS-0.3.34-x64.zip';    Sha256 = 'e9cb6134541f36c27346d5fc5995652f060fba227cebbbabcbda5a5a44d7c76b' }
        ilp64 = @{ Zip = 'OpenBLAS-0.3.34-x64-64.zip'; Sha256 = '55a5ff6f9e734a1d3caf84e79f2f8209ec34353e312c4d493853d4029ce4e0a9' }
    }
}
if (-not $pins.ContainsKey($Version)) { throw "OpenBLAS $Version is not pinned in this script (pinned: $($pins.Keys -join ', '))" }

# Well-known SIDs, so this works on any Windows language.
$sidAdmins = New-Object Security.Principal.SecurityIdentifier 'S-1-5-32-544'
$sidSystem = New-Object Security.Principal.SecurityIdentifier 'S-1-5-18'
$sidUsers  = New-Object Security.Principal.SecurityIdentifier 'S-1-5-32-545'
$sidEvery  = New-Object Security.Principal.SecurityIdentifier 'S-1-1-0'
$sidAuth   = New-Object Security.Principal.SecurityIdentifier 'S-1-5-11'

if (-not $VerifyOnly) {
    $me = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    if (-not $me.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw 'Run this from an elevated (Administrator) PowerShell, or pass -VerifyOnly.'
    }
}

[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
$work = Join-Path ([IO.Path]::GetTempPath()) ("cpufish-openblas-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
try {
    # 1. Get both zips and hash them. Nothing is extracted until both match.
    $zips = @{}
    foreach ($w in 'lp64', 'ilp64') {
        $pin = $pins[$Version][$w]
        $dest = Join-Path $work $pin.Zip
        if ($ZipDir) {
            $src = Join-Path $ZipDir $pin.Zip
            if (-not (Test-Path -LiteralPath $src)) { throw "$src not found" }
            Copy-Item -LiteralPath $src -Destination $dest
        } else {
            $url = "https://github.com/OpenMathLib/OpenBLAS/releases/download/v$Version/$($pin.Zip)"
            "downloading $url"
            Invoke-WebRequest -Uri $url -OutFile $dest -UseBasicParsing
        }
        $got = (Get-FileHash -LiteralPath $dest -Algorithm SHA256).Hash.ToLowerInvariant()
        "{0,-30} {1,10} bytes  sha256 {2}" -f $pin.Zip, (Get-Item -LiteralPath $dest).Length, $got
        if ($got -ne $pin.Sha256) { throw "SHA-256 MISMATCH for $($pin.Zip): expected $($pin.Sha256), got $got. Nothing was installed." }
        $zips[$w] = $dest
    }
    'both zips match the pinned SHA-256 values'
    if ($VerifyOnly) { return }

    # 2. Extract only bin/libopenblas.dll, by exact entry name, straight to a staging file (no path from the zip is used).
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $staged = @{}
    foreach ($w in 'lp64', 'ilp64') {
        $zip = [IO.Compression.ZipFile]::OpenRead($zips[$w])
        try {
            $entry = $zip.Entries | Where-Object { $_.FullName -eq 'bin/libopenblas.dll' } | Select-Object -First 1
            if (-not $entry) { throw "$($pins[$Version][$w].Zip) has no bin/libopenblas.dll" }
            $file = Join-Path $work "$w-libopenblas.dll"
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $file)
            $staged[$w] = $file
        } finally { $zip.Dispose() }
    }

    # 3. Install under Program Files, one directory per width (the two builds share the file name, so never one directory).
    $root = Join-Path $env:ProgramFiles "OpenBLAS\$Version"
    foreach ($w in 'lp64', 'ilp64') {
        $dir = Join-Path $root $w
        $target = Join-Path $dir 'libopenblas.dll'
        $newHash = (Get-FileHash -LiteralPath $staged[$w] -Algorithm SHA256).Hash.ToLowerInvariant()
        if (Test-Path -LiteralPath $target) {
            $oldHash = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
            if ($oldHash -ne $newHash) { throw "$target exists with different content ($oldHash); remove that directory yourself if it is wrong. Not overwriting." }
            "$target already installed, same content"
        } else {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
            Copy-Item -LiteralPath $staged[$w] -Destination $target
        }
        $pin = $pins[$Version][$w]
        @(
            "library : OpenBLAS $Version ($w)"
            "source  : https://github.com/OpenMathLib/OpenBLAS/releases/download/v$Version/$($pin.Zip)"
            "zip     : $($pin.Zip) sha256 $($pin.Sha256)"
            "dll     : bin/libopenblas.dll sha256 $newHash"
            "installed: $([DateTime]::UtcNow.ToString('o')) by tools/install-openblas.ps1"
        ) | Set-Content -LiteralPath (Join-Path $dir 'MANIFEST.txt') -Encoding ASCII
    }

    # 4. Lock it down: owner Administrators; Administrators and SYSTEM full control; Users read and execute only.
    & icacls.exe $root /setowner '*S-1-5-32-544' /T /C | Out-Null
    & icacls.exe $root /inheritance:r /T /C | Out-Null
    & icacls.exe $root /grant:r '*S-1-5-32-544:(OI)(CI)F' '*S-1-5-18:(OI)(CI)F' '*S-1-5-32-545:(OI)(CI)RX' /T /C | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "icacls failed ($LASTEXITCODE)" }

    # 5. Read it back. Nothing above is trusted until this passes.
    ''
    '--- read-back ---'
    $bad = @()
    foreach ($w in 'lp64', 'ilp64') {
        $target = Join-Path $root "$w\libopenblas.dll"
        $pin = $pins[$Version][$w]
        $dllHash = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
        $want = (Get-Content -LiteralPath (Join-Path $root "$w\MANIFEST.txt") | Where-Object { $_ -like 'dll*' }) -replace '^.*sha256 ', ''
        "$target  sha256 $dllHash  (manifest: $want)"
        if ($dllHash -ne $want) { $bad += "$target does not match its manifest" }
    }
    foreach ($item in (Get-ChildItem -LiteralPath $root -Recurse -Force) + (Get-Item -LiteralPath $root)) {
        $acl = Get-Acl -LiteralPath $item.FullName
        $owner = $acl.GetOwner([Security.Principal.SecurityIdentifier])
        if ($owner.Value -ne $sidAdmins.Value -and $owner.Value -ne $sidSystem.Value) { $bad += "$($item.FullName): owner is $owner, not Administrators/SYSTEM" }
        foreach ($ace in $acl.Access) {
            $sid = $ace.IdentityReference.Translate([Security.Principal.SecurityIdentifier])
            if ($ace.AccessControlType -eq 'Allow' -and (@($sidUsers.Value, $sidEvery.Value, $sidAuth.Value) -contains $sid.Value) -and
                ($ace.FileSystemRights.ToString() -match 'Write|Modify|FullControl|Delete|TakeOwnership|ChangePermissions|AppendData|CreateFiles')) {
                $bad += "$($item.FullName): $sid can $($ace.FileSystemRights)"
            }
        }
    }
    & icacls.exe $root
    $path = @([Environment]::GetEnvironmentVariable('Path', 'Machine'), [Environment]::GetEnvironmentVariable('Path', 'User')) -join ';'
    if ($path -like "*OpenBLAS*") { $bad += 'an OpenBLAS directory is on PATH (machine or user); remove it, the loader does not use PATH' } else { 'not on PATH (machine or user): ok' }
    if ($bad.Count -gt 0) { ''; 'READ-BACK FAILED:'; $bad | ForEach-Object { "  $_" }; throw 'read-back failed' }
    ''
    "OK: OpenBLAS $Version installed under $root, owner Administrators, read-only for non-admin accounts."
    "CPUFish finds it with no setting needed on Windows; to use another copy set CPUFISH_OPENBLAS to an absolute path."
} finally {
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
