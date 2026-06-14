use core::ffi::c_void;
use std::ffi::CString;

use crate::addresses;
use crate::engines;
use crate::hooks as hook_manager;
use crate::logging;

use super::frame_registry;
use super::frame_type::FrameType;
use super::offsets;
use super::structs::CFrame;

type CGameUIGetOrCreateFn = unsafe extern "C" fn(i32, i32) -> *mut CFrame;
type CBackdropSetTextureFn = unsafe extern "thiscall" fn(*mut c_void, u32, u32, u32, u32, u32);
type CSimpleTextureSetTextureFn = unsafe extern "thiscall" fn(usize, u32, u32);
type CSimpleStatusBarSetTextureFn = unsafe extern "thiscall" fn(usize, u32, u32);
type StringHashNodeGrowFn = unsafe extern "thiscall" fn(usize, i32);
type BaseFrameHashNodeGrowFn = unsafe extern "thiscall" fn(usize, i32);
type FdFileReadFn = unsafe extern "C" fn(*const i8, usize, usize, usize) -> i32;
type CLayoutFrameSetAllPointsFn = unsafe extern "thiscall" fn(usize, usize, i32) -> i32;
type CLayoutFrameClearAllPointsFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CLayoutFrameSetPointAbsFn = unsafe extern "thiscall" fn(usize, i32, f32, f32, i32) -> i32;
type CLayoutFrameSetPointFn = unsafe extern "thiscall" fn(usize, i32, usize, i32, f32, f32, i32) -> i32;
type CLayerShowHideFn = unsafe extern "thiscall" fn(usize);
type CFrameDestroyFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CLayerSetAlphaFn = unsafe extern "thiscall" fn(usize, i32, i32);
type CSimpleFrameSetAlphaFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CTextFrameSetTextColorFn = unsafe extern "thiscall" fn(usize, *const u32) -> i32;
type CSimpleFontStringSetTextFn = unsafe extern "thiscall" fn(usize, *const i8) -> i32;
type CTextFrameSetTextFn = unsafe extern "thiscall" fn(usize, *const i8) -> i32;
type CObserverRegisterEventFn = unsafe extern "thiscall" fn(*mut c_void, u32, u32, *mut c_void) -> i32;
type SimpleVTableShowHideFn = unsafe extern "thiscall" fn(usize);
type FrameDefCreateSimpleFrameFn = unsafe extern "C" fn(*const i8, *mut CFrame, i32) -> *mut CFrame;
type FrameRegistryGetEntryFn = unsafe extern "C" fn(*const i8, i32) -> usize;
type CLayoutFrameUpdateFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CLayoutFrameCageMouseFn = unsafe extern "thiscall" fn(usize, bool) -> i32;
type CSimpleFrameSetLayoutScaleFn = unsafe extern "thiscall" fn(usize, f32) -> i32;
type CSimpleFontStringSetLayoutScaleFn = unsafe extern "thiscall" fn(usize, f32) -> i32;
type CSimpleGlueFrameSetLayoutScaleFn = unsafe extern "thiscall" fn(usize, f32) -> i32;
type CSpriteFrameSetLayoutScaleFn = unsafe extern "thiscall" fn(usize, f32) -> i32;
type CControlDispatchClickFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CControlEnableFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CControlCheckStateFn = unsafe extern "thiscall" fn(usize, i32) -> bool;
type CSimpleButtonSetEnableFn = unsafe extern "thiscall" fn(usize, bool) -> i32;
type CTextFrameSetJustificationFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CTextFrameUpdateControlFn = unsafe extern "thiscall" fn(usize) -> i32;
type CEditBoxSetFocusFn = unsafe extern "thiscall" fn(usize, bool) -> bool;
type CFrameSetFontFn = unsafe extern "C" fn(usize, *const i8, f32, i32) -> i32;
type CEditBoxSetFontFn = unsafe extern "thiscall" fn(usize, *const i8, f32, i32) -> i32;
type CEditBoxSetTextSizeLimitFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CSimpleFrameSetFontFn = unsafe extern "thiscall" fn(usize, *const i8, f32, i32) -> i32;
type CSliderSetValueFn = unsafe extern "thiscall" fn(usize, f32) -> i32;
type CSimpleStatusBarSetValueFn = unsafe extern "thiscall" fn(usize, f32) -> bool;
type CSimpleStatusBarSetMinMaxValueFn = unsafe extern "thiscall" fn(usize, f32, f32) -> bool;
type CStatusBarSetArtFn = unsafe extern "thiscall" fn(usize, *const i8, i32) -> i32;
type CStatusBarSetValueFn = unsafe extern "thiscall" fn(usize, f32, i32) -> bool;
type CStatusBarSetMinMaxValueFn = unsafe extern "thiscall" fn(usize, f32, f32) -> bool;
type CModelFrameAddModelFn = unsafe extern "thiscall" fn(usize, *const i8, i32) -> i32;
type CSpriteFrameSetArtFn = unsafe extern "thiscall" fn(usize, *const i8, i32, i32) -> i32;
type CSpriteFrameGetSpriteFn = unsafe extern "thiscall" fn(usize) -> usize;
type CSpriteUberSetAnimationFn = unsafe extern "thiscall" fn(usize, *const i32, i32, i32) -> i32;
type SimpleVTableVoidFn = unsafe extern "thiscall" fn(usize);




fn get_game_ui() -> *mut CFrame {
    unsafe {
        let get_or_create: CGameUIGetOrCreateFn =
            core::mem::transmute(addresses::get().frames.c_game_ui_get_or_create);
        get_or_create(1, 0)
    }
}

fn native_string_cstring(native_string: u32) -> CString {
    CString::new(
        crate::jass::raw::native_string_to_str(native_string as usize).unwrap_or_default(),
    )
    .unwrap_or_default()
}

#[inline]
unsafe fn read_usize(addr: usize) -> usize {
    unsafe { (addr as *const usize).read_unaligned() }
}

#[inline]
unsafe fn read_i32(addr: usize) -> i32 {
    unsafe { (addr as *const i32).read_unaligned() }
}

#[inline]
unsafe fn write_i32(addr: usize, value: i32) {
    unsafe { (addr as *mut i32).write_unaligned(value) };
}

#[inline]
unsafe fn read_u32(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_unaligned() }
}

#[inline]
unsafe fn read_f32(addr: usize) -> f32 {
    unsafe { (addr as *const f32).read_unaligned() }
}

#[inline]
unsafe fn write_i32_raw(addr: usize, value: i32) {
    unsafe { (addr as *mut i32).write_unaligned(value) };
}

#[inline]
unsafe fn write_f32(addr: usize, value: f32) {
    unsafe { (addr as *mut f32).write_unaligned(value) };
}

const C_FRAME_LAYER_STYLE: usize = 0x0C;
const C_FRAME_NAME_LAYOUT: usize = 0x7C;
const C_FRAME_NAME_NORMAL: usize = 0x16C;
const C_FRAME_CREATE_CONTEXT: usize = 0x170;
const C_CONTROL_STYLE: usize = 0x1DC;
const C_SIMPLE_NAME: usize = 0x80;
const C_SIMPLE_CREATE_CONTEXT: usize = 0x84;
const C_SIMPLE_FONT_NAME: usize = 0x88;
const C_SIMPLE_FONT_CREATE_CONTEXT: usize = 0x8C;

const CGAME_UI_CURSOR_FRAME: usize = 0x178;
const CGAME_UI_WORLD_FRAME: usize = 0x3FC;
const CGAME_UI_MINIMAP: usize = 0x400;
const CGAME_UI_INFO_BAR: usize = 0x404;
const CGAME_UI_COMMAND_BAR: usize = 0x408;
const CGAME_UI_RESOURCE_BAR: usize = 0x40C;
const CGAME_UI_UPPER_BUTTON_BAR: usize = 0x410;
const CGAME_UI_SIMPLE_FRAME: usize = 0x414;
const CGAME_UI_HERO_BAR: usize = 0x41C;
const CGAME_UI_PEON_BAR: usize = 0x420;
const CGAME_UI_ERROR_MESSAGE: usize = 0x424;
const CGAME_UI_UNIT_MESSAGE: usize = 0x428;
const CGAME_UI_CHAT_MESSAGE: usize = 0x42C;
const CGAME_UI_TOP_MESSAGE: usize = 0x430;
const CGAME_UI_PORTRAIT_BUTTON: usize = 0x434;
const CGAME_UI_TIME_OF_DAY_INDICATOR: usize = 0x438;
const CGAME_UI_CINEMATIC_PANEL: usize = 0x440;
const CGAME_UI_MINIMAP_BUTTON_SIGNAL: usize = 0x448;
const CGAME_UI_MINIMAP_BUTTON_TERRAIN: usize = 0x44C;
const CGAME_UI_MINIMAP_BUTTON_ALLY: usize = 0x450;
const CGAME_UI_MINIMAP_BUTTON_CREEP: usize = 0x454;
const CGAME_UI_MINIMAP_BUTTON_FORMATION: usize = 0x458;

const CINFO_BAR_UNIT_DETAIL: usize = 0x134;
const CINFO_BAR_CARGO_DETAIL: usize = 0x13C;
const CINFO_BAR_GROUP: usize = 0x140;
const CINFO_BAR_INVENTORY_BAR: usize = 0x14C;
const CINFO_BAR_INVENTORY_COVER: usize = 0x150;
const CINFO_BAR_INVENTORY_TEXT: usize = 0x154;

const CCOMMAND_BAR_BUTTONS: usize = 0x154;
const CINVENTORY_BAR_BUTTONS: usize = 0x130;
const CHERO_BAR_HERO_BUTTONS: usize = 0x154;
const CHERO_BAR_BUTTON_BAR1: usize = 0x1D4;
const CHERO_BAR_BUTTON_BAR2: usize = 0x1D8;
const CHERO_BAR_BUTTON_SPRITE: usize = 0x1E0;
const CPEON_BAR_PEON_BUTTON: usize = 0x138;
const CINFO_PANEL_UNIT_DETAIL_BUFF_BAR: usize = 0x170;
const CBUFF_BAR_BUFF_BAR_LABEL: usize = 0x160;
const CCARGO_BUTTON_BAR1: usize = 0x138;
const CCARGO_BUTTON_BAR2: usize = 0x13C;
const CGROUP_BUTTON_BAR1: usize = 0x13C;
const CGROUP_BUTTON_BAR2: usize = 0x140;
const CPORTRAIT_BUTTON_TEXT_LIFE: usize = 0x258;
const CPORTRAIT_BUTTON_TEXT_MANA: usize = 0x25C;
const CUBER_TIP_STRING0: usize = 0x12C;
const CUBER_TIP_ICON0: usize = 0x13C;
const CUBER_TIP_STRING6: usize = 0x184;
const CRESOURCE_BAR_FPS: usize = 0x14C;
const CRESOURCE_BAR_APM: usize = 0x158;
const CRESOURCE_BAR_PING: usize = 0x160;

const C_FRAME_COLOR: usize = 0xB4;
const C_FRAME_VISIBILITY_FLAG: usize = 0xB8;
const C_LAYOUT_SCALE: usize = 0x60;
const C_LAYOUT_MOUSE_CAGED: usize = 0x64;
const C_SIMPLE_COLOR: usize = 0x8C;
const C_SIMPLE_VISIBILITY_FLAG: usize = 0x98;
const C_SIMPLE_BUTTON_ENABLED: usize = 0x13C;
const C_SIMPLE_FONT_ALPHA: usize = 0x70;
const C_SIMPLE_FONT_PARENT: usize = 0x78;
const CUPPER_BUTTON_BAR_MENU: usize = 0x134;
const CUPPER_BUTTON_BAR_ALLIES: usize = 0x138;
const CUPPER_BUTTON_BAR_LOG: usize = 0x13C;
const CUPPER_BUTTON_BAR_QUESTS: usize = 0x164;

const CGAME_UI_SIMPLE_CONSOLE_2: usize = 0x468;
const CCOMMAND_BUTTON_DATA: usize = 0x194;
const CCOMMAND_BUTTON_DATA_A: usize = 0x04;
const CCOMMAND_BUTTON_DATA_B: usize = 0x08;
const CCOMMAND_BUTTON_DATA_C: usize = 0x0C;

unsafe fn ui_field(ui: usize, offset: usize) -> usize {
    unsafe { read_usize(ui + offset) }
}

