use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};

use walkdir::WalkDir;

use super::categories::{matches_pattern, Category};

#[derive(Debug, Clone)]
pub struct FileItem {
    pub path: PathBuf,
    pub size: u64,
    pub category_id: String,
}

#[derive(Debug, Clone)]
pub struct CategoryResult {
    pub category: Category,
    pub files: Vec<FileItem>,
    pub total_size: u64,
}

impl CategoryResult {
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

#[derive(Debug, Clone)]
pub struct ScanReport {
    pub results: Vec<CategoryResult>,
    pub total_size: u64,
    pub total_files: usize,
    pub duration: Duration,
}

impl ScanReport {
    pub fn recoverable_for(&self, selected: &std::collections::HashSet<String>) -> (u64, usize) {
        let mut size = 0;
        let mut count = 0;
        for r in &self.results {
            if selected.contains(&r.category.id) {
                size += r.total_size;
                count += r.files.len();
            }
        }
        (size, count)
    }
}

#[derive(Debug, Clone)]
pub enum ScanEvent {
    Started { category: String },
    Progress { category: String, files: usize, size: u64 },
    CategoryDone(CategoryResult),
    Done(ScanReport),
    Error(String),
}

fn should_include(path: &PathBuf, metadata: &std::fs::Metadata, category: &Category) -> bool {
    if category.id == "large_files" {
        return metadata.len() > 500 * 1024 * 1024;
    }
    if category.id == "empty_folders" {
        return false;
    }
    // Check pattern
    if !matches_pattern(path, &category.patterns) {
        return false;
    }
    // Check age
    if category.min_age > Duration::from_secs(0) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = SystemTime::now().duration_since(modified) {
                if age < category.min_age {
                    return false;
                }
            }
        }
    }
    // Safety: skip very large files? No, include but we will not delete if suspicious
    // Skip if file is too recent and hidden? For now allow
    true
}

fn scan_category(category: &Category, sender: Option<&Sender<ScanEvent>>) -> CategoryResult {
    let mut files = Vec::new();
    let mut total_size = 0u64;

    if let Some(s) = sender {
        let _ = s.send(ScanEvent::Started {
            category: category.id.clone(),
        });
    }

    if category.id == "empty_folders" {
        for base in &category.paths {
            if !base.exists() {
                continue;
            }
            let walk = WalkDir::new(base)
                .follow_links(false)
                .max_depth(8)
                .into_iter()
                .filter_entry(|e| {
                    let ft = e.file_type();
                    if ft.is_symlink() {
                        return false;
                    }
                    true
                });
            let mut local_count = 0usize;
            for entry in walk.filter_map(|e| e.ok()) {
                if !entry.file_type().is_dir() {
                    continue;
                }
                let path = entry.path().to_path_buf();
                let is_empty = match std::fs::read_dir(&path) {
                    Ok(mut iter) => iter.next().is_none(),
                    Err(_) => false,
                };
                if !is_empty {
                    continue;
                }
                if category.min_age > Duration::from_secs(0) {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(age) = SystemTime::now().duration_since(modified) {
                                if age < category.min_age {
                                    continue;
                                }
                            }
                        }
                    }
                }
                files.push(FileItem {
                    path: path.clone(),
                    size: 0,
                    category_id: category.id.clone(),
                });
                local_count += 1;
                if local_count % 500 == 0 {
                    if let Some(s) = sender {
                        let _ = s.send(ScanEvent::Progress {
                            category: category.id.clone(),
                            files: files.len(),
                            size: total_size,
                        });
                    }
                }
                if files.len() > 100_000 {
                    break;
                }
            }
        }
        let result = CategoryResult {
            category: category.clone(),
            files,
            total_size,
        };
        if let Some(s) = sender {
            let _ = s.send(ScanEvent::CategoryDone(result.clone()));
        }
        return result;
    }

    for base in &category.paths {
        if !base.exists() {
            continue;
        }
        // For Firefox profiles, need to handle nested
        let walk = WalkDir::new(base)
            .follow_links(false)
            .max_depth(8)
            .into_iter()
            .filter_entry(|e| {
                // Skip reparse points / junctions that could loop
                let ft = e.file_type();
                if ft.is_symlink() {
                    return false;
                }
                true
            });

        let mut local_count = 0usize;
        for entry in walk.filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();
            if entry.file_type().is_dir() {
                continue;
            }
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if !metadata.is_file() {
                continue;
            }
            if !should_include(&path, &metadata, category) {
                continue;
            }
            let size = metadata.len();
            // Safety: skip files larger than 500MB in temp? Probably not temp
            // Keep but maybe cap? For MVP include all
            files.push(FileItem {
                path: path.clone(),
                size,
                category_id: category.id.clone(),
            });
            total_size += size;
            local_count += 1;

            // Progress every 500 files
            if local_count % 500 == 0 {
                if let Some(s) = sender {
                    let _ = s.send(ScanEvent::Progress {
                        category: category.id.clone(),
                        files: files.len(),
                        size: total_size,
                    });
                }
            }
            // Safety limit: avoid OOM on huge cache — cap at 100k files per category
            if files.len() > 100_000 {
                break;
            }
        }
    }

    let result = CategoryResult {
        category: category.clone(),
        files,
        total_size,
    };
    if let Some(s) = sender {
        let _ = s.send(ScanEvent::CategoryDone(result.clone()));
    }
    result
}

pub fn scan_all(categories: Vec<Category>, sender: Sender<ScanEvent>) {
    let start = std::time::Instant::now();
    let mut results = Vec::new();
    let mut total_size = 0u64;
    let mut total_files = 0usize;

    for cat in categories {
        let res = scan_category(&cat, Some(&sender));
        total_size += res.total_size;
        total_files += res.files.len();
        results.push(res);
    }

    let report = ScanReport {
        results,
        total_size,
        total_files,
        duration: start.elapsed(),
    };
    let _ = sender.send(ScanEvent::Done(report));
}

pub fn scan_all_sync(categories: Vec<Category>) -> ScanReport {
    let start = std::time::Instant::now();
    let mut results = Vec::new();
    let mut total_size = 0u64;
    let mut total_files = 0usize;
    for cat in categories {
        let res = scan_category(&cat, None);
        total_size += res.total_size;
        total_files += res.files.len();
        results.push(res);
    }
    ScanReport {
        results,
        total_size,
        total_files,
        duration: start.elapsed(),
    }
}
