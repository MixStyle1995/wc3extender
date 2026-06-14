pub use wc3::addresses;
mod archives;
mod bootstrap;

mod c_abi;
mod engines;
mod hooks;
mod error;
mod game;
mod natives;
mod ui;
mod jass;
mod lifecycle;
mod logging;
mod paths;
mod plugins;

use core::ffi::c_void;
use core::ptr;

use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::Threading::CreateThread;

const DLL_PROCESS_ATTACH: u32 = 1;


#[unsafe(no_mangle)]
pub extern "C" fn wc3sys_game_addrs() -> *const wc3::addresses::GameAddrs {
    wc3::addresses::get_ptr()
}

#[unsafe(no_mangle)]
pub extern "system" fn DllMain(
    hmodule: HMODULE,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
			paths::set_wc3sys_module(hmodule);
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
