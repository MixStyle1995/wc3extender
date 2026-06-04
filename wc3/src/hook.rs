use core::ptr;
use std::ffi::CStr;
use std::path::Path;

use crate::patch::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HookType {
    PatchBefore,
    PatchAfter,
    Replace,
}

pub struct Hook {
    pub name: &'static str,
    pub call_site: usize,
    pub original_target: usize,
    pub hook_fn: usize,
    pub hook_type: HookType,
    pub trampoline: Option<usize>,
    pub original_bytes: [u8; 5],
    pub active: bool,
}

impl Hook {
    pub fn new(
        name: &'static str,
        call_site: usize,
        hook_fn: usize,
        hook_type: HookType,
    ) -> Self {
        let original_bytes = unsafe {
            let mut buf = [0u8; 5];

            ptr::copy_nonoverlapping(call_site as *const u8, buf.as_mut_ptr(), 5);

            buf
        };

        let original_target = if original_bytes[0] == 0xE8 {
            let disp = i32::from_le_bytes([
                original_bytes[1],
                original_bytes[2],
                original_bytes[3],
                original_bytes[4],
            ]);

            ((call_site + 5) as i64 + disp as i64) as usize
        } else {
            0
        };

        Hook {
            name,
            call_site,
            original_target,
            hook_fn,
            hook_type,
            trampoline: None,
            original_bytes,
            active: false,
        }
    }

    pub fn replace(name: &'static str, call_site: usize, hook_fn: usize) -> Self {
        Self::new(name, call_site, hook_fn, HookType::Replace)
    }

    pub fn before(name: &'static str, call_site: usize, hook_fn: usize) -> Self {
        Self::new(name, call_site, hook_fn, HookType::PatchBefore)
    }

    pub fn after(name: &'static str, call_site: usize, hook_fn: usize) -> Self {
        Self::new(name, call_site, hook_fn, HookType::PatchAfter)
    }

    pub unsafe fn install(&mut self) -> Result<(), &'static str> {
        unsafe {
            if self.active {
                return Err("Hook already active");
            }

            if self.original_bytes[0] != 0xE8 {
                return Err("Call site is not a CALL instruction");
            }

            match self.hook_type {
                HookType::Replace => {
                    patch_call_target(self.call_site, self.hook_fn)?;
                }

                HookType::PatchBefore => {
                    let tramp = self.build_trampoline_before()?;

                    self.trampoline = Some(tramp);

                    patch_call_target(self.call_site, tramp)?;
                }

                HookType::PatchAfter => {
                    let tramp = self.build_trampoline_after()?;

                    self.trampoline = Some(tramp);

                    patch_call_target(self.call_site, tramp)?;
                }
            }

            self.active = true;

            Ok(())
        }
    }

    pub unsafe fn uninstall(&mut self) -> Result<(), &'static str> {
        unsafe {
            if !self.active {
                return Err("Hook not active");
            }

            write_bytes(self.call_site, &self.original_bytes)?;

            self.active = false;

            Ok(())
        }
    }

    unsafe fn build_trampoline_before(&self) -> Result<usize, &'static str> {
        unsafe {
            let tramp = alloc_executable(16).ok_or("Failed to allocate trampoline")?;

            let mut code = [0u8; 11];

            let call1 = build_call32(tramp, self.hook_fn);

            code[0..5].copy_from_slice(&call1);

            let call2 = build_call32(tramp + 5, self.original_target);

            code[5..10].copy_from_slice(&call2);

            code[10] = 0xC3;

            ptr::copy_nonoverlapping(code.as_ptr(), tramp as *mut u8, 11);

            Ok(tramp)
        }
    }

    unsafe fn build_trampoline_after(&self) -> Result<usize, &'static str> {
        unsafe {
            let tramp = alloc_executable(16).ok_or("Failed to allocate trampoline")?;

            let mut code = [0u8; 11];

            let call1 = build_call32(tramp, self.original_target);

            code[0..5].copy_from_slice(&call1);

            let call2 = build_call32(tramp + 5, self.hook_fn);

            code[5..10].copy_from_slice(&call2);

            code[10] = 0xC3;

            ptr::copy_nonoverlapping(code.as_ptr(), tramp as *mut u8, 11);

            Ok(tramp)
        }
    }

    pub fn get_original(&self) -> usize {
        self.original_target
    }
}

pub struct HookManager {
    hooks: Vec<Hook>,
}

impl HookManager {
    pub fn new() -> Self {
        HookManager {
            hooks: Vec::new(),
        }
    }

    pub fn add(&mut self, hook: Hook) {
        self.hooks.push(hook);
    }

    pub unsafe fn install_all(&mut self) -> Result<(), &'static str> {
        unsafe {
            for hook in &mut self.hooks {
                hook.install()?;
            }

            Ok(())
        }
    }

    pub unsafe fn uninstall_all(&mut self) -> Result<(), &'static str> {
        unsafe {
            for hook in &mut self.hooks {
                if hook.active {
                    hook.uninstall()?;
                }
            }

            Ok(())
        }
    }

    pub fn get(&self, name: &str) -> Option<&Hook> {
        self.hooks.iter().find(|h| h.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Hook> {
        self.hooks.iter_mut().find(|h| h.name == name)
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}
