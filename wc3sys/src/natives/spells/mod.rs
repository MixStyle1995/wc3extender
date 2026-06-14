use crate::logging;

pub mod natives;
pub mod cbuff;
pub mod cability;
pub mod hooks;
pub mod cbuff_generated;

pub fn init() {
    if let Err(e) = hooks::install() {
        logging::error_value("spells hooks init failed", &e);
    }
    natives::register_custom_natives();
    cbuff_generated::register_generated_buff_spell_natives();
    cbuff::register_debug_natives();
    cability::register_test_natives();
}
