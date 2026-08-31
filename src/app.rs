use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};

use chrono::Local;

use crate::scanner::{
    cleaner::{CleanEvent, CleanResult}, config::CleanerConfig, disk::{get_disk_info, DiskInfo, format_bytes}, engine::{ScanEvent, ScanReport}, get_categories as get_cats, history::{append_history, load_history, HistoryEntry}
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Scanning,
    Review,
    Cleaning,
    Results,
}

#[derive(Debug, Clone)]
pub struct ScanProgressItem {
    pub id: String,
    pub label: String,
    pub icon: char,
    pub files: usize,
    pub size: u64,
    pub done: bool,
    pub spinner_frame: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DashboardAction {
    Scan,
    Clean,
    Rescan,
    Help,
}

impl DashboardAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Scan => "Scan",
            Self::Clean => "Clean",
            Self::Rescan => "Rescan",
            Self::Help => "Help",
        }
    }
    pub fn key(&self) -> &'static str {
        match self {
            Self::Scan => "S",
            Self::Clean => "C",
            Self::Rescan => "R",
            Self::Help => "?",
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub disk: DiskInfo,
    pub last_scan: Option<String>,
    pub scan_report: Option<ScanReport>,
    pub selected: HashSet<String>,
    pub selected_idx: usize,
    pub scan_progress: Vec<ScanProgressItem>,
    pub scan_rx: Option<Receiver<ScanEvent>>,
    pub scan_tx: Option<Sender<ScanEvent>>,
    pub scanning: bool,

    pub clean_result: Option<CleanResult>,
    pub clean_progress: (usize, usize, u64),
    pub clean_current: Option<String>,
    pub clean_rx: Option<Receiver<CleanEvent>>,
    pub clean_total_files: usize,
    pub clean_selected_snapshot: HashSet<String>,

    pub show_confirm: bool,
    pub show_help: bool,
    pub show_details: bool,
    pub should_quit: bool,
    pub tick: usize,

    // Focus navigation
    pub dashboard_focus: usize,
    pub review_focus_is_list: bool, // true = list focused, false = actions focused if we had actions bar in review

    pub config: CleanerConfig,
    pub history: Vec<HistoryEntry>,
}

impl App {
    pub fn new() -> Self {
        let config = CleanerConfig::load();
        let history = load_history(&HistoryEntry::history_path()).unwrap_or_default();
        Self {
            screen: Screen::Dashboard,
            disk: get_disk_info(),
            last_scan: None,
            scan_report: None,
            selected: HashSet::new(),
            selected_idx: 0,
            scan_progress: Vec::new(),
            scan_rx: None,
            scan_tx: None,
            scanning: false,
            clean_result: None,
            clean_progress: (0,0,0),
            clean_current: None,
            clean_rx: None,
            clean_total_files: 0,
            clean_selected_snapshot: HashSet::new(),
            show_confirm: false,
            show_help: false,
            show_details: false,
            should_quit: false,
            tick: 0,
            dashboard_focus: 0,
            review_focus_is_list: true,
            config,
            history,
        }
    }

    pub fn recoverable_str(&self) -> String {
        if let Some(report) = &self.scan_report {
            format_bytes(report.total_size)
        } else {
            "—".to_string()
        }
    }

    pub fn selected_total(&self) -> (u64, usize) {
        if let Some(report) = &self.scan_report {
            report.recoverable_for(&self.selected)
        } else {
            (0,0)
        }
    }

    pub fn dashboard_actions(&self) -> Vec<DashboardAction> {
        let mut actions = Vec::new();
        actions.push(DashboardAction::Scan);
        let (sel_size, sel_count) = self.selected_total();
        let has_scan = self.scan_report.is_some();
        // Clean only if something selected
        if has_scan && sel_count > 0 {
            actions.push(DashboardAction::Clean);
        }
        if has_scan || self.last_scan.is_some() {
            actions.push(DashboardAction::Rescan);
        }
        actions.push(DashboardAction::Help);
        actions
    }

    pub fn ensure_dashboard_focus_in_range(&mut self) {
        let n = self.dashboard_actions().len();
        if n == 0 {
            self.dashboard_focus = 0;
        } else if self.dashboard_focus >= n {
            self.dashboard_focus = n - 1;
        }
    }

