use core::ffi::c_void;
use std::ffi::CString;

use crate::{addresses, engines, logging};


pub use crate::game::raw::RawUnit;

const ABILITY_DWORDS: usize = 0x80;
const ABILITY_NODE_DWORDS: usize = 132;
const ABILITY_LEVEL_DWORDS: usize = 28;
const FAKE_VTABLE_SLOTS: usize = 122;

const CABILITY_METAMORPHOSIS_RAWCODE: u32 = fourcc("AEme");
const CBUFF_METAMORPHOSIS_REGISTRY_RAWCODE: u32 = fourcc("BEme");

const VT_CONFIG: usize = 0x32C;
const VT_METAMORPHOSIS_INITIALIZE: usize = 0x360;

const ABILITY_OWNER_DWORD: usize = 0x30 / 4;
const ABILITY_ID_DWORD: usize = 0x34 / 4;
const ABILITY_LEVEL_DWORD: usize = 0x50 / 4;
const ABILITY_DATA_CACHE_DWORD: usize = 0x54 / 4;
const ABILITY_DATA_ACTIVE_DWORD: usize = 0x58 / 4;

const NODE_VALID_DWORD: usize = 0x2C / 4;
const NODE_LEVELS_DWORD: usize = 0x3C / 4;
const NODE_FLAG_B_DWORD: usize = 0x40 / 4;
const NODE_GLOBAL_VALUE_DWORD: usize = 0x48 / 4;
const NODE_LEVEL_COUNT_DWORD: usize = 0x4C / 4;
const NODE_LEVEL_PTR_DWORD: usize = 0x50 / 4;
const NODE_LEVEL_PTR_SENTINEL_DWORD: usize = 0x20C / 4;

const LEVEL_DURATION_NORMAL_DWORD: usize = 2;
const LEVEL_DURATION_HERO_DWORD: usize = 3;
const LEVEL_HIT_POINT_BONUS_DWORD: usize = 12;
const LEVEL_UNIT_ID_DWORD: usize = 17;
const LEVEL_BUFF_LIST_DWORD: usize = 18;
const LEVEL_BUFF_LIST_COUNT_DWORD: usize = 22;

type AbilityDataLookupFn = unsafe extern "C" fn(ability_id: u32, must_exist: i32) -> usize;
type MetamorphosisCreateFn = unsafe extern "C" fn() -> usize;
type AbilityMorphInitializeFn =
    unsafe extern "thiscall" fn(this: usize, unit: RawUnit, ability_data: *mut c_void) -> i32;
type MetamorphosisApplyFn = unsafe extern "thiscall" fn(this: usize) -> i32;
type UnitCodeToMorphCodeFn = unsafe extern "C" fn(unit_code: u32) -> u32;
type MorphActivateFn = unsafe extern "thiscall" fn(this: usize);