unsafe fn info_bar_field(ui: usize, offset: usize) -> usize {
    let info_bar = unsafe { ui_field(ui, CGAME_UI_INFO_BAR) };
    if info_bar == 0 { 0 } else { unsafe { read_usize(info_bar + offset) } }
}

unsafe fn c_frame_from_name(name: *const i8, create_context: i32) -> usize {
    if name.is_null() {
        return 0;
    }

    let addrs = addresses::get();
    let normal: FrameRegistryGetEntryFn = unsafe { core::mem::transmute(addrs.frames.c_frame_registry_get_entry) };
    let frame = unsafe { normal(name, create_context) };
    if frame != 0 {
        return frame;
    }

    for addr in [
        addrs.frames.c_simple_frame_registry_get_entry_a,
        addrs.frames.c_simple_frame_registry_get_entry_b,
        addrs.frames.c_simple_frame_registry_get_entry_c,
    ] {
        let lookup: FrameRegistryGetEntryFn = unsafe { core::mem::transmute(addr) };
        let frame = unsafe { lookup(name, create_context) };
        if frame != 0 {
            return frame;
        }
    }

    0
}

fn parse_suffix_index(name: &str, prefix: &str) -> Option<i32> {
    name.strip_prefix(prefix)?.parse::<i32>().ok()
}

unsafe fn cargo_button(ui: usize, index: i32) -> usize {
    let cargo_detail = unsafe { info_bar_field(ui, CINFO_BAR_CARGO_DETAIL) };
    if cargo_detail == 0 {
        return 0;
    }

    let children = unsafe { read_usize(cargo_detail + 0x124) };
    let offset = match index {
        0 => 44,
        1 => 92,
        2 => 188,
        3 => 284,
        4 => 380,
        5 => 476,
        6 => 608,
        7 => 668,
        _ => return 0,
    };

    if children == 0 { 0 } else { unsafe { read_usize(children + offset) } }
}

unsafe fn group_button(ui: usize, index: i32) -> usize {
    let group = unsafe { info_bar_field(ui, CINFO_BAR_GROUP) };
    if group == 0 {
        return 0;
    }

    let children = unsafe { read_usize(group + 0x124) };
    let offset = match index {
        0 => 152,
        1 => 188,
        2 => 344,
        3 => 500,
        4 => 656,
        5 => 812,
        6 => 968,
        7 => 1124,
        8 => 1296,
        9 => 1416,
        10 => 1608,
        11 => 1764,
        _ => return 0,
    };

    if children == 0 { 0 } else { unsafe { read_usize(children + offset) } }
}

unsafe fn origin_frame(frame_type: u32, index: i32) -> usize {
    let ui = get_game_ui() as usize;
    if ui == 0 {
        return 0;
    }

    match frame_type {
        0 => ui,
        1 => {
            if !(0..=11).contains(&index) {
                return 0;
            }
            let command_bar = unsafe { ui_field(ui, CGAME_UI_COMMAND_BAR) };
            if command_bar == 0 {
                return 0;
            }
            let buttons = unsafe { read_usize(command_bar + CCOMMAND_BAR_BUTTONS) };
            if buttons == 0 {
                return 0;
            }
            let row = index / 4;
            let col = index % 4;
            let row_ptr = unsafe { read_usize(buttons + 4 + row as usize * 16) };
            if row_ptr == 0 { 0 } else { unsafe { read_usize(row_ptr + col as usize * 4) } }
        }
        2 => unsafe { ui_field(ui, CGAME_UI_HERO_BAR) },
        3 => {
            let hero_bar = unsafe { ui_field(ui, CGAME_UI_HERO_BAR) };
            if hero_bar == 0 {
                return 0;
            }
            if (0..6).contains(&index) {
                let list = unsafe { read_usize(hero_bar + CHERO_BAR_HERO_BUTTONS) };
                if list == 0 {
                    return 0;
                }
                let node = unsafe { read_usize(list + 20 + 16 * index as usize) };
                if node == 0 { 0 } else { unsafe { read_usize(node + 16) } }
            } else if index == 6 {
                unsafe { super::ops::child(hero_bar, index).unwrap_or(0) }
            } else {
                0
            }
        }
        4 => {
            let button = unsafe { origin_frame(3, index) };
            if button == 0 { 0 } else { unsafe { read_usize(button + CHERO_BAR_BUTTON_BAR1) } }
        }
        5 => {
            let button = unsafe { origin_frame(3, index) };
            if button == 0 { 0 } else { unsafe { read_usize(button + CHERO_BAR_BUTTON_BAR2) } }
        }
        6 => {
            let button = unsafe { origin_frame(3, index) };
            if button == 0 { 0 } else { unsafe { read_usize(button + CHERO_BAR_BUTTON_SPRITE) } }
        }
        7 => {
            if !(0..=5).contains(&index) {
                return 0;
            }
            let inv_bar = unsafe { info_bar_field(ui, CINFO_BAR_INVENTORY_BAR) };
            if inv_bar == 0 {
                return 0;
            }
            let buttons = unsafe { read_usize(inv_bar + CINVENTORY_BAR_BUTTONS) };
            if buttons == 0 { 0 } else { unsafe { read_usize(buttons + 4 + 8 * index as usize) } }
        }
        8 => unsafe { ui_field(ui, CGAME_UI_MINIMAP) },
        9 => match index {
            0 => unsafe { ui_field(ui, CGAME_UI_MINIMAP_BUTTON_SIGNAL) },
            1 => unsafe { ui_field(ui, CGAME_UI_MINIMAP_BUTTON_ALLY) },
            2 => unsafe { ui_field(ui, CGAME_UI_MINIMAP_BUTTON_CREEP) },
            3 => unsafe { ui_field(ui, CGAME_UI_MINIMAP_BUTTON_TERRAIN) },
            4 => unsafe { ui_field(ui, CGAME_UI_MINIMAP_BUTTON_FORMATION) },
            _ => 0,
        },
        10 => {
            let upper = unsafe { ui_field(ui, CGAME_UI_UPPER_BUTTON_BAR) };
            if upper == 0 {
                return 0;
            }
            match index {
                0 => unsafe { read_usize(upper + CUPPER_BUTTON_BAR_MENU) },
                1 => unsafe { read_usize(upper + CUPPER_BUTTON_BAR_ALLIES) },
                2 => unsafe { read_usize(upper + CUPPER_BUTTON_BAR_LOG) },
                3 => unsafe { read_usize(upper + CUPPER_BUTTON_BAR_QUESTS) },
                _ => 0,
            }
        }
        11 => unsafe { ui_field(ui, 0x1D8) },
        12 => unsafe { ui_field(ui, 0x1DC) },
        13 => unsafe { ui_field(ui, CGAME_UI_CHAT_MESSAGE) },
        14 => unsafe { ui_field(ui, CGAME_UI_UNIT_MESSAGE) },
        15 => unsafe { ui_field(ui, CGAME_UI_TOP_MESSAGE) },
        16 => unsafe { ui_field(ui, CGAME_UI_PORTRAIT_BUTTON) },
        17 => unsafe { ui_field(ui, CGAME_UI_WORLD_FRAME) },
        18 => unsafe { ui_field(ui, CGAME_UI_SIMPLE_FRAME) },
        19 => {
            let portrait = unsafe { ui_field(ui, CGAME_UI_PORTRAIT_BUTTON) };
            if portrait == 0 { 0 } else { unsafe { read_usize(portrait + CPORTRAIT_BUTTON_TEXT_LIFE) } }
        }
        20 => {
            let portrait = unsafe { ui_field(ui, CGAME_UI_PORTRAIT_BUTTON) };
            if portrait == 0 { 0 } else { unsafe { read_usize(portrait + CPORTRAIT_BUTTON_TEXT_MANA) } }
        }
        21 => {
            let unit_detail = unsafe { info_bar_field(ui, CINFO_BAR_UNIT_DETAIL) };
            if unit_detail == 0 { 0 } else { unsafe { read_usize(unit_detail + CINFO_PANEL_UNIT_DETAIL_BUFF_BAR) } }
        }
        22 => {
            let buff_bar = unsafe { origin_frame(21, 0) };
            if buff_bar == 0 { 0 } else { unsafe { read_usize(buff_bar + CBUFF_BAR_BUFF_BAR_LABEL) } }
        }
        23 => unsafe { ui_field(ui, CGAME_UI_TIME_OF_DAY_INDICATOR) },
        24 => unsafe { ui_field(ui, CGAME_UI_CINEMATIC_PANEL) },
        25 => unsafe { ui_field(ui, CGAME_UI_ERROR_MESSAGE) },
        26 => {
            let peon_bar = unsafe { ui_field(ui, CGAME_UI_PEON_BAR) };
            if peon_bar == 0 { 0 } else { unsafe { read_usize(peon_bar + CPEON_BAR_PEON_BUTTON) } }
        }
        27 => unsafe { group_button(ui, index) },
        28 => {
            let button = unsafe { group_button(ui, index) };
            if button == 0 { 0 } else { unsafe { read_usize(button + CGROUP_BUTTON_BAR1) } }
        }
        29 => {
            let button = unsafe { group_button(ui, index) };
            if button == 0 { 0 } else { unsafe { read_usize(button + CGROUP_BUTTON_BAR2) } }
        }
        30 => unsafe { cargo_button(ui, index) },
        31 => {
            let button = unsafe { cargo_button(ui, index) };
            if button == 0 { 0 } else { unsafe { read_usize(button + CCARGO_BUTTON_BAR1) } }
        }
        32 => {
            let button = unsafe { cargo_button(ui, index) };
            if button == 0 { 0 } else { unsafe { read_usize(button + CCARGO_BUTTON_BAR2) } }
        }
        33 => {
            let uber = unsafe { ui_field(ui, 0x1DC) };
            if uber == 0 {
                return 0;
            }
            match index {
                0..=5 => unsafe { read_usize(uber + CUBER_TIP_STRING0 + index as usize * 4) },
                6 => unsafe { read_usize(uber + CUBER_TIP_STRING6) },
                _ => 0,
            }
        }
        34 => {
            let uber = unsafe { ui_field(ui, 0x1DC) };
            if uber == 0 || !(0..=3).contains(&index) {
                0
            } else {
                unsafe { read_usize(uber + CUBER_TIP_ICON0 + index as usize * 4) }
            }
        }
        35 => unsafe { ui_field(ui, CGAME_UI_CURSOR_FRAME) },
        36 => {
            let resource = unsafe { ui_field(ui, CGAME_UI_RESOURCE_BAR) };
            if resource == 0 { 0 } else { unsafe { read_usize(resource + CRESOURCE_BAR_FPS) } }
        }
        37 => {
            let resource = unsafe { ui_field(ui, CGAME_UI_RESOURCE_BAR) };
            if resource == 0 { 0 } else { unsafe { read_usize(resource + CRESOURCE_BAR_APM) } }
        }
        38 => {
            let resource = unsafe { ui_field(ui, CGAME_UI_RESOURCE_BAR) };
            if resource == 0 { 0 } else { unsafe { read_usize(resource + CRESOURCE_BAR_PING) } }
        }
        _ => 0,
    }
}

