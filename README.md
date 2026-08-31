# ◈ CleanSweep — Modern Disk Cleaner for Windows

> A modern, lightweight disk-cleaning app that happens to live in the terminal.
> Inspired by **lazygit**, **btop**, and contemporary developer tools.

![Version](https://img.shields.io/badge/version-0.1.0-7c8cff)
![Rust](https://img.shields.io/badge/rust-1.77+-orange)
![Windows](https://img.shields.io/badge/platform-Windows-blue)
![License](https://img.shields.io/badge/license-MIT-green)

---

## ✨ What it does

Scans your PC for safe-to-remove temporary and cache files, shows you exactly what will be recovered, and cleans with one confirmation.

**Flow:** `Scan → Review → Clean → Results`

**Categories (safe-only):**

| Icon | Category | Example Path | Size |
|------|----------|-------------|------|
| ◆ | Temporary Files | `%TEMP%`, `AppData\Local\Temp` | 2.4 GB |
| ◎ | Browser Cache | Chrome/Edge/Firefox `Cache` | 1.1 GB |
| ◈ | Windows Temp | `C:\Windows\Temp` | 340 MB |
| ◐ | Logs & Other Cache | `*.log`, `Thumbs.db` | 180 MB |
| ♻ | Recycle Bin | `C:\$Recycle.Bin` | 1.36 GB |
| ▣ | Update Cache | `SoftwareDistribution\Download` | — |

> **Safety first:** whitelist-only, skips in-use/recent files, preview before delete, explicit `y/N` confirm. Never touches `System32`, `Program Files`, or your documents.

---

## 🚀 One-line PowerShell launch (recommended)

No Python, no Node, no admin. Just a loading spinner → TUI.

```powershell
irm https://raw.githubusercontent.com/you/cleansweep/main/install.ps1 | iex
```

**What it does:**
1. Shows `⟳ Loading CleanSweep…`
2. Downloads `cleansweep.exe` (~0.8 MB) to `%LOCALAPPDATA%\CleanSweep\bin` if needed
3. Launches the TUI instantly

**Local (double-click) alternative:**

```powershell
# From repo root — builds if needed, then runs:
powershell -ExecutionPolicy Bypass -File install.ps1
# or
powershell -ExecutionPolicy Bypass -File run.ps1
```

---

## 🖥️ TUI Preview

```
 ◈ CleanSweep  v0.1.0  ·  C:\  128 GB free
────────────────────────────────────────────────
 ● Disk Usage
 ▓▓▓▓▓▓▓▓▓░░░░  62% used
 512 GB total  ·  384 GB used  ·  128 GB free

 [Last scan: Never]  [Recoverable: —]  [Status: Idle]

          [s]  Quick Scan — scan for safe-to-remove files
  Press 's' to start. Your files are safe — preview before cleaning.
────────────────────────────────────────────────
  s Scan   ·   q Quit
```

**Scan → Review → Clean → Results:**

- **Dashboard:** disk gauge, last scan, recoverable pill, `[s] Quick Scan`
- **Scanning:** per-category spinner `◐ Temp Files ─ 1.2 GB · 3.4k files` + overall gauge
- **Review:** `[✓] ◆ Temporary Files  2.4 GB (4,201)` — `Space` toggle, `a` all, `Enter` to clean → confirm modal
- **Cleaning:** `⟳ 1.2k removed · 3 skipped (in use)` + progress bar
- **Results:** hero `✓ 3.7 GB recovered · 5,203 removed · 12 skipped` + breakdown

**Keyboard:** `↑↓`/`j/k` navigate, `Space` toggle, `a` all, `Enter` clean, `y`/`n` confirm, `Esc` back, `q` quit, `Ctrl+C` force quit.

Theme: dark `#0f1117`, card `#1a1d27`, border `#2a2e3f`, accent `#7c8cff`, success `#3dd68c`, Unicode icons.

---

## 📦 Install & Build

### Option A: Download binary (fastest)
```powershell
# One-liner (above) or manually:
Invoke-WebRequest -Uri https://github.com/you/cleansweep/releases/latest/download/cleansweep.exe -OutFile cleansweep.exe
.\cleansweep.exe
```

### Option B: Build from source
```powershell
# Prereq: Rust via rustup.rs
cargo build --release   # → target\release\cleansweep.exe (0.8 MB, instant startup)
cargo run -- --dry-run  # headless scan for CI
.\target\release\cleansweep.exe --scan  # start with scan
```

### Dry-run (no TUI, for CI/verification)
```powershell
.\cleansweep.exe --dry-run
# Output:
# Total recoverable: 2.1 GB in 6114 files (0.54s)
#   ◆ Temporary Files          17.8 MB  (324 files)
#   ◎ Browser Cache           514.5 MB  (4877 files)
#   ...
```

---

## 🛡️ Safety

- **Whitelist-only:** only `%TEMP%`, `Windows\Temp`, browser caches, logs, Recycle Bin, Update cache
- **Age gate:** temp >24h, logs >7d (avoids deleting active files)
- **In-use skip:** locked files are skipped with reason `in use`
- **Preview + confirm:** shows `→ 3.9 GB to recover` and asks `Remove 3.9 GB in 4 categories? [y/N]`
- **No admin:** skips protected paths, never elevates
- **Log:** `%LOCALAPPDATA%\CleanSweep\clean.log` (future)

---

## 📂 Project Structure

```
cleansweep/
├── Cargo.toml
├── install.ps1              # One-line PowerShell loader (download + run)
├── run.ps1                  # Double-click local launcher
├── src/
│   ├── main.rs              # Terminal setup, app loop, --dry-run
│   ├── app.rs               # State machine: Dashboard→Scan→Review→Clean→Results
│   ├── scanner/
│   │   ├── categories.rs    # Whitelisted paths & patterns
│   │   ├── engine.rs        # Parallel walk, ScanReport
│   │   ├── cleaner.rs       # Safe delete, CleanResult
│   │   └── disk.rs          # sysinfo disk + format_bytes
│   └── ui/
│       ├── theme.rs         # Premium dark palette
│       └── screens/mod.rs   # All screen renderers
└── docs/superpowers/specs/  # Design doc
```

---

## 🔧 Tech

- **Rust + ratatui + crossterm** — btop/lazygit-grade TUI, single ~0.8 MB exe, no runtime
- **walkdir + rayon** — fast parallel scanning (0.5s for 6k files)
- **sysinfo + chrono + clap** — disk, time, args
- **PowerShell loader** — `irm | iex` pattern familiar to Windows users

---

## 📝 License

MIT — do what you want, no warranty. Be careful, test on your machine first.

---

**Made for Windows.** *Scan → Review → Clean → Results.* Press `s` to start.
