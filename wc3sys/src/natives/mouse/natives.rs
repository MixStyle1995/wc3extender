use core::ffi::c_void;
use std::ffi::CString;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::addresses;
use crate::engines;
use crate::hooks as hook_manager;
use crate::logging;

type CGameUIGetOrCreateFn = unsafe extern "C" fn(i32, i32) -> *mut c_void;

pub const WORLD_FRAME_SET_CURSOR_MODE_HOOK: &str = "world_frame_set_cursor_mode";
pub const OBJECT_SELECTION_SCALE_GET_HOOK: &str = "object_selection_scale_get";
pub const SELECTION_CIRCLE_RADIUS_GET_HOOK: &str = "selection_circle_radius_get";
type CursorFrameGetOrCreateFn = unsafe extern "thiscall" fn(ui: usize, index: i32) -> usize;
type SpriteFrameGetSpriteFn = unsafe extern "thiscall" fn(frame: usize) -> usize;

type ObjectSelectionScaleGetFn = unsafe extern "C" fn(object_id: u32) -> f32;
type SelectionCircleRadiusGetFn = unsafe extern "thiscall" fn(this: usize) -> f32;

static CURSOR_SELECTION_SCALE_OVERRIDE_BITS: AtomicU32 = AtomicU32::new(0);
static CURSOR_SELECTION_SCALE_OVERRIDE_OBJECT: AtomicU32 = AtomicU32::new(0);


unsafe fn get_world_frame() -> usize {
    let get_or_create: CGameUIGetOrCreateFn =
        unsafe { core::mem::transmute(addresses::get().c_game_ui_get_or_create) };
    let ui = unsafe { get_or_create(1, 0) as usize };
    if ui == 0 {
        return 0;
    }
    // CWorldFrameWar3 is located at offset 0x3FC of CGameUI
    unsafe { ((ui + 0x3FC) as *const usize).read_unaligned() }
}

type WorldFrameSetCursorModeFn = unsafe extern "thiscall" fn(
    world_frame: usize,
    cursor_mode: i32,
    art_path: *const i8,
    flag: i32,
) -> i32;

unsafe fn world_frame_set_cursor_mode(
    world_frame: usize,
    cursor_mode: i32,
    art_path: *const i8,
    flag: i32,
) -> i32 {
    let target = addresses::get().world_frame_set_cursor_mode;
    let addr = hook_manager::trampoline(target).unwrap_or(target);
    let f: WorldFrameSetCursorModeFn = unsafe { core::mem::transmute(addr) };
    unsafe { f(world_frame, cursor_mode, art_path, flag) }
}

unsafe fn cursor_sprite_ptr() -> usize {
    let get_or_create: CGameUIGetOrCreateFn =
        unsafe { core::mem::transmute(addresses::get().c_game_ui_get_or_create) };
    let ui = unsafe { get_or_create(1, 0) as usize };
    if ui == 0 {
        return 0;
    }

    let get_cursor_frame: CursorFrameGetOrCreateFn =
        unsafe { core::mem::transmute(addresses::get().cursor_frame_get_or_create) };
    let cursor_frame = unsafe { get_cursor_frame(ui, 0) };
    if cursor_frame == 0 {
        return 0;
    }

    let get_sprite: SpriteFrameGetSpriteFn =
        unsafe { core::mem::transmute(addresses::get().c_sprite_frame_get_sprite) };
    unsafe { get_sprite(cursor_frame) }
}

unsafe fn cstr_lossy(ptr: *const i8) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

unsafe fn cursor_field_i32(world_frame: usize, offset: usize) -> i32 {
    if world_frame == 0 {
        return 0;
    }
    unsafe { ((world_frame + offset) as *const i32).read_unaligned() }
}

pub unsafe extern "thiscall" fn world_frame_set_cursor_mode_hook(
    world_frame: usize,
    cursor_mode: i32,
    art_path: *const i8,
    flag: i32,
) -> i32 {
    let before_mode = unsafe { cursor_field_i32(world_frame, 0x1B8) };
    let before_busy = unsafe { cursor_field_i32(world_frame, 0x1CC) };
    let sprite_before = unsafe { cursor_sprite_ptr() };
    let art = unsafe { cstr_lossy(art_path) };

    logging::info(&format!(
        "[cursor/hook] enter wf=0x{world_frame:x} mode={cursor_mode} art_ptr=0x{:x} art='{}' flag={} before_mode={} before_busy={} sprite=0x{:x}",
        art_path as usize,
        art,
        flag,
        before_mode,
        before_busy,
        sprite_before
    ));

    let result = unsafe {
        let tramp = hook_manager::trampoline(addresses::get().world_frame_set_cursor_mode)
            .expect("world_frame_set_cursor_mode trampoline missing");
        let original: WorldFrameSetCursorModeFn = core::mem::transmute(tramp);
        original(world_frame, cursor_mode, art_path, flag)
    };

    let after_mode = unsafe { cursor_field_i32(world_frame, 0x1B8) };
    let after_busy = unsafe { cursor_field_i32(world_frame, 0x1CC) };
    let sprite_after = unsafe { cursor_sprite_ptr() };

    logging::info(&format!(
        "[cursor/hook] exit wf=0x{world_frame:x} mode={cursor_mode} flag={flag} ret={result} after_mode={} after_busy={} sprite=0x{:x}",
        after_mode,
        after_busy,
        sprite_after
    ));

    result
}

