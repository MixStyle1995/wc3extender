use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;

static WC3SYS_MODULE: OnceLock<usize> = OnceLock::new();

pub fn set_wc3sys_module(module: HMODULE) {
    let _ = WC3SYS_MODULE.set(module as usize);
}

fn module_path(module: HMODULE) -> Option<PathBuf> {
    let mut buf = [0u16; 1024];

    let len = unsafe { GetModuleFileNameW(module, buf.as_mut_ptr(), buf.len() as u32) };

    if len == 0 {
        return None;
    }

    Some(PathBuf::from(OsString::from_wide(&buf[..len as usize])))
}

pub fn wc3sys_dir() -> Option<PathBuf> {
    let module = *WC3SYS_MODULE.get()?;
    module_path(module as HMODULE)?.parent().map(|p| p.to_path_buf())
}

pub fn process_exe_dir() -> Option<PathBuf> {
    module_path(std::ptr::null_mut())?.parent().map(|p| p.to_path_buf())
}

pub fn logs_dir() -> Option<PathBuf> {
    Some(process_exe_dir()?.join("logs"))
}

pub fn plugins_dir() -> Option<PathBuf> {
    Some(wc3sys_dir()?.join("plugins"))
}
