use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, DashboardAction, Screen};
use crate::scanner::{format_bytes, format_bytes_precise};
use crate::ui::theme::Theme;

// ── Entry ──────────────────────────────────────────────────────────────

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Terminal too small?
    if area.width < 60 || area.height < 20 {
        draw_too_small(frame, area);
        return;
    }

    // Global bg
    frame.render_widget(Block::default().style(Style::default().bg(Theme::bg())), area);

    // Layout: header (2) + separator (1) + body + footer (2 incl separator)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header
            Constraint::Length(1), // header rule
            Constraint::Min(0),    // body
            Constraint::Length(1), // footer rule
            Constraint::Length(1), // footer
        ])
        .split(area);

    draw_header(frame, chunks[0], app);
    draw_header_rule(frame, chunks[1]);
    match app.screen {
        Screen::Dashboard => draw_dashboard(frame, chunks[2], app),
        Screen::Scanning => draw_scanning(frame, chunks[2], app),
        Screen::Review => draw_review(frame, chunks[2], app),
        Screen::Cleaning => draw_cleaning(frame, chunks[2], app),
        Screen::Results => draw_results(frame, chunks[2], app),
    }
    draw_footer_rule(frame, chunks[3]);
    draw_footer(frame, chunks[4], app);

    if app.show_help {
        draw_help_modal(frame, area);
    } else if app.show_confirm {
        draw_confirm_modal(frame, area, app);
    } else if app.show_details {
        draw_details_modal(frame, area, app);
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

fn is_wide(area: Rect) -> bool {
    area.width >= 100
}

fn draw_too_small(frame: &mut Frame, area: Rect) {
    frame.render_widget(Block::default().style(Style::default().bg(Theme::bg())), area);
    let msg = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(" Terminal too small ", Style::default().fg(Theme::warning()).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("{} × {}  —  need at least 60 × 20", area.width, area.height), Style::default().fg(Theme::muted()))),
        Line::from(Span::styled("Resize the window to continue.", Style::default().fg(Theme::dim()))),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(msg, area);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    // ◆ CleanSweep   v0.1.0   ·   C:\  128 GB free   (subtle right side)
    //   Modern disk cleaner for Windows
    let right_info = format!("{}  {} free", app.disk.mount, format_bytes(app.disk.available));
    let title = Line::from(vec![
        Span::styled("  ◆ ", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)),
        Span::styled("CleanSweep", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
        Span::styled("  v0.1.0  ", Style::default().fg(Theme::muted())),
        Span::styled(right_info, Style::default().fg(Theme::dim())),
    ]);
    let subtitle = Line::from(Span::styled("    Modern disk cleaner for Windows", Style::default().fg(Theme::dim())));
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    frame.render_widget(Paragraph::new(title), inner[0]);
    frame.render_widget(Paragraph::new(subtitle), inner[1]);
}

fn draw_header_rule(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(line, Style::default().fg(Theme::border())))),
        area,
    );
}

fn draw_footer_rule(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(line, Style::default().fg(Theme::border())))),
        area,
    );
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<(&str, &str)> = match app.screen {
        Screen::Dashboard => {
            let mut v = vec![("↑↓←→", "Navigate"), ("Enter", "Select")];
            // contextual scan/clean
            let acts = app.dashboard_actions();
            for a in &acts {
                match a {
                    DashboardAction::Scan => v.push(("S", "Scan")),
                    DashboardAction::Clean => v.push(("C", "Clean")),
                    DashboardAction::Rescan => v.push(("R", "Rescan")),
                    DashboardAction::Help => v.push(("?", "Help")),
                }
            }
            v.push(("Q", "Quit"));
            v
        }
        Screen::Scanning => vec![("Esc", "Cancel"), ("Q", "Quit")],
        Screen::Review => {
            if app.show_confirm {
                vec![("Y", "Confirm"), ("N", "Cancel"), ("Enter", "Clean")]
            } else if app.show_details {
                vec![("Esc", "Close"), ("Q", "Quit")]
            } else {
                vec![
                    ("↑↓", "Navigate"),
                    ("Space", "Toggle"),
                    ("Enter", "Details"),
                    ("C", "Clean"),
                    ("A", "All"),
                    ("Esc", "Back"),
                ]
            }
        }
        Screen::Cleaning => vec![("Q", "Quit")],
        Screen::Results => vec![("R", "Rescan"), ("S", "Scan"), ("Q", "Quit"), ("Esc", "Dashboard")],
    };

    let mut spans = Vec::new();
    for (i, (k, v)) in items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default().fg(Theme::dim())));
        }
        // key pill subtle
        spans.push(Span::styled(format!(" {} ", k), Style::default().fg(Theme::muted()).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(*v, Style::default().fg(Theme::dim())));
    }
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

