use super::frame_registry;
use super::frame_type::FrameType;
use super::offsets;
use crate::memory::{read_f32, read_i32, read_usize, write_f32, write_i32, write_u32, write_usize};

const C_FRAME_CHILDREN: usize = 0x1C;
const C_FRAME_POS_BOTTOM: usize = 0x100;
const C_FRAME_POS_LEFT: usize = 0x104;
const C_FRAME_POS_TOP: usize = 0x108;
const C_FRAME_POS_RIGHT: usize = 0x10C;

const C_LAYOUT_WIDTH: usize = 0x58;
const C_LAYOUT_HEIGHT: usize = 0x5C;

const C_SIMPLE_POS_BOTTOM: usize = 0x44;
const C_SIMPLE_POS_LEFT: usize = 0x48;
const C_SIMPLE_POS_TOP: usize = 0x4C;
const C_SIMPLE_POS_RIGHT: usize = 0x50;
const C_SIMPLE_CHILDREN: usize = 0x124;



pub unsafe fn is_valid(frame: usize) -> bool {
    unsafe { frame_registry::is_valid(frame) }
}

pub unsafe fn parent(frame: usize) -> Option<usize> {
    if !unsafe { is_valid(frame) } {
        return None;
    }

    let parent = unsafe { frame_registry::parent(frame) };
    if parent == 0 || !unsafe { is_valid(parent) } {
        None
    } else {
        Some(parent)
    }
}

unsafe fn normal_children_head(frame: usize) -> usize {
    let children = unsafe { read_usize(frame + C_FRAME_CHILDREN) };
    if children & 1 == 0 { children } else { 0 }
}

unsafe fn simple_children_head(frame: usize) -> usize {
    let children = unsafe { read_usize(frame + C_SIMPLE_CHILDREN) };
    if children & 1 == 0 { children } else { 0 }
}

pub unsafe fn child(frame: usize, index: i32) -> Option<usize> {
    if index < 0 || !unsafe { is_valid(frame) } {
        return None;
    }

    let simple = unsafe { frame_registry::is_simple(frame) };
    let mut node = if simple {
        unsafe { simple_children_head(frame) }
    } else {
        unsafe { normal_children_head(frame) }
    };

    let mut current = 0i32;
    while node != 0 && node & 1 == 0 {
        let child = if simple {
            unsafe { read_usize(node + 0x08) }
        } else {
            unsafe { read_usize(node + 0x0C) }
        };

        if child == 0 {
            return None;
        }

        if current == index {
            return if unsafe { is_valid(child) } { Some(child) } else { None };
        }

        node = if simple {
            unsafe { read_usize(node + 0x04) }
        } else {
            unsafe { read_usize(node + 0x08) }
        };
        current += 1;
    }

    None
}

pub unsafe fn children_count(frame: usize) -> i32 {
    if !unsafe { is_valid(frame) } {
        return 0;
    }

    let simple = unsafe { frame_registry::is_simple(frame) };
    let mut node = if simple {
        unsafe { simple_children_head(frame) }
    } else {
        unsafe { normal_children_head(frame) }
    };

    let mut count = 0i32;
    while node != 0 && node & 1 == 0 {
        let child = if simple {
            unsafe { read_usize(node + 0x08) }
        } else {
            unsafe { read_usize(node + 0x0C) }
        };

        if child == 0 {
            break;
        }

        count += 1;
        node = if simple {
            unsafe { read_usize(node + 0x04) }
        } else {
            unsafe { read_usize(node + 0x08) }
        };
    }

    count
}

pub unsafe fn width(frame: usize) -> f32 {
    let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
        return 0.0;
    };
    unsafe { read_f32(layout + C_LAYOUT_WIDTH) }
}

pub unsafe fn height(frame: usize) -> f32 {
    let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
        return 0.0;
    };
    unsafe { read_f32(layout + C_LAYOUT_HEIGHT) }
}

