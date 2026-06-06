pub mod frames;

pub mod buffs;

pub mod spells;

pub fn init() -> crate::error::Result<()> {
    frames::init()?;
    buffs::init();
    spells::init();
    Ok(())
}
