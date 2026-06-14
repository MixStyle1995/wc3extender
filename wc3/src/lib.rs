#![allow(non_camel_case_types)]

pub mod addresses;
pub mod archives;
pub mod c_abi;
pub mod game;
pub mod memory;
pub mod frames;
pub mod abi;
pub mod hook;
pub mod inline_hook;
pub mod jass;
pub mod patch;
pub mod plugins;

mod sys;

use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

pub use abi::{
    OnPluginLoadedFn,
    Wc3Plugin,
    Wc3PluginInitFn,
    WC3_API_VERSION,
    WC3_PLUGIN_ENTRYPOINT,
};

pub use jass::{
    callbacks_mint,
    is_plugin_loaded,
    make_jass_string,
    mount_mpq_file,
    register_native,
};

pub use hook::{
    Hook,
    HookManager,
    HookType,
};

pub use inline_hook::{InlineHook, InlineHookError, InlineHookErrorKind};

pub use archives::queue_mpq_file;

pub use patch::{
    alloc_executable,
    build_call32,
    build_jmp,
    calc_rel32,
    patch_call_target,
    patch_jmp_target,
    read_bytes,
    with_writable,
    write_bytes,
};

pub fn process_base() -> usize {
    static BASE: OnceLock<usize> = OnceLock::new();
    *BASE.get_or_init(|| unsafe { GetModuleHandleW(core::ptr::null()) as usize })
}

pub const IDA_BASE: usize = 0x400000;

#[inline]
pub fn rebase(static_addr: usize) -> usize {
    static_addr - IDA_BASE + process_base()
}
