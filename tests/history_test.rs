use cleansweep::scanner::history::{HistoryEntry, append_history, load_history};
#[test]
fn test_append_and_load() {
    let dir = std::env::temp_dir().join("wincleaner_hist_test");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("history.json");
    // Ensure deterministic run (remove previous history if exists)
    let _ = std::fs::remove_file(&p);
    let e = HistoryEntry {
        date: "2026-09-01".to_string(),
        freed: 123456,
        per_category: vec![("temp".to_string(), 123456)],
        total_files: 10,
    };
    append_history(&p, &e).unwrap();
    let v = load_history(&p).unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].freed, 123456);
}