pub unsafe fn center_x(frame: usize) -> f32 {
    if !unsafe { is_valid(frame) } {
        return 0.0;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    let (left, right) = if unsafe { frame_registry::is_simple(frame) } {
        match ft {
            FrameType::CSimpleFontString | FrameType::CSimpleTexture => (
                unsafe { read_f32(frame + C_SIMPLE_POS_LEFT) },
                unsafe { read_f32(frame + C_SIMPLE_POS_RIGHT) },
            ),
            _ => (
                unsafe { read_f32(frame + C_SIMPLE_POS_LEFT) },
                unsafe { read_f32(frame + C_SIMPLE_POS_RIGHT) },
            ),
        }
    } else {
        (
            unsafe { read_f32(frame + C_FRAME_POS_LEFT) },
            unsafe { read_f32(frame + C_FRAME_POS_RIGHT) },
        )
    };

    left + (right - left) / 2.0
}

pub unsafe fn center_y(frame: usize) -> f32 {
    if !unsafe { is_valid(frame) } {
        return 0.0;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    let (bottom, top) = if unsafe { frame_registry::is_simple(frame) } {
        match ft {
            FrameType::CSimpleFontString | FrameType::CSimpleTexture => (
                unsafe { read_f32(frame + C_SIMPLE_POS_BOTTOM) },
                unsafe { read_f32(frame + C_SIMPLE_POS_TOP) },
            ),
            _ => (
                unsafe { read_f32(frame + C_SIMPLE_POS_BOTTOM) },
                unsafe { read_f32(frame + C_SIMPLE_POS_TOP) },
            ),
        }
    } else {
        (
            unsafe { read_f32(frame + C_FRAME_POS_BOTTOM) },
            unsafe { read_f32(frame + C_FRAME_POS_TOP) },
        )
    };

    bottom + (top - bottom) / 2.0
}

pub unsafe fn functional_child(frame: usize, index: i32) -> Option<usize> {
    if index < 0 || !unsafe { is_valid(frame) } {
        return None;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    if unsafe { frame_registry::is_simple(frame) } {
        let child = match ft {
            FrameType::CCommandButton
            | FrameType::CHeroBarButton
            | FrameType::CSimpleButton
            | FrameType::CTrainableButton
            | FrameType::CShrinkingButton
            | FrameType::CReplayButton
            | FrameType::CSimpleCheckbox => match index {
                0 => unsafe { read_usize(frame + offsets::C_SIMPLE_BUTTON_TEXTURE_DEFAULT) },
                1 => unsafe { read_usize(frame + offsets::C_SIMPLE_BUTTON_TEXTURE_DISABLED) },
                2 => unsafe { read_usize(frame + offsets::C_SIMPLE_BUTTON_TEXTURE_PUSHED) },
                3 => unsafe { read_usize(frame + offsets::C_SIMPLE_BUTTON_TEXT_ENABLED) },
                4 => unsafe { read_usize(frame + offsets::C_SIMPLE_BUTTON_TEXT_DISABLED) },
                5 => unsafe { read_usize(frame + offsets::C_SIMPLE_BUTTON_TEXT_HIGHLIGHT) },
                _ => 0,
            },
            FrameType::CStatBar
            | FrameType::CSimpleStatusBar
            | FrameType::CProgressIndicator
            | FrameType::CBuildTimeIndicator => unsafe {
                read_usize(frame + offsets::C_SIMPLE_STATUS_BAR_TEXTURE)
            },
            _ => 0,
        };
        return if child != 0 && unsafe { is_valid(child) } { Some(child) } else { None };
    }

    if super::frame_inherits_from_control(ft, false) {
        let child = match index {
            0 => unsafe { read_usize(frame + offsets::C_CONTROL_BACKDROP_DEFAULT) },
            1 => unsafe { read_usize(frame + offsets::C_CONTROL_BACKDROP_DISABLED) },
            2 => unsafe { read_usize(frame + offsets::C_CONTROL_BACKDROP_PUSHED) },
            3 => unsafe { read_usize(frame + offsets::C_CONTROL_HIGHLIGHT_FOCUS) },
            4 => unsafe { read_usize(frame + offsets::C_CONTROL_HIGHLIGHT_HOVER) },
            _ => 0,
        };
        if child != 0 && unsafe { is_valid(child) } {
            return Some(child);
        }
    }

    let child = match ft {
        FrameType::CTextButtonFrame | FrameType::CGlueTextButtonWar3 => unsafe {
            read_usize(frame + offsets::C_TEXT_BUTTON_TEXT_FRAME)
        },
        FrameType::CGlueCheckBoxWar3 | FrameType::CCheckBox => match index {
            5 => unsafe { read_usize(frame + offsets::C_CHECK_BOX_FRAME_CHECKED) },
            6 => unsafe { read_usize(frame + offsets::C_CHECK_BOX_FRAME_CHECKED_DISABLED) },
            _ => 0,
        },
        FrameType::CEditBox
        | FrameType::CGlueEditBoxWar3
        | FrameType::CSlashChatBox
        | FrameType::CBattleNetChatEditBox
        | FrameType::CChatEditBox => match index {
            5 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_SCROLL_FRAME) },
            6 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_TEXT_FRAME_CONTENT) },
            7 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_TEXT_FRAME_B) },
            8 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_TEXT_FRAME_C) },
            9 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_TEXT_FRAME_D) },
            10 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_TEXT_FRAME_E) },
            11 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_HIGHLIGHT_A) },
            12 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_HIGHLIGHT_B) },
            13 => unsafe { read_usize(frame + offsets::C_EDIT_BOX_HIGHLIGHT_C) },
            _ => 0,
        },
        FrameType::CGluePopupMenuWar3 | FrameType::CPopupMenu => match index {
            5 => unsafe { read_usize(frame + offsets::C_POPUP_MENU_TEXT_BUTTON) },
            6 => unsafe { read_usize(frame + offsets::C_POPUP_MENU_BUTTON) },
            7 => unsafe { read_usize(frame + offsets::C_POPUP_MENU_MENU) },
            _ => 0,
        },
        FrameType::CSlider => unsafe { read_usize(frame + offsets::C_SLIDER_BUTTON) },
        FrameType::CScrollBar => match index {
            5 => unsafe { read_usize(frame + offsets::C_SLIDER_BUTTON) },
            6 => unsafe { read_usize(frame + offsets::C_SCROLL_BAR_BUTTON_A) },
            7 => unsafe { read_usize(frame + offsets::C_SCROLL_BAR_BUTTON_B) },
            _ => 0,
        },
        FrameType::CRadioGroup => unsafe { read_usize(frame + offsets::C_RADIO_GROUP_BUTTON) },
        FrameType::CAllianceDialog
        | FrameType::CDialogWar3
        | FrameType::CDialog
        | FrameType::CBattleNetClanInviteDialog
        | FrameType::CBattleNetConnectDialog
        | FrameType::CBattleNetCustomFilterDialog
        | FrameType::CBattleNetHelpDialog
        | FrameType::CBattleNetIconSelectDialog
        | FrameType::CBattleNetPatchDialog
        | FrameType::CBattleNetScheduledGame
        | FrameType::CBattleNetTeamInviteDialog
        | FrameType::CGameResultDialog
        | FrameType::CGameSaveSplashDialog
        | FrameType::COptionsConfirmDialog
        | FrameType::CQuickReplayConfirmDialog
        | FrameType::CQuickReplayDialog
        | FrameType::CScriptDialog
        | FrameType::CSuspendDialog
        | FrameType::CUnresponsiveDialog => unsafe {
            read_usize(frame + offsets::C_DIALOG_BUTTON)
        },
        _ => 0,
    };

    if child != 0 && unsafe { is_valid(child) } {
        Some(child)
    } else {
        None
    }
}


