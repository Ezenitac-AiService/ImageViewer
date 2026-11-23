# Builds the portable Windows release executable and verifies it
# (specs/004-launch-reliability-refactor/contracts/release-verification-contract.md §1).
#
# Usage: pwsh packaging/windows/package-windows.ps1
# Output: dist/ImageViewer-windows-x64.exe
#
# Optional signing (FR-005): when WINDOWS_SIGNING_CERT_BASE64 and WINDOWS_SIGNING_PASSWORD are
# set, the executable is signed with signtool and verified; otherwise it ships unsigned and the
# script prints `signature=none`.

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..\..')
Set-Location $root

$target = 'x86_64-pc-windows-msvc'
cargo build --release --locked --target $target -p iv-app
if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }

$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
$exe = Join-Path $dist 'ImageViewer-windows-x64.exe'
Copy-Item (Join-Path $root "target\$target\release\iv-app.exe") $exe -Force

if ($env:WINDOWS_SIGNING_CERT_BASE64 -and $env:WINDOWS_SIGNING_PASSWORD) {
    $signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if (-not $signtool) {
        $signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
            Sort-Object FullName -Descending | Select-Object -First 1
    }
    if (-not $signtool) { throw 'signing requested but signtool.exe was not found' }
    $pfx = Join-Path ([System.IO.Path]::GetTempPath()) ("iv-sign-" + [guid]::NewGuid() + '.pfx')
    try {
        [System.IO.File]::WriteAllBytes($pfx, [Convert]::FromBase64String($env:WINDOWS_SIGNING_CERT_BASE64))
        & $signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /f $pfx /p $env:WINDOWS_SIGNING_PASSWORD $exe
        if ($LASTEXITCODE -ne 0) { throw "signtool sign failed ($LASTEXITCODE)" }
        & $signtool verify /pa $exe
        if ($LASTEXITCODE -ne 0) { throw "signtool verify failed ($LASTEXITCODE)" }
        Write-Output 'signature=publisher'
    } finally {
        Remove-Item $pfx -Force -ErrorAction SilentlyContinue
    }
} else {
    Write-Output 'signature=none'
}

& (Join-Path $PSScriptRoot '..\verify\assert-windows-exe.ps1') $exe
if ($LASTEXITCODE -ne 0) { throw 'Windows executable assertions failed' }
Write-Output "built $exe"
