use core::ffi::c_void;
use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Mutex, OnceLock};

use crate::jass::raw as jass_raw;
use crate::logging;

#[derive(Debug, Clone)]
pub struct NativeRecord {
    pub name: String,
    pub signature: String,
    pub func: usize,
}

struct PendingNative {
    name: CString,
    signature: CString,
    func: *const c_void,
}

unsafe impl Send for PendingNative {}

static NATIVES: OnceLock<Mutex<HashMap<String, NativeRecord>>> = OnceLock::new();
static PENDING: Mutex<Vec<PendingNative>> = Mutex::new(Vec::new());

fn map() -> &'static Mutex<HashMap<String, NativeRecord>> {
    NATIVES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn init() {
    let _ = map();
}

pub fn observe_registered(name: &str, signature: &str, func: *const c_void) {
    map().lock().unwrap().insert(
        name.to_owned(),
        NativeRecord {
            name: name.to_owned(),
            signature: signature.to_owned(),
            func: func as usize,
        },
    );
}

pub fn request_plugin_native(
    name: CString,
    signature: CString,
    func: *const c_void,
) -> Result<(), String> {
    if func.is_null() {
        return Err("null function".to_string());
    }

    PENDING.lock().unwrap().push(PendingNative {
        name,
        signature,
        func,
    });

    Ok(())
}

pub fn flush_pending() {
    let mut queue = PENDING.lock().unwrap();
    let drained: Vec<PendingNative> = queue.drain(..).collect();
    drop(queue);

    for n in drained {
        logging::info(&format!(
            "registering plugin native {} {}",
            n.name.to_string_lossy(),
            n.signature.to_string_lossy()
        ));

        let ret = jass_raw::register_native(&n.name, &n.signature, n.func);

        logging::info(&format!(
            "registered plugin native {} {} ret={ret}",
            n.name.to_string_lossy(),
            n.signature.to_string_lossy()
        ));
    }
}

#[allow(dead_code)]
pub fn contains(name: &str) -> bool {
    map().lock().unwrap().contains_key(name)
}

#[allow(dead_code)]
pub fn get(name: &str) -> Option<NativeRecord> {
    map().lock().unwrap().get(name).cloned()
}

pub fn snapshot() -> Vec<NativeRecord> {
    map().lock().unwrap().values().cloned().collect()
}
