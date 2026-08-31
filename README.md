<p align="center">
  <img src="Cleaner-logo.png" alt="WinCleaner Logo" width="220" />
</p>

# ◈ WinCleaner — CleanSweep

> **A modern, lightweight disk-cleaning app that happens to live in the terminal.**
> Built in **Rust** — single 0.8 MB exe, no installer, no Python/Node. Inspired by **lazygit** and **btop**.

[![Version](https://img.shields.io/badge/version-0.1.0-7c8cff)](https://github.com/hazemezz123/WinCleaner)
[![Rust](https://img.shields.io/badge/rust-1.77+-orange)](https://www.rust-lang.org)
[![Windows](https://img.shields.io/badge/platform-Windows-0078D6)](https://github.com/hazemezz123/WinCleaner)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

---

## 🚀 Quick Start — PowerShell (1 line)

**No admin. No install. Just paste in PowerShell and press Enter:**

```powershell
irm https://raw.githubusercontent.com/hazemezz123/WinCleaner/master/install.ps1 | iex
```

That’s it. You’ll see `Loading CleanSweep…` → TUI opens.

**Other PowerShell ways to start:**

```powershell
# 1) From this repo (local) — double-click friendly
powershell -ExecutionPolicy Bypass -File install.ps1

# 2) Alternative launcher (auto-builds if needed)
powershell -ExecutionPolicy Bypass -File run.ps1

# 3) After downloading exe from Releases
.\cleansweep.exe
.\cleansweep.exe --scan      # start with scan
.\cleansweep.exe --dry-run   # headless preview — no delete

# 4) Manual download
Invoke-WebRequest -Uri https://github.com/hazemezz123/WinCleaner/releases/latest/download/cleansweep.exe -OutFile cleansweep.exe
.\cleansweep.exe
```

> **Tip:** If execution policy blocks you: `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned` then re-run.

---

## ✨ What it does

`Scan → Review → Clean → Results` — you see exactly what will be removed before anything is deleted.

| Icon | Category | Path | Example |
|------|----------|------|---------|
| ◆ | Temporary Files | `%TEMP%`, `AppData\Local\Temp` | `*.tmp`, `~*` (>24h) |
| ◎ | Browser Cache | Chrome / Edge / Firefox | `Cache`, `Code Cache` |
| ◈ | Windows Temp | `C:\Windows\Temp` | >24h |
| ◐ | Logs & Cache | `Logs`, `INetCache`, `Prefetch` | `*.log`, `Thumbs.db` |
| ♻ | Recycle Bin | `C:\$Recycle.Bin` | — |
| ▣ | Update Cache | `SoftwareDistribution\Download` | >7d |

**Safety:** whitelist-only paths, skips `in-use`/`recent` files, preview + `y/N` confirm. Never touches `System32`, `Program Files`, or Documents. `src/scanner/cleaner.rs:101`

---

## 📦 Install & Build

**Requires:** Windows 10/11, PowerShell 5.1+

**Option A — One-liner (recommended):**
```powershell
irm https://raw.githubusercontent.com/hazemezz123/WinCleaner/master/install.ps1 | iex
```

**Option B — Build from source:**
```powershell
git clone https://github.com/hazemezz123/WinCleaner.git
cd WinCleaner
cargo build --release   # → target/release/cleansweep.exe (0.8 MB)
cargo run               # run TUI
```

---

## 🛡️ Safety

- Whitelist only, age gate (`>24h` temp, `>7d` logs)
- Skips locked files → `skipped (in use)`
- Explicit confirm `Remove 3.9 GB in 4 categories? [y/N]`
- No admin elevation
- Dry-run to verify: `cleansweep.exe --dry-run`

---

## 📂 Project Structure

```
WinCleaner/
├── Cargo.toml
├── install.ps1          # 1-line loader
├── run.ps1              # local launcher
└── src/
    ├── main.rs
    ├── app.rs           # Dashboard → Scan → Review → Clean → Results
    ├── scanner/         # categories, engine, cleaner, disk
    └── ui/              # theme + screens (ratatui)
```

---

## 🔧 Tech

Rust + ratatui + crossterm — 0.8 MB static exe, `walkdir` + `sysinfo` fast scan.

---

## 📝 License

MIT — use at your own risk. Test with `--dry-run` first.

**Made for Windows. Press `S` to start.**
