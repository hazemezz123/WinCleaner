use cleansweep::scanner::config::{CleanerConfig, load_config};
#[test]
fn test_default_config_has_all_categories() {
    let cfg = CleanerConfig::default();
    assert!(cfg.enabled_categories.contains(&"temp".to_string()));
    assert!(cfg.enabled_categories.contains(&"large_files".to_string()));
}
#[test]
fn test_save_and_load_roundtrip() {
    let dir = std::env::temp_dir().join("wincleaner_test_config");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.toml");
    let cfg = CleanerConfig::default();
    cfg.save_to(&path).unwrap();
    let loaded = CleanerConfig::load_from(&path).unwrap();
    assert_eq!(cfg.enabled_categories, loaded.enabled_categories);
}
