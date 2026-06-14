pub mod frames;

pub mod conversions;

pub mod buffs;

pub mod spells;

pub fn init() -> crate::error::Result<()> {
    frames::init()?;
    conversions::register_custom_natives();
    buffs::init();
    spells::init();
    Ok(())
}
