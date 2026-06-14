const WC3SYS_DLL_W: [u16; 11] = [119, 99, 51, 115, 121, 115, 46, 100, 108, 108, 0];

use core::ffi::c_void;
use std::sync::OnceLock;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

fn wc3sys_handle() -> *mut c_void {
    static H: OnceLock<usize> = OnceLock::new();
    *H.get_or_init(|| unsafe {
        GetModuleHandleW(WC3SYS_DLL_W.as_ptr()) as usize
    }) as *mut c_void
}

macro_rules! sys_fn {
    ($vis:vis $name:ident: $ty:ty) => {
        $vis fn $name() -> $ty {
            static F: OnceLock<usize> = OnceLock::new();
            let addr = *F.get_or_init(|| unsafe {
                GetProcAddress(wc3sys_handle() as _, concat!(stringify!($name), "\0").as_ptr())
                    .map(|f| f as usize)
                    .expect(concat!("wc3sys export ", stringify!($name), " not found"))
            });
            unsafe { core::mem::transmute::<usize, $ty>(addr) }
        }
    };
}

sys_fn!(pub(crate) wc3sys_register_native: extern "C" fn(*const u8, *const u8, *const c_void));
sys_fn!(pub(crate) wc3sys_make_jass_string: extern "C" fn(*const u8) -> i32);
sys_fn!(pub(crate) wc3sys_is_plugin_loaded: extern "C" fn(*const u8) -> bool);
sys_fn!(pub(crate) wc3sys_callbacks_mint: extern "C" fn(*const u8, u64) -> u32);
sys_fn!(pub(crate) wc3sys_mount_mpq_file: extern "C" fn(*const u8, i32) -> u32);
sys_fn!(pub(crate) wc3sys_queue_mpq_file: extern "C" fn(*const u8, i32) -> i32);

sys_fn!(pub(crate) wc3sys_game_addrs: extern "C" fn() -> *const crate::addresses::GameAddrs);
