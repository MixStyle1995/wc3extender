use std::path::PathBuf;

use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameA;

pub fn process_exe_dir() -> Option<PathBuf> {
    let mut buf = [0u8; 1024];

    let len = unsafe {
        GetModuleFileNameA(
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            buf.len() as u32,
        )
    };

    if len == 0 {
        return None;
    }

    let exe = String::from_utf8_lossy(&buf[..len as usize]).to_string();

    PathBuf::from(exe).parent().map(|p| p.to_path_buf())
}

pub fn logs_dir() -> Option<PathBuf> {
    Some(process_exe_dir()?.join("logs"))
}

pub fn plugins_dir() -> Option<PathBuf> {
    Some(process_exe_dir()?.join("plugins"))
}
