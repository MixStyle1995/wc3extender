pub mod exports;
pub mod hooks;
pub mod mount;
pub mod raw;
pub mod sites;

mod bootstrap;
pub(crate) mod queue;
mod read;

#[allow(unused_imports)]
pub use mount::mount_mpq_file;

pub use queue::queue_mpq_file;
pub use read::{read_cached_game_file, read_mpq_file};

pub fn init() -> crate::error::Result<()> {
    queue::init();
    bootstrap::init();

    crate::log_mpq_mounting!("archives: bootstrap MPQ loader registered");

    Ok(())
}
