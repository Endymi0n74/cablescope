// ─── Simple File Logger ───────────────────────────────────────────

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();

        let log_dir = log_dir();
        let _ = std::fs::create_dir_all(&log_dir);
    });
}

pub fn log_file(message: &str) {
    let path = today_log_path();
    let now = chrono_free();

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();

    if let Some(ref mut f) = file {
        let _ = writeln!(f, "[{}] {}", now, message);
    }
}

pub fn read_today_log() -> String {
    let path = today_log_path();
    std::fs::read_to_string(&path).unwrap_or_default()
}

// Reserved for the Settings tab log viewer (not yet wired to the UI).
#[allow(dead_code)]
pub fn list_log_files() -> Vec<String> {
    let dir = log_dir();
    if !dir.exists() {
        return Vec::new();
    }

    std::fs::read_dir(&dir)
        .ok()
        .map(|entries| {
            let mut files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "log").unwrap_or(false))
                .filter_map(|e| {
                    e.file_name()
                        .to_str()
                        .map(|s| s.replace(".log", "").to_string())
                })
                .collect();
            files.sort();
            files.reverse();
            files
        })
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn read_log_file(date: &str) -> String {
    let path = log_dir().join(format!("{}.log", date));
    std::fs::read_to_string(&path).unwrap_or_default()
}

// ─── Helpers ──────────────────────────────────────────────────────

fn log_dir() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(base)
        .join("CableScope")
        .join("logs")
}

fn today_log_path() -> std::path::PathBuf {
    let date = today_date();
    log_dir().join(format!("{}.log", date))
}

fn today_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Days since Unix epoch (simplified UTC calendar)
    let days = secs / 86400;
    let mut year = 1970u32;
    let mut remaining = days;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining < days_in_year as u64 {
            break;
        }
        remaining -= days_in_year as u64;
        year += 1;
    }

    let leap = is_leap(year);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut month = 1u32;
    for &md in &month_days {
        if remaining < md as u64 {
            break;
        }
        remaining -= md as u64;
        month += 1;
    }

    let day = remaining as u32 + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn chrono_free() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