// ── Dashboard ──────────────────────────────────────────────────────────

fn draw_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let wide = is_wide(area);

    if wide {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::SpaceBetween)
            .constraints([
                Constraint::Length(7), // top row storage + recoverable
                Constraint::Length(1), // status
                Constraint::Length(7), // primary scan
                Constraint::Length(1), // quick actions
                Constraint::Length(1), // tip
            ])
            .split(area);
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(outer[0]);
        draw_storage(frame, top[0], app);
        draw_recoverable(frame, top[1], app);
        draw_system_status(frame, outer[1], app);
        draw_primary_scan(frame, outer[2], app);
        draw_quick_actions(frame, outer[3], app);
        let tip = Paragraph::new(Line::from(Span::styled(
            "  Tip: Everything stays on this PC — preview before you clean.  Press ? for help.",
            Style::default().fg(Theme::dim()),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(tip, outer[4]);
    } else {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .flex(Flex::SpaceBetween)
            .constraints([
                Constraint::Length(6), // storage
                Constraint::Length(6), // recoverable
                Constraint::Length(1), // status
                Constraint::Length(5), // primary
                Constraint::Length(1), // quick actions
                Constraint::Length(1), // tip
            ])
            .split(area);
        draw_storage(frame, outer[0], app);
        draw_recoverable(frame, outer[1], app);
        draw_system_status(frame, outer[2], app);
        draw_primary_scan(frame, outer[3], app);
        draw_quick_actions(frame, outer[4], app);
        let tip = Paragraph::new(Line::from(Span::styled(
            "  Preview before you clean — press ? for help.",
            Style::default().fg(Theme::dim()),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(tip, outer[5]);
    }
}

fn draw_storage(frame: &mut Frame, area: Rect, app: &App) {
    // Center 5-line content vertically within area
    let content_h: u16 = 5;
    let start_y = area.y + area.height.saturating_sub(content_h) / 2;
    let title = Line::from(Span::styled("  STORAGE", Style::default().fg(Theme::muted()).add_modifier(Modifier::BOLD)));
    frame.render_widget(Paragraph::new(title), Rect { x: area.x, y: start_y, width: area.width, height: 1 });

    let percent = app.disk.percent_used.clamp(0.0, 100.0) as u16;
    let drive_line = Line::from(vec![
        Span::styled(format!("  {}  ", app.disk.mount), Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}% Used", percent), Style::default().fg(if percent > 85 { Theme::warning() } else { Theme::muted() })),
        Span::styled("  ·  NTFS  ·  Healthy", Style::default().fg(Theme::dim())),
    ]);
    frame.render_widget(Paragraph::new(drive_line), Rect { x: area.x, y: start_y + 1, width: area.width, height: 1 });

    let bar_width = (area.width.saturating_sub(4)) as usize;
    let filled = ((percent as usize * bar_width) / 100).min(bar_width);
    let empty = bar_width.saturating_sub(filled);
    let bar = format!("  {}{}", "█".repeat(filled), "░".repeat(empty));
    let bar_style = Style::default().fg(if percent > 85 { Theme::warning() } else { Theme::accent() });
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(bar, bar_style))),
        Rect { x: area.x, y: start_y + 2, width: area.width, height: 1 },
    );

    let trio = Line::from(vec![
        Span::styled(format!("  {} used", format_bytes(app.disk.used)), Style::default().fg(Theme::text())),
        Span::styled("   ", Style::default().fg(Theme::dim())),
        Span::styled(format!("{} free", format_bytes(app.disk.available)), Style::default().fg(Theme::success()).add_modifier(Modifier::BOLD)),
        Span::styled("   ", Style::default().fg(Theme::dim())),
        Span::styled(format!("{} total", format_bytes(app.disk.total)), Style::default().fg(Theme::muted())),
    ]);
    frame.render_widget(Paragraph::new(trio), Rect { x: area.x, y: start_y + 3, width: area.width, height: 1 });

    if content_h >= 5 {
        if let Some(report) = &app.scan_report {
            let est_free = app.disk.available.saturating_add(report.total_size);
            let line = Line::from(vec![
                Span::styled("  After clean: ", Style::default().fg(Theme::dim())),
                Span::styled(format!("{} free", format_bytes(est_free)), Style::default().fg(Theme::success())),
                Span::styled(format!("  (+{})", format_bytes(report.total_size)), Style::default().fg(Theme::accent())),
            ]);
            frame.render_widget(Paragraph::new(line), Rect { x: area.x, y: start_y + 4, width: area.width, height: 1 });
        } else {
            let line = Line::from(Span::styled("  Run a scan to see recoverable space", Style::default().fg(Theme::dim())));
            frame.render_widget(Paragraph::new(line), Rect { x: area.x, y: start_y + 4, width: area.width, height: 1 });
        }
    }
}

fn draw_recoverable(frame: &mut Frame, area: Rect, app: &App) {
    let has_scan = app.scan_report.is_some();
    let content_h: u16 = 5;
    let start_y = area.y + area.height.saturating_sub(content_h) / 2;
    let title = Line::from(Span::styled("  POTENTIALLY RECOVERABLE", Style::default().fg(Theme::muted()).add_modifier(Modifier::BOLD)));
    frame.render_widget(Paragraph::new(title), Rect { x: area.x, y: start_y, width: area.width, height: 1 });

    if !has_scan {
        let empty = vec![
            Line::from(Span::styled("  No scan performed yet", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("  Start a scan to find unnecessary files.", Style::default().fg(Theme::dim()))),
            Line::from(Span::styled("  Safe — only temp, cache and logs.", Style::default().fg(Theme::dim()))),
        ];
        frame.render_widget(
            Paragraph::new(empty).wrap(Wrap { trim: false }),
            Rect { x: area.x, y: start_y + 1, width: area.width, height: 3 },
        );
    } else {
        let report = app.scan_report.as_ref().unwrap();
        let size_str = format_bytes_precise(report.total_size);
        let count = report.results.len();
        let total_files: usize = report.results.iter().map(|r| r.files.len()).sum();

        let big = Line::from(Span::styled(format!("  {}", size_str), Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)));
        frame.render_widget(Paragraph::new(big), Rect { x: area.x, y: start_y + 1, width: area.width, height: 1 });
        let sub = Line::from(vec![
            Span::styled(format!("  {} categories", count), Style::default().fg(Theme::text())),
            Span::styled(format!("  ·  {} files", total_files), Style::default().fg(Theme::muted())),
        ]);
        frame.render_widget(Paragraph::new(sub), Rect { x: area.x, y: start_y + 2, width: area.width, height: 1 });

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled("  ", Style::default().fg(Theme::dim())));
        for (i, cat) in report.results.iter().enumerate() {
            if i > 0 { spans.push(Span::styled("  ", Style::default().fg(Theme::dim()))); }
            let is_sel = app.selected.contains(&cat.category.id);
            let col = if is_sel { Theme::success() } else { Theme::dim() };
            spans.push(Span::styled(format!("{} {}", cat.category.icon, format_bytes(cat.total_size)), Style::default().fg(col)));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false }),
            Rect { x: area.x, y: start_y + 3, width: area.width, height: 2 },
        );
    }
}

