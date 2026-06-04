use core::ffi::CStr;

use crate::addresses;

pub type OpenArchiveFileFn = unsafe extern "C" fn(
    path: *const u8,
    priority: i32,
    flags: u32,
    out_archive: *mut u32,
) -> i32;

pub fn open_archive_file(path: &CStr, priority: i32, flags: u32) -> Option<u32> {
    let addrs = addresses::get();
    let mut archive = 0u32;

    let ok = unsafe {
        let f: OpenArchiveFileFn = core::mem::transmute(addrs.open_archive_file);
        f(
            path.as_ptr() as *const u8,
            priority,
            flags,
            &mut archive,
        )
    };

    if ok == 0 || archive == 0 {
        None
    } else {
        Some(archive)
    }
}
