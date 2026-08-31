# CleanSweep — Design Spec (Rust + PowerShell Loader)

**Date:** 2026-08-31  
**Version:** 0.1.0 MVP  
**Stack:** Rust 1.77+ / ratatui 0.28 + crossterm 0.27 / clap / walkdir / rayon / sysinfo  
**Distribution:** Single `cleansweep.exe` + `install.ps1` one-liner (`irm ... | iex`)

## 1. Overview & Goals

CleanSweep is a modern, lightweight PC deep-cleaning TUI for Windows. Focus: simplicity, safety, premium feel — "a disk-cleaning app that happens to live in the terminal."

**Core flow:** Scan → Review → Clean → Results

User starts scan, sees recoverable space per category, selects categories, confirms, sees summary (freed, removed, skipped).

**Goals:**
- Premium TUI (lazygit/btop inspiration): dark theme, good spacing, minimal borders, Unicode icons, progress indicators, keyboard-first
- Safe-by-default: whitelist-only paths, skip in-use/recent files, explicit confirm before delete
- One-line PowerShell launch: `irm https://.../install.ps1 | iex` → loading → TUI
- Single static exe, instant startup, no Python/Node runtime

**Non-goals (MVP):**
- Registry cleaning, duplicate finder, uninstaller, startup manager
- Background service / scheduler
- Admin-required deep system cleaning

## 2. Architecture

```
cleansweep/
  Cargo.toml
  src/
    main.rs              # clap args, init terminal, run App
    app.rs               # App state, Screen enum, event loop
    scanner/
      mod.rs
      categories.rs      # Category definitions + path resolution
      engine.rs          # rayon scan, progress channel, ScanReport
      cleaner.rs         # delete files, CleanResult
      disk.rs            # sysinfo disk usage
    ui/
      mod.rs
      theme.rs           # Color palette, Styles
      widgets.rs         # CategoryCard, Gauge, Spinner
      screens/
        dashboard.rs
        scanning.rs
        review.rs
        cleaning.rs
        results.rs
  install.ps1            # PowerShell loader (download + run exe)
  CleanSweep.ps1         # Optional pure-PS fallback (future)
  README.md
```

**Data flow:**
ScanJob (mpsc) → per-category parallel walk (rayon) → CategoryResult → aggregated ScanReport (Arc<Mutex>) → Review screen (selection) → CleanJob → CleanResult → Results screen. TUI stays responsive via crossterm event polling + channel.

**Modules are isolated:**
- `categories.rs`: pure data + path logic, testable without FS
- `engine.rs`: FS walk, no UI
- `cleaner.rs`: deletion logic, no UI
- `app.rs`: state machine, no FS
- `ui/*`: rendering only

## 3. Data Model

```rust
struct Category { id: String, label: String, icon: char, paths: Vec<PathBuf>, patterns: Vec<String>, min_age: Duration }
struct FileItem { path: PathBuf, size: u64, category: String, age: Duration }
struct CategoryResult { category: Category, files: Vec<FileItem>, total_size: u64 }
struct ScanReport { results: Vec<CategoryResult>, total_size: u64, total_files: usize, duration: Duration }
struct CleanResult { removed: usize, skipped: usize, freed: u64, errors: Vec<(PathBuf, String)> }
enum Screen { Dashboard, Scanning, Review, Cleaning, Results }
struct App { screen: Screen, report: Option<ScanReport>, selected: HashSet<String>, disk: DiskInfo, last_scan: Option<DateTime> }
```

## 4. Scan Categories (safe-only, real Windows paths)

| Category | Icon | Paths | Patterns | Safety |
|---|---|---|---|---|
| Temporary Files | ◆ | %TEMP%, %LOCALAPPDATA%\Temp | *.tmp, ~*, *.temp | age >24h, skip if locked |
| Browser Cache | ◎ | Edge/Chrome/Firefox Cache | Cache/*, Code Cache/* | all files |
| Windows Temp | ◈ | C:\Windows\Temp | * | skip if in-use |
| Logs & Other Cache | ◐ | *.log in temp, INetCache, ThumbnailCache | *.log, Thumbs.db, IconCache.db | age >7d for logs |
| Recycle Bin | ♻ | C:\$Recycle.Bin | * | via metadata, empty via API |
| Delivery Opt / Update | ▣ | SoftwareDistribution\Download | * | expired only |

Never touches: System32, Program Files, Users\Documents, etc. Skip if modified <24h (temp) or <7d (logs).

## 5. TUI Screens & Interaction

**Theme:** bg #0f1117, card #1a1d27, border #2a2e3f, accent #7c8cff, success #3dd68c, warning #f5a524, muted #8b8fa3. Unicode: ◈ ◆ ◎ ◐ ♻ ▣ ✓ ✗ ▓ ◐.

- **Dashboard:** header `◈ CleanSweep v0.1.0`, disk gauge `▓▓▓░░ 62% used · 128 GB free`, last scan pill, recoverable pill, `[s] Quick Scan  [q] Quit`. `s` → Scanning.
- **Scanning:** per-category rows with spinner `◐ Temp Files ─ 1.2 GB · 3.4k files`, overall progress bar, `Esc` cancel.
- **Review:** checkbox list `[✓] ◆ Temporary Files  2.4 GB (4,201)` — `Space` toggle, `a` all/none, `Enter` → confirm modal. Footer: `→ 3.9 GB to recover`.
- **Confirm modal:** `Remove 3.9 GB in 4 categories? [y/N]` — safety note.
- **Cleaning:** progress bar + live `✓ 1.2k removed  ✗ 3 skipped (in use)`.
- **Results:** hero `✓ 3.7 GB recovered · 5,203 removed · 12 skipped`, breakdown table, `[r] Rescan [q] Quit`.

Keyboard-first: `↑↓` navigate, `Space` toggle, `Enter` confirm, `Esc` back, `q` quit. No mouse required.

## 6. Safety & Error Handling

- Whitelist-only deletion; no recursive delete outside category paths
- Skip locked/in-use (OpenOptions fails), skip recent files, skip if path escapes whitelist
- Preview before delete; explicit `y` confirm
- Per-file try/catch; collect skipped with reason; never panic on FS error
- No admin required; skip protected paths silently
- Log to `%LOCALAPPDATA%\CleanSweep\clean.log`

## 7. Distribution — PowerShell Loader

**`install.ps1`:**
```powershell
# irm https://github.com/you/cleansweep/releases/latest/download/install.ps1 | iex
$exe = "$env:TEMP\cleansweep.exe"
Write-Host "⟳ Loading CleanSweep..." -ForegroundColor DarkGray
if (!(Test-Path $exe)) { Invoke-WebRequest -Uri "https://.../cleansweep.exe" -OutFile $exe }
& $exe
```
Supports `-NoDownload` (use local exe), `-Version` flag, checksum verify (future). Also provide `run.ps1` for double-click.

**Build:** `cargo build --release` → `target/release/cleansweep.exe` (~6 MB). `cargo install` not required for end user.

## 8. Testing

- Unit: category path resolution (mock env), size formatting, whitelist checks, cleaner skip logic
- Integration: scan on temp fixture dir
- UI: ratatui TestBackend snapshots for each screen
- Manual: real scan on Windows 10/11

## 9. File Structure (MVP)

See §2. `Cargo.toml` deps: ratatui, crossterm, clap (derive), walkdir, rayon, sysinfo, chrono, serde (for config future).

## 10. Future Expansion

- Config file for custom categories
- Scheduler, duplicate finder
- Pure `CleanSweep.ps1` fallback

---

**Approved:** 2026-08-31 — Rust + loader selected for easy one-line PowerShell UX.