    pub fn start_scan(&mut self) {
        self.screen = Screen::Scanning;
        self.scanning = true;
        self.scan_progress.clear();
        self.scan_report = None;
        self.selected.clear();
        self.selected_idx = 0;
        self.show_help = false;
        self.show_details = false;

        let cats: Vec<_> = get_cats().into_iter().filter(|c| self.config.is_enabled(&c.id)).collect();
        for c in &cats {
            self.scan_progress.push(ScanProgressItem {
                id: c.id.clone(),
                label: c.label.clone(),
                icon: c.icon,
                files: 0,
                size: 0,
                done: false,
                spinner_frame: 0,
            });
        }

        let (tx, rx) = channel();
        self.scan_rx = Some(rx);
        let tx_clone = tx.clone();
        self.scan_tx = Some(tx);

        std::thread::spawn(move || {
            crate::scanner::engine::scan_all(cats, tx_clone);
        });
    }

    pub fn start_clean(&mut self) {
        if self.scan_report.is_none() { return; }
        let report = self.scan_report.clone().unwrap();
        let selected = self.selected.clone();
        self.clean_selected_snapshot = selected.clone();
        let (_sz, total_files) = report.recoverable_for(&selected);
        if total_files == 0 { return; }

        self.screen = Screen::Cleaning;
        self.show_confirm = false;
        self.show_help = false;
        self.show_details = false;
        self.clean_progress = (0,0,0);
        self.clean_current = None;
        self.clean_total_files = total_files;
        self.clean_result = None;

        let (tx, rx) = channel();
        self.clean_rx = Some(rx);
        std::thread::spawn(move || {
            crate::scanner::cleaner::clean_selected(&report, &selected, tx);
        });
    }

    pub fn on_tick(&mut self) {
        self.tick += 1;
        if self.tick % 20 == 0 {
            self.disk = get_disk_info();
        }
        for p in &mut self.scan_progress {
            if !p.done {
                p.spinner_frame = p.spinner_frame.wrapping_add(1);
            }
        }
        self.poll_scan();
        self.poll_clean();
        self.ensure_dashboard_focus_in_range();
    }

    fn poll_scan(&mut self) {
        let events: Vec<ScanEvent> = if let Some(rx) = &self.scan_rx {
            let mut v = Vec::new();
            while let Ok(ev) = rx.try_recv() {
                v.push(ev);
            }
            v
        } else {
            Vec::new()
        };
        let mut done_report: Option<ScanReport> = None;
        for ev in events {
            match ev {
                ScanEvent::Started { category: _ } => {}
                ScanEvent::Progress { category, files, size } => {
                    if let Some(item) = self.scan_progress.iter_mut().find(|p| p.id == category) {
                        item.files = files;
                        item.size = size;
                    }
                }
                ScanEvent::CategoryDone(res) => {
                    if let Some(item) = self.scan_progress.iter_mut().find(|p| p.id == res.category.id) {
                        item.files = res.files.len();
                        item.size = res.total_size;
                        item.done = true;
                    }
                }
                ScanEvent::Done(report) => {
                    done_report = Some(report);
                }
                ScanEvent::Error(e) => {
                    eprintln!("scan error: {}", e);
                }
            }
        }
        if let Some(report) = done_report {
            self.scan_report = Some(report.clone());
            self.selected = report.results.iter().map(|r| r.category.id.clone()).collect();
            self.last_scan = Some(Local::now().format("%Y-%m-%d %H:%M").to_string());
            self.scanning = false;
            self.scan_rx = None;
            self.screen = Screen::Review;
            self.selected_idx = 0;
            for p in &mut self.scan_progress {
                p.done = true;
            }
        }
    }

