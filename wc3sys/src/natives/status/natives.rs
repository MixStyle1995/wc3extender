use core::ffi::c_void;
use std::ffi::CString;

use crate::{addresses, engines, logging};

const BPSE: u32 = 0x4250_5345;
const BSLO: u32 = 0x4273_6C6F;
const BEER: u32 = 0x4245_6572;
const VT_GET_RAWCODE: usize = 0x01C;
const VT_INIT: usize = 0x360;
const VT_CONFIG: usize = 0x32C;
const VT_BASE_BIND: usize = 0x328;
const VT_REFRESH: usize = 0x368;

type RawUnit = usize;
type RawEffect = usize;

type UnitHandleToCUnitFn = unsafe extern "C" fn(handle: u32) -> RawUnit;

type UnitFindEffectFn = unsafe extern "thiscall" fn(
    unit: RawUnit,
    rawcode: u32,
    alias_match: i32,
    compare_stored_rawcode: i32,
    include_normal: i32,
    include_hidden: i32,
) -> RawEffect;

type CreateBpseEffectFn = unsafe extern "C" fn() -> RawEffect;

type AttachEffectToUnitFn = unsafe extern "thiscall" fn(unit: RawUnit, effect: RawEffect) -> i32;

type EffectInitFn = unsafe extern "thiscall" fn(
    effect: RawEffect,
    target: RawUnit,
    duration: *const f32,
    source_context: RawUnit,
) -> i32;

type EffectRefreshFn = unsafe extern "thiscall" fn(
    effect: RawEffect,
    config: *const u8,
    duration: *const f32,
    source_context: RawUnit,
) -> i32;

type ApplyBsloSlowFn = unsafe extern "C" fn(
    target: RawUnit,
    config: *const u8,
    duration: *const f32,
    source_context: RawUnit,
    move_speed_delta: *const f32,
    attack_speed_delta: *const f32,
) -> i32;

type ApplyBeerRootsFn = unsafe extern "C" fn(
    target: RawUnit,
    config: *const u8,
    duration: *const f32,
    source_context: RawUnit,
    location_context: *const u8,
) -> i32;

type CreateCbufEffectFn = unsafe extern "thiscall" fn(
    out_slot: *mut RawEffect,
    a2: i32,
    a3: i32,
    a4: i32,
) -> *mut RawEffect;

type EffectConfigFn = unsafe extern "thiscall" fn(
    effect: RawEffect,
    config: *const u8,
) -> i32;

type EffectBaseBindFn = unsafe extern "thiscall" fn(
    effect: RawEffect,
    target: RawUnit,
    duration: *const f32,
    show_art: i32,
) -> i32;

// Generic-factory primitives — faithful port of sub_C01160 with rawcode parameterized.
type ObjectHashFn = unsafe extern "thiscall" fn(rawcode_ptr: *mut u32) -> u32;

type RegistryFindFn = unsafe extern "thiscall" fn(
    table_field_addr: *mut c_void,
    hash: u32,
    rawcode_ptr: *mut u32,
) -> usize;

type DescriptorBuildFn = unsafe extern "C" fn(descriptor: *mut u8, rawcode: u32, schema: u32);

type MaterializeFn = unsafe extern "C" fn(descriptor: *const u8, a: i32, b: i32) -> usize;

type RawcodeCompatFn = unsafe extern "C" fn(declared: u32, requested: u32) -> i32;

type GetRawcodeFn = unsafe extern "thiscall" fn(this: RawEffect) -> u32;

unsafe fn vfunc<T>(object: usize, offset: usize) -> T {
    let vtable = unsafe { (object as *const usize).read_unaligned() };
    let target = unsafe { ((vtable + offset) as *const usize).read_unaligned() };
    unsafe { core::mem::transmute_copy::<usize, T>(&target) }
}

unsafe fn unit_handle_to_cunit(handle: u32) -> RawUnit {
    if handle == 0 {
        return 0;
    }

    let f: UnitHandleToCUnitFn = unsafe {
        core::mem::transmute(addresses::get().unit_handle_to_cunit)
    };

    unsafe { f(handle) }
}

unsafe fn find_bpse(unit: RawUnit) -> RawEffect {
    let f: UnitFindEffectFn = unsafe {
        core::mem::transmute(addresses::get().unit_find_effect)
    };

    unsafe { f(unit, BPSE, 0, 0, 1, 1) }
}