const C_FRAME_LEVEL: usize = 0xB0;
const C_SIMPLE_TOOLTIP: usize = 0x74;

type CLayoutFrameSetWidthFn = unsafe extern "thiscall" fn(usize, f32) -> i32;
type CLayoutFrameSetHeightFn = unsafe extern "thiscall" fn(usize, f32) -> i32;
type CSimpleFrameSetFrameLevelFn = unsafe extern "thiscall" fn(usize, i32) -> i32;
type CSimpleFrameSetParentFn = unsafe extern "thiscall" fn(usize, usize) -> i32;
type CLayerSetOwnerFn = unsafe extern "thiscall" fn(usize, usize, i32, i32) -> i32;
type CLayerSetTooltipFn = unsafe extern "thiscall" fn(usize, usize);



pub unsafe fn set_width(frame: usize, value: f32) -> bool {
    let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
        return false;
    };
    let f: CLayoutFrameSetWidthFn =
        unsafe { core::mem::transmute(crate::addresses::get().frames.c_layout_frame_set_width) };
    unsafe { f(layout, value) };
    true
}

pub unsafe fn set_height(frame: usize, value: f32) -> bool {
    let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
        return false;
    };
    let f: CLayoutFrameSetHeightFn =
        unsafe { core::mem::transmute(crate::addresses::get().frames.c_layout_frame_set_height) };
    unsafe { f(layout, value) };
    true
}

