use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_log_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("vayload-kit").join("logs")
}

static LOG_WRITER: Mutex<Option<File>> = Mutex::new(None);

pub fn init_logging() {
    let log_dir = get_log_dir();
    let _ = fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("vk.log");

    // Abrimos el archivo UNA SOLA VEZ
    let file = OpenOptions::new().create(true).append(true).open(log_path).ok();

    if let Ok(mut guard) = LOG_WRITER.lock() {
        *guard = file;
    }
}

fn get_timestamp() -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();

    let z = (secs / 86400) as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + (if m <= 2 { 1 } else { 0 });

    let hh = (secs % 86400) / 3600;
    let mm = (secs % 3600) / 60;
    let ss = secs % 60;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hh, mm, ss)
}

pub fn log(level: &str, message: &str) {
    let timestamp = get_timestamp();

    let log_entry = format!("[{}] [{}] {}\n", timestamp, level, message);

    #[allow(clippy::collapsible_if)]
    if let Ok(mut guard) = LOG_WRITER.lock() {
        if let Some(ref mut file) = *guard {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }
}

#[allow(unused)]
pub fn info(message: &str) {
    log("INFO", message);
}

pub fn error(message: &str) {
    log("ERROR", message);
}

#[allow(unused)]
pub fn warn(message: &str) {
    log("WARN", message);
}

pub fn get_log_path_for_read() -> PathBuf {
    get_log_dir().join("vk.log")
}

#[macro_export]
macro_rules! lerror {
    ($($arg:tt)*) => {{
        $crate::logger::error(&format!($($arg)*));
    }};
}