fn draw_system_status(frame: &mut Frame, area: Rect, app: &App) {
    let last = app.last_scan.as_deref().unwrap_or("never");
    let status = if app.scan_report.is_some() { "Ready to clean" } else { "Idle" };
    let status_color = if app.scan_report.is_some() { Theme::success() } else { Theme::muted() };
    let line = Line::from(vec![
        Span::styled("  Last scan: ", Style::default().fg(Theme::dim())),
        Span::styled(last, Style::default().fg(Theme::muted())),
        Span::styled("   ·   Status: ", Style::default().fg(Theme::dim())),
        Span::styled(status, Style::default().fg(status_color)),
        Span::styled("   ·   ", Style::default().fg(Theme::dim())),
        Span::styled(format!("{} files scanned", app.scan_report.as_ref().map(|r| r.total_files).unwrap_or(0)), Style::default().fg(Theme::dim())),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_primary_scan(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.dashboard_actions().get(app.dashboard_focus) == Some(&DashboardAction::Scan);
    let border_style = if focused { Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD) } else { Style::default().fg(Theme::border()) };
    let bg = if focused { Theme::card_selected() } else { Theme::card() };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(Style::default().bg(bg));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 10 || inner.height == 0 {
        return;
    }

    let prefix = if focused { "▸ " } else { "  " };
    // Responsive: if narrow or short, single line compact
    if inner.height < 3 || area.width < 80 {
        let line = Line::from(vec![
            Span::styled(prefix, Style::default().fg(if focused { Theme::accent() } else { Theme::bg() }).add_modifier(Modifier::BOLD)),
            Span::styled("[ S ] ", Style::default().bg(Theme::accent()).fg(Color::Rgb(15,17,23)).add_modifier(Modifier::BOLD)),
            Span::styled("Start Scan", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
            Span::styled(" — temp & cache", Style::default().fg(Theme::dim())),
        ]);
        let padded = Rect { x: inner.x + 1, y: inner.y + inner.height / 2, width: inner.width.saturating_sub(2), height: 1 };
        frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), padded);
    } else {
        let title = Line::from(vec![
            Span::styled(prefix, Style::default().fg(if focused { Theme::accent() } else { Theme::bg() }).add_modifier(Modifier::BOLD)),
            Span::styled("[ S ]  ", Style::default().bg(Theme::accent()).fg(Color::Rgb(15,17,23)).add_modifier(Modifier::BOLD)),
            Span::styled(" Start Scan", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
        ]);
        let subtitle = Line::from(Span::styled(
            "     Find temporary files, caches and other safe data",
            Style::default().fg(Theme::dim()),
        ));
        let padded = Rect { x: inner.x + 2, y: inner.y + 1, width: inner.width.saturating_sub(4), height: inner.height.saturating_sub(2).min(2) };
        frame.render_widget(Paragraph::new(vec![title, subtitle]).alignment(Alignment::Left), padded);
        if focused && inner.height >= 3 {
            let hint = Line::from(Span::styled("↵ Enter", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)));
            frame.render_widget(Paragraph::new(hint).alignment(Alignment::Right), Rect { x: inner.x, y: inner.y + inner.height.saturating_sub(1), width: inner.width.saturating_sub(2), height: 1 });
        }
    }
}

fn draw_quick_actions(frame: &mut Frame, area: Rect, app: &App) {
    let actions = app.dashboard_actions();
    let mut spans = Vec::new();
    for (idx, act) in actions.iter().enumerate() {
        let is_focused = idx == app.dashboard_focus;
        let key_style = if is_focused {
            Style::default().bg(Theme::accent()).fg(Color::Rgb(15,17,23)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Theme::border()).fg(Theme::text()).add_modifier(Modifier::BOLD)
        };
        let label_style = if is_focused {
            Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Theme::muted())
        };
        if idx > 0 {
            spans.push(Span::styled("   ", Style::default().fg(Theme::dim())));
        }
        if is_focused {
            spans.push(Span::styled("▸ ", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)));
        }
        spans.push(Span::styled(format!(" {} ", act.key()), key_style));
        spans.push(Span::styled(format!(" {}", act.label()), label_style));
    }
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

