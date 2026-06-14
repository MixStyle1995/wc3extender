use core::ffi::CStr;

pub unsafe fn borrowed_str_from_ptr<'a>(p: *const u8) -> Option<&'a str> {
    if p.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(p as *const i8).to_str().ok() }
}
