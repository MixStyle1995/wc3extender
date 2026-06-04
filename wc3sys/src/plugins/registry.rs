use core::ffi::CStr;
use std::sync::Mutex;

use windows_sys::Win32::Foundation::HMODULE;

use wc3::Wc3Plugin;

#[derive(Debug)]
pub struct LoadedPlugin {
    #[allow(dead_code)]
    pub module: HMODULE,
    pub descriptor: *const Wc3Plugin,
}

unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

static PLUGINS: Mutex<Vec<LoadedPlugin>> = Mutex::new(Vec::new());

pub fn add(plugin: LoadedPlugin) {
    PLUGINS.lock().unwrap().push(plugin);
}

pub fn is_loaded(name: &str) -> bool {
    let plugins = PLUGINS.lock().unwrap();

    plugins.iter().any(|p| {
        let descriptor = unsafe { &*p.descriptor };

        if descriptor.name.is_null() {
            return false;
        }

        let cstr = unsafe { CStr::from_ptr(descriptor.name as *const i8) };
        cstr.to_str().map(|s| s == name).unwrap_or(false)
    })
}