// ── Scanning ───────────────────────────────────────────────────────────

fn draw_scanning(frame: &mut Frame, area: Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled("  Scanning for unnecessary files", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
        Span::styled("  ·  ", Style::default().fg(Theme::dim())),
        Span::styled(format!("{} categories", app.scan_progress.len()), Style::default().fg(Theme::muted())),
    ]);
    frame.render_widget(Paragraph::new(title), Rect { x: area.x, y: area.y, width: area.width, height: 1 });
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("  Safe scan — only temp and cache locations", Style::default().fg(Theme::dim())))),
        Rect { x: area.x, y: area.y + 1, width: area.width, height: 1 },
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(Theme::border())))),
        Rect { x: area.x, y: area.y + 2, width: area.width, height: 1 },
    );

    let list_start_y = area.y + 3;
    let max_rows = (area.height.saturating_sub(6)) as usize;
    for (idx, p) in app.scan_progress.iter().enumerate().take(max_rows) {
        let y = list_start_y + idx as u16;
        let spinner = match p.spinner_frame % 4 {
            0 => "◐", 1 => "◓", 2 => "◑", 3 => "◒", _ => "◐",
        };
        let size_str = format_bytes(p.size);
        let mut spans = vec![
            Span::styled(format!("  {} ", spinner), Style::default().fg(if p.done { Theme::success() } else { Theme::accent() })),
            Span::styled(format!("{} ", p.icon), Style::default().fg(Theme::muted())),
            Span::styled(format!("{:<20}", p.label), Style::default().fg(Theme::text())),
            Span::styled(format!("{:>8}", size_str), Style::default().fg(if p.done { Theme::success() } else { Theme::accent() }).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {} files", p.files), Style::default().fg(Theme::muted())),
        ];
        if p.done {
            spans.push(Span::styled("  ✓", Style::default().fg(Theme::success())));
        } else {
            spans.push(Span::styled("  …", Style::default().fg(Theme::dim())));
        }
        let bg = if p.done { Theme::card() } else { Theme::bg() };
        frame.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)), Rect { x: area.x, y, width: area.width, height: 1 });
    }

    // Overall progress — pinned near bottom but not huge gap
    let gauge_y = (list_start_y + app.scan_progress.len() as u16 + 1).min(area.y + area.height - 2);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(Theme::border())))),
        Rect { x: area.x, y: gauge_y, width: area.width, height: 1 },
    );
    let total = app.scan_progress.len() as f64;
    let done = app.scan_progress.iter().filter(|p| p.done).count() as f64;
    let pct = if total > 0.0 { (done / total * 100.0) as u16 } else { 0 };
    let bar_w = area.width.saturating_sub(10) as usize;
    let filled = (pct as usize * bar_w / 100).min(bar_w);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_w - filled));
    let gauge_line = Line::from(vec![
        Span::styled("  ", Style::default().fg(Theme::dim())),
        Span::styled(bar, Style::default().fg(Theme::accent())),
        Span::styled(format!("  {}/{}  {}%", done as usize, total as usize, pct), Style::default().fg(Theme::muted())),
    ]);
    frame.render_widget(Paragraph::new(gauge_line), Rect { x: area.x, y: gauge_y + 1, width: area.width, height: 1 });
}

