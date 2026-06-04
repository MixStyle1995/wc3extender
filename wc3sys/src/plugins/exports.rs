use crate::{c_abi, logging};

pub fn init() -> Result<(), String> {
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn wc3sys_is_plugin_loaded(name: *const u8) -> bool {
    let Some(n) = (unsafe { c_abi::borrowed_str_from_ptr(name) }) else {
        logging::warn("wc3sys_is_plugin_loaded: null/invalid name");
        return false;
    };

    super::registry::is_loaded(n)
}