unsafe fn find_effect_by_rawcode(unit: RawUnit, rawcode: u32) -> RawEffect {
    let f: UnitFindEffectFn = unsafe {
        core::mem::transmute(addresses::get().unit_find_effect)
    };

    unsafe { f(unit, rawcode, 0, 0, 1, 1) }
}

unsafe fn apply_bpse_stun(target: RawUnit, duration: f32, source_context: RawUnit) -> bool {
    if target == 0 || duration <= 0.0 {
        return false;
    }

    let existing = unsafe { find_bpse(target) };

    if existing != 0 {
        let refresh: EffectRefreshFn = unsafe { vfunc(existing, VT_REFRESH) };
        let config = [0u8; 32];

        unsafe {
            refresh(existing, config.as_ptr(), &duration, source_context);
        }

        return true;
    }

    let create: CreateBpseEffectFn = unsafe {
        core::mem::transmute(addresses::get().create_bpse_effect)
    };

    let effect = unsafe { create() };
    if effect == 0 {
        return false;
    }

    let init: EffectInitFn = unsafe { vfunc(effect, VT_INIT) };
    let attach: AttachEffectToUnitFn = unsafe {
        core::mem::transmute(addresses::get().attach_effect_to_unit)
    };

    unsafe {
        init(effect, target, &duration, source_context);
        attach(target, effect);
    }

    true
}

unsafe fn apply_bslo_slow(
    target: RawUnit,
    duration: f32,
    move_speed_delta: f32,
    attack_speed_delta: f32,
    source_context: RawUnit,
) -> bool {
    if target == 0 || duration <= 0.0 {
        return false;
    }

    let apply: ApplyBsloSlowFn = unsafe {
        core::mem::transmute(addresses::get().apply_bslo_slow)
    };

    let config = [0u8; 32];

    unsafe {
        apply(
            target,
            config.as_ptr(),
            &duration,
            source_context,
            &move_speed_delta,
            &attack_speed_delta,
        );
    }

    true
}


unsafe fn create_effect_with_factory(
    create_addr: usize,
) -> RawEffect {
    let create: CreateCbufEffectFn = unsafe {
        core::mem::transmute(create_addr)
    };

    let mut slot: RawEffect = 0;
    unsafe {
        create(&mut slot as *mut RawEffect, 0, 0, 0);
    }

    let effect = slot;
    if effect == 0 {
        return 0;
    }

    // Blizzard factory wrappers return through a temporary smart-pointer slot.
    // Match compiled callers: extract the raw effect, then release the temporary slot ref
    // before initializing/binding/attaching the effect.
    unsafe {
        let refcount = (effect + 4) as *mut u32;
        let cur = refcount.read_unaligned();
        refcount.write_unaligned(cur.wrapping_sub(1));
        if cur == 1 {
            let dtor: unsafe extern "thiscall" fn(RawEffect) =
                vfunc(effect, 0);
            dtor(effect);
            return 0;
        }
    }

    effect
}

unsafe fn bind_visual_cbuf(
    effect: RawEffect,
    target: RawUnit,
    duration: f32,
    config: *const u8,
) -> bool {
    if effect == 0 || target == 0 || duration <= 0.0 {
        return false;
    }

    let cfg: EffectConfigFn = unsafe { vfunc(effect, VT_CONFIG) };
    let bind: EffectBaseBindFn = unsafe { vfunc(effect, VT_BASE_BIND) };

    unsafe {
        cfg(effect, config);
        bind(effect, target, &duration, 1);
    }

    true
}

unsafe fn apply_visual_cbuf_with_factory(
    target: RawUnit,
    duration: f32,
    cbuf_rawcode: u32,
    create_addr: usize,
    config: *const u8,
) -> bool {
    if target == 0 || duration <= 0.0 {
        return false;
    }

    // Existing effect: refresh only generic config/duration/art. Do not call class gameplay slots.
    let existing = unsafe { find_effect_by_rawcode(target, cbuf_rawcode) };
    if existing != 0 {
        return unsafe { bind_visual_cbuf(existing, target, duration, config) };
    }

    // Missing effect: create the requested CBuff carrier with the known Blizzard factory wrapper,
    // bind through generic CBuff base visual/duration path, then attach to unit.
    let effect = unsafe { create_effect_with_factory(create_addr) };
    if effect == 0 {
        return false;
    }

    if !unsafe { bind_visual_cbuf(effect, target, duration, config) } {
        return false;
    }

    let attach: AttachEffectToUnitFn = unsafe {
        core::mem::transmute(addresses::get().attach_effect_to_unit)
    };

    unsafe {
        attach(target, effect);
    }

    true
}