    fn poll_clean(&mut self) {
        let events: Vec<CleanEvent> = if let Some(rx) = &self.clean_rx {
            let mut v = Vec::new();
            while let Ok(ev) = rx.try_recv() {
                v.push(ev);
            }
            v
        } else {
            Vec::new()
        };
        let mut done: Option<CleanResult> = None;
        for ev in events {
            match ev {
                CleanEvent::Progress { removed, skipped, freed, current } => {
                    self.clean_progress = (removed, skipped, freed);
                    self.clean_current = Some(current.display().to_string());
                }
                CleanEvent::Done(res) => {
                    done = Some(res);
                }
                CleanEvent::Error(e) => {
                    eprintln!("clean error: {}", e);
                }
            }
        }
        if let Some(res) = done {
            // Append history entry
            let per_category = self.scan_report.as_ref().map(|report| report.results.iter().map(|r| (r.category.id.clone(), r.total_size)).collect()).unwrap_or_default();
            let entry = HistoryEntry { date: Local::now().format("%Y-%m-%d").to_string(), freed: res.freed, per_category, total_files: res.removed };
            append_history(&HistoryEntry::history_path(), &entry).ok();
            self.history.push(entry);
            self.clean_result = Some(res);
            self.clean_rx = None;
            self.screen = Screen::Results;
            self.disk = get_disk_info();
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(report) = &self.scan_report {
            if self.selected_idx < report.results.len() {
                let id = report.results[self.selected_idx].category.id.clone();
                if self.selected.contains(&id) {
                    self.selected.remove(&id);
                } else {
                    self.selected.insert(id);
                }
            }
        }
    }

    pub fn toggle_all(&mut self) {
        if let Some(report) = &self.scan_report {
            if self.selected.len() == report.results.len() {
                self.selected.clear();
            } else {
                self.selected = report.results.iter().map(|r| r.category.id.clone()).collect();
            }
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        if let Some(report) = &self.scan_report {
            let len = report.results.len() as i32;
            if len == 0 { return; }
            let mut idx = self.selected_idx as i32 + delta;
            if idx < 0 { idx = len - 1; }
            if idx >= len { idx = 0; }
            self.selected_idx = idx as usize;
        }
    }

    pub fn move_dashboard_focus(&mut self, delta: i32) {
        let n = self.dashboard_actions().len() as i32;
        if n == 0 { return; }
        let mut idx = self.dashboard_focus as i32 + delta;
        if idx < 0 { idx = n - 1; }
        if idx >= n { idx = 0; }
        self.dashboard_focus = idx as usize;
    }

    pub fn activate_dashboard_focused(&mut self) {
        let actions = self.dashboard_actions();
        if self.dashboard_focus >= actions.len() { return; }
        match actions[self.dashboard_focus] {
            DashboardAction::Scan => self.start_scan(),
            DashboardAction::Clean => {
                // trigger clean via review flow? If on dashboard, go to review first if needed, or start clean directly
                if self.scan_report.is_some() {
                    let (size, count) = self.selected_total();
                    if count > 0 {
                        self.show_confirm = true;
                        self.screen = Screen::Review;
                    }
                } else {
                    self.start_scan();
                }
            },
            DashboardAction::Rescan => self.start_scan(),
            DashboardAction::Help => self.show_help = true,
        }
    }

    // Global key handling — returns true if consumed
    pub fn handle_global_keys(&mut self, code: crossterm::event::KeyCode, modifiers: crossterm::event::KeyModifiers) -> bool {
        use crossterm::event::KeyCode::*;
        // Help toggle
        if code == Char('?') || (code == Char('/') && modifiers.contains(crossterm::event::KeyModifiers::SHIFT)) {
            self.show_help = !self.show_help;
            // help modal takes precedence, don't propagate
            if self.show_help {
                self.show_confirm = false;
                self.show_details = false;
            }
            return true;
        }
        if self.show_help {
            if code == Esc || code == Char('q') || code == Char('?') {
                self.show_help = false;
                return true;
            }
            // Any key closes help? No, only specified
            return true;
        }
        if self.show_details {
            if code == Esc || code == Enter || code == Char('q') {
                self.show_details = false;
                return true;
            }
            return true;
        }
        if self.show_confirm {
            // handled per-screen, but Esc should close confirm before global
            return false;
        }
        false
    }

    pub fn handle_key_dashboard(&mut self, key: crossterm::event::KeyCode, mods: crossterm::event::KeyModifiers) {
        use crossterm::event::KeyCode::*;
        if self.handle_global_keys(key, mods) { return; }
        match key {
            Char('s') | Char('S') => { self.start_scan(); },
            Char('c') | Char('C') => {
                let actions = self.dashboard_actions();
                if actions.contains(&DashboardAction::Clean) {
                    self.show_confirm = true;
                    // need scan_report to be Some to show review
                    if self.scan_report.is_some() {
                        self.screen = Screen::Review;
                    } else {
                        self.start_scan();
                    }
                }
            },
            Char('r') | Char('R') => {
                if self.scan_report.is_some() || self.last_scan.is_some() {
                    self.start_scan();
                }
            },
            Char('q') => { self.should_quit = true; },
            Esc => { /* nothing to go back */ },
            Up | Char('k') => { self.move_dashboard_focus(-1); },
            Down | Char('j') => { self.move_dashboard_focus(1); },
            Left | Char('h') => { self.move_dashboard_focus(-1); },
            Right | Char('l') => { self.move_dashboard_focus(1); },
            Enter | Char(' ') => { self.activate_dashboard_focused(); },
            _ => {}
        }
    }

    pub fn handle_key_scanning(&mut self, key: crossterm::event::KeyCode, mods: crossterm::event::KeyModifiers) {
        use crossterm::event::KeyCode::*;
        if self.handle_global_keys(key, mods) { return; }
        match key {
            Esc | Char('q') => { self.should_quit = true; },
            Char('c') if mods.contains(crossterm::event::KeyModifiers::CONTROL) => { self.should_quit = true; },
            _ => {}
        }
    }

    pub fn handle_key_review(&mut self, key: crossterm::event::KeyCode, mods: crossterm::event::KeyModifiers) {
        if self.handle_global_keys(key, mods) { return; }
        if self.show_confirm {
            match key {
                crossterm::event::KeyCode::Char('y') | crossterm::event::KeyCode::Char('Y') => {
                    self.start_clean();
                }
                crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') | crossterm::event::KeyCode::Esc => {
                    self.show_confirm = false;
                }
                crossterm::event::KeyCode::Enter => {
                    self.start_clean();
                }
                _ => {}
            }
            return;
        }
        if self.show_details {
            // handled in global
            return;
        }
        match key {
            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => self.move_selection(-1),
            crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => self.move_selection(1),
            crossterm::event::KeyCode::Left | crossterm::event::KeyCode::Char('h') => self.move_selection(-1),
            crossterm::event::KeyCode::Right | crossterm::event::KeyCode::Char('l') => self.move_selection(1),
            crossterm::event::KeyCode::Char(' ') => self.toggle_selected(),
            crossterm::event::KeyCode::Char('a') | crossterm::event::KeyCode::Char('A') => self.toggle_all(),
            crossterm::event::KeyCode::Enter => {
                // Open details for focused category
                self.show_details = true;
            },
            crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Char('C') => {
                let (_size, count) = self.selected_total();
                if count > 0 {
                    self.show_confirm = true;
                }
            },
            crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                self.start_scan();
            },
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                self.start_scan();
            },
            crossterm::event::KeyCode::Esc => {
                self.screen = Screen::Dashboard;
                self.ensure_dashboard_focus_in_range();
            }
            crossterm::event::KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    pub fn handle_key_cleaning(&mut self, key: crossterm::event::KeyCode, mods: crossterm::event::KeyModifiers) {
        if self.handle_global_keys(key, mods) { return; }
        match key {
            crossterm::event::KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    pub fn handle_key_results(&mut self, key: crossterm::event::KeyCode, mods: crossterm::event::KeyModifiers) {
        if self.handle_global_keys(key, mods) { return; }
        match key {
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Char('R') => {
                self.screen = Screen::Dashboard;
                self.start_scan();
            }
            crossterm::event::KeyCode::Char('s') | crossterm::event::KeyCode::Char('S') => {
                self.screen = Screen::Dashboard;
                self.start_scan();
            },
            crossterm::event::KeyCode::Esc => self.screen = Screen::Dashboard,
            crossterm::event::KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }
}
