use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use super::engine::{CategoryResult, ScanReport};

#[derive(Debug, Clone)]
pub struct CleanResult {
    pub removed: usize,
    pub skipped: usize,
    pub freed: u64,
    pub errors: Vec<(PathBuf, String)>,
}

#[derive(Debug, Clone)]
pub enum CleanEvent {
    Progress { removed: usize, skipped: usize, freed: u64, current: PathBuf },
    Done(CleanResult),
    Error(String),
}

pub fn clean_selected(
    report: &ScanReport,
    selected: &HashSet<String>,
    sender: Sender<CleanEvent>,
) -> CleanResult {
    let mut removed = 0usize;
    let mut skipped = 0usize;
    let mut freed = 0u64;
    let mut errors = Vec::new();

    // Collect files to delete
    let mut to_delete: Vec<(PathBuf, u64)> = Vec::new();
    for cat in &report.results {
        if selected.contains(&cat.category.id) {
            for f in &cat.files {
                to_delete.push((f.path.clone(), f.size));
            }
        }
    }

    for (path, size) in to_delete {
        // Safety checks
        if !is_safe_to_delete(&path) {
            skipped += 1;
            errors.push((path.clone(), "skipped: not in whitelist".to_string()));
            let _ = sender.send(CleanEvent::Progress {
                removed,
                skipped,
                freed,
                current: path,
            });
            continue;
        }

        let delete_result = if path.is_dir() {
            std::fs::remove_dir(&path)
        } else {
            std::fs::remove_file(&path)
        };
        match delete_result {
            Ok(_) => {
                removed += 1;
                freed += size;
            }
            Err(e) => {
                // Try to handle directory case? If path is file but we list only files, ok
                let msg = e.to_string();
                if msg.contains("Access is denied") || msg.contains("being used") || msg.contains("os error 32") {
                    skipped += 1;
                    errors.push((path.clone(), "in use".to_string()));
                } else if msg.contains("os error 5") {
                    skipped += 1;
                    errors.push((path.clone(), "access denied".to_string()));
                } else {
                    skipped += 1;
                    errors.push((path.clone(), msg));
                }
            }
        }

        // Send progress every 10 files or on skip
        if (removed + skipped) % 10 == 0 || skipped > 0 {
            let _ = sender.send(CleanEvent::Progress {
                removed,
                skipped,
                freed,
                current: path,
            });
        }
    }

    // Try to remove empty directories in temp (optional, safe)
    // For MVP skip dir removal to stay safe

    let result = CleanResult {
        removed,
        skipped,
        freed,
        errors,
    };
    let _ = sender.send(CleanEvent::Done(result.clone()));
    result
}

fn is_safe_to_delete(path: &PathBuf) -> bool {
    let path_str = path.to_string_lossy().to_lowercase().replace("/", "\\");
    // Whitelist prefixes
    let lad = std::env::var("LOCALAPPDATA").unwrap_or_default().to_lowercase();
    let temp = std::env::var("TEMP").unwrap_or_default().to_lowercase();
    let tmp = std::env::var("TMP").unwrap_or_default().to_lowercase();
    let download = dirs::download_dir()
        .map(|p| p.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let mut safe_prefixes = vec![
        lad.clone(),
        temp.clone(),
        tmp.clone(),
        r"c:\windows\temp".to_string(),
        r"c:\windows\logs".to_string(),
        r"c:\windows\softwaredistribution\download".to_string(),
        r"c:\$recycle.bin".to_string(),
        format!(r"{}\microsoft\windows\inetcache", lad),
        format!(r"{}\microsoft\windows\deliveryoptimization", lad),
    ];
    if !download.is_empty() {
        safe_prefixes.push(download.clone());
    }

    for prefix in safe_prefixes {
        if !prefix.is_empty() && path_str.starts_with(&prefix.replace("/", "\\")) {
            return true;
        }
    }

    // Also allow any path that contains \temp\ or \cache\ if under user profile
    if path_str.contains(r"\appdata\local\temp\") || path_str.contains(r"\temp\") {
        // Ensure not system critical
        if !path_str.contains(r"\system32") && !path_str.contains(r"\program files") {
            return true;
        }
    }
    if path_str.contains(r"\cache") && path_str.contains(r"\appdata\") {
        return true;
    }
    // Be conservative: if not matched, skip
    false
}

pub fn clean_sync(report: &ScanReport, selected: &HashSet<String>) -> CleanResult {
    let (tx, _rx) = std::sync::mpsc::channel();
    clean_selected(report, selected, tx)
}