unsafe fn frame_name_ptr(frame: usize) -> usize {
    if !unsafe { frame_registry::is_valid(frame) } {
        return 0;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    if unsafe { frame_registry::is_simple(frame) } {
        let offset = if ft == FrameType::CSimpleFontString || ft == FrameType::CSimpleTexture {
            C_SIMPLE_FONT_NAME
        } else {
            C_SIMPLE_NAME
        };
        return unsafe { read_usize(frame + offset) };
    }

    let offset = if unsafe { frame_registry::get_layout(frame) } == Some(frame) {
        C_FRAME_NAME_LAYOUT
    } else {
        C_FRAME_NAME_NORMAL
    };
    unsafe { read_usize(frame + offset) }
}

unsafe fn create_context_offset(frame: usize) -> Option<usize> {
    if !unsafe { frame_registry::is_valid(frame) } {
        return None;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    if ft == FrameType::CSimpleTexture || ft == FrameType::CSimpleFontString {
        Some(C_SIMPLE_FONT_CREATE_CONTEXT)
    } else if unsafe { frame_registry::is_simple(frame) } {
        Some(C_SIMPLE_CREATE_CONTEXT)
    } else {
        Some(C_FRAME_CREATE_CONTEXT)
    }
}

unsafe fn frame_alpha(frame: usize) -> i32 {
    if !unsafe { frame_registry::is_valid(frame) } {
        return 0;
    }
    if unsafe { frame_registry::is_simple(frame) } {
        let ft = unsafe { frame_registry::frame_type(frame) };
        if ft == FrameType::CSimpleFontString || ft == FrameType::CSimpleTexture {
            return unsafe { read_u32(frame + C_SIMPLE_FONT_ALPHA) as u8 as i32 };
        }
        return unsafe { read_u32(frame + C_SIMPLE_COLOR) as u8 as i32 };
    }
    unsafe { read_u32(frame + C_FRAME_COLOR) as u8 as i32 }
}

unsafe fn frame_is_visible(frame: usize) -> bool {
    if !unsafe { frame_registry::is_valid(frame) } {
        return false;
    }
    if unsafe { frame_registry::is_simple(frame) } {
        let ft = unsafe { frame_registry::frame_type(frame) };
        if ft == FrameType::CSimpleFontString || ft == FrameType::CSimpleTexture {
            let parent = unsafe { read_usize(frame + C_SIMPLE_FONT_PARENT) };
            if unsafe { frame_registry::is_valid(parent) } {
                if unsafe { frame_is_visible(parent) } {
                    return unsafe { read_i32(frame + 128) != 0 };
                }
                return false;
            }
            return true;
        }
        return unsafe { read_i32(frame + C_SIMPLE_VISIBILITY_FLAG) & 1 == 1 };
    }
    unsafe { read_i32(frame + C_FRAME_VISIBILITY_FLAG) & 1 == 0 }
}

unsafe fn frame_get_scale(frame: usize) -> f32 {
    let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
        return 0.0;
    };
    unsafe { read_f32(layout + C_LAYOUT_SCALE) }
}

unsafe fn frame_set_scale(frame: usize, scale: f32) -> bool {
    if !unsafe { frame_registry::is_valid(frame) } {
        return false;
    }
    let ft = unsafe { frame_registry::frame_type(frame) };
    match ft {
        FrameType::CSimpleFontString => {
            let f: CSimpleFontStringSetLayoutScaleFn =
                unsafe { core::mem::transmute(addresses::get().frames.c_simple_font_string_set_layout_scale) };
            unsafe { f(frame, scale) };
            true
        }
        FrameType::CSimpleGlueFrame => {
            let f: CSimpleGlueFrameSetLayoutScaleFn =
                unsafe { core::mem::transmute(addresses::get().frames.c_simple_glue_frame_set_layout_scale) };
            unsafe { f(frame, scale) };
            true
        }
        FrameType::CSimpleMessageFrame | FrameType::CSimpleStatusBar => {
            let f: CSimpleFrameSetLayoutScaleFn =
                unsafe { core::mem::transmute(addresses::get().frames.c_simple_frame_set_layout_scale) };
            unsafe { f(frame, scale) };
            true
        }
        FrameType::CSpriteFrame => {
            let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
                return false;
            };
            let f: CSpriteFrameSetLayoutScaleFn =
                unsafe { core::mem::transmute(addresses::get().frames.c_sprite_frame_set_layout_scale) };
            unsafe { f(layout, scale) };
            true
        }
        _ => {
            let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
                return false;
            };
            unsafe { write_f32(layout + C_LAYOUT_SCALE, scale) };
            let update: CLayoutFrameUpdateFn =
                unsafe { core::mem::transmute(addresses::get().frames.c_layout_frame_update) };
            unsafe { update(layout, 1) };
            true
        }
    }
}

unsafe fn frame_get_enable(frame: usize) -> bool {
    if !unsafe { frame_registry::is_valid(frame) } {
        return false;
    }
    let ft = unsafe { frame_registry::frame_type(frame) };
    if unsafe { frame_registry::is_simple(frame) } {
        if ft == FrameType::CSimpleButton {
            return unsafe { read_i32(frame + C_SIMPLE_BUTTON_ENABLED) & 1 == 1 };
        }
        return false;
    }
    if frame_inherits_from_control(ft, false) {
        let f: CControlCheckStateFn =
            unsafe { core::mem::transmute(addresses::get().frames.c_control_check_state) };
        return unsafe { f(frame, 1) };
    }
    true
}

unsafe fn frame_set_enable(frame: usize, enabled: bool) -> bool {
    if !unsafe { frame_registry::is_valid(frame) } {
        return false;
    }
    let ft = unsafe { frame_registry::frame_type(frame) };
    if unsafe { frame_registry::is_simple(frame) } {
        if ft == FrameType::CSimpleButton {
            let f: CSimpleButtonSetEnableFn =
                unsafe { core::mem::transmute(addresses::get().frames.c_simple_button_set_enable) };
            unsafe { f(frame, enabled) };
            return true;
        }
        return false;
    }
    if frame_inherits_from_control(ft, false) {
        let f: CControlEnableFn =
            unsafe { core::mem::transmute(addresses::get().frames.c_control_enable) };
        unsafe { f(frame, if enabled { 1 } else { 0 }) };
        return true;
    }
    false
}

unsafe fn frame_set_focus(frame: usize, flag: bool) -> bool {
    if !unsafe { frame_registry::is_valid(frame) } {
        return false;
    }
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CEditBox {
        return false;
    }
    let f: CEditBoxSetFocusFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_edit_box_set_focus) };
    unsafe { f(frame, flag) };
    true
}

unsafe fn ansi_string_value(addr: usize) -> Option<String> {
    let p = unsafe { read_usize(addr) };
    if p == 0 {
        return None;
    }

    unsafe { std::ffi::CStr::from_ptr(p as *const i8) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

unsafe fn text_value(frame: usize) -> Option<String> {
    if !unsafe { frame_registry::is_valid(frame) } {
        return None;
    }

    match unsafe { frame_registry::frame_type(frame) } {
        FrameType::CEditBox => unsafe { ansi_string_value(frame + offsets::C_EDIT_BOX_TEXT) },
        FrameType::CTextArea => unsafe { ansi_string_value(frame + offsets::C_TEXT_AREA_TEXT) },
        FrameType::CSimpleFontString => unsafe { ansi_string_value(frame + offsets::C_SIMPLE_FONT_STRING_TEXT) },
        FrameType::CTextFrame | FrameType::CTimerTextFrame | FrameType::CListBoxItem => {
            unsafe { ansi_string_value(frame + offsets::C_TEXT_FRAME_TEXT) }
        }
        FrameType::CGlueTextButtonWar3 => {
            let text_frame = unsafe { read_usize(frame + offsets::C_TEXT_BUTTON_TEXT_FRAME) };
            if text_frame == 0 {
                None
            } else {
                unsafe { text_value(text_frame) }
            }
        }
        _ => None,
    }
}

unsafe fn call_vtable_void(frame: usize, index: usize) -> bool {
    let Some(vtable) = (unsafe { frame_registry::vtable(frame) }) else {
        return false;
    };
    let fn_ptr = unsafe { read_usize(vtable + index * 4) };
    if fn_ptr == 0 {
        return false;
    }
    let f: SimpleVTableVoidFn = unsafe { core::mem::transmute(fn_ptr) };
    unsafe { f(frame) };
    true
}

unsafe fn control_is_checked(frame: usize) -> bool {
    let f: CControlCheckStateFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_control_check_state) };
    unsafe { f(frame, 2) }
}

unsafe fn set_slider_value(frame: usize, value: f32) {
    let f: CSliderSetValueFn = unsafe { core::mem::transmute(addresses::get().frames.c_slider_set_current_value) };
    unsafe { f(frame, value) };
}

unsafe fn frame_set_text_alignment(frame: usize, mut vert: i32, mut horz: i32) -> bool {
    if !unsafe { frame_registry::is_valid(frame) } {
        return false;
    }
    if vert == 3 {
        vert = 0;
    } else if vert == 4 {
        vert = 1;
    } else if vert == 5 {
        vert = 2;
    }
    if horz == 3 {
        horz = 0;
    } else if horz == 4 {
        horz = 1;
    } else if horz == 5 {
        horz = 2;
    }
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CTextFrame {
        return false;
    }
    let set_h: CTextFrameSetJustificationFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_text_frame_set_horizontal_justification) };
    let set_v: CTextFrameSetJustificationFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_text_frame_set_vertical_justification) };
    let update: CTextFrameUpdateControlFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_text_frame_update_control) };
    unsafe {
        set_h(frame, horz);
        set_v(frame, vert);
        update(frame);
    }
    true
}

unsafe fn frame_cage_mouse(frame: usize, enable: bool) -> bool {
    let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
        return false;
    };
    unsafe { write_i32_raw(layout + C_LAYOUT_MOUSE_CAGED, if enable { 1 } else { 0 }) };
    if !enable {
        let f: CLayoutFrameCageMouseFn =
            unsafe { core::mem::transmute(addresses::get().frames.c_layout_frame_cage_mouse) };
        unsafe { f(layout, enable) };
    }
    true
}

pub(super) fn frame_inherits_from_control(ft: FrameType, simple: bool) -> bool {
    if simple {
        return false;
    }

    matches!(
        ft,
        FrameType::CBackdropFrame
            | FrameType::CButtonFrame
            | FrameType::CEditBox
            | FrameType::CModelFrame
            | FrameType::CPortraitButton
            | FrameType::CSlider
            | FrameType::CTextArea
            | FrameType::CTextButtonFrame
            | FrameType::CTextFrame
            | FrameType::CGlueButtonWar3
            | FrameType::CGlueTextButtonWar3
            | FrameType::CGlueCheckBoxWar3
            | FrameType::CGluePopupMenuWar3
            | FrameType::CGlueEditBoxWar3
            | FrameType::CSlashChatBox
            | FrameType::CTimerTextFrame
            | FrameType::CListBoxWar3
            | FrameType::CCheckBox
            | FrameType::CMapList
            | FrameType::CPopupMenu
            | FrameType::CListBoxItemWar3
            | FrameType::CActionMenuItem
            | FrameType::CBattleNetClanMateListBoxItem
            | FrameType::CBattleNetFriendsListBoxItem
            | FrameType::CBattleNetNewsBoxItem
            | FrameType::CBattleNetProfileListBoxItem
            | FrameType::CBattleNetUserListBoxItem
            | FrameType::CCampaignListBoxItem
            | FrameType::CCheckListBoxItem
            | FrameType::CLeaderboardListBoxItem
            | FrameType::CMapListBoxItem
            | FrameType::CMapPreferenceBoxItem
            | FrameType::CMultiboardListBoxRow
            | FrameType::CQuestItemListBoxItem
            | FrameType::CQuestListBoxItem
            | FrameType::CTeamColorItem
            | FrameType::CScrollBar
            | FrameType::CControl
            | FrameType::CBattleNetClanPane
            | FrameType::CBattleNetFriendsPane
            | FrameType::CBattleNetIconSelectBox
            | FrameType::CBattleNetIconSelectBoxItem
            | FrameType::CBattleNetStatusBox
            | FrameType::CChatDisplay
            | FrameType::CClockCover
            | FrameType::CBattleNetChatEditBox
            | FrameType::CChatEditBox
            | FrameType::CListBox
            | FrameType::CBattleNetChatActionMenu
            | FrameType::CBattleNetClanMateListBox
            | FrameType::CBattleNetFriendsListBox
            | FrameType::CBattleNetNewsBox
            | FrameType::CBattleNetProfileListBox
            | FrameType::CBattleNetUserListBox
            | FrameType::CCampaignListBox
            | FrameType::CCheckListBox
            | FrameType::CLeaderboardList
            | FrameType::CMapPreferenceBox
            | FrameType::CMultiboardList
            | FrameType::CQuestItemList
            | FrameType::CQuestList
            | FrameType::CTeamColorMenu
            | FrameType::CListButton
            | FrameType::CMenu
            | FrameType::CChatEditBar
            | FrameType::CPlayerSlot
            | FrameType::CRadioGroup
            | FrameType::CTeamSetup
            | FrameType::CListBoxItem
            | FrameType::CXPBarCover
    )
}

unsafe fn frame_layout(frame: u32, native: &str) -> Option<usize> {
    let frame = frame as usize;
    if frame == 0 {
        return None;
    }

    if let Some(layout) = unsafe { frame_registry::get_layout(frame) } {
        return Some(layout);
    }

    let vtable = unsafe { frame_registry::vtable(frame) }.unwrap_or(0);
    let off = unsafe { frame_registry::vtable_static_key(frame) }.unwrap_or(0);
    logging::warn(&format!(
        "[frames] {native}: unknown frame vtable frame=0x{frame:x} vtable=0x{vtable:x} static_key=0x{off:x}; falling back to frame+188"
    ));
    Some(frame + 188)
}

