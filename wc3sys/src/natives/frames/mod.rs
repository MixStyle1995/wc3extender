use std::sync::Arc;

use crate::lifecycle::{self, ConfigStarted};

pub mod natives;
pub mod structs;
pub mod hooks;

pub mod events;
pub mod trigger_events;
pub mod types;
pub mod offsets;
pub mod frame_registry;
pub mod ops;
pub mod frame_type;

#[allow(unused_imports)]
pub use types::*;

struct FramesLifecycle;

impl FramesLifecycle {
    fn on_config_started(&self, event: &ConfigStarted) {
        let _ = event.reload;
        events::clear();
    }
}

pub fn init() -> crate::error::Result<()> {
    hooks::install()?;
    lifecycle::component(Arc::new(FramesLifecycle)).on(FramesLifecycle::on_config_started);
    Ok(())
}
