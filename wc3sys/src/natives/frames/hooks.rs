use wc3::InlineHook;
use crate::addresses;
use crate::hooks as hook_manager;
use crate::logging;
use super::structs::CFrame;

pub const FRAME_DEF_CREATE_FRAME: &str = "frame_def_create_frame";

pub type FrameDefCreateFrameFn = unsafe extern "C" fn(
    name_str: *const i8,
    parent: *mut CFrame,
    a3: i32,
    a4: i32,
    create_context: i32,
) -> *mut CFrame;

pub fn frame_def_create_frame(handler: FrameDefCreateFrameFn) -> InlineHook {
    InlineHook::new(
        FRAME_DEF_CREATE_FRAME,
        addresses::get().frames.frame_def_create_frame,
        handler as *const () as usize,
    )
}

pub fn install() -> crate::error::Result<()> {
    hook_manager::install(frame_def_create_frame(frame_def_create_frame_handler))?;
    super::natives::register_custom_natives();
    logging::info("frames: hooks and custom natives installed");
    Ok(())
}

unsafe extern "C" fn frame_def_create_frame_handler(
    name_str: *const i8,
    parent: *mut CFrame,
    a3: i32,
    a4: i32,
    create_context: i32,
) -> *mut CFrame {
    let create_context = if unsafe { should_reset_create_context(name_str) } {
        0
    } else {
        create_context
    };

    let tramp = hook_manager::trampoline(addresses::get().frames.frame_def_create_frame)
        .expect("frame_def_create_frame trampoline missing");
    let original: FrameDefCreateFrameFn = unsafe { core::mem::transmute(tramp) };
    unsafe { original(name_str, parent, a3, a4, create_context) }
}

unsafe fn should_reset_create_context(name_str: *const i8) -> bool {
    if name_str.is_null() {
        return false;
    }

    let Ok(name) = (unsafe { core::ffi::CStr::from_ptr(name_str) }).to_str() else {
        return false;
    };

    matches!(name, "Multiboard" | "Leaderboard" | "TimerDialog")
}
