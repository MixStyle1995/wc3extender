pub const WC3_API_VERSION: u32 = 1;

pub const WC3_PLUGIN_ENTRYPOINT: &[u8] = b"wc3_plugin_init\0";

pub type OnPluginLoadedFn = unsafe extern "C" fn();

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Wc3Plugin {
    pub version: u32,
    pub name: *const u8,
    pub on_plugin_loaded: Option<OnPluginLoadedFn>,
    pub reserved: [usize; 8],
}

impl Wc3Plugin {
    pub fn is_compatible(&self) -> bool {
        self.version == WC3_API_VERSION
    }
}

unsafe impl Sync for Wc3Plugin {}

pub type Wc3PluginInitFn = unsafe extern "C" fn() -> *const Wc3Plugin;
