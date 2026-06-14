pub mod abi;
pub mod invoke;
pub mod raw;
pub mod signature;

use core::ffi::c_void;
use std::ffi::CString;

use crate::sys::{
    wc3sys_callbacks_mint,
    wc3sys_is_plugin_loaded,
    wc3sys_make_jass_string,
    wc3sys_mount_mpq_file,
    wc3sys_register_native,
};

pub fn make_jass_string(s: &str) -> i32 {
    let owner = CString::new(s).expect("invalid string");
    wc3sys_make_jass_string()(owner.as_ptr() as *const u8)
}

pub fn is_plugin_loaded(plugin: &str) -> bool {
    let owner = CString::new(plugin).expect("invalid plugin name");
    wc3sys_is_plugin_loaded()(owner.as_ptr() as *const u8)
}

pub fn register_native(func: *const c_void, name: &str, signature: &str) {
    let c_name = CString::new(name).expect("invalid native name");
    let c_sig  = CString::new(signature).expect("invalid native signature");

    wc3sys_register_native()(
        c_name.as_ptr() as *const u8,
        c_sig.as_ptr()  as *const u8,
        func,
    );
}

pub fn callbacks_mint(engine_name: &str, opaque: u64) -> u32 {
    let owner = CString::new(engine_name).expect("invalid engine name");
    wc3sys_callbacks_mint()(owner.as_ptr() as *const u8, opaque)
}


pub fn mount_mpq_file(path: &str, priority: i32) -> Option<u32> {
    let owner = CString::new(path).expect("invalid mpq path");
    match wc3sys_mount_mpq_file()(owner.as_ptr() as *const u8, priority) {
        0 => None,
        archive => Some(archive),
    }
}
