use wc3::InlineHook;

use crate::addresses;

pub const INIT_WAR3_MPQ_ARCHIVES: &str = "archives_init_war3_mpq_archives";

pub type InitWar3MpqArchivesFn = unsafe extern "C" fn() -> i32;
pub type InitWar3MpqArchivesHandler = InitWar3MpqArchivesFn;

pub fn init_war3_mpq_archives(handler: InitWar3MpqArchivesHandler) -> InlineHook {
    InlineHook::new(
        INIT_WAR3_MPQ_ARCHIVES,
        addresses::get().archives.init_war3_mpq_archives,
        handler as *const () as usize,
    )
}
