#[inline]
pub unsafe fn read_usize(addr: usize) -> usize {
    unsafe { (addr as *const usize).read_unaligned() }
}

#[inline]
pub unsafe fn write_usize(addr: usize, value: usize) {
    unsafe { (addr as *mut usize).write_unaligned(value) };
}

#[inline]
pub unsafe fn read_i32(addr: usize) -> i32 {
    unsafe { (addr as *const i32).read_unaligned() }
}

#[inline]
pub unsafe fn write_i32(addr: usize, value: i32) {
    unsafe { (addr as *mut i32).write_unaligned(value) };
}

#[inline]
pub unsafe fn read_u32(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_unaligned() }
}

#[inline]
pub unsafe fn write_u32(addr: usize, value: u32) {
    unsafe { (addr as *mut u32).write_unaligned(value) };
}

#[inline]
pub unsafe fn read_f32(addr: usize) -> f32 {
    unsafe { (addr as *const f32).read_unaligned() }
}

#[inline]
pub unsafe fn write_f32(addr: usize, value: f32) {
    unsafe { (addr as *mut f32).write_unaligned(value) };
}
