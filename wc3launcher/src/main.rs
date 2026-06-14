#![windows_subsystem = "windows"]
use std::ffi::{c_void, CString};
use std::fs;
use std::mem::{size_of, zeroed};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::thread;
use std::time::Duration;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Memory::{
    VirtualAllocEx, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE,
};
use windows_sys::Win32::System::Threading::{
    CreateProcessA, CreateRemoteThread, ResumeThread, WaitForSingleObject, CREATE_SUSPENDED,
    PROCESS_INFORMATION, STARTUPINFOA,
};
use windows_sys::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameA, OPENFILENAMEA, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_HIDEREADONLY,
    OFN_NOCHANGEDIR,
};

const CONFIG_FILE: &str = "wc3launcher.cfg";
const DLL_FILE: &str = "wc3sys.dll";
const INFINITE: u32 = 0xFFFF_FFFF;

fn main() {
    if let Err(e) = run() {
        eprintln!("wc3launcher: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let launcher_dir = launcher_dir()?;
    let config_path = launcher_dir.join(CONFIG_FILE);
    let dll_path = launcher_dir.join(DLL_FILE);

    if !dll_path.is_file() {
        return Err(format!("missing {}", dll_path.display()));
    }

    let wc3_exe = load_or_pick_exe(&config_path)?;
    let process = launch_suspended(&wc3_exe)?;

    thread::sleep(Duration::from_millis(500));

    let inject_result = unsafe { inject_dll(process.process_info.hProcess, &dll_path) };

    unsafe {
        ResumeThread(process.process_info.hThread);
        CloseHandle(process.process_info.hThread);
        CloseHandle(process.process_info.hProcess);
    }

    inject_result
}

fn launcher_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "launcher exe has no parent directory".to_string())
}

fn load_or_pick_exe(config_path: &Path) -> Result<PathBuf, String> {
    if let Ok(saved) = fs::read_to_string(config_path) {
        let path = PathBuf::from(saved.trim());
        if !path.as_os_str().is_empty() {
            return Ok(path);
        }
    }

    let picked = pick_exe_file().ok_or_else(|| "no exe selected".to_string())?;

    fs::write(config_path, picked.to_string_lossy().as_bytes())
        .map_err(|e| format!("write {}: {e}", config_path.display()))?;

    Ok(picked)
}

fn pick_exe_file() -> Option<PathBuf> {
    let mut file_buf = [0u8; 260];
    let filter = b"Executable Files (*.exe)\0*.exe\0All Files (*.*)\0*.*\0\0";
    let title = b"Select Warcraft III executable\0";

    let mut ofn: OPENFILENAMEA = unsafe { zeroed() };
    ofn.lStructSize = size_of::<OPENFILENAMEA>() as u32;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_NOCHANGEDIR;

    let ok = unsafe { GetOpenFileNameA(&mut ofn) };
    if ok == 0 {
        return None;
    }

    let nul = file_buf.iter().position(|b| *b == 0)?;
    Some(PathBuf::from(
        String::from_utf8_lossy(&file_buf[..nul]).to_string(),
    ))
}

struct LaunchedProcess {
    process_info: PROCESS_INFORMATION,
}

fn launch_suspended(exe: &Path) -> Result<LaunchedProcess, String> {
    let exe_c = cstring_path(exe)?;
    let working_dir = exe.parent().map(cstring_path).transpose()?;

    let mut startup: STARTUPINFOA = unsafe { zeroed() };
    startup.cb = size_of::<STARTUPINFOA>() as u32;

    let mut process_info: PROCESS_INFORMATION = unsafe { zeroed() };

    let ok = unsafe {
        CreateProcessA(
            exe_c.as_ptr() as *const u8,
            null_mut(),
            null(),
            null(),
            0,
            CREATE_SUSPENDED,
            null(),
            working_dir
                .as_ref()
                .map(|s| s.as_ptr() as *const u8)
                .unwrap_or(null()),
            &startup,
            &mut process_info,
        )
    };

    if ok == 0 {
        return Err(format!("CreateProcessA failed for {}", exe.display()));
    }

    Ok(LaunchedProcess { process_info })
}

unsafe fn inject_dll(process: HANDLE, dll_path: &Path) -> Result<(), String> {
    let dll_c = cstring_path(dll_path)?;
    let bytes = dll_c.as_bytes_with_nul();

    let remote_mem = unsafe {
        VirtualAllocEx(
            process,
            null(),
            bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        )
    };

    if remote_mem.is_null() {
        return Err("VirtualAllocEx failed".to_string());
    }

    let mut written = 0usize;
    let ok = unsafe {
        WriteProcessMemory(
            process,
            remote_mem,
            bytes.as_ptr() as *const c_void,
            bytes.len(),
            &mut written,
        )
    };

    if ok == 0 || written != bytes.len() {
        return Err("WriteProcessMemory failed".to_string());
    }

    let kernel32 = unsafe { GetModuleHandleA(b"kernel32.dll\0".as_ptr()) };
    if kernel32.is_null() {
        return Err("GetModuleHandleA(kernel32.dll) failed".to_string());
    }

    let load_library = unsafe { GetProcAddress(kernel32, b"LoadLibraryA\0".as_ptr()) }
        .ok_or_else(|| "GetProcAddress(LoadLibraryA) failed".to_string())?;

    let thread = unsafe {
        CreateRemoteThread(
            process,
            null(),
            0,
            Some(std::mem::transmute(load_library)),
            remote_mem,
            0,
            null_mut(),
        )
    };

    if thread.is_null() {
        return Err("CreateRemoteThread failed".to_string());
    }

    unsafe {
        WaitForSingleObject(thread, INFINITE);
        CloseHandle(thread);
    }

    Ok(())
}

fn cstring_path(path: &Path) -> Result<CString, String> {
    CString::new(path.to_string_lossy().as_bytes())
        .map_err(|_| format!("path contains NUL: {}", path.display()))
}
