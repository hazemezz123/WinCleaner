use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerConfig {
    pub enabled_categories: HashSet<String>,
    pub temp_age_hours: u64,
    pub log_age_days: u64,
    pub large_file_mb: u64,
}

impl Default for CleanerConfig {
    fn default() -> Self {
        Self {
            enabled_categories: [
                "temp",
                "browser",
                "windows_temp",
                "logs",
                "recycle",
                "update",
                "downloads",
                "large_files",
                "empty_folders",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            temp_age_hours: 24,
            log_age_days: 7,
            large_file_mb: 500,
        }
    }
}

impl CleanerConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or(PathBuf::from("."))
            .join("wincleaner")
            .join("config.toml")
    }

    pub fn load() -> Self {
        Self::load_from(&Self::config_path()).unwrap_or_default()
    }

    pub fn load_from(p: &Path) -> anyhow::Result<Self> {
        Ok(toml::from_str(&std::fs::read_to_string(p)?)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to(&Self::config_path())
    }

    pub fn save_to(&self, p: &Path) -> anyhow::Result<()> {
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(p, toml::to_string(self)?)?;
        Ok(())
    }

    pub fn is_enabled(&self, id: &str) -> bool {
        self.enabled_categories.contains(id)
    }
}

/// Free function alias for `CleanerConfig::load()` — kept for backwards-compat with
/// the spec's `load_config` import (`use cleansweep::scanner::config::load_config`).
pub fn load_config() -> CleanerConfig {
    CleanerConfig::load()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_all_categories() {
        let cfg = CleanerConfig::default();
        assert!(cfg.enabled_categories.contains(&"temp".to_string()));
        assert!(cfg.enabled_categories.contains(&"large_files".to_string()));
        // also verify full default set
        assert!(cfg.enabled_categories.contains(&"browser".to_string()));
        assert!(cfg.enabled_categories.contains(&"logs".to_string()));
        assert_eq!(cfg.temp_age_hours, 24);
        assert_eq!(cfg.log_age_days, 7);
        assert_eq!(cfg.large_file_mb, 500);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("wincleaner_test_config_inline");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        let cfg = CleanerConfig::default();
        cfg.save_to(&path).unwrap();
        let loaded = CleanerConfig::load_from(&path).unwrap();
        assert_eq!(cfg.enabled_categories, loaded.enabled_categories);
        assert_eq!(cfg.temp_age_hours, loaded.temp_age_hours);
        assert_eq!(cfg.log_age_days, loaded.log_age_days);
        assert_eq!(cfg.large_file_mb, loaded.large_file_mb);
    }

    #[test]
    fn test_is_enabled() {
        let cfg = CleanerConfig::default();
        assert!(cfg.is_enabled("temp"));
        assert!(!cfg.is_enabled("nonexistent"));
    }

    #[test]
    fn test_load_nonexistent_returns_default_via_load_from_err() {
        let p = std::env::temp_dir().join("wincleaner_test_config_nonexistent_12345.toml");
        let _ = std::fs::remove_file(&p);
        assert!(CleanerConfig::load_from(&p).is_err());
        // load() should fallback to default when file missing
        // we don't test load() directly as it uses real config path, but load_from error is expected
    }
}