unsafe fn show_simple_frame(frame: usize, show: bool) {
    let Some(vtable) = (unsafe { frame_registry::vtable(frame) }) else {
        return;
    };
    let index = if show { 27usize } else { 26usize };
    let fn_ptr = unsafe { ((vtable + index * 4) as *const usize).read_unaligned() };
    if fn_ptr == 0 {
        return;
    }
    let f: SimpleVTableShowHideFn = unsafe { core::mem::transmute(fn_ptr) };
    unsafe { f(frame) };
}

unsafe fn frame_show_hide(frame: usize, show: bool) {
    if unsafe { frame_registry::is_simple(frame) } {
        unsafe { show_simple_frame(frame, show) };
        return;
    }

    let addrs = addresses::get();
    let f: CLayerShowHideFn = unsafe {
        core::mem::transmute(if show { addrs.frames.c_layer_show } else { addrs.frames.c_layer_hide })
    };
    unsafe { f(frame) };
}

pub unsafe extern "C" fn blz_get_game_ui() -> u32 {
    let ui = get_game_ui();
    logging::info(&format!("[frames] BlzGetGameUI -> 0x{:x}", ui as usize));
    ui as u32
}

pub unsafe extern "C" fn blz_create_frame(
    name_handle: u32,
    parent: u32,
    priority: u32,
    create_context: u32,
) -> u32 {
    let tramp = hook_manager::trampoline(addresses::get().frames.frame_def_create_frame)
        .expect("frame_def_create_frame trampoline missing");
    let original: super::hooks::FrameDefCreateFrameFn = unsafe { core::mem::transmute(tramp) };

    let parent_ptr = if parent == 0 {
        get_game_ui()
    } else if unsafe { frame_registry::is_simple(parent as usize) } {
        core::ptr::null_mut()
    } else {
        parent as *mut CFrame
    };

    let c_name = native_string_cstring(name_handle);

    logging::info(&format!(
        "[frames] BlzCreateFrame: name='{}' parent=0x{:x}",
        c_name.to_string_lossy(),
        parent_ptr as usize
    ));

    let result = unsafe {
        original(
            c_name.as_ptr(),
            parent_ptr,
            0,
            priority as i32,
            create_context as i32,
        )
    };
    logging::info(&format!("[frames] BlzCreateFrame called -> 0x{:x}", result as usize));
    result as u32
}

pub unsafe extern "C" fn blz_create_simple_frame(
    name_handle: u32,
    parent: u32,
    create_context: u32,
) -> u32 {
    let c_name = native_string_cstring(name_handle);
    let parent_ptr = if parent != 0
        && unsafe { frame_registry::is_valid(parent as usize) }
        && unsafe { frame_registry::is_simple(parent as usize) }
    {
        parent as *mut CFrame
    } else {
        core::ptr::null_mut()
    };

    let create: FrameDefCreateSimpleFrameFn =
        unsafe { core::mem::transmute(addresses::get().frames.frame_def_create_simple_frame) };
    let result = unsafe { create(c_name.as_ptr(), parent_ptr, create_context as i32) };
    logging::info(&format!(
        "[frames] BlzCreateSimpleFrame: name='{}' parent=0x{:x} -> 0x{:x}",
        c_name.to_string_lossy(),
        parent_ptr as usize,
        result as usize
    ));
    result as u32
}

pub unsafe extern "C" fn blz_get_origin_frame(frame_type: u32, index: u32) -> u32 {
    unsafe { origin_frame(frame_type, index as i32) as u32 }
}

pub unsafe extern "C" fn blz_frame_get_by_name(name_handle: u32, create_context: u32) -> u32 {
    let c_name = native_string_cstring(name_handle);
    let name = c_name.to_string_lossy();

    if let Some(index) = parse_suffix_index(&name, "CommandButton_") {
        return unsafe { origin_frame(1, index) as u32 };
    }
    if let Some(index) = parse_suffix_index(&name, "InventoryButton_") {
        return unsafe { origin_frame(7, index) as u32 };
    }

    let ui = get_game_ui() as usize;
    let alias = match name.as_ref() {
        "MinimapSignalButton" => unsafe { origin_frame(9, 0) },
        "MiniMapTerrainButton" => unsafe { origin_frame(9, 1) },
        "MiniMapAllyButton" => unsafe { origin_frame(9, 2) },
        "MiniMapCreepButton" => unsafe { origin_frame(9, 3) },
        "FormationButton" if create_context == 0 => unsafe { origin_frame(9, 4) },
        "MinimapButtonBar" => {
            let button = unsafe { origin_frame(9, 0) };
            if button == 0 { 0 } else { unsafe { frame_registry::parent(button) } }
        }
        "MiniMapFrame" => unsafe { origin_frame(8, 0) },
        "CommandBarFrame" => unsafe { ui_field(ui, CGAME_UI_COMMAND_BAR) },
        "InventoryCoverTexture" => {
            let cover = unsafe { info_bar_field(ui, CINFO_BAR_INVENTORY_COVER) };
            if cover == 0 {
                0
            } else {
                let a = unsafe { read_usize(cover + 64) };
                if a == 0 { 0 } else { unsafe { read_usize(a + 12) } }
            }
        }
        "InventoryText" => unsafe { info_bar_field(ui, CINFO_BAR_INVENTORY_TEXT) },
        "SimpleInventoryBar" => unsafe { info_bar_field(ui, CINFO_BAR_INVENTORY_BAR) },
        "SimpleInventoryCover" => unsafe { info_bar_field(ui, CINFO_BAR_INVENTORY_COVER) },
        _ => 0,
    };
    if alias != 0 {
        return alias as u32;
    }

    unsafe { c_frame_from_name(c_name.as_ptr(), create_context as i32) as u32 }
}

pub unsafe extern "C" fn blz_get_mouse_hovered_frame() -> u32 {
    unsafe { crate::ui::events::hovered_frame() as u32 }
}

pub unsafe extern "C" fn blz_hide_origin_frames(flag: u32) {
    let hide = flag != 0;
    let ui = get_game_ui() as usize;
    if ui == 0 {
        return;
    }

    for frame in [
        unsafe { ui_field(ui, CGAME_UI_RESOURCE_BAR) },
        unsafe { ui_field(ui, CGAME_UI_UPPER_BUTTON_BAR) },
        unsafe { ui_field(ui, CGAME_UI_TIME_OF_DAY_INDICATOR) },
        unsafe { ui_field(ui, CGAME_UI_PORTRAIT_BUTTON) },
        unsafe { ui_field(ui, CGAME_UI_MINIMAP) },
    ] {
        if frame != 0 {
            unsafe { frame_show_hide(frame, !hide) };
        }
    }

    let console_name = CString::new("CeConsoleUITextures").unwrap();
    let console_textures = unsafe { c_frame_from_name(console_name.as_ptr(), 0) };
    if console_textures != 0 {
        unsafe { frame_show_hide(console_textures, !hide) };
    }

    let simple_console = unsafe { ui_field(ui, CGAME_UI_SIMPLE_CONSOLE_2) };
    let count = unsafe { super::ops::children_count(simple_console) };
    for i in 1..count {
        if let Some(child) = unsafe { super::ops::child(simple_console, i) } {
            unsafe { frame_show_hide(child, !hide) };
        }
    }

    logging::info(&format!("[frames] BlzHideOriginFrames hide={hide}"));
}

pub unsafe extern "C" fn ce_get_command_button_data(button_index: u32, data_index: u32) -> u32 {
    if button_index > 11 {
        return 0;
    }

    let button = unsafe { origin_frame(1, button_index as i32) };
    if button == 0 {
        return 0;
    }

    let data = unsafe { read_usize(button + CCOMMAND_BUTTON_DATA) };
    if data == 0 {
        return 0;
    }

    let offset = match data_index {
        0 => CCOMMAND_BUTTON_DATA_A,
        1 => CCOMMAND_BUTTON_DATA_B,
        2 => CCOMMAND_BUTTON_DATA_C,
        _ => return 0,
    };

    unsafe { read_i32(data + offset) as u32 }
}

pub unsafe extern "C" fn blz_frame_get_name(frame: u32) -> u32 {
    let name = unsafe { frame_name_ptr(frame as usize) };
    if name == 0 {
        return 0;
    }

    crate::jass::raw::make_jass_string(name as *const u8) as u32
}

pub unsafe extern "C" fn ce_get_frame_create_context(frame: u32) -> u32 {
    let frame = frame as usize;
    let Some(offset) = (unsafe { create_context_offset(frame) }) else {
        return 0;
    };
    unsafe { read_i32(frame + offset) as u32 }
}

pub unsafe extern "C" fn ce_set_frame_create_context(frame: u32, context: u32) {
    let frame = frame as usize;
    let Some(offset) = (unsafe { create_context_offset(frame) }) else {
        return;
    };
    unsafe { write_i32(frame + offset, context as i32) };
}

pub unsafe extern "C" fn ce_get_frame_layer_style(frame: u32) -> u32 {
    let frame = frame as usize;
    if frame == 0 || !unsafe { frame_registry::is_valid(frame) } || unsafe { frame_registry::is_simple(frame) } {
        return 0;
    }
    unsafe { read_i32(frame + C_FRAME_LAYER_STYLE) as u32 }
}

pub unsafe extern "C" fn ce_set_frame_layer_style(frame: u32, value: u32) {
    let frame = frame as usize;
    if frame == 0 || !unsafe { frame_registry::is_valid(frame) } || unsafe { frame_registry::is_simple(frame) } {
        return;
    }
    unsafe { write_i32(frame + C_FRAME_LAYER_STYLE, value as i32) };
}

pub unsafe extern "C" fn ce_get_frame_control_style(frame: u32) -> u32 {
    let frame = frame as usize;
    if frame == 0 || !unsafe { frame_registry::is_valid(frame) } {
        return 0;
    }

    let simple = unsafe { frame_registry::is_simple(frame) };
    let ft = unsafe { frame_registry::frame_type(frame) };
    if !frame_inherits_from_control(ft, simple) {
        return 0;
    }

    unsafe { read_i32(frame + C_CONTROL_STYLE) as u32 }
}

pub unsafe extern "C" fn ce_set_frame_control_style(frame: u32, value: u32) {
    let frame = frame as usize;
    if frame == 0 || !unsafe { frame_registry::is_valid(frame) } {
        return;
    }

    let simple = unsafe { frame_registry::is_simple(frame) };
    let ft = unsafe { frame_registry::frame_type(frame) };
    if !frame_inherits_from_control(ft, simple) {
        return;
    }

    unsafe { write_i32(frame + C_CONTROL_STYLE, value as i32) };
}

pub unsafe extern "C" fn blz_frame_is_visible(frame: u32) -> u32 {
    if unsafe { frame_is_visible(frame as usize) } { 1 } else { 0 }
}

pub unsafe extern "C" fn blz_frame_get_alpha(frame: u32) -> u32 {
    unsafe { frame_alpha(frame as usize) as u32 }
}

pub unsafe extern "C" fn blz_frame_get_scale(frame: u32) -> u32 {
    unsafe { frame_get_scale(frame as usize).to_bits() }
}

pub unsafe extern "C" fn blz_frame_set_scale(frame: u32, scale_ptr: u32) {
    if frame == 0 || scale_ptr == 0 {
        return;
    }
    let scale = unsafe { *(scale_ptr as *const f32) };
    if !unsafe { frame_set_scale(frame as usize, scale) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!("[frames] BlzFrameSetScale failed frame=0x{frame:x} type={ft:?}"));
    }
}

pub unsafe extern "C" fn blz_frame_get_enable(frame: u32) -> u32 {
    if unsafe { frame_get_enable(frame as usize) } { 1 } else { 0 }
}

pub unsafe extern "C" fn blz_frame_set_enable(frame: u32, enabled: u32) {
    if !unsafe { frame_set_enable(frame as usize, enabled != 0) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!("[frames] BlzFrameSetEnable failed frame=0x{frame:x} type={ft:?}"));
    }
}

pub unsafe extern "C" fn blz_frame_set_focus(frame: u32, flag: u32) {
    if !unsafe { frame_set_focus(frame as usize, flag != 0) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!("[frames] BlzFrameSetFocus failed frame=0x{frame:x} type={ft:?}"));
    }
}

