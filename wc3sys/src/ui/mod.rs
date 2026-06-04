pub mod events;
pub mod exports;
pub mod hooks;

use crate::hooks as hook_manager;
use crate::logging;

pub fn init() -> crate::error::Result<()> {
    exports::init()?;
    hook_manager::install(hooks::c_observer_dispatch_event(
        events::c_observer_dispatch_event_handler,
    ))?;
    logging::info("ui: observer dispatch hook installed");
    Ok(())
}
