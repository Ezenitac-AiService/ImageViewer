param(
    [string]$Version = "1.1.0",
    [switch]$BuildInstaller
)

$ErrorActionPreference = "Stop"
cargo build --release --target x86_64-pc-windows-msvc -p iv-app
$outDir = "target/x86_64-pc-windows-msvc/release"
New-Item -ItemType Directory -Force -Path "packaging/windows/dist" | Out-Null
Copy-Item "$outDir/iv-app.exe" "packaging/windows/dist/ImageViewer.exe" -Force

if ($BuildInstaller) {
    makensis /DPRODUCT_VERSION=$Version src/ImageViewer.Installer/ImageViewer.nsi
}