pub unsafe extern "C" fn blz_frame_set_text_alignment(frame: u32, vert: u32, horz: u32) {
    if !unsafe { frame_set_text_alignment(frame as usize, vert as i32, horz as i32) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!("[frames] BlzFrameSetTextAlignment failed frame=0x{frame:x} type={ft:?}"));
    }
}

pub unsafe extern "C" fn blz_frame_cage_mouse(frame: u32, enable: u32) {
    if !unsafe { frame_cage_mouse(frame as usize, enable != 0) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!("[frames] BlzFrameCageMouse failed frame=0x{frame:x} type={ft:?}"));
    }
}

pub unsafe extern "C" fn blz_frame_set_font(
    frame: u32,
    filename_handle: u32,
    height_ptr: u32,
    flags: u32,
) {
    if frame == 0 || height_ptr == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }

    let height = unsafe { *(height_ptr as *const f32) };
    let filename = native_string_cstring(filename_handle);
    let frame_usize = frame as usize;
    let ft = unsafe { frame_registry::frame_type(frame_usize) };
    unsafe {
        match ft {
            FrameType::CEditBox => {
                let f: CEditBoxSetFontFn = core::mem::transmute(addresses::get().frames.c_edit_box_set_font);
                f(frame_usize, filename.as_ptr(), height, flags as i32);
            }
            FrameType::CSimpleMessageFrame => {
                let f: CSimpleFrameSetFontFn =
                    core::mem::transmute(addresses::get().frames.c_simple_message_frame_set_font);
                f(frame_usize, filename.as_ptr(), height, flags as i32);
            }
            FrameType::CSimpleFontString => {
                let f: CSimpleFrameSetFontFn =
                    core::mem::transmute(addresses::get().frames.c_simple_font_string_set_font);
                f(frame_usize, filename.as_ptr(), height, flags as i32);
            }
            _ if !frame_registry::is_simple(frame_usize) => {
                let f: CFrameSetFontFn = core::mem::transmute(addresses::get().frames.c_frame_set_font);
                f(frame_usize, filename.as_ptr(), height, flags as i32);
            }
            _ => logging::warn(&format!("[frames] BlzFrameSetFont unsupported frame=0x{frame:x} type={ft:?}")),
        }
    }
}

pub unsafe extern "C" fn blz_frame_get_text(frame: u32) -> u32 {
    let Some(text) = (unsafe { text_value(frame as usize) }) else {
        return 0;
    };
    let Ok(c_text) = CString::new(text) else {
        return 0;
    };
    crate::jass::raw::make_jass_string(c_text.as_ptr() as *const u8) as u32
}

pub unsafe extern "C" fn blz_frame_get_text_size_limit(frame: u32) -> u32 {
    if frame == 0 || unsafe { frame_registry::frame_type(frame as usize) } != FrameType::CEditBox {
        return 0;
    }
    unsafe { read_i32(frame as usize + offsets::C_EDIT_BOX_TEXT_SIZE_LIMIT) as u32 }
}

pub unsafe extern "C" fn blz_frame_set_text_size_limit(frame: u32, size: u32) {
    if frame == 0 || unsafe { frame_registry::frame_type(frame as usize) } != FrameType::CEditBox {
        return;
    }
    let f: CEditBoxSetTextSizeLimitFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_edit_box_set_text_size_limit) };
    unsafe { f(frame as usize, size as i32) };
}

pub unsafe extern "C" fn blz_frame_click(frame: u32) {
    if frame == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }

    let frame_usize = frame as usize;
    let ft = unsafe { frame_registry::frame_type(frame_usize) };
    if unsafe { frame_registry::is_simple(frame_usize) } {
        if matches!(
            ft,
            FrameType::CCommandButton
                | FrameType::CHeroBarButton
                | FrameType::CSimpleButton
                | FrameType::CTrainableButton
                | FrameType::CShrinkingButton
                | FrameType::CReplayButton
                | FrameType::CSimpleCheckbox
        ) {
            let _ = unsafe { call_vtable_void(frame_usize, 28) };
        }
        return;
    }

    if frame_inherits_from_control(ft, false) {
        let f: CControlDispatchClickFn =
            unsafe { core::mem::transmute(addresses::get().frames.c_control_dispatch_click) };
        unsafe { f(frame_usize, 1) };
    }
}

pub unsafe extern "C" fn blz_frame_set_value(frame: u32, value_ptr: u32) {
    if frame == 0 || value_ptr == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }

    let frame_usize = frame as usize;
    let value = unsafe { *(value_ptr as *const f32) };
    let ft = unsafe { frame_registry::frame_type(frame_usize) };
    unsafe {
        match ft {
            FrameType::CGlueCheckBoxWar3 | FrameType::CCheckBox => {
                if (value >= 1.0) != control_is_checked(frame_usize) {
                    let _ = call_vtable_void(frame_usize, 61);
                }
            }
            FrameType::CSlider | FrameType::CScrollBar => set_slider_value(frame_usize, value),
            FrameType::CSimpleStatusBar => {
                let f: CSimpleStatusBarSetValueFn =
                    core::mem::transmute(addresses::get().frames.c_simple_status_bar_set_value);
                f(frame_usize, value);
            }
            FrameType::CStatusBar => {
                let f: CStatusBarSetValueFn = core::mem::transmute(addresses::get().frames.c_status_bar_set_value);
                f(frame_usize, value, 0);
            }
            FrameType::CTextArea => {
                let scroll_bar = read_usize(frame_usize + offsets::C_TEXT_AREA_SCROLL_BAR);
                if scroll_bar != 0 {
                    set_slider_value(scroll_bar, value);
                }
            }
            _ => logging::warn(&format!("[frames] BlzFrameSetValue unsupported frame=0x{frame:x} type={ft:?}")),
        }
    }
}

pub unsafe extern "C" fn blz_frame_get_value(frame: u32) -> u32 {
    if frame == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return 0.0f32.to_bits();
    }

    let frame_usize = frame as usize;
    let value = unsafe {
        match frame_registry::frame_type(frame_usize) {
            FrameType::CSlider | FrameType::CScrollBar => read_f32(frame_usize + offsets::C_SLIDER_VALUE_CURRENT),
            FrameType::CTextArea => {
                let scroll_bar = read_usize(frame_usize + offsets::C_TEXT_AREA_SCROLL_BAR);
                if scroll_bar == 0 {
                    0.0
                } else {
                    read_f32(scroll_bar + offsets::C_SLIDER_VALUE_CURRENT)
                }
            }
            FrameType::CGlueCheckBoxWar3 | FrameType::CCheckBox => {
                if control_is_checked(frame_usize) { 1.0 } else { 0.0 }
            }
            FrameType::CStatBar
            | FrameType::CSimpleStatusBar
            | FrameType::CProgressIndicator
            | FrameType::CHeroLevelBar
            | FrameType::CBuildTimeIndicator => {
                read_f32(frame_usize + offsets::C_SIMPLE_STATUS_BAR_VALUE_CURRENT)
            }
            FrameType::CStatusBar => read_f32(frame_usize + offsets::C_STATUS_BAR_VALUE_CURRENT),
            FrameType::CPopupMenu => read_i32(frame_usize + offsets::C_POPUP_MENU_VALUE) as f32,
            _ => 0.0,
        }
    };
    value.to_bits()
}

pub unsafe extern "C" fn blz_frame_set_min_max_value(frame: u32, min_ptr: u32, max_ptr: u32) {
    if frame == 0 || min_ptr == 0 || max_ptr == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }

    let frame_usize = frame as usize;
    let min = unsafe { *(min_ptr as *const f32) };
    let max = unsafe { *(max_ptr as *const f32) };
    let ft = unsafe { frame_registry::frame_type(frame_usize) };
    unsafe {
        match ft {
            FrameType::CSlider | FrameType::CScrollBar => {
                write_f32(frame_usize + offsets::C_SLIDER_VALUE_MIN, min);
                write_f32(frame_usize + offsets::C_SLIDER_VALUE_MAX, max);
                let cur = read_f32(frame_usize + offsets::C_SLIDER_VALUE_CURRENT);
                set_slider_value(frame_usize, cur.clamp(min, max));
            }
            FrameType::CSimpleStatusBar => {
                let f: CSimpleStatusBarSetMinMaxValueFn =
                    core::mem::transmute(addresses::get().frames.c_simple_status_bar_set_min_max_value);
                f(frame_usize, min, max);
            }
            FrameType::CStatusBar => {
                let f: CStatusBarSetMinMaxValueFn =
                    core::mem::transmute(addresses::get().frames.c_status_bar_set_min_max_value);
                f(frame_usize, min, max);
            }
            _ => logging::warn(&format!("[frames] BlzFrameSetMinMaxValue unsupported frame=0x{frame:x} type={ft:?}")),
        }
    }
}

pub unsafe extern "C" fn blz_frame_set_step_size(frame: u32, step_ptr: u32) {
    if frame == 0 || step_ptr == 0 || unsafe { frame_registry::frame_type(frame as usize) } != FrameType::CSlider {
        return;
    }

    let step = unsafe { *(step_ptr as *const f32) };
    let frame_usize = frame as usize;
    let flags = unsafe { read_i32(frame_usize + offsets::C_CONTROL_STYLE_FLAGS) };
    unsafe {
        write_i32(frame_usize + offsets::C_CONTROL_STYLE_FLAGS, flags & !0x400);
        write_f32(frame_usize + offsets::C_SLIDER_STEP_SIZE, step);
    }
}

pub unsafe extern "C" fn blz_frame_set_model(
    frame: u32,
    model_handle: u32,
    model_type: u32,
    flag: u32,
) {
    if frame == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }

    let model = native_string_cstring(model_handle);
    let frame_usize = frame as usize;
    let ft = unsafe { frame_registry::frame_type(frame_usize) };
    unsafe {
        match ft {
            FrameType::CModelFrame => {
                let f: CModelFrameAddModelFn = core::mem::transmute(addresses::get().frames.c_model_frame_add_model);
                f(frame_usize, model.as_ptr(), model_type as i32);
            }
            FrameType::CSpriteFrame => {
                let f: CSpriteFrameSetArtFn = core::mem::transmute(addresses::get().frames.c_sprite_frame_set_art);
                f(frame_usize, model.as_ptr(), model_type as i32, flag as i32);
            }
            FrameType::CStatusBar => {
                let f: CStatusBarSetArtFn = core::mem::transmute(addresses::get().frames.c_status_bar_set_art);
                f(frame_usize, model.as_ptr(), model_type as i32);
            }
            _ => logging::warn(&format!("[frames] BlzFrameSetModel unsupported frame=0x{frame:x} type={ft:?}")),
        }
    }
}

pub unsafe extern "C" fn blz_frame_set_sprite_animate(frame: u32, primary_prop: u32, flags: u32) {
    if frame == 0 || unsafe { frame_registry::frame_type(frame as usize) } != FrameType::CSpriteFrame {
        return;
    }

    let get_sprite: CSpriteFrameGetSpriteFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_sprite_frame_get_sprite) };
    let sprite = unsafe { get_sprite(frame as usize) };
    if sprite == 0 {
        return;
    }

    let prop = primary_prop as i32;
    let set_anim: CSpriteUberSetAnimationFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_sprite_uber_set_animation) };
    unsafe { set_anim(sprite, &prop as *const i32, 1, flags as i32) };
}

