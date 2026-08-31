<#
.SYNOPSIS
    CleanSweep - One-line loader for Windows
.DESCRIPTION
    Downloads (if needed) and launches the CleanSweep TUI.
    Usage:
        irm https://raw.githubusercontent.com/you/cleansweep/main/install.ps1 | iex
        # or locally:
        powershell -ExecutionPolicy Bypass -File install.ps1
.NOTES
    No admin required. Only touches safe temp/cache paths.
#>
param(
    [string]$Version = "latest",
    [string]$BinDir = "$env:LOCALAPPDATA\CleanSweep\bin",
    [switch]$NoDownload,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    Write-Host @"
CleanSweep v0.1.0 - Modern disk cleaner for Windows

Usage:
  install.ps1              # download if needed + launch
  install.ps1 -Version v0.1.0
  install.ps1 -NoDownload  # run local build only
  irm https://.../install.ps1 | iex   # one-liner

Keys in TUI:
  s = Scan, Space = Toggle, a = All, Enter = Clean, y/n = Confirm, q = Quit
"@
    exit 0
}

$Repo = "you/cleansweep"
$ExeName = "cleansweep.exe"
$LocalExe = Join-Path $BinDir $ExeName
$RootExe  = Join-Path $PSScriptRoot $ExeName
$TempExe  = Join-Path $env:TEMP $ExeName

function Write-Pretty($msg, $color = "DarkGray") {
    Write-Host $msg -ForegroundColor $color
}

function Show-Loading($text) {
    $spin = @('|','/','-','\')
    for ($i=0; $i -lt 8; $i++) {
        Write-Host -NoNewline ("`r{0} {1} " -f $spin[$i % 4], $text) -ForegroundColor Cyan
        Start-Sleep -Milliseconds 80
    }
    Write-Host "`r   " -NoNewline
    Write-Host "`r" -NoNewline
}

# Resolve exe: prefer BinDir > Root > Temp > local cargo build
$Exe = $null
$candidates = @($LocalExe, $RootExe, $TempExe, ".\target\release\$ExeName", ".\target\debug\$ExeName")
foreach ($c in $candidates) {
    if (Test-Path -LiteralPath $c) { $Exe = (Resolve-Path $c).Path; break }
}

Show-Loading "Loading CleanSweep..."

if (-not $Exe -and -not $NoDownload) {
    Write-Pretty "  -> No local binary found, preparing download..." "Gray"
    $null = New-Item -ItemType Directory -Force -Path $BinDir

    if ($Version -eq "latest") {
        $Url = "https://github.com/$Repo/releases/latest/download/$ExeName"
    } else {
        $Url = "https://github.com/$Repo/releases/download/$Version/$ExeName"
    }

    try {
        Write-Pretty "  Downloading $ExeName ($Version)..." "Cyan"
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        try {
            Invoke-WebRequest -Uri $Url -OutFile $LocalExe -UseBasicParsing -TimeoutSec 30
        } catch {
            (New-Object Net.WebClient).DownloadFile($Url, $LocalExe)
        }
        if (Test-Path $LocalExe) {
            Write-Pretty "  Downloaded to $LocalExe" "Green"
            $Exe = $LocalExe
        }
    } catch {
        Write-Pretty "  Download failed: $($_.Exception.Message)" "Red"
        Write-Pretty "  Hint: build locally with 'cargo build --release' then re-run install.ps1" "Yellow"
    }
}

if (-not $Exe) {
    Write-Pretty "" 
    Write-Pretty "  CleanSweep binary not found." "Red"
    Write-Pretty "  Options:" "Yellow"
    Write-Pretty "    1) cargo build --release  (from this repo)" "Gray"
    Write-Pretty "    2) Place cleansweep.exe next to install.ps1" "Gray"
    Write-Pretty "    3) Re-run with download: install.ps1" "Gray"
    Write-Host ""
    if (Test-Path "Cargo.toml") {
        Write-Pretty "  Trying cargo run..." "Cyan"
        cargo run --quiet
        exit $LASTEXITCODE
    }
    exit 1
}

Write-Pretty "  Launching CleanSweep" "Green"
Write-Pretty "    $Exe" "DarkGray"
Write-Pretty "  -----------------------------------------" "DarkGray"

try { Unblock-File -Path $Exe -ErrorAction SilentlyContinue } catch {}

& $Exe @args
$code = $LASTEXITCODE

Write-Pretty ""
Write-Pretty "  -----------------------------------------" "DarkGray"
if ($code -eq 0) {
    Write-Pretty "  CleanSweep exited cleanly. Run again with: install.ps1" "DarkGray"
} else {
    Write-Pretty "  Exit code: $code" "Yellow"
}
exit $code