// ── Review ─────────────────────────────────────────────────────────────

fn draw_review(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.scan_report {
        Some(r) => r,
        None => {
            frame.render_widget(Paragraph::new("No scan data").alignment(Alignment::Center), area);
            return;
        }
    };
    let (sel_size, sel_count) = app.selected_total();

    // Header
    let header = Line::from(vec![
        Span::styled("  Review & Select", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  ·  {} selected", format_bytes(sel_size)), Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  ({} files)", sel_count), Style::default().fg(Theme::muted())),
    ]);
    frame.render_widget(Paragraph::new(header), Rect { x: area.x, y: area.y, width: area.width, height: 1 });
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("  Space toggle  ·  a all  ·  Enter details  ·  C clean", Style::default().fg(Theme::dim())))),
        Rect { x: area.x, y: area.y + 1, width: area.width, height: 1 },
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(Theme::border())))),
        Rect { x: area.x, y: area.y + 2, width: area.width, height: 1 },
    );

    // Split remaining area: list on top, details at bottom
    let remaining = Rect { x: area.x, y: area.y + 3, width: area.width, height: area.height.saturating_sub(5) };
    let list_height = (report.results.len() as u16 + 1).min(remaining.height.saturating_sub(4).max(6));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(list_height), Constraint::Length(1), Constraint::Min(0)])
        .split(remaining);

    // List
    for (idx, cat) in report.results.iter().enumerate() {
        if idx as u16 >= chunks[0].height { break; }
        let y = chunks[0].y + idx as u16;
        let selected = app.selected.contains(&cat.category.id);
        let is_focused = idx == app.selected_idx;
        let prefix = if is_focused { "▸ " } else { "  " };
        let checkbox = if selected { "[✓]" } else { "[ ]" };
        let size = format_bytes_precise(cat.total_size);
        let count = cat.files.len();
        let mut spans = vec![
            Span::styled(prefix, Style::default().fg(if is_focused { Theme::accent() } else { Theme::bg() }).add_modifier(Modifier::BOLD)),
            Span::styled(checkbox, Style::default().fg(if selected { Theme::success() } else { Theme::muted() }).add_modifier(Modifier::BOLD)),
            Span::styled(" ", Style::default().fg(Theme::text())),
            Span::styled(format!("{} ", cat.category.icon), Style::default().fg(Theme::accent())),
            Span::styled(format!("{:<22}", cat.category.label), Style::default().fg(Theme::text()).add_modifier(if is_focused { Modifier::BOLD } else { Modifier::empty() })),
            Span::styled(format!("{:>9}", size), Style::default().fg(if selected { Theme::accent() } else { Theme::muted() }).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  {} files", count), Style::default().fg(Theme::muted())),
        ];
        if area.width >= 90 {
            spans.push(Span::styled(format!("  ·  {}", cat.category.description), Style::default().fg(Theme::dim())));
        }
        let bg = if is_focused { Theme::card_selected() } else { Theme::bg() };
        frame.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)), Rect { x: area.x, y, width: area.width, height: 1 });
    }

    // Separator between list and details
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(Theme::border())))),
        chunks[1],
    );

    // Details for focused category — uses remaining space efficiently
    if chunks[2].height >= 3 {
        let cat = &report.results[app.selected_idx.min(report.results.len() - 1)];
        let title = Line::from(vec![
            Span::styled(format!("  {} {}  ", cat.category.icon, cat.category.label), Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{}  ·  {} files", format_bytes(cat.total_size), cat.files.len()), Style::default().fg(Theme::muted())),
            Span::styled(format!("  ·  {}", if app.selected.contains(&cat.category.id) { "selected" } else { "not selected" }), Style::default().fg(if app.selected.contains(&cat.category.id) { Theme::success() } else { Theme::dim() })),
        ]);
        frame.render_widget(Paragraph::new(title), Rect { x: chunks[2].x, y: chunks[2].y, width: chunks[2].width, height: 1 });
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!("  {}", cat.category.description), Style::default().fg(Theme::dim())))),
            Rect { x: chunks[2].x, y: chunks[2].y + 1, width: chunks[2].width, height: 1 },
        );
        // Top files
        let mut y = chunks[2].y + 2;
        let mut files = cat.files.clone();
        files.sort_by(|a,b| b.size.cmp(&a.size));
        for f in files.iter().take((chunks[2].height as usize).saturating_sub(4)) {
            if y >= chunks[2].y + chunks[2].height - 1 { break; }
            let name = f.path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
            let line = Line::from(vec![
                Span::styled(format!("    {:>8}  ", format_bytes(f.size)), Style::default().fg(Theme::accent())),
                Span::styled(name.to_string(), Style::default().fg(Theme::muted())),
            ]);
            frame.render_widget(Paragraph::new(line), Rect { x: chunks[2].x, y, width: chunks[2].width, height: 1 });
            y += 1;
        }
        // Footer summary inside details area
        let summary_y = area.y + area.height.saturating_sub(1);
        let summary = Line::from(vec![
            Span::styled("  → ", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} to recover", format_bytes(sel_size)), Style::default().fg(Theme::success()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  in {} files  ", sel_count), Style::default().fg(Theme::muted())),
            Span::styled("·  Enter details  ·  C to clean", Style::default().fg(Theme::dim())),
        ]);
        frame.render_widget(Paragraph::new(summary), Rect { x: area.x, y: summary_y, width: area.width, height: 1 });
    } else {
        // Fallback footer if no space for details
        let summary = Line::from(vec![
            Span::styled("  → ", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} to recover", format_bytes(sel_size)), Style::default().fg(Theme::success()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  in {} files  ", sel_count), Style::default().fg(Theme::muted())),
            Span::styled("·  Enter details  ·  C to clean", Style::default().fg(Theme::dim())),
        ]);
        frame.render_widget(Paragraph::new(summary), Rect { x: area.x, y: area.y + area.height - 1, width: area.width, height: 1 });
    }
}