pub unsafe extern "C" fn blz_frame_set_texture(
    frame: u32,
    texture_handle: u32,
    flag: u32,
    blend: u32,
) {
    if frame == 0 {
        return;
    }

    let frame_usize = frame as usize;
    let known = unsafe { frame_registry::is_valid(frame_usize) };

    let tex = native_string_cstring(texture_handle);
    let tex_ptr = tex.as_ptr() as u32;
    let ft = unsafe { frame_registry::frame_type(frame_usize) };
    let addrs = addresses::get();

    logging::info(&format!(
        "[frames] BlzFrameSetTexture: frame=0x{frame:x} type={ft:?} tex='{}'",
        tex.to_string_lossy()
    ));

    unsafe {
        match ft {
            FrameType::Unknown if !known => {
                let vtable = frame_registry::vtable(frame_usize).unwrap_or(0);
                let off = frame_registry::vtable_static_key(frame_usize).unwrap_or(0);
                logging::warn(&format!(
                    "[frames] BlzFrameSetTexture: unknown frame type frame=0x{frame:x} vtable=0x{vtable:x} static_key=0x{off:x}; trying backdrop texture fallback"
                ));
                let set_texture: CBackdropSetTextureFn =
                    core::mem::transmute(addrs.frames.c_backdrop_set_texture);
                set_texture(frame as *mut c_void, tex_ptr, 0, flag, 0, blend);
            }
            FrameType::CBackdropFrame => {
                let set_texture: CBackdropSetTextureFn =
                    core::mem::transmute(addrs.frames.c_backdrop_set_texture);
                set_texture(frame as *mut c_void, tex_ptr, 0, flag, 0, blend);
            }
            FrameType::CStatBar
            | FrameType::CSimpleStatusBar
            | FrameType::CProgressIndicator
            | FrameType::CHeroLevelBar
            | FrameType::CBuildTimeIndicator => {
                let set_texture: CSimpleStatusBarSetTextureFn =
                    core::mem::transmute(addrs.frames.c_simple_status_bar_set_texture);
                set_texture(frame_usize, tex_ptr, blend);
            }
            FrameType::CSimpleFrame | FrameType::CSimpleTexture => {
                let set_texture: CSimpleTextureSetTextureFn =
                    core::mem::transmute(addrs.frames.c_simple_texture_set_texture);
                set_texture(frame_usize, tex_ptr, blend);
            }
            _ => {
                logging::warn(&format!(
                    "[frames] BlzFrameSetTexture: unsupported type {ft:?} on 0x{frame:x}"
                ));
            }
        }
    }
}

pub unsafe extern "C" fn blz_load_toc_file(name_handle: u32) -> u32 {
    let c_name = native_string_cstring(name_handle);

    let addrs = addresses::get();
    let str_hash_table = addrs.frames.string_hash_node_table;
    let frame_hash_table = addrs.frames.frame_hash_node_table;
    let unk_global = addrs.frames.toc_unk_global;

    let str_hash_cap = unsafe { *((str_hash_table + 24) as *const i32) };
    if str_hash_cap < 65535 {
        let grow_fn: StringHashNodeGrowFn = unsafe { core::mem::transmute(addrs.frames.string_hash_node_grow) };
        unsafe { grow_fn(str_hash_table, 65535) };
    }

    let frame_hash_cap = unsafe { *((frame_hash_table + 24) as *const i32) };
    if frame_hash_cap < 65535 {
        let grow_fn: BaseFrameHashNodeGrowFn =
            unsafe { core::mem::transmute(addrs.frames.base_frame_hash_node_grow) };
        unsafe { grow_fn(frame_hash_table, 65535) };
    }

    let read_fn: FdFileReadFn = unsafe { core::mem::transmute(addrs.frames.frame_fd_file_read) };
    let result = unsafe { read_fn(c_name.as_ptr(), str_hash_table, frame_hash_table, unk_global) };

    logging::info(&format!(
        "[frames] BlzLoadTOCFile: '{}' -> {}",
        c_name.to_string_lossy(),
        result
    ));
    result as u32
}

pub unsafe extern "C" fn blz_frame_set_all_points(frame: u32, relative: u32) {
    let Some(layout_frame) = (unsafe { frame_layout(frame, "BlzFrameSetAllPoints") }) else {
        return;
    };
    let Some(relative_layout) = (unsafe { frame_layout(relative, "BlzFrameSetAllPoints") }) else {
        return;
    };
    let set_all_points: CLayoutFrameSetAllPointsFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_layout_frame_set_all_points) };
    unsafe { set_all_points(layout_frame, relative_layout, 1) };
    logging::info(&format!("[frames] BlzFrameSetAllPoints called on 0x{frame:x}"));
}

pub unsafe extern "C" fn blz_frame_clear_all_points(frame: u32) {
    let Some(layout_frame) = (unsafe { frame_layout(frame, "BlzFrameClearAllPoints") }) else {
        return;
    };
    let clear_all_points: CLayoutFrameClearAllPointsFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_layout_frame_clear_all_points) };
    unsafe { clear_all_points(layout_frame, 1) };
    logging::info(&format!("[frames] BlzFrameClearAllPoints called on 0x{frame:x}"));
}

pub unsafe extern "C" fn blz_frame_show(frame: u32, show: u32) {
    if frame == 0 {
        return;
    }
    let frame_usize = frame as usize;
    if !unsafe { frame_registry::is_valid(frame_usize) } || frame_usize == get_game_ui() as usize {
        return;
    }
    unsafe { frame_show_hide(frame_usize, show != 0) };
    logging::info(&format!("[frames] BlzFrameShow called on 0x{frame:x} show={show}"));
}

pub unsafe extern "C" fn blz_frame_set_size(frame: u32, width_ptr: u32, height_ptr: u32) {
    if unsafe { frame_layout(frame, "BlzFrameSetSize") }.is_none() {
        return;
    }
    if width_ptr == 0 || height_ptr == 0 {
        logging::warn("[frames] BlzFrameSetSize: null width/height pointer");
        return;
    }
    let width = unsafe { *(width_ptr as *const f32) };
    let height = unsafe { *(height_ptr as *const f32) };
    unsafe {
        super::ops::set_width(frame as usize, width);
        super::ops::set_height(frame as usize, height);
    };
    logging::info(&format!(
        "[frames] BlzFrameSetSize called on 0x{frame:x} ({width}x{height})"
    ));
}

pub unsafe extern "C" fn blz_frame_set_abs_point(frame: u32, point: u32, x_ptr: u32, y_ptr: u32) {
    let Some(layout_frame) = (unsafe { frame_layout(frame, "BlzFrameSetAbsPoint") }) else {
        return;
    };
    if x_ptr == 0 || y_ptr == 0 {
        logging::warn("[frames] BlzFrameSetAbsPoint: null x/y pointer");
        return;
    }
    let x = unsafe { *(x_ptr as *const f32) };
    let y = unsafe { *(y_ptr as *const f32) };
    let set_abs_point: CLayoutFrameSetPointAbsFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_layout_frame_set_point_abs) };
    unsafe { set_abs_point(layout_frame, point as i32, x, y, 1) };
    logging::info(&format!(
        "[frames] BlzFrameSetAbsPoint called on 0x{frame:x} point={point} x={x} y={y}"
    ));
}

pub unsafe extern "C" fn blz_frame_set_point(
    frame: u32,
    point: u32,
    rel_frame: u32,
    rel_point: u32,
    x_ptr: u32,
    y_ptr: u32,
) {
    let Some(layout_frame) = (unsafe { frame_layout(frame, "BlzFrameSetPoint") }) else {
        return;
    };
    let Some(rel_layout_frame) = (unsafe { frame_layout(rel_frame, "BlzFrameSetPoint") }) else {
        return;
    };
    if x_ptr == 0 || y_ptr == 0 {
        logging::warn("[frames] BlzFrameSetPoint: null x/y pointer");
        return;
    }
    let x = unsafe { *(x_ptr as *const f32) };
    let y = unsafe { *(y_ptr as *const f32) };
    let set_point: CLayoutFrameSetPointFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_layout_frame_set_point) };
    unsafe {
        set_point(
            layout_frame,
            point as i32,
            rel_layout_frame,
            rel_point as i32,
            x,
            y,
            1,
        )
    };
    logging::info(&format!("[frames] BlzFrameSetPoint called on 0x{frame:x}"));
}

pub unsafe extern "C" fn blz_frame_set_text(frame: u32, text_handle: u32) {
    if frame == 0 {
        return;
    }
    let c_text = native_string_cstring(text_handle);
    if !unsafe { super::ops::set_text(frame as usize, c_text.as_ptr()) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetText unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_add_text(frame: u32, text_handle: u32) {
    if frame == 0 {
        return;
    }
    let c_text = native_string_cstring(text_handle);
    if !unsafe { super::ops::add_text(frame as usize, c_text.as_ptr()) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameAddText unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_simple_font_string_set_text(frame: u32, text_handle: u32) {
    if frame == 0 {
        return;
    }
    let c_text = native_string_cstring(text_handle);
    let set_text: CSimpleFontStringSetTextFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_simple_font_string_set_text) };
    unsafe { set_text(frame as usize, c_text.as_ptr()) };
    logging::info(&format!("[frames] BlzSimpleFontStringSetText called on 0x{frame:x}"));
}

pub unsafe extern "C" fn blz_text_frame_set_text(frame: u32, text_handle: u32) {
    if frame == 0 {
        return;
    }
    let c_text = native_string_cstring(text_handle);
    let set_text: CTextFrameSetTextFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_text_frame_set_text) };
    unsafe { set_text(frame as usize, c_text.as_ptr()) };
    logging::info(&format!("[frames] BlzTextFrameSetText called on 0x{frame:x}"));
}

pub unsafe extern "C" fn ce_convert_type(value: u32) -> u32 {
    value
}

unsafe fn additional_screen_width() -> f32 {
    let ui = get_game_ui() as usize;
    if ui == 0 {
        return 0.0;
    }

    let cursor = unsafe { ui_field(ui, CGAME_UI_CURSOR_FRAME) };
    if cursor == 0 {
        return 0.0;
    }

    0.0 - unsafe { read_f32(cursor + offsets::C_CURSOR_FRAME_ADDITIONAL_SCREEN_WIDTH) }
}

unsafe fn mouse_x_frame() -> f32 {
    let ui = get_game_ui() as usize;
    if ui == 0 {
        return 0.0;
    }

    let hero_bar = unsafe { ui_field(ui, CGAME_UI_HERO_BAR) };
    if hero_bar == 0 {
        return 0.0;
    }

    let simple_top = unsafe { read_usize(hero_bar + offsets::C_SIMPLE_LAYER_MASTER) };
    if simple_top == 0 {
        return 0.0;
    }

    let extra = unsafe { additional_screen_width() };
    let left = 0.0 - extra;
    let right = 0.8 + extra;
    let t = unsafe { read_f32(simple_top + offsets::C_SIMPLE_TOP_MOUSE_X_NORM) };
    left + (right - left) * t
}

unsafe fn mouse_y_frame() -> f32 {
    let ui = get_game_ui() as usize;
    if ui == 0 {
        return 0.0;
    }

    let hero_bar = unsafe { ui_field(ui, CGAME_UI_HERO_BAR) };
    if hero_bar == 0 {
        return 0.0;
    }

    let simple_top = unsafe { read_usize(hero_bar + offsets::C_SIMPLE_LAYER_MASTER) };
    if simple_top == 0 {
        return 0.0;
    }

    unsafe { read_f32(simple_top + offsets::C_SIMPLE_TOP_MOUSE_Y_NORM) * 0.6 }
}

pub unsafe extern "C" fn ce_get_additional_screen_width() -> u32 {
    unsafe { additional_screen_width().to_bits() }
}

pub unsafe extern "C" fn ce_get_frame_functional_child(frame: u32, index: u32) -> u32 {
    unsafe { super::ops::functional_child(frame as usize, index as i32).unwrap_or(0) as u32 }
}

pub unsafe extern "C" fn blz_frame_get_parent(frame: u32) -> u32 {
    unsafe { super::ops::parent(frame as usize).unwrap_or(0) as u32 }
}

pub unsafe extern "C" fn blz_frame_get_child(frame: u32, index: u32) -> u32 {
    unsafe { super::ops::child(frame as usize, index as i32).unwrap_or(0) as u32 }
}

pub unsafe extern "C" fn blz_frame_get_children_count(frame: u32) -> u32 {
    unsafe { super::ops::children_count(frame as usize) as u32 }
}

pub unsafe extern "C" fn blz_frame_get_width(frame: u32) -> u32 {
    unsafe { super::ops::width(frame as usize).to_bits() }
}

pub unsafe extern "C" fn blz_frame_get_height(frame: u32) -> u32 {
    unsafe { super::ops::height(frame as usize).to_bits() }
}

pub unsafe extern "C" fn blz_frame_get_x(frame: u32) -> u32 {
    let cursor = unsafe { origin_frame(35, 0) as u32 };
    if frame != 0 && frame == cursor {
        return unsafe { mouse_x_frame().to_bits() };
    }
    unsafe { super::ops::center_x(frame as usize).to_bits() }
}

pub unsafe extern "C" fn blz_frame_get_y(frame: u32) -> u32 {
    let cursor = unsafe { origin_frame(35, 0) as u32 };
    if frame != 0 && frame == cursor {
        return unsafe { mouse_y_frame().to_bits() };
    }
    unsafe { super::ops::center_y(frame as usize).to_bits() }
}

