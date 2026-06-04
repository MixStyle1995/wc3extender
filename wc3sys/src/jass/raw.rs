use core::ffi::{c_void, CStr};

use crate::addresses;

pub type MakeJassStringFn = unsafe extern "thiscall" fn(
    this: *const c_void,
    c_string: *const u8,
) -> i32;

pub type JassStringToCStrFn = unsafe extern "thiscall" fn(
    this: *const c_void,
) -> *const i8;

pub type RegisterNativeFn = unsafe extern "C" fn(
    impl_fn: *const c_void,
    name: *const u8,
    sig: *const u8,
) -> i32;

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


pub fn make_jass_string(c_string: *const u8) -> i32 {
    let addrs = addresses::get();

    unsafe {
        let jass_vm = *(addrs.jass_vm_global as *const *const c_void);

        if jass_vm.is_null() {
            return 0;
        }

        let f: MakeJassStringFn = core::mem::transmute(addrs.make_jass_string);
        f(jass_vm, c_string)
    }
}

pub fn jass_string_to_str(handle: i32) -> Option<String> {
    if handle == 0 {
        return None;
    }
    let addrs = addresses::get();

    unsafe {
        let f: JassStringToCStrFn = core::mem::transmute(addrs.jass_string_to_cstr);
        let p = f(handle as usize as *const c_void);
        if p.is_null() {
            return None;
        }
        Some(CStr::from_ptr(p).to_string_lossy().into_owned())
    }
}

pub unsafe fn jass_instance_from_index(instance_index: u32) -> Option<usize> {
    let addrs = addresses::get();

    unsafe {
        let f: JassInstanceFromIndexFn = core::mem::transmute(addrs.jass_instance_from_index);
        let p = f(instance_index);
        if p.is_null() {
            None
        } else {
            Some(p as usize)
        }
    }
}

pub fn jass_string_handle_to_arg(handle: u32, instance_index: u32) -> Option<usize> {
    let addrs = addresses::get();

    unsafe {
        let instance = jass_instance_from_index(instance_index)?;
        let f: JassStringHandleToArgFn = core::mem::transmute(addrs.jass_string_handle_to_arg);
        let p = f(instance as *const c_void, handle);
        if p.is_null() {
            None
        } else {
            Some(p as usize)
        }
    }
}

pub fn native_string_to_str(native_string: usize) -> Option<String> {
    if native_string == 0 {
        return None;
    }

    unsafe {

        let value = ((native_string + 0x08) as *const usize).read_unaligned();
        if value == 0 {
            return None;
        }

        let c_string = ((value + 0x1C) as *const *const i8).read_unaligned();
        if c_string.is_null() {
            return None;
        }

        Some(CStr::from_ptr(c_string).to_string_lossy().into_owned())
    }
}

pub fn jass_string_handle_to_str(handle: u32, instance_index: u32) -> Option<String> {
    let native_string = jass_string_handle_to_arg(handle, instance_index)?;
    native_string_to_str(native_string)
}

pub fn string_to_arg(c_string: *const u8, instance_index: u32) -> Option<usize> {
    if c_string.is_null() {
        return None;
    }

    let addrs = addresses::get();

    unsafe {
        let from_cstr: JassStringHandleFromCStrByIndexFn =
            core::mem::transmute(addrs.jass_string_handle_from_cstr_by_index);
        let to_arg: JassStringArgFromHandleByIndexFn =
            core::mem::transmute(addrs.jass_string_arg_from_handle_by_index);

        let handle = from_cstr(instance_index, c_string);
        if handle == 0 {
            return None;
        }

        let arg = to_arg(instance_index, handle);
        if arg.is_null() {
            None
        } else {
            Some(arg as usize)
        }
    }
}


pub fn register_native(name: &CStr, signature: &CStr, function: *const c_void) -> i32 {
    let addrs = addresses::get();

    unsafe {
        let f: RegisterNativeFn = core::mem::transmute(addrs.register_native);
        f(function, name.as_ptr() as *const u8, signature.as_ptr() as *const u8)
    }
}
