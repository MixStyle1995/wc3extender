use core::ffi::{c_void, CStr};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{addresses, hooks as hook_manager, jass, logging};
use crate::engines::CallbackContext;

static INVOKE_CODE_CALLS: AtomicUsize = AtomicUsize::new(0);
static CURRENT_JASS_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(usize::MAX);
static LAST_JASS_INSTANCE_INDEX: AtomicUsize = AtomicUsize::new(usize::MAX);

pub fn install() -> crate::error::Result<()> {
    hook_manager::install(jass::hooks::register_native(register_native_handler))?;
    hook_manager::install(jass::hooks::run_function(run_function_handler))?;
    hook_manager::install(jass::hooks::register_natives(register_natives_handler))?;
    hook_manager::install(jass::hooks::invoke_code_by_id(invoke_code_handler))?;
    Ok(())
}

pub fn current_jass_instance_index() -> Option<u32> {
    let v = CURRENT_JASS_INSTANCE_INDEX.load(Ordering::Relaxed);
    if v == usize::MAX {
        None
    } else {
        Some(v as u32)
    }
}

pub fn last_jass_instance_index() -> Option<u32> {
    let v = LAST_JASS_INSTANCE_INDEX.load(Ordering::Relaxed);
    if v == usize::MAX {
        None
    } else {
        Some(v as u32)
    }
}


pub fn best_jass_instance_index() -> Option<u32> {
    current_jass_instance_index().or_else(last_jass_instance_index)
}

pub struct JassInstanceIndexGuard {
    saved: usize,
}

impl Drop for JassInstanceIndexGuard {
    fn drop(&mut self) {
        CURRENT_JASS_INSTANCE_INDEX.store(self.saved, Ordering::Relaxed);
    }
}

pub fn enter_jass_instance_index(jass_instance_index: u32) -> JassInstanceIndexGuard {
    LAST_JASS_INSTANCE_INDEX.store(jass_instance_index as usize, Ordering::Relaxed);
    let saved = CURRENT_JASS_INSTANCE_INDEX.swap(jass_instance_index as usize, Ordering::Relaxed);
    JassInstanceIndexGuard { saved }
}

unsafe extern "C" fn register_native_handler(
    function: *const c_void,
    name: *const u8,
    signature: *const u8,
) -> i32 {
    if !name.is_null() && !signature.is_null() {
        let name_s = unsafe { CStr::from_ptr(name as *const i8) }.to_str().ok();
        let sig_s = unsafe { CStr::from_ptr(signature as *const i8) }.to_str().ok();

        if let (Some(n), Some(s)) = (name_s, sig_s) {
            super::manager::on_jass_native_registered(n, s, function);
        }
    }

    let tramp = hook_manager::trampoline(addresses::get().register_native)
        .expect("jass_register_native trampoline missing");

    unsafe {
        let original: jass::raw::RegisterNativeFn = core::mem::transmute(tramp);
        original(function, name, signature)
    }
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
            let _instance_guard = enter_jass_instance_index(instance_index);
            super::manager::on_jass_function_called(name);
        }
    }

    let tramp = hook_manager::trampoline(addresses::get().run_function)
        .expect("jass_run_function trampoline missing");

    unsafe {
        let original: jass::hooks::RunFunctionFn = core::mem::transmute(tramp);
        original(this, function_name, arg_block, flag, op_limit, a6)
    }
}

unsafe extern "C" fn register_natives_handler() -> i32 {
    let tramp = hook_manager::trampoline(addresses::get().register_natives)
        .expect("jass_register_natives trampoline missing");

    let ret = unsafe {
        let original: unsafe extern "C" fn() -> i32 = core::mem::transmute(tramp);
        let retv = original();
        logging::info(&format!("registered game native ret={retv}"));
        retv
    };

    super::manager::on_jass_native_registration_phase();
    ret
}

unsafe fn vm_ptr() -> usize {
    let g = addresses::get().jass_vm_global;
    unsafe { (g as *const usize).read_unaligned() }
}

unsafe extern "cdecl" fn invoke_code_handler(
    jass_instance_index: u32,
    code_id: u32,
    a3: u32,
    out_ptr: u32,
    op_limit: u32,
    a6: u32,
    a7: u32,
) -> i32 {
    let n = INVOKE_CODE_CALLS.fetch_add(1, Ordering::Relaxed) + 1;

    let _instance_guard = enter_jass_instance_index(jass_instance_index);
    crate::natives::frames::events::flush_pending_callbacks();

    if code_id >= 0x80000000 {
        let vm = unsafe { vm_ptr() };
        if vm == 0 {
            logging::warn(&format!("[invoke_code #{n}] ours but vm ptr is null"));
            return 1;
        }

        let cur_fn_addr = vm + 8;
        let saved = unsafe { (cur_fn_addr as *const u32).read_unaligned() };

        logging::info(&format!(
            "[invoke_code #{n}] ours code_id=0x{:x} jass_instance_index=0x{:x} vm=0x{:x} [vm+8]=0x{:x}",
            code_id, jass_instance_index, vm, saved
        ));

        unsafe { (cur_fn_addr as *mut u32).write_unaligned(jass_instance_index) };

        let dispatched = super::manager::try_dispatch_callback_code(
            code_id,
            CallbackContext::InvokeCodeById { jass_instance_index },
        );

        unsafe { (cur_fn_addr as *mut u32).write_unaligned(saved) };

        if dispatched {
            return 1;
        }
    }

    let tramp = hook_manager::trampoline(addresses::get().invoke_code_by_id)
        .expect("jass_invoke_code_by_id trampoline missing");

    unsafe {
        let original: jass::hooks::InvokeCodeFn = core::mem::transmute(tramp);
        original(jass_instance_index, code_id, a3, out_ptr, op_limit, a6, a7)
    }
}
