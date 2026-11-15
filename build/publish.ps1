<#
.SYNOPSIS
    ImageViewer 를 Self-contained 로 게시하고, NSIS 가 설치되어 있으면 설치 프로그램을 만든다.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File build\publish.ps1
    powershell -ExecutionPolicy Bypass -File build\publish.ps1 -Runtime win-arm64 -SkipInstaller
#>
param(
    [string]$Runtime = "win-x64",
    [string]$Configuration = "Release",
    [switch]$SkipTests,
    [switch]$SkipInstaller
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$project = Join-Path $root "src\ImageViewer\ImageViewer.csproj"
$output = Join-Path $root "artifacts\publish\$Runtime"

if (-not $SkipTests) {
    Write-Host "== 테스트 실행" -ForegroundColor Cyan
    dotnet test (Join-Path $root "ImageViewer.sln") -c $Configuration
    if ($LASTEXITCODE -ne 0) { throw "테스트 실패" }
}

Write-Host "== 게시: $Runtime -> $output" -ForegroundColor Cyan
if (Test-Path $output) { Remove-Item $output -Recurse -Force }
dotnet publish $project -c $Configuration -r $Runtime --self-contained true -o $output `
    -p:PublishReadyToRun=true -p:DebugType=none
if ($LASTEXITCODE -ne 0) { throw "게시 실패" }

if ($SkipInstaller) { return }

if ($Runtime -ne "win-x64") {
    Write-Warning "설치 프로그램은 win-x64 게시본만 지원합니다. 설치 프로그램 생성을 건너뜁니다."
    return
}

$makensis = @(
    "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
    "$env:ProgramFiles\NSIS\makensis.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $makensis) {
    $makensis = (Get-Command makensis -ErrorAction SilentlyContinue).Source
}

if (-not $makensis) {
    Write-Warning "NSIS (makensis.exe) 를 찾지 못해 설치 프로그램 생성을 건너뜁니다. https://nsis.sourceforge.io"
    return
}

[xml]$csproj = Get-Content $project -Encoding UTF8
$version = ($csproj.Project.PropertyGroup | Where-Object { $_.Version } | Select-Object -First 1).Version
if (-not $version) { $version = "1.0.0" }

Write-Host "== 설치 프로그램 생성 (NSIS, v$version)" -ForegroundColor Cyan
New-Item -ItemType Directory -Force (Join-Path $root "artifacts\installer") | Out-Null
& $makensis /V2 "/DVERSION=$version" "/DPUBLISH_DIR=$output" (Join-Path $root "src\ImageViewer.Installer\ImageViewer.nsi")
if ($LASTEXITCODE -ne 0) { throw "설치 프로그램 생성 실패" }
Get-Item (Join-Path $root "artifacts\installer\ImageViewer-$version-setup.exe")
