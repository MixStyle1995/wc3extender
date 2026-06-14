use core::ffi::c_void;

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

pub type RegisterNativeFn = unsafe extern "C" fn(
    impl_fn: *const c_void,
    name: *const u8,
    sig: *const u8,
) -> i32;

pub type RegisterNativeHandler = RegisterNativeFn;

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

pub type MakeJassStringFn = unsafe extern "thiscall" fn(
    this: *const c_void,
    c_string: *const u8,
) -> i32;

#[allow(dead_code)]
pub type JassStringToCStrFn = unsafe extern "thiscall" fn(
    this: *const c_void,
) -> *const i8;

#[allow(dead_code)]
pub type JassGetSubsystemFn = unsafe extern "C" fn(id: u32) -> *const c_void;

pub type JassStringHandleToArgFn = unsafe extern "thiscall" fn(
    this: *const c_void,
    handle: u32,
) -> *const c_void;

pub type JassStringArgFromHandleByIndexFn = unsafe extern "C" fn(
    instance_index: u32,
    handle: u32,
) -> *const c_void;

pub type JassStringHandleFromCStrByIndexFn = unsafe extern "C" fn(
    instance_index: u32,
    c_string: *const u8,
) -> u32;

pub type JassInstanceFromIndexFn = unsafe extern "C" fn(instance_index: u32) -> *const c_void;
pub type IsTriggerEnabledFn = unsafe extern "C" fn(trigger: u32) -> i32;
pub type TriggerEvaluateFn = unsafe extern "C" fn(trigger: u32) -> i32;
pub type TriggerExecuteFn = unsafe extern "C" fn(trigger: u32) -> i32;
pub type PlayerFn = unsafe extern "C" fn(index: u32) -> u32;
