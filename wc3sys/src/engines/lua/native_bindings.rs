use core::ffi::c_void;
use std::sync::Arc;

use mlua::{Value, Variadic};

use super::invoke::{invoke_int, invoke_real, invoke_void};
use super::marshal::Marshaller;
use super::runtime::LuaRuntime;
use crate::jass::raw;
use super::sig::{parse_signature, JassType};

pub fn register_native(
    runtime: Arc<LuaRuntime>,
    name: &str,
    signature: &str,
    func: *const c_void,
) -> Result<(), String> {
    let Some(lua) = runtime.lua() else { return Ok(()) };

    let (arg_types, ret_type) = parse_signature(signature)?;
    let addr = func as usize;
    let owned_name = name.to_string();

    let lua_fn = lua
         .create_function(move |lua_ctx, vals: Variadic<Value>| {
            if vals.len() != arg_types.len() {
                return Err(mlua::Error::external(format!(
                    "{owned_name}: expected {} args, got {}",
                    arg_types.len(),
                    vals.len()
                )));
            }

            let mut marshaller = Marshaller::new(runtime.clone(), &arg_types);

            let mut native_args: Vec<u32> = Vec::with_capacity(arg_types.len());
            for (i, (ty, val)) in arg_types.iter().zip(vals.iter()).enumerate() {
                let raw = marshaller
                    .marshal_in(*ty, val, &owned_name, i)
                    .map_err(|e| mlua::Error::external(format!("{owned_name} arg {i}: {e}")))?;
                native_args.push(raw);
            }

            unsafe {
                match ret_type {
                    JassType::Void => {
                        invoke_void(addr, &native_args).map_err(mlua::Error::external)?;
                        Ok(Value::Nil)
                    }
                    JassType::Real => {
                        let f = invoke_real(addr, &native_args).map_err(mlua::Error::external)?;
                        Ok(Value::Number(f as f64))
                    }
                    JassType::Bool => {
                        let v = invoke_int(addr, &native_args).map_err(mlua::Error::external)?;
                        Ok(Value::Boolean(v != 0))
                    }
                    JassType::Str => {
                        let handle = invoke_int(addr, &native_args).map_err(mlua::Error::external)?;
                        let jass_instance_index = crate::engines::current_jass_instance_index()
                            .ok_or_else(|| mlua::Error::external(format!("{owned_name}: string return outside a JASS instance")))?;
                        let s = raw::jass_string_handle_to_str(handle, jass_instance_index)
                            .ok_or_else(|| mlua::Error::external(format!("{owned_name}: could not resolve string handle {handle}")))?;
                        Ok(Value::String(lua_ctx.create_string(&s)?))
                    }
                    _ => {
                        let v = invoke_int(addr, &native_args).map_err(mlua::Error::external)?;
                        Ok(Value::Integer(v as i32))
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;

    lua.globals().set(name, lua_fn).map_err(|e| e.to_string())
}
