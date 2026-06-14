use core::ffi::CStr;
use std::ffi::CString;
use core::fmt;

use crate::addresses;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountError {
    InteriorNul,
    EngineRejected,
}

impl fmt::Display for MountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InteriorNul => f.write_str("path contains an interior NUL byte"),
            Self::EngineRejected => f.write_str("engine open-archive call failed"),
        }
    }
}

impl std::error::Error for MountError {}

pub const DEFAULT_MPQ_OPEN_FLAGS: u32 = 2;

pub type OpenArchiveFileFn = unsafe extern "C" fn(
    path: *const u8,
    priority: i32,
    flags: u32,
    out_archive: *mut u32,
) -> i32;

pub type ReadMpqFileFn = unsafe extern "C" fn(
    path: *const u8,
    out_buffer: *mut *mut u8,
    out_size: *mut u32,
    extra_zero_bytes: usize,
    out_file_record: *mut u32,
) -> i32;

pub fn open_archive_file(path: &CStr, priority: i32, flags: u32) -> Option<u32> {
    let addrs = addresses::get();
    let mut archive = 0u32;

    let ok = unsafe {
        let f: OpenArchiveFileFn = core::mem::transmute(addrs.archives.open_archive_file);
        f(path.as_ptr() as *const u8, priority, flags, &mut archive)
    };

    if ok == 0 || archive == 0 {
        None
    } else {
        Some(archive)
    }
}

pub fn mount_mpq_file_cstr(path: &CStr, priority: i32) -> Option<u32> {
    open_archive_file(path, priority, DEFAULT_MPQ_OPEN_FLAGS)
}

pub fn mount_mpq_file(path: &str, priority: i32) -> Result<u32, MountError> {
    let cstring = CString::new(path).map_err(|_| MountError::InteriorNul)?;
    mount_mpq_file_cstr(&cstring, priority).ok_or(MountError::EngineRejected)
}

pub type LoadCachedGameFileFn = unsafe extern "C" fn(
    path: *const u8,
    out_buffer: *mut *mut u8,
    out_size: *mut u32,
    take_ownership: i32,
) -> i32;

pub type ReleaseLoadedFileBufferFn = unsafe extern "C" fn(
    buffer: *mut u8,
    force_free: i32,
) -> i32;

fn release_loaded_file_buffer(buffer: *mut u8) {
    if buffer.is_null() {
        return;
    }

    unsafe {
        let release: ReleaseLoadedFileBufferFn =
            core::mem::transmute(addresses::get().archives.release_loaded_file_buffer);
        release(buffer, 1);
    }
}

/// Lower-level whole-file reader for mounted MPQ/virtual files.
///
/// IDA chain:
/// `sub_439570 -> sub_439600 -> sub_439C20(open/resolve) + sub_43A1D0(read)`.
/// This is intentionally distinct from `read_cached_game_file`, which uses the cached game-file
/// loader at `sub_4593B0`.
pub fn read_mpq_file(path: &CStr) -> Option<Vec<u8>> {
    let addrs = addresses::get();
    let mut buffer: *mut u8 = core::ptr::null_mut();
    let mut size = 0u32;

    let ok = unsafe {
        let f: ReadMpqFileFn = core::mem::transmute(addrs.archives.read_mpq_file);
        f(
            path.as_ptr() as *const u8,
            &mut buffer,
            &mut size,
            1,
            core::ptr::null_mut(),
        )
    };

    if ok == 0 || buffer.is_null() {
        return None;
    }

    let bytes = unsafe { core::slice::from_raw_parts(buffer, size as usize).to_vec() };
    release_loaded_file_buffer(buffer);
    Some(bytes)
}

/// High-level cached game virtual-file loader.
///
/// This wraps `sub_4593B0`, not the lower MPQ reader. IDA showed this is
/// used by map/JASS/preloader/model/UI-ish paths and can fall through to the
/// general mounted-MPQ resolver. The name is about cache/ownership semantics,
/// not about being restricted to map archives.
pub fn read_cached_game_file(path: &CStr) -> Option<Vec<u8>> {
    let addrs = addresses::get();
    let mut buffer: *mut u8 = core::ptr::null_mut();
    let mut size = 0u32;

    let ok = unsafe {
        let f: LoadCachedGameFileFn = core::mem::transmute(addrs.archives.load_cached_game_file);
        f(path.as_ptr() as *const u8, &mut buffer, &mut size, 0)
    };

    if ok == 0 || buffer.is_null() {
        return None;
    }

    let bytes = unsafe { core::slice::from_raw_parts(buffer, size as usize).to_vec() };
    release_loaded_file_buffer(buffer);
    Some(bytes)
}

pub fn read_mpq_file_str(path: &str) -> Option<Vec<u8>> {
    let path = CString::new(path).ok()?;
    read_mpq_file(&path)
}

pub fn read_cached_game_file_str(path: &str) -> Option<Vec<u8>> {
    let path = CString::new(path).ok()?;
    read_cached_game_file(&path)
}


/// Queue an MPQ to be mounted by wc3sys when the game archive lifecycle is ready.
///
/// If the archive lifecycle has already reached the mount-ready phase, wc3sys may
/// mount it immediately.
pub fn queue_mpq_file(path: &str, priority: i32) -> bool {
    let Ok(path) = CString::new(path) else {
        return false;
    };

    crate::sys::wc3sys_queue_mpq_file()(path.as_ptr() as *const u8, priority) != 0
}