pub unsafe extern "C" fn blz_frame_set_width(frame: u32, width_ptr: u32) {
    if frame == 0 || width_ptr == 0 {
        return;
    }
    let width = unsafe { *(width_ptr as *const f32) };
    if !unsafe { super::ops::set_width(frame as usize, width) } {
        logging::warn(&format!("[frames] BlzFrameSetWidth failed on 0x{frame:x}"));
    }
}

pub unsafe extern "C" fn blz_frame_set_height(frame: u32, height_ptr: u32) {
    if frame == 0 || height_ptr == 0 {
        return;
    }
    let height = unsafe { *(height_ptr as *const f32) };
    if !unsafe { super::ops::set_height(frame as usize, height) } {
        logging::warn(&format!("[frames] BlzFrameSetHeight failed on 0x{frame:x}"));
    }
}

pub unsafe extern "C" fn blz_frame_set_level(frame: u32, level: u32) {
    if !unsafe { super::ops::set_level(frame as usize, level as i32) } {
        logging::warn(&format!("[frames] BlzFrameSetLevel failed on 0x{frame:x}"));
    }
}

pub unsafe extern "C" fn blz_frame_set_parent(frame: u32, parent: u32) {
    if !unsafe { super::ops::set_parent(frame as usize, parent as usize) } {
        logging::warn(&format!(
            "[frames] BlzFrameSetParent failed frame=0x{frame:x} parent=0x{parent:x}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_set_tooltip(frame: u32, tooltip: u32) {
    if !unsafe { super::ops::set_tooltip(frame as usize, tooltip as usize) } {
        logging::warn(&format!(
            "[frames] BlzFrameSetTooltip failed frame=0x{frame:x} tooltip=0x{tooltip:x}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_set_script(frame: u32, event_id: u32, callback_id: u32) {
    logging::info(&format!(
        "[frames] BlzFrameSetScript entry: frame=0x{frame:x} event={event_id} cb=0x{callback_id:x}"
    ));

    if frame == 0 || callback_id == 0 {
        logging::warn(&format!(
            "[frames] BlzFrameSetScript skipped: frame=0x{frame:x} cb=0x{callback_id:x}"
        ));
        return;
    }

    if !unsafe { frame_registry::is_valid(frame as usize) } {
        let vtable = unsafe { frame_registry::vtable(frame as usize) }.unwrap_or(0);
        let off = unsafe { frame_registry::vtable_static_key(frame as usize) }.unwrap_or(0);
        logging::warn(&format!(
            "[frames] BlzFrameSetScript: unknown frame vtable frame=0x{frame:x} vtable=0x{vtable:x} static_key=0x{off:x}; registering anyway"
        ));
    }

    super::events::register_event(frame as usize, event_id, callback_id);

    let register_event: CObserverRegisterEventFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_observer_register_event) };

    let parent_observer = unsafe {
        let p = frame_registry::parent(frame as usize);
        if p == 0 {
            get_game_ui() as *mut c_void
        } else {
            p as *mut c_void
        }
    };

    unsafe {
        register_event(
            frame as *mut c_void,
            event_id,
            event_id,
            parent_observer,
        )
    };

    logging::info(&format!(
        "[frames] BlzFrameSetScript: frame=0x{frame:x} event={event_id} cb={callback_id}"
    ));
}

pub unsafe extern "C" fn blz_destroy_frame(frame: u32) {
    if frame == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }
    if unsafe { frame_registry::is_simple(frame as usize) } {
        unsafe { frame_show_hide(frame as usize, false) };
    } else {
        let destroy: CFrameDestroyFn = unsafe { core::mem::transmute(addresses::get().frames.c_frame_destroy) };
        unsafe { destroy(frame as usize, 1) };
    }
    logging::info(&format!("[frames] BlzDestroyFrame called on 0x{frame:x}"));
}

pub unsafe extern "C" fn blz_frame_set_alpha(frame: u32, alpha: u32) {
    if frame == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }
    let alpha = alpha as i32;
    if unsafe { frame_registry::is_simple(frame as usize) } {
        let set_alpha: CSimpleFrameSetAlphaFn =
            unsafe { core::mem::transmute(addresses::get().frames.c_simple_frame_set_alpha) };
        unsafe { set_alpha(frame as usize, alpha) };
    } else if unsafe { frame_registry::get_layout(frame as usize) } != Some(frame as usize) {
        let set_alpha: CLayerSetAlphaFn =
            unsafe { core::mem::transmute(addresses::get().frames.c_layer_set_alpha) };
        unsafe { set_alpha(frame as usize, alpha, 0) };
    }
    logging::info(&format!("[frames] BlzFrameSetAlpha called on 0x{frame:x} alpha={alpha}"));
}

pub unsafe extern "C" fn blz_frame_set_vertex_color(frame: u32, color: u32) {
    if !unsafe { super::ops::set_vertex_color(frame as usize, color) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetVertexColor unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_set_tex_coord(
    frame: u32,
    left_ptr: u32,
    right_ptr: u32,
    up_ptr: u32,
    down_ptr: u32,
) {
    if frame == 0 || left_ptr == 0 || right_ptr == 0 || up_ptr == 0 || down_ptr == 0 {
        return;
    }
    let left = unsafe { *(left_ptr as *const f32) };
    let right = unsafe { *(right_ptr as *const f32) };
    let up = unsafe { *(up_ptr as *const f32) };
    let down = unsafe { *(down_ptr as *const f32) };
    if !unsafe { super::ops::set_tex_coord(frame as usize, left, right, up, down) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetTexCoord unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_set_alpha_mode(frame: u32, mode: u32) {
    if !unsafe { super::ops::set_alpha_mode(frame as usize, mode as i32) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetAlphaMode unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_get_alpha_mode(frame: u32) -> u32 {
    unsafe { super::ops::alpha_mode(frame as usize).unwrap_or(0) as u32 }
}

pub unsafe extern "C" fn blz_frame_set_backdrop_mirrored(frame: u32, enabled: u32) {
    if !unsafe { super::ops::set_backdrop_mirrored(frame as usize, enabled != 0) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetBackdropMirrored unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_get_backdrop_mirrored(frame: u32) -> u32 {
    unsafe { if super::ops::backdrop_mirrored(frame as usize).unwrap_or(false) { 1 } else { 0 } }
}

pub unsafe extern "C" fn blz_frame_set_backdrop_tile_size(frame: u32, size_ptr: u32) {
    if frame == 0 || size_ptr == 0 {
        return;
    }
    let size = unsafe { *(size_ptr as *const f32) };
    if !unsafe { super::ops::set_backdrop_tile_size(frame as usize, size) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetBackdropTileSize unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_get_backdrop_tile_size(frame: u32) -> u32 {
    unsafe { super::ops::backdrop_tile_size(frame as usize).unwrap_or(0.0).to_bits() }
}

pub unsafe extern "C" fn blz_frame_set_backdrop_border_size(frame: u32, size_ptr: u32) {
    if frame == 0 || size_ptr == 0 {
        return;
    }
    let size = unsafe { *(size_ptr as *const f32) };
    if !unsafe { super::ops::set_backdrop_border_size(frame as usize, size) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetBackdropBorderSize unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_get_backdrop_border_size(frame: u32) -> u32 {
    unsafe { super::ops::backdrop_border_size(frame as usize).unwrap_or(0.0).to_bits() }
}

pub unsafe extern "C" fn blz_frame_set_backdrop_border_flag(frame: u32, flag: u32) {
    if !unsafe { super::ops::set_backdrop_border_flag(frame as usize, flag as i32) } {
        let ft = unsafe { frame_registry::frame_type(frame as usize) };
        logging::warn(&format!(
            "[frames] BlzFrameSetBackdropBorderFlag unsupported frame=0x{frame:x} type={ft:?}"
        ));
    }
}

pub unsafe extern "C" fn blz_frame_get_backdrop_border_flag(frame: u32) -> u32 {
    unsafe { super::ops::backdrop_border_flag(frame as usize).unwrap_or(0) as u32 }
}

pub unsafe extern "C" fn blz_frame_add_backdrop_border_flag(frame: u32, flag: u32) {
    let current = unsafe { super::ops::backdrop_border_flag(frame as usize).unwrap_or(0) };
    let _ = unsafe { super::ops::set_backdrop_border_flag(frame as usize, current | flag as i32) };
}

pub unsafe extern "C" fn blz_frame_remove_backdrop_border_flag(frame: u32, flag: u32) {
    let current = unsafe { super::ops::backdrop_border_flag(frame as usize).unwrap_or(0) };
    let _ = unsafe { super::ops::set_backdrop_border_flag(frame as usize, current & !(flag as i32)) };
}

pub unsafe extern "C" fn ce_set_frame_text_color_ex(frame: u32, color: u32, state_index: u32) {
    if frame == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }

    let frame_usize = frame as usize;
    let ft = unsafe { frame_registry::frame_type(frame_usize) };
    match ft {
        FrameType::CTextFrame | FrameType::CTimerTextFrame | FrameType::CListBoxItem => {
            match state_index {
                1 => unsafe { write_i32(frame_usize + offsets::C_TEXT_FRAME_COLOR_HIGHLIGHT, color as i32) },
                2 => unsafe { write_i32(frame_usize + offsets::C_TEXT_FRAME_COLOR_DISABLED, color as i32) },
                _ => unsafe { blz_frame_set_text_color(frame, color) },
            }
        }
        FrameType::CGluePopupMenuWar3 | FrameType::CPopupMenu => {
            let child = unsafe { read_usize(frame_usize + offsets::C_POPUP_MENU_TEXT_BUTTON) };
            if child != 0 {
                unsafe { ce_set_frame_text_color_ex(child as u32, color, state_index) };
            }
        }
        FrameType::CTextButtonFrame | FrameType::CGlueTextButtonWar3 => {
            let child = unsafe { read_usize(frame_usize + offsets::C_TEXT_BUTTON_TEXT_FRAME) };
            if child != 0 {
                unsafe { ce_set_frame_text_color_ex(child as u32, color, state_index) };
            }
        }
        FrameType::CSimpleButton => {
            let child_index = state_index.saturating_add(3);
            let child = unsafe { super::ops::functional_child(frame_usize, child_index as i32) };
            if let Some(child) = child {
                let _ = unsafe { super::ops::set_vertex_color(child, color) };
            }
        }
        FrameType::CSimpleFontString => {
            let _ = unsafe { super::ops::set_vertex_color(frame_usize, color) };
        }
        _ => {
            logging::warn(&format!(
                "[frames] SetFrameTextColorEx unsupported frame=0x{frame:x} type={ft:?} state={state_index}"
            ));
        }
    }
}

pub unsafe extern "C" fn blz_frame_set_text_color(frame: u32, color: u32) {
    if frame == 0 || !unsafe { frame_registry::is_valid(frame as usize) } {
        return;
    }
    let set_color: CTextFrameSetTextColorFn =
        unsafe { core::mem::transmute(addresses::get().frames.c_text_frame_set_text_color) };
    unsafe { set_color(frame as usize, &color as *const u32) };
    logging::info(&format!("[frames] BlzFrameSetTextColor called on 0x{frame:x}"));
}

pub fn register_custom_natives() {
    let natives = [
        ("BlzGetGameUI", "()I", blz_get_game_ui as *const c_void),
        ("BlzCreateFrame", "(SIII)I", blz_create_frame as *const c_void),
        ("BlzCreateSimpleFrame", "(SII)I", blz_create_simple_frame as *const c_void),
        ("BlzGetOriginFrame", "(II)I", blz_get_origin_frame as *const c_void),
        ("BlzFrameGetByName", "(SI)I", blz_frame_get_by_name as *const c_void),
        ("TriggerRegisterFrameEvent", "(III)I", super::trigger_events::trigger_register_frame_event as *const c_void),
        ("BlzTriggerRegisterFrameEvent", "(III)I", super::trigger_events::trigger_register_frame_event as *const c_void),
        ("GetTriggerFrame", "()I", super::trigger_events::get_trigger_frame as *const c_void),
        ("BlzGetTriggerFrame", "()I", super::trigger_events::get_trigger_frame as *const c_void),
        ("GetTriggerFrameEvent", "()I", super::trigger_events::get_trigger_frame_event as *const c_void),
        ("BlzGetTriggerFrameEvent", "()I", super::trigger_events::get_trigger_frame_event as *const c_void),
        ("GetTriggerFrameValue", "()R", super::trigger_events::get_trigger_frame_value as *const c_void),
        ("BlzGetTriggerFrameValue", "()R", super::trigger_events::get_trigger_frame_value as *const c_void),
        ("GetTriggerFrameText", "()S", super::trigger_events::get_trigger_frame_text as *const c_void),
        ("BlzGetTriggerFrameText", "()S", super::trigger_events::get_trigger_frame_text as *const c_void),
        ("BlzFrameGetName", "(I)S", blz_frame_get_name as *const c_void),
        ("GetMouseHoveredFrame", "()I", blz_get_mouse_hovered_frame as *const c_void),
        ("BlzHideOriginFrames", "(B)V", blz_hide_origin_frames as *const c_void),
        ("HideOriginFrames", "(B)V", blz_hide_origin_frames as *const c_void),
        ("GetCommandButtonData", "(II)I", ce_get_command_button_data as *const c_void),
        ("BlzFrameSetTexture", "(ISII)V", blz_frame_set_texture as *const c_void),
        ("BlzLoadTOCFile", "(S)I", blz_load_toc_file as *const c_void),
        ("BlzFrameSetAllPoints", "(II)V", blz_frame_set_all_points as *const c_void),
        ("BlzFrameClearAllPoints", "(I)V", blz_frame_clear_all_points as *const c_void),
        ("BlzFrameShow", "(II)V", blz_frame_show as *const c_void),
        ("BlzFrameIsVisible", "(I)B", blz_frame_is_visible as *const c_void),
        ("BlzFrameGetAlpha", "(I)I", blz_frame_get_alpha as *const c_void),
        ("BlzFrameSetScale", "(IR)V", blz_frame_set_scale as *const c_void),
        ("BlzFrameGetScale", "(I)R", blz_frame_get_scale as *const c_void),
        ("BlzFrameSetEnable", "(IB)V", blz_frame_set_enable as *const c_void),
        ("BlzFrameGetEnable", "(I)B", blz_frame_get_enable as *const c_void),
        ("BlzFrameSetFocus", "(IB)V", blz_frame_set_focus as *const c_void),
        ("BlzFrameSetTextAlignment", "(III)V", blz_frame_set_text_alignment as *const c_void),
        ("BlzFrameCageMouse", "(IB)V", blz_frame_cage_mouse as *const c_void),
        ("BlzFrameSetSize", "(IRR)V", blz_frame_set_size as *const c_void),
        ("BlzFrameSetWidth", "(IR)V", blz_frame_set_width as *const c_void),
        ("BlzFrameSetHeight", "(IR)V", blz_frame_set_height as *const c_void),
        ("BlzFrameSetLevel", "(II)V", blz_frame_set_level as *const c_void),
        ("BlzFrameSetParent", "(II)V", blz_frame_set_parent as *const c_void),
        ("BlzFrameSetTooltip", "(II)V", blz_frame_set_tooltip as *const c_void),
        ("BlzFrameSetAbsPoint", "(IIRR)V", blz_frame_set_abs_point as *const c_void),
        ("BlzFrameSetPoint", "(IIIIIRR)V", blz_frame_set_point as *const c_void),
        ("BlzFrameSetText", "(IS)V", blz_frame_set_text as *const c_void),
        ("BlzFrameAddText", "(IS)V", blz_frame_add_text as *const c_void),
        ("BlzFrameSetFont", "(ISRI)V", blz_frame_set_font as *const c_void),
        ("BlzFrameGetText", "(I)S", blz_frame_get_text as *const c_void),
        ("BlzFrameGetTextSizeLimit", "(I)I", blz_frame_get_text_size_limit as *const c_void),
        ("BlzFrameSetTextSizeLimit", "(II)V", blz_frame_set_text_size_limit as *const c_void),
        ("BlzFrameClick", "(I)V", blz_frame_click as *const c_void),
        ("BlzFrameSetValue", "(IR)V", blz_frame_set_value as *const c_void),
        ("BlzFrameGetValue", "(I)R", blz_frame_get_value as *const c_void),
        ("BlzFrameSetMinMaxValue", "(IRR)V", blz_frame_set_min_max_value as *const c_void),
        ("BlzFrameSetStepSize", "(IR)V", blz_frame_set_step_size as *const c_void),
        ("BlzFrameSetModel", "(ISII)V", blz_frame_set_model as *const c_void),
        ("BlzFrameSetSpriteAnimate", "(III)V", blz_frame_set_sprite_animate as *const c_void),
        ("FrameSetFont", "(ISRI)V", blz_frame_set_font as *const c_void),
        ("FrameGetText", "(I)S", blz_frame_get_text as *const c_void),
        ("FrameGetTextSizeLimit", "(I)I", blz_frame_get_text_size_limit as *const c_void),
        ("FrameSetTextSizeLimit", "(II)V", blz_frame_set_text_size_limit as *const c_void),
        ("FrameClick", "(I)V", blz_frame_click as *const c_void),
        ("FrameSetValue", "(IR)V", blz_frame_set_value as *const c_void),
        ("FrameGetValue", "(I)R", blz_frame_get_value as *const c_void),
        ("FrameSetMinMaxValue", "(IRR)V", blz_frame_set_min_max_value as *const c_void),
        ("FrameSetStepSize", "(IR)V", blz_frame_set_step_size as *const c_void),
        ("FrameSetModel", "(ISII)V", blz_frame_set_model as *const c_void),
        ("FrameSetSpriteAnimate", "(III)V", blz_frame_set_sprite_animate as *const c_void),
        ("BlzSimpleFontStringSetText", "(IS)V", blz_simple_font_string_set_text as *const c_void),
        ("BlzTextFrameSetText", "(IS)V", blz_text_frame_set_text as *const c_void),
        ("BlzFrameSetScript", "(IIC)V", blz_frame_set_script as *const c_void),
        ("BlzFrameGetParent", "(I)I", blz_frame_get_parent as *const c_void),
        ("BlzFrameGetChild", "(II)I", blz_frame_get_child as *const c_void),
        ("BlzFrameGetChildrenCount", "(I)I", blz_frame_get_children_count as *const c_void),
        ("BlzFrameGetWidth", "(I)R", blz_frame_get_width as *const c_void),
        ("BlzFrameGetHeight", "(I)R", blz_frame_get_height as *const c_void),
        ("BlzFrameGetX", "(I)R", blz_frame_get_x as *const c_void),
        ("BlzFrameGetY", "(I)R", blz_frame_get_y as *const c_void),
        ("BlzDestroyFrame", "(I)V", blz_destroy_frame as *const c_void),
        ("BlzFrameSetAlpha", "(II)V", blz_frame_set_alpha as *const c_void),
        ("BlzFrameSetVertexColor", "(II)V", blz_frame_set_vertex_color as *const c_void),
        ("BlzFrameSetTextColor", "(II)V", blz_frame_set_text_color as *const c_void),
        ("BlzFrameSetTexCoord", "(IRRRR)V", blz_frame_set_tex_coord as *const c_void),
        ("BlzFrameSetAlphaMode", "(II)V", blz_frame_set_alpha_mode as *const c_void),
        ("BlzFrameGetAlphaMode", "(I)I", blz_frame_get_alpha_mode as *const c_void),
        ("BlzFrameSetBackdropMirrored", "(IB)V", blz_frame_set_backdrop_mirrored as *const c_void),
        ("BlzFrameGetBackdropMirrored", "(I)B", blz_frame_get_backdrop_mirrored as *const c_void),
        ("BlzFrameSetBackdropTileSize", "(IR)V", blz_frame_set_backdrop_tile_size as *const c_void),
        ("BlzFrameGetBackdropTileSize", "(I)R", blz_frame_get_backdrop_tile_size as *const c_void),
        ("BlzFrameSetBackdropBorderSize", "(IR)V", blz_frame_set_backdrop_border_size as *const c_void),
        ("BlzFrameGetBackdropBorderSize", "(I)R", blz_frame_get_backdrop_border_size as *const c_void),
        ("BlzFrameSetBackdropBorderFlag", "(II)V", blz_frame_set_backdrop_border_flag as *const c_void),
        ("BlzFrameGetBackdropBorderFlag", "(I)I", blz_frame_get_backdrop_border_flag as *const c_void),
        ("GetFrameCreateContext", "(I)I", ce_get_frame_create_context as *const c_void),
        ("SetFrameCreateContext", "(II)V", ce_set_frame_create_context as *const c_void),
        ("GetFrameLayerStyle", "(I)I", ce_get_frame_layer_style as *const c_void),
        ("SetFrameLayerStyle", "(II)V", ce_set_frame_layer_style as *const c_void),
        ("GetFrameControlStyle", "(I)I", ce_get_frame_control_style as *const c_void),
        ("SetFrameControlStyle", "(II)V", ce_set_frame_control_style as *const c_void),
        ("GetFrameScale", "(I)R", blz_frame_get_scale as *const c_void),
        ("GetFrameX", "(I)R", blz_frame_get_x as *const c_void),
        ("GetFrameY", "(I)R", blz_frame_get_y as *const c_void),
        ("GetFrameAlphaMode", "(I)I", blz_frame_get_alpha_mode as *const c_void),
        ("SetFrameAlphaMode", "(II)V", blz_frame_set_alpha_mode as *const c_void),
        ("GetFrameBackdropBorderFlag", "(I)I", blz_frame_get_backdrop_border_flag as *const c_void),
        ("SetFrameBackdropBorderFlag", "(II)V", blz_frame_set_backdrop_border_flag as *const c_void),
        ("AddFrameBackdropBorderFlag", "(II)V", blz_frame_add_backdrop_border_flag as *const c_void),
        ("RemoveFrameBackdropBorderFlag", "(II)V", blz_frame_remove_backdrop_border_flag as *const c_void),
        ("GetFrameBackdropMirrored", "(I)B", blz_frame_get_backdrop_mirrored as *const c_void),
        ("SetFrameBackdropMirrored", "(IB)V", blz_frame_set_backdrop_mirrored as *const c_void),
        ("GetFrameBackdropBorderSize", "(I)R", blz_frame_get_backdrop_border_size as *const c_void),
        ("SetFrameBackdropBorderSize", "(IR)V", blz_frame_set_backdrop_border_size as *const c_void),
        ("GetFrameBackdropTileSize", "(I)R", blz_frame_get_backdrop_tile_size as *const c_void),
        ("SetFrameBackdropTileSize", "(IR)V", blz_frame_set_backdrop_tile_size as *const c_void),
        ("GetAdditionalScreenWidth", "()R", ce_get_additional_screen_width as *const c_void),
        ("GetFrameFunctionalChild", "(II)I", ce_get_frame_functional_child as *const c_void),
        ("SetFrameTextColorEx", "(III)V", ce_set_frame_text_color_ex as *const c_void),
        ("SetFrameTexCoord", "(IRRRR)V", blz_frame_set_tex_coord as *const c_void),
        ("ConvertLayerStyleFlag", "(I)I", ce_convert_type as *const c_void),
        ("ConvertControlStyleFlag", "(I)I", ce_convert_type as *const c_void),
        ("ConvertBackdropBorderFlag", "(I)I", ce_convert_type as *const c_void),
        ("ConvertFrameAlphaMode", "(I)I", ce_convert_type as *const c_void),
        ("GetFrameEnable", "(I)B", blz_frame_get_enable as *const c_void),
        ("SetFrameEnable", "(IB)V", blz_frame_set_enable as *const c_void),
        ("IsFrameEnabled", "(I)B", blz_frame_get_enable as *const c_void),
        ("BlzFrameAddBackdropBorderFlag", "(II)V", blz_frame_add_backdrop_border_flag as *const c_void),
        ("BlzFrameRemoveBackdropBorderFlag", "(II)V", blz_frame_remove_backdrop_border_flag as *const c_void),
        ("CreateFrame", "(SIII)I", blz_create_frame as *const c_void),
        ("CreateSimpleFrame", "(SII)I", blz_create_simple_frame as *const c_void),
        ("GetOriginFrame", "(II)I", blz_get_origin_frame as *const c_void),
        ("GetFrameByName", "(SI)I", blz_frame_get_by_name as *const c_void),
    ];
    for (name, sig, func) in natives {
        let c_name = CString::new(name).unwrap();
        let c_sig = CString::new(sig).unwrap();
        match engines::request_plugin_native(c_name, c_sig, func) {
            Ok(_) => logging::info(&format!("frames: queued {name} for registration")),
            Err(e) => logging::error(&format!("frames: failed to queue {name}: {e}")),
        }
    }
}