pub unsafe fn set_level(frame: usize, level: i32) -> bool {
    if !unsafe { is_valid(frame) } {
        return false;
    }

    if unsafe { frame_registry::is_simple(frame) } {
        let f: CSimpleFrameSetFrameLevelFn =
            unsafe { core::mem::transmute(crate::addresses::get().frames.c_simple_frame_set_level) };
        unsafe { f(frame, level) };
    } else {
        unsafe { write_i32(frame + C_FRAME_LEVEL, level) };
    }

    true
}

pub unsafe fn set_parent(frame: usize, parent: usize) -> bool {
    if !unsafe { is_valid(frame) } || !unsafe { is_valid(parent) } {
        return false;
    }

    let frame_simple = unsafe { frame_registry::is_simple(frame) };
    let parent_simple = unsafe { frame_registry::is_simple(parent) };

    if frame_simple {
        if !parent_simple {
            return false;
        }
        let f: CSimpleFrameSetParentFn =
            unsafe { core::mem::transmute(crate::addresses::get().frames.c_simple_frame_set_parent) };
        unsafe { f(frame, parent) };
        return true;
    }

    if parent_simple {
        return false;
    }

    let f: CLayerSetOwnerFn =
        unsafe { core::mem::transmute(crate::addresses::get().frames.c_layer_set_owner) };
    unsafe { f(frame, parent, 1, 0) };
    true
}

pub unsafe fn set_tooltip(frame: usize, tooltip: usize) -> bool {
    if !unsafe { is_valid(frame) } || !unsafe { is_valid(tooltip) } {
        return false;
    }

    if unsafe { frame_registry::is_simple(frame) } {
        let ft = unsafe { frame_registry::frame_type(frame) };
        if ft == FrameType::CSimpleFontString || ft == FrameType::CSimpleTexture {
            return false;
        }
        unsafe { write_usize(frame + C_SIMPLE_TOOLTIP, tooltip) };
        return true;
    }

    let f: CLayerSetTooltipFn =
        unsafe { core::mem::transmute(crate::addresses::get().frames.c_layer_set_tooltip) };
    unsafe { f(frame, tooltip) };
    true
}

const C_SIMPLE_BUTTON_TEXT_DISABLED: usize = 0x148;
const C_SIMPLE_BUTTON_TEXT_ENABLED: usize = 0x14C;
const C_TEXT_BUTTON_TEXT_FRAME: usize = 0x1F4;

type CSimpleFontStringSetTextFn = unsafe extern "thiscall" fn(usize, *const i8) -> i32;
type CTextFrameSetTextFn = unsafe extern "thiscall" fn(usize, *const i8) -> i32;
type CTextAreaSetTextFn = unsafe extern "thiscall" fn(usize, *const i8) -> i32;
type CTextAreaAddTextFn = unsafe extern "thiscall" fn(usize, *const i8) -> i32;
type CEditBoxSetTextFn = unsafe extern "thiscall" fn(usize, *const i8, i32) -> i32;

