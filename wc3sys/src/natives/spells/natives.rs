use core::ffi::c_void;
use std::ffi::CString;

use crate::{addresses, engines, logging};

use crate::game::raw::{unit_handle_to_cunit, RawUnit};
type RawCBuff = usize;

const VT_INITIALIZE: usize = 0x35C;

type AttachEffectToUnitFn = unsafe extern "thiscall" fn(unit: RawUnit, effect: RawCBuff) -> i32;
type CBuffEntanglingRootsInitializeFn = unsafe extern "thiscall" fn(
    buff: RawCBuff,
    unit: RawUnit,
    source: RawUnit,
    duration: *const f32,
    damage_per_second: *const f32,
) -> i32;

#[inline]
unsafe fn vtable_slot(object: usize, offset: usize) -> usize {
    unsafe {
        let vtable = (object as *const usize).read_unaligned();
        ((vtable + offset) as *const usize).read_unaligned()
    }
}

pub unsafe extern "C" fn cbuff_entangling_roots_initialize_native(
    visual_rawcode: u32,
    unit_handle: u32,
    source_handle: u32,
    duration_ptr: u32,
    damage_per_second_ptr: u32,
) -> u32 {
    if visual_rawcode == 0 || duration_ptr == 0 || damage_per_second_ptr == 0 {
        return 0;
    }

    let unit = unit_handle_to_cunit(unit_handle);
    let source = unit_handle_to_cunit(source_handle);
    if unit == 0 || source == 0 {
        return 0;
    }

    let buff = unsafe { super::cbuff::construct_cbuff(visual_rawcode,0) };
    if buff == 0 {
        logging::warn(&format!(
            "[spells] CBuffEntanglingRootsInitialize: construct_cbuff failed for 0x{visual_rawcode:x}"
        ));
        return 0;
    }

    let initialize_addr = unsafe { vtable_slot(buff, VT_INITIALIZE) };
    if initialize_addr == 0 {
        return 0;
    }

    let initialize: CBuffEntanglingRootsInitializeFn =
        unsafe { core::mem::transmute(initialize_addr) };

    let ok = unsafe {
        initialize(
            buff,
            unit,
            source,
            duration_ptr as *const f32,
            damage_per_second_ptr as *const f32,
        )
    };

    let attach: AttachEffectToUnitFn =
        unsafe { core::mem::transmute(addresses::get().buffs.attach_effect_to_unit) };
    unsafe { attach(unit, buff) };

    logging::info(&format!(
        "[spells] CBuffEntanglingRootsInitialize: buff=0x{buff:x} unit=0x{unit:x} ok={ok}"
    ));

    ok as u32
}

fn register_spell_native(name: &str, signature: &str, func: *const c_void) {
    let c_name = CString::new(name).unwrap();
    let c_sig = CString::new(signature).unwrap();

    match engines::request_plugin_native(c_name, c_sig, func) {
        Ok(_) => crate::log_native_registration!("spells: queued {name} {signature}"),
        Err(e) => logging::error(&format!("spells: failed to queue {name}: {e}")),
    }
}

pub fn register_custom_natives() {
    register_spell_native(
        "CBuffEntanglingRootsInitialize",
        "(IHunit;Hunit;RR)I",
        cbuff_entangling_roots_initialize_native as *const c_void,
    );
}
