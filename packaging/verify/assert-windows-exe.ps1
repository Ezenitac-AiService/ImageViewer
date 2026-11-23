# Build-time assertions for the Windows executable
# (specs/004-launch-reliability-refactor/contracts/release-verification-contract.md §1).
#
# Usage: pwsh packaging/verify/assert-windows-exe.ps1 <path\to\ImageViewer.exe>
# Exits non-zero if any check fails; prints one line per check.
param(
    [Parameter(Mandatory = $true)][string]$Exe
)

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$script:status = 0

function Check([string]$Name, [scriptblock]$Test) {
    try {
        if (& $Test) { Write-Output "PASS $Name" } else { Write-Output "FAIL $Name"; $script:status = 1 }
    } catch {
        Write-Output "FAIL $Name ($($_.Exception.Message))"
        $script:status = 1
    }
}

$bytes = [System.IO.File]::ReadAllBytes((Resolve-Path $Exe))
$peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
$optHeader = $peOffset + 24
$sectionCount = [BitConverter]::ToUInt16($bytes, $peOffset + 6)
$optSize = [BitConverter]::ToUInt16($bytes, $peOffset + 20)

function Get-Subsystem { [BitConverter]::ToUInt16($bytes, $optHeader + 68) }

function Get-SectionSize([string]$Name) {
    $table = $optHeader + $optSize
    for ($i = 0; $i -lt $sectionCount; $i++) {
        $entry = $table + 40 * $i
        $secName = [System.Text.Encoding]::ASCII.GetString($bytes, $entry, 8).TrimEnd([char]0)
        if ($secName -eq $Name) { return [BitConverter]::ToUInt32($bytes, $entry + 8) }
    }
    return 0
}

function Get-ImportedDlls {
    $dumpbin = Get-Command dumpbin.exe -ErrorAction SilentlyContinue
    if ($dumpbin) {
        return (& $dumpbin.Source /nologo /dependents $Exe) | Where-Object { $_ -match '\.dll\s*$' } | ForEach-Object { $_.Trim() }
    }
    # Fallback: import names are stored as ASCII strings in the image.
    $text = [System.Text.Encoding]::ASCII.GetString($bytes)
    return [regex]::Matches($text, '[A-Za-z0-9_\-\.]+\.dll') | ForEach-Object { $_.Value } | Sort-Object -Unique
}

$workspaceVersion = (cargo metadata --format-version 1 --no-deps --manifest-path (Join-Path $root 'Cargo.toml') |
        ConvertFrom-Json).packages | Where-Object { $_.name -eq 'iv-app' } | Select-Object -ExpandProperty version
$numericVersion = ($workspaceVersion -split '-')[0]

Check 'gui-subsystem' { (Get-Subsystem) -eq 2 }
Check 'no-dynamic-crt' {
    $bad = Get-ImportedDlls | Where-Object { $_ -match '^(vcruntime|msvcp|api-ms-win-crt-)' }
    if ($bad) { Write-Output "  dynamic CRT imports: $($bad -join ', ')" }
    -not $bad
}
Check 'file-version-matches-workspace' {
    $info = (Get-Item $Exe).VersionInfo
    $info.FileVersion -and $info.FileVersion.StartsWith($numericVersion)
}
Check 'product-name' { (Get-Item $Exe).VersionInfo.ProductName -eq 'ImageViewer' }
Check 'resources-present' { (Get-SectionSize '.rsrc') -ge 1024 }

exit $script:status
