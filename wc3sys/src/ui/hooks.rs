use core::ffi::c_void;

use wc3::InlineHook;

use crate::addresses;

pub const C_OBSERVER_DISPATCH_EVENT: &str = "c_observer_dispatch_event";
pub type CObserverDispatchEventFn = unsafe extern "thiscall" fn(
    this: *mut c_void,
    event: *mut c_void,
) -> i32;

pub fn c_observer_dispatch_event(handler: CObserverDispatchEventFn) -> InlineHook {
    InlineHook::new(
        C_OBSERVER_DISPATCH_EVENT,
        addresses::get().c_observer_dispatch_event,
        handler as *const () as usize,
    )
}