// ── Cleaning ───────────────────────────────────────────────────────────

fn draw_cleaning(frame: &mut Frame, area: Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled("  Cleaning", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  ·  {} removed", app.clean_progress.0), Style::default().fg(Theme::success())),
        Span::styled(format!("  ·  {} skipped", app.clean_progress.1), Style::default().fg(Theme::muted())),
        Span::styled(format!("  ·  {} freed", format_bytes(app.clean_progress.2)), Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(title), Rect { x: area.x, y: area.y, width: area.width, height: 1 });
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("  Removing safe temporary files…", Style::default().fg(Theme::dim())))),
        Rect { x: area.x, y: area.y + 1, width: area.width, height: 1 },
    );

    // Progress bar
    let bar_y = area.y + 3;
    let total = app.clean_total_files;
    let done = app.clean_progress.0 + app.clean_progress.1;
    let pct = if total > 0 { (done as f64 / total as f64 * 100.0) as u16 } else { 0 };
    let bar_w = area.width.saturating_sub(6) as usize;
    let filled = (pct as usize * bar_w / 100).min(bar_w);
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_w - filled));
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  ", Style::default().fg(Theme::dim())),
            Span::styled(bar, Style::default().fg(Theme::success())),
            Span::styled(format!("  {}/{}  {}%", done, total, pct), Style::default().fg(Theme::muted())),
        ])),
        Rect { x: area.x, y: bar_y, width: area.width, height: 1 },
    );

    // Current file
    let cur = app.clean_current.as_deref().unwrap_or("—");
    // Truncate if too long
    let max_len = area.width.saturating_sub(4) as usize;
    let cur_short = if cur.len() > max_len { format!("…{}", &cur[cur.len().saturating_sub(max_len)..]) } else { cur.to_string() };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  Current: ", Style::default().fg(Theme::dim())),
            Span::styled(cur_short, Style::default().fg(Theme::muted())),
        ])),
        Rect { x: area.x, y: bar_y + 2, width: area.width, height: 1 },
    );
}

// ── Results ────────────────────────────────────────────────────────────

