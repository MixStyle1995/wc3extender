use crate::jass::raw as jass_raw;

#[unsafe(no_mangle)]
pub extern "C" fn wc3sys_make_jass_string(c_string: *const u8) -> i32 {
    if c_string.is_null() {
        return 0;
    }
    jass_raw::make_jass_string(c_string)
}