pub unsafe extern "C" fn object_selection_scale_get_hook(object_id: u32) -> f32 {
    let bits = CURSOR_SELECTION_SCALE_OVERRIDE_BITS.load(Ordering::Relaxed);
    let filter = CURSOR_SELECTION_SCALE_OVERRIDE_OBJECT.load(Ordering::Relaxed);

    if bits != 0 && (filter == 0 || filter == object_id) {
        let scale = f32::from_bits(bits);
        logging::info(&format!(
            "[cursor/scale] override object_id=0x{object_id:08x} scale={scale}"
        ));
        return scale;
    }

    let tramp = hook_manager::trampoline(addresses::get().object_selection_scale_get)
        .expect("object_selection_scale_get trampoline missing");
    let original: ObjectSelectionScaleGetFn = unsafe { core::mem::transmute(tramp) };
    unsafe { original(object_id) }
}

pub unsafe extern "thiscall" fn selection_circle_radius_get_hook(this: usize) -> f32 {
    let bits = CURSOR_SELECTION_SCALE_OVERRIDE_BITS.load(Ordering::Relaxed);
    if bits != 0 {
        let scale = f32::from_bits(bits);
        logging::info(&format!(
            "[cursor/scale] override radius this=0x{this:x} scale={scale}"
        ));
        return scale;
    }

    let tramp = hook_manager::trampoline(addresses::get().selection_circle_radius_get)
        .expect("selection_circle_radius_get trampoline missing");
    let original: SelectionCircleRadiusGetFn = unsafe { core::mem::transmute(tramp) };
    unsafe { original(this) }
}


pub fn install_hook() -> crate::error::Result<()> {
    hook_manager::install(wc3::InlineHook::new(
        WORLD_FRAME_SET_CURSOR_MODE_HOOK,
        addresses::get().world_frame_set_cursor_mode,
        world_frame_set_cursor_mode_hook as *const () as usize,
    ))?;
    logging::info("[cursor/hook] world_frame_set_cursor_mode observer installed");

    hook_manager::install(wc3::InlineHook::new(
        OBJECT_SELECTION_SCALE_GET_HOOK,
        addresses::get().object_selection_scale_get,
        object_selection_scale_get_hook as *const () as usize,
    ))?;
    logging::info("[cursor/scale] object_selection_scale_get override hook installed");

    hook_manager::install(wc3::InlineHook::new(
        SELECTION_CIRCLE_RADIUS_GET_HOOK,
        addresses::get().selection_circle_radius_get,
        selection_circle_radius_get_hook as *const () as usize,
    ))?;
    logging::info("[cursor/scale] selection_circle_radius_get override hook installed");

    Ok(())
}

type ObjectHashFn = unsafe extern "thiscall" fn(key: *mut u32) -> u32;
type ObjectDataFindFn = unsafe extern "thiscall" fn(table: usize, hash: u32, key: *const u32) -> usize;
type ObjectDataCreateFn = unsafe extern "thiscall" fn(table: usize, hash: u32, a3: i32, a4: i32) -> usize;
type ObjectDataInitFn = unsafe extern "thiscall" fn(entry: usize, object_id: u32);

const OBJECT_SELECTION_SCALE: usize = 0x54;

unsafe fn object_data_entry(object_id: u32) -> Option<usize> {
    let addrs = addresses::get();
    let mut key = object_id;

    let hash_fn: ObjectHashFn = unsafe { core::mem::transmute(addrs.object_hash) };
    let hash = unsafe { hash_fn(&mut key as *mut u32) };

    let find: ObjectDataFindFn = unsafe { core::mem::transmute(addrs.object_data_find) };
    let existing = unsafe { find(addrs.object_data_table, hash, &key as *const u32) };
    if existing != 0 {
        return Some(existing);
    }

    let create: ObjectDataCreateFn = unsafe { core::mem::transmute(addrs.object_data_create) };
    let entry = unsafe { create(addrs.object_data_table, hash, 0, 0) };
    if entry == 0 {
        return None;
    }

    unsafe {
        ((entry + 0x04) as *mut u32).write_unaligned(hash);
        ((entry + 0x18) as *mut u32).write_unaligned(key);

        let vtable = (entry as *const usize).read_unaligned();
        if vtable == 0 {
            return None;
        }

        let init_ptr = (vtable as *const usize).read_unaligned();
        if init_ptr == 0 {
            return None;
        }

        let init: ObjectDataInitFn = core::mem::transmute(init_ptr);
        init(entry, object_id);
    }

    Some(entry)
}

