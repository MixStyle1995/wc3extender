pub mod natives;

pub fn init() {
    if let Err(e) = natives::install_hook() {
        crate::logging::error(&format!("mouse: cursor hook install failed: {e}"));
    }
    natives::register_custom_natives();
}
