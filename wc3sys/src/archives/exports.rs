use core::ffi::CStr;

use crate::logging;

#[unsafe(no_mangle)]
pub extern "C" fn wc3sys_mount_mpq_file(path: *const u8, priority: i32) -> u32 {
    if path.is_null() {
        logging::warn("wc3sys_mount_mpq_file: null path");
        return 0;
    }

    let path = unsafe { CStr::from_ptr(path as *const i8) };

    match super::mount::mount_mpq_file_cstr(path, priority) {
        Some(archive) => archive,
        None => {
            logging::warn(&format!(
                "wc3sys_mount_mpq_file({:?}, priority={priority}) failed",
                path.to_string_lossy()
            ));
            0
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn wc3sys_queue_mpq_file(path: *const u8, priority: i32) -> i32 {
    if path.is_null() {
        logging::warn("wc3sys_queue_mpq_file: null path");
        return 0;
    }

    let path = unsafe { CStr::from_ptr(path as *const i8) };

    match super::queue::queue_mpq_file_cstr(path, priority) {
        Ok(()) => 1,
        Err(e) => {
            logging::warn(&format!(
                "wc3sys_queue_mpq_file({:?}, priority={priority}) failed: {e}",
                path.to_string_lossy()
            ));
            0
        }
    }
}
