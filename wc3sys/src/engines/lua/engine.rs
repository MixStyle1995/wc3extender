use core::ffi::c_void;
use std::sync::Arc;

use mlua::Function;

use crate::engines::{CallbackContext, Engine, EngineContext};
use crate::logging;

use super::compat_scripts;
use super::native_bindings;
use super::runtime::LuaRuntime;

pub struct LuaEngine {
    runtime: Arc<LuaRuntime>,
}

impl LuaEngine {
    pub fn new() -> Self {
        Self {
            runtime: LuaRuntime::new_shared(),
        }
    }
}

impl Engine for LuaEngine {
    fn name(&self) -> &'static str {
        "lua"
    }

    fn install(&self, context: EngineContext) -> crate::error::Result<()> {
        self.runtime.install(context)?;
        Ok(())
    }

    fn map_entrypoint(&self) -> Option<&'static str> {
        Some("war3map.lua")
    }

    fn set_map_payload(&self, payload: Option<Vec<u8>>) {
        let source = payload.and_then(|bytes| match String::from_utf8(bytes) {
            Ok(source) => Some(source),
            Err(e) => {
                logging::warn(&format!("[lua] map payload is not utf-8: {e}"));
                None
            }
        });
        self.runtime.set_map_script(source);
    }

    fn config(&self) {
        if let Err(e) = self.runtime.rebuild() {
            logging::error(&format!("LuaEngine config: {e}"));
        }
    }

    fn post_config(&self) {
        let Some(lua) = self.runtime.lua() else {
            return;
        };
        compat_scripts::load_all(&lua);
    }

    fn function_called(&self, name: &str) {
        let Some(lua) = self.runtime.lua() else {
            return;
        };

        let func: mlua::Result<Function> = lua.globals().get(name);
        let Ok(func) = func else {
            return;
        };

        if let Err(e) = func.call::<()>(()) {
            logging::warn(&format!("[lua] {name}: {e}"));
        }
    }

    fn register_native(&self, name: &str, signature: &str, func: *const c_void) {
        if let Err(e) = native_bindings::register_native(self.runtime.clone(), name, signature, func)
        {
            logging::warn(&format!("[lua] register_native {name}: {e}"));
        }
    }

    fn dispatch_callback(&self, opaque: u64, context: CallbackContext) {
        let _instance_guard = match context {
            CallbackContext::InvokeCodeById {
                jass_instance_index,
            } => Some(crate::engines::handlers::enter_jass_instance_index(
                jass_instance_index,
            )),
        };

        if let Err(e) = self.runtime.dispatch_callback(opaque) {
            logging::warn(&format!("[lua] dispatch_callback: {e}"));
        }
    }
}
