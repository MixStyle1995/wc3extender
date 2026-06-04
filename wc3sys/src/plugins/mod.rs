pub mod exports;
pub mod loader;
pub mod registry;

pub fn init() -> Result<(), String> {
    exports::init()?;
    unsafe { loader::load_all() };
    Ok(())
}
