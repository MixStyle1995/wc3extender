use std::ffi::CString;

use super::raw;

pub fn read_cached_game_file(path: &str) -> Option<Vec<u8>> {
    let path = CString::new(path).ok()?;
    raw::read_cached_game_file(&path)
}

pub fn read_mpq_file(path: &str) -> Option<Vec<u8>> {
    let path = CString::new(path).ok()?;
    raw::read_mpq_file(&path)
}