unsafe fn apply_bslo_visual_only(
    target: RawUnit,
    duration: f32,
) -> bool {
    let config = [0u8; 32];

    unsafe {
        apply_visual_cbuf_with_factory(
            target,
            duration,
            BSLO,
            addresses::get().create_bslo_effect,
            config.as_ptr(),
        )
    }
}

unsafe fn apply_visual_buff_with_bslo_carrier(
    target: RawUnit,
    duration: f32,
    display_buff_rawcode: u32,
) -> bool {
    if display_buff_rawcode == 0 {
        return false;
    }

    // Generic visual-buff experiment:
    // Use safe/proven CBuffSlow/Bslo as carrier, but pass the requested Object Editor
    // buff rawcode through config[0]. Generic config ingest (+0x32C / 0x6F6900)
    // stores config[0] at effect+0x34, which ShowBonusArt uses for art/icon metadata.
    let mut config = [0u8; 32];
    config[0..4].copy_from_slice(&display_buff_rawcode.to_le_bytes());

    unsafe {
        apply_visual_cbuf_with_factory(
            target,
            duration,
            BSLO,
            addresses::get().create_bslo_effect,
            config.as_ptr(),
        )
    }
}



unsafe fn apply_beer_roots(
    target: RawUnit,
    duration: f32,
    source_context: RawUnit,
) -> bool {
    if target == 0 || duration <= 0.0 {
        return false;
    }

    // Blizzard Entangling Roots applier: sub_C48FB0(target, config, duration, source, location_context).
    // This mirrors the real CAbilityEntanglingRoots path and applies gameplay rooting.
    // The final argument is normally a small stack context produced by the ability from sub_46ED70.
    // For unit-native use we provide a zero local context, matching the "no extra location context" case.
    let apply: ApplyBeerRootsFn = unsafe {
        core::mem::transmute(addresses::get().apply_beer_roots)
    };

    let config = [0u8; 32];
    let location_context = [0u8; 4];

    unsafe {
        apply(
            target,
            config.as_ptr(),
            &duration,
            source_context,
            location_context.as_ptr(),
        );
    }

    true
}

unsafe fn apply_beer_visual_only(
    target: RawUnit,
    duration: f32,
) -> bool {
    let config = [0u8; 32];

    unsafe {
        apply_visual_cbuf_with_factory(
            target,
            duration,
            BEER,
            addresses::get().create_beer_effect,
            config.as_ptr(),
        )
    }
}

