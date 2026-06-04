use core::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

use crate::paths;

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

pub fn init() -> Result<(), String> {
    let dir = paths::logs_dir().ok_or_else(|| "could not determine logs dir".to_string())?;

    fs::create_dir_all(&dir).map_err(|e| format!("create_dir_all: {e}"))?;

    let path = dir.join("wc3sys.log");

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open {path:?}: {e}"))?;

    LOG_FILE
        .set(Mutex::new(file))
        .map_err(|_| "log already initialized".to_string())?;

    Ok(())
}

fn write_line(level: &str, msg: &str) {
    let Some(lock) = LOG_FILE.get() else { return };
    let Ok(mut file) = lock.lock() else { return };
    let _ = writeln!(file, "[{level}] {msg}");
    let _ = file.flush();
}

pub fn error(msg: &str) { write_line("ERROR", msg); }
pub fn warn(msg: &str)  { write_line("WARN",  msg); }
pub fn info(msg: &str)  { write_line("INFO",  msg); }

pub fn error_value(context: &str, err: &impl fmt::Display) {
    error(&format!("{context}: {err}"));
}

pub fn warn_value(context: &str, err: &impl fmt::Display) {
    warn(&format!("{context}: {err}"));
}
