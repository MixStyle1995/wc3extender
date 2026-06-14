use core::fmt;

pub use wc3::archives::MountError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Static(&'static str),
    InlineHook(wc3::InlineHookError),
    Mount(MountError),
    HookTargetAlreadyRegistered {
        target: usize,
        existing_name: &'static str,
        new_name: &'static str,
    },
    HookNotFound {
        target: usize,
    },
    HookTrampolineMissing {
        target: usize,
        name: &'static str,
    },
}

impl Error {
    #[allow(dead_code)]
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(msg) => f.write_str(msg),
            Self::Static(msg) => f.write_str(msg),
            Self::InlineHook(e) => e.fmt(f),
            Self::Mount(e) => e.fmt(f),
            Self::HookTargetAlreadyRegistered {
                target,
                existing_name,
                new_name,
            } => write!(
                f,
                "hook target 0x{target:x} already registered: existing=`{existing_name}`, new=`{new_name}`"
            ),
            Self::HookNotFound { target } => {
                write!(f, "hook at target 0x{target:x} not found")
            }
            Self::HookTrampolineMissing { target, name } => write!(
                f,
                "hook `{name}` at target 0x{target:x} has no trampoline"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl From<wc3::InlineHookError> for Error {
    fn from(value: wc3::InlineHookError) -> Self {
        Self::InlineHook(value)
    }
}

impl From<MountError> for Error {
    fn from(value: MountError) -> Self {
        Self::Mount(value)
    }
}
