use wc3::InlineHook;

use crate::addresses;


pub const REGISTER_NATIVE: &str = "jass_register_native";
pub const RUN_FUNCTION: &str = "jass_run_function";
pub const REGISTER_NATIVES: &str = "jass_register_natives";
pub const INVOKE_CODE_BY_ID: &str = "jass_invoke_code_by_id";

pub use wc3::jass::abi::{
    CodeResult,
    InvokeCodeFn,
    InvokeCodeHandler,
    RegisterNativeHandler,
    RegisterNativesHandler,
    RunFunctionFn,
    RunFunctionHandler,
};


pub fn register_native(handler: RegisterNativeHandler) -> InlineHook {
    InlineHook::new(
        REGISTER_NATIVE,
        addresses::get().jass.register_native,
        handler as *const () as usize,
    )
}

pub fn run_function(handler: RunFunctionHandler) -> InlineHook {
    InlineHook::new(
        RUN_FUNCTION,
        addresses::get().jass.run_function,
        handler as *const () as usize,
    )
}

pub fn register_natives(handler: RegisterNativesHandler) -> InlineHook {
    InlineHook::new(
        REGISTER_NATIVES,
        addresses::get().jass.register_natives,
        handler as *const () as usize,
    )
}

pub fn invoke_code_by_id(handler: InvokeCodeHandler) -> InlineHook {
    InlineHook::new(
        INVOKE_CODE_BY_ID,
        addresses::get().jass.invoke_code_by_id,
        handler as *const () as usize,
    )
}
