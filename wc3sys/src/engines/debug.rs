use core::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::Engine;
use crate::logging;

pub struct DebugEngine {
    config_count: AtomicUsize,
    function_count: AtomicUsize,
    register_count: AtomicUsize,
}

impl DebugEngine {
    pub fn new() -> Self {
        Self {
            config_count: AtomicUsize::new(0),
            function_count: AtomicUsize::new(0),
            register_count: AtomicUsize::new(0),
        }
    }
}

impl Engine for DebugEngine {
    fn name(&self) -> &'static str { "debug" }

    fn config(&self) {
        let n = self.config_count.fetch_add(1, Ordering::Relaxed) + 1;
        logging::info(&format!("[debug] config #{n}"));
    }

    fn function_called(&self, name: &str) {
        let n = self.function_count.fetch_add(1, Ordering::Relaxed) + 1;
        logging::info(&format!("[debug] function_called #{n}: {name}"));
    }

    fn register_native(&self, name: &str, signature: &str, _func: *const c_void) {
        let n = self.register_count.fetch_add(1, Ordering::Relaxed) + 1;
        if n <= 3 || n % 50 == 0 {
            logging::info(&format!("[debug] register_native #{n}: {name} {signature}"));
        }
    }
}
