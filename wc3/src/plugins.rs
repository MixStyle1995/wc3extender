#[macro_export]
macro_rules! wc3_plugin {
    (
        name: $name:literal,
        on_load: $on_load:path $(,)?
    ) => {
        static PLUGIN_NAME: &[u8] = concat!($name, "\0").as_bytes();

        static PLUGIN: $crate::Wc3Plugin = $crate::Wc3Plugin {
            version: $crate::WC3_API_VERSION,
            name: PLUGIN_NAME.as_ptr(),
            on_plugin_loaded: Some($on_load),
            reserved: [0; 8],
        };

        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn wc3_plugin_init() -> *const $crate::Wc3Plugin {
            &PLUGIN
        }
    };
}
