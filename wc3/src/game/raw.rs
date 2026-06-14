use crate::addresses;

pub type RawUnit = usize;

pub type UnitHandleToCUnitFn = unsafe extern "C" fn(handle: u32) -> RawUnit;

pub fn unit_handle_to_cunit(handle: u32) -> RawUnit {
    if handle == 0 {
        return 0;
    }

    let f: UnitHandleToCUnitFn =
        unsafe { core::mem::transmute(addresses::get().buffs.unit_handle_to_cunit) };

    unsafe { f(handle) }
}
