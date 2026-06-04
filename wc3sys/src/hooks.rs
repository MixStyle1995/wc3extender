use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use wc3::InlineHook;

use crate::error::{Error, Result};

static HOOKS: OnceLock<Mutex<HashMap<usize, InlineHook>>> = OnceLock::new();

fn hooks() -> &'static Mutex<HashMap<usize, InlineHook>> {
    HOOKS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn install(mut hook: InlineHook) -> Result<()> {
    let target = hook.function;
    let new_name = hook.name;

    if let Some(existing) = hooks().lock().unwrap().get(&target) {
        return Err(Error::HookTargetAlreadyRegistered {
            target,
            existing_name: existing.name,
            new_name,
        });
    }

    unsafe { hook.install()? };

    hooks().lock().unwrap().insert(target, hook);
    Ok(())
}

pub fn trampoline(target: usize) -> Result<usize> {
    let hooks = hooks().lock().unwrap();

    let hook = hooks
        .get(&target)
        .ok_or(Error::HookNotFound { target })?;

    hook.trampoline().ok_or(Error::HookTrampolineMissing {
        target,
        name: hook.name,
    })
}

#[allow(dead_code)]
pub fn uninstall(target: usize) -> Result<()> {
    let mut hooks = hooks().lock().unwrap();

    let hook = hooks
        .get_mut(&target)
        .ok_or(Error::HookNotFound { target })?;

    unsafe { hook.uninstall()? };
    Ok(())
}

#[allow(dead_code)]
pub fn is_active(target: usize) -> bool {
    hooks()
        .lock()
        .unwrap()
        .get(&target)
        .map(|hook| hook.active)
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn names() -> Vec<&'static str> {
    hooks()
        .lock()
        .unwrap()
        .values()
        .map(|hook| hook.name)
        .collect()
}