#[inline]
const fn fourcc(s: &str) -> u32 {
    let b = s.as_bytes();
    ((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | b[3] as u32
}

#[inline]
unsafe fn vtable_slot(object: usize, offset: usize) -> usize {
    unsafe {
        let vtable = (object as *const usize).read_unaligned();
        ((vtable + offset) as *const usize).read_unaligned()
    }
}

use crate::game::raw::unit_handle_to_cunit;

unsafe extern "thiscall" fn fake_ability_is_buff_source(_this: *mut c_void) -> i32 {
    0
}

unsafe extern "thiscall" fn fake_ability_is_other_source(_this: *mut c_void) -> i32 {
    0
}

#[repr(C)]
pub struct AbilityData {
    raw: [u32; ABILITY_DWORDS],
    node: Box<[u32; ABILITY_NODE_DWORDS]>,
    level: Box<[u32; ABILITY_LEVEL_DWORDS]>,
    vtable: Box<[usize; FAKE_VTABLE_SLOTS]>,
}

impl AbilityData {
    pub fn metamorphosis(owner: RawUnit, visual_rawcode: u32, morph_unit_rawcode: u32, duration: f32) -> Self {
        let mut node = Box::new([0u32; ABILITY_NODE_DWORDS]);
        let mut level = Box::new([0u32; ABILITY_LEVEL_DWORDS]);
        let mut vtable = Box::new([0usize; FAKE_VTABLE_SLOTS]);

        vtable[120] = fake_ability_is_buff_source as usize;
        vtable[121] = fake_ability_is_other_source as usize;

        let duration_bits = duration.to_bits();
        level[LEVEL_DURATION_NORMAL_DWORD] = duration_bits;
        level[LEVEL_DURATION_HERO_DWORD] = duration_bits;
        level[LEVEL_HIT_POINT_BONUS_DWORD] = 500.0f32.to_bits();
        level[LEVEL_UNIT_ID_DWORD] = morph_unit_rawcode;
        level[LEVEL_BUFF_LIST_DWORD] = visual_rawcode;
        level[LEVEL_BUFF_LIST_COUNT_DWORD] = 1;

        node[NODE_VALID_DWORD] = 1;
        node[NODE_LEVELS_DWORD] = 1;
        node[NODE_FLAG_B_DWORD] = 1;
        node[NODE_GLOBAL_VALUE_DWORD] = 0;
        node[NODE_LEVEL_COUNT_DWORD] = 1;
        node[NODE_LEVEL_PTR_DWORD] = level.as_ptr() as u32;
        node[NODE_LEVEL_PTR_SENTINEL_DWORD] = u32::MAX;

        let mut raw = [0u32; ABILITY_DWORDS];
        raw[0] = vtable.as_ptr() as u32;
        raw[ABILITY_OWNER_DWORD] = owner as u32;
        raw[ABILITY_ID_DWORD] = CABILITY_METAMORPHOSIS_RAWCODE;
        raw[ABILITY_LEVEL_DWORD] = 0;
        raw[ABILITY_DATA_CACHE_DWORD] = node.as_ptr() as u32;
        raw[ABILITY_DATA_ACTIVE_DWORD] = node.as_ptr() as u32;

        Self {
            raw,
            node,
            level,
            vtable,
        }
    }

    pub fn node_ptr(&self) -> usize {
        self.node.as_ptr() as usize
    }
}

/// AbilityData owns a synthetic CAbility-shaped object plus the ability data node
/// that Blizzard getters expect. Metamorphosis may retain the pointer until
/// expiry, so the debug path leaks it until the buff lifecycle cleanup is mapped.
unsafe fn leak_ability_data_for_buff(ability: AbilityData) -> *mut c_void {
    Box::into_raw(Box::new(ability)) as *mut c_void
}

/// JASS ABI:
/// CBuffMetamorphosisApplyDataDebug(visualRawcode:int, unit:unit,
///     morphUnitRawcode:int, duration:real) -> integer
pub unsafe extern "C" fn cbuff_metamorphosis_apply_data_debug_native(
    visual_rawcode: u32,
    unit_handle: u32,
    morph_unit_rawcode: u32,
    duration_ptr: u32,
) -> u32 {
    if visual_rawcode == 0 || morph_unit_rawcode == 0 || duration_ptr == 0 {
        logging::warn("[spells] CBuffMetamorphosisApplyDataDebug: invalid argument");
        return 0;
    }

    let unit = unit_handle_to_cunit(unit_handle);
    if unit == 0 {
        logging::warn("[spells] CBuffMetamorphosisApplyDataDebug: unit handle resolved to null CUnit");
        return 0;
    }

    let duration = unsafe { *(duration_ptr as *const f32) };
    let synthetic = AbilityData::metamorphosis(unit, visual_rawcode, morph_unit_rawcode, duration);
    let synthetic_node = synthetic.node_ptr();

    let create: MetamorphosisCreateFn =
        unsafe { core::mem::transmute(addresses::get().abilities.metamorphosis_create) };
    let ability = unsafe { create() };
    if ability == 0 {
        logging::warn("[spells] CBuffMetamorphosisApplyDataDebug: failed to create CAbilityMetamorphosis");
        return 0;
    }

    let lookup: AbilityDataLookupFn =
        unsafe { core::mem::transmute(addresses::get().abilities.ability_data_lookup) };
    let real_node = unsafe { lookup(CABILITY_METAMORPHOSIS_RAWCODE, 0) };
    if real_node == 0 {
        logging::warn("[spells] CBuffMetamorphosisApplyDataDebug: failed to resolve AEme data node");
        return 0;
    }

    unsafe {
        ((ability + 0x30) as *mut u32).write_unaligned(unit as u32);
        ((ability + 0x34) as *mut u32).write_unaligned(CABILITY_METAMORPHOSIS_RAWCODE);
        ((ability + 0x50) as *mut u32).write_unaligned(0);
    }

    let morph_init: AbilityMorphInitializeFn =
        unsafe { core::mem::transmute(addresses::get().abilities.ability_morph_initialize) };
    let init_ok = unsafe {
        morph_init(ability, unit, (real_node + 0x18) as *mut c_void)
    };

    let current_unit_code = unsafe { ((unit + 0x34) as *const u32).read_unaligned() };
    let convert: UnitCodeToMorphCodeFn =
        unsafe { core::mem::transmute(addresses::get().abilities.unit_code_to_morph_code) };
    let current_morph_code = unsafe { convert(current_unit_code) };
    let alternate_morph_code = unsafe { convert(morph_unit_rawcode) };

    unsafe {
        ((ability + 0x34) as *mut u32).write_unaligned(CABILITY_METAMORPHOSIS_RAWCODE);
        ((ability + 0x4C) as *mut u32).write_unaligned(0);
        ((ability + 0x50) as *mut u32).write_unaligned(0);
        ((ability + 0x54) as *mut u32).write_unaligned(synthetic_node as u32);
        ((ability + 0x58) as *mut u32).write_unaligned(synthetic_node as u32);

        ((ability + 0x190) as *mut u32).write_unaligned(current_unit_code);
        ((ability + 0x194) as *mut u32).write_unaligned(morph_unit_rawcode);
        ((ability + 0x1A0) as *mut u32).write_unaligned(current_morph_code);
        ((ability + 0x1A4) as *mut u32).write_unaligned(alternate_morph_code);
    }

    let activate_addr = unsafe { vtable_slot(ability, 0x408) };
    if activate_addr != 0 {
        let activate: MorphActivateFn = unsafe { core::mem::transmute(activate_addr) };
        unsafe {
            activate(ability);
        }
    }

    let apply: MetamorphosisApplyFn =
        unsafe { core::mem::transmute(addresses::get().abilities.metamorphosis_apply) };
    let result = unsafe { apply(ability) };

    let _leaked_synthetic = unsafe { leak_ability_data_for_buff(synthetic) };

    logging::info(&format!(
        "[spells] CBuffMetamorphosisApplyDataDebug: visual=0x{visual_rawcode:x} morph_unit=0x{morph_unit_rawcode:x} current_unit=0x{current_unit_code:x} ability=0x{ability:x} init_ok={init_ok} duration={duration} result={result}"
    ));

    result as u32
}


fn register_spell_native(name: &str, signature: &str, func: *const c_void) {
    let c_name = CString::new(name).unwrap();
    let c_sig = CString::new(signature).unwrap();

    match engines::request_plugin_native(c_name, c_sig, func) {
        Ok(_) => crate::log_native_registration!("spells: queued {name} {signature}"),
        Err(e) => logging::error(&format!("spells: failed to queue {name}: {e}")),
    }
}

pub fn register_test_natives() {
    register_spell_native(
        "CBuffMetamorphosisApplyDataDebug",
        "(IHunit;IR)I",
        cbuff_metamorphosis_apply_data_debug_native as *const c_void,
    );
}