// =====================================================================================
// Generic buff factory — faithful port of sub_C01160 with rawcode parameterized.
//
// The engine factory at 0xC01160 hardcodes the rawcode via sub_6F4BB0() which always
// returns BPSE. The neighbors at 0xC01260 / 0xC01360 are byte-for-byte identical
// except the rawcode is a literal immediate. This Rust function does the same job but
// takes the rawcode at runtime — usable for ANY 4CC the engine has a factory for
// (BSTN, Bslo, Bdet, etc.), including visual-only buffs.
//
// Disasm anchor: 0xC01160.
// =====================================================================================
unsafe fn create_effect_by_rawcode(rawcode: u32) -> RawEffect {
    let addrs = addresses::get();
    let mut rc = rawcode;

    // Step 1: hash the rawcode.
    //   sub_95C860 — thiscall, ecx = &rawcode_local, returns u32 hash.
    let hash_fn: ObjectHashFn = unsafe { core::mem::transmute(addrs.object_hash) };
    let hash = unsafe { hash_fn(&mut rc) };

    // Step 2: registry lookup.
    //   sub_429B90 — thiscall, ecx = &(registry_root + 0x0C), 2 stack args.
    //   The asm is `lea ecx, [esi+0Ch]` where esi = dword_115E9EC, so ecx is the
    //   ADDRESS of the field, not the value at it.
    let table_field = (addrs.effect_registry_root + 0x0C) as *mut c_void;
    let find_fn: RegistryFindFn = unsafe { core::mem::transmute(addrs.effect_registry_find) };
    let entry = unsafe { find_fn(table_field, hash, &mut rc) };
    if entry == 0 {
        return 0;
    }

    // Step 3: read schema pointer at entry + 0x78.
    let schema = unsafe { *((entry + 0x78) as *const u32) };

    // Step 4: build the descriptor + write the trailing var_C field.
    //   sub_7F2650(&descriptor, rawcode, schema) — cdecl, 3 args.
    //   The engine stack frame puts v15[36] adjacent to v16, and `sub_8A0660` reads
    //   both. We allocate a combined 40-byte buffer: descriptor [0..0x24], then
    //   write `-2` (the value of `(a4 != 2) - 2` when a4 == 0) at offset 0x24.
    let mut descriptor = [0u8; 0x30];
    let build_fn: DescriptorBuildFn = unsafe { core::mem::transmute(addrs.effect_descriptor_build) };
    unsafe { build_fn(descriptor.as_mut_ptr(), rawcode, schema) };
    unsafe { *(descriptor.as_mut_ptr().add(0x24) as *mut i32) = -2_i32 };

    // Step 5: materialize the effect object.
    //   sub_8A0660(&descriptor, 1, 1) — cdecl, 3 args.
    let materialize_fn: MaterializeFn = unsafe { core::mem::transmute(addrs.effect_materialize) };
    let holder = unsafe { materialize_fn(descriptor.as_ptr(), 1, 1) };
    if holder == 0 {
        return 0;
    }

    // Step 6: extract the effect object at holder + 0x54.
    let obj = unsafe { *((holder + 0x54) as *const usize) };
    if obj == 0 {
        return 0;
    }

    // Step 7: rawcode-compat check.
    //   Calls vtable + 0x1C on the effect (thiscall, no stack args) to read its
    //   declared rawcode, then sub_7F55F0(declared, requested) — cdecl, 2 args.
    let vtbl = unsafe { (obj as *const usize).read_unaligned() };
    let get_rc_fn: GetRawcodeFn = unsafe {
        core::mem::transmute(((vtbl + VT_GET_RAWCODE) as *const usize).read_unaligned())
    };
    let declared = unsafe { get_rc_fn(obj) };
    let compat_fn: RawcodeCompatFn = unsafe { core::mem::transmute(addrs.rawcode_compatible) };
    if unsafe { compat_fn(declared, rawcode) } == 0 {
        return 0;
    }

    // Step 8: acquire refcount.
    //   Slot release is a no-op (our slot starts at 0). Only the acquire half runs:
    //   `++obj[1]`. This matches the refcount=1 transfer that sub_C00DF0 produces
    //   for BPSE today.
    let refcount_ptr = (obj + 4) as *mut u32;
    unsafe {
        let cur = refcount_ptr.read_unaligned();
        refcount_ptr.write_unaligned(cur.wrapping_add(1));
    }

    obj
}

// =====================================================================================
// Generic buff applier — single entry point for any 4CC.
//
// Mirrors apply_bpse_stun's flow (find-or-create + init + attach) but uses the generic
// factory above, so the rawcode is supplied at runtime instead of being baked in.
// =====================================================================================
unsafe fn apply_buff_to_unit(
    target: RawUnit,
    rawcode: u32,
    duration: f32,
    source_context: RawUnit,
) -> bool {
    if target == 0 || duration <= 0.0 {
        return false;
    }

    let existing = unsafe { find_effect_by_rawcode(target, rawcode) };

    if existing != 0 {
        let refresh: EffectRefreshFn = unsafe { vfunc(existing, VT_REFRESH) };
        let config = [0u8; 32];

        unsafe {
            refresh(existing, config.as_ptr(), &duration, source_context);
        }

        return true;
    }

    let effect = unsafe { create_effect_by_rawcode(rawcode) };
    if effect == 0 {
        return false;
    }

    let init: EffectInitFn = unsafe { vfunc(effect, VT_INIT) };
    let attach: AttachEffectToUnitFn = unsafe {
        core::mem::transmute(addresses::get().attach_effect_to_unit)
    };

    unsafe {
        init(effect, target, &duration, source_context);
        attach(target, effect);
    }

    true
}

