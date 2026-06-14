use mlua::Lua;

use crate::{archives, logging};

const SCRIPTS: &[&str] = &["Scripts\\common.j.lua", "Scripts\\Blizzard.j.lua", "Scripts\\1.32.10.compat.j.lua"];

pub fn load_all(lua: &Lua) {
    for name in SCRIPTS {
        load(lua, name);
    }
}

fn load(lua: &Lua, name: &str) {
    let Some(source) = archives::read_mpq_file(name) else {
        logging::warn(&format!("[lua] compat script not found: {name}"));
        return;
    };

    match lua.load(source).set_name(name).exec() {
        Ok(()) => logging::info(&format!("[lua] loaded compat script {name}")),
        Err(e) => logging::error(&format!("[lua] compat script {name}: {e}")),
    }
}
