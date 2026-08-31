# Simple double-click launcher — no download, just run local exe
$ErrorActionPreference = "SilentlyContinue"
$exe = Join-Path $PSScriptRoot "target\release\cleansweep.exe"
if (!(Test-Path $exe)) { $exe = Join-Path $PSScriptRoot "target\debug\cleansweep.exe" }
if (!(Test-Path $exe)) { $exe = Join-Path $PSScriptRoot "cleansweep.exe" }
if (Test-Path $exe) {
    Write-Host "Launching CleanSweep..." -ForegroundColor Cyan
    & $exe
} else {
    Write-Host "Building CleanSweep (first run)..." -ForegroundColor Yellow
    cargo build --release
    if (Test-Path "target\release\cleansweep.exe") {
        & "target\release\cleansweep.exe"
    } else {
        Write-Host "Build failed. Install Rust from https://rustup.rs" -ForegroundColor Red
        pause
    }
}
