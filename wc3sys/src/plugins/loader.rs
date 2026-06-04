use std::ffi::CString;
use std::fs;
use std::path::{Path, PathBuf};

use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

use wc3::{Wc3PluginInitFn, WC3_API_VERSION, WC3_PLUGIN_ENTRYPOINT};

use super::registry::{self, LoadedPlugin};
use crate::{logging, paths};

pub unsafe fn load_all() {
    let dlls = find_plugin_dlls();

    if dlls.is_empty() {
        logging::info("no plugins found");
        return;
    }

    for dll in dlls {
        match unsafe { load_one(&dll) } {
            Ok(plugin) => {
                logging::info(&format!("loaded {}", dll.display()));

                let descriptor = unsafe { &*plugin.descriptor };
                if let Some(on_load) = descriptor.on_plugin_loaded {
                    unsafe { on_load() };
                }

                registry::add(plugin);
            }
            Err(e) => {
                logging::error(&format!("plugin {} failed: {e}", dll.display()));
            }
        }
    }
}

unsafe fn load_one(path: &Path) -> Result<LoadedPlugin, String> {
    let path_str = path.to_string_lossy();

    let c_path = CString::new(path_str.as_bytes())
        .map_err(|_| "plugin path contained a null byte".to_string())?;

    let module = unsafe { LoadLibraryA(c_path.as_ptr() as *const u8) };

    if module.is_null() {
        return Err("LoadLibraryA failed".into());
    }

    let proc = unsafe { GetProcAddress(module, WC3_PLUGIN_ENTRYPOINT.as_ptr()) }
        .ok_or_else(|| "missing wc3_plugin_init export".to_string())?;

    let init_fn: Wc3PluginInitFn = unsafe { core::mem::transmute(proc) };

    let descriptor = unsafe { init_fn() };

    if descriptor.is_null() {
        return Err("plugin returned null descriptor".into());
    }

    if unsafe { (*descriptor).version } != WC3_API_VERSION {
        return Err("plugin API version mismatch".into());
    }

    Ok(LoadedPlugin { module, descriptor })
}

fn find_plugin_dlls() -> Vec<PathBuf> {
    let Some(plugins_dir) = paths::plugins_dir() else {
        return Vec::new();
    };

    if !plugins_dir.is_dir() {
        return Vec::new();
    }

    let mut dlls = Vec::new();

    let Ok(entries) = fs::read_dir(&plugins_dir) else {
        return dlls;
    };

    for entry in entries.flatten() {
        let folder = entry.path();
        if !folder.is_dir() { continue; }

        let Ok(inner) = fs::read_dir(&folder) else { continue };

        for f in inner.flatten() {
            let path = f.path();
            if !path.is_file() { continue; }

            let is_dll = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("dll"))
                .unwrap_or(false);

            if is_dll {
                dlls.push(path);
            }
        }
    }

    dlls
}
