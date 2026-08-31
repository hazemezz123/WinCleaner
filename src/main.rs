mod app;
mod scanner;
mod ui;

use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use app::{App, Screen};

#[derive(Parser, Debug)]
#[command(name = "cleansweep", version = "0.1.0", about = "Modern disk cleaner for Windows")]
struct Args {
    /// Run scan immediately on start
    #[arg(long)]
    scan: bool,

    /// Dry run — don't actually delete files
    #[arg(long)]
    dry_run: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Headless dry-run for CI / verification: no TUI
    if args.dry_run {
        println!("CleanSweep v0.1.0 — dry run scan (no files will be deleted)");
        println!("Scanning safe categories (TEMP, Browser Cache, Windows Temp, Logs, Recycle, Update)…\n");
        let cats = scanner::get_categories();
        let report = scanner::engine::scan_all_sync(cats);
        println!("────────────────────────────────────────");
        println!(" Total recoverable: {} in {} files ({:.2}s)", scanner::format_bytes(report.total_size), report.total_files, report.duration.as_secs_f64());
        println!("────────────────────────────────────────");
        for r in &report.results {
            let size = scanner::format_bytes_precise(r.total_size);
            println!("  {} {:<22} {:>9}  ({} files)  · {}", r.category.icon, r.category.label, size, r.files.len(), r.category.description);
        }
        println!("\nDry run complete — no files were removed.");
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    if args.scan {
        app.start_scan();
    }

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui::screens::draw(f, &app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.screen {
                        Screen::Dashboard => app.handle_key_dashboard(key.code, key.modifiers),
                        Screen::Scanning => app.handle_key_scanning(key.code, key.modifiers),
                        Screen::Review => app.handle_key_review(key.code, key.modifiers),
                        Screen::Cleaning => app.handle_key_cleaning(key.code, key.modifiers),
                        Screen::Results => app.handle_key_results(key.code, key.modifiers),
                    }
                    // Global quit
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                        app.should_quit = true;
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = std::time::Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print summary if cleaned
    if let Some(res) = app.clean_result {
        println!(
            "CleanSweep: {} recovered, {} removed, {} skipped",
            scanner::format_bytes(res.freed),
            res.removed,
            res.skipped
        );
    }

    Ok(())
}
