use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub date: String,
    pub freed: u64,
    pub per_category: Vec<(String, u64)>,
    pub total_files: usize,
}
impl HistoryEntry {
    pub fn history_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or(PathBuf::from("."))
            .join("wincleaner")
            .join("history.json")
    }
}
pub fn load_history(p: &Path) -> anyhow::Result<Vec<HistoryEntry>> {
    if !p.exists() {
        return Ok(vec![]);
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(p)?)?)
}
pub fn append_history(p: &Path, e: &HistoryEntry) -> anyhow::Result<()> {
    let mut v = load_history(p).unwrap_or_default();
    v.push(e.clone());
    if v.len() > 30 {
        v.drain(0..v.len() - 30);
    }
    if let Some(d) = p.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}
pub fn sparkline_data(history: &[HistoryEntry]) -> Vec<u64> {
    history.iter().map(|h| h.freed / (1024 * 1024) as u64).collect() // MB
}
