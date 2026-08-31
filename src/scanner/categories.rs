use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Category {
    pub id: String,
    pub label: String,
    pub icon: char,
    pub description: String,
    pub paths: Vec<PathBuf>,
    pub patterns: Vec<String>, // glob-like, empty means all files
    pub min_age: Duration,
    pub enabled: bool,
}

impl Category {
    pub fn new(
        id: &str,
        label: &str,
        icon: char,
        description: &str,
        paths: Vec<PathBuf>,
        patterns: Vec<&str>,
        min_age: Duration,
    ) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            icon,
            description: description.to_string(),
            paths: paths.into_iter().filter(|p| !p.as_os_str().is_empty()).collect(),
            patterns: patterns.into_iter().map(|s| s.to_string()).collect(),
            min_age,
            enabled: true,
        }
    }
}

fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var_os(var).map(PathBuf::from)
}

fn local_app_data() -> PathBuf {
    env_path("LOCALAPPDATA").unwrap_or_else(|| PathBuf::from(r"C:\Users\Default\AppData\Local"))
}

fn temp_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(p) = env_path("TEMP") {
        v.push(p);
    }
    if let Some(p) = env_path("TMP") {
        if !v.contains(&p) {
            v.push(p);
        }
    }
    let lad = local_app_data();
    let t = lad.join("Temp");
    if !v.contains(&t) {
        v.push(t);
    }
    v
}

pub fn get_categories() -> Vec<Category> {
    let lad = local_app_data();
    let appdata = env_path("APPDATA").unwrap_or_else(|| lad.clone());

    let mut cats = Vec::new();

    // Temporary Files — 2.4 GB example
    cats.push(Category::new(
        "temp",
        "Temporary Files",
        '◆',
        "User temporary files and caches",
        temp_paths(),
        vec!["*.tmp", "*.temp", "~*"],
        Duration::from_secs(24 * 3600),
    ));

    // Browser Cache
    let mut browser_paths = Vec::new();
    browser_paths.push(lad.join(r"Google\Chrome\User Data\Default\Cache"));
    browser_paths.push(lad.join(r"Google\Chrome\User Data\Default\Code Cache"));
    browser_paths.push(lad.join(r"Microsoft\Edge\User Data\Default\Cache"));
    browser_paths.push(lad.join(r"Microsoft\Edge\User Data\Default\Code Cache"));
    browser_paths.push(lad.join(r"Mozilla\Firefox\Profiles"));
    // Firefox is special — will scan subfolders
    browser_paths.push(appdata.join(r"Mozilla\Firefox\Profiles"));
    cats.push(Category::new(
        "browser",
        "Browser Cache",
        '◎',
        "Chrome, Edge, Firefox caches",
        browser_paths,
        vec![], // all files in cache dirs
        Duration::from_secs(0),
    ));

    // Windows Temp
    cats.push(Category::new(
        "windows_temp",
        "Windows Temp",
        '◈',
        "System temporary files",
        vec![PathBuf::from(r"C:\Windows\Temp")],
        vec![],
        Duration::from_secs(24 * 3600),
    ));

    // Logs & Other Cache
    let mut log_paths = Vec::new();
    log_paths.extend(temp_paths());
    log_paths.push(PathBuf::from(r"C:\Windows\Logs"));
    log_paths.push(lad.join(r"Microsoft\Windows\INetCache"));
    log_paths.push(PathBuf::from(r"C:\Windows\Prefetch"));
    cats.push(Category::new(
        "logs",
        "Logs & Other Cache",
        '◐',
        "Log files, thumbnails, icon cache",
        log_paths,
        vec!["*.log", "*.etl", "Thumbs.db", "IconCache.db", "*.old"],
        Duration::from_secs(7 * 24 * 3600),
    ));

    // Recycle Bin
    cats.push(Category::new(
        "recycle",
        "Recycle Bin",
        '♻',
        "Files in recycle bin",
        vec![PathBuf::from(r"C:\$Recycle.Bin")],
        vec![],
        Duration::from_secs(0),
    ));

    // Delivery Optimization / Update Cache
    cats.push(Category::new(
        "update",
        "Update Cache",
        '▣',
        "Windows update delivery files",
        vec![
            PathBuf::from(r"C:\Windows\SoftwareDistribution\Download"),
            lad.join(r"Microsoft\Windows\DeliveryOptimization\Cache"),
        ],
        vec![],
        Duration::from_secs(7 * 24 * 3600),
    ));

    cats.push(Category::new(
        "downloads",
        "Downloads",
        '⬇',
        "Old downloads >30d",
        vec![dirs::download_dir().unwrap_or(PathBuf::from(r"C:\Users\Default\Downloads"))],
        vec![],
        Duration::from_secs(30 * 24 * 3600),
    ));
    cats.push(Category::new(
        "large_files",
        "Large Files",
        '⬢',
        "Files >500MB anywhere in temp/docs",
        temp_paths(),
        vec![],
        Duration::from_secs(0),
    ));
    cats.push(Category::new(
        "empty_folders",
        "Empty Folders",
        '∅',
        "Empty directories in temp",
        temp_paths(),
        vec![],
        Duration::from_secs(0),
    ));

    // Filter to only categories that have at least one existing path
    // But keep them all for UI; scanner will skip missing paths gracefully
    cats
}

pub fn matches_pattern(path: &PathBuf, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return true;
    }
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let path_str = path.to_string_lossy().to_lowercase();
    for pat in patterns {
        let p = pat.to_lowercase();
        if p.starts_with("*.") {
            let ext = &p[2..];
            if file_name.ends_with(ext) || path_str.ends_with(ext) {
                return true;
            }
        } else if p.contains('*') {
            // simple wildcard: prefix/suffix
            let parts: Vec<&str> = p.split('*').collect();
            if parts.len() == 2 {
                if path_str.starts_with(parts[0]) && path_str.ends_with(parts[1]) {
                    return true;
                }
                if file_name.starts_with(parts[0]) && file_name.ends_with(parts[1]) {
                    return true;
                }
            }
        } else {
            if file_name == p || path_str.contains(&p) {
                return true;
            }
        }
    }
    false
}
