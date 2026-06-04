use core::ffi::c_void;

use wc3::InlineHook;

use crate::addresses;


pub const REGISTER_NATIVE: &str = "jass_register_native";
pub const RUN_FUNCTION: &str = "jass_run_function";
pub const REGISTER_NATIVES: &str = "jass_register_natives";
pub const INVOKE_CODE_BY_ID: &str = "jass_invoke_code_by_id";

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CodeResult {
    Success = 1,
    OpLimit = 2,
    ThreadPause = 3,
    ThreadSync = 4,
    VariableUninitialized = 6,
    DivideByZero = 7,
}

pub type RegisterNativeHandler = unsafe extern "C" fn(
    function: *const c_void,
    name: *const u8,
    signature: *const u8,
) -> i32;

pub type RunFunctionFn = unsafe extern "C" fn(
    this: *mut c_void,
    function_name: *const u8,
    arg_block: i32,
    flag: i32,
    op_limit: u32,
    a6: i32,
) -> CodeResult;

pub type RunFunctionHandler = RunFunctionFn;

pub type RegisterNativesHandler = unsafe extern "C" fn() -> i32;

pub type InvokeCodeFn = unsafe extern "cdecl" fn(
    jass_instance_index: u32,
    code_id: u32,
    a3: u32,
    out_ptr: u32,
    op_limit: u32,
    a6: u32,
    a7: u32,
) -> i32;

pub type InvokeCodeHandler = InvokeCodeFn;

pub fn register_native(handler: RegisterNativeHandler) -> InlineHook {
    InlineHook::new(
        REGISTER_NATIVE,
        addresses::get().register_native,
        handler as *const () as usize,
    )
}

pub fn run_function(handler: RunFunctionHandler) -> InlineHook {
    InlineHook::new(
        RUN_FUNCTION,
        addresses::get().run_function,
        handler as *const () as usize,
    )
}

pub fn register_natives(handler: RegisterNativesHandler) -> InlineHook {
    InlineHook::new(
        REGISTER_NATIVES,
        addresses::get().register_natives,
        handler as *const () as usize,
    )
}

pub fn invoke_code_by_id(handler: InvokeCodeHandler) -> InlineHook {
    InlineHook::new(
        INVOKE_CODE_BY_ID,
        addresses::get().invoke_code_by_id,
        handler as *const () as usize,
    )
}