unsafe fn set_simple_font_string_text(frame: usize, text: *const i8) {
    if frame == 0 {
        return;
    }
    let f: CSimpleFontStringSetTextFn =
        unsafe { core::mem::transmute(crate::addresses::get().frames.c_simple_font_string_set_text) };
    unsafe { f(frame, text) };
}

unsafe fn set_text_frame_text(frame: usize, text: *const i8) {
    if frame == 0 {
        return;
    }
    let f: CTextFrameSetTextFn =
        unsafe { core::mem::transmute(crate::addresses::get().frames.c_text_frame_set_text) };
    unsafe { f(frame, text) };
}

pub unsafe fn set_text(frame: usize, text: *const i8) -> bool {
    if frame == 0 || text.is_null() {
        return false;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    match ft {
        FrameType::CEditBox | FrameType::CGlueEditBoxWar3 | FrameType::CSlashChatBox => {
            let f: CEditBoxSetTextFn =
                unsafe { core::mem::transmute(crate::addresses::get().frames.c_edit_box_set_text) };
            unsafe { f(frame, text, 1) };
            true
        }
        FrameType::CSimpleFontString => {
            unsafe { set_simple_font_string_text(frame, text) };
            true
        }
        FrameType::CTextArea => {
            let f: CTextAreaSetTextFn =
                unsafe { core::mem::transmute(crate::addresses::get().frames.c_text_area_set_text) };
            unsafe { f(frame, text) };
            true
        }
        FrameType::CSimpleButton => {
            let enabled = unsafe { read_usize(frame + C_SIMPLE_BUTTON_TEXT_ENABLED) };
            let disabled = unsafe { read_usize(frame + C_SIMPLE_BUTTON_TEXT_DISABLED) };
            unsafe {
                set_simple_font_string_text(enabled, text);
                set_simple_font_string_text(disabled, text);
            }
            true
        }
        FrameType::CTextButtonFrame | FrameType::CGlueTextButtonWar3 => {
            let text_frame = unsafe { read_usize(frame + C_TEXT_BUTTON_TEXT_FRAME) };
            unsafe { set_text_frame_text(text_frame, text) };
            true
        }
        FrameType::CTextFrame | FrameType::CTimerTextFrame | FrameType::CListBoxItem => {
            unsafe { set_text_frame_text(frame, text) };
            true
        }
        _ => false,
    }
}

pub unsafe fn add_text(frame: usize, text: *const i8) -> bool {
    if frame == 0 || text.is_null() {
        return false;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    match ft {
        FrameType::CTextArea => {
            let f: CTextAreaAddTextFn =
                unsafe { core::mem::transmute(crate::addresses::get().frames.c_text_area_add_text) };
            unsafe { f(frame, text) };
            true
        }
        _ => false,
    }
}

const C_BACKDROP_BLEND_ALL: usize = 0x224;
const C_BACKDROP_CORNER_FLAGS: usize = 0x240;
const C_BACKDROP_CORNER_SIZE: usize = 0x248;
const C_BACKDROP_BACKGROUND_SIZE: usize = 0x24C;
const C_BACKDROP_MIRRORED: usize = 0x254;
const C_BACKDROP_INSET_TOP: usize = 0x258;
const C_BACKDROP_INSET_BOT: usize = 0x25C;
const C_BACKDROP_INSET_LEFT: usize = 0x260;
const C_BACKDROP_INSET_RIGHT: usize = 0x264;

const C_SIMPLE_TEXTURE_ALPHA_MOD: usize = 0x94;
const C_SIMPLE_TEXTURE_TEX_LEFT_A: usize = 0xC8;
const C_SIMPLE_TEXTURE_TEX_UP_A: usize = 0xCC;
const C_SIMPLE_TEXTURE_TEX_LEFT_B: usize = 0xD0;
const C_SIMPLE_TEXTURE_TEX_BOT_A: usize = 0xD4;
const C_SIMPLE_TEXTURE_TEX_RIGHT_A: usize = 0xD8;
const C_SIMPLE_TEXTURE_TEX_UP_B: usize = 0xDC;
const C_SIMPLE_TEXTURE_TEX_RIGHT_B: usize = 0xE0;
const C_SIMPLE_TEXTURE_TEX_BOT_B: usize = 0xE4;

const C_HIGHLIGHT_UPDATE_FLAG: usize = 0x17C;
const C_HIGHLIGHT_COLOR_ARGB: usize = 0x298;
const C_HIGHLIGHT_ALPHA_MOD: usize = 0x2AC;

const C_SIMPLE_STATUS_BAR_TEXTURE: usize = 0x138;

type CSimpleRegionSetColorFn = unsafe extern "thiscall" fn(usize, u32) -> i32;




fn is_status_like(ft: FrameType) -> bool {
    matches!(
        ft,
        FrameType::CStatBar
            | FrameType::CSimpleStatusBar
            | FrameType::CProgressIndicator
            | FrameType::CHeroLevelBar
            | FrameType::CBuildTimeIndicator
    )
}

unsafe fn simple_status_texture(frame: usize) -> Option<usize> {
    let texture = unsafe { read_usize(frame + C_SIMPLE_STATUS_BAR_TEXTURE) };
    if texture == 0 { None } else { Some(texture) }
}

unsafe fn force_backdrop_update(frame: usize) {
    let Some(layout) = (unsafe { frame_registry::get_layout(frame) }) else {
        return;
    };
    let width = unsafe { read_f32(layout + C_LAYOUT_WIDTH) };
    let height = unsafe { read_f32(layout + C_LAYOUT_HEIGHT) };
    unsafe {
        set_width(frame, width);
        set_height(frame, height);
    }
}

pub unsafe fn set_tex_coord(frame: usize, left: f32, right: f32, up: f32, down: f32) -> bool {
    if !unsafe { is_valid(frame) } {
        return false;
    }

    match unsafe { frame_registry::frame_type(frame) } {
        FrameType::CSimpleTexture => {
            unsafe {
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_LEFT_A, left);
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_LEFT_B, left);
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_RIGHT_A, right);
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_RIGHT_B, right);
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_UP_A, up);
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_UP_B, up);
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_BOT_A, down);
                write_f32(frame + C_SIMPLE_TEXTURE_TEX_BOT_B, down);
            }
            true
        }
        FrameType::CBackdropFrame => {
            unsafe {
                write_f32(frame + C_BACKDROP_INSET_LEFT, left);
                write_f32(frame + C_BACKDROP_INSET_RIGHT, right);
                write_f32(frame + C_BACKDROP_INSET_TOP, up);
                write_f32(frame + C_BACKDROP_INSET_BOT, down);
                force_backdrop_update(frame);
            }
            true
        }
        _ => false,
    }
}