fn draw_results(frame: &mut Frame, area: Rect, app: &App) {
    let res = match &app.clean_result {
        Some(r) => r,
        None => {
            frame.render_widget(Paragraph::new("No results yet").alignment(Alignment::Center), area);
            return;
        }
    };

    // Hero
    let hero_y = area.y;
    let hero = vec![
        Line::from(Span::styled("  ✓  Clean Complete!", Style::default().fg(Theme::success()).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled(format!("  {} ", format_bytes(res.freed)), Style::default().fg(Theme::success()).add_modifier(Modifier::BOLD)),
            Span::styled("recovered  ", Style::default().fg(Theme::text())),
            Span::styled("·", Style::default().fg(Theme::dim())),
            Span::styled(format!("  {} removed", res.removed), Style::default().fg(Theme::text())),
            Span::styled("  ·", Style::default().fg(Theme::dim())),
            Span::styled(format!("  {} skipped", res.skipped), Style::default().fg(Theme::muted())),
        ]),
        Line::from(Span::styled("  Safe to use — only temporary and cache files were removed.", Style::default().fg(Theme::dim()))),
    ];
    frame.render_widget(Paragraph::new(hero), Rect { x: area.x, y: hero_y, width: area.width, height: 3 });

    // Separator
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("─".repeat(area.width as usize), Style::default().fg(Theme::border())))),
        Rect { x: area.x, y: hero_y + 4, width: area.width, height: 1 },
    );

    // Breakdown
    let mut y = hero_y + 5;
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("  Breakdown by category:", Style::default().fg(Theme::muted()).add_modifier(Modifier::BOLD)))),
        Rect { x: area.x, y, width: area.width, height: 1 },
    );
    y += 1;
    if let Some(rep) = &app.scan_report {
        for cat in &rep.results {
            if y >= area.y + area.height - 2 { break; }
            let selected = app.clean_selected_snapshot.contains(&cat.category.id);
            let icon = if selected { "✓" } else { "–" };
            let col = if selected { Theme::success() } else { Theme::dim() };
            let line = Line::from(vec![
                Span::styled(format!("    {} ", icon), Style::default().fg(col)),
                Span::styled(format!("{} ", cat.category.icon), Style::default().fg(Theme::muted())),
                Span::styled(format!("{:<22}", cat.category.label), Style::default().fg(Theme::text())),
                Span::styled(format!("{:>9}", format_bytes(cat.total_size)), Style::default().fg(Theme::muted())),
                Span::styled(format!("  ({} files)", cat.files.len()), Style::default().fg(Theme::dim())),
            ]);
            frame.render_widget(Paragraph::new(line), Rect { x: area.x, y, width: area.width, height: 1 });
            y += 1;
        }
    }

    if !res.errors.is_empty() && y + 2 < area.y + area.height {
        y += 1;
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(format!("    {} files skipped (in use or protected):", res.skipped), Style::default().fg(Theme::warning())))),
            Rect { x: area.x, y, width: area.width, height: 1 },
        );
        y += 1;
        for (p, reason) in res.errors.iter().take(3) {
            if y >= area.y + area.height - 1 { break; }
            let path_str = p.display().to_string();
            let short = if path_str.len() > 50 { format!("…{}", &path_str[path_str.len()-50..]) } else { path_str };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(format!("      · {} — {}", short, reason), Style::default().fg(Theme::dim())))),
                Rect { x: area.x, y, width: area.width, height: 1 },
            );
            y += 1;
        }
        if res.errors.len() > 3 && y < area.y + area.height {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(format!("      … and {} more", res.errors.len() - 3), Style::default().fg(Theme::dim())))),
                Rect { x: area.x, y, width: area.width, height: 1 },
            );
        }
    }

    // Footer actions
    let footer_y = area.y + area.height.saturating_sub(1);
    let footer = Line::from(vec![
        Span::styled("  [R] Rescan", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
        Span::styled("   [Q] Quit", Style::default().fg(Theme::muted())),
    ]);
    frame.render_widget(Paragraph::new(footer).alignment(Alignment::Center), Rect { x: area.x, y: footer_y, width: area.width, height: 1 });
}

// ── Modals ─────────────────────────────────────────────────────────────

fn draw_confirm_modal(frame: &mut Frame, area: Rect, app: &App) {
    let (total_size, total_files) = app.selected_total();
    let w = 60.min(area.width.saturating_sub(4));
    let h = 9;
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let modal = Rect { x, y, width: w, height: h };

    frame.render_widget(Clear, modal);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Theme::card()))
        .title(Line::from(Span::styled(" Confirm Cleanup ", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD))));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Remove ", Style::default().fg(Theme::text())),
            Span::styled(format!("{} ", format_bytes(total_size)), Style::default().fg(Theme::warning()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("in {} files", total_files), Style::default().fg(Theme::text())),
            Span::styled("?", Style::default().fg(Theme::text())),
        ]),
        Line::from(Span::styled(format!(" {} categories selected", app.selected.len()), Style::default().fg(Theme::muted()))),
        Line::from(""),
        Line::from(Span::styled(" This only removes safe temp/cache files. ", Style::default().fg(Theme::dim()))),
        Line::from(vec![
            Span::styled("  [Y] Yes, clean  ", Style::default().bg(Theme::success()).fg(Color::Rgb(15,17,23)).add_modifier(Modifier::BOLD)),
            Span::styled("  [N] Cancel  ", Style::default().bg(Theme::border()).fg(Theme::text())),
        ]),
    ];
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}

