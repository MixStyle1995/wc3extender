use std::ffi::c_void;
use std::ptr;

use windows_sys::Win32::System::Memory::{
    VirtualAlloc,
    VirtualProtect,
    MEM_COMMIT,
    MEM_RESERVE,
    PAGE_EXECUTE_READWRITE,
};

pub fn with_writable<T, F: FnOnce() -> T>(
    memory: usize,
    len: usize,
    closure: F,
) -> Result<T, &'static str> {
    unsafe {
        let mut old_prot: u32 = 0;

        let ok = VirtualProtect(
            memory as *const c_void,
            len,
            PAGE_EXECUTE_READWRITE,
            &mut old_prot,
        );

        if ok == 0 {
            return Err("Could not unprotect memory");
        }

        let closure_return: T = closure();

        let mut throwaway: u32 = 0;

        VirtualProtect(
            memory as *const c_void,
            len,
            old_prot,
            &mut throwaway,
        );

        Ok(closure_return)
    }
}

pub unsafe fn write_bytes(dest: usize, bytes: &[u8]) -> Result<(), &'static str> {
    unsafe {
        with_writable(dest, bytes.len(), || {
            ptr::copy_nonoverlapping(bytes.as_ptr(), dest as *mut u8, bytes.len());
        })
    }
}

pub unsafe fn read_bytes(src: usize, len: usize) -> Vec<u8> {
    unsafe {
        let mut buffer = vec![0u8; len];

        ptr::copy_nonoverlapping(src as *const u8, buffer.as_mut_ptr(), len);

        buffer
    }
}

pub unsafe fn alloc_executable(size: usize) -> Option<usize> {
    unsafe {
        let ptr = VirtualAlloc(
            ptr::null(),
            size,
            MEM_RESERVE | MEM_COMMIT,
            PAGE_EXECUTE_READWRITE,
        );

        if ptr.is_null() {
            None
        } else {
            Some(ptr as usize)
        }
    }
}

pub fn calc_rel32(from_addr: usize, to_addr: usize) -> i32 {
    (to_addr as i64 - (from_addr + 5) as i64) as i32
}

pub fn build_call32(from_addr: usize, to_addr: usize) -> [u8; 5] {
    let mut buf = [0u8; 5];

    buf[0] = 0xE8;

    let disp = calc_rel32(from_addr, to_addr);

    buf[1..5].copy_from_slice(&disp.to_le_bytes());

    buf
}

pub fn build_jmp(from_addr: usize, to_addr: usize) -> [u8; 5] {
    let mut buf = [0u8; 5];

    buf[0] = 0xE9;

    let disp = calc_rel32(from_addr, to_addr);

    buf[1..5].copy_from_slice(&disp.to_le_bytes());

    buf
}

pub unsafe fn patch_call_target(call_site: usize, new_target: usize) -> Result<(), &'static str> {
    unsafe {
        let new_disp = calc_rel32(call_site, new_target);

        write_bytes(call_site + 1, &new_disp.to_le_bytes())
    }
}

pub unsafe fn patch_jmp_target(jmp_site: usize, new_target: usize) -> Result<(), &'static str> {
    unsafe {
        let new_disp = calc_rel32(jmp_site, new_target);

        write_bytes(jmp_site + 1, &new_disp.to_le_bytes())
    }
}
