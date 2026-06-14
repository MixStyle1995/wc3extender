use core::ffi::c_void;
use std::ffi::CString;

use crate::{addresses, engines, logging};

pub type RawCBuff = usize;

use crate::game::raw::{unit_handle_to_cunit, RawUnit};

const ADD_REJUVINATION_STATIC: usize = 0xBEF960;
const SBUFF_DATA_DWORDS: usize = 8;

type AddRejuvinationFn = unsafe extern "C" fn(
    unit: RawUnit,
    buff_data: *const u32,
    duration: *const f32,
    heal_life: *const f32,
    heal_mana: *const f32,
    allow_full_life: i32,
    allow_full_mana: i32,
) -> i32;


const REGISTRY_THIS_OFFSET: usize = 0x0C;
const RECORD_DESCRIPTOR_FIELD: usize = 0x78;
const WRAPPER_TO_CBUFF: usize = 0x54;
const VT_CONFIG: usize = 0x32C;
const DESCRIPTOR_SIZE_DWORDS: usize = 11;

type ObjectHashFn = unsafe extern "thiscall" fn(this: *const u32) -> u32;
type RegistryFindFn = unsafe extern "thiscall" fn(this: usize, hash: u32, key: *const u32) -> usize;
type DescriptorBuildFn = unsafe extern "C" fn(out: *mut u32, rawcode: u32, record_field: u32) -> *mut u32;
type MaterializeFn = unsafe extern "C" fn(desc: *mut u32, a: i32, b: i32) -> usize;
type VtableConfigFn = unsafe extern "thiscall" fn(this: RawCBuff, config: *const u32) -> i32;

#[inline]
unsafe fn vtable_slot(object: usize, offset: usize) -> usize {
    unsafe {
        let vtable = (object as *const usize).read_unaligned();
        ((vtable + offset) as *const usize).read_unaligned()
    }
}

pub unsafe fn construct_cbuff(registry_rawcode: u32, visual_rawcode: u32) -> RawCBuff {
    if registry_rawcode == 0 {
        return 0;
    }

    let addrs = addresses::get();
    let registry_root = unsafe {
        (addrs.buffs.effect_registry_root as *const usize).read_unaligned()
    };
    if registry_root == 0 {
        return 0;
    }

    let key = registry_rawcode;
    let object_hash: ObjectHashFn = unsafe { core::mem::transmute(addrs.buffs.object_hash) };
    let hash = unsafe { object_hash(&key as *const u32) };

    let registry_find: RegistryFindFn =
        unsafe { core::mem::transmute(addrs.buffs.effect_registry_find) };
    let record = unsafe {
        registry_find(registry_root + REGISTRY_THIS_OFFSET, hash, &key as *const u32)
    };
    if record == 0 {
        return 0;
    }

    let record_field = unsafe {
        ((record + RECORD_DESCRIPTOR_FIELD) as *const u32).read_unaligned()
    };

    let mut descriptor = [0u32; DESCRIPTOR_SIZE_DWORDS];
    let descriptor_build: DescriptorBuildFn =
        unsafe { core::mem::transmute(addrs.buffs.effect_descriptor_build) };
    unsafe { descriptor_build(descriptor.as_mut_ptr(), registry_rawcode, record_field) };

    let materialize: MaterializeFn = unsafe { core::mem::transmute(addrs.buffs.effect_materialize) };
    let wrapper = unsafe { materialize(descriptor.as_mut_ptr(), 1, 1) };
    if wrapper == 0 {
        return 0;
    }

    let cbuff = unsafe { ((wrapper + WRAPPER_TO_CBUFF) as *const usize).read_unaligned() };
    if cbuff == 0 {
        return 0;
    }

    // The +0x32C config does NOT take the materialize descriptor. Per sub_6F6900
    // it reads its own config struct: config[0] -> this+0x34 (the visual rawcode).
    // The materialize descriptor's [0] is a magic tag, so reusing it stamps the
    // wrong value into the visual field and the art lookup falls back to the
    // dev-default buff. Build a dedicated config struct carrying the rawcode.
    let mut config_struct = [0u32; DESCRIPTOR_SIZE_DWORDS];
    config_struct[0] = visual_rawcode;

    let config_addr = unsafe { vtable_slot(cbuff, VT_CONFIG) };
    if config_addr == 0 {
        return 0;
    }

    let config: VtableConfigFn = unsafe { core::mem::transmute(config_addr) };
    unsafe { config(cbuff, config_struct.as_ptr()) };

    cbuff
}

/// Debug native for testing the real Blizzard Rejuvination applier path.
///
/// JASS ABI: CBuffApplyRejuvinationDebug(visualRawcode:int, unit:unit,
/// duration:real, healLife:real, healMana:real, allowFullLife:boolean,
/// allowFullMana:boolean) -> integer
///
/// Internally this calls 1.29 AddRejuvination at 0xBEF960:
/// AddRejuvination(CUnit*, SBuffData*, duration, healLife, healMana, bool, bool)
pub unsafe extern "C" fn cbuff_apply_rejuvination_debug_native(
    visual_rawcode: u32,
    unit_handle: u32,
    duration_ptr: u32,
    heal_life_ptr: u32,
    heal_mana_ptr: u32,
    allow_full_life: u32,
    allow_full_mana: u32,
) -> u32 {
    if visual_rawcode == 0 || duration_ptr == 0 || heal_life_ptr == 0 || heal_mana_ptr == 0 {
        logging::warn("[spells] CBuffApplyRejuvinationDebug: null visual/real argument");
        return 0;
    }

    let unit = unit_handle_to_cunit(unit_handle);
    if unit == 0 {
        logging::warn("[spells] CBuffApplyRejuvinationDebug: unit handle resolved to null CUnit");
        return 0;
    }

    // SBuffData layout consumed by CBuff config vtable +0x32C / sub_6F6900:
    // [0] visual/effect rawcode -> CBuff+0x34
    // [1] -> CBuff+0xBC
    // [2] -> CBuff+0xB4
    // [3] -> CBuff+0xB8
    // [4] flag -> CBuff flags 0x40000000
    // [5] flag -> CBuff flags 0x20000000
    // [6] -> CBuff+0xC0 subobject
    // [7] -> CBuff+0xC8 subobject
    //
    // This is intentionally minimal for debug: override only the visual field.
    // If the real Rejuv tick still fails, the next target is filling [1..7]
    // from a real sub_B3ABE0-produced SBuffData instead of zeroes.
    let mut buff_data = [0u32; SBUFF_DATA_DWORDS];
    buff_data[0] = visual_rawcode;

    let add_rejuv_addr = addresses::rebase(addresses::get().base, ADD_REJUVINATION_STATIC);
    let add_rejuv: AddRejuvinationFn = unsafe { core::mem::transmute(add_rejuv_addr) };

    let ok = unsafe {
        add_rejuv(
            unit,
            buff_data.as_ptr(),
            duration_ptr as *const f32,
            heal_life_ptr as *const f32,
            heal_mana_ptr as *const f32,
            (allow_full_life != 0) as i32,
            (allow_full_mana != 0) as i32,
        )
    };

    logging::info(&format!(
        "[spells] CBuffApplyRejuvinationDebug: visual=0x{visual_rawcode:x} unit=0x{unit:x} ok={ok}"
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

pub fn register_debug_natives() {
    register_spell_native(
        "CBuffApplyRejuvinationDebug",
        "(IHunit;RRRBB)I",
        cbuff_apply_rejuvination_debug_native as *const c_void,
    );
}