pub unsafe fn set_backdrop_mirrored(frame: usize, enabled: bool) -> bool {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return false;
    }
    unsafe {
        write_i32(frame + C_BACKDROP_MIRRORED, if enabled { 1 } else { 0 });
        force_backdrop_update(frame);
    }
    true
}

pub unsafe fn backdrop_mirrored(frame: usize) -> Option<bool> {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return None;
    }
    Some(unsafe { read_i32(frame + C_BACKDROP_MIRRORED) == 1 })
}

pub unsafe fn set_backdrop_tile_size(frame: usize, size: f32) -> bool {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return false;
    }
    unsafe {
        write_f32(frame + C_BACKDROP_BACKGROUND_SIZE, size);
        force_backdrop_update(frame);
    }
    true
}

pub unsafe fn backdrop_tile_size(frame: usize) -> Option<f32> {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return None;
    }
    Some(unsafe { read_f32(frame + C_BACKDROP_BACKGROUND_SIZE) })
}

pub unsafe fn set_backdrop_border_size(frame: usize, size: f32) -> bool {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return false;
    }
    unsafe {
        write_f32(frame + C_BACKDROP_CORNER_SIZE, size);
        force_backdrop_update(frame);
    }
    true
}

pub unsafe fn backdrop_border_size(frame: usize) -> Option<f32> {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return None;
    }
    Some(unsafe { read_f32(frame + C_BACKDROP_CORNER_SIZE) })
}