fn draw_details_modal(frame: &mut Frame, area: Rect, app: &App) {
    let report = match &app.scan_report {
        Some(r) => r,
        None => return,
    };
    if app.selected_idx >= report.results.len() { return; }
    let cat = &report.results[app.selected_idx];
    let w = 70.min(area.width.saturating_sub(4));
    let h = 16.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let modal = Rect { x, y, width: w, height: h };
    frame.render_widget(Clear, modal);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::border_accent()))
        .style(Style::default().bg(Theme::card()))
        .title(Line::from(vec![
            Span::styled(format!(" {} ", cat.category.icon), Style::default().fg(Theme::accent())),
            Span::styled(cat.category.label.clone(), Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ·  {}  ·  {} files", format_bytes(cat.total_size), cat.files.len()), Style::default().fg(Theme::muted())),
        ]));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    // Show top files
    let mut lines = vec![
        Line::from(Span::styled(format!(" {}  — {}", cat.category.description, if app.selected.contains(&cat.category.id) { "selected" } else { "not selected" }), Style::default().fg(Theme::dim()))),
        Line::from(Span::styled("─".repeat(inner.width as usize), Style::default().fg(Theme::border()))),
    ];
    // Sort by size descending and take top
    let mut files = cat.files.clone();
    files.sort_by(|a,b| b.size.cmp(&a.size));
    for f in files.iter().take((h as usize).saturating_sub(6)) {
        let name = f.path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let dir = f.path.parent().and_then(|p| p.to_str()).unwrap_or("");
        let short_dir = if dir.len() > 30 { format!("…{}", &dir[dir.len()-30..]) } else { dir.to_string() };
        lines.push(Line::from(vec![
            Span::styled(format!("  {:>8}  ", format_bytes(f.size)), Style::default().fg(Theme::accent())),
            Span::styled(name.to_string(), Style::default().fg(Theme::text())),
        ]));
        lines.push(Line::from(Span::styled(format!("    {}", short_dir), Style::default().fg(Theme::dim()))));
    }
    if files.len() > (h as usize).saturating_sub(6) {
        lines.push(Line::from(Span::styled(format!("  … and {} more", files.len() - (h as usize).saturating_sub(6)), Style::default().fg(Theme::dim()))));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Space toggle  ·  Esc close", Style::default().fg(Theme::muted()))));
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn draw_help_modal(frame: &mut Frame, area: Rect) {
    let w = 62.min(area.width.saturating_sub(4));
    let h = 18.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let modal = Rect { x, y, width: w, height: h };
    frame.render_widget(Clear, modal);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::accent()))
        .style(Style::default().bg(Theme::card()))
        .title(Line::from(Span::styled(" Help — Keyboard Shortcuts ", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD))));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    let help = vec![
        Line::from(Span::styled("  Navigation", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD))),
        Line::from(vec![Span::styled("    ↑/k  Down/j  ←/h  →/l", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("  Move focus", Style::default().fg(Theme::muted()))]),
        Line::from(vec![Span::styled("    Enter", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("          Activate focused action", Style::default().fg(Theme::muted()))]),
        Line::from(vec![Span::styled("    Space", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("          Toggle / select", Style::default().fg(Theme::muted()))]),
        Line::from(vec![Span::styled("    Esc", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("            Back / cancel", Style::default().fg(Theme::muted()))]),
        Line::from(""),
        Line::from(Span::styled("  Actions", Style::default().fg(Theme::text()).add_modifier(Modifier::BOLD))),
        Line::from(vec![Span::styled("    S", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("              Start scan", Style::default().fg(Theme::muted()))]),
        Line::from(vec![Span::styled("    C", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("              Clean selected (when available)", Style::default().fg(Theme::muted()))]),
        Line::from(vec![Span::styled("    R", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("              Rescan", Style::default().fg(Theme::muted()))]),
        Line::from(vec![Span::styled("    Q", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("              Quit", Style::default().fg(Theme::muted()))]),
        Line::from(vec![Span::styled("    ?", Style::default().fg(Theme::accent()).add_modifier(Modifier::BOLD)), Span::styled("              Toggle this help", Style::default().fg(Theme::muted()))]),
        Line::from(""),
        Line::from(Span::styled("  Tip: Focused items show ▸ and accent highlight.", Style::default().fg(Theme::dim()))),
        Line::from(Span::styled("  Press ? or Esc to close.", Style::default().fg(Theme::dim()))),
    ];
    frame.render_widget(Paragraph::new(help), inner);
}
