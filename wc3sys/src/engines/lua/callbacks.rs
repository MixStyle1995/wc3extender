use std::sync::Mutex;

use mlua::{Function, Lua, RegistryKey};

pub struct LuaCallbacks {
    keys: Mutex<Vec<RegistryKey>>,
}

impl LuaCallbacks {
    pub fn new() -> Self {
        Self {
            keys: Mutex::new(Vec::new()),
        }
    }

    pub fn clear(&self) {
        self.keys.lock().unwrap().clear();
    }

    pub fn store(&self, lua: &Lua, f: Function) -> Result<u64, String> {
        let key = lua
            .create_registry_value(f)
            .map_err(|e| format!("create_registry_value: {e}"))?;

        let mut keys = self.keys.lock().unwrap();
        let slot_id = keys.len() as u64;
        keys.push(key);
        Ok(slot_id)
    }

    pub fn get(&self, lua: &Lua, opaque: u64) -> Result<Function, String> {
        let keys = self.keys.lock().unwrap();
        let key = keys
            .get(opaque as usize)
            .ok_or_else(|| format!("no slot {opaque}"))?;
        lua.registry_value(key)
            .map_err(|e| format!("registry_value: {e}"))
    }
}
