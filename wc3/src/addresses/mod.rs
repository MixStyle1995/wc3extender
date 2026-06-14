use std::sync::OnceLock;

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

macro_rules! address_group {
    (
        pub struct $name:ident {
            $($field:ident: $static_addr:literal,)+
        }
    ) => {
        #[repr(C)]
        #[derive(Debug, Clone, Copy)]
        #[allow(dead_code)]
        pub struct $name {
            $(pub $field: usize,)+
        }

        impl $name {
            pub fn from_base(base: usize) -> Self {
                Self {
                    $($field: super::rebase(base, $static_addr),)+
                }
            }
        }
    };
}

pub mod abilities;
pub mod archives;
pub mod buffs;
pub mod frames;
pub mod jass;
pub mod objects;

pub const IDA_BASE: usize = 0x400000;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct GameAddrs {
    pub base: usize,
    pub abilities: abilities::AbilityAddrs,
    pub archives: archives::ArchiveAddrs,
    pub buffs: buffs::BuffAddrs,
    pub frames: frames::FrameAddrs,
    pub jass: jass::JassAddrs,
    pub objects: objects::ObjectAddrs,
}

impl GameAddrs {
    pub fn from_base(base: usize) -> Self {
        Self {
            base,
            abilities: abilities::AbilityAddrs::from_base(base),
            archives: archives::ArchiveAddrs::from_base(base),
            buffs: buffs::BuffAddrs::from_base(base),
            frames: frames::FrameAddrs::from_base(base),
            jass: jass::JassAddrs::from_base(base),
            objects: objects::ObjectAddrs::from_base(base),
        }
    }
}

static GAME_ADDRS: OnceLock<GameAddrs> = OnceLock::new();

#[inline]
pub fn rebase(dynamic_base: usize, static_addr: usize) -> usize {
    static_addr - IDA_BASE + dynamic_base
}

pub fn init_from_base(dynamic_base: usize) -> Result<(), &'static str> {
    if dynamic_base == 0 {
        return Err("base pointer was null");
    }

    GAME_ADDRS
        .set(GameAddrs::from_base(dynamic_base))
        .map_err(|_| "GAME_ADDRS already set")
}

pub fn init_from_process() -> Result<(), &'static str> {
    let dynamic_base = unsafe { GetModuleHandleW(core::ptr::null()) } as usize;
    if dynamic_base == 0 {
        return Err("GetModuleHandleW failed");
    }

    init_from_base(dynamic_base)
}

pub unsafe fn init_from_ptr(ptr: *const GameAddrs) -> Result<(), &'static str> {
    if ptr.is_null() {
        return Err("GameAddrs pointer was null");
    }

    let addrs = unsafe { ptr.read_unaligned() };

    GAME_ADDRS
        .set(addrs)
        .map_err(|_| "GAME_ADDRS already set")
}

pub fn get() -> &'static GameAddrs {
    GAME_ADDRS.get().expect("GameAddrs not initialized")
}

pub fn try_get() -> Option<&'static GameAddrs> {
    GAME_ADDRS.get()
}

pub fn ptr() -> *const GameAddrs {
    get() as *const GameAddrs
}

pub fn get_ptr() -> *const GameAddrs {
    ptr()
}

pub fn init_from_wc3sys() -> Result<(), &'static str> {
    let ptr = crate::sys::wc3sys_game_addrs()();
    unsafe { init_from_ptr(ptr) }
}
