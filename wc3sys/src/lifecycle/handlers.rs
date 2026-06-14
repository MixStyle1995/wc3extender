use core::ffi::{c_void, CStr};

use crate::addresses;
use crate::archives;
use crate::hooks as hook_manager;
use crate::jass;
use crate::logging;

use super::{
    War3MpqArchivesFailed, War3MpqArchivesInitialized, War3MpqArchivesInitializing,
};

pub fn install() -> crate::error::Result<()> {
    hook_manager::install(jass::hooks::run_function(run_function_handler))?;
    hook_manager::install(jass::hooks::register_natives(register_natives_handler))?;
    hook_manager::install(archives::hooks::init_war3_mpq_archives(
        init_war3_mpq_archives_handler,
    ))?;

    logging::info("lifecycle: hooks installed");
    Ok(())
}

unsafe extern "C" fn run_function_handler(
    this: *mut c_void,
    function_name: *const u8,
    arg_block: i32,
    flag: i32,
    op_limit: u32,
    a6: i32,
) -> jass::hooks::CodeResult {
    if !function_name.is_null() {
        if let Ok(name) = unsafe { CStr::from_ptr(function_name as *const i8) }.to_str() {
            let instance_index = this as usize as u32;
            let _instance_guard = crate::engines::enter_jass_instance_index(instance_index);
            super::observe_jass_function(name);
        }
    }

    let tramp = hook_manager::trampoline(addresses::get().jass.run_function)
        .expect("jass_run_function trampoline missing");

    unsafe {
        let original: jass::hooks::RunFunctionFn = core::mem::transmute(tramp);
        original(this, function_name, arg_block, flag, op_limit, a6)
    }
}

unsafe extern "C" fn register_natives_handler() -> i32 {
    let tramp = hook_manager::trampoline(addresses::get().jass.register_natives)
        .expect("jass_register_natives trampoline missing");

    let ret = unsafe {
        let original: unsafe extern "C" fn() -> i32 = core::mem::transmute(tramp);
        let retv = original();
        crate::log_native_registration!("registered game native ret={retv}");
        retv
    };

    super::observe_native_registration();
    ret
}

unsafe extern "C" fn init_war3_mpq_archives_handler() -> i32 {
    super::emit(War3MpqArchivesInitializing);

    let tramp = hook_manager::trampoline(addresses::get().archives.init_war3_mpq_archives)
        .expect("InitWar3MpqArchives trampoline missing");
    let original: archives::hooks::InitWar3MpqArchivesFn = unsafe { core::mem::transmute(tramp) };

    let result = unsafe { original() };

    if result != 0 {
        super::emit(War3MpqArchivesInitialized);
    } else {
        super::emit(War3MpqArchivesFailed);
    }

    result
}
