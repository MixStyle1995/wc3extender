use core::ffi::c_void;

use crate::addresses;
use crate::hooks as hook_manager;

pub const MOUSE_EVENT_PRESS: u32 = 1074069704;
pub const MOUSE_EVENT_RELEASE: u32 = 1074069705;
pub const MOUSE_EVENT_SCROLL: u32 = 1074069709;
pub const CONTROL_CLICK_EVENT: u32 = 1074331748;

type CLayerFindLayerUnderCursorFn =
    unsafe extern "thiscall" fn(layer: usize, mouse_event: usize) -> usize;

#[derive(Debug, Clone, Copy)]
pub struct UiEvent {
    pub observer: usize,
    pub frame: usize,
    pub event_id: u32,
}

pub fn is_mouse_event(event_id: u32) -> bool {
    event_id == MOUSE_EVENT_PRESS || event_id == MOUSE_EVENT_RELEASE || event_id == MOUSE_EVENT_SCROLL
}

pub unsafe fn hovered_frame() -> usize {
    let addrs = addresses::get();
    let active_layer = unsafe { (addrs.frames.c_layer_active_layer as *const usize).read_unaligned() };
    if active_layer == 0 {
        return 0;
    }

    let find_under_cursor: CLayerFindLayerUnderCursorFn =
        unsafe { core::mem::transmute(addrs.frames.c_layer_find_under_cursor) };
    unsafe { find_under_cursor(active_layer, addrs.frames.c_layer_find_under_cursor_arg) }
}

unsafe fn read_event_id(event: *mut c_void) -> u32 {
    if event.is_null() {
        return 0;
    }

    unsafe { *((event as *const u8).add(8) as *const u32) }
}

unsafe fn capture_event(this: *mut c_void, event: *mut c_void) -> UiEvent {
    let event_id = unsafe { read_event_id(event) };
    let observer = this as usize;
    let mut frame = observer;

    if is_mouse_event(event_id) {
        let hovered = unsafe { hovered_frame() };
        if hovered != 0 {
            frame = hovered;
        }
    }

    UiEvent {
        observer,
        frame,
        event_id,
    }
}

pub unsafe extern "thiscall" fn c_observer_dispatch_event_handler(
    this: *mut c_void,
    event: *mut c_void,
) -> i32 {
    let ui_event = unsafe { capture_event(this, event) };
    crate::natives::frames::events::on_ui_event(ui_event);

    let tramp = hook_manager::trampoline(addresses::get().frames.c_observer_dispatch_event)
        .expect("c_observer_dispatch_event trampoline missing");
    let original: crate::ui::hooks::CObserverDispatchEventFn = unsafe { core::mem::transmute(tramp) };
    unsafe { original(this, event) }
}
