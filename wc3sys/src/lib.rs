mod addresses;
mod archives;
mod bootstrap;

mod c_abi;
mod engines;
mod hooks;
mod error;
mod natives;
mod ui;
mod jass;
mod logging;
mod paths;
mod plugins;

use core::ffi::c_void;
use core::ptr;

use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::Threading::CreateThread;

const DLL_PROCESS_ATTACH: u32 = 1;

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    _hmodule: HMODULE,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            CreateThread(
                ptr::null(),
                0,
                Some(bootstrap::worker_thread),
                ptr::null(),
                0,
                ptr::null_mut(),
            );
        }
    }

    1
}
