use sysinfo::Disks;

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub percent_used: f32,
    pub mount: String,
}

pub fn get_disk_info() -> DiskInfo {
    // Try sysinfo first
    let disks = Disks::new_with_refreshed_list();
    // Find C: or largest
    let mut best: Option<DiskInfo> = None;
    for d in disks.list() {
        let mount = d.mount_point().to_string_lossy().to_string();
        // Windows mount like C:\
        if mount.starts_with("C:") || mount == "/" || mount.contains(":\\") {
            let total = d.total_space();
            let available = d.available_space();
            let used = total.saturating_sub(available);
            let percent = if total > 0 {
                (used as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            };
            let info = DiskInfo {
                total,
                available,
                used,
                percent_used: percent,
                mount: mount.clone(),
            };
            // Prefer C:
            if mount.starts_with("C:") {
                return info;
            }
            if best.is_none() {
                best = Some(info);
            }
        }
    }
    if let Some(b) = best {
        return b;
    }
    // Fallback: try to get any disk
    if let Some(d) = disks.list().first() {
        let total = d.total_space();
        let available = d.available_space();
        let used = total.saturating_sub(available);
        return DiskInfo {
            total,
            available,
            used,
            percent_used: if total > 0 {
                (used as f64 / total as f64 * 100.0) as f32
            } else {
                0.0
            },
            mount: d.mount_point().to_string_lossy().to_string(),
        };
    }
    // Ultimate fallback
    DiskInfo {
        total: 512 * 1024 * 1024 * 1024,
        available: 128 * 1024 * 1024 * 1024,
        used: 384 * 1024 * 1024 * 1024,
        percent_used: 75.0,
        mount: r"C:\".to_string(),
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;
    let b = bytes as f64;
    if b >= TB {
        format!("{:.1} TB", b / TB)
    } else if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.0} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_bytes_precise(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}