unsafe fn object_selection_scale(object_id: u32) -> Option<f32> {
    let entry = unsafe { object_data_entry(object_id)? };
    Some(unsafe { ((entry + OBJECT_SELECTION_SCALE) as *const f32).read_unaligned() })
}

unsafe fn set_object_selection_scale(object_id: u32, scale: f32) -> bool {
    let Some(entry) = (unsafe { object_data_entry(object_id) }) else {
        return false;
    };
    unsafe { ((entry + OBJECT_SELECTION_SCALE) as *mut f32).write_unaligned(scale) };
    true
}

pub unsafe extern "C" fn ce_set_cursor_mode(mode: u32, art_handle: u32, flag: u32) {
    let wf = unsafe { get_world_frame() };
    if wf == 0 {
        logging::warn("[mouse] CeSetCursorMode: null world frame");
        return;
    }

    let art = if art_handle == 0 {
        None
    } else {
        crate::jass::raw::native_string_to_str(art_handle as usize)
    };

    let c_art = art
        .as_deref()
        .and_then(|s| CString::new(s).ok());

    let art_ptr = c_art
        .as_ref()
        .map(|s| s.as_ptr())
        .unwrap_or(core::ptr::null());

    let result = unsafe {
        world_frame_set_cursor_mode(wf, mode as i32, art_ptr, flag as i32)
    };

    logging::info(&format!(
        "[mouse] CeSetCursorMode mode={} art='{}' flag={} ret={}",
        mode,
        c_art.as_ref().map(|s| s.to_string_lossy()).unwrap_or_default(),
        flag,
        result
    ));
}

pub unsafe extern "C" fn ce_set_cursor_selection_scale(scale_ptr: u32) {
    if scale_ptr == 0 {
        return;
    }

    let scale = unsafe { (scale_ptr as *const f32).read_unaligned() };
    CURSOR_SELECTION_SCALE_OVERRIDE_BITS.store(scale.to_bits(), Ordering::Relaxed);
    logging::info(&format!("[cursor/scale] enabled global scale override scale={scale}"));
}

pub unsafe extern "C" fn ce_set_cursor_selection_scale_object(object_id: u32) {
    CURSOR_SELECTION_SCALE_OVERRIDE_OBJECT.store(object_id, Ordering::Relaxed);
    logging::info(&format!(
        "[cursor/scale] override object filter set to 0x{object_id:08x}"
    ));
}

pub unsafe extern "C" fn ce_clear_cursor_selection_scale() {
    CURSOR_SELECTION_SCALE_OVERRIDE_BITS.store(0, Ordering::Relaxed);
    CURSOR_SELECTION_SCALE_OVERRIDE_OBJECT.store(0, Ordering::Relaxed);
    logging::info("[cursor/scale] cleared scale override");
}

pub unsafe extern "C" fn ce_get_object_selection_scale(object_id: u32) -> u32 {
    unsafe { object_selection_scale(object_id).unwrap_or(0.0).to_bits() }
}

pub unsafe extern "C" fn ce_set_object_selection_scale(object_id: u32, scale_ptr: u32) {
    if scale_ptr == 0 {
        return;
    }

    let scale = unsafe { (scale_ptr as *const f32).read_unaligned() };
    if !unsafe { set_object_selection_scale(object_id, scale) } {
        logging::warn(&format!(
            "[mouse] CeSetObjectSelectionScale failed object_id=0x{object_id:08x} scale={scale}"
        ));
    }
}

pub fn register_custom_natives() {
    let natives = [
        ("CeSetCursorMode", "(ISI)V", ce_set_cursor_mode as *const c_void),
        ("CeGetObjectSelectionScale", "(I)R", ce_get_object_selection_scale as *const c_void),
        ("CeSetObjectSelectionScale", "(IR)V", ce_set_object_selection_scale as *const c_void),
        ("CeSetCursorSelectionScale", "(R)V", ce_set_cursor_selection_scale as *const c_void),
        ("CeSetCursorSelectionScaleObject", "(I)V", ce_set_cursor_selection_scale_object as *const c_void),
        ("CeClearCursorSelectionScale", "()V", ce_clear_cursor_selection_scale as *const c_void),
    ];

    for (name, sig, func) in natives {
        let c_name = CString::new(name).unwrap();
        let c_sig = CString::new(sig).unwrap();
        match engines::request_plugin_native(c_name, c_sig, func) {
            Ok(_) => logging::info(&format!("mouse: queued {} for registration", name)),
            Err(e) => logging::error(&format!("mouse: failed to queue {}: {}", name, e)),
        }
    }
}
