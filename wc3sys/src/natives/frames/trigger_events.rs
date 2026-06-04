use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::jass::custom_triggers::{self, CustomTriggerContext};
use crate::jass::raw;
use crate::logging;
use crate::addresses;

use super::frame_registry;
use super::frame_type::FrameType;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameEvent {
    ControlClick = 1,
    MouseEnter = 2,
    MouseLeave = 3,
    MouseUp = 4,
    MouseDown = 5,
    MouseWheel = 6,
    CheckboxChecked = 7,
    CheckboxUnchecked = 8,
    EditBoxTextChanged = 9,
    PopupMenuItemChanged = 10,
    MouseDoubleClick = 11,
    SpriteAnimUpdate = 12,
    SliderValueChanged = 13,
    DialogCancel = 14,
    DialogAccept = 15,
    EditBoxEnter = 16,
    ListBoxItemSelect = 17,
    ListBoxItemDoubleClick = 18,
    ListBoxItemChanged = 19,
    SimpleFrameClick = 20,
}

impl FrameEvent {
    fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::ControlClick),
            2 => Some(Self::MouseEnter),
            3 => Some(Self::MouseLeave),
            4 => Some(Self::MouseUp),
            5 => Some(Self::MouseDown),
            6 => Some(Self::MouseWheel),
            7 => Some(Self::CheckboxChecked),
            8 => Some(Self::CheckboxUnchecked),
            9 => Some(Self::EditBoxTextChanged),
            10 => Some(Self::PopupMenuItemChanged),
            11 => Some(Self::MouseDoubleClick),
            12 => Some(Self::SpriteAnimUpdate),
            13 => Some(Self::SliderValueChanged),
            14 => Some(Self::DialogCancel),
            15 => Some(Self::DialogAccept),
            16 => Some(Self::EditBoxEnter),
            17 => Some(Self::ListBoxItemSelect),
            18 => Some(Self::ListBoxItemDoubleClick),
            19 => Some(Self::ListBoxItemChanged),
            20 => Some(Self::SimpleFrameClick),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct RegisteredFrameTrigger {
    trigger: u32,
    frame_event: FrameEvent,
}

static REGISTRY: OnceLock<Mutex<HashMap<usize, Vec<RegisteredFrameTrigger>>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<usize, Vec<RegisteredFrameTrigger>>> {
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clear() {
    registry().lock().unwrap().clear();
}

fn event_supported_by_frame(frame: usize, event: FrameEvent) -> bool {
    let frame_type = unsafe { frame_registry::frame_type(frame) };
    match event {
        FrameEvent::CheckboxChecked | FrameEvent::CheckboxUnchecked => matches!(
            frame_type,
            FrameType::CCheckBox | FrameType::CGlueCheckBoxWar3
        ),
        FrameEvent::PopupMenuItemChanged => matches!(
            frame_type,
            FrameType::CPopupMenu | FrameType::CGluePopupMenuWar3
        ),
        FrameEvent::SliderValueChanged => frame_type == FrameType::CSlider,
        FrameEvent::EditBoxTextChanged | FrameEvent::EditBoxEnter => matches!(
            frame_type,
            FrameType::CEditBox
                | FrameType::CChatEditBox
                | FrameType::CGlueEditBoxWar3
                | FrameType::CChatEditBar
        ),
        _ => true,
    }
}

fn player_handle(index: u32) -> u32 {
    let f: raw::PlayerFn = unsafe { core::mem::transmute(addresses::get().player) };
    unsafe { f(index) }
}

pub fn register_frame_event(trigger: u32, frame: usize, event_id: i32) -> i32 {
    let Some(frame_event) = FrameEvent::from_i32(event_id) else {
        return 0;
    };

    if trigger == 0 || frame == 0 || !unsafe { frame_registry::is_valid(frame) } {
        return 0;
    }

    if !event_supported_by_frame(frame, frame_event) {
        let ft = unsafe { frame_registry::frame_type(frame) };
        logging::warn(&format!(
            "[frames/trigger_events] unsupported frame event {:?} for frame=0x{frame:x} type={ft:?}",
            frame_event
        ));
        return 0;
    }

    registry()
        .lock()
        .unwrap()
        .entry(frame)
        .or_default()
        .push(RegisteredFrameTrigger {
            trigger,
            frame_event,
        });

    if unsafe { frame_registry::is_simple(frame) } && frame_event == FrameEvent::ControlClick {
        registry()
            .lock()
            .unwrap()
            .entry(frame)
            .or_default()
            .push(RegisteredFrameTrigger {
                trigger,
                frame_event: FrameEvent::SimpleFrameClick,
            });
    }

    logging::info(&format!(
        "[frames/trigger_events] registered trigger=0x{trigger:x} frame=0x{frame:x} event={:?}",
        frame_event
    ));

    0
}

pub fn fire_frame_event(frame: usize, frame_event: FrameEvent, value: Option<f32>, text: Option<String>) {
    let entries = registry()
        .lock()
        .unwrap()
        .get(&frame)
        .cloned()
        .unwrap_or_default();

    for entry in entries {
        if entry.frame_event != frame_event {
            continue;
        }

        let ctx = CustomTriggerContext {
            triggering_trigger: Some(entry.trigger),
            triggering_player: Some(player_handle(0)),
            triggering_event_id: Some(frame_event as i32),
            trigger_frame: Some(frame),
            trigger_frame_event: Some(frame_event as i32),
            trigger_frame_value: value,
            trigger_frame_text: text.clone(),
        };

        custom_triggers::fire_custom_trigger(entry.trigger, ctx);
    }
}

pub unsafe extern "C" fn trigger_register_frame_event(trigger: u32, frame: u32, event_id: u32) -> i32 {
    register_frame_event(trigger, frame as usize, event_id as i32)
}

pub unsafe extern "C" fn get_trigger_frame() -> u32 {
    custom_triggers::current_context()
        .and_then(|ctx| ctx.trigger_frame)
        .unwrap_or(0) as u32
}

pub unsafe extern "C" fn get_trigger_frame_event() -> u32 {
    custom_triggers::current_context()
        .and_then(|ctx| ctx.trigger_frame_event)
        .unwrap_or(0) as u32
}

pub unsafe extern "C" fn get_trigger_frame_value() -> u32 {
    custom_triggers::current_context()
        .and_then(|ctx| ctx.trigger_frame_value)
        .unwrap_or(0.0)
        .to_bits()
}

pub unsafe extern "C" fn get_trigger_frame_text() -> u32 {
    let Some(text) = custom_triggers::current_context().and_then(|ctx| ctx.trigger_frame_text) else {
        return 0;
    };

    let Ok(c_text) = std::ffi::CString::new(text) else {
        return 0;
    };

    crate::jass::raw::make_jass_string(c_text.as_ptr() as *const u8) as u32
}
