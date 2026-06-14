use std::sync::Mutex;
use std::vec::Vec;

use crate::addresses;
use crate::hooks as hook_manager;
use crate::jass::raw;
use crate::logging;

#[derive(Debug, Clone)]
pub struct CustomTriggerContext {
    pub triggering_trigger: Option<u32>,
    pub triggering_player: Option<u32>,
    pub triggering_event_id: Option<i32>,
    pub trigger_frame: Option<usize>,
    pub trigger_frame_event: Option<i32>,
    pub trigger_frame_value: Option<f32>,
    pub trigger_frame_text: Option<String>,
}

impl Default for CustomTriggerContext {
    fn default() -> Self {
        Self {
            triggering_trigger: None,
            triggering_player: None,
            triggering_event_id: None,
            trigger_frame: None,
            trigger_frame_event: None,
            trigger_frame_value: None,
            trigger_frame_text: None,
        }
    }
}

static CONTEXT_STACK: Mutex<Vec<CustomTriggerContext>> = Mutex::new(Vec::new());

fn push_context(ctx: CustomTriggerContext) {
    CONTEXT_STACK.lock().unwrap().push(ctx);
}

fn pop_context() {
    CONTEXT_STACK.lock().unwrap().pop();
}

fn peek_context() -> Option<CustomTriggerContext> {
    CONTEXT_STACK.lock().unwrap().last().cloned()
}

pub fn current_context() -> Option<CustomTriggerContext> {
    peek_context()
}

pub fn fire_custom_trigger(
    trigger_handle: u32,
    context: CustomTriggerContext,
) {
    push_context(context);

    let addrs = addresses::get();

    let is_enabled: bool = unsafe {
        let f: raw::IsTriggerEnabledFn = core::mem::transmute(addrs.jass.is_trigger_enabled);
        f(trigger_handle) != 0
    };

    if !is_enabled {
        pop_context();
        return;
    }

    let evaluate_ok: bool = unsafe {
        let f: raw::TriggerEvaluateFn = core::mem::transmute(addrs.jass.trigger_evaluate);
        f(trigger_handle) != 0
    };

    if !evaluate_ok {
        pop_context();
        return;
    }

    unsafe {
        let f: raw::TriggerExecuteFn = core::mem::transmute(addrs.jass.trigger_execute);
        f(trigger_handle);
    }

    pop_context();
}

type GetTriggerPlayerFn = unsafe extern "C" fn() -> u32;
type GetTriggeringTriggerFn = unsafe extern "C" fn() -> u32;
type GetTriggerEventIdFn = unsafe extern "C" fn() -> i32;

unsafe extern "C" fn get_trigger_player_hook() -> u32 {
    if let Some(ctx) = peek_context() {
        if let Some(p) = ctx.triggering_player {
            return p;
        }
    }

    let tramp = hook_manager::trampoline(addresses::get().jass.get_trigger_player)
        .expect("GetTriggerPlayer trampoline missing");
    let original: GetTriggerPlayerFn = unsafe { core::mem::transmute(tramp) };
    unsafe { original() }
}

unsafe extern "C" fn get_triggering_trigger_hook() -> u32 {
    if let Some(ctx) = peek_context() {
        if let Some(t) = ctx.triggering_trigger {
            return t;
        }
    }

    let tramp = hook_manager::trampoline(addresses::get().jass.get_triggering_trigger)
        .expect("GetTriggeringTrigger trampoline missing");
    let original: GetTriggeringTriggerFn = unsafe { core::mem::transmute(tramp) };
    unsafe { original() }
}

unsafe extern "C" fn get_trigger_event_id_hook() -> i32 {
    if let Some(ctx) = peek_context() {
        if let Some(id) = ctx.triggering_event_id {
            return id;
        }
    }

    let tramp = hook_manager::trampoline(addresses::get().jass.get_trigger_event_id)
        .expect("GetTriggerEventId trampoline missing");
    let original: GetTriggerEventIdFn = unsafe { core::mem::transmute(tramp) };
    unsafe { original() }
}

pub fn install() -> crate::error::Result<()> {
    use wc3::InlineHook;

    let addrs = addresses::get();
    hook_manager::install(InlineHook::new(
        "custom_get_trigger_player",
        addrs.jass.get_trigger_player,
        get_trigger_player_hook as *const () as usize,
    ))?;

    hook_manager::install(InlineHook::new(
        "custom_get_triggering_trigger",
        addrs.jass.get_triggering_trigger,
        get_triggering_trigger_hook as *const () as usize,
    ))?;

    hook_manager::install(InlineHook::new(
        "custom_get_trigger_event_id",
        addrs.jass.get_trigger_event_id,
        get_trigger_event_id_hook as *const () as usize,
    ))?;

    logging::info("jass::custom_triggers: hooks installed");
    Ok(())
}
