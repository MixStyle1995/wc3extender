use core::ffi::c_void;

#[repr(C)]
pub struct CFrame {
    pub vtable: *const c_void,
    pub unk_04: [u8; 12], // Padding to 0x10
    pub field_10: u32,
    pub parent: *mut CFrame,
    pub layer_master: *mut c_void,
    pub pos_left: f32,
    pub pos_bottom: f32,
    pub pos_right: f32,
    pub pos_top: f32,
    pub visibility_flag: u32,
    pub layer_style: u32,
    pub level: u32,
}

#[repr(C)]
#[allow(dead_code)]
pub struct CBackdropFrame {
    pub base: CFrame,
    pub mirrored: u32, // Accessed as ((CBackdropFrame*)frame)->Mirrored == 1
}

#[repr(C)]
#[allow(dead_code)]
pub struct CHighlightFrame {
    pub base: CFrame,
    pub alpha_mod: u32, // Accessed as ((CHighlightFrame*)frame)->AlphaMod
}

#[repr(C)]
#[allow(dead_code)]
pub struct CSimpleTop {
    pub base: CFrame,
    pub hero_bar: *mut CFrame,
}

#[repr(C)]
#[allow(dead_code)]
pub struct CSpriteFrame {
    pub base: CFrame,
}

#[repr(C)]
#[allow(dead_code)]
pub struct CAgent {
    pub base: CFrame,
    pub button_unit: *mut c_void,
    pub cargo_unit: *mut c_void,
    pub group_unit: *mut c_void,
    pub portrait_button: *mut c_void,
}