pub unsafe extern "C" fn ce_stun_unit(target_handle: u32, duration_ptr: u32, source_handle: u32) -> u32 {
    if duration_ptr == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };
    let source = unsafe { unit_handle_to_cunit(source_handle) };

    if unsafe { apply_bpse_stun(target, duration, source) } {
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn ce_slow_unit(
    target_handle: u32,
    duration_ptr: u32,
    move_speed_delta_ptr: u32,
    attack_speed_delta_ptr: u32,
    source_handle: u32,
) -> u32 {
    if duration_ptr == 0 || move_speed_delta_ptr == 0 || attack_speed_delta_ptr == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let move_speed_delta = unsafe { (move_speed_delta_ptr as *const f32).read_unaligned() };
    let attack_speed_delta = unsafe { (attack_speed_delta_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };
    let source = unsafe { unit_handle_to_cunit(source_handle) };

    if unsafe { apply_bslo_slow(target, duration, move_speed_delta, attack_speed_delta, source) } {
        1
    } else {
        0
    }
}


pub unsafe extern "C" fn ce_visual_slow_unit(
    target_handle: u32,
    duration_ptr: u32,
    _source_handle: u32,
) -> u32 {
    if duration_ptr == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };

    if unsafe { apply_bslo_visual_only(target, duration) } {
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn ce_visual_buff_unit(
    target_handle: u32,
    buff_rawcode: u32,
    duration_ptr: u32,
    _source_handle: u32,
) -> u32 {
    if duration_ptr == 0 || buff_rawcode == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };

    if unsafe { apply_visual_buff_with_bslo_carrier(target, duration, buff_rawcode) } {
        1
    } else {
        0
    }
}


pub unsafe extern "C" fn ce_roots_unit(
    target_handle: u32,
    duration_ptr: u32,
    source_handle: u32,
) -> u32 {
    if duration_ptr == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };
    let source = unsafe { unit_handle_to_cunit(source_handle) };

    if unsafe { apply_beer_roots(target, duration, source) } {
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn ce_visual_roots_unit(
    target_handle: u32,
    duration_ptr: u32,
    _source_handle: u32,
) -> u32 {
    if duration_ptr == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };

    if unsafe { apply_beer_visual_only(target, duration) } {
        1
    } else {
        0
    }
}

pub unsafe extern "C" fn ce_apply_buff_to_unit(
    target_handle: u32,
    rawcode: u32,
    duration_ptr: u32,
    source_handle: u32,
) -> u32 {
    if duration_ptr == 0 {
        return 0;
    }

    let duration = unsafe { (duration_ptr as *const f32).read_unaligned() };
    let target = unsafe { unit_handle_to_cunit(target_handle) };
    let source = unsafe { unit_handle_to_cunit(source_handle) };

    if unsafe { apply_buff_to_unit(target, rawcode, duration, source) } {
        1
    } else {
        0
    }
}

pub fn register_custom_natives() {
    let natives = [
        ("CeStunUnit", "(Hunit;RHunit;)B", ce_stun_unit as *const c_void),
        ("CeSlowUnit", "(Hunit;RRRHunit;)B", ce_slow_unit as *const c_void),
        ("CeVisualSlowUnit", "(Hunit;RHunit;)B", ce_visual_slow_unit as *const c_void),
        ("CeVisualBuffUnit", "(Hunit;IRHunit;)B", ce_visual_buff_unit as *const c_void),
        ("CeRootsUnit", "(Hunit;RHunit;)B", ce_roots_unit as *const c_void),
        ("CeVisualRootsUnit", "(Hunit;RHunit;)B", ce_visual_roots_unit as *const c_void),
        ("CeApplyBuffToUnit", "(Hunit;IRHunit;)B", ce_apply_buff_to_unit as *const c_void),
    ];

    for (name, sig, func) in natives {
        let c_name = CString::new(name).unwrap();
        let c_sig = CString::new(sig).unwrap();

        match engines::request_plugin_native(c_name, c_sig, func) {
            Ok(_) => logging::info(&format!("status: queued {} for registration", name)),
            Err(e) => logging::error(&format!("status: failed to queue {}: {}", name, e)),
        }
    }
}