use std::ffi::CString;
use std::sync::Arc;

use mlua::Value;

use crate::jass::raw as jass_raw;
use crate::logging;

use super::runtime::LuaRuntime;
use super::sig::JassType;

pub struct Marshaller {
    runtime: Arc<LuaRuntime>,
    real_storage: Vec<f32>,
    next_real: usize,
}

impl Marshaller {
    pub fn new(runtime: Arc<LuaRuntime>, arg_types: &[JassType]) -> Self {
        let real_count = arg_types.iter().filter(|t| **t == JassType::Real).count();
        Self {
            runtime,
            real_storage: vec![0.0; real_count],
            next_real: 0,
        }
    }

    pub fn marshal_in(
        &mut self,
        ty: JassType,
        v: &Value,
        native_name: &str,
        arg_idx: usize,
    ) -> Result<u32, String> {
        match ty {
            JassType::Int | JassType::Handle => match v {
                Value::Nil => Ok(0),
                Value::Integer(i) => Ok(*i as i32 as u32),
                Value::Number(n) => Ok(*n as i32 as u32),
                _ => Err("expected integer".into()),
            },
            JassType::Bool => match v {
                Value::Nil => Ok(0),
                Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
                Value::Integer(i) => Ok(if *i != 0 { 1 } else { 0 }),
                _ => Err("expected bool".into()),
            },
            JassType::Real => {
                let f = match v {
                    Value::Nil => 0.0,
                    Value::Number(n) => *n as f32,
                    Value::Integer(i) => *i as f32,
                    _ => return Err("expected number".into()),
                };
                let slot = self.next_real;
                self.next_real += 1;
                self.real_storage[slot] = f;
                Ok((&self.real_storage[slot] as *const f32 as usize) as u32)
            }
            JassType::Str => self.string_arg(v, native_name, arg_idx),
            JassType::Code => match v {
                Value::Nil => Ok(0),
                Value::Function(f) => {
                    let Some(lua) = self.runtime.lua() else {
                        return Err("lua state not initialized".to_string());
                    };
                    let slot = self.runtime.callbacks().store(&lua, f.clone())?;
                    self.runtime.mint_callback(slot)
                }
                Value::Integer(i) => Ok(*i as u32),
                _ => Err("expected function or integer for code".into()),
            },
            JassType::Void => Err("can't pass void as arg".into()),
        }
    }

    fn string_arg(&mut self, v: &Value, native_name: &str, arg_idx: usize) -> Result<u32, String> {
        let jass_instance_index = crate::engines::best_jass_instance_index()
            .ok_or_else(|| "string argument passed with no known JASS instance".to_string())?;

        let handle = match v {
            Value::Nil => return Ok(0),
            Value::Integer(i) => *i as i32 as u32,
            Value::Number(n) => *n as i32 as u32,
            Value::String(s) => {
                let bytes = s.as_bytes();
                let cstr = CString::new(&*bytes)
                    .map_err(|_| "string contains NUL".to_string())?;

                let arg_ptr = jass_raw::string_to_arg(cstr.as_ptr() as *const u8, jass_instance_index)
                    .ok_or_else(|| format!("could not create JASS string arg in instance {jass_instance_index}"))?;

                logging::info(&format!(
                    "[marshal] {native_name} arg{arg_idx}: lua string -> arg 0x{arg_ptr:x} via instance 0x{jass_instance_index:x}"
                ));
                return Ok(arg_ptr as u32);
            }
            _ => return Err("expected string or raw JASS string handle".into()),
        };

        let arg_ptr = jass_raw::jass_string_handle_to_arg(handle, jass_instance_index)
            .ok_or_else(|| format!("could not resolve string handle {handle} in instance {jass_instance_index}"))?;

        logging::info(&format!(
            "[marshal] {native_name} arg{arg_idx}: string handle {handle} -> arg 0x{arg_ptr:x} via instance 0x{jass_instance_index:x}"
        ));
        Ok(arg_ptr as u32)
    }


}