pub unsafe fn set_backdrop_border_flag(frame: usize, flag: i32) -> bool {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return false;
    }
    unsafe {
        write_i32(frame + C_BACKDROP_CORNER_FLAGS, flag);
        force_backdrop_update(frame);
    }
    true
}

pub unsafe fn backdrop_border_flag(frame: usize) -> Option<i32> {
    if unsafe { frame_registry::frame_type(frame) } != FrameType::CBackdropFrame {
        return None;
    }
    Some(unsafe { read_i32(frame + C_BACKDROP_CORNER_FLAGS) })
}

pub unsafe fn set_alpha_mode(frame: usize, mode: i32) -> bool {
    if !unsafe { is_valid(frame) } {
        return false;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    match ft {
        FrameType::CSimpleTexture => {
            unsafe { write_i32(frame + C_SIMPLE_TEXTURE_ALPHA_MOD, mode) };
            true
        }
        FrameType::CBackdropFrame => {
            unsafe {
                write_i32(frame + C_BACKDROP_BLEND_ALL, mode);
                force_backdrop_update(frame);
            }
            true
        }
        FrameType::CHighlightFrame => {
            unsafe {
                write_i32(frame + C_HIGHLIGHT_ALPHA_MOD, mode);
                force_backdrop_update(frame);
            }
            true
        }
        _ if is_status_like(ft) => {
            let Some(texture) = (unsafe { simple_status_texture(frame) }) else {
                return false;
            };
            unsafe { set_alpha_mode(texture, mode) }
        }
        _ => false,
    }
}

pub unsafe fn alpha_mode(frame: usize) -> Option<i32> {
    if !unsafe { is_valid(frame) } {
        return None;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    match ft {
        FrameType::CSimpleTexture => Some(unsafe { read_i32(frame + C_SIMPLE_TEXTURE_ALPHA_MOD) }),
        FrameType::CBackdropFrame => Some(unsafe { read_i32(frame + C_BACKDROP_BLEND_ALL) }),
        FrameType::CHighlightFrame => Some(unsafe { read_i32(frame + C_HIGHLIGHT_ALPHA_MOD) }),
        _ if is_status_like(ft) => {
            let texture = unsafe { simple_status_texture(frame) }?;
            unsafe { alpha_mode(texture) }
        }
        _ => None,
    }
}

unsafe fn set_simple_region_color(frame: usize, color: u32) {
    let f: CSimpleRegionSetColorFn =
        unsafe { core::mem::transmute(crate::addresses::get().frames.c_simple_region_set_vertex_color) };
    unsafe { f(frame, color) };
}

pub unsafe fn set_vertex_color(frame: usize, color: u32) -> bool {
    if !unsafe { is_valid(frame) } {
        return false;
    }

    let ft = unsafe { frame_registry::frame_type(frame) };
    match ft {
        FrameType::CStatBar
        | FrameType::CSimpleStatusBar
        | FrameType::CProgressIndicator
        | FrameType::CHeroLevelBar
        | FrameType::CBuildTimeIndicator => {
            let Some(texture) = (unsafe { simple_status_texture(frame) }) else {
                return false;
            };
            unsafe { set_simple_region_color(texture, color) };
            true
        }
        FrameType::CSimpleTexture => {
            unsafe { set_simple_region_color(frame, color) };
            true
        }
        FrameType::CHighlightFrame => {
            unsafe {
                write_u32(frame + C_HIGHLIGHT_COLOR_ARGB, color);
                write_i32(frame + C_HIGHLIGHT_UPDATE_FLAG, 2);
                force_backdrop_update(frame);
            }
            true
        }
        _ => false,
    }
}
