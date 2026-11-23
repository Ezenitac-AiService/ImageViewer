# Automated launch check for Windows
# (specs/004-launch-reliability-refactor/contracts/release-verification-contract.md §2–§3).
#
# Usage: pwsh packaging/verify/launch-check.ps1 <ImageViewer.exe> <reference-image> [record.json]
#
# Starts the executable as a user would, waits for the `first_frame` diagnostic record, checks the
# main window exists and keeps responding (the reported Windows symptom), checks no console host
# is attached, optionally steps to the next image, asks the window to close the way Windows does,
# and requires a clean exit. Writes a Launch Verification Record (data-model.md §2).
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$Image,
    [string]$Record = 'launch-verification-windows-x64.json'
)

$ErrorActionPreference = 'Stop'
$Exe = (Resolve-Path $Exe).Path
$Image = (Resolve-Path $Image).Path
$logDir = Join-Path $env:LOCALAPPDATA 'ImageViewer\logs'
$log = Join-Path $logDir ("imageviewer-{0}.log" -f [DateTime]::UtcNow.ToString('yyyy-MM-dd'))
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$offset = if (Test-Path $log) { (Get-Item $log).Length } else { 0 }

function New-LogText {
    if (-not (Test-Path $log)) { return '' }
    $stream = [System.IO.File]::Open($log, 'Open', 'Read', 'ReadWrite')
    try {
        $null = $stream.Seek($offset, 'Begin')
        return (New-Object System.IO.StreamReader($stream)).ReadToEnd()
    } finally { $stream.Dispose() }
}

function Wait-For([string]$Pattern, [int]$Seconds) {
    $deadline = (Get-Date).AddSeconds($Seconds)
    while ((Get-Date) -lt $deadline) {
        if ((New-LogText) -match [regex]::Escape($Pattern)) { return $true }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

$windowShown = $false; $firstFrame = $false; $responding = $true; $consoleWindow = $false
$navigated = $null; $cleanExit = $false; $exitCode = -1; $notes = ''

$appOut = [System.IO.Path]::GetTempFileName()
$proc = Start-Process -FilePath $Exe -ArgumentList ('"{0}"' -f $Image) -PassThru -RedirectStandardError $appOut

# Window within 30 s.
$deadline = (Get-Date).AddSeconds(30)
while ((Get-Date) -lt $deadline) {
    $proc.Refresh()
    if ($proc.MainWindowHandle -ne [IntPtr]::Zero) { $windowShown = $true; break }
    if ($proc.HasExited) { break }
    Start-Sleep -Milliseconds 250
}
$firstFrame = Wait-For ("phase=first_frame source=" + (Split-Path $Image -Leaf)) 30

# Responsiveness: 20 samples over 10 s (SC-012).
for ($i = 0; $i -lt 20; $i++) {
    $proc.Refresh()
    if ($proc.HasExited -or -not $proc.Responding) { $responding = $false; break }
    Start-Sleep -Milliseconds 500
}

# No console host attached to a GUI-subsystem process (FR-003).
$children = Get-CimInstance Win32_Process -Filter "ParentProcessId = $($proc.Id)" -ErrorAction SilentlyContinue
if ($children | Where-Object { $_.Name -ieq 'conhost.exe' }) { $consoleWindow = $true }

# Interaction step (§3): Right Arrow → navigate index=1.
try {
    Add-Type -AssemblyName Microsoft.VisualBasic
    Add-Type -AssemblyName System.Windows.Forms
    Start-Sleep -Seconds 1
    foreach ($attempt in 1..2) {
        [Microsoft.VisualBasic.Interaction]::AppActivate($proc.Id)
        Start-Sleep -Milliseconds 500
        [System.Windows.Forms.SendKeys]::SendWait('{RIGHT}')
        $navigated = Wait-For 'phase=navigate index=1' 2
        if ($navigated) { break }
    }
} catch {
    $notes = 'interaction step moved to clean-machine (synthetic input not permitted)'
}

# Close the way Windows does.
if (-not $proc.HasExited) { $null = $proc.CloseMainWindow() }
if ($proc.WaitForExit(10000)) {
    $exitCode = $proc.ExitCode
    if ($exitCode -eq 0) { $cleanExit = Wait-For 'exit=clean' 2 }
} else {
    $proc.Kill()
    $notes = ($notes, 'process did not exit within 10 s of CloseMainWindow' | Where-Object { $_ }) -join '; '
}

$pass = $windowShown -and $firstFrame -and $responding -and (-not $consoleWindow) -and $cleanExit -and ($navigated -ne $false)
$os = Get-CimInstance Win32_OperatingSystem
$result = [ordered]@{
    release_file   = Split-Path $Exe -Leaf
    target         = [ordered]@{ os = 'windows'; os_version = "$($os.Caption) $($os.Version)"; arch = 'x86_64' }
    layer          = 'automated'
    window_shown   = $windowShown
    first_frame    = $firstFrame
    navigated      = $navigated
    responding     = $responding
    console_window = $consoleWindow
    clean_exit     = $cleanExit
    exit_code      = $exitCode
    result         = if ($pass) { 'pass' } else { 'fail' }
    performed_by   = if ($env:GITHUB_ACTIONS) { 'ci' } else { 'local' }
    recorded_at    = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ')
    notes          = $notes
}
$json = $result | ConvertTo-Json -Depth 4
Set-Content -Path $Record -Value $json -Encoding utf8
Write-Output $json
if (-not $pass) {
    Write-Output '---- application stderr (last 40 lines)'
    Get-Content $appOut -Tail 40 -ErrorAction SilentlyContinue
    Write-Output '---- diagnostic record lines from this run'
    (New-LogText) -split "`n" | Select-Object -Last 40
    exit 1
}
