use core::ffi::c_void;
use std::ffi::CString;

use crate::{c_abi, engines, logging};

#[unsafe(no_mangle)]
pub extern "C" fn wc3sys_register_native(
    name: *const u8,
    signature: *const u8,
    function: *const c_void,
) -> i32 {
    let Some(name_s) = (unsafe { c_abi::borrowed_str_from_ptr(name) }) else {
        logging::warn("wc3sys_register_native: null/invalid name");
        return 0;
    };

    let Some(sig_s) = (unsafe { c_abi::borrowed_str_from_ptr(signature) }) else {
        logging::warn(&format!("wc3sys_register_native({name_s}): null/invalid signature"));
        return 0;
    };

    let Ok(c_name) = CString::new(name_s) else { return 0 };
    let Ok(c_sig) = CString::new(sig_s) else { return 0 };

    match engines::request_plugin_native(c_name, c_sig, function) {
        Ok(()) => 1,
        Err(e) => {
            logging::warn(&format!("wc3sys_register_native({name_s}): {e}"));
            0
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wc3sys_callbacks_mint(engine_name: *const u8, opaque: u64) -> u32 {
    let Some(name) = (unsafe { c_abi::borrowed_str_from_ptr(engine_name) }) else {
        logging::warn("wc3sys_callbacks_mint: null/invalid engine name");
        return 0;
    };

    match name {
        "lua" => engines::mint_callback("lua", opaque),
        _ => {
            logging::warn(&format!("wc3sys_callbacks_mint: unsupported engine '{name}'"));
            0
        }
    }
}
