use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::engines;
use crate::logging;
use crate::ui::events::{
    UiEvent,
    CONTROL_CLICK_EVENT,
    MOUSE_EVENT_PRESS,
    MOUSE_EVENT_RELEASE,
    MOUSE_EVENT_SCROLL,
};

static EVENT_REGISTRY: OnceLock<Mutex<HashMap<(usize, u32), u32>>> = OnceLock::new();
static PENDING_CALLBACKS: OnceLock<Mutex<Vec<u32>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<(usize, u32), u32>> {
    EVENT_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pending_callbacks() -> &'static Mutex<Vec<u32>> {
    PENDING_CALLBACKS.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn clear() {
    registry().lock().unwrap().clear();
    pending_callbacks().lock().unwrap().clear();
    super::trigger_events::clear();
    logging::info("[frames/events] cleared registry and pending callbacks");
}

pub fn register_event(frame_ptr: usize, event_id: u32, callback_id: u32) {
    logging::info(&format!(
        "[frames/events] register_event: frame=0x{:x} event={} cb=0x{:x}",
        frame_ptr, event_id, callback_id
    ));
    registry().lock().unwrap().insert((frame_ptr, event_id), callback_id);
}

pub fn flush_pending_callbacks() {
    let current = match engines::current_jass_instance_index() {
        Some(v) => v,
        None => return,
    };

    let callbacks = {
        let mut q = pending_callbacks().lock().unwrap();
        if q.is_empty() {
            return;
        }
        q.drain(..).collect::<Vec<_>>()
    };

    for callback_id in callbacks {
        logging::info(&format!(
            "[frames/events] flushing deferred callback cb=0x{:x} in jass_instance_index=0x{:x}",
            callback_id, current
        ));
        let context = engines::CallbackContext::InvokeCodeById {
            jass_instance_index: current,
        };
        engines::try_dispatch_callback_code(callback_id, context);
    }
}

pub fn on_ui_event(event: UiEvent) {

    {
        use super::trigger_events::FrameEvent as TE;
        match event.event_id {
            CONTROL_CLICK_EVENT => {
                super::trigger_events::fire_frame_event(event.frame, TE::ControlClick, None, None);
            }
            MOUSE_EVENT_PRESS => {
                super::trigger_events::fire_frame_event(event.frame, TE::MouseDown, None, None);
            }
            MOUSE_EVENT_RELEASE => {
                super::trigger_events::fire_frame_event(event.frame, TE::MouseUp, None, None);
                super::trigger_events::fire_frame_event(event.frame, TE::ControlClick, None, None);
            }
            MOUSE_EVENT_SCROLL => {
                super::trigger_events::fire_frame_event(event.frame, TE::MouseWheel, None, None);
            }
            _ => {}
        }
    }
    let reg = registry().lock().unwrap();
    let observer_has_events = reg.keys().any(|k| k.0 == event.observer);
    let frame_has_events = reg.keys().any(|k| k.0 == event.frame);

    if observer_has_events
        || frame_has_events
        || event.event_id == CONTROL_CLICK_EVENT
        || event.event_id == MOUSE_EVENT_RELEASE
        || event.event_id == MOUSE_EVENT_PRESS
    {
        logging::info(&format!(
            "[frames/events] DispatchEvent: this=0x{:x} hover=0x{:x} eventId={} observer_reg={} hover_reg={}",
            event.observer, event.frame, event.event_id, observer_has_events, frame_has_events
        ));
    }

    if let Some(&callback_id) = reg.get(&(event.frame, event.event_id)) {
        logging::info(&format!(
            "[frames/events] defer exact: frame=0x{:x} event={} cb=0x{:x}",
            event.frame, event.event_id, callback_id
        ));
        enqueue_callback(callback_id);
    }

    if event.event_id == MOUSE_EVENT_RELEASE {
        if let Some(&callback_id) = reg.get(&(event.frame, CONTROL_CLICK_EVENT)) {
            logging::info(&format!(
                "[frames/events] defer synthesized ControlClick: frame=0x{:x} cb=0x{:x}",
                event.frame, callback_id
            ));
            enqueue_callback(callback_id);
        }
    }
}

fn enqueue_callback(callback_id: u32) {
    logging::info(&format!("[frames/events] queue callback cb=0x{:x}", callback_id));
    pending_callbacks().lock().unwrap().push(callback_id);
}
