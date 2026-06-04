use core::ffi::CStr;

use super::raw;

const DEFAULT_MPQ_OPEN_FLAGS: u32 = 2;

pub fn mount_mpq_file(path: &CStr, priority: i32) -> Option<u32> {
    raw::open_archive_file(path, priority, DEFAULT_MPQ_OPEN_FLAGS)
}
